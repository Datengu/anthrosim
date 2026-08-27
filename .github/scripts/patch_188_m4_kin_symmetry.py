from pathlib import Path


def replace_exact(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one match, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Core M4 representation: retain every unique living direct-parent cell.  A parent is a kin
# anchor regardless of reproductive-sex role or whether they are co-resident with the household.
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "const MAX_KIN_LOCATIONS_PER_HOUSEHOLD: usize = 4;\n",
    "",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "    kin_locations: Vec<[CellId; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]>,\n    kin_location_counts: Vec<u8>,\n",
    "    /// Every unique cell containing a living direct parent of a living household member.\n"
    "    ///\n"
    "    /// Co-resident parents are intentionally retained: excluding them would make the\n"
    "    /// female-parent household inheritance rule an accidental sex-specific M4 preference.\n"
    "    /// There is no record-order-dependent truncation; the set is bounded by represented\n"
    "    /// living direct-parent relationships in the authoritative population.\n"
    "    kin_locations: Vec<Vec<CellId>>,\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "            kin_locations: vec![[CellId::INVALID; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]; households],\n            kin_location_counts: vec![0; households],\n",
    "            kin_locations: vec![Vec::new(); households],\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "        if self.living_members.len() != population.household_count()\n            || self.cell_living.len() != world.cell_count()\n            || self.boundary_demand_living.len() != world.cell_count()\n",
    "        if self.living_members.len() != population.household_count()\n            || self.kin_locations.len() != population.household_count()\n            || self.cell_living.len() != world.cell_count()\n            || self.boundary_demand_living.len() != world.cell_count()\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "        self.kin_locations\n            .fill([CellId::INVALID; MAX_KIN_LOCATIONS_PER_HOUSEHOLD]);\n        self.kin_location_counts.fill(0);\n",
    "        for locations in &mut self.kin_locations {\n            locations.clear();\n        }\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "                self.note_external_kin_location(population, household_index, household, parent)?;\n",
    "                self.note_kin_location(population, household_index, parent)?;\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "    fn note_external_kin_location(\n        &mut self,\n        population: &Population,\n        household_index: usize,\n        household: HouseholdId,\n        parent: PersonId,\n    ) -> Result<(), MigrationError> {\n        if parent == PersonId::INVALID {\n            return Ok(());\n        }\n        let Some(parent_snapshot) = population.person(parent) else {\n            return Ok(());\n        };\n        if !parent_snapshot.is_alive() || parent_snapshot.household == household {\n            return Ok(());\n        }\n        let count = usize::from(self.kin_location_counts[household_index]);\n        let locations = &mut self.kin_locations[household_index];\n        if locations[..count].contains(&parent_snapshot.location) {\n            return Ok(());\n        }\n        if count < MAX_KIN_LOCATIONS_PER_HOUSEHOLD {\n            locations[count] = parent_snapshot.location;\n            self.kin_location_counts[household_index] =\n                self.kin_location_counts[household_index].saturating_add(1);\n        }\n        Ok(())\n    }\n",
    "    fn note_kin_location(\n        &mut self,\n        population: &Population,\n        household_index: usize,\n        parent: PersonId,\n    ) -> Result<(), MigrationError> {\n        if parent == PersonId::INVALID {\n            return Ok(());\n        }\n        let Some(parent_snapshot) = population.person(parent) else {\n            return Ok(());\n        };\n        if !parent_snapshot.is_alive() {\n            return Ok(());\n        }\n        let locations = &mut self.kin_locations[household_index];\n        if !locations.contains(&parent_snapshot.location) {\n            locations.push(parent_snapshot.location);\n        }\n        Ok(())\n    }\n",
)
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "        let kin_count = usize::from(self.kin_location_counts[household_index]);\n        let kin_matches = self.kin_locations[household_index][..kin_count]\n            .iter()\n            .filter(|&&kin_cell| kin_cell == cell)\n            .count();\n        let kin_score = u16::try_from(kin_matches.saturating_mul(250))\n            .unwrap_or(PERMILLE_MAX)\n            .min(PERMILLE_MAX);\n",
    "        let kin_score = if self.kin_locations[household_index].contains(&cell) {\n            250\n        } else {\n            0\n        };\n",
)

