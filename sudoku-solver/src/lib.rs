use crate::{
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::SudokuSolvingStrategy,
};

pub mod numbers;
pub mod strategies;

#[derive(Clone, Debug)]
pub struct SudokuBlock {
    row: SudokuNumber,
    col: SudokuNumber,
    pub status: SudokuBlockStatus,
}

impl SudokuBlock {
    pub fn new(row: SudokuNumber, col: SudokuNumber, status: SudokuBlockStatus) -> Self {
        Self { status, row, col }
    }

    pub fn is_fixed(&self) -> bool {
        self.status.is_fixed()
    }

    pub fn is_possibilities(&self) -> bool {
        self.status.is_possibilities()
    }

    pub fn is_resolved(&self) -> bool {
        self.status.is_resolved()
    }

    pub fn is_unresolved(&self) -> bool {
        self.status.is_unresolved()
    }

    pub fn row(&self) -> SudokuNumber {
        self.row
    }

    pub fn col(&self) -> SudokuNumber {
        self.col
    }

    pub fn square_number(&self) -> SudokuNumber {
        square_number(self.row, self.col)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SudokuBlockStatus {
    #[default]
    Unresolved,
    Fixed(SudokuNumber),
    Resolved(SudokuNumber),
    Possibilities(SudokuNumbers),
}

impl SudokuBlockStatus {
    /// Returns `true` if the sudoku block status is [`Unresolved`].
    ///
    /// [`Unresolved`]: SudokuBlockStatus::Unresolved
    #[must_use]
    pub fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved)
    }

