#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:120]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"patch anchor not unique in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def insert_before(path: str, marker: str, addition: str) -> None:
    replace_once(path, marker, addition + marker)


resources = "crates/anthrosim-core/src/resources.rs"
replace_once(resources, "    ids::HouseholdId,\n", "    ids::{CellId, HouseholdId},\n")

insert_before(resources, "/// Dynamic M3 resource state.\n", r'''#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCellPeriodObservation {
    pub cell: CellId,
    pub stock_before_regeneration: u64,
    pub regenerated: u64,
    pub stock_after_regeneration: u64,
    pub home_need: u64,
    pub visitor_need: u64,
    pub total_need: u64,
    pub supplied: u64,
    pub unmet: u64,
    pub stock_after_harvest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseholdSupplyFractionDistribution {
    pub households_with_need: u64,
    pub supplied_0_to_249_permille: u64,
    pub supplied_250_to_499_permille: u64,
    pub supplied_500_to_749_permille: u64,
    pub supplied_750_to_999_permille: u64,
    pub supplied_1000_permille: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionDistributionObservation {
    pub living_people: u64,
    pub mean_permille: Option<u16>,
    pub minimum_permille: Option<u16>,
    pub p10_permille: Option<u16>,
    pub p25_permille: Option<u16>,
    pub median_permille: Option<u16>,
    pub p75_permille: Option<u16>,
    pub p90_permille: Option<u16>,
    pub maximum_permille: Option<u16>,
    pub living_below_250_permille: u64,
    pub living_below_500_permille: u64,
    pub living_below_750_permille: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePeriodObservation {
    pub schema_version: u32,
    pub sequence: u64,
    pub period_index_in_year: u16,
    pub start_day: u64,
    pub end_day: u64,
    pub stock_before_regeneration: u64,
    pub regenerated: u64,
    pub stock_after_regeneration: u64,
    pub total_need: u64,
    pub supplied: u64,
    pub unmet: u64,
    pub stock_after_harvest: u64,
    pub households_with_unmet_need: u64,
    pub household_supply_fraction: HouseholdSupplyFractionDistribution,
    pub condition_after_resource_response: ConditionDistributionObservation,
    pub condition_after_mortality: ConditionDistributionObservation,
    pub cells: Vec<ResourceCellPeriodObservation>,
}

impl ResourcePeriodObservation {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

''')

replace_once(resources, r'''    #[serde(rename = "conditionMortalityDeaths")]
    scarcity_deaths: u64,
}
''', r'''    #[serde(rename = "conditionMortalityDeaths")]
    scarcity_deaths: u64,
    /// Retained diagnostic history. This is downstream observability material and is deliberately
    /// excluded from `digest64`, so adding it cannot alter causal scientific state identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    period_observations: Vec<ResourcePeriodObservation>,
    /// New runs preserve observations from period one. Legacy checkpoints deserialize this as
    /// false, making pre-checkpoint history loss explicit rather than fabricating it.
    #[serde(default)]
    period_observation_history_complete_from_start: bool,
}
''')

replace_once(resources, r'''            household_periods_with_unmet_need: 0,
            scarcity_deaths: 0,
        })
''', r'''            household_periods_with_unmet_need: 0,
            scarcity_deaths: 0,
            period_observations: Vec::new(),
            period_observation_history_complete_from_start: true,
        })
''')

replace_once(resources, r'''    #[must_use]
    pub fn cell_food_stock(&self, cell: crate::ids::CellId) -> Option<u64> {
        let index = usize::try_from(cell.0.checked_sub(1)?).ok()?;
        self.cell_food_stock.get(index).copied()
    }

''', r'''    #[must_use]
    pub fn cell_food_stock(&self, cell: crate::ids::CellId) -> Option<u64> {
        let index = usize::try_from(cell.0.checked_sub(1)?).ok()?;
        self.cell_food_stock.get(index).copied()
    }

    #[must_use]
    pub fn period_observations(&self) -> &[ResourcePeriodObservation] {
        &self.period_observations
    }

    #[must_use]
    pub const fn period_observation_history_complete_from_start(&self) -> bool {
        self.period_observation_history_complete_from_start
    }

''')

