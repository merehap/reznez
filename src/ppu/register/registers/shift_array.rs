use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use crate::ppu::pixel_index::ColumnInTile;

pub struct ShiftArray<T, const N: usize>(VecDeque<T>);

impl <T: Copy + Default, const N: usize> ShiftArray<T, N> {
    pub fn new() -> ShiftArray<T, N> {
        ShiftArray(VecDeque::from_iter([Default::default(); N]))
    }

    pub fn shift_left(&mut self) {
        self.0.pop_front();
        self.0.push_back(Default::default());
    }

    pub fn push(&mut self, value: T) {
        self.0.pop_front();
        self.0.push_back(value);
    }
}

impl <T, const N: usize> Index<ColumnInTile> for ShiftArray<T, N> {
    type Output = T;

    // Indexes greater than 7 are intentionally inaccessible.
    fn index(&self, column_in_tile: ColumnInTile) -> &T {
        &self.0[column_in_tile as usize]
    }
}

impl <T, const N: usize> Index<usize> for ShiftArray<T, N> {
    type Output = T;

    // Indexes greater than 7 are intentionally inaccessible.
    fn index(&self, index: usize) -> &T {
        &self.0[index]
    }
}

impl <T, const N: usize> IndexMut<usize> for ShiftArray<T, N> {
    // Indexes greater than 7 are intentionally inaccessible.
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.0[index]
    }
}
