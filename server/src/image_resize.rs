use axum::{
    extract::{Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use image::{DynamicImage, ImageFormat, imageops::FilterType};
use serde::Deserialize;
use std::path::{Component, Path as StdPath, PathBuf};
use std::sync::LazyLock;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

const RESOURCES_DIR: &str = "resources";
const CACHE_DIR: &str = "resources/.cache";
const MAX_DIMENSION: u32 = 4096;
const DEFAULT_QUALITY: u8 = 82;

/// Resizing is CPU-bound; when the library grid requests dozens of images at
/// once we don't want to spin up dozens of concurrent full-res decodes and
/// starve the executor, so cap how many run at the same time.
static RESIZE_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| {
    let permits = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    Semaphore::new(permits)
});

#[derive(Debug, Deserialize)]
pub struct ResizeParams {
    w: Option<u32>,
    h: Option<u32>,
    q: Option<u8>,
}

/// Serves an asset from `resources/`, optionally resized/re-encoded on
/// demand. Results are cached on disk under `resources/.cache/` so a given
/// (path, width, height, quality) combination is only ever computed once,
/// which matters a lot when the frontend renders a whole library grid of
/// cover art at the same size on every load.
pub async fn resize_handler(Path(path): Path<String>, Query(params): Query<ResizeParams>) -> Response {
    let rel_path = PathBuf::from(&path);
    if rel_path
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
    {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }

    let source_path = PathBuf::from(RESOURCES_DIR).join(&rel_path);
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_ascii_lowercase();

    // Only still-image formats are resizable; anything else (mp4, etc.) is
    // served as-is so callers can point videos/thumbnails at this route too.
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") {
        return serve_original(&source_path).await;
    }

    let w = params.w.map(|w| w.clamp(1, MAX_DIMENSION));
    let h = params.h.map(|h| h.clamp(1, MAX_DIMENSION));
    let quality = params.q.unwrap_or(DEFAULT_QUALITY).clamp(1, 100);

    if w.is_none() && h.is_none() {
        return serve_original(&source_path).await;
    }

    if !tokio::fs::try_exists(&source_path).await.unwrap_or(false) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let cache_path = cache_path_for(&rel_path, w, h, quality, &ext);

    if let Ok(bytes) = tokio::fs::read(&cache_path).await {
        debug!("Serving cached resize {:?}", cache_path);
        return image_response(bytes, &ext);
    }

    let _permit = match RESIZE_SEMAPHORE.acquire().await {
        Ok(permit) => permit,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Another request may have populated the cache while we waited on the
    // semaphore permit, so check again before doing the work ourselves.
    if let Ok(bytes) = tokio::fs::read(&cache_path).await {
        return image_response(bytes, &ext);
    }

    let source_path_owned = source_path.clone();
    let ext_owned = ext.clone();
    let encoded = tokio::task::spawn_blocking(move || {
        encode_resized(&source_path_owned, w, h, quality, &ext_owned)
    })
    .await;

    let bytes = match encoded {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(err)) => {
            warn!("Failed to resize {:?}: {err}", source_path);
            return serve_original(&source_path).await;
        }
        Err(err) => {
            warn!("Resize task panicked for {:?}: {err}", source_path);
            return serve_original(&source_path).await;
        }
    };

    if let Some(parent) = cache_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Err(err) = tokio::fs::write(&cache_path, &bytes).await {
        warn!("Failed to write resize cache {:?}: {err}", cache_path);
    }

    image_response(bytes, &ext)
}

fn cache_path_for(rel_path: &StdPath, w: Option<u32>, h: Option<u32>, quality: u8, ext: &str) -> PathBuf {
    let flat_name = rel_path.to_string_lossy().replace('/', "_");
    let stem = flat_name.strip_suffix(&format!(".{ext}")).unwrap_or(&flat_name);
    let key = format!(
        "{stem}_w{}_h{}_q{quality}.{ext}",
        w.map(|v| v.to_string()).unwrap_or_default(),
        h.map(|v| v.to_string()).unwrap_or_default(),
    );
    PathBuf::from(CACHE_DIR).join(key)
}

