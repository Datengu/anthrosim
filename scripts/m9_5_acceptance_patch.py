from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Ledger boundary semantics: a visit crossing a resource boundary is split exactly,
# rather than attributed as a whole period on either side.
path = "crates/anthrosim-core/src/temporary_resource.rs"
anchor = '''    #[test]
    fn visitor_destination_cannot_change_inside_one_resource_period() {
        let mut ledger = TemporaryResourceLedger::new(1, 0);
        ledger
            .accrue_until(
                1,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();
        let error = ledger
            .accrue_until(
                2,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(2),
                    destination: CellId::new(3),
                }],
            )
            .unwrap_err();
        assert!(matches!(
            error,
            TemporaryResourceAccountingError::VisitorDestinationChanged { .. }
        ));
    }
}'''
replacement = '''    #[test]
    fn visitor_destination_cannot_change_inside_one_resource_period() {
        let mut ledger = TemporaryResourceLedger::new(1, 0);
        ledger
            .accrue_until(
                1,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();
        let error = ledger
            .accrue_until(
                2,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(2),
                    destination: CellId::new(3),
                }],
            )
            .unwrap_err();
        assert!(matches!(
            error,
            TemporaryResourceAccountingError::VisitorDestinationChanged { .. }
        ));
    }

    #[test]
    fn visit_crossing_resource_boundary_is_split_without_double_counting() {
        use crate::{WorldConfig, rng::RngFactory};

        let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(95)).unwrap();
        let journey = TemporaryJourneyId::new(1);
        let destination = CellId::new(2);
        let visiting = HouseholdPresence::Visiting {
            journey,
            destination,
        };

        let mut ledger = TemporaryResourceLedger::new(1, 0);
        ledger
            .accrue_until(90, &[HouseholdPresence::AtResidence])
            .unwrap();
        ledger.accrue_until(91, &[visiting]).unwrap();
        let first = ledger.snapshot_period(91, &[visiting], &world).unwrap();
        assert_eq!(first.households[0].at_residence_days, 90);
        assert_eq!(first.households[0].visiting_days, 1);
        assert_eq!(first.households[0].total_days().unwrap(), 91);

        ledger.reset_after_settlement(91).unwrap();
        ledger.accrue_until(95, &[visiting]).unwrap();
        ledger
            .accrue_until(182, &[HouseholdPresence::AtResidence])
            .unwrap();
        let second = ledger
            .snapshot_period(182, &[HouseholdPresence::AtResidence], &world)
            .unwrap();
        assert_eq!(second.households[0].visiting_days, 4);
        assert_eq!(second.households[0].at_residence_days, 87);
        assert_eq!(second.households[0].total_days().unwrap(), 91);
        assert_eq!(
            first.households[0].visiting_days + second.households[0].visiting_days,
            5
        );
    }
}'''
replace_once(path, anchor, replacement)


