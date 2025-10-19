use axum::{
    extract::State,
    response::{Json},
    routing::{get_service, post},
    Router,
    serve
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{Level, error};

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
    
	let clients = Arc::new(Mutex::new(clients::ApiClients::new("resources").await.expect("Failed to initialize API clients")));
	let app_state = AppState { clients };

    let static_files = get_service(ServeDir::new("resources"));

    let app = Router::new()
        .nest_service("/cdn/", static_files)
        .route("/api/data", post(api_data))
		.with_state(app_state);

    println!("Server running at http://localhost:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await.unwrap();
}

async fn api_data(State(state): State<AppState>, Json(body): Json<HashMap<String, ApiClient>>) -> Json<HashMap<String, GameMetadata>> {
    println!("Received API data request with {} clients", body.len());
    
    let mut metadata = HashMap::new();

    for (name, client) in body {
	    let clients = state.clients.lock().await;
        if let Some(data) = clients.fetch_game_metadata(client, "french").await {
            metadata.insert(name, data);
        }
    }

    Json(metadata)
}
