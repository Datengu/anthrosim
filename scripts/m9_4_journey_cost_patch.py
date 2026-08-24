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
    "const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 1;",
    "const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;",
)

replace_once(
    path,
    '''    pub residence: CellId,\n    pub destination: CellId,\n    pub departure_day: u64,''',
    '''    pub residence: CellId,\n    pub destination: CellId,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub travel_model_identity: Option<String>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub accumulated_travel_cost_units: Option<u64>,\n    pub departure_day: u64,''',
)

replace_once(
    path,
    '''        if self.region_id.trim().is_empty() || self.region_identity.trim().is_empty() {\n            return Err(TemporaryMobilityError::InvalidRegionIdentity {\n                household: self.household,\n                journey: self.journey,\n            });\n        }\n        Ok(())''',
    '''        if self.region_id.trim().is_empty() || self.region_identity.trim().is_empty() {\n            return Err(TemporaryMobilityError::InvalidRegionIdentity {\n                household: self.household,\n                journey: self.journey,\n            });\n        }\n        match (\n            self.travel_model_identity.as_deref(),\n            self.accumulated_travel_cost_units,\n        ) {\n            (None, None) => {}\n            (Some(identity), Some(_)) if !identity.trim().is_empty() => {}\n            _ => {\n                return Err(TemporaryMobilityError::InvalidTravelMetadata {\n                    household: self.household,\n                    journey: self.journey,\n                });\n            }\n        }\n        Ok(())''',
)

replace_once(
    path,
    '''        digest_u64(hash, self.residence.0);\n        digest_u64(hash, self.destination.0);\n        digest_u64(hash, self.departure_day);''',
    '''        digest_u64(hash, self.residence.0);\n        digest_u64(hash, self.destination.0);\n        match &self.travel_model_identity {\n            None => digest_u64(hash, 0),\n            Some(identity) => {\n                digest_u64(hash, 1);\n                digest_str(hash, identity);\n            }\n        }\n        match self.accumulated_travel_cost_units {\n            None => digest_u64(hash, 0),\n            Some(cost) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, cost);\n            }\n        }\n        digest_u64(hash, self.departure_day);''',
)

replace_once(
    path,
    '''                    if let Some(program) = &self.program\n                        && (active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !program.region.contains(active.destination))\n                    {\n                        return Err(\n                            TemporaryMobilityValidationError::ActiveJourneyProgramMismatch {\n                                household,\n                            },\n                        );\n                    }''',
    '''                    if let Some(program) = &self.program {\n                        let expected_model_identity =\n                            program.travel.travel_model().map(|model| model.identity());\n                        let expected_cost =\n                            program.travel.accumulated_cost_units(active.residence);\n                        if active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !program.region.contains(active.destination)\n                            || active.travel_model_identity != expected_model_identity\n                            || active.accumulated_travel_cost_units != expected_cost\n                        {\n                            return Err(\n                                TemporaryMobilityValidationError::ActiveJourneyProgramMismatch {\n                                    household,\n                                },\n                            );\n                        }\n                    }''',
)

replace_once(
    path,
    '''        let TemporaryTravelResolution::Reachable {\n            destination,\n            outbound_travel_days,\n            return_travel_days,\n        } = resolution\n        else {''',
    '''        let TemporaryTravelResolution::Reachable {\n            destination,\n            outbound_travel_days,\n            return_travel_days,\n        } = resolution\n        else {''',
)

# Add cost/model metadata after reachability has been established.
replace_once(
    path,
    '''            return Ok(TriggerEvaluation::Skipped(reason));\n        };\n\n        let departure_day = match program.schedule.trigger_timing {''',
    '''            return Ok(TriggerEvaluation::Skipped(reason));\n        };\n\n        let travel_model_identity = program.travel.travel_model().map(|model| model.identity());\n        let accumulated_travel_cost_units = if travel_model_identity.is_some() {\n            Some(\n                program\n                    .travel\n                    .accumulated_cost_units(residence)\n                    .ok_or(TemporaryMobilityExecutionError::MissingTravelCost { residence })?,\n            )\n        } else {\n            None\n        };\n\n        let departure_day = match program.schedule.trigger_timing {''',
)