replace_once(resources, r'''        let stock_before = self.total_food_stock()?;
        let regenerated = self.regenerate(world, config, period_index_in_year)?;

        let household_count = population.household_count();
''', r'''        let stock_before = self.total_food_stock()?;
        let cell_stock_before_regeneration = self.cell_food_stock.clone();
        let regenerated = self.regenerate(world, config, period_index_in_year)?;
        let cell_stock_after_regeneration = self.cell_food_stock.clone();
        let stock_after_regeneration = self.total_food_stock()?;

        let household_count = population.household_count();
''')

replace_once(resources, r'''        let mut household_need = vec![0_u64; household_count];
        let mut cell_need = vec![0_u64; world.cell_count()];
        let mut total_need = 0_u64;
''', r'''        let mut household_need = vec![0_u64; household_count];
        let mut cell_need = vec![0_u64; world.cell_count()];
        let mut cell_home_need = vec![0_u64; world.cell_count()];
        let mut cell_visitor_need = vec![0_u64; world.cell_count()];
        let mut total_need = 0_u64;
''')

replace_once(resources, r'''                if home_need > 0 {
                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: residence_index,
                        need: home_need,
                    });
                }
''', r'''                if home_need > 0 {
                    cell_home_need[residence_index] = cell_home_need[residence_index]
                        .checked_add(home_need)
                        .ok_or(ResourceError::AccountingOverflow)?;
                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: residence_index,
                        need: home_need,
                    });
                }
''')

replace_once(resources, r'''                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: destination_index,
                        need: visiting_need,
                    });
''', r'''                    cell_visitor_need[destination_index] = cell_visitor_need[destination_index]
                        .checked_add(visiting_need)
                        .ok_or(ResourceError::AccountingOverflow)?;
                    claims.push(ResourceDemandClaim {
                        household_index: household_index_value,
                        cell_index: destination_index,
                        need: visiting_need,
                    });
''')

replace_once(resources, r'''            } else {
                claims.push(ResourceDemandClaim {
                    household_index: household_index_value,
                    cell_index: residence_index,
                    need,
                });
            }
''', r'''            } else {
                cell_home_need[residence_index] = cell_home_need[residence_index]
                    .checked_add(need)
                    .ok_or(ResourceError::AccountingOverflow)?;
                claims.push(ResourceDemandClaim {
                    household_index: household_index_value,
                    cell_index: residence_index,
                    need,
                });
            }
''')

replace_once(resources, r'''        let mut harvested = 0_u64;
''', r'''        let household_supply_fraction =
            household_supply_fraction_distribution(&household_need, &household_harvest)?;
        let households_with_unmet_need = household_need
            .iter()
            .zip(&household_harvest)
            .filter(|(need, supplied)| **need > **supplied)
            .count() as u64;

        let mut harvested = 0_u64;
''')

replace_once(resources, r'''        let people_at_mortality_boundary = population.person_count();
''', r'''        let condition_after_resource_response = condition_distribution(population)?;

        let people_at_mortality_boundary = population.person_count();
''')

replace_once(resources, r'''        let stock_after = self.total_food_stock()?;
        let expected_after = stock_before
''', r'''        let condition_after_mortality = condition_distribution(population)?;
        let stock_after = self.total_food_stock()?;
        let expected_after = stock_before
''')

replace_once(resources, r'''        if stock_after != expected_after {
            return Err(ResourceError::ResourceAccountingMismatch {
                expected: expected_after,
                actual: stock_after,
            });
        }
        self.validate_accounting()?;

        if population.living_count() == 0 {
''', r'''        if stock_after != expected_after {
            return Err(ResourceError::ResourceAccountingMismatch {
                expected: expected_after,
                actual: stock_after,
            });
        }

        let period_duration = period_end
            .checked_sub(period_start)
            .ok_or(ResourceError::AccountingOverflow)?;
        let absolute_start_day = day
            .checked_sub(period_duration)
            .ok_or(ResourceError::AccountingOverflow)?;
        let mut cells = Vec::with_capacity(world.cell_count());
        for cell_index in 0..world.cell_count() {
            let cell_total_need = cell_need[cell_index];
            let supplied = cell_allocated[cell_index];
            cells.push(ResourceCellPeriodObservation {
                cell: CellId::new(cell_index as u64 + 1),
                stock_before_regeneration: cell_stock_before_regeneration[cell_index],
                regenerated: cell_stock_after_regeneration[cell_index]
                    .checked_sub(cell_stock_before_regeneration[cell_index])
                    .ok_or(ResourceError::AccountingOverflow)?,
                stock_after_regeneration: cell_stock_after_regeneration[cell_index],
                home_need: cell_home_need[cell_index],
                visitor_need: cell_visitor_need[cell_index],
                total_need: cell_total_need,
                supplied,
                unmet: cell_total_need
                    .checked_sub(supplied)
                    .ok_or(ResourceError::AccountingOverflow)?,
                stock_after_harvest: self.cell_food_stock[cell_index],
            });
        }
        self.period_observations.push(ResourcePeriodObservation {
            schema_version: ResourcePeriodObservation::CURRENT_SCHEMA_VERSION,
            sequence: self.periods_processed,
            period_index_in_year,
            start_day: absolute_start_day,
            end_day: day,
            stock_before_regeneration: stock_before,
            regenerated,
            stock_after_regeneration,
            total_need,
            supplied: harvested,
            unmet,
            stock_after_harvest: stock_after,
            households_with_unmet_need,
            household_supply_fraction,
            condition_after_resource_response,
            condition_after_mortality,
            cells,
        });
        self.validate_accounting()?;

        if population.living_count() == 0 {
''')

