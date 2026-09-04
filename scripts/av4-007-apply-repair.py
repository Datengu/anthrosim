from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one replacement target, found {count}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "crates/anthrosim-core/src/events.rs",
    '''        destination: CellId,\n        #[serde(default, skip_serializing_if = "Option::is_none")]\n        travel_model_identity: Option<String>,''',
    '''        destination: CellId,\n        #[serde(default, skip_serializing_if = "Option::is_none")]\n        destination_tie_coupling_key: Option<u64>,\n        #[serde(default, skip_serializing_if = "Option::is_none")]\n        travel_model_identity: Option<String>,''',
)
replace_once(
    "crates/anthrosim-core/src/events.rs",
    '''    /// v3 makes M4's nominal per-person travel decrement explicit and records the exact realized\n    /// bounded condition loss for every authoritative household-migration event.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
    '''    /// v4 records the scientific household coupling key used when M9 resolves an exact-cost\n    /// destination tie, so authoritative temporary-mobility choices remain auditable without\n    /// treating canonical HouseholdId as a causal input.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 4;''',
)

replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;\nconst M9_DESTINATION_TIE_POLICY_ID: &str = "m9/equal-cost-destination-keyed-v1";''',
    '''const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 3;\nconst M9_DESTINATION_TIE_POLICY_ID: &str =\n    "m9/equal-cost-destination-scientific-coupling-v2";''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
    '''    pub const CURRENT_SCHEMA_VERSION: u32 = 4;''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''    /// Resolve one household/trigger destination without consuming a mutable RNG stream.\n    #[must_use]\n    pub fn resolution_for(\n        &self,\n        origin: CellId,\n        household: HouseholdId,\n        trigger_index: u32,\n    ) -> Option<TemporaryTravelResolution> {\n        let base = self.resolution(origin)?;\n        let TemporaryTravelResolution::Reachable {\n            outbound_travel_days,\n            return_travel_days,\n            ..\n        } = base\n        else {\n            return Some(base);\n        };\n        let Some(candidates) = self.equal_cost_destinations(origin) else {\n            return Some(base);\n        };\n        if candidates.len() <= 1 {\n            return Some(base);\n        }\n        let mut hash = FNV_OFFSET_BASIS;\n        digest_str(&mut hash, M9_DESTINATION_TIE_POLICY_ID);\n        digest_u64(&mut hash, self.destination_tie_seed.unwrap_or(0));\n        digest_u64(&mut hash, origin.0);\n        digest_u64(&mut hash, household.0);\n        digest_u64(&mut hash, u64::from(trigger_index));\n        hash = avalanche64(hash);\n        let index = usize::try_from(hash % candidates.len() as u64).ok()?;\n        Some(TemporaryTravelResolution::Reachable {\n            destination: candidates[index].destination,\n            outbound_travel_days,\n            return_travel_days,\n        })\n    }\n''',
    '''    /// Compatibility resolver for callers that only have a canonical HouseholdId.\n    ///\n    /// Canonical household numbering is deliberately non-causal under the v2 tie policy, so this\n    /// surface uses a neutral scientific coupling key. Authoritative simulation execution calls\n    /// `resolution_for_coupling_key` with the persistent household coupling key derived from living\n    /// person stochastic-coupling ranks.\n    #[must_use]\n    pub fn resolution_for(\n        &self,\n        origin: CellId,\n        _household: HouseholdId,\n        trigger_index: u32,\n    ) -> Option<TemporaryTravelResolution> {\n        self.resolution_for_coupling_key(origin, 0, trigger_index)\n    }\n\n    /// Resolve one scientific household/trigger destination without consuming a mutable RNG stream.\n    #[must_use]\n    pub fn resolution_for_coupling_key(\n        &self,\n        origin: CellId,\n        household_coupling_key: u64,\n        trigger_index: u32,\n    ) -> Option<TemporaryTravelResolution> {\n        let base = self.resolution(origin)?;\n        let TemporaryTravelResolution::Reachable {\n            outbound_travel_days,\n            return_travel_days,\n            ..\n        } = base\n        else {\n            return Some(base);\n        };\n        let Some(candidates) = self.equal_cost_destinations(origin) else {\n            return Some(base);\n        };\n        if candidates.len() <= 1 {\n            return Some(base);\n        }\n        let mut hash = FNV_OFFSET_BASIS;\n        digest_str(&mut hash, M9_DESTINATION_TIE_POLICY_ID);\n        digest_u64(&mut hash, self.destination_tie_seed.unwrap_or(0));\n        digest_u64(&mut hash, origin.0);\n        digest_u64(&mut hash, household_coupling_key);\n        digest_u64(&mut hash, u64::from(trigger_index));\n        hash = avalanche64(hash);\n        let index = usize::try_from(hash % candidates.len() as u64).ok()?;\n        Some(TemporaryTravelResolution::Reachable {\n            destination: candidates[index].destination,\n            outbound_travel_days,\n            return_travel_days,\n        })\n    }\n''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''    fn digest_into(&self, hash: &mut u64) {\n        digest_u64(hash, u64::from(self.schema_version));\n        digest_u64(hash, self.resolutions.len() as u64);''',
    '''    fn digest_into(&self, hash: &mut u64) {\n        digest_u64(hash, u64::from(self.schema_version));\n        digest_str(hash, M9_DESTINATION_TIE_POLICY_ID);\n        digest_u64(hash, self.resolutions.len() as u64);''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''        let residence = population\n            .household_location(household)\n            .ok_or(TemporaryMobilityExecutionError::InvalidHousehold { household })?;\n        let resolution = program\n            .travel\n            .resolution_for(residence, household, trigger_index)\n            .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;''',
    '''        let residence = population\n            .household_location(household)\n            .ok_or(TemporaryMobilityExecutionError::InvalidHousehold { household })?;\n        let destination_tie_coupling_key =\n            household_stochastic_coupling_key(population, household)?;\n        let resolution = program\n            .travel\n            .resolution_for_coupling_key(\n                residence,\n                destination_tie_coupling_key,\n                trigger_index,\n            )\n            .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''                residence,\n                destination,\n                travel_model_identity,''',
    '''                residence,\n                destination,\n                destination_tie_coupling_key: program\n                    .travel\n                    .equal_cost_destination_count(residence)\n                    .is_some_and(|count| count > 1)\n                    .then_some(destination_tie_coupling_key),\n                travel_model_identity,''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''fn household_living_count(population: &Population, household: HouseholdId) -> u32 {''',
    '''fn household_stochastic_coupling_key(\n    population: &Population,\n    household: HouseholdId,\n) -> Result<u64, TemporaryMobilityExecutionError> {\n    let mut key: Option<u64> = None;\n    for index in 0..population.person_count() {\n        if !population.is_alive_index(index)\n            || population.household_at_index(index) != Some(household)\n        {\n            continue;\n        }\n        let rank = population.stochastic_coupling_rank_at_index(index).ok_or(\n            TemporaryMobilityExecutionError::MissingHouseholdCouplingKey { household },\n        )?;\n        key = Some(key.map_or(rank, |prior| prior.min(rank)));\n    }\n    key.ok_or(TemporaryMobilityExecutionError::MissingHouseholdCouplingKey { household })\n}\n\nfn household_living_count(population: &Population, household: HouseholdId) -> u32 {''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    '''    #[error("temporary mobility travel table has no entry for residence {residence:?}")]\n    MissingTravelResolution { residence: CellId },''',
    '''    #[error("temporary mobility household {household:?} has no living scientific coupling key")]\n    MissingHouseholdCouplingKey { household: HouseholdId },\n    #[error("temporary mobility travel table has no entry for residence {residence:?}")]\n    MissingTravelResolution { residence: CellId },''',
)

replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;''',
    '''const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 3;''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''                residence,\n                destination,\n                travel_model_identity,''',
    '''                residence,\n                destination,\n                destination_tie_coupling_key,\n                travel_model_identity,''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''                    *residence,\n                    *destination,\n                    travel_model_identity.as_deref(),''',
    '''                    *residence,\n                    *destination,\n                    *destination_tie_coupling_key,\n                    travel_model_identity.as_deref(),''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''    residence: CellId,\n    destination: CellId,\n    travel_model_identity: Option<&str>,''',
    '''    residence: CellId,\n    destination: CellId,\n    destination_tie_coupling_key: Option<u64>,\n    travel_model_identity: Option<&str>,''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''    match replay\n        .program\n        .travel\n        .resolution_for(residence, household, trigger_index)\n    {\n        Some(TemporaryTravelResolution::Reachable {''',
    '''    let resolution = match replay\n        .program\n        .travel\n        .equal_cost_destination_count(residence)\n    {\n        Some(count) if count > 1 => {\n            let coupling_key = destination_tie_coupling_key.ok_or_else(|| {\n                invalid("tied temporary departure is missing its scientific coupling key")\n            })?;\n            replay\n                .program\n                .travel\n                .resolution_for_coupling_key(residence, coupling_key, trigger_index)\n        }\n        _ => {\n            if destination_tie_coupling_key.is_some() {\n                return Err(invalid(\n                    "non-tied temporary departure unexpectedly records a tie coupling key",\n                ));\n            }\n            replay.program.travel.resolution(residence)\n        }\n    };\n    match resolution {\n        Some(TemporaryTravelResolution::Reachable {''',
)
replace_once(
    "crates/anthrosim-core/src/temporary_observability.rs",
    '''    household: HouseholdId,\n    trigger_index: u32,\n    residence: CellId,\n    destination: CellId,\n    destination_tie_coupling_key: Option<u64>,''',
    '''    _household: HouseholdId,\n    trigger_index: u32,\n    residence: CellId,\n    destination: CellId,\n    destination_tie_coupling_key: Option<u64>,''',
)

replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    '''/// under v30 with unchanged mortality RNG positions.\npub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v30";''',
    '''/// under v30 with unchanged mortality RNG positions.\n///\n/// v31 removes canonical `HouseholdId` from the explicit M9 equal-cost destination tie key.\n/// Authoritative M9 execution now keys ties by the minimum persistent person stochastic-coupling\n/// rank among a household's living members, while a versioned label-neutral compatibility resolver\n/// remains available for callers that do not carry scientific household coupling identity. The\n/// tie-policy identifier is bound into the travel-table/program identity and each tied departure\n/// records the scientific coupling key used for observability verification. A v30 checkpoint must\n/// therefore not resume under v31 while silently changing future tied M9 destinations.\npub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v31";''',
)
replace_once(
    "crates/anthrosim-core/src/checkpoint.rs",
    '''    pub const PRE_CONDITION_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 17;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 18;''',
    '''    pub const PRE_CONDITION_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 17;\n    pub const PRE_M9_HOUSEHOLD_TIE_COUPLING_SCHEMA_VERSION: u32 = 18;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 19;''',
)

p = Path("crates/anthrosim-core/tests/m9_equal_cost_destinations.rs")
text = p.read_text()
text = text.replace(
    ".resolution_for(origin, HouseholdId::new(household), 0)",
    ".resolution_for_coupling_key(origin, household, 0)",
)
text = text.replace(
    ".resolution_for(origin, HouseholdId::new(household), 1)",
    ".resolution_for_coupling_key(origin, household, 1)",
)
text = text.replace(
    ".resolution_for(origin, HouseholdId::new(1), 0)",
    ".resolution_for_coupling_key(origin, 1, 0)",
)
p.write_text(text)

Path("crates/anthrosim-core/tests/m9_household_label_invariance.rs").write_text(r'''use anthrosim_core::{
    FocalRegion, FocalRegionSource, ParameterProvenance, TemporaryTravelModel,
    TemporaryTravelResolution, World, WorldConfig,
    ids::{CellId, HouseholdId},
    rng::RngFactory,
};

fn uniform_world() -> World {
    World::generate(WorldConfig::new(3, 3), RngFactory::new(9_401))
        .unwrap()
        .with_model_field_overlay(Some(&[1_000; 9]), None, None)
        .unwrap()
}

fn destination_for_household_label(seed: u64, household: HouseholdId) -> CellId {
    let world = uniform_world();
    let region = FocalRegion::new(
        "m9-household-label-invariance",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(8)],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "m9-household-label-invariance",
        ParameterProvenance::SyntheticValidation,
        3_000,
        u16::MAX,
    )
    .unwrap();
    let table = model
        .derive_table_with_tie_seed(&region, &world, seed)
        .unwrap();
    assert_eq!(table.equal_cost_destination_count(CellId::new(5)), Some(2));
    match table
        .resolution_for(CellId::new(5), household, 0)
        .expect("center origin must be reachable")
    {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => panic!("center origin unexpectedly unreachable"),
    }
}

#[test]
fn exact_original_style_household_relabel_sweep_is_invariant() {
    for seed in 1..=1_000 {
        assert_eq!(
            destination_for_household_label(seed, HouseholdId::new(1)),
            destination_for_household_label(seed, HouseholdId::new(2)),
            "label-neutral M9 compatibility resolution diverged at seed {seed}"
        );
    }
}

#[test]
fn scientific_coupling_keys_retain_non_degenerate_tie_diversity() {
    let world = uniform_world();
    let region = FocalRegion::new(
        "m9-scientific-key-diversity",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(8)],
    )
    .unwrap();
    let table = TemporaryTravelModel::synthetic_validation_v1()
        .derive_table_with_tie_seed(&region, &world, 91)
        .unwrap();
    let mut top = 0_u32;
    let mut bottom = 0_u32;
    for key in 0..512_u64 {
        let resolution = table
            .resolution_for_coupling_key(CellId::new(5), key, 0)
            .unwrap();
        match resolution {
            TemporaryTravelResolution::Reachable { destination, .. }
                if destination == CellId::new(2) => top += 1,
            TemporaryTravelResolution::Reachable { destination, .. }
                if destination == CellId::new(8) => bottom += 1,
            other => panic!("unexpected tied resolution {other:?}"),
        }
    }
    assert!(top > 0 && bottom > 0, "top={top}, bottom={bottom}");
    assert!(top.abs_diff(bottom) < 128, "top={top}, bottom={bottom}");
}
''')
