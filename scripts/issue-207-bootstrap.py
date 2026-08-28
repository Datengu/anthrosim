from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[1]


def replace(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"replacement anchor not found in {path}: {old[:80]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")


# ---------------------------------------------------------------------------
# Versioned experiment configuration: historical baseline stays omitted;
# the structural alternative is explicit and provenance-bearing.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/config.rs",
    """    pub founder_population: Option<FounderPopulationDefinition>,\n    pub demography: DemographyConfig,""",
    """    pub founder_population: Option<FounderPopulationDefinition>,\n    /// Optional structural household-lifecycle treatment. `None` preserves the historical\n    /// founder-defined household lifecycle exactly; configured alternatives are explicit\n    /// scientific treatments included in ordinary experiment identity and provenance.\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub household_lifecycle: Option<HouseholdLifecycleConfig>,\n    pub demography: DemographyConfig,""",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    """            founder_population: None,\n            demography: DemographyConfig::synthetic_validation_v1(),""",
    """            founder_population: None,\n            household_lifecycle: None,\n            demography: DemographyConfig::synthetic_validation_v1(),""",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    """    pub fn with_founder_population(\n        mut self,\n        founder_population: FounderPopulationDefinition,\n    ) -> Self {\n        self.population.initialization = PopulationInitialization::DeclaredFounderStateV1;\n        self.founder_population = Some(founder_population);\n        self\n    }\n\n    #[must_use]\n    pub fn with_demography""",
    """    pub fn with_founder_population(\n        mut self,\n        founder_population: FounderPopulationDefinition,\n    ) -> Self {\n        self.population.initialization = PopulationInitialization::DeclaredFounderStateV1;\n        self.founder_population = Some(founder_population);\n        self\n    }\n\n    #[must_use]\n    pub fn with_household_lifecycle(\n        mut self,\n        household_lifecycle: HouseholdLifecycleConfig,\n    ) -> Self {\n        self.household_lifecycle = Some(household_lifecycle);\n        self\n    }\n\n    #[must_use]\n    pub fn with_demography""",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    """/// Annual event probability for a half-open age interval.""",
    """/// Stable identity for the historical baseline in which founder-defined households persist\n/// for the complete run except for extinction. It remains represented by an omitted optional\n/// lifecycle field so pre-#207 synthetic experiment serialization is unchanged.\npub const FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID: &str = \"fixed_founder_v1\";\n\n/// Stable identity for the deliberately neutral structural-sensitivity alternative introduced by\n/// #207. It is a stress-test mechanism, not a calibrated household-formation model.\npub const DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str =\n    \"deterministic_size_fission_v1\";\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\npub struct HouseholdLifecycleConfig {\n    pub schema_version: u32,\n    pub model_id: String,\n    pub provenance: ParameterProvenance,\n    /// Maximum number of living members retained in one household after an annual lifecycle\n    /// boundary. Oversized at-residence households are partitioned deterministically into the\n    /// minimum number of balanced co-resident groups needed to satisfy this ceiling.\n    pub max_living_members: u16,\n}\n\nimpl HouseholdLifecycleConfig {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;\n\n    #[must_use]\n    pub fn deterministic_size_fission_v1(max_living_members: u16) -> Self {\n        Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            model_id: DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),\n            provenance: ParameterProvenance::SyntheticValidation,\n            max_living_members,\n        }\n    }\n}\n\n/// Annual event probability for a half-open age interval.""",
)

# ---------------------------------------------------------------------------
# Population topology mutation. Only living membership changes; persistent
# residence and person identity remain untouched.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/population.rs",
    """pub(crate) struct HouseholdRelocationOutcome {\n    pub people_moved: u64,\n    pub condition_loss_total: u64,\n}\n""",
    """pub(crate) struct HouseholdRelocationOutcome {\n    pub people_moved: u64,\n    pub condition_loss_total: u64,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]\npub(crate) struct HouseholdFissionOutcome {\n    pub households_created: u64,\n    pub people_reassigned: u64,\n}\n""",
)
replace(
    "crates/anthrosim-core/src/population.rs",
    """    pub(crate) fn apply_household_relocations(\n""",
    """    pub(crate) fn fission_oversized_households(\n        &mut self,\n        max_living_members: u16,\n        eligible_households: &[bool],\n    ) -> Result<HouseholdFissionOutcome, PopulationError> {\n        if max_living_members == 0 {\n            return Err(PopulationError::ZeroLifecycleHouseholdSize);\n        }\n        let original_household_count = self.household_count();\n        if eligible_households.len() != original_household_count {\n            return Err(PopulationError::HouseholdLifecycleShapeMismatch);\n        }\n\n        let ceiling = usize::from(max_living_members);\n        let mut outcome = HouseholdFissionOutcome::default();\n        for household_index in 0..original_household_count {\n            if !eligible_households[household_index] {\n                continue;\n            }\n            let household = HouseholdId::new(\n                u64::try_from(household_index)\n                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?\n                    .checked_add(1)\n                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?,\n            );\n            let living_members = (0..self.person_count())\n                .filter(|&person_index| {\n                    self.is_alive_index(person_index)\n                        && self.households[person_index] == household\n                })\n                .collect::<Vec<_>>();\n            if living_members.len() <= ceiling {\n                continue;\n            }\n\n            // Use the minimum number of groups needed to obey the configured ceiling, then\n            // balance group sizes so deterministic fission does not manufacture avoidable\n            // singleton households. Stable PersonId order is the explicit neutral partition rule.\n            let group_count = living_members.len().div_ceil(ceiling);\n            let base_group_size = living_members.len() / group_count;\n            let larger_group_count = living_members.len() % group_count;\n            let source_group_size =\n                base_group_size + usize::from(larger_group_count > 0);\n            let residence = self.household_locations[household_index];\n            let mut cursor = source_group_size;\n\n            for group_index in 1..group_count {\n                let group_size =\n                    base_group_size + usize::from(group_index < larger_group_count);\n                let new_household_raw = u64::try_from(self.household_locations.len())\n                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?\n                    .checked_add(1)\n                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?;\n                let new_household = HouseholdId::new(new_household_raw);\n                self.household_locations.push(residence);\n                for &person_index in &living_members[cursor..cursor + group_size] {\n                    self.households[person_index] = new_household;\n                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);\n                }\n                outcome.households_created = outcome.households_created.saturating_add(1);\n                cursor += group_size;\n            }\n\n            debug_assert_eq!(cursor, living_members.len());\n        }\n        Ok(outcome)\n    }\n\n    pub(crate) fn apply_household_relocations(\n""",
)
replace(
    "crates/anthrosim-core/src/population.rs",
    """    #[error(\"household relocation arrays do not match the population household layout\")]\n    RelocationShapeMismatch,""",
    """    #[error(\"household relocation arrays do not match the population household layout\")]\n    RelocationShapeMismatch,\n    #[error(\"household lifecycle eligibility does not match the population household layout\")]\n    HouseholdLifecycleShapeMismatch,\n    #[error(\"household lifecycle maximum living size must be greater than zero\")]\n    ZeroLifecycleHouseholdSize,\n    #[error(\"household identity space is exhausted\")]\n    HouseholdIdSpaceExhausted,""",
)

