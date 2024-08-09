use crate::game::Coord;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::iter::zip;
use yansi::Paint;

use crate::grid::Grid;
use crate::simulation::{Snake, State};

#[derive(PartialEq, Clone, Copy, Default)]
pub enum CellFlood {
    #[default]
    Free,
    Snake {
        id: u8,
        tail_dist: u8,
    },
    Owned {
        id: u8,
        length: u8,
        health: u8,
        was_snake: bool,
        step: u32,
    },
    Draw,
}

impl From<&FloodElement> for CellFlood {
    fn from(elem: &FloodElement) -> Self {
        CellFlood::Owned {
            id: elem.id,
            step: elem.step,
            length: elem.length,
            health: elem.health as _,
            was_snake: false,
        }
    }
}

impl Debug for CellFlood {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CellFlood::Free => write!(f, "O"),
            CellFlood::Draw => write!(f, "x"),
            CellFlood::Owned { id, .. } => write!(f, "{}", Paint::green(id + 1)),
            CellFlood::Snake { id, .. } => write!(f, "{}", Paint::black(id + 1)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloodElement {
    id: u8,
    pos: Coord,
    step: u32,
    health: i16,
    length: u8,
    food_eaten: u8,
}

impl FloodElement {
    #[must_use]
    pub fn get_neighbours(&self) -> [FloodElement; 4] {
        let neighbour_coords = self.pos.get_neighbours();

        neighbour_coords.map(|pos| Self {
            id: self.id,
            pos,
            step: self.step + 1,
            health: self.health - 1,
            length: self.length,
            food_eaten: self.food_eaten,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FloodType {
    Simple,
    FollowSnakes,
    Constrictor,
}

#[derive(Clone, Debug)]
pub struct Floodfill<'a> {
    pub state: &'a State,
    pub cells: Grid<CellFlood>,
    pub dead_ends: Vec<bool>,
}

impl<'a> Floodfill<'a> {
    pub fn new(state: &'a State, flood_type: FloodType) -> Self {
        let mut floodfill = Self {
            state,
            cells: Grid::new(state.grid.width, state.grid.height, state.grid.wrapped),
            dead_ends: vec![true; state.snakes.len()],
        };
        floodfill.fill_snakes(&state.snakes);

        match flood_type {
            FloodType::Simple => floodfill.calc_simple(),
            FloodType::FollowSnakes => floodfill.calc_follow_snakes(),
            FloodType::Constrictor => floodfill.calc_constrictor(),
        }

        floodfill
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Fill grid with snakes
    fn fill_snakes(&mut self, snakes: &[Snake]) {
        for (id, snake) in snakes.iter().enumerate().filter(|(_, s)| s.is_alive()) {
            for (tail_dist, body_part) in snake.body.iter().rev().enumerate() {
                self.cells[*body_part] = CellFlood::Snake {
                    id: id as u8,
                    tail_dist: tail_dist as u8,
                };
            }
        }
    }

    fn get_ordered_ids(&self, snakes: &Vec<Snake>) -> Vec<usize> {
        let mut ordered_ids: Vec<usize> = (0..snakes.len())
            .filter(|&i| snakes[i].is_alive())
            .collect();
        ordered_ids.sort_by(|&i, &j| snakes[j].len().cmp(&snakes[i].len()));

        ordered_ids
    }

    fn calc_simple(&mut self) {
        let snakes = &self.state.snakes;
        let ordered_ids = self.get_ordered_ids(snakes);

        let mut elem_queue =
            VecDeque::with_capacity(self.cells.width * self.cells.height * snakes.len());

        // init queue with heads
        for id in ordered_ids.iter() {
            let snake = &snakes[*id];
            elem_queue.push_back(FloodElement {
                id: *id as u8,
                pos: snake.head(),
                step: 0,
                health: snake.health,
                length: snake.len() as u8,
                food_eaten: 0,
            });
        }

        while let Some(elem) = elem_queue.pop_front() {
            // check if pos ownership was reverted (draw)
            if self.cells[elem.pos] == CellFlood::Draw {
                continue;
            }

            // iterate over cell neighbours
            for neighbour in elem.get_neighbours().iter() {
                if !self.cells.contains(neighbour.pos) || self.state.grid.is_snake(neighbour.pos) {
                    continue;
                }

                let cell = &mut self.cells[neighbour.pos];

                match cell {
                    CellFlood::Free => {
                        *cell = neighbour.into();
                        elem_queue.push_back(*neighbour);
                    }
                    CellFlood::Owned {
                        id, step, length, ..
                    } => {
                        if *id != neighbour.id
                            && *step == neighbour.step
                            && *length == neighbour.length
                        {
                            *cell = CellFlood::Draw;
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn calc_follow_snakes(&mut self) {
        // prepare board with snakes
        let snakes = &self.state.snakes;

        // order snake-ids by length
        let ordered_ids = self.get_ordered_ids(snakes);

        let mut elem_queue =
            VecDeque::with_capacity(self.cells.width * self.cells.height * snakes.len());

        // init queue with heads
        for id in ordered_ids.iter() {
            let snake = &snakes[*id];
            elem_queue.push_back(FloodElement {
                id: *id as u8,
                step: 0,
                pos: snake.head(),
                health: snake.health,
                length: snake.len() as u8,
                food_eaten: 0,
            });
        }

        while let Some(mut elem) = elem_queue.pop_front() {
            // check if pos ownership was reverted (eg. draw)
            if self.cells[elem.pos] == CellFlood::Draw {
                self.dead_ends[elem.id as usize] = false;
                continue;
            }

            if let CellFlood::Owned { id, step, .. } = self.cells[elem.pos] {
                debug_assert!(step <= elem.step);
                if id != elem.id {
                    self.dead_ends[elem.id as usize] = false;
                    continue;
                }
            }

            // check if eaten or starved
            if self.state.grid.is_food(elem.pos) {
                elem.health = 100;
                elem.length += 1;
                elem.food_eaten += 1;
            } else if elem.health <= 0 {
                continue;
            }

            // iterate over new positions
            for neighbour in elem.get_neighbours().iter_mut() {
                // continue if out of bounds
                if !self.cells.contains(neighbour.pos) {
                    continue;
                }

                neighbour.health -= self.state.grid[neighbour.pos].hazard as i16 * 14;

                let cell = &mut self.cells[neighbour.pos];

                match cell {
                    CellFlood::Free => {
                        *cell = CellFlood::from(&*neighbour);
                        elem_queue.push_back(*neighbour);
                    }

                    // same snake
                    CellFlood::Snake { id, tail_dist } if *id == neighbour.id => {
                        if ((*tail_dist + neighbour.food_eaten) as u32) < neighbour.step {
                            *cell = CellFlood::Owned {
                                id: neighbour.id,
                                step: neighbour.step,
                                length: neighbour.length,
                                health: neighbour.health as _,
                                was_snake: true,
                            };
                            elem_queue.push_back(*neighbour);
                        }
                    }
                    // enemy snake TODO: check if other snake ate (maybe not possible)
                    CellFlood::Snake { id, tail_dist } => {
                        if (*tail_dist as u32) < neighbour.step {
                            *cell = CellFlood::Owned {
                                id: neighbour.id,
                                step: neighbour.step,
                                length: neighbour.length,
                                health: neighbour.health as _,
                                was_snake: true,
                            };
                            elem_queue.push_back(*neighbour);
                        }
                    }

                    CellFlood::Owned {
                        id,
                        length,
                        was_snake,
                        step,
                        ..
                    } => {
                        assert!(*step <= neighbour.step);
                        // must be connected to another snakes area
                        if *id != elem.id {
                            self.dead_ends[elem.id as usize] = false;
                            // self.dead_ends[*id as usize] = false;
                        }

                        if *step == neighbour.step && *id != neighbour.id {
                            match neighbour.length.cmp(length) {
                                Ordering::Equal => *cell = CellFlood::Draw,
                                Ordering::Greater => {
                                    *cell = CellFlood::Owned {
                                        id: neighbour.id,
                                        length: neighbour.length,
                                        health: neighbour.health as _,
                                        was_snake: *was_snake,
                                        step: neighbour.step,
                                    };
                                    elem_queue.push_back(*neighbour);
                                }
                                Ordering::Less => (),
                            }
                        }
                    }

                    CellFlood::Draw => {
                        // must be connected to another snakes area
                        self.dead_ends[elem.id as usize] = false;
                    }
                }
            }
        }
    }

    fn calc_constrictor(&mut self) {
        // prepare board with snakes
        let snakes = &self.state.snakes;

        // order snake-ids by length
        let ordered_ids = self.get_ordered_ids(snakes);

        let mut elem_queue =
            VecDeque::with_capacity(self.cells.width * self.cells.height * snakes.len());

        // init queue with heads
        for id in ordered_ids.iter() {
            let snake = &snakes[*id];
            elem_queue.push_back(FloodElement {
                id: *id as u8,
                step: 0,
                pos: snake.head(),
                health: 100,
                length: snake.len() as u8,
                food_eaten: 0,
            });
        }

        while let Some(mut elem) = elem_queue.pop_front() {
            // check if pos ownership was reverted (eg. draw)
            if self.cells[elem.pos] == CellFlood::Draw {
                self.dead_ends[elem.id as usize] = false;
                continue;
            }

            if let CellFlood::Owned { id, step, .. } = self.cells[elem.pos] {
                debug_assert!(step <= elem.step);
                if id != elem.id {
                    self.dead_ends[elem.id as usize] = false;
                    continue;
                }
            }

            // iterate over new positions
            for neighbour in elem.get_neighbours().iter_mut() {
                // continue if out of bounds
                if !self.cells.contains(neighbour.pos) {
                    continue;
                }

                let cell = &mut self.cells[neighbour.pos];

                match cell {
                    CellFlood::Free => {
                        *cell = CellFlood::from(&*neighbour);
                        elem_queue.push_back(*neighbour);
                    }

                    // snake == do nothing
                    CellFlood::Snake { .. } => {}

                    CellFlood::Owned {
                        id,
                        length,
                        was_snake,
                        step,
                        ..
                    } => {
                        assert!(*step <= neighbour.step);
                        // must be connected to another snakes area
                        if *id != elem.id {
                            self.dead_ends[elem.id as usize] = false;
                            // self.dead_ends[*id as usize] = false;
                        }

                        if *id != neighbour.id {
                            *cell = CellFlood::Draw;
                        }
                    }

                    CellFlood::Draw => {
                        // must be connected to another snakes area
                        self.dead_ends[elem.id as usize] = false;
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn count_duels(&self) -> (usize, usize, usize, usize) {
        let mut our_cells = 0;
        let mut our_s_cells = 0;

        let mut e_cells = 0;
        let mut e_s_cells = 0;

        for cell in self.cells.cells.iter() {
            if let CellFlood::Owned { id, was_snake, .. } = cell {
                if *id == 0 {
                    if *was_snake {
                        our_s_cells += 1;
                    } else {
                        our_cells += 1;
                    }
                } else if *was_snake {
                    e_s_cells += 1;
                } else {
                    e_cells += 1;
                }
            }
        }

        (our_cells, our_s_cells, e_cells, e_s_cells)
    }

    #[must_use]
    pub fn count_owned(&self, snake_id: u8) -> (usize, usize) {
        let mut owned = 0;
        let mut owned_snake = 0;

        for cell in self.cells.cells.iter() {
            if let CellFlood::Owned { id, was_snake, .. } = cell {
                if *id == snake_id {
                    if *was_snake {
                        owned_snake += 1;
                    } else {
                        owned += 1;
                    }
                }
            }
        }

        (owned, owned_snake)
    }

    #[must_use]
    pub fn count_owned_all(&self) -> Vec<i32> {
        let mut owned = vec![0; self.state.snakes.len()];

        for cell in self.cells.cells.iter() {
            if let CellFlood::Owned { id, was_snake, .. } = cell {
                owned[*id as usize] += 1;
            }
        }
        owned
    }

    pub fn count_owned_royale(&self, snake_id: u8) -> (usize, usize, usize, usize) {
        let mut owned = 0;
        let mut owned_hazards = 0;
        let mut owned_snakes = 0;
        let mut owned_snake_hazards = 0;

        for (flood_cell, grid_cell) in zip(self.cells.cells.iter(), self.state.grid.cells.iter()) {
            if let CellFlood::Owned { id, was_snake, .. } = flood_cell {
                if *id != snake_id {
                    continue;
                }

                if *was_snake {
                    if grid_cell.hazard > 0 {
                        owned_snake_hazards += 1;
                    } else {
                        owned_snakes += 1;
                    }
                } else if grid_cell.hazard > 0 {
                    owned_hazards += 1;
                } else {
                    owned += 1;
                }
            }
        }

        (owned, owned_hazards, owned_snakes, owned_snake_hazards)
    }
}
