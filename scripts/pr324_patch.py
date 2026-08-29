#!/usr/bin/env python3
from pathlib import Path
import json


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected one replacement, found {text.count(old)}")
    p.write_text(text.replace(old, new, 1))


# Scientific configuration identity: do not silently reinterpret v1.
p = Path("crates/anthrosim-core/src/config.rs")
text = p.read_text()
text = text.replace(
    'pub const DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str = "deterministic_size_fission_v1";',
    'pub const DETERMINISTIC_AGE_BALANCED_FISSION_HOUSEHOLD_LIFECYCLE_ID: &str =\n    "deterministic_age_balanced_fission_v2";',
)
text = text.replace(
    "/// Stable identity for the deliberately neutral structural-sensitivity alternative introduced by\n/// #207. It is a stress-test mechanism, not a calibrated household-formation model.",
    "/// Stable identity for the age/sex-balanced structural-sensitivity alternative.\n///\n/// Version 2 supersedes the PersonId-sliced v1 treatment after audit-v2 #324. It remains a\n/// deterministic stress-test mechanism, not a calibrated household-formation model.",
)
text = text.replace(
    "pub const CURRENT_SCHEMA_VERSION: u32 = 1;\n\n    #[must_use]\n    pub fn deterministic_size_fission_v1(max_living_members: u16) -> Self {",
    "pub const CURRENT_SCHEMA_VERSION: u32 = 2;\n\n    #[must_use]\n    pub fn deterministic_age_balanced_fission_v2(max_living_members: u16) -> Self {",
    1,
)
text = text.replace(
    "model_id: DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),",
    "model_id: DETERMINISTIC_AGE_BALANCED_FISSION_HOUSEHOLD_LIFECYCLE_ID.to_owned(),",
    1,
)
p.write_text(text)

p = Path("crates/anthrosim-core/src/household_lifecycle.rs")
p.write_text(
    p.read_text().replace(
        "DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID",
        "DETERMINISTIC_AGE_BALANCED_FISSION_HOUSEHOLD_LIFECYCLE_ID",
    )
)

replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v20";',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v21";',
)

p = Path("crates/anthrosim-core/src/population.rs")
text = p.read_text()
old = '''            // Use the minimum number of groups needed to obey the configured ceiling, then
            // balance group sizes so deterministic fission does not manufacture avoidable
            // singleton households. Stable PersonId order is the explicit neutral partition rule.
            let group_count = living_members.len().div_ceil(ceiling);
            let base_group_size = living_members.len() / group_count;
            let larger_group_count = living_members.len() % group_count;
            let source_group_size = base_group_size + if larger_group_count > 0 { 1 } else { 0 };
            let residence = self.household_locations[household_index];
            let mut cursor = source_group_size;

            for group_index in 1..group_count {
                let group_size = base_group_size
                    + if group_index < larger_group_count {
                        1
                    } else {
                        0
                    };
                let new_household_raw = u64::try_from(self.household_locations.len())
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?;
                let new_household = HouseholdId::new(new_household_raw);
                self.household_locations.push(residence);
                let mut reassigned = Vec::with_capacity(group_size);
                for &person_index in &living_members[cursor..cursor + group_size] {
                    self.households[person_index] = new_household;
                    reassigned.push(person_id_from_index(person_index));
                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);
                }
                outcome.households_created = outcome.households_created.saturating_add(1);
                outcome.fissions.push(HouseholdFissionRecord {
                    source_household: household,
                    new_household,
                    residence,
                    people_reassigned: reassigned,
                });
                cursor += group_size;
            }

            debug_assert_eq!(cursor, living_members.len());
'''
new = '''            // Use the minimum number of groups needed to obey the configured ceiling and
            // balance target sizes as before, but assign membership by scientific composition
            // rather than packed-storage/PersonId order. Oldest members are distributed first;
            // reproductive sex is the deterministic secondary key. PersonId breaks ties only
            // among otherwise age/sex-equivalent records, so it cannot create cohort sorting.
            let group_count = living_members.len().div_ceil(ceiling);
            let base_group_size = living_members.len() / group_count;
            let larger_group_count = living_members.len() % group_count;
            let target_sizes = (0..group_count)
                .map(|group_index| {
                    base_group_size + usize::from(group_index < larger_group_count)
                })
                .collect::<Vec<_>>();
            let mut ordered_members = living_members;
            ordered_members.sort_by_key(|&person_index| {
                let sex_key = match self.reproductive_sexes[person_index] {
                    ReproductiveSex::Female => 0_u8,
                    ReproductiveSex::Male => 1_u8,
                };
                (self.birth_days[person_index], sex_key, person_index)
            });
            let mut groups = target_sizes
                .iter()
                .map(|&target| Vec::with_capacity(target))
                .collect::<Vec<_>>();
            let mut next_group = 0_usize;
            for person_index in ordered_members {
                while groups[next_group].len() >= target_sizes[next_group] {
                    next_group = (next_group + 1) % group_count;
                }
                groups[next_group].push(person_index);
                next_group = (next_group + 1) % group_count;
            }
            debug_assert!(groups
                .iter()
                .zip(&target_sizes)
                .all(|(group, &target)| group.len() == target));

            let residence = self.household_locations[household_index];
            for group_members in groups.iter().skip(1) {
                let new_household_raw = u64::try_from(self.household_locations.len())
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?;
                let new_household = HouseholdId::new(new_household_raw);
                self.household_locations.push(residence);
                let mut reassigned = Vec::with_capacity(group_members.len());
                for &person_index in group_members {
                    self.households[person_index] = new_household;
                    reassigned.push(person_id_from_index(person_index));
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
'''
if text.count(old) != 1:
    raise SystemExit(f"population fission block mismatch: {text.count(old)}")
