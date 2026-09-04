from pathlib import Path

migration = Path('crates/anthrosim-core/src/migration.rs')
s = migration.read_text()

old_sort = '''            self.evaluations
                .sort_unstable_by_key(candidate_scientific_key);

            // A deterministic same-seed function cannot choose one unique member of a true
            // automorphism orbit equivariantly. Only deterministically eligible duplicates matter:
            // candidate uncertainty is a non-negative penalty, so an ineligible deterministic
            // candidate cannot later become selectable. Ineligible duplicate context therefore
            // must not suppress a unique scientifically better destination.
            if self.evaluations.windows(2).any(|pair| {
                pair[0].utility.total_utility > required
                    && candidate_scientific_key(&pair[0]) == candidate_scientific_key(&pair[1])
            }) {
                continue;
            }
'''
new_sort = '''            // Order scientifically distinct classes by active deterministic M4 state. CellId is
            // used only to serialize members *inside* an exact scientific equivalence class; it
            // does not decide class uncertainty assignment or cumulative class probability.
            self.evaluations.sort_unstable_by_key(|evaluation| {
                (candidate_scientific_key(evaluation), evaluation.cell)
            });
'''
if s.count(old_sort) != 1:
    raise SystemExit(f'expected one old equivalence-abstention block, found {s.count(old_sort)}')
s = s.replace(old_sort, new_sort, 1)

start_marker = '''            let mut total_weight = 0_u64;
            let mut best_candidate = CellId::INVALID;
            let mut best_candidate_utility = i32::MIN;
            let mut best_candidate_key = None;
            let mut eligible_count = 0_usize;

            // Candidate uncertainty is now assigned in scientific-key order. Under a
            // reflection/rotation/permutation, the same physical alternative therefore receives
            // the same sequential uncertainty realization even when its CellId changes.
'''
end_marker = '''            self.evaluations.truncate(eligible_count);
'''
start = s.find(start_marker)
if start < 0:
    raise SystemExit('candidate evaluation loop start not found')
end = s.find(end_marker, start)
if end < 0:
    raise SystemExit('candidate evaluation loop end not found')
end += len(end_marker)

replacement = '''            let mut total_weight = 0_u64;
            let mut best_candidate = CellId::INVALID;
            let mut best_candidate_utility = i32::MIN;
            let mut best_candidate_key = None;
            let mut eligible_count = 0_usize;

            // Assign one uncertainty realization to each exact deterministic scientific class.
            // Members of a class are indistinguishable to active M4 semantics before uncertainty,
            // so sharing that realization prevents arbitrary CellId order from attaching different
            // uncertainty draws to physically symmetric alternatives. Distinct classes retain the
            // original independent sequential uncertainty semantics in scientific-key order.
            let mut class_start = 0_usize;
            while class_start < self.evaluations.len() {
                let class_key = candidate_scientific_key(&self.evaluations[class_start]);
                let mut class_end = class_start + 1;
                while class_end < self.evaluations.len()
                    && candidate_scientific_key(&self.evaluations[class_end]) == class_key
                {
                    class_end += 1;
                }
                let uncertainty = if config.max_uncertainty_penalty_permille == 0 {
                    0
                } else {
                    u16::try_from(draw_bounded(
                        &mut rngs.uncertainty,
                        u64::from(config.max_uncertainty_penalty_permille) + 1,
                    ))
                    .unwrap_or(config.max_uncertainty_penalty_permille)
                };

                for evaluation_index in class_start..class_end {
                    let deterministic = self.evaluations[evaluation_index];
                    let scientific_key = candidate_scientific_key(&deterministic);
                    let destination_demand_population = self
                        .boundary_demand_population(deterministic.cell)?
                        .saturating_add(members);
                    let utility = self.evaluate_relocation(
                        household_index,
                        deterministic.cell,
                        deterministic.distance,
                        destination_demand_population,
                        resources,
                        world,
                        config,
                        period_need_per_person,
                        uncertainty,
                    )?;
                    let replace_best = if utility.total_utility > best_candidate_utility {
                        true
                    } else if utility.total_utility == best_candidate_utility {
                        best_candidate_key.is_none_or(|best_key| scientific_key < best_key)
                    } else {
                        false
                    };
                    if replace_best {
                        best_candidate = deterministic.cell;
                        best_candidate_utility = utility.total_utility;
                        best_candidate_key = Some(scientific_key);
                    }
                    if utility.total_utility <= required {
                        continue;
                    }
                    let improvement = i64::from(utility.total_utility) - i64::from(required);
                    // Every member keeps its original proportional improvement weight. Exact-class
                    // members therefore occupy equal subintervals of one scientifically defined
                    // class interval; CellId only names the exchangeable realized member.
                    let weight = proportional_choice_weight(improvement);
                    total_weight = total_weight
                        .checked_add(weight)
                        .ok_or(MigrationError::AccountingOverflow)?;
                    self.evaluations[eligible_count] = CandidateEvaluation {
                        cell: deterministic.cell,
                        distance: deterministic.distance,
                        utility,
                        weight,
                    };
                    eligible_count += 1;
                }
                class_start = class_end;
            }
            self.evaluations.truncate(eligible_count);
'''
s = s[:start] + replacement + s[end:]
migration.write_text(s)

