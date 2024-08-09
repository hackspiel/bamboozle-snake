use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

use crate::game::Coord;

#[derive(Clone)]
pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    pub wrapped: bool,
    pub cells: Vec<T>,
}

impl<T: Default + Copy> Grid<T> {
    #[must_use]
    pub fn new(width: usize, height: usize, wrapped: bool) -> Self {
        Self {
            width,
            height,
            cells: vec![T::default(); width * height],
            wrapped,
        }
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = T::default();
        }
    }

    #[must_use]
    pub fn contains(&self, pos: Coord) -> bool {
        if self.wrapped {
            return true;
        }

        if pos.x < 0 || self.width as i32 <= pos.x {
            return false;
        }

        if pos.y < 0 || self.height as i32 <= pos.y {
            return false;
        }

        true
    }

    #[must_use]
    pub fn max_dist(&self) -> u32 {
        if self.wrapped {
            ((self.height + self.width) / 2) as u32
        } else {
            (self.height + self.width) as u32
        }
    }

    #[must_use]
    pub fn manhattan_dist(&self, pos1: &Coord, pos2: &Coord) -> u32 {
        if self.wrapped {
            let mut x_diff = (pos1.x - pos2.x).abs();
            let mut y_diff = (pos1.y - pos2.y).abs();

            if x_diff > self.width as i32 / 2 {
                x_diff = self.width as i32 - x_diff;
            }
            if y_diff > self.height as i32 / 2 {
                y_diff = self.height as i32 - y_diff;
            }

            (x_diff + y_diff) as u32
        } else {
            pos1.manhattan_dist(pos2)
        }
    }
}

impl<T> Grid<T> {
    pub fn wrap_around(&self, coord: &mut Coord) {
        let width = self.width as i32;
        let height = self.height as i32;

        if coord.x < 0 {
            coord.x += width;
        } else if coord.x >= width {
            coord.x -= width;
        }

        if coord.y < 0 {
            coord.y += height;
        } else if coord.y >= height {
            coord.y -= height;
        }
    }
}

impl<T> Index<Coord> for Grid<T> {
    type Output = T;
    #[inline]
    fn index(&self, mut coord: Coord) -> &Self::Output {
        if self.wrapped {
            coord.x = coord.x.rem_euclid(self.width as i32);
            coord.y = coord.y.rem_euclid(self.height as i32);
        }

        debug_assert!(0 <= coord.x && coord.x < self.width as i32);
        debug_assert!(0 <= coord.y && coord.y < self.height as i32);

        let index = coord.y as usize * self.width + coord.x as usize;

        &self.cells[index]

        // unsafe { self.cells.get_unchecked(index) }
    }
}

impl<T> IndexMut<Coord> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, mut coord: Coord) -> &mut Self::Output {
        if self.wrapped {
            coord.x = coord.x.rem_euclid(self.width as i32);
            coord.y = coord.y.rem_euclid(self.height as i32);
        }

        debug_assert!(0 <= coord.x && coord.x < self.width as i32);
        debug_assert!(0 <= coord.y && coord.y < self.height as i32);

        let index = coord.y as usize * self.width + coord.x as usize;

        &mut self.cells[index]

        // unsafe { self.cells.get_unchecked_mut(index) }
    }
}

impl<T: Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Grid {}x{} (wrapped={})",
            self.width, self.height, self.wrapped
        )?;

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{:?} ", self.cells[y * self.width + x])?;
            }
            writeln!(f, " ")?;
        }
        Ok(())
    }
}
