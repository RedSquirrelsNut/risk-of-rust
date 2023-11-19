use bevy::prelude::*;

// pub struct Crowbar {
//     modifier_handle: StatModifierHandleTag,
//     name: String,
// }

// type ReferenceCounted<T> = std::rc::Rc<T>;
// type Weak<T> = std::sync::Weak<T>;
// type InteriorCell<T> = std::sync::Arc<std::sync::Mutex<T>>;
//
// pub type StatModifierHandle = ReferenceCounted<StatModifierHandleTag>;
//
// #[inline]
// fn new_interior_cell<T>(value: T) -> InteriorCell<T> {
//     std::sync::Arc::new(std::sync::Mutex::new(value))
// }
//
// #[derive(Copy, Clone, Debug)]
// pub enum StatModifier {
//     Flat(f32),
//     PercentAdd(f32),
//     PercentMultiply(f32),
// }
//
// #[derive(Copy, Clone, Debug, Default)]
// pub struct StatModifierHandleTag;
//
// #[derive(Clone, Debug)]
// struct ModifierMeta {
//     modifier: StatModifier,
//     order: i32,
//     owner_modifier_weak: Weak<StatModifierHandleTag>,
// }
//
// pub struct Stat {
//     pub base_value: f32,
//     value: InteriorCell<f32>,
//     modifiers: InteriorCell<Vec<>>
// }
//
// impl Stat {
//     pub fn new(base_value: f32) -> Self {
//         Self {
//             base_value,
//             value: new_interior_cell(base_value),
//             modifiers: new_interior_cell(vec![]),
//         }
//     }
//
//     pub fn add_modifier(&mut self, modifier: StatModifier) -> StatModifierHandle {
//         let handle = ReferenceCounted::new(StatModifierHandleTag);
//         let meta = ModifierMeta {
//             order:
//         }
//     }
// }
pub enum StatEffectType {
    Speed(f32),
    JumpHeight(f32),
}

pub struct StatEffect {
    modifiers: Vec<StatEffectType>,
}

pub struct StatPair {
    base: f32,
    current: f32,
}

#[derive(Component, Reflect)]
pub struct SpeedStat(pub f32);

#[derive(Component, Reflect)]
pub struct JumpsStat {
    pub max_jumps: u32,
    pub jumps_left: u32,
    pub jump_height: f32,
}

impl JumpsStat {
    pub fn new(max_jumps: u32, jump_height: f32) -> Self {
        Self {
            max_jumps,
            jumps_left: max_jumps,
            jump_height,
        }
    }
}

#[derive(Component, Reflect)]
pub struct HealthStat(pub f32);

#[derive(Component, Reflect)]
pub struct DamageStat(pub f32);

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpeedStat>()
            .register_type::<JumpsStat>()
            .register_type::<HealthStat>()
            .register_type::<DamageStat>();
    }
}
