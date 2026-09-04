from pathlib import Path

p = Path('crates/anthrosim-core/src/migration.rs')
s = p.read_text()
old = '''            fill_candidate_cells(
                &mut self.candidates,
                world,
                origin,
                config.candidate_radius_cells,
            );
            self.evaluations.clear();
            let mut total_weight = 0_u64;
            let mut best_candidate = CellId::INVALID;
            let mut best_candidate_utility = i32::MIN;

            for &candidate in &self.candidates {
                let distance = manhattan_distance(world, origin, candidate).ok_or(
                    MigrationError::InternalInvariant("candidate coordinates invalid"),
                )?;
                let destination_demand_population = self
                    .boundary_demand_population(candidate)?
                    .saturating_add(members);
                let uncertainty = if config.max_uncertainty_penalty_permille == 0 {
                    0
                } else {
                    u16::try_from(draw_bounded(
                        &mut rngs.uncertainty,
                        u64::from(config.max_uncertainty_penalty_permille) + 1,
                    ))
                    .unwrap_or(config.max_uncertainty_penalty_permille)
                };
                let utility = self.evaluate_relocation(
                    household_index,
                    candidate,
                    distance,
                    destination_demand_population,
                    resources,
                    world,
                    config,
                    period_need_per_person,
                    uncertainty,
                )?;
                if utility.total_utility > best_candidate_utility
                    || (utility.total_utility == best_candidate_utility
                        && (best_candidate == CellId::INVALID || candidate < best_candidate))
                {
                    best_candidate = candidate;
                    best_candidate_utility = utility.total_utility;
                }
                let required = origin_utility.total_utility.saturating_add(
                    i32::try_from(config.minimum_utility_improvement).unwrap_or(i32::MAX),
                );
                if utility.total_utility <= required {
                    continue;
                }
                let improvement = i64::from(utility.total_utility) - i64::from(required);
                // Strict eligibility above guarantees a positive improvement, so the
                // stochastic weight is exactly proportional to declared utility improvement.
                let weight = proportional_choice_weight(improvement);
                total_weight = total_weight
                    .checked_add(weight)
                    .ok_or(MigrationError::AccountingOverflow)?;
                self.evaluations.push(CandidateEvaluation {
                    cell: candidate,
                    distance,
                    utility,
                    weight,
                });
            }

            if total_weight == 0 {
                continue;
            }
'''
new = '''            fill_candidate_cells(
                &mut self.candidates,
                world,
                origin,
                config.candidate_radius_cells,
            );
            self.evaluations.clear();
            let required = origin_utility.total_utility.saturating_add(
                i32::try_from(config.minimum_utility_improvement).unwrap_or(i32::MAX),
            );

            // First evaluate every destination without candidate uncertainty. The coupling key
            // contains only deterministic quantities that can affect this M4 decision/outcome:
            // total deterministic utility drives eligibility/weight, and distance drives the
            // movement consequence. Raw utility components whose configured weights are zero are
            // deliberately excluded so scientifically inactive metadata cannot invent orientation.
            for &candidate in &self.candidates {
                let distance = manhattan_distance(world, origin, candidate).ok_or(
                    MigrationError::InternalInvariant("candidate coordinates invalid"),
                )?;
                let destination_demand_population = self
                    .boundary_demand_population(candidate)?
                    .saturating_add(members);
                let deterministic_utility = self.evaluate_relocation(
                    household_index,
                    candidate,
                    distance,
                    destination_demand_population,
                    resources,
                    world,
                    config,
                    period_need_per_person,
                    0,
                )?;
                self.evaluations.push(CandidateEvaluation {
                    cell: candidate,
                    distance,
                    utility: deterministic_utility,
                    weight: 0,
                });
            }
            self.evaluations
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

            let mut total_weight = 0_u64;
            let mut best_candidate = CellId::INVALID;
            let mut best_candidate_utility = i32::MIN;
            let mut best_candidate_key = None;
            let mut eligible_count = 0_usize;

            // Candidate uncertainty is now assigned in scientific-key order. Under a
            // reflection/rotation/permutation, the same physical alternative therefore receives
            // the same sequential uncertainty realization even when its CellId changes.
            for evaluation_index in 0..self.evaluations.len() {
                let deterministic = self.evaluations[evaluation_index];
                let scientific_key = candidate_scientific_key(&deterministic);
                let uncertainty = if config.max_uncertainty_penalty_permille == 0 {
                    0
                } else {
                    u16::try_from(draw_bounded(
                        &mut rngs.uncertainty,
                        u64::from(config.max_uncertainty_penalty_permille) + 1,
                    ))
                    .unwrap_or(config.max_uncertainty_penalty_permille)
                };
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
                // Strict eligibility above guarantees a positive improvement, so the
                // stochastic weight is exactly proportional to declared utility improvement.
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
            self.evaluations.truncate(eligible_count);

            if total_weight == 0 {
                continue;
            }
'''
if s.count(old) != 1:
    raise SystemExit(f'expected exactly one M4 candidate-selection block, found {s.count(old)}')
s = s.replace(old, new, 1)

anchor = '''fn proportional_choice_weight(improvement: i64) -> u64 {
'''
helper = '''fn candidate_scientific_key(evaluation: &CandidateEvaluation) -> (i32, u16) {
    (evaluation.utility.total_utility, evaluation.distance)
}

'''
if s.count(anchor) != 1:
    raise SystemExit(f'expected one proportional-choice anchor, found {s.count(anchor)}')
s = s.replace(anchor, helper + anchor, 1)
p.write_text(s)
