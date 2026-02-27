use bevy::prelude::*;

use crate::{
    create_game_command,
    plugins::game_commands::GameCommand,
    shared::{
        DEFAULT_HELP_TEXT,
        components::HelpText,
        resources::{ActiveBoardProvider, BoardPlayingState, BoardsStateMap, SudokuBoardResources},
    },
};

create_game_command!(ResetBoardCommand, reset_board);

fn reset_board(
    active_board: ActiveBoardProvider,
    mut boards: ResMut<SudokuBoardResources>,
    mut boards_state: ResMut<BoardsStateMap>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    let active_board = if let Some(active_board) = active_board.active_board() {
        active_board
    } else {
        return;
    };

    let board = boards.active_board_mut(active_board);
    let board_state = boards_state.get_mut(active_board);

    #[cfg(feature = "debug")]
    println!("Resetting.");
    board.reset();

    if let Some(state) = board_state {
        state.stats = Default::default();
        state.playing_state = BoardPlayingState::Playing;
    }

    help_text.0 = DEFAULT_HELP_TEXT.to_string();
}
