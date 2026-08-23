# Run-directory transaction recovery

AnthroSim writes a complete run bundle into a sibling staging directory and only promotes it after the bundle has been written and validated. Fresh runs use one final sibling rename. Verified in-place resumes replace an existing canonical run directory and therefore require a recoverable multi-step transaction.

## Verified replacement protocol

Before moving the existing run, AnthroSim creates a versioned sibling recovery marker that binds exactly one canonical target, one staging directory, and one backup directory. The replacement then proceeds as:

1. write and validate the complete staged resumed bundle;
2. write the recovery marker;
3. rename the canonical target to the bound verified backup;
4. rename the bound staging directory to the canonical target;
5. remove the verified backup;
6. remove the recovery marker.

The stage and backup are siblings of the canonical run directory so the rename operations remain on the same filesystem.

A staged bundle is **never** treated as canonical merely because it exists. Recovery does not promote an abandoned stage.

## Recovery command

If a run/resume command reports interrupted transaction state, run:

```text
cargo run --release -p anthrosim-cli --bin anthrosim-recover -- --run-dir PATH_TO_RUN
```

For a built release binary, the equivalent is:

```text
anthrosim-recover --run-dir PATH_TO_RUN
```

The command reconciles only transaction artifacts bound to that canonical run path. Ordinary run/resume commands fail before starting new work when unresolved transaction state is present. Running the recovery command when no transaction state exists is a safe no-op and reports that no recovery was needed.

## Deterministic recovery rules

For a marked transaction, the canonical target, bound stage, and bound backup determine the safe action:

| Canonical target | Stage | Backup | Recovery |
| --- | --- | --- | --- |
| present | present | absent | Keep canonical target; remove abandoned stage. |
| absent | present | present | Restore verified backup; remove abandoned stage. |
| absent | absent | present | Restore verified backup. |
| present | absent | present | Promotion already happened; keep canonical target and remove stale backup. |
| present | absent | absent | Replacement already completed; remove stale marker. |

Any other marked state is ambiguous and fails closed without choosing between competing bundles.

AnthroSim also recognises a narrow set of remnants created by the pre-marker transactional implementation. If the canonical target is absent and there is exactly one legacy backup, that backup is restored. If exactly one legacy stage also exists it is discarded. Multiple or otherwise ambiguous unmarked remnants fail closed.

## Cleanup failures after promotion

If promotion succeeds but the previous backup cannot be deleted, the command reports that the **new canonical bundle has already been committed** and that cleanup is pending. It does not describe the run as uncommitted. The marker remains so `anthrosim-recover` can remove the stale backup deterministically later.

Likewise, if the backup has been removed but recovery-marker cleanup fails, the canonical run remains committed and the recovery command can remove the stale marker.

## Guarantees and filesystem assumptions

The recovery protocol is intended to make abrupt interruption during the final replacement window recoverable without manual filesystem archaeology. It assumes:

- the staging, canonical, and backup directories remain on the same filesystem;
- filesystem rename provides the platform's normal atomic rename semantics for one directory entry;
- the filesystem and storage device provide their normal durability guarantees after process or machine failure;
- users do not manually rename, copy, or edit hidden `.anthrosim-*` transaction remnants while a transaction is unresolved.

AnthroSim does not claim that the sequence of multiple filesystem operations is itself atomic. The marker and deterministic recovery rules are the mechanism that makes those intermediate states recoverable. Ambiguity is never resolved by guessing or by promoting an unverified stage.
