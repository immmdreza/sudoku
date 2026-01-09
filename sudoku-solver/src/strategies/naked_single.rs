use crate::strategies::SudokuSolvingStrategy;

pub struct NakedSingleStrategy;

impl SudokuSolvingStrategy for NakedSingleStrategy {
    const STRATEGY: super::Strategy = super::Strategy::NakedSingle;

    fn update_possible_numbers(&self, _board: &mut crate::SudokuBoard) {}
}