# M3 exact need partitioning and mixed-cell supply consequences.
path = "crates/anthrosim-core/src/resources.rs"
anchor = '''    #[test]
    fn seasonal_factor_has_expected_peak_and_trough() {
        assert_eq!(seasonal_factor_permille(0, 0, 1_000), 2_000);
        assert_eq!(seasonal_factor_permille(182, 0, 1_000), 0);
        assert_eq!(seasonal_factor_permille(100, 100, 0), 1_000);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 0), 1_000);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 500), 1_400);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 1_000), 1_800);

        let invalid =
            ResourceConfig::synthetic_validation_v1().with_seasonality_scale_permille(1_001);
        assert!(matches!(
            validate_resource_config(&invalid),
            Err(ResourceConfigError::InvalidSeasonalityScale { .. })
        ));
    }
}'''
replacement = '''    #[test]
    fn seasonal_factor_has_expected_peak_and_trough() {
        assert_eq!(seasonal_factor_permille(0, 0, 1_000), 2_000);
        assert_eq!(seasonal_factor_permille(182, 0, 1_000), 0);
        assert_eq!(seasonal_factor_permille(100, 100, 0), 1_000);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 0), 1_000);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 500), 1_400);
        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 1_000), 1_800);

        let invalid =
            ResourceConfig::synthetic_validation_v1().with_seasonality_scale_permille(1_001);
        assert!(matches!(
            validate_resource_config(&invalid),
            Err(ResourceConfigError::InvalidSeasonalityScale { .. })
        ));
    }

    #[test]
    fn duration_weighting_conserves_one_and_five_day_visits_and_treats_transit_as_home() {
        let one_day = TemporaryResourcePresenceDays {
            at_residence_days: 88,
            outbound_transit_days: 1,
            visiting_days: 1,
            return_transit_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
        };
        assert_eq!(one_day.total_days().unwrap(), 91);
        assert_eq!(duration_weighted_needs(91, &one_day).unwrap(), (90, 1));

        let five_days = TemporaryResourcePresenceDays {
            at_residence_days: 84,
            outbound_transit_days: 1,
            visiting_days: 5,
            return_transit_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
        };
        assert_eq!(five_days.home_provisioning_days().unwrap(), 86);
        assert_eq!(five_days.total_days().unwrap(), 91);
        assert_eq!(duration_weighted_needs(91, &five_days).unwrap(), (86, 5));

        let tie = TemporaryResourcePresenceDays {
            at_residence_days: 1,
            visiting_days: 1,
            visitor_destination: Some(crate::ids::CellId::new(2)),
            ..TemporaryResourcePresenceDays::default()
        };
        assert_eq!(duration_weighted_needs(1, &tie).unwrap(), (1, 0));
    }

    #[test]
    fn mixed_cell_supply_reconciles_to_one_household_satisfaction_without_losing_need() {
        let world = World::generate(WorldConfig::new(2, 1), RngFactory::new(119)).unwrap();
        let mut population = Population::initialize(
            PopulationConfig::new(1).with_target_household_size(1),
            &world,
            RngFactory::new(119),
        )
        .unwrap();
        let household = HouseholdId::new(1);
        let residence = population.household_location(household).unwrap();
        let destination = if residence == crate::ids::CellId::new(1) {
            crate::ids::CellId::new(2)
        } else {
            crate::ids::CellId::new(1)
        };
        let residence_index = cell_index_for(&world, residence).unwrap();
        let destination_index = cell_index_for(&world, destination).unwrap();

        let mut config = ResourceConfig::synthetic_validation_v1();
        config.periods_per_year = 1;
        config.annual_need_units_per_person = 100;
        config.annual_regeneration_units_per_productivity = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        let mut system = ResourceSystem::initialize(&world, &config).unwrap();
        system.cell_food_stock.fill(0);
        system.cell_food_stock[destination_index] = 100;
        system.initial_food_stock = 100;

        let presence = TemporaryResourcePresenceDays {
            at_residence_days: 183,
            visiting_days: 182,
            visitor_destination: Some(destination),
            ..TemporaryResourcePresenceDays::default()
        };
        assert_eq!(duration_weighted_needs(100, &presence).unwrap(), (50, 50));
        let period = TemporaryResourcePeriod {
            schema_version: TemporaryResourcePeriod::CURRENT_SCHEMA_VERSION,
            start_day: 0,
            end_day: 365,
            households: vec![presence],
        };
        let before_condition = population.condition_at_index(0).unwrap();
        let mut rngs = ResourceRngs::new(RngFactory::new(119));
        let mut events = EventLog::new();

        system
            .process_period_recorded_with_presence(
                &mut population,
                &ResourcePeriodContext {
                    world: &world,
                    config: &config,
                    period_index_in_year: 0,
                    day: 365,
                },
                &mut rngs.scarcity_mortality,
                &mut events,
                Some(&period),
            )
            .unwrap();

        assert_eq!(system.cell_food_stock[residence_index], 0);
        assert_eq!(system.cell_food_stock[destination_index], 50);
        assert_eq!(system.harvested_food, 50);
        assert_eq!(system.unmet_need, 50);
        assert_eq!(system.harvested_food + system.unmet_need, 100);
        assert!(population.condition_at_index(0).unwrap() < before_condition);
        system.validate_accounting().unwrap();
    }
}'''
replace_once(path, anchor, replacement)

print("M9.5 acceptance-test patch applied")
