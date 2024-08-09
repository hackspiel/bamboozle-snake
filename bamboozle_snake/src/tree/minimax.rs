use crate::game::Direction;
use crate::heuristic::Heuristic;
use crate::simulation::Outcome;
use crate::simulation::Outcome::{Draw, Loss, Win};
use crate::tree::{get_best_action, Node};
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub fn run_minimax(
    root_node: &Node,
    heuristic: Arc<dyn Heuristic>,
    max_depth: u32,
    should_abort: Arc<AtomicBool>,
) -> (Direction, Outcome, usize) {
    eval_node(root_node, max_depth, heuristic.as_ref(), should_abort)
}

fn eval_node(
    node: &Node,
    max_depth: u32,
    heuristic: &dyn Heuristic,
    should_abort: Arc<AtomicBool>,
) -> (Direction, Outcome, usize) {
    if should_abort.load(Ordering::Relaxed) {
        return (Direction::None, Loss, 1);
    }

    if node.state.is_end_state() {
        return match node.state.get_winner() {
            -1 => (Direction::None, Draw, 1),
            0 => (Direction::None, Win, 1),
            _ => (Direction::None, Loss, 1),
        };
    }

    if !node.state.snakes[0].is_alive() {
        return (Direction::None, Loss, 1);
    }

    if node.depth == max_depth {
        return (Direction::None, heuristic.eval(&node.state), 1);
    }

    let evaluated_nodes = Arc::new(AtomicUsize::new(1));
    let num_snakes = node.state.snakes.len();
    let mut scores = [Loss; 4];

    // prepare blueprint for valid actions
    let mut valid_actions_blueprint: Vec<Vec<Direction>> = Vec::with_capacity(num_snakes);
    valid_actions_blueprint.push(Vec::new());

    let our_snake = &node.state.snakes[0];

    for i in 1..num_snakes {
        // only fully simulate nearby snakes
        if num_snakes > 3
            && node
                .state
                .grid
                .manhattan_dist(&our_snake.head(), &node.state.snakes[i].head())
                > (max_depth - node.depth) * 2
        {
            valid_actions_blueprint.push(vec![Direction::None]);
        } else {
            valid_actions_blueprint.push(node.state.get_valid_actions(i));
        }
    }

    // closure for multi-threading
    let eval_action = |(action, outcome): &mut (Direction, Outcome)| {
        if should_abort.load(Ordering::Relaxed) {
            return;
        }

        let mut valid_actions = valid_actions_blueprint.clone();
        valid_actions[0].push(*action);

        let mut worst_outcome = Win;

        // iterate over possible actions of other snakes
        for action_set in valid_actions.into_iter().multi_cartesian_product() {
            let action_node = node.step(&action_set);

            let (_, outcome, node_count) =
                eval_node(&action_node, max_depth, heuristic, should_abort.clone());

            evaluated_nodes.fetch_add(node_count, Ordering::Relaxed);

            if outcome.get_score() < worst_outcome.get_score() {
                worst_outcome = outcome;
            }

            if worst_outcome == Loss {
                break;
            }
        }
        *outcome = worst_outcome;
    };

    let mut pair_vec: Vec<(Direction, Outcome)> = node
        .state
        .get_valid_actions(0)
        .into_iter()
        .map(|a| (a, Draw))
        .collect();

    // iterate over own valid actions
    if node.depth < 1 {
        pair_vec.par_iter_mut().for_each(|x| eval_action(x));
    } else {
        pair_vec.iter_mut().for_each(|x| eval_action(x));
    }

    for (action, outcome) in pair_vec.into_iter() {
        scores[action as usize] = outcome;
    }

    let (dir, best_outcome) = get_best_action(scores);

    (dir, best_outcome, evaluated_nodes.load(Ordering::Relaxed))
}
