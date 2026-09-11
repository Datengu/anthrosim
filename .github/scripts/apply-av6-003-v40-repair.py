#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one target, found {count}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


def replace_required(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"{path}: required text not found: {old[:80]!r}")
    p.write_text(text.replace(old, new), encoding="utf-8")


# Executable repair: distinguish living external direct-parent context by persistent residence,
# the same causal spatial context consumed downstream by M4 kin utility.
replace_once(
    "crates/anthrosim-core/src/population.rs",
    """                    let parent_signature = |parent: PersonId| -> (u8, u64) {
                        if parent == PersonId::INVALID {
                            return (0, 0);
                        }
                        let Some(parent_index) = person_index(parent, person_count) else {
                            return (1, 0);
                        };
                        let position = member_positions[parent_index];
                        if position != usize::MAX {
                            (3, ranks[parent_index])
                        } else if self.is_alive_index(parent_index) {
                            (2, 0)
                        } else {
                            (1, 0)
                        }
                    };
""",
    """                    let parent_signature = |parent: PersonId| -> (u8, CellId, u64) {
                        if parent == PersonId::INVALID {
                            return (0, CellId::INVALID, 0);
                        }
                        let Some(parent_index) = person_index(parent, person_count) else {
                            return (1, CellId::INVALID, 0);
                        };
                        let position = member_positions[parent_index];
                        if position != usize::MAX {
                            (3, CellId::INVALID, ranks[parent_index])
                        } else if self.is_alive_index(parent_index) {
                            // Living direct parents outside this source household carry causal
                            // cross-household context through persistent residence: M4 consumes
                            // these same residence cells as first-degree kin anchors. Preserve
                            // that scientific distinction without introducing parent/household
                            // identity, packed-record order, or a global coupling ordinal.
                            (2, self.locations[parent_index], 0)
                        } else {
                            (1, CellId::INVALID, 0)
                        }
                    };
""",
)

replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v39";',
    """///
/// v40 extends dependency-aware household-fission relationship refinement to living direct
/// parents outside the source household. Their persistent residence cell is causal context
/// because M4 consumes the same cross-household parent/child residences as first-degree kin
/// anchors. Canonical PersonId, HouseholdId, packed-record order and global stochastic-coupling
/// rank remain excluded from this social assignment key. A v39 checkpoint must not resume under
/// v40 while silently changing which relationship-distinct adults seed daughter households.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v40";""",
)

# Current-facing semantics markers. Historical audit/release evidence stays untouched.
for path in (
    "README.md",
    "docs/architecture.md",
    "docs/roadmap.md",
    "docs/scientific-model.md",
    "docs/research/README.md",
    "docs/research/trace.md",
    "docs/research/odd.md",
    "docs/research/odd-d.md",
):
    replace_required(path, "current model semantics v39", "current model semantics v40")

# Living release/source distinction.
replace_once(
    "docs/release-versioning.md",
    "AV6-002/#694 then unified male-parent reproductive-age chronology at the child-birth boundary and advances the repair branch to **`anthrosim-model-semantics-v39`**. The package version remains `0.3.6`; these living identities do not rewrite immutable v0.3.6/v35 or its Audit-v6 discovery evidence.",
    "AV6-002/#694 then unified male-parent reproductive-age chronology at the child-birth boundary and advanced protected `main` to `anthrosim-model-semantics-v39`; AV6-003/#699 extends dependency-aware household-fission relationship refinement to living external-parent persistent-residence context and advances the active repair branch to **`anthrosim-model-semantics-v40`**. The package version remains `0.3.6`; these living identities do not rewrite immutable v0.3.6/v35 or its Audit-v6 discovery evidence.",
)
replace_once(
    "docs/release-versioning.md",
    "AV6-001 at v36, AV6-004 at v37, AV6-006 at v38, and AV6-002 at v39.",
    "AV6-001 at v36, AV6-004 at v37, AV6-006 at v38, AV6-002 at v39, and AV6-003 at v40.",
)

# Normative lifecycle/scientific contract.
replace_once(
    "docs/scientific-model.md",
    "The historical `deterministic_size_fission_v1` treatment is superseded because its stable-PersonId slicing made packed record order an unintended cohort/generation rule. Neither lifecycle treatment is an empirical household-formation claim; the current v2 contract is [`research/household-lifecycle-structural-sensitivity-v2.md`](research/household-lifecycle-structural-sensitivity-v2.md).",
    "The historical `deterministic_size_fission_v1` treatment is superseded because its stable-PersonId slicing made packed record order an unintended cohort/generation rule. From model semantics v40, living direct parents outside the source household are relationship-distinct when their persistent residence cells differ, matching the first-degree kin-location context consumed by M4; external parents at the same persistent residence remain equivalent, and external PersonId/HouseholdId, packed-record order and global stochastic-coupling rank are not social assignment keys. Neither lifecycle treatment is an empirical household-formation claim; the current v2 contract is [`research/household-lifecycle-structural-sensitivity-v2.md`](research/household-lifecycle-structural-sensitivity-v2.md).",
)

