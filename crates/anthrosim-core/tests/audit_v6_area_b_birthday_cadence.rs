use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const ONE_MILLION: u32 = 1_000_000;
const FOCAL_FEMALE: PersonId = PersonId::new(1);
const CADENCES: [u16; 4] = [1, 4, 12, 365];
const STOCHASTIC_SEEDS: u64 = 4_096;
const MAX_EXPECTED_COUNT_DEVIATION: i64 = 160;
const MAX_PAIRWISE_COUNT_DIFFERENCE: i64 = 220;

fn demography(start_band_mortality: u32) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.mortality_bands = vec![
        AgeProbabilityBand::new(0, 30, 0),
        AgeProbabilityBand::new(30, 40, start_band_mortality),
        // The focal female turns 40 on day 165. The authoritative contract says this newly
        // entered band must not replace the interval-start band during the same model year.
        AgeProbabilityBand::new(40, u32::MAX, ONE_MILLION),
    ];
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 35, 0),
        AgeProbabilityBand::new(35, 40, ONE_MILLION),
        // The same birthday also crosses out of the certain-fertility band, but M2 fertility is
        // likewise selected from age at interval start.
        AgeProbabilityBand::new(40, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-b-birthday-cadence",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: FOCAL_FEMALE,
                // Age 39 years + 200 days at model day zero: the 40th birthday is day 165.
                birth_day: -((39 * 365 + 200) as i64),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(25 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FocalOutcome {
    female_died: bool,
    female_gave_birth: bool,
}

fn focal_outcome(seed: u64, periods_per_year: u16, start_band_mortality: u32) -> FocalOutcome {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = periods_per_year;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(4),
        )
        .with_founder_population(founders())
        .with_demography(demography(start_band_mortality))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();

    let female_died = run.events().events.iter().any(|record| {
        matches!(
            record.event,
            EventKind::Death { person, .. } if person == FOCAL_FEMALE
        )
    });
    let female_gave_birth = run.events().events.iter().any(|record| {
        matches!(
            record.event,
            EventKind::Birth { female_parent, .. } if female_parent == FOCAL_FEMALE
        )
    });

    assert_eq!(
        female_gave_birth,
        !female_died,
        "certain interval-start fertility with one immortal local male must produce a birth iff the focal female survives the model year"
    );

    FocalOutcome {
        female_died,
        female_gave_birth,
    }
}

#[test]
fn birthday_crossing_keeps_interval_start_demography_under_m3_cadence_refinement() {
    // Exact limiting case. The focal female starts in a zero-mortality, certain-fertility band,
    // then turns 40 on day 165 into certain mortality and zero fertility. No M3 cadence is allowed
    // to reinterpret that birthday as a mid-year switch of the annual M2 schedule.
    for periods_per_year in CADENCES {
        for seed in [1_u64, 2, 3, 17, 101] {
            let outcome = focal_outcome(seed, periods_per_year, 0);
            assert_eq!(
                outcome,
                FocalOutcome {
                    female_died: false,
                    female_gave_birth: true,
                },
                "periods_per_year={periods_per_year}, seed={seed}: a mid-year birthday must not replace the zero-mortality/certain-fertility interval-start bands"
            );
        }
    }

    // Non-trivial stochastic case. The focal female starts with exactly 0.5 annual background
    // mortality and certain fertility, while the age-40 band remains certain mortality/zero
    // fertility. Under the declared interval-start + partition contract, every cadence therefore
    // has P(survive and give birth) = 0.5. Same-seed outcomes are intentionally NOT required to
    // align across cadences because AnthroSim's sequential-stream contract does not promise
    // agent/event-level common random numbers after draw schedules differ. We test the marginal law.
    let mut birth_counts = Vec::new();
    for periods_per_year in CADENCES {
        let mut births = 0_i64;
        for seed in 0..STOCHASTIC_SEEDS {
            if focal_outcome(seed, periods_per_year, 500_000).female_gave_birth {
                births += 1;
            }
        }
        birth_counts.push((periods_per_year, births));
    }

    let expected = i64::try_from(STOCHASTIC_SEEDS / 2).unwrap();
    println!(
        "birthday-crossing survival-conditioned births over {STOCHASTIC_SEEDS} seeds: {birth_counts:?}; expected per cadence={expected}; per-cadence tolerance=±{MAX_EXPECTED_COUNT_DEVIATION}; pairwise tolerance={MAX_PAIRWISE_COUNT_DIFFERENCE}"
    );

    for &(periods_per_year, births) in &birth_counts {
        let deviation = (births - expected).abs();
        assert!(
            deviation <= MAX_EXPECTED_COUNT_DEVIATION,
            "periods_per_year={periods_per_year}: birth count {births}/{STOCHASTIC_SEEDS} deviates by {deviation} from the exact 0.5 interval-start survival/fertility law; tolerance={MAX_EXPECTED_COUNT_DEVIATION}"
        );
    }

    for left in 0..birth_counts.len() {
        for right in (left + 1)..birth_counts.len() {
            let difference = (birth_counts[left].1 - birth_counts[right].1).abs();
            assert!(
                difference <= MAX_PAIRWISE_COUNT_DIFFERENCE,
                "M3 cadence materially changed the birthday-crossing birth/survival law: {:?} vs {:?}, absolute count difference={difference}, tolerance={MAX_PAIRWISE_COUNT_DIFFERENCE}",
                birth_counts[left],
                birth_counts[right]
            );
        }
    }
}
