use std::fmt::{Display, Write as _};

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

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::HiddenSingle => f.write_char('H'),
            Strategy::NakedSingle => f.write_char('S'),
            Strategy::NakedPair => f.write_char('P'),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyMarker {
    strategy: Strategy,
    effect: StrategyEffect,
}

impl StrategyMarker {
    pub fn is_effected(&self) -> bool {
        self.effect.is_effected()
    }

    pub fn is_source(&self) -> bool {
        self.effect.is_source()
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }
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

impl StrategyEffect {
    /// Returns `true` if the strategy effect is [`Source`].
    ///
    /// [`Source`]: StrategyEffect::Source
    #[must_use]
    pub fn is_source(&self) -> bool {
        matches!(self, Self::Source)
    }

    /// Returns `true` if the strategy effect is [`Effected`].
    ///
    /// [`Effected`]: StrategyEffect::Effected
    #[must_use]
    pub fn is_effected(&self) -> bool {
        matches!(self, Self::Effected)
    }
}

pub trait SudokuSolvingStrategy {
    const STRATEGY: Strategy;

    fn update_possible_numbers(&self, board: &mut SudokuBoard, show_only_effect: bool);
}