insert_before(resources, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub(crate) enum ResourceStepOutcome", r'''fn household_supply_fraction_distribution(
    household_need: &[u64],
    household_harvest: &[u64],
) -> Result<HouseholdSupplyFractionDistribution, ResourceError> {
    if household_need.len() != household_harvest.len() {
        return Err(ResourceError::InternalInvariant(
            "household resource observation arrays differ in length",
        ));
    }
    let mut result = HouseholdSupplyFractionDistribution {
        households_with_need: 0,
        supplied_0_to_249_permille: 0,
        supplied_250_to_499_permille: 0,
        supplied_500_to_749_permille: 0,
        supplied_750_to_999_permille: 0,
        supplied_1000_permille: 0,
    };
    for (&need, &supplied) in household_need.iter().zip(household_harvest) {
        if need == 0 {
            continue;
        }
        result.households_with_need = result
            .households_with_need
            .checked_add(1)
            .ok_or(ResourceError::AccountingOverflow)?;
        let fraction = supplied
            .saturating_mul(u64::from(PERMILLE_MAX))
            .checked_div(need)
            .ok_or(ResourceError::AccountingOverflow)?
            .min(u64::from(PERMILLE_MAX));
        let bucket = if fraction < 250 {
            &mut result.supplied_0_to_249_permille
        } else if fraction < 500 {
            &mut result.supplied_250_to_499_permille
        } else if fraction < 750 {
            &mut result.supplied_500_to_749_permille
        } else if fraction < 1000 {
            &mut result.supplied_750_to_999_permille
        } else {
            &mut result.supplied_1000_permille
        };
        *bucket = bucket
            .checked_add(1)
            .ok_or(ResourceError::AccountingOverflow)?;
    }
    Ok(result)
}

fn condition_distribution(
    population: &Population,
) -> Result<ConditionDistributionObservation, ResourceError> {
    let mut values = Vec::with_capacity(population.living_count());
    let mut sum = 0_u64;
    let mut below_250 = 0_u64;
    let mut below_500 = 0_u64;
    let mut below_750 = 0_u64;
    for person_index in 0..population.person_count() {
        if !population.is_alive_index(person_index) {
            continue;
        }
        let value = population.condition_at_index(person_index).ok_or(
            ResourceError::InternalInvariant("living person has no condition state"),
        )?;
        sum = sum
            .checked_add(u64::from(value))
            .ok_or(ResourceError::AccountingOverflow)?;
        below_250 += u64::from(value < 250);
        below_500 += u64::from(value < 500);
        below_750 += u64::from(value < 750);
        values.push(value);
    }
    values.sort_unstable();
    let living_people = values.len() as u64;
    let quantile = |numerator: usize, denominator: usize| -> Option<u16> {
        if values.is_empty() {
            None
        } else {
            let index = (values.len() - 1).saturating_mul(numerator) / denominator;
            values.get(index).copied()
        }
    };
    Ok(ConditionDistributionObservation {
        living_people,
        mean_permille: if living_people == 0 {
            None
        } else {
            Some(u16::try_from(sum / living_people).map_err(|_| ResourceError::AccountingOverflow)?)
        },
        minimum_permille: values.first().copied(),
        p10_permille: quantile(1, 10),
        p25_permille: quantile(1, 4),
        median_permille: quantile(1, 2),
        p75_permille: quantile(3, 4),
        p90_permille: quantile(9, 10),
        maximum_permille: values.last().copied(),
        living_below_250_permille: below_250,
        living_below_500_permille: below_500,
        living_below_750_permille: below_750,
    })
}

