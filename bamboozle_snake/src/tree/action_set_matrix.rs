use crate::game::Direction;
use crate::tree::Node;
use itertools::Itertools;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct ActionSetMatrix {
    pub sets: Vec<Option<Node>>,
    pub action_sets: Vec<Vec<Direction>>,
    pub num_snakes: u32,
}

impl Index<&Vec<Direction>> for ActionSetMatrix {
    type Output = Option<Node>;

    fn index(&self, action_set: &Vec<Direction>) -> &Self::Output {
        let mut index: usize = 0;

        for (i, action) in action_set.iter().enumerate() {
            index += 4usize.pow(i as u32) * (*action) as usize;
        }
        &self.sets[index]
    }
}

impl IndexMut<&Vec<Direction>> for ActionSetMatrix {
    fn index_mut(&mut self, action_set: &Vec<Direction>) -> &mut Self::Output {
        let mut index: usize = 0;

        for (i, action) in action_set.iter().enumerate() {
            index += 4usize.pow(i as u32) * (*action) as usize;
        }
        &mut self.sets[index]
    }
}

impl ActionSetMatrix {
    pub fn new(num_snakes: usize) -> Self {
        Self {
            sets: vec![Option::None; 4_usize.pow(num_snakes as u32)],
            action_sets: Vec::with_capacity(3_usize.pow(num_snakes as u32)),
            num_snakes: num_snakes as u32,
        }
    }

    pub fn fill(&mut self, node: &Node, valid_actions: Vec<Vec<Direction>>) {
        for action_set in valid_actions.into_iter().multi_cartesian_product() {
            self[&action_set] = Option::Some(node.step(&action_set));
            self.action_sets.push(action_set);
        }
    }

    #[must_use]
    pub fn get_nodes(&self, snake_id: u32, action: Direction) -> Vec<&Node> {
        let base_index = 4i32.pow(snake_id) * action as i32;

        let dims_to_search = 4i32.pow(self.num_snakes - snake_id - 1);
        let dim_step = 4i32.pow(snake_id + 1);

        let possible_nodes = 4i32.pow(self.num_snakes - 1);
        let steps_in_dim = possible_nodes / dims_to_search;

        let mut nodes = Vec::with_capacity(possible_nodes as usize);

        for i in 0..dims_to_search {
            let index = base_index + (i * dim_step);

            for s in 0..steps_in_dim {
                match &self.sets[(index + s) as usize] {
                    Some(node) => nodes.push(node),
                    None => (),
                }
            }
        }

        nodes
    }
}
