pub use bevy_xpbd_2d::prelude::*;

#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    Enemy,
    Climbable,
    Interactable,
    Ground,
}
