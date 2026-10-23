use bluecity::{
    city::{City, Kind},
    civic::{self, CivicState, Hazard, ScenarioResult},
    content::{self, CivicEvent, Metric, OrdinanceId},
};
use std::collections::BTreeSet;

#[test]
fn authored_catalogs_have_exact_counts_unique_text_and_real_conditions() {
    assert_eq!(
        (
            content::MILESTONES.len(),
            content::REQUESTS.len(),
            content::ACHIEVEMENTS.len(),
            content::SCENARIOS.len()
        ),
        (25, 12, 8, 8)
    );
    assert_eq!(
        (
            content::CHIRPER.len(),
            content::ADVISORS.len(),
            content::ADVISOR_TRIGGERS.len(),
            content::HEADLINES.len(),
            content::HINTS.len(),
            content::ORDINANCES.len()
        ),
        (150, 76, 90, 61, 20, 17)
    );
    for items in [
        &content::CHIRPER[..],
        &content::HEADLINES[..],
        &content::HINTS[..],
    ] {
        let texts: BTreeSet<_> = items.iter().map(|m| m.text).collect();
        assert_eq!(texts.len(), items.len());
        assert!(items
            .iter()
            .all(|m| m.text.len() > 20 && !m.goal.requirements.is_empty()));
    }
    assert!(content::HEADLINES
        .iter()
        .any(|m| m.text.starts_with("Mayor's Cat:")));
    assert!(content::ADVISOR_TRIGGERS
        .iter()
        .all(|t| t.line < 76 && !t.goal.requirements.is_empty()));
    assert_eq!(
        content::ADVISOR_TRIGGERS
            .iter()
            .map(|t| format!("{:?}", t.goal.requirements))
            .collect::<BTreeSet<_>>()
            .len(),
        90,
        "advisor conditions must describe ninety distinct rule situations"
    );
    assert_eq!(
        content::REQUESTS
            .iter()
            .map(|r| r.reward.perk.unwrap() as u8)
            .collect::<BTreeSet<_>>()
            .len(),
        12
    );
    assert!(content::MILESTONES
        .windows(2)
        .all(|m| m[0].population < m[1].population));
}

#[test]
fn monthly_goals_are_sustained_once_and_coordinator_is_idempotent() {
    let mut city = City::new(7, true);
    let mut government = CivicState::new(9);
    city.population = 500;
    city.power_supply = 10000;
    city.power_demand = 0;
    city.water_supply = 10000;
    city.water_demand = 0;
    let before = city.money;
    city.month = 1;
    government.month_end(&mut city);
    assert_eq!(government.milestone_bits & 7, 7);
    assert!(city.money > before);
    let cash = city.money;
    let hash = serde_json::to_vec(&government).unwrap();
    government.month_end(&mut city);
    assert_eq!(cash, city.money);
    assert_eq!(hash, serde_json::to_vec(&government).unwrap());
    city.month = 2;
    government.month_end(&mut city);
    assert_eq!(
        cash, city.money,
        "population grants cannot be farmed by the next monthly tick"
    );
    city.population = 1500;
    city.happiness = 80;
    city.month = 12;
    government.month_end(&mut city);
    assert_eq!(government.request_bits & 1, 0);
    city.happiness = 80;
    city.month = 13;
    government.month_end(&mut city);
    assert_eq!(government.request_bits & 1, 0);
    city.happiness = 80;
    city.month = 14;
    government.month_end(&mut city);
    assert_ne!(
        government.request_bits & 1,
        0,
        "mayoral residence requires three qualifying months after year one"
    );
}

#[test]
fn milestone_fireworks_are_saved_real_awards_and_expire_without_repeating() {
    use vesper3d::{
        runtime::snapshot,
        two_d::{GameLogic, Simulation},
    };
    let mut game = bluecity::BlueCity::new(7);
    for _ in 0..30 {
        game.state.city.advance_day();
    }
    let city = &game.state.city;
    assert_ne!(city.civic.milestone_bits, 0);
    assert!(!city.civic.celebrations.is_empty() && city.civic.celebrations.len() <= 4);
    for event in &city.civic.celebrations {
        let id = event.milestone.expect("actual milestone origin");
        assert_ne!(city.civic.milestone_bits & (1 << id), 0);
        let lot = city.tiles[event.tile as usize];
        assert!(lot.kind == Kind::CityHall || lot.population + lot.jobs > 0);
        assert_eq!((event.started_day, event.until_day), (30, 40));
    }
    let before = city.civic.celebrations.clone();
    let mut civic = std::mem::take(&mut game.state.city.civic);
    civic.month_end(&mut game.state.city);
    assert_eq!(
        civic.celebrations, before,
        "same monthly award cannot fire twice"
    );
    game.state.city.civic = civic;
    let frame = snapshot::save(&game, "Actual milestone fireworks").unwrap();
    let mut restored = bluecity::BlueCity::new(0);
    snapshot::restore(&mut restored, &frame).unwrap();
    assert_eq!(restored.state_hash(), game.state_hash());
    let mut bad = game.state.city.civic.clone();
    bad.celebrations[0].milestone = Some(25);
    assert!(bad.validate(game.state.city.tiles.len()).is_err());
    for _ in 0..10 {
        game.state.city.advance_day();
        restored.state.city.advance_day();
    }
    assert!(game.state.city.civic.celebrations.is_empty());
    assert_eq!(restored.state_hash(), game.state_hash());
}

#[test]
fn old_project_celebrations_keep_their_existing_wire_shape_and_meaning() {
    let old = serde_json::json!({"project":3,"tile":120,"started_day":90,"until_day":100});
    let record: civic::Celebration = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(record.project, 3);
    assert_eq!(record.milestone, None);
    assert_eq!(serde_json::to_value(record).unwrap(), old);
}

