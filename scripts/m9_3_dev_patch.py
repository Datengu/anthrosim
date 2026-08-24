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


path = "crates/anthrosim-core/src/spatial_observability.rs"
replace_once(
    path,
    '''                distance.people_moved = distance\n                    .people_moved\n                    .checked_add(people_moved)\n                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;\n            }\n        }''',
    '''                distance.people_moved = distance\n                    .people_moved\n                    .checked_add(people_moved)\n                    .ok_or(SpatialObservabilityError::AccountingOverflow)?;\n            }\n            EventKind::TemporaryJourneyNotStarted { .. }\n            | EventKind::TemporaryJourneyDeparted { .. }\n            | EventKind::TemporaryJourneyArrived { .. }\n            | EventKind::TemporaryReturnDeparted { .. }\n            | EventKind::TemporaryJourneyCompleted { .. } => {\n                // M8 observability remains residence/permanent-migration based. M9.6 adds\n                // temporary-presence observability rather than overloading these M8 metrics.\n            }\n        }''',
)

print("M9.3 development patches applied")
