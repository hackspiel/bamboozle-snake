mod cell;
mod outcome;
mod snake;
mod state;

pub use cell::{CellGame, CellType};
pub use outcome::{LossType, Outcome};
pub use snake::Snake;
pub use state::State;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Standard,
    Duels,
    Royale,
    Constrictor,
    Snail,
}