# Tests use exact declared founder states so parent role, household membership and person-record
# ordering can be varied independently while holding the represented kin graph controlled.
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "        config::{ExperimentConfig, PopulationConfig, ResourceConfig, WorldConfig},\n        focal_region::{FocalRegion, FocalRegionSource},\n        population::Population,\n",
    "        config::{\n            ExperimentConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,\n            ResourceConfig, WorldConfig,\n        },\n        focal_region::{FocalRegion, FocalRegionSource},\n        founder_initialization::{\n            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,\n        },\n        population::{Population, ReproductiveSex},\n",
)

tests = r'''

    fn declared_parent_role_fixture(world: &World, internal_parent_is_female: bool) -> Population {
        let origin = world.cell_id(0, 0).unwrap();
        let external = world.cell_id(1, 0).unwrap();
        let household_one = HouseholdId::new(1);
        let household_two = HouseholdId::new(2);
        let internal_parent = PersonId::new(1);
        let external_parent = PersonId::new(2);
        let child = PersonId::new(3);

        let (internal_sex, external_sex, female_parent, male_parent) =
            if internal_parent_is_female {
                (
                    ReproductiveSex::Female,
                    ReproductiveSex::Male,
                    Some(internal_parent),
                    Some(external_parent),
                )
            } else {
                (
                    ReproductiveSex::Male,
                    ReproductiveSex::Female,
                    Some(external_parent),
                    Some(internal_parent),
                )
            };

        let definition = FounderPopulationDefinition::new(
            "m4-parent-role-symmetry",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![
                FounderHousehold {
                    id: household_one,
                    location: origin,
                },
                FounderHousehold {
                    id: household_two,
                    location: external,
                },
            ],
            vec![
                FounderPerson {
                    id: internal_parent,
                    birth_day: -18_250,
                    reproductive_sex: internal_sex,
                    household: household_one,
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: PERMILLE_MAX,
                },
                FounderPerson {
                    id: external_parent,
                    birth_day: -18_250,
                    reproductive_sex: external_sex,
                    household: household_two,
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: PERMILLE_MAX,
                },
                FounderPerson {
                    id: child,
                    birth_day: -7_300,
                    reproductive_sex: ReproductiveSex::Male,
                    household: household_one,
                    female_parent,
                    male_parent,
                    last_birth_day: None,
                    condition_permille: PERMILLE_MAX,
                },
            ],
        );
        Population::initialize_declared_founder_state_v1(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
            &definition,
            world,
        )
        .unwrap()
    }

    #[test]
    fn co_resident_and_external_parent_roles_are_symmetric_kin_anchors() {
        let factory = RngFactory::new(188_001);
        let world = World::generate(WorldConfig::new(2, 1), factory).unwrap();
        let origin = world.cell_id(0, 0).unwrap();
        let external = world.cell_id(1, 0).unwrap();
        let config = MigrationConfig::synthetic_validation_v1();

        for internal_parent_is_female in [true, false] {
            let population = declared_parent_role_fixture(&world, internal_parent_is_female);
            let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();
            migration.prepare_snapshot(&population, &world, None).unwrap();

            let anchors = &migration.kin_locations[0];
            assert_eq!(anchors.len(), 2);
            assert!(anchors.contains(&origin));
            assert!(anchors.contains(&external));
        }
    }

    fn many_parent_locations_fixture(world: &World, pair_order: [usize; 5]) -> Population {
        let mut households = Vec::with_capacity(11);
        households.push(FounderHousehold {
            id: HouseholdId::new(1),
            location: world.cell_id(0, 0).unwrap(),
        });
        for index in 0..10 {
            households.push(FounderHousehold {
                id: HouseholdId::new(index as u64 + 2),
                location: world.cell_id(index as u32 + 1, 0).unwrap(),
            });
        }

        let mut people = Vec::with_capacity(15);
        for index in 0..5 {
            people.push(FounderPerson {
                id: PersonId::new(index as u64 + 1),
                birth_day: -18_250,
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(index as u64 + 2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: PERMILLE_MAX,
            });
        }
        for index in 0..5 {
            people.push(FounderPerson {
                id: PersonId::new(index as u64 + 6),
                birth_day: -18_250,
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(index as u64 + 7),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: PERMILLE_MAX,
            });
        }
        for (child_index, pair_index) in pair_order.into_iter().enumerate() {
            people.push(FounderPerson {
                id: PersonId::new(child_index as u64 + 11),
                birth_day: -7_300,
                reproductive_sex: if child_index.is_multiple_of(2) {
                    ReproductiveSex::Female
                } else {
                    ReproductiveSex::Male
                },
                household: HouseholdId::new(1),
                female_parent: Some(PersonId::new(pair_index as u64 + 1)),
                male_parent: Some(PersonId::new(pair_index as u64 + 6)),
                last_birth_day: None,
                condition_permille: PERMILLE_MAX,
            });
        }

        let definition = FounderPopulationDefinition::new(
            "m4-record-order-invariance",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            households,
            people,
        );
        Population::initialize_declared_founder_state_v1(
            PopulationConfig::new(15)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
            &definition,
            world,
        )
        .unwrap()
    }

    #[test]
    fn all_parent_locations_are_retained_independent_of_person_record_order() {
        let factory = RngFactory::new(188_002);
        let world = World::generate(WorldConfig::new(11, 1), factory).unwrap();
        let config = MigrationConfig::synthetic_validation_v1();
        let forward = many_parent_locations_fixture(&world, [0, 1, 2, 3, 4]);
        let reverse = many_parent_locations_fixture(&world, [4, 3, 2, 1, 0]);

        let mut forward_migration = MigrationSystem::initialize(&forward, &world, &config).unwrap();
        forward_migration
            .prepare_snapshot(&forward, &world, None)
            .unwrap();
        let mut reverse_migration = MigrationSystem::initialize(&reverse, &world, &config).unwrap();
        reverse_migration
            .prepare_snapshot(&reverse, &world, None)
            .unwrap();

        let mut forward_anchors = forward_migration.kin_locations[0].clone();
        let mut reverse_anchors = reverse_migration.kin_locations[0].clone();
        forward_anchors.sort_unstable();
        reverse_anchors.sort_unstable();
        assert_eq!(forward_anchors.len(), 10);
        assert_eq!(forward_anchors, reverse_anchors);
        for x in 1..=10 {
            assert!(forward_anchors.contains(&world.cell_id(x, 0).unwrap()));
        }
    }

    #[test]
    fn kin_weight_alone_rewards_a_living_direct_parent_cell() {
        let factory = RngFactory::new(188_003);
        let world = World::generate(WorldConfig::new(3, 1), factory).unwrap();
        let origin = world.cell_id(0, 0).unwrap();
        let kin_destination = world.cell_id(1, 0).unwrap();
        let non_kin_destination = world.cell_id(2, 0).unwrap();
        let definition = FounderPopulationDefinition::new(
            "m4-kin-only-utility",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![
                FounderHousehold {
                    id: HouseholdId::new(1),
                    location: origin,
                },
                FounderHousehold {
                    id: HouseholdId::new(2),
                    location: kin_destination,
                },
            ],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -18_250,
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(2),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: PERMILLE_MAX,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -7_300,
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: Some(PersonId::new(1)),
                    last_birth_day: None,
                    condition_permille: PERMILLE_MAX,
                },
            ],
        );
        let population = Population::initialize_declared_founder_state_v1(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
            &definition,
            &world,
        )
        .unwrap();
        let resources =
            ResourceSystem::initialize(&world, &ResourceConfig::synthetic_validation_v1()).unwrap();
        let mut config = MigrationConfig::synthetic_validation_v1();
        config.resource_weight = 0;
        config.water_security_weight = 0;
        config.kin_weight = 1;
        config.travel_cost_weight = 0;
        config.max_uncertainty_penalty_permille = 0;
        config.relocation_risk_base_penalty_permille = 0;
        config.relocation_risk_per_cell_permille = 0;
        let mut migration = MigrationSystem::initialize(&population, &world, &config).unwrap();
        migration.prepare_snapshot(&population, &world, None).unwrap();

        let period_need = 25;
        let stay = migration
            .evaluate_stay(0, origin, 1, &resources, &world, &config, period_need)
            .unwrap();
        let kin = migration
            .evaluate_relocation(
                0,
                kin_destination,
                1,
                2,
                &resources,
                &world,
                &config,
                period_need,
                0,
            )
            .unwrap();
        let non_kin = migration
            .evaluate_relocation(
                0,
                non_kin_destination,
                2,
                1,
                &resources,
                &world,
                &config,
                period_need,
                0,
            )
            .unwrap();

        assert_eq!(stay.kin_score_permille, 0);
        assert_eq!(non_kin.kin_score_permille, 0);
        assert_eq!(kin.kin_score_permille, 250);
        assert_eq!(kin.total_utility - non_kin.total_utility, 250);
        assert!(kin.total_utility > stay.total_utility);
    }
'''
replace_exact(
    "crates/anthrosim-core/src/migration.rs",
    "\n    #[test]\n    fn same_seed_and_state_produce_identical_moves() {\n",
    tests + "\n    #[test]\n    fn same_seed_and_state_produce_identical_moves() {\n",
)

