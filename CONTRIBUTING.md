# Contributing to AnthroSim

AnthroSim is being developed as research-oriented simulation software. Changes should preserve reproducibility, inspectability, and measurable performance rather than optimise for feature count.

## Development rules

1. **Model causes, not outcomes.** Do not add a high-level historical outcome merely to make runs look interesting.
2. **Keep the simulation core headless.** Rendering, notebooks, and interactive analysis consume outputs; they do not own authoritative state.
3. **Determinism is an API contract.** Random draws must come from explicit deterministic streams. Do not use ambient/thread randomness in simulation code.
4. **Assumptions must be visible.** New behavioural parameters require documentation, units, defaults, and a statement of whether they are placeholders or evidence-grounded.
5. **Performance is tested.** Avoid allocation-heavy hot loops and global all-to-all scans. If complexity is added for speed, attach benchmark evidence.
6. **Interpretation must be traceable.** A derived explanation must be traceable back to metrics/events/state.
7. **No silent schema drift.** Persisted experiment, manifest, event, and checkpoint formats are versioned.
8. **No silent dependency drift.** The Rust toolchain and `Cargo.lock` are part of build/experiment provenance.

## Pull requests

A PR should normally include:

- the problem being addressed;
- the modelling assumption introduced or changed;
- tests for invariants and deterministic behaviour;
- benchmark impact when a hot path changes;
- documentation updates when configuration or scientific meaning changes.

Run before requesting review:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo bench --locked -p anthrosim-core --bench empty_run -- --noplot
cargo build --locked --workspace --release
```

## Dependency updates

`Cargo.lock` is committed intentionally. Normal builds and CI use `--locked`; they must not resolve a new dependency graph implicitly.

When dependencies are deliberately changed:

1. edit the relevant `Cargo.toml` manifest(s);
2. regenerate/update `Cargo.lock` explicitly with the pinned toolchain;
3. review the direct and transitive dependency changes;
4. run the full locked validation suite;
5. commit manifest and lockfile changes together in a reviewable PR.

A lockfile change is part of software provenance and should not be treated as generated noise.

## Scientific changes

A change is *scientific* when it changes simulated meaning rather than implementation only. Scientific changes should state:

- hypothesis or modelling purpose;
- source/evidence, if any;
- parameter units and plausible range;
- expected directional behaviour;
- validation or sensitivity test required.

Place durable model documentation in `docs/scientific-model.md`, evidence/provenance notes in `docs/research/`, and architecture decisions in `docs/adr/`.
