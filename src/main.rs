//! Conway's Game of Life - Main Entry Point
//!
//! A web server implementing Conway's Game of Life simulation
//! with an interactive REST API.

mod api;
mod game;

use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{create_router, AppState};
use game::GameState;

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "game_of_life=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Conway's Game of Life server...");

    // Create initial game state (60x40 grid)
    let game_state = GameState::new(60, 40);
    let state: AppState = Arc::new(Mutex::new(game_state));

    // Configure CORS to allow frontend access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create router with CORS
    let app = create_router(state).layer(cors);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port 8080");

    info!("Server listening on http://127.0.0.1:8080");
    info!("API endpoints:");
    info!("  GET  /api/state      - Get current game state");
    info!("  POST /api/init       - Initialize grid (width, height)");
    info!("  POST /api/toggle/:x/:y - Toggle cell");
    info!("  POST /api/set_alive/:x/:y - Set cell alive");
    info!("  POST /api/set_dead/:x/:y - Set cell dead");
    info!("  POST /api/step       - Advance one generation");
    info!("  POST /api/randomize   - Randomize (density)");
    info!("  POST /api/clear       - Clear grid");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
