use std::str::FromStr;
use crate::heuristic::floodfill::FloodType;
use crate::heuristic::{Floodfill, Heuristic, StandardHeuristic};
use crate::simulation::{Outcome, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoyaleHeuristic {
    pub area: f32,
    pub health: f32,
    pub length: f32,
    pub food: f32,
    pub alive_enemies: f32,
    pub central: f32,
}

impl Default for RoyaleHeuristic {
    fn default() -> Self {
        Self {
            area: 3.5,
            health: 2.0,
            length: 2.0,
            food: 1.0,
            alive_enemies: 4.0,
            central: 0.25,
        }
    }
}

impl FromStr for RoyaleHeuristic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl ToString for RoyaleHeuristic {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Heuristic for RoyaleHeuristic {
    fn eval(&self, state: &State) -> Outcome {
        if !state.snakes[0].is_alive() {
            return Outcome::Loss(state.snakes[0].loss_reason);
        }

        let floodmap = Floodfill::new(state, FloodType::FollowSnakes);

        Outcome::Heuristic(self.calc_score(state, &floodmap))
    }

    fn eval_all(&self, state: &State) -> Vec<Outcome> {
        todo!()
    }
}

impl RoyaleHeuristic {
    pub fn calc_score(&self, state: &State, floodmap: &Floodfill) -> f32 {
        let (own_cells, own_area_score) = self.area(floodmap, 0.4, 0.4, 0);
        let (e_cells, e_area_score) = self.area(floodmap, 0.4, 0.4, 1);

        let mut area_score = own_area_score / (e_area_score + own_area_score);

        if floodmap.dead_ends[0] || floodmap.dead_ends[1] {
            let area_diff = own_cells - e_cells;
            area_score += area_diff;
        }

        let health_score = StandardHeuristic::health(state, 0);
        let length_score = StandardHeuristic::length(state, 0);
        let food_score = StandardHeuristic::food(state, floodmap, 0);
        let alive_enemies_score = StandardHeuristic::alive_enemies(state, 0);
        let central_score = StandardHeuristic::central(state, 0);

        self.area * area_score
            + self.health * health_score
            + self.length * length_score
            + self.food * food_score
            + self.alive_enemies * alive_enemies_score
            + self.central * central_score
    }

    pub fn area(
        &self,
        floodmap: &Floodfill,
        snake_discount: f32,
        hazard_discount: f32,
        snake_id: usize,
    ) -> (f32, f32) {
        let (owned, owned_hazards, owned_snakes, owned_snake_hazards) =
            floodmap.count_owned_royale(snake_id as u8);

        let cells_sum = owned + owned_hazards + owned_snakes;

        let owned_hazards = owned_hazards as f32 * hazard_discount;
        let owned_snake_hazards = owned_snake_hazards as f32 * snake_discount * hazard_discount;

        let owned_snakes = owned_snakes as f32 * snake_discount;

        (
            cells_sum as f32,
            owned as f32 + owned_hazards + owned_snakes + owned_snake_hazards,
        )
    }
}
