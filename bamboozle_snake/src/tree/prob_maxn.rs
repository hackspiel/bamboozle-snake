use crate::game::{Direction, GameState};
use crate::heuristic::Heuristic;
use crate::simulation::Outcome;
use crate::simulation::State;
use crate::tree::action_set_matrix::ActionSetMatrix;
use crate::tree::Node;
use itertools::Itertools;
use std::cmp::{min, Ordering};
use std::collections::BinaryHeap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
struct PriorityElement {
    prob: f32,
    action_set: Vec<Direction>,
    index: usize,
}

impl Eq for PriorityElement {}

impl Ord for PriorityElement {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.prob < other.prob {
            return Ordering::Less;
        }

        if self.prob > other.prob {
            return Ordering::Greater;
        }

        return Ordering::Equal;
    }
}

impl PartialOrd for PriorityElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn maxn(
    game_state: State,
    branch_factor: usize,
    heuristic: &dyn Heuristic,
    max_depth: u32,
) -> Direction {
    // let state = State::from(&game_state);
    let mut root_node = Node::new(game_state.clone(), 0);

    let (outcome, direction) =
        maxn_invoke_node(&mut root_node, branch_factor, heuristic, max_depth);

    direction
}

pub fn get_action_values(
    valid_actions: &Vec<Vec<Direction>>,
    action_sets: &Vec<Vec<Direction>>,
    values: &Vec<Vec<f32>>,
) -> Vec<Vec<f32>> {
    // compute values for each snakes action by averaging over all successor nodes, taking one action

    let num_snakes = valid_actions.len();

    let mut action_values: Vec<Vec<f32>> = vec![Vec::new(); num_snakes];
    for snake in 0..num_snakes {
        for action in valid_actions[snake].iter() {
            let mut action_value = 0.0;
            let mut succ_node_count = 0;
            for (i, value) in values[snake].iter().enumerate() {
                if action_sets[i][snake] == *action {
                    action_value += value;
                    succ_node_count += 1;
                }
            }
            action_values[snake].push(action_value / succ_node_count as f32);
        }
    }

    action_values
}

pub fn get_action_probabilities(action_values: &Vec<Vec<f32>>, num_snakes: usize) -> Vec<Vec<f32>> {
    let mut action_probs = vec![Vec::new(); num_snakes];
    for snake in 0..num_snakes {
        let action_value_sum: f32 = action_values[snake].iter().sum();
        for action_value in action_values[snake].iter() {
            action_probs[snake].push(action_value / action_value_sum);
        }
    }

    action_probs
}

pub fn get_action_set_probabilities(
    valid_actions: &Vec<Vec<Direction>>,
    action_sets: &Vec<Vec<Direction>>,
    action_probs: &Vec<Vec<f32>>,
) -> Vec<f32> {
    let mut action_set_probs = Vec::new();
    for action_set in action_sets.iter() {
        let mut prob = 1.0;
        for (s_i, action) in action_set.iter().enumerate() {
            let action_i = valid_actions[s_i]
                .iter()
                .position(|&a| a == *action)
                .unwrap();
            prob *= action_probs[s_i][action_i];
        }
        action_set_probs.push(prob);
    }

    action_set_probs
}

