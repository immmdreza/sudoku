#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, fmt::Display};

use bevy::{
    color::palettes::{
        css::{BLACK, RED, WHITE, YELLOW},
        tailwind::ORANGE_300,
    },
    ecs::system::{SystemId, SystemParam},
    log::{self},
    prelude::*,
    sprite::Anchor,
    text::TextBounds,
};
use sudoku_bevy::{
    BlocksAccessInfo, SquareIndex, gen_random_city_name,
    plugins::{
        input_handling::InputHandlingPlugin,
        setup::{DefaultAssets, DefaultMaterials, SetupPlugin, StrategyMarkerColors},
        shared::{AppState, CommandType, Direction, GameInputs, TextBundle},
    },
};
use sudoku_solver::{
    BlockIndex, Conflicting, Possibilities as SudokuPossibilities, SudokuBlockStatus, SudokuBoard,
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::{Strategy, hidden_single::HiddenSingleStrategy, naked_pair::NakedPairStrategy},
};

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SudokuBoardDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Expert,
}

impl Display for SudokuBoardDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuBoardDifficulty::Easy => write!(f, "Easy"),
            SudokuBoardDifficulty::Normal => write!(f, "Normal"),
            SudokuBoardDifficulty::Hard => write!(f, "Hard"),
            SudokuBoardDifficulty::Expert => write!(f, "Expert"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct BoardId {
    difficulty: Option<SudokuBoardDifficulty>,
    index: usize,
}

impl From<(Option<SudokuBoardDifficulty>, usize)> for BoardId {
    fn from((difficulty, index): (Option<SudokuBoardDifficulty>, usize)) -> Self {
        Self::new(difficulty, index)
    }
}

impl BoardId {
    fn new(difficulty: Option<SudokuBoardDifficulty>, index: usize) -> Self {
        Self { difficulty, index }
    }
}

#[derive(Debug, Clone, Resource, Default, Deref, DerefMut)]
struct ActiveBoardsMapping(HashMap<Entity, BoardId>);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
struct ActiveBoardChanged(bool);

#[derive(Debug, Resource, Default, PartialEq, Eq)]
struct SudokuBoardResources {
    boards: HashMap<Option<SudokuBoardDifficulty>, Vec<SudokuBoard>>,
}

impl SudokuBoardResources {
    fn active_board(&self, active_board: &BoardId) -> &SudokuBoard {
        self.boards
            .get(&active_board.difficulty)
            .unwrap()
            .get(active_board.index)
            .unwrap()
    }

    fn active_board_mut(&mut self, active_board: &BoardId) -> &mut SudokuBoard {
        self.boards
            .get_mut(&active_board.difficulty)
            .unwrap()
            .get_mut(active_board.index)
            .unwrap()
    }
}

#[derive(Debug, Resource, Default, PartialEq, Eq)]
struct SudokuBoardSnapshotResources {
    boards: HashMap<Option<SudokuBoardDifficulty>, Vec<SudokuBoard>>,
}

impl SudokuBoardSnapshotResources {
    fn active_board(&self, active_board: &BoardId) -> &SudokuBoard {
        self.boards
            .get(&active_board.difficulty)
            .unwrap()
            .get(active_board.index)
            .unwrap()
    }

    fn active_board_mut(&mut self, active_board: &BoardId) -> &mut SudokuBoard {
        self.boards
            .get_mut(&active_board.difficulty)
            .unwrap()
            .get_mut(active_board.index)
            .unwrap()
    }
}

#[derive(Debug, Default)]
enum SelectionMode {
    #[default]
    Resolving,
    Possibilities,
}

#[derive(Debug, Resource, Default)]
struct SelectedBlock {
    mode: SelectionMode,
    current: (usize, usize),
}

impl SelectedBlock {
    #[allow(dead_code)]
    fn block_index(&self) -> BlockIndex {
        BlockIndex::from_index(self.current.1, self.current.0).unwrap()
    }
}

#[derive(Debug, Resource, Default)]
struct Stats {
    /// Mistakes while resolving a block number
    mistakes: u32,

    /// Mistakes while marking a number as possible in a block
    possibility_mistakes: u32,
}

#[derive(Debug, Component)]
struct MistakesCountText;

#[derive(Debug, Component)]
struct HelperBlock {
    command_type: CommandType,
}

impl HelperBlock {
    fn new(command_type: CommandType) -> Self {
        Self { command_type }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
enum BoardState {
    #[default]
    Playing,
    FinishedVerified,
}

impl BoardState {
    /// Returns `true` if the board state is [`Playing`].
    ///
    /// [`Playing`]: BoardState::Playing
    #[allow(dead_code)]
    #[must_use]
    fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns `true` if the board state is [`FinishedVerified`].
    ///
    /// [`FinishedVerified`]: BoardState::FinishedVerified
    #[must_use]
    fn is_finished_verified(&self) -> bool {
        matches!(self, Self::FinishedVerified)
    }
}

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
struct BoardsStateMap {
    boards: HashMap<BoardId, BoardState>,
}

#[derive(Debug, Component)]
struct HelpText;

#[derive(Debug, Component)]
struct ActiveBoardText;

#[derive(Debug, Component, Deref, DerefMut)]
struct BoardInfoBlock(BoardId);

#[derive(Debug, Component)]
struct RequestNewBoardBlock;

const DEFAULT_HELP_TEXT: &str = "Use 'Space' to update possible values, 'Enter' to resolve blocks, 'R' to reset, 'M' to change selection mode, 'C' to clear block, 1 to 9 to set number and 'H' to engage Hidden single strategy.";

#[derive(Debug, Resource, Default)]
struct EngagingStrategy {
    strategy: Option<Strategy>,
    showed_effect: bool,
}

#[derive(Debug, Resource, Default, Deref, DerefMut)]
struct EngagingStrategyMap {
    engaging: HashMap<BoardId, EngagingStrategy>,
}

#[derive(Debug, Event)]
struct UpdateBoardList;

#[derive(Debug, Component, Deref)]
struct SudokuBoardVisual {
    name: String,
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct ActiveBoardVisual(Entity);

#[derive(Debug, SystemParam)]
struct ActiveBoardProvider<'w> {
    active_visual: If<Res<'w, ActiveBoardVisual>>,
    boards_mapping: If<Res<'w, ActiveBoardsMapping>>,
}

impl<'w> ActiveBoardProvider<'w> {
    fn active_board(&self) -> Option<&BoardId> {
        self.boards_mapping.0.get(&self.active_visual)
    }
}

#[derive(Debug, SystemParam)]
struct ActiveBoardProviderMut<'w> {
    active_visual: If<Res<'w, ActiveBoardVisual>>,
    boards_mapping: If<ResMut<'w, ActiveBoardsMapping>>,
}

#[allow(dead_code)]
impl<'w> ActiveBoardProviderMut<'w> {
    fn active_board(&mut self) -> Option<&mut BoardId> {
        self.boards_mapping.0.get_mut(&self.active_visual)
    }

    fn update_active_board(&mut self, id: BoardId) -> Option<BoardId> {
        self.boards_mapping.0.insert(***self.active_visual, id)
    }
}

#[derive(Debug, Resource)]
struct CreateBoardVisualSystemId(SystemId<In<Vec2>>);

fn main() {
    App::new()
        .add_plugins((SetupPlugin, InputHandlingPlugin))
        .init_resource::<ActiveBoardsMapping>()
        .init_resource::<ActiveBoardChanged>()
        .init_resource::<SudokuBoardResources>()
        .init_resource::<SudokuBoardSnapshotResources>()
        .init_resource::<SelectedBlock>()
        .init_resource::<Stats>()
        .init_resource::<EngagingStrategyMap>()
        .init_resource::<BoardsStateMap>()
        .init_resource::<ShouldUpdateAnyway>()
        .add_observer(update_boards_list)
        .add_observer(on_game_input)
        .add_observer(on_should_update_event)
        .add_observer(
            |event: On<Add, SudokuBoardVisual>,
             mut commands: Commands,
             active_visual: Option<ResMut<ActiveBoardVisual>>,
             mut active_board_mapping: ResMut<ActiveBoardsMapping>,
             mut active_board_changed: ResMut<ActiveBoardChanged>| {
                if let Some(mut active_visual) = active_visual {
                    active_visual.0 = event.entity;
                    active_board_changed.0 = true;
                } else {
                    commands.insert_resource(ActiveBoardVisual(event.entity));
                    active_board_mapping.insert(
                        event.entity,
                        BoardId {
                            difficulty: Some(SudokuBoardDifficulty::Normal),
                            index: 0,
                        },
                    );
                }
            },
        )
        // Ready state systems
        .add_systems(
            OnEnter(AppState::Ready),
            (setup_game, check_foundation_squares, check_block_squares).chain(),
        )
        .add_systems(
            PostUpdate,
            (
                (
                    update_board.run_if(
                        resource_changed::<SudokuBoardResources>
                            .or(resource_changed::<ActiveBoardsMapping>)
                            .or(resource_changed::<ShouldUpdateAnyway>),
                    ),
                    final_verification.run_if(resource_changed::<SudokuBoardResources>),
                )
                    .chain(),
                update_mistakes_text.run_if(resource_changed::<Stats>),
                update_active_board_text.run_if(
                    resource_changed::<ActiveBoardsMapping>
                        .or(resource_changed::<ActiveBoardChanged>),
                ),
                active_board_visual_changed
                    .run_if(resource_exists_and_changed::<ActiveBoardVisual>),
            )
                .chain()
                .run_if(in_state(AppState::Ready)),
        )
        .run();
}

const AVAILABLE_COMMANDS: [CommandType; 9] = [
    CommandType::CalculatePossibilities,
    CommandType::Direction(Direction::Left),
    CommandType::ResolveNakedSingles,
    CommandType::Direction(Direction::Up),
    CommandType::Reset,
    CommandType::Direction(Direction::Down),
    CommandType::ChangeSelectionMode,
    CommandType::Direction(Direction::Right),
    CommandType::ClearBlock,
];

fn setup_game(
    mut commands: Commands,
    mut sudoku_boards: ResMut<SudokuBoardResources>,
    mut sudoku_snapshots: ResMut<SudokuBoardSnapshotResources>,
    mut boards_state: ResMut<BoardsStateMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    defaults: Res<DefaultMaterials>,
    defaults_assets: Res<DefaultAssets>,
    strategy_colors: Res<StrategyMarkerColors>,
) {
    log::warn!("Death is close.");

    let boards = [
        (None, vec![SudokuBoard::default()]),
        (
            Some(SudokuBoardDifficulty::Easy),
            vec![SudokuBoard::from_u8(sudoku_samples::easy::FIRST)],
        ),
        (
            Some(SudokuBoardDifficulty::Normal),
            vec![SudokuBoard::from_u8(sudoku_samples::normal::FIRST)],
        ),
    ];

    for (k, v) in boards {
        let boards_count = v.len();
        sudoku_boards.boards.insert(k, v);
        sudoku_snapshots
            .boards
            .insert(k, (0..boards_count).map(|_| Default::default()).collect());
        (0..boards_count).for_each(|i| {
            boards_state
                .boards
                .insert((k, i).into(), Default::default());
        });
    }

    // Add create board system id resources and use it once.
    let spawn_board_system: SystemId<In<Vec2>> =
        commands.register_system(spawn_sudoku_board_visual);
    commands.run_system_with(spawn_board_system, Vec2::default().with_y(50.));
    commands.insert_resource(CreateBoardVisualSystemId(spawn_board_system));

    commands.spawn(TextBundle::new(
        "Sudoku",
        defaults_assets.default_font.clone(),
        100.,
        WHITE,
        Transform::from_translation(Vec3::default().with_y(460.)),
    ));

    commands.spawn((
        TextBundle::new(
            "Mistakes: ",
            defaults_assets.default_font.clone(),
            20.,
            WHITE,
            Transform::from_translation(Vec3::default().with_y(400.)),
        ),
        children![(
            TextSpan::new("0 (Numbers) / 0 (Possibilities)"),
            TextFont {
                font: defaults_assets.default_font.clone(),
                font_size: 20.,
                ..default()
            },
            TextColor(Color::from(RED)),
            MistakesCountText,
        )],
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(630., 110.))),
        MeshMaterial2d(defaults.default_foundation_block_color.clone()),
        Transform::default().with_translation(Vec3 {
            y: -340.,
            ..Default::default()
        }),
        children![(
            Mesh2d(meshes.add(Rectangle::new(610., 90.))),
            MeshMaterial2d(defaults.default_block_color.clone()),
            Transform::from_translation(Vec3::Z),
            children![(
                Text2d::new(DEFAULT_HELP_TEXT.to_string()),
                TextFont {
                    font: defaults_assets.default_font.clone(),
                    font_size: 20.,
                    ..default()
                },
                TextColor(defaults.default_base_text_color),
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                TextBounds::new(600., 80.),
                Transform::from_translation(Vec3::Z),
                HelpText,
            )]
        ),],
    ));

    commands.spawn(TextBundle::new(
        "Accessibility",
        defaults_assets.default_font.clone(),
        20.,
        WHITE,
        Transform::from_translation(Vec3::default().with_xy(vec2(420., 340.))),
    ));

    let spawn_info = SquareSpawnInfo {
        width: 180.,
        translation: vec2(420., 50.),
        index: (0, 0),
    };

    // Helpers buttons
    commands
        .spawn((SquareBundle::new(
            defaults.default_foundation_block_color.clone(),
            &mut meshes,
            spawn_info.clone(),
            None,
        ),))
        .with_children(|builder| {
            let width = spawn_info.width;
            let master_index = spawn_info.index;

            for (index, spawn_info) in square_group_info(width, 5., Default::default()).enumerate()
            {
                let number = (index + 1).try_into().unwrap();

                builder
                    .spawn((
                        SquareBundle::new(
                            defaults.default_block_color.clone(),
                            &mut meshes,
                            spawn_info.clone(),
                            Some(master_index),
                        ),
                        HelperBlock::new(CommandType::Number(number)),
                        Pickable::default(),
                        children![TextBundle::new(
                            format!("{}", index + 1),
                            defaults_assets.default_font.clone(),
                            spawn_info.width,
                            defaults.default_fixed_number_color,
                            Default::default(),
                        )],
                    ))
                    .observe(on_helper_block_clicked)
                    .observe(on_helper_block_hovered)
                    .observe(on_pointer_out);
            }
        });

    let spawn_info = SquareSpawnInfo {
        width: 180.,
        translation: vec2(420., 235.),
        index: (0, 0),
    };

    commands
        .spawn((SquareBundle::new(
            defaults.default_foundation_block_color.clone(),
            &mut meshes,
            spawn_info.clone(),
            None,
        ),))
        .with_children(|builder| {
            let width = spawn_info.width;
            let master_index = spawn_info.index;

            for (index, spawn_info) in square_group_info(width, 5., Default::default()).enumerate()
            {
                let command_type = AVAILABLE_COMMANDS[index];
                let command_type_text = command_type.to_string();
                let char_count = command_type_text.chars().count();
                let text_width =
                    spawn_info.width / (if char_count == 1 { 1 } else { char_count - 1 }) as f32;

                builder
                    .spawn((
                        SquareBundle::new(
                            defaults.default_block_color.clone(),
                            &mut meshes,
                            spawn_info.clone(),
                            Some(master_index),
                        ),
                        HelperBlock::new(command_type),
                        Pickable::default(),
                        children![TextBundle::new(
                            command_type_text,
                            defaults_assets.default_font.clone(),
                            text_width,
                            if let CommandType::Direction(_) = &command_type {
                                RED
                            } else {
                                BLACK
                            },
                            Default::default(),
                        )],
                    ))
                    .observe(on_helper_block_clicked)
                    .observe(on_helper_block_hovered)
                    .observe(on_pointer_out);
            }
        });

    commands.spawn(TextBundle::new(
        "Strategies",
        defaults_assets.default_font.clone(),
        20.,
        ORANGE_300,
        Transform::from_translation(Vec3::default().with_xy(vec2(420., -60.))),
    ));

    // Strategies
    let spawn_info = SquareSpawnInfo {
        width: 180.,
        translation: vec2(420., -165.),
        index: (0, 0),
    };

    commands
        .spawn((SquareBundle::new(
            defaults.default_foundation_block_color.clone(),
            &mut meshes,
            spawn_info.clone(),
            None,
        ),))
        .with_children(|builder| {
            let width = spawn_info.width;
            let master_index = spawn_info.index;
            let strategies = [Strategy::HiddenSingle, Strategy::NakedPair];

            for (index, spawn_info) in square_group_info(width, 5., Default::default()).enumerate()
            {
                if let Some(strategy) = strategies.get(index) {
                    builder
                        .spawn((
                            SquareBundle::new(
                                if let Some(color) = strategy_colors.get(strategy) {
                                    color.background.clone()
                                } else {
                                    defaults.default_block_color.clone()
                                },
                                &mut meshes,
                                spawn_info.clone(),
                                Some(master_index),
                            ),
                            HelperBlock::new(CommandType::Strategy(*strategy)),
                            Pickable::default(),
                            children![TextBundle::new(
                                strategy.to_string(),
                                defaults_assets.default_font.clone(),
                                spawn_info.width,
                                if let Some(color) = strategy_colors.get(strategy) {
                                    color.text
                                } else {
                                    defaults.default_fixed_number_color
                                },
                                Default::default(),
                            )],
                        ))
                        .observe(on_helper_block_clicked)
                        .observe(on_helper_block_hovered)
                        .observe(on_pointer_out);
                }
            }
        });

    commands.spawn((
        TextBundle::new_with_layout(
            "?!",
            defaults_assets.default_font.clone(),
            20.,
            YELLOW,
            Transform::from_translation(Vec3::default().with_x(90.).with_y(492.)),
            TextLayout::new(Justify::Left, LineBreak::NoWrap),
        ),
        Anchor::CENTER_LEFT,
        ActiveBoardText,
    ));

    commands.spawn(TextBundle::new(
        "Available boards:",
        defaults_assets.default_font.clone(),
        20.,
        YELLOW,
        Transform::from_translation(Vec3::default().with_xy(vec2(-420., 330.))),
    ));

    commands.spawn((
        TextBundle::new(
            "Heresy,",
            defaults_assets.default_font.clone(),
            100.,
            RED,
            Transform::default().with_translation(Vec3::default().with_y(120.).with_z(-5.)),
        ),
        children![TextBundle::new(
            "You say?",
            defaults_assets.default_font.clone(),
            83.,
            WHITE,
            Transform::default().with_translation(Vec3::default().with_y(-80.)),
        )],
    ));

    commands.trigger(UpdateBoardList);
}

