from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:160]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


# Split the competing-risk resolver so each cause stream can assign latent triggers in its own
# scientific entity order before the existing exchange-symmetric tie rule consumes extra draws.
mortality = Path("crates/anthrosim-core/src/mortality.rs")
text = mortality.read_text(encoding="utf-8")
old = '''pub(crate) fn resolve_two_cause_competing_mortality(
    condition_probability: ProbabilityFraction,
    background_probability: ProbabilityFraction,
    condition_rng: &mut ChaCha8Rng,
    background_rng: &mut ChaCha8Rng,
) -> Result<Option<CompetingMortalityCause>, MortalityMathError> {
    let condition_trigger = draw_probability_fraction(condition_rng, condition_probability)?;
    let background_trigger = draw_probability_fraction(background_rng, background_probability)?;

    match (condition_trigger, background_trigger) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(CompetingMortalityCause::ConditionMediated)),
        (false, true) => Ok(Some(CompetingMortalityCause::Background)),
        (true, true) => {
            let condition_weight = u64::from(probability_fraction_per_million_ceil(
                condition_probability,
            )?);
            let background_weight = u64::from(probability_fraction_per_million_ceil(
                background_probability,
            )?);
            let total_weight = condition_weight
                .checked_add(background_weight)
                .ok_or(MortalityMathError::ArithmeticOverflow)?;
            if total_weight == 0 {
                return Err(MortalityMathError::ZeroDenominator);
            }
            let draw = draw_symmetric_bounded(condition_rng, background_rng, total_weight);
            if draw < condition_weight {
                Ok(Some(CompetingMortalityCause::ConditionMediated))
            } else {
                Ok(Some(CompetingMortalityCause::Background))
            }
        }
    }
}
'''
new = '''pub(crate) fn resolve_two_cause_competing_mortality(
    condition_probability: ProbabilityFraction,
    background_probability: ProbabilityFraction,
    condition_rng: &mut ChaCha8Rng,
    background_rng: &mut ChaCha8Rng,
) -> Result<Option<CompetingMortalityCause>, MortalityMathError> {
    let condition_trigger = draw_probability_fraction(condition_rng, condition_probability)?;
    let background_trigger = draw_probability_fraction(background_rng, background_probability)?;
    resolve_two_cause_competing_mortality_from_triggers(
        condition_trigger,
        background_trigger,
        condition_probability,
        background_probability,
        condition_rng,
        background_rng,
    )
}

/// Resolve already-sampled independent cause triggers using the same symmetric tie rule as the
/// ordinary two-cause resolver. Splitting trigger assignment from tie attribution lets each named
/// cause stream couple its latent trigger to the scientifically appropriate entity order without
/// introducing first-called cause priority.
pub(crate) fn resolve_two_cause_competing_mortality_from_triggers(
    condition_trigger: bool,
    background_trigger: bool,
    condition_probability: ProbabilityFraction,
    background_probability: ProbabilityFraction,
    condition_rng: &mut ChaCha8Rng,
    background_rng: &mut ChaCha8Rng,
) -> Result<Option<CompetingMortalityCause>, MortalityMathError> {
    match (condition_trigger, background_trigger) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(CompetingMortalityCause::ConditionMediated)),
        (false, true) => Ok(Some(CompetingMortalityCause::Background)),
        (true, true) => {
            let condition_weight = u64::from(probability_fraction_per_million_ceil(
                condition_probability,
            )?);
            let background_weight = u64::from(probability_fraction_per_million_ceil(
                background_probability,
            )?);
            let total_weight = condition_weight
                .checked_add(background_weight)
                .ok_or(MortalityMathError::ArithmeticOverflow)?;
            if total_weight == 0 {
                return Err(MortalityMathError::ZeroDenominator);
            }
            let draw = draw_symmetric_bounded(condition_rng, background_rng, total_weight);
            if draw < condition_weight {
                Ok(Some(CompetingMortalityCause::ConditionMediated))
            } else {
                Ok(Some(CompetingMortalityCause::Background))
            }
        }
    }
}
'''
if old not in text:
    raise SystemExit("mortality resolver source did not match expected v26 text")
mortality.write_text(text.replace(old, new, 1), encoding="utf-8")

