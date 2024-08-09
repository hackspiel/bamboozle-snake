use crate::game::{Battlesnake, Coord, Direction};
use crate::simulation::outcome::LossType;

#[derive(Debug, Clone)]
pub struct Snake {
    pub health: i16,
    pub body: Vec<Coord>,
    pub last_action: Direction,
    pub should_simulate: bool,
    pub loss_reason: LossType,
}

impl From<&Battlesnake> for Snake {
    fn from(snake: &Battlesnake) -> Self {
        Self {
            health: snake.health as i16,
            body: snake.body.clone(),
            last_action: Direction::None,
            should_simulate: true,
            loss_reason: LossType::None,
        }
    }
}

impl Snake {
    pub fn new(health: i16, body: Vec<Coord>, last_action: Direction) -> Self {
        Self {
            health,
            body,
            last_action,
            should_simulate: true,
            loss_reason: LossType::None,
        }
    }

    pub fn head(&self) -> Coord {
        self.body[0]
    }

    pub fn tail(&self) -> &Coord {
        self.body.last().unwrap()
    }

    pub fn is_alive(&self) -> bool {
        self.loss_reason == LossType::None
    }

    pub fn die(&mut self, reason: LossType) {
        self.loss_reason = reason;
    }

    pub fn eat(&mut self) {
        self.health = 100;
        self.body.push(*self.tail());
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    #[must_use]
    pub fn step(&self, action: Direction) -> Self {
        let new_head = self.head().step(action);
        let used_body_len = self.body.len() - 1; // drop tail
        let new_body = [&[new_head], &self.body[0..used_body_len]].concat();

        Self {
            health: self.health - 1,
            body: new_body,
            last_action: action,
            should_simulate: self.should_simulate,
            loss_reason: self.loss_reason,
        }
    }

    #[must_use]
    pub fn step_constrictor(&self, action: Direction) -> Self {
        let new_head = self.head().step(action);
        let new_body = [&[new_head], &self.body[..]].concat();

        Self {
            health: 100,
            body: new_body,
            last_action: action,
            should_simulate: self.should_simulate,
            loss_reason: self.loss_reason,
        }
    }
}