text = text.replace(old, new, 1)

marker = "    #[test]\n    fn rejects_zero_household_size() {\n"
tests = r'''    fn household_science_composition(population: &Population) -> Vec<Vec<(i64, u8)>> {
        let mut groups = (1..=population.household_count())
            .map(|household_raw| {
                let household = HouseholdId::new(household_raw as u64);
                let mut records = (0..population.person_count())
                    .filter(|&index| {
                        population.is_alive_index(index) && population.households[index] == household
                    })
                    .map(|index| {
                        (
                            population.birth_days[index],
                            match population.reproductive_sexes[index] {
                                ReproductiveSex::Female => 0_u8,
                                ReproductiveSex::Male => 1_u8,
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                records.sort_unstable();
                records
            })
            .collect::<Vec<_>>();
        groups.sort();
        groups
    }

    #[test]
    fn age_balanced_fission_is_invariant_to_person_id_relabeling_of_unique_science_records() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(71)).unwrap();
        let config = PopulationConfig::new(9).with_target_household_size(9);
        let mut a = Population::initialize(config, &world, RngFactory::new(71)).unwrap();
        let mut b = a.clone();
        let records = [
            (-18_250, ReproductiveSex::Female),
            (-17_885, ReproductiveSex::Male),
            (-14_600, ReproductiveSex::Female),
            (-13_870, ReproductiveSex::Male),
            (-10_950, ReproductiveSex::Female),
            (-7_300, ReproductiveSex::Male),
            (-3_650, ReproductiveSex::Female),
            (-365, ReproductiveSex::Male),
            (0, ReproductiveSex::Female),
        ];
        for (index, &(birth_day, sex)) in records.iter().enumerate() {
            a.birth_days[index] = birth_day;
            a.reproductive_sexes[index] = sex;
            let reverse = records.len() - 1 - index;
            b.birth_days[reverse] = birth_day;
            b.reproductive_sexes[reverse] = sex;
        }
        a.fission_oversized_households(5, &[true]).unwrap();
        b.fission_oversized_households(5, &[true]).unwrap();
        assert_eq!(household_science_composition(&a), household_science_composition(&b));
    }

    #[test]
    fn age_balanced_fission_prevents_newborn_only_tail_when_older_members_can_seed_groups() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(73)).unwrap();
        let config = PopulationConfig::new(9).with_target_household_size(9);
        let mut population = Population::initialize(config, &world, RngFactory::new(73)).unwrap();
        let birth_days = [-18_250, -17_885, -14_600, -13_870, -10_950, 0, 0, 0, 0];
        for (index, &birth_day) in birth_days.iter().enumerate() {
            population.birth_days[index] = birth_day;
            population.reproductive_sexes[index] = if index % 2 == 0 {
                ReproductiveSex::Female
            } else {
                ReproductiveSex::Male
            };
        }
        population.fission_oversized_households(5, &[true]).unwrap();
        let mut older_counts = vec![0_usize; population.household_count()];
        let mut newborn_counts = vec![0_usize; population.household_count()];
        for index in 0..population.person_count() {
            let household_index = usize::try_from(population.households[index].0 - 1).unwrap();
            if population.birth_days[index] < 0 {
                older_counts[household_index] += 1;
            } else {
                newborn_counts[household_index] += 1;
            }
        }
        assert_eq!(older_counts, vec![3, 2]);
        assert_eq!(newborn_counts, vec![2, 2]);
        assert!(older_counts.iter().all(|&count| count > 0));
    }

'''
if text.count(marker) != 1:
    raise SystemExit("population test insertion marker mismatch")
