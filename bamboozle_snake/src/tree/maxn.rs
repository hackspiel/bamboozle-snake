use crate::game::{Direction, GameState};
use crate::heuristic::Heuristic;
use crate::simulation::{Outcome, State};
use crate::tree::Node;
use itertools::Itertools;
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

const MAX_DEPTH: u32 = 32;
const AVAILABLE_TIME: u64 = 400;

#[must_use]
pub fn iterative_maxn(game_state: GameState, heuristic: Arc<dyn Heuristic>) -> Direction {
    let start_time = Instant::now();

    let state = State::from(&game_state);

    if state.get_valid_actions(0).len() == 1 {
        return state.get_valid_actions(0)[0];
    }

    let root_node = Node::new(state.clone(), 0);

    let available_time = Duration::from_millis(AVAILABLE_TIME);

    let (sender, receiver) = mpsc::channel();

    let should_abort = Arc::new(AtomicBool::new(false));
    let abort_copy = should_abort.clone();

    let _thread = thread::spawn(move || _iterative_maxn(root_node, sender, heuristic, abort_copy));

    let mut best_action = Direction::None;
    let mut remaining_time = available_time - (Instant::now() - start_time);

    // calc best action with respect to timeout
    while let Ok((action, outcome)) = receiver.recv_timeout(remaining_time) {
        if outcome <= Outcome::Loss.get_score() {
            break;
        }
        best_action = action;
        if outcome >= Outcome::Win.get_score() {
            break;
        }

        remaining_time = available_time.saturating_sub(start_time.elapsed());
        if remaining_time == Duration::ZERO {
            break;
        }
    }
    should_abort.store(true, Ordering::Relaxed);

    if best_action == Direction::None {
        best_action = *(state.get_valid_actions(0).first().unwrap_or(&Direction::Up));
    }

    best_action
}

fn _iterative_maxn(
    root_node: Node,
    sender: mpsc::Sender<(Direction, f32)>,
    heuristic: Arc<dyn Heuristic>,
    should_abort: Arc<AtomicBool>,
) {
    for depth in 1..MAX_DEPTH {
        let start_time = Instant::now();

        let (dir, outcome) = eval_node(&root_node, heuristic.as_ref(), depth, should_abort.clone());
        if sender.send((dir, outcome[0])).is_err() {
            break;
        }
        debug!("depth {} in {} ms", depth, start_time.elapsed().as_millis());
    }
}

#[must_use]
pub fn eval_node(
    node: &Node,
    heuristic: &dyn Heuristic,
    max_depth: u32,
    should_abort: Arc<AtomicBool>,
) -> (Direction, Vec<f32>) {
    let num_snakes = node.state.snakes.len();

    if should_abort.load(Ordering::Relaxed) {
        return (Direction::None, vec![Outcome::Loss.get_score(); num_snakes]);
    }

    // ============ termination conditions ============

    if node.state.is_end_state() {
        let mut scores = vec![Outcome::Loss.get_score(); num_snakes];

        let winner = node.state.get_winner();
        if winner != -1 {
            scores[winner as usize] = Outcome::Win.get_score();
            return (Direction::None, scores);
        } else {
            return (Direction::None, vec![Outcome::Draw.get_score(); num_snakes]);
        }
    }

    if !node.state.snakes[0].is_alive() {
        let mut scores = vec![Outcome::Draw.get_score(); num_snakes];
        scores[0] = Outcome::Loss.get_score();
        return (Direction::None, scores);
    }

    if node.depth == max_depth {
        return (
            Direction::None,
            heuristic
                .eval_all(&node.state)
                .into_iter()
                .map(|o| o.get_score())
                .collect(),
        );
    }

    // ============ iterative evaluation ============

    // get valid actions for each snake
    let mut valid_actions: Vec<Vec<Direction>> = Vec::with_capacity(num_snakes);
    for snake_i in 0..num_snakes {
        valid_actions.push(node.state.get_valid_actions(snake_i));
    }
    let action_sets: Vec<Vec<Direction>> = valid_actions
        .clone()
        .into_iter()
        .multi_cartesian_product()
        .collect();

    // save each outcome
    let mut outcomes = Vec::with_capacity(action_sets.len());

    // save outcomes for each snake mapped by taken action
    let mut snake_outcomes: Vec<[Vec<f32>; 4]> = Vec::with_capacity(num_snakes);
    for _ in 0..num_snakes {
        snake_outcomes.push(Default::default());
    }

    for action_set in action_sets.iter() {
        let new_node = node.step(action_set);

        // recursive eval
        let (_, results) = eval_node(&new_node, heuristic, max_depth, should_abort.clone());

        outcomes.push(results.clone());

        // save results in associated vectors
        for (i, result) in results.iter().enumerate() {
            snake_outcomes[i][action_set[i] as usize].push(*result);
        }
    }

    // calculate mean of results of enemies
    let mut outcomes_mean: Vec<[f32; 4]> = vec![Default::default(); num_snakes];
    for (i, outcome) in snake_outcomes.iter().enumerate() {
        let outcome_mean = &mut outcomes_mean[i];

        for action_i in 0..4 {
            // action the snake didnt take
            if outcome[action_i].is_empty() {
                outcome_mean[action_i] = f32::MIN;
            } else {
                outcome_mean[action_i] =
                    outcome[action_i].iter().sum::<f32>() / outcome[action_i].len() as f32;
            }
        }
    }

    // action set of enemies snakes if each take best mean action
    let mut best_action_set = Vec::with_capacity(num_snakes);

    for outcome_mean in outcomes_mean.iter().skip(1) {
        let (best_action, _) = get_best_action(outcome_mean);
        best_action_set.push(best_action);
    }

    // find best own action in nodes with these actions taken
    let mut best_i = 0;
    let mut best_outcome = f32::MIN;
    let mut best_mean = f32::MIN;

    let action_means = &outcomes_mean[0];
    for (i, action_set) in action_sets.iter().enumerate() {
        if action_set[1..] == best_action_set {
            let outcome = outcomes[i][0];
            let mean = action_means[action_set[0] as usize];

            // dont take actions that could kill our self's
            if (outcome > best_outcome && mean > 0.0) || (best_mean < 0.0 && mean > best_mean) {
                best_i = i;
                best_outcome = outcome;
                best_mean = mean;
            }
        }
    }

    (action_sets[best_i][0], outcomes[best_i].clone())
}

#[must_use]
fn get_best_action(outcomes: &[f32; 4]) -> (Direction, f32) {
    let max_i = outcomes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
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
