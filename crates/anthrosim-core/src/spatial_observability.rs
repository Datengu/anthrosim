use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    DeathCause, EventKind, EventLog, EventProvenance, GridGeometry, LandscapeBundle,
    LandscapeError, LandscapeLayerRole, LandscapeValueDomain, MetricProvenance, Population,
    PopulationValidationError, ResourceError, ResourcePeriodObservation, ResourceSystem,
    SimulationCheckpoint, SpatialMechanismBinding, World,
    ids::{CellId, PersonId},
};

/// Derived M8.5 spatial observability artifact.
///
/// This report is downstream of authoritative simulation execution. It never participates in the
/// hot loop, RNG state or checkpoint state and can be regenerated from preserved run artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialObservabilityReport {
    pub schema_version: u32,
    pub provenance: MetricProvenance,
    pub source: SpatialObservabilitySource,
    pub semantics: SpatialObservabilitySemantics,
    pub geometry: GridGeometry,
    pub width: u32,
    pub height: u32,
    pub normalized_layers: Vec<SpatialLayerDescriptor>,
    pub cells: Vec<SpatialCellObservability>,
    pub migration_flows: Vec<SpatialMigrationFlow>,
    pub migration_distance_distribution: Vec<SpatialMigrationDistanceBin>,
    pub resource_periods: Vec<ResourcePeriodObservation>,
    pub resource_temporal_summary: ResourceTemporalObservabilitySummary,
    pub summary: SpatialObservabilitySummary,
    pub unavailable_observables: Vec<String>,
}

