use serde::{Deserialize, Serialize};

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
    pub demography: DemographyConfig,
}

impl ExperimentConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;

    #[must_use]
    pub fn new(seed: u64, duration_years: u64) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            seed,
            duration_years,
            world: WorldConfig::default_config(),
            population: PopulationConfig::default_config(),
            demography: DemographyConfig::synthetic_validation_v1(),
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
    pub fn with_demography(mut self, demography: DemographyConfig) -> Self {
        self.demography = demography;
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
/// The first mode is deliberately named synthetic: it is an engine-validation
/// distribution, not a reconstruction of a prehistoric population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopulationInitialization {
    SyntheticValidationV1,
}

/// Configuration for persistent people and synthetic co-resident households.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationConfig {
    pub schema_version: u32,
    pub initial_population: u32,
    pub target_household_size: u16,
    pub initialization: PopulationInitialization,
    /// Upper bound for the synthetic uniform founder age distribution.
    /// This is an explicit validation parameter, not an empirical lifespan.
    pub synthetic_max_age_years: u16,
    /// Male share of synthetic founders, expressed in permille.
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