text = text.replace(marker, tests + marker, 1)
p.write_text(text)

# Migrate current code/config/test surfaces. Preserve historical v1 reference provenance.
old_literal = "deterministic_size_fission_v1"
new_literal = "deterministic_age_balanced_fission_v2"
historical = {
    Path("docs/research/household-lifecycle-structural-sensitivity-v1.md"),
    Path("research/household-lifecycle-sensitivity-v1/reference-result.json"),
    Path("docs/research/household-lifecycle-structural-sensitivity-result.md"),
}
for path in Path(".").rglob("*"):
    if not path.is_file() or path in historical or ".git" in path.parts:
        continue
    if path.suffix.lower() not in {".rs", ".json", ".md", ".py", ".toml", ".yml", ".yaml"}:
        continue
    try:
        file_text = path.read_text()
    except UnicodeDecodeError:
        continue
    if old_literal in file_text:
        path.write_text(file_text.replace(old_literal, new_literal))


def bump_lifecycle_schema(value) -> bool:
    changed = False
    if isinstance(value, dict):
        if value.get("modelId") == new_literal and value.get("schemaVersion") != 2:
            value["schemaVersion"] = 2
            changed = True
        for child in value.values():
            changed = bump_lifecycle_schema(child) or changed
    elif isinstance(value, list):
        for child in value:
            changed = bump_lifecycle_schema(child) or changed
    return changed


for path in Path(".").rglob("*.json"):
    if path in historical or ".git" in path.parts:
        continue
    try:
        value = json.loads(path.read_text())
    except Exception:
        continue
    if bump_lifecycle_schema(value):
        path.write_text(json.dumps(value, indent=2) + "\n")

p = Path("docs/research/household-lifecycle-structural-sensitivity-v1.md")
v1 = p.read_text()
notice = "> **Superseded historical contract.** Audit-v2 issue #324 demonstrated that the v1 stable-PersonId partition rule sorts cohort/generation structure. Current executable treatment semantics are defined in `household-lifecycle-structural-sensitivity-v2.md`; this file remains for provenance of the original #207 comparison.\n\n"
if notice not in v1:
    v1 = v1.replace("# Household lifecycle structural sensitivity v1\n\n", "# Household lifecycle structural sensitivity v1\n\n" + notice, 1)
p.write_text(v1)

