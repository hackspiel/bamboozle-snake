use crate::game::Direction;
use crate::simulation::{Mode, State};

#[derive(Debug, Clone)]
pub struct Node {
    pub state: State,
    pub depth: u32,
}

impl Node {
    #[must_use]
    pub fn new(state: State, depth: u32) -> Self {
        Self { state, depth }
    }

    #[must_use]
    pub fn step(&self, action_set: &Vec<Direction>) -> Self {
        let state = self.state.step(action_set);
        let depth = self.depth + 1;

        Self::new(state, depth)
    }

    pub fn update_snake_simulation(&mut self, max_depth: u32) {
        let snakes = &mut self.state.snakes;

        let our_head = snakes[0].head();

        if snakes.len() > 3 && self.state.mode != Mode::Constrictor  {
            for other_snake in snakes.iter_mut().skip(1) {
                other_snake.should_simulate =
                    our_head.manhattan_dist(&other_snake.head()) <= 2 * max_depth;
            }
        }
    }
}
