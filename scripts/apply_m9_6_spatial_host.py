from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "crates/anthrosim-core/src/spatial_simulation.rs"
text = PATH.read_text()


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one occurrence, found {count}: {old[:80]!r}")
    text = text.replace(old, new, 1)


replace_once(
    "        RngCheckpoint, SimulationCheckpoint, state_digest64, state_digest64_with_temporary_mobility,\n",
    "        RngCheckpoint, SimulationCheckpoint, state_digest64_with_temporary_mobility,\n",
)
replace_once(
    "    spatial_mechanisms::{\n        SPATIAL_MODEL_SEMANTICS_ID, SpatialMechanismConfig, SpatialMechanismError,\n        transform_landscape,\n    },\n    time::{DAYS_PER_YEAR, SimTime},\n",
    "    spatial_mechanisms::{\n        SPATIAL_MODEL_SEMANTICS_ID, SpatialMechanismConfig, SpatialMechanismError,\n        transform_landscape,\n    },\n    temporary_mobility::{\n        TemporaryMobilityExecutionError, TemporaryMobilityProgram, TemporaryMobilityProgramError,\n        TemporaryMobilityState, TemporaryMobilityValidationError,\n    },\n    time::{DAYS_PER_YEAR, SimTime},\n",
)
replace_once(
    "    world: World,\n    population: Population,\n    resources: ResourceSystem,\n",
    "    world: World,\n    population: Population,\n    temporary_mobility: TemporaryMobilityState,\n    resources: ResourceSystem,\n",
)

start = text.index("impl SpatialLandscapeSimulation {\n    pub fn new(")
end = text.index("\n    pub fn from_checkpoint(", start)
constructor = '''impl SpatialLandscapeSimulation {
    pub fn new(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
    ) -> Result<Self, SpatialLandscapeError> {
        Self::new_internal(config, landscape, mechanisms, None)
    }

    /// Construct a transformed-landscape simulation with the same authoritative M9 temporary-
    /// mobility program supported by the core host.
    pub fn new_with_temporary_mobility(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
        program: TemporaryMobilityProgram,
    ) -> Result<Self, SpatialLandscapeError> {
        Self::new_internal(config, landscape, mechanisms, Some(program))
    }

    fn new_internal(
        config: ExperimentConfig,
        landscape: LandscapeBundle,
        mechanisms: SpatialMechanismConfig,
        program: Option<TemporaryMobilityProgram>,
    ) -> Result<Self, SpatialLandscapeError> {
        validate_experiment(&config)?;
        let landscape_binding = LandscapeBinding::from_bundle(&landscape)?;
        validate_grid_match(&config, &landscape_binding)?;
        if let Some(evidence) = &config.evidence {
            evidence.validate()?;
            landscape.validate_evidence_links(evidence)?;
        }

        let world = reconstruct_world(&config, &landscape, &mechanisms)?;
        let spatial_binding = SpatialMechanismBinding::new(mechanisms, &world)?;
        let rng_factory = RngFactory::new(config.seed);
        let population = Population::initialize(config.population, &world, rng_factory)?;
        let temporary_mobility = match program {
            Some(program) => TemporaryMobilityState::with_program(&population, program, &world)?,
            None => TemporaryMobilityState::at_residence(&population),
        };
        temporary_mobility.validate_at_day(0, &population, &world)?;
        let resources = ResourceSystem::initialize(&world, &config.resources)?;
        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;

        Ok(Self {
            config,
            time: SimTime::ZERO,
            terminal_stop_reason: None,
            resume_lineage: ResumeLineage::new(),
            landscape,
            landscape_binding,
            spatial_binding,
            world,
            population,
            temporary_mobility,
            resources,
            migration,
            demography_rngs: DemographyRngs::new(rng_factory),
            resource_rngs: ResourceRngs::new(rng_factory),
            migration_rngs: MigrationRngs::new(rng_factory),
            events: EventLog::new(),
            metrics: MetricSeries::annual(),
        })
    }
'''
text = text[:start] + constructor + text[end:]

replace_once(
    "            population: checkpoint.core_checkpoint.population,\n            resources: checkpoint.core_checkpoint.resources,\n",
    "            population: checkpoint.core_checkpoint.population,\n            temporary_mobility: checkpoint.core_checkpoint.temporary_mobility,\n            resources: checkpoint.core_checkpoint.resources,\n",
)
replace_once(
    "    pub const fn population(&self) -> &Population {\n        &self.population\n    }\n\n    #[must_use]\n    pub const fn resources(&self) -> &ResourceSystem {\n",
    "    pub const fn population(&self) -> &Population {\n        &self.population\n    }\n\n    #[must_use]\n    pub const fn temporary_mobility(&self) -> &TemporaryMobilityState {\n        &self.temporary_mobility\n    }\n\n    #[must_use]\n    pub const fn resources(&self) -> &ResourceSystem {\n",
)

