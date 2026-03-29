//! Two-Faction Game of Life - HTTP API Handlers
//!
//! Provides REST API endpoints for game manipulation

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::game::GameState;

/// Application state shared across handlers
pub type AppState = Arc<Mutex<GameState>>;

/// Request body for initialization
#[derive(Debug, Deserialize)]
pub struct InitRequest {
    pub width: usize,
    pub height: usize,
}

/// Request body for randomize
#[derive(Debug, Deserialize)]
pub struct RandomizeRequest {
    pub density: f64,
}

/// Response body for game state
#[derive(Debug, Serialize)]
pub struct GameStateResponse {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<u8>>,
    pub generation: u64,
    pub population: usize,
    pub red_count: usize,
    pub blue_count: usize,
}

impl From<&GameState> for GameStateResponse {
    fn from(state: &GameState) -> Self {
        Self {
            width: state.width,
            height: state.height,
            cells: state.cells.clone(),
            generation: state.generation,
            population: state.population,
            red_count: state.red_count,
            blue_count: state.blue_count,
        }
    }
}

/// GET /api/state - Returns current game state
pub async fn get_state(State(state): State<AppState>) -> Json<GameStateResponse> {
    let game = state.lock().await;
    Json(GameStateResponse::from(&*game))
}

/// POST /api/init - Initialize grid with new dimensions
pub async fn init_grid(
    State(state): State<AppState>,
    Json(req): Json<InitRequest>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.resize(req.width, req.height);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/toggle/:x/:y - Toggle cell state (cycles: dead -> red -> blue -> dead)
pub async fn toggle_cell(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.toggle_cell(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/set_red/:x/:y - Set cell to red (team 1)
pub async fn set_cell_red(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.set_cell_red(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/set_blue/:x/:y - Set cell to blue (team 2)
pub async fn set_cell_blue(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.set_cell_blue(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/set_dead/:x/:y - Set cell to dead
pub async fn set_cell_dead(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.set_cell_dead(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/step - Advance one generation
pub async fn step(State(state): State<AppState>) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.step();
    Json(GameStateResponse::from(&*game))
}

/// POST /api/randomize - Randomize the grid
pub async fn randomize(
    State(state): State<AppState>,
    Json(req): Json<RandomizeRequest>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.randomize(req.density);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/clear - Clear the grid
pub async fn clear(State(state): State<AppState>) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.clear();
    Json(GameStateResponse::from(&*game))
}

/// Creates the API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/state", get(get_state))
        .route("/api/init", post(init_grid))
        .route("/api/toggle/:x/:y", post(toggle_cell))
        .route("/api/set_red/:x/:y", post(set_cell_red))
        .route("/api/set_blue/:x/:y", post(set_cell_blue))
        .route("/api/set_dead/:x/:y", post(set_cell_dead))
        .route("/api/step", post(step))
        .route("/api/randomize", post(randomize))
        .route("/api/clear", post(clear))
        .with_state(state)
}