fn encode_resized(
    source_path: &StdPath,
    w: Option<u32>,
    h: Option<u32>,
    quality: u8,
    ext: &str,
) -> Result<Vec<u8>, String> {
    let img = image::open(source_path).map_err(|e| e.to_string())?;
    let (orig_w, orig_h) = (img.width(), img.height());
    let (target_w, target_h) = fit_dimensions(orig_w, orig_h, w, h);

    // Never upscale: if the requested box is bigger than the source, hand
    // back the original pixels instead of manufacturing blurry detail.
    let resized = if target_w < orig_w || target_h < orig_h {
        img.resize(target_w, target_h, FilterType::Lanczos3)
    } else {
        img
    };

    encode(&resized, ext, quality)
}

fn fit_dimensions(orig_w: u32, orig_h: u32, w: Option<u32>, h: Option<u32>) -> (u32, u32) {
    let orig_w_f = orig_w.max(1) as f64;
    let orig_h_f = orig_h.max(1) as f64;

    let scale = match (w, h) {
        (Some(w), Some(h)) => (w as f64 / orig_w_f).min(h as f64 / orig_h_f),
        (Some(w), None) => w as f64 / orig_w_f,
        (None, Some(h)) => h as f64 / orig_h_f,
        (None, None) => 1.0,
    };

    (
        ((orig_w_f * scale).round() as u32).max(1),
        ((orig_h_f * scale).round() as u32).max(1),
    )
}

fn encode(img: &DynamicImage, ext: &str, quality: u8) -> Result<Vec<u8>, String> {
    match ext {
        "png" => {
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, ImageFormat::Png).map_err(|e| e.to_string())?;
            Ok(buf.into_inner())
        }
        "webp" => {
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, ImageFormat::WebP).map_err(|e| e.to_string())?;
            Ok(buf.into_inner())
        }
        _ => {
            let rgb = img.to_rgb8();
            let mut jpeg_buf = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, quality);
            encoder.encode_image(&rgb).map_err(|e| e.to_string())?;
            Ok(jpeg_buf)
        }
    }
}

async fn serve_original(path: &StdPath) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            image_response(bytes, ext)
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[test]
    fn fit_dimensions_preserves_aspect_ratio() {
        assert_eq!(fit_dimensions(1200, 1800, Some(300), None), (300, 450));
        assert_eq!(fit_dimensions(1200, 1800, None, Some(450)), (300, 450));
        assert_eq!(fit_dimensions(1200, 1800, Some(300), Some(300)), (200, 300));
        assert_eq!(fit_dimensions(1200, 1800, None, None), (1200, 1800));
    }

    #[tokio::test]
    async fn resize_handler_shrinks_and_caches() {
        let rel = "steam/620/library_600x900.jpg";
        let cache = cache_path_for(StdPath::new(rel), Some(300), None, DEFAULT_QUALITY, "jpg");
        let _ = tokio::fs::remove_file(&cache).await;

        let res = resize_handler(
            Path(rel.to_string()),
            Query(ResizeParams { w: Some(300), h: None, q: None }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!(decoded.width(), 300);

        // Original source is 1200x1800 at ~34KB; resized output must be
        // meaningfully smaller, since that's the whole point of this route.
        let original_len = tokio::fs::metadata(PathBuf::from(RESOURCES_DIR).join(rel))
            .await
            .unwrap()
            .len();
        assert!((bytes.len() as u64) < original_len);

        // Second call should hit the on-disk cache written by the first.
        assert!(tokio::fs::try_exists(&cache).await.unwrap());
        let res2 = resize_handler(
            Path(rel.to_string()),
            Query(ResizeParams { w: Some(300), h: None, q: None }),
        )
        .await;
        assert_eq!(res2.status(), StatusCode::OK);

        let _ = tokio::fs::remove_file(&cache).await;
    }

    #[tokio::test]
    async fn resize_handler_rejects_path_traversal() {
        let res = resize_handler(
            Path("../Cargo.toml".to_string()),
            Query(ResizeParams { w: Some(100), h: None, q: None }),
        )
        .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}

fn image_response(bytes: Vec<u8>, ext: &str) -> Response {
    let content_type = match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "image/jpeg",
    };

    (
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000, immutable".to_string(),
            ),
        ],
        bytes,
    )
        .into_response()
}
