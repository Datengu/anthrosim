from pathlib import Path

p = Path('crates/anthrosim-core/src/founder_initialization.rs')
s = p.read_text()
s = s.replace('    config::ParameterProvenance,\n', '    config::{DemographyConfig, ParameterProvenance},\n')
s = s.replace('    population::ReproductiveSex,\n    world::{PERMILLE_MAX, World},\n', '    population::ReproductiveSex,\n    time::DAYS_PER_YEAR,\n    world::{PERMILLE_MAX, World},\n')
old_sig = '''    pub fn validate(\n        &self,\n        expected_initial_population: u32,\n        max_person_records: u64,\n        world: &World,\n    ) -> Result<(), FounderPopulationError> {'''
new_sig = '''    pub fn validate(\n        &self,\n        expected_initial_population: u32,\n        max_person_records: u64,\n        world: &World,\n        demography: &DemographyConfig,\n    ) -> Result<(), FounderPopulationError> {'''
assert old_sig in s
s = s.replace(old_sig, new_sig)
old_parent_calls = '            validate_parent(self, person, person.female_parent, ReproductiveSex::Female)?;\n            validate_parent(self, person, person.male_parent, ReproductiveSex::Male)?;\n'
new_parent_calls = '''            validate_parent(\n                self,\n                person,\n                person.female_parent,\n                ReproductiveSex::Female,\n                demography,\n            )?;\n            validate_parent(\n                self,\n                person,\n                person.male_parent,\n                ReproductiveSex::Male,\n                demography,\n            )?;\n'''
assert old_parent_calls in s
s = s.replace(old_parent_calls, new_parent_calls)
old_last = '''                if last_birth_day <= person.birth_day {\n                    return Err(FounderPopulationError::PriorBirthNotAfterOwnBirth {\n                        person: person.id,\n                        birth_day: person.birth_day,\n                        last_birth_day,\n                    });\n                }\n'''
assert old_last in s
s = s.replace(old_last, old_last + '''                let age_days = reproductive_age_days(person.birth_day, last_birth_day)\n                    .expect("validated prior birth must be after founder birth");\n                if !female_reproductive_age_supported(demography, age_days) {\n                    return Err(FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge {\n                        person: person.id,\n                        last_birth_day,\n                        age_days,\n                    });\n                }\n''')
old_parent_sig = '''fn validate_parent(\n    definition: &FounderPopulationDefinition,\n    child: &FounderPerson,\n    parent: Option<PersonId>,\n    expected_sex: ReproductiveSex,\n) -> Result<(), FounderPopulationError> {'''
assert old_parent_sig in s
s = s.replace(old_parent_sig, '''fn validate_parent(\n    definition: &FounderPopulationDefinition,\n    child: &FounderPerson,\n    parent: Option<PersonId>,\n    expected_sex: ReproductiveSex,\n    demography: &DemographyConfig,\n) -> Result<(), FounderPopulationError> {''')
old_parent_end = '''    if parent_person.birth_day >= child.birth_day {\n        return Err(FounderPopulationError::ParentNotOlder {\n            person: child.id,\n            parent,\n        });\n    }\n    Ok(())\n}\n'''
assert old_parent_end in s
s = s.replace(old_parent_end, '''    if parent_person.birth_day >= child.birth_day {\n        return Err(FounderPopulationError::ParentNotOlder {\n            person: child.id,\n            parent,\n        });\n    }\n    let age_days = reproductive_age_days(parent_person.birth_day, child.birth_day)\n        .expect("validated parent must be older than child");\n    let supported = match expected_sex {\n        ReproductiveSex::Female => female_reproductive_age_supported(demography, age_days),\n        ReproductiveSex::Male => male_reproductive_age_supported(demography, age_days),\n    };\n    if !supported {\n        return Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge {\n            person: child.id,\n            parent,\n            parent_sex: expected_sex,\n            age_days,\n        });\n    }\n    Ok(())\n}\n\nfn reproductive_age_days(birth_day: i64, event_day: i64) -> Option<u64> {\n    u64::try_from(event_day.checked_sub(birth_day)?).ok()\n}\n\nfn female_reproductive_age_supported(demography: &DemographyConfig, age_days: u64) -> bool {\n    let age_years = age_days / DAYS_PER_YEAR;\n    demography.fertility_bands.iter().any(|band| {\n        age_years >= u64::from(band.start_age_years)\n            && age_years < u64::from(band.end_age_years_exclusive)\n            && band.annual_probability_per_million > 0\n    })\n}\n\nfn male_reproductive_age_supported(demography: &DemographyConfig, age_days: u64) -> bool {\n    let age_years = age_days / DAYS_PER_YEAR;\n    age_years >= u64::from(demography.male_parent_min_age_years)\n        && age_years < u64::from(demography.male_parent_max_age_years_exclusive)\n}\n''')
old_err = '    #[error("founder {person:?} parent {parent:?} is not older than the child")]\n    ParentNotOlder { person: PersonId, parent: PersonId },\n'
assert old_err in s
s = s.replace(old_err, old_err + '''    #[error(\n        "founder {person:?} parent {parent:?} ({parent_sex:?}) was age {age_days} days at the child's birth, outside the configured reproductive-age support"\n    )]\n    ParentOutsideConfiguredReproductiveAge {\n        person: PersonId,\n        parent: PersonId,\n        parent_sex: ReproductiveSex,\n        age_days: u64,\n    },\n''')
old_prior_err = '''    PriorBirthNotAfterOwnBirth {\n        person: PersonId,\n        birth_day: i64,\n        last_birth_day: i64,\n    },\n'''
assert old_prior_err in s
s = s.replace(old_prior_err, old_prior_err + '''    #[error(\n        "founder {person:?} prior birth day {last_birth_day} occurred at age {age_days} days, outside the configured female fertility-age support"\n    )]\n    PriorBirthOutsideConfiguredFertilityAge {\n        person: PersonId,\n        last_birth_day: i64,\n        age_days: u64,\n    },\n''')
s = s.replace('definition.validate(3, 10, &world())', 'definition.validate(3, 10, &world(), &DemographyConfig::synthetic_validation_v1())')
s = s.replace('loaded.validate(3, 10, &world())', 'loaded.validate(3, 10, &world(), &DemographyConfig::synthetic_validation_v1())')
marker = '''    #[test]\n    fn serialized_content_identity_detects_valid_post_load_mutation() {'''
assert marker in s
tests = r'''    #[test]
    fn parent_reproductive_age_support_follows_declared_demography_boundaries() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;

        definition.people[0].birth_day = child_birth - (18 * year - 1);
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Female, .. })));
        definition.people[0].birth_day = child_birth - 18 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].birth_day = child_birth - (45 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].birth_day = child_birth - 45 * year;
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Female, .. })));

        definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        definition.people[1].birth_day = child_birth - (18 * year - 1);
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Male, .. })));
        definition.people[1].birth_day = child_birth - 18 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[1].birth_day = child_birth - (70 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[1].birth_day = child_birth - 70 * year;
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Male, .. })));
    }

    #[test]
    fn one_day_old_parent_is_rejected() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        definition.people[0].birth_day = definition.people[2].birth_day - 1;
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Female, age_days: 1, .. })));
    }

    #[test]
    fn prior_birth_reproductive_age_support_uses_fertility_schedule_boundaries() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = -80 * year;
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year - 1);
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge { .. })));
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 45 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 45 * year);
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge { .. })));
    }

    #[test]
    fn founder_reproductive_history_tracks_custom_fertility_support() {
        let mut demography = DemographyConfig::synthetic_validation_v1();
        demography.fertility_bands = vec![
            crate::config::AgeProbabilityBand::new(0, 21, 0),
            crate::config::AgeProbabilityBand::new(21, 22, 1),
            crate::config::AgeProbabilityBand::new(22, u32::MAX, 0),
        ];
        let mut definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = child_birth - 20 * year;
        assert!(matches!(definition.validate(3, 10, &world(), &demography), Err(FounderPopulationError::ParentOutsideConfiguredReproductiveAge { parent_sex: ReproductiveSex::Female, .. })));
        definition.people[0].birth_day = child_birth - 21 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
    }

'''
s = s.replace(marker, tests + marker)
p.write_text(s)

