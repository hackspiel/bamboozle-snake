#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Outcome {
    // Possible outcomes of a game
    Loss(LossType),
    Draw,
    Heuristic(f32),
    Win(f32),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub enum LossType {
    // Different ways a snake can lose
    #[default]
    OwnOrWallCollision = 0,
    Starvation,
    SnakeCollision,
    HeadCollision,
    None,
}

impl Outcome {
    #[must_use]
    pub fn get_score(&self) -> f32 {
        match self {
            Outcome::Win(score) => 1_000_000.0 + score,
            Outcome::Loss(reason) => match reason {
                LossType::OwnOrWallCollision => -1_000_010.0,
                LossType::Starvation => -1_000_008.0,
                LossType::SnakeCollision => -1_000_006.0,
                LossType::HeadCollision => -1_000_004.0,
                LossType::None => todo!(),
            },
            Outcome::Draw => -1000.0,
            Outcome::Heuristic(score) => *score,
        }
    }
}
