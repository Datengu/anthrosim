from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace(path: str, old: str, new: str, count: int = 1) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"anchor not found in {path}: {old[:120]!r}")
    target.write_text(text.replace(old, new, count), encoding="utf-8")


replace(
    "crates/anthrosim-core/src/invariants.rs",
    """            EventKind::TemporaryJourneyNotStarted {\n                event_schema_version,""",
    """            EventKind::HouseholdFission {\n                event_schema_version,\n                source_household,\n                new_household,\n                residence,\n                people_reassigned,\n            } => {\n                if *event_schema_version != 1\n                    || source_household.0 == 0\n                    || source_household.0 > population.household_count\n                    || new_household.0 == 0\n                    || new_household.0 > population.household_count\n                    || source_household == new_household\n                    || world.cell(*residence).is_none()\n                    || people_reassigned.is_empty()\n                {\n                    return violation(\"household fission event is invalid\");\n                }\n                let mut unique_people = BTreeSet::new();\n                for person in people_reassigned {\n                    if !unique_people.insert(*person) || population_state.person(*person).is_none() {\n                        return violation(\n                            \"household fission event has duplicate or invalid reassigned people\",\n                        );\n                    }\n                }\n            }\n            EventKind::TemporaryJourneyNotStarted {\n                event_schema_version,""",
)
replace(
    "crates/anthrosim-core/src/invariants.rs",
    """                if snapshot.birth_day != birth_day\n                    || snapshot.female_parent != *female_parent\n                    || snapshot.male_parent != *male_parent\n                    || snapshot.household != *household\n                    || snapshot.reproductive_sex != *reproductive_sex\n                    || world.cell(*cell).is_none()""",
    """                if snapshot.birth_day != birth_day\n                    || snapshot.female_parent != *female_parent\n                    || snapshot.male_parent != *male_parent\n                    || household.0 == 0\n                    || household.0 > population.household_count\n                    || snapshot.reproductive_sex != *reproductive_sex\n                    || world.cell(*cell).is_none()""",
)
replace(
    "crates/anthrosim-core/src/invariants.rs",
    """                EventKind::TemporaryJourneyNotStarted { .. }\n                | EventKind::TemporaryJourneyDeparted { .. }""",
    """                EventKind::HouseholdFission { .. }\n                | EventKind::TemporaryJourneyNotStarted { .. }\n                | EventKind::TemporaryJourneyDeparted { .. }""",
)

replace(
    "crates/anthrosim-core/src/spatial_observability.rs",
    """            EventKind::TemporaryJourneyNotStarted { .. }\n            | EventKind::TemporaryJourneyDeparted { .. }""",
    """            EventKind::HouseholdFission { .. }\n            | EventKind::TemporaryJourneyNotStarted { .. }\n            | EventKind::TemporaryJourneyDeparted { .. }""",
)

replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """            EventKind::TemporaryJourneyCompleted {\n                event_schema_version,\n                journey,\n                residence,\n                people_affected,\n                ..\n            } => {\n                validate_temp_event_schema(*event_schema_version)?;\n                require_active(replay, index, *journey, HouseholdPresenceKind::Return)?;\n                if replay.households[index].residence != *residence\n                    || u64::from(*people_affected) != replay.households[index].living\n                {\n                    return Err(invalid(\n                        \"temporary completion does not reconcile with replay household\",\n                    ));\n                }\n                replay.households[index].presence = HouseholdPresence::AtResidence;\n                replay.households[index].active_journey = None;\n                let row_index = journey_row_index(replay, *journey)?;\n                if replay.journeys[row_index].realized_route_legs != 1 {\n                    return Err(invalid(\n                        \"temporary completion occurred without exactly one realized outbound route leg\",\n                    ));\n                }\n                replay.journeys[row_index].realized_route_legs = 2;\n                replay.journeys[row_index].status = TemporaryJourneyObservedStatus::Completed;\n                replay.summary.journeys_completed = add(replay.summary.journeys_completed, 1)?;\n                let origin = replay.origins.entry(residence.0).or_default();\n                origin.journeys_completed = add(origin.journeys_completed, 1)?;\n            }\n        }""",
    """            EventKind::TemporaryJourneyCompleted {\n                event_schema_version,\n                journey,\n                residence,\n                people_affected,\n                ..\n            } => {\n                validate_temp_event_schema(*event_schema_version)?;\n                require_active(replay, index, *journey, HouseholdPresenceKind::Return)?;\n                if replay.households[index].residence != *residence\n                    || u64::from(*people_affected) != replay.households[index].living\n                {\n                    return Err(invalid(\n                        \"temporary completion does not reconcile with replay household\",\n                    ));\n                }\n                replay.households[index].presence = HouseholdPresence::AtResidence;\n                replay.households[index].active_journey = None;\n                let row_index = journey_row_index(replay, *journey)?;\n                if replay.journeys[row_index].realized_route_legs != 1 {\n                    return Err(invalid(\n                        \"temporary completion occurred without exactly one realized outbound route leg\",\n                    ));\n                }\n                replay.journeys[row_index].realized_route_legs = 2;\n                replay.journeys[row_index].status = TemporaryJourneyObservedStatus::Completed;\n                replay.summary.journeys_completed = add(replay.summary.journeys_completed, 1)?;\n                let origin = replay.origins.entry(residence.0).or_default();\n                origin.journeys_completed = add(origin.journeys_completed, 1)?;\n            }\n            EventKind::HouseholdFission { .. } => {\n                unreachable!(\"household fission is handled before ordinary event replay\")\n            }\n        }""",
)

replace(
    "crates/anthrosim-core/src/temporary_history.rs",
    """        if let EventKind::HouseholdFission { new_household, .. } = record.event {""",
    """        if let EventKind::HouseholdFission { new_household, .. } = &record.event {""",
)

print("issue 207 exhaustive event consumer coverage applied")
