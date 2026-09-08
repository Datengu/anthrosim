use anthrosim_core::{
    DemographyConfig, ExperimentConfig, HouseholdLifecycleConfig, MigrationConfig, PopulationConfig,
    ResourceConfig, Simulation, WorldConfig, derive_household_observability,
    validate_recorded_run_invariants,
    config::ParameterProvenance,
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
};

const DAYS_PER_YEAR: i64 = 365;

fn founder(
    id: u64,
    age_years: i64,
    sex: ReproductiveSex,
    female_parent: Option<u64>,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -age_years * DAYS_PER_YEAR,
        reproductive_sex: sex,
        household: HouseholdId::new(1),
        female_parent: female_parent.map(PersonId::new),
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn no_background_mortality() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn no_resource_pressure() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

#[test]
fn parent_anchoring_cannot_leave_household_above_configured_living_member_cap() {
    let founders = FounderPopulationDefinition::new(
        "audit-v5-area-c-parent-anchored-cap",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            // Independent anchors sort oldest first. With max=2 these three anchors are assigned
            // round-robin as group 0, group 1, group 0, filling group 0 before the child is placed.
            founder(1, 50, ReproductiveSex::Female, None),
            founder(2, 40, ReproductiveSex::Male, None),
            founder(3, 30, ReproductiveSex::Male, None),
            // The only declared living parent is person 1 in the already-full group 0.
            founder(4, 10, ReproductiveSex::Female, Some(1)),
        ],
    );

    let config = ExperimentConfig::new(50_401, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(4).with_target_household_size(4))
        .with_founder_population(founders)
        .with_demography(no_background_mortality())
        .with_resources(no_resource_pressure())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();

    let report = derive_household_observability(
        &run.checkpoint.population,
        &run.checkpoint.experiment,
        &run.checkpoint.events,
        run.checkpoint.time.days(),
    )
    .unwrap();
    let sizes = report
        .living_household_size_distribution
        .iter()
        .map(|bin| (bin.living_members, bin.household_count))
        .collect::<Vec<_>>();

    eprintln!(
        "post-fission household sizes={sizes:?}; largest={}; configured cap=2",
        report.largest_living_household_size
    );

    assert!(
        report.largest_living_household_size <= 2,
        "dependency-aware fission left a household above maxLivingMembers=2 even though two independent-age groups were feasible: sizes={sizes:?}"
    );
}
