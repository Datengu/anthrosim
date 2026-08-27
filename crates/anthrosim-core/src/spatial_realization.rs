use serde::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig,
    spatial_mechanisms::{SpatialMechanismConfig, SpatialTargetField},
};

/// How a spatial run resolved its environment and stochastic-founder seeds relative to the dynamic
/// process seed stored in `ExperimentConfig.seed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialRealizationMode {
    /// Historical behavior: all three realization roles use the process seed.
    JointProcessSeed,
    /// Environment and stochastic-founder seeds are explicitly declared independently.
    ExplicitSplit,
}

/// Fully resolved, immutable seed roles for one spatial run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialResolvedRealization {
    pub schema_version: u32,
    pub mode: SpatialRealizationMode,
    pub environment_seed: u64,
    pub population_seed: u64,
    pub process_seed: u64,
}

impl SpatialResolvedRealization {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn resolve(experiment: &ExperimentConfig, mechanisms: &SpatialMechanismConfig) -> Self {
        match mechanisms.run_realization {
            Some(realization) => Self {
                schema_version: Self::CURRENT_SCHEMA_VERSION,
                mode: SpatialRealizationMode::ExplicitSplit,
                environment_seed: realization.environment_seed,
                population_seed: realization.population_seed,
                process_seed: experiment.seed,
            },
            None => Self {
                schema_version: Self::CURRENT_SCHEMA_VERSION,
                mode: SpatialRealizationMode::JointProcessSeed,
                environment_seed: experiment.seed,
                population_seed: experiment.seed,
                process_seed: experiment.seed,
            },
        }
    }
}

/// `World` fields still supplied by the synthetic generator after the declared M8 transform is
/// applied. These are provenance labels, not claims that every field is directly consumed by every
/// enabled mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialResidualSyntheticWorldField {
    Elevation,
    MovementCost,
    WaterAccess,
    BaseProductivity,
    InitialFoodStock,
    SeasonPhaseDays,
    SeasonAmplitude,
    EnvironmentalStress,
}

/// Resolved environment provenance exposed in every spatial manifest/checkpoint binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpatialEnvironmentProvenance {
    pub schema_version: u32,
    pub realization: SpatialResolvedRealization,
    /// Synthetic fields remaining in the final authoritative world after spatial transforms.
    pub residual_synthetic_fields: Vec<SpatialResidualSyntheticWorldField>,
    /// Subset of `residual_synthetic_fields` whose initial values vary with `environmentSeed`.
    pub seed_varying_residual_fields: Vec<SpatialResidualSyntheticWorldField>,
}

impl SpatialEnvironmentProvenance {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn resolve(experiment: &ExperimentConfig, mechanisms: &SpatialMechanismConfig) -> Self {
        let realization = SpatialResolvedRealization::resolve(experiment, mechanisms);
        let has_target = |target| {
            mechanisms
                .transforms
                .iter()
                .any(|transform| transform.target == target)
        };

        let mut residual_synthetic_fields = vec![SpatialResidualSyntheticWorldField::Elevation];
        let mut seed_varying_residual_fields = vec![SpatialResidualSyntheticWorldField::Elevation];

        if !has_target(SpatialTargetField::MovementCost) {
            residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::MovementCost);
            seed_varying_residual_fields.push(SpatialResidualSyntheticWorldField::MovementCost);
        }
        if !has_target(SpatialTargetField::WaterAccess) {
            residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::WaterAccess);
            seed_varying_residual_fields.push(SpatialResidualSyntheticWorldField::WaterAccess);
        }
        if !has_target(SpatialTargetField::BaseProductivity) {
            residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::BaseProductivity);
            residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::InitialFoodStock);
            seed_varying_residual_fields.push(SpatialResidualSyntheticWorldField::BaseProductivity);
            seed_varying_residual_fields.push(SpatialResidualSyntheticWorldField::InitialFoodStock);
        }

        // Phase is a synthetic grid/hemisphere rule and is not seed-varying. Seasonal amplitude
        // includes the synthetic climate field and therefore is seed-varying. Environmental stress
        // is a synthetic initial constant (zero) in the current World generator.
        residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::SeasonPhaseDays);
        residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::SeasonAmplitude);
        residual_synthetic_fields.push(SpatialResidualSyntheticWorldField::EnvironmentalStress);
        seed_varying_residual_fields.push(SpatialResidualSyntheticWorldField::SeasonAmplitude);

        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            realization,
            residual_synthetic_fields,
            seed_varying_residual_fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        landscape::LandscapeValueDomain,
        spatial_mechanisms::{NoDataPolicy, SpatialFieldTransform, TransformDirection},
    };

    fn full_overlay(realization: Option<crate::spatial_mechanisms::SpatialRunRealization>) -> SpatialMechanismConfig {
        let transform = |target, role: &str| {
            SpatialFieldTransform::new(
                target,
                role,
                "normalized",
                LandscapeValueDomain { min: 0, max: 1_000 },
                if target == SpatialTargetField::MovementCost { 1_000 } else { 0 },
                if target == SpatialTargetField::MovementCost { 2_000 } else { 1_000 },
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            )
        };
        let mut config = SpatialMechanismConfig::new(
            "realization-provenance-test",
            vec![
                transform(SpatialTargetField::MovementCost, "terrain"),
                transform(SpatialTargetField::WaterAccess, "water"),
                transform(SpatialTargetField::BaseProductivity, "resources"),
            ],
        );
        config.run_realization = realization;
        config
    }

    #[test]
    fn joint_mode_resolves_all_roles_to_process_seed() {
        let experiment = ExperimentConfig::new(42, 1);
        let mechanisms = full_overlay(None);
        let provenance = SpatialEnvironmentProvenance::resolve(&experiment, &mechanisms);
        assert_eq!(provenance.realization.mode, SpatialRealizationMode::JointProcessSeed);
        assert_eq!(provenance.realization.environment_seed, 42);
        assert_eq!(provenance.realization.population_seed, 42);
        assert_eq!(provenance.realization.process_seed, 42);
    }

    #[test]
    fn explicit_mode_separates_all_three_seed_roles() {
        let experiment = ExperimentConfig::new(300, 1);
        let mechanisms = full_overlay(Some(
            crate::spatial_mechanisms::SpatialRunRealization::new(100, 200),
        ));
        let provenance = SpatialEnvironmentProvenance::resolve(&experiment, &mechanisms);
        assert_eq!(provenance.realization.mode, SpatialRealizationMode::ExplicitSplit);
        assert_eq!(provenance.realization.environment_seed, 100);
        assert_eq!(provenance.realization.population_seed, 200);
        assert_eq!(provenance.realization.process_seed, 300);
        assert_eq!(
            provenance.residual_synthetic_fields,
            vec![
                SpatialResidualSyntheticWorldField::Elevation,
                SpatialResidualSyntheticWorldField::SeasonPhaseDays,
                SpatialResidualSyntheticWorldField::SeasonAmplitude,
                SpatialResidualSyntheticWorldField::EnvironmentalStress,
            ]
        );
        assert_eq!(
            provenance.seed_varying_residual_fields,
            vec![
                SpatialResidualSyntheticWorldField::Elevation,
                SpatialResidualSyntheticWorldField::SeasonAmplitude,
            ]
        );
    }
}
