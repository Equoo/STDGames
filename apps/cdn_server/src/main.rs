use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get_service, post},
    serve,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use tracing::{Level, error, info};

mod clients;
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

    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

    let static_files = get_service(ServeDir::new("resources"));

    let app = Router::new()
        .nest_service("/cdn/", static_files)
        .route("/api/data", post(api_data))
        .with_state(app_state)
        .layer(cors);

    info!("Server running on port 3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
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
