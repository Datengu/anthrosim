//! Deterministic core primitives for AnthroSim.
//!
//! v0.1 begins with a deliberately small headless lifecycle. Domain systems
//! are added behind this boundary and remain independent of rendering/networking.

pub mod config;
pub mod ids;
pub mod manifest;
pub mod population;
pub mod rng;
pub mod simulation;
pub mod time;
pub mod world;

pub use config::{
    ExperimentConfig, PopulationConfig, PopulationInitialization, WorldConfig,
};
pub use manifest::{RunManifest, StopReason};
pub use population::{
    CellOccupancy, PersonSnapshot, Population, PopulationError, PopulationSummary,
    PopulationValidationError, ReproductiveSex,
};
pub use simulation::{Simulation, SimulationError};
pub use time::SimTime;
pub use world::{Cell, World, WorldError, WorldSummary, WorldValidationError};
