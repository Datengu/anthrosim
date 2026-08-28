use serde::{Deserialize, Serialize};

use crate::{
    evidence::EvidenceCatalog, founder_initialization::FounderPopulationDefinition,
    temporary_mobility::TemporaryMobilityConfig,
};

pub const PROBABILITY_PER_MILLION: u32 = 1_000_000;

/// Versioned input that fully defines an AnthroSim experiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub duration_years: u64,
    pub world: WorldConfig,
    pub population: PopulationConfig,
    /// Explicit founder state required by `declared_founder_state_v1` initialization. Synthetic
    /// experiments omit this field, preserving their established serialized experiment identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founder_population: Option<FounderPopulationDefinition>,
    /// Optional structural household-lifecycle treatment. `None` preserves the historical
    /// founder-defined household lifecycle exactly; configured alternatives are explicit
    /// scientific treatments included in ordinary experiment identity and provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub household_lifecycle: Option<HouseholdLifecycleConfig>,
    pub demography: DemographyConfig,
    pub resources: ResourceConfig,
    pub migration: MigrationConfig,
    /// Optional world-independent M9 definition. The authoritative resolved travel table is
    /// derived from each run's actual world rather than copied across seeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporary_mobility: Option<TemporaryMobilityConfig>,
    /// Optional machine-readable evidence catalogue. Empty synthetic-validation
    /// experiments omit this field entirely, preserving the existing v0.1
    /// serialized identity. Evidence-grounded experiments include it in their
    /// ordinary serialized configuration and therefore in experiment identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceCatalog>,
}

impl ExperimentConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 10;

    #[must_use]
    pub fn new(seed: u64, duration_years: u64) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            seed,
            duration_years,
            world: WorldConfig::default_config(),
            population: PopulationConfig::default_config(),
            founder_population: None,
            household_lifecycle: None,
            demography: DemographyConfig::synthetic_validation_v1(),
            resources: ResourceConfig::synthetic_validation_v1(),
            migration: MigrationConfig::synthetic_validation_v1(),
            temporary_mobility: None,
            evidence: None,
        }
    }

    #[must_use]
    pub const fn with_world(mut self, world: WorldConfig) -> Self {
        self.world = world;
        self
    }

    #[must_use]
    pub const fn with_population(mut self, population: PopulationConfig) -> Self {
        self.population = population;
        self
    }

    #[must_use]
    pub fn with_founder_population(
        mut self,
        founder_population: FounderPopulationDefinition,
    ) -> Self {
        self.population.initialization = PopulationInitialization::DeclaredFounderStateV1;
        self.founder_population = Some(founder_population);
        self
    }

    #[must_use]
    pub fn with_household_lifecycle(
        mut self,
        household_lifecycle: HouseholdLifecycleConfig,
    ) -> Self {
        self.household_lifecycle = Some(household_lifecycle);
        self
    }

    #[must_use]
    pub fn with_demography(mut self, demography: DemographyConfig) -> Self {
        self.demography = demography;
        self
    }

    #[must_use]
    pub fn with_resources(mut self, resources: ResourceConfig) -> Self {
        self.resources = resources;
        self
    }

    #[must_use]
    pub fn with_migration(mut self, migration: MigrationConfig) -> Self {
        self.migration = migration;
        self
    }

    #[must_use]
    pub fn with_temporary_mobility(mut self, temporary_mobility: TemporaryMobilityConfig) -> Self {
        self.temporary_mobility = Some(temporary_mobility);
        self
    }

    #[must_use]
    pub fn with_evidence(mut self, evidence: EvidenceCatalog) -> Self {
        self.evidence = Some(evidence);
        self
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self::new(1, 1_000)
    }
}

/// Configuration for the synthetic v0.1 spatial environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldConfig {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
}

