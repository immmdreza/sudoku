use crate::SudokuBoard;

pub mod hidden_single;
pub mod naked_pair;
pub mod naked_single;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    NakedSingle,
    HiddenSingle,
    NakedPair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyMarker {
    strategy: Strategy,
    effect: StrategyEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyEffect {
    /// The source.
    Source,

    /// Effected by a [`StrategyMarker::Source`].
    Effected,
    // {
    //     /// The index of source block.
    //     index: BlockIndex,
    //     /// The source number (mostly in possibilities).
    //     number: Option<SudokuNumber>,
    // },
}

pub trait SudokuSolvingStrategy {
    const STRATEGY: Strategy;

    fn update_possible_numbers(&self, board: &mut SudokuBoard);
}
