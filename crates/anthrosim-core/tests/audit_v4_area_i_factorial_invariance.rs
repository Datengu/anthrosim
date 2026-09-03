use std::collections::BTreeSet;

use anthrosim_core::{
    ExperimentConfig, ResearchDimension, ResearchDimensionKind, ResearchExperimentDefinition,
    ResearchExperimentError, ResearchRunConfig,
};
use serde_json::{Value, json};

fn numeric(id: &str, path: &str, values: Vec<Value>) -> ResearchDimension {
    ResearchDimension {
        id: id.to_owned(),
        kind: ResearchDimensionKind::Numeric,
        path: path.to_owned(),
        values,
    }
}

fn base_definition() -> ResearchExperimentDefinition {
    ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![88_001, 88_002, 88_003],
        base: ResearchRunConfig {
            experiment: ExperimentConfig::new(88_001, 3),
            spatial: None,
        },
        dimensions: Vec::new(),
    }
}

fn base_dimensions() -> Vec<ResearchDimension> {
    vec![
        numeric(
            "duration-years",
            "/experiment/durationYears",
            vec![json!(1), json!(3)],
        ),
        numeric(
            "resource-periods",
            "/experiment/resources/periodsPerYear",
            vec![json!(1), json!(12), json!(365)],
        ),
        numeric(
            "initial-stock",
            "/experiment/resources/initialStockUnitsPerProductivity",
            vec![json!(0), json!(10)],
        ),
    ]
}

fn executable_treatments(definition: &ResearchExperimentDefinition) -> BTreeSet<Vec<u8>> {
    let points = definition.expand().unwrap();
    assert_eq!(points.len(), 12);

    let mut treatments = BTreeSet::new();
    for point in points {
        let encoded = serde_json::to_value(&point.run_config).unwrap();
        assert_eq!(point.coordinates.len(), 3);
        for coordinate in &point.coordinates {
            assert_eq!(
                encoded.pointer(&coordinate.path),
                Some(&coordinate.value),
                "recorded coordinate must remain exactly represented by final executable config: {}",
                coordinate.id
            );
        }
        assert!(
            treatments.insert(serde_json::to_vec(&point.run_config).unwrap()),
            "the declared 2×3×2 design produced a duplicate executable treatment"
        );
    }
    assert_eq!(treatments.len(), 12);
    treatments
}

#[test]
fn factorial_treatment_set_is_invariant_to_dimension_and_value_declaration_order() {
    let permutations = [
        [0_usize, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let base_dimensions = base_dimensions();
    let mut canonical_treatments: Option<BTreeSet<Vec<u8>>> = None;
    let mut expansions = 0_u64;
    let mut point_coordinate_checks = 0_u64;

    for permutation in permutations {
        for reverse_mask in 0_u8..8 {
            let mut dimensions = Vec::with_capacity(3);
            for index in permutation {
                let mut dimension = base_dimensions[index].clone();
                if reverse_mask & (1 << index) != 0 {
                    dimension.values.reverse();
                }
                dimensions.push(dimension);
            }

            let mut definition = base_definition();
            definition.dimensions = dimensions;
            let treatments = executable_treatments(&definition);
            if let Some(canonical) = &canonical_treatments {
                assert_eq!(
                    &treatments, canonical,
                    "scientifically identical factorial design changed its executable treatment set under declaration-order transformation: permutation={permutation:?} reverse_mask={reverse_mask}"
                );
            } else {
                canonical_treatments = Some(treatments);
            }
            expansions += 1;
            point_coordinate_checks += 12;
        }
    }

    println!("audit_v4_area_i_factorial_expansions={expansions}");
    println!("audit_v4_area_i_factorial_point_coordinate_checks={point_coordinate_checks}");
    println!(
        "audit_v4_area_i_distinct_executable_treatments={}",
        canonical_treatments.unwrap().len()
    );
    assert_eq!(expansions, 48);
    assert_eq!(point_coordinate_checks, 576);
}

#[test]
fn ancestor_descendant_dimensions_fail_closed_in_both_declaration_orders() {
    let child = numeric(
        "annual-need",
        "/experiment/resources/annualNeedUnitsPerPerson",
        vec![json!(100), json!(200)],
    );

    let base = base_definition();
    let mut low = base.base.experiment.resources.clone();
    low.annual_need_units_per_person = 300;
    let mut high = base.base.experiment.resources.clone();
    high.annual_need_units_per_person = 400;
    let parent = ResearchDimension {
        id: "whole-resource-model".to_owned(),
        kind: ResearchDimensionKind::Structural,
        path: "/experiment/resources".to_owned(),
        values: vec![
            serde_json::to_value(low).unwrap(),
            serde_json::to_value(high).unwrap(),
        ],
    };

    for dimensions in [vec![child.clone(), parent.clone()], vec![parent.clone(), child.clone()]] {
        let mut definition = base_definition();
        definition.dimensions = dimensions;
        assert!(matches!(
            definition.validate(),
            Err(ResearchExperimentError::OverlappingDimensionPaths { .. })
        ));
    }
}
