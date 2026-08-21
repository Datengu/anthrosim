use serde::{Deserialize, Serialize};

/// Versioned input that fully defines the v0.1 skeleton experiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentConfig {
    pub schema_version: u32,
    pub seed: u64,
    pub duration_years: u64,
}

impl ExperimentConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub const fn new(seed: u64, duration_years: u64) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            seed,
            duration_years,
        }
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self::new(1, 1_000)
    }
}
