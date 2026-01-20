use bevy::{
    asset::LoadState,
    color::palettes::{
        basic::PURPLE,
        css::{BLACK, BLUE, RED, WHITE, YELLOW},
        tailwind::{BLUE_200, GRAY_600, GREEN_400, ORANGE_400, ORANGE_500, RED_400, YELLOW_400},
    },
    platform::collections::HashMap,
    prelude::*,
    window::{EnabledButtons, WindowTheme},
};
use sudoku_solver::strategies::Strategy;

use crate::{
    pancam::{DirectionKeys, PanCam, PanCamPlugin},
    plugins::shared::TextBundle,
};

#[derive(Debug, States, Default, PartialEq, Eq, Hash, Clone)]
pub enum AppState {
    #[default]
    Loading,
    Ready,
}

#[derive(Debug, Resource, Default)]
pub struct DefaultMaterials {
    // Handles
    pub default_deactivate_board_color: Handle<ColorMaterial>,
    pub default_active_board_color: Handle<ColorMaterial>,
    pub default_foundation_block_color: Handle<ColorMaterial>,
    pub default_possibilities_block_color: Handle<ColorMaterial>,
    pub default_block_color: Handle<ColorMaterial>,
    pub default_solved_block_color: Handle<ColorMaterial>,
    pub selected_resolving_block_color: Handle<ColorMaterial>,
    pub selected_possibilities_block_color: Handle<ColorMaterial>,

    pub conflicting_source_color: Handle<ColorMaterial>,
    pub conflicting_affected_color: Handle<ColorMaterial>,

    pub strategy_source_color: Handle<ColorMaterial>,
    pub strategy_effected_color: Handle<ColorMaterial>,
    pub strategy_source_text_color: Color,

    // Colors
    pub default_base_text_color: Color,
    pub default_fixed_number_color: Color,
    pub default_resolved_number_color: Color,
    pub default_possibility_number_color: Color,
}

#[derive(Debug, Resource, Default)]
pub struct DefaultAssets {
    // Handles
    pub default_font: Handle<Font>,
}

#[derive(Debug, Component)]
struct LoadingEntity;

#[derive(Debug)]
pub struct BlockColorInfo {
    pub text: Color,
    pub background: Handle<ColorMaterial>,
}

impl BlockColorInfo {
    fn new(text: impl Into<Color>, background: Handle<ColorMaterial>) -> Self {
        Self {
            text: text.into(),
            background,
        }
    }
}

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct StrategyMarkerColors(pub HashMap<Strategy, BlockColorInfo>);

/// This actually takes care of adding default plugins.
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Let's play Sudoku".into(),
                    name: Some("sudoku.bevy.app".into()),
                    resolution: (860, 720).into(),
                    window_theme: Some(WindowTheme::Dark),
                    resizable: false,
                    enabled_buttons: EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            // .set(LogPlugin {
            //     filter: "warn,sudoku-bevy=trace".to_string(), //specific filters
            //     level: Level::TRACE, //Change this to be globally change levels
            //     ..Default::default()
            // }),
            MeshPickingPlugin,
            PanCamPlugin,
        ))
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ..Default::default()
        })
        .init_resource::<DefaultAssets>()
        .init_resource::<DefaultMaterials>()
        .init_resource::<StrategyMarkerColors>()
        .init_state::<AppState>()
        // Loading state systems
        .add_systems(OnEnter(AppState::Loading), setup_asset_loading)
        .add_systems(
            Update,
            check_assets_ready.run_if(in_state(AppState::Loading)),
        );
    }
}

fn setup_asset_loading(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut defaults: ResMut<DefaultMaterials>,
    mut defaults_assets: ResMut<DefaultAssets>,
    mut strategy_colors: ResMut<StrategyMarkerColors>,
    asset_server: Res<AssetServer>,
) {
    defaults.default_active_board_color = materials.add(Color::from(ORANGE_500));
    defaults.default_deactivate_board_color = materials.add(Color::from(PURPLE));
    defaults.default_block_color = materials.add(Color::from(YELLOW));
    defaults.default_solved_block_color = materials.add(Color::from(GREEN_400));
    defaults.selected_resolving_block_color = materials.add(Color::from(YELLOW_400));
    defaults.selected_possibilities_block_color = materials.add(Color::from(BLUE_200));
    defaults.default_foundation_block_color = materials.add(Color::from(PURPLE));
    defaults.default_possibilities_block_color = materials.add(Color::from(BLUE));

    defaults.conflicting_source_color = materials.add(Color::from(RED));
    defaults.conflicting_affected_color = materials.add(Color::from(RED_400));

    defaults.strategy_source_color = materials.add(Color::from(GREEN_400));
    defaults.strategy_effected_color = materials.add(Color::from(BLACK));
    defaults.strategy_source_text_color = Color::from(BLACK);

    defaults.default_base_text_color = Color::from(BLACK);
    defaults.default_fixed_number_color = Color::from(GRAY_600);
    defaults.default_possibility_number_color = Color::from(WHITE);
    defaults.default_resolved_number_color = Color::from(BLACK);

    strategy_colors.insert(
        Strategy::HiddenSingle,
        BlockColorInfo::new(BLACK, materials.add(Color::from(GREEN_400))),
    );

    strategy_colors.insert(
        Strategy::NakedPair,
        BlockColorInfo::new(BLACK, materials.add(Color::from(ORANGE_400))),
    );

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    defaults_assets.default_font = font;

    let mut ortho = OrthographicProjection::default_2d();
    ortho.scale = 1.5;

    commands.spawn((
        Camera2d,
        MeshPickingCamera,
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

    commands.spawn((
        TextBundle::new(
            "Loading things ...",
            Handle::<Font>::default(),
            40.,
            WHITE,
            Default::default(),
        ),
        LoadingEntity,
    ));
}

fn check_assets_ready(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    mut defaults_assets: ResMut<DefaultAssets>,
    loading_entity: Single<Option<Entity>, With<LoadingEntity>>,
) {
    let resume = match asset_server.load_state(&defaults_assets.default_font) {
        LoadState::Loaded => true,
        LoadState::Failed(_) => {
            defaults_assets.default_font = Handle::<Font>::default();
            eprintln!("Failed to load font! Using default font.");
            false
        }
        _ => {
            // Wait ...
            false
        }
    };

    if resume {
        next_state.set(AppState::Ready);
        if let Some(entity) = loading_entity.as_ref() {
            commands.entity(*entity).despawn();
        }
    }
}