contract_path = Path("docs/research/household-lifecycle-structural-sensitivity-v2.md")
contract = contract_path.read_text(encoding="utf-8")
old_step = "7. independent-age members are assigned deterministically by age, reproductive sex and an ID-independent relationship-role refinement over the living source-household parent/child graph; PersonId is used only as a final tie-break within the same stabilized relationship class;"
new_step = "7. independent-age members are assigned deterministically by age, reproductive sex and an ID-independent relationship-role refinement over the living source-household parent/child graph plus living direct-parent context outside that household; a living external parent is distinguished only by persistent residence cell, matching the downstream M4 first-degree kin-location state, and PersonId is used only as a final tie-break within the same stabilized scientific context;"
if old_step not in contract:
    raise SystemExit("household lifecycle step-7 contract target missing")
contract = contract.replace(old_step, new_step, 1)
marker = "From model semantics v25, the relationship comparison is executable rather than documentary: living members of the source household are partitioned into age/sex role classes, then those classes are iteratively refined by female-parent state, male-parent state and the multiset of living in-household child roles until stable. Only records still in the same stabilized role class may fall through to PersonId. This keeps deterministic replay without allowing canonical record labels to choose between relationship-distinct anchors or dependents.\n"
if marker not in contract:
    raise SystemExit("household lifecycle v25 refinement paragraph missing")
contract = contract.replace(
    marker,
    marker
    + "\nFrom model semantics v40, female- and male-parent refinement also preserves one explicit cross-household context for a represented living parent: that parent's persistent residence cell. This is the same causal context M4 exposes as a reciprocal first-degree kin-location anchor after household topology changes. The refinement does **not** use the external parent's PersonId, HouseholdId, packed-record position or global stochastic-coupling rank, and two otherwise equivalent external parents at the same persistent residence remain in the same relationship context. This is a relabelling-invariance rule, not an ethnographic preference for one direction or relative.\n",
    1,
)
contract += "\nModel semantics v40 changes authoritative daughter-household membership for the AV6-003/#699 external-kin equivalence case, so v39 checkpoints are not continuation-compatible with v40. Package version remains `0.3.6`, and immutable v0.3.6/v35 Audit-v6 discovery evidence is unchanged.\n"
contract_path.write_text(contract, encoding="utf-8")

appendix = """

### Current v40 household-fission external-kin context

For the optional `deterministic_dependency_fission_v2` structural treatment, relationship refinement preserves the persistent residence cell of a represented living direct parent outside the source household. That is existing causal model state already consumed by M4 as a reciprocal first-degree kin-location anchor. It is not an empirical residence norm: external PersonId, HouseholdId, packed-record order and global stochastic-coupling rank are excluded from the social assignment key, and external parents sharing one residence remain equivalent. AV6-003/#699 therefore advances living model semantics v39 → v40 without changing package version `0.3.6` or immutable v0.3.6/v35 audit evidence.
"""
for path in ("docs/research/odd.md", "docs/research/odd-d.md", "docs/research/trace.md"):
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if "Current v40 household-fission external-kin context" not in text:
        p.write_text(text.rstrip() + appendix + "\n", encoding="utf-8")

# Keep the live audit ledger honest about protected main versus the active repair branch.
status_path = Path("docs/research/audit-v6/STATUS.md")
status = status_path.read_text(encoding="utf-8")
status_replacements = (
    (
        "| Active ownership | **AV6-002/#694 is the active first P2 remediation in PR #771; AV6-003/#699 is next unless live overlap/dependency state changes** |",
        "| Active ownership | **AV6-003/#699 is the active P2 remediation on `audit/repair-699-av6-003`; AV6-002/#694 is merged/closed via #771** |",
    ),
    (
        "Protected `main` after the verified AV6-006 production repair is `e729866f4a46d833a566a557e38570daf854d5ae`, with living model semantics `anthrosim-model-semantics-v38`. Active AV6-002 repair PR #771 advances its branch to `anthrosim-model-semantics-v39`; immutable Audit-v6 discovery remains attributed to `v0.3.6` / v35.",
        "Protected `main` after merged AV6-002/#694 is `9d9c89495f66e50f30d434255069544cd8dfa866`, with living model semantics `anthrosim-model-semantics-v39`. Active AV6-003/#699 advances this repair branch to `anthrosim-model-semantics-v40`; immutable Audit-v6 discovery remains attributed to `v0.3.6` / v35.",
    ),
    (
        "| AV6-002 | **P2** | B primary; C/G/M/N | #694 | **open — active production repair #771** |",
        "| AV6-002 | **P2** | B primary; C/G/M/N | #694 | **closed — repaired/verified by #771** |",
    ),
    (
        "| AV6-003 | **P2** | C primary; E/N | #699 | open; remediation pending |",
        "| AV6-003 | **P2** | C primary; E/N | #699 | **open — active production repair** |",
    ),
)
for old, new in status_replacements:
    if old not in status:
        raise SystemExit(f"audit-v6 status target missing: {old[:90]}")
    status = status.replace(old, new, 1)