# Authoritative M3/M2 mortality becomes a two-pass trigger schedule. Condition triggers retain the
# existing record order so AV4-006 stays independently open; background triggers and tie-only extra
# draws use the persistent person-level scientific coupling rank introduced by v26.
resources = Path("crates/anthrosim-core/src/resources.rs")
text = resources.read_text(encoding="utf-8")
text = text.replace(
    "        probability_fraction_per_million_ceil, resolve_two_cause_competing_mortality,\n",
    "        probability_fraction_per_million_ceil, resolve_two_cause_competing_mortality,\n"
    "        resolve_two_cause_competing_mortality_from_triggers,\n",
    1,
)
start = text.index("        let condition_after_resource_response = condition_distribution(population)?;\n")
end_marker = "        let condition_after_mortality = condition_distribution(population)?;\n"
end = text.index(end_marker, start) + len(end_marker)
replacement = '''        let condition_after_resource_response = condition_distribution(population)?;

        #[derive(Clone, Copy)]
        struct MortalityCandidate {
            person_index: usize,
            stochastic_coupling_rank: u64,
            condition: u16,
            condition_probability: ProbabilityFraction,
            condition_probability_per_million: u32,
            background_probability: ProbabilityFraction,
            background_probability_per_million: u32,
        }

        let people_at_mortality_boundary = population.person_count();
        let year_start_day =
            day.checked_sub(period_end)
                .ok_or(ResourceError::InternalInvariant(
                    "resource mortality boundary precedes its model-year interval",
                ))?;
        let mut mortality_candidates = Vec::new();
        for person_index in 0..people_at_mortality_boundary {
            if !population.is_alive_index(person_index) {
                continue;
            }
            let condition = population.condition_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no condition state"),
            )?;
            let deficit = u64::from(PERMILLE_MAX - condition);
            let reference_probability = u32::try_from(
                deficit * u64::from(config.max_scarcity_mortality_probability_per_million)
                    / u64::from(PERMILLE_MAX),
            )
            .map_err(|_| ResourceError::AccountingOverflow)?;
            let condition_probability = reference_quarter_probability_for_interval(
                reference_probability,
                period_start,
                period_end,
            )?;
            let condition_probability_per_million =
                probability_fraction_per_million_ceil(condition_probability)?;
            let stochastic_coupling_rank = population
                .stochastic_coupling_rank_at_index(person_index)
                .ok_or(ResourceError::InternalInvariant(
                    "living person has no stochastic coupling identity",
                ))?;

            let (background_probability, background_probability_per_million) =
                if let Some(background) = background_mortality.as_ref() {
                    let age_days = population
                        .age_days_at_index(person_index, year_start_day)
                        .ok_or(ResourceError::InternalInvariant(
                            "living person has no representable age at demographic year start",
                        ))?;
                    let annual_background_probability =
                        annual_probability_for_age(&background.config.mortality_bands, age_days);
                    let background_probability = annual_probability_for_interval(
                        annual_background_probability,
                        period_start,
                        period_end,
                    )?;
                    let background_probability_per_million =
                        probability_fraction_per_million_ceil(background_probability)?;
                    (background_probability, background_probability_per_million)
                } else {
                    (ProbabilityFraction::ZERO, 0)
                };

            mortality_candidates.push(MortalityCandidate {
                person_index,
                stochastic_coupling_rank,
                condition,
                condition_probability,
                condition_probability_per_million,
                background_probability,
                background_probability_per_million,
            });
        }

        let mut resolved_causes = vec![None; mortality_candidates.len()];
        if let Some(background) = background_mortality.as_mut() {
            let mut condition_triggers = Vec::with_capacity(mortality_candidates.len());
            for candidate in &mortality_candidates {
                condition_triggers.push(draw_probability_fraction(
                    scarcity_rng,
                    candidate.condition_probability,
                )?);
            }

            let mut background_order = (0..mortality_candidates.len()).collect::<Vec<_>>();
            background_order.sort_unstable_by_key(|&candidate_index| {
                let candidate = mortality_candidates[candidate_index];
                (candidate.stochastic_coupling_rank, candidate.person_index)
            });
            let mut background_triggers = vec![false; mortality_candidates.len()];
            for &candidate_index in &background_order {
                background_triggers[candidate_index] = draw_probability_fraction(
                    &mut *background.mortality_rng,
                    mortality_candidates[candidate_index].background_probability,
                )?;
            }

            for &candidate_index in &background_order {
                let candidate = mortality_candidates[candidate_index];
                resolved_causes[candidate_index] =
                    resolve_two_cause_competing_mortality_from_triggers(
                        condition_triggers[candidate_index],
                        background_triggers[candidate_index],
                        candidate.condition_probability,
                        candidate.background_probability,
                        scarcity_rng,
                        &mut *background.mortality_rng,
                    )?;
            }
        } else {
            for (candidate_index, candidate) in mortality_candidates.iter().enumerate() {
                resolved_causes[candidate_index] = draw_probability_fraction(
                    scarcity_rng,
                    candidate.condition_probability,
                )?
                .then_some(CompetingMortalityCause::ConditionMediated);
            }
        }

        for (candidate, resolved_cause) in mortality_candidates
            .iter()
            .zip(resolved_causes.into_iter())
        {
            let Some(resolved_cause) = resolved_cause else {
                continue;
            };
            let person_index = candidate.person_index;
            let person = population.person_id_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no stable ID"),
            )?;
            let household = population.household_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no household"),
            )?;
            let cell = population.location_at_index(person_index).ok_or(
                ResourceError::InternalInvariant("living person has no location"),
            )?;
            if population.mark_death(person_index, day) {
                let (cause, probability_per_million) = match resolved_cause {
                    CompetingMortalityCause::ConditionMediated => {
                        self.scarcity_deaths = self
                            .scarcity_deaths
                            .checked_add(1)
                            .ok_or(ResourceError::AccountingOverflow)?;
                        (
                            DeathCause::ResourceScarcity,
                            candidate.condition_probability_per_million,
                        )
                    }
                    CompetingMortalityCause::Background => (
                        DeathCause::DemographicMortality,
                        candidate.background_probability_per_million,
                    ),
                };
                events.push_authoritative(
                    day,
                    EventKind::Death {
                        person,
                        household,
                        cell,
                        cause,
                        condition_permille: candidate.condition,
                        probability_per_million,
                    },
                );
            }
        }

        let condition_after_mortality = condition_distribution(population)?;
'''
resources.write_text(text[:start] + replacement + text[end:], encoding="utf-8")

