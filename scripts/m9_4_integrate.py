from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


path = "crates/anthrosim-core/src/temporary_mobility.rs"

replace_once(
    path,
    '''    population::Population,\n    world::World,\n};''',
    '''    population::Population,\n    temporary_travel::TemporaryTravelModel,\n    world::World,\n};''',
)

replace_once(
    path,
    '''pub struct TemporaryTravelTable {\n    pub schema_version: u32,\n    resolutions: Vec<TemporaryTravelResolution>,\n}\n\nimpl TemporaryTravelTable {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;''',
    '''pub struct TemporaryTravelTable {\n    pub schema_version: u32,\n    resolutions: Vec<TemporaryTravelResolution>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    travel_model: Option<TemporaryTravelModel>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    accumulated_cost_units: Option<Vec<Option<u64>>>,\n}\n\nimpl TemporaryTravelTable {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
)

replace_once(
    path,
    '''        let table = Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            resolutions,\n        };\n        table.validate(region, world)?;\n        Ok(table)\n    }\n\n    #[must_use]\n    pub fn resolution(&self, origin: CellId) -> Option<TemporaryTravelResolution> {''',
    '''        let table = Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            resolutions,\n            travel_model: None,\n            accumulated_cost_units: None,\n        };\n        table.validate(region, world)?;\n        Ok(table)\n    }\n\n    pub(crate) fn new_m9_4(\n        resolutions: Vec<TemporaryTravelResolution>,\n        accumulated_cost_units: Vec<Option<u64>>,\n        travel_model: TemporaryTravelModel,\n        region: &FocalRegion,\n        world: &World,\n    ) -> Result<Self, TemporaryMobilityProgramError> {\n        let table = Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            resolutions,\n            travel_model: Some(travel_model),\n            accumulated_cost_units: Some(accumulated_cost_units),\n        };\n        table.validate(region, world)?;\n        Ok(table)\n    }\n\n    #[must_use]\n    pub fn resolution(&self, origin: CellId) -> Option<TemporaryTravelResolution> {''',
)

replace_once(
    path,
    '''    pub fn resolution(&self, origin: CellId) -> Option<TemporaryTravelResolution> {\n        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;\n        self.resolutions.get(index).copied()\n    }\n\n    pub fn validate(''',
    '''    pub fn resolution(&self, origin: CellId) -> Option<TemporaryTravelResolution> {\n        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;\n        self.resolutions.get(index).copied()\n    }\n\n    #[must_use]\n    pub fn travel_model(&self) -> Option<&TemporaryTravelModel> {\n        self.travel_model.as_ref()\n    }\n\n    #[must_use]\n    pub fn accumulated_cost_units(&self, origin: CellId) -> Option<u64> {\n        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;\n        self.accumulated_cost_units\n            .as_ref()?\n            .get(index)\n            .copied()\n            .flatten()\n    }\n\n    pub fn validate(''',
)

replace_once(
    path,
    '''        if self.resolutions.len() != world.cell_count() {\n            return Err(TemporaryMobilityProgramError::TravelTableShapeMismatch {\n                table: self.resolutions.len(),\n                world: world.cell_count(),\n            });\n        }\n        for (index, resolution) in self.resolutions.iter().enumerate() {''',
    '''        if self.resolutions.len() != world.cell_count() {\n            return Err(TemporaryMobilityProgramError::TravelTableShapeMismatch {\n                table: self.resolutions.len(),\n                world: world.cell_count(),\n            });\n        }\n\n        let m9_4 = match (&self.travel_model, &self.accumulated_cost_units) {\n            (None, None) => None,\n            (Some(model), Some(costs)) => {\n                model.validate().map_err(|error| {\n                    TemporaryMobilityProgramError::InvalidTravelModel {\n                        reason: error.to_string(),\n                    }\n                })?;\n                if costs.len() != world.cell_count() {\n                    return Err(TemporaryMobilityProgramError::TravelCostTableShapeMismatch {\n                        table: costs.len(),\n                        world: world.cell_count(),\n                    });\n                }\n                for &cell in region.cells() {\n                    if !model.is_traversable(world, cell) {\n                        return Err(TemporaryMobilityProgramError::TravelRegionCellImpassable {\n                            cell,\n                        });\n                    }\n                }\n                Some((model, costs))\n            }\n            _ => {\n                return Err(TemporaryMobilityProgramError::IncompleteTravelCostMetadata);\n            }\n        };\n\n        for (index, resolution) in self.resolutions.iter().enumerate() {''',
)

