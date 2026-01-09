use std::collections::HashMap;

use crate::{
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::{StrategyMarker, SudokuSolvingStrategy},
};

use SudokuNumber::*;

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
        SudokuBoard::square_number(self.row, self.col)
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
    strategy_markers: HashMap<SudokuNumber, StrategyMarker>,
}

impl Possibilities {
    pub fn new(numbers: SudokuNumbers) -> Self {
        Self {
            numbers,
            conflicting_numbers: Default::default(),
            strategy_markers: Default::default(),
        }
    }

    pub fn is_conflicting(&self, number: SudokuNumber) -> bool {
        self.numbers.has_number(number) && self.conflicting_numbers.has_number(number)
    }

    pub fn update_strategy_marker(&mut self, number: SudokuNumber, marker: StrategyMarker) {
        if let Some(inside) = self.strategy_markers.get_mut(&number) {
            *inside = marker;
        } else {
            self.strategy_markers.insert(number, marker);
        }
    }

    pub fn clear_strategy_marker(&mut self, number: SudokuNumber) -> Option<StrategyMarker> {
        self.strategy_markers.remove(&number)
    }

    pub fn has_strategy_effect(&self, number: &SudokuNumber) -> Option<&StrategyMarker> {
        self.strategy_markers.get(number)
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
        SudokuBoard::get_numbers(self)
    }

