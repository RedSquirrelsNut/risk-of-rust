use bevy::{ecs::query::Has, prelude::*};

#[derive(Component)]
pub struct Rope;

#[derive(Event)]
pub enum RopeAction {
    Mount,
    Dismount,
}

pub struct RopePlugin;

impl Plugin for RopePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RopeAction>();
    }
}
