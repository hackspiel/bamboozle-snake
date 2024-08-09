use crate::game::{Coord, Direction, GameState};
use crate::grid::Grid;
use crate::simulation::outcome::LossType;
use crate::simulation::{CellGame, CellType, Mode, Snake};
use log::debug;
use std::cmp::Ordering;
use std::iter::zip;

const HAZARD_DAMAGE: i16 = 14;

#[derive(Debug, Clone)]
pub struct State {
    pub turn: u32,
    pub snakes: Vec<Snake>,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub grid: Grid<CellGame>,
    pub mode: Mode,
}

impl From<&GameState> for State {
    fn from(game_state: &GameState) -> Self {
        let board = &game_state.board;

        let mut snakes: Vec<Snake> = Vec::with_capacity(board.snakes.len());
        let you = &game_state.you;
        snakes.push(you.into());

        let mut filtered_snakes: Vec<Snake> = board
            .snakes
            .iter()
            .filter(|s| s.id != you.id)
            .map(Snake::from)
            .collect();

        snakes.append(&mut filtered_snakes);

        let wrapped = game_state.game.ruleset.name.contains("wrapped");
        let mode = State::determine_mode(game_state);

        debug!("State detected mode {:?}", mode);

        let grid = Grid::new(board.width as usize, board.height as usize, wrapped);

        let mut state = Self {
            turn: game_state.turn,
            snakes,
            food: board.food.clone(),
            hazards: board.hazards.clone(),
            grid,
            mode,
        };
        state.fill_grid();

        state
    }
}

impl State {
    #[must_use]
    pub fn new(
        turn: u32,
        snakes: Vec<Snake>,
        food: Vec<Coord>,
        hazards: Vec<Coord>,
        width: usize,
        height: usize,
        wrapped: bool,
        mode: Mode,
    ) -> Self {
        let grid = Grid::new(width, height, wrapped);

        Self {
            turn,
            snakes,
            food,
            hazards,
            grid,
            mode,
        }
    }

    pub fn determine_mode(game_state: &GameState) -> Mode {
        if game_state.game.ruleset.name == "constrictor" {
            Mode::Constrictor
        } else if game_state.game.map == "snail_mode" {
            Mode::Snail
        } else if game_state.game.map == "royale" {
            Mode::Royale
        } else if game_state.board.snakes.len() == 2 {
            Mode::Duels
        } else {
            Mode::Standard
        }
    }

    pub fn fill_grid(&mut self) {
        self.grid.fill(&self.snakes, &self.food, &self.hazards);
    }

    #[must_use]
    pub fn alive_snakes(&self) -> Vec<&Snake> {
        self.snakes.iter().filter(|s| s.is_alive()).collect()
    }

    #[must_use]
    pub fn is_end_state(&self) -> bool {
        self.snakes.iter().filter(|s| s.is_alive()).count() <= 1
    }

    #[must_use]
    pub fn get_winner(&self) -> i8 {
        debug_assert!(self.is_end_state());

        for (i, snake) in self.snakes.iter().enumerate() {
            if snake.is_alive() {
                return i as i8;
            }
        }
        -1
    }

    pub fn kill_starved(&mut self) {
        for snake in self.snakes.iter_mut() {
            if snake.health == 0 {
                snake.die(LossType::Starvation);
            }
        }
    }

    pub fn get_valid_actions(&self, snake_i: usize) -> Vec<Direction> {
        let snake = &self.snakes[snake_i];

        if !snake.should_simulate || !snake.is_alive() {
            return vec![Direction::None];
        }

        let head = snake.head();

        let mut valid_actions = Vec::with_capacity(4);

        if snake.last_action != Direction::None
            && self.grid.is_valid_pos(head.step(snake.last_action))
        {
            valid_actions.push(snake.last_action);
        }

        valid_actions.extend(
            Direction::get_alive_actions()
                .into_iter()
                .filter(|d| *d != snake.last_action && self.grid.is_valid_pos(head.step(*d))),
        );

        if valid_actions.is_empty() {
            valid_actions.push(Direction::Up)
        }

        valid_actions
    }

    fn check_head_collisions(&self, snakes: &mut Vec<Snake>) {
        for s1_i in 0..snakes.len() - 1 {
            if !snakes[s1_i].is_alive() {
                continue;
            }

            for s2_i in s1_i + 1..snakes.len() {
                if snakes[s2_i].is_alive() && snakes[s1_i].head() == snakes[s2_i].head() {
                    let s1_len = snakes[s1_i].len();
                    let s2_len = snakes[s2_i].len();

                    match s1_len.cmp(&s2_len) {
                        Ordering::Less => snakes[s1_i].die(LossType::HeadCollision),
                        Ordering::Greater => snakes[s2_i].die(LossType::HeadCollision),
                        Ordering::Equal => {
                            snakes[s1_i].die(LossType::HeadCollision);
                            snakes[s2_i].die(LossType::HeadCollision);
                        }
                    }
                }
            }
        }
    }