advance_start = text.index("    fn advance_to_year(\n")
advance_end = text.index("\n    fn completed_years(&self)", advance_start)
advance = '''    fn advance_to_year(
        &mut self,
        target_year: u64,
    ) -> Result<Option<StopReason>, SpatialLandscapeError> {
        if let Some(stop_reason) = self.terminal_stop_reason {
            return Ok(Some(stop_reason));
        }
        let current_year = self.completed_years()?;
        if self.population.living_count() == 0 {
            self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
            self.record_metric_snapshot();
            return Ok(self.terminal_stop_reason);
        }

        for year in current_year.saturating_add(1)..=target_year {
            let periods = u64::from(self.config.resources.periods_per_year);
            let year_start_day = (year - 1).saturating_mul(DAYS_PER_YEAR);
            for period_index in 0..self.config.resources.periods_per_year {
                let period_number = u64::from(period_index) + 1;
                let day = year_start_day
                    .saturating_add(period_number.saturating_mul(DAYS_PER_YEAR) / periods);
                self.process_temporary_boundaries_before(day)?;
                self.time = SimTime::from_days(day);
                let temporary_resource_period = self
                    .temporary_mobility
                    .resource_period_snapshot(day, &self.world)?;
                let outcome = self.resources.process_period_recorded_with_presence(
                    &mut self.population,
                    &ResourcePeriodContext {
                        world: &self.world,
                        config: &self.config.resources,
                        period_index_in_year: period_index,
                        day,
                    },
                    &mut self.resource_rngs.scarcity_mortality,
                    &mut self.events,
                    temporary_resource_period.as_ref(),
                )?;
                self.temporary_mobility.complete_resource_period(day)?;
                self.temporary_mobility
                    .reconcile_after_population_change(&self.population);
                if outcome == ResourceStepOutcome::PopulationExtinct {
                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
                    self.record_metric_snapshot();
                    return Ok(self.terminal_stop_reason);
                }
                self.temporary_mobility.process_day(
                    day,
                    &self.population,
                    &self.world,
                    &mut self.events,
                )?;
                self.migration.process_boundary_recorded_with_presence(
                    &mut self.population,
                    &MigrationBoundaryContext {
                        world: &self.world,
                        resources: &self.resources,
                        migration: &self.config.migration,
                        annual_food_need: self.config.resources.annual_need_units_per_person,
                        resource_periods_per_year: self.config.resources.periods_per_year,
                        day,
                    },
                    &mut self.migration_rngs,
                    &mut self.events,
                    Some(&self.temporary_mobility),
                )?;
            }

            self.time = SimTime::from_years(year);
            let outcome = process_demographic_year_recorded(
                &mut self.population,
                &self.world,
                &self.config.demography,
                self.time.days(),
                &mut self.demography_rngs,
                &mut self.events,
            )?;
            self.temporary_mobility
                .reconcile_after_population_change(&self.population);
            self.record_metric_snapshot();
            match outcome {
                DemographyStepOutcome::Continue => {}
                DemographyStepOutcome::PopulationExtinct => {
                    self.terminal_stop_reason = Some(StopReason::PopulationExtinct);
                    return Ok(self.terminal_stop_reason);
                }
                DemographyStepOutcome::PersonRecordLimitReached => {
                    self.terminal_stop_reason = Some(StopReason::PersonRecordLimitReached);
                    return Ok(self.terminal_stop_reason);
                }
            }
        }
        Ok(None)
    }

    fn process_temporary_boundaries_before(
        &mut self,
        fixed_day: u64,
    ) -> Result<(), SpatialLandscapeError> {
        let Some(end_day) = fixed_day.checked_sub(1) else {
            return Ok(());
        };
        loop {
            let current_day = self.time.days();
            let Some(day) = self.temporary_mobility.next_boundary_day(
                current_day,
                end_day,
                &self.population,
            )?
            else {
                break;
            };
            self.time = SimTime::from_days(day);
            self.temporary_mobility.process_day(
                day,
                &self.population,
                &self.world,
                &mut self.events,
            )?;
        }
        Ok(())
    }
'''
text = text[:advance_start] + advance + text[advance_end:]

old_digest = '''    fn state_digest64(&self) -> u64 {
        state_digest64(
            self.time.days(),
            self.world.digest64(),
            self.population.digest64(),
            self.resources.digest64(),
            self.migration.digest64(),
        )
    }
'''
new_digest = '''    fn state_digest64(&self) -> u64 {
        state_digest64_with_temporary_mobility(
            self.time.days(),
            self.world.digest64(),
            self.population.digest64(),
            self.resources.digest64(),
            self.migration.digest64(),
            &self.temporary_mobility,
        )
    }
'''
replace_once(old_digest, new_digest)
replace_once(
    "        self.population\n            .validate(&self.world)\n            .map_err(PopulationError::from)?;\n        self.resources\n",
    "        self.population\n            .validate(&self.world)\n            .map_err(PopulationError::from)?;\n        self.temporary_mobility\n            .validate_at_day(self.time.days(), &self.population, &self.world)?;\n        self.resources\n",
)
replace_once(
    "        let temporary_mobility =\n            crate::temporary_mobility::TemporaryMobilityState::at_residence(&self.population);\n        SimulationCheckpoint {\n",
    "        SimulationCheckpoint {\n",
)
replace_once(
    "            population: self.population,\n            temporary_mobility,\n            resources: self.resources,\n",
    "            population: self.population,\n            temporary_mobility: self.temporary_mobility,\n            resources: self.resources,\n",
)
replace_once(
    "    if !checkpoint.temporary_mobility.is_disabled() {\n        return Err(SpatialLandscapeError::ActiveTemporaryMobilityUnsupported);\n    }\n    Ok(())\n",
    "    Ok(())\n",
)
replace_once(
    "    #[error(\"spatial checkpoint temporary mobility state is invalid: {reason}\")]\n    TemporaryMobilityInvalid { reason: String },\n    #[error(\"spatial landscape simulation does not support active temporary mobility\")]\n    ActiveTemporaryMobilityUnsupported,\n",
    "    #[error(\"spatial checkpoint temporary mobility state is invalid: {reason}\")]\n    TemporaryMobilityInvalid { reason: String },\n    #[error(transparent)]\n    TemporaryMobility(#[from] TemporaryMobilityValidationError),\n    #[error(transparent)]\n    TemporaryMobilityProgram(#[from] TemporaryMobilityProgramError),\n    #[error(transparent)]\n    TemporaryMobilityExecution(#[from] TemporaryMobilityExecutionError),\n",
)