#[test]
fn ordinance_slots_costs_and_effects_are_authoritative() {
    let mut city = City::new(1, true);
    city.money = 100000;
    let mut g = CivicState::new(7);
    g.milestone_bits = (1 << 25) - 1;
    let cash = city.money;
    g.toggle_ordinance(&mut city, OrdinanceId::WaterConservation)
        .unwrap();
    assert_eq!(city.money, cash - 100);
    assert_eq!(g.modifiers().water_demand_percent, 85);
    g.toggle_ordinance(&mut city, OrdinanceId::EnergyEfficiency)
        .unwrap();
    g.toggle_ordinance(&mut city, OrdinanceId::PublicHealth)
        .unwrap();
    assert!(g
        .toggle_ordinance(&mut city, OrdinanceId::RentSupport)
        .is_err());
    city.health = 40;
    city.month = 1;
    g.month_end(&mut city);
    assert!(city.health > 40);
    assert!(city.expenses > 0);
    assert!(city.money < cash - 300);
    g.toggle_ordinance(&mut city, OrdinanceId::EnergyEfficiency)
        .unwrap();
    assert_eq!(g.modifiers().power_demand_percent, 100);
}

#[test]
fn bond_maturity_and_financial_choices_have_costs_and_cooldowns() {
    let mut city = City::new(7, true);
    city.population = 10000;
    city.money = 50000;
    let mut g = CivicState::new(2);
    g.milestone_bits = (1 << 25) - 1;
    g.issue_bond(&mut city, 10000).unwrap();
    assert_eq!(city.money, 59900);
    assert!(g.issue_bond(&mut city, 12345).is_err());
    g.bonds[0].months_left = 1;
    city.month = 1;
    g.month_end(&mut city);
    assert!(g.bonds.is_empty());
    assert!(city.money <= 49850);
    city.money = -100;
    city.debt = 10000;
    g.declare_bankruptcy(&mut city).unwrap();
    assert_eq!((city.money, city.debt), (5000, 0));
    assert!(g.issue_bond(&mut city, 10000).is_err());
    assert!(g.declare_bankruptcy(&mut city).is_err());
    assert_eq!(g.events[CivicEvent::DeclaredBankruptcy as usize], 1);
    let mut other = CivicState::new(3);
    city.money = -100;
    other.bailout(&mut city).unwrap();
    assert_eq!(other.bonds.len(), 1);
    assert!(city.taxes.iter().all(|&t| t >= 12));
    assert!(other.bailout(&mut city).is_err());
}

#[test]
fn a_century_of_neglected_bonds_stays_bounded_and_resumes_exactly() {
    use vesper3d::{
        runtime::snapshot,
        two_d::{GameLogic, Simulation},
    };
    let mut game = bluecity::BlueCity::new(7);
    // Established monthly-rule town: this isolates debt settlement from the
    // separate whole-world daily century CLI stress run.
    game.state.city = City::new(7, true);
    game.state.city.money = 0;
    game.state.city.civic.disaster_frequency = 0;
    game.state.city.civic.milestone_bits = (1 << 25) - 1;
    game.state.city.civic.request_bits = (1 << 12) - 1;
    game.state.city.civic.achievement_bits = u8::MAX;
    game.state.city.civic.bonds.push(civic::Bond {
        principal: 25000,
        months_left: 1,
        monthly_interest: 9999,
    });
    let mut resumed = None;
    let mut observed_receivership = false;
    for month in 1..=1200 {
        game.state.city.advance_month();
        game.state.city.validate().unwrap();
        assert!(game.state.city.money >= -civic::overdraft_limit(&game.state.city));
        assert!(game
            .state
            .city
            .civic
            .bonds
            .iter()
            .all(|b| b.monthly_interest <= 10000));
        if month == 1 {
            assert_eq!(game.state.city.civic.bonds[0].monthly_interest, 10000);
            assert!(game
                .state
                .city
                .messages
                .iter()
                .any(|m| m.contains("extension")));
        }
        if !observed_receivership
            && game.state.city.civic.events[CivicEvent::DeclaredBankruptcy as usize] > 0
        {
            observed_receivership = true;
            assert_eq!((game.state.city.money, game.state.city.debt), (5000, 0));
            assert!(game.state.city.civic.bonds.is_empty());
            assert!(game.state.city.funding.iter().all(|&f| f <= 70));
            assert_eq!(game.state.city.civic.bankruptcy_until, month + 12);
            let popup = game
                .state
                .city
                .civic
                .popups
                .iter()
                .find(|p| p.kind == civic::PopupKind::Bankruptcy)
                .unwrap();
            assert_eq!(popup.title, "Court receivership");
            assert!(
                popup.text.contains("overdraft limit")
                    && popup.text.contains("Credit remains closed until")
            );
        }
        if let Some(other) = &mut resumed {
            let other: &mut bluecity::BlueCity = other;
            other.state.city.advance_month();
            assert_eq!(other.state_hash(), game.state_hash(), "month {month}");
        }
        if month == 600 {
            let frame = snapshot::save(&game, "Neglected bond century midpoint").unwrap();
            let mut loaded = bluecity::BlueCity::new(0);
            snapshot::restore(&mut loaded, &frame).unwrap();
            assert_eq!(loaded.state_hash(), game.state_hash());
            resumed = Some(loaded);
        }
    }
    assert!(observed_receivership);
    let frame = snapshot::save(&game, "Neglected bond century complete").unwrap();
    let mut loaded = bluecity::BlueCity::new(0);
    snapshot::restore(&mut loaded, &frame).unwrap();
    assert_eq!(loaded.state_hash(), game.state_hash());
}

#[test]
fn court_default_is_explicit_and_cannot_be_prevented_by_manual_aid_cooldown() {
    let mut city = City::new(7, true);
    city.money = -200000;
    city.debt = 30000;
    city.month = 1;
    let mut civic = CivicState::new(7);
    civic.milestone_bits = (1 << 25) - 1;
    civic.last_bankruptcy_month = Some(0);
    assert!(civic.declare_bankruptcy(&mut city).is_err());
    let limit = civic::overdraft_limit(&city);
    let warning = civic::financial_emergency_text(&city);
    assert!(warning.contains(&format!("-${limit}")) && warning.contains("Review due 3/2026"));
    civic.month_end(&mut city);
    assert_eq!((city.money, city.debt), (5000, 0));
    assert_eq!(civic.last_bankruptcy_month, Some(1));
    assert_eq!(civic.events[CivicEvent::DeclaredBankruptcy as usize], 1);
    assert!(civic
        .notebook
        .iter()
        .any(|n| n.title == "Court receivership"));
    city.civic = civic;
    city.validate().unwrap();
}

