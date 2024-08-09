mod constrictor;
mod duels;
mod floodfill;
mod royale;
mod royale_duels;
mod standard;

use crate::simulation::{Outcome, State};
pub use constrictor::ConstrictorHeuristic;
pub use duels::DuelsHeuristic;
pub use floodfill::{CellFlood, Floodfill};
pub use royale::RoyaleHeuristic;
pub use royale_duels::RoyaleDuelsHeuristic;
pub use standard::StandardHeuristic;
use std::fmt::Debug;

pub trait Heuristic: Debug + Send + Sync {
    /// Evaluate the current state and return an outcome
    /// Trait for all different types of heuristics
    fn eval(&self, state: &State) -> Outcome;
    fn eval_all(&self, state: &State) -> Vec<Outcome>;
}
