#![feature(trivial_bounds)]
//Current frame limiting solutions: https://github.com/aevyrie/bevy_framepace
//or: https://www.reddit.com/r/bevy/comments/kn5172/controlling_framerate/
//or: https://github.com/bevyengine/bevy/issues/1343
mod assets;
mod engine;
mod game;

use crate::assets::*;
use crate::engine::fps_text::*;
use crate::game::clock::*;
use crate::game::enemy::dummy::spawn_temp_dummy;
use crate::game::physics_layers::Layer;
use crate::game::player::PlayerPlugin;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, render::camera::ScalingMode,
    text::TextSettings,
};

use bevy_xpbd_2d::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::stats::StatsPlugin;

pub const CLEAR_COLOR: Color = Color::rgb(0.270588, 0.266666, 0.309803);
pub const TEXT_SCALE: f32 = 4.0;

//GAME RESOLUTION
pub const GAME_WIDTH: f32 = 320.0; //480.; //320.; //240.0;
pub const GAME_HEIGHT: f32 = 240.0; //360.; //240.; //160.0;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Risk of Rust".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            FrameTimeDiagnosticsPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::new(FixedUpdate),
        ))
        .insert_resource(TextSettings {
            allow_dynamic_font_size: false,
            ..default()
        })
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(StatsPlugin)
        .add_state::<AppState>()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(CLEAR_COLOR))
        .insert_resource(SubstepCount(12))
        .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
        .add_systems(PreStartup, setup)
        .add_systems(
            Startup,
            (
                spawn_fps_text,
                spawn_temp_floor,
                spawn_temp_dummy,
                spawn_rope,
                startup_disable_debug_view,
                spawn_clock_text,
            ),
        )
        .add_systems(
            Update,
            (
                text_update_system,
                clock_text_update_system,
                toggle_debug_view,
            ),
        )
        .add_systems(PostUpdate, change_grav)
        .add_plugins(PlayerPlugin)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::Fixed {
        width: GAME_WIDTH,
        height: GAME_HEIGHT,
    };

    commands.spawn(camera_bundle);
    commands.insert_resource(GameFont(asset_server.load("fonts/a4ep.ttf")));
}

#[derive(Component)]
pub struct Climbable;

fn spawn_rope(mut commands: Commands) {
    // Rectangle
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(2.0, 40.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-20., -205., -2.)),
            ..Default::default()
        },
        Climbable,
        Name::new("Rope"),
        Sensor,
        Collider::cuboid(2.0, 40.0),
        CollisionLayers::new([Layer::Enemy], []),
    ));
}

#[derive(Component)]
pub struct Ground;

fn spawn_temp_floor(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Name::new("Temp_Floor"),
        Ground,
        SpriteBundle {
            texture: assets.load("sprites/temp_floor.png"),
            transform: Transform::from_xyz(0.0, -232.0, 0.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(252.0, 14.0),
        Friction::new(0.0),
        CollisionLayers::new([Layer::Ground], [Layer::Player, Layer::Enemy]),
    ));

    commands.spawn((
        Name::new("Temp_Floor"),
        Ground,
        SpriteBundle {
            texture: assets.load("sprites/temp_floor.png"),
            transform: Transform::from_xyz(252.0, -232.0, 0.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(252.0, 14.0),
        Friction::new(0.0),
        CollisionLayers::new([Layer::Ground], [Layer::Player, Layer::Enemy]),
    ));
}

fn change_grav(mut gravity: ResMut<Gravity>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.any_just_pressed([KeyCode::R]) {
        gravity.0 = Vec2::new(0., 100.);
    }
}

fn startup_disable_debug_view(mut debug_config: ResMut<PhysicsDebugConfig>) {
    debug_config.enabled = false;
}

fn toggle_debug_view(
    mut debug_config: ResMut<PhysicsDebugConfig>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::F3]) {
        debug_config.enabled = !debug_config.enabled;
    }
}