# Align the test-only standalone annual background path with the same background coupling order.
demography = Path("crates/anthrosim-core/src/demography.rs")
text = demography.read_text(encoding="utf-8")
old = '''    if apply_background_mortality {
        for index in 0..records_at_boundary_start {
            if !population.is_alive_index(index) {
                continue;
            }
            let age_days = population
'''
new = '''    if apply_background_mortality {
        let mut mortality_order = Vec::new();
        for index in 0..records_at_boundary_start {
            if !population.is_alive_index(index) {
                continue;
            }
            let stochastic_coupling_rank = population
                .stochastic_coupling_rank_at_index(index)
                .ok_or(PopulationError::InternalInvariant {
                    reason: "living person is missing stochastic coupling identity at mortality boundary",
                })?;
            mortality_order.push((stochastic_coupling_rank, index));
        }
        mortality_order.sort_unstable();
        for (_, index) in mortality_order {
            let age_days = population
'''
if old not in text:
    raise SystemExit("demography mortality loop did not match expected v26 text")
demography.write_text(text.replace(old, new, 1), encoding="utf-8")

replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    '''/// v26 persists a person-level stochastic coupling rank canonicalized from represented day-zero
/// scientific state, then consumes annual fertility draws in that rank order instead of canonical
/// packed-record/PersonId order. A v25 checkpoint must therefore not continue under v26 with the
/// same RNG positions while silently changing which represented person/household receives a
/// subsequent fertility realization.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v26";
''',
    '''/// v26 persists a person-level stochastic coupling rank canonicalized from represented day-zero
/// scientific state, then consumes annual fertility draws in that rank order instead of canonical
/// packed-record/PersonId order. A v25 checkpoint must therefore not continue under v26 with the
/// same RNG positions while silently changing which represented person/household receives a
/// subsequent fertility realization.
///
/// v27 extends that scientific coupling identity to the background-demographic side of M3/M2
/// competing mortality. Background latent triggers are assigned in coupling-rank order, while the
/// independently tracked condition-mediated stream remains on its pre-existing ordering pending
/// its own Audit-v4 finding. Simultaneous triggers retain symmetric proportional cause attribution.
/// A v26 checkpoint must therefore not resume under v27 with unchanged mortality RNG positions.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v27";
''',
)
replace_once(
    "crates/anthrosim-core/src/checkpoint.rs",
    '''    pub const PRE_STOCHASTIC_COUPLING_SCHEMA_VERSION: u32 = 13;
    pub const CURRENT_SCHEMA_VERSION: u32 = 14;
''',
    '''    pub const PRE_STOCHASTIC_COUPLING_SCHEMA_VERSION: u32 = 13;
    pub const PRE_BACKGROUND_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 14;
    pub const CURRENT_SCHEMA_VERSION: u32 = 15;
''',
)
replace_once(
    "scripts/test-current-model-semantics-docs.py",
    '''CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v26"
CURRENT_SHORT = "v26"
''',
    '''CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v27"
CURRENT_SHORT = "v27"
''',
)

