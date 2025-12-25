use crate::SudokuBoard;

pub mod hidden_single;
pub mod naked_single;

pub trait SudokuSolvingStrategy {
    fn update_possible_numbers(&self, board: &mut SudokuBoard);
}