# This changes authoritative M4 utility and therefore checkpoint-continuation scientific meaning.
replace_exact(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v12";\n',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v13";\n',
)

# Normative M4 documentation.
replace_exact(
    "docs/research/migration-v0.1.md",
    "| Kin score | Presence of a bounded set of known, living direct-parent locations outside the household | Minimal genealogical proxy |\n",
    "| Kin score | Presence of any unique cell containing a known living direct parent of a living household member, including co-resident parents | Minimal genealogical proxy |\n",
)
replace_exact(
    "docs/research/migration-v0.1.md",
    "## Kin scope\n\nM4 uses only genealogical information already present in the model. For a household, the first implementation records up to four unique cells containing living direct parents of living household members when those parents reside outside the household. A candidate receives a bounded kin-proximity contribution when it matches one of those cells.\n\nThis is deliberately narrow. It is **not** a model of clans, descent groups, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial communities or culturally defined kin obligations. Those would require additional social state and evidence.\n",
    "## Kin scope\n\nM4 uses only genealogical information already present in the model. For a household, the v13 kin contract collects **every unique cell containing a living direct parent of a living household member**. A living parent counts regardless of reproductive-sex role and regardless of whether that parent shares the moving household. Co-resident parents therefore contribute a kin anchor at the current residence, while parents living elsewhere can contribute anchors at candidate residences.\n\nThe score is deliberately a binary per-cell presence proxy: a residence receives `250` kin-score permille when at least one represented living direct parent is at that cell and `0` otherwise. Multiple direct parents at the same cell do not stack. All unique represented parent cells are retained; there is no fixed first-N cap, so packed person-record/birth order cannot decide which kin locations exist in M4 utility.\n\nThis symmetric rule is important because newborns normally join the female parent's household. The earlier external-parent-only rule silently removed the co-resident female-parent side in ordinary model-generated families and therefore made an apparently neutral kin term behave predominantly like an external male-parent/father-location signal. v13 removes that accidental interaction rather than adding a sex-specific social rule. Reproductive sex still has its limited M2 biological meaning and is **not** a model of social gender, patrilocality or descent.\n\nThis is deliberately narrow. It is **not** a model of clans, descent groups, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial communities or culturally defined kin obligations. Those would require additional social state and evidence. The normative symmetry and ordering contract is [`m4-kin-proxy-v1.md`](m4-kin-proxy-v1.md).\n",
)
replace_exact(
    "docs/research/migration-v0.1.md",
    "- changing only M3 resource-period count does not multiply the configured M4 decision-opportunity count;\n",
    "- changing only M3 resource-period count does not multiply the configured M4 decision-opportunity count;\n"
    "- co-resident and external living direct-parent locations are treated symmetrically by M4 regardless of female/male parent role;\n"
    "- permuting otherwise-equivalent person/birth record order cannot remove or substitute represented kin-location anchors;\n",
)

