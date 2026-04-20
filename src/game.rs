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
use std::collections::HashMap;

/// Cell state: 0 = dead, 1 = red (team 1), 2 = blue (team 2), 3 = wall, 4 = bomb, 5 = high-value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Dead = 0,
    Red = 1,       // Team 1 (我方)
    Blue = 2,      // Team 2 (敌方)
    Wall = 3,      // 墙壁 (不参与任何判定)
    Bomb = 4,      // 炸弹 (被细胞触碰时爆炸)
    HighValue = 5, // 高价值单位 (红色触碰消除，蓝色视为墙)
}

impl Cell {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Cell::Red,
            2 => Cell::Blue,
            3 => Cell::Wall,
            4 => Cell::Bomb,
            5 => Cell::HighValue,
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
    /// 2D grid of cells (0 = dead, 1 = red, 2 = blue, 3 = wall, 4 = bomb, 5 = high-value)
    pub cells: Vec<Vec<u8>>,
    /// Current generation number
    pub generation: u64,
    /// Population count (number of alive cells)
    pub population: usize,
    /// Red team population
    pub red_count: usize,
    /// Blue team population
    pub blue_count: usize,
    /// Bomb radii: key = "x,y", value = radius
    #[serde(default)]
    pub bomb_radii: HashMap<String, usize>,
    /// Explosions that happened in the last step: [(x, y, radius)]
    #[serde(default)]
    pub last_explosions: Vec<(usize, usize, usize)>,
    /// High-value units destroyed in the last step: [(x, y)]
    #[serde(default)]
    pub last_hv_destroyed: Vec<(usize, usize)>,
    /// Whether the border has air walls (no wrapping)
    #[serde(default = "default_true")]
    pub has_border_walls: bool,
}

fn default_true() -> bool {
    true
}