    fn check_collisions(&self, snakes: &mut Vec<Snake>) {
        for (i, snake) in snakes.iter_mut().enumerate() {
            // ignore not simulated and dead snakes
            if snake.body[0] == snake.body[1] || !snake.is_alive() {
                continue;
            }

            let head = snake.head();
            // if snake.is_alive() && (!self.grid.contains(head) || self.grid.is_snake(head)) {
            //     // probably not all collisions are own/ wall collisions but should not matter
            //     snake.die(LossType::OwnOrWallCollision);
            // }
            if !self.grid.contains(head)
                || matches!(self.grid[head].cell, CellType::Snake(si) if si == i as u8)
            {
                snake.die(LossType::OwnOrWallCollision);
            } else if self.grid.is_snake(head) {
                snake.die(LossType::SnakeCollision);
            }
        }
    }

    #[must_use]
    pub fn step(&self, actions: &Vec<Direction>) -> Self {
        debug_assert!(actions.len() == self.snakes.len());

        if self.mode == Mode::Constrictor {
            return self.step_constrictor(actions);
        }

        let mut new_snakes: Vec<Snake> = Vec::with_capacity(self.snakes.len());
        let mut new_food = self.food.clone();

        // create new moved snakes and apply actions
        for (snake, action) in zip(&self.snakes, actions) {
            if snake.is_alive() {
                new_snakes.push(snake.step(*action));
            } else {
                new_snakes.push(snake.clone());
            }
        }

        //check head collisions
        self.check_head_collisions(&mut new_snakes);

        // check collisions
        self.check_collisions(&mut new_snakes);

        // check hazards and food
        for snake in new_snakes.iter_mut().filter(|s| s.is_alive()) {
            snake.health -= HAZARD_DAMAGE * self.grid[snake.head()].hazard as i16;

            if self.grid.is_food(snake.head()) {
                snake.eat();

                // delete eaten food
                new_food.retain(|&f| f != snake.head());
            }
            if snake.health < 0 {
                snake.die(LossType::Starvation);
            }
        }

        let mut new_state = State::new(
            self.turn + 1,
            new_snakes,
            new_food,
            self.hazards.clone(),
            self.grid.width,
            self.grid.height,
            self.grid.wrapped,
            self.mode,
        );

        new_state.kill_starved();

        if new_state.mode == Mode::Snail {
            new_state.apply_snail_mode(self);
        } else {
            new_state.fill_grid();
        }

        new_state
    }

    fn step_constrictor(&self, actions: &Vec<Direction>) -> Self {
        let mut new_snakes: Vec<Snake> = Vec::with_capacity(self.snakes.len());

        for (snake, action) in zip(&self.snakes, actions) {
            if snake.is_alive() {
                new_snakes.push(snake.step_constrictor(*action));
            } else {
                new_snakes.push(snake.clone());
            }
        }

        //check head collisions
        self.check_head_collisions(&mut new_snakes);

        // check collisions
        self.check_collisions(&mut new_snakes);

        let mut new_state = State::new(
            self.turn + 1,
            new_snakes,
            Vec::new(),
            Vec::new(),
            self.grid.width,
            self.grid.height,
            self.grid.wrapped,
            self.mode,
        );

        new_state.fill_grid();
        new_state
    }

    fn apply_snail_mode(&mut self, old_state: &State) {

        // dont use the hazards vector in snail mode
        self.hazards.clear();
        self.fill_grid();

        // set hazards to reduced old value
        for (old_cell, new_cell) in zip(old_state.grid.cells.iter(), self.grid.cells.iter_mut()) {
            if old_cell.hazard > 1 {
                new_cell.hazard = old_cell.hazard.saturating_sub(1);
            }
        }

        // add new hazards
        for old_snake in old_state.snakes.iter().filter(|s| s.is_alive()) {

            // hazards spawn only if tail "disappeared" (snake did not eat food)
            if *old_snake.tail() != old_snake.body[old_snake.body.len() - 2] {
                let cell = &mut self.grid[*old_snake.tail()];
                if !matches!(cell.cell, CellType::Snake(_)) {
                    cell.hazard = old_snake.len() as u8;
                }
            }
        }
    }
}
