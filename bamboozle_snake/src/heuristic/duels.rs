use crate::heuristic::floodfill::FloodType;
use crate::heuristic::{Floodfill, Heuristic, StandardHeuristic};
use crate::simulation::{Outcome, Snake, State};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DuelsHeuristic {
    pub area: f32,
    pub snake_area: f32,
    pub health: f32,
    pub length: f32,
    pub food: f32,
}

impl Default for DuelsHeuristic {
    fn default() -> Self {
        Self {
            area: 1.0,
            snake_area: 0.1,
            health: 0.05,
            length: 0.0,
            food: 0.0,
        }
    }
}

impl FromStr for DuelsHeuristic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl ToString for DuelsHeuristic {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Heuristic for DuelsHeuristic {
    fn eval(&self, state: &State) -> Outcome {
        if !state.snakes[0].is_alive() {
            return Outcome::Loss(state.snakes[0].loss_reason);
        }
        let floodfill = Floodfill::new(state, FloodType::FollowSnakes);

        let our_snake = &state.snakes[0];
        let (enemy_id, enemy_snake) = state
            .snakes
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, s)| s.is_alive())
            .unwrap();

        let health_score = self.health(our_snake);
        let length_score = self.length(our_snake, enemy_snake);
        let food_score = StandardHeuristic::food(state, &floodfill, 0);

        // area score
        let (our_cells, our_snake_cells, enemy_cells, enemy_snake_cells) = floodfill.count_duels();

        let our_cell_sum = our_cells as f32 + self.snake_area * our_snake_cells as f32;
        let enemy_cell_sum = enemy_cells as f32 + self.snake_area * enemy_snake_cells as f32;

        let mut area_score = our_cell_sum / (our_cell_sum + enemy_cell_sum);

        if floodfill.dead_ends[0] || floodfill.dead_ends[1] {
            let area_diff =
                (our_cells + our_snake_cells) as f32 - (enemy_cells + enemy_snake_cells) as f32;
            area_score += area_diff;
        }

        let score = self.health * health_score
            + self.area * area_score
            + self.length * length_score
            + self.food * food_score;

        Outcome::Heuristic(score)
    }

    fn eval_all(&self, state: &State) -> Vec<Outcome> {
        todo!()
    }
}

impl DuelsHeuristic {
    fn length(&self, our_snake: &Snake, enemy_snake: &Snake) -> f32 {
        our_snake.len() as f32 / (our_snake.len() as f32 + enemy_snake.len() as f32)
    }

    fn health(&self, our_snake: &Snake) -> f32 {
        if our_snake.health < 10 {
            return 0.0;
        }
        if our_snake.health < 20 {
            return 0.5;
        }
        1.0
    }
}
