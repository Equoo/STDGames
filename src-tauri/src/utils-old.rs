use std::path::Path;
use std::pin::Pin;
use std::future::Future;
use tokio::fs;
use tokio::io;
use tokio::fs::symlink;

pub fn copy_recursively<'a>(src: &'a Path, dst: &'a Path) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if !src.is_symlink() && !fs::try_exists(src).await? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path {:?} does not exist", src),
            ));
        }

        let metadata = fs::symlink_metadata(src).await?;

        if metadata.is_file() || metadata.is_symlink() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).await?;
            }

            if fs::try_exists(dst).await? {
                return Ok(());
            }

            if src.is_symlink() {
                let link_target = fs::read_link(src).await?;
                let link_target = link_target
                    .strip_prefix(src.parent().unwrap())
                    .unwrap_or(&link_target)
                    .to_path_buf();

                if let Err(e) = symlink(link_target, dst).await {
                    eprintln!("Warning: failed to create symlink {:?} to {:?}: {}", src, dst, e);
                }
                return Ok(());
            }

            if let Err(e) = fs::copy(src, dst).await {
                eprintln!("Warning: failed to copy {:?} to {:?}: {}", src, dst, e);
            }

            return Ok(());
        }

        if metadata.is_dir() {
            fs::create_dir_all(dst).await?;

            let mut entries = fs::read_dir(src).await?;

            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let dest_path = dst.join(entry.file_name());

                copy_recursively(&entry_path, &dest_path).await?;
            }
        }

        Ok(())
    })
}