    /// Returns `true` if the sudoku block status is [`Fixed`].
    ///
    /// [`Fixed`]: SudokuBlockStatus::Fixed
    #[must_use]
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(..))
    }

    /// Returns `true` if the sudoku block status is [`Resolved`].
    ///
    /// [`Resolved`]: SudokuBlockStatus::Resolved
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(..))
    }

    /// Returns `true` if the sudoku block status is [`Possibilities`].
    ///
    /// [`Possibilities`]: SudokuBlockStatus::Possibilities
    #[must_use]
    pub fn is_possibilities(&self) -> bool {
        matches!(self, Self::Possibilities(..))
    }

    pub fn as_possibilities(&self) -> Option<&SudokuNumbers> {
        if let Self::Possibilities(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_possibilities_mut(&mut self) -> Option<&mut SudokuNumbers> {
        if let Self::Possibilities(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_resolved(&self) -> Option<&SudokuNumber> {
        if let Self::Resolved(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_fixed(&self) -> Option<&SudokuNumber> {
        if let Self::Fixed(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct SudokuBoard {
    blocks: [[SudokuBlock; 9]; 9],
}

impl Default for SudokuBoard {
    fn default() -> Self {
        use SudokuNumber::*;

        let blocks = [One, Two, Three, Four, Five, Six, Seven, Eight, Nine]
            .iter()
            .map(|row| {
                [One, Two, Three, Four, Five, Six, Seven, Eight, Nine]
                    .iter()
                    .map(|col| SudokuBlock::new(*row, *col, Default::default()))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self { blocks }
    }
}

impl SudokuBoard {
    /// Fills the board with given numbers or empty blocks.
    fn fill_board(&mut self, numbers: [[Option<SudokuNumber>; 9]; 9]) {
        for (row_index, row) in numbers.iter().enumerate() {
            for (col_index, &number_option) in row.iter().enumerate() {
                let row = (row_index + 1).try_into().unwrap();
                let col = (col_index + 1).try_into().unwrap();

                self.blocks[row_index][col_index] = match number_option {
                    Some(number) => SudokuBlock::new(row, col, SudokuBlockStatus::Fixed(number)),
                    None => SudokuBlock::new(row, col, SudokuBlockStatus::Unresolved),
                };
            }
        }
    }

    pub fn fill_board_u8(&mut self, numbers: [[Option<u8>; 9]; 9]) -> Result<(), ()> {
        for (row_index, row) in numbers.iter().enumerate() {
            for (col_index, &number_option) in row.iter().enumerate() {
                let row = (row_index + 1).try_into().unwrap();
                let col = (col_index + 1).try_into().unwrap();

                self.blocks[row_index][col_index] = match number_option {
                    Some(number) => SudokuBlock::new(
                        row,
                        col,
                        SudokuBlockStatus::Fixed((number as usize).try_into()?),
                    ),
                    None => SudokuBlock::new(row, col, SudokuBlockStatus::Unresolved),
                };
            }
        }

        Ok(())
    }

    pub fn new(numbers: [[Option<SudokuNumber>; 9]; 9]) -> Self {
        let mut board = Self::default();
        board.fill_board(numbers);
        board
    }

    pub fn get_block(&self, row: SudokuNumber, col: SudokuNumber) -> &SudokuBlock {
        &self.blocks[row.to_index()][col.to_index()]
    }

    pub fn get_block_mut(&mut self, row: SudokuNumber, col: SudokuNumber) -> &mut SudokuBlock {
        &mut self.blocks[row.to_index()][col.to_index()]
    }

    pub fn get_blocks(&self) -> impl Iterator<Item = &SudokuBlock> {
        self.blocks.iter().flatten()
    }

    pub fn get_blocks_mut(&mut self) -> impl Iterator<Item = &mut SudokuBlock> {
        self.blocks.iter_mut().flatten()
    }

    pub fn get_row(&self, row_number: SudokuNumber) -> impl Iterator<Item = &SudokuBlock> {
        self.blocks[row_number.to_index()].iter()
    }

    pub fn get_row_mut(
        &mut self,
        row_number: SudokuNumber,
    ) -> impl Iterator<Item = &mut SudokuBlock> {
        self.blocks[row_number.to_index()].iter_mut()
    }

    pub fn get_column(&self, column_number: SudokuNumber) -> impl Iterator<Item = &SudokuBlock> {
        self.blocks
            .iter()
            .map(move |row| &row[column_number.to_index()])
    }

    pub fn get_column_mut(
        &mut self,
        column_number: SudokuNumber,
    ) -> impl Iterator<Item = &mut SudokuBlock> {
        self.blocks
            .iter_mut()
            .map(move |row| &mut row[column_number.to_index()])
    }

    pub fn get_square(&self, square_number: SudokuNumber) -> impl Iterator<Item = &SudokuBlock> {
        let start_row = (square_number.to_index() / 3) * 3;
        let start_col = (square_number.to_index() % 3) * 3;
        self.blocks[start_row..start_row + 3]
            .iter()
            .flat_map(move |row| &row[start_col..start_col + 3])
    }

    pub fn get_square_mut(
        &mut self,
        square_number: SudokuNumber,
    ) -> impl Iterator<Item = &mut SudokuBlock> {
        let start_row = (square_number.to_index() / 3) * 3;
        let start_col = (square_number.to_index() % 3) * 3;
        self.blocks[start_row..start_row + 3]
            .iter_mut()
            .flat_map(move |row| &mut row[start_col..start_col + 3])
    }

    pub fn get_block_possible_numbers(
        &self,
        row: SudokuNumber,
        col: SudokuNumber,
    ) -> SudokuNumbers {
        let mut possible_numbers = SudokuNumbers::new_all();

        for row_n in get_numbers(self.get_row(row)).get_numbers() {
            possible_numbers.del_number(row_n);
        }

        for col_n in get_numbers(self.get_column(col)).get_numbers() {
            possible_numbers.del_number(col_n);
        }

        for square_n in get_numbers(self.get_square(square_number(row, col))).get_numbers() {
            possible_numbers.del_number(square_n);
        }

        possible_numbers
    }

    pub fn update_possibilities(&mut self) {
        use SudokuNumber::*;
        for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
                if let SudokuBlockStatus::Unresolved | SudokuBlockStatus::Possibilities(_) =
                    self.get_block_mut(row, col).status
                {
                    let possibles = self.get_block_possible_numbers(row, col);
                    let block = self.get_block_mut(row, col);
                    block.status = SudokuBlockStatus::Possibilities(possibles);
                }
            }
        }
    }

    pub fn engage_strategy<S>(&mut self, strategy: S)
    where
        S: SudokuSolvingStrategy,
    {
        strategy.update_possible_numbers(self);
    }

    pub fn resolve_satisfied_blocks(&mut self) {
        for block in self.get_blocks_mut().filter(|f| f.is_possibilities()) {
            let possibles = block.status.as_possibilities().unwrap();
            if possibles.count_numbers() == 1 {
                let single_naked = possibles.get_numbers().next().unwrap();
                block.status = SudokuBlockStatus::Resolved(single_naked);
            }
        }

        self.update_possibilities();
    }

    pub fn reset(&mut self) {
        for block in self.get_blocks_mut().filter(|f| !f.is_fixed()) {
            block.status = SudokuBlockStatus::Unresolved;
        }
    }
}

pub fn get_numbers<'s>(iterator: impl Iterator<Item = &'s SudokuBlock>) -> SudokuNumbers {
    SudokuNumbers::new(iterator.filter_map(|f| match f.status {
        SudokuBlockStatus::Fixed(sudoku_number) | SudokuBlockStatus::Resolved(sudoku_number) => {
            Some(sudoku_number)
        }
        _ => None,
    }))
}

pub fn get_missing_numbers<'s>(iterator: impl Iterator<Item = &'s SudokuBlock>) -> SudokuNumbers {
    SudokuNumbers::new(get_numbers(iterator).get_missing_numbers())
}

pub fn square_number(row: SudokuNumber, col: SudokuNumber) -> SudokuNumber {
    (((row.to_index() / 3) * 3 + (col.to_index() / 3)) + 1)
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possibles() {
        use SudokuNumber::*;

        let mut board = SudokuBoard::default();
        board.fill_board_u8(sudoku_samples::easy::FIRST).unwrap();

        let numbers = board.get_block_possible_numbers(One, One);
        println!("{:?}", numbers.get_numbers().collect::<Vec<_>>());
    }
}
