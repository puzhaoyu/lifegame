//! Two-Faction Game of Life - Core Game Logic
//!
//! A variant of Conway's Game of Life with two competing factions:
//! - Red cells (team 1) and Blue cells (team 2)
//! - Dead cells resurrect as the majority faction among their neighbors
//!
//! Rules:
//! 1. Underpopulation: Live cells with < 2 neighbors die
//! 2. Survival: Live cells with 2-3 neighbors survive
//! 3. Overpopulation: Live cells with > 3 neighbors die
//! 4. Reproduction: Dead cells with exactly 3 neighbors resurrect as the majority faction

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Cell state: 0 = dead, 1 = red (team 1), 2 = blue (team 2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Dead = 0,
    Red = 1,   // Team 1 (我方)
    Blue = 2,  // Team 2 (敌方)
}

impl Cell {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Cell::Red,
            2 => Cell::Blue,
            _ => Cell::Dead,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn is_alive(&self) -> bool {
        *self != Cell::Dead
    }
}

/// Represents the state of the entire game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Grid width (number of columns)
    pub width: usize,
    /// Grid height (number of rows)
    pub height: usize,
    /// 2D grid of cells (0 = dead, 1 = red, 2 = blue)
    pub cells: Vec<Vec<u8>>,
    /// Current generation number
    pub generation: u64,
    /// Population count (number of alive cells)
    pub population: usize,
    /// Red team population
    pub red_count: usize,
    /// Blue team population
    pub blue_count: usize,
}

/// Neighbor counts for a cell (with toroidal wrapping)
struct NeighborCounts {
    total: usize,
    red: usize,
    blue: usize,
}

