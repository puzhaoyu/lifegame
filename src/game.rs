//! Conway's Game of Life - Core Game Logic
//!
//! This module implements the classic cellular automaton rules:
//! - Any live cell with fewer than two live neighbors dies (underpopulation)
//! - Any live cell with two or three live neighbors lives on
//! - Any live cell with more than three live neighbors dies (overpopulation)
//! - Any dead cell with exactly three live neighbors becomes alive (reproduction)

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Represents the state of the entire game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Grid width (number of columns)
    pub width: usize,
    /// Grid height (number of rows)
    pub height: usize,
    /// 2D grid of cells (true = alive, false = dead)
    pub cells: Vec<Vec<bool>>,
    /// Current generation number
    pub generation: u64,
    /// Population count (number of alive cells)
    pub population: usize,
}

impl GameState {
    /// Creates a new game with the specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![false; width]; height];
        let population = 0;
        Self {
            width,
            height,
            cells,
            generation: 0,
            population,
        }
    }

    /// Creates a new game with random initial cells
    pub fn with_random(width: usize, height: usize, density: f64) -> Self {
        let mut rng = rand::thread_rng();
        let cells: Vec<Vec<bool>> = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| rng.gen_bool(density))
                    .collect()
            })
            .collect();

        let population = cells.iter()
            .map(|row| row.iter().filter(|&&c| c).count())
            .sum();

        Self {
            width,
            height,
            cells,
            generation: 0,
            population,
        }
    }

    /// Toggles the state of a cell at the given coordinates
    /// Coordinates wrap around (toroidal topology)
    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        let was_alive = self.cells[wrapped_y][wrapped_x];
        self.cells[wrapped_y][wrapped_x] = !was_alive;

        // Update population count
        if was_alive {
            self.population -= 1;
        } else {
            self.population += 1;
        }
    }

    /// Sets a cell to alive (used for painting)
    pub fn set_cell_alive(&mut self, x: usize, y: usize) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        if !self.cells[wrapped_y][wrapped_x] {
            self.cells[wrapped_y][wrapped_x] = true;
            self.population += 1;
        }
    }

    /// Sets a cell to dead (used for erasing)
    pub fn set_cell_dead(&mut self, x: usize, y: usize) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        if self.cells[wrapped_y][wrapped_x] {
            self.cells[wrapped_y][wrapped_x] = false;
            self.population -= 1;
        }
    }

    /// Clears the grid (sets all cells to dead)
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
        self.population = 0;
        self.generation = 0;
    }

    /// Randomizes the grid with the given density
    pub fn randomize(&mut self, density: f64) {
        let mut rng = rand::thread_rng();

        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = rng.gen_bool(density);
            }
        }

        self.population = self.cells.iter()
            .map(|row| row.iter().filter(|&&c| c).count())
            .sum();
        self.generation = 0;
    }

    /// Counts the number of live neighbors for a cell (with toroidal wrapping)
    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;

        // Check all 8 neighbors with wrapping
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue; // Skip self
                }

                // Calculate wrapped coordinates
                let nx = ((x as isize + dx + self.width as isize) as usize) % self.width;
                let ny = ((y as isize + dy + self.height as isize) as usize) % self.height;

                if self.cells[ny][nx] {
                    count += 1;
                }
            }
        }

        count
    }

    /// Advances the game by one generation according to Conway's rules
    pub fn step(&mut self) {
        let mut new_cells = vec![vec![false; self.width]; self.height];
        let mut new_population = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let is_alive = self.cells[y][x];

                // Apply Conway's Game of Life rules
                let new_state = match (is_alive, neighbors) {
                    // Rule 1 & 3: Live cell dies if < 2 or > 3 neighbors
                    (true, n) if n < 2 || n > 3 => false,
                    // Rule 2: Live cell survives with 2 or 3 neighbors
                    (true, 2 | 3) => true,
                    // Rule 4: Dead cell becomes alive with exactly 3 neighbors
                    (false, 3) => true,
                    // All other cases: state unchanged
                    _ => is_alive,
                };

                new_cells[y][x] = new_state;
                if new_state {
                    new_population += 1;
                }
            }
        }

        self.cells = new_cells;
        self.population = new_population;
        self.generation += 1;
    }

    /// Resizes the grid to new dimensions
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let old_cells = std::mem::replace(&mut self.cells, vec![]);

        // Create new grid
        let mut new_cells = vec![vec![false; new_width]; new_height];
        let mut new_population = 0;

        // Copy existing cells (clipped to new dimensions)
        for y in 0..std::cmp::min(self.height, new_height) {
            for x in 0..std::cmp::min(self.width, new_width) {
                if old_cells[y][x] {
                    new_cells[y][x] = true;
                    new_population += 1;
                }
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.cells = new_cells;
        self.population = new_population;
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
    }

    #[test]
    fn test_toggle_cell() {
        let mut game = GameState::new(10, 10);
        assert!(!game.cells[0][0]);

        game.toggle_cell(0, 0);
        assert!(game.cells[0][0]);
        assert_eq!(game.population, 1);

        game.toggle_cell(0, 0);
        assert!(!game.cells[0][0]);
        assert_eq!(game.population, 0);
    }

    #[test]
    fn test_toroidal_wrapping() {
        let mut game = GameState::new(10, 10);
        game.toggle_cell(10, 10); // Should wrap to (0, 0)
        assert!(game.cells[0][0]);
    }

    #[test]
    fn test_cell_dies_from_underpopulation() {
        let mut game = GameState::new(3, 3);
        game.cells[1][1] = true; // Single cell
        game.population = 1;

        game.step();
        assert!(!game.cells[1][1]); // Should die
    }

    #[test]
    fn test_cell_survives_with_two_neighbors() {
        let mut game = GameState::new(3, 3);
        game.cells[1][1] = true;
        game.cells[1][0] = true;
        game.cells[1][2] = true;
        game.population = 3;

        game.step();
        assert!(game.cells[1][1]); // Should survive
    }

    #[test]
    fn test_reproduction() {
        let mut game = GameState::new(3, 3);
        // Pattern that should create a new cell at center
        game.cells[0][0] = true;
        game.cells[0][1] = true;
        game.cells[1][0] = true;
        // Center (1,1) has exactly 3 neighbors
        game.population = 3;

        game.step();
        assert!(game.cells[1][1]); // Should come alive
    }

    #[test]
    fn test_clear_resets_generation() {
        let mut game = GameState::with_random(10, 10, 0.5);
        game.step();
        game.step();
        assert!(game.generation > 0);

        game.clear();
        assert_eq!(game.generation, 0);
        assert_eq!(game.population, 0);
    }
}
