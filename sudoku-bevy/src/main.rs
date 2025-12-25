use bevy::{
    color::palettes::{
        basic::PURPLE,
        css::{BLACK, BLUE, WHITE, YELLOW},
        tailwind::{BLUE_200, BLUE_400, YELLOW_400},
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
    text::TextBounds,
};
use sudoku_solver::{
    SudokuBlockStatus, SudokuBoard,
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
    prev: Option<(usize, usize)>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SudokuBoardResources>()
        .init_resource::<SelectedBlock>()
        .add_systems(Startup, setup)
        .add_systems(
            PostStartup,
            (check_foundation_squares, check_block_squares).chain(),
        )
        .add_systems(
            Update,
            (
                change_selected_block.run_if(
                    input_just_pressed(KeyCode::ArrowDown)
                        .or(input_just_pressed(KeyCode::ArrowUp))
                        .or(input_just_pressed(KeyCode::ArrowLeft))
                        .or(input_just_pressed(KeyCode::ArrowRight)),
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
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    let center = vec2(0., -50.);
    let width = 630.;
    let offset = 5.;

    let board = &mut sudoku_board.current;

    board.fill_board_u8(sudoku_samples::easy::FIRST).unwrap();

    spawn_sudoku_board(
        &mut commands,
        &mut meshes,
        &mut materials,
        center,
        width,
        offset,
        Color::from(PURPLE),
    );

    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(620., 100.))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::default().with_translation(Vec3 {
                y: -320.,
                ..Default::default()
            }),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Mesh2d(meshes.add(Rectangle::new(610., 90.))),
                    MeshMaterial2d(materials.add(Color::from(YELLOW))),
                ))
                .with_children(|builder| {
                    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
                    let text_font = TextFont {
                        font: font.clone(),
                        font_size: 17.,
                        ..default()
                    };

                    builder.spawn((
                        Text2d::new(format!("Use 'Space' to update possible values, 'Enter' to resolve blocks,\n'R' to reset, 'M' to change selection mode, 'C' to clear block,\n1 to 9 to set number and 'H' to engage Hidden single strategy.")),
                        text_font,
                        TextColor(Color::from(BLACK)),
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        // Wrap text in the rectangle
                        // TextBounds::from(Vec2::new(300., 500.)),
                    ));
                });
        });
}

fn check_foundation_squares(query: Query<(Entity, &SquareSpawnInfo), With<FoundationSquare>>) {
    println!("Foundation squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        println!("{}- {:?}", i + 1, index.index)
    }
}

fn check_block_squares(query: Query<(Entity, &SquareIndex), With<BlockSquare>>) {
    println!("Block squares:");
    for (i, (_, index)) in query.iter().enumerate() {
        println!("{}- {:?} (index: {:?})", i + 1, index, index.actual_index())
    }
}