/// Neighbor counts for a cell
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
            bomb_radii: HashMap::new(),
            last_explosions: Vec::new(),
            last_hv_destroyed: Vec::new(),
            has_border_walls: true,
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
            bomb_radii: HashMap::new(),
            last_explosions: Vec::new(),
            last_hv_destroyed: Vec::new(),
            has_border_walls: true,
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

    /// Get neighbor coordinate, respecting border walls (returns None if out of bounds)
    fn get_neighbor_coord(&self, x: usize, y: usize, dx: isize, dy: isize) -> Option<(usize, usize)> {
        if self.has_border_walls {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || nx >= self.width as isize || ny < 0 || ny >= self.height as isize {
                return None;
            }
            Some((nx as usize, ny as usize))
        } else {
            let nx = ((x as isize + dx + self.width as isize) as usize) % self.width;
            let ny = ((y as isize + dy + self.height as isize) as usize) % self.height;
            Some((nx, ny))
        }
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

    /// Sets a cell to a specific team (0=dead, 1=red, 2=blue, 3=wall, 5=high-value)
    pub fn set_cell(&mut self, x: usize, y: usize, team: u8) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        let old_state = self.cells[wrapped_y][wrapped_x];
        let new_state = match team {
            1 | 2 | 3 | 5 => team,
            _ => 0,
        };

        if old_state == new_state {
            return;
        }

        // 如果覆盖了炸弹，移除炸弹半径记录
        if old_state == 4 {
            self.bomb_radii.remove(&format!("{},{}", wrapped_x, wrapped_y));
        }

        self.cells[wrapped_y][wrapped_x] = new_state;

        // Update population counts (walls, bombs, high-value don't count as population)
        let old_alive = old_state == 1 || old_state == 2;
        let new_alive = new_state == 1 || new_state == 2;
        if !old_alive && new_alive {
            self.population += 1;
        } else if old_alive && !new_alive {
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

    /// Sets a cell to wall
    pub fn set_cell_wall(&mut self, x: usize, y: usize) {
        self.set_cell(x, y, 3);
    }

    /// Sets a cell to high-value unit
    pub fn set_cell_high_value(&mut self, x: usize, y: usize) {
        self.set_cell(x, y, 5);
    }

    /// Toggle border walls (air walls vs toroidal wrapping)
    pub fn toggle_border_walls(&mut self) {
        self.has_border_walls = !self.has_border_walls;
    }

    /// Places a bomb on the grid with a specified explosion radius
    pub fn place_bomb(&mut self, x: usize, y: usize, radius: usize) {
        let wrapped_x = x % self.width;
        let wrapped_y = y % self.height;

        let old_state = self.cells[wrapped_y][wrapped_x];
        if old_state == 1 { self.population -= 1; self.red_count -= 1; }
        if old_state == 2 { self.population -= 1; self.blue_count -= 1; }
        if old_state == 4 {
            self.bomb_radii.remove(&format!("{},{}", wrapped_x, wrapped_y));
        }

        self.cells[wrapped_y][wrapped_x] = 4;
        self.bomb_radii.insert(format!("{},{}", wrapped_x, wrapped_y), radius);
    }

    /// Internal: explode a bomb at (cx, cy) with given radius
    fn explode_bomb(&mut self, cx: usize, cy: usize, radius: usize) {
        let r2 = (radius * radius) as f64;
        for dy in 0..=radius {
            for dx in 0..=radius {
                if (dx * dx + dy * dy) as f64 <= r2 {
                    for &sx in &[-1isize, 1] {
                        for &sy in &[-1isize, 1] {
                            let raw_x = cx as isize + sx * dx as isize;
                            let raw_y = cy as isize + sy * dy as isize;
                            let (nx, ny) = if self.has_border_walls {
                                if raw_x < 0 || raw_x >= self.width as isize || raw_y < 0 || raw_y >= self.height as isize {
                                    continue;
                                }
                                (raw_x as usize, raw_y as usize)
                            } else {
                                (
                                    (raw_x + self.width as isize) as usize % self.width,
                                    (raw_y + self.height as isize) as usize % self.height,
                                )
                            };
                            if self.cells[ny][nx] == 4 {
                                self.bomb_radii.remove(&format!("{},{}", nx, ny));
                            }
                            self.cells[ny][nx] = 0;
                        }
                    }
                }
            }
        }
    }

    /// Check if a bomb has any living neighbor
    fn bomb_has_living_neighbor(&self, x: usize, y: usize) -> bool {
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 { continue; }
                if let Some((nx, ny)) = self.get_neighbor_coord(x, y, dx, dy) {
                    let cell = self.cells[ny][nx];
                    if cell == 1 || cell == 2 { return true; }
                }
            }
        }
        false
    }

    /// Check if a high-value unit has any red neighbor
    fn hv_has_red_neighbor(&self, x: usize, y: usize) -> bool {
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 { continue; }
                if let Some((nx, ny)) = self.get_neighbor_coord(x, y, dx, dy) {
                    if self.cells[ny][nx] == 1 { return true; }
                }
            }
        }
        false
    }

    /// Check if a high-value unit has any red neighbor in given grid
    fn hv_has_red_neighbor_in(cells: &[Vec<u8>], x: usize, y: usize, width: usize, height: usize, has_border_walls: bool) -> bool {
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 { continue; }
                let coord = if has_border_walls {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || nx >= width as isize || ny < 0 || ny >= height as isize {
                        continue;
                    }
                    (nx as usize, ny as usize)
                } else {
                    (
                        ((x as isize + dx + width as isize) as usize) % width,
                        ((y as isize + dy + height as isize) as usize) % height,
                    )
                };
                if cells[coord.1][coord.0] == 1 { return true; }
            }
        }
        false
    }

    /// Clears the grid (sets all cells to dead, including walls, bombs, high-value)
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
        self.bomb_radii.clear();
        self.last_explosions.clear();
        self.last_hv_destroyed.clear();
    }

    /// Randomizes the grid with the given density (50-50 red/blue), preserving walls, bombs, and high-value
    pub fn randomize(&mut self, density: f64) {
        let mut rng = rand::thread_rng();

        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if *cell == 3 || *cell == 4 || *cell == 5 { continue; } // 保留墙壁、炸弹和高价值单位
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

        // Check all 8 neighbors
        for dy in [-1isize, 0, 1] {
            for dx in [-1isize, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue; // Skip self
                }

                if let Some((nx, ny)) = self.get_neighbor_coord(x, y, dx, dy) {
                    match self.cells[ny][nx] {
                        1 => { red += 1; total += 1; }
                        2 => { blue += 1; total += 1; }
                        _ => {}
                    }
                }
                // If None (out of bounds with border walls), treat as dead
            }
        }

        NeighborCounts { total, red, blue }
    }

    /// Advances the game by one generation according to the two-faction rules
    pub fn step(&mut self) {
        // 清除上一轮的记录
        self.last_explosions.clear();
        self.last_hv_destroyed.clear();

        // === 第一阶段：预演算检测 ===

        // 1a. 检测哪些炸弹被细胞触碰
        let mut triggered_bombs: Vec<(usize, usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x] == 4 {
                    if self.bomb_has_living_neighbor(x, y) {
                        let key = format!("{},{}", x, y);
                        let radius = self.bomb_radii.get(&key).copied().unwrap_or(5);
                        triggered_bombs.push((x, y, radius));
                    }
                }
            }
        }

        // 1b. 检测哪些高价值单位被红色细胞触碰
        let mut triggered_hv: Vec<(usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y][x] == 5 {
                    if self.hv_has_red_neighbor(x, y) {
                        triggered_hv.push((x, y));
                    }
                }
            }
        }

        // === 第二阶段：执行预演算触发 ===

        // 2a. 引爆炸弹
        for &(bx, by, radius) in &triggered_bombs {
            self.explode_bomb(bx, by, radius);
            self.last_explosions.push((bx, by, radius));
        }

        // 2b. 消除被触碰的高价值单位
        for &(hx, hy) in &triggered_hv {
            if self.cells[hy][hx] == 5 { // 可能已被炸弹炸毁
                self.cells[hy][hx] = 0;
                self.last_hv_destroyed.push((hx, hy));
            }
        }

        // === 第三阶段：正常的生命游戏演算 ===
        let mut new_cells = vec![vec![0u8; self.width]; self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let current = self.cells[y][x];

                // 墙壁永远保持不变
                if current == 3 {
                    new_cells[y][x] = 3;
                    continue;
                }

                // 炸弹保持不变（未被触发的炸弹继续存在）
                if current == 4 {
                    new_cells[y][x] = 4;
                    continue;
                }

                // 高价值单位保持不变（未被触碰的继续存在）
                if current == 5 {
                    new_cells[y][x] = 5;
                    continue;
                }

                let neighbors = self.count_neighbors(x, y);

                let new_state = match (current, neighbors.total) {
                    // Live cell: apply survival rules
                    (1, n) | (2, n) => {
                        if n < 2 || n > 3 {
                            0 // Die
                        } else {
                            current // Survive
                        }
                    }
                    // Dead cell with exactly 3 neighbors: majority principle
                    (0, 3) => {
                        if neighbors.red > neighbors.blue {
                            1
                        } else if neighbors.blue > neighbors.red {
                            2
                        } else {
                            if rand::thread_rng().gen_bool(0.5) { 1 } else { 2 }
                        }
                    }
                    _ => 0,
                };

                new_cells[y][x] = new_state;
            }
        }

        // === 第四阶段：演算后再次检测 ===

        // 4a. 检测新生成的细胞是否触碰了炸弹
        let mut post_triggered_bombs: Vec<(usize, usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if new_cells[y][x] == 4 {
                    let mut has_neighbor = false;
                    for dy in [-1isize, 0, 1] {
                        for dx in [-1isize, 0, 1] {
                            if dx == 0 && dy == 0 { continue; }
                            if let Some((nx, ny)) = self.get_neighbor_coord(x, y, dx, dy) {
                                if new_cells[ny][nx] == 1 || new_cells[ny][nx] == 2 {
                                    has_neighbor = true;
                                    break;
                                }
                            }
                        }
                        if has_neighbor { break; }
                    }
                    if has_neighbor {
                        let key = format!("{},{}", x, y);
                        let radius = self.bomb_radii.get(&key).copied().unwrap_or(5);
                        post_triggered_bombs.push((x, y, radius));
                    }
                }
            }
        }

        // 4b. 检测新生成的红色细胞是否触碰了高价值单位
        let mut post_triggered_hv: Vec<(usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if new_cells[y][x] == 5 {
                    if Self::hv_has_red_neighbor_in(&new_cells, x, y, self.width, self.height, self.has_border_walls) {
                        post_triggered_hv.push((x, y));
                    }
                }
            }
        }

        self.cells = new_cells;

        // 引爆演算后触发的炸弹
        for &(bx, by, radius) in &post_triggered_bombs {
            self.explode_bomb(bx, by, radius);
            self.last_explosions.push((bx, by, radius));
        }

        // 消除演算后被触碰的高价值单位
        for &(hx, hy) in &post_triggered_hv {
            if self.cells[hy][hx] == 5 {
                self.cells[hy][hx] = 0;
                self.last_hv_destroyed.push((hx, hy));
            }
        }

        // 重新计算人口
        let (pop, rc, bc) = Self::count_population(&self.cells);
        self.population = pop;
        self.red_count = rc;
        self.blue_count = bc;
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
        assert!(game.has_border_walls);
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
        let mut game = GameState::new(5, 5);
        game.cells[2][2] = 1; // Single red cell in center
        game.population = 1;
        game.red_count = 1;

        game.step();
        assert_eq!(game.cells[2][2], 0); // Should die
    }

    #[test]
    fn test_survival_with_neighbors() {
        let mut game = GameState::new(5, 5);
        // Blinker pattern (horizontal) in center
        game.cells[2][1] = 1;
        game.cells[2][2] = 1;
        game.cells[2][3] = 1;
        game.population = 3;
        game.red_count = 3;

        game.step();
        // Should become vertical
        assert_eq!(game.cells[1][2], 1);
        assert_eq!(game.cells[2][2], 1);
        assert_eq!(game.cells[3][2], 1);
    }

    #[test]
    fn test_high_value_destroyed_by_red() {
        let mut game = GameState::new(10, 10);
        game.cells[5][5] = 5; // High-value unit
        game.cells[5][4] = 1; // Red cell next to it
        game.cells[4][4] = 1; // More red cells to keep alive
        game.cells[6][4] = 1;
        let (pop, rc, bc) = GameState::count_population(&game.cells);
        game.population = pop;
        game.red_count = rc;
        game.blue_count = bc;

        game.step();
        // High-value should be destroyed (red touched it)
        assert_ne!(game.cells[5][5], 5);
    }

    #[test]
    fn test_high_value_survives_with_blue() {
        let mut game = GameState::new(10, 10);
        game.cells[5][5] = 5; // High-value unit
        game.cells[5][4] = 2; // Blue cell next to it
        game.cells[4][4] = 2;
        game.cells[6][4] = 2;
        let (pop, rc, bc) = GameState::count_population(&game.cells);
        game.population = pop;
        game.red_count = rc;
        game.blue_count = bc;

        game.step();
        // High-value should survive (only blue neighbors, blue treats as wall)
        assert_eq!(game.cells[5][5], 5);
    }

    #[test]
    fn test_border_walls_no_wrapping() {
        let mut game = GameState::new(5, 5);
        game.has_border_walls = true;
        // Place cells at corner - should not wrap
        game.cells[0][0] = 1;
        game.cells[0][1] = 1;
        game.cells[1][0] = 1;
        game.population = 3;
        game.red_count = 3;

        game.step();
        // With border walls, corner cells don't wrap
        assert_eq!(game.cells[0][0], 1); // Should survive (2 neighbors)
    }
}
