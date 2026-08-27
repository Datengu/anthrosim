use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::MigrationConfig,
    events::{EventKind, EventLog},
    ids::{CellId, HouseholdId, PersonId},
    population::{Population, PopulationError},
    resources::{
        ResourceSystem, fixed_annual_quantity_at_resource_boundary,
        fixed_annual_quantity_for_period,
    },
    rng::{RngFactory, RngStreamPosition},
    temporary_mobility::{HouseholdPresence, TemporaryMobilityState},
    world::{BASE_MOVEMENT_COST, PERMILLE_MAX, World},
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MAX_KIN_LOCATIONS_PER_HOUSEHOLD: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationUtilityBreakdown {
    pub resource_score_permille: u16,
    pub water_security_score_permille: u16,
    pub kin_score_permille: u16,
    pub travel_penalty_permille: u16,
    pub uncertainty_penalty_permille: u16,
    pub relocation_risk_penalty_permille: u16,
    pub total_utility: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationDecisionTrace {
    pub decision_day: u64,
    pub completed_day: u64,
    pub household: HouseholdId,
    pub people_moved: u32,
    pub origin: CellId,
    pub destination: CellId,
    pub distance_cells: u16,
    pub pressure_permille: u16,
    pub candidate_count: u16,
    pub origin_utility: MigrationUtilityBreakdown,
    pub destination_utility: MigrationUtilityBreakdown,
    pub best_candidate: CellId,
    pub best_candidate_utility: i32,
    pub selected_weight: u64,
    pub total_move_weight: u64,
    pub choice_draw: u64,
    pub travel_condition_cost_per_person: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationSummary {
    pub schema_version: u32,
    pub model_id: String,
    pub decision_boundaries: u64,
    pub households_evaluated: u64,
    pub households_under_pressure: u64,
    pub moves_completed: u64,
    pub people_moved: u64,
    pub total_distance_cells: u64,
    pub northward_steps: u64,
    pub eastward_steps: u64,
    pub southward_steps: u64,
    pub westward_steps: u64,
    pub travel_condition_cost_total: u64,
    /// Mean origin resource score across completed moves, or `None` when no moves occurred.
    pub mean_origin_resource_score_permille: Option<u16>,
    /// Mean destination resource score across completed moves, or `None` when no moves occurred.
    pub mean_destination_resource_score_permille: Option<u16>,
    /// Mean origin water-security score across completed moves, or `None` when no moves occurred.
    pub mean_origin_water_security_score_permille: Option<u16>,
    /// Mean destination water-security score across completed moves, or `None` when no moves occurred.
    pub mean_destination_water_security_score_permille: Option<u16>,
    pub occupied_cell_delta_from_migration: i64,
    pub recorded_decision_traces: Vec<MigrationDecisionTrace>,
    pub digest64: u64,
}

impl MigrationSummary {
    /// v2 represents move-conditional means as null when the move observation set is empty.
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationCheckpointState {
    pub schema_version: u32,
    pub model_id: String,
    pub decision_boundaries: u64,
    pub households_evaluated: u64,
    pub households_under_pressure: u64,
    pub moves_completed: u64,
    pub people_moved: u64,
    pub total_distance_cells: u64,
    pub northward_steps: u64,
    pub eastward_steps: u64,
    pub southward_steps: u64,
    pub westward_steps: u64,
    pub travel_condition_cost_total: u64,
    pub origin_resource_score_total: u64,
    pub destination_resource_score_total: u64,
    pub origin_water_security_score_total: u64,
    pub destination_water_security_score_total: u64,
    pub occupied_cell_delta_from_migration: i64,
    pub recorded_decision_traces: Vec<MigrationDecisionTrace>,
}

impl MigrationCheckpointState {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, Copy)]
struct CandidateEvaluation {
    cell: CellId,
    distance: u16,
    utility: MigrationUtilityBreakdown,
    weight: u64,
}

#[derive(Debug, Clone, Copy)]
struct ResidenceUtilityTerms {
    resource_score_permille: u16,
    water_security_score_permille: u16,
    kin_score_permille: u16,
}

#[derive(Debug, Clone, Copy, Default)]
struct RelocationActionCosts {
    travel_penalty_permille: u16,
    uncertainty_penalty_permille: u16,
    relocation_risk_penalty_permille: u16,
}

pub(crate) struct MigrationBoundaryContext<'a> {
    pub world: &'a World,
    pub resources: &'a ResourceSystem,
    pub migration: &'a MigrationConfig,
    pub annual_food_need: u32,
    pub decision_periods_per_year: u16,
    pub decision_index_in_year: u16,
    pub day: u64,
}

#[derive(Debug)]
pub struct MigrationRngs {
    choice: ChaCha8Rng,
    uncertainty: ChaCha8Rng,
}

impl MigrationRngs {
    #[must_use]
    pub(crate) fn new(factory: RngFactory) -> Self {
        Self {
            choice: factory.stream("migration/choice"),
            uncertainty: factory.stream("migration/uncertainty"),
        }
    }

    pub(crate) fn positions(&self) -> [RngStreamPosition; 2] {
        [
            RngStreamPosition::capture(&self.choice),
            RngStreamPosition::capture(&self.uncertainty),
        ]
    }

    pub(crate) fn restore_positions(&mut self, positions: [RngStreamPosition; 2]) {
        positions[0].restore(&mut self.choice);
        positions[1].restore(&mut self.uncertainty);
    }
}

/// M4 local migration state and compact explanatory metrics.
///
/// Decisions are evaluated in stable household-ID order against one shared
/// pre-move snapshot. Selected household moves are then applied simultaneously
/// in a packed population pass. This prevents earlier households in a boundary
/// from changing the information seen by later households.
#[derive(Debug)]
pub struct MigrationSystem {
    model_id: String,
    decision_boundaries: u64,
    households_evaluated: u64,
    households_under_pressure: u64,
    moves_completed: u64,
    people_moved: u64,
    total_distance_cells: u64,
    northward_steps: u64,
    eastward_steps: u64,
    southward_steps: u64,
    westward_steps: u64,
    travel_condition_cost_total: u64,
    origin_resource_score_total: u64,
    destination_resource_score_total: u64,
    origin_water_security_score_total: u64,
    destination_water_security_score_total: u64,
    occupied_cell_delta_from_migration: i64,
    recorded_decision_traces: Vec<MigrationDecisionTrace>,
    living_members: Vec<u32>,
    condition_sums: Vec<u64>,
    /// Living population by persistent residence, used only for permanent-occupancy accounting.
    cell_living: Vec<u32>,
    /// Living population provisioned from each cell at the current M4 decision boundary.
    ///
    /// With M9 disabled this equals `cell_living`. With M9 enabled, visitors are attributed to
    /// their destination while outbound/return transit remains provisioned from persistent home,
    /// matching the M9.5 resource-accounting rule without inventing a physical transit cell.
    boundary_demand_living: Vec<u32>,
    post_move_cell_living: Vec<u32>,
    kin_locations: Vec<[CellId; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]>,
    kin_location_counts: Vec<u8>,
    planned_destinations: Vec<CellId>,
    planned_condition_costs: Vec<u16>,
    candidates: Vec<CellId>,
    evaluations: Vec<CandidateEvaluation>,
}

impl MigrationSystem {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn initialize(
        population: &Population,
        world: &World,
        config: &MigrationConfig,
    ) -> Result<Self, MigrationConfigError> {
        validate_migration_config(config)?;
        let households = population.household_count();
        let cells = world.cell_count();
        let candidate_capacity = candidate_count_upper_bound(config.candidate_radius_cells);
        Ok(Self {
            model_id: config.model_id.clone(),
            decision_boundaries: 0,
            households_evaluated: 0,
            households_under_pressure: 0,
            moves_completed: 0,
            people_moved: 0,
            total_distance_cells: 0,
            northward_steps: 0,
            eastward_steps: 0,
            southward_steps: 0,
            westward_steps: 0,
            travel_condition_cost_total: 0,
            origin_resource_score_total: 0,
            destination_resource_score_total: 0,
            origin_water_security_score_total: 0,
            destination_water_security_score_total: 0,
            occupied_cell_delta_from_migration: 0,
            recorded_decision_traces: Vec::with_capacity(
                usize::try_from(config.max_recorded_decision_traces).unwrap_or(0),
            ),
            living_members: vec![0; households],
            condition_sums: vec![0; households],
            cell_living: vec![0; cells],
            boundary_demand_living: vec![0; cells],
            post_move_cell_living: vec![0; cells],
            kin_locations: vec![[CellId::INVALID; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]; households],
            kin_location_counts: vec![0; households],
            planned_destinations: vec![CellId::INVALID; households],
            planned_condition_costs: vec![0; households],
            candidates: Vec::with_capacity(candidate_capacity),
            evaluations: Vec::with_capacity(candidate_capacity),
        })
    }

    #[cfg(test)]
    pub(crate) fn process_boundary(
        &mut self,
        population: &mut Population,
        context: &MigrationBoundaryContext<'_>,
        rngs: &mut MigrationRngs,
    ) -> Result<(), MigrationError> {
        let mut events = EventLog::new();
        self.process_boundary_recorded(population, context, rngs, &mut events)
    }

    #[cfg(test)]
    pub(crate) fn process_boundary_recorded(
        &mut self,
        population: &mut Population,
        context: &MigrationBoundaryContext<'_>,
        rngs: &mut MigrationRngs,
        events: &mut EventLog,
    ) -> Result<(), MigrationError> {
        self.process_boundary_recorded_with_presence(population, context, rngs, events, None)
    }

    pub(crate) fn process_boundary_recorded_with_presence(
        &mut self,
        population: &mut Population,
        context: &MigrationBoundaryContext<'_>,
        rngs: &mut MigrationRngs,
        events: &mut EventLog,
        temporary_mobility: Option<&TemporaryMobilityState>,
    ) -> Result<(), MigrationError> {
        let MigrationBoundaryContext {
            world,
            resources,
            migration: config,
            annual_food_need,
            decision_periods_per_year,
            decision_index_in_year,
            day,
        } = *context;
        validate_migration_config(config)?;
        if !config.enabled || population.living_count() == 0 {
            return Ok(());
        }
        if decision_periods_per_year != config.decision_periods_per_year {
            return Err(MigrationError::InternalInvariant(
                "migration decision schedule does not match configuration",
            ));
        }
        self.prepare_snapshot(population, world, temporary_mobility)?;
        self.decision_boundaries = self
            .decision_boundaries
            .checked_add(1)
            .ok_or(MigrationError::AccountingOverflow)?;

        let period_need_per_person = fixed_annual_quantity_for_period(
            u64::from(annual_food_need),
            decision_index_in_year,
            decision_periods_per_year,
        )
        .ok_or(MigrationError::InternalInvariant(
            "migration decision interval could not be allocated",
        ))?;
        let boundary_need_per_person = fixed_annual_quantity_at_resource_boundary(
            u64::from(annual_food_need),
            decision_periods_per_year,
            day,
        )
        .ok_or(MigrationError::InternalInvariant(
            "migration decision day does not align with its declared schedule",
        ))?;
        if boundary_need_per_person != period_need_per_person {
            return Err(MigrationError::InternalInvariant(
                "migration decision index and boundary day disagree",
            ));
        }

        for household_index in 0..population.household_count() {
            let members = self.living_members[household_index];
            if members == 0 {
                continue;
            }
            let household = HouseholdId::new(household_index as u64 + 1);
            if let Some(temporary_mobility) = temporary_mobility {
                match temporary_mobility.is_at_residence(household) {
                    Some(true) => {}
                    Some(false) => continue,
                    None => {
                        return Err(MigrationError::InternalInvariant(
                            "temporary mobility is missing household state",
                        ));
                    }
                }
            }
            self.households_evaluated = self
                .households_evaluated
                .checked_add(1)
                .ok_or(MigrationError::AccountingOverflow)?;
            let origin = population.household_location(household).ok_or(
                MigrationError::InternalInvariant("household has no location"),
            )?;
            let mean_condition =
                u16::try_from(self.condition_sums[household_index] / u64::from(members))
                    .unwrap_or(PERMILLE_MAX);
            let origin_demand_population = self.boundary_demand_population(origin)?;
            let origin_utility = self.evaluate_stay(
                household_index,
                origin,
                origin_demand_population,
                resources,
                world,
                config,
                period_need_per_person,
            )?;
            let pressure = migration_pressure_permille(
                mean_condition,
                origin_utility.resource_score_permille,
                config,
            );
            if pressure == 0 {
                continue;
            }
            self.households_under_pressure = self
                .households_under_pressure
                .checked_add(1)
                .ok_or(MigrationError::AccountingOverflow)?;

            fill_candidate_cells(
                &mut self.candidates,
                world,
                origin,
                config.candidate_radius_cells,
            );
            self.evaluations.clear();
            let mut total_weight = 0_u64;
            let mut best_candidate = CellId::INVALID;
            let mut best_candidate_utility = i32::MIN;

            for &candidate in &self.candidates {
                let distance = manhattan_distance(world, origin, candidate).ok_or(
                    MigrationError::InternalInvariant("candidate coordinates invalid"),
                )?;
                let destination_demand_population = self
                    .boundary_demand_population(candidate)?
                    .saturating_add(members);
                let uncertainty = if config.max_uncertainty_penalty_permille == 0 {
                    0
                } else {
                    u16::try_from(draw_bounded(
                        &mut rngs.uncertainty,
                        u64::from(config.max_uncertainty_penalty_permille) + 1,
                    ))
                    .unwrap_or(config.max_uncertainty_penalty_permille)
                };
                let utility = self.evaluate_relocation(
                    household_index,
                    candidate,
                    distance,
                    destination_demand_population,
                    resources,
                    world,
                    config,
                    period_need_per_person,
                    uncertainty,
                )?;
                if utility.total_utility > best_candidate_utility
                    || (utility.total_utility == best_candidate_utility
                        && (best_candidate == CellId::INVALID || candidate < best_candidate))
                {
                    best_candidate = candidate;
                    best_candidate_utility = utility.total_utility;
                }
                let required = origin_utility.total_utility.saturating_add(
                    i32::try_from(config.minimum_utility_improvement).unwrap_or(i32::MAX),
                );
                if utility.total_utility <= required {
                    continue;
                }
                let improvement = i64::from(utility.total_utility) - i64::from(required);
                let weight = u64::try_from(improvement)
                    .unwrap_or(u64::MAX)
                    .saturating_add(1);
                total_weight = total_weight
                    .checked_add(weight)
                    .ok_or(MigrationError::AccountingOverflow)?;
                self.evaluations.push(CandidateEvaluation {
                    cell: candidate,
                    distance,
                    utility,
                    weight,
                });
            }

            if total_weight == 0 {
                continue;
            }
            let choice_draw = draw_bounded(&mut rngs.choice, total_weight);
            let mut cursor = choice_draw;
            let mut selected =
                *self
                    .evaluations
                    .last()
                    .ok_or(MigrationError::InternalInvariant(
                        "positive weight has no candidates",
                    ))?;
            for evaluation in &self.evaluations {
                if cursor < evaluation.weight {
                    selected = *evaluation;
                    break;
                }
                cursor -= evaluation.weight;
            }

            let condition_cost = u16::try_from(
                u32::from(config.travel_condition_cost_per_cell)
                    .saturating_mul(u32::from(selected.distance))
                    .min(u32::from(PERMILLE_MAX)),
            )
            .unwrap_or(PERMILLE_MAX);
            self.planned_destinations[household_index] = selected.cell;
            self.planned_condition_costs[household_index] = condition_cost;
            self.record_selected_move(
                population,
                world,
                config,
                day,
                household_index,
                household,
                members,
                origin,
                pressure,
                origin_utility,
                selected,
                best_candidate,
                best_candidate_utility,
                total_weight,
                choice_draw,
                condition_cost,
                events,
            )?;
        }

        self.apply_planned_moves(population, world)?;
        Ok(())
    }

    fn prepare_snapshot(
        &mut self,
        population: &Population,
        world: &World,
        temporary_mobility: Option<&TemporaryMobilityState>,
    ) -> Result<(), MigrationError> {
        if self.living_members.len() != population.household_count()
            || self.cell_living.len() != world.cell_count()
            || self.boundary_demand_living.len() != world.cell_count()
        {
            return Err(MigrationError::StateShapeMismatch);
        }
        self.living_members.fill(0);
        self.condition_sums.fill(0);
        self.cell_living.fill(0);
        self.boundary_demand_living.fill(0);
        self.post_move_cell_living.fill(0);
        self.kin_locations
            .fill([CellId::INVALID; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]);
        self.kin_location_counts.fill(0);
        self.planned_destinations.fill(CellId::INVALID);
        self.planned_condition_costs.fill(0);

        for person_index in 0..population.person_count() {
            if !population.is_alive_index(person_index) {
                continue;
            }
            let household = population.household_at_index(person_index).ok_or(
                MigrationError::InternalInvariant("living person has no household"),
            )?;
            let household_index = household_index(household, population.household_count())?;
            let location = population.location_at_index(person_index).ok_or(
                MigrationError::InternalInvariant("living person has no location"),
            )?;
            let residence_cell_index = cell_index(location, world.cell_count())?;
            let demand_location = match temporary_mobility {
                None => location,
                Some(state) => match state.presence(household) {
                    Some(HouseholdPresence::AtResidence)
                    | Some(HouseholdPresence::OutboundTransit { .. })
                    | Some(HouseholdPresence::ReturnTransit { .. }) => location,
                    Some(HouseholdPresence::Visiting { destination, .. }) => destination,
                    None => {
                        return Err(MigrationError::InternalInvariant(
                            "temporary mobility is missing household state",
                        ));
                    }
                },
            };
            let demand_cell_index = cell_index(demand_location, world.cell_count())?;
            let condition = population.condition_at_index(person_index).ok_or(
                MigrationError::InternalInvariant("living person has no condition"),
            )?;
            self.living_members[household_index] = self.living_members[household_index]
                .checked_add(1)
                .ok_or(MigrationError::AccountingOverflow)?;
            self.condition_sums[household_index] = self.condition_sums[household_index]
                .checked_add(u64::from(condition))
                .ok_or(MigrationError::AccountingOverflow)?;
            self.cell_living[residence_cell_index] = self.cell_living[residence_cell_index]
                .checked_add(1)
                .ok_or(MigrationError::AccountingOverflow)?;
            self.boundary_demand_living[demand_cell_index] = self.boundary_demand_living
                [demand_cell_index]
                .checked_add(1)
                .ok_or(MigrationError::AccountingOverflow)?;

            for parent in [
                population.female_parent_at_index(person_index),
                population.male_parent_at_index(person_index),
            ]
            .into_iter()
            .flatten()
            {
                self.note_external_kin_location(population, household_index, household, parent)?;
            }
        }
        Ok(())
    }

    fn note_external_kin_location(
        &mut self,
        population: &Population,
        household_index: usize,
        household: HouseholdId,
        parent: PersonId,
    ) -> Result<(), MigrationError> {
        if parent == PersonId::INVALID {
            return Ok(());
        }
        let Some(parent_snapshot) = population.person(parent) else {
            return Ok(());
        };
        if !parent_snapshot.is_alive() || parent_snapshot.household == household {
            return Ok(());
        }
        let count = usize::from(self.kin_location_counts[household_index]);
        let locations = &mut self.kin_locations[household_index];
        if locations[..count].contains(&parent_snapshot.location) {
            return Ok(());
        }
        if count < MAX_KIN_LOCATIONS_PER_HOUSEHOLD {
            locations[count] = parent_snapshot.location;
            self.kin_location_counts[household_index] =
                self.kin_location_counts[household_index].saturating_add(1);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_stay(
        &self,
        household_index: usize,
        cell: CellId,
        local_demand_population: u32,
        resources: &ResourceSystem,
        world: &World,
        config: &MigrationConfig,
        period_need_per_person: u64,
    ) -> Result<MigrationUtilityBreakdown, MigrationError> {
        let residence = self.evaluate_residence_terms(
            household_index,
            cell,
            local_demand_population,
            resources,
            world,
            period_need_per_person,
        )?;
        Ok(compose_utility(
            residence,
            config,
            RelocationActionCosts::default(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_relocation(
        &self,
        household_index: usize,
        cell: CellId,
        distance: u16,
        destination_demand_population: u32,
        resources: &ResourceSystem,
        world: &World,
        config: &MigrationConfig,
        period_need_per_person: u64,
        uncertainty_penalty: u16,
    ) -> Result<MigrationUtilityBreakdown, MigrationError> {
        let residence = self.evaluate_residence_terms(
            household_index,
            cell,
            destination_demand_population,
            resources,
            world,
            period_need_per_person,
        )?;
        let world_cell = world
            .cell(cell)
            .ok_or(MigrationError::InternalInvariant("candidate outside world"))?;
        let terrain_excess = world_cell.movement_cost.saturating_sub(BASE_MOVEMENT_COST);
        let travel_penalty = u16::try_from(
            (u32::from(distance).saturating_mul(120) + u32::from(terrain_excess) / 3)
                .min(u32::from(PERMILLE_MAX)),
        )
        .unwrap_or(PERMILLE_MAX);
        let relocation_risk = u16::try_from(
            (u32::from(config.relocation_risk_base_penalty_permille)
                + u32::from(config.relocation_risk_per_cell_permille)
                    .saturating_mul(u32::from(distance)))
            .min(u32::from(PERMILLE_MAX)),
        )
        .unwrap_or(PERMILLE_MAX);

        Ok(compose_utility(
            residence,
            config,
            RelocationActionCosts {
                travel_penalty_permille: travel_penalty,
                uncertainty_penalty_permille: uncertainty_penalty,
                relocation_risk_penalty_permille: relocation_risk,
            },
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_residence_terms(
        &self,
        household_index: usize,
        cell: CellId,
        local_demand_population: u32,
        resources: &ResourceSystem,
        world: &World,
        period_need_per_person: u64,
    ) -> Result<ResidenceUtilityTerms, MigrationError> {
        let world_cell = world
            .cell(cell)
            .ok_or(MigrationError::InternalInvariant("candidate outside world"))?;
        let stock = resources
            .cell_food_stock(cell)
            .ok_or(MigrationError::InternalInvariant(
                "resource cell outside world",
            ))?;
        let demand = period_need_per_person.saturating_mul(u64::from(local_demand_population));
        let resource_score = u16::try_from(
            stock
                .saturating_mul(u64::from(PERMILLE_MAX))
                .checked_div(demand)
                .unwrap_or(u64::from(PERMILLE_MAX)),
        )
        .unwrap_or(PERMILLE_MAX)
        .min(PERMILLE_MAX);
        let water_security_score = u16::try_from(
            (u32::from(world_cell.water_access) * 3
                + u32::from(PERMILLE_MAX - world_cell.environmental_stress))
                / 4,
        )
        .unwrap_or(PERMILLE_MAX);
        let kin_count = usize::from(self.kin_location_counts[household_index]);
        let kin_matches = self.kin_locations[household_index][..kin_count]
            .iter()
            .filter(|&&kin_cell| kin_cell == cell)
            .count();
        let kin_score = u16::try_from(kin_matches.saturating_mul(250))
            .unwrap_or(PERMILLE_MAX)
            .min(PERMILLE_MAX);

        Ok(ResidenceUtilityTerms {
            resource_score_permille: resource_score,
            water_security_score_permille: water_security_score,
            kin_score_permille: kin_score,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn record_selected_move(
        &mut self,
        population: &Population,
        world: &World,
        config: &MigrationConfig,
        day: u64,
        household_index: usize,
        household: HouseholdId,
        members: u32,
        origin: CellId,
        pressure: u16,
        origin_utility: MigrationUtilityBreakdown,
        selected: CandidateEvaluation,
        best_candidate: CellId,
        best_candidate_utility: i32,
        total_weight: u64,
        choice_draw: u64,
        condition_cost: u16,
        events: &mut EventLog,
    ) -> Result<(), MigrationError> {
        self.moves_completed = self
            .moves_completed
            .checked_add(1)
            .ok_or(MigrationError::AccountingOverflow)?;
        self.people_moved = self
            .people_moved
            .checked_add(u64::from(members))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.total_distance_cells = self
            .total_distance_cells
            .checked_add(u64::from(selected.distance))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.origin_resource_score_total = self
            .origin_resource_score_total
            .checked_add(u64::from(origin_utility.resource_score_permille))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.destination_resource_score_total = self
            .destination_resource_score_total
            .checked_add(u64::from(selected.utility.resource_score_permille))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.origin_water_security_score_total = self
            .origin_water_security_score_total
            .checked_add(u64::from(origin_utility.water_security_score_permille))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.destination_water_security_score_total = self
            .destination_water_security_score_total
            .checked_add(u64::from(selected.utility.water_security_score_permille))
            .ok_or(MigrationError::AccountingOverflow)?;
        let (origin_x, origin_y) =
            world
                .coordinates(origin)
                .ok_or(MigrationError::InternalInvariant(
                    "origin coordinates invalid",
                ))?;
        let (destination_x, destination_y) =
            world
                .coordinates(selected.cell)
                .ok_or(MigrationError::InternalInvariant(
                    "destination coordinates invalid",
                ))?;
        self.eastward_steps = self
            .eastward_steps
            .checked_add(u64::from(destination_x.saturating_sub(origin_x)))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.westward_steps = self
            .westward_steps
            .checked_add(u64::from(origin_x.saturating_sub(destination_x)))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.southward_steps = self
            .southward_steps
            .checked_add(u64::from(destination_y.saturating_sub(origin_y)))
            .ok_or(MigrationError::AccountingOverflow)?;
        self.northward_steps = self
            .northward_steps
            .checked_add(u64::from(origin_y.saturating_sub(destination_y)))
            .ok_or(MigrationError::AccountingOverflow)?;

        if self.recorded_decision_traces.len()
            < usize::try_from(config.max_recorded_decision_traces).unwrap_or(usize::MAX)
        {
            self.recorded_decision_traces.push(MigrationDecisionTrace {
                decision_day: day,
                completed_day: day,
                household,
                people_moved: members,
                origin,
                destination: selected.cell,
                distance_cells: selected.distance,
                pressure_permille: pressure,
                candidate_count: u16::try_from(self.candidates.len()).unwrap_or(u16::MAX),
                origin_utility,
                destination_utility: selected.utility,
                best_candidate,
                best_candidate_utility,
                selected_weight: selected.weight,
                total_move_weight: total_weight,
                choice_draw,
                travel_condition_cost_per_person: condition_cost,
            });
        }
        events.push_authoritative(
            day,
            EventKind::HouseholdMigration {
                household,
                people_moved: members,
                origin,
                destination: selected.cell,
                distance_cells: selected.distance,
                pressure_permille: pressure,
                origin_utility,
                destination_utility: selected.utility,
                best_candidate,
                best_candidate_utility,
                selected_weight: selected.weight,
                total_move_weight: total_weight,
                choice_draw,
                travel_condition_cost_per_person: condition_cost,
            },
        );
        let _ = household_index;
        let _ = population;
        Ok(())
    }

    fn apply_planned_moves(
        &mut self,
        population: &mut Population,
        world: &World,
    ) -> Result<(), MigrationError> {
        self.post_move_cell_living
            .copy_from_slice(&self.cell_living);
        for household_index in 0..population.household_count() {
            let destination = self.planned_destinations[household_index];
            if destination == CellId::INVALID {
                continue;
            }
            let household = HouseholdId::new(household_index as u64 + 1);
            let origin = population.household_location(household).ok_or(
                MigrationError::InternalInvariant("household has no location"),
            )?;
            let members = self.living_members[household_index];
            let origin_index = cell_index(origin, world.cell_count())?;
            let destination_index = cell_index(destination, world.cell_count())?;
            self.post_move_cell_living[origin_index] = self.post_move_cell_living[origin_index]
                .checked_sub(members)
                .ok_or(MigrationError::InternalInvariant(
                    "planned move exceeds origin population",
                ))?;
            self.post_move_cell_living[destination_index] = self.post_move_cell_living
                [destination_index]
                .checked_add(members)
                .ok_or(MigrationError::AccountingOverflow)?;
        }
        let before = self.cell_living.iter().filter(|&&count| count > 0).count() as i64;
        let after = self
            .post_move_cell_living
            .iter()
            .filter(|&&count| count > 0)
            .count() as i64;
        self.occupied_cell_delta_from_migration = self
            .occupied_cell_delta_from_migration
            .checked_add(after - before)
            .ok_or(MigrationError::AccountingOverflow)?;

        let relocation = population.apply_household_relocations(
            &self.planned_destinations,
            &self.planned_condition_costs,
            world,
        )?;
        self.travel_condition_cost_total = self
            .travel_condition_cost_total
            .checked_add(relocation.condition_loss_total)
            .ok_or(MigrationError::AccountingOverflow)?;
        if relocation.people_moved
            != self
                .planned_destinations
                .iter()
                .enumerate()
                .filter(|(_, destination)| **destination != CellId::INVALID)
                .map(|(index, _)| u64::from(self.living_members[index]))
                .sum::<u64>()
        {
            return Err(MigrationError::InternalInvariant(
                "relocation people count did not reconcile",
            ));
        }
        Ok(())
    }

    fn boundary_demand_population(&self, cell: CellId) -> Result<u32, MigrationError> {
        let index = cell_index(cell, self.boundary_demand_living.len())?;
        Ok(self.boundary_demand_living[index])
    }

    #[must_use]
    pub fn summary(&self) -> MigrationSummary {
        let moves = self.moves_completed;
        MigrationSummary {
            schema_version: MigrationSummary::CURRENT_SCHEMA_VERSION,
            model_id: self.model_id.clone(),
            decision_boundaries: self.decision_boundaries,
            households_evaluated: self.households_evaluated,
            households_under_pressure: self.households_under_pressure,
            moves_completed: moves,
            people_moved: self.people_moved,
            total_distance_cells: self.total_distance_cells,
            northward_steps: self.northward_steps,
            eastward_steps: self.eastward_steps,
            southward_steps: self.southward_steps,
            westward_steps: self.westward_steps,
            travel_condition_cost_total: self.travel_condition_cost_total,
            mean_origin_resource_score_permille: mean_score(
                self.origin_resource_score_total,
                moves,
            ),
            mean_destination_resource_score_permille: mean_score(
                self.destination_resource_score_total,
                moves,
            ),
            mean_origin_water_security_score_permille: mean_score(
                self.origin_water_security_score_total,
                moves,
            ),
            mean_destination_water_security_score_permille: mean_score(
                self.destination_water_security_score_total,
                moves,
            ),
            occupied_cell_delta_from_migration: self.occupied_cell_delta_from_migration,
            recorded_decision_traces: self.recorded_decision_traces.clone(),
            digest64: self.digest64(),
        }
    }

    pub(crate) fn checkpoint_state(&self) -> MigrationCheckpointState {
        MigrationCheckpointState {
            schema_version: MigrationCheckpointState::CURRENT_SCHEMA_VERSION,
            model_id: self.model_id.clone(),
            decision_boundaries: self.decision_boundaries,
            households_evaluated: self.households_evaluated,
            households_under_pressure: self.households_under_pressure,
            moves_completed: self.moves_completed,
            people_moved: self.people_moved,
            total_distance_cells: self.total_distance_cells,
            northward_steps: self.northward_steps,
            eastward_steps: self.eastward_steps,
            southward_steps: self.southward_steps,
            westward_steps: self.westward_steps,
            travel_condition_cost_total: self.travel_condition_cost_total,
            origin_resource_score_total: self.origin_resource_score_total,
            destination_resource_score_total: self.destination_resource_score_total,
            origin_water_security_score_total: self.origin_water_security_score_total,
            destination_water_security_score_total: self.destination_water_security_score_total,
            occupied_cell_delta_from_migration: self.occupied_cell_delta_from_migration,
            recorded_decision_traces: self.recorded_decision_traces.clone(),
        }
    }

    pub(crate) fn from_checkpoint_state(
        population: &Population,
        world: &World,
        config: &MigrationConfig,
        state: MigrationCheckpointState,
    ) -> Result<Self, MigrationError> {
        if state.schema_version != MigrationCheckpointState::CURRENT_SCHEMA_VERSION
            || state.model_id != config.model_id
        {
            return Err(MigrationError::CheckpointStateMismatch);
        }
        let mut system = Self::initialize(population, world, config)?;
        system.decision_boundaries = state.decision_boundaries;
        system.households_evaluated = state.households_evaluated;
        system.households_under_pressure = state.households_under_pressure;
        system.moves_completed = state.moves_completed;
        system.people_moved = state.people_moved;
        system.total_distance_cells = state.total_distance_cells;
        system.northward_steps = state.northward_steps;
        system.eastward_steps = state.eastward_steps;
        system.southward_steps = state.southward_steps;
        system.westward_steps = state.westward_steps;
        system.travel_condition_cost_total = state.travel_condition_cost_total;
        system.origin_resource_score_total = state.origin_resource_score_total;
        system.destination_resource_score_total = state.destination_resource_score_total;
        system.origin_water_security_score_total = state.origin_water_security_score_total;
        system.destination_water_security_score_total =
            state.destination_water_security_score_total;
        system.occupied_cell_delta_from_migration = state.occupied_cell_delta_from_migration;
        system.recorded_decision_traces = state.recorded_decision_traces;
        Ok(system)
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, self.decision_boundaries);
        digest_u64(&mut hash, self.households_evaluated);
        digest_u64(&mut hash, self.households_under_pressure);
        digest_u64(&mut hash, self.moves_completed);
        digest_u64(&mut hash, self.people_moved);
        digest_u64(&mut hash, self.total_distance_cells);
        digest_u64(&mut hash, self.travel_condition_cost_total);
        digest_u64(&mut hash, self.occupied_cell_delta_from_migration as u64);
        for trace in &self.recorded_decision_traces {
            digest_u64(&mut hash, trace.decision_day);
            digest_u64(&mut hash, trace.household.0);
            digest_u64(&mut hash, trace.origin.0);
            digest_u64(&mut hash, trace.destination.0);
            digest_u64(&mut hash, u64::from(trace.distance_cells));
            digest_u64(&mut hash, trace.choice_draw);
        }
        hash
    }
}

fn compose_utility(
    residence: ResidenceUtilityTerms,
    config: &MigrationConfig,
    action_costs: RelocationActionCosts,
) -> MigrationUtilityBreakdown {
    let positive = i64::from(residence.resource_score_permille) * i64::from(config.resource_weight)
        + i64::from(residence.water_security_score_permille)
            * i64::from(config.water_security_weight)
        + i64::from(residence.kin_score_permille) * i64::from(config.kin_weight);
    let negative = i64::from(action_costs.travel_penalty_permille)
        * i64::from(config.travel_cost_weight)
        + i64::from(action_costs.uncertainty_penalty_permille)
        + i64::from(action_costs.relocation_risk_penalty_permille);
    let total_utility = i32::try_from(positive.saturating_sub(negative)).unwrap_or({
        if positive >= negative {
            i32::MAX
        } else {
            i32::MIN
        }
    });

    MigrationUtilityBreakdown {
        resource_score_permille: residence.resource_score_permille,
        water_security_score_permille: residence.water_security_score_permille,
        kin_score_permille: residence.kin_score_permille,
        travel_penalty_permille: action_costs.travel_penalty_permille,
        uncertainty_penalty_permille: action_costs.uncertainty_penalty_permille,
        relocation_risk_penalty_permille: action_costs.relocation_risk_penalty_permille,
        total_utility,
    }
}

#[must_use]
pub fn migration_pressure_permille(
    mean_condition_permille: u16,
    local_resource_score_permille: u16,
    config: &MigrationConfig,
) -> u16 {
    let condition_pressure = config
        .condition_pressure_threshold_permille
        .saturating_sub(mean_condition_permille);
    let resource_pressure = config
        .resource_pressure_threshold_permille
        .saturating_sub(local_resource_score_permille);
    condition_pressure
        .saturating_add(resource_pressure)
        .min(PERMILLE_MAX)
}

#[must_use]
pub fn bounded_candidate_cells(world: &World, origin: CellId, radius: u16) -> Vec<CellId> {
    let mut cells = Vec::with_capacity(candidate_count_upper_bound(radius));
    fill_candidate_cells(&mut cells, world, origin, radius);
    cells
}

#[must_use]
pub const fn candidate_count_upper_bound(radius: u16) -> usize {
    let radius = radius as usize;
    2 * radius * (radius + 1)
}

fn fill_candidate_cells(cells: &mut Vec<CellId>, world: &World, origin: CellId, radius: u16) {
    cells.clear();
    let Some((origin_x, origin_y)) = world.coordinates(origin) else {
        return;
    };
    let radius = i64::from(radius);
    for dy in -radius..=radius {
        let remaining = radius - dy.abs();
        for dx in -remaining..=remaining {
            if dx == 0 && dy == 0 {
                continue;
            }
            let x = i64::from(origin_x) + dx;
            let y = i64::from(origin_y) + dy;
            let Ok(x) = u32::try_from(x) else {
                continue;
            };
            let Ok(y) = u32::try_from(y) else {
                continue;
            };
            if let Some(cell) = world.cell_id(x, y) {
                cells.push(cell);
            }
        }
    }
}

fn manhattan_distance(world: &World, a: CellId, b: CellId) -> Option<u16> {
    let (ax, ay) = world.coordinates(a)?;
    let (bx, by) = world.coordinates(b)?;
    u16::try_from(ax.abs_diff(bx).saturating_add(ay.abs_diff(by))).ok()
}

fn cell_index(cell: CellId, cell_count: usize) -> Result<usize, MigrationError> {
    let index = usize::try_from(
        cell.0
            .checked_sub(1)
            .ok_or(MigrationError::InternalInvariant("invalid cell ID"))?,
    )
    .map_err(|_| MigrationError::InternalInvariant("cell index does not fit usize"))?;
    if index >= cell_count {
        return Err(MigrationError::InternalInvariant("cell is outside world"));
    }
    Ok(index)
}

fn household_index(
    household: HouseholdId,
    household_count: usize,
) -> Result<usize, MigrationError> {
    let index = usize::try_from(
        household
            .0
            .checked_sub(1)
            .ok_or(MigrationError::InternalInvariant("invalid household ID"))?,
    )
    .map_err(|_| MigrationError::InternalInvariant("household index does not fit usize"))?;
    if index >= household_count {
        return Err(MigrationError::InternalInvariant(
            "household is outside population",
        ));
    }
    Ok(index)
}

fn draw_bounded<R: Rng + ?Sized>(rng: &mut R, upper_exclusive: u64) -> u64 {
    debug_assert!(upper_exclusive > 0);
    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);
    loop {
        let draw = rng.next_u64();
        if draw < acceptance_limit {
            return draw % upper_exclusive;
        }
    }
}

fn mean_score(total: u64, count: u64) -> Option<u16> {
    let mean = total.checked_div(count)?;
    Some(u16::try_from(mean).unwrap_or(PERMILLE_MAX))
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

pub fn validate_migration_config(config: &MigrationConfig) -> Result<(), MigrationConfigError> {
    if config.schema_version != MigrationConfig::CURRENT_SCHEMA_VERSION {
        return Err(MigrationConfigError::UnsupportedSchema {
            found: config.schema_version,
            supported: MigrationConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.model_id.trim().is_empty() {
        return Err(MigrationConfigError::EmptyModelId);
    }
    if config.decision_periods_per_year == 0 || config.decision_periods_per_year > 365 {
        return Err(MigrationConfigError::InvalidDecisionPeriodsPerYear {
            value: config.decision_periods_per_year,
        });
    }
    if config.candidate_radius_cells == 0 || config.candidate_radius_cells > 32 {
        return Err(MigrationConfigError::InvalidCandidateRadius {
            value: config.candidate_radius_cells,
        });
    }
    for (field, value) in [
        (
            "condition_pressure_threshold_permille",
            config.condition_pressure_threshold_permille,
        ),
        (
            "resource_pressure_threshold_permille",
            config.resource_pressure_threshold_permille,
        ),
        (
            "max_uncertainty_penalty_permille",
            config.max_uncertainty_penalty_permille,
        ),
        (
            "relocation_risk_base_penalty_permille",
            config.relocation_risk_base_penalty_permille,
        ),
        (
            "relocation_risk_per_cell_permille",
            config.relocation_risk_per_cell_permille,
        ),
        (
            "travel_condition_cost_per_cell",
            config.travel_condition_cost_per_cell,
        ),
    ] {
        if value > PERMILLE_MAX {
            return Err(MigrationConfigError::PermilleOutOfRange { field, value });
        }
    }
    if config.resource_weight == 0 && config.water_security_weight == 0 && config.kin_weight == 0 {
        return Err(MigrationConfigError::NoPositiveUtilityWeights);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum MigrationConfigError {
    #[error("migration schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("migration model ID must not be empty")]
    EmptyModelId,
    #[error("migration decision periods per year must be in 1..=365, found {value}")]
    InvalidDecisionPeriodsPerYear { value: u16 },
    #[error("candidate radius must be in 1..=32 cells, found {value}")]
    InvalidCandidateRadius { value: u16 },
    #[error("migration permille field {field} is out of range: {value}")]
    PermilleOutOfRange { field: &'static str, value: u16 },
    #[error("migration utility must have at least one positive attraction weight")]
    NoPositiveUtilityWeights,
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error(transparent)]
    Config(#[from] MigrationConfigError),
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error("migration accounting overflow")]
    AccountingOverflow,
    #[error("migration state shape does not match population/world")]
    StateShapeMismatch,
    #[error("migration checkpoint state is incompatible with this model/config")]
    CheckpointStateMismatch,
    #[error("migration internal invariant failed: {0}")]
    InternalInvariant(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{ExperimentConfig, PopulationConfig, ResourceConfig, WorldConfig},
        focal_region::{FocalRegion, FocalRegionSource},
        population::Population,
        resources::ResourceSystem,
        rng::RngFactory,
        temporary_mobility::{
            TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryTravelResolution,
            TemporaryTravelTable, TemporaryTriggerTiming,
        },
    };

    #[test]
    fn candidate_lookup_is_bounded_and_local() {
        let factory = RngFactory::new(1);
        let world = World::generate(WorldConfig::new(9, 9), factory).unwrap();
        let origin = world.cell_id(4, 4).unwrap();
        let candidates = bounded_candidate_cells(&world, origin, 3);
        assert_eq!(candidates.len(), candidate_count_upper_bound(3));
        assert!(candidates.iter().all(|&cell| {
            manhattan_distance(&world, origin, cell).is_some_and(|distance| distance <= 3)
        }));
        let corner = world.cell_id(0, 0).unwrap();
        assert!(bounded_candidate_cells(&world, corner, 3).len() < candidates.len());
    }

    #[test]
    fn local_deterioration_increases_pressure() {
        let config = MigrationConfig::synthetic_validation_v1();
        let healthy = migration_pressure_permille(1_000, 1_000, &config);
        let deteriorated = migration_pressure_permille(600, 300, &config);
        assert_eq!(healthy, 0);
        assert!(deteriorated > healthy);
    }

    #[test]
    fn migration_score_means_distinguish_undefined_from_true_zero() {
        assert_eq!(mean_score(0, 0), None);
        assert_eq!(mean_score(0, 1), Some(0));
        assert_eq!(mean_score(750, 1), Some(750));

        let factory = RngFactory::new(42);
        let world = World::generate(WorldConfig::new(4, 4), factory).unwrap();
        let population =
            Population::initialize(PopulationConfig::new(20), &world, factory).unwrap();
        let migration = MigrationSystem::initialize(
            &population,
            &world,
            &MigrationConfig::synthetic_validation_v1(),
        )
        .unwrap();
        let summary = migration.summary();
        assert_eq!(summary.moves_completed, 0);
        assert_eq!(summary.mean_origin_resource_score_permille, None);
        assert_eq!(summary.mean_destination_resource_score_permille, None);
        assert_eq!(summary.mean_origin_water_security_score_permille, None);
        assert_eq!(summary.mean_destination_water_security_score_permille, None);
        let json = serde_json::to_value(summary).unwrap();
        assert_eq!(
            json["meanOriginResourceScorePermille"],
            serde_json::Value::Null
        );
        assert_eq!(
            json["meanDestinationResourceScorePermille"],
            serde_json::Value::Null
        );
        assert_eq!(
            json["meanOriginWaterSecurityScorePermille"],
            serde_json::Value::Null
        );
        assert_eq!(
            json["meanDestinationWaterSecurityScorePermille"],
            serde_json::Value::Null
        );
    }

    fn two_household_fixture() -> (RngFactory, World, Population, CellId, CellId, CellId) {
        let factory = RngFactory::new(196_001);
        let world = World::generate(WorldConfig::new(3, 1), factory).unwrap();
        let mut population = Population::initialize(
            PopulationConfig::new(4).with_target_household_size(2),
            &world,
            factory,
        )
        .unwrap();
        assert_eq!(population.household_count(), 2);
        let mover_home = world.cell_id(0, 0).unwrap();
        let destination = world.cell_id(1, 0).unwrap();
        let visitor_home = world.cell_id(2, 0).unwrap();
        population
            .apply_household_relocations(&[mover_home, visitor_home], &[0, 0], &world)
            .unwrap();
        (
            factory,
            world,
            population,
            mover_home,
            destination,
            visitor_home,
        )
    }

    fn temporary_visitor_state(
        population: &Population,
        world: &World,
        destination: CellId,
        visitor_home: CellId,
        outbound_travel_days: u32,
    ) -> TemporaryMobilityState {
        let mut resolutions = vec![TemporaryTravelResolution::Unreachable; world.cell_count()];
        resolutions[cell_index(visitor_home, world.cell_count()).unwrap()] =
            TemporaryTravelResolution::Reachable {
                destination,
                outbound_travel_days,
                return_travel_days: outbound_travel_days,
            };
        let region = FocalRegion::new(
            "m4-m9-demand-fixture",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap();
        let travel = TemporaryTravelTable::new(resolutions, &region, world).unwrap();
        let schedule = TemporaryMobilitySchedule::new(
            "m4-m9-demand-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![91],
            5,
        )
        .unwrap();
        let program = TemporaryMobilityProgram::new(region, schedule, travel, world).unwrap();
        let mut state = TemporaryMobilityState::with_program(population, program, world).unwrap();
        state
            .process_day(91, population, world, &mut EventLog::new())
            .unwrap();
        state
    }

    #[test]
    fn disabled_m9_preserves_persistent_m4_demand_snapshot() {
        let (_, world, population, _, _, _) = two_household_fixture();
        let config = MigrationConfig::synthetic_validation_v1();
        let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();

        migration
            .prepare_snapshot(&population, &world, None)
            .unwrap();
        let baseline_residence = migration.cell_living.clone();
        let baseline_demand = migration.boundary_demand_living.clone();
        assert_eq!(baseline_demand, baseline_residence);

        let disabled = TemporaryMobilityState::at_residence(&population);
        migration
            .prepare_snapshot(&population, &world, Some(&disabled))
            .unwrap();
        assert_eq!(migration.cell_living, baseline_residence);
        assert_eq!(migration.boundary_demand_living, baseline_demand);
    }

    #[test]
    fn same_day_m9_arrival_moves_m4_demand_to_visitor_destination() {
        let (_, world, population, _, destination, visitor_home) = two_household_fixture();
        let config = MigrationConfig::synthetic_validation_v1();
        let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();
        migration
            .prepare_snapshot(&population, &world, None)
            .unwrap();
        let resident_counts = migration.cell_living.clone();
        let baseline_demand = migration.boundary_demand_living.clone();
        let visitor_members = migration.living_members[1];

        let visiting = temporary_visitor_state(&population, &world, destination, visitor_home, 0);
        assert!(matches!(
            visiting.presence(HouseholdId::new(2)),
            Some(HouseholdPresence::Visiting { destination: found, .. }) if found == destination
        ));
        migration
            .prepare_snapshot(&population, &world, Some(&visiting))
            .unwrap();

        let destination_index = cell_index(destination, world.cell_count()).unwrap();
        let visitor_home_index = cell_index(visitor_home, world.cell_count()).unwrap();
        assert_eq!(migration.cell_living, resident_counts);
        assert_eq!(
            migration.boundary_demand_living[destination_index],
            baseline_demand[destination_index] + visitor_members
        );
        assert_eq!(
            migration.boundary_demand_living[visitor_home_index],
            baseline_demand[visitor_home_index] - visitor_members
        );
    }

    #[test]
    fn m9_transit_remains_home_provisioned_for_m4_resource_demand() {
        let (_, world, population, _, destination, visitor_home) = two_household_fixture();
        let config = MigrationConfig::synthetic_validation_v1();
        let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();
        migration
            .prepare_snapshot(&population, &world, None)
            .unwrap();
        let baseline_demand = migration.boundary_demand_living.clone();

        let transit = temporary_visitor_state(&population, &world, destination, visitor_home, 2);
        assert!(matches!(
            transit.presence(HouseholdId::new(2)),
            Some(HouseholdPresence::OutboundTransit { destination: found, .. }) if found == destination
        ));
        migration
            .prepare_snapshot(&population, &world, Some(&transit))
            .unwrap();
        assert_eq!(migration.boundary_demand_living, baseline_demand);
    }

    #[test]
    fn visitor_crowding_reduces_m4_candidate_resource_utility() {
        let (_, world, population, mover_home, destination, visitor_home) = two_household_fixture();
        let mut config = MigrationConfig::synthetic_validation_v1();
        config.resource_weight = 1;
        config.water_security_weight = 0;
        config.kin_weight = 0;
        config.travel_cost_weight = 0;
        config.max_uncertainty_penalty_permille = 0;
        config.relocation_risk_base_penalty_permille = 0;
        config.relocation_risk_per_cell_permille = 0;

        let resources =
            ResourceSystem::initialize(&world, &ResourceConfig::synthetic_validation_v1()).unwrap();
        let stock = resources.cell_food_stock(destination).unwrap();
        assert!(stock > 0);
        let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();
        let mover_members = 2_u32;
        let period_need_per_person = stock
            .checked_div(u64::from(mover_members))
            .unwrap_or(0)
            .max(1);
        let distance = manhattan_distance(&world, mover_home, destination).unwrap();

        migration
            .prepare_snapshot(&population, &world, None)
            .unwrap();
        let baseline_population = migration
            .boundary_demand_population(destination)
            .unwrap()
            .saturating_add(mover_members);
        let baseline = migration
            .evaluate_relocation(
                0,
                destination,
                distance,
                baseline_population,
                &resources,
                &world,
                &config,
                period_need_per_person,
                0,
            )
            .unwrap();

        let visiting = temporary_visitor_state(&population, &world, destination, visitor_home, 0);
        migration
            .prepare_snapshot(&population, &world, Some(&visiting))
            .unwrap();
        let visitor_aware_population = migration
            .boundary_demand_population(destination)
            .unwrap()
            .saturating_add(mover_members);
        let visitor_aware = migration
            .evaluate_relocation(
                0,
                destination,
                distance,
                visitor_aware_population,
                &resources,
                &world,
                &config,
                period_need_per_person,
                0,
            )
            .unwrap();

        assert!(visitor_aware_population > baseline_population);
        assert!(visitor_aware.resource_score_permille < baseline.resource_score_permille);
        assert!(visitor_aware.total_utility < baseline.total_utility);
    }

    #[test]
    fn same_seed_and_state_produce_identical_moves() {
        let experiment = ExperimentConfig::new(44, 3)
            .with_world(WorldConfig::new(16, 16))
            .with_population(PopulationConfig::new(500))
            .with_resources(
                ResourceConfig::synthetic_validation_v1().with_productivity_scale_permille(250),
            );
        let run = || {
            let factory = RngFactory::new(experiment.seed);
            let world = World::generate(experiment.world, factory).unwrap();
            let mut population =
                Population::initialize(experiment.population, &world, factory).unwrap();
            for index in 0..population.person_count() {
                if population.is_alive_index(index) {
                    assert!(population.set_condition_at_index(index, 500));
                }
            }
            let resources = ResourceSystem::initialize(&world, &experiment.resources).unwrap();
            let mut migration =
                MigrationSystem::initialize(&population, &world, &experiment.migration).unwrap();
            let mut rngs = MigrationRngs::new(factory);
            migration
                .process_boundary(
                    &mut population,
                    &MigrationBoundaryContext {
                        world: &world,
                        resources: &resources,
                        migration: &experiment.migration,
                        annual_food_need: experiment.resources.annual_need_units_per_person,
                        decision_periods_per_year: experiment.migration.decision_periods_per_year,
                        decision_index_in_year: 0,
                        day: 91,
                    },
                    &mut rngs,
                )
                .unwrap();
            (population.digest64(), migration.summary())
        };
        assert_eq!(run(), run());
    }
}