''')

# Spatial report: expose retained period history and compact temporal scarcity duration/intensity.
spatial = "crates/anthrosim-core/src/spatial_observability.rs"
replace_once(spatial, r'''    PopulationValidationError, ResourceError, ResourceSystem, SimulationCheckpoint,
''', r'''    PopulationValidationError, ResourceError, ResourcePeriodObservation, ResourceSystem,
    SimulationCheckpoint,
''')
replace_once(spatial, r'''    pub migration_distance_distribution: Vec<SpatialMigrationDistanceBin>,
    pub summary: SpatialObservabilitySummary,
    pub unavailable_observables: Vec<String>,
''', r'''    pub migration_distance_distribution: Vec<SpatialMigrationDistanceBin>,
    pub resource_periods: Vec<ResourcePeriodObservation>,
    pub resource_temporal_summary: ResourceTemporalObservabilitySummary,
    pub summary: SpatialObservabilitySummary,
    pub unavailable_observables: Vec<String>,
''')
replace_once(spatial, "    pub const CURRENT_SCHEMA_VERSION: u32 = 3;\n", "    pub const CURRENT_SCHEMA_VERSION: u32 = 4;\n")
insert_before(spatial, "#[derive(Debug, Clone, Default)]\nstruct CellAccumulator", r'''#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemporalObservabilitySummary {
    pub provenance: MetricProvenance,
    pub history_complete_from_start: bool,
    pub preserved_periods: u64,
    pub periods_with_unmet_need: u64,
    pub longest_consecutive_scarcity_periods: u64,
    pub total_unmet_need: u64,
    pub maximum_period_unmet_need: u64,
}

''')
replace_once(spatial, r'''    let summary = build_summary(end_day, &cell_rows, checkpoint)?;
    let normalized_layers = landscape
''', r'''    let summary = build_summary(end_day, &cell_rows, checkpoint)?;
    let resource_periods = checkpoint.resources.period_observations().to_vec();
    let resource_temporal_summary = build_resource_temporal_summary(
        &resource_periods,
        checkpoint
            .resources
            .period_observation_history_complete_from_start(),
    )?;
    let normalized_layers = landscape
''')
replace_once(spatial, r'''        migration_flows,
        migration_distance_distribution,
        summary,
        unavailable_observables: vec![
            "historical per-cell food stock between serialized checkpoint boundaries is not recorded"
                .to_owned(),
            "per-cell unmet resource need is not recorded by the current resource system".to_owned(),
            "historical per-person condition between authoritative death/checkpoint observations is not recorded"
                .to_owned(),
''', r'''        migration_flows,
        migration_distance_distribution,
        resource_periods,
        resource_temporal_summary,
        summary,
        unavailable_observables: {
            let mut unavailable = vec![
            "historical per-person condition trajectories are not retained; compact condition distributions are preserved at resource-period boundaries"
                .to_owned(),
''')
replace_once(spatial, r'''            "Death.cell and spatial death counts are attributed to persistent residence, not necessarily the physical location of death while a household is away"
                .to_owned(),
        ],
    })
}
''', r'''            "Death.cell and spatial death counts are attributed to persistent residence, not necessarily the physical location of death while a household is away"
                .to_owned(),
            ];
            if !checkpoint
                .resources
                .period_observation_history_complete_from_start()
            {
                unavailable.push(
                    "resource-period history before the source checkpoint boundary is unavailable because this run resumed from a legacy checkpoint without retained M3 period observations"
                        .to_owned(),
                );
            }
            unavailable
        },
    })
}
''')
insert_before(spatial, "fn build_summary(\n", r'''fn build_resource_temporal_summary(
    periods: &[ResourcePeriodObservation],
    history_complete_from_start: bool,
) -> Result<ResourceTemporalObservabilitySummary, SpatialObservabilityError> {
    let mut periods_with_unmet_need = 0_u64;
    let mut current_scarcity_run = 0_u64;
    let mut longest_consecutive_scarcity_periods = 0_u64;
    let mut total_unmet_need = 0_u64;
    let mut maximum_period_unmet_need = 0_u64;
    for period in periods {
        if period.schema_version != ResourcePeriodObservation::CURRENT_SCHEMA_VERSION {
            return Err(SpatialObservabilityError::UnsupportedResourcePeriodSchema {
                found: period.schema_version,
                supported: ResourcePeriodObservation::CURRENT_SCHEMA_VERSION,
            });
        }
        total_unmet_need = total_unmet_need
            .checked_add(period.unmet)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        maximum_period_unmet_need = maximum_period_unmet_need.max(period.unmet);
        if period.unmet > 0 {
            periods_with_unmet_need = periods_with_unmet_need
                .checked_add(1)
                .ok_or(SpatialObservabilityError::AccountingOverflow)?;
            current_scarcity_run = current_scarcity_run
                .checked_add(1)
                .ok_or(SpatialObservabilityError::AccountingOverflow)?;
            longest_consecutive_scarcity_periods =
                longest_consecutive_scarcity_periods.max(current_scarcity_run);
        } else {
            current_scarcity_run = 0;
        }
    }
    Ok(ResourceTemporalObservabilitySummary {
        provenance: MetricProvenance::Derived,
        history_complete_from_start,
        preserved_periods: periods.len() as u64,
        periods_with_unmet_need,
        longest_consecutive_scarcity_periods,
        total_unmet_need,
        maximum_period_unmet_need,
    })
}

