use std::time::Duration;

use bevy::{
    color::palettes::{
        basic::PURPLE,
        css::{BLACK, BLUE, RED, WHITE, YELLOW},
        tailwind::{BLUE_200, GRAY_600, RED_400, YELLOW_400},
    },
    input::common_conditions::{input_just_pressed, input_pressed},
    prelude::*,
};
use sudoku_bevy::pancam::{DirectionKeys, PanCam, PanCamPlugin};
use sudoku_solver::{
    BlockIndex, Conflicting, Possibilities as SudokuPossibilities, SudokuBlockStatus, SudokuBoard,
    numbers::{SudokuNumber, SudokuNumbers},
    strategies::hidden_single::HiddenSingleStrategy,
};

#[derive(Debug, Resource, Default)]
struct SudokuBoardResources {
    current: SudokuBoard,
    snapshot: SudokuBoard,
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

#[derive(Debug, Resource, Default)]
struct DefaultMaterials {
    // Handles
    default_foundation_block_color: Handle<ColorMaterial>,
    default_possibilities_block_color: Handle<ColorMaterial>,
    default_block_color: Handle<ColorMaterial>,
    selected_resolving_block_color: Handle<ColorMaterial>,
    selected_possibilities_block_color: Handle<ColorMaterial>,

    conflicting_source_color: Handle<ColorMaterial>,
    conflicting_affected_color: Handle<ColorMaterial>,

