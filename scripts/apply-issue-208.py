from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return (ROOT / path).read_text()


def write(path, text):
    (ROOT / path).write_text(text)


def replace_once(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# lib.rs: register the internal mortality kernel and acceptance tests.
path = "crates/anthrosim-core/src/lib.rs"
text = read(path)
text = replace_once(text, "pub mod migration;\n", "pub mod migration;\nmod mortality;\n", "lib mortality module")
text = replace_once(
    text,
    "#[cfg(test)]\nmod condition_mortality_acceptance_tests;\n",
    "#[cfg(test)]\nmod competing_mortality_acceptance_tests;\n#[cfg(test)]\nmod condition_mortality_acceptance_tests;\n",
    "lib competing mortality tests",
)
write(path, text)

# Authoritative semantics boundary.
path = "crates/anthrosim-core/src/provenance.rs"
text = read(path)
text = replace_once(
    text,
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v14";',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v15";',
    "semantics id",
)
write(path, text)

# Demography: expose the existing mortality stream and split annual fertility finalization from
# the legacy test helper that still exercises one annual mortality transition in isolation.
path = "crates/anthrosim-core/src/demography.rs"
text = read(path)
needle = '''    pub(crate) fn restore_positions(&mut self, positions: [RngStreamPosition; 4]) {
        positions[0].restore(&mut self.mortality);
        positions[1].restore(&mut self.fertility);
        positions[2].restore(&mut self.parentage);
        positions[3].restore(&mut self.newborn_sex);
    }
'''
replacement = needle + '''
    pub(crate) fn mortality_rng_mut(&mut self) -> &mut ChaCha8Rng {
        &mut self.mortality
    }
'''
text = replace_once(text, needle, replacement, "demography mortality rng accessor")
old_header = '''pub(crate) fn process_demographic_year_recorded_with_founder_history(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
'''
new_header = '''pub(crate) fn process_demographic_year_recorded_with_founder_history(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_recorded_internal(
        population,
        world,
        config,
        day,
        rngs,
        events,
        true,
        founder_population,
    )
}

pub(crate) fn process_demographic_year_after_competing_mortality_recorded(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_after_competing_mortality_recorded_with_founder_history(
        population, world, config, day, rngs, events, None,
    )
}

pub(crate) fn process_demographic_year_after_competing_mortality_recorded_with_founder_history(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
    process_demographic_year_recorded_internal(
        population,
        world,
        config,
        day,
        rngs,
        events,
        false,
        founder_population,
    )
}

fn process_demographic_year_recorded_internal(
    population: &mut Population,
    world: &World,
    config: &DemographyConfig,
    day: u64,
    rngs: &mut DemographyRngs,
    events: &mut EventLog,
    apply_background_mortality: bool,
    founder_population: Option<&FounderPopulationDefinition>,
) -> Result<DemographyStepOutcome, PopulationError> {
'''
text = replace_once(text, old_header, new_header, "demography internal split")
start_marker = "    for index in 0..records_at_boundary_start {\n"
start = text.find(start_marker, text.find("fn process_demographic_year_recorded_internal"))
if start < 0:
    raise SystemExit("demography mortality loop start not found")
text = text[:start] + "    if apply_background_mortality {\n    " + text[start:] 
end_marker = "\n    if population.living_count() == 0 {\n"
end = text.find(end_marker, start)
if end < 0:
    raise SystemExit("demography mortality loop end not found")
text = text[:end] + "\n    }" + text[end:]
text = replace_once(
    text,
    "/// Mortality is then drawn first. Fertility is a conditional live-birth opportunity among\n/// surviving females, so a female that undergoes demographic mortality on this boundary cannot\n/// also give birth on it. This is an explicit annual discrete competing-transition contract, not\n/// a continuous-time hazard model.\n",
    "/// The test-only standalone transition can still draw the annual background-mortality risk in\n/// one step. Authoritative simulation hosts instead partition that same annual risk over elapsed\n/// M3 intervals and resolve it jointly with condition-mediated mortality before calling the annual\n/// fertility/parentage finalizer below. Fertility therefore remains conditional on survival through\n/// all mortality processes in the elapsed year.\n",
    "demography contract comment",
)
write(path, text)

# Resource system: add elapsed background risk to the existing M3 condition-mortality boundary and
# resolve the two latent causes symmetrically.
path = "crates/anthrosim-core/src/resources.rs"
text = read(path)
text = replace_once(
    text,
    "    config::{PROBABILITY_PER_MILLION, ResourceConfig},\n",
    "    config::{DemographyConfig, PROBABILITY_PER_MILLION, ResourceConfig},\n    demography::annual_probability_for_age,\n",
    "resource demography imports",
)
text = replace_once(
    text,
    "    ids::HouseholdId,\n",
    "    ids::HouseholdId,\n    mortality::{\n        CompetingMortalityCause, MortalityMathError, ProbabilityFraction,\n        annual_probability_for_interval, draw_probability_fraction,\n        probability_fraction_per_million_ceil, resolve_two_cause_competing_mortality,\n    },\n",
    "resource mortality imports",
)
prob_struct = '''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbabilityFraction {
    numerator: u128,
    denominator: u128,
}

'''
text = replace_once(text, prob_struct, "", "remove local probability fraction")
context = '''pub(crate) struct ResourcePeriodContext<'a> {
    pub world: &'a World,
    pub config: &'a ResourceConfig,
    pub period_index_in_year: u16,
    pub day: u64,
}
'''
text = replace_once(
    text,
    context,
    context + '''
/// M2 background mortality inputs evaluated on the same elapsed interval as M3 condition
/// mortality. The demographic schedule remains age-indexed from the start of the model year.
pub(crate) struct BackgroundMortalityContext<'a> {
    pub config: &'a DemographyConfig,
    pub mortality_rng: &'a mut ChaCha8Rng,
}
''',
    "background mortality context",
)
old_method = '''    pub(crate) fn process_period_recorded_with_presence(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        events: &mut EventLog,
        temporary_presence: Option<&TemporaryResourcePeriod>,
    ) -> Result<ResourceStepOutcome, ResourceError> {
'''
new_method = '''    pub(crate) fn process_period_recorded_with_presence(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        events: &mut EventLog,
        temporary_presence: Option<&TemporaryResourcePeriod>,
    ) -> Result<ResourceStepOutcome, ResourceError> {
        self.process_period_recorded_with_presence_and_background(
            population,
            context,
            scarcity_rng,
            None,
            events,
            temporary_presence,
        )
    }

    pub(crate) fn process_period_recorded_with_presence_and_background(
        &mut self,
        population: &mut Population,
        context: &ResourcePeriodContext<'_>,
        scarcity_rng: &mut ChaCha8Rng,
        mut background_mortality: Option<BackgroundMortalityContext<'_>>,
        events: &mut EventLog,
        temporary_presence: Option<&TemporaryResourcePeriod>,
    ) -> Result<ResourceStepOutcome, ResourceError> {
'''
text = replace_once(text, old_method, new_method, "resource competing method")
start = text.find("        let people_at_mortality_boundary = population.person_count();\n")
end = text.find("        let stock_after = self.total_food_stock()?;\n", start)
if start < 0 or end < 0:
    raise SystemExit("resource mortality block not found")
new_block = '''        let people_at_mortality_boundary = population.person_count();
        let year_start_day = day
            .checked_sub(period_end)
            .ok_or(ResourceError::InternalInvariant(
                "resource mortality boundary precedes its model-year interval",
            ))?;
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

            let (resolved_cause, background_probability_per_million) =
                if let Some(background) = background_mortality.as_mut() {
                    let age_days = population
                        .age_days_at_index(person_index, year_start_day)
                        .ok_or(ResourceError::InternalInvariant(
                            "living person has no representable age at demographic year start",
                        ))?;
                    let annual_background_probability = annual_probability_for_age(
                        &background.config.mortality_bands,
                        age_days,
                    );
                    let background_probability = annual_probability_for_interval(
                        annual_background_probability,
                        period_start,
                        period_end,
                    )?;
                    let background_probability_per_million =
                        probability_fraction_per_million_ceil(background_probability)?;
                    (
                        resolve_two_cause_competing_mortality(
                            condition_probability,
                            background_probability,
                            scarcity_rng,
                            &mut *background.mortality_rng,
                        )?,
                        background_probability_per_million,
                    )
                } else {
                    (
                        draw_probability_fraction(scarcity_rng, condition_probability)?
                            .then_some(CompetingMortalityCause::ConditionMediated),
                        0,
                    )
                };

            let Some(resolved_cause) = resolved_cause else {
                continue;
            };
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
                            condition_probability_per_million,
                        )
                    }
                    CompetingMortalityCause::Background => (
                        DeathCause::DemographicMortality,
                        background_probability_per_million,
                    ),
                };
                events.push_authoritative(
                    day,
                    EventKind::Death {
                        person,
                        household,
                        cell,
                        cause,
                        condition_permille: condition,
                        probability_per_million,
                    },
                );
            }
        }

'''
text = text[:start] + new_block + text[end:]
start = text.find("fn draw_probability_fraction(")
end = text.find("fn seasonal_prefix_table()", start)
if start < 0 or end < 0:
    raise SystemExit("resource local probability helpers not found")
text = text[:start] + text[end:]
text = replace_once(
    text,
    "    TemporaryResource(#[from] TemporaryResourceAccountingError),\n",
    "    TemporaryResource(#[from] TemporaryResourceAccountingError),\n    #[error(transparent)]\n    Mortality(#[from] MortalityMathError),\n",
    "resource mortality error",
)
write(path, text)

# Core host: every M3 interval receives the annual-background risk slice; annual M2 then performs
# fertility/parentage only.
path = "crates/anthrosim-core/src/simulation.rs"
text = read(path)
text = replace_once(
    text,
    "        process_demographic_year_recorded_with_founder_history, validate_demography_config,\n",
    "        process_demographic_year_after_competing_mortality_recorded_with_founder_history,\n        validate_demography_config,\n",
    "core demography import",
)
text = replace_once(
    text,
    "        ResourceConfigError, ResourceError, ResourcePeriodContext, ResourceRngs,\n",
    "        BackgroundMortalityContext, ResourceConfigError, ResourceError, ResourcePeriodContext,\n        ResourceRngs,\n",
    "core resource import",
)
text = replace_once(
    text,
    "                    let outcome = self.resources.process_period_recorded_with_presence(\n",
    "                    let outcome = self\n                        .resources\n                        .process_period_recorded_with_presence_and_background(\n",
    "core resource call",
)
text = replace_once(
    text,
    "                        &mut self.resource_rngs.scarcity_mortality,\n                        &mut self.events,\n",
    "                        &mut self.resource_rngs.scarcity_mortality,\n                        Some(BackgroundMortalityContext {\n                            config: &self.config.demography,\n                            mortality_rng: self.demography_rngs.mortality_rng_mut(),\n                        }),\n                        &mut self.events,\n",
    "core competing args",
)
text = replace_once(
    text,
    "            let outcome = process_demographic_year_recorded_with_founder_history(\n",
    "            let outcome =\n                process_demographic_year_after_competing_mortality_recorded_with_founder_history(\n",
    "core annual finalizer",
)
write(path, text)

# Spatial host parity.
path = "crates/anthrosim-core/src/spatial_simulation.rs"
text = read(path)
text = replace_once(
    text,
    "        process_demographic_year_recorded, validate_demography_config,\n",
    "        process_demographic_year_after_competing_mortality_recorded, validate_demography_config,\n",
    "spatial demography import",
)
text = replace_once(
    text,
    "        ResourceConfigError, ResourceError, ResourcePeriodContext, ResourceRngs,\n",
    "        BackgroundMortalityContext, ResourceConfigError, ResourceError, ResourcePeriodContext,\n        ResourceRngs,\n",
    "spatial resource import",
)
text = replace_once(
    text,
    "                    let outcome = self.resources.process_period_recorded_with_presence(\n",
    "                    let outcome = self\n                        .resources\n                        .process_period_recorded_with_presence_and_background(\n",
    "spatial resource call",
)
text = replace_once(
    text,
    "                        &mut self.resource_rngs.scarcity_mortality,\n                        &mut self.events,\n",
    "                        &mut self.resource_rngs.scarcity_mortality,\n                        Some(BackgroundMortalityContext {\n                            config: &self.config.demography,\n                            mortality_rng: self.demography_rngs.mortality_rng_mut(),\n                        }),\n                        &mut self.events,\n",
    "spatial competing args",
)
text = replace_once(
    text,
    "            let outcome = process_demographic_year_recorded(\n",
    "            let outcome = process_demographic_year_after_competing_mortality_recorded(\n",
    "spatial annual finalizer",
)
write(path, text)

# Event meaning: same wire shape, interval-specific cause probability under v15.
path = "crates/anthrosim-core/src/events.rs"
text = read(path)
text = replace_once(
    text,
    "        condition_permille: u16,\n        probability_per_million: u32,\n",
    "        condition_permille: u16,\n        /// Cause-specific conditional probability for the elapsed interval in which this death\n        /// was resolved. Under v15 this is not the joint all-cause competing-risk probability.\n        probability_per_million: u32,\n",
    "death probability comment",
)
write(path, text)

# Demography observability: background risk is now evaluated at every M3 interval. Reconstruct
# interval exposures/deaths directly from persistent person histories and resource-boundary timing;
# keep the annual event replay for fertility/parentage only.
path = "crates/anthrosim-core/src/demography_observability.rs"
text = read(path)
text = replace_once(
    text,
    "    manifest::StopReason,\n",
    "    manifest::StopReason,\n    mortality::{annual_probability_for_interval, probability_fraction_per_million_ceil},\n",
    "observability mortality imports",
)
text = replace_once(
    text,
    "    rng::RngFactory,\n",
    "    resources::resource_period_day_bounds,\n    rng::RngFactory,\n",
    "observability resource import",
)
text = replace_once(
    text,
    "    pub fertility_probability_is_conditional_on_m2_survival: bool,\n",
    "    pub mortality_risk_intervals_per_year: u16,\n    pub mortality_is_order_invariant_competing_risk: bool,\n    /// Historical field name retained for wire continuity. Under v15 this means fertility is\n    /// conditional on survival through the complete competing-mortality process for the year.\n    pub fertility_probability_is_conditional_on_m2_survival: bool,\n",
    "observability v15 fields",
)
text = replace_once(
    text,
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 1;\n",
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 2;\n",
    "observability schema",
)
text = replace_once(
    text,
    "    let mut summary = empty_summary(initial_population, &checkpoint.population);\n",
    "    let mut summary = empty_summary(initial_population, &checkpoint.population);\n    summarize_background_mortality(\n        checkpoint,\n        &mut mortality_bands,\n        &mut summary,\n    )?;\n",
    "observability mortality summary call",
)
pattern = re.compile(r'''\n        count_mortality_exposures\(\n            &people,\n            config,\n            interval_start_day,\n            &mut mortality_bands,\n            &mut summary,\n        \)\?;\n        apply_demographic_deaths\(\n            &mut people,\n            day_events,\n            config,\n            interval_start_day,\n            &mut mortality_bands,\n            &mut summary,\n            day,\n        \)\?;\n''')
text, count = pattern.subn("\n", text, count=1)
if count != 1:
    raise SystemExit(f"observability annual mortality replay removal: found {count}")
text = replace_once(
    text,
    "        fertility_probability_is_conditional_on_m2_survival: true,\n",
    "        mortality_risk_intervals_per_year: checkpoint.experiment.resources.periods_per_year,\n        mortality_is_order_invariant_competing_risk: true,\n        fertility_probability_is_conditional_on_m2_survival: true,\n",
    "observability report fields",
)
text = replace_once(
    text,
    '''            EventKind::Death {
                person,
                cause: DeathCause::ResourceScarcity,
                ..
            } => mark_replay_death(people, *person, day)?,
''',
    '''            EventKind::Death { person, .. } => mark_replay_death(people, *person, day)?,
''',
    "observability same-day all deaths",
)
start = text.find("fn count_mortality_exposures(\n")
end = text.find("fn births_by_female(\n", start)
if start < 0 or end < 0:
    raise SystemExit("observability mortality helper block not found")
new_helpers = r'''fn summarize_background_mortality(
    checkpoint: &SimulationCheckpoint,
    bands: &mut [DemographicMortalityBandObservability],
    summary: &mut DemographyObservabilitySummary,
) -> Result<(), DemographyObservabilityError> {
    let config = &checkpoint.experiment.demography;
    let periods_per_year = checkpoint.experiment.resources.periods_per_year;
    let end_day = checkpoint.time.days();
    let years_touched = end_day.div_ceil(DAYS_PER_YEAR);

    for index in 0..checkpoint.population.person_count() {
        let id = PersonId::new(index as u64 + 1);
        let person = checkpoint
            .population
            .person(id)
            .ok_or_else(|| invalid(format!("final population is missing {id:?}")))?;
        for year_index in 0..years_touched {
            let year_start = year_index.saturating_mul(DAYS_PER_YEAR);
            if year_start >= end_day {
                break;
            }
            let year_start_i64 = i64::try_from(year_start)
                .map_err(|_| invalid("demographic year start does not fit i64".to_owned()))?;
            if person.birth_day > year_start_i64
                || person.death_day.is_some_and(|death_day| death_day <= year_start)
            {
                continue;
            }
            let age_days = u64::try_from(
                year_start_i64
                    .checked_sub(person.birth_day)
                    .ok_or_else(|| invalid("demographic age subtraction overflowed".to_owned()))?,
            )
            .map_err(|_| invalid(format!("{id:?} has negative age at day {year_start}")))?;
            let band = schedule_band_index(&config.mortality_bands, age_days)?;

            for period_index in 0..periods_per_year {
                let (_, interval_end) = resource_period_day_bounds(period_index, periods_per_year)
                    .ok_or_else(|| invalid("resource mortality interval is invalid".to_owned()))?;
                let boundary_day = year_start.saturating_add(interval_end);
                if boundary_day > end_day {
                    break;
                }
                if person
                    .death_day
                    .is_some_and(|death_day| death_day < boundary_day)
                {
                    break;
                }
                bands[band].exposures = bands[band].exposures.saturating_add(1);
                summary.mortality_exposures = summary.mortality_exposures.saturating_add(1);
                if person.death_day == Some(boundary_day) {
                    break;
                }
            }
        }
    }

    for record in &checkpoint.events.events {
        let EventKind::Death {
            person,
            cause: DeathCause::DemographicMortality,
            probability_per_million,
            ..
        } = &record.event
        else {
            continue;
        };
        if record.day == 0 {
            return Err(invalid(format!("background death for {person:?} occurs at day zero")));
        }
        let person_record = checkpoint.population.person(*person).ok_or_else(|| {
            invalid(format!("background death references unknown {person:?}"))
        })?;
        let year_start = (record.day - 1) / DAYS_PER_YEAR * DAYS_PER_YEAR;
        let year_start_i64 = i64::try_from(year_start)
            .map_err(|_| invalid("demographic year start does not fit i64".to_owned()))?;
        let age_days = u64::try_from(
            year_start_i64
                .checked_sub(person_record.birth_day)
                .ok_or_else(|| invalid("demographic age subtraction overflowed".to_owned()))?,
        )
        .map_err(|_| invalid(format!("{person:?} has negative age at day {year_start}")))?;
        let band = schedule_band_index(&config.mortality_bands, age_days)?;
        let annual_probability = config.mortality_bands[band].annual_probability_per_million;
        let offset = record.day - year_start;
        let mut interval = None;
        for period_index in 0..periods_per_year {
            let bounds = resource_period_day_bounds(period_index, periods_per_year)
                .ok_or_else(|| invalid("resource mortality interval is invalid".to_owned()))?;
            if bounds.1 == offset {
                interval = Some(bounds);
                break;
            }
        }
        let (interval_start, interval_end) = interval.ok_or_else(|| {
            invalid(format!(
                "background death for {person:?} at day {} is not an M3 mortality boundary",
                record.day
            ))
        })?;
        let expected = probability_fraction_per_million_ceil(
            annual_probability_for_interval(
                annual_probability,
                interval_start,
                interval_end,
            )
            .map_err(|error| invalid(error.to_string()))?,
        )
        .map_err(|error| invalid(error.to_string()))?;
        if *probability_per_million != expected {
            return Err(invalid(format!(
                "background death probability for {person:?} at day {} is {probability_per_million}, expected {expected}",
                record.day
            )));
        }
        bands[band].deaths = bands[band].deaths.saturating_add(1);
        summary.demographic_deaths = summary.demographic_deaths.saturating_add(1);
    }
    Ok(())
}

'''
text = text[:start] + new_helpers + text[end:]
write(path, text)

# Update observability acceptance fixtures for interval exposure semantics.
path = "crates/anthrosim-core/src/demography_observability_tests.rs"
text = read(path)
text = replace_once(
    text,
    "    assert!(report.fertility_probability_is_conditional_on_m2_survival);\n",
    "    assert_eq!(report.mortality_risk_intervals_per_year, 4);\n    assert!(report.mortality_is_order_invariant_competing_risk);\n    assert!(report.fertility_probability_is_conditional_on_m2_survival);\n",
    "observability fixture flags",
)
text = replace_once(
    text,
    "    assert_eq!(report.summary.mortality_exposures, 0);\n",
    "    assert_eq!(report.summary.mortality_exposures, 4);\n",
    "partial-year exposure count",
)
text = replace_once(
    text,
    "    resources.max_scarcity_mortality_probability_per_million = 0;\n    let config = ExperimentConfig::new(7, 1)\n",
    "    resources.max_scarcity_mortality_probability_per_million = 0;\n    resources.periods_per_year = 1;\n    let config = ExperimentConfig::new(7, 1)\n",
    "total mortality single interval",
)
write(path, text)

# Scientific-model narrative: v15 order-invariant competing risks and ordering.
path = "docs/scientific-model.md"
text = read(path)
text = re.sub(
    r"\*\*Status:\*\* working specification for the AnthroSim v0\.3\.0 package / post-M9 scientific-hardening line / model semantics v\d+",
    "**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v15",
    text,
    count=1,
)
text = replace_once(
    text,
    "Authoritative simulation time is represented in integer days. M2 baseline demography is an annual discrete transition.\n",
    "Authoritative simulation time is represented in integer days. M2 fertility/parentage remains an annual discrete transition. Under v15, M2 background mortality is an annual age-specific risk parameter whose risk is partitioned over the elapsed M3 mortality intervals of each model year, so it is no longer treated as a privileged instantaneous death process at the year-end function call.\n",
    "scientific time M2",
)
text = replace_once(
    text,
    "M3 resource settlement, condition response and condition-mediated survival occur at the configured M3 interval ends.",
    "M3 resource settlement and condition response occur at the configured M3 interval ends. Mortality is then resolved at those same elapsed interval ends as an order-invariant competition between the M3 condition-mediated cause and the M2 background cause.",
    "scientific competing mortality intro",
)
text = replace_once(
    text,
    "At a shared M3/M4 day, elapsed M3 resource/condition/survival processing occurs first, then due M9 transitions/start processing, then M4 permanent migration. Either M3 or M4 may otherwise occur alone. M2 annual demography follows the year's subannual processing.\n",
    "At a shared M3/M4 day, elapsed M3 resource/condition processing and joint M3/M2 competing mortality occur first, then due M9 transitions/start processing, then M4 permanent migration. Either M3 or M4 may otherwise occur alone. The year-end M2 stage then performs fertility/parentage only because background mortality has already been resolved over the elapsed year. A death on a shared boundary therefore retains the persistent residence that existed before that day's M4 move opportunity.\n",
    "scientific shared boundary",
)
text = replace_once(
    text,
    "3. if M3 is due, settle the elapsed resource interval using duration-aware residence/visitor/transit person-days, update condition, and apply the elapsed condition-mediated mortality probability;\n4. process due M9 temporary transitions/start decisions for that day;\n5. if M4 is due, evaluate permanent migration only for eligible households physically at residence, using the M4 decision interval's resource-support demand;\n6. apply selected permanent moves simultaneously;\n7. after the year's subannual schedules complete, run M2 annual demography.\n",
    "3. if M3 is due, settle the elapsed resource interval, update condition, convert the age-at-year-start M2 annual background risk to that elapsed interval, and resolve background plus condition-mediated mortality jointly with no first-called cause priority;\n4. process due M9 temporary transitions/start decisions for that day;\n5. if M4 is due, evaluate permanent migration only for eligible households physically at residence, using the M4 decision interval's resource-support demand;\n6. apply selected permanent moves simultaneously;\n7. after the year's subannual schedules complete, run the annual M2 fertility/parentage stage for survivors.\n",
    "scientific process steps",
)
text = replace_once(
    text,
    "M2 mortality/fertility remains annual and residence-based. Births inherit household residence. If a person dies while the household is away, temporary-presence state is updated so visitor/transit counts remain correct; `Death.cell` remains a residence attribution field and must not be read as an observed physical death location.\n",
    "M2 fertility remains annual and residence-based. M2 background mortality retains an annual age-specific parameter but is executed as elapsed interval risk in the v15 competing-risk resolver. Births inherit household residence. If a person dies while the household is away, temporary-presence state is updated so visitor/transit counts remain correct; `Death.cell` remains a residence attribution field and must not be read as an observed physical death location.\n",
    "scientific M2 overview",
)
write(path, text)

# Condition-mortality contract: close the previously documented #208 limitation without changing
# the v10 meaning of condition-mediated cause.
path = "docs/research/m3-condition-mortality-contract-v1.md"
text = read(path)
text = replace_once(
    text,
    "## 9. Remaining competing-risk limitation\n\nIssue #208 remains separate. On a day when the condition-mediated M3 hazard and annual M2 demographic mortality coincide, total and cause-specific mortality still require an explicit competing-risk attribution contract. v10 corrects what the M3 hazard itself means; it does not by itself solve same-boundary competition between M3 and M2.\n",
    "## 9. v15 competing-risk extension\n\nIssue #208 is resolved by the separate [`competing-mortality-risks-v1.md`](competing-mortality-risks-v1.md) contract. The v10 condition-mediated meaning above is unchanged: condition remains a general shared mediator. v15 additionally prevents that cause from receiving automatic priority over M2 background mortality merely because M3 executes first. The two cause-specific interval risks receive independent latent triggers; all-cause survival is their product, and dual-trigger attribution is risk-weighted with a symmetric deterministic draw.\n",
    "condition contract competing section",
)
write(path, text)

print("issue 208 source patch applied")