#[test]
fn restructuring_refuses_a_debt_rollup_that_would_violate_saved_bond_limits() {
    let mut city = City::new(7, true);
    let mut civic = CivicState::new(7);
    civic.bonds = vec![
        civic::Bond {
            principal: 200000,
            months_left: 60,
            monthly_interest: 1000
        };
        2
    ];
    civic.validate(city.tiles.len()).unwrap();
    let before = (city.clone(), civic.clone());
    assert!(civic.restructure(&mut city).is_err());
    assert_eq!((city, civic), before);
}

#[test]
fn districts_and_heritage_enforce_ownership_and_demolition() {
    let mut city = City::new(7, true);
    city.money = 50000;
    let mut g = CivicState::new(9);
    let tiles = vec![0, 1, 2, 3];
    g.create_district(&mut city, "Old Quarter".into(), tiles.clone(), 4)
        .unwrap();
    assert!(g
        .create_district(&mut city, "Overlap".into(), tiles, 1)
        .is_err());
    let i = city
        .tiles
        .iter()
        .position(|t| t.kind == Kind::Residential && t.population > 0)
        .unwrap() as u32;
    g.designate_heritage(&mut city, i).unwrap();
    g.toggle_ordinance(&mut city, OrdinanceId::Heritage)
        .unwrap();
    assert!(g.can_bulldoze(i).is_err());
    assert_eq!(g.demolition_permission(i, true).unwrap(), 10000);
    g.remove_heritage(&mut city, i).unwrap();
    assert!(g.can_bulldoze(i).is_ok());
    g.validate(city.tiles.len()).unwrap();
}

#[test]
fn scheduled_disaster_recovery_is_saved_deterministic_and_not_farmable() {
    let mut a = City::new(7, true);
    let mut ga = CivicState::new(11);
    ga.set_disaster_frequency(0).unwrap();
    ga.schedule_disaster(Hazard::Meteor, 4).unwrap();
    let mut b = a.clone();
    let mut gb = ga.clone();
    for _ in 0..4 {
        ga.advance_day(&mut a);
        gb.advance_day(&mut b);
    }
    assert_eq!(ga, gb);
    assert_eq!(a.tiles, b.tiles);
    assert!(!ga.damage.is_empty());
    assert!(!ga.active_events.is_empty());
    assert!(ga.warnings.is_empty());
    a.money = 100000;
    ga.request_relief(&mut a).unwrap();
    assert!(ga.request_relief(&mut a).is_err());
    let tile = ga.damage[0].tile;
    let previous = ga.damage[0].previous.kind;
    ga.repair(&mut a, tile).unwrap();
    assert_eq!(a.tiles[tile as usize].kind, previous);
    assert_eq!(ga.events[CivicEvent::RepairedDamage as usize], 1);
    ga.drill(&mut a).unwrap();
    assert!(ga.drill(&mut a).is_err());
    assert!(ga.modifiers().disaster_reduction > 0);
    let bytes = serde_json::to_vec(&ga).unwrap();
    let restored: CivicState = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, ga);
}

#[test]
fn scenario_deadlines_and_sustained_success_have_real_outcomes() {
    let mut city = City::new(7, true);
    let mut g = CivicState::new(9);
    g.start_scenario(&mut city, 7).unwrap();
    assert_eq!(g.scenario.as_ref().unwrap().deadline_month, 36);
    assert!(city.tiles.iter().any(|t| t.population > 0));
    city.month = 36;
    g.month_end(&mut city);
    assert_eq!(g.scenario.as_ref().unwrap().result, ScenarioResult::Lost);
    g.start_scenario(&mut city, 0).unwrap();
    city.population = 4000;
    city.happiness = 90;
    city.health = 90;
    city.garbage = 0;
    city.debt = 0;
    for t in &mut city.tiles {
        t.pollution = 0;
    }
    for month in 1..=6 {
        city.month = month;
        city.happiness = 90;
        g.month_end(&mut city);
    }
    assert_eq!(g.scenario.as_ref().unwrap().result, ScenarioResult::Won);
    assert_eq!(g.events[CivicEvent::WonScenario as usize], 1);
}

fn scenario_city(id: u8) -> City {
    let mut city = City::new(7, true);
    let mut government = CivicState::new(9);
    government.start_scenario(&mut city, id).unwrap();
    city.civic = government;
    city
}

#[test]
fn all_eight_scenarios_have_authored_recipes_and_actual_starting_problems() {
    use bluecity::transport::{route_between, TransitMode};
    let mut signatures = BTreeSet::new();
    for id in 0..8 {
        let city = scenario_city(id);
        city.validate().unwrap();
        let def = content::SCENARIOS[id as usize];
        assert_eq!(
            (city.ext.seed, city.ext.style, city.ext.climate),
            (def.seed, def.style, def.climate)
        );
        assert_eq!(city.population, def.start_population);
        assert_eq!(city.money, def.start_money);
        assert!(
            !city.civic.current_facts(&city).meets(def.goal),
            "scenario {id} starts solved"
        );
        assert!(city
            .civic
            .notebook
            .iter()
            .any(|n| n.title == "Your first decisions"));
        let kinds: Vec<_> = Kind::ALL
            .iter()
            .map(|&k| city.tiles.iter().filter(|t| t.kind == k).count())
            .collect();
        signatures.insert(format!(
            "{:?}:{:?}:{:?}:{}",
            def.style, def.climate, kinds, city.debt
        ));
        match id {
            0 => {
                assert!(city
                    .tiles
                    .iter()
                    .any(|t| t.kind == Kind::Farm && t.jobs > 0));
                assert!(city
                    .tiles
                    .iter()
                    .filter(|t| t.kind == Kind::Residential)
                    .all(|t| t.level == 1));
                assert!(!city
                    .tiles
                    .iter()
                    .any(|t| matches!(t.kind, Kind::Coal | Kind::Gas)));
            }
            1 => {
                let a = city.idx(60, 64).unwrap();
                let b = city.idx(68, 64).unwrap();
                let path = route_between(&city, a, b, TransitMode::Bus).unwrap();
                assert!(path.contains(&(city.idx(63, 64).unwrap() as u32)));
                let mut broken = city.clone();
                let i = broken.idx(63, 64).unwrap();
                broken.tiles[i].kind = Kind::Empty;
                assert!(route_between(&broken, a, b, TransitMode::Bus).is_none());
                assert!(city.ext.transport.lines.is_empty());
            }
            2 => {
                assert!(city.civic.current_facts(&city).get(Metric::WaterCoverage) < 95);
                assert_eq!(
                    city.civic
                        .warnings
                        .iter()
                        .map(|w| w.impact_day)
                        .collect::<Vec<_>>(),
                    vec![180, 540]
                );
                assert!(city.ext.cells.iter().any(|c| c.outage_days == 45));
            }
            3 => {
                assert_eq!(
                    city.civic.current_facts(&city).get(Metric::OperationalPort),
                    0
                );
                assert!(city
                    .ext
                    .neighbors
                    .iter()
                    .any(|n| n.enabled && n.goods == 8000));
                assert!(city
                    .tile(94, 65)
                    .is_some_and(|t| t.kind == Kind::Seaport && !t.connected));
            }
            4 => {
                assert!(city.tiles.iter().any(|t| t.kind == Kind::University));
                assert!(city.ext.people.households.iter().all(|h| h.education == 15));
                assert_eq!(city.education, 20);
            }
            5 => {
                assert_eq!(city.civic.damage.len(), 8);
                assert!(city
                    .civic
                    .damage
                    .iter()
                    .all(|d| city.ext.cells[d.tile as usize].rubble));
                assert_eq!(
                    city.tiles.iter().filter(|t| t.kind == Kind::Fire).count(),
                    1
                );
                assert!(city.tiles.iter().any(|t| t.kind == Kind::MaintenanceDepot
                    && t.connected
                    && t.powered
                    && t.watered));
            }
            6 => {
                assert_eq!(
                    city.tiles.iter().filter(|t| t.kind == Kind::Coal).count(),
                    3
                );
                assert!(city.civic.current_facts(&city).get(Metric::RenewableShare) < 70);
            }
            7 => {
                assert_eq!(city.debt, 30000);
                assert_eq!(city.civic.bonds[0].principal, 25000);
                assert_eq!(city.funding, [150; 4]);
                assert!(city.taxes.iter().all(|&t| t >= 14));
            }
            _ => unreachable!(),
        }
        let encoded = serde_json::to_vec(&city).unwrap();
        let resumed: City = serde_json::from_slice(&encoded).unwrap();
        resumed.validate().unwrap();
        assert_eq!(resumed, city);
    }
    assert_eq!(signatures.len(), 8);
}