    pub fn filter_resolved(
        self,
    ) -> SudokuContainer<
        std::iter::Filter<
            SudokuContainer<T, &'b SudokuBlock>,
            impl FnMut(&&'b SudokuBlock) -> bool,
        >,
        &'b SudokuBlock,
    > {
        SudokuContainer::new(self.filter(|f| f.is_resolved()))
    }

    pub fn filter_unresolved(
        self,
    ) -> SudokuContainer<
        std::iter::Filter<
            SudokuContainer<T, &'b SudokuBlock>,
            impl FnMut(&&'b SudokuBlock) -> bool,
        >,
        &'b SudokuBlock,
    > {
        SudokuContainer::new(self.filter(|f| f.is_unresolved()))
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

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ContainerType {
    Row,
    Column,
    Square,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SudokuBoard {
    blocks: [[SudokuBlock; 9]; 9],
}

impl Default for SudokuBoard {
    fn default() -> Self {
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

    pub fn from_u8(numbers: [[Option<u8>; 9]; 9]) -> Self {
        let mut board = Self::default();
        board.fill_board_u8(numbers).unwrap();
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

    pub fn get_container(
        &self,
        container_type: ContainerType,
        number: SudokuNumber,
    ) -> Vec<&SudokuBlock> {
        match container_type {
            ContainerType::Row => self.get_row(number).collect(),
            ContainerType::Column => self.get_col(number).collect(),
            ContainerType::Square => self.get_square(number).collect(),
        }
    }

    pub fn get_container_mut(
        &mut self,
        container_type: ContainerType,
        number: SudokuNumber,
    ) -> Vec<&mut SudokuBlock> {
        match container_type {
            ContainerType::Row => self.get_row_mut(number).collect(),
            ContainerType::Column => self.get_col_mut(number).collect(),
            ContainerType::Square => self.get_square_mut(number).collect(),
        }
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
        let (start_row, start_col) = SudokuBoard::square_number_to_index(square_number);
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
        let (start_row, start_col) = SudokuBoard::square_number_to_index(square_number);
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

    pub fn engage_strategy<S>(&mut self, strategy: S, show_only_effect: bool)
    where
        S: SudokuSolvingStrategy,
    {
        // self.update_possibilities();
        strategy.update_possible_numbers(self, show_only_effect);
    }

    pub fn resolve_satisfied_blocks(&mut self) {
        for block in self.get_blocks_mut().filter(|f| f.is_possibilities()) {
            let possibles = block.status.as_possibilities().unwrap();
            if possibles.numbers.count_numbers() == 1 {
                let single_naked = possibles.numbers.iter().next().unwrap();
                block.status = SudokuBlockStatus::Resolved(single_naked);
            }
        }

        if self.mark_all_conflicts() {
            self.update_possibilities();
        }
    }

    pub fn reset(&mut self) {
        for block in self.get_blocks_mut() {
            if !block.is_fixed() {
                block.status = SudokuBlockStatus::Unresolved;
            }

            block.conflicting = None;
        }
    }

    fn find_mistakes(&self, index: &BlockIndex, number: SudokuNumber) -> Option<Vec<BlockIndex>> {
        let row_m =
            SudokuBoard::find_similar_in_container(number, self.get_row(index.row), Some(index));
        let col_m =
            SudokuBoard::find_similar_in_container(number, self.get_col(index.col), Some(index));
        let square_m = SudokuBoard::find_similar_in_container(
            number,
            self.get_square(index.square_number()),
            Some(index),
        );

        let mut mistakes = row_m.chain(col_m).chain(square_m).collect::<Vec<_>>();
        mistakes.dedup();

        if mistakes.is_empty() {
            return None;
        } else {
            return Some(mistakes);
        }
    }

    pub fn find_resolved_block_mistakes(&self, index: &BlockIndex) -> Option<Vec<BlockIndex>> {
        let block = self.get_block(index);
        let resolved = block.status.as_resolved()?;
        self.find_mistakes(index, *resolved)
    }

    pub fn mark_conflicts(
        &mut self,
        index: &BlockIndex,
        possibility_number_info: Option<(SudokuNumber, bool)>,
    ) -> bool {
        let block_status = &self.get_block(index).status;

        match block_status {
            SudokuBlockStatus::Unresolved => self.mark_conflicts_unresolved(index),
            SudokuBlockStatus::Resolved(_) => self.mark_conflicts_resolved(index),
            SudokuBlockStatus::Possibilities(_) => {
                self.mark_conflicts_possibilities(index, possibility_number_info)
            }
            SudokuBlockStatus::Fixed(_) => true,
        }
    }

    fn mark_conflicts_possibilities(
        &mut self,
        index: &BlockIndex,
        possibility_number_info: Option<(SudokuNumber, bool)>,
    ) -> bool {
        if let Some((pos, is_cleared)) = possibility_number_info {
            let mut similar = vec![];

            if !is_cleared {
                if self
                    .get_block(index)
                    .status
                    .as_possibilities()
                    .unwrap()
                    .numbers
                    .count_numbers()
                    == 1
                {
                    // Clean up previous conflicts
                    self.get_blocks_mut()
                        .filter(|f| {
                            f.conflicting
                                .as_ref()
                                .is_some_and(|conf| conf.is_affected_by_and(|f| f == index))
                        })
                        .for_each(|f| f.conflicting = None);
                }

                if let Some(mistakes) = self.find_mistakes(index, pos) {
                    similar = mistakes;
                }
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
                false
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
                true
            }
        } else {
            true
        }
    }

    fn mark_conflicts_resolved(&mut self, index: &BlockIndex) -> bool {
        // Clean up previous conflicts
        self.clear_all_previous_conflicts(index);

        if let Some(affected_indexes) = self.find_resolved_block_mistakes(index) {
            self.get_block_mut(index).conflicting = Some(Conflicting::Source);
            for affected in affected_indexes {
                if &affected == index {
                    continue;
                }

                self.get_block_mut(&affected).conflicting =
                    Some(Conflicting::AffectedBy(index.clone()));
            }
            false
        } else {
            self.get_block_mut(index).conflicting = None;
            true
        }
    }

    fn mark_conflicts_unresolved(&mut self, index: &BlockIndex) -> bool {
        self.get_block_mut(index).conflicting = None;
        self.clear_all_previous_conflicts(index);
        true
    }

    fn clear_all_previous_conflicts(&mut self, index: &BlockIndex) {
        self.get_blocks_mut()
            .filter(|f| {
                f.conflicting.as_ref().is_some_and(|conf| {
                    conf.is_affected_by_and(|f| f == index)
                        || conf
                            .is_affected_by_possibilities_and(|block_index, _| block_index == index)
                })
            })
            .for_each(|f| f.conflicting = None);
    }

    pub fn mark_all_conflicts(&mut self) -> bool {
        use SudokuNumber::*;

        let mut verified = true;
        for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
                let index = BlockIndex::new(row, col);
                let block = self.get_block(&index);
                if !match &block.status {
                    SudokuBlockStatus::Resolved(_) => self.mark_conflicts_resolved(&index),
                    SudokuBlockStatus::Unresolved => self.mark_conflicts_unresolved(&index),
                    _ => true,
                } {
                    verified = false;
                }
            }
        }

        verified
    }

    pub fn verify_board(&self) -> bool {
        use SudokuNumber::*;

        for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
                let index = BlockIndex::new(row, col);
                if let SudokuBlockStatus::Resolved(number) = &self.get_block(&index).status {
                    if self.find_mistakes(&index, *number).is_some() {
                        return false;
                    }
                }
            }
        }

        true
    }
}

// Static functions.
impl SudokuBoard {
    pub fn iter_block_indexes() -> impl Iterator<Item = BlockIndex> {
        SudokuNumber::iter_numbers().map(|(row, col)| BlockIndex::new(row, col))
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
    ) -> impl Iterator<Item = BlockIndex> {
        iterator
            .filter(|f| f.is_fixed() || f.is_resolved())
            .filter(move |f| ignore_index.is_some_and(|g| f.index != *g))
            .filter(move |f| match f.status {
                SudokuBlockStatus::Fixed(sudoku_number)
                | SudokuBlockStatus::Resolved(sudoku_number) => sudoku_number == number,
                _ => false,
            })
            .map(|f| f.index.clone())
    }

    pub fn get_numbers<'s>(iterator: impl Iterator<Item = &'s SudokuBlock>) -> SudokuNumbers {
        SudokuNumbers::new(iterator.filter_map(|f| match f.status {
            SudokuBlockStatus::Fixed(sudoku_number)
            | SudokuBlockStatus::Resolved(sudoku_number) => Some(sudoku_number),
            _ => None,
        }))
    }

    pub fn get_missing_numbers<'s>(
        iterator: impl Iterator<Item = &'s SudokuBlock>,
    ) -> SudokuNumbers {
        SudokuNumbers::new(SudokuBoard::get_numbers(iterator).get_missing_numbers())
    }

    pub fn square_number(row: SudokuNumber, col: SudokuNumber) -> SudokuNumber {
        (((row.to_index() / 3) * 3 + (col.to_index() / 3)) + 1)
            .try_into()
            .unwrap()
    }
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