impl GameState {
    /// Creates a new game with the specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![0u8; width]; height];
        Self {
            width,
            height,
            cells,
            generation: 0,
            population: 0,
            red_count: 0,
            blue_count: 0,
        }
    }

    /// Creates a new game with random initial cells (50-50 red/blue)
    pub fn with_random(width: usize, height: usize, density: f64) -> Self {
        let mut rng = rand::thread_rng();
        let cells: Vec<Vec<u8>> = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| {
                        if rng.gen_bool(density) {
                            if rng.gen_bool(0.5) { 1 } else { 2 } // 50-50 red/blue
                        } else {
                            0
                        }
                    })
                    .collect()
            })
            .collect();

        let (population, red_count, blue_count) = Self::count_population(&cells);

        Self {
            width,
            height,
            cells,
            generation: 0,
            population,
            red_count,
            blue_count,
        }
    }

    /// Count population from cells
    fn count_population(cells: &[Vec<u8>]) -> (usize, usize, usize) {
        let mut population = 0;
        let mut red_count = 0;
        let mut blue_count = 0;

        for row in cells {
            for &cell in row {
                if cell == 1 {
                    population += 1;
                    red_count += 1;
                } else if cell == 2 {
                    population += 1;
                    blue_count += 1;
                }
            }
        }

        (population, red_count, blue_count)
    }

    /// Toggles the state of a cell (cycles: dead -> red -> blue -> dead)
    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        let old_state = self.cells[wrapped_y][wrapped_x];
        let new_state = match old_state {
            0 => 1, // dead -> red
            1 => 2, // red -> blue
            _ => 0, // blue -> dead
        };

        self.cells[wrapped_y][wrapped_x] = new_state;

        // Update population counts
        if old_state == 0 && new_state != 0 {
            self.population += 1;
        } else if old_state != 0 && new_state == 0 {
            self.population -= 1;
        }

        if old_state == 1 { self.red_count -= 1; }
        if old_state == 2 { self.blue_count -= 1; }
        if new_state == 1 { self.red_count += 1; }
        if new_state == 2 { self.blue_count += 1; }
    }

    /// Sets a cell to a specific team (0=neutral/red, 1=red, 2=blue)
    pub fn set_cell(&mut self, x: usize, y: usize, team: u8) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        let old_state = self.cells[wrapped_y][wrapped_x];
        let new_state = match team {
            1 | 2 => team,
            _ => 0,
        };

        if old_state == new_state {
            return;
        }

        self.cells[wrapped_y][wrapped_x] = new_state;

        // Update population counts
        if old_state == 0 && new_state != 0 {
            self.population += 1;
        } else if old_state != 0 && new_state == 0 {
            self.population -= 1;
        }

        if old_state == 1 { self.red_count -= 1; }
        if old_state == 2 { self.blue_count -= 1; }
        if new_state == 1 { self.red_count += 1; }
        if new_state == 2 { self.blue_count += 1; }
    }

    /// Sets a cell to red (team 1)
    pub fn set_cell_red(&mut self, x: usize, y: usize) {
        self.set_cell(x, y, 1);
    }

    /// Sets a cell to blue (team 2)
    pub fn set_cell_blue(&mut self, x: usize, y: usize) {
        self.set_cell(x, y, 2);
    }

    /// Sets a cell to dead
    pub fn set_cell_dead(&mut self, x: usize, y: usize) {
        self.set_cell(x, y, 0);
    }

    /// Clears the grid (sets all cells to dead)
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = 0;
            }
        }
        self.population = 0;
        self.red_count = 0;
        self.blue_count = 0;
        self.generation = 0;
    }

    /// Randomizes the grid with the given density (50-50 red/blue)
    pub fn randomize(&mut self, density: f64) {
        let mut rng = rand::thread_rng();

        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if rng.gen_bool(density) {
                    *cell = if rng.gen_bool(0.5) { 1 } else { 2 };
                } else {
                    *cell = 0;
                }
            }
        }

        let (population, red_count, blue_count) = Self::count_population(&self.cells);
        self.population = population;
        self.red_count = red_count;
        self.blue_count = blue_count;
        self.generation = 0;
    }

    fn count_neighbors(&self, x: usize, y: usize) -> NeighborCounts {
        let mut total = 0;
        let mut red = 0;
        let mut blue = 0;

        // Check all 8 neighbors with wrapping
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue; // Skip self
                }

                // Calculate wrapped coordinates
                let nx = ((x as isize + dx + self.width as isize) as usize) % self.width;
                let ny = ((y as isize + dy + self.height as isize) as usize) % self.height;

                match self.cells[ny][nx] {
                    1 => { red += 1; total += 1; }
                    2 => { blue += 1; total += 1; }
                    _ => {}
                }
            }
        }

        NeighborCounts { total, red, blue }
    }

    /// Advances the game by one generation according to the two-faction rules
    pub fn step(&mut self) {
        let mut new_cells = vec![vec![0u8; self.width]; self.height];
        let mut population = 0;
        let mut red_count = 0;
        let mut blue_count = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let current = self.cells[y][x];

                let new_state = match (current, neighbors.total) {
                    // Live cell: apply survival rules
                    (1, n) | (2, n) => {
                        // Rules 1, 2, 3: die if < 2 or > 3 neighbors, survive if 2-3
                        if n < 2 || n > 3 {
                            0 // Die
                        } else {
                            current // Survive
                        }
                    }
                    // Dead cell with exactly 3 neighbors: majority principle
                    (0, 3) => {
                        if neighbors.red > neighbors.blue {
                            1 // Resurrect as red (我方)
                        } else if neighbors.blue > neighbors.red {
                            2 // Resurrect as blue (敌方)
                        } else {
                            // Tie: randomly pick (or stay dead)
                            if rand::thread_rng().gen_bool(0.5) { 1 } else { 2 }
                        }
                    }
                    // All other cases: stay dead
                    _ => 0,
                };

                new_cells[y][x] = new_state;

                if new_state == 1 {
                    population += 1;
                    red_count += 1;
                } else if new_state == 2 {
                    population += 1;
                    blue_count += 1;
                }
            }
        }

        self.cells = new_cells;
        self.population = population;
        self.red_count = red_count;
        self.blue_count = blue_count;
        self.generation += 1;
    }

    /// Resizes the grid to new dimensions
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let old_cells = std::mem::replace(&mut self.cells, vec![]);

        // Create new grid
        let mut new_cells = vec![vec![0u8; new_width]; new_height];
        let mut population = 0;
        let mut red_count = 0;
        let mut blue_count = 0;

        // Copy existing cells (clipped to new dimensions)
        for y in 0..std::cmp::min(self.height, new_height) {
            for x in 0..std::cmp::min(self.width, new_width) {
                let cell = old_cells[y][x];
                new_cells[y][x] = cell;

                if cell == 1 {
                    population += 1;
                    red_count += 1;
                } else if cell == 2 {
                    population += 1;
                    blue_count += 1;
                }
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.cells = new_cells;
        self.population = population;
        self.red_count = red_count;
        self.blue_count = blue_count;
        self.generation = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_is_empty() {
        let game = GameState::new(10, 10);
        assert_eq!(game.population, 0);
        assert_eq!(game.generation, 0);
        assert_eq!(game.cells[0][0], 0);
    }

    #[test]
    fn test_toggle_cell() {
        let mut game = GameState::new(10, 10);
        assert_eq!(game.cells[0][0], 0);

        game.toggle_cell(0, 0);
        assert_eq!(game.cells[0][0], 1); // dead -> red
        assert_eq!(game.population, 1);
        assert_eq!(game.red_count, 1);

        game.toggle_cell(0, 0);
        assert_eq!(game.cells[0][0], 2); // red -> blue
        assert_eq!(game.population, 1);
        assert_eq!(game.red_count, 0);
        assert_eq!(game.blue_count, 1);

        game.toggle_cell(0, 0);
        assert_eq!(game.cells[0][0], 0); // blue -> dead
        assert_eq!(game.population, 0);
    }

    #[test]
    fn test_cell_dies_from_underpopulation() {
        let mut game = GameState::new(3, 3);
        game.cells[1][1] = 1; // Single red cell
        game.population = 1;
        game.red_count = 1;

        game.step();
        assert_eq!(game.cells[1][1], 0); // Should die
    }

    #[test]
    fn test_survival_with_neighbors() {
        let mut game = GameState::new(3, 3);
        // Blinker pattern (horizontal)
        game.cells[1][0] = 1;
        game.cells[1][1] = 1;
        game.cells[1][2] = 1;
        game.population = 3;
        game.red_count = 3;

        game.step();
        // Should become vertical
        assert_eq!(game.cells[0][1], 1);
        assert_eq!(game.cells[1][1], 1);
        assert_eq!(game.cells[2][1], 1);
    }

    #[test]
    fn test_majority_principle_reproduction() {
        let mut game = GameState::new(3, 3);
        // Pattern: center is dead, surrounded by 2 red + 1 blue
        game.cells[0][0] = 1; // red
        game.cells[0][1] = 1; // red
        game.cells[0][2] = 2; // blue
        game.population = 3;
        game.red_count = 2;
        game.blue_count = 1;

        game.step();
        // Center should become red (majority)
        assert_eq!(game.cells[1][1], 1);
    }
}