Path("docs/research/household-lifecycle-structural-sensitivity-v2.md").write_text('''# Household lifecycle structural sensitivity v2

## Status

This is a **synthetic structural-sensitivity contract**, not an ethnographic or archaeological model of household formation. It supersedes the PersonId-sliced v1 treatment after audit-v2 issue #324.

## Alternative treatment

`deterministic_age_balanced_fission_v2` is enabled through the versioned `householdLifecycle` experiment field (schema version 2). At each completed annual boundary after M2 fertility:

- only households physically at residence are eligible;
- an oversized household is divided into the minimum number of balanced groups needed to satisfy `maxLivingMembers`;
- living members are ordered by birth day (oldest first), then reproductive sex, with PersonId used only to break ties among otherwise age/sex-equivalent records;
- ordered members are distributed round-robin across groups while respecting the precomputed balanced target sizes;
- the original household retains group 0 and each remaining group becomes a new household at the same persistent residence;
- person identity, parent links, condition and persistent residence are otherwise unchanged;
- future M4/M9 decisions treat daughter households independently after topology reconciliation.

This is an explicit **age/sex-composition balancing rule**, not a claim about real household fission, adulthood, marriage, inheritance, post-marital residence or dependency. Parent-child co-residence is not guaranteed. Cross-household kin can therefore result. If fewer older/adult-like members exist than daughter groups, the model cannot invent adults and a child-only unit can still be unavoidable.

The scientific purpose is narrower: remove storage/PersonId cohort sorting from the structural-sensitivity arm while keeping the treatment deterministic, balanced in size and transparent. For any chosen adulthood threshold, if at least as many members above that threshold exist as resulting groups, distributing the oldest members first seeds every group with one of them before younger members fill capacity.

## Determinism and relabelling contract

Relabelling people while preserving unique age/sex scientific records must not change the per-household age/sex composition produced by fission. PersonId may affect only which otherwise age/sex-equivalent individual occupies a tied slot; it must not determine cohort structure. Exact repeated execution and checkpoint/resume equivalence remain required.

## Integration

M3 resource sharing, M4 permanent migration and M9 temporary mobility continue to operate at household level after fission. Because composition changes those downstream mechanisms can change scientifically; v2 therefore advances global `MODEL_SEMANTICS_ID` from v20 to v21 rather than silently reinterpreting prior results. The historical v1 reference remains preserved for provenance and must be re-run/rebaselined explicitly before a v2 comparison is treated as canonical.
''')

Path("docs/research/audit-v2/area-c-household-fission-composition-audit.py").write_text(r'''#!/usr/bin/env python3
"""Independent arithmetic checker for audit-v2 #324 household composition."""
from collections import Counter


def targets(n, ceiling):
    group_count = (n + ceiling - 1) // ceiling
    base, larger = divmod(n, group_count)
    return [base + (i < larger) for i in range(group_count)]


def legacy(records, ceiling):
    out, cursor = [], 0
    for size in targets(len(records), ceiling):
        out.append(records[cursor:cursor + size])
        cursor += size
    return out


def repaired(records, ceiling):
    sizes = targets(len(records), ceiling)
    groups = [[] for _ in sizes]
    next_group = 0
    for record in sorted(records, key=lambda r: (r[1], r[0])):
        while len(groups[next_group]) >= sizes[next_group]:
            next_group = (next_group + 1) % len(groups)
        groups[next_group].append(record)
        next_group = (next_group + 1) % len(groups)
    return groups


def counts(groups):
    return [Counter("newborn" if age == 0 else "older" for _, age in group) for group in groups]


def signature(groups):
    return sorted(tuple(sorted(age for _, age in group)) for group in groups)


records = [(f"p{i+1}", age) for i, age in enumerate([-50, -45, -40, -35, -30, 0, 0, 0, 0])]
assert counts(legacy(records, 5)) == [Counter({"older": 5}), Counter({"newborn": 4})]
assert counts(repaired(records, 5)) == [Counter({"older": 3, "newborn": 2}), Counter({"older": 2, "newborn": 2})]
unique = [(f"a{i}", age) for i, age in enumerate([-90, -80, -70, -60, -50, -40, -30, -20, -10])]
relabelled = list(reversed([(f"z{i}", age) for i, (_, age) in enumerate(unique)]))
assert signature(repaired(unique, 5)) == signature(repaired(relabelled, 5))
print("legacy cohort counts:", counts(legacy(records, 5)))
print("repaired cohort counts:", counts(repaired(records, 5)))
print("relabelled unique-age composition invariant: yes")
''')
