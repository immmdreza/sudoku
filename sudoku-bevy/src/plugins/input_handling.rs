use std::time::Duration;

use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use sudoku_solver::{numbers::SudokuNumber, strategies::Strategy};

use crate::plugins::shared::{AppState, CommandType, Direction, GameInputs};

#[derive(Resource)]
struct ChangeSelectionTimer(Timer);

pub struct InputHandlingPlugin;

impl Plugin for InputHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChangeSelectionTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .add_systems(
            Update,
            ((
                change_selected_block.run_if(
                    input_pressed(KeyCode::ArrowDown)
                        .or(input_pressed(KeyCode::ArrowUp))
                        .or(input_pressed(KeyCode::ArrowLeft))
                        .or(input_pressed(KeyCode::ArrowRight)),
                ),
                change_selection_mode.run_if(input_just_pressed(KeyCode::KeyM)),
                (
                    engage_strategy.run_if(
                        input_just_pressed(KeyCode::KeyH).or(input_just_pressed(KeyCode::KeyP)),
                    ),
                    update_possibilities.run_if(input_just_pressed(KeyCode::Space)),
                    resolve_satisfied.run_if(input_just_pressed(KeyCode::Enter)),
                    manually_clear_block.run_if(input_just_pressed(KeyCode::KeyC)),
                    digit_1_to_9_clicked.run_if(
                        input_just_pressed(KeyCode::Digit1)
                            .or(input_just_pressed(KeyCode::Digit2))
                            .or(input_just_pressed(KeyCode::Digit3))
                            .or(input_just_pressed(KeyCode::Digit4))
                            .or(input_just_pressed(KeyCode::Digit5))
                            .or(input_just_pressed(KeyCode::Digit6))
                            .or(input_just_pressed(KeyCode::Digit7))
                            .or(input_just_pressed(KeyCode::Digit8))
                            .or(input_just_pressed(KeyCode::Digit9))
                            .or(input_just_pressed(KeyCode::Numpad1))
                            .or(input_just_pressed(KeyCode::Numpad2))
                            .or(input_just_pressed(KeyCode::Numpad3))
                            .or(input_just_pressed(KeyCode::Numpad4))
                            .or(input_just_pressed(KeyCode::Numpad5))
                            .or(input_just_pressed(KeyCode::Numpad6))
                            .or(input_just_pressed(KeyCode::Numpad7))
                            .or(input_just_pressed(KeyCode::Numpad8))
                            .or(input_just_pressed(KeyCode::Numpad9)),
                    ),
                ),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
            )
                .run_if(in_state(AppState::Ready)),),
        );
    }
}

fn update_possibilities(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.trigger(GameInputs::new(CommandType::CalculatePossibilities));
    }
}

fn engage_strategy(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        commands.trigger(GameInputs::new(CommandType::Strategy(
            Strategy::HiddenSingle,
        )));
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        commands.trigger(GameInputs::new(CommandType::Strategy(Strategy::NakedPair)));
    }
}

fn resolve_satisfied(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        commands.trigger(GameInputs::new(CommandType::ResolveNakedSingles));
    }
}

fn reset(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(GameInputs::new(CommandType::Reset));
    }
}

fn change_selection_mode(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        commands.trigger(GameInputs::new(CommandType::ChangeSelectionMode));
    }
}

fn manually_clear_block(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        commands.trigger(GameInputs::new(CommandType::ClearBlock));
    }
}

fn digit_1_to_9_clicked(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if let Some(sudoku_number) =
        if keyboard_input.any_just_pressed([KeyCode::Digit1, KeyCode::Numpad1]) {
            Some(SudokuNumber::One)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit2, KeyCode::Numpad2]) {
            Some(SudokuNumber::Two)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit3, KeyCode::Numpad3]) {
            Some(SudokuNumber::Three)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit4, KeyCode::Numpad4]) {
            Some(SudokuNumber::Four)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit5, KeyCode::Numpad5]) {
            Some(SudokuNumber::Five)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit6, KeyCode::Numpad6]) {
            Some(SudokuNumber::Six)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit7, KeyCode::Numpad7]) {
            Some(SudokuNumber::Seven)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit8, KeyCode::Numpad8]) {
            Some(SudokuNumber::Eight)
        } else if keyboard_input.any_just_pressed([KeyCode::Digit9, KeyCode::Numpad9]) {
            Some(SudokuNumber::Nine)
        } else {
            None
        }
    {
        commands.trigger(GameInputs::new(CommandType::Number(sudoku_number)));
    }
}

fn change_selected_block(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ChangeSelectionTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            commands.trigger(GameInputs::new(CommandType::Direction(Direction::Left)));
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            commands.trigger(GameInputs::new(CommandType::Direction(Direction::Right)));
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            commands.trigger(GameInputs::new(CommandType::Direction(Direction::Down)));
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            commands.trigger(GameInputs::new(CommandType::Direction(Direction::Up)));
        }
    }
}