fn active_board_visual_changed(
    mut commands: Commands,
    active_visual: Res<ActiveBoardVisual>,
    defaults: If<Res<DefaultMaterials>>,
    mut visuals: Query<(Entity, &mut MeshMaterial2d<ColorMaterial>), With<SudokuBoardVisual>>,
) {
    #[cfg(debug_assertions)]
    println!("Active  board changed.");

    for (entity, mut material) in visuals.iter_mut() {
        if entity == active_visual.0 {
            material.0 = defaults.default_active_board_color.clone();
        } else {
            material.0 = defaults.default_deactivate_board_color.clone();
        }
    }

    commands.trigger(UpdateBoardList);
}

fn update_active_board_text(
    active_board_provider: ActiveBoardProvider,
    visuals: Query<&SudokuBoardVisual>,
    mut text: Single<&mut Text2d, With<ActiveBoardText>>,
) {
    if let Some(active_board) = active_board_provider.active_board() {
        let name = if let Ok(visual_info) = visuals.get(***active_board_provider.active_visual) {
            format!(" In {}", visual_info.name)
        } else {
            "".to_string()
        };

        text.0 = format!(
            "#{} {}{}",
            active_board.index + 1,
            match active_board.difficulty {
                Some(diff) => diff.to_string(),
                None => "Unspecified".to_string(),
            },
            name
        )
    }
}