status_path.write_text(status, encoding="utf-8")

# Promote the exact Area-C adversary construction from evidence PR #698 as a permanent regression.
test = r'''use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig,
    MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const SOURCE_HOUSEHOLD: HouseholdId = HouseholdId::new(1);
const WEST_PARENT_HOUSEHOLD: HouseholdId = HouseholdId::new(2);
const EAST_PARENT_HOUSEHOLD: HouseholdId = HouseholdId::new(3);
const WEST_PARENT: PersonId = PersonId::new(5);
const EAST_PARENT: PersonId = PersonId::new(6);
const DAYS_PER_YEAR: i64 = 365;

fn neutral_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 30, 1),
        AgeProbabilityBand::new(30, u32::MAX, 0),
    ];
    config
}

fn neutral_resources() -> ResourceConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    resources
}

fn source_adult(id: PersonId, female_parent: Option<PersonId>) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(30 * DAYS_PER_YEAR),
        reproductive_sex: ReproductiveSex::Male,
        household: SOURCE_HOUSEHOLD,
        female_parent,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn external_mother(id: PersonId, household: HouseholdId) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(55 * DAYS_PER_YEAR),
        reproductive_sex: ReproductiveSex::Female,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(swapped_linked_adult_ids: bool) -> (FounderPopulationDefinition, PersonId, PersonId) {
    let west_linked = if swapped_linked_adult_ids { PersonId::new(4) } else { PersonId::new(3) };
    let east_linked = if swapped_linked_adult_ids { PersonId::new(3) } else { PersonId::new(4) };
    let parent_for = |id: PersonId| {
        if id == west_linked {
            Some(WEST_PARENT)
        } else if id == east_linked {
            Some(EAST_PARENT)
        } else {
            None
        }
    };
    (
        FounderPopulationDefinition::new(
            if swapped_linked_adult_ids {
                "audit-v6-area-c-external-kin-relabel-swapped"
            } else {
                "audit-v6-area-c-external-kin-relabel-baseline"
            },
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![
                FounderHousehold { id: SOURCE_HOUSEHOLD, location: CellId::new(2) },
                FounderHousehold { id: WEST_PARENT_HOUSEHOLD, location: CellId::new(1) },
                FounderHousehold { id: EAST_PARENT_HOUSEHOLD, location: CellId::new(3) },
            ],
            vec![
                source_adult(PersonId::new(1), None),
                source_adult(PersonId::new(2), None),
                source_adult(PersonId::new(3), parent_for(PersonId::new(3))),
                source_adult(PersonId::new(4), parent_for(PersonId::new(4))),
                external_mother(WEST_PARENT, WEST_PARENT_HOUSEHOLD),
                external_mother(EAST_PARENT, EAST_PARENT_HOUSEHOLD),
            ],
        ),
        west_linked,
        east_linked,
    )
}

fn fission_outcome(swapped_linked_adult_ids: bool) -> (bool, bool, Vec<PersonId>) {
    let (founders, west_linked, east_linked) = founders(swapped_linked_adult_ids);
    let config = ExperimentConfig::new(6_301, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(6)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders)
        .with_demography(neutral_demography())
        .with_resources(neutral_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();
    let fissions = run
        .events()
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::HouseholdFission {
                source_household,
                residence,
                people_reassigned,
                ..
            } if *source_household == SOURCE_HOUSEHOLD => {
                assert_eq!(record.day, 365);
                assert_eq!(*residence, CellId::new(2));
                Some(people_reassigned.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fissions.len(), 1, "the source household must fission exactly once");
    let moved = fissions.into_iter().next().unwrap();
    assert_eq!(moved.len(), 2, "four adults with target two must split 2+2");
    let west_moved = moved.contains(&west_linked);
    let east_moved = moved.contains(&east_linked);
    assert_ne!(west_moved, east_moved, "exactly one externally linked adult should enter the daughter household");
    (west_moved, east_moved, moved)
}

#[test]
fn external_kin_context_is_not_reassigned_by_canonical_person_id() {
    let baseline = fission_outcome(false);
    let relabelled = fission_outcome(true);
    println!(
        "external-kin fission controls: baseline west_moved={} east_moved={} moved={:?}; relabelled west_moved={} east_moved={} moved={:?}",
        baseline.0, baseline.1, baseline.2, relabelled.0, relabelled.1, relabelled.2
    );
    assert_eq!(
        (baseline.0, baseline.1),
        (relabelled.0, relabelled.1),
        "pure canonical PersonId relabelling must not decide whether the adult linked to the west versus east external living parent retains source-household continuity"
    );
}
'''
Path("crates/anthrosim-core/tests/av6_003_household_fission_external_kin_context.rs").write_text(test, encoding="utf-8")