# ---------------------------------------------------------------------------
# M9 duration accounting and state need to grow with newly created households.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/temporary_resource.rs",
    """    pub(crate) fn validate(\n        &self,\n        household_count: usize,""",
    """    pub(crate) fn extend_households_at_boundary(\n        &mut self,\n        household_count: usize,\n    ) -> Result<(), TemporaryResourceAccountingError> {\n        if household_count < self.households.len() {\n            return Err(TemporaryResourceAccountingError::HouseholdCountContracted {\n                ledger: self.households.len(),\n                requested: household_count,\n            });\n        }\n        if self.period_start_day != self.accounted_until_day {\n            return Err(TemporaryResourceAccountingError::HouseholdTopologyChangedMidPeriod {\n                period_start_day: self.period_start_day,\n                accounted_until_day: self.accounted_until_day,\n            });\n        }\n        self.households\n            .resize(household_count, TemporaryResourcePresenceDays::default());\n        Ok(())\n    }\n\n    pub(crate) fn validate(\n        &self,\n        household_count: usize,""",
)
replace(
    "crates/anthrosim-core/src/temporary_resource.rs",
    """    #[error(\"temporary resource household count mismatch: ledger {ledger}, expected {expected}\")]\n    HouseholdCountMismatch { ledger: usize, expected: usize },""",
    """    #[error(\"temporary resource household count mismatch: ledger {ledger}, expected {expected}\")]\n    HouseholdCountMismatch { ledger: usize, expected: usize },\n    #[error(\"temporary resource household count cannot contract from {ledger} to {requested}\")]\n    HouseholdCountContracted { ledger: usize, requested: usize },\n    #[error(\"household topology changed inside resource period {period_start_day}..{accounted_until_day}\")]\n    HouseholdTopologyChangedMidPeriod {\n        period_start_day: u64,\n        accounted_until_day: u64,\n    },""",
)

replace(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    """    pub(crate) fn process_day(\n""",
    """    pub(crate) fn reconcile_household_topology_at_boundary(\n        &mut self,\n        population: &Population,\n        day: u64,\n    ) -> Result<(), TemporaryMobilityExecutionError> {\n        let previous_count = self.household_count();\n        let next_count = population.household_count();\n        if next_count < previous_count {\n            return Err(TemporaryMobilityExecutionError::HouseholdCountContracted {\n                state: previous_count,\n                population: next_count,\n            });\n        }\n        if next_count == previous_count {\n            self.reconcile_after_population_change(population);\n            return Ok(());\n        }\n\n        if let Some(ledger) = self.resource_ledger.as_mut() {\n            ledger.extend_households_at_boundary(next_count)?;\n        }\n        self.household_presence\n            .resize(next_count, HouseholdPresence::AtResidence);\n        self.active_journeys.resize(next_count, None);\n\n        if let Some(program) = &self.program {\n            let past_trigger_indices = program\n                .schedule\n                .trigger_days\n                .iter()\n                .enumerate()\n                .filter(|(_, trigger_day)| **trigger_day <= day)\n                .map(|(index, _)| {\n                    u32::try_from(index).map_err(|_| TemporaryMobilityExecutionError::TooManyTriggers)\n                })\n                .collect::<Result<Vec<_>, _>>()?;\n            for household_index in previous_count..next_count {\n                let household = HouseholdId::new(\n                    u64::try_from(household_index)\n                        .map_err(|_| TemporaryMobilityExecutionError::HouseholdIdOverflow)?\n                        .checked_add(1)\n                        .ok_or(TemporaryMobilityExecutionError::HouseholdIdOverflow)?,\n                );\n                for &trigger_index in &past_trigger_indices {\n                    self.processed_triggers.push(ProcessedTemporaryTrigger {\n                        trigger_index,\n                        household,\n                    });\n                }\n            }\n            self.processed_triggers.sort_unstable();\n            self.processed_triggers.dedup();\n        }\n\n        self.reconcile_after_population_change(population);\n        Ok(())\n    }\n\n    pub(crate) fn process_day(\n""",
)
replace(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    """    #[error(\"temporary journey ID space exhausted\")]\n    JourneyIdExhausted,""",
    """    #[error(\"temporary journey ID space exhausted\")]\n    JourneyIdExhausted,\n    #[error(\"temporary mobility household identity does not fit supported u64 space\")]\n    HouseholdIdOverflow,\n    #[error(\"temporary mobility household state contracted from {state} to population {population}\")]\n    HouseholdCountContracted { state: usize, population: usize },""",
)

# ---------------------------------------------------------------------------
# M4 scratch state is intentionally non-persistent; resize it lazily when the
# authoritative population creates new household IDs.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/migration.rs",
    """    ) -> Result<(), MigrationError> {\n        if self.living_members.len() != population.household_count()\n            || self.condition_sums.len() != population.household_count()\n            || self.living_conditions.len() != population.household_count()\n            || self.kin_locations.len() != population.household_count()\n            || self.planned_destinations.len() != population.household_count()\n            || self.planned_condition_costs.len() != population.household_count()\n            || self.planned_realized_condition_losses.len() != population.household_count()\n            || self.cell_living.len() != world.cell_count()""",
    """    ) -> Result<(), MigrationError> {\n        let household_count = population.household_count();\n        self.living_members.resize(household_count, 0);\n        self.condition_sums.resize(household_count, 0);\n        self.living_conditions.resize_with(household_count, Vec::new);\n        self.kin_locations.resize_with(household_count, Vec::new);\n        self.planned_destinations\n            .resize(household_count, CellId::INVALID);\n        self.planned_condition_costs.resize(household_count, 0);\n        self.planned_realized_condition_losses\n            .resize(household_count, 0);\n\n        if self.living_members.len() != household_count\n            || self.condition_sums.len() != household_count\n            || self.living_conditions.len() != household_count\n            || self.kin_locations.len() != household_count\n            || self.planned_destinations.len() != household_count\n            || self.planned_condition_costs.len() != household_count\n            || self.planned_realized_condition_losses.len() != household_count\n            || self.cell_living.len() != world.cell_count()""",
)

