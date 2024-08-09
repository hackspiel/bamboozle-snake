use crate::game::Direction;
use crate::heuristic::Heuristic;
use crate::simulation::{LossType, Outcome};
use crate::tree::{get_best_action, Node};
use itertools::Itertools;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct AlphaBeta {
    pub alpha: f32,
    pub beta: f32,
}

impl AlphaBeta {
    #[must_use]
    pub fn new(a: f32, b: f32) -> Self {
        Self { alpha: a, beta: b }
    }

    #[must_use]
    pub fn should_abort(&self) -> bool {
        self.alpha >= self.beta
    }
}

pub fn run_alphabeta(
    root_node: &Node,
    heuristic: Arc<dyn Heuristic>,
    max_depth: u32,
    should_abort: Arc<AtomicBool>,
) -> (Direction, Outcome, usize) {
    let alpha_beta = AlphaBeta::new(f32::MIN, f32::MAX);

    eval_node(
        root_node,
        max_depth,
        heuristic.as_ref(),
        alpha_beta,
        should_abort,
    )
}

pub fn eval_node(
    node: &Node,
    max_depth: u32,
    heuristic: &dyn Heuristic,
    alpha_beta: AlphaBeta,
    should_abort: Arc<AtomicBool>,
) -> (Direction, Outcome, usize) {
    // ============ termination conditions ============
    if should_abort.load(Ordering::Relaxed) || alpha_beta.should_abort() {
        return (Direction::None, Outcome::Loss(LossType::default()), 1);
    }
    if node.state.is_end_state() {
        return match node.state.get_winner() {
            -1 => (Direction::None, Outcome::Draw, 1),
            0 => (
                Direction::None,
                Outcome::Win(-(node.state.snakes[0].len() as f32)),
                1,
            ),
            _ => (
                Direction::None,
                Outcome::Loss(node.state.snakes[0].loss_reason),
                1,
            ),
        };
    }

    if !node.state.snakes[0].is_alive() {
        return (
            Direction::None,
            Outcome::Loss(node.state.snakes[0].loss_reason),
            1,
        );
    }

    if node.depth == max_depth {
        return (Direction::None, heuristic.eval(&node.state), 1);
    }

    // ============ recursive evaluation ============
    let num_snakes = node.state.snakes.len();
    let mut evaluated_nodes = 1;

    // valid actions of snakes (our actions are empty for now)
    let mut valid_actions_blueprint: Vec<Vec<Direction>> = Vec::with_capacity(num_snakes);
    valid_actions_blueprint.push(Vec::new());
    valid_actions_blueprint.extend((1..num_snakes).map(|si| node.state.get_valid_actions(si)));

    let mut alpha_beta = alpha_beta;

    // ============ max step ============
    let mut scores = [Outcome::Loss(LossType::OwnOrWallCollision); 4];

    for own_action in node.state.get_valid_actions(0).into_iter() {
        if alpha_beta.should_abort() {
            break;
        }
        let mut valid_actions = valid_actions_blueprint.clone();
        valid_actions[0].push(own_action);

        // ============ min step ============
        let mut worst_outcome = Outcome::Win(1000.0);
        let mut alpha_beta_min = alpha_beta;
        for action_set in valid_actions.into_iter().multi_cartesian_product() {
            if alpha_beta_min.should_abort() {
                worst_outcome = Outcome::Loss(LossType::default());
                break;
            }

            // simulate actions
            let next_node = node.step(&action_set);

            let (_, outcome, ev_nodes) = eval_node(
                &next_node,
                max_depth,
                heuristic,
                alpha_beta_min,
                should_abort.clone(),
            );

            evaluated_nodes += ev_nodes;

            if outcome < worst_outcome {
                worst_outcome = outcome;
            }
            if alpha_beta_min.beta > worst_outcome.get_score() {
                alpha_beta_min.beta = worst_outcome.get_score();
            }

            // TODO: check LossType
            if worst_outcome == Outcome::Loss(LossType::OwnOrWallCollision) {
                break;
            }
        }
        scores[own_action as usize] = worst_outcome;

        if alpha_beta.alpha < worst_outcome.get_score() {
            alpha_beta.alpha = worst_outcome.get_score();
        }
    }

    let (dir, best_outcome) = get_best_action(scores);

    (dir, best_outcome, evaluated_nodes)
}