#[test]
fn scenario_recovery_changes_real_transport_water_port_damage_and_debt_objectives() {
    use bluecity::transport::TransitMode;
    let mut grid = scenario_city(1);
    grid.create_line(
        TransitMode::Bus,
        vec![
            grid.idx(58, 49).unwrap() as u32,
            grid.idx(70, 49).unwrap() as u32,
        ],
    )
    .unwrap();
    for _ in 0..12 {
        grid.advance_day();
    }
    assert!(grid.ext.transport.ridership > 0);
    assert!(grid
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| v.kind == bluecity::transport::VehicleKind::Bus));
    grid.validate().unwrap();

    let mut delta = scenario_city(2);
    let shortage = delta.civic.current_facts(&delta).get(Metric::WaterCoverage);
    delta.place(86, 66, Kind::WaterTower).unwrap();
    for _ in 0..15 {
        delta.advance_day();
    }
    assert!(delta.civic.current_facts(&delta).get(Metric::WaterCoverage) > shortage);
    delta.validate().unwrap();

    let mut harbor = scenario_city(3);
    let money = harbor.money;
    harbor.place(89, 64, Kind::Road).unwrap();
    harbor.place(90, 64, Kind::Road).unwrap();
    assert!(harbor.money < money);
    assert_eq!(
        harbor
            .civic
            .current_facts(&harbor)
            .get(Metric::OperationalPort),
        1
    );
    harbor.validate().unwrap();

    let mut ashes = scenario_city(5);
    let repairs: Vec<_> = ashes.civic.damage.iter().map(|d| d.tile).collect();
    let mut service = std::mem::take(&mut ashes.ext.services);
    for &tile in &repairs {
        service.queue_repair(&ashes, tile).unwrap();
    }
    ashes.ext.services = service;
    for _ in 0..20 {
        ashes.advance_day();
    }
    assert!(ashes.civic.damage.is_empty());
    assert_eq!(ashes.civic.events[CivicEvent::RepairedDamage as usize], 8);
    assert_eq!(ashes.ext.services.completed_repairs, 8);
    ashes.validate().unwrap();

    let mut capital = scenario_city(7);
    let money = capital.money;
    let mut civic = std::mem::take(&mut capital.civic);
    assert!(civic.repay_bond(&mut capital, 0).is_err());
    assert_eq!(capital.money, money);
    let bonds = civic.bonds.clone();
    assert!(civic.bailout(&mut capital).is_err());
    assert_eq!(capital.money, money);
    assert_eq!(capital.debt, 30000);
    assert_eq!(civic.bonds, bonds);
    assert!(capital.taxes.iter().all(|&t| t >= 12));
    capital.civic = civic;
    capital.validate().unwrap();
}

#[test]
fn village_campus_and_clean_power_scenarios_respond_to_player_investment() {
    let mut village = scenario_city(0);
    village.place(62, 50, Kind::Residential).unwrap();
    for _ in 0..35 {
        village.advance_day();
    }
    assert!(
        village.tile(62, 50).unwrap().population > 0,
        "new village housing must actually grow"
    );
    village.validate().unwrap();

    let mut campus = scenario_city(4);
    let schooling: u32 = campus
        .ext
        .people
        .households
        .iter()
        .map(|h| h.education as u32)
        .sum();
    campus.place(54, 54, Kind::Hospital).unwrap();
    for _ in 0..90 {
        campus.advance_day();
    }
    assert!(
        campus
            .ext
            .people
            .households
            .iter()
            .map(|h| h.education as u32)
            .sum::<u32>()
            > schooling
    );
    assert!(campus.ext.people.education_levels[1..].iter().sum::<u32>() > 0);
    assert!(
        campus.ext.people.employed > 0,
        "schooling must unlock real qualified jobs"
    );
    campus.validate().unwrap();

    let mut clean = scenario_city(6);
    let before = clean
        .civic
        .current_facts(&clean)
        .get(Metric::RenewableShare);
    clean.place(86, 66, Kind::Wind).unwrap();
    let i = clean.idx(86, 66).unwrap();
    assert_eq!(
        clean.power_output(i),
        0,
        "a foundation cannot contribute clean power"
    );
    assert_eq!(
        clean
            .civic
            .current_facts(&clean)
            .get(Metric::RenewableShare),
        before
    );
    clean.bulldoze(46, 46).unwrap();
    for _ in 0..15 {
        clean.advance_day();
    }
    assert!(clean.power_output(i) > 0);
    assert!(
        clean
            .civic
            .current_facts(&clean)
            .get(Metric::RenewableShare)
            > before
    );
    clean.validate().unwrap();
}