fn update_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board: ResMut<SudokuBoardResources>,
    blocks: Query<(Entity, &SquareSpawnInfo, &SquareIndex), With<BlockSquare>>,
) {
    let mut snapshot_should_update = false;

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_justification = Justify::Center;

    let mut text_font = TextFont {
        font: font.clone(),
        ..default()
    };

    use SudokuNumber::*;
    for row in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
        for col in [One, Two, Three, Four, Five, Six, Seven, Eight, Nine] {
            if board.current.get_block(row, col).status != board.snapshot.get_block(row, col).status
            {
                snapshot_should_update = true;
                let i = col.to_index();
                let j = row.to_index();

                if let Some((entity, spawn_info, _)) = blocks.iter().find(|(_, _, index)| {
                    let index = index.actual_index();
                    index.0 == i && index.1 == j
                }) {
                    commands.entity(entity).despawn_children();
                    let block = board.current.get_block(row, col);

                    match &block.status {
                        SudokuBlockStatus::Unresolved => (),
                        SudokuBlockStatus::Fixed(sudoku_number)
                        | SudokuBlockStatus::Resolved(sudoku_number) => {
                            text_font.font_size = spawn_info.width;

                            let child = commands
                                .spawn((
                                    Text2d::new(format!("{}", sudoku_number.to_u8())),
                                    text_font.clone(),
                                    TextColor(Color::from(BLACK)),
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
                                    .get_numbers()
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
                                                    Color::from(BLUE),
                                                    &mut meshes,
                                                    &mut materials,
                                                    spawn_info,
                                                    Some(master_index),
                                                ),
                                                PossibilitiesSquare,
                                            ))
                                            .with_children(|builder| {
                                                builder.spawn((
                                                    Text2d::new(format!("{}", number)),
                                                    text_font.clone(),
                                                    TextColor(Color::from(WHITE)),
                                                    TextLayout::new_with_justify(
                                                        text_justification,
                                                    ),
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
        }
    }

    if snapshot_should_update {
        board.snapshot = board.current.clone();
    }
}

fn change_selected_block(
    mut selected: ResMut<SelectedBlock>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if selected.current.0 > 0 {
            selected.prev = selected.current.into();
            selected.current.0 -= 1;
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        if selected.current.0 < 8 {
            selected.prev = selected.current.into();
            selected.current.0 += 1;
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        if selected.current.1 < 8 {
            selected.prev = selected.current.into();
            selected.current.1 += 1;
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if selected.current.1 > 0 {
            selected.prev = selected.current.into();
            selected.current.1 -= 1;
        }
    }
}

fn update_selected_block(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut selected: ResMut<SelectedBlock>,
    mut blocks: Query<(&SquareIndex, &mut MeshMaterial2d<ColorMaterial>), With<BlockSquare>>,
) {
    if let Some((_, mut material)) = blocks.iter_mut().find(|(index, _)| {
        let index = index.actual_index();
        index.0 == selected.current.0 && index.1 == selected.current.1
    }) {
        material.0 = materials.add(Color::from(match selected.mode {
            SelectionMode::Resolving => YELLOW_400,
            SelectionMode::Possibilities => BLUE_200,
        }));
    }

    if let Some(prev) = selected.prev {
        if let Some((_, mut material)) = blocks.iter_mut().find(|(index, _)| {
            let index = index.actual_index();
            index.0 == prev.0 && index.1 == prev.1
        }) {
            material.0 = materials.add(Color::from(YELLOW));
        }
    }

    selected.prev = None;
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
    let block = sudoku_board.current.get_block_mut(
        (selected.current.1 + 1).try_into().unwrap(),
        (selected.current.0 + 1).try_into().unwrap(),
    );

    match &block.status {
        SudokuBlockStatus::Fixed(_) => (),
        _ => {
            if keyboard_input.just_pressed(KeyCode::KeyC) {
                block.status = SudokuBlockStatus::Unresolved;
            }
        }
    }
}

fn manually_update_block(
    mut sudoku_board: ResMut<SudokuBoardResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedBlock>,
) {
    let block = sudoku_board.current.get_block_mut(
        (selected.current.1 + 1).try_into().unwrap(),
        (selected.current.0 + 1).try_into().unwrap(),
    );

    match &block.status {
        SudokuBlockStatus::Fixed(_) => (),
        _ => {
            if keyboard_input.just_pressed(KeyCode::Digit1) {
                _update_block(&selected, block, SudokuNumber::One);
            }
            if keyboard_input.just_pressed(KeyCode::Digit2) {
                _update_block(&selected, block, SudokuNumber::Two);
            }
            if keyboard_input.just_pressed(KeyCode::Digit3) {
                _update_block(&selected, block, SudokuNumber::Three);
            }
            if keyboard_input.just_pressed(KeyCode::Digit4) {
                _update_block(&selected, block, SudokuNumber::Four);
            }
            if keyboard_input.just_pressed(KeyCode::Digit5) {
                _update_block(&selected, block, SudokuNumber::Five);
            }
            if keyboard_input.just_pressed(KeyCode::Digit6) {
                _update_block(&selected, block, SudokuNumber::Six);
            }
            if keyboard_input.just_pressed(KeyCode::Digit7) {
                _update_block(&selected, block, SudokuNumber::Seven);
            }
            if keyboard_input.just_pressed(KeyCode::Digit8) {
                _update_block(&selected, block, SudokuNumber::Eight);
            }
            if keyboard_input.just_pressed(KeyCode::Digit9) {
                _update_block(&selected, block, SudokuNumber::Nine);
            }
        }
    }
}

fn _update_block(
    selected: &SelectedBlock,
    block: &mut sudoku_solver::SudokuBlock,
    number: SudokuNumber,
) {
    match selected.mode {
        SelectionMode::Resolving => {
            if let SudokuBlockStatus::Resolved(already) = block.status {
                if already == number {
                    block.status = SudokuBlockStatus::Unresolved;
                    return;
                }
            }

            block.status = SudokuBlockStatus::Resolved(number);
        }
        SelectionMode::Possibilities => {
            if let Some(pos) = block.status.as_possibilities_mut() {
                if pos.has_number(number) {
                    pos.del_number(number);
                } else {
                    pos.set_number(number);
                }
            } else {
                block.status = SudokuBlockStatus::Possibilities(SudokuNumbers::new([number]));
            }
        }
    }
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
struct FoundationSquare;

#[derive(Debug, Component)]
struct BlockSquare;

#[derive(Debug, Component)]
struct PossibilitiesSquare;

fn spawn_sudoku_board(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    center: Vec2,
    width: f32,
    offset: f32,
    color: Color,
) {
    for spawn_info in square_group_info(width, offset, center) {
        commands
            .spawn((
                SquareBundle::new(color, meshes, materials, spawn_info.clone(), None),
                FoundationSquare,
            ))
            .with_children(|builder| {
                let width = spawn_info.width;
                let master_index = spawn_info.index;

                for spawn_info in square_group_info(width, 5., Default::default()) {
                    let bundle = SquareBundle::new(
                        Color::from(YELLOW),
                        meshes,
                        materials,
                        spawn_info.clone(),
                        Some(master_index),
                    );
                    builder.spawn((bundle, BlockSquare));
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
        color: Color,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        spawn_info: SquareSpawnInfo,
        master_index: Option<(usize, usize)>,
    ) -> Self {
        Self {
            mesh: Mesh2d(meshes.add(Rectangle::new(spawn_info.width, spawn_info.width))),
            material: MeshMaterial2d(materials.add(color)),
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
    (0..3)
        .into_iter()
        .map(move |i| {
            (0..3).into_iter().map(move |j| {
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
        .flatten()
}