    // Colors
    default_base_text_color: Color,
    default_fixed_number_color: Color,
    default_resolved_number_color: Color,
    default_possibility_number_color: Color,
}

#[derive(Debug, Resource, Default)]
struct DefaultAssets {
    // Handles
    default_font: Handle<Font>,
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin::default()))
        .init_resource::<SudokuBoardResources>()
        .init_resource::<SelectedBlock>()
        .init_resource::<DefaultMaterials>()
        .init_resource::<DefaultAssets>()
        .init_resource::<Stats>()
        .insert_resource(ChangeSelectionTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(
            PostStartup,
            (check_foundation_squares, check_block_squares).chain(),
        )
        .add_systems(
            Update,
            (
                change_selected_block.run_if(
                    input_pressed(KeyCode::ArrowDown)
                        .or(input_pressed(KeyCode::ArrowUp))
                        .or(input_pressed(KeyCode::ArrowLeft))
                        .or(input_pressed(KeyCode::ArrowRight)),
                ),
                engage_strategy.run_if(input_just_pressed(KeyCode::KeyH)),
                update_possibilities.run_if(input_just_pressed(KeyCode::Space)),
                resolve_satisfied.run_if(input_just_pressed(KeyCode::Enter)),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
                change_selection_mode.run_if(input_just_pressed(KeyCode::KeyM)),
                manually_clear_block.run_if(input_just_pressed(KeyCode::KeyC)),
                manually_update_block.run_if(
                    input_just_pressed(KeyCode::Digit1)
                        .or(input_just_pressed(KeyCode::Digit2))
                        .or(input_just_pressed(KeyCode::Digit3))
                        .or(input_just_pressed(KeyCode::Digit4))
                        .or(input_just_pressed(KeyCode::Digit5))
                        .or(input_just_pressed(KeyCode::Digit6))
                        .or(input_just_pressed(KeyCode::Digit7))
                        .or(input_just_pressed(KeyCode::Digit8))
                        .or(input_just_pressed(KeyCode::Digit9)),
                ),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                update_selected_block.run_if(resource_changed::<SelectedBlock>),
                update_board.run_if(resource_changed::<SudokuBoardResources>),
                update_mistakes_text.run_if(resource_changed::<Stats>),
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut sudoku_board: ResMut<SudokuBoardResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut defaults: ResMut<DefaultMaterials>,
    mut defaults_assets: ResMut<DefaultAssets>,
    asset_server: Res<AssetServer>,
) {
    let mut ortho = OrthographicProjection::default_2d();
    ortho.scale = 1.5;

    commands.spawn((
        Camera2d,
        Projection::Orthographic(ortho),
        PanCam {
            grab_buttons: vec![MouseButton::Left], // which buttons should drag the camera
            move_keys: DirectionKeys {
                // the keyboard buttons used to move the camera
                up: vec![KeyCode::KeyW], // initalize the struct like this or use the provided methods for
                down: vec![KeyCode::KeyS], // common key combinations
                left: vec![KeyCode::KeyA],
                right: vec![KeyCode::KeyD],
            },
            min_scale: 1., // prevent the camera from zooming too far in
            max_scale: 5., // prevent the camera from zooming too far out
            min_x: -1500., // minimum x position of the camera window
            max_x: 1500.,  // maximum x position of the camera window
            min_y: -1500., // minimum y position of the camera window
            max_y: 1500.,  // maximum y position of the camera window
            ..Default::default()
        },
    ));

    let center = vec2(0., -50.);
    let width = 630.;
    let offset = 5.;

    let board = &mut sudoku_board.current;

    board.fill_board_u8(sudoku_samples::easy::FIRST).unwrap();

    defaults.default_block_color = materials.add(Color::from(YELLOW));
    defaults.selected_resolving_block_color = materials.add(Color::from(YELLOW_400));
    defaults.selected_possibilities_block_color = materials.add(Color::from(BLUE_200));
    defaults.default_foundation_block_color = materials.add(Color::from(PURPLE));
    defaults.default_possibilities_block_color = materials.add(Color::from(BLUE));

    defaults.conflicting_source_color = materials.add(Color::from(RED));
    defaults.conflicting_affected_color = materials.add(Color::from(RED_400));

    defaults.default_base_text_color = Color::from(BLACK);
    defaults.default_fixed_number_color = Color::from(GRAY_600);
    defaults.default_possibility_number_color = Color::from(WHITE);
    defaults.default_resolved_number_color = Color::from(BLACK);

    spawn_sudoku_board(&mut commands, &mut meshes, &defaults, center, width, offset);

    #[cfg(target_arch = "wasm32")]
    let font = asset_server.load("assets/fonts/FiraSans-Bold.ttf");
    #[cfg(not(target_arch = "wasm32"))]
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    defaults_assets.default_font = font;

    commands.spawn((
        Text2d::new("Sudoku".to_string()),
        TextFont {
            font: defaults_assets.default_font.clone(),
            font_size: 100.,
            ..default()
        },
        TextColor(Color::from(WHITE)),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        Transform::default().with_translation(Vec3 {
            y: 450.,
            ..Default::default()
        }),
    ));

    commands
        .spawn((
            Text2d::new("Mistakes: "),
            TextFont {
                font: defaults_assets.default_font.clone(),
                font_size: 20.,
                ..default()
            },
            TextColor(Color::from(WHITE)),
            Transform::default().with_translation(Vec3 {
                y: 380.,
                ..Default::default()
            }),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextSpan::new("0 (Numbers) / 0 (Possibilities)"),
                TextFont {
                    font: defaults_assets.default_font.clone(),
                    font_size: 20.,
                    ..default()
                },
                TextColor(Color::from(RED)),
                // TextBackgroundColor(Color::from(WHITE)),
                MistakesCountText,
            ));
        });

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(620., 100.))),
            MeshMaterial2d(defaults.default_foundation_block_color.clone()),
            Transform::default().with_translation(Vec3 {
                y: -320.,
                ..Default::default()
            }),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(610., 90.))),
                    MeshMaterial2d(defaults.default_block_color.clone()),
                ))
                .with_children(|builder| {
                    let text_font = TextFont {
                        font: defaults_assets.default_font.clone(),
                        font_size: 17.,
                        ..default()
                    };

                    builder.spawn((
                        Text2d::new("Use 'Space' to update possible values, 'Enter' to resolve blocks,\n'R' to reset, 'M' to change selection mode, 'C' to clear block,\n1 to 9 to set number and 'H' to engage Hidden single strategy.".to_string()),
                        text_font,
                        TextColor(defaults.default_base_text_color),
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        // Wrap text in the rectangle
                        // TextBounds::from(Vec2::new(300., 500.)),
                    ));
                });
        });
}

fn check_foundation_squares(query: Query<(Entity, &SquareSpawnInfo), With<Foundation>>) {
    println!("Foundation squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        println!("{}- {:?}", i + 1, index.index)
    }
}

fn check_block_squares(query: Query<(Entity, &SquareIndex), With<Block>>) {
    println!("Block squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        println!("{}- {:?} (index: {:?})", i + 1, index, index.actual_index())
    }
}

