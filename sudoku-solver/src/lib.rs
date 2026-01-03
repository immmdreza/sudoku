use std::collections::HashMap;

use crate::{
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::SudokuSolvingStrategy,
};

pub mod numbers;
pub mod strategies;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockIndex {
    row: SudokuNumber,
    col: SudokuNumber,
}

impl BlockIndex {
    pub fn new(row: SudokuNumber, col: SudokuNumber) -> Self {
        Self { row, col }
    }

    pub fn from_index(row: usize, col: usize) -> Result<Self, ()> {
        Ok(Self::new((row + 1).try_into()?, (col + 1).try_into()?))
    }

    pub fn actual_indexes(&self) -> (usize, usize) {
        (self.row.to_index(), self.col.to_index())
    }

    pub fn indexes(&self) -> (SudokuNumber, SudokuNumber) {
        (self.row, self.col)
    }

    pub fn square_number(&self) -> SudokuNumber {
        square_number(self.row, self.col)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conflicting {
    AffectedBy(BlockIndex),
    AffectedByPossibilities {
        block_index: BlockIndex,
        number: SudokuNumber,
    },
    Source,
}

impl Conflicting {
    /// Returns `true` if the conflicting is [`AffectedBy`].
    ///
    /// [`AffectedBy`]: Conflicting::AffectedBy
    #[must_use]
    pub fn is_affected_by(&self) -> bool {
        matches!(self, Self::AffectedBy(..))
    }

    /// Returns `true` if the conflicting is [`AffectedBy`].
    ///
    /// [`AffectedBy`]: Conflicting::AffectedBy
    #[must_use]
    pub fn is_affected_by_and(&self, f: impl FnOnce(&BlockIndex) -> bool) -> bool {
        match self {
            Conflicting::AffectedBy(block_index) => f(block_index),
            _ => false,
        }
    }

    /// Returns `true` if the conflicting is [`AffectedByPossibilities`].
    ///
    /// [`AffectedByPossibilities`]: Conflicting::AffectedByPossibilities
    #[must_use]
    pub fn is_affected_by_possibilities(&self) -> bool {
        matches!(self, Self::AffectedByPossibilities { .. })
    }

    /// Returns `true` if the conflicting is [`AffectedByPossibilities`].
    ///
    /// [`AffectedByPossibilities`]: Conflicting::AffectedByPossibilities
    #[must_use]
    pub fn is_affected_by_possibilities_and(
        &self,
        f: impl FnOnce(&BlockIndex, &SudokuNumber) -> bool,
    ) -> bool {
        match self {
            Conflicting::AffectedByPossibilities {
                block_index,
                number,
            } => f(block_index, number),
            _ => false,
        }
    }

    /// Returns `true` if the conflicting is [`Source`].
    ///
    /// [`Source`]: Conflicting::Source
    #[must_use]
    pub fn is_source(&self) -> bool {
        matches!(self, Self::Source)
    }
}

#[derive(Clone, Debug)]
pub struct SudokuBlock {
    index: BlockIndex,
    pub conflicting: Option<Conflicting>,
    pub status: SudokuBlockStatus,
}

impl SudokuBlock {
    pub fn new(row: SudokuNumber, col: SudokuNumber, status: SudokuBlockStatus) -> Self {
        Self {
            status,
            index: BlockIndex::new(row, col),
            conflicting: None,
        }
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
        self.index.row
    }

    pub fn col(&self) -> SudokuNumber {
        self.index.col
    }

    pub fn square_number(&self) -> SudokuNumber {
        self.index.square_number()
    }

    pub fn index(&self) -> &BlockIndex {
        &self.index
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Possibilities {
    pub numbers: SudokuNumbers,
    conflicting_numbers: SudokuNumbers,
}

impl Possibilities {
    pub fn new(numbers: SudokuNumbers) -> Self {
        Self {
            numbers,
            conflicting_numbers: Default::default(),
        }
    }

    pub fn is_conflicting(&self, number: SudokuNumber) -> bool {
        self.numbers.has_number(number) && self.conflicting_numbers.has_number(number)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SudokuBlockStatus {
    #[default]
    Unresolved,
    Fixed(SudokuNumber),
    Resolved(SudokuNumber),
    Possibilities(Possibilities),
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

    pub fn as_possibilities(&self) -> Option<&Possibilities> {
        if let Self::Possibilities(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_possibilities_mut(&mut self) -> Option<&mut Possibilities> {
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

#[derive(Debug, Clone)]
/// Refers to a row, column or an square in a sudoku board (Typically a sequence of blocks).
pub struct SudokuContainer<T, Item>
where
    T: Iterator<Item = Item>,
{
    blocks: T,
}

impl<T, Item> SudokuContainer<T, Item>
where
    T: Iterator<Item = Item>,
{
    pub fn new(blocks: T) -> Self {
        Self { blocks }
    }
}

impl<'b, T> SudokuContainer<T, &'b SudokuBlock>
where
    T: Iterator<Item = &'b SudokuBlock>,
{
    pub fn get_numbers(self) -> SudokuNumbers {
        get_numbers(self)
    }
}

impl<T, Item> Iterator for SudokuContainer<T, Item>
where
    T: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.blocks.next()
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
                let index = BlockIndex::from_index(row_index, col_index).unwrap();
                let (row, col) = index.indexes();

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
                let index = BlockIndex::from_index(row_index, col_index).unwrap();
                let (row, col) = index.indexes();

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

    pub fn get_block(&self, index: &BlockIndex) -> &SudokuBlock {
        let (row, col) = index.actual_indexes();
        &self.blocks[row][col]
    }

    pub fn get_block_mut(&mut self, index: &BlockIndex) -> &mut SudokuBlock {
        let (row, col) = index.actual_indexes();
        &mut self.blocks[row][col]
    }

    pub fn get_blocks(
        &self,
    ) -> SudokuContainer<std::iter::Flatten<std::slice::Iter<'_, [SudokuBlock; 9]>>, &SudokuBlock>
    {
        SudokuContainer::new(self.blocks.iter().flatten())
    }

    pub fn get_blocks_mut(
        &mut self,
    ) -> SudokuContainer<
        std::iter::Flatten<std::slice::IterMut<'_, [SudokuBlock; 9]>>,
        &mut SudokuBlock,
    > {
        SudokuContainer::new(self.blocks.iter_mut().flatten())
    }

    pub fn get_row(
        &self,
        row_number: SudokuNumber,
    ) -> SudokuContainer<std::slice::Iter<'_, SudokuBlock>, &SudokuBlock> {
        SudokuContainer::new(self.blocks[row_number.to_index()].iter())
    }

    pub fn get_row_mut(
        &mut self,
        row_number: SudokuNumber,
    ) -> SudokuContainer<std::slice::IterMut<'_, SudokuBlock>, &mut SudokuBlock> {
        SudokuContainer::new(self.blocks[row_number.to_index()].iter_mut())
    }

    pub fn get_col<'b>(
        &'b self,
        column_number: SudokuNumber,
    ) -> SudokuContainer<
        std::iter::Map<
            std::slice::Iter<'b, [SudokuBlock; 9]>,
            impl FnMut(&'b [SudokuBlock; 9]) -> &'b SudokuBlock + 'b,
        >,
        &'b SudokuBlock,
    > {
        SudokuContainer::new(
            self.blocks
                .iter()
                .map(move |row| &row[column_number.to_index()]),
        )
    }

    pub fn get_col_mut<'b>(
        &'b mut self,
        column_number: SudokuNumber,
    ) -> SudokuContainer<
        std::iter::Map<
            std::slice::IterMut<'b, [SudokuBlock; 9]>,
            impl FnMut(&'b mut [SudokuBlock; 9]) -> &'b mut SudokuBlock,
        >,
        &'b mut SudokuBlock,
    > {
        SudokuContainer::new(
            self.blocks
                .iter_mut()
                .map(move |row| &mut row[column_number.to_index()]),
        )
    }

    pub fn get_square<'b>(
        &'b self,
        square_number: SudokuNumber,
    ) -> SudokuContainer<
        std::iter::FlatMap<
            std::slice::Iter<'b, [SudokuBlock; 9]>,
            &'b [SudokuBlock],
            impl FnMut(&'b [SudokuBlock; 9]) -> &'b [SudokuBlock],
        >,
        &'b SudokuBlock,
    > {
        let (start_row, start_col) = square_number_to_index(square_number);
        SudokuContainer::new(
            self.blocks[start_row..start_row + 3]
                .iter()
                .flat_map(move |row| &row[start_col..start_col + 3]),
        )
    }

    pub fn get_square_mut<'b>(
        &'b mut self,
        square_number: SudokuNumber,
    ) -> SudokuContainer<
        std::iter::FlatMap<
            std::slice::IterMut<'b, [SudokuBlock; 9]>,
            &'b mut [SudokuBlock],
            impl FnMut(&'b mut [SudokuBlock; 9]) -> &'b mut [SudokuBlock],
        >,
        &'b mut SudokuBlock,
    > {
        let (start_row, start_col) = square_number_to_index(square_number);
        SudokuContainer::new(
            self.blocks[start_row..start_row + 3]
                .iter_mut()
                .flat_map(move |row| &mut row[start_col..start_col + 3]),
        )
    }

    pub fn get_block_possible_numbers(&self, index: &BlockIndex) -> SudokuNumbers {
        let mut possible_numbers = SudokuNumbers::new_all();

        possible_numbers.del_numbers(
            self.get_row(index.row)
                .get_numbers()
                .into_iter()
                .chain(self.get_col(index.col).get_numbers().into_iter())
                .chain(
                    self.get_square(index.square_number())
                        .get_numbers()
                        .into_iter(),
                ),
        );

        possible_numbers
    }

    /// Updates possible values for each [`SudokuBlockStatus::Unresolved`] or [`SudokuBlockStatus::Possibilities`]
    /// block based on [`SudokuBlockStatus::Fixed`] blocks values.
    pub fn update_possibilities(&mut self) {
        use SudokuNumber::*;

        // self.get_blocks_mut()
        //     .filter(|block| block.is_possibilities() || block.is_unresolved())
        //     .for_each(|block| {
        //         let possibles = self.get_block_possible_numbers(block.index());
        //         block.status = SudokuBlockStatus::Possibilities(Possibilities::new(possibles));
        //     });

        for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
                let index = BlockIndex::new(row, col);
                if let SudokuBlockStatus::Unresolved | SudokuBlockStatus::Possibilities(_) =
                    self.get_block(&index).status
                {
                    let possibles = self.get_block_possible_numbers(&index);
                    let block = self.get_block_mut(&index);
                    block.status = SudokuBlockStatus::Possibilities(Possibilities::new(possibles));
                }
            }
        }
    }

    /// Since these strategies work with possible values in blocks and updating them,
    /// Then [`SudokuBoard::update_possibilities`] is always called before engaging the strategy.
    pub fn engage_strategy<S>(&mut self, strategy: S)
    where
        S: SudokuSolvingStrategy,
    {
        self.update_possibilities();
        strategy.update_possible_numbers(self);
    }

    pub fn resolve_satisfied_blocks(&mut self) {
        for block in self.get_blocks_mut().filter(|f| f.is_possibilities()) {
            let possibles = block.status.as_possibilities().unwrap();
            if possibles.numbers.count_numbers() == 1 {
                let single_naked = possibles.numbers.iter().next().unwrap();
                block.status = SudokuBlockStatus::Resolved(single_naked);
            }
        }

        self.update_possibilities();
    }

    pub fn reset(&mut self) {
        for block in self.get_blocks_mut() {
            if !block.is_fixed() {
                block.status = SudokuBlockStatus::Unresolved;
            }

            block.conflicting = None;
        }
    }

    pub fn find_block_mistakes(&self, index: &BlockIndex) -> Option<Vec<BlockIndex>> {
        let block = self.get_block(index);
        if let SudokuBlockStatus::Resolved(resolved) = block.status {
            let mut mistakes = vec![];

            let mut row_m =
                find_similar_in_container(resolved, self.get_row(index.row), Some(&block.index));
            let mut col_m =
                find_similar_in_container(resolved, self.get_col(index.col), Some(&block.index));
            let mut square_m = find_similar_in_container(
                resolved,
                self.get_square(index.square_number()),
                Some(&block.index),
            );

            mistakes.append(&mut row_m);
            mistakes.append(&mut col_m);
            mistakes.append(&mut square_m);
            mistakes.dedup();

            if mistakes.is_empty() {
                return None;
            } else {
                return Some(mistakes);
            }
        }
        None
    }

    pub fn mark_conflicts(
        &mut self,
        index: &BlockIndex,
        possibility_number_info: Option<(SudokuNumber, bool)>,
    ) {
        let block_status = &self.get_block(index).status;

        match block_status {
            SudokuBlockStatus::Unresolved => {
                self.get_block_mut(index).conflicting = None;
                self.get_blocks_mut()
                    .filter(|f| {
                        f.conflicting.as_ref().is_some_and(|conf| {
                            conf.is_affected_by_and(|f| f == index)
                                || conf.is_affected_by_possibilities_and(|block_index, _| {
                                    block_index == index
                                })
                        })
                    })
                    .for_each(|f| f.conflicting = None);
            }
            SudokuBlockStatus::Fixed(_) => (),
            SudokuBlockStatus::Resolved(_) => {
                if let Some(affected_indexes) = self.find_block_mistakes(index) {
                    self.get_block_mut(index).conflicting = Some(Conflicting::Source);
                    for affected in affected_indexes {
                        if &affected == index {
                            continue;
                        }

                        self.get_block_mut(&affected).conflicting =
                            Some(Conflicting::AffectedBy(index.clone()));
                    }
                } else {
                    self.get_block_mut(index).conflicting = None;
                    self.get_blocks_mut()
                        .filter(|f| {
                            f.conflicting
                                .as_ref()
                                .is_some_and(|conf| conf.is_affected_by_and(|f| f == index))
                        })
                        .for_each(|f| f.conflicting = None);
                }
            }
            SudokuBlockStatus::Possibilities(_) => {
                if let Some((pos, is_cleared)) = possibility_number_info {
                    let mut similar = vec![];

                    if !is_cleared {
                        let mut row_similar =
                            find_similar_in_container(pos, self.get_row(index.row), None);
                        let mut col_similar =
                            find_similar_in_container(pos, self.get_col(index.col), None);
                        let mut square_similar = find_similar_in_container(
                            pos,
                            self.get_square(index.square_number()),
                            None,
                        );

                        similar.append(&mut row_similar);
                        similar.append(&mut col_similar);
                        similar.append(&mut square_similar);
                        similar.dedup();
                    }

                    if !similar.is_empty() {
                        let block = self.get_block_mut(index);
                        let poss = block.status.as_possibilities_mut().unwrap();
                        poss.conflicting_numbers.set_number(pos);

                        for block_index in similar {
                            let block = self.get_block_mut(&block_index);
                            block.conflicting = Some(Conflicting::AffectedByPossibilities {
                                block_index: index.clone(),
                                number: pos,
                            });
                        }
                    } else {
                        let block = self.get_block_mut(index);
                        let poss = block.status.as_possibilities_mut().unwrap();
                        poss.conflicting_numbers.del_number(pos);

                        self.get_blocks_mut()
                            .filter(|f| {
                                f.conflicting.as_ref().is_some_and(|conf| {
                                    conf.is_affected_by_possibilities_and(|block_index, number| {
                                        block_index == index && number == &pos
                                    })
                                })
                            })
                            .for_each(|f| f.conflicting = None);
                    }
                }
            }
        }
    }
}

fn square_number_to_index(square_number: SudokuNumber) -> (usize, usize) {
    let start_row = (square_number.to_index() / 3) * 3;
    let start_col = (square_number.to_index() % 3) * 3;
    (start_row, start_col)
}

pub fn find_mistake_in_container<'s>(
    iterator: impl Iterator<Item = &'s SudokuBlock>,
) -> HashMap<SudokuNumber, Vec<BlockIndex>> {
    let mut counts = HashMap::new();

    for x in iterator {
        let number = match x.status {
            SudokuBlockStatus::Fixed(sudoku_number) => sudoku_number,
            _ => continue,
        };

        let indexes = counts.entry(number).or_insert(vec![]);
        indexes.push(x.index.clone());
    }

    counts
}

pub fn find_similar_in_container<'s>(
    number: SudokuNumber,
    iterator: impl Iterator<Item = &'s SudokuBlock>,
    ignore_index: Option<&BlockIndex>,
) -> Vec<BlockIndex> {
    let mut counts = Vec::new();

    for x in iterator {
        let found_number = match x.status {
            SudokuBlockStatus::Fixed(sudoku_number) => sudoku_number,
            _ => continue,
        };

        if let Some(ignore) = ignore_index {
            if &x.index == ignore {
                continue;
            }
        }

        if number == found_number {
            counts.push(x.index.clone());
        }
    }

    counts
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

        let numbers = board.get_block_possible_numbers(&BlockIndex::new(One, One));
        println!("{:?}", numbers.iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_conflicts() {
        use SudokuNumber::*;

        let mut board = SudokuBoard::default();
        board.fill_board_u8(sudoku_samples::easy::FIRST).unwrap();

        board.get_block_mut(&BlockIndex::new(One, One)).status = SudokuBlockStatus::Resolved(Seven);
        //TODO -
    }
}