impl WorldConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn default_config() -> Self {
        Self::new(128, 128)
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Initialization rule used before the dynamic M2 demographic schedules run.
///
/// `SyntheticValidationV1` remains the frozen engineering/null-model preset and serializes to the
/// same `synthetic_validation_v1` string used by earlier experiment identities. The declared mode
/// requires the complete founder state in `ExperimentConfig.founder_population` and never silently
/// reuses synthetic ages, sexes, reproductive history, or kin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopulationInitialization {
    SyntheticValidationV1,
    DeclaredFounderStateV1,
}

/// Configuration for persistent people and co-resident households.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationConfig {
    pub schema_version: u32,
    pub initial_population: u32,
    /// Synthetic-validation target household size. Declared founder state supplies exact household
    /// membership and does not reinterpret this value as a research assumption.
    pub target_household_size: u16,
    pub initialization: PopulationInitialization,
    /// Upper bound for the synthetic uniform founder age distribution.
    /// This is an explicit validation parameter, not an empirical lifespan, and is ignored by
    /// declared founder-state initialization.
    pub synthetic_max_age_years: u16,
    /// Male share of synthetic founders, expressed in permille. Ignored by declared founder-state
    /// initialization, which supplies reproductive sex person by person.
    pub synthetic_male_permille: u16,
    /// Operational safety ceiling for persistent person records. Reaching this
    /// stops a run; it is not a population-regulation mechanism.
    pub max_person_records: u64,
}

impl PopulationConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub const fn new(initial_population: u32) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initial_population,
            target_household_size: 5,
            initialization: PopulationInitialization::SyntheticValidationV1,
            synthetic_max_age_years: 60,
            synthetic_male_permille: 500,
            max_person_records: 1_000_000,
        }
    }

    #[must_use]
    pub const fn with_target_household_size(mut self, target_household_size: u16) -> Self {
        self.target_household_size = target_household_size;
        self
    }

    #[must_use]
    pub const fn with_initialization(mut self, initialization: PopulationInitialization) -> Self {
        self.initialization = initialization;
        self
    }

    #[must_use]
    pub const fn with_max_person_records(mut self, max_person_records: u64) -> Self {
        self.max_person_records = max_person_records;
        self
    }

    #[must_use]
    pub const fn default_config() -> Self {
        Self::new(10_000)
    }
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Provenance status for a model parameter set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterProvenance {
    EmpiricalDirect,
    EmpiricalDerived,
    EvidenceInformed,
    SyntheticValidation,
    Unresolved,
}

/// Stable identity for the historical baseline in which founder-defined households persist
/// for the complete run except for extinction. It remains represented by an omitted optional
/// lifecycle field so pre-#207 synthetic experiment serialization is unchanged.
pub const FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID: &str = "fixed_founder_v1";

/// Stable identity for the deliberately neutral structural-sensitivity alternative introduced by
/// #207. It is a stress-test mechanism, not a calibrated household-formation model.
pub const DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str = "deterministic_size_fission_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdLifecycleConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    /// Maximum number of living members retained in one household after an annual lifecycle
    /// boundary. Oversized at-residence households are partitioned deterministically into the
    /// minimum number of balanced co-resident groups needed to satisfy this ceiling.
    pub max_living_members: u16,
}

impl HouseholdLifecycleConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn deterministic_size_fission_v1(max_living_members: u16) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            max_living_members,
        }
    }
}

/// Annual event probability for a half-open age interval.
///
/// Probabilities are integer parts-per-million so authoritative demographic
/// draws do not require floating-point state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgeProbabilityBand {
    pub start_age_years: u32,
    pub end_age_years_exclusive: u32,
    pub annual_probability_per_million: u32,
}

impl AgeProbabilityBand {
    #[must_use]
    pub const fn new(
        start_age_years: u32,
        end_age_years_exclusive: u32,
        annual_probability_per_million: u32,
    ) -> Self {
        Self {
            start_age_years,
            end_age_years_exclusive,
            annual_probability_per_million,
        }
    }
}