fn update_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    defaults: Res<DefaultMaterials>,
    defaults_assets: Res<DefaultAssets>,
    mut board: ResMut<SudokuBoardResources>,
    mut blocks: Query<
        (
            Entity,
            &SquareSpawnInfo,
            &SquareIndex,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Block>,
    >,
    selected: Res<SelectedBlock>,
) {
    let mut snapshot_should_update = false;

    let text_justification = Justify::Center;

    let mut text_font = TextFont {
        font: defaults_assets.default_font.clone(),
        ..default()
    };

    use SudokuNumber::*;
    for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
        for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            let block_index = BlockIndex::new(row, col);
            let block = board.current.get_block(&block_index);
            let snapshot_block = board.snapshot.get_block(&block_index);
            let (j, i) = block_index.actual_indexes();

            if block.status != snapshot_block.status {
                snapshot_should_update = true;

                if let Some((entity, spawn_info, _, _)) = blocks.iter().find(|(_, _, index, _)| {
                    let index = index.actual_index();
                    index.0 == i && index.1 == j
                }) {
                    commands.entity(entity).despawn_children();

                    match &block.status {
                        SudokuBlockStatus::Unresolved => (),
                        SudokuBlockStatus::Fixed(sudoku_number)
                        | SudokuBlockStatus::Resolved(sudoku_number) => {
                            text_font.font_size = spawn_info.width;

                            let child = commands
                                .spawn((
                                    Block,
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
                                ))
                                .id();
                            commands.entity(entity).add_child(child);
                        }
                        SudokuBlockStatus::Possibilities(sudoku_numbers) => {
                            commands.entity(entity).with_children(|builder| {
                                let master_index = spawn_info.index;
                                let width = spawn_info.width;

                                let numbers = sudoku_numbers
                                    .numbers
                                    .iter()
                                    .map(|f| f.to_index())
                                    .map(|f| {
                                        let i = (f) / 3;
                                        let j = (f) % 3;
                                        (f + 1, i, j)
                                    })
                                    .collect::<Vec<_>>();

                                for spawn_info in square_group_info(width, 2., Default::default()) {
                                    if let Some((number, _, _)) =
                                        numbers.iter().find(|(_, i, j)| {
                                            i == &spawn_info.index.1 && j == &spawn_info.index.0
                                        })
                                    {
                                        text_font.font_size = spawn_info.width;

                                        builder
                                            .spawn((
                                                SquareBundle::new(
                                                    if sudoku_numbers.is_conflicting(
                                                        (*number).try_into().unwrap(),
                                                    ) {
                                                        defaults.conflicting_source_color.clone()
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
                                            ))
                                            .with_children(|builder| {
                                                builder.spawn((
                                                    Text2d::new(format!("{}", number)),
                                                    text_font.clone(),
                                                    TextColor(
                                                        defaults.default_possibility_number_color,
                                                    ),
                                                    TextLayout::new_with_justify(
                                                        text_justification,
                                                    ),
                                                    Possibilities,
                                                ));
                                            });
                                    }
                                }
                            });
                        }
                    }
                }

                println!("Updated ({:?}, {:?})", row, col);
            }

            if block.conflicting != snapshot_block.conflicting && selected.current != (i, j) {
                if let Some((_, _, _, mut material)) = blocks.iter_mut().find(|(_, _, index, _)| {
                    let index = index.actual_index();
                    index.0 == i && index.1 == j
                }) {
                    match &block.conflicting {
                        Some(conflicting) => match conflicting {
                            sudoku_solver::Conflicting::AffectedBy(_) => {
                                material.0 = defaults.conflicting_affected_color.clone();
                            }
                            sudoku_solver::Conflicting::Source => {
                                material.0 = defaults.conflicting_source_color.clone();
                            }
                            sudoku_solver::Conflicting::AffectedByPossibilities { .. } => {
                                material.0 = defaults.conflicting_affected_color.clone();
                            }
                        },
                        None => {
                            material.0 = defaults.default_block_color.clone();
                        }
                    }
                }
            }
        }
    }

    if snapshot_should_update {
        board.snapshot = board.current.clone();
    }
}

#[derive(Resource)]
struct ChangeSelectionTimer(Timer);

fn change_selected_block(
    time: Res<Time>,
    mut timer: ResMut<ChangeSelectionTimer>,
    mut selected: ResMut<SelectedBlock>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if selected.current.0 > 0 {
                selected.current.0 -= 1;
            } else {
                selected.current.0 = 8;
            }
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            if selected.current.0 < 8 {
                selected.current.0 += 1;
            } else {
                selected.current.0 = 0;
            }
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            if selected.current.1 < 8 {
                selected.current.1 += 1;
            } else {
                selected.current.1 = 0;
            }
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            if selected.current.1 > 0 {
                selected.current.1 -= 1;
            } else {
                selected.current.1 = 8;
            }
        }
    }
}