fn update_boards_list(
    _ev: On<UpdateBoardList>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    active_board: ActiveBoardProvider,
    defaults: Res<DefaultMaterials>,
    defaults_assets: Res<DefaultAssets>,
    boards: Res<SudokuBoardResources>,
    boards_state: Res<BoardsStateMap>,
    info_blocks: Query<Entity, With<BoardInfoBlock>>,
    rnb_block: Query<Entity, With<RequestNewBoardBlock>>,
    rnbv_block: Query<Entity, With<RequestNewBoardVisual>>,
    visuals: Query<&SudokuBoardVisual>,
) {
    if let Ok(rnb) = rnb_block.single() {
        commands.entity(rnb).despawn();
    }

    if let Ok(rnbv) = rnbv_block.single() {
        commands.entity(rnbv).despawn();
    }

    for block in info_blocks.iter() {
        commands.entity(block).despawn();
    }

    let mut font = TextFont {
        font: defaults_assets.default_font.clone(),
        font_size: 20.,
        ..default()
    };

    let mut latest_y = 286. + 55.;

    let mut items = boards.boards.iter().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0);

    for (diff, boards) in items {
        for (index, _) in boards.iter().enumerate() {
            let id = BoardId::new(*diff, index);
            let selected = active_board
                .active_board()
                .is_some_and(|active_board| active_board == &id);
            let finished = boards_state
                .get(&id)
                .is_some_and(|f| f.is_finished_verified());
            let visual_info = active_board
                .boards_mapping
                .iter()
                .find(|(_, v)| v == &&id)
                .and_then(|(e, _)| visuals.get(*e).ok());

            latest_y -= 55.;
            let mut spawned = commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(180., 50.))),
                MeshMaterial2d(defaults.default_foundation_block_color.clone()),
                Transform::from_translation(Vec3::default().with_xy(vec2(-420., latest_y))),
                Pickable::default(),
                BoardInfoBlock(BoardId::new(*diff, index)),
            ));

            spawned
                .observe(board_info_block_clicked)
                .observe(board_info_block_over)
                .observe(on_pointer_out)
                .with_children(|builder| {
                    builder
                        .spawn((
                            Mesh2d(meshes.add(Rectangle::new(170., 45.))),
                            MeshMaterial2d(if selected {
                                defaults.selected_resolving_block_color.clone()
                            } else if finished {
                                defaults.default_solved_block_color.clone()
                            } else {
                                defaults.default_block_color.clone()
                            }),
                            Transform::from_translation(Vec3::Z),
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text2d::new(format!(
                                    "#{} {}",
                                    index + 1,
                                    match diff {
                                        Some(diff) => diff.to_string(),
                                        None => "Unspecified".to_string(),
                                    },
                                )),
                                font.clone(),
                                TextColor(defaults.default_base_text_color),
                                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                                Transform::from_translation(Vec3::Z),
                            ));
                        });
                });

            if let Some(visual_info) = visual_info {
                spawned.with_child((
                    TextBundle::new(
                        visual_info.name.to_string(),
                        font.font.clone(),
                        17.,
                        WHITE,
                        Transform::default().with_translation(Vec3::default().with_x(-95.)),
                    ),
                    Anchor::CENTER_RIGHT,
                ));
            }
        }
    }

    font.font_size = 40.;
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(50., 50.))),
            MeshMaterial2d(defaults.default_foundation_block_color.clone()),
            Transform::from_translation(Vec3::default().with_xy(vec2(-367., latest_y - 55.))),
            Pickable::default(),
            RequestNewBoardBlock,
        ))
        .observe(request_new_board)
        .observe(request_new_board_over)
        .observe(on_pointer_out)
        .with_children(|builder| {
            builder
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(45., 45.))),
                    MeshMaterial2d(defaults.default_block_color.clone()),
                    Transform::from_translation(Vec3::Z),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text2d::new("+".to_string()),
                        font.clone(),
                        TextColor(defaults.default_base_text_color),
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        Transform::from_translation(Vec3::Z),
                    ));
                });
        });

    font.font_size = 20.;
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(100., 50.))),
            MeshMaterial2d(defaults.default_foundation_block_color.clone()),
            Transform::from_translation(Vec3::default().with_xy(vec2(-447., latest_y - 55.))),
            Pickable::default(),
            RequestNewBoardVisual,
        ))
        .observe(request_new_board_visual)
        .observe(request_new_board_visual_over)
        .observe(on_pointer_out)
        .with_children(|builder| {
            builder
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(95., 45.))),
                    MeshMaterial2d(defaults.default_block_color.clone()),
                    Transform::from_translation(Vec3::Z),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text2d::new("New city".to_string()),
                        font,
                        TextColor(defaults.default_base_text_color),
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        Transform::from_translation(Vec3::Z),
                    ));
                });
        });
}

