use clap::Parser;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;

use crate::game::{Direction, GameState};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use serde_json::json;
use yansi::Paint;

use crate::heuristic::{ConstrictorHeuristic, RoyaleDuelsHeuristic};
use crate::simulation::{Mode, State};
use crate::tree::iterative_search_mt;

use crate::heuristic::{DuelsHeuristic, Heuristic, RoyaleHeuristic, StandardHeuristic};

#[derive(Parser, Debug)]
#[command(author, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 4)]
    pub threads_per_game: usize,
    #[arg(long, default_value_t = 444)]
    pub timeout: u64,
    #[arg(long, default_value_t = 32)]
    pub max_depth: u32,
    #[arg(long, default_value_t = 8005)]
    pub port: u16,
    #[arg(long, default_value_t = String::from("bamboozle snake"))]
    pub name: String,
    #[arg(long, default_value_t = DuelsHeuristic::default())]
    pub duel_heuristic: DuelsHeuristic,
    #[arg(long, default_value_t = RoyaleHeuristic::default())]
    pub royal_heuristic: RoyaleHeuristic,
}

pub static CONFIG: Lazy<Args> = Lazy::new(Args::parse);

pub fn handle_start(game_state: GameState) {
    let mut snakes = "".to_string();
    let num_snakes = game_state.board.snakes.len();

    for i in 0..num_snakes {
        snakes.push_str(&game_state.board.snakes[i].name);
        if i != num_snakes - 1 {
            snakes.push_str(", ");
        }
    }

    info!(
        "Start ({:?}): {} ({})",
        State::determine_mode(&game_state), game_state.game.id, snakes
    );
}

pub async fn handle_move(game_state: GameState) -> Result<impl warp::Reply, Infallible> {
    let start_time = Instant::now();

    let action = tokio::task::spawn_blocking(move || {
        let player_count = game_state.board.snakes.len();
        let heuristic: Arc<dyn Heuristic>;

        let mode = State::determine_mode(&game_state);

        if mode == Mode::Constrictor {
            heuristic = Arc::new(ConstrictorHeuristic::default());
        } else if mode == Mode::Royale {
            if player_count == 2 {
                heuristic = Arc::new(RoyaleDuelsHeuristic::default());
            } else {
                heuristic = Arc::new(CONFIG.royal_heuristic);
            }
        } else if mode == Mode::Standard {
            heuristic = Arc::new(StandardHeuristic::default());
        } else {
            heuristic = Arc::new(CONFIG.duel_heuristic);
        }
        debug!("using {:?} in step {}", heuristic, game_state.turn);
        iterative_search_mt(game_state, heuristic)
    })
        .await
        .unwrap_or(Direction::None);

    if action == Direction::None {
        error!("{:?} in {}ms", action, start_time.elapsed().as_millis());
    } else {
        debug!("{:?} in {}ms", action, start_time.elapsed().as_millis());
    }

    if start_time.elapsed().as_millis() > 450 {
        warn!(
            "Calculation took too much time ({} ms)",
            start_time.elapsed().as_millis()
        )
    }
    Ok(warp::reply::json(&json!({"move":action.to_string()})))
}

pub fn handle_end(game_state: GameState) {
    let outcome;

    if game_state.board.snakes.is_empty() {
        outcome = Paint::yellow("Draw").to_string();
    } else if game_state.board.snakes[0] == game_state.you {
        outcome = Paint::green("Win").to_string();
    } else {
        outcome = format!(
            "{} (winner: {})",
            Paint::red("Loss"),
            game_state.board.snakes[0].name
        );
    }

    info!(
        "End: {} after {} turns ({})",
        outcome, game_state.turn, game_state.game.id
    );
}
