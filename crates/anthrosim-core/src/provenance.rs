/// Compatibility identity for the authoritative scientific/model semantics used by this build.
///
/// This is intentionally independent of the package version and exact Git commit. Increment the
/// identifier whenever authoritative simulation meaning changes in a way that makes checkpoint
/// continuation scientifically incompatible. Documentation, tooling, or other source-neutral
/// changes do not require a new identity.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v1";