for name in [
    "README.md",
    "docs/scientific-model.md",
    "docs/research/odd.md",
    "docs/research/odd-d.md",
    "docs/research/trace.md",
    "docs/research/README.md",
    "docs/roadmap.md",
]:
    p = Path(name)
    text = p.read_text(encoding="utf-8")
    text = text.replace("current model semantics v26", "current model semantics v27")
    text = text.replace(
        "current source tree implements model semantics `anthrosim-model-semantics-v26`",
        "current source tree implements model semantics `anthrosim-model-semantics-v27`",
    )
    text = text.replace("current source tree is model semantics v26", "current source tree is model semantics v27")
    p.write_text(text, encoding="utf-8")

sci = Path("docs/scientific-model.md")
text = sci.read_text(encoding="utf-8")
needle = "M3 resource settlement and condition response occur at the configured M3 interval ends. Mortality is then resolved at those same elapsed interval ends as an order-invariant competition between the M3 condition-mediated cause and the M2 background cause."
addition = needle + " Under v27, the background cause's latent RNG trigger is assigned to living people by the persisted scientific stochastic-coupling rank rather than canonical `PersonId` record position. Condition-mediated trigger assignment remains on its independently tracked pre-v27 stream ordering until AV4-006 is remediated; this does not reintroduce first-called cause priority because both latent triggers are sampled before any simultaneous-trigger attribution. Simultaneous triggers continue to use the exchange-symmetric proportional tie rule."
if needle not in text:
    raise SystemExit("scientific-model competing mortality paragraph not found")
sci.write_text(text.replace(needle, addition, 1), encoding="utf-8")

