use std::fmt::Display;

pub mod components;
pub mod resources;

pub const DEFAULT_HELP_TEXT: &str = "Use 'Space' to update possible values, 'Enter' to resolve blocks, 'R' to reset, 'M' to change selection mode, 'C' to clear block, 1 to 9 to set number and 'H' to engage Hidden single strategy.";

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SudokuBoardDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Expert,
}

impl Display for SudokuBoardDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuBoardDifficulty::Easy => write!(f, "Easy"),
            SudokuBoardDifficulty::Normal => write!(f, "Normal"),
            SudokuBoardDifficulty::Hard => write!(f, "Hard"),
            SudokuBoardDifficulty::Expert => write!(f, "Expert"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardId {
    pub difficulty: Option<SudokuBoardDifficulty>,
    pub index: usize,
}

impl From<(Option<SudokuBoardDifficulty>, usize)> for BoardId {
    fn from((difficulty, index): (Option<SudokuBoardDifficulty>, usize)) -> Self {
        Self::new(difficulty, index)
    }
}

impl BoardId {
    pub fn new(difficulty: Option<SudokuBoardDifficulty>, index: usize) -> Self {
        Self { difficulty, index }
    }
}
