use log::debug;
use rand::seq::SliceRandom;
use tokio::time::Instant;

use crate::game::{Direction, GameState};
use crate::simulation::{LossType, Outcome, State};

pub fn run_to_end(state: &State) -> (Outcome, Direction) {
    let mut state = state.clone();

    let mut actions = vec![Direction::None; state.snakes.len()];

    for i in 0..(state.snakes.len()) {
        let possible_actions = state.get_valid_actions(i);
        actions[i] = *possible_actions.choose(&mut rand::thread_rng()).unwrap();
    }
    let action = actions[0];

    while !state.is_end_state() {
        for i in 0..(state.snakes.len()) {
            let possible_actions = state.get_valid_actions(i);
            actions[i] = *possible_actions.choose(&mut rand::thread_rng()).unwrap();
        }
        state = state.step(&actions);
    }

    let outcome = match state.get_winner() {
        0 => Outcome::Win(0.0),
        -1 => Outcome::Draw,
        _ => Outcome::Loss(LossType::None)
    };
    // debug!("{:?}, {:?}", actions[0], outcome);
    (outcome, action)
}

pub fn monte_carlo(game_state: GameState) -> Direction {
    let start_time = Instant::now();
    let mut counter = 0;
    let state = State::from(&game_state);
    debug!("{:?}", state.get_valid_actions(0));

    let mut dir_results = [f64::NEG_INFINITY; 4];

    let valid_actions = state.get_valid_actions(0);
    for action in valid_actions.iter() {
        dir_results[*action as usize] = 0.0;
    }

    while start_time.elapsed().as_millis() < 400 {
        let (outcome, direction) = run_to_end(&state);
        match outcome {
            Outcome::Win(_) => { dir_results[direction as usize] += 1.0; }
            Outcome::Loss(_) => { dir_results[direction as usize] -= 1.0; }
            _ => ()
        }
        counter += 1;
    }
    let mut max_result = -100000000.0;
    let mut max_i = -1;

    for i in 0..4 {
        if dir_results[i] >= max_result {
            max_result = dir_results[i];
            max_i = i as i32;
        }
    }

    let action = match max_i {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        3 => Direction::Left,
        _ => Direction::None,
    };

    debug!("Run {:?} simulations with result {:?}",counter, dir_results);

    action
}