use crate::strategies::SudokuSolvingStrategy;

pub struct NakedSingleStrategy;

impl SudokuSolvingStrategy for NakedSingleStrategy {
    fn update_possible_numbers(&self, _board: &mut crate::SudokuBoard) {}
}