# ODD 2020 / ODD+D summaries are updated at the model-description boundary, without rewriting the
# v10 condition-mortality references that still identify that separate historical contract.
replace_exact(
    "docs/research/odd.md",
    "**AnthroSim baseline:** v0.3.0 package / post-M9 scientific-hardening line / model semantics v10  \n",
    "**AnthroSim baseline:** v0.3.0 package / post-M9 scientific-hardening line / model semantics v13  \n",
)
replace_exact(
    "docs/research/odd.md",
    "M4 uses an explicit synthetic bounded utility comparison. Resource support, water/security and a narrow direct-parent/kin proxy are treated as residence-state terms.",
    "M4 uses an explicit synthetic bounded utility comparison. Resource support, water/security and a symmetric living-direct-parent location proxy are treated as residence-state terms. The kin proxy includes co-resident and external living direct parents without a fixed record-order-dependent cap; reproductive-sex role does not decide whether a represented parent location contributes.",
)
replace_exact(
    "docs/research/odd.md",
    "- a narrow genealogical/parent-location contribution to M4 utility, including declared living direct-parent state available from day 0 when supplied;\n",
    "- a narrow symmetric living-direct-parent-location contribution to M4 utility, including co-resident and external declared parent state available from day 0 when supplied;\n",
)
replace_exact(
    "docs/research/odd.md",
    "Primary specifications: [`migration-v0.1.md`](migration-v0.1.md), [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md), [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).  \n",
    "Primary specifications: [`migration-v0.1.md`](migration-v0.1.md), [`m4-kin-proxy-v1.md`](m4-kin-proxy-v1.md), [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md), [`m3-condition-mortality-contract-v1.md`](m3-condition-mortality-contract-v1.md).  \n",
)

