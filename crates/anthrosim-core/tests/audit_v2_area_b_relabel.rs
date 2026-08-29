use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
};
use anthrosim_core::ids::{CellId, HouseholdId, PersonId};

const DAYS_PER_YEAR_I64: i64 = 365;

fn founder_definition(swapped: bool) -> FounderPopulationDefinition {
    let young = FounderPerson {
        id: PersonId::new(if swapped { 2 } else { 1 }),
        birth_day: -(20 * DAYS_PER_YEAR_I64),
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    };
    let old = FounderPerson {
        id: PersonId::new(if swapped { 1 } else { 2 }),
        birth_day: -(40 * DAYS_PER_YEAR_I64),
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    };
    let people = if swapped { vec![old, young] } else { vec![young, old] };
    FounderPopulationDefinition::new(
        if swapped { "audit-v2-relabel-b" } else { "audit-v2-relabel-a" },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold { id: HouseholdId::new(1), location: CellId::new(1) }],
        people,
    )
}

fn config(seed: u64, swapped: bool) -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    demography.schedule_id = "audit-v2-relabel-probe".to_owned();
    demography.mortality_bands = vec![
        AgeProbabilityBand::new(0, 30, 200_000),
        AgeProbabilityBand::new(30, u32::MAX, 800_000),
    ];
    demography.fertility_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(2))
        .with_founder_population(founder_definition(swapped))
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn final_living(seed: u64, swapped: bool) -> u64 {
    Simulation::new(config(seed, swapped))
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .population
        .living_population
}

#[test]
fn audit_probe_consistent_person_id_relabelling_changes_demographic_aggregate_for_some_seeds() {
    let mut differing_seeds = Vec::new();
    for seed in 1..=2_000_u64 {
        let a = final_living(seed, false);
        let b = final_living(seed, true);
        if a != b {
            differing_seeds.push((seed, a, b));
        }
    }
    eprintln!("audit-v2 Area B relabel probe: differing_seeds={} of 2000; first={:?}", differing_seeds.len(), differing_seeds.first());
    assert!(
        !differing_seeds.is_empty(),
        "expected at least one seed where pure PersonId relabelling changes final living population"
    );
}