#[derive(Debug, Component)]
struct RequestNewBoardVisual;

const AVAILABLE_BOARD_POSITIONS: [Vec2; 5] = [
    vec2(0., 50.),
    vec2(650., -600.),
    vec2(-650., -350.),
    vec2(845., 70.),
    vec2(0., -740.),
];

fn request_new_board_visual(
    _ev: On<Pointer<Click>>,
    mut commands: Commands,
    system_id: Res<CreateBoardVisualSystemId>,
    visuals: Query<&SudokuBoardVisual>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    let visual_count = visuals.count();
    if visual_count >= 5 {
        help_text.0 = "Can't have more than 5 visuals at the same time".to_string();
        return;
    }

    let pos = AVAILABLE_BOARD_POSITIONS[visual_count];
    commands.run_system_with(system_id.0, pos);
}

fn request_new_board_visual_over(
    _ev: On<Pointer<Over>>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    help_text.0 = "Will create a new board visual.".to_string();
}

fn request_new_board(
    _ev: On<Pointer<Click>>,
    mut commands: Commands,
    mut boards: ResMut<SudokuBoardResources>,
    mut snapshots: ResMut<SudokuBoardSnapshotResources>,
    mut boards_state: ResMut<BoardsStateMap>,
    mut active_board: ActiveBoardProviderMut,
    mut active_board_changed: ResMut<ActiveBoardChanged>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    let default = boards.boards.entry(None).or_default();

    if default.len() >= 3 {
        help_text.0 = "Can't add more than 3.".to_string();
        return;
    }

    default.push(Default::default());
    let default_snapshots = snapshots.boards.entry(None).or_default();
    default_snapshots.push(Default::default());

    let id = BoardId::new(None, default.len() - 1);
    boards_state.boards.insert(id, Default::default());
    active_board.update_active_board(id);

    active_board_changed.0 = true;

    commands.trigger(UpdateBoardList);
}

fn board_info_block_clicked(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    mut active_board: ActiveBoardProviderMut,
    mut active_board_changed: ResMut<ActiveBoardChanged>,
    board_info_block: Query<&BoardInfoBlock>,
) {
    if let Ok(block_info) = board_info_block.get(event.entity) {
        active_board.update_active_board(block_info.0);
        active_board_changed.0 = true;
        commands.trigger(UpdateBoardList);
    }
}

fn board_info_block_over(
    event: On<Pointer<Over>>,
    board_info_block: Query<&BoardInfoBlock>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    if let Ok(block_info) = board_info_block.get(event.entity) {
        help_text.0 = format!(
            "Will switch to board: #{} {}.",
            block_info.index + 1,
            match block_info.difficulty {
                Some(diff) => diff.to_string(),
                None => "Unspecified".to_string(),
            },
        );
    }
}

fn request_new_board_over(
    _ev: On<Pointer<Over>>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    help_text.0 = "Will create a new board.".to_string();
}

fn on_pointer_out(_ev: On<Pointer<Out>>, mut help_text: Single<&mut Text2d, With<HelpText>>) {
    help_text.0 = DEFAULT_HELP_TEXT.to_string();
}

fn check_foundation_squares(query: Query<(Entity, &SquareSpawnInfo), With<Foundation>>) {
    log::info!("Foundation squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        log::info!("{}- {:?}", i + 1, index.index)
    }
}

fn check_block_squares(query: Query<(Entity, &SquareIndex), With<Block>>) {
    log::info!("Block squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        log::info!("{}- {:?} (index: {:?})", i + 1, index, index.actual_index())
    }
}