fn update_selected_block(
    defaults: Res<DefaultMaterials>,
    selected: Res<SelectedBlock>,
    board: Res<SudokuBoardResources>,
    mut blocks: Query<(&SquareIndex, &mut MeshMaterial2d<ColorMaterial>), With<Block>>,
) {
    if let Some((_, mut material)) = blocks.iter_mut().find(|(index, _)| {
        let index = index.actual_index();
        index.0 == selected.current.0 && index.1 == selected.current.1
    }) {
        material.0 = match selected.mode {
            SelectionMode::Resolving => defaults.selected_resolving_block_color.clone(),
            SelectionMode::Possibilities => defaults.selected_possibilities_block_color.clone(),
        };
    }

    for (index, mut material) in blocks.iter_mut() {
        let index = index.actual_index();
        if index.0 == selected.current.0 && index.1 == selected.current.1 {
            continue;
        }

        if material.0.id() == defaults.selected_possibilities_block_color.id()
            || material.0.id() == defaults.selected_resolving_block_color.id()
        {
            let block = board
                .current
                .get_block(&BlockIndex::from_index(index.1, index.0).unwrap());

            match &block.conflicting {
                Some(conflicting) => match conflicting {
                    sudoku_solver::Conflicting::AffectedBy(_) => {
                        material.0 = defaults.conflicting_affected_color.clone();
                    }
                    sudoku_solver::Conflicting::Source => {
                        material.0 = defaults.conflicting_source_color.clone();
                    }
                    sudoku_solver::Conflicting::AffectedByPossibilities { .. } => {
                        material.0 = defaults.conflicting_affected_color.clone();
                    }
                },
                None => {
                    material.0 = defaults.default_block_color.clone();
                }
            }
        }
    }
}

fn update_possibilities(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Updating possibilities.");
        sudoku_board.current.update_possibilities();
    }
}

fn engage_strategy(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("Engaging Hidden single Strategy.");
        sudoku_board.current.engage_strategy(HiddenSingleStrategy);
    }
}

fn resolve_satisfied(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        println!("Resolving satisfied blocks (Naked single).");
        sudoku_board.current.resolve_satisfied_blocks();
    }
}

fn reset(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        println!("Resetting.");
        sudoku_board.current.reset();
    }
}

fn change_selection_mode(
    mut selected: ResMut<SelectedBlock>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        println!("Changing mode.");
        selected.mode = match selected.mode {
            SelectionMode::Resolving => SelectionMode::Possibilities,
            SelectionMode::Possibilities => SelectionMode::Resolving,
        };
    }
}

fn manually_clear_block(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedBlock>,
) {
    let block_index = BlockIndex::from_index(selected.current.1, selected.current.0).unwrap();
    let block = sudoku_board.current.get_block_mut(&block_index);

    match &block.status {
        SudokuBlockStatus::Fixed(_) => (),
        _ => {
            if keyboard_input.just_pressed(KeyCode::KeyC) {
                block.status = SudokuBlockStatus::Unresolved;

                sudoku_board.current.mark_conflicts(&block_index, None);
            }
        }
    }
}

