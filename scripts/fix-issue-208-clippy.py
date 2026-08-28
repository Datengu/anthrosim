from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path, old, new):
    file = ROOT / path
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old!r}")
    file.write_text(text.replace(old, new, 1))


replace_once(
    "crates/anthrosim-core/src/resources.rs",
    "use rand::Rng;\n",
    "",
)
replace_once(
    "crates/anthrosim-core/src/resources.rs",
    "    pub(crate) fn process_period_recorded_with_presence(\n",
    "    #[cfg(test)]\n    pub(crate) fn process_period_recorded_with_presence(\n",
)
replace_once(
    "crates/anthrosim-core/src/demography.rs",
    "pub(crate) fn process_demographic_year_recorded(\n",
    "#[cfg(test)]\npub(crate) fn process_demographic_year_recorded(\n",
)
replace_once(
    "crates/anthrosim-core/src/demography.rs",
    "pub(crate) fn process_demographic_year_recorded_with_founder_history(\n",
    "#[cfg(test)]\npub(crate) fn process_demographic_year_recorded_with_founder_history(\n",
)
replace_once(
    "crates/anthrosim-core/src/demography.rs",
    "fn process_demographic_year_recorded_internal(\n",
    "#[allow(clippy::too_many_arguments)]\nfn process_demographic_year_recorded_internal(\n",
)
replace_once(
    "crates/anthrosim-core/src/mortality.rs",
    "/// symmetrically in proportion to the two cause-specific interval risks. The tie draw combines one\n/// draw from each stream with XOR, so exchanging the two cause labels/streams exchanges the\n/// attribution but cannot create a first-called advantage.\n",
    "/// symmetrically in proportion to the two cause-specific interval risks. The tie allocator first\n/// derives an exchange-reversing orientation from an independent draw pair, then combines later\n/// draws with XOR. Exchanging cause labels/streams therefore complements the bounded tie draw and\n/// exchanges attribution exactly; neither argument position can create a first-called advantage.\n",
)
replace_once(
    "crates/anthrosim-core/src/mortality.rs",
    "fn draw_symmetric_bounded(\n    left: &mut ChaCha8Rng,\n    right: &mut ChaCha8Rng,\n    upper_exclusive: u64,\n) -> u64 {\n    debug_assert!(upper_exclusive > 0);\n    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);\n    loop {\n        // XOR is commutative and preserves a uniform word when the two named streams are\n        // independent. The tie allocator therefore has no left/right or call-order preference.\n        let draw = left.next_u64() ^ right.next_u64();\n        if draw < acceptance_limit {\n            return draw % upper_exclusive;\n        }\n    }\n}\n",
    "fn draw_symmetric_bounded(\n    left: &mut ChaCha8Rng,\n    right: &mut ChaCha8Rng,\n    upper_exclusive: u64,\n) -> u64 {\n    debug_assert!(upper_exclusive > 0);\n\n    // A commutative XOR alone gives the same bounded draw after exchanging the two streams.\n    // Weighted attribution needs the stronger property d(right,left) = upper - 1 - d(left,right)\n    // so that swapping cause weights swaps the selected cause exactly. Use one independent draw\n    // pair only to choose orientation; equality consumes another pair symmetrically.\n    let left_is_lower = loop {\n        let left_order = left.next_u64();\n        let right_order = right.next_u64();\n        if left_order != right_order {\n            break left_order < right_order;\n        }\n    };\n\n    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);\n    loop {\n        // These later draws are independent of the orientation pair. XOR is commutative, so the\n        // base value is identical after exchanging streams; the orientation then complements it.\n        let draw = left.next_u64() ^ right.next_u64();\n        if draw < acceptance_limit {\n            let base = draw % upper_exclusive;\n            return if left_is_lower {\n                base\n            } else {\n                upper_exclusive - 1 - base\n            };\n        }\n    }\n}\n",
)
print("issue 208 clippy and exchange-symmetry cleanup applied")
