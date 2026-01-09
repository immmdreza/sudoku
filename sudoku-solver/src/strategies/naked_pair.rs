use std::collections::{HashMap, HashSet};

use crate::{
    BlockIndex,
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::SudokuSolvingStrategy,
};

pub struct NakedPairStrategy;

impl SudokuSolvingStrategy for NakedPairStrategy {
    const STRATEGY: super::Strategy = super::Strategy::NakedPair;

    fn update_possible_numbers(&self, board: &mut crate::SudokuBoard, show_only_effect: bool) {
        for index in SudokuNumber::ALL {
            let mut grouping: HashMap<SudokuNumbers, HashSet<BlockIndex>> = HashMap::new();
            for (block_index, poss) in board
                .get_row(index)
                .filter_map(|b| b.status.as_possibilities().map(|f| (b.index().clone(), f)))
            {
                let group = grouping.entry(poss.numbers.clone()).or_default();
                group.insert(block_index);
            }

            for (numbers, indexes) in grouping {
                // This (condition below) means the count of blocks having exact possible numbers is as same
                // as the count of each one's possible numbers. And this means these n numbers are
                // only valid in these n blocks (So remove them from others)
                if numbers.count_numbers() == indexes.len() {
                    board
                        .get_row_mut(index)
                        .filter(|f| f.is_possibilities())
                        .for_each(|block| {
                            let index = block.index.clone();
                            let poss = block.status.as_possibilities_mut().unwrap();

                            if indexes.contains(&index) {
                                // This is a pair
                                for number in numbers.iter() {
                                    if show_only_effect {
                                        poss.update_strategy_marker(
                                            number,
                                            super::StrategyMarker {
                                                strategy: super::Strategy::NakedPair,
                                                effect: super::StrategyEffect::Source,
                                            },
                                        );
                                    } else {
                                        poss.clear_strategy_marker(number);
                                    }
                                }
                            } else {
                                // This is not a pair remove pair possibilities from it.

                                if !show_only_effect {
                                    poss.numbers.del_numbers(numbers.iter());
                                    for number in numbers.iter() {
                                        poss.clear_strategy_marker(number);
                                    }
                                } else {
                                    for number in numbers.iter() {
                                        poss.update_strategy_marker(
                                            number,
                                            super::StrategyMarker {
                                                strategy: super::Strategy::NakedPair,
                                                effect: super::StrategyEffect::Effected,
                                            },
                                        );
                                    }
                                }
                            }
                        });
                }
            }
        }
    }
}