// fn get_board_block_mut<'w, 's, D: QueryData>(
//     blocks: &'w mut Query<'w, 's, D, With<Block>>,
//     access_infos: &BlocksAccessInfo,
//     block_index: &BlockIndex,
// ) -> Option<D::Item<'w, 's>>
// where
//     's: 'w,
// {
//     let block_entity = access_infos.get(&block_index)?;
//     blocks.get_mut(*block_entity).ok()
// }

fn update_mistakes_text(
    stats: Res<Stats>,
    mut mistakes_text: Single<&mut TextSpan, With<MistakesCountText>>,
) {
    mistakes_text.0 = format!(
        "{} (Numbers) / {} (Possibilities)",
        stats.mistakes, stats.possibility_mistakes
    );
}

fn on_should_update_event(
    event: On<ShouldUpdateEvent>,
    mut should_updates: ResMut<ShouldUpdateAnyway>,
) {
    match event.event() {
        ShouldUpdateEvent::Clear => {
            should_updates.clear();
        }
        ShouldUpdateEvent::Add(block_index) => {
            if !should_updates.contains(block_index) {
                should_updates.push(block_index.clone());
            }
        }
        ShouldUpdateEvent::Remove(block_index) => {
            if should_updates.contains(block_index) {
                should_updates.retain(|x| x != block_index);
            }
        }
        ShouldUpdateEvent::AddMany(items) => {
            for block_index in items {
                if !should_updates.contains(block_index) {
                    should_updates.push(block_index.clone());
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Event)]
enum ShouldUpdateEvent {
    Clear,
    Add(BlockIndex),
    AddMany(Vec<BlockIndex>),
    Remove(BlockIndex),
}

//TODO - Update later to reflect many `SudokuBoardMarker`s.
#[derive(Debug, Resource, Deref, DerefMut, Default)]
struct ShouldUpdateAnyway(Vec<BlockIndex>);

fn update_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    defaults: Res<DefaultMaterials>,
    defaults_assets: Res<DefaultAssets>,
    strategy_colors: Res<StrategyMarkerColors>,
    active_board: ActiveBoardProvider,
    mut active_board_changed: ResMut<ActiveBoardChanged>,
    boards: Res<SudokuBoardResources>,
    boards_state: Res<BoardsStateMap>,
    mut snapshots: ResMut<SudokuBoardSnapshotResources>,
    mut blocks: Query<(Entity, &SquareSpawnInfo, &mut MeshMaterial2d<ColorMaterial>), With<Block>>,
    board_visuals: Query<&BlocksAccessInfo, With<SudokuBoardVisual>>,
    should_updates: Res<ShouldUpdateAnyway>,
    selected: Res<SelectedBlock>,
) {
    #[cfg(debug_assertions)]
    println!("Board needs update!");

    let mut snapshot_should_update = false;
    let text_justification = Justify::Center;
    let mut text_font = TextFont {
        font: defaults_assets.default_font.clone(),
        ..default()
    };

    let board_visual = if let Ok(board_visual) = board_visuals.get(**active_board.active_visual.0) {
        board_visual
    } else {
        return;
    };

    let active_board = if let Some(active_board) = active_board.active_board() {
        active_board
    } else {
        return;
    };

    for (row, col) in SudokuNumber::iter_numbers() {
        let block_index = BlockIndex::new(row, col);
        let block = boards.active_board(active_board).get_block(&block_index);
        let snapshot_block = snapshots.active_board(active_board).get_block(&block_index);

        if (block != snapshot_block)
            || active_board_changed.0
            || should_updates.contains(&block_index)
        {
            #[cfg(debug_assertions)]
            println!("Block {:?} seems changed.", &block_index);

            snapshot_should_update = true;

            let block_entity = board_visual.get(&block_index);
            if let Some((entity, spawn_info, mut material)) =
                block_entity.and_then(|e| blocks.get_mut(*e).ok())
            {
                let (j, i) = block_index.actual_indexes();

                // Update blocks based on finished or not
                let board_state = boards_state.boards.get(active_board);
                if let Some(state) = board_state
                    && selected.current != (i, j)
                {
                    #[cfg(debug_assertions)]
                    println!("Board has state.");
                    match (state, &block.status) {
                        (BoardState::FinishedVerified, SudokuBlockStatus::Resolved(_)) => {
                            material.0 = defaults.default_solved_block_color.clone();
                        }
                        (_, _) => {
                            #[cfg(debug_assertions)]
                            println!("Playing.");
                            material.0 = defaults.default_block_color.clone();
                        }
                    }
                }

                // Regular block update.
                if block.status != snapshot_block.status || active_board_changed.0 {
                    commands.entity(entity).despawn_children();
                    match &block.status {
                        SudokuBlockStatus::Unresolved => (),
                        SudokuBlockStatus::Fixed(sudoku_number)
                        | SudokuBlockStatus::Resolved(sudoku_number) => {
                            text_font.font_size = spawn_info.width;
                            commands.entity(entity).with_child((
                                Text2d::new(format!("{}", sudoku_number.to_u8())),
                                text_font.clone(),
                                TextColor(
                                    if matches!(&block.status, SudokuBlockStatus::Fixed(_)) {
                                        defaults.default_fixed_number_color
                                    } else {
                                        defaults.default_resolved_number_color
                                    },
                                ),
                                TextLayout::new_with_justify(text_justification),
                            ));
                        }
                        SudokuBlockStatus::Possibilities(possibilities) => {
                            commands.entity(entity).with_children(|builder| {
                                let master_index = spawn_info.index;
                                let width = spawn_info.width;

                                let numbers = possibilities
                                    .numbers
                                    .iter()
                                    .map(|f| {
                                        let indexed = f.to_index();
                                        let i = (indexed) / 3;
                                        let j = (indexed) % 3;
                                        (f, indexed + 1, i, j)
                                    })
                                    .collect::<Vec<_>>();

                                for spawn_info in square_group_info(width, 2., Default::default()) {
                                    if let Some((the_number, number, _, _)) = numbers
                                        .iter()
                                        .find(|(_, _, i, j)| spawn_info.index == (*j, *i))
                                    {
                                        text_font.font_size = spawn_info.width;

                                        builder.spawn((
                                            SquareBundle::new(
                                                if possibilities
                                                    .is_conflicting((*number).try_into().unwrap())
                                                {
                                                    defaults.conflicting_source_color.clone()
                                                } else if let Some(strategy) =
                                                    possibilities.has_strategy_effect(the_number)
                                                {
                                                    if strategy.is_effected() {
                                                        defaults.strategy_effected_color.clone()
                                                    } else if let Some(color) =
                                                        strategy_colors.get(&strategy.strategy())
                                                    {
                                                        color.background.clone()
                                                    } else {
                                                        defaults.strategy_source_color.clone()
                                                    }
                                                } else {
                                                    defaults
                                                        .default_possibilities_block_color
                                                        .clone()
                                                },
                                                &mut meshes,
                                                spawn_info,
                                                Some(master_index),
                                            ),
                                            Possibilities,
                                            children![(
                                                Text2d::new(format!("{}", number)),
                                                text_font.clone(),
                                                TextColor({
                                                    if let Some(strategy) = possibilities
                                                        .has_strategy_effect(the_number)
                                                    {
                                                        if strategy.is_source() {
                                                            if let Some(color) = strategy_colors
                                                                .get(&strategy.strategy())
                                                            {
                                                                color.text
                                                            } else {
                                                                defaults.strategy_source_text_color
                                                            }
                                                        } else {
                                                            defaults
                                                                .default_possibility_number_color
                                                        }
                                                    } else {
                                                        defaults.default_possibility_number_color
                                                    }
                                                },),
                                                TextLayout::new_with_justify(text_justification,),
                                            )],
                                        ));
                                    }
                                }
                            });
                        }
                    }
                }

                // Update blocks based on conflicts.
                if (block.conflicting != snapshot_block.conflicting || active_board_changed.0)
                    && selected.current != (i, j)
                {
                    match &block.conflicting {
                        Some(sudoku_solver::Conflicting::AffectedBy(_)) => {
                            material.0 = defaults.conflicting_affected_color.clone();
                        }
                        Some(sudoku_solver::Conflicting::Source) => {
                            material.0 = defaults.conflicting_source_color.clone();
                        }
                        Some(sudoku_solver::Conflicting::AffectedByPossibilities { .. }) => {
                            material.0 = defaults.conflicting_affected_color.clone();
                        }
                        _ => {}
                    }
                }

                if selected.current == (i, j) {
                    material.0 = match selected.mode {
                        SelectionMode::Resolving => defaults.selected_resolving_block_color.clone(),
                        SelectionMode::Possibilities => {
                            defaults.selected_possibilities_block_color.clone()
                        }
                    };
                }
            }

            #[cfg(debug_assertions)]
            println!("Updated ({:?}, {:?})", row, col);
        }
    }

    if !should_updates.is_empty() {
        commands.trigger(ShouldUpdateEvent::Clear);
    }
    active_board_changed.0 = false;
    if snapshot_should_update {
        *snapshots.active_board_mut(active_board) = boards.active_board(active_board).clone();
    }
}

fn final_verification(
    mut commands: Commands,
    active_board: ActiveBoardProvider,
    boards: Res<SudokuBoardResources>,
    mut boards_state: ResMut<BoardsStateMap>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    let active_board = if let Some(active_board) = active_board.active_board() {
        active_board
    } else {
        return;
    };

    let board = boards.active_board(active_board);
    if board
        .get_blocks()
        .filter(|f| f.is_unresolved() || f.is_possibilities())
        .count()
        == 0
    {
        if board.verify_board() {
            if let Some(state) = boards_state.boards.get_mut(active_board) {
                *state = BoardState::FinishedVerified;
            }

            help_text.0 =
                "The sudoku board solved successfully!\nYou can try resetting.".to_string();
            commands.trigger(ShouldUpdateEvent::AddMany(
                (board.get_blocks().filter_resolved().map(|f| f.index()))
                    .cloned()
                    .collect(),
            ));

            #[cfg(debug_assertions)]
            println!("Sudoku solved successfully!");
        } else {
            #[cfg(debug_assertions)]
            println!("Sudoku has mistakes!");
        }
    }
}

//TODO -
// let index = index.actual_index();
// let index = BlockIndex::from_index(index.1, index.0).unwrap();
// let block = board.get_block(&index);

// let text = format!(
//     "This is block {:?}. {}",
//     (index.actual_indexes()),
//     match &block.status {
//         SudokuBlockStatus::Unresolved => format!(
//             "This block is empty, Use number to resolve or put a possibility onto it"
//         ),
//         SudokuBlockStatus::Fixed(sudoku_number) => format!(
//             "This is a fixed block with number {}. This means you can't mess around with this one.",
//             sudoku_number.to_u8()
//         ),
//         SudokuBlockStatus::Resolved(sudoku_number) => format!(
//             "The number {} is placed here. {}",
//             sudoku_number.to_u8(),
//             match &block.conflicting {
//                 Some(conflicting) => match conflicting {
//                     Conflicting::AffectedBy(_) => format!(""),
//                     Conflicting::AffectedByPossibilities {
//                         block_index: _,
//                         number: _,
//                     } => format!(""),
//                     Conflicting::Source =>
//                         format!("But this number you out in here caused conflicting."),
//                 },
//                 None =>
//                     format!("The number is currently ok, but you can always change it."),
//             }
//         ),
//         SudokuBlockStatus::Possibilities(_) =>
//             format!("This is block of possibilities. (Quantum block!)"),
//     }
// );

#[derive(Debug, Component)]
struct Foundation;

#[derive(Debug, Component)]
struct Block;

#[derive(Debug, Component)]
struct Possibilities;

#[derive(Debug, Component)]
struct RelatedBoardVisual(Entity);

fn on_block_clicked(
    over: On<Pointer<Click>>,
    mut commands: Commands,
    indexes: Query<(&RelatedBoardVisual, &SquareIndex), With<Block>>,
    mut selected: ResMut<SelectedBlock>,
    active_visual: Option<ResMut<ActiveBoardVisual>>,
    mut active_board_changed: ResMut<ActiveBoardChanged>,
) {
    if let Ok((related_visual, index)) = indexes.get(over.entity) {
        if let Some(mut visual_id) = active_visual {
            if visual_id.0 != related_visual.0 {
                visual_id.0 = related_visual.0;
                active_board_changed.0 = true;
            }
        } else {
            return;
        };

        let index = index.actual_index();
        if selected.current != index || active_board_changed.0 {
            let pervious_selection = selected.block_index();
            selected.current = index;
            commands.trigger(ShouldUpdateEvent::AddMany(vec![
                selected.block_index(),
                pervious_selection,
            ]));
        }
    }
}

fn on_helper_block_clicked(
    over: On<Pointer<Click>>,
    mut commands: Commands,
    indexes: Query<&HelperBlock>,
) {
    if let Ok(block) = indexes.get(over.entity) {
        commands.trigger(GameInputs::new(block.command_type));
    }
}

fn on_game_input(
    input: On<GameInputs>,
    mut commands: Commands,
    active_board: ActiveBoardProvider,
    mut boards: ResMut<SudokuBoardResources>,
    mut boards_state: ResMut<BoardsStateMap>,
    mut stats: ResMut<Stats>,
    mut selected: ResMut<SelectedBlock>,
    mut engaging: ResMut<EngagingStrategyMap>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    let active_board = if let Some(active_board) = active_board.active_board() {
        active_board
    } else {
        return;
    };

    let board = boards.active_board_mut(active_board);
    let board_state = boards_state.boards.get_mut(active_board);

    match input.event().command_type() {
        CommandType::Number(sudoku_number) => {
            if board_state.is_some_and(|f| matches!(f, BoardState::FinishedVerified)) {
                // Board in finished state do nothing.
                return;
            }

            let block_index =
                BlockIndex::from_index(selected.current.1, selected.current.0).unwrap();
            let block = board.get_block_mut(&block_index);

            match &block.status {
                SudokuBlockStatus::Fixed(_) => (),
                _ => {
                    let update_result = Some(_update_block(&selected, block, sudoku_number));

                    if let Some(result) = update_result {
                        match result {
                            BlockUpdateResult::Cleared => {
                                board.mark_conflicts(&block_index, None);
                            }
                            BlockUpdateResult::Resolved => {
                                board.mark_conflicts(&block_index, None);

                                let block = board.get_block(&block_index);
                                if block
                                    .conflicting
                                    .as_ref()
                                    .is_some_and(|f| matches!(f, Conflicting::Source))
                                {
                                    // This is a mistake!
                                    stats.mistakes += 1;
                                    #[cfg(debug_assertions)]
                                    println!("This is a mistake!")
                                }
                            }
                            BlockUpdateResult::Possible { number, is_cleared } => {
                                board.mark_conflicts(&block_index, Some((number, is_cleared)));

                                let block = board.get_block(&block_index);
                                let poss = block.status.as_possibilities().unwrap(); // This must be possibilities

                                if poss.is_conflicting(number) {
                                    // This is also a mistake
                                    stats.possibility_mistakes += 1;
                                    #[cfg(debug_assertions)]
                                    println!("This is also a mistake!")
                                }
                            }
                        }
                    }
                }
            }
        }
        CommandType::CalculatePossibilities => {
            if board_state.is_some_and(|f| matches!(f, BoardState::FinishedVerified)) {
                // Board in finished state do nothing.
                return;
            }

            #[cfg(debug_assertions)]
            println!("Updating possibilities.");
            board.update_possibilities();
        }
        CommandType::ResolveNakedSingles => {
            if board_state.is_some_and(|f| matches!(f, BoardState::FinishedVerified)) {
                // Board in finished state do nothing.
                return;
            }

            #[cfg(debug_assertions)]
            println!("Resolving satisfied blocks (Naked single).");
            board.resolve_satisfied_blocks();
        }
        CommandType::Reset => {
            #[cfg(debug_assertions)]
            println!("Resetting.");
            board.reset();

            if let Some(state) = board_state {
                *state = BoardState::Playing;
            }

            help_text.0 = DEFAULT_HELP_TEXT.to_string();
        }
        CommandType::ChangeSelectionMode => {
            selected.mode = match selected.mode {
                SelectionMode::Resolving => SelectionMode::Possibilities,
                SelectionMode::Possibilities => SelectionMode::Resolving,
            };
        }
        CommandType::ClearBlock => {
            if board_state.is_some_and(|f| matches!(f, BoardState::FinishedVerified)) {
                // Board in finished state do nothing.
                return;
            }

            let block_index =
                BlockIndex::from_index(selected.current.1, selected.current.0).unwrap();
            let block = board.get_block_mut(&block_index);

            match &block.status {
                SudokuBlockStatus::Fixed(_) => (),
                _ => {
                    block.status = SudokuBlockStatus::Unresolved;
                    board.mark_conflicts(&block_index, None);
                }
            }
        }
        CommandType::Direction(direction) => {
            let pervious_selection = selected.block_index();

            match direction {
                Direction::Up => {
                    if selected.current.1 > 0 {
                        selected.current.1 -= 1;
                    } else {
                        selected.current.1 = 8;
                    }
                }
                Direction::Down => {
                    if selected.current.1 < 8 {
                        selected.current.1 += 1;
                    } else {
                        selected.current.1 = 0;
                    }
                }
                Direction::Left => {
                    if selected.current.0 > 0 {
                        selected.current.0 -= 1;
                    } else {
                        selected.current.0 = 8;
                    }
                }
                Direction::Right => {
                    if selected.current.0 < 8 {
                        selected.current.0 += 1;
                    } else {
                        selected.current.0 = 0;
                    }
                }
            };

            commands.trigger(ShouldUpdateEvent::AddMany(vec![
                selected.block_index(),
                pervious_selection,
            ]));
        }
        CommandType::Strategy(strategy) => {
            if board_state.is_some_and(|f| matches!(f, BoardState::FinishedVerified)) {
                // Board in finished state do nothing.
                return;
            }

            let mut show_only_effect = false;
            let engaging = engaging.engaging.entry(*active_board).or_default();

            if engaging.strategy.is_some_and(|f| f == strategy) {
                // This strategy was engaged before
                if engaging.showed_effect {
                    // The effects shown already! Time to take action.
                    engaging.showed_effect = false;
                } else {
                    show_only_effect = true;
                    engaging.showed_effect = true;
                }
            } else {
                // This is a new strategy marker.
                board.clear_strategy_markers();
                show_only_effect = true;
                engaging.showed_effect = true;
                engaging.strategy = Some(strategy);
            }

            match strategy {
                Strategy::HiddenSingle => {
                    #[cfg(debug_assertions)]
                    println!("Engaging Hidden single Strategy.");
                    board.engage_strategy(HiddenSingleStrategy, show_only_effect);
                }
                Strategy::NakedPair => {
                    #[cfg(debug_assertions)]
                    println!("Engaging Naked pair Strategy.");
                    board.engage_strategy(NakedPairStrategy, show_only_effect);
                }
                _ => (),
            }
        }
    }
}

fn on_helper_block_hovered(
    over: On<Pointer<Over>>,
    indexes: Query<&HelperBlock>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    if let Ok(block) = indexes.get(over.entity) {
        let new_help_text = match &block.command_type {
            CommandType::Number(sudoku_number) => {
                format!("Puts number {} in selected block.", (sudoku_number.to_u8()))
            }
            CommandType::CalculatePossibilities => {
                "Updates possible values based on currently filled blocks.".to_string()
            }
            CommandType::ResolveNakedSingles => {
                "Resolve blocks that are naked singles (Blocks with only one possible number)."
                    .to_string()
            }
            CommandType::Reset => "Reset the whole board (Use this if nothing works).".to_string(),
            CommandType::ChangeSelectionMode => {
                "Changes selection mode. Putting numbers or possibilities.".to_string()
            }
            CommandType::ClearBlock => "Clears currently selected block.".to_string(),
            CommandType::Direction(direction) => {
                format!(
                    "Moves the selector to the {} by one block.",
                    match direction {
                        Direction::Up => "Top",
                        Direction::Down => "Bottom",
                        Direction::Left => "Left",
                        Direction::Right => "Right",
                    }
                )
            }
            CommandType::Strategy(strategy) => {
                format!(
                    "Applying strategy: {}",
                    match strategy {
                        Strategy::HiddenSingle => "Hidden singles.",
                        Strategy::NakedSingle => "Naked single",
                        Strategy::NakedPair => "Naked pair",
                    }
                )
            }
        };

        help_text.0 = new_help_text;
    }
}

#[derive(Debug, Bundle)]
struct SquareBundle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    index: SquareIndex,
    spawn_info: SquareSpawnInfo,
}

impl SquareBundle {
    fn new(
        color: Handle<ColorMaterial>,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        spawn_info: SquareSpawnInfo,
        master_index: Option<(usize, usize)>,
    ) -> Self {
        Self {
            mesh: Mesh2d(meshes.add(Rectangle::new(spawn_info.width, spawn_info.width))),
            material: MeshMaterial2d(color),
            transform: Transform::default().with_translation(Vec3 {
                x: spawn_info.translation.x,
                y: spawn_info.translation.y,
                z: 1.,
            }),
            index: SquareIndex::new(spawn_info.index.0, spawn_info.index.1, master_index),
            spawn_info,
        }
    }
}

#[derive(Debug, Clone, Component)]
struct SquareSpawnInfo {
    width: f32,
    translation: Vec2,
    index: (usize, usize),
}

fn square_group_info(
    width: f32,
    offset: f32,
    center_translation: Vec2,
) -> impl Iterator<Item = SquareSpawnInfo> {
    (0..3).flat_map(move |i| {
        (0..3).map(move |j| {
            let i_f32 = i as f32;
            let j_f32 = j as f32;

            let center = center_translation;
            let width = (width - 4. * offset) / 3.;

            SquareSpawnInfo {
                translation: Vec2 {
                    x: (i_f32 * (width + offset) - width) + center.x - offset,
                    y: -((j_f32 * (width + offset) - width) + center.y - offset),
                },
                width,
                index: (i, j),
            }
        })
    })
}

fn spawn_sudoku_board_visual(
    center: In<Vec2>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    defaults: Res<DefaultMaterials>,
    defaults_assets: Res<DefaultAssets>,
    visuals: Query<&SudokuBoardVisual>,
    help_text: Option<Single<&mut Text2d, With<HelpText>>>,
) {
    let width = 630.;
    let top_padding = 15.;
    let mut access_infos: HashMap<BlockIndex, Entity> = Default::default();

    let existing_names = visuals.iter().map(|f| &f.name).collect::<Vec<_>>();

    if existing_names.len() >= 5 {
        if let Some(mut help_text) = help_text {
            help_text.0 = "To many visuals!".to_string();
        }
        return;
    }

    let mut name = gen_random_city_name();
    while existing_names.contains(&&name) {
        name = gen_random_city_name()
    }

    let mut spawned = commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(width, width + top_padding))),
        MeshMaterial2d(defaults.default_deactivate_board_color.clone()),
        Transform::default().with_translation(Vec3 {
            x: center.x,
            y: center.y,
            ..Default::default()
        }),
        SudokuBoardVisual { name: name.clone() },
        Pickable::default(),
    ));

    let visual_id = spawned.id();

    spawned.with_children(|builder| {
        for spawn_info in square_group_info(width, 5., Vec2::default().with_y(top_padding - 5.)) {
            builder
                .spawn((
                    SquareBundle::new(
                        defaults.default_foundation_block_color.clone(),
                        &mut meshes,
                        spawn_info.clone(),
                        None,
                    ),
                    Foundation,
                ))
                .with_children(|builder| {
                    let width = spawn_info.width;
                    let master_index = spawn_info.index;

                    for spawn_info in square_group_info(width, 5., Default::default()) {
                        let bundle = SquareBundle::new(
                            defaults.default_block_color.clone(),
                            &mut meshes,
                            spawn_info.clone(),
                            Some(master_index),
                        );

                        let block_index = bundle.index.block_index();
                        let entity = builder
                            .spawn((
                                bundle,
                                Block,
                                RelatedBoardVisual(visual_id),
                                Pickable::default(),
                            ))
                            .observe(on_block_clicked)
                            .id();

                        access_infos.insert(block_index, entity);
                    }
                });
        }
    });

    spawned.with_child((
        TextBundle::new(
            name,
            defaults_assets.default_font.clone(),
            17.,
            WHITE,
            Transform::default().with_translation(Vec3::default().with_xy(vec2(-305., 310.))),
        ),
        Anchor::CENTER_LEFT,
    ));

    // Close btn
    spawned.with_children(|builder| {
        builder
            .spawn((
                Mesh2d(meshes.add(Rectangle::new(30., 30.))),
                MeshMaterial2d(defaults.default_foundation_block_color.clone()),
                Transform::from_translation(Vec3::default().with_xy(vec2(290., 320.))),
                Pickable::default(),
                DeleteBoardVisual(visual_id),
                children![(
                    Mesh2d(meshes.add(Rectangle::new(25., 25.))),
                    MeshMaterial2d(defaults.default_block_color.clone()),
                    Transform::from_translation(Vec3::Z),
                    children![TextBundle::new(
                        "x",
                        defaults_assets.default_font.clone(),
                        30.,
                        BLACK,
                        Transform::from_translation(Vec3::default().with_y(1.)),
                    )]
                )],
            ))
            .observe(delete_board_visual)
            .observe(delete_board_visual_over)
            .observe(on_pointer_out);
    });

    spawned.insert(BlocksAccessInfo::new(access_infos));
}