replace_once(
    path,
    '''            residence,\n            destination,\n            departure_day,''',
    '''            residence,\n            destination,\n            travel_model_identity: travel_model_identity.clone(),\n            accumulated_travel_cost_units,\n            departure_day,''',
)

replace_once(
    path,
    '''                residence,\n                destination,\n                people_affected,''',
    '''                residence,\n                destination,\n                travel_model_identity,\n                accumulated_travel_cost_units,\n                people_affected,''',
)

replace_once(
    path,
    '''        destination,\n        departure_day,\n        arrival_day,''',
    '''        destination,\n        travel_model_identity: None,\n        accumulated_travel_cost_units: None,\n        departure_day,\n        arrival_day,''',
)

replace_once(
    path,
    '''    #[error("household {household:?} journey {journey:?} has invalid region identity")]\n    InvalidRegionIdentity {\n        household: HouseholdId,\n        journey: TemporaryJourneyId,\n    },''',
    '''    #[error("household {household:?} journey {journey:?} has invalid region identity")]\n    InvalidRegionIdentity {\n        household: HouseholdId,\n        journey: TemporaryJourneyId,\n    },\n    #[error("household {household:?} journey {journey:?} has invalid travel metadata")]\n    InvalidTravelMetadata {\n        household: HouseholdId,\n        journey: TemporaryJourneyId,\n    },''',
)

replace_once(
    path,
    '''    #[error("temporary mobility travel table has no entry for residence {residence:?}")]\n    MissingTravelResolution { residence: CellId },''',
    '''    #[error("temporary mobility travel table has no entry for residence {residence:?}")]\n    MissingTravelResolution { residence: CellId },\n    #[error("temporary mobility M9.4 table has no travel cost for reachable residence {residence:?}")]\n    MissingTravelCost { residence: CellId },''',
)

# Departure events carry the same causal travel metadata as active journey state.
path = "crates/anthrosim-core/src/events.rs"
replace_once(
    path,
    '''        residence: CellId,\n        destination: CellId,\n        people_affected: u32,''',
    '''        residence: CellId,\n        destination: CellId,\n        #[serde(default, skip_serializing_if = "Option::is_none")]\n        travel_model_identity: Option<String>,\n        #[serde(default, skip_serializing_if = "Option::is_none")]\n        accumulated_travel_cost_units: Option<u64>,\n        people_affected: u32,''',
)

# Invariant validation understands event schema v2 and requires travel metadata to be paired.
path = "crates/anthrosim-core/src/invariants.rs"
text = Path(path).read_text()
text = text.replace("*event_schema_version != 1", "*event_schema_version != 2")
Path(path).write_text(text)
replace_once(
    path,
    '''                return_travel_days,\n                ..\n            } => {\n                if *event_schema_version != 2''',
    '''                return_travel_days,\n                travel_model_identity,\n                accumulated_travel_cost_units,\n                ..\n            } => {\n                let travel_metadata_valid = match (\n                    travel_model_identity.as_deref(),\n                    accumulated_travel_cost_units,\n                ) {\n                    (None, None) => true,\n                    (Some(identity), Some(_)) => !identity.trim().is_empty(),\n                    _ => false,\n                };\n                if *event_schema_version != 2''',
)
replace_once(
    path,
    '''                    || *completion_day\n                        != return_departure_day.saturating_add(u64::from(*return_travel_days))\n                {''',
    '''                    || *completion_day\n                        != return_departure_day.saturating_add(u64::from(*return_travel_days))\n                    || !travel_metadata_valid\n                {''',
)

print("M9.4 journey cost patch applied")