# ---------------------------------------------------------------------------
# New lifecycle module centralizes validation and annual-boundary coordination.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/src/household_lifecycle.rs",
    r'''
    use thiserror::Error;

    use crate::{
        config::{
            DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID,
            FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID, HouseholdLifecycleConfig,
        },
        ids::HouseholdId,
        population::{HouseholdFissionOutcome, Population, PopulationError},
        temporary_mobility::{TemporaryMobilityExecutionError, TemporaryMobilityState},
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct HouseholdLifecycleOutcome {
        pub households_created: u64,
        pub people_reassigned: u64,
    }

    pub fn validate_household_lifecycle_config(
        config: &HouseholdLifecycleConfig,
    ) -> Result<(), HouseholdLifecycleError> {
        if config.schema_version != HouseholdLifecycleConfig::CURRENT_SCHEMA_VERSION {
            return Err(HouseholdLifecycleError::UnsupportedSchema {
                found: config.schema_version,
                supported: HouseholdLifecycleConfig::CURRENT_SCHEMA_VERSION,
            });
        }
        if config.model_id != DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID {
            return Err(HouseholdLifecycleError::UnsupportedModel {
                model_id: config.model_id.clone(),
            });
        }
        if config.max_living_members == 0 {
            return Err(HouseholdLifecycleError::ZeroMaximumLivingMembers);
        }
        Ok(())
    }

    #[must_use]
    pub fn household_lifecycle_model_id(config: Option<&HouseholdLifecycleConfig>) -> &str {
        config
            .map(|value| value.model_id.as_str())
            .unwrap_or(FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID)
    }

    pub(crate) fn apply_household_lifecycle_at_annual_boundary(
        population: &mut Population,
        temporary_mobility: &mut TemporaryMobilityState,
        config: &HouseholdLifecycleConfig,
        day: u64,
    ) -> Result<HouseholdLifecycleOutcome, HouseholdLifecycleError> {
        validate_household_lifecycle_config(config)?;
        let household_count = population.household_count();
        let mut eligible = Vec::with_capacity(household_count);
        for index in 0..household_count {
            let household = HouseholdId::new(
                u64::try_from(index)
                    .map_err(|_| HouseholdLifecycleError::HouseholdIdOverflow)?
                    .checked_add(1)
                    .ok_or(HouseholdLifecycleError::HouseholdIdOverflow)?,
            );
            eligible.push(
                temporary_mobility
                    .is_at_residence(household)
                    .ok_or(HouseholdLifecycleError::MissingTemporaryPresence { household })?,
            );
        }
        let HouseholdFissionOutcome {
            households_created,
            people_reassigned,
        } = population.fission_oversized_households(config.max_living_members, &eligible)?;
        temporary_mobility.reconcile_household_topology_at_boundary(population, day)?;
        Ok(HouseholdLifecycleOutcome {
            households_created,
            people_reassigned,
        })
    }

    #[derive(Debug, Error)]
    pub enum HouseholdLifecycleError {
        #[error("household lifecycle schema {found} is unsupported; supported schema is {supported}")]
        UnsupportedSchema { found: u32, supported: u32 },
        #[error("household lifecycle model {model_id:?} is unsupported")]
        UnsupportedModel { model_id: String },
        #[error("household lifecycle maximum living members must be greater than zero")]
        ZeroMaximumLivingMembers,
        #[error("household identity does not fit supported u64 space")]
        HouseholdIdOverflow,
        #[error("temporary mobility has no presence state for household {household:?}")]
        MissingTemporaryPresence { household: HouseholdId },
        #[error(transparent)]
        Population(#[from] PopulationError),
        #[error(transparent)]
        TemporaryMobility(#[from] TemporaryMobilityExecutionError),
    }
    ''',
)

# ---------------------------------------------------------------------------
# Integrate annual lifecycle after M2 fertility and before the annual metric
# snapshot in both synthetic and transformed-spatial authoritative hosts.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/simulation.rs",
    """    founder_initialization::FounderGenealogyStatus,\n    manifest::{ArtifactSchemas, RunManifest, RunStatistics, StopReason},""",
    """    founder_initialization::FounderGenealogyStatus,\n    household_lifecycle::{\n        HouseholdLifecycleError, apply_household_lifecycle_at_annual_boundary,\n        validate_household_lifecycle_config,\n    },\n    manifest::{ArtifactSchemas, RunManifest, RunStatistics, StopReason},""",
)
replace(
    "crates/anthrosim-core/src/simulation.rs",
    """            self.temporary_mobility\n                .reconcile_after_population_change(&self.population);\n            self.record_metric_snapshot();\n            match outcome {""",
    """            self.temporary_mobility\n                .reconcile_after_population_change(&self.population);\n            if let Some(household_lifecycle) = self.config.household_lifecycle.clone() {\n                apply_household_lifecycle_at_annual_boundary(\n                    &mut self.population,\n                    &mut self.temporary_mobility,\n                    &household_lifecycle,\n                    self.time.days(),\n                )?;\n            }\n            self.record_metric_snapshot();\n            match outcome {""",
)
replace(
    "crates/anthrosim-core/src/simulation.rs",
    """    validate_demography_config(&config.demography)?;\n    validate_resource_config(&config.resources)?;""",
    """    validate_demography_config(&config.demography)?;\n    if let Some(household_lifecycle) = &config.household_lifecycle {\n        validate_household_lifecycle_config(household_lifecycle)?;\n    }\n    validate_resource_config(&config.resources)?;""",
)
replace(
    "crates/anthrosim-core/src/simulation.rs",
    """    #[error(transparent)]\n    Population(#[from] PopulationError),""",
    """    #[error(transparent)]\n    Population(#[from] PopulationError),\n    #[error(transparent)]\n    HouseholdLifecycle(#[from] HouseholdLifecycleError),""",
)

