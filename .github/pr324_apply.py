from pathlib import Path

# Config: replace the misleading ID-sorted v1 treatment with an explicit v2 dependency-aware treatment.
p=Path('crates/anthrosim-core/src/config.rs')
s=p.read_text()
old='''/// Stable identity for the deliberately neutral structural-sensitivity alternative introduced by
/// #207. It is a stress-test mechanism, not a calibrated household-formation model.
pub const DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str = "deterministic_size_fission_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdLifecycleConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    /// Maximum number of living members retained in one household after an annual lifecycle
    /// boundary. Oversized at-residence households are partitioned deterministically into the
    /// minimum number of balanced co-resident groups needed to satisfy this ceiling.
    pub max_living_members: u16,
}

impl HouseholdLifecycleConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn deterministic_size_fission_v1(max_living_members: u16) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            max_living_members,
        }
    }
}
'''
new='''/// Stable identity for the dependency-aware structural-sensitivity alternative introduced by
/// #324. It is an explicit synthetic stress-test, not a calibrated household-formation model.
pub const DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str =
    "deterministic_dependency_fission_v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdLifecycleConfig {
    pub schema_version: u32,
    pub model_id: String,
    pub provenance: ParameterProvenance,
    /// Target maximum living size after an annual lifecycle boundary. The model will not satisfy
    /// this target by manufacturing a household without an independent-age anchor; if too few
    /// anchors exist, the smallest dependency-safe number of groups is used and a group may remain
    /// above this target.
    pub max_living_members: u16,
    /// Minimum age used by this synthetic treatment to define an independent household anchor.
    /// This is a declared structural assumption, not an archaeological universal.
    pub minimum_independent_age_years: u16,
}

impl HouseholdLifecycleConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    #[must_use]
    pub fn deterministic_dependency_fission_v2(
        max_living_members: u16,
        minimum_independent_age_years: u16,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            model_id: DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),
            provenance: ParameterProvenance::SyntheticValidation,
            max_living_members,
            minimum_independent_age_years,
        }
    }
}
'''
assert s.count(old)==1
p.write_text(s.replace(old,new,1))

p=Path('crates/anthrosim-core/src/lib.rs')
s=p.read_text()
s=s.replace('AgeProbabilityBand, DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, DemographyConfig,\n    ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID, HouseholdLifecycleConfig,',
            'AgeProbabilityBand, DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID, DemographyConfig,\n    ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID, HouseholdLifecycleConfig,')
assert 'DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID' not in s
p.write_text(s)

p=Path('crates/anthrosim-core/src/household_lifecycle.rs')
s=p.read_text()
s=s.replace('DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID,',
            'DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID,')
s=s.replace('config.model_id != DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID',
            'config.model_id != DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID')
old='''    if config.max_living_members == 0 {
        return Err(HouseholdLifecycleError::ZeroMaximumLivingMembers);
    }
    Ok(())
'''
new='''    if config.max_living_members == 0 {
        return Err(HouseholdLifecycleError::ZeroMaximumLivingMembers);
    }
    if config.minimum_independent_age_years == 0 {
        return Err(HouseholdLifecycleError::ZeroMinimumIndependentAgeYears);
    }
    Ok(())
'''
assert s.count(old)==1
s=s.replace(old,new,1)
old='''    } = population.fission_oversized_households(config.max_living_members, &eligible)?;'''
new='''    } = population.fission_oversized_households(
        config.max_living_members,
        config.minimum_independent_age_years,
        day,
        &eligible,
    )?;'''
assert s.count(old)==1
s=s.replace(old,new,1)
needle='''    #[error("household lifecycle maximum living members must be greater than zero")]
    ZeroMaximumLivingMembers,
'''
assert s.count(needle)==1
s=s.replace(needle,needle+'''    #[error("household lifecycle minimum independent age must be greater than zero")]
    ZeroMinimumIndependentAgeYears,
''',1)
p.write_text(s)

