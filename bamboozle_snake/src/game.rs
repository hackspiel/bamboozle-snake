use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[macro_export]
macro_rules! coord {
    ($x:expr, $y:expr) => {
        Coord { x: $x, y: $y }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum Gamemode {
    Standard,
    Duels,
    Royale,
    Constrictor,
}

#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
    pub mode: Gamemode,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub timeout: u32,
    pub map: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Ruleset {
    pub name: String,
    // pub version: String,
    // pub settings: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Board {
    pub height: u32,
    pub width: u32,
    pub food: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
    pub hazards: Vec<Coord>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub body: Vec<Coord>,
    pub head: Coord,
    // pub length: u32,
    pub latency: Option<String>,
    pub shout: Option<String>,
}

impl PartialEq for Battlesnake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Battlesnake,
}

#[derive(Serialize, Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
    None,
}

impl ToString for Direction {
    fn to_string(&self) -> String {
        match self {
            Direction::Up => String::from("up"),
            Direction::Right => String::from("right"),
            Direction::Down => String::from("down"),
            Direction::Left => String::from("left"),
            Direction::None => String::from("none"),
        }
    }
}

impl Direction {
    #[inline]
    pub fn get_alive_actions() -> [Direction; 4] {
        [
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::Up,
        ]
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy, Default)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Coord {
    #[inline]
    pub fn get_neighbours(&self) -> [Coord; 4] {
        [
            *self + Direction::Down.into(),
            *self + Direction::Left.into(),
            *self + Direction::Right.into(),
            *self + Direction::Up.into(),
        ]
    }

    #[must_use]
    pub fn step(&self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Self {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
            Direction::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Direction::None => *self,
        }
    }
    #[must_use]
    pub fn manhattan_dist(&self, other: &Coord) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }
}

impl From<Direction> for Coord {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => coord!(0, 1),
            Direction::Right => coord!(1, 0),
            Direction::Down => coord!(0, -1),
            Direction::Left => coord!(-1, 0),
            Direction::None => coord!(0, 0),
        }
    }
}