replace_once(
    path,
    '''                if !region.contains(origin) && *destination == origin {\n                    return Err(TemporaryMobilityProgramError::TravelDestinationIsOrigin {\n                        origin,\n                    });\n                }\n            }\n        }\n        Ok(())\n    }''',
    '''                if !region.contains(origin) && *destination == origin {\n                    return Err(TemporaryMobilityProgramError::TravelDestinationIsOrigin {\n                        origin,\n                    });\n                }\n            }\n\n            if let Some((model, costs)) = m9_4 {\n                match (*resolution, costs[index]) {\n                    (TemporaryTravelResolution::Unreachable, None) => {}\n                    (\n                        TemporaryTravelResolution::Reachable {\n                            outbound_travel_days,\n                            return_travel_days,\n                            ..\n                        },\n                        Some(cost),\n                    ) => {\n                        let expected = model.travel_days(cost).map_err(|error| {\n                            TemporaryMobilityProgramError::InvalidTravelModel {\n                                reason: error.to_string(),\n                            }\n                        })?;\n                        if outbound_travel_days != expected || return_travel_days != expected {\n                            return Err(\n                                TemporaryMobilityProgramError::TravelDurationCostMismatch {\n                                    origin,\n                                    expected,\n                                    outbound: outbound_travel_days,\n                                    returning: return_travel_days,\n                                },\n                            );\n                        }\n                    }\n                    _ => {\n                        return Err(\n                            TemporaryMobilityProgramError::TravelCostResolutionMismatch { origin },\n                        );\n                    }\n                }\n            }\n        }\n        Ok(())\n    }''',
)

replace_once(
    path,
    '''        for resolution in &self.resolutions {\n            match *resolution {\n                TemporaryTravelResolution::Unreachable => digest_u64(hash, 0),\n                TemporaryTravelResolution::Reachable {\n                    destination,\n                    outbound_travel_days,\n                    return_travel_days,\n                } => {\n                    digest_u64(hash, 1);\n                    digest_u64(hash, destination.0);\n                    digest_u64(hash, u64::from(outbound_travel_days));\n                    digest_u64(hash, u64::from(return_travel_days));\n                }\n            }\n        }\n    }\n}''',
    '''        for resolution in &self.resolutions {\n            match *resolution {\n                TemporaryTravelResolution::Unreachable => digest_u64(hash, 0),\n                TemporaryTravelResolution::Reachable {\n                    destination,\n                    outbound_travel_days,\n                    return_travel_days,\n                } => {\n                    digest_u64(hash, 1);\n                    digest_u64(hash, destination.0);\n                    digest_u64(hash, u64::from(outbound_travel_days));\n                    digest_u64(hash, u64::from(return_travel_days));\n                }\n            }\n        }\n        match &self.travel_model {\n            None => digest_u64(hash, 0),\n            Some(model) => {\n                digest_u64(hash, 1);\n                digest_str(hash, &model.identity());\n            }\n        }\n        match &self.accumulated_cost_units {\n            None => digest_u64(hash, 0),\n            Some(costs) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, costs.len() as u64);\n                for cost in costs {\n                    match cost {\n                        None => digest_u64(hash, 0),\n                        Some(cost) => {\n                            digest_u64(hash, 1);\n                            digest_u64(hash, *cost);\n                        }\n                    }\n                }\n            }\n        }\n    }\n}''',
)