replace(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    """    founder_initialization::FounderGenealogyStatus,\n    landscape::LandscapeBundle,""",
    """    founder_initialization::FounderGenealogyStatus,\n    household_lifecycle::{\n        HouseholdLifecycleError, apply_household_lifecycle_at_annual_boundary,\n        validate_household_lifecycle_config,\n    },\n    landscape::LandscapeBundle,""",
)
replace(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    """            self.temporary_mobility\n                .reconcile_after_population_change(&self.population);\n            self.record_metric_snapshot();\n            match outcome {""",
    """            self.temporary_mobility\n                .reconcile_after_population_change(&self.population);\n            if let Some(household_lifecycle) = self.config.household_lifecycle.clone() {\n                apply_household_lifecycle_at_annual_boundary(\n                    &mut self.population,\n                    &mut self.temporary_mobility,\n                    &household_lifecycle,\n                    self.time.days(),\n                )?;\n            }\n            self.record_metric_snapshot();\n            match outcome {""",
)
replace(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    """    validate_demography_config(&config.demography)?;\n    validate_resource_config(&config.resources)?;""",
    """    validate_demography_config(&config.demography)?;\n    if let Some(household_lifecycle) = &config.household_lifecycle {\n        validate_household_lifecycle_config(household_lifecycle)?;\n    }\n    validate_resource_config(&config.resources)?;""",
)
replace(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    """    #[error(transparent)]\n    Population(#[from] PopulationError),""",
    """    #[error(transparent)]\n    Population(#[from] PopulationError),\n    #[error(transparent)]\n    HouseholdLifecycle(#[from] HouseholdLifecycleError),""",
)

# ---------------------------------------------------------------------------
# Derived terminal/checkpoint household observability. Existing M3/M4/M9
# reports already cover scarcity, migration and temporary aggregation effects;
# this fills the topology-size/generational gap without altering authority.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/src/household_observability.rs",
    r'''
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::{
        config::{ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID},
        household_lifecycle::household_lifecycle_model_id,
        ids::{HouseholdId, PersonId},
        population::Population,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdSizeBin {
        pub living_members: u32,
        pub household_count: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdGenerationSpanBin {
        /// Number of genealogical generations represented among living members of one household.
        pub generations: u32,
        pub household_count: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdObservabilityReport {
        pub schema_version: u32,
        pub day: u64,
        pub lifecycle_model_id: String,
        pub total_household_records: u64,
        pub active_households: u64,
        pub extinct_households: u64,
        /// Under the historical fixed-founder baseline every household record was created at day
        /// zero, so its age is exactly the run day. Dynamic lifecycle variants deliberately return
        /// null because creation-day history is not persisted by this minimal sensitivity model.
        pub uniform_founder_household_age_days: Option<u64>,
        pub largest_living_household_size: u32,
        pub living_household_size_distribution: Vec<HouseholdSizeBin>,
        pub maximum_living_generation_span: u32,
        pub multigenerational_households: u64,
        pub living_household_generation_span_distribution: Vec<HouseholdGenerationSpanBin>,
    }

    impl HouseholdObservabilityReport {
        pub const CURRENT_SCHEMA_VERSION: u32 = 1;
    }

    pub fn derive_household_observability(
        population: &Population,
        experiment: &ExperimentConfig,
        day: u64,
    ) -> Result<HouseholdObservabilityReport, HouseholdObservabilityError> {
        let household_count = population.household_count();
        let mut living_sizes = vec![0_u32; household_count];
        let mut minimum_generation = vec![u32::MAX; household_count];
        let mut maximum_generation = vec![0_u32; household_count];
        let mut has_living_member = vec![false; household_count];
        let mut memo = vec![None; population.person_count()];
        let mut visiting = vec![false; population.person_count()];

        for person_index in 0..population.person_count() {
            let person = population
                .person(PersonId::new(
                    u64::try_from(person_index)
                        .map_err(|_| HouseholdObservabilityError::PersonIdOverflow)?
                        .checked_add(1)
                        .ok_or(HouseholdObservabilityError::PersonIdOverflow)?,
                ))
                .ok_or(HouseholdObservabilityError::MissingPerson { person_index })?;
            if !person.is_alive() {
                continue;
            }
            let household_index = usize::try_from(
                person
                    .household
                    .0
                    .checked_sub(1)
                    .ok_or(HouseholdObservabilityError::InvalidHousehold {
                        household: person.household,
                    })?,
            )
            .map_err(|_| HouseholdObservabilityError::InvalidHousehold {
                household: person.household,
            })?;
            if household_index >= household_count {
                return Err(HouseholdObservabilityError::InvalidHousehold {
                    household: person.household,
                });
            }
            let generation = generation_depth(
                population,
                person.id,
                &mut memo,
                &mut visiting,
            )?;
            living_sizes[household_index] = living_sizes[household_index].saturating_add(1);
            has_living_member[household_index] = true;
            minimum_generation[household_index] = minimum_generation[household_index].min(generation);
            maximum_generation[household_index] = maximum_generation[household_index].max(generation);
        }

        let mut size_bins = BTreeMap::<u32, u64>::new();
        let mut span_bins = BTreeMap::<u32, u64>::new();
        let mut active_households = 0_u64;
        let mut largest = 0_u32;
        let mut max_span = 0_u32;
        let mut multigenerational = 0_u64;
        for index in 0..household_count {
            if !has_living_member[index] {
                continue;
            }
            active_households = active_households.saturating_add(1);
            let size = living_sizes[index];
            largest = largest.max(size);
            *size_bins.entry(size).or_default() += 1;
            let span = maximum_generation[index]
                .saturating_sub(minimum_generation[index])
                .saturating_add(1);
            max_span = max_span.max(span);
            if span >= 2 {
                multigenerational = multigenerational.saturating_add(1);
            }
            *span_bins.entry(span).or_default() += 1;
        }

        let total_household_records = u64::try_from(household_count)
            .map_err(|_| HouseholdObservabilityError::HouseholdCountOverflow)?;
        Ok(HouseholdObservabilityReport {
            schema_version: HouseholdObservabilityReport::CURRENT_SCHEMA_VERSION,
            day,
            lifecycle_model_id: household_lifecycle_model_id(
                experiment.household_lifecycle.as_ref(),
            )
            .to_owned(),
            total_household_records,
            active_households,
            extinct_households: total_household_records.saturating_sub(active_households),
            uniform_founder_household_age_days: (experiment.household_lifecycle.is_none()
                && household_lifecycle_model_id(None) == FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID)
                .then_some(day),
            largest_living_household_size: largest,
            living_household_size_distribution: size_bins
                .into_iter()
                .map(|(living_members, household_count)| HouseholdSizeBin {
                    living_members,
                    household_count,
                })
                .collect(),
            maximum_living_generation_span: max_span,
            multigenerational_households: multigenerational,
            living_household_generation_span_distribution: span_bins
                .into_iter()
                .map(|(generations, household_count)| HouseholdGenerationSpanBin {
                    generations,
                    household_count,
                })
                .collect(),
        })
    }

    fn generation_depth(
        population: &Population,
        person: PersonId,
        memo: &mut [Option<u32>],
        visiting: &mut [bool],
    ) -> Result<u32, HouseholdObservabilityError> {
        let index = usize::try_from(
            person
                .0
                .checked_sub(1)
                .ok_or(HouseholdObservabilityError::InvalidPerson { person })?,
        )
        .map_err(|_| HouseholdObservabilityError::InvalidPerson { person })?;
        if index >= memo.len() {
            return Err(HouseholdObservabilityError::InvalidPerson { person });
        }
        if let Some(value) = memo[index] {
            return Ok(value);
        }
        if visiting[index] {
            return Err(HouseholdObservabilityError::GenealogyCycle { person });
        }
        visiting[index] = true;
        let snapshot = population
            .person(person)
            .ok_or(HouseholdObservabilityError::InvalidPerson { person })?;
        let mut has_parent = false;
        let mut parent_depth = 0_u32;
        for parent in [snapshot.female_parent, snapshot.male_parent] {
            if parent == PersonId::INVALID {
                continue;
            }
            if population.person(parent).is_none() {
                return Err(HouseholdObservabilityError::InvalidParent { person, parent });
            }
            has_parent = true;
            parent_depth = parent_depth.max(generation_depth(population, parent, memo, visiting)?);
        }
        visiting[index] = false;
        let depth = if has_parent {
            parent_depth
                .checked_add(1)
                .ok_or(HouseholdObservabilityError::GenerationDepthOverflow)?
        } else {
            0
        };
        memo[index] = Some(depth);
        Ok(depth)
    }

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum HouseholdObservabilityError {
        #[error("person index {person_index} has no persistent record")]
        MissingPerson { person_index: usize },
        #[error("person identity space overflowed while deriving household observability")]
        PersonIdOverflow,
        #[error("household count does not fit u64")]
        HouseholdCountOverflow,
        #[error("person {person:?} references invalid household {household:?}")]
        InvalidHousehold {
            household: HouseholdId,
        },
        #[error("invalid person identity {person:?}")]
        InvalidPerson { person: PersonId },
        #[error("person {person:?} references invalid parent {parent:?}")]
        InvalidParent { person: PersonId, parent: PersonId },
        #[error("genealogy cycle encountered while deriving generation depth at {person:?}")]
        GenealogyCycle { person: PersonId },
        #[error("genealogical generation depth overflowed u32")]
        GenerationDepthOverflow,
    }
    ''',
)