Path("crates/anthrosim-core/tests/background_mortality_label_invariance.rs").write_text(r'''use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography(probability_per_million: u32) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = probability_per_million;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn person(id: u64, household: u64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(30 * 365),
        reproductive_sex: ReproductiveSex::Male,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn two_household_founders(swapped_labels: bool) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        if swapped_labels { "mortality-relabel-b" } else { "mortality-relabel-a" },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold { id: HouseholdId::new(1), location: CellId::new(1) },
            FounderHousehold { id: HouseholdId::new(2), location: CellId::new(2) },
        ],
        if swapped_labels {
            vec![person(1, 2), person(2, 1)]
        } else {
            vec![person(1, 1), person(2, 2)]
        },
    )
}

#[derive(Debug, PartialEq, Eq)]
struct OneYearOutcome {
    death_cells: Vec<CellId>,
    background_rng: anthrosim_core::RngStreamPosition,
    condition_rng: anthrosim_core::RngStreamPosition,
}

fn run_one_year(seed: u64, swapped_labels: bool) -> OneYearOutcome {
    let mut resources = ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(2).with_initialization(PopulationInitialization::DeclaredFounderStateV1).with_max_person_records(10))
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(500_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut death_cells = recorded.events().events.iter().filter_map(|record| match record.event {
        EventKind::Death { cell, .. } => Some(cell),
        _ => None,
    }).collect::<Vec<_>>();
    death_cells.sort_unstable_by_key(|cell| cell.0);
    OneYearOutcome {
        death_cells,
        background_rng: recorded.checkpoint.rng.demography_mortality,
        condition_rng: recorded.checkpoint.rng.resource_scarcity_mortality,
    }
}

#[test]
fn background_mortality_cells_and_rng_positions_are_person_label_invariant() {
    for seed in 1..=1_000 {
        assert_eq!(run_one_year(seed, false), run_one_year(seed, true),
            "background mortality diverged under pure PersonId relabelling at seed {seed}");
    }
}

fn three_household_founders(rotation: u64) -> FounderPopulationDefinition {
    let household_for_label = |label: u64| ((label - 1 + rotation) % 3) + 1;
    FounderPopulationDefinition::new(
        format!("mortality-cycle-{rotation}"),
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold { id: HouseholdId::new(1), location: CellId::new(1) },
            FounderHousehold { id: HouseholdId::new(2), location: CellId::new(2) },
            FounderHousehold { id: HouseholdId::new(3), location: CellId::new(3) },
        ],
        vec![
            person(1, household_for_label(1)),
            person(2, household_for_label(2)),
            person(3, household_for_label(3)),
        ],
    )
}

fn three_cell_deaths(seed: u64, rotation: u64) -> Vec<CellId> {
    let mut resources = ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(3).with_initialization(PopulationInitialization::DeclaredFounderStateV1).with_max_person_records(10))
        .with_founder_population(three_household_founders(rotation))
        .with_demography(demography(350_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut cells = recorded.events().events.iter().filter_map(|record| match record.event {
        EventKind::Death { cell, .. } => Some(cell),
        _ => None,
    }).collect::<Vec<_>>();
    cells.sort_unstable_by_key(|cell| cell.0);
    cells
}

#[test]
fn background_mortality_is_invariant_to_three_person_cyclic_relabelling() {
    for seed in 1..=256 {
        let baseline = three_cell_deaths(seed, 0);
        assert_eq!(baseline, three_cell_deaths(seed, 1), "rotation 1 diverged at seed {seed}");
        assert_eq!(baseline, three_cell_deaths(seed, 2), "rotation 2 diverged at seed {seed}");
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DownstreamOutcome {
    death_cells: Vec<CellId>,
    living_by_cell: [u64; 2],
    final_food_stock: [u64; 2],
}

fn run_downstream(seed: u64, swapped_labels: bool) -> DownstreamOutcome {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(100)
        .with_initial_stock_units_per_productivity(10)
        .with_seasonality_scale_permille(0);
    resources.max_scarcity_mortality_probability_per_million = 0;
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(2).with_initialization(PopulationInitialization::DeclaredFounderStateV1).with_max_person_records(10))
        .with_founder_population(two_household_founders(swapped_labels))
        .with_demography(demography(250_000))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut death_cells = recorded.events().events.iter().filter_map(|record| match record.event {
        EventKind::Death { cell, .. } => Some(cell),
        _ => None,
    }).collect::<Vec<_>>();
    death_cells.sort_unstable_by_key(|cell| cell.0);
    let population = &recorded.checkpoint.population;
    let mut living_by_cell = [0_u64; 2];
    for raw_id in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw_id)).unwrap();
        if person.death_day.is_none() {
            living_by_cell[usize::try_from(person.location.0 - 1).unwrap()] += 1;
        }
    }
    let resources = &recorded.checkpoint.resources;
    DownstreamOutcome {
        death_cells,
        living_by_cell,
        final_food_stock: [
            resources.cell_food_stock(CellId::new(1)).unwrap(),
            resources.cell_food_stock(CellId::new(2)).unwrap(),
        ],
    }
}

#[test]
fn background_mortality_relabelling_does_not_propagate_into_resource_state() {
    for seed in 1..=256 {
        assert_eq!(run_downstream(seed, false), run_downstream(seed, true),
            "downstream state diverged under pure PersonId relabelling at seed {seed}");
    }
}
''', encoding="utf-8")
