use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};
use sudoku_solver::SudokuBoard;

use crate::shared::{BoardId, SudokuBoardDifficulty};

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct ActiveBoardChanged(pub bool);

#[derive(Debug, Resource, Default, PartialEq, Eq, Deref, DerefMut)]
pub struct SudokuBoardResources {
    boards: HashMap<Option<SudokuBoardDifficulty>, Vec<SudokuBoard>>,
}

impl SudokuBoardResources {
    pub fn active_board(&self, active_board: &BoardId) -> &SudokuBoard {
        self.boards
            .get(&active_board.difficulty)
            .unwrap()
            .get(active_board.index)
            .unwrap()
    }

    pub fn active_board_mut(&mut self, active_board: &BoardId) -> &mut SudokuBoard {
        self.boards
            .get_mut(&active_board.difficulty)
            .unwrap()
            .get_mut(active_board.index)
            .unwrap()
    }
}

#[derive(Debug, Resource, Default, PartialEq, Eq, Deref, DerefMut)]
pub struct SudokuBoardSnapshotResources {
    boards: HashMap<Option<SudokuBoardDifficulty>, Vec<SudokuBoard>>,
}

impl SudokuBoardSnapshotResources {
    pub fn active_board(&self, active_board: &BoardId) -> &SudokuBoard {
        self.boards
            .get(&active_board.difficulty)
            .unwrap()
            .get(active_board.index)
            .unwrap()
    }

    pub fn active_board_mut(&mut self, active_board: &BoardId) -> &mut SudokuBoard {
        self.boards
            .get_mut(&active_board.difficulty)
            .unwrap()
            .get_mut(active_board.index)
            .unwrap()
    }
}

#[derive(Debug, Clone, Resource, Default, Deref, DerefMut)]
pub struct ActiveBoardsMapping(HashMap<Entity, BoardId>);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct ActiveBoardVisual(pub Entity);

#[derive(Debug, SystemParam)]
pub struct ActiveBoardProvider<'w> {
    pub active_visual: If<Res<'w, ActiveBoardVisual>>,
    pub boards_mapping: If<Res<'w, ActiveBoardsMapping>>,
}

impl<'w> ActiveBoardProvider<'w> {
    pub fn active_board(&self) -> Option<&BoardId> {
        self.boards_mapping.0.get(&self.active_visual)
    }
}

#[derive(Debug, SystemParam)]
pub struct ActiveBoardProviderMut<'w> {
    pub active_visual: If<Res<'w, ActiveBoardVisual>>,
    pub boards_mapping: If<ResMut<'w, ActiveBoardsMapping>>,
}

#[allow(dead_code)]
impl<'w> ActiveBoardProviderMut<'w> {
    pub fn active_board(&self) -> Option<&BoardId> {
        self.boards_mapping.0.get(&self.active_visual)
    }

    pub fn active_board_mut(&mut self) -> Option<&mut BoardId> {
        self.boards_mapping.0.get_mut(&self.active_visual)
    }

    pub fn update_active_board(&mut self, id: BoardId) -> Option<BoardId> {
        self.boards_mapping.0.insert(***self.active_visual, id)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum BoardPlayingState {
    #[default]
    Playing,
    FinishedVerified,
}

impl BoardPlayingState {
    /// Returns `true` if the board state is [`Playing`].
    ///
    /// [`Playing`]: BoardState::Playing
    #[must_use]
    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns `true` if the board state is [`FinishedVerified`].
    ///
    /// [`FinishedVerified`]: BoardState::FinishedVerified
    #[must_use]
    pub fn is_finished_verified(&self) -> bool {
        matches!(self, Self::FinishedVerified)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    /// Mistakes while resolving a block number
    pub mistakes: u32,

    /// Mistakes while marking a number as possible in a block
    pub possibility_mistakes: u32,
}

#[derive(Debug, Default, Clone)]
pub struct BoardState {
    pub stats: Stats,
    pub playing_state: BoardPlayingState,
}

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
pub struct BoardsStateMap {
    boards: HashMap<BoardId, BoardState>,
}
