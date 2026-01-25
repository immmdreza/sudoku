use bevy::prelude::*;
use std::fmt::{Display, Write as _};
use sudoku_solver::{numbers::SudokuNumber, strategies::Strategy};

#[derive(Debug, States, Default, PartialEq, Eq, Hash, Clone)]
pub enum AppState {
    #[default]
    Loading,
    Ready,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Direction::Up => '↑',
            Direction::Down => '↓',
            Direction::Left => '←',
            Direction::Right => '→',
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    Number(SudokuNumber),
    CalculatePossibilities,
    ResolveNakedSingles,
    Reset,
    ChangeSelectionMode,
    ClearBlock,
    Direction(Direction),
    Strategy(Strategy),
}

impl Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::Number(sudoku_number) => sudoku_number.to_u8().fmt(f),
            CommandType::CalculatePossibilities => f.write_str("Space"),
            CommandType::ResolveNakedSingles => f.write_str("ENTER"),
            CommandType::Reset => f.write_char('R'),
            CommandType::ChangeSelectionMode => f.write_char('M'),
            CommandType::ClearBlock => f.write_char('C'),
            CommandType::Direction(direction) => direction.fmt(f),
            CommandType::Strategy(strategy) => strategy.fmt(f),
        }
    }
}

#[derive(Debug, Event)]
pub struct GameInputs {
    command_type: CommandType,
}

impl GameInputs {
    pub fn new(command_type: CommandType) -> Self {
        Self { command_type }
    }

    pub fn command_type(&self) -> CommandType {
        self.command_type
    }
}

#[derive(Debug, Bundle)]
pub struct TextBundle {
    text: Text2d,
    font: TextFont,
    color: TextColor,
    layout: TextLayout,
    transform: Transform,
}

impl TextBundle {
    pub fn new(
        text: impl Into<String>,
        font: impl Into<Handle<Font>>,
        font_size: f32,
        color: impl Into<Color>,
        transform: Transform,
    ) -> Self {
        Self {
            text: Text2d(text.into()),
            font: TextFont {
                font: font.into(),
                font_size,
                ..Default::default()
            },
            color: TextColor(color.into()),
            layout: TextLayout::new(Justify::Center, LineBreak::NoWrap),
            transform,
        }
    }

    pub fn new_with_layout(
        text: impl Into<String>,
        font: impl Into<Handle<Font>>,
        font_size: f32,
        color: impl Into<Color>,
        transform: Transform,
        layout: TextLayout,
    ) -> Self {
        Self {
            text: Text2d(text.into()),
            font: TextFont {
                font: font.into(),
                font_size,
                ..Default::default()
            },
            color: TextColor(color.into()),
            layout,
            transform,
        }
    }
}