# Fix a deliberately explicit error field shape after writing the module.
replace(
    "crates/anthrosim-core/src/household_observability.rs",
    """        InvalidHousehold {\n            household: HouseholdId,\n        },""",
    """        InvalidHousehold { household: HouseholdId },""",
)

# ---------------------------------------------------------------------------
# Public exports.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/lib.rs",
    """pub mod founder_initialization;\npub mod ids;""",
    """pub mod founder_initialization;\npub mod household_lifecycle;\npub mod household_observability;\npub mod ids;""",
)
replace(
    "crates/anthrosim-core/src/lib.rs",
    """    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,\n    PopulationConfig, PopulationInitialization, ResourceConfig, WorldConfig,\n};""",
    """    AgeProbabilityBand, DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, DemographyConfig,\n    ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID, HouseholdLifecycleConfig,\n    MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization, ResourceConfig,\n    WorldConfig,\n};""",
)
replace(
    "crates/anthrosim-core/src/lib.rs",
    """pub use invariants::{""",
    """pub use household_lifecycle::{\n    HouseholdLifecycleError, HouseholdLifecycleOutcome, household_lifecycle_model_id,\n    validate_household_lifecycle_config,\n};\npub use household_observability::{\n    HouseholdGenerationSpanBin, HouseholdObservabilityError, HouseholdObservabilityReport,\n    HouseholdSizeBin, derive_household_observability,\n};\npub use invariants::{""",
)

# ---------------------------------------------------------------------------
# CLI for derived checkpoint/run-bundle observability.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-cli/src/bin/anthrosim-household-observability.rs",
    r'''
    use std::{path::PathBuf, process::ExitCode};

    #[path = "../artifact_fs.rs"]
    mod artifact_fs;

    use anthrosim_core::{
        HouseholdObservabilityReport, SimulationCheckpoint, derive_household_observability,
    };
    use clap::Parser;

    #[derive(Debug, Parser)]
    #[command(
        name = "anthrosim-household-observability",
        version,
        about = "Derive or verify household topology observability for a run bundle"
    )]
    struct Cli {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long, conflicts_with = "output")]
        check: Option<PathBuf>,
    }

    fn main() -> ExitCode {
        match run(Cli::parse()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("anthrosim-household-observability: {error}");
                ExitCode::FAILURE
            }
        }
    }

    fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
        let checkpoint: SimulationCheckpoint = read_json(&cli.run_dir.join("checkpoint.json"))?;
        let report = derive_household_observability(
            &checkpoint.population,
            &checkpoint.experiment,
            checkpoint.time.days(),
        )?;

        if let Some(path) = cli.check {
            let expected: HouseholdObservabilityReport = read_json(&path)?;
            if expected != report {
                return Err(format!(
                    "derived household observability does not match {}",
                    path.display()
                )
                .into());
            }
            println!("verified {}", path.display());
            return Ok(());
        }

        let output = cli
            .output
            .unwrap_or_else(|| cli.run_dir.join("household-observability.json"));
        write_json(&output, &report)?;
        println!("wrote {}", output.display());
        Ok(())
    }

    fn read_json<T: serde::de::DeserializeOwned>(
        path: &std::path::Path,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let content = artifact_fs::read_to_string(path, "household observability source artifact")?;
        Ok(serde_json::from_str(&content)?)
    }

    fn write_json<T: serde::Serialize + ?Sized>(
        path: &std::path::Path,
        value: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(value)?;
        artifact_fs::atomic_write(
            path,
            format!("{json}\n").as_bytes(),
            "household observability output",
        )?;
        Ok(())
    }
    ''',
)

