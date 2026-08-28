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
    "crates/anthrosim-core/src/mortality.rs",
    "/// symmetrically in proportion to the two cause-specific interval risks. The tie allocator first\n/// derives an exchange-reversing orientation from an independent draw pair, then combines later\n/// draws with XOR. Exchanging cause labels/streams therefore complements the bounded tie draw and\n/// exchanges attribution exactly; neither argument position can create a first-called advantage.\n",
    "/// symmetrically in proportion to the two cause-specific interval risks. One draw from each\n/// stream supplies both an exchange-reversing orientation and a commutative XOR base. Exchanging\n/// cause labels/streams therefore complements the bounded tie draw and exchanges attribution\n/// exactly without consuming extra ordinary-case RNG words.\n",
)
replace_once(
    "crates/anthrosim-core/src/mortality.rs",
    "fn draw_symmetric_bounded(\n    left: &mut ChaCha8Rng,\n    right: &mut ChaCha8Rng,\n    upper_exclusive: u64,\n) -> u64 {\n    debug_assert!(upper_exclusive > 0);\n\n    // A commutative XOR alone gives the same bounded draw after exchanging the two streams.\n    // Weighted attribution needs the stronger property d(right,left) = upper - 1 - d(left,right)\n    // so that swapping cause weights swaps the selected cause exactly. Use one independent draw\n    // pair only to choose orientation; equality consumes another pair symmetrically.\n    let left_is_lower = loop {\n        let left_order = left.next_u64();\n        let right_order = right.next_u64();\n        if left_order != right_order {\n            break left_order < right_order;\n        }\n    };\n\n    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);\n    loop {\n        // These later draws are independent of the orientation pair. XOR is commutative, so the\n        // base value is identical after exchanging streams; the orientation then complements it.\n        let draw = left.next_u64() ^ right.next_u64();\n        if draw < acceptance_limit {\n            let base = draw % upper_exclusive;\n            return if left_is_lower {\n                base\n            } else {\n                upper_exclusive - 1 - base\n            };\n        }\n    }\n}\n",
    "fn draw_symmetric_bounded(\n    left: &mut ChaCha8Rng,\n    right: &mut ChaCha8Rng,\n    upper_exclusive: u64,\n) -> u64 {\n    debug_assert!(upper_exclusive > 0);\n    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);\n\n    loop {\n        let left_draw = left.next_u64();\n        let right_draw = right.next_u64();\n        if left_draw == right_draw {\n            // Equality has no exchange-reversing orientation. Resample both streams together; this\n            // path has probability 2^-64 for independent 64-bit words and remains exactly symmetric.\n            continue;\n        }\n\n        // Conditional on unequal independent draws, XOR is uniform over 1..=u64::MAX and the\n        // comparison orientation is balanced for every non-zero XOR value. Map XOR-1 to a uniform\n        // zero-based rank, reject the incomplete modulo tail, then complement under reversed order.\n        let rank = (left_draw ^ right_draw) - 1;\n        if rank < acceptance_limit {\n            let base = rank % upper_exclusive;\n            return if left_draw < right_draw {\n                base\n            } else {\n                upper_exclusive - 1 - base\n            };\n        }\n    }\n}\n",
)
print("issue 208 exchange symmetry updated without extra ordinary-case RNG consumption")
