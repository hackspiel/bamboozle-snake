use crate::coord;
use crate::game::Coord;
use crate::heuristic::floodfill::FloodType;
use crate::heuristic::{CellFlood, Floodfill, Heuristic};
use crate::simulation::{Outcome, State};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct StandardHeuristic {
    pub area: f32,
    pub health: f32,
    pub length: f32,
    pub alive_enemies: f32,
    pub food: f32,
    pub central: f32,
}

impl Default for StandardHeuristic {
    fn default() -> Self {
        Self {
            area: 3.0,
            health: 3.0,
            length: 1.5,
            alive_enemies: 4.0,
            food: 1.0,
            central: 0.25,
        }
    }
}

impl Heuristic for StandardHeuristic {
    fn eval(&self, state: &State) -> Outcome {
        if !state.snakes[0].is_alive() {
            return Outcome::Loss(state.snakes[0].loss_reason);
        }

        let floodmap = Floodfill::new(state, FloodType::FollowSnakes);

        Outcome::Heuristic(self.calc_score(state, &floodmap, 0))
    }

    fn eval_all(&self, state: &State) -> Vec<Outcome> {
        let mut outcomes = Vec::with_capacity(state.snakes.len());

        let floodmap = Floodfill::new(state, FloodType::Simple);

        for (snake_id, snake) in state.snakes.iter().enumerate() {
            if !snake.is_alive() {
                outcomes.push(Outcome::Loss(snake.loss_reason));
                continue;
            }
            outcomes.push(Outcome::Heuristic(
                self.calc_score(state, &floodmap, snake_id),
            ));
        }
        outcomes
    }
}

impl StandardHeuristic {
    pub fn calc_score(&self, state: &State, floodmap: &Floodfill, snake_id: usize) -> f32 {
        let mut area_score = self.area(floodmap, 0.0, snake_id);
        if floodmap.dead_ends[0] && floodmap.dead_ends.iter().skip(1).any(|d| !*d) {
            area_score -= 10.0;
        }

        let health_score = StandardHeuristic::health(state, snake_id);
        let length_score = StandardHeuristic::length(state, snake_id);
        let alive_enemies_score = StandardHeuristic::alive_enemies(state, snake_id);
        let central_score = StandardHeuristic::central(state, snake_id);
        let food_score = StandardHeuristic::food(state, floodmap, snake_id);

        self.area * area_score
            + self.health * health_score
            + self.length * length_score
            + self.alive_enemies * alive_enemies_score
            + self.central * central_score
            + self.food * food_score
    }

    pub fn area(&self, floodmap: &Floodfill, snake_discount: f32, snake_id: usize) -> f32 {
        let (owned_cells, owned_snake_cells) = floodmap.count_owned(snake_id as u8);

        let snake_length_sum = floodmap
            .state
            .snakes
            .iter()
            .filter(|s| s.is_alive())
            .map(|s| s.len())
            .sum::<usize>();

        (owned_cells as f32 + owned_snake_cells as f32 * snake_discount)
            / (floodmap.cells.width * floodmap.cells.height - snake_length_sum) as f32
    }

    pub fn health(state: &State, snake_id: usize) -> f32 {
        let health = state.snakes[snake_id].health;

        if health > 95 {
            1.0
        } else {
            (health as f32 / 95.0).sqrt()
        }
    }

    pub fn length(state: &State, snake_id: usize) -> f32 {
        let length = state.snakes[snake_id].len() as i32;

        // we want to be the longest snake so we have to challenge the longest enemy snake
        let max_other_length = state
            .snakes
            .iter()
            .skip(1)
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        let mut length_diff = length - max_other_length as i32;

        // we do not have to become much longer than the others
        if length_diff > 3 {
            length_diff = 3;
        }

        (length_diff.abs() as f32).sqrt() * length_diff.signum() as f32
    }

    pub fn food(state: &State, floodmap: &Floodfill, snake_id: usize) -> f32 {
        let mut food_dists = Vec::with_capacity(state.food.len());

        // get owned foods
        for food in state.food.iter() {
            if let CellFlood::Owned { id, step, .. } = floodmap.cells[*food] {
                if id as usize == snake_id {
                    food_dists.push(step);
                }
            }
        }

        // only care about 3 nearest foods
        if food_dists.len() > 3 {
            food_dists = food_dists.into_iter().sorted().take(3).collect();
        }

        // dont care about food too far away
        let max_dist = (state.grid.width + state.grid.height) as u32;

        let mut score = 0.0;
        for food_dist in food_dists.iter() {
            score += max_dist.saturating_sub(*food_dist) as f32 / max_dist as f32;
        }

        score / 3.0
    }

    pub fn alive_enemies(state: &State, snake_id: usize) -> f32 {
        let num_of_other_snakes = state.snakes.len() - 1;
        let num_other_alive = state
            .snakes
            .iter()
            .enumerate()
            .filter(|(i, s)| *i != snake_id && s.is_alive())
            .count();

        if num_of_other_snakes == 0 {
            return 1.0;
        }

        1.0 - (num_other_alive as f32 / num_of_other_snakes as f32)
    }

    pub fn central(state: &State, snake_id: usize) -> f32 {
        let dist_to_center = coord!(state.grid.width as i32 / 2, state.grid.height as i32 / 2)
            .manhattan_dist(&state.snakes[snake_id].head());

        if dist_to_center == 0 {
            1.0
        } else {
            1.0 / dist_to_center as f32
        }
    }
}
