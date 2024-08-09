use crate::game::Coord;
use crate::grid::Grid;
use crate::simulation::Snake;
use std::fmt::{Debug, Formatter};
use yansi::Color;
use yansi::Paint;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum CellType {
    /// Different state a cell can be in
    #[default]
    Free,
    Food,
    Snake(u8),
    Tail(u8),
}

#[derive(PartialEq, Clone, Copy)]
pub struct CellGame {
    // Represents the state of a cell in the game
    pub cell: CellType,
    pub hazard: u8,
}

impl Default for CellGame {
    fn default() -> Self {
        Self {
            cell: CellType::Free,
            hazard: 0,
        }
    }
}

impl CellGame {
    pub fn new(cell: CellType) -> Self {
        Self { cell, hazard: 0 }
    }
}

impl Debug for CellGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bg = if self.hazard > 0 {
            Color::Cyan
        } else {
            Color::Default
        };

        match self.cell {
            CellType::Free => write!(f, "{}", Paint::new("O").bg(bg)),
            CellType::Food => write!(f, "{}", Paint::green("x").bg(bg)),
            CellType::Snake(id) => write!(f, "{}", Paint::red(id + 1).bg(bg)),
            CellType::Tail(id) => write!(f, "{}", Paint::yellow(id + 1).bg(bg)),
        }
    }
}

impl Grid<CellGame> {
    pub fn fill(&mut self, snakes: &[Snake], food: &[Coord], hazards: &[Coord]) {
        for (i, snake) in snakes.iter().enumerate() {
            if !snake.is_alive() {
                continue;
            }

            for pos in snake.body[..snake.body.len() - 1].iter() {
                self[*pos] = CellGame::new(CellType::Snake(i as u8));
            }
            // tail coord is "unique", mark as free in next turn
            if snake.len() == 1 || *snake.tail() != snake.body[snake.body.len() - 2] {
                self[*snake.body.last().unwrap()] = CellGame::new(CellType::Tail(i as u8));
            }
        }

        // fill grid with food
        for pos in food.iter() {
            self[*pos] = CellGame::new(CellType::Food);
        }

        // fill grid with hazards
        for pos in hazards.iter() {
            if self.contains(*pos) {
                self[*pos].hazard += 1;
            }
        }
    }

    #[must_use]
    pub fn is_food(&self, pos: Coord) -> bool {
        self[pos].cell == CellType::Food
    }

    #[must_use]
    pub fn is_snake(&self, pos: Coord) -> bool {
        // not checking for tail is intentional
        matches!(self[pos].cell, CellType::Snake(_))
    }

    #[must_use]
    pub fn is_hazard(&self, pos: Coord) -> bool {
        self[pos].hazard > 0
    }

    #[must_use]
    pub fn is_valid_pos(&self, pos: Coord) -> bool {
        self.contains(pos) && !self.is_snake(pos)
    }
}
