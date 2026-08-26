use serde::{Deserialize, Serialize};

use crate::{
    migration::MigrationSummary, population::PopulationSummary, resources::ResourceSummary,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricProvenance {
    Derived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationMetrics {
    pub living_population: u64,
    pub person_records: u64,
    pub births_since_start: u64,
    pub deaths_since_start: u64,
    pub living_occupied_cell_count: u64,
    pub mean_living_condition_permille: u16,
    pub living_below_half_condition: u64,
    pub digest64: u64,
}

impl From<&PopulationSummary> for PopulationMetrics {
    fn from(value: &PopulationSummary) -> Self {
        Self {
            living_population: value.living_population,
            person_records: value.person_records,
            births_since_start: value.births_since_start,
            deaths_since_start: value.deaths_since_start,
            living_occupied_cell_count: value.living_occupied_cell_count,
            mean_living_condition_permille: value.mean_living_condition_permille,
            living_below_half_condition: value.living_below_half_condition,
            digest64: value.digest64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetrics {
    pub periods_processed: u64,
    pub regenerated_food: u64,
    pub harvested_food: u64,
    pub unmet_need: u64,
    pub final_food_stock: u64,
    pub household_periods_with_unmet_need: u64,
    /// Deaths from the general condition-mediated hazard. This is not a resource-scarcity cause
    /// count: the shared condition state may include resource and non-resource upstream effects.
    pub condition_mortality_deaths: u64,
    pub digest64: u64,
}

impl From<&ResourceSummary> for ResourceMetrics {
    fn from(value: &ResourceSummary) -> Self {
        Self {
            periods_processed: value.periods_processed,
            regenerated_food: value.regenerated_food,
            harvested_food: value.harvested_food,
            unmet_need: value.unmet_need,
            final_food_stock: value.final_food_stock,
            household_periods_with_unmet_need: value.household_periods_with_unmet_need,
            condition_mortality_deaths: value.condition_mortality_deaths,
            digest64: value.digest64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationMetrics {
    pub decision_boundaries: u64,
    pub households_evaluated: u64,
    pub households_under_pressure: u64,
    pub moves_completed: u64,
    pub people_moved: u64,
    pub total_distance_cells: u64,
    pub occupied_cell_delta_from_migration: i64,
    pub digest64: u64,
}

impl From<&MigrationSummary> for MigrationMetrics {
    fn from(value: &MigrationSummary) -> Self {
        Self {
            decision_boundaries: value.decision_boundaries,
            households_evaluated: value.households_evaluated,
            households_under_pressure: value.households_under_pressure,
            moves_completed: value.moves_completed,
            people_moved: value.people_moved,
            total_distance_cells: value.total_distance_cells,
            occupied_cell_delta_from_migration: value.occupied_cell_delta_from_migration,
            digest64: value.digest64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricSnapshot {
    pub schema_version: u32,
    pub day: u64,
    pub provenance: MetricProvenance,
    pub population: PopulationMetrics,
    pub resources: ResourceMetrics,
    pub migration: MigrationMetrics,
    pub state_digest64: u64,
}

impl MetricSnapshot {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricSeries {
    pub schema_version: u32,
    pub cadence: String,
    pub snapshots: Vec<MetricSnapshot>,
}

impl MetricSeries {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub fn annual() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            cadence: "annual_boundary_plus_terminal".to_owned(),
            snapshots: Vec::new(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

impl Default for MetricSeries {
    fn default() -> Self {
        Self::annual()
    }
}
