use battlesnake::coord;
use battlesnake::game::{Coord, Direction, GameState};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;

use battlesnake::grid::Grid;
use battlesnake::heuristic::floodfill::FloodType;
use battlesnake::heuristic::Floodfill;
use battlesnake::simulation::State;

pub fn load_games(path: &str) -> Vec<GameState> {
    let file = File::open(path).unwrap();
    serde_json::from_reader(file).expect("Could not parse games")
}

pub fn grid_benchmark(c: &mut Criterion) {
    let mut grid: Grid<u32> = Grid::new(11, 11, false);

    let coord = coord!(2, 5);

    c.bench_function("read grid", |b| b.iter(|| grid[black_box(coord)]));
    c.bench_function("set grid", |b| b.iter(|| grid[black_box(coord)] = 2));
}

pub fn games(c: &mut Criterion) {
    let games = load_games("games.json");

    for (i, game) in games.iter().enumerate() {
        let state = State::from(game);

        let actions: Vec<Direction> = (0..state.snakes.len())
            .map(|i| state.get_valid_actions(i)[0])
            .collect();

        let title = format!("floodfill game {}", i);
        c.bench_function(title.as_str(), |b| {
            b.iter(|| state.step(black_box(&actions)))
            // b.iter(|| Floodfill::new(black_box(&state), FloodType::FollowSnakes))
        });
    }
}

criterion_group!(benches, grid_benchmark, games);
criterion_main!(benches);
