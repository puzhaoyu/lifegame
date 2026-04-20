//! Two-Faction Game of Life - HTTP API Handlers
//!
//! Provides REST API endpoints for game manipulation

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Save/Load data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<u8>>,
    #[serde(default)]
    pub bomb_radii: HashMap<String, usize>,
    #[serde(default = "default_true")]
    pub has_border_walls: bool,
    #[serde(default)]
    pub generation: u64,
}

fn default_true() -> bool {
    true
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
    pub last_explosions: Vec<(usize, usize, usize)>,
    pub last_hv_destroyed: Vec<(usize, usize)>,
    pub has_border_walls: bool,
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
            last_explosions: state.last_explosions.clone(),
            last_hv_destroyed: state.last_hv_destroyed.clone(),
            has_border_walls: state.has_border_walls,
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

/// POST /api/set_wall/:x/:y - Set cell to wall (inert, blocks life)
pub async fn set_cell_wall(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.set_cell_wall(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/set_high_value/:x/:y - Set cell to high-value unit
pub async fn set_cell_high_value(
    State(state): State<AppState>,
    Path((x, y)): Path<(usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.set_cell_high_value(x, y);
    Json(GameStateResponse::from(&*game))
}

/// POST /api/bomb/:x/:y/:radius - Place a bomb on the grid (explodes when a cell touches it)
pub async fn place_bomb(
    State(state): State<AppState>,
    Path((x, y, radius)): Path<(usize, usize, usize)>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.place_bomb(x, y, radius);
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

/// GET /api/save - Export current game state as save data
pub async fn save_state(State(state): State<AppState>) -> Json<SaveData> {
    let game = state.lock().await;
    Json(SaveData {
        width: game.width,
        height: game.height,
        cells: game.cells.clone(),
        bomb_radii: game.bomb_radii.clone(),
        has_border_walls: game.has_border_walls,
        generation: game.generation,
    })
}

/// POST /api/load - Import game state from save data
pub async fn load_state(
    State(state): State<AppState>,
    Json(data): Json<SaveData>,
) -> Json<GameStateResponse> {
    let mut game = state.lock().await;

    game.width = data.width;
    game.height = data.height;
    game.cells = data.cells;
    game.bomb_radii = data.bomb_radii;
    game.has_border_walls = data.has_border_walls;
    game.generation = data.generation;
    game.last_explosions.clear();
    game.last_hv_destroyed.clear();

    // Recalculate population
    let mut population = 0;
    let mut red_count = 0;
    let mut blue_count = 0;
    for row in &game.cells {
        for &cell in row {
            if cell == 1 { population += 1; red_count += 1; }
            if cell == 2 { population += 1; blue_count += 1; }
        }
    }
    game.population = population;
    game.red_count = red_count;
    game.blue_count = blue_count;

    Json(GameStateResponse::from(&*game))
}

/// POST /api/toggle_border - Toggle border walls (air walls vs toroidal wrapping)
pub async fn toggle_border(State(state): State<AppState>) -> Json<GameStateResponse> {
    let mut game = state.lock().await;
    game.toggle_border_walls();
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
        .route("/api/set_wall/:x/:y", post(set_cell_wall))
        .route("/api/set_high_value/:x/:y", post(set_cell_high_value))
        .route("/api/bomb/:x/:y/:radius", post(place_bomb))
        .route("/api/step", post(step))
        .route("/api/randomize", post(randomize))
        .route("/api/clear", post(clear))
        .route("/api/save", get(save_state))
        .route("/api/load", post(load_state))
        .route("/api/toggle_border", post(toggle_border))
        .with_state(state)
}
