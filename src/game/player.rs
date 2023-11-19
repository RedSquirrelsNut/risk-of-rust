use crate::engine::animation::{AnimationIndices, AnimationTimer};

use bevy::{
    prelude::*,
    transform::{commands, TransformSystem},
};
use bevy_xpbd_2d::math::Quaternion;
use bevy_xpbd_2d::plugins::spatial_query::ShapeCaster;
use bevy_xpbd_2d::prelude::*;

use super::physics_layers::Layer;
use super::stats::*;
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

#[derive(Bundle)]
struct PlayerCollisionBundle {
    body: RigidBody,
    collider: Collider,
    axis_lock: LockedAxes,
    restitution: Restitution,
    friciton: Friction,
    col_layers: CollisionLayers,
    sleep: SleepingDisabled,
    ground_caster: ShapeCaster,
}

impl PlayerCollisionBundle {
    pub fn new() -> Self {
        Self {
            body: RigidBody::Dynamic,
            collider: Collider::cuboid(6.0, 11.0), //ball(7.5 as Scalar),
            axis_lock: LockedAxes::new().lock_rotation(),
            restitution: Restitution::new(0.0),
            friciton: Friction::new(0.0),
            col_layers: CollisionLayers::new([Layer::Player], [Layer::Ground]),
            sleep: SleepingDisabled,
            ground_caster: ShapeCaster::new(
                Collider::cuboid(5.9, 10.9),
                Vec2::NEG_Y * 0.05,
                0.0,
                Vec2::NEG_Y,
            )
            .with_max_time_of_impact(0.2)
            .with_max_hits(1)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layer::Ground])),
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

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
        PlayerCollisionBundle::new(),
        Player,
    ));
}

pub fn detect_grounded(
    mut commands: Commands,
    mut hit_query: Query<(Entity, &ShapeCaster, &ShapeHits, &mut JumpsStat), With<Player>>,
) {
    let mut is_grounded = false;
    for (player, _shape_caster, hits, mut jumps) in &mut hit_query {
        for _hit in hits.iter() {
            is_grounded = true;
        }

        if is_grounded {
            commands.entity(player).insert(Grounded);
            jumps.jumps_left = jumps.max_jumps;
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
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
        if vel.x != 0.0 {
            let dv = time.delta().mul_f32(vel.x.abs() / 20.0);
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

//FIXME: MAX JUMPS DOESNT DECREASE AFTER FIRST JUMP
pub fn move_player(
    // time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<
        (
            &mut LinearVelocity,
            &SpeedStat,
            &mut JumpsStat,
            Has<Grounded>,
        ),
        With<Player>,
    >,
) {
    // let delta_time = time.delta_seconds_f64().adjust_precision();
    for (mut linear_velocity, player_speed, mut player_jump, is_grounded) in &mut player {
        let player_speed = player_speed.0; // * time.delta_seconds(); //player_stats.speed.value();
        let jump_height = player_jump.jump_height; // * time.delta_seconds();

        linear_velocity.x = 0.0;

        if keyboard_input.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space])
            && (is_grounded || (player_jump.jumps_left > 1))
        {
            // Use a higher acceleration for upwards movement to overcome gravity
            linear_velocity.y = jump_height; //player_stats.jump_height.value();
            player_jump.jumps_left -= 1;
        }
        if keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]) {
            linear_velocity.x = -player_speed;
        }
        if keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]) {
            linear_velocity.x = player_speed;
        }
    }
}

pub fn camera_follow(
    mut camera_pos: Query<(&Camera2d, &mut Transform), Without<Player>>,
    player_pos: Query<&Position, With<Player>>,
) {
    for player_position in &player_pos {
        for (_, mut transform) in &mut camera_pos {
            transform.translation.y = player_position.y;
            transform.translation.x = player_position.x;
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerLevel>()
            .register_type::<PlayerXp>()
            .add_event::<LevelUpEvent>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    animate_player,
                    move_player,
                    add_level,
                    // camera_follow,
                    (
                        player_level_up,
                        reset_player_xp_level.run_if(on_event::<LevelUpEvent>()),
                        spawn_levelup_text.run_if(on_event::<LevelUpEvent>()),
                        detect_grounded,
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
