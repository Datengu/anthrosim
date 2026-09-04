#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement target, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


resources = Path("crates/anthrosim-core/src/resources.rs")
replace_once(
    resources,
    "        let mut resolved_causes = vec![None; mortality_candidates.len()];\n",
    "        // Couple every mortality-stream draw and simultaneous-trigger attribution to the\n"
    "        // represented scientific person rather than canonical PersonId/record position.\n"
    "        // Persistent person stochastic-coupling ranks are globally unique.\n"
    "        let mut mortality_order = (0..mortality_candidates.len()).collect::<Vec<_>>();\n"
    "        mortality_order.sort_unstable_by_key(|&candidate_index| {\n"
    "            mortality_candidates[candidate_index].stochastic_coupling_rank\n"
    "        });\n\n"
    "        let mut resolved_causes = vec![None; mortality_candidates.len()];\n",
)
replace_once(
    resources,
    "            let mut condition_triggers = Vec::with_capacity(mortality_candidates.len());\n"
    "            for candidate in &mortality_candidates {\n"
    "                condition_triggers.push(draw_probability_fraction(\n"
    "                    scarcity_rng,\n"
    "                    candidate.condition_probability,\n"
    "                )?);\n"
    "            }\n\n"
    "            let mut background_order = (0..mortality_candidates.len()).collect::<Vec<_>>();\n"
    "            background_order.sort_unstable_by_key(|&candidate_index| {\n"
    "                let candidate = mortality_candidates[candidate_index];\n"
    "                (candidate.stochastic_coupling_rank, candidate.person_index)\n"
    "            });\n"
    "            let mut background_triggers = vec![false; mortality_candidates.len()];\n"
    "            for &candidate_index in &background_order {\n",
    "            let mut condition_triggers = vec![false; mortality_candidates.len()];\n"
    "            for &candidate_index in &mortality_order {\n"
    "                condition_triggers[candidate_index] = draw_probability_fraction(\n"
    "                    scarcity_rng,\n"
    "                    mortality_candidates[candidate_index].condition_probability,\n"
    "                )?;\n"
    "            }\n\n"
    "            let mut background_triggers = vec![false; mortality_candidates.len()];\n"
    "            for &candidate_index in &mortality_order {\n",
)
text = resources.read_text(encoding="utf-8")
if text.count("for &candidate_index in &background_order {") != 1:
    raise SystemExit("unexpected remaining background-order loop count")
resources.write_text(
    text.replace(
        "for &candidate_index in &background_order {",
        "for &candidate_index in &mortality_order {",
        1,
    ),
    encoding="utf-8",
)
replace_once(
    resources,
    "        } else {\n"
    "            for (candidate_index, candidate) in mortality_candidates.iter().enumerate() {\n"
    "                resolved_causes[candidate_index] =\n"
    "                    draw_probability_fraction(scarcity_rng, candidate.condition_probability)?\n"
    "                        .then_some(CompetingMortalityCause::ConditionMediated);\n"
    "            }\n"
    "        }\n",
    "        } else {\n"
    "            for &candidate_index in &mortality_order {\n"
    "                let candidate = mortality_candidates[candidate_index];\n"
    "                resolved_causes[candidate_index] =\n"
    "                    draw_probability_fraction(scarcity_rng, candidate.condition_probability)?\n"
    "                        .then_some(CompetingMortalityCause::ConditionMediated);\n"
    "            }\n"
    "        }\n",
)

provenance = Path("crates/anthrosim-core/src/provenance.rs")
replace_once(
    provenance,
    "/// kin role receives each parentage realization.\n"
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v29";\n',
    "/// kin role receives each parentage realization.\n"
    "///\n"
    "/// v30 removes the remaining arbitrary canonical `PersonId`/packed-record ordering from\n"
    "/// M3 condition-mediated mortality. Condition latent triggers and simultaneous-trigger\n"
    "/// cause-attribution draws now follow the same persistent person stochastic-coupling-rank\n"
    "/// schedule as background mortality while retaining independent named RNG streams and the\n"
    "/// exchange-symmetric competing-risk resolver. A v29 checkpoint must therefore not resume\n"
    "/// under v30 with unchanged mortality RNG positions.\n"
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v30";\n',
)