replace_exact(
    "docs/research/odd-d.md",
    "**AnthroSim baseline:** v0.3.0 / completed M9 / post-M9 v10 scientific-hardening semantics  \n",
    "**AnthroSim baseline:** v0.3.0 / completed M9 / post-M9 v13 scientific-hardening semantics  \n",
)
replace_exact(
    "docs/research/odd-d.md",
    "| M4 kin contribution | Narrow direct-parent-location proxy; declared founder parent state can exist from the first boundary, but the proxy is not a theory of kinship, alliance or social obligation. |\n",
    "| M4 kin contribution | Symmetric living-direct-parent-location proxy: co-resident and external parents contribute by location regardless of reproductive-sex role, with no first-record truncation; not a theory of kinship, alliance or social obligation. |\n",
)
replace_exact(
    "docs/research/odd-d.md",
    "- narrow kin/parent-location proxy.\n",
    "- narrow symmetric living-direct-parent-location proxy, including co-resident and external parents.\n",
)
replace_exact(
    "docs/research/odd-d.md",
    "- direct-parent location used by the narrow kin proxy;\n",
    "- co-resident and external living direct-parent locations used symmetrically by the narrow kin proxy;\n",
)

replace_exact(
    "docs/scientific-model.md",
    "**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v10\n",
    "**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v13\n",
)
replace_exact(
    "docs/scientific-model.md",
    "  + bounded direct-parent-location score × kin weight\n",
    "  + symmetric living-direct-parent-location score × kin weight\n",
)
replace_exact(
    "docs/scientific-model.md",
    "### Kin proximity\n\nM4 uses only genealogical state that already exists in the model. For each household, it can retain a small bounded set of cells containing living direct parents of living household members when those parents reside outside the household. A candidate receives a limited kin contribution when it matches one of those cells.\n\nThis is deliberately narrow. It is not a model of clans, lineages, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial groups or culturally defined obligations.\n",
    "### Kin proximity\n\nM4 uses only genealogical state that already exists in the model. Under the v13 kin contract, each household retains every unique cell containing a living direct parent of a living household member. Co-resident and external parents are treated identically by the collector, and female/male reproductive-parent role does not decide whether a location contributes. A cell receives the bounded kin contribution when at least one represented living direct parent is there; multiple parents at the same cell do not stack.\n\nThere is no fixed first-N kin-location cap. This makes M4 utility invariant to irrelevant packed person/birth record ordering and prevents the M2 female-parent household-inheritance rule from silently turning the supposedly neutral M4 term into an external-father preference. The detailed normative contract is [`research/m4-kin-proxy-v1.md`](research/m4-kin-proxy-v1.md).\n\nThis is deliberately narrow. It is not a model of clans, lineages, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial groups or culturally defined obligations.\n",
)

