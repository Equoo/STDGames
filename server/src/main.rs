use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, get_service, post},
    serve,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tracing::{Level, error, info};

mod clients;
mod image_resize;
use crate::clients::{ApiClient, GameMetadata};

#[derive(Clone)]
struct AppState {
    clients: Arc<Mutex<clients::ApiClients>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let clients = Arc::new(Mutex::new(
        clients::ApiClients::new("resources")
            .await
            .expect("Failed to initialize API clients"),
    ));
    let app_state = AppState { clients };

    let static_files = get_service(ServeDir::new("resources")).layer(middleware::from_fn(cache_asset_response));

    let app = Router::new()
        .nest_service("/cdn/", static_files)
        .route(
            "/img/{*path}",
            get(image_resize::resize_handler).layer(middleware::from_fn(cache_asset_response)),
        )
        .route("/api/data", post(api_data).layer(CompressionLayer::new()))
        .with_state(app_state);

    info!("Server running on port 3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
}

/// Downloaded assets never change in place (a fresh fetch gets a fresh file
/// path), so successful responses can be cached by the browser/proxy
/// indefinitely. Only applied to 2xx responses so a transient 404 doesn't
/// get pinned in caches for a year.
async fn cache_asset_response(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    if res.status().is_success() && !res.headers().contains_key(header::CACHE_CONTROL) {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    res.into_response()
}

async fn api_data(
    State(state): State<AppState>,
    Json(body): Json<HashMap<String, ApiClient>>,
) -> Json<HashMap<String, GameMetadata>> {
    info!("Received API data request with {} clients", body.len());

    let mut metadata = HashMap::new();

    for (name, client) in body {
        let mut clients = state.clients.lock().await;
        if let Some(data) = clients.fetch_game_metadata(client, "french").await {
            metadata.insert(name, data);
        }
    }

    Json(metadata)
}