checkpoint = Path("crates/anthrosim-core/src/checkpoint.rs")
replace_once(
    checkpoint,
    "    pub const PRE_PARENTAGE_COUPLING_SCHEMA_VERSION: u32 = 16;\n"
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 17;\n",
    "    pub const PRE_PARENTAGE_COUPLING_SCHEMA_VERSION: u32 = 16;\n"
    "    pub const PRE_CONDITION_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 17;\n"
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 18;\n",
)

guard = Path("scripts/test-current-model-semantics-docs.py")
guard_text = guard.read_text(encoding="utf-8")
if guard_text.count('CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v29"') != 1:
    raise SystemExit("unexpected current semantics ID guard")
if guard_text.count('CURRENT_SHORT = "v29"') != 1:
    raise SystemExit("unexpected current semantics short guard")
guard_text = guard_text.replace(
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v29"',
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v30"',
    1,
)
guard_text = guard_text.replace('CURRENT_SHORT = "v29"', 'CURRENT_SHORT = "v30"', 1)
guard.write_text(guard_text, encoding="utf-8")

for name in ("docs/scientific-model.md", "docs/research/odd.md", "docs/research/odd-d.md"):
    path = Path(name)
    text = path.read_text(encoding="utf-8")
    if text.count("current model semantics v29") != 1:
        raise SystemExit(f"{name}: unexpected current semantics label count")
    path.write_text(
        text.replace("current model semantics v29", "current model semantics v30", 1),
        encoding="utf-8",
    )

scientific = Path("docs/scientific-model.md")
replace_once(
    scientific,
    "M3 resource settlement and condition response occur at the configured M3 interval ends. Mortality is then resolved at those same elapsed interval ends as an order-invariant competition between the M3 condition-mediated cause and the M2 background cause. Under v27, the background cause's latent RNG trigger is assigned to living people by the persisted scientific stochastic-coupling rank rather than canonical `PersonId` record position. Condition-mediated trigger assignment remains on its independently tracked pre-v27 stream ordering until AV4-006 is remediated; this does not reintroduce first-called cause priority because both latent triggers are sampled before any simultaneous-trigger attribution. Simultaneous triggers continue to use the exchange-symmetric proportional tie rule.",
    "M3 resource settlement and condition response occur at the configured M3 interval ends. Mortality is then resolved at those same elapsed interval ends as an order-invariant competition between the M3 condition-mediated cause and the M2 background cause. Under v30, both causes' latent triggers are assigned to living people in persistent scientific stochastic-coupling-rank order rather than canonical `PersonId`/record position, while the condition and background causes retain their independent named RNG streams. Simultaneous-trigger cause attribution follows that same scientific person schedule and continues to use the exchange-symmetric proportional tie rule.",
)

odd = Path("docs/research/odd.md")
replace_once(
    odd,
    "- M3 condition recovery/loss coefficients and the condition-mediated mortality probability are interpreted against four fixed reference-quarter intervals, then converted deterministically to the actual elapsed M3 interval. Changing only `P` therefore does not multiply the complete-year response budget or fixed-condition survival probability merely by creating more M3 boundaries.\n",
    "- M3 condition recovery/loss coefficients and the condition-mediated mortality probability are interpreted against four fixed reference-quarter intervals, then converted deterministically to the actual elapsed M3 interval. Changing only `P` therefore does not multiply the complete-year response budget or fixed-condition survival probability merely by creating more M3 boundaries. Under v30, both condition-mediated and background latent mortality triggers, plus simultaneous-trigger attribution, are coupled to persistent person stochastic-coupling-rank order rather than canonical `PersonId`/record order; the two named RNG streams and symmetric competing-risk mathematics remain distinct.\n",
)