# Replace the over-strong exact-symmetry abstention regression with an exchangeability/replay guard.
test = Path('crates/anthrosim-core/tests/m4_spatial_isomorphism_invariance.rs')
t = test.read_text()
old_test = '''#[test]
fn m4_does_not_invent_an_orientation_for_exactly_indistinguishable_destinations() {
    for seed in 1..=256 {
        let moves = run(seed, 3, 1, vec![900, 100, 900], 80);
        assert!(
            moves.is_empty(),
            "seed {seed}: exact left/right M4 symmetry must not be broken by CellId/container order: {moves:?}"
        );
    }
}
'''
new_test = '''#[test]
fn m4_exact_symmetry_is_exchangeable_and_replay_exact() {
    let mut left = 0_u32;
    let mut right = 0_u32;
    for seed in 1..=256 {
        let first = run(seed, 3, 1, vec![900, 100, 900], 80);
        let replay = run(seed, 3, 1, vec![900, 100, 900], 80);
        assert_eq!(first, replay, "seed {seed}: exact replay diverged");
        assert_eq!(first.len(), 1, "seed {seed}: symmetric weighted choice must remain active");
        match first[0].1 {
            cell if cell == CellId::new(1) => left += 1,
            cell if cell == CellId::new(3) => right += 1,
            other => panic!("seed {seed}: unexpected symmetric destination {other:?}"),
        }
    }
    assert!(left > 0 && right > 0, "exact symmetry must sample both exchangeable alternatives: left={left}, right={right}");
}
'''
if t.count(old_test) != 1:
    raise SystemExit(f'expected one old exact-symmetry test, found {t.count(old_test)}')
t = t.replace(old_test, new_test, 1)
test.write_text(t)

# Keep v33 but correct its scientific description.
prov = Path('crates/anthrosim-core/src/provenance.rs')
p = prov.read_text()
old_prov = '''/// v33 removes canonical spatial candidate order from M4 uncertainty and proportional-choice
/// assignment. Deterministically eligible candidates are coupled by their active deterministic
/// M4 utility and movement distance rather than CellId/container position; exact scientifically
/// indistinguishable eligible destination orbits are left unresolved instead of inventing an
/// unmodelled orientation. A v32 checkpoint must therefore not resume under v33 with unchanged
/// migration RNG positions while silently reassigning candidate uncertainty/choice draws.
'''
new_prov = '''/// v33 removes canonical spatial candidate order from M4 uncertainty and proportional-choice
/// assignment. Candidates are coupled by active deterministic M4 utility and movement distance;
/// exact deterministic equivalence classes share one uncertainty realization and retain their
/// full aggregate proportional-choice probability, with exchangeable members sampled uniformly
/// by equal subintervals of the existing choice draw. A v32 checkpoint must therefore not resume
/// under v33 with unchanged migration RNG positions while silently reassigning candidate draws.
'''
if p.count(old_prov) != 1:
    raise SystemExit(f'expected one old v33 provenance paragraph, found {p.count(old_prov)}')
p = p.replace(old_prov, new_prov, 1)
prov.write_text(p)
