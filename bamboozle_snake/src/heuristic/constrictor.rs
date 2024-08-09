use crate::heuristic::floodfill::FloodType;
use crate::heuristic::{Floodfill, Heuristic, StandardHeuristic};
use crate::simulation::{Outcome, State};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConstrictorHeuristic {
    pub area: f32,
    pub alive_enemies: f32,
}

impl Default for ConstrictorHeuristic {
    fn default() -> Self {
        Self {
            area: 0.01,
            alive_enemies: 0.1,
        }
    }
}

impl FromStr for ConstrictorHeuristic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl ToString for ConstrictorHeuristic {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Heuristic for ConstrictorHeuristic {
    fn eval(&self, state: &State) -> Outcome {
        if !state.snakes[0].is_alive() {
            return Outcome::Loss(state.snakes[0].loss_reason);
        }


        let alive_enemies_score = StandardHeuristic::alive_enemies(state, 0);

        let floodfill = Floodfill::new(state, FloodType::Simple);
        let owned_areas = floodfill.count_owned_all();

        let mut max_enemy_area = -1;
        for i in 1..state.snakes.len() {
            if state.snakes[i].is_alive() && owned_areas[i] > max_enemy_area {
                max_enemy_area = owned_areas[i];
            }
        }
        
        let area_score = (owned_areas[0] - max_enemy_area) as f32;
        let score = self.alive_enemies * alive_enemies_score + self.area * area_score;
        Outcome::Heuristic(score)
    }

    fn eval_all(&self, state: &State) -> Vec<Outcome> {
        todo!()
    }
}