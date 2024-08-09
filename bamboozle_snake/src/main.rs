use std::env;
use log::debug;
use serde_json::json;
use warp::Filter;

use bamboozle_snake::game::GameState;
use bamboozle_snake::logic::{handle_end, handle_move, handle_start, CONFIG};

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info,rocket=warn,_=warn");
    }

    env_logger::init();
    debug!("{:?}", *CONFIG);

    let index_endpoint = warp::get().and(warp::path::end()).map(|| {
        warp::reply::json(&json!({
            "apiversion": "1",
            "author": "lesshack",
            "name": CONFIG.name,
            "color": "#000000",
            "head": "pirate",
            "tail": "pirate"
        }))
    });

    let start_endpoint = warp::path("start")
        .and(warp::post())
        .and(warp::body::json::<GameState>())
        .map(|game_state: GameState| {
            handle_start(game_state);
            warp::reply()
        });

    let move_endpoint = warp::path("move")
        .and(warp::post())
        .and(warp::body::json::<GameState>())
        .and_then(handle_move);

    let end_endpoint = warp::path("end")
        .and(warp::post())
        .and(warp::body::json::<GameState>())
        .map(|game_state: GameState| {
            handle_end(game_state);
            warp::reply()
        });

    warp::serve(
        index_endpoint
            .or(start_endpoint)
            .or(move_endpoint)
            .or(end_endpoint),
    )
        .run(([0, 0, 0, 0], CONFIG.port))
        .await;
}
