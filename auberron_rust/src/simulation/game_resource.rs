use godot::prelude::*;
use std::fmt;
use strum::{AsRefStr, EnumCount, VariantArray};

// region: GameResources
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
pub type GameResourceAmounts = [f64; GameResourceType::COUNT];

pub struct GameResources {
    amounts: GameResourceAmounts,
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

// region: GameResourceDynamics
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

    pub const fn combine(self, accumulator: f64, value: f64) -> f64 {
        return match self {
            Self::Additive => accumulator + value,
            Self::MultiplyAdd => accumulator + value,
            Self::MultiplyCompound => accumulator * value,
        }
    }
}

// endregion

pub struct GameResourceDynamics {
    amounts: [GameResourceAmounts; GameResourceDynamicsType::COUNT],
}

impl GameResourceDynamics {
    pub fn new() -> Self {
        return Self {
            amounts: std::array::from_fn(|dyn_type| {
                [GameResourceDynamicsType::VARIANTS[dyn_type].neutral(); GameResourceType::COUNT]
            })
        };
    }

    pub fn get_amount(&self, resource_type: GameResourceType, dynamics_type: GameResourceDynamicsType) -> f64 {
        return self.amounts[dynamics_type as usize][resource_type as usize];
    }

    pub fn compute_resource_rate(&self, resource_type: GameResourceType) -> f64 {
        use GameResourceDynamicsType::*;
        return self.get_amount(resource_type, Additive) * (1.0 + self.get_amount(resource_type, MultiplyAdd)) * self.get_amount(resource_type, MultiplyCompound);
    }

    // todo: cache result from this as well
    pub fn compute_resource_rates(&self) -> GameResourceAmounts {
        return std::array::from_fn(|i| self.compute_resource_rate(GameResourceType::VARIANTS[i]));
    }

    pub fn wipe(&mut self) {
        for dynamics_type in GameResourceDynamicsType::VARIANTS {
            self.amounts[*dynamics_type as usize] = [dynamics_type.neutral(); GameResourceType::COUNT]
        }
    }

    pub fn update(&mut self, resource_type: GameResourceType, dynamics_type: GameResourceDynamicsType, amount: f64) {
        let current_value: f64 = self.amounts[dynamics_type as usize][resource_type as usize];
        self.amounts[dynamics_type as usize][resource_type as usize] = dynamics_type.combine(current_value, amount);
    }
}

impl Default for GameResourceDynamics {
    fn default() -> Self {
        return Self::new();
    }
}

// endregion

// region: GameResourceLedger
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ContributorId(pub u32);

// region: GameResourceContribution
pub struct GameResourceContribution {
    pub source: ContributorId,
    pub resource_type: GameResourceType,
    pub dynamics_type: GameResourceDynamicsType,
    pub amount: f64,
    pub enabled: bool
}

impl GameResourceContribution {
    fn new(source_id: ContributorId, resource_type: GameResourceType, dynamics_type: GameResourceDynamicsType, amount: f64) -> Self {
        return Self {
            source: source_id,
            resource_type: resource_type,
            dynamics_type: dynamics_type,
            amount: amount,
            enabled: true
        };
    }

    fn new_additive(source_id: ContributorId, resource_type: GameResourceType, amount: f64) -> Self {
        return GameResourceContribution::new(source_id, resource_type, GameResourceDynamicsType::Additive, amount);
    }

    fn new_additive_multiplier(source_id: ContributorId, resource_type: GameResourceType, amount: f64) -> Self {
        return GameResourceContribution::new(source_id, resource_type, GameResourceDynamicsType::MultiplyAdd, amount);
    }

    fn new_compounding_multiplier(source_id: ContributorId, resource_type: GameResourceType, amount: f64) -> Self {
        return GameResourceContribution::new(source_id, resource_type, GameResourceDynamicsType::MultiplyCompound, amount);
    }
}
// endregion

pub struct GameResourceLedger {
    contributions: Vec<GameResourceContribution>,
    cache: GameResourceDynamics
}

impl GameResourceLedger {
    fn new() -> Self {
        return Self {
            contributions: Vec::with_capacity(16),
            cache: GameResourceDynamics::default()
        }
    }

    fn add_contribution(&mut self, contribution: GameResourceContribution) {
        self.contributions.push(contribution);
    }

    fn enable_contributor(&mut self, id: ContributorId, enabled: bool) {
        let mut dirty: bool = false;
        for contribution in &mut self.contributions {
            if contribution.source == id {
                if contribution.enabled != enabled {
                    contribution.enabled = enabled;
                    dirty = true;
                }
            }
        }

        if dirty {
            self.recompute();
        }
    }

    fn recompute(&mut self) {
        self.cache.wipe();
        for contribution in &self.contributions {
            if !contribution.enabled { continue; }
            self.cache.update(contribution.resource_type, contribution.dynamics_type, contribution.amount);
        }
    }
}

// endregion