# ---------------------------------------------------------------------------
# Focused integration tests: cap/partition, deterministic replay/resume and
# explicit M9 participation effect after household creation.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/tests/household_lifecycle_sensitivity.rs",
    r'''
    use anthrosim_core::{
        DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
        HouseholdLifecycleConfig, MigrationConfig, Population, PopulationConfig, ResourceConfig,
        Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming, World, WorldConfig, derive_household_observability,
        ids::{CellId, HouseholdId}, rng::RngFactory,
    };

    fn no_event_demography() -> DemographyConfig {
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }
        config
    }

    fn no_pressure_resources() -> ResourceConfig {
        let mut config = ResourceConfig::synthetic_validation_v1();
        config.annual_need_units_per_person = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        config
    }

    fn base_config(seed: u64, duration_years: u64) -> ExperimentConfig {
        ExperimentConfig::new(seed, duration_years)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(12).with_target_household_size(12))
            .with_demography(no_event_demography())
            .with_resources(no_pressure_resources())
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
    }

    #[test]
    fn deterministic_size_fission_balances_and_caps_living_households() {
        let config = base_config(20701, 1).with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_size_fission_v1(5),
        );
        let run = Simulation::new(config).unwrap().run_recorded().unwrap();
        let report = derive_household_observability(
            &run.checkpoint.population,
            &run.checkpoint.experiment,
            run.checkpoint.time.days(),
        )
        .unwrap();
        assert_eq!(report.active_households, 3);
        assert_eq!(report.largest_living_household_size, 4);
        assert_eq!(run.manifest.population.living_population, 12);
        assert_eq!(run.manifest.population.births_since_start, 0);
        assert_eq!(run.manifest.population.deaths_since_start, 0);
    }

    #[test]
    fn lifecycle_is_exactly_deterministic_and_checkpoint_resumable() {
        let config = base_config(20702, 3).with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_size_fission_v1(5),
        );
        let first = Simulation::new(config.clone()).unwrap().run_recorded().unwrap();
        let duplicate = Simulation::new(config.clone()).unwrap().run_recorded().unwrap();
        assert_eq!(first.checkpoint.state_digest64, duplicate.checkpoint.state_digest64);
        assert_eq!(first.checkpoint.population, duplicate.checkpoint.population);

        let checkpoint = Simulation::new(config).unwrap().checkpoint_at_year(1).unwrap();
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();
        assert_eq!(first.checkpoint.state_digest64, resumed.checkpoint.state_digest64);
        assert_eq!(first.checkpoint.population, resumed.checkpoint.population);
    }

    #[test]
    fn fissioned_households_become_independent_future_m9_participants() {
        let seed = 20703;
        let base = base_config(seed, 2);
        let factory = RngFactory::new(seed);
        let world = World::generate(base.world, factory).unwrap();
        let population = Population::initialize(base.population, &world, factory).unwrap();
        let residence = population.household_location(HouseholdId::new(1)).unwrap();
        let destination = (1..=world.cell_count() as u64)
            .map(CellId::new)
            .find(|&cell| cell != residence)
            .unwrap();
        let mobility = TemporaryMobilityConfig::new(
            FocalRegion::new(
                "issue-207-test-region",
                FocalRegionSource::Synthetic,
                vec![destination],
            )
            .unwrap(),
            TemporaryMobilitySchedule::new(
                "issue-207-two-year-schedule",
                TemporaryTriggerTiming::DepartureDay,
                vec![100, 465],
                3,
            )
            .unwrap(),
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .unwrap();

        let baseline = Simulation::new(base.clone().with_temporary_mobility(mobility.clone()))
            .unwrap()
            .run_recorded()
            .unwrap();
        let fission = Simulation::new(
            base.with_temporary_mobility(mobility)
                .with_household_lifecycle(
                    HouseholdLifecycleConfig::deterministic_size_fission_v1(5),
                ),
        )
        .unwrap()
        .run_recorded()
        .unwrap();

        let departures = |events: &anthrosim_core::EventLog| {
            events
                .events
                .iter()
                .filter(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
                .count()
        };
        assert_eq!(departures(&baseline.checkpoint.events), 2);
        assert_eq!(departures(&fission.checkpoint.events), 4);
        assert_eq!(fission.checkpoint.population.household_count(), 3);
    }
    ''',
)

