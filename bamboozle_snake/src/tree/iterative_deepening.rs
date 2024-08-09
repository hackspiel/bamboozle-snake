use crate::game::{Direction, GameState};
use crate::heuristic::Heuristic;
use crate::simulation::{LossType, Outcome, State};
use crate::tree::{alphabeta, Node, TreeAlgorithm};
use log::debug;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::logic::CONFIG;

pub fn iterative_search_mt(game_state: GameState, heuristic: Arc<dyn Heuristic>) -> Direction {
    let start_time = Instant::now();
    let available_time = Duration::from_millis(CONFIG.timeout);

    let state = State::from(&game_state);

    // return if only one action is available
    if state.get_valid_actions(0).len() == 1 {
        return state.get_valid_actions(0)[0];
    }
    let root_node = Node::new(state.clone(), 0);

    // prepare work queue for multithreading
    let work_queue = Arc::new(Mutex::new(
        (0..CONFIG.max_depth)
            .map(|i| (i + 1, root_node.clone()))
            .collect::<VecDeque<_>>(),
    ));

    // thread stuff
    let (sender, receiver) = mpsc::channel();
    let should_abort = Arc::new(AtomicBool::new(false));

    // start threads
    for _ in 0..CONFIG.threads_per_game {
        let work_queue = work_queue.clone();
        let sender = sender.clone();
        let should_abort = should_abort.clone();
        let heuristic = heuristic.clone();
        let _thread = thread::spawn(move || {
            iterative_deepening_work(work_queue, sender, heuristic, should_abort)
        });
    }

    let mut best_action = Direction::None;
    let mut current_outcome = Outcome::Loss(LossType::OwnOrWallCollision);
    let mut current_depth = 0;
    let mut remaining_time = available_time - start_time.elapsed();
    // let mut death_reasons = [(Outcome::Loss(LossType::None), 0); 4];

    // calc best action with respect to timeout
    while let Ok((depth, action, outcome)) = receiver.recv_timeout(remaining_time) {
        debug!(
            "{:?} ({:?}) after a depth of {} in {:?} ",
            outcome,
            action,
            depth,
            start_time.elapsed(),
        );

        // if death_reasons[action as usize].0 < outcome && death_reasons[action as usize].1 < depth{
        //     death_reasons[action as usize] = (outcome, depth);
        // }

        if depth > current_depth && outcome > Outcome::Loss(LossType::None) {
            current_depth = depth;
            best_action = action;
            current_outcome = outcome;
            if matches!(outcome, Outcome::Win(_)) {
                break;
            }
        } else if current_outcome < Outcome::Loss(LossType::None) && outcome > current_outcome {
            current_depth = depth;
            best_action = action;
            current_outcome = outcome;
        }

        remaining_time = available_time.saturating_sub(start_time.elapsed());
        if remaining_time == Duration::ZERO {
            break;
        }
    }
    should_abort.store(true, Ordering::SeqCst);

    if best_action == Direction::None {
        best_action = *(state.get_valid_actions(0).first().unwrap_or(&Direction::Up));
    }

    debug!(
        "Took {:?} to calculate action {:?} at depth {} with outcome {:?}",
        start_time.elapsed(),
        best_action,
        current_depth,
        current_outcome,
    );

    best_action
}

fn iterative_deepening_work(
    work_queue: Arc<Mutex<VecDeque<(u32, Node)>>>,
    sender: mpsc::Sender<(u32, Direction, Outcome)>,
    heuristic: Arc<dyn Heuristic>,
    should_abort: Arc<AtomicBool>,
) {
    while !should_abort.load(Ordering::SeqCst) {
        let mut queue = work_queue.lock().unwrap();
        if queue.is_empty() {
            break;
        }
        let (depth, mut root_node) = queue.pop_front().unwrap();
        // unlock mutex
        drop(queue);

        root_node.update_snake_simulation(depth);
        let (dir, outcome, evaluated_nodes) =
            alphabeta::run_alphabeta(&root_node, heuristic.clone(), depth, should_abort.clone());

        if sender.send((depth, dir, outcome)).is_err() {
            break;
        }
    }
}

pub fn iterative_search(
    game_state: GameState,
    algorithm: TreeAlgorithm,
    heuristic: Arc<dyn Heuristic>,
) -> Direction {
    let start_time = Instant::now();
    let available_time = Duration::from_millis(CONFIG.timeout);

    let state = State::from(&game_state);

    if state.get_valid_actions(0).len() == 1 {
        return state.get_valid_actions(0)[0];
    }

    let root_node = Node::new(state.clone(), 0);

    // thread stuff
    let (sender, receiver) = mpsc::channel();
    let should_abort = Arc::new(AtomicBool::new(false));
    let should_abort_clone = should_abort.clone();

    let _thread = thread::spawn(move || {
        iterative_deepening(root_node, sender, algorithm, heuristic, should_abort_clone)
    });

    let mut best_action = Direction::None;
    let mut remaining_time = available_time - start_time.elapsed();

    // calc best action with respect to timeout
    while let Ok((action, outcome)) = receiver.recv_timeout(remaining_time) {
        if outcome == Outcome::Loss(LossType::OwnOrWallCollision) {
            break;
        }
        best_action = action;
        if matches!(outcome, Outcome::Win(_)) {
            break;
        }

        remaining_time = available_time.saturating_sub(start_time.elapsed());
        if remaining_time == Duration::ZERO {
            break;
        }
    }
    should_abort.store(true, Ordering::SeqCst);

    if best_action == Direction::None {
        best_action = *(state.get_valid_actions(0).first().unwrap_or(&Direction::Up));
    }

    best_action
}

fn iterative_deepening(
    mut root_node: Node,
    sender: mpsc::Sender<(Direction, Outcome)>,
    algorithm: TreeAlgorithm,
    heuristic: Arc<dyn Heuristic>,
    should_abort: Arc<AtomicBool>,
) {
    for depth in 1..CONFIG.max_depth {
        let start_time = Instant::now();

        let (dir, outcome, evaluated_nodes) = match algorithm {
            TreeAlgorithm::AlphaBeta => {
                root_node.update_snake_simulation(depth);
                alphabeta::run_alphabeta(&root_node, heuristic.clone(), depth, should_abort.clone())
            }
            TreeAlgorithm::AlphabetaMultithread => {
                todo!()
                // root_node.update_snake_simulation(depth);
                // alphabeta_multithread::run_alphabeta_multithread(
                //     &root_node,
                //     heuristic.clone(),
                //     depth,
                //     should_abort.clone(),
                // )
            }
            TreeAlgorithm::Minimax => {
                todo!()
                // minimax::run_minimax(&root_node, heuristic.clone(), depth, should_abort.clone())
            }
            TreeAlgorithm::MaxN => todo!(),
        };

        if sender.send((dir, outcome)).is_err() {
            break;
        }

        debug!(
            "{:?} after a depth of {} in {} ms with {} evaluated nodes",
            outcome,
            depth,
            start_time.elapsed().as_millis(),
            evaluated_nodes
        );
    }
}