replace_once(
    path,
    '''impl TemporaryMobilityProgram {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;''',
    '''impl TemporaryMobilityProgram {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
)

replace_once(
    path,
    '''impl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
    '''impl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
)

replace_once(
    path,
    '''    #[error("temporary travel table has {table} entries but world has {world} cells")]\n    TravelTableShapeMismatch { table: usize, world: usize },''',
    '''    #[error("temporary travel table has {table} entries but world has {world} cells")]\n    TravelTableShapeMismatch { table: usize, world: usize },\n    #[error("temporary travel-cost table has {table} entries but world has {world} cells")]\n    TravelCostTableShapeMismatch { table: usize, world: usize },\n    #[error("temporary travel table has incomplete M9.4 model/cost metadata")]\n    IncompleteTravelCostMetadata,\n    #[error("temporary travel model is invalid: {reason}")]\n    InvalidTravelModel { reason: String },\n    #[error("temporary travel M9.4 cost presence does not match resolution for {origin:?}")]\n    TravelCostResolutionMismatch { origin: CellId },\n    #[error(\n        "temporary travel duration for {origin:?} does not match stored cost: expected {expected}, outbound {outbound}, return {returning}"\n    )]\n    TravelDurationCostMismatch {\n        origin: CellId,\n        expected: u32,\n        outbound: u32,\n        returning: u32,\n    },\n    #[error("focal-region cell {cell:?} is impassable under the stored temporary travel model")]\n    TravelRegionCellImpassable { cell: CellId },''',
)

# Register and export M9.4 from the public core boundary.
path = "crates/anthrosim-core/src/lib.rs"
replace_once(
    path,
    '''pub mod temporary_mobility;\npub mod time;''',
    '''pub mod temporary_mobility;\npub mod temporary_travel;\npub mod time;''',
)
replace_once(
    path,
    '''pub use temporary_mobility::{\n    ActiveTemporaryJourney, HouseholdPresence, TemporaryJourneySkip, TemporaryMobilityDayOutcome,\n    TemporaryMobilityError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n    TemporaryMobilityProgramError, TemporaryMobilitySchedule, TemporaryMobilityState,\n    TemporaryMobilityValidationError, TemporaryTravelResolution, TemporaryTravelTable,\n    TemporaryTriggerTiming,\n};\npub use time::SimTime;''',
    '''pub use temporary_mobility::{\n    ActiveTemporaryJourney, HouseholdPresence, TemporaryJourneySkip, TemporaryMobilityDayOutcome,\n    TemporaryMobilityError, TemporaryMobilityExecutionError, TemporaryMobilityProgram,\n    TemporaryMobilityProgramError, TemporaryMobilitySchedule, TemporaryMobilityState,\n    TemporaryMobilityValidationError, TemporaryTravelResolution, TemporaryTravelTable,\n    TemporaryTriggerTiming,\n};\npub use temporary_travel::{\n    TemporaryTravelModel, TemporaryTravelModelError, temporary_travel_edge_cost,\n};\npub use time::SimTime;''',
)

# M9.4 changes authoritative route/duration meaning and persisted nested schemas.
replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v3";',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v4";',
)
replace_once(
    "crates/anthrosim-core/src/checkpoint.rs",
    '''    pub const PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION: u32 = 6;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 7;''',
    '''    pub const PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION: u32 = 6;\n    pub const PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION: u32 = 7;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 8;''',
)

# Reconcile roadmap status and link the frozen M9.4 semantics.
path = "docs/roadmap.md"
replace_once(
    path,
    "#### M9.1 — Residence/presence state separation",
    "#### M9.1 — Residence/presence state separation — complete",
)
replace_once(
    path,
    "#### M9.2 — Generic focal-region binding",
    "#### M9.2 — Generic focal-region binding — complete",
)
replace_once(
    path,
    "#### M9.3 — Deterministic temporary journey lifecycle",
    "#### M9.3 — Deterministic temporary journey lifecycle — complete",
)
replace_once(
    path,
    '''#### M9.4 — Travel-time and cost semantics\n\nDefine the minimum deterministic travel-duration/cost calculation required for temporary journeys, using existing model-facing movement-cost information where appropriate.''',
    '''#### M9.4 — Travel-time and cost semantics\n\n`docs/research/m9-temporary-travel-semantics-v1.md` freezes the M9.4 integer edge-cost, reachability, destination tie-break and travel-capacity semantics before authoritative implementation is merged.\n\nDefine the minimum deterministic travel-duration/cost calculation required for temporary journeys, using existing model-facing movement-cost information where appropriate.''',
)

print("M9.4 integration patch applied")
