use serde::{Deserialize, Serialize};

/// Versioned input that fully defines an AnthroSim experiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub duration_years: u64,
    pub world: WorldConfig,
    pub population: PopulationConfig,
}

impl ExperimentConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 3;

    #[must_use]
    pub const fn new(seed: u64, duration_years: u64) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            seed,
            duration_years,
            world: WorldConfig::default_config(),
            population: PopulationConfig::default_config(),
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
}

impl PopulationConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub const fn new(initial_population: u32) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initial_population,
            target_household_size: 5,
            initialization: PopulationInitialization::SyntheticValidationV1,
            synthetic_max_age_years: 60,
            synthetic_male_permille: 500,
        }
    }

    #[must_use]
    pub const fn with_target_household_size(mut self, target_household_size: u16) -> Self {
        self.target_household_size = target_household_size;
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