p=Path('crates/anthrosim-core/src/population.rs')
s=p.read_text()
start=s.index('    pub(crate) fn fission_oversized_households(')
end=s.index('    pub(crate) fn apply_household_relocations(', start)
new=r'''    pub(crate) fn fission_oversized_households(
        &mut self,
        max_living_members: u16,
        minimum_independent_age_years: u16,
        current_day: u64,
        eligible_households: &[bool],
    ) -> Result<HouseholdFissionOutcome, PopulationError> {
        if max_living_members == 0 {
            return Err(PopulationError::ZeroLifecycleHouseholdSize);
        }
        if minimum_independent_age_years == 0 {
            return Err(PopulationError::ZeroLifecycleIndependentAge);
        }
        let original_household_count = self.household_count();
        if eligible_households.len() != original_household_count {
            return Err(PopulationError::HouseholdLifecycleShapeMismatch);
        }
        let minimum_independent_age_days =
            u64::from(minimum_independent_age_years).saturating_mul(365);
        let ceiling = usize::from(max_living_members);
        let mut outcome = HouseholdFissionOutcome::default();

        for (household_index, &is_eligible) in eligible_households.iter().enumerate() {
            if !is_eligible {
                continue;
            }
            let household = HouseholdId::new(
                u64::try_from(household_index)
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?,
            );
            let living_members = (0..self.person_count())
                .filter(|&person_index| {
                    self.is_alive_index(person_index) && self.households[person_index] == household
                })
                .collect::<Vec<_>>();
            if living_members.len() <= ceiling {
                continue;
            }

            let required_groups = living_members.len().div_ceil(ceiling);
            let mut independent = living_members
                .iter()
                .copied()
                .filter(|&person_index| {
                    self.age_days_at_index(person_index, current_day)
                        .is_some_and(|age| age >= minimum_independent_age_days)
                })
                .collect::<Vec<_>>();
            independent.sort_by_key(|&index| {
                (
                    self.birth_days[index],
                    match self.reproductive_sexes[index] {
                        ReproductiveSex::Female => 0_u8,
                        ReproductiveSex::Male => 1_u8,
                    },
                    person_id_from_index(index).0,
                )
            });
            let group_count = required_groups.min(independent.len());
            if group_count < 2 {
                continue;
            }

            let base_group_size = living_members.len() / group_count;
            let larger_group_count = living_members.len() % group_count;
            let target_sizes = (0..group_count)
                .map(|group_index| {
                    base_group_size + usize::from(group_index < larger_group_count)
                })
                .collect::<Vec<_>>();
            let mut groups = vec![Vec::<usize>::new(); group_count];
            let mut assigned_group = vec![None; self.person_count()];

            for (ordinal, &person_index) in independent.iter().enumerate() {
                let group_index = ordinal % group_count;
                groups[group_index].push(person_index);
                assigned_group[person_index] = Some(group_index);
            }

            let mut dependents = living_members
                .iter()
                .copied()
                .filter(|&index| assigned_group[index].is_none())
                .collect::<Vec<_>>();
            dependents.sort_by_key(|&index| {
                (
                    self.birth_days[index],
                    match self.reproductive_sexes[index] {
                        ReproductiveSex::Female => 0_u8,
                        ReproductiveSex::Male => 1_u8,
                    },
                    person_id_from_index(index).0,
                )
            });

            for member_index in dependents {
                let mut parent_groups = Vec::with_capacity(2);
                for parent in [self.female_parents[member_index], self.male_parents[member_index]] {
                    if parent == PersonId::INVALID {
                        continue;
                    }
                    if let Some(parent_index_value) = person_index(parent, self.person_count()) {
                        if self.is_alive_index(parent_index_value)
                            && self.households[parent_index_value] == household
                            && let Some(group_index) = assigned_group[parent_index_value]
                            && !parent_groups.contains(&group_index)
                        {
                            parent_groups.push(group_index);
                        }
                    }
                }
                let candidates = if parent_groups.is_empty() {
                    (0..group_count).collect::<Vec<_>>()
                } else {
                    parent_groups
                };
                let group_index = candidates
                    .into_iter()
                    .max_by_key(|&candidate| {
                        (
                            target_sizes[candidate].saturating_sub(groups[candidate].len()),
                            std::cmp::Reverse(candidate),
                        )
                    })
                    .expect("dependency-safe fission must have at least one anchored group");
                groups[group_index].push(member_index);
                assigned_group[member_index] = Some(group_index);
            }

            debug_assert_eq!(groups.iter().map(Vec::len).sum::<usize>(), living_members.len());
            debug_assert!(groups.iter().all(|group| group.iter().any(|&member_index| {
                self.age_days_at_index(member_index, current_day)
                    .is_some_and(|age| age >= minimum_independent_age_days)
            })));

            let residence = self.household_locations[household_index];
            for group in groups.iter().skip(1) {
                let new_household_raw = u64::try_from(self.household_locations.len())
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?;
                let new_household = HouseholdId::new(new_household_raw);
                self.household_locations.push(residence);
                let mut reassigned = Vec::with_capacity(group.len());
                for &member_index in group {
                    self.households[member_index] = new_household;
                    reassigned.push(person_id_from_index(member_index));
                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);
                }
                outcome.households_created = outcome.households_created.saturating_add(1);
                outcome.fissions.push(HouseholdFissionRecord {
                    source_household: household,
                    new_household,
                    residence,
                    people_reassigned: reassigned,
                });
            }
        }
        Ok(outcome)
    }

'''
p.write_text(s[:start]+new+s[end:])
s=p.read_text()
needle='''    #[error("household lifecycle maximum living size must be greater than zero")]
    ZeroLifecycleHouseholdSize,
'''
assert s.count(needle)==1
p.write_text(s.replace(needle,needle+'''    #[error("household lifecycle minimum independent age must be greater than zero")]
    ZeroLifecycleIndependentAge,
''',1))

for name in [
    'crates/anthrosim-core/tests/household_lifecycle_sensitivity.rs',
    'crates/anthrosim-core/examples/household_lifecycle_sensitivity.rs',
]:
    p=Path(name); s=p.read_text()
    s=s.replace('HouseholdLifecycleConfig::deterministic_size_fission_v1(5)',
                'HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18)')
    s=s.replace('HouseholdLifecycleConfig::deterministic_size_fission_v1(8)',
                'HouseholdLifecycleConfig::deterministic_dependency_fission_v2(8, 18)')
    p.write_text(s)

p=Path('research/general-demography-baseline-v1/confirmatory-definition.json')
s=p.read_text()
old='''      "householdLifecycle": {
        "maxLivingMembers": 8,
        "modelId": "deterministic_size_fission_v1",
        "provenance": "synthetic_validation",
        "schemaVersion": 1
      },'''
new='''      "householdLifecycle": {
        "maxLivingMembers": 8,
        "minimumIndependentAgeYears": 18,
        "modelId": "deterministic_dependency_fission_v2",
        "provenance": "synthetic_validation",
        "schemaVersion": 2
      },'''
assert s.count(old)==1
p.write_text(s.replace(old,new,1))

p=Path('crates/anthrosim-core/src/provenance.rs')
s=p.read_text()
assert s.count('anthrosim-model-semantics-v20')==1
p.write_text(s.replace('anthrosim-model-semantics-v20','anthrosim-model-semantics-v21',1))