/// Replaceable schedule consumed by the M2 demographic engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographyConfig {
    pub schema_version: u32,
    pub schedule_id: String,
    pub provenance: ParameterProvenance,
    pub mortality_bands: Vec<AgeProbabilityBand>,
    pub fertility_bands: Vec<AgeProbabilityBand>,
    pub minimum_birth_spacing_days: u32,
    pub male_birth_permille: u16,
    pub male_parent_min_age_years: u32,
    pub male_parent_max_age_years_exclusive: u32,
}

impl DemographyConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Deliberately synthetic schedule used to exercise M2 mechanisms.
    ///
    /// Its qualitative shape is evidence-informed, but the complete schedule
    /// is not calibrated to any real population and carries no prehistoric
    /// reconstruction claim.
    #[must_use]
    pub fn synthetic_validation_v1() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            schedule_id: "synthetic_validation_v1".to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            mortality_bands: vec![
                AgeProbabilityBand::new(0, 1, 180_000),
                AgeProbabilityBand::new(1, 5, 50_000),
                AgeProbabilityBand::new(5, 15, 8_000),
                AgeProbabilityBand::new(15, 40, 10_000),
                AgeProbabilityBand::new(40, 55, 20_000),
                AgeProbabilityBand::new(55, 65, 50_000),
                AgeProbabilityBand::new(65, 75, 120_000),
                AgeProbabilityBand::new(75, u32::MAX, 300_000),
            ],
            fertility_bands: vec![
                AgeProbabilityBand::new(0, 18, 0),
                AgeProbabilityBand::new(18, 25, 220_000),
                AgeProbabilityBand::new(25, 35, 250_000),
                AgeProbabilityBand::new(35, 40, 180_000),
                AgeProbabilityBand::new(40, 45, 80_000),
                AgeProbabilityBand::new(45, u32::MAX, 0),
            ],
            minimum_birth_spacing_days: 1_278,
            male_birth_permille: 512,
            male_parent_min_age_years: 18,
            male_parent_max_age_years_exclusive: 70,
        }
    }
}

impl Default for DemographyConfig {
    fn default() -> Self {
        Self::synthetic_validation_v1()
    }
}

/// Transparent M3 energetic/resource assumptions plus the shared condition response used by M3.
///
/// All resource quantities are abstract integer units. This configuration is
/// an engine-validation mechanism, not a calibrated caloric or palaeoecological
/// reconstruction. `periods_per_year` controls M3 settlement/integration only;
/// condition-response and condition-mediated mortality coefficients below are interpreted
/// against a fixed quarter-year reference interval and rescaled by elapsed days.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    pub periods_per_year: u16,
    pub annual_need_units_per_person: u32,
    pub annual_regeneration_units_per_productivity: u32,
    /// Day-zero stock units per cell productivity unit before the ordinary productivity scale.
    /// This is an explicit initial-condition assumption, independent of storage capacity. Capacity
    /// may cap an impossible starting stock but increasing capacity does not create historical stock.
    pub initial_stock_units_per_productivity: u32,
    pub productivity_scale_permille: u16,
    /// Scales the generated cell seasonal amplitude, 0..=1000.
    /// 0 removes the seasonal swing; 1000 preserves the synthetic v0.1 baseline.
    pub seasonality_scale_permille: u16,
    pub cell_stock_capacity_years: u16,
    /// Legacy wire name retained for input compatibility. Under schema v5 this is the maximum
    /// condition recovery over one reference quarter-year (365/4 days), not over an arbitrary
    /// configured resource period. M3 rescales it by actual elapsed interval duration.
    pub condition_recovery_per_period: u16,
    /// Legacy wire name retained for input compatibility. Under schema v5 this is the maximum
    /// condition loss over one reference quarter-year and is rescaled by elapsed duration.
    pub max_condition_loss_per_period: u16,
    /// Historical Rust field name retained to minimize execution-code churn. In v10 configuration
    /// this serializes only as `maxConditionMortalityProbabilityPerMillion`; no alias accepts the
    /// former scarcity-specific wire name. The shared condition may reflect M3 resource balance,
    /// M4 permanent-travel cost, and any future explicitly documented condition pathway.
    #[serde(rename = "maxConditionMortalityProbabilityPerMillion")]
    pub max_scarcity_mortality_probability_per_million: u32,
}

