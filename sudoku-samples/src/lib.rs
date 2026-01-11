pub mod easy;
pub mod normal;

/// Macro to quickly create a `SudokuBoard` from inline values.
/// Example usage:
/// let mut board = sudoku_board![
///     [5, 3, _, _, 7, _, _, _, _],
///     [6, _, _, 1, 9, 5, _, _, _],
///     [_, 9, 8, _, _, _, _, 6, _],
///     [8, _, _, _, 6, _, _, _, 3],
///     [4, _, _, 8, _, 3, _, _, 1],
///     [7, _, _, _, 2, _, _, _, 6],
///     [_, 6, _, _, _, _, 2, 8, _],
///     [_, _, _, 4, 1, 9, _, _, 5],
///     [_, _, _, _, 8, _, _, 7, 9],
/// ];
#[macro_export]
macro_rules! sudoku_board {
    ( $( [ $($cell:tt),* ] ),* $(,)? ) => {{
        let data: [[Option<u8>; 9]; 9] = [
            $(
                [
                    $(
                        sudoku_board!(@cell $cell),
                    )*
                ],
            )*
        ];
        data
    }};
    // Helper for cell parsing: _ => None, N => Some(N)
    (@cell _) => { None };
    (@cell $e:expr) => { Some($e) };
}

#[macro_export]
macro_rules! define_sudoku_board {
    ($name:ident, $board:tt) => {
        pub const $name: [[Option<u8>; 9]; 9] = sudoku_board!$board;
    };
}