pub fn maxn_invoke_node(
    node: &Node,
    branch_factor: usize,
    heuristic: &dyn Heuristic,
    max_depth: u32,
) -> (Vec<f32>, Direction) {
    let num_snakes = node.state.snakes.len();

    // ============ termination conditions ============

    if node.state.is_end_state() {
        let mut scores = vec![Outcome::Loss.get_score(); num_snakes];

        let winner = node.state.get_winner();
        if winner != -1 {
            scores[winner as usize] = Outcome::Win.get_score();
            return (scores, Direction::None);
        } else {
            return (vec![Outcome::Draw.get_score(); num_snakes], Direction::None);
        }
    }

    if !node.state.snakes[0].is_alive() {
        let mut scores = vec![Outcome::Draw.get_score(); num_snakes];
        scores[0] = Outcome::Loss.get_score();
        return (scores, Direction::None);
    }

    if node.depth == max_depth {
        return (
            heuristic
                .eval_all(&node.state)
                .into_iter()
                .map(|o| o.get_score())
                .collect(),
            Direction::None,
        );
    }

    // ============== pruning calculations =====================

    // get valid actions for each snake
    let mut valid_actions: Vec<Vec<Direction>> = Vec::with_capacity(num_snakes);
    for i in 0..num_snakes {
        valid_actions.push(node.state.get_valid_actions(i));
    }

    // generate all possible action sets for this state
    let action_sets: Vec<Vec<Direction>> = valid_actions
        .clone()
        .into_iter()
        .multi_cartesian_product()
        .collect();

    // simulate all successor nodes with the action sets
    let mut next_nodes: Vec<(Node, Vec<Direction>)> = Vec::new();
    for action_set in action_sets.iter() {
        next_nodes.push((node.step(&action_set), (*action_set).clone()));
    }

    // compute the heuristic for each successor node for each snake
    let mut next_node_values: Vec<Vec<f32>> = vec![Vec::new(); num_snakes];
    for (n, _) in next_nodes.iter() {
        // TODO: check endstate
        for (i, value) in heuristic.eval_all(&n.state).into_iter().enumerate() {
            next_node_values[i].push(value.get_score());
        }
    }

    let mut action_values = get_action_values(&valid_actions, &action_sets, &next_node_values);
    let mut action_probs = get_action_probabilities(&action_values, num_snakes);

    // compute a probability for an action set to occur and sort them
    let mut action_set_probs = BinaryHeap::with_capacity(action_sets.len());
    for (node_i, action_set) in action_sets.iter().enumerate() {
        let mut prop = 1.0;
        for (s_i, action) in action_set.iter().enumerate() {
            let action_i = valid_actions[s_i]
                .iter()
                .position(|&a| a == *action)
                .unwrap();
            prop *= action_probs[s_i][action_i];
        }
        action_set_probs.push(PriorityElement {
            action_set: (*action_set).clone(),
            prob: prop,
            index: node_i,
        });
    }

    // take the n most likely action sets
    let mut expanded_nodes = Vec::new();
    let n = min(branch_factor, action_set_probs.len());
    for i in 0..n {
        let action_set_with_prob = action_set_probs.pop().unwrap();
        expanded_nodes.push((
            &next_nodes[action_set_with_prob.index].0,
            action_set_with_prob,
        ));
    }

    // =================== recurse ====================

    // calculate the values of expanded nodes

    let mut new_next_node_values: Vec<Vec<f32>> = vec![Vec::new(); num_snakes];
    for (exp_node, prio_element) in expanded_nodes.iter_mut() {
        let (new_node_values, _) = maxn_invoke_node(exp_node, branch_factor, heuristic, max_depth);
        for (snake_i, value) in new_node_values.iter().enumerate() {
            new_next_node_values[snake_i].push(*value);
        }
    }

    let mut new_action_sets: Vec<Vec<Direction>> = Vec::new();
    for (node, prio_element) in expanded_nodes.iter() {
        new_action_sets.push(prio_element.action_set.clone());
    }

    let mut new_action_values =
        get_action_values(&valid_actions, &new_action_sets, &new_next_node_values);
    let mut new_action_probs = get_action_probabilities(&new_action_values, num_snakes);
    let mut new_action_set_probs =
        get_action_set_probabilities(&valid_actions, &new_action_sets, &new_action_probs);

    for snake_i in 0..num_snakes {}

    // get maximizing action
    // let max_i = scores
    //     .iter()
    //     .enumerate()
    //     .max_by(|(_, a), (_, b)| a.total_cmp(&b))
    //     .map(|(index, _)| index)
    //     .unwrap();
    //
    // let action = match max_i {
    //     0 => Direction::Up,
    //     1 => Direction::Right,
    //     2 => Direction::Down,
    //     3 => Direction::Left,
    //     _ => Direction::None,
    // };

    // println!("{} {:?} {:?}", scores[max_i], action, node.state.grid);
    todo!()

    // (scores[max_i], action)
}
