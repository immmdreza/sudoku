use crate::{
    SudokuBlock, SudokuBoard,
    numbers::{SudokuNumber, SudokuNumbers},
    square_number,
    strategies::SudokuSolvingStrategy,
};

pub struct HiddenSingleStrategy;

impl SudokuSolvingStrategy for HiddenSingleStrategy {
    fn update_possible_numbers(&self, board: &mut crate::SudokuBoard) {
        use SudokuNumber::*;

        for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
                let mut hidden_number = None;

                if let Some(row_hidden) = get_hidden_single(&board, row, col, |b| b.get_row(row)) {
                    hidden_number = Some(row_hidden);
                } else if let Some(col_hidden) =
                    get_hidden_single(&board, row, col, |b| b.get_column(col))
                {
                    hidden_number = Some(col_hidden);
                } else if let Some(square_hidden) =
                    get_hidden_single(&board, row, col, |b| b.get_square(square_number(row, col)))
                {
                    hidden_number = Some(square_hidden);
                }

                if let Some(hidden) = hidden_number {
                    if let Some(possibilities) =
                        board.get_block_mut(row, col).status.as_possibilities_mut()
                    {
                        *possibilities = Default::default();
                        possibilities.set_number(hidden);
                    }

                    for possibilities in board
                        .get_row_mut(row)
                        .filter(|b| b.col != col)
                        .filter_map(|f| f.status.as_possibilities_mut())
                    {
                        possibilities.del_number(hidden);
                    }

                    for possibilities in board
                        .get_column_mut(col)
                        .filter(|b| b.row != row)
                        .filter_map(|f| f.status.as_possibilities_mut())
                    {
                        possibilities.del_number(hidden);
                    }

                    for possibilities in board
                        .get_square_mut(square_number(row, col))
                        .filter(|b| b.col != col && b.row != row)
                        .filter_map(|f| f.status.as_possibilities_mut())
                    {
                        possibilities.del_number(hidden);
                    }
                }
            }
        }
    }
}

pub fn get_hidden_single<'s, F, S>(
    board: &'s SudokuBoard,
    row: SudokuNumber,
    col: SudokuNumber,
    container: F,
) -> Option<SudokuNumber>
where
    F: FnOnce(&'s SudokuBoard) -> S,
    S: Iterator<Item = &'s SudokuBlock>,
{
    let block = board.get_block(row, col);
    let possibles = block.status.as_possibilities()?;
    // All in this row except this one.
    let row_pos = get_all_possible_numbers(
        container(board).filter(|x| !(x.col == block.col && x.row == block.row)),
    );

    let hidden = possibles
        .get_numbers()
        .filter(|f| !row_pos.has_number(*f))
        .collect::<Vec<_>>();
    if hidden.len() == 1 {
        Some(hidden[0])
    } else {
        None
    }
}

pub fn get_all_possible_numbers<'s>(
    iterator: impl Iterator<Item = &'s SudokuBlock>,
) -> SudokuNumbers {
    iterator.filter_map(|f| f.status.as_possibilities()).fold(
        SudokuNumbers::default(),
        |mut acc, fold| {
            for f in fold.get_numbers() {
                acc.set_number(f);
            }
            acc
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{SudokuBoard, numbers::SudokuNumber};

    use super::*;

    #[test]
    fn test_all_possible_numbers() {
        use SudokuNumber::*;

        let mut board = SudokuBoard::default();
        board.fill_board_u8(sudoku_samples::easy::FIRST).unwrap();
        board.update_possibilities();

        let pos = get_hidden_single(&board, Three, One, |f| f.get_column(One));
        println!("{:?}", pos)
    }
}
