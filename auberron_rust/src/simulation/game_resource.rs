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
pub const ZERO_RESOURCES: GameResourceAmounts = [0.0; GameResourceType::COUNT];

pub struct GameResources {
    amounts: GameResourceAmounts,
}

impl GameResources {
    pub const fn new() -> Self {
        return Self {
            amounts: ZERO_RESOURCES,
        };
    }

    pub fn integrate(
        &mut self,
        positive_rates: GameResourceAmounts,
        negative_rates: GameResourceAmounts,
        delta: f64,
    ) {
        for ((income, consumption), amount) in positive_rates
            .into_iter()
            .zip(negative_rates)
            .zip(self.amounts.iter_mut())
        {
            *amount += (income - consumption) * delta;
        }
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumCount, VariantArray, AsRefStr)]
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
        };
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
            }),
        };
    }

    pub fn amount(
        &self,
        resource_type: GameResourceType,
        dynamics_type: GameResourceDynamicsType,
    ) -> f64 {
        return self.amounts[dynamics_type as usize][resource_type as usize];
    }

    pub fn resource_rate(&self, resource_type: GameResourceType) -> f64 {
        use GameResourceDynamicsType::*;
        return self.amount(resource_type, Additive)
            * (1.0 + self.amount(resource_type, MultiplyAdd))
            * self.amount(resource_type, MultiplyCompound);
    }

    // todo: cache result from this as well
    pub fn resource_rates(&self) -> GameResourceAmounts {
        return std::array::from_fn(|i| self.resource_rate(GameResourceType::VARIANTS[i]));
    }

    pub fn wipe(&mut self) {
        for dynamics_type in GameResourceDynamicsType::VARIANTS {
            self.amounts[*dynamics_type as usize] =
                [dynamics_type.neutral(); GameResourceType::COUNT]
        }
    }

    pub fn update(
        &mut self,
        resource_type: GameResourceType,
        dynamics_type: GameResourceDynamicsType,
        amount: f64,
    ) {
        let current_value: f64 = self.amounts[dynamics_type as usize][resource_type as usize];
        self.amounts[dynamics_type as usize][resource_type as usize] =
            dynamics_type.combine(current_value, amount);
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
pub struct ContributionId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ContributorId(pub u32);

// region: GameResourceContribution
pub struct GameResourceContribution {
    pub id: ContributionId,
    pub source: ContributorId,
    pub resource_type: GameResourceType,
    pub dynamics_type: GameResourceDynamicsType,
    pub amount: f64,
    pub enabled: bool,
}

impl GameResourceContribution {
    fn new(
        id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        dynamics_type: GameResourceDynamicsType,
        amount: f64,
    ) -> Self {
        return Self {
            id,
            source: source_id,
            resource_type,
            dynamics_type,
            amount,
            enabled: true,
        };
    }

    fn new_additive(
        id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            id,
            source_id,
            resource_type,
            GameResourceDynamicsType::Additive,
            amount,
        );
    }

    fn new_additive_multiplier(
        id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            id,
            source_id,
            resource_type,
            GameResourceDynamicsType::MultiplyAdd,
            amount,
        );
    }

    fn new_compounding_multiplier(
        id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            id,
            source_id,
            resource_type,
            GameResourceDynamicsType::MultiplyCompound,
            amount,
        );
    }

    fn is_neutral(&self) -> bool {
        return self.amount == self.dynamics_type.neutral();
    }
}
// endregion

pub struct GameResourceLedger {
    contributions: Vec<GameResourceContribution>,

    // Cache
    cached_dynamics: GameResourceDynamics,
    cached_rates: GameResourceAmounts,
}

impl GameResourceLedger {
    pub fn new() -> Self {
        return Self {
            contributions: Vec::with_capacity(16),
            cached_dynamics: GameResourceDynamics::default(),
            cached_rates: ZERO_RESOURCES,
        };
    }

    // region: Contributions/Contributors
    pub fn add_contribution(&mut self, contribution: GameResourceContribution) {
        let dirty: bool = contribution.enabled && contribution.is_neutral();

        self.contributions.push(contribution);

        if dirty {
            self.recompute();
        }
    }

    /// Add all contributions then updates the cache.
    /// Assumes at least one contributor is active and non-neutral.
    /// Use `add_contribution` instead if all contributions are neutral.
    ///
    /// # Arguments
    /// * `contributions`: a non-neutral set of contributions
    pub fn add_contributions(&mut self, contributions: Vec<GameResourceContribution>) {
        self.contributions.extend(contributions);
        self.recompute();
    }

    /// Enable/Disable contributions associated to the given contributor ID.
    /// If the state of at least a non-neutral contribution changes, recomputes the cache.
    pub fn enable_contributor(&mut self, id: ContributorId, enabled: bool) {
        let mut dirty: bool = false;
        for contribution in &mut self.contributions {
            if contribution.source == id {
                if contribution.enabled != enabled {
                    contribution.enabled = enabled;
                    dirty |= !contribution.is_neutral();
                }
            }
        }

        if dirty {
            self.recompute();
        }
    }
    // endregion

    // region: Update(s)
    /// Wipes the resource Dynamics cache and rebuilds it based on active contributions.
    /// The resulting rates are computed and stored in the resource Rates cache.
    fn recompute(&mut self) {
        self.cached_dynamics.wipe();

        for contribution in &self.contributions {
            if !contribution.enabled {
                continue;
            }
            self.cached_dynamics.update(
                contribution.resource_type,
                contribution.dynamics_type,
                contribution.amount,
            );
        }

        self.cached_rates = self.cached_dynamics.resource_rates();
    }

    // endregion

    /// Getter for the current cached resource Rates.
    pub fn resource_rates(&self) -> GameResourceAmounts {
        return self.cached_rates;
    }

    //
}

impl Default for GameResourceLedger {
    fn default() -> Self {
        return Self::new();
    }
}

// endregion
