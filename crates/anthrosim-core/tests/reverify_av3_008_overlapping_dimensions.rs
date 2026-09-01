use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResearchDimension,
    ResearchDimensionKind, ResearchExperimentDefinition, ResearchExperimentError,
    ResearchRunConfig, ResourceConfig, WorldConfig,
};
use serde_json::Value;

fn definition() -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(41_500, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(
            PopulationConfig::new(20)
                .with_target_household_size(5)
                .with_max_person_records(100),
        )
        .with_demography(DemographyConfig::synthetic_validation_v1())
        .with_resources(ResourceConfig::synthetic_validation_v1())
        .with_migration(MigrationConfig::synthetic_validation_v1());
    experiment
        .resources
        .max_scarcity_mortality_probability_per_million = 0;
    ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![41_500],
        base: ResearchRunConfig {
            experiment,
            spatial: None,
        },
        dimensions: Vec::new(),
    }
}

fn numeric(id: &str, path: &str, values: &[u64]) -> ResearchDimension {
    ResearchDimension {
        id: id.to_owned(),
        kind: ResearchDimensionKind::Numeric,
        path: path.to_owned(),
        values: values.iter().copied().map(Value::from).collect(),
    }
}

fn resource_parent(base: &ResearchExperimentDefinition) -> ResearchDimension {
    let mut first =
        serde_json::to_value(&base.base.experiment.resources).expect("serialize resources");
    let mut second = first.clone();
    first["annualNeedUnitsPerPerson"] = Value::from(300);
    second["annualNeedUnitsPerPerson"] = Value::from(400);
    ResearchDimension {
        id: "resource_structure".to_owned(),
        kind: ResearchDimensionKind::Structural,
        path: "/experiment/resources".to_owned(),
        values: vec![first, second],
    }
}

#[test]
fn original_two_by_two_parent_child_overwrite_is_rejected_in_both_orders() {
    let base = definition();
    let child = numeric(
        "annual_need",
        "/experiment/resources/annualNeedUnitsPerPerson",
        &[100, 200],
    );
    let parent = resource_parent(&base);

    for dimensions in [
        vec![child.clone(), parent.clone()],
        vec![parent.clone(), child.clone()],
    ] {
        let mut candidate = base.clone();
        candidate.dimensions = dimensions;
        let error = candidate
            .expand()
            .expect_err("ancestor/descendant dimensions must fail before point publication");
        assert!(
            matches!(
                error,
                ResearchExperimentError::OverlappingDimensionPaths { .. }
            ),
            "unexpected fail-closed reason: {error:?}"
        );
    }
}

#[test]
fn independent_sibling_dimensions_still_form_the_full_factorial() {
    let mut candidate = definition();
    candidate.dimensions = vec![
        numeric(
            "annual_need",
            "/experiment/resources/annualNeedUnitsPerPerson",
            &[100, 200],
        ),
        numeric(
            "condition_mortality",
            "/experiment/resources/maxConditionMortalityProbabilityPerMillion",
            &[0, 1_000],
        ),
    ];

    let points = candidate.expand().expect("sibling dimensions remain valid");
    assert_eq!(points.len(), 4);
    let treatments = points
        .iter()
        .map(|point| {
            (
                point
                    .run_config
                    .experiment
                    .resources
                    .annual_need_units_per_person,
                point
                    .run_config
                    .experiment
                    .resources
                    .max_condition_mortality_probability_per_million,
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        treatments.len(),
        4,
        "4 recorded coordinates must remain 4 executable treatments"
    );
}