#[derive(Debug, Component)]
struct DeleteBoardVisual(Entity);

fn delete_board_visual(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    delete_btns: Query<&DeleteBoardVisual>,
    visuals: Query<Entity, With<SudokuBoardVisual>>,
    active_visual: Option<ResMut<ActiveBoardVisual>>,
    mut active_board_mapping: ResMut<ActiveBoardsMapping>,
    mut active_board_changed: ResMut<ActiveBoardChanged>,
) {
    if let Ok(delete_btn) = delete_btns.get(ev.entity) {
        commands.entity(delete_btn.0).despawn();

        active_board_mapping.0.remove(&delete_btn.0);

        if let Some(mut active_visual) = active_visual
            && active_visual.0 == delete_btn.0
        {
            if let Some(other_visual) = visuals.iter().next() {
                active_visual.0 = other_visual;
                active_board_changed.0 = true;
            } else {
                commands.remove_resource::<ActiveBoardVisual>();
            }
        }
    }
}

fn delete_board_visual_over(
    _ev: On<Pointer<Over>>,
    mut help_text: Single<&mut Text2d, With<HelpText>>,
) {
    help_text.0 = "Will delete this board!".to_string();
}

// fn on_drag(event: On<Pointer<Drag>>) {}

enum BlockUpdateResult {
    Cleared,
    Resolved,
    Possible {
        number: SudokuNumber,
        is_cleared: bool,
    },
}

