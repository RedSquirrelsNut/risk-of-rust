pub use bevy::prelude::*;
pub use bevy_xpbd_2d::prelude::*;

pub use crate::game::physics_layers::Layer;

#[derive(Component)]
pub struct Dummy;

pub fn spawn_temp_dummy(mut commands: Commands, asset: Res<AssetServer>) {
    commands.spawn((
        Name::new("Dummy"),
        Dummy,
        SpriteBundle {
            texture: asset.load("sprites/dummy.png"),
            transform: Transform::from_xyz(0.0, -215.0, -1.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(15.0, 20.0),
        Friction::new(0.0),
        CollisionLayers::new([Layer::Enemy], [Layer::Ground]),
    ));
}
