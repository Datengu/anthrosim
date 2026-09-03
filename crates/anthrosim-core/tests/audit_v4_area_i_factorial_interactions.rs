use std::collections::BTreeSet;

use anthrosim_core::{
    ExperimentConfig, ResearchDimension, ResearchDimensionKind, ResearchExperimentDefinition,
    ResearchRunConfig,
};

fn definition(dimensions: Vec<ResearchDimension>) -> ResearchExperimentDefinition {
    ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![941_001],
        base: ResearchRunConfig {
            experiment: ExperimentConfig::new(941_001, 1),
            spatial: None,
        },
        dimensions,
    }
}

fn resource_dimensions() -> (ResearchDimension, ResearchDimension) {
    (
        ResearchDimension {
            id: "annual-need".to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: "/experiment/resources/annualNeedUnitsPerPerson".to_owned(),
            values: vec![serde_json::json!(100), serde_json::json!(200), serde_json::json!(400)],
        },
        ResearchDimension {
            id: "regeneration".to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: "/experiment/resources/annualRegenerationUnitsPerProductivity".to_owned(),
            values: vec![serde_json::json!(1), serde_json::json!(3), serde_json::json!(9)],
        },
    )
}

fn executable_pairs(definition: &ResearchExperimentDefinition) -> BTreeSet<(u32, u32)> {
    definition
        .expand()
        .unwrap()
        .into_iter()
        .map(|point| {
            let resources = point.run_config.experiment.resources;
            (
                resources.annual_need_units_per_person,
                resources.annual_regeneration_units_per_productivity,
            )
        })
        .collect()
}

#[test]
fn numeric_factorial_preserves_all_interaction_combinations_independent_of_dimension_order() {
    let (need, regeneration) = resource_dimensions();
    let forward = definition(vec![need.clone(), regeneration.clone()]);
    let reversed = definition(vec![regeneration, need]);

    let forward_points = forward.expand().unwrap();
    let reversed_points = reversed.expand().unwrap();

    assert_eq!(forward_points.len(), 9);
    assert_eq!(reversed_points.len(), 9);

    let expected: BTreeSet<(u32, u32)> = [
        (100, 1), (100, 3), (100, 9),
        (200, 1), (200, 3), (200, 9),
        (400, 1), (400, 3), (400, 9),
    ]
    .into_iter()
    .collect();

    assert_eq!(executable_pairs(&forward), expected);
    assert_eq!(executable_pairs(&reversed), expected);

    for point in forward_points {
        assert_eq!(point.coordinates.len(), 2);
        assert_eq!(point.coordinates[0].id, "annual-need");
        assert_eq!(point.coordinates[1].id, "regeneration");
    }
    for point in reversed_points {
        assert_eq!(point.coordinates.len(), 2);
        assert_eq!(point.coordinates[0].id, "regeneration");
        assert_eq!(point.coordinates[1].id, "annual-need");
    }
}
