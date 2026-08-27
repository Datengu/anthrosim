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
print("issue 208 clippy cleanup applied")
