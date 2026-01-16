use std::collections::HashMap;

use bevy::{
    ecs::{component::Component, entity::Entity},
    prelude::Deref,
};
use sudoku_solver::BlockIndex;

pub mod pancam;
pub mod plugins;

#[derive(Debug, Component, Deref)]
pub struct BlocksAccessInfo(HashMap<BlockIndex, Entity>);

impl BlocksAccessInfo {
    pub fn new(_0: HashMap<BlockIndex, Entity>) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Component)]
pub struct SquareIndex {
    i: usize,
    j: usize,
    master: Option<(usize, usize)>,
    actual_index: (usize, usize),
}

impl SquareIndex {
    pub fn new(i: usize, j: usize, master: Option<(usize, usize)>) -> Self {
        let mut s = Self {
            i,
            j,
            master,
            actual_index: Default::default(),
        };
        s.actual_index = s._actual_index();
        s
    }

    fn _actual_index(&self) -> (usize, usize) {
        if let Some(master) = self.master {
            (master.0 * 3 + self.i, master.1 * 3 + self.j)
        } else {
            (self.i, self.j)
        }
    }

    pub fn actual_index(&self) -> (usize, usize) {
        self.actual_index
    }

    pub fn block_index(&self) -> BlockIndex {
        let (col, row) = self.actual_index();
        BlockIndex::from_index(row, col).unwrap()
    }
}
