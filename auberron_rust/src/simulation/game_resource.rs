use godot::prelude::*;
use std::collections::HashMap;
use std::fmt;
use strum::{AsRefStr, EnumCount, FromRepr, VariantArray};

use crate::utils::math::float_eq;

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
    FromRepr,
)]
#[godot(via = i64)]
pub enum GameResourceType {
    Solarite = 0,
    Gold = 1,
}

impl From<GameResourceType> for usize {
    fn from(value: GameResourceType) -> usize {
        return value as usize;
    }
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumCount, VariantArray, AsRefStr, FromRepr)]
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

impl From<GameResourceDynamicsType> for usize {
    fn from(value: GameResourceDynamicsType) -> usize {
        return value as usize;
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
#[derive(Clone, Copy)]
pub struct GameResourceContribution {
    pub contribution_id: ContributionId,
    pub source_id: ContributorId,
    pub resource_type: GameResourceType,
    pub dynamics_type: GameResourceDynamicsType,
    pub amount: f64,
}

impl GameResourceContribution {
    pub fn new(
        contribution_id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        dynamics_type: GameResourceDynamicsType,
        amount: f64,
    ) -> Self {
        return Self {
            contribution_id,
            source_id,
            resource_type,
            dynamics_type,
            amount,
        };
    }

    pub fn new_additive(
        contribution_id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            contribution_id,
            source_id,
            resource_type,
            GameResourceDynamicsType::Additive,
            amount,
        );
    }

    pub fn new_additive_multiplier(
        contribution_id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            contribution_id,
            source_id,
            resource_type,
            GameResourceDynamicsType::MultiplyAdd,
            amount,
        );
    }

