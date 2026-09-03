use anthrosim_core::{
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, SpatialAnalysisDomain, SpatialAnalysisExtent,
    SpatialBoundaryDeclaration, World, WorldConfig, assess_spatial_boundary,
    ids::CellId, rng::RngFactory,
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values,
    }
}

fn landscape(buffer: u32) -> LandscapeBundle {
    let side = buffer.checked_mul(2).unwrap().checked_add(1).unwrap();
    let count = usize::try_from(u64::from(side) * u64::from(side)).unwrap();
    LandscapeBundle::new(
        side,
        side,
        GridGeometry {
            origin_x: -i64::from(buffer),
            origin_y: i64::from(buffer) + 1,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:AUDIT-V4-BOUNDARY".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0); count],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(500); count],
            ),
        ],
    )
}

fn center_cell(bundle: &LandscapeBundle) -> CellId {
    let count = u64::from(bundle.width) * u64::from(bundle.height);
    (1..=count)
        .map(CellId::new)
        .find(|&cell| {
            let extent = bundle.cell_extent(cell).unwrap();
            extent.min_x == 0 && extent.min_y == 0
        })
        .expect("center physical cell must exist")
}

#[test]
fn m4_candidate_horizon_is_clear_if_and_only_if_buffer_reaches_radius() {
    let mut cases = 0_u64;
    for radius in 1_u16..=6 {
        for buffer in 0_u32..=7 {
            let bundle = landscape(buffer);
            let world = World::generate(
                WorldConfig::new(bundle.width, bundle.height),
                RngFactory::new(79_001 + u64::from(radius) * 100 + u64::from(buffer)),
            )
            .unwrap();
            let migration = MigrationConfig::synthetic_validation_v1()
                .with_candidate_radius_cells(radius);
            let analysis = SpatialAnalysisDomain::new(
                "audit-v4-center-cell",
                SpatialAnalysisExtent {
                    min_x: 0,
                    min_y: 0,
                    max_x: 1,
                    max_y: 1,
                },
            );
            let assessment = assess_spatial_boundary(
                &bundle,
                &world,
                &migration,
                SpatialBoundaryDeclaration::analyst_defined_crop(
                    "audit-v4-crop",
                    "Synthetic exhaustive buffer/radius adversary.",
                ),
                analysis,
                None,
            )
            .unwrap();

            assert_eq!(assessment.cells.len(), 1);
            assert_eq!(center_cell(&bundle), assessment.cells[0].cell);
            let expected_clear = buffer >= u32::from(radius);
            assert_eq!(
                assessment.m4_analysis_horizon_clear_of_boundary,
                expected_clear,
                "radius={radius} buffer={buffer}"
            );
            assert_eq!(
                assessment.cells[0].m4_candidate_set_truncated,
                !expected_clear,
                "radius={radius} buffer={buffer}"
            );
            assert_eq!(
                assessment.analysis_cells_with_truncated_m4_candidates,
                u64::from(!expected_clear),
                "radius={radius} buffer={buffer}"
            );
            if expected_clear {
                assert_eq!(
                    assessment.cells[0].m4_candidate_count,
                    assessment.m4_full_interior_candidate_count,
                    "radius={radius} buffer={buffer}"
                );
                assert_eq!(assessment.cells[0].m4_missing_candidate_count, 0);
            } else {
                assert!(assessment.cells[0].m4_missing_candidate_count > 0);
            }
            cases += 1;
        }
    }
    println!("audit_v4_boundary_matrix_cases={cases}");
    assert_eq!(cases, 48);
}
