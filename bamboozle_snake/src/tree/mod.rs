mod action_set_matrix;
mod alphabeta;
mod iterative_deepening;
mod node;
mod monte_carlo;

use crate::game::Direction;
use crate::simulation::Outcome;
pub use action_set_matrix::ActionSetMatrix;
pub use alphabeta::run_alphabeta;
pub use iterative_deepening::{iterative_search, iterative_search_mt};
pub use node::Node;
pub use monte_carlo::monte_carlo;

pub enum TreeAlgorithm {
    Minimax,
    MaxN,
    AlphaBeta,
    AlphabetaMultithread,
}

#[must_use]
pub fn get_best_action(outcomes: [Outcome; 4]) -> (Direction, Outcome) {
    let max_i = outcomes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.get_score().total_cmp(&b.get_score()))
        .map(|(index, _)| index)
        .unwrap();

    let action = match max_i {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        3 => Direction::Left,
        _ => Direction::None,
    };
    (action, outcomes[max_i])
}