# ---------------------------------------------------------------------------
# Paired structural sensitivity executable. The bootstrap workflow runs this
# after validation and commits the machine-readable first comparison.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/examples/household_lifecycle_sensitivity.rs",
    r'''
    use anthrosim_core::{
        DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
        HouseholdLifecycleConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation,
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming, WorldConfig, derive_household_observability,
        ids::CellId,
    };
    use serde::Serialize;

    const DURATION_YEARS: u64 = 40;
    const SEEDS: [u64; 8] = [20701, 20702, 20703, 20704, 20705, 20706, 20707, 20708];

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ArmAggregate {
        lifecycle_model_id: String,
        completed_runs: u64,
        population_extinct_runs: u64,
        terminal_living_population_total: u64,
        terminal_active_households_total: u64,
        terminal_largest_household_size_total: u64,
        terminal_multigenerational_households_total: u64,
        terminal_living_occupied_cells_total: u64,
        mean_living_condition_permille_sum: u64,
        mean_living_condition_defined_runs: u64,
        unmet_need_total: u64,
        migration_moves_total: u64,
        migration_people_moved_total: u64,
        temporary_departures_total: u64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Comparison {
        schema_version: u32,
        purpose: &'static str,
        scientific_status: &'static str,
        seeds: Vec<u64>,
        duration_years: u64,
        founder_population: u32,
        founder_target_household_size: u16,
        alternative_max_living_members: u16,
        baseline: ArmAggregate,
        deterministic_size_fission: ArmAggregate,
    }

    fn replacement_demography() -> DemographyConfig {
        serde_json::from_str(include_str!(
            "../../../research/demography-controls-v1/replacement-control.json"
        ))
        .unwrap()
    }

    fn config(seed: u64, fission: bool) -> ExperimentConfig {
        let trigger_days = (0..DURATION_YEARS)
            .map(|year| year * 365 + 180)
            .collect::<Vec<_>>();
        let region = FocalRegion::new(
            "issue-207-structural-sensitivity-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(1), CellId::new(2), CellId::new(3), CellId::new(4)],
        )
        .unwrap();
        let temporary_mobility = TemporaryMobilityConfig::new(
            region,
            TemporaryMobilitySchedule::new(
                "issue-207-annual-midyear",
                TemporaryTriggerTiming::DepartureDay,
                trigger_days,
                7,
            )
            .unwrap(),
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .unwrap();
        let mut resources = ResourceConfig::synthetic_validation_v1();
        resources.productivity_scale_permille = 1_200;
        let mut config = ExperimentConfig::new(seed, DURATION_YEARS)
            .with_world(WorldConfig::new(12, 12))
            .with_population(PopulationConfig::new(120).with_target_household_size(5))
            .with_demography(replacement_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1())
            .with_temporary_mobility(temporary_mobility);
        if fission {
            config = config.with_household_lifecycle(
                HouseholdLifecycleConfig::deterministic_size_fission_v1(8),
            );
        }
        config
    }

    fn aggregate(fission: bool) -> ArmAggregate {
        let mut aggregate = ArmAggregate {
            lifecycle_model_id: String::new(),
            completed_runs: 0,
            population_extinct_runs: 0,
            terminal_living_population_total: 0,
            terminal_active_households_total: 0,
            terminal_largest_household_size_total: 0,
            terminal_multigenerational_households_total: 0,
            terminal_living_occupied_cells_total: 0,
            mean_living_condition_permille_sum: 0,
            mean_living_condition_defined_runs: 0,
            unmet_need_total: 0,
            migration_moves_total: 0,
            migration_people_moved_total: 0,
            temporary_departures_total: 0,
        };
        for seed in SEEDS {
            let run = Simulation::new(config(seed, fission))
                .unwrap()
                .run_recorded()
                .unwrap();
            let household = derive_household_observability(
                &run.checkpoint.population,
                &run.checkpoint.experiment,
                run.checkpoint.time.days(),
            )
            .unwrap();
            aggregate.lifecycle_model_id = household.lifecycle_model_id.clone();
            aggregate.completed_runs += u64::from(
                run.checkpoint.completed_years == run.checkpoint.experiment.duration_years,
            );
            aggregate.population_extinct_runs +=
                u64::from(run.checkpoint.population.living_count() == 0);
            aggregate.terminal_living_population_total += run.checkpoint.population.living_count();
            aggregate.terminal_active_households_total += household.active_households;
            aggregate.terminal_largest_household_size_total +=
                u64::from(household.largest_living_household_size);
            aggregate.terminal_multigenerational_households_total +=
                household.multigenerational_households;
            aggregate.terminal_living_occupied_cells_total +=
                run.checkpoint.population.summary().living_occupied_cell_count;
            if let Some(condition) = run.checkpoint.population.mean_living_condition_permille() {
                aggregate.mean_living_condition_permille_sum += u64::from(condition);
                aggregate.mean_living_condition_defined_runs += 1;
            }
            let resources = run.checkpoint.resources.summary(&run.checkpoint.population);
            aggregate.unmet_need_total += resources.unmet_need;
            let migration = anthrosim_core::MigrationSystem::from_checkpoint_state(
                &run.checkpoint.population,
                &anthrosim_core::World::generate(
                    run.checkpoint.experiment.world,
                    anthrosim_core::rng::RngFactory::new(run.checkpoint.experiment.seed),
                )
                .unwrap(),
                &run.checkpoint.experiment.migration,
                run.checkpoint.migration.clone(),
            )
            .unwrap()
            .summary();
            aggregate.migration_moves_total += migration.moves_completed;
            aggregate.migration_people_moved_total += migration.people_moved;
            aggregate.temporary_departures_total += run
                .checkpoint
                .events
                .events
                .iter()
                .filter(|record| {
                    matches!(record.event, EventKind::TemporaryJourneyDeparted { .. })
                })
                .count() as u64;
        }
        aggregate
    }

    fn main() {
        let comparison = Comparison {
            schema_version: 1,
            purpose: "TRACE structural sensitivity to founder-defined versus deterministic size-fission household lifecycles",
            scientific_status: "synthetic structural sensitivity; not empirical household validation",
            seeds: SEEDS.to_vec(),
            duration_years: DURATION_YEARS,
            founder_population: 120,
            founder_target_household_size: 5,
            alternative_max_living_members: 8,
            baseline: aggregate(false),
            deterministic_size_fission: aggregate(true),
        };
        println!("{}", serde_json::to_string_pretty(&comparison).unwrap());
    }
    ''',
)

# ---------------------------------------------------------------------------
# Documentation and result renderer.
# ---------------------------------------------------------------------------
write(
    "docs/research/household-lifecycle-structural-sensitivity-v1.md",
    r'''
    # Household lifecycle structural sensitivity v1

    ## Status

    This is a **synthetic structural-sensitivity contract**, not an ethnographic or archaeological
    model of household formation. It addresses TRACE audit issue #207 by making the historical
    fixed-founder household lifecycle testable against one deliberately neutral alternative.

    ## Baseline

    `fixed_founder_v1` is the historical AnthroSim rule. Founder household IDs persist for the run;
    births join the female parent's current household; M3 shares resources at household level; M4
    permanently relocates the living household; M9 temporary mobility treats the household as one
    participant. No fission, dissolution, adult departure or transfer occurs.

    This must not be interpreted as evidence that real households were permanent descent groups.
    The persistence is a null-model structural assumption.

    ## Alternative

    `deterministic_size_fission_v1` is enabled only through the optional versioned
    `householdLifecycle` experiment field. At each completed annual boundary, after M2 fertility:

    - only households physically at residence are eligible, avoiding ambiguous division of an
      active M9 journey;
    - a household above `maxLivingMembers` is divided into the minimum number of groups required to
      satisfy the ceiling;
    - group sizes are balanced as evenly as possible;
    - living members are partitioned in stable `PersonId` order;
    - all daughter households begin at the same persistent residence;
    - person identity, genealogy, condition and residence are unchanged;
    - past M9 triggers are marked processed for newly created households, while future triggers
      treat them as independent participants;
    - M4 non-persistent household scratch arrays expand deterministically before the next decision.

    Stable-ID partitioning is intentionally simple. It is not a claim about marriage, inheritance,
    post-marital residence, age at departure or culturally specific household composition. Its role
    is to ask whether scientific conclusions survive removal of permanent founder-group topology.

    ## Observability

    `anthrosim-household-observability` derives a versioned checkpoint report containing:

    - total and active household records;
    - the living household-size distribution and maximum;
    - living genealogical-generation-span distribution and multi-generational household count;
    - exact uniform household age for the fixed-founder baseline.

    Existing authoritative/derived reports continue to provide the other #207 comparison targets:
    M3 unmet need and condition, M4 move frequency and people moved, M9 journey/aggregation events,
    terminal population and spatial occupancy. No explorer-only state becomes authoritative.

    ## First paired comparison

    The repository example `household_lifecycle_sensitivity` runs eight paired seeds for 40 years
    with the same founder population, replacement-control demography, M3/M4 assumptions, annual M9
    schedule and synthetic world dimensions in both arms. The only structural treatment is the
    household lifecycle. The machine-readable first result is preserved in
    `research/household-lifecycle-sensitivity-v1/reference-result.json`; the generated interpretation
    is in `docs/research/household-lifecycle-structural-sensitivity-result.md`.

    The comparison is diagnostic only. A material effect means household lifecycle remains a
    scientific model choice that must be propagated in claims using household sharing, permanent
    migration or temporary aggregation. Lack of an effect for this one alternative would establish
    robustness only to this declared contrast, not validate either lifecycle historically.

    ## Compatibility / semantics review

    The historical `None` lifecycle path executes the pre-#207 rule and preserves its serialized
    omission. The new field is included in full experiment and continuation identity when enabled.
    No existing parameter is reinterpreted, so the repository model-semantics identity is not
    advanced solely for this opt-in structural treatment. Exact Git provenance and full experiment
    identity distinguish runs, and continuation integrity binds the configured lifecycle.
    ''',
)