#[test]
fn districts_report_population_weighted_local_livability_instead_of_city_approval() {
    let mut city = City::new(7, true);
    city.money = 100000;
    let homes: Vec<_> = city
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.population > 0)
        .take(8)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(homes.len(), 8);
    for (n, &i) in homes.iter().enumerate() {
        let t = &mut city.tiles[i as usize];
        let good = n < 4;
        t.land_value = if good { 100 } else { 0 };
        t.pollution = if good { 0 } else { 100 };
        t.crime = t.pollution;
        t.traffic = t.pollution;
        t.powered = good;
        t.watered = good;
        t.connected = good;
    }
    let mut civic = CivicState::new(5);
    civic
        .create_district(&mut city, "Garden Ward".into(), homes[..4].to_vec(), 0)
        .unwrap();
    civic
        .create_district(&mut city, "Factory Ward".into(), homes[4..].to_vec(), 0)
        .unwrap();
    assert!(civic.districts[0].happiness > civic.districts[1].happiness + 50);
    city.happiness = 50;
    city.month = 1;
    civic.month_end(&mut city);
    assert!(civic.districts[0].happiness > civic.districts[1].happiness + 50);
    let weighted: u32 = homes[..4]
        .iter()
        .map(|&i| city.tiles[i as usize].population)
        .sum();
    assert_eq!(civic.districts[0].population, weighted);
}

#[test]
fn advisor_disaster_conditions_follow_live_warnings_damage_and_relief() {
    let mut city = scenario_city(5);
    let mut civic = std::mem::take(&mut city.civic);
    civic.schedule_disaster(Hazard::Flood, 10).unwrap();
    let f = civic.current_facts(&city);
    assert_eq!(f.get(Metric::ActiveWarnings), 1);
    assert_eq!(f.get(Metric::UnrepairedDamage), 8);
    assert_eq!(f.get(Metric::ReliefAvailable), 1);
    for line in [43, 45] {
        assert!(content::ADVISOR_TRIGGERS
            .iter()
            .any(|t| t.line == line && f.meets(t.goal)));
    }
    civic.request_relief(&mut city).unwrap();
    let f = civic.current_facts(&city);
    assert_eq!(f.get(Metric::ReliefAvailable), 0);
    assert!(!content::ADVISOR_TRIGGERS
        .iter()
        .any(|t| t.line == 45 && f.meets(t.goal)));
}

#[test]
fn civic_goals_require_open_funded_facilities_instead_of_foundations() {
    let city = scenario_city(4);
    let i = city
        .tiles
        .iter()
        .position(|t| t.kind == Kind::University)
        .unwrap();
    assert_eq!(civic::facts(&city).count(Kind::University, true), 1);
    for case in 0..5 {
        let mut blocked = city.clone();
        match case {
            0 => blocked.ext.cells[i].construction = 99,
            1 => blocked.ext.cells[i].outage_days = 1,
            2 => blocked.ext.cells[i].rubble = true,
            3 => blocked.ext.cells[i].abandoned = true,
            4 => blocked.funding[Kind::University.funding()] = 0,
            _ => unreachable!(),
        }
        assert_eq!(
            civic::facts(&blocked).count(Kind::University, true),
            0,
            "case {case}"
        );
        assert_eq!(civic::facts(&blocked).count(Kind::University, false), 1);
    }
}

#[test]
fn civic_kind_counts_keep_all_catalog_entries_and_first_encounter_order() {
    let mut city = City::new(7, false);
    let kinds = Kind::ALL.len();
    for (i, &kind) in Kind::ALL.iter().enumerate() {
        assert_eq!(
            kind as usize, i,
            "catalog enum must remain dense for indexed facts"
        );
    }
    for (i, tile) in city.tiles.iter_mut().enumerate() {
        *tile = bluecity::city::Tile::new(bluecity::city::Terrain::Grass);
        tile.kind = Kind::ALL[kinds - 1 - i % kinds];
        tile.connected = true;
        tile.powered = true;
        tile.watered = true;
        tile.fire = if i % 3 == 0 { 20 } else { 0 };
    }
    let facts = civic::facts(&city);
    assert_eq!(facts.counts.len(), kinds);
    for (first, &kind) in Kind::ALL.iter().rev().enumerate() {
        let mut expected = 0;
        let mut operating = 0;
        for i in (first..city.tiles.len()).step_by(kinds) {
            expected += 1;
            operating += u16::from(i % 3 != 0);
        }
        assert_eq!(facts.counts[first], (kind, expected, operating));
        assert_eq!(facts.count(kind, false), expected);
        assert_eq!(facts.count(kind, true), operating);
    }
}

#[test]
fn megaprojects_charge_installments_pause_and_complete_once() {
    let mut city = City::new(7, true);
    city.population = 30000;
    city.education = 90;
    city.money = 500000;
    for t in &mut city.tiles {
        t.powered = true;
        t.watered = true;
    }
    let mut g = CivicState::new(8);
    let tile = city
        .tiles
        .iter()
        .position(|t| t.kind == Kind::Empty && t.terrain != bluecity::city::Terrain::Water)
        .unwrap() as u32;
    g.start_project(&mut city, 0, tile).unwrap();
    assert_eq!(g.projects[0].paid, 30000);
    let after = city.money;
    city.money = 0;
    city.month = 1;
    g.month_end(&mut city);
    assert!(g.projects[0].paused);
    assert_eq!(g.projects[0].months_done, 0);
    city.money = after;
    city.power_supply = 100000;
    city.power_demand = 0;
    city.water_supply = 100000;
    city.water_demand = 0;
    for month in 2..=25 {
        city.month = month;
        g.month_end(&mut city);
    }
    assert!(g.projects[0].complete);
    assert_eq!(g.projects[0].paid, 120000);
    assert_eq!(city.tiles[tile as usize].kind, Kind::Arcology);
    assert_eq!(g.events[CivicEvent::CompletedProject as usize], 1);
    assert_eq!(g.modifiers().housing_capacity_percent, 125);
    assert!(g.start_project(&mut city, 0, tile).is_err());
}