# Add the new normative contract as part of the same production commit.
Path("docs/research/m4-kin-proxy-v1.md").write_text(r'''# M4 living-direct-parent kin proxy v1

**Status:** normative post-M9 scientific-hardening contract  
**Model semantics:** `anthrosim-model-semantics-v13`  
**Scientific status:** synthetic / unvalidated

## Purpose

M4 has a deliberately narrow genealogical residence term so a represented direct-parent relationship can affect permanent-migration utility without introducing a general social-network or kinship model. This contract defines that term so reproductive-sex role and packed person-record order cannot become accidental social rules.

## Authoritative rule

At each M4 decision boundary, for each household:

1. inspect every living household member;
2. inspect both represented direct-parent links (`female_parent` and `male_parent`);
3. ignore a missing/invalid parent link and a parent who is no longer living;
4. otherwise retain the parent's current persistent residence cell, **including when the parent belongs to the same household**;
5. deduplicate locations, but do not truncate the resulting set according to encounter order.

For any residence cell `c`:

```text
kinScore(c) = 250  if at least one retained living direct-parent location == c
              0    otherwise
```

Multiple parents at one cell do not stack. The configured `migration.kinWeight` multiplies this score in the ordinary M4 residence utility.

## Why co-resident parents count

Model-born children join the female parent's persistent household. Under the pre-v13 rule, M4 discarded every parent in the moving household and then retained at most the first four external parent locations. In normal model-generated families that made the female parent structurally unable to provide an external anchor while a male parent from another household could do so. The declared gender-neutral/direct-parent description therefore hid an effectively paternal spatial preference.

The v13 rule does not add a maternal, paternal, patrilocal, matrilocal or descent-system assumption. It removes the household-membership filter from the kin concept: if a represented living direct parent is at a cell, that cell is a direct-parent location whether the parent is co-resident or external.

A co-resident parent can consequently support the explicit **stay** utility at the current residence. An external parent can support a candidate residence. This is intentional symmetry, not an attempt to force movement toward kin.

## Record-order invariance

The kin-location set contains every unique represented living direct-parent cell. It has no first-four or first-N selection rule. Therefore reordering otherwise-equivalent person/birth records cannot cause a later parent location to disappear merely because another relationship happened to be encountered first.

The transient vector order used while collecting cells has no scientific meaning. M4 asks only whether an evaluated cell is present in the complete unique set.

## Scope and non-claims

This proxy is intentionally minimal. It does **not** represent:

- clans, lineages or bilateral kindreds;
- marriage, residence rules or descent systems;
- friendship, exchange, alliance or political obligation;
- culturally differentiated maternal versus paternal ties;
- kin-distance decay, relationship strength or household fission;
- empirical prehistoric mobility preferences.

The synthetic default `kinWeight` remains a mechanism-testing value, not a measured social coefficient. A study that interprets kin-sensitive migration must evidence-ground or structurally sensitivity-test an appropriate social model rather than treating this null proxy as anthropology.

## Verification invariants

The implementation must prove with controlled tests that:

- a living co-resident female parent and a living co-resident male parent are both valid parent-location anchors under otherwise equivalent declared state;
- external female and male direct parents are handled by the same collector rule;
- more than four unique parent locations remain represented;
- changing only irrelevant person-record/child insertion order cannot change the represented kin-location set or its cell-wise utility;
- with all non-kin attraction/action terms neutralized, adding a represented living direct parent at a candidate cell increases that candidate's utility by exactly the configured kin contribution.

These are model-verification claims only. They do not validate the proxy against archaeological or ethnographic evidence.
''')