''')
insert_before(spatial, "    #[error(\"spatial observability accounting overflow\")]\n", r'''    #[error("unsupported retained resource-period schema {found}; supported schema is {supported}")]
    UnsupportedResourcePeriodSchema { found: u32, supported: u32 },
''')

# Public exports.
lib = "crates/anthrosim-core/src/lib.rs"
replace_once(lib, r'''pub use resources::{
    ResourceConfigError, ResourceError, ResourceSummary, ResourceSystem, validate_resource_config,
};
''', r'''pub use resources::{
    ConditionDistributionObservation, HouseholdSupplyFractionDistribution, ResourceCellPeriodObservation,
    ResourceConfigError, ResourceError, ResourcePeriodObservation, ResourceSummary, ResourceSystem,
    validate_resource_config,
};
''')
replace_once(lib, r'''    SpatialObservabilitySemantics, SpatialObservabilitySource, SpatialObservabilitySummary,
    derive_spatial_observability,
};
''', r'''    SpatialObservabilitySemantics, SpatialObservabilitySource, SpatialObservabilitySummary,
    ResourceTemporalObservabilitySummary, derive_spatial_observability,
};
''')

# Integration tests use public simulation/checkpoint surfaces and prove preservation + diagnostic distinctions.
test = Path("crates/anthrosim-core/tests/resource_period_observability.rs")
test.write_text(r'''use anthrosim_core::{
    ExperimentConfig, ResourcePeriodObservation, Simulation,
};

#[test]
fn resource_period_history_is_preserved_and_reconciles() {
    let mut config = ExperimentConfig::new(21501, 2);
    config.world.width = 3;
    config.world.height = 2;
    config.population.initial_population = 24;
    config.resources.annual_need_units_per_person = 220;
    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let periods = checkpoint.resources.period_observations();
    assert_eq!(periods.len() as u64, checkpoint.resources.summary(&checkpoint.population).periods_processed);
    assert!(checkpoint.resources.period_observation_history_complete_from_start());
    for period in periods {
        assert_eq!(period.schema_version, ResourcePeriodObservation::CURRENT_SCHEMA_VERSION);
        assert_eq!(period.total_need, period.supplied + period.unmet);
        assert_eq!(
            period.stock_before_regeneration + period.regenerated - period.supplied,
            period.stock_after_harvest
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.total_need).sum::<u64>(),
            period.total_need
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.supplied).sum::<u64>(),
            period.supplied
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.unmet).sum::<u64>(),
            period.unmet
        );
    }
}

#[test]
fn checkpoint_resume_preserves_exact_period_history() {
    let mut config = ExperimentConfig::new(21502, 3);
    config.world.width = 2;
    config.world.height = 2;
    config.population.initial_population = 20;
    let uninterrupted = Simulation::new(config.clone()).unwrap().run_recorded().unwrap();
    let boundary = Simulation::new(config).unwrap().checkpoint_at_year(1).unwrap();
    let resumed = Simulation::from_checkpoint(boundary).unwrap().run_recorded().unwrap();
    assert_eq!(
        uninterrupted.checkpoint.resources.period_observations(),
        resumed.checkpoint.resources.period_observations()
    );
}