#[test]
fn january_rewind_restores_world_progress_and_rejects_repeated_use() {
    let mut city = City::new(7, true);
    let mut g = CivicState::new(44);
    city.month = 12;
    g.day = 360;
    g.save_january(&city);
    let saved = city.clone();
    g.issue_bond(&mut city, 10000).unwrap();
    city.money -= 123;
    city.month = 15;
    g.day = 450;
    g.rewind_january(&mut city).unwrap();
    assert_eq!(city.tiles, saved.tiles);
    assert_eq!(
        (city.money, city.month, city.ext.clone()),
        (saved.money, saved.month, saved.ext)
    );
    assert!(g.bonds.is_empty());
    assert!(g.rewind_january(&mut city).is_err());
    let json = serde_json::to_vec(&g).unwrap();
    let copy: CivicState = serde_json::from_slice(&json).unwrap();
    assert_eq!(copy, g);
    copy.validate(city.tiles.len()).unwrap();
}

#[test]
fn metrics_do_not_hide_congestion_in_empty_land_and_history_has_sixteen_graphs() {
    let mut city = City::new(7, false);
    for t in &mut city.tiles {
        t.kind = Kind::Empty;
        t.population = 0;
    }
    city.tiles[0].kind = Kind::Road;
    city.tiles[0].connected = true;
    city.tiles[0].traffic = 80;
    assert_eq!(civic::facts(&city).get(Metric::Traffic), 80);
    let mut g = CivicState::new(2);
    city.month = 1;
    g.month_end(&mut city);
    assert_eq!(g.history[0].values.len(), 16);
    assert_eq!(civic::GRAPH_NAMES.len(), 16);
    assert!(!g.headlines.is_empty());
}

#[test]
fn all_ten_hazards_have_real_saved_effects_and_elections_have_outcomes() {
    for kind in Hazard::ALL {
        let mut city = City::new(7, true);
        city.tiles[300].kind = Kind::Nuclear;
        city.tiles[301].kind = Kind::Airport;
        let mut g = CivicState::new(88);
        g.disaster_frequency = 0;
        g.schedule_disaster(kind, 1).unwrap();
        let cash = city.money;
        g.advance_day(&mut city);
        assert!(g.active_events.iter().any(|e| e.kind == kind), "{kind:?}");
        if kind == Hazard::FriendlyUfo {
            assert!(city.money > cash);
            assert!(g.damage.is_empty());
        } else {
            assert!(!g.damage.is_empty(), "{kind:?}");
        }
        g.validate(city.tiles.len()).unwrap();
    }
    let mut city = City::new(7, true);
    let mut g = CivicState::new(8);
    city.month = 48;
    city.happiness = 100;
    city.health = 100;
    city.unemployment = 0;
    city.garbage = 0;
    g.month_end(&mut city);
    assert!(g.elections[0].won);
    assert_eq!(g.events[CivicEvent::WonElection as usize], 1);
    city.month = 96;
    city.happiness = 0;
    city.health = 0;
    city.unemployment = 100;
    city.garbage = 100;
    g.month_end(&mut city);
    assert!(!g.elections[1].won);
    assert_eq!(g.events[CivicEvent::LostElection as usize], 1);
}

#[test]
fn council_slots_require_won_elections_and_never_repeat_within_a_term() {
    let mut city = City::new(7, true);
    city.money = 100000;
    let mut g = CivicState::new(8);
    g.milestone_bits = (1 << 25) - 1;
    g.request_bits = (1 << 12) - 1;
    city.population = 200000;
    city.month = 1;
    g.month_end(&mut city);
    assert_eq!(
        g.ordinance_slots, 3,
        "population does not create council mandates"
    );
    for term in 1..=6 {
        city.month = term * 48;
        city.happiness = 100;
        city.health = 100;
        city.unemployment = 0;
        city.garbage = 0;
        g.month_end(&mut city);
        let expected = (3 + term).min(8) as u8;
        assert_eq!(g.ordinance_slots, expected);
        g.month_end(&mut city);
        assert_eq!(g.ordinance_slots, expected, "a term is processed once");
    }
    city.month = 7 * 48;
    city.happiness = 0;
    city.health = 0;
    city.unemployment = 100;
    city.garbage = 100;
    g.month_end(&mut city);
    assert!(!g.elections.last().unwrap().won);
    assert_eq!(g.ordinance_slots, 8);

    let mut migrated = CivicState::new(10);
    migrated.ordinance_slots = 4;
    city.month += 1;
    migrated.month_end(&mut city);
    assert_eq!(
        migrated.ordinance_slots, 4,
        "released policy capacity is preserved"
    );
}

fn literal_january_archive(value: &serde_json::Value) -> Vec<u8> {
    let bytes = serde_json::to_vec(value).unwrap();
    let mut archive = Vec::new();
    for chunk in bytes.chunks(128) {
        archive.push((chunk.len() - 1) as u8);
        archive.extend_from_slice(chunk);
    }
    archive
}

#[test]
fn framed_corrupt_january_checkpoints_are_rejected_without_changing_the_city() {
    use bluecity::BlueCity;
    use vesper3d::{
        runtime::snapshot,
        two_d::{GameLogic, Simulation},
    };
    let mut live = BlueCity::new(19);
    live.state.city.advance_day();
    live.state.city.validate().unwrap();
    let mut continuation = BlueCity::new(0);
    continuation.state = live.state.clone();
    for case in 0..12 {
        let mut forged = BlueCity::new(0);
        forged.state = live.state.clone();
        let jan = forged.state.city.civic.january.as_mut().unwrap();
        match case {
            0 => jan.values[2] = -1,
            1 => jan.values[4] = 356,
            2 => jan.values[1] = i64::from(u32::MAX) + 1,
            3 => jan.taxes[0] = 21,
            4 => jan.tiles[0].level = 6,
            5 => jan.government = vec![255, 0, 0],
            6 => jan.government = literal_january_archive(&serde_json::json!({"january":{}})),
            7 => {
                jan.government =
                    literal_january_archive(&serde_json::json!({"ordinances":1_u32 << 20}))
            }
            8 => jan.government = literal_january_archive(&serde_json::json!({"approval":101})),
            9 => {
                jan.government = literal_january_archive(
                    &serde_json::json!({"warnings":[{"kind":"Flood","impact_day":10,"tile":u32::MAX,"announced":false}]}),
                )
            }
            10 => jan.world.people.households[0].home = u32::MAX,
            11 => jan.values[5] = i64::MAX,
            _ => unreachable!(),
        }
        let bytes =
            snapshot::save(&forged, "Correctly framed, invalid January checkpoint").unwrap();
        let before = live.state_hash();
        assert!(snapshot::restore(&mut live, &bytes).is_err(), "case {case}");
        assert_eq!(
            live.state_hash(),
            before,
            "case {case} changed the running city"
        );
        live.state.city.advance_day();
        continuation.state.city.advance_day();
        assert_eq!(
            live.state_hash(),
            continuation.state_hash(),
            "case {case} broke continuation"
        );
    }
}