fn _update_block(
    selected: &SelectedBlock,
    block: &mut sudoku_solver::SudokuBlock,
    number: SudokuNumber,
) -> BlockUpdateResult {
    use BlockUpdateResult::*;

    match selected.mode {
        SelectionMode::Resolving => {
            if let SudokuBlockStatus::Resolved(already) = block.status
                && already == number
            {
                block.status = SudokuBlockStatus::Unresolved;
                return Cleared;
            }

            block.status = SudokuBlockStatus::Resolved(number);
            Resolved
        }
        SelectionMode::Possibilities => {
            if let Some(pos) = block.status.as_possibilities_mut() {
                if pos.numbers.has_number(number) {
                    pos.numbers.del_number(number);

                    if pos.numbers.count_numbers() == 0 {
                        block.status = SudokuBlockStatus::Unresolved;
                        Cleared
                    } else {
                        Possible {
                            number,
                            is_cleared: true,
                        }
                    }
                } else {
                    pos.numbers.set_number(number);
                    Possible {
                        number,
                        is_cleared: false,
                    }
                }
            } else {
                block.status = SudokuBlockStatus::Possibilities(SudokuPossibilities::new(
                    SudokuNumbers::new([number]),
                ));
                Possible {
                    number,
                    is_cleared: false,
                }
            }
        }
    }
}