write(
    "scripts/render-household-lifecycle-result.py",
    r'''
    import json
    import sys
    from pathlib import Path

    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    data = json.loads(source.read_text(encoding="utf-8"))
    b = data["baseline"]
    f = data["deterministicSizeFission"]
    n = len(data["seeds"])

    def mean(total):
        return total / n

    lines = [
        "# Household lifecycle structural sensitivity — first result",
        "",
        "**Scientific status:** synthetic structural sensitivity; not empirical household validation.",
        "",
        f"Eight paired seeds were run for {data['durationYears']} years. The arms differ only in household lifecycle: `fixed_founder_v1` versus `deterministic_size_fission_v1` with a maximum of {data['alternativeMaxLivingMembers']} living members per eligible household after an annual boundary.",
        "",
        "| Observable | Fixed founder | Size fission |",
        "| --- | ---: | ---: |",
        f"| Completed runs | {b['completedRuns']}/{n} | {f['completedRuns']}/{n} |",
        f"| Extinct runs | {b['populationExtinctRuns']}/{n} | {f['populationExtinctRuns']}/{n} |",
        f"| Mean terminal living population | {mean(b['terminalLivingPopulationTotal']):.2f} | {mean(f['terminalLivingPopulationTotal']):.2f} |",
        f"| Mean terminal active households | {mean(b['terminalActiveHouseholdsTotal']):.2f} | {mean(f['terminalActiveHouseholdsTotal']):.2f} |",
        f"| Mean terminal largest household | {mean(b['terminalLargestHouseholdSizeTotal']):.2f} | {mean(f['terminalLargestHouseholdSizeTotal']):.2f} |",
        f"| Mean terminal multi-generational households | {mean(b['terminalMultigenerationalHouseholdsTotal']):.2f} | {mean(f['terminalMultigenerationalHouseholdsTotal']):.2f} |",
        f"| Mean terminal occupied residence cells | {mean(b['terminalLivingOccupiedCellsTotal']):.2f} | {mean(f['terminalLivingOccupiedCellsTotal']):.2f} |",
        f"| Total unmet resource need | {b['unmetNeedTotal']} | {f['unmetNeedTotal']} |",
        f"| Total M4 moves | {b['migrationMovesTotal']} | {f['migrationMovesTotal']} |",
        f"| Mean people per M4 move | {(b['migrationPeopleMovedTotal'] / b['migrationMovesTotal']) if b['migrationMovesTotal'] else 0:.3f} | {(f['migrationPeopleMovedTotal'] / f['migrationMovesTotal']) if f['migrationMovesTotal'] else 0:.3f} |",
        f"| Total M9 departures | {b['temporaryDeparturesTotal']} | {f['temporaryDeparturesTotal']} |",
        "",
        "## Interpretation",
        "",
    ]
    materially_different = (
        b["terminalActiveHouseholdsTotal"] != f["terminalActiveHouseholdsTotal"]
        or b["migrationMovesTotal"] != f["migrationMovesTotal"]
        or b["temporaryDeparturesTotal"] != f["temporaryDeparturesTotal"]
        or b["unmetNeedTotal"] != f["unmetNeedTotal"]
    )
    if materially_different:
        lines.append(
            "The declared lifecycle contrast is **material for at least one predeclared household/resource/mobility observable** in this synthetic ensemble. Household lifecycle must therefore remain an explicit structural uncertainty dimension for claims that depend on household sharing, M4 permanent migration, or M9 participation. This does not establish which lifecycle is historically correct."
        )
    else:
        lines.append(
            "The declared lifecycle contrast did not alter the predeclared aggregate observables in this synthetic ensemble. That is robustness evidence only for this exact contrast and does not validate either lifecycle historically."
        )
    lines.extend([
        "",
        "The fixed-founder arm's household ages are exactly the run duration by construction. Its size and generation-span distributions can be regenerated from each checkpoint with `anthrosim-household-observability`; the alternative removes that permanent founder-topology assumption and creates younger household records at annual fission boundaries.",
        "",
        "The machine-readable aggregate used for this page is `research/household-lifecycle-sensitivity-v1/reference-result.json`.",
    ])
    target.write_text("\n".join(lines) + "\n", encoding="utf-8")
    ''',
)

# Update the durable scientific model wording without claiming empirical realism.
replace(
    "docs/scientific-model.md",
    """A household is a persistent resource-sharing and mobility unit, not a tribe, clan, lineage, settlement, marriage or universal nuclear-family structure. Parentage and household membership are separate relationships.""",
    """A household is a resource-sharing and mobility unit, not a tribe, clan, lineage, settlement, marriage or universal nuclear-family structure. Parentage and household membership are separate relationships. The historical baseline keeps founder-defined household IDs persistent; post-audit structural-sensitivity experiments may instead enable the explicit `deterministic_size_fission_v1` lifecycle, which partitions oversized at-residence households at annual boundaries. That alternative is a neutral stress test rather than an empirical household-formation claim.""",
)

print("issue 207 source transformation complete")