#[test]
fn period_history_distinguishes_temporal_shapes_with_same_terminal_totals() {
    let mut config = ExperimentConfig::new(21503, 1);
    config.world.width = 1;
    config.world.height = 1;
    config.population.initial_population = 5;
    let checkpoint = Simulation::new(config).unwrap().run_recorded().unwrap().checkpoint;
    let template = checkpoint.resources.period_observations()[0].clone();
    let mut chronic = vec![template.clone(), template.clone(), template.clone(), template.clone()];
    let mut acute = chronic.clone();
    for (index, period) in chronic.iter_mut().enumerate() {
        period.sequence = index as u64 + 1;
        period.unmet = 25;
    }
    for (index, period) in acute.iter_mut().enumerate() {
        period.sequence = index as u64 + 1;
        period.unmet = if index == 0 { 100 } else { 0 };
    }
    assert_eq!(
        chronic.iter().map(|period| period.unmet).sum::<u64>(),
        acute.iter().map(|period| period.unmet).sum::<u64>()
    );
    assert_ne!(
        chronic.iter().map(|period| period.unmet).collect::<Vec<_>>(),
        acute.iter().map(|period| period.unmet).collect::<Vec<_>>()
    );
}
''')

# Normative research contract.
Path("docs/research/resource-condition-period-observability-v1.md").write_text(r'''# Resource and condition period observability v1

Issue #215 adds retained M3 resource-period observations so a preserved run can diagnose trajectory shape without rerunning with undocumented instrumentation.

## Authoritative timing

One observation is recorded after every authoritative M3 resource boundary. The row is downstream observability: it does not feed resource allocation, condition response, mortality, migration, RNG choice, or scheduling. Rows are retained in the checkpoint's `ResourceSystem` and therefore survive exact checkpoint/resume. They are also copied into `spatial-observability.json`, whose source block binds them to the exact run state, model semantics, seed, landscape identity and spatial configuration.

The observation schema is `ResourcePeriodObservation` v1. The parent resource-system causal schema is intentionally unchanged: observation history is excluded from `ResourceSystem::digest64`, so causal state identity and `MODEL_SEMANTICS_ID` do not change. Complete continuation identity still binds the serialized observation history, because an exact resumed output must preserve it.

## Per-period quantities

Each period preserves the absolute start/end day, period sequence/index, total stock before regeneration, regeneration, stock after regeneration, need, supplied amount, unmet need and stock after harvest. Each cell preserves the same stock/need/supply accounting, plus demand decomposed into `homeNeed` and `visitorNeed` under M9 duration-weighted provisioning. Without temporary mobility all demand is home demand.

The identities reconcile exactly:

`need = supplied + unmet`

`stockAfterHarvest = stockBeforeRegeneration + regenerated - supplied`

and cell totals reconcile to the period totals.

## Household supply distribution

Rather than retain every household allocation, each period records the count of households with positive need in fixed supplied-fraction bins: 0-249, 250-499, 500-749, 750-999 and 1000 permille. This distinguishes broad mild shortfall from concentrated severe shortfall without creating a household event log.

## Condition distribution

Each period records compact living-condition distributions twice: immediately after the deterministic resource response and again after the competing M2/M3 mortality boundary. The summaries include living count, mean, min/max, p10/p25/median/p75/p90, and counts below 250/500/750 permille. Quantiles use the deterministic lower order statistic at `floor((n-1)*p)`.

Individual longitudinal condition trajectories are intentionally not retained in v1. They remain explicitly unavailable because the audit requirement can be met with compact distribution histories and retaining person-level time series would have materially larger storage/privacy-like analysis costs.

## Scarcity duration and intensity

Spatial observability schema v4 exposes the exact period rows plus a compact temporal summary: preserved period count, periods with unmet need, longest consecutive scarcity run, cumulative unmet need and maximum single-period unmet need. This makes chronic mild scarcity distinguishable from acute scarcity even when cumulative totals or terminal state are similar.

## Legacy resumes

New simulations mark resource-period history as complete from the start. Legacy checkpoints deserialize with an empty history and `historyCompleteFromStart = false`; subsequent observations are still retained, but spatial observability emits an explicit warning that the pre-checkpoint trajectory is unavailable. Missing history is never fabricated.

## Scientific boundary

These records are observability only. They do not change M3 allocation, condition response, M2/M3 competing mortality, M4/M9 behavior, random streams, scientific state digests, or `MODEL_SEMANTICS_ID`.
''')

print("issue 215 source patch applied")