impl ResourceConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 5;

    #[must_use]
    pub fn synthetic_validation_v1() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: "synthetic_validation_v1".to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            periods_per_year: 4,
            annual_need_units_per_person: 100,
            annual_regeneration_units_per_productivity: 1,
            initial_stock_units_per_productivity: 10,
            productivity_scale_permille: 1_000,
            seasonality_scale_permille: 1_000,
            cell_stock_capacity_years: 10,
            condition_recovery_per_period: 25,
            max_condition_loss_per_period: 200,
            max_scarcity_mortality_probability_per_million: 200_000,
        }
    }

    #[must_use]
    pub const fn with_initial_stock_units_per_productivity(mut self, value: u32) -> Self {
        self.initial_stock_units_per_productivity = value;
        self
    }

    #[must_use]
    pub const fn with_productivity_scale_permille(mut self, value: u16) -> Self {
        self.productivity_scale_permille = value;
        self
    }

    #[must_use]
    pub const fn with_seasonality_scale_permille(mut self, value: u16) -> Self {
        self.seasonality_scale_permille = value;
        self
    }

    #[must_use]
    pub const fn with_annual_need_units_per_person(mut self, value: u32) -> Self {
        self.annual_need_units_per_person = value;
        self
    }

    #[must_use]
    pub const fn with_annual_regeneration_units_per_productivity(mut self, value: u32) -> Self {
        self.annual_regeneration_units_per_productivity = value;
        self
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self::synthetic_validation_v1()
    }
}

/// Transparent M4 migration assumptions.
///
/// Households have bounded local knowledge and compare a small Manhattan
/// neighbourhood using explicit synthetic utility factors. The default weights
/// are an engine-validation parameterization, not an empirical reconstruction
/// of hunter-gatherer mobility preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    pub enabled: bool,
    /// Independent M4 permanent-relocation decision clock. This is intentionally separate from
    /// M3 `periods_per_year`; the default retains four decision opportunities per model year.
    pub decision_periods_per_year: u16,
    pub candidate_radius_cells: u16,
    pub condition_pressure_threshold_permille: u16,
    pub resource_pressure_threshold_permille: u16,
    pub minimum_utility_improvement: u32,
    pub resource_weight: u16,
    pub water_security_weight: u16,
    pub kin_weight: u16,
    pub travel_cost_weight: u16,
    pub max_uncertainty_penalty_permille: u16,
    pub relocation_risk_base_penalty_permille: u16,
    pub relocation_risk_per_cell_permille: u16,
    pub travel_condition_cost_per_cell: u16,
    pub max_recorded_decision_traces: u32,
}

impl MigrationConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub fn synthetic_validation_v1() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: "synthetic_validation_v1".to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            enabled: true,
            decision_periods_per_year: 4,
            candidate_radius_cells: 3,
            condition_pressure_threshold_permille: 900,
            resource_pressure_threshold_permille: 850,
            minimum_utility_improvement: 150,
            resource_weight: 5,
            water_security_weight: 2,
            kin_weight: 1,
            travel_cost_weight: 2,
            max_uncertainty_penalty_permille: 100,
            relocation_risk_base_penalty_permille: 50,
            relocation_risk_per_cell_permille: 25,
            travel_condition_cost_per_cell: 10,
            max_recorded_decision_traces: 256,
        }
    }

    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn with_candidate_radius_cells(mut self, radius: u16) -> Self {
        self.candidate_radius_cells = radius;
        self
    }

    #[must_use]
    pub const fn with_decision_periods_per_year(mut self, periods: u16) -> Self {
        self.decision_periods_per_year = periods;
        self
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self::synthetic_validation_v1()
    }
}
