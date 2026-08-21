//! Deterministic core primitives for AnthroSim.
//!
//! v0.1 begins with a deliberately small headless lifecycle. Domain systems
//! (world, demography, food, migration) will be added behind this boundary.

pub mod config;
pub mod ids;
pub mod manifest;
pub mod rng;
pub mod simulation;
pub mod time;

pub use config::ExperimentConfig;
pub use manifest::{RunManifest, StopReason};
pub use simulation::Simulation;
pub use time::SimTime;