#[test]
fn rewind_validation_is_transactional_even_for_directly_damaged_checkpoints() {
    let mut city = City::new(7, true);
    let mut g = CivicState::new(8);
    g.save_january(&city);
    g.january.as_mut().unwrap().values[4] = 256;
    let original_city = city.clone();
    let original_government = g.clone();
    assert!(g.rewind_january(&mut city).is_err());
    assert_eq!(city, original_city);
    assert_eq!(g, original_government);
}

#[test]
fn yearly_reviews_scores_medals_and_century_story_stay_bounded() {
    let mut city = City::new(7, true);
    let mut g = CivicState::new(9);
    g.disaster_frequency = 0;
    for month in 1..=1200 {
        city.month = month;
        g.month_end(&mut city);
        if month == 12 {
            assert!(g.notebook.iter().any(|n| n.title == "Annual Budget Review"));
            assert!(g
                .popups
                .iter()
                .any(|p| p.kind == civic::PopupKind::AnnualReview));
            assert_eq!(g.history.len(), 12);
        }
    }
    assert!(
        g.notebook.len() <= 120
            && g.popups.len() <= 32
            && g.elections.len() <= 24
            && g.history.len() <= 240
            && g.chirper.len() <= 24
            && g.advisors.len() <= 12
            && g.headlines.len() <= 16
    );
    assert_eq!(g.history.last().unwrap().month, 1200);
    assert!(g
        .notebook
        .iter()
        .any(|n| n.title == "Annual Budget Review" && n.month == 1200));
    g.validate(city.tiles.len()).unwrap();
    city.civic = g.clone();
    let rating = civic::evaluation(&city);
    assert!(rating.score <= 1000);
    assert!(["A", "B", "C", "D", "F"].contains(&rating.grade));
    city.population = 10000;
    city.health = 100;
    city.happiness = 100;
    city.education = 100;
    city.unemployment = 0;
    city.month = 1201;
    g.month_end(&mut city);
    assert!(g.medal_bits != 0);
    city.population = 0;
    city.civic = g;
    assert_eq!(civic::evaluation(&city).score, 0);
    assert!(
        !civic::evaluation(&city).medals.is_empty(),
        "earned medals survive a later decline"
    );
}

#[test]
fn each_ordinance_has_its_published_kind_of_effect() {
    for id in OrdinanceId::ALL {
        let mut city = City::new(7, true);
        city.money = 100000;
        let mut g = CivicState::new(2);
        g.milestone_bits = (1 << 25) - 1;
        g.toggle_ordinance(&mut city, id).unwrap();
        city.health = 40;
        city.happiness = 40;
        city.education = 40;
        city.garbage = 60;
        city.transport_usage = 0;
        city.population = 5000;
        city.jobs = 1000;
        for t in &mut city.tiles {
            t.pollution = 60;
            t.crime = 60;
            if t.kind.is_road() {
                t.connected = true;
                t.traffic = 80;
            }
        }
        city.month = 1;
        g.month_end(&mut city);
        assert!(city.expenses > 0, "{id:?} has a recurring budget cost");
        match id {
            OrdinanceId::CleanAir => assert!(city.tiles.iter().all(|t| t.pollution < 60)),
            OrdinanceId::FreeTransit => assert!(city.transport_usage > 0),
            OrdinanceId::Recycling => assert!(city.garbage < 60),
            OrdinanceId::SmallBusinessGrants => assert!(city.income > 0),
            OrdinanceId::Heritage => assert!(g.active(id)),
            OrdinanceId::NoiseControl => {
                assert!(city.happiness > 40);
                assert_eq!(g.modifiers().trade_percent, 90);
            }
            OrdinanceId::RentSupport => assert!(city.happiness > 40),
            OrdinanceId::Childcare => assert!(city.health > 40 && city.happiness > 40),
            OrdinanceId::PublicHealth => assert!(city.health > 40),
            OrdinanceId::WaterConservation => assert_eq!(g.modifiers().water_demand_percent, 85),
            OrdinanceId::EnergyEfficiency => assert_eq!(g.modifiers().power_demand_percent, 88),
            OrdinanceId::IndustrialSafety => assert!(g.modifiers().disaster_reduction >= 15),
            OrdinanceId::TruckRoutes => assert!(city
                .tiles
                .iter()
                .filter(|t| t.kind.is_road())
                .all(|t| t.traffic < 80)),
            OrdinanceId::TourismPromotion => assert!(city.income > 0),
            OrdinanceId::SchoolMeals => assert!(city.health > 40 && city.education > 40),
            OrdinanceId::PoliceAccountability => assert!(city.tiles.iter().all(|t| t.crime < 60)),
            OrdinanceId::DisasterPreparedness => assert!(g.modifiers().disaster_reduction >= 30),
        }
    }
}

#[test]
fn social_links_use_real_households_and_notifications_are_dismissable() {
    let mut city = City::new(7, true);
    city.ext
        .people
        .households
        .push(bluecity::people::Household {
            id: 481,
            home: 300,
            size: 3,
            education: 65,
            health: 77,
            income_class: 2,
            employer: Some(301),
            age_days: 40,
        });
    let mut g = CivicState::new(7);
    city.month = 1;
    g.month_end(&mut city);
    let post = g.chirper.last().unwrap();
    assert_eq!(post.citizen, Some(481));
    assert_eq!(post.tile, Some(300));
    let person = post.person.as_ref().unwrap();
    assert_eq!((person.id, person.education, person.health), (481, 65, 77));
    let popup = g.popups[0].id;
    g.dismiss_popup(popup).unwrap();
    assert!(g.dismiss_popup(popup).is_err());
    city.ext.people.households.clear();
    assert_eq!(
        g.chirper.last().unwrap().person.as_ref().unwrap().id,
        481,
        "archived profile remains readable after migration"
    );
}

