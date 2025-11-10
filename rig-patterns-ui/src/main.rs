//! Web UI server for rig-patterns interactive demonstration

mod routes;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rig_patterns_ui=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing rig-patterns-ui server");

    // Initialize app state
    let state = Arc::new(AppState::new());

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/execute", post(routes::execute_pattern))
        .route("/api/compare", post(routes::compare_patterns))
        .route("/api/presets", get(routes::get_presets))
        .route("/ws", get(routes::ws_handler))
        // Serve static files
        .nest_service("/", ServeDir::new("static"))
        // Add state
        .with_state(state)
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3009".to_string())
        .parse::<u16>()
        .unwrap_or(3009);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Open http://localhost:{} in your browser", port);
    tracing::info!("To change port: Set PORT environment variable (e.g., PORT=8080 cargo run)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