p = Path('crates/anthrosim-core/src/population.rs')
s = p.read_text()
s = s.replace('    config::{PopulationConfig, PopulationInitialization},\n', '    config::{DemographyConfig, PopulationConfig, PopulationInitialization},\n')
old = '''    pub fn initialize_declared_founder_state_v1(\n        config: PopulationConfig,\n        definition: &FounderPopulationDefinition,\n        world: &World,\n    ) -> Result<Self, PopulationError> {'''
assert old in s
s = s.replace(old, '''    pub fn initialize_declared_founder_state_v1(\n        config: PopulationConfig,\n        definition: &FounderPopulationDefinition,\n        world: &World,\n        demography: &DemographyConfig,\n    ) -> Result<Self, PopulationError> {''')
s = s.replace('        definition.validate(config.initial_population, config.max_person_records, world)?;\n', '        definition.validate(\n            config.initial_population,\n            config.max_person_records,\n            world,\n            demography,\n        )?;\n')
p.write_text(s)

p = Path('crates/anthrosim-core/src/simulation.rs')
s = p.read_text()
old = '''                Population::initialize_declared_founder_state_v1(\n                    config.population,\n                    definition,\n                    &world,\n                )?'''
assert old in s
s = s.replace(old, '''                Population::initialize_declared_founder_state_v1(\n                    config.population,\n                    definition,\n                    &world,\n                    &config.demography,\n                )?''')
old = '''            .validate(\n                config.population.initial_population,\n                config.population.max_person_records,\n                world,\n            )'''
assert old in s
s = s.replace(old, '''            .validate(\n                config.population.initial_population,\n                config.population.max_person_records,\n                world,\n                &config.demography,\n            )''')
p.write_text(s)