#[test]
fn civic_windows_share_reachable_actions_and_paginate_their_real_rows() {
    use bluecity::ui_model::{self, Command, Details, Window};
    use vesper3d::two_d::GameLogic;
    let mut game = bluecity::BlueCity::new(7);
    game.state.city.money = 200000;
    game.command(Command::Open(Window::Council));
    let controls = ui_model::controls(&game.state);
    for command in [
        Command::NewMap,
        Command::Surprise,
        Command::Save,
        Command::Load,
        Command::Pause,
    ] {
        assert!(controls.iter().any(|c| c.command == command));
    }
    game.command(Command::District);
    assert_eq!(game.state.city.civic.districts.len(), 1);
    let id = game.state.city.civic.districts[0].id;
    game.command(Command::DistrictPolicy(id));
    assert_eq!(game.state.city.civic.districts[0].specialization, 1);
    game.command(Command::RenameDistrict(id));
    assert_eq!(game.state.city.civic.districts[0].name, "Old Town");
    game.command(Command::BondAmount(10000));
    assert_eq!(game.state.city.civic.bonds.len(), 1);
    game.command(Command::RepayBond(0));
    assert!(game.state.city.civic.bonds.is_empty());
    game.command(Command::Disaster(2));
    for _ in 0..3 {
        game.state.city.advance_day();
    }
    assert!(!game.state.city.civic.damage.is_empty());
    game.command(Command::Relief);
    assert!(game.state.city.civic.relief_claimed);
    let tile = game.state.city.civic.damage[0].tile;
    let cash = game.state.city.money;
    game.command(Command::Repair(tile));
    assert!(game.state.city.civic.damage.iter().any(|d| d.tile == tile));
    assert!(game.state.city.ext.services.repair_requests.is_empty());
    assert_eq!(game.state.city.money, cash);
    assert!(game.state.notice.contains("maintenance depot"));
    game.command(Command::Details(Details::Milestone(0)));
    assert!(game
        .state
        .ui
        .card
        .as_ref()
        .unwrap()
        .body
        .contains("currently"));
    let rect = ui_model::card_rect(&game.state);
    assert!(rect.h <= 210 && rect.y + rect.h == 366);
    game.command(Command::Dismiss);
    assert!(game.state.ui.card.is_none());
    for window in Window::ALL {
        game.command(Command::Open(window));
        if let Some(rows) = bluecity::civic_ui::records(&game.state, window) {
            let pages = rows.len().div_ceil(6).max(1);
            assert_eq!(ui_model::page_count(&game.state) as usize, pages);
            for page in 0..pages {
                game.state.ui.page = page as u16;
                let controls = ui_model::controls(&game.state);
                for row in rows.iter().skip(page * 6).take(6) {
                    if let Some((cmd, _)) = row.action {
                        assert!(
                            controls.iter().any(|c| c.command == cmd),
                            "{window:?}: {:?}",
                            row.title
                        );
                    }
                }
                assert!(controls.iter().all(|c| c.rect.x >= 0
                    && c.rect.y >= 0
                    && c.rect.x + c.rect.w <= 800
                    && c.rect.y + c.rect.h <= 450));
            }
        }
    }
    game.command(Command::RemoveDistrict(id));
    assert!(game.state.city.civic.districts.is_empty());
}

#[test]
fn neighbor_trade_controls_edit_real_contracts_and_evaluation_exposes_people() {
    use bluecity::ui_model::{Command, Window};
    use vesper3d::two_d::GameLogic;
    let mut game = bluecity::BlueCity::new(7);
    game.state.city.money = 100000;
    assert_eq!(game.state.city.ext.neighbors.len(), 3);
    game.command(Command::Open(Window::Trade));
    let rows = bluecity::civic_ui::records(&game.state, Window::Trade).unwrap();
    assert!(rows
        .iter()
        .any(|r| r.action == Some((Command::NeighborPower(0, 5000), "Power +"))));
    game.command(Command::NeighborPower(0, 5000));
    assert_eq!(game.state.city.ext.neighbors[0].power, 5000);
    game.command(Command::NeighborGoods(0, 1000));
    assert_eq!(game.state.city.ext.neighbors[0].goods, 1000);
    game.command(Command::NeighborTariff(0, 1));
    assert_eq!(game.state.city.ext.neighbors[0].tariff, 3);
    game.command(Command::NeighborToggle(0));
    assert!(game.state.city.ext.neighbors[0].enabled);
    game.command(Command::NeighborPower(0, 200000));
    assert_eq!(game.state.city.ext.neighbors[0].power, 100000);
    game.command(Command::NeighborGoods(0, -200000));
    assert_eq!(game.state.city.ext.neighbors[0].goods, 0);
    let rows = bluecity::civic_ui::records(&game.state, Window::Evaluation).unwrap();
    assert_eq!(
        rows.iter()
            .filter(|r| r.title.starts_with("Happiness factor:"))
            .count(),
        16
    );
    assert!(rows.iter().any(|r| r.title == "People and ages"));
    assert!(rows.iter().any(|r| r.title == "Workforce"));
}

#[test]
fn dense_civic_monthly_runtime_keeps_bounded_complete_history() {
    let mut city = bluecity::mayor::dense_fixture(7, 15000);
    let mut government = std::mem::take(&mut city.civic);
    let mut micros = Vec::new();
    for month in 121..=144 {
        city.month = month;
        city.ext.day = u64::from(month) * 30;
        let start = std::time::Instant::now();
        government.month_end(&mut city);
        micros.push(start.elapsed().as_micros());
    }
    assert_eq!(government.history.len(), 24);
    assert!(government
        .notebook
        .iter()
        .any(|n| n.title == "Annual Budget Review"));
    government.validate(city.tiles.len()).unwrap();
    micros.sort_unstable();
    eprintln!(
        "civic_dense_month_end_us median={} p95={} max={}",
        micros[micros.len() / 2],
        micros[micros.len() * 95 / 100],
        micros.last().unwrap()
    );
}
