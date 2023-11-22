use crate::engine::animation::{AnimationIndices, AnimationTimer};

use bevy::{
    math::*,
    prelude::*,
    transform::{commands, TransformSystem},
};
use bevy_xpbd_2d::math::*;
use bevy_xpbd_2d::plugins::spatial_query::ShapeCaster;
use bevy_xpbd_2d::prelude::*;

use super::{physics_layers::Layer, player_controller::CharacterControllerPlugin};
use super::{player_controller::CharacterControllerBundle, stats::*};
use crate::{GameFont, Ground};

#[derive(Event)]
struct LevelUpEvent(Entity);

#[derive(Component)]
struct LevelUpText;

#[derive(Component, Deref, DerefMut)]
pub struct LevelUpTextTimer(pub Timer);

const LVL_TEXT_HEIGHT_OFFSET: f32 = 10.0;

fn reset_player_xp_level(
    mut ev_levelup: EventReader<LevelUpEvent>,
    mut query: Query<(&mut PlayerXp, &mut PlayerLevel)>,
) {
    for ev in ev_levelup.read() {
        if let Ok((mut xp, mut level)) = query.get_mut(ev.0) {
            xp.0 = 0;
            level.0 += 1;
        }
    }
}

fn destroy_levelup_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut LevelUpTextTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn levelup_text_follow(
    mut text_query: Query<&mut Transform, With<LevelUpText>>,
    player_query: Query<&Position, With<Player>>,
) {
    for mut text_pos in &mut text_query {
        for player_pos in &player_query {
            text_pos.translation.x = player_pos.x;
            text_pos.translation.y = player_pos.y + LVL_TEXT_HEIGHT_OFFSET;
        }
    }
}

fn spawn_levelup_text(
    mut commands: Commands,
    game_font: Res<GameFont>,
    mut ev_levelup: EventReader<LevelUpEvent>,
    query: Query<&Position, With<Player>>,
) {
    let text_style = TextStyle {
        font: game_font.0.clone(),
        font_size: 8.0,
        color: Color::YELLOW,
    };

    for _ev in ev_levelup.read() {
        let player_pos = query.get_single().unwrap();
        commands.spawn((
            LevelUpText,
            Text2dBundle {
                text: Text::from_section("Level Up!", text_style.clone())
                    .with_alignment(TextAlignment::Center),
                transform: Transform::from_xyz(
                    player_pos.x,
                    player_pos.y + LVL_TEXT_HEIGHT_OFFSET,
                    0.,
                )
                .with_scale(Vec3::new(0.5, 0.5, 1.0)),
                ..default()
            },
            LevelUpTextTimer(Timer::from_seconds(1., TimerMode::Once)),
        ));
    }
}

fn player_level_up(mut ev_levelup: EventWriter<LevelUpEvent>, query: Query<(Entity, &PlayerXp)>) {
    for (entity, xp) in query.iter() {
        if xp.0 > 1 {
            ev_levelup.send(LevelUpEvent(entity));
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct PlayerXp(pub i32);

#[derive(Component, Reflect, Default)]
pub struct PlayerLevel(pub i32);

#[derive(Bundle)]
struct PlayerStatBundle {
    // xp: PlayerXp,
    speed: SpeedStat,
    // health: HealthStat,
    // jump_height: JumpHeightStat,
    jumps: JumpsStat,
}

impl PlayerStatBundle {
    pub fn new() -> Self {
        Self {
            speed: SpeedStat(40.0),
            jumps: JumpsStat::new(1, 120.0),
        }
    }
}

#[derive(Component)]
pub struct Player;

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/commando_run.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(6.0, 11.0),
        8,
        1,
        Some(Vec2::new(1.0, 0.0)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 1, last: 7 };

    commands.spawn((
        Name::new("Player"),
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerXp::default(),
        PlayerLevel::default(),
        PlayerStatBundle::new(),
        // PlayerCollisionBundle::new(),
        CharacterControllerBundle::new(Collider::cuboid(6.0, 11.0), Vector::NEG_Y * 1000.0)
            .with_movement(220.0, 0.85, 220.0, 1, (30.0 as Scalar).to_radians()),
        Player,
    ));
}

pub fn animate_player(
    time: Res<Time>,
    mut query: Query<(
        &LinearVelocity,
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (vel, indices, mut timer, mut sprite) in &mut query {
        if vel.x.abs().floor() != 0.0 {
            let dv = time.delta();
            timer.tick(dv);
            if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
            }
            if vel.x <= 0.0 {
                sprite.flip_x = true;
            } else {
                sprite.flip_x = false;
            }
        } else {
            sprite.index = indices.first;
        }
    }
}

fn add_level(keyboard_input: Res<Input<KeyCode>>, mut player: Query<&mut PlayerXp, With<Player>>) {
    for mut player_xp in &mut player {
        if keyboard_input.any_just_pressed([KeyCode::L]) {
            player_xp.0 += 2;
        }
    }
}

pub fn camera_follow(
    mut camera_pos: Query<&mut Transform, (Without<Player>, With<Camera>)>,
    player_pos: Query<&Transform, With<Player>>,
) {
    for player_transform in &player_pos {
        let mut camera_transform = camera_pos.get_single_mut().unwrap();
        camera_transform.translation = camera_transform
            .translation
            .truncate()
            .lerp(player_transform.translation.truncate(), 0.1)
            .extend(0.0);
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerLevel>()
            .register_type::<PlayerXp>()
            .add_event::<LevelUpEvent>()
            .add_plugins(CharacterControllerPlugin)
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    animate_player,
                    add_level,
                    (
                        player_level_up,
                        reset_player_xp_level.run_if(on_event::<LevelUpEvent>()),
                        spawn_levelup_text.run_if(on_event::<LevelUpEvent>()),
                        // detect_grounded,
                    )
                        .chain(),
                    destroy_levelup_text,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    camera_follow
                        .after(PhysicsSet::Sync)
                        .before(TransformSystem::TransformPropagate),
                    levelup_text_follow
                        .after(PhysicsSet::Sync)
                        .before(TransformSystem::TransformPropagate),
                ),
            );
    }
}