p = Path('docs/research/m2-founder-initialization-contract-v1.md')
s = p.read_text()
s = s.replace("A declared pre-run `lastBirthDay` must be strictly negative and strictly later than the founder's own birth day.\n", "A declared pre-run `lastBirthDay` must be strictly negative and strictly later than the founder's own birth day. It must also occur at a female age supported by the experiment's declared fertility schedule, as specified below.\n")
marker = '## 5. Reproductive-history semantics\n'
assert marker in s
section = '''### Schedule-relative reproductive-age validity\n\nDeclared pre-run reproductive chronology is validated against the **same `DemographyConfig` carried by the experiment**. AnthroSim does not introduce a separate universal human reproductive-age constant at the founder boundary.\n\nFor a declared birth event, age is the exact signed-day difference between the founder birth day and the declared event day, interpreted in completed 365-day model years. The event is structurally valid only when:\n\n- a declared **female parent** is in a configured fertility age band whose `annualProbabilityPerMillion` is greater than zero at the child's birth day;\n- a declared **male parent** is within `[maleParentMinAgeYears, maleParentMaxAgeYearsExclusive)` at the child's birth day; and\n- a female founder's declared pre-run **`lastBirthDay`** is in a configured fertility age band with positive fertility support.\n\nThis is a schedule-consistency and biological-plausibility boundary, not a claim that the configured age ranges are universal or empirically correct. A research configuration that changes reproductive-age assumptions changes which founder histories are admissible, and those assumptions retain the provenance of the declared demographic schedule. The check also does not claim that a pre-run event would have been generated on an exact annual M2 scheduler boundary; founder chronology predates model execution and is validated for reproductive-age support at the declared event day.\n\nThe default synthetic validation schedule therefore accepts female reproductive events from completed age 18 through the day before completed age 45, and male parentage from completed age 18 through the day before completed age 70. A one-day-old parent is invalid.\n\n'''
s = s.replace(marker, section + marker)
s = s.replace('- a founder definition whose counts/IDs/chronology/parent relationships/households/locations/condition are invalid;\n', '- a founder definition whose counts/IDs/chronology/parent relationships/households/locations/condition are invalid;\n- a declared parent or pre-run birth event whose parent age lies outside the experiment\'s configured reproductive-age support;\n')
s = s.replace('- invalid founder chronology/genealogy is rejected;\n', '- invalid founder chronology/genealogy is rejected, including female/male parent ages immediately below, at and above configured reproductive-age boundaries;\n- pre-run `lastBirthDay` is rejected below/above configured female fertility support and accepted on supported boundaries;\n- custom fertility schedules change founder reproductive-history acceptance consistently rather than being overridden by a hidden universal age constant;\n')
p.write_text(s)

p = Path('docs/research/founder-population-cli-v1.md')
s = p.read_text()
s += '\n\n## Reproductive chronology validation\n\nFounder parent ages and declared pre-run `lastBirthDay` are validated against the experiment\'s `DemographyConfig` before execution. Female events require positive fertility-band support at the declared event age; male parentage uses the configured male-parent age interval. These are experiment-declared assumptions, not universal anthropological constants.\n'
p.write_text(s)

Path('docs/research/audit-v2/area-b-founder-reproductive-chronology-audit.py').write_text(r'''#!/usr/bin/env python3
"""Independent arithmetic checker for audit-v2 issue #320."""
DAYS_PER_YEAR = 365

def legacy_parent_valid(parent_birth, child_birth): return parent_birth < child_birth
def female_supported(age, bands): return any(a <= age < b and p > 0 for a,b,p in bands)
def male_supported(age, lo, hi): return lo <= age < hi

def main():
    fertility=[(0,18,0),(18,25,220000),(25,35,250000),(35,40,180000),(40,45,80000),(45,2**32-1,0)]
    child=-100
    assert legacy_parent_valid(child-1, child)
    female_days=[18*365-1,18*365,45*365-1,45*365]
    assert [female_supported(d//365,fertility) for d in female_days] == [False,True,True,False]
    male_days=[18*365-1,18*365,70*365-1,70*365]
    assert [male_supported(d//365,18,70) for d in male_days] == [False,True,True,False]
    custom=[(0,21,0),(21,22,1),(22,2**32-1,0)]
    assert [female_supported(a,custom) for a in (20,21,22)] == [False,True,False]
    print('legacy one-day parent accepted by older-only rule: yes')
    print('female boundary support: reject, accept, accept, reject')
    print('male boundary support: reject, accept, accept, reject')
    print('custom fertility support followed exactly: yes')
if __name__ == '__main__': main()
''')