fn manually_update_block(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    mut stats: ResMut<Stats>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedBlock>,
) {
    let block_index = BlockIndex::from_index(selected.current.1, selected.current.0).unwrap();
    let block = sudoku_board.current.get_block_mut(&block_index);

    // let digit_map = |digit: KeyCode| match digit {
    //     KeyCode::Digit1 => Some(SudokuNumber::One),
    //     KeyCode::Digit2 => Some(SudokuNumber::Two),
    //     KeyCode::Digit3 => Some(SudokuNumber::Three),
    //     KeyCode::Digit4 => Some(SudokuNumber::Four),
    //     KeyCode::Digit5 => Some(SudokuNumber::Five),
    //     KeyCode::Digit6 => Some(SudokuNumber::Six),
    //     KeyCode::Digit7 => Some(SudokuNumber::Seven),
    //     KeyCode::Digit8 => Some(SudokuNumber::Eight),
    //     KeyCode::Digit9 => Some(SudokuNumber::Nine),
    //     _ => None,
    // };

    match &block.status {
        SudokuBlockStatus::Fixed(_) => (),
        _ => {
            let update_result = if keyboard_input.just_pressed(KeyCode::Digit1) {
                Some(_update_block(&selected, block, SudokuNumber::One))
            } else if keyboard_input.just_pressed(KeyCode::Digit2) {
                Some(_update_block(&selected, block, SudokuNumber::Two))
            } else if keyboard_input.just_pressed(KeyCode::Digit3) {
                Some(_update_block(&selected, block, SudokuNumber::Three))
            } else if keyboard_input.just_pressed(KeyCode::Digit4) {
                Some(_update_block(&selected, block, SudokuNumber::Four))
            } else if keyboard_input.just_pressed(KeyCode::Digit5) {
                Some(_update_block(&selected, block, SudokuNumber::Five))
            } else if keyboard_input.just_pressed(KeyCode::Digit6) {
                Some(_update_block(&selected, block, SudokuNumber::Six))
            } else if keyboard_input.just_pressed(KeyCode::Digit7) {
                Some(_update_block(&selected, block, SudokuNumber::Seven))
            } else if keyboard_input.just_pressed(KeyCode::Digit8) {
                Some(_update_block(&selected, block, SudokuNumber::Eight))
            } else if keyboard_input.just_pressed(KeyCode::Digit9) {
                Some(_update_block(&selected, block, SudokuNumber::Nine))
            } else {
                None
            };

            match update_result {
                Some(result) => {
                    match result {
                        BlockUpdateResult::Cleared => {
                            sudoku_board.current.mark_conflicts(&block_index, None);
                        }
                        BlockUpdateResult::Resolved => {
                            sudoku_board.current.mark_conflicts(&block_index, None);

                            let block = sudoku_board.current.get_block(&block_index);
                            if block
                                .conflicting
                                .as_ref()
                                .is_some_and(|f| matches!(f, Conflicting::Source))
                            {
                                // This is a mistake!
                                stats.mistakes += 1;
                                println!("This is a mistake!")
                            }
                        }
                        BlockUpdateResult::Possible { number, is_cleared } => {
                            sudoku_board
                                .current
                                .mark_conflicts(&block_index, Some((number, is_cleared)));

                            let block = sudoku_board.current.get_block(&block_index);
                            let poss = block.status.as_possibilities().unwrap(); // This must be possibilities

                            if poss.is_conflicting(number) {
                                // This is also a mistake
                                stats.possibility_mistakes += 1;
                                println!("This is also a mistake!")
                            }
                        }
                    }
                }
                None => (),
            }
        }
    }
}

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
            return Resolved;
        }
        SelectionMode::Possibilities => {
            if let Some(pos) = block.status.as_possibilities_mut() {
                if pos.numbers.has_number(number) {
                    pos.numbers.del_number(number);

                    if pos.numbers.count_numbers() == 0 {
                        block.status = SudokuBlockStatus::Unresolved;
                        return Cleared;
                    } else {
                        return Possible {
                            number,
                            is_cleared: true,
                        };
                    }
                } else {
                    pos.numbers.set_number(number);
                    return Possible {
                        number,
                        is_cleared: false,
                    };
                }
            } else {
                block.status = SudokuBlockStatus::Possibilities(SudokuPossibilities::new(
                    SudokuNumbers::new([number]),
                ));
                return Possible {
                    number,
                    is_cleared: false,
                };
            }
        }
    }
}

fn update_mistakes_text(
    stats: Res<Stats>,
    mut mistakes_text: Single<&mut TextSpan, With<MistakesCountText>>,
) {
    mistakes_text.0 = format!(
        "{} (Numbers) / {} (Possibilities)",
        stats.mistakes, stats.possibility_mistakes
    );
}

#[derive(Debug, Component)]
struct SquareIndex {
    i: usize,
    j: usize,
    master: Option<(usize, usize)>,
}

impl SquareIndex {
    fn actual_index(&self) -> (usize, usize) {
        if let Some(master) = self.master {
            (master.0 * 3 + self.i, master.1 * 3 + self.j)
        } else {
            (self.i, self.j)
        }
    }
}

#[derive(Debug, Component)]
struct Foundation;

#[derive(Debug, Component)]
struct Block;

#[derive(Debug, Component)]
struct Possibilities;

fn spawn_sudoku_board(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    defaults: &DefaultMaterials,
    center: Vec2,
    width: f32,
    offset: f32,
) {
    for spawn_info in square_group_info(width, offset, center) {
        commands
            .spawn((
                SquareBundle::new(
                    defaults.default_foundation_block_color.clone(),
                    meshes,
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
                        meshes,
                        spawn_info.clone(),
                        Some(master_index),
                    );
                    builder.spawn((bundle, Block));
                }
            });
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
                ..Default::default()
            }),
            index: SquareIndex {
                i: spawn_info.index.0,
                j: spawn_info.index.1,
                master: master_index,
            },
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
