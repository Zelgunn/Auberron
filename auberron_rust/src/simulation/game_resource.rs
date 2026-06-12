use godot::prelude::*;
use std::fmt;
use strum::{AsRefStr, EnumCount, VariantArray};

// region: GameResourceType
#[derive(
    GodotConvert,
    Var,
    Export,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    EnumCount,
    VariantArray,
    AsRefStr,
)]
#[godot(via = i64)]
pub enum GameResourceType {
    Solarite = 0,
}
// endregion

// region: GameResources
pub struct GameResources {
    amounts: [f64; GameResourceType::COUNT],
}

impl GameResources {
    pub fn new() -> Self {
        return Self {
            amounts: [0.0; GameResourceType::COUNT],
        };
    }
}

impl Default for GameResources {
    fn default() -> Self {
        return Self::new();
    }
}

impl fmt::Debug for GameResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = f.debug_struct("GameResources");
        for game_resource_type in GameResourceType::VARIANTS {
            let amount = self.amounts[*game_resource_type as usize];
            if amount != 0.0 {
                text.field(game_resource_type.as_ref(), &amount);
            }
        }
        return text.finish();
    }
}
// endregion

// region: GameResourceDynamicsType
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    EnumCount,
    VariantArray,
    AsRefStr,
)]
pub enum GameResourceDynamicsType {
    Additive = 0,
    MultiplyAdd = 1,
    MultiplyCompound = 2,
}

impl GameResourceDynamicsType {
    pub const fn neutral(self) -> f64 {
        return match self {
            Self::Additive => 0.0,
            Self::MultiplyAdd => 0.0,
            Self::MultiplyCompound => 1.0,
        };
    }
}

// endregion

// region: GameResourceDynamics
pub struct GameResourceDynamics {
    amounts: [[f64; GameResourceType::COUNT]; GameResourceDynamicsType::COUNT],
}

impl GameResourceDynamics {
    pub fn new() -> Self {
        return Self {
            amounts: std::array::from_fn(|dyn_type| {
                [GameResourceDynamicsType::VARIANTS[dyn_type].neutral(); GameResourceType::COUNT]
            })
        };
    }

    pub fn wipe(&mut self) {
        for game_resource_dynamics_type in GameResourceDynamicsType::VARIANTS {
            self.amounts[*game_resource_dynamics_type as usize] = [game_resource_dynamics_type.neutral(); GameResourceType::COUNT]
        }
    }
}

impl Default for GameResourceDynamics {
    fn default() -> Self {
        return Self::new();
    }
}

// endregion