    pub fn new_compounding_multiplier(
        contribution_id: ContributionId,
        source_id: ContributorId,
        resource_type: GameResourceType,
        amount: f64,
    ) -> Self {
        return GameResourceContribution::new(
            contribution_id,
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

// region: LedgerError
#[derive(Debug)]
pub enum LedgerError {
    UnknownContributor {
        id: ContributorId,
    },

    UnknownContribution {
        source_id: ContributorId,
        contribution_id: ContributionId,
    },
}
// endregion

pub struct GameResourceLedger {
    contributor_states: HashMap<ContributorId, bool>,
    contributions: Vec<GameResourceContribution>,

    // Cache
    dirty: bool,
    cached_dynamics: GameResourceDynamics,
    cached_rates: GameResourceAmounts,
}

impl GameResourceLedger {
    pub fn new() -> Self {
        return Self {
            contributor_states: HashMap::with_capacity(16),
            contributions: Vec::with_capacity(32),

            dirty: false,
            cached_dynamics: GameResourceDynamics::default(),
            cached_rates: ZERO_RESOURCES,
        };
    }

    // region: Contributors/Contributions

    /// Enable/Disable the contributor in the Ledger.
    /// The resources cache is invalidated if this change affects any non-neutral contribution.
    ///
    /// Returns "Ok(())" on success, including the no-op case where the given contributor
    /// was already in the requested state.
    ///
    /// # Errors
    /// Returns [`LedgerError::UnknownContributor`] if the requested contributor `id`
    /// was not registered in the Ledger.
    pub fn enable_contributor(
        &mut self,
        id: ContributorId,
        enabled: bool,
    ) -> Result<(), LedgerError> {
        let current_state = self
            .contributor_states
            .get_mut(&id)
            .ok_or(LedgerError::UnknownContributor { id })?;

        if *current_state == enabled {
            return Ok(());
        }
        *current_state = enabled;

        self.dirty |= self
            .contributions
            .iter()
            .any(|c| c.source_id == id && !c.is_neutral());

        return Ok(());
    }

    pub fn contributor_enabled(&self, id: ContributorId) -> bool {
        return self.contributor_states.get(&id).copied().unwrap_or(false);
    }

    pub fn contains_contributor(self, id: ContributorId) -> bool {
        return self.contributor_states.contains_key(&id);
    }

    pub fn contains_contribution(
        self,
        source_id: ContributorId,
        contribution_id: ContributionId,
    ) -> bool {
        for contribution in self.contributions {
            if (contribution.source_id == source_id)
                && (contribution.contribution_id == contribution_id)
            {
                return true;
            }
        }
        return false;
    }

    pub fn contribution_is_identity(&self, contribution: &GameResourceContribution) -> bool {
        return !self.contributor_enabled(contribution.source_id) || contribution.is_neutral();
    }

    /// Add a contribution to the ledger.
    /// If the corresponding contributor is not registered yet, it is added and enabled.
    /// Invalidates the cache, unless the contribution is an identity.
    pub fn add_contribution(&mut self, contribution: GameResourceContribution) {
        self.contributor_states
            .entry(contribution.source_id)
            .or_insert(true);
        self.dirty |= !self.contribution_is_identity(&contribution);
        self.contributions.push(contribution);
    }

    /// Add all contributions then invalidates the cache, unless all contributions were identities.
    /// Corresponding contributors are registered if they were not already.
    pub fn add_contributions(&mut self, contributions: Vec<GameResourceContribution>) {
        self.contributions.reserve(contributions.len());
        for contribution in contributions {
            self.add_contribution(contribution);
        }
    }

    pub fn update_contribution_amount(
        &mut self,
        source_id: ContributorId,
        contribution_id: ContributionId,
        amount: f64,
    ) -> Result<(), LedgerError> {
        let contribution = self
            .contributions
            .iter_mut()
            .find(|c| c.source_id == source_id && c.contribution_id == contribution_id)
            .ok_or(LedgerError::UnknownContribution {
                source_id: source_id,
                contribution_id: contribution_id,
            })?;

        if !float_eq(contribution.amount, amount) {
            contribution.amount = amount;
            self.dirty = true;
        }

        return Ok(());
    }
    // endregion

    // region: Cache Update(s)
    /// Wipes the resource Dynamics cache and rebuilds it based on active contributions.
    /// The resulting rates are computed and stored in the resource Rates cache.
    fn recompute(&mut self) {
        self.cached_dynamics.wipe();

        for contribution in &self.contributions {
            if !self.contributor_enabled(contribution.source_id) {
                continue;
            }
            self.cached_dynamics.update(
                contribution.resource_type,
                contribution.dynamics_type,
                contribution.amount,
            );
        }

        self.cached_rates = self.cached_dynamics.resource_rates();
        self.dirty = false;
    }

    // endregion

    /// Computes or gets the current resource rates.
    /// If the cache is marked as dirty/invalid, it is recomputed first.
    pub fn compute_resource_rates(&mut self) -> GameResourceAmounts {
        if self.dirty {
            self.recompute();
        }

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

// region: Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::enum_tests::assert_enum_discriminants_self_index;
    use crate::utils::math_tests::assert_float_eq;

    #[test]
    fn test_ledger_resource_rate_example() {
        let test_resource_type = GameResourceType::Solarite;
        let contributions = [
            GameResourceContribution::new_additive(
                ContributionId(0),
                ContributorId(0),
                test_resource_type,
                7.0,
            ),
            GameResourceContribution::new_additive_multiplier(
                ContributionId(1),
                ContributorId(0),
                test_resource_type,
                0.2,
            ),
            GameResourceContribution::new_additive_multiplier(
                ContributionId(2),
                ContributorId(1),
                test_resource_type,
                0.1,
            ),
            GameResourceContribution::new_compounding_multiplier(
                ContributionId(3),
                ContributorId(0),
                test_resource_type,
                1.2,
            ),
            GameResourceContribution::new_compounding_multiplier(
                ContributionId(4),
                ContributorId(1),
                test_resource_type,
                1.1,
            ),
        ];

        let mut ledger = GameResourceLedger::default();
        ledger.add_contributions(contributions.to_vec());
        let resource_rates = ledger.compute_resource_rates();
        let resource_rate = resource_rates[test_resource_type as usize];
        assert_float_eq(resource_rate, 12.012);
    }

    #[test]
    fn test_resource_amount_update() {
        let test_resource_type = GameResourceType::Solarite;
        let contribution = GameResourceContribution::new_additive(
            ContributionId(0),
            ContributorId(0),
            test_resource_type,
            6.0,
        );

        let mut ledger = GameResourceLedger::default();
        ledger.add_contribution(contribution);
        ledger
            .update_contribution_amount(
                contribution.source_id,
                contribution.contribution_id,
                contribution.amount + 1.0,
            )
            .expect("Contribution IDs were re-used, the update should always succeed.");

        let resource_rates = ledger.compute_resource_rates();
        let resource_rate = resource_rates[test_resource_type as usize];

        assert_float_eq(resource_rate, 7.0);
    }

    #[test]
    fn test_advance_example() {
        let test_resource_type = GameResourceType::Solarite;
        let delta = 2.0;

        let mut positive_rates = ZERO_RESOURCES;
        positive_rates[test_resource_type as usize] = 3.0;

        let mut negative_rates = ZERO_RESOURCES;
        negative_rates[test_resource_type as usize] = 1.0;

        let mut expected_amounts = ZERO_RESOURCES;
        expected_amounts[test_resource_type as usize] = 4.0;

        let mut resources = GameResources::default();
        resources.integrate(positive_rates, negative_rates, delta);

        assert_eq!(resources.amounts, expected_amounts);
    }

    #[test]
    fn test_updating_unregistered_contribution_fails() {
        let contribution = GameResourceContribution::new_additive(
            ContributionId(0),
            ContributorId(0),
            GameResourceType::Solarite,
            6.0,
        );

        let mut ledger = GameResourceLedger::default();
        ledger.add_contribution(contribution);

        let contribution_result =
            ledger.update_contribution_amount(ContributorId(0), ContributionId(1), 1.0);
        let contributor_result =
            ledger.update_contribution_amount(ContributorId(1), ContributionId(0), 1.0);

        assert!(matches!(
            contribution_result,
            Err(LedgerError::UnknownContribution {
                source_id: ContributorId(0),
                contribution_id: ContributionId(1)
            })
        ));
        assert!(matches!(
            contributor_result,
            Err(LedgerError::UnknownContribution {
                source_id: ContributorId(1),
                contribution_id: ContributionId(0)
            })
        ));
    }

    #[test]
    fn test_neutral_contribution_does_not_dirty() {
        let mut ledger = GameResourceLedger::default();
        for (i, game_resource_dynamics_type) in
            GameResourceDynamicsType::VARIANTS.iter().enumerate()
        {
            let contribution = GameResourceContribution::new(
                ContributionId(i as u32),
                ContributorId(0),
                GameResourceType::Solarite,
                *game_resource_dynamics_type,
                game_resource_dynamics_type.neutral(),
            );
            ledger.add_contribution(contribution);

            assert!(!ledger.dirty);
            assert_eq!(ledger.compute_resource_rates(), ZERO_RESOURCES);
        }
    }

    #[test]
    fn test_neutral_contributions_do_not_dirty() {
        let mut ledger = GameResourceLedger::default();
        let mut contributions: Vec<GameResourceContribution> =
            Vec::with_capacity(GameResourceDynamicsType::COUNT);
        for (i, game_resource_dynamics_type) in
            GameResourceDynamicsType::VARIANTS.iter().enumerate()
        {
            let contribution = GameResourceContribution::new(
                ContributionId(i as u32),
                ContributorId(0),
                GameResourceType::Solarite,
                *game_resource_dynamics_type,
                game_resource_dynamics_type.neutral(),
            );
            contributions.push(contribution);
        }
        ledger.add_contributions(contributions);

        assert!(!ledger.dirty);
        assert_eq!(ledger.compute_resource_rates(), ZERO_RESOURCES);
    }

    #[test]
    fn test_toggling_non_neutral_contributor_changes_rate() {
        let mut ledger = GameResourceLedger::default();

        let test_resource_type = GameResourceType::Solarite;
        let contribution = GameResourceContribution::new_additive(
            ContributionId(0),
            ContributorId(0),
            test_resource_type,
            6.0,
        );
        ledger.add_contribution(contribution);
        let initial_resource_rate = ledger.compute_resource_rates()[test_resource_type as usize];

        ledger
            .enable_contributor(contribution.source_id, false)
            .expect("Contributor was added through `add_contribution`");
        let off_resource_rate = ledger.compute_resource_rates()[test_resource_type as usize];
        assert_ne!(initial_resource_rate, off_resource_rate);

        ledger
            .enable_contributor(contribution.source_id, true)
            .expect("Contributor was added through `add_contribution`");
        let on_resource_rate = ledger.compute_resource_rates()[test_resource_type as usize];
        assert_ne!(off_resource_rate, on_resource_rate);

        assert_float_eq(initial_resource_rate, on_resource_rate);
    }

    #[test]
    fn test_toggling_neutral_contributor_does_not_dirty() {
        let mut ledger = GameResourceLedger::default();

        let test_contributor_id = ContributorId(0);
        let mut contributions: Vec<GameResourceContribution> =
            Vec::with_capacity(GameResourceDynamicsType::COUNT);
        for (i, game_resource_dynamics_type) in
            GameResourceDynamicsType::VARIANTS.iter().enumerate()
        {
            let contribution = GameResourceContribution::new(
                ContributionId(i as u32),
                test_contributor_id,
                GameResourceType::Solarite,
                *game_resource_dynamics_type,
                game_resource_dynamics_type.neutral(),
            );
            contributions.push(contribution);
        }
        ledger.add_contributions(contributions);

        ledger.compute_resource_rates();
        ledger
            .enable_contributor(test_contributor_id, false)
            .expect("Contributor was manually registered");
        assert!(!ledger.dirty);

        ledger
            .enable_contributor(test_contributor_id, true)
            .expect("Contributor was manually registered");
        assert!(!ledger.dirty);
    }

    #[test]
    fn test_toggling_non_neutral_contributor_dirties() {
        let mut ledger = GameResourceLedger::default();

        let mut contributions: Vec<GameResourceContribution> =
            Vec::with_capacity(GameResourceDynamicsType::COUNT);
        for (i, game_resource_dynamics_type) in
            GameResourceDynamicsType::VARIANTS.iter().enumerate()
        {
            let contribution = GameResourceContribution::new(
                ContributionId(0),
                ContributorId(i as u32),
                GameResourceType::Solarite,
                *game_resource_dynamics_type,
                game_resource_dynamics_type.neutral() + 1.0,
            );
            contributions.push(contribution);
        }
        ledger.add_contributions(contributions);

        for game_resource_dynamics_type in GameResourceDynamicsType::VARIANTS {
            ledger.compute_resource_rates();
            ledger
                .enable_contributor(ContributorId(*game_resource_dynamics_type as u32), false)
                .expect("Contributor was manually registered");
            assert!(ledger.dirty);
        }

        for game_resource_dynamics_type in GameResourceDynamicsType::VARIANTS {
            ledger.compute_resource_rates();
            ledger
                .enable_contributor(ContributorId(*game_resource_dynamics_type as u32), true)
                .expect("Contributor was manually registered");
            assert!(ledger.dirty);
        }
    }

    #[test]
    fn test_toggling_contributor_only_affects_matching_id() {
        let mut ledger = GameResourceLedger::default();
        let contribution_0 = GameResourceContribution::new_additive(
            ContributionId(0),
            ContributorId(0),
            GameResourceType::Solarite,
            1.0,
        );
        ledger.add_contribution(contribution_0);
        let contribution_1 = GameResourceContribution::new_additive(
            ContributionId(0),
            ContributorId(1),
            GameResourceType::Solarite,
            1.0,
        );
        ledger.add_contribution(contribution_1);
    }

    #[test]
    fn test_resource_type_self_indexes() {
        assert_enum_discriminants_self_index::<GameResourceType>();
    }

    #[test]
    fn test_resource_dynamics_type_self_indexes() {
        assert_enum_discriminants_self_index::<GameResourceDynamicsType>();
    }
}
// endregion
