use bevy::{ecs::query::Has, prelude::*};
use bevy_xpbd_2d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use super::enemy::dummy::Layer;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<JumpCount>()
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    gamepad_input,
                    update_grounded,
                    check_can_climb,
                    update_climbing,
                    apply_deferred,
                    apply_gravity,
                    movement,
                    apply_movement_damping,
                )
                    .chain(),
            )
            .add_systems(
                // Run collision handling in substep schedule
                SubstepSchedule,
                kinematic_controller_collisions.in_set(SubstepSet::SolveUserConstraints),
            );
    }
}

/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
    Move(Scalar),
    Jump,
    Climb(Scalar),
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// A marker component indicating that an entity can climb (Climbing counts as grounded as well)
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CanClimb;

/// A marker component indicating that an entity is climbing (Climbing counts as grounded as well)
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Climbing;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

#[derive(Component, Reflect)]
pub struct JumpCount {
    pub current: u32,
    pub max: u32,
}

impl JumpCount {
    pub fn new(max: u32) -> Self {
        Self { current: 0, max }
    }
}

/// The gravitational acceleration used for a character controller.
#[derive(Component)]
pub struct ControllerGravity(Vector);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    col_layers: CollisionLayers,
    ground_caster: ShapeCaster,
    gravity: ControllerGravity,
    movement: MovementBundle,
    // sleeping: SleepingDisabled,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    jump_count: JumpCount,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        jump_count: u32,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            jump_count: JumpCount::new(jump_count),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, 1, PI * 0.45)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider, gravity: Vector) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 1);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Kinematic,
            collider,
            col_layers: CollisionLayers::new([Layer::Player], [Layer::Ground]),
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Vector::NEG_Y)
                .with_max_time_of_impact(0.2)
                .with_max_hits(1)
                .with_query_filter(SpatialQueryFilter::new().with_masks([Layer::Ground])),
            gravity: ControllerGravity(gravity),
            movement: MovementBundle::default(),
            // sleeping: SleepingDisabled,
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        jump_count: u32,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(
            acceleration,
            damping,
            jump_impulse,
            jump_count,
            max_slope_angle,
        );
        self
    }
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]);
    let right = keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]);

    let horizontal = right as i8 - left as i8;
    let h_direction = horizontal as Scalar;

    if h_direction != 0.0 {
        movement_event_writer.send(MovementAction::Move(h_direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }

    let up = keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]);
    let down = keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]);

    let vertical = up as i8 - down as i8;
    let v_direction = vertical as Scalar;

    if v_direction != 0.0 {
        movement_event_writer.send(MovementAction::Climb(v_direction));
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
) {
    for gamepad in gamepads.iter() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };

        if let Some(x) = axes.get(axis_lx) {
            movement_event_writer.send(MovementAction::Move(x as Scalar));
        }

        let jump_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };

        if buttons.just_pressed(jump_button) {
            movement_event_writer.send(MovementAction::Jump);
        }
    }
}

fn update_climbing(
    mut commands: Commands,
    mut movement_event_reader: EventReader<MovementAction>,
    query: Query<(Entity, Has<CanClimb>, Has<Climbing>), With<CharacterController>>,
) {
    for event in movement_event_reader.read() {
        for (entity, can_climb, is_climbing) in &query {
            let mut should_climb = is_climbing;
            match event {
                MovementAction::Climb(_) => {
                    should_climb = true;
                }
                MovementAction::Jump => {
                    should_climb = false;
                }
                _ => {}
            }
            if can_climb && should_climb {
                commands.entity(entity).insert(Climbing);
            } else {
                commands.entity(entity).remove::<Climbing>();
            }
        }
    }
}

fn check_can_climb(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            Entity,
            &Collider,
            &Position,
            &mut ControllerGravity,
            Has<Climbing>,
        ),
        With<CharacterController>,
    >,
) {
    let mut can_climb = false;
    for (entity, collider, position, mut gravity, is_climbing) in &mut query {
        let intersections = spatial_query.shape_intersections(
            &collider,                                                // Shape
            Vec2::new(position.x, position.y),                        // Shape position
            0.0,                                                      // Shape rotation
            SpatialQueryFilter::new().with_masks([Layer::Climbable]), // Query filter
        );

        if let Some(_) = intersections.first() {
            can_climb = true;
        }

        if can_climb {
            commands.entity(entity).insert(CanClimb);
        } else {
            commands.entity(entity).remove::<CanClimb>();
            commands.entity(entity).remove::<Climbing>();
        }

        if is_climbing {
            gravity.0 = Vec2::new(0., 0.);
        } else {
            gravity.0 = Vector::NEG_Y * 1000.0;
        }
    }
}

//TODO: Jump count should have its own system?
/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut JumpCount,
            &ShapeHits,
            &Rotation,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
) {
    for (entity, mut jump_count, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            jump_count.current = 0;
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut JumpCount,
        &mut LinearVelocity,
        &mut Position,
        Has<Grounded>,
        Has<Climbing>,
    )>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (
            movement_acceleration,
            jump_impulse,
            mut jump_count,
            mut linear_velocity,
            mut position,
            is_grounded,
            is_climbing,
        ) in &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    if !is_climbing {
                        linear_velocity.x += *direction * movement_acceleration.0 * delta_time;
                    }
                }
                MovementAction::Jump => {
                    if is_grounded || is_climbing || jump_count.current < jump_count.max {
                        linear_velocity.y = jump_impulse.0;
                        jump_count.current += 1;
                    }
                }
                MovementAction::Climb(direction) => {
                    if is_climbing {
                        linear_velocity.x = 0.;
                        linear_velocity.y += *direction * movement_acceleration.0 * delta_time;
                    }
                }
            }
        }
    }
}

/// Applies [`ControllerGravity`] to character controllers.
fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for (gravity, mut linear_velocity) in &mut controllers {
        linear_velocity.0 += gravity.0 * delta_time;
    }
}

/// Slows down movement in the X direction, Y if climbing.
fn apply_movement_damping(
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity, Has<Climbing>)>,
) {
    for (damping_factor, mut linear_velocity, is_climbing) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis when
        // not climbing.
        if is_climbing {
            linear_velocity.y *= damping_factor.0;
        } else {
            linear_velocity.x *= damping_factor.0;
        }
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system performs very basic collision response for kinematic
/// character controllers by pushing them along their contact normals
/// by the current penetration depths.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, Without<Sensor>>,
    mut character_controllers: Query<
        (
            &RigidBody,
            &mut Position,
            &Rotation,
            &mut LinearVelocity,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }

        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([collider_parent1, collider_parent2]) =
            collider_parents.get_many([contacts.entity1, contacts.entity2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;
        let (rb, mut position, rotation, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(collider_parent1.get()) {
                is_first = true;
                character
            } else if let Ok(character) = character_controllers.get_mut(collider_parent2.get()) {
                is_first = false;
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers
        if !rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.global_normal1(rotation)
            } else {
                -manifold.global_normal2(rotation)
            };

            // Solve each penetrating contact in the manifold
            for contact in manifold.contacts.iter().filter(|c| c.penetration > 0.0) {
                position.0 += normal * contact.penetration;
            }

            // If the slope isn't too steep to walk on but the character
            // is falling, reset vertical velocity.
            if max_slope_angle.is_some_and(|angle| normal.angle_between(Vector::Y).abs() <= angle.0)
                && linear_velocity.y < 0.0
            {
                linear_velocity.y = linear_velocity.y.max(0.0);
            }
        }
    }
}