PATH.write_text(text)

TEST = ROOT / "crates/anthrosim-core/tests/spatial_temporary_mobility.rs"
TEST.write_text(r'''use anthrosim_core::{
    CellId, DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    GridGeometry, HouseholdId, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig, ResourceConfig,
    SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField,
    TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryTravelResolution,
    TemporaryTravelTable, TemporaryTriggerTiming, TransformDirection, WorldConfig,
};

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values,
    }
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        4,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "cell".to_owned(),
            spatial_reference: "LOCAL_CS[generic]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(250), Some(500), Some(750)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(750), Some(500), Some(250)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(500), Some(500), Some(500), Some(500)],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "m9_6_spatial_host_fixture_v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                2_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
}

fn config() -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    ExperimentConfig::new(96_001, 2)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(24).with_target_household_size(4))
        .with_demography(demography)
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn program() -> TemporaryMobilityProgram {
    let source = landscape();
    let mechanisms = mechanisms();
    let baseline = SpatialLandscapeSimulation::new(config(), source, mechanisms)
        .expect("baseline spatial host");
    let household = HouseholdId::new(1);
    let residence = baseline
        .population()
        .household_location(household)
        .expect("household residence");
    let destination = (1..=baseline.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| *cell != residence)
        .expect("world has another cell");
    let region = FocalRegion::new(
        "generic-m9-6-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .expect("region");
    let resolutions = (1..=baseline.world().cell_count() as u64)
        .map(|_| TemporaryTravelResolution::Reachable {
            destination,
            outbound_travel_days: 10,
            return_travel_days: 10,
        })
        .collect();
    let travel = TemporaryTravelTable::new(resolutions, &region, baseline.world()).expect("travel");
    let schedule = TemporaryMobilitySchedule::new(
        "annual-boundary-active-journey",
        TemporaryTriggerTiming::DepartureDay,
        vec![360],
        5,
    )
    .expect("schedule");
    TemporaryMobilityProgram::new(region, schedule, travel, baseline.world()).expect("program")
}

#[test]
fn transformed_spatial_host_executes_and_resumes_active_temporary_journeys_exactly() {
    let source = landscape();
    let mechanisms = mechanisms();
    let program = program();

    let uninterrupted = SpatialLandscapeSimulation::new_with_temporary_mobility(
        config(),
        source.clone(),
        mechanisms.clone(),
        program.clone(),
    )
    .expect("temporary spatial host")
    .run_recorded()
    .expect("uninterrupted run");

    assert!(uninterrupted.events().events.iter().any(|record| matches!(
        record.event,
        EventKind::TemporaryJourneyDeparted { .. }
    )));

    let paused = SpatialLandscapeSimulation::new_with_temporary_mobility(
        config(),
        source.clone(),
        mechanisms,
        program,
    )
    .expect("temporary spatial host")
    .checkpoint_at_year(1)
    .expect("annual checkpoint");
    assert!(
        paused
            .core_checkpoint
            .temporary_mobility
            .active_journey(HouseholdId::new(1))
            .is_some(),
        "household 1 should be in outbound transit across the annual checkpoint"
    );

    let resumed = SpatialLandscapeSimulation::from_checkpoint(paused, source)
        .expect("resume active spatial journey")
        .run_recorded()
        .expect("resumed run");

    let expected = uninterrupted.core_checkpoint();
    let actual = resumed.core_checkpoint();
    assert_eq!(actual.state_digest64, expected.state_digest64);
    assert_eq!(actual.population, expected.population);
    assert_eq!(actual.temporary_mobility, expected.temporary_mobility);
    assert_eq!(actual.resources, expected.resources);
    assert_eq!(actual.migration, expected.migration);
    assert_eq!(actual.rng, expected.rng);
    assert_eq!(actual.events, expected.events);
    assert_eq!(actual.metrics, expected.metrics);
}
''')

print("patched spatial host and wrote M9.6 integration test")