impl SpatialObservabilityReport {
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialLocationAttribution {
    PersistentResidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialObservabilitySemantics {
    pub population_location_basis: SpatialLocationAttribution,
    pub occupancy_includes_temporary_visitors: bool,
    pub occupancy_includes_transit: bool,
    pub birth_cell_attribution: SpatialLocationAttribution,
    pub death_cell_attribution: SpatialLocationAttribution,
    pub physical_presence_companion_artifact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialObservabilitySource {
    pub model_version: String,
    pub model_semantics_id: String,
    pub git_commit: Option<String>,
    pub seed: u64,
    pub end_day: u64,
    pub run_state_digest64: u64,
    pub landscape_identity: String,
    pub landscape_digest64: u64,
    pub transformed_world_digest64: u64,
    pub spatial_model_semantics_id: Option<String>,
    pub spatial_config_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialLayerDescriptor {
    pub layer_id: String,
    pub role: LandscapeLayerRole,
    pub unit: String,
    pub value_domain: Option<LandscapeValueDomain>,
    pub evidence_input_id: Option<String>,
    pub nodata_cells: u64,
    pub source_artifact: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialCellObservability {
    pub cell: CellId,
    pub grid_x: u32,
    pub grid_y: u32,
    pub model_facing: SpatialModelFacingCell,
    pub derived: SpatialDerivedCell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialModelFacingCell {
    pub provenance: String,
    pub movement_cost: u16,
    pub water_access: u16,
    pub base_productivity: u16,
    pub initial_food_stock: u64,
    pub terminal_food_stock: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialDerivedCell {
    pub provenance: MetricProvenance,
    pub initial_living_population: u64,
    pub terminal_living_population: u64,
    pub occupied_duration_days: u64,
    pub occupancy_fraction_permille: Option<u16>,
    pub living_person_days: u64,
    pub births: u64,
    pub deaths: u64,
    #[serde(rename = "conditionMortalityDeaths")]
    pub resource_scarcity_deaths: u64,
    pub migration_moves_in: u64,
    pub migration_moves_out: u64,
    pub migration_people_in: u64,
    pub migration_people_out: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialMigrationFlow {
    pub provenance: MetricProvenance,
    pub origin: CellId,
    pub destination: CellId,
    pub distance_cells: u16,
    pub moves: u64,
    pub people_moved: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialMigrationDistanceBin {
    pub provenance: MetricProvenance,
    pub distance_cells: u16,
    pub moves: u64,
    pub people_moved: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialObservabilitySummary {
    pub provenance: MetricProvenance,
    pub observation_duration_days: u64,
    pub terminal_living_population: u64,
    pub terminal_occupied_cells: u64,
    pub occupied_cell_days: u64,
    pub cell_time_occupied_permille: Option<u16>,
    pub living_person_days: u64,
    pub terminal_largest_cell_population: u64,
    pub terminal_largest_cell_share_permille: Option<u16>,
    pub terminal_population_herfindahl_per_million: Option<u32>,
    pub births: u64,
    pub deaths: u64,
    #[serde(rename = "conditionMortalityDeaths")]
    pub resource_scarcity_deaths: u64,
    pub migration_moves: u64,
    pub migration_people_moved: u64,
    pub migration_total_distance_cells: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default)]
struct CellAccumulator {
    current_population: u64,
    initial_population: u64,
    last_change_day: u64,
    occupied_duration_days: u64,
    living_person_days: u64,
    births: u64,
    deaths: u64,
    resource_scarcity_deaths: u64,
    migration_moves_in: u64,
    migration_moves_out: u64,
    migration_people_in: u64,
    migration_people_out: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct FlowAccumulator {
    distance_cells: u16,
    moves: u64,
    people_moved: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct DistanceAccumulator {
    moves: u64,
    people_moved: u64,
}

pub fn derive_spatial_observability(
    landscape: &LandscapeBundle,
    world: &World,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
    spatial: Option<&SpatialMechanismBinding>,
) -> Result<SpatialObservabilityReport, SpatialObservabilityError> {
    landscape.validate()?;
    if landscape.width != world.width() || landscape.height != world.height() {
        return Err(SpatialObservabilityError::GridMismatch);
    }
    initial_population.validate(world)?;
    checkpoint.population.validate(world)?;
    if checkpoint.world_digest64 != world.digest64() {
        return Err(SpatialObservabilityError::WorldDigestMismatch {
            expected: checkpoint.world_digest64,
            actual: world.digest64(),
        });
    }
    if let Some(spatial) = spatial
        && spatial.transformed_world_digest64 != world.digest64()
    {
        return Err(SpatialObservabilityError::SpatialWorldDigestMismatch {
            expected: spatial.transformed_world_digest64,
            actual: world.digest64(),
        });
    }
    let initial_resources = ResourceSystem::initialize(world, &checkpoint.experiment.resources)?;

    let end_day = checkpoint.time.days();
    let mut cells = vec![CellAccumulator::default(); world.cell_count()];
    initialize_counts(initial_population, &mut cells)?;
    let mut flow_map = BTreeMap::<(u64, u64), FlowAccumulator>::new();
    let mut distance_map = BTreeMap::<u16, DistanceAccumulator>::new();
    process_events(
        &checkpoint.events,
        end_day,
        &mut cells,
        &mut flow_map,
        &mut distance_map,
    )?;
    finalize_cells(end_day, &mut cells)?;
    validate_terminal_counts(&checkpoint.population, &cells)?;

    let mut cell_rows = Vec::with_capacity(cells.len());
    for (index, accumulator) in cells.iter().enumerate() {
        let cell = CellId::new(index as u64 + 1);
        let (grid_x, grid_y) = world
            .coordinates(cell)
            .ok_or(SpatialObservabilityError::InvalidCell(cell))?;
        let world_cell = world
            .cell(cell)
            .ok_or(SpatialObservabilityError::InvalidCell(cell))?;
        let initial_food_stock = initial_resources
            .cell_food_stock(cell)
            .ok_or(SpatialObservabilityError::InvalidCell(cell))?;
        let terminal_food_stock = checkpoint
            .resources
            .cell_food_stock(cell)
            .ok_or(SpatialObservabilityError::InvalidCell(cell))?;
        cell_rows.push(SpatialCellObservability {
            cell,
            grid_x,
            grid_y,
            model_facing: SpatialModelFacingCell {
                provenance: "authoritative_world_m3_initialization_and_checkpoint".to_owned(),
                movement_cost: world_cell.movement_cost,
                water_access: world_cell.water_access,
                base_productivity: world_cell.base_productivity,
                initial_food_stock,
                terminal_food_stock,
            },
            derived: SpatialDerivedCell {
                provenance: MetricProvenance::Derived,
                initial_living_population: accumulator.initial_population,
                terminal_living_population: accumulator.current_population,
                occupied_duration_days: accumulator.occupied_duration_days,
                occupancy_fraction_permille: ratio_permille(
                    accumulator.occupied_duration_days,
                    end_day,
                )?,
                living_person_days: accumulator.living_person_days,
                births: accumulator.births,
                deaths: accumulator.deaths,
                resource_scarcity_deaths: accumulator.resource_scarcity_deaths,
                migration_moves_in: accumulator.migration_moves_in,
                migration_moves_out: accumulator.migration_moves_out,
                migration_people_in: accumulator.migration_people_in,
                migration_people_out: accumulator.migration_people_out,
            },
        });
    }

    let migration_flows = flow_map
        .into_iter()
        .map(
            |((origin, destination), accumulator)| SpatialMigrationFlow {
                provenance: MetricProvenance::Derived,
                origin: CellId::new(origin),
                destination: CellId::new(destination),
                distance_cells: accumulator.distance_cells,
                moves: accumulator.moves,
                people_moved: accumulator.people_moved,
            },
        )
        .collect::<Vec<_>>();
    let migration_distance_distribution = distance_map
        .into_iter()
        .map(
            |(distance_cells, accumulator)| SpatialMigrationDistanceBin {
                provenance: MetricProvenance::Derived,
                distance_cells,
                moves: accumulator.moves,
                people_moved: accumulator.people_moved,
            },
        )
        .collect::<Vec<_>>();

    let summary = build_summary(end_day, &cell_rows, checkpoint)?;
    let resource_periods = checkpoint.resources.period_observations().to_vec();
    let resource_temporal_summary = build_resource_temporal_summary(
        &resource_periods,
        checkpoint
            .resources
            .period_observation_history_complete_from_start(),
    )?;
    let normalized_layers = landscape
        .layers
        .iter()
        .map(|layer| SpatialLayerDescriptor {
            layer_id: layer.layer_id.clone(),
            role: layer.role,
            unit: layer.unit.clone(),
            value_domain: layer.value_domain,
            evidence_input_id: layer.evidence_input_id.clone(),
            nodata_cells: layer.values.iter().filter(|value| value.is_none()).count() as u64,
            source_artifact: "landscape.json".to_owned(),
        })
        .collect();

    Ok(SpatialObservabilityReport {
        schema_version: SpatialObservabilityReport::CURRENT_SCHEMA_VERSION,
        provenance: MetricProvenance::Derived,
        source: SpatialObservabilitySource {
            model_version: checkpoint.model_version.clone(),
            model_semantics_id: checkpoint.model_semantics_id.clone(),
            git_commit: checkpoint.git_commit.clone(),
            seed: checkpoint.experiment.seed,
            end_day,
            run_state_digest64: checkpoint.state_digest64,
            landscape_identity: landscape.identity(),
            landscape_digest64: landscape.digest64(),
            transformed_world_digest64: world.digest64(),
            spatial_model_semantics_id: spatial
                .map(|binding| binding.spatial_model_semantics_id.clone()),
            spatial_config_identity: spatial.map(|binding| binding.config_identity.clone()),
        },
        semantics: SpatialObservabilitySemantics {
            population_location_basis: SpatialLocationAttribution::PersistentResidence,
            occupancy_includes_temporary_visitors: false,
            occupancy_includes_transit: false,
            birth_cell_attribution: SpatialLocationAttribution::PersistentResidence,
            death_cell_attribution: SpatialLocationAttribution::PersistentResidence,
            physical_presence_companion_artifact: checkpoint
                .experiment
                .temporary_mobility
                .as_ref()
                .map(|_| "temporary-observability.json".to_owned()),
        },
        geometry: landscape.geometry.clone(),
        width: landscape.width,
        height: landscape.height,
        normalized_layers,
        cells: cell_rows,
        migration_flows,
        migration_distance_distribution,
        resource_periods,
        resource_temporal_summary,
        summary,
        unavailable_observables: {
            let mut unavailable = vec![
            "historical per-person condition trajectories are not retained; compact condition distributions are preserved at resource-period boundaries"
                .to_owned(),
            "spatial population, occupancy, person-day, birth and death cell observables use persistent residence and exclude temporary visitors and transit; use temporary-observability.json for M9 physical presence"
                .to_owned(),
            "Death.cell and spatial death counts are attributed to persistent residence, not necessarily the physical location of death while a household is away"
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

fn initialize_counts(
    initial_population: &Population,
    cells: &mut [CellAccumulator],
) -> Result<(), SpatialObservabilityError> {
    for raw_id in 1..=initial_population.person_count() as u64 {
        let person = initial_population
            .person(PersonId::new(raw_id))
            .ok_or(SpatialObservabilityError::InvalidInitialPerson(raw_id))?;
        if !person.is_alive() {
            return Err(SpatialObservabilityError::InitialPopulationContainsDeath {
                person: person.id,
            });
        }
        let index = cell_index(person.location, cells.len())?;
        cells[index].current_population = cells[index]
            .current_population
            .checked_add(1)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        cells[index].initial_population = cells[index].current_population;
    }
    Ok(())
}

fn process_events(
    events: &EventLog,
    end_day: u64,
    cells: &mut [CellAccumulator],
    flows: &mut BTreeMap<(u64, u64), FlowAccumulator>,
    distances: &mut BTreeMap<u16, DistanceAccumulator>,
) -> Result<(), SpatialObservabilityError> {
    let mut previous_sequence = 0_u64;
    let mut previous_day = 0_u64;
    for record in &events.events {
        if record.provenance != EventProvenance::Authoritative {
            return Err(SpatialObservabilityError::NonAuthoritativeEvent {
                sequence: record.sequence,
            });
        }
        if record.sequence != previous_sequence.saturating_add(1) {
            return Err(SpatialObservabilityError::EventSequenceMismatch {
                expected: previous_sequence.saturating_add(1),
                actual: record.sequence,
            });
        }
        if record.day < previous_day || record.day > end_day {
            return Err(SpatialObservabilityError::EventDayOutOfRange {
                sequence: record.sequence,
                day: record.day,
                end_day,
            });
        }
        previous_sequence = record.sequence;
        previous_day = record.day;

        match &record.event {
            EventKind::Birth { cell, .. } => {
                let accumulator = touch_cell(*cell, record.day, cells)?;
                accumulator.current_population = accumulator
                    .current_population
                    .checked_add(1)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                accumulator.births = accumulator
                    .births
                    .checked_add(1)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
            }
            EventKind::Death { cell, cause, .. } => {
                let accumulator = touch_cell(*cell, record.day, cells)?;
                accumulator.current_population = accumulator
                    .current_population
                    .checked_sub(1)
                    .ok_or(SpatialObservabilityError::NegativeCellPopulation {
                        cell: *cell,
                        day: record.day,
                    })?;
                accumulator.deaths = accumulator
                    .deaths
                    .checked_add(1)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                if *cause == DeathCause::ResourceScarcity {
                    accumulator.resource_scarcity_deaths = accumulator
                        .resource_scarcity_deaths
                        .checked_add(1)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                }
            }
            EventKind::HouseholdMigration {
                people_moved,
                origin,
                destination,
                distance_cells,
                ..
            } => {
                let people_moved = u64::from(*people_moved);
                {
                    let origin_accumulator = touch_cell(*origin, record.day, cells)?;
                    origin_accumulator.current_population = origin_accumulator
                        .current_population
                        .checked_sub(people_moved)
                        .ok_or(SpatialObservabilityError::NegativeCellPopulation {
                            cell: *origin,
                            day: record.day,
                        })?;
                    origin_accumulator.migration_moves_out = origin_accumulator
                        .migration_moves_out
                        .checked_add(1)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                    origin_accumulator.migration_people_out = origin_accumulator
                        .migration_people_out
                        .checked_add(people_moved)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                }
                {
                    let destination_accumulator = touch_cell(*destination, record.day, cells)?;
                    destination_accumulator.current_population = destination_accumulator
                        .current_population
                        .checked_add(people_moved)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                    destination_accumulator.migration_moves_in = destination_accumulator
                        .migration_moves_in
                        .checked_add(1)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                    destination_accumulator.migration_people_in = destination_accumulator
                        .migration_people_in
                        .checked_add(people_moved)
                        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                }

                let flow = flows
                    .entry((origin.0, destination.0))
                    .or_insert(FlowAccumulator {
                        distance_cells: *distance_cells,
                        ..FlowAccumulator::default()
                    });
                if flow.distance_cells != *distance_cells {
                    return Err(SpatialObservabilityError::FlowDistanceMismatch {
                        origin: *origin,
                        destination: *destination,
                    });
                }
                flow.moves = flow
                    .moves
                    .checked_add(1)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                flow.people_moved = flow
                    .people_moved
                    .checked_add(people_moved)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;

                let distance = distances.entry(*distance_cells).or_default();
                distance.moves = distance
                    .moves
                    .checked_add(1)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
                distance.people_moved = distance
                    .people_moved
                    .checked_add(people_moved)
                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;
            }
            EventKind::TemporaryJourneyNotStarted { .. }
            | EventKind::TemporaryJourneyDeparted { .. }
            | EventKind::TemporaryJourneyArrived { .. }
            | EventKind::TemporaryReturnDeparted { .. }
            | EventKind::TemporaryJourneyCompleted { .. } => {
                // M8 observability remains residence/permanent-migration based. M9.6 adds
                // temporary-presence observability rather than overloading these M8 metrics.
            }
        }
    }
    Ok(())
}

fn touch_cell(
    cell: CellId,
    day: u64,
    cells: &mut [CellAccumulator],
) -> Result<&mut CellAccumulator, SpatialObservabilityError> {
    let index = cell_index(cell, cells.len())?;
    let accumulator = &mut cells[index];
    accumulate_interval(accumulator, day)?;
    Ok(accumulator)
}

fn accumulate_interval(
    cell: &mut CellAccumulator,
    day: u64,
) -> Result<(), SpatialObservabilityError> {
    let duration = day
        .checked_sub(cell.last_change_day)
        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    if cell.current_population > 0 {
        cell.occupied_duration_days = cell
            .occupied_duration_days
            .checked_add(duration)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    }
    let person_days = cell
        .current_population
        .checked_mul(duration)
        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    cell.living_person_days = cell
        .living_person_days
        .checked_add(person_days)
        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    cell.last_change_day = day;
    Ok(())
}

fn finalize_cells(
    end_day: u64,
    cells: &mut [CellAccumulator],
) -> Result<(), SpatialObservabilityError> {
    for cell in cells {
        accumulate_interval(cell, end_day)?;
    }
    Ok(())
}

fn validate_terminal_counts(
    population: &Population,
    cells: &[CellAccumulator],
) -> Result<(), SpatialObservabilityError> {
    let mut authoritative = vec![0_u64; cells.len()];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population
            .person(PersonId::new(raw_id))
            .ok_or(SpatialObservabilityError::InvalidTerminalPerson(raw_id))?;
        if person.is_alive() {
            let index = cell_index(person.location, cells.len())?;
            authoritative[index] = authoritative[index]
                .checked_add(1)
                .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        }
    }
    for (index, (derived, authoritative)) in cells
        .iter()
        .map(|cell| cell.current_population)
        .zip(authoritative)
        .enumerate()
    {
        if derived != authoritative {
            return Err(SpatialObservabilityError::TerminalCellPopulationMismatch {
                cell: CellId::new(index as u64 + 1),
                derived,
                authoritative,
            });
        }
    }
    Ok(())
}

fn build_resource_temporal_summary(
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

fn build_summary(
    end_day: u64,
    cells: &[SpatialCellObservability],
    checkpoint: &SimulationCheckpoint,
) -> Result<SpatialObservabilitySummary, SpatialObservabilityError> {
    let mut terminal_population = 0_u64;
    let mut terminal_occupied_cells = 0_u64;
    let mut occupied_cell_days = 0_u64;
    let mut living_person_days = 0_u64;
    let mut largest_cell_population = 0_u64;
    let mut sum_squares = 0_u128;
    let mut births = 0_u64;
    let mut deaths = 0_u64;
    let mut scarcity_deaths = 0_u64;
    let mut migration_moves = 0_u64;
    let mut migration_people_moved = 0_u64;

    for cell in cells {
        let derived = &cell.derived;
        terminal_population = terminal_population
            .checked_add(derived.terminal_living_population)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        if derived.terminal_living_population > 0 {
            terminal_occupied_cells = terminal_occupied_cells
                .checked_add(1)
                .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        }
        largest_cell_population = largest_cell_population.max(derived.terminal_living_population);
        let population = u128::from(derived.terminal_living_population);
        sum_squares = sum_squares
            .checked_add(population.saturating_mul(population))
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        occupied_cell_days = occupied_cell_days
            .checked_add(derived.occupied_duration_days)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        living_person_days = living_person_days
            .checked_add(derived.living_person_days)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        births = births
            .checked_add(derived.births)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        deaths = deaths
            .checked_add(derived.deaths)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        scarcity_deaths = scarcity_deaths
            .checked_add(derived.resource_scarcity_deaths)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        migration_moves = migration_moves
            .checked_add(derived.migration_moves_out)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        migration_people_moved = migration_people_moved
            .checked_add(derived.migration_people_out)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    }

    let population_summary = checkpoint.population.summary();
    if terminal_population != population_summary.living_population
        || terminal_occupied_cells != population_summary.living_occupied_cell_count
        || births != population_summary.births_since_start
        || deaths != population_summary.deaths_since_start
        || scarcity_deaths
            != checkpoint
                .resources
                .summary(&checkpoint.population)
                .scarcity_deaths
        || migration_moves != checkpoint.migration.moves_completed
        || migration_people_moved != checkpoint.migration.people_moved
    {
        return Err(SpatialObservabilityError::SummaryMismatch);
    }

    let total_distance_cells = checkpoint.migration.total_distance_cells;
    let cell_time_denominator = end_day
        .checked_mul(cells.len() as u64)
        .ok_or(SpatialObservabilityError::AccountingOverflow)?;
    let hhi = if terminal_population == 0 {
        None
    } else {
        let denominator = u128::from(terminal_population)
            .checked_mul(u128::from(terminal_population))
            .ok_or(SpatialObservabilityError::AccountingOverflow)?;
        let scaled = sum_squares
            .checked_mul(1_000_000)
            .ok_or(SpatialObservabilityError::AccountingOverflow)?
            / denominator;
        Some(u32::try_from(scaled).map_err(|_| SpatialObservabilityError::AccountingOverflow)?)
    };

    Ok(SpatialObservabilitySummary {
        provenance: MetricProvenance::Derived,
        observation_duration_days: end_day,
        terminal_living_population: terminal_population,
        terminal_occupied_cells,
        occupied_cell_days,
        cell_time_occupied_permille: ratio_permille(occupied_cell_days, cell_time_denominator)?,
        living_person_days,
        terminal_largest_cell_population: largest_cell_population,
        terminal_largest_cell_share_permille: ratio_permille(
            largest_cell_population,
            terminal_population,
        )?,
        terminal_population_herfindahl_per_million: hhi,
        births,
        deaths,
        resource_scarcity_deaths: scarcity_deaths,
        migration_moves,
        migration_people_moved,
        migration_total_distance_cells: total_distance_cells,
    })
}

fn ratio_permille(
    numerator: u64,
    denominator: u64,
) -> Result<Option<u16>, SpatialObservabilityError> {
    if denominator == 0 {
        return Ok(None);
    }
    let scaled = u128::from(numerator)
        .checked_mul(1_000)
        .ok_or(SpatialObservabilityError::AccountingOverflow)?
        / u128::from(denominator);
    Ok(Some(u16::try_from(scaled).map_err(|_| {
        SpatialObservabilityError::AccountingOverflow
    })?))
}

fn cell_index(cell: CellId, cell_count: usize) -> Result<usize, SpatialObservabilityError> {
    let index = usize::try_from(
        cell.0
            .checked_sub(1)
            .ok_or(SpatialObservabilityError::InvalidCell(cell))?,
    )
    .map_err(|_| SpatialObservabilityError::InvalidCell(cell))?;
    if index >= cell_count {
        return Err(SpatialObservabilityError::InvalidCell(cell));
    }
    Ok(index)
}

#[derive(Debug, Error)]
pub enum SpatialObservabilityError {
    #[error(transparent)]
    Landscape(#[from] LandscapeError),
    #[error(transparent)]
    Population(#[from] PopulationValidationError),
    #[error(transparent)]
    Resource(#[from] ResourceError),
    #[error("landscape and world grids do not match")]
    GridMismatch,
    #[error("checkpoint world digest {expected} does not match supplied world digest {actual}")]
    WorldDigestMismatch { expected: u64, actual: u64 },
    #[error(
        "spatial binding world digest {expected} does not match supplied world digest {actual}"
    )]
    SpatialWorldDigestMismatch { expected: u64, actual: u64 },
    #[error("invalid cell {0:?} in spatial observability input")]
    InvalidCell(CellId),
    #[error("initial population person {0} is missing")]
    InvalidInitialPerson(u64),
    #[error("terminal population person {0} is missing")]
    InvalidTerminalPerson(u64),
    #[error("initial population unexpectedly contains a death for person {person:?}")]
    InitialPopulationContainsDeath { person: PersonId },
    #[error("event {sequence} is not authoritative")]
    NonAuthoritativeEvent { sequence: u64 },
    #[error("event sequence mismatch: expected {expected}, found {actual}")]
    EventSequenceMismatch { expected: u64, actual: u64 },
    #[error("event {sequence} day {day} is outside 0..={end_day}")]
    EventDayOutOfRange {
        sequence: u64,
        day: u64,
        end_day: u64,
    },
    #[error("cell {cell:?} would have negative population at day {day}")]
    NegativeCellPopulation { cell: CellId, day: u64 },
    #[error("migration flow {origin:?}->{destination:?} has inconsistent recorded distance")]
    FlowDistanceMismatch { origin: CellId, destination: CellId },
    #[error(
        "terminal population reconstruction mismatch for {cell:?}: derived {derived}, authoritative {authoritative}"
    )]
    TerminalCellPopulationMismatch {
        cell: CellId,
        derived: u64,
        authoritative: u64,
    },
    #[error("spatial observability totals do not reconcile with checkpoint summaries")]
    SummaryMismatch,
    #[error("unsupported retained resource-period schema {found}; supported schema is {supported}")]
    UnsupportedResourcePeriodSchema { found: u32, supported: u32 },
    #[error("spatial observability accounting overflow")]
    AccountingOverflow,
}
