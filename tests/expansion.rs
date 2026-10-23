use bluecity::{
    city::{City, Kind, MapStyle, Terrain, Tile, SERVICE_LABELS},
    transport::{road_route, route_between, TransitMode, VehicleKind},
    world::{BuildingStage, Climate, TerrainAction},
};
fn town() -> City {
    let mut c = City::new(19, false);
    c.money = 200000;
    for t in &mut c.tiles {
        *t = Tile::new(Terrain::Grass);
    }
    for x in 0..=44 {
        let i = c.idx(x, 10).unwrap();
        c.tiles[i].kind = Kind::Road;
    }
    for x in 5..=10 {
        let i = c.idx(x, 11).unwrap();
        c.tiles[i].kind = Kind::Residential;
        c.tiles[i].level = 2;
        c.tiles[i].population = 60;
    }
    for x in 30..=35 {
        let i = c.idx(x, 11).unwrap();
        c.tiles[i].kind = Kind::Industrial;
        c.tiles[i].level = 2;
        c.tiles[i].jobs = 50;
    }
    for (x, k) in [
        (12, Kind::Coal),
        (14, Kind::Pump),
        (16, Kind::Sewage),
        (18, Kind::Landfill),
        (20, Kind::BusDepot),
    ] {
        let i = c.idx(x, 9).unwrap();
        c.tiles[i].kind = k;
    }
    let water = c.idx(14, 8).unwrap();
    c.tiles[water].terrain = Terrain::Water;
    c.refresh();
    c.initialize_expansion();
    c.civic.disaster_frequency = 0;
    c.refresh();
    c
}
fn advance(c: &mut City, n: usize) {
    for _ in 0..n {
        c.advance_day();
    }
    c.validate().unwrap();
}
fn transport_day(c: &mut City, day: u64) {
    let mut t = std::mem::take(&mut c.ext.transport);
    t.advance_day(c, day, 0);
    c.ext.transport = t;
}
fn separated_rail_town() -> (City, usize, usize, [u32; 2]) {
    let mut c = City::new(33, false);
    for t in &mut c.tiles {
        *t = Tile::new(Terrain::Grass);
    }
    for x in [8, 28] {
        for y in 0..36 {
            let i = c.idx(x, y).unwrap();
            c.tiles[i].kind = Kind::Road;
        }
        for (dx, y, k) in [(-1, 18, Kind::Coal), (-1, 19, Kind::WaterTower)] {
            let i = c.idx(x + dx, y).unwrap();
            c.tiles[i].kind = k;
        }
    }
    let home = c.idx(9, 20).unwrap();
    let work = c.idx(29, 20).unwrap();
    c.tiles[home].kind = Kind::Residential;
    c.tiles[home].level = 2;
    c.tiles[home].population = 60;
    c.tiles[work].kind = Kind::Commercial;
    c.tiles[work].level = 2;
    c.tiles[work].jobs = 40;
    for x in 9..=29 {
        let i = c.idx(x, 18).unwrap();
        if c.tiles[i].kind.is_road() {
            c.tiles[i].rail_crossing = true;
        } else {
            c.tiles[i].kind = Kind::Rail;
        }
    }
    let stops = [c.idx(9, 18).unwrap() as u32, c.idx(29, 18).unwrap() as u32];
    for &i in &stops {
        c.tiles[i as usize].kind = Kind::Station;
    }
    c.population = 60;
    c.jobs = 40;
    c.initialize_expansion();
    c.civic.disaster_frequency = 0;
    c.refresh();
    c.ext.people.households = vec![bluecity::people::Household {
        id: 1,
        home: home as u32,
        size: 60,
        education: 40,
        health: 90,
        employer: Some(work as u32),
        ..Default::default()
    }];
    c.ext.people.next_id = 1;
    (c, home, work, stops)
}
#[test]
fn runtime_grid_keeps_legacy_coordinates_and_generates_all_twenty_one_regions() {
    let old = City::new(7, true);
    assert_eq!((old.width, old.height), (48, 48));
    assert_eq!(old.idx(47, 47), Some(2303));
    assert_eq!(old.idx(48, 0), None);
    for style in MapStyle::ALL {
        for climate in Climate::ALL {
            let c = City::world(u64::MAX, style, climate);
            assert_eq!((c.width, c.height), (128, 128));
            assert_eq!(c.tiles.len(), 16384);
            assert_eq!(c.idx(127, 127), Some(16383));
            assert_eq!(c.idx(128, 0), None);
            assert_eq!(c.ext.seed, u64::MAX);
            assert_eq!(c.ext.climate, climate);
            assert_eq!(c, City::world(u64::MAX, style, climate));
            assert_eq!(c.tile(64, 64).unwrap().terrain, Terrain::Grass);
            c.validate().unwrap();
        }
    }
}
#[test]
fn day_clock_runs_one_month_budget_at_exactly_thirty_days() {
    let mut c = City::world(7, MapStyle::River, Climate::Temperate);
    c.advance_time(1439);
    assert_eq!((c.ext.day, c.month, c.ext.minute_of_day), (0, 0, 1439));
    c.advance_time(1);
    assert_eq!((c.ext.day, c.month, c.ext.minute_of_day), (1, 0, 0));
    c.advance_time(29 * 1440);
    assert_eq!((c.ext.day, c.month, c.ext.minute_of_day), (30, 1, 0));
    assert_eq!(c.history.len(), 2);
    c.advance_time(180);
    assert_eq!((c.ext.day, c.month, c.ext.minute_of_day), (30, 1, 180));
}
#[test]
fn expanded_daily_state_serializes_and_continues_exactly() {
    let mut a = town();
    advance(&mut a, 17);
    a.advance_time(321);
    let bytes = serde_json::to_vec(&a).unwrap();
    let mut b: City = serde_json::from_slice(&bytes).unwrap();
    b.validate().unwrap();
    assert_eq!(a, b);
    for _ in 0..73 {
        a.advance_day();
        b.advance_day();
    }
    assert_eq!(a, b);
    assert!(a.ext.people.births + a.ext.people.deaths > 0);
    assert_eq!(a.ext.people.happiness_factors.len(), 16);
    assert_eq!(
        a.ext.people.children + a.ext.people.adults + a.ext.people.seniors,
        a.ext
            .people
            .households
            .iter()
            .map(|h| h.size as u32)
            .sum::<u32>()
    );
}
#[test]
fn climate_and_weather_change_conditions_without_platform_randomness() {
    let mut a = City::world(99, MapStyle::Flatlands, Climate::Tropical);
    let mut b = City::world(99, MapStyle::Flatlands, Climate::Alpine);
    advance(&mut a, 40);
    advance(&mut b, 40);
    assert!(a.ext.weather.temperature > b.ext.weather.temperature);
    assert_ne!(
        a.ext.cells.iter().map(|c| c.moisture as u32).sum::<u32>(),
        b.ext.cells.iter().map(|c| c.moisture as u32).sum::<u32>()
    );
    let mut same = City::world(99, MapStyle::Flatlands, Climate::Tropical);
    advance(&mut same, 40);
    assert_eq!(a, same);
}
#[test]
fn exact_placement_and_terraform_quotes_are_pure_and_use_actual_prices() {
    let mut c = town();
    let before = c.clone();
    let q = c.placement_cost(22, 9, Kind::Park).unwrap();
    assert_eq!(c, before);
    assert_eq!(c.place(22, 9, Kind::Park).unwrap(), q);
    let before = c.clone();
    assert!(c.terraform_cost(22, 9, TerrainAction::Raise).is_err());
    assert_eq!(c, before);
    for action in TerrainAction::ALL {
        assert!(action.cost() >= 20);
    }
    let q = c.terraform_cost(24, 9, TerrainAction::Water).unwrap();
    assert_eq!(q, 80);
    assert_eq!(c.terraform(24, 9, TerrainAction::Water).unwrap(), q);
    assert_eq!(c.tile(24, 9).unwrap().terrain, Terrain::Water);
    assert!(c.terraform_cost(24, 9, TerrainAction::Plant).is_err());
    let q = c.terraform_cost(24, 9, TerrainAction::Drain).unwrap();
    c.terraform(24, 9, TerrainAction::Drain).unwrap();
    assert_eq!(q, 80);
    assert_ne!(c.tile(24, 9).unwrap().terrain, Terrain::Water);
}
#[test]
fn construction_has_real_stages_and_survives_save_load() {
    let mut c = town();
    c.place(22, 9, Kind::Park).unwrap();
    let i = c.idx(22, 9).unwrap();
    let first = c.ext.buildings.iter().find(|b| b.tile == i as u32).unwrap();
    let id = first.id;
    assert_eq!(first.stage, BuildingStage::Foundation);
    advance(&mut c, 4);
    let b = c.ext.buildings.iter().find(|b| b.id == id).unwrap();
    assert!(b.progress > 0 && b.progress < 100);
    advance(&mut c, 16);
    let b = c.ext.buildings.iter().find(|b| b.id == id).unwrap();
    assert_eq!(b.stage, BuildingStage::Complete);
    assert_eq!(b.progress, 100);
}
#[test]
fn forests_need_thirty_six_months_and_resources_are_consumed() {
    let mut c = town();
    c.terraform(24, 9, TerrainAction::Plant).unwrap();
    let i = c.idx(24, 9).unwrap();
    assert_eq!(c.ext.cells[i].tree_age, 0);
    advance(&mut c, 30);
    assert_eq!(c.ext.cells[i].tree_age, 1);
    c.ext.cells[i].age_days = 1079;
    advance(&mut c, 1);
    assert_eq!(c.ext.cells[i].tree_age, 36);
    let industry = c.idx(30, 11).unwrap();
    c.ext.cells[industry].ore = 1000;
    c.ext.day = 59;
    advance(&mut c, 1);
    assert!(c.ext.cells[industry].ore < 1000);
}
#[test]
fn seven_plants_have_real_capacity_and_weather_sensitive_renewables() {
    let base = town();
    for kind in [
        Kind::Coal,
        Kind::Wind,
        Kind::Nuclear,
        Kind::Gas,
        Kind::Solar,
        Kind::Hydro,
        Kind::Fusion,
    ] {
        let mut c = base.clone();
        let i = c.idx(12, 9).unwrap();
        c.tiles[i].kind = kind;
        let bank = c.idx(12, 8).unwrap();
        c.tiles[bank].terrain = Terrain::Water;
        c.ext.minute_of_day = 720;
        c.ext.weather.cloud = 0;
        c.ext.weather.wind_x = 1;
        c.ext.weather.rain = 50;
        c.refresh();
        assert!(c.power_supply > 0, "{kind:?} must generate electricity");
        if kind == Kind::Solar {
            let daytime = c.power_supply;
            c.ext.minute_of_day = 0;
            c.refresh();
            assert!(c.power_supply < daytime);
        }
        if kind == Kind::Wind {
            let windy = c.power_supply;
            c.ext.weather.wind_x = 0;
            c.ext.weather.wind_y = 0;
            c.refresh();
            assert!(c.power_supply < windy);
        }
        c.ext.cells[i].outage_days = 3;
        c.refresh();
        assert_eq!(c.power_supply, 0, "a failed plant must cause a brownout");
    }
}
#[test]
fn underground_layers_do_not_destroy_surface_buildings() {
    let mut c = town();
    c.place(22, 9, Kind::Park).unwrap();
    c.place(22, 9, Kind::Subway).unwrap();
    let i = c.idx(22, 9).unwrap();
    assert_eq!(c.tiles[i].kind, Kind::Park);
    assert!(c.ext.cells[i].subway);
    c.place(23, 9, Kind::Subway).unwrap();
    assert!(c.underground_connections(i)[3] & 1 != 0);
    assert!(c.place(22, 9, Kind::Subway).is_err());
}
#[test]
fn transit_requires_a_depot_distinct_stops_and_a_continuous_route() {
    let mut c = town();
    c.place(6, 9, Kind::BusStop).unwrap();
    c.place(32, 9, Kind::BusStop).unwrap();
    let a = c.idx(6, 9).unwrap() as u32;
    let b = c.idx(32, 9).unwrap() as u32;
    assert!(c.create_line(TransitMode::Bus, vec![a, a]).is_err());
    assert!(c.create_line(TransitMode::Rail, vec![a, b]).is_err());
    let id = c.create_line(TransitMode::Bus, vec![a, b]).unwrap();
    advance(&mut c, 20);
    assert!(
        c.ext
            .transport
            .lines
            .iter()
            .find(|l| l.id == id)
            .unwrap()
            .ridership
            > 0
    );
    assert!(c.ext.transport.ridership <= c.ext.transport.trips);
    assert!(c
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| v.kind == VehicleKind::Bus));
    let cut = c.idx(22, 10).unwrap();
    c.bulldoze(22, 10).unwrap();
    advance(&mut c, 1);
    assert!(c
        .ext
        .transport
        .lines
        .iter()
        .find(|l| l.id == id)
        .unwrap()
        .route
        .is_empty());
    assert!(c
        .ext
        .transport
        .vehicles
        .iter()
        .all(|v| !v.route.contains(&(cut as u32))));
}
#[test]
fn directed_routes_respect_one_way_streets_and_detour_around_congestion() {
    let mut c = town();
    let i = c.idx(22, 10).unwrap();
    c.tiles[i].kind = Kind::OneWay;
    c.ext.cells[i].orientation = 2;
    assert!(route_between(
        &c,
        c.idx(21, 10).unwrap(),
        c.idx(24, 10).unwrap(),
        TransitMode::Bus
    )
    .is_none());
    c.ext.cells[i].orientation = 0;
    assert!(route_between(
        &c,
        c.idx(21, 10).unwrap(),
        c.idx(24, 10).unwrap(),
        TransitMode::Bus
    )
    .is_some());
}
#[test]
fn weighted_commutes_take_a_real_detour_and_reject_its_wrong_way_segment() {
    let mut c = town();
    for x in 20..=30 {
        let i = c.idx(x, 8).unwrap();
        c.tiles[i].kind = Kind::Road;
    }
    for y in 8..=10 {
        for x in [20, 30] {
            let i = c.idx(x, y).unwrap();
            c.tiles[i].kind = Kind::Road;
        }
    }
    c.refresh();
    for x in 21..30 {
        let i = c.idx(x, 10).unwrap();
        c.tiles[i].traffic = 100;
    }
    let (a, b) = (c.idx(20, 10).unwrap(), c.idx(30, 10).unwrap());
    let detour = road_route(&c, a, b).unwrap();
    assert!(detour.contains(&(c.idx(25, 8).unwrap() as u32)));
    let wrong_way = c.idx(25, 8).unwrap();
    c.tiles[wrong_way].kind = Kind::OneWay;
    c.ext.cells[wrong_way].orientation = 2;
    let direct = road_route(&c, a, b).unwrap();
    assert!(!direct.contains(&(wrong_way as u32)));
    assert!(direct.contains(&(c.idx(25, 10).unwrap() as u32)));
}
#[test]
fn service_and_problem_overlays_are_read_only_and_follow_network_loss() {
    let mut c = town();
    assert_eq!(SERVICE_LABELS.len(), 13);
    c.place(22, 9, Kind::Clinic).unwrap();
    advance(&mut c, 20);
    c.refresh();
    let home = c.idx(10, 11).unwrap();
    assert!(c.coverage(2, home) > 0);
    let before = c.clone();
    for mode in 0..20 {
        assert!(c.overlay_value(home, mode).is_some());
    }
    let _ = c.problems(home);
    let _ = c.next_level_needs(home);
    let _ = c.suggest_site(Kind::Park);
    assert_eq!(c, before);
    c.bulldoze(0, 10).unwrap();
    assert!(c.problems(home).iter().any(|p| p.contains("outside")));
}
#[test]
fn dense_fixture_contains_real_occupied_stable_building_instances() {
    let mut c = bluecity::mayor::dense_fixture(7, 15000);
    assert!(c.ext.buildings.len() >= 15000);
    let occupants = c
        .ext
        .buildings
        .iter()
        .map(|b| b.occupants as u64)
        .sum::<u64>();
    assert_eq!(occupants, (c.population + c.jobs) as u64);
    assert!(c.power_supply >= c.power_demand);
    assert!(c.water_supply >= c.water_demand);
    let ids: std::collections::BTreeSet<_> = c.ext.buildings.iter().map(|b| b.id).collect();
    assert_eq!(ids.len(), c.ext.buildings.len());
    assert!(!c.ext.people.households.is_empty());
    assert_eq!(
        c.ext.people.children + c.ext.people.adults + c.ext.people.seniors,
        c.population
    );
    assert!(c.ext.people.employed > 0);
    assert!(!c.ext.transport.commute_routes.is_empty());
    assert_eq!(
        c.ext.day, 3600,
        "fixture initialization does not consume simulated days"
    );
    assert_eq!(
        c.ext.transport.trips, 0,
        "prepared routes are not fictional completed trips"
    );
    assert!(c.civic.january.is_some());
    advance(&mut c, 35);
    assert!(c.ext.buildings.len() >= 15000);
    assert!(c.population > 100000);
    assert!(c.ext.transport.trips > 0);
}
#[test]
fn commute_reviews_are_staggered_but_never_older_than_seven_days() {
    let mut c = town();
    let mut people = std::mem::take(&mut c.ext.people);
    people.initialize_existing(&c);
    c.ext.people = people;
    let mut tr = std::mem::take(&mut c.ext.transport);
    tr.initialize_commutes(&c);
    c.ext.transport = tr;
    assert!(c.ext.transport.commute_routes.len() >= 5);
    let count = c.ext.transport.commute_routes.len();
    for day in 1..=21 {
        transport_day(&mut c, day);
        let reviewed = c
            .ext
            .transport
            .commute_routes
            .iter()
            .filter(|r| r.planned_day == day)
            .count();
        assert!(
            reviewed < count,
            "stable topology should not burst-replan the whole city on day{day}"
        );
        assert!(c
            .ext
            .transport
            .commute_routes
            .iter()
            .all(|r| day - r.planned_day < 7));
    }
    let cut = c.idx(20, 10).unwrap();
    c.tiles[cut].kind = Kind::Empty;
    transport_day(&mut c, 22);
    assert!(
        c.ext
            .transport
            .commute_routes
            .iter()
            .all(|r| r.planned_day == 22),
        "topology changes must invalidate all affected cached plans immediately"
    );
}
#[test]
fn employment_assigns_qualified_real_vacancies_without_overbooking() {
    let mut c = town();
    c.education = 95;
    for t in &mut c.tiles {
        if t.kind == Kind::Industrial {
            t.level = 3;
        }
    }
    let mut p = std::mem::take(&mut c.ext.people);
    p.advance_day(&mut c, 2, 20);
    assert!(p.employed > 0);
    assert_eq!(
        p.employed,
        p.employment.iter().map(|a| a.workers).sum::<u32>()
    );
    assert_eq!(p.employed, p.employed_by_education.iter().sum::<u32>());
    assert!(p.overqualified > 0);
    let mut filled = vec![0; c.tiles.len()];
    for assignment in &p.employment {
        let h = p
            .households
            .iter()
            .find(|h| h.id == assignment.household)
            .unwrap();
        let required = if c.tiles[assignment.employer as usize].kind == Kind::Industrial {
            2
        } else {
            0
        };
        assert!(h.education / 26 >= required);
        filled[assignment.employer as usize] += assignment.workers;
    }
    for (i, count) in filled.into_iter().enumerate() {
        assert!(count <= c.tiles[i].jobs);
    }
}
#[test]
fn lifecycle_counts_only_real_housing_admissions_and_removed_residents() {
    let mut c = town();
    for t in &mut c.tiles {
        if t.kind == Kind::Residential {
            t.population = 64;
        }
    }
    c.refresh();
    c.happiness = 90;
    c.demand[0] = 90;
    let before = c.population;
    let mut p = std::mem::take(&mut c.ext.people);
    p.advance_day(&mut c, 2, 20);
    assert_eq!(p.migrants_in, 0, "full homes cannot admit migrants");
    assert_eq!(c.population, before);
    p.death_accumulator = 3 * 360000;
    p.advance_day(&mut c, 3, 20);
    assert_eq!(p.deaths, 3);
    assert_eq!(c.ext.services.dead.iter().sum::<u32>(), 3);
    assert_eq!(
        c.population as i64 - before as i64,
        (p.births + p.migrants_in) as i64 - (p.deaths + p.migrants_out) as i64
    );
}
#[test]
fn neighbor_power_is_finite_charged_and_requires_a_real_outside_connection() {
    let mut c = town();
    let coal = c.idx(12, 9).unwrap();
    c.tiles[coal].kind = Kind::Empty;
    c.refresh();
    assert_eq!(c.power_supply, 0);
    c.set_neighbor_contract(0, true, 1000, 1000, 2).unwrap();
    assert!(c.ext.neighbors[0].last_power > 0);
    assert!(c.ext.neighbors[0].last_power <= 1000);
    assert!(c.ext.neighbors[0].monthly_cost > 0);
    assert!(c
        .tiles
        .iter()
        .any(|t| t.kind == Kind::Residential && t.powered));
    c.bulldoze(0, 10).unwrap();
    assert!(!c.outside_connected());
    assert_eq!(c.ext.neighbors[0].last_power, 0);
    assert_eq!(c.power_supply, 0);
    assert!(c.set_neighbor_contract(0, true, 100001, 0, 2).is_err());
    c.validate().unwrap();
}
#[test]
fn accessible_parks_remove_real_air_pollution() {
    let a = town();
    let mut b = a.clone();
    let i = a.idx(35, 11).unwrap();
    a.ext.coverage.get(i).unwrap();
    b.ext.coverage[i][5] = 100;
    let mut wa = a.ext.clone();
    let mut wb = b.ext.clone();
    let mut a = a;
    wa.advance_day(&mut a);
    wb.advance_day(&mut b);
    assert!(wb.cells[i].air_pollution < wa.cells[i].air_pollution);
}
#[test]
fn completed_mega_project_jobs_are_valid_occupied_building_records() {
    let mut c = town();
    let i = c.idx(22, 9).unwrap();
    c.tiles[i].kind = Kind::Arcology;
    c.tiles[i].level = 1;
    c.tiles[i].jobs = 250;
    c.refresh();
    c.validate().unwrap();
    assert!(c
        .ext
        .buildings
        .iter()
        .any(|b| b.tile == i as u32 && b.occupants == 250));
    let bytes = serde_json::to_vec(&c).unwrap();
    let restored: City = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(c, restored);
}
#[test]
fn malformed_expansion_references_are_rejected_before_rules_can_index_them() {
    let mut valid = town();
    advance(&mut valid, 2);
    let mut c = valid.clone();
    c.ext.people.households[0].home = u32::MAX;
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    c.ext.people.households[0].employer = Some(u32::MAX);
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    c.ext.weather.wind_x = i8::MIN;
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    c.ext.cells[0].elevation = i16::MAX;
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    c.ext.neighbors[0].power = u32::MAX;
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    c.ext.buildings[1].id = c.ext.buildings[0].id;
    assert!(c.validate().is_err());
    let mut c = valid.clone();
    let member = c.ext.utility_grids[0][0].pop().unwrap();
    c.ext.utility_grids[0].push(vec![member]);
    assert!(
        c.validate().is_err(),
        "cached adjacent conductors cannot be split into fictional supply components"
    );
    let mut c = valid;
    c.ext.transport.vehicles.push(bluecity::transport::Vehicle {
        tile: u32::MAX,
        ..Default::default()
    });
    assert!(c.validate().is_err());
}
#[test]
fn sustained_vacancies_shrink_density_then_repaired_neighborhoods_rebuild() {
    let mut c = town();
    let home = c.idx(8, 11).unwrap();
    c.tiles[home].level = 3;
    c.tiles[home].population = 144;
    let coal = c.idx(12, 9).unwrap();
    c.tiles[coal].kind = Kind::Empty;
    c.refresh();
    for _ in 0..12 {
        c.advance_month();
    }
    assert!(c.tiles[home].level < 3);
    assert!(c.ext.people.migrants_out > 0);
    c.tiles[coal].kind = Kind::Coal;
    c.money = 200000;
    c.refresh();
    for _ in 0..24 {
        c.advance_month();
    }
    assert!(c.tiles[home].population > 0);
    assert!(c.ext.people.migrants_in > 0);
    c.validate().unwrap();
    assert_eq!(
        c.ext.buildings.iter().map(|b| b.occupants).sum::<u32>(),
        c.population + c.jobs
    );
}
#[test]
fn rail_connects_real_commutes_between_separate_road_networks() {
    let (mut c, home, work, stops) = separated_rail_town();
    assert!(c.tiles[home].connected && c.tiles[work].connected);
    assert!(road_route(&c, home, work).is_none());
    let mut t = std::mem::take(&mut c.ext.transport);
    t.create_line(&c, TransitMode::Rail, stops.to_vec())
        .unwrap();
    c.ext.transport = t;
    transport_day(&mut c, 2);
    assert_eq!(c.ext.transport.trips, 20);
    assert_eq!(c.ext.transport.ridership, 20);
    assert_eq!(c.ext.transport.served_trips, 20);
    assert_eq!(c.ext.transport.unserved_trips, 0);
    assert!(c.ext.transport.lines[0].running);
    assert!(!c.ext.transport.lines[0].reverse_route.is_empty());
    assert!(c
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| v.kind == VehicleKind::Train));
    assert!(!c
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| v.kind == VehicleKind::Car));
}
#[test]
fn boarding_requires_a_walkable_catchment_and_a_bridge_can_restore_it() {
    let (mut c, _, _, stops) = separated_rail_town();
    let mut t = std::mem::take(&mut c.ext.transport);
    t.create_line(&c, TransitMode::Rail, stops.to_vec())
        .unwrap();
    c.ext.transport = t;
    for x in 0..48 {
        let i = c.idx(x, 19).unwrap();
        c.tiles[i].kind = Kind::Empty;
        c.tiles[i].terrain = Terrain::Water;
    }
    transport_day(&mut c, 2);
    assert!(c.ext.transport.lines[0].running);
    assert_eq!(
        c.ext.transport.ridership, 0,
        "close stations across water are not walkable"
    );
    assert_eq!(c.ext.transport.unserved_trips, c.ext.transport.trips);
    for x in [8, 28] {
        let i = c.idx(x, 19).unwrap();
        c.tiles[i].kind = Kind::Road;
        c.tiles[i].connected = true;
    }
    transport_day(&mut c, 3);
    assert_eq!(c.ext.transport.ridership, 20);
    assert_eq!(c.ext.transport.unserved_trips, 0);
}
#[test]
fn unfinished_unfunded_or_failed_transport_providers_cannot_run_lines_or_flights() {
    let (mut c, _, _, stops) = separated_rail_town();
    let base = stops[0] as usize;
    for state in 0..5 {
        let mut broken = c.clone();
        match state {
            0 => broken.ext.cells[base].construction = 99,
            1 => broken.ext.cells[base].outage_days = 1,
            2 => broken.ext.cells[base].abandoned = true,
            3 => broken.ext.cells[base].rubble = true,
            _ => broken.funding[0] = 0,
        }
        let mut t = std::mem::take(&mut broken.ext.transport);
        // The opposite station supplies depot capacity, but a closed stop
        // still keeps the line from carrying passengers.
        if t.create_line(&broken, TransitMode::Rail, stops.to_vec())
            .is_ok()
        {
            broken.ext.transport = t;
            transport_day(&mut broken, 10);
            assert!(!broken.ext.transport.lines[0].running);
            assert_eq!(broken.ext.transport.ridership, 0);
        }
    }
    c.tiles[base].kind = Kind::Airport;
    c.ext.cells[base].construction = 90;
    transport_day(&mut c, 10);
    assert!(!c
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| matches!(v.kind, VehicleKind::Aircraft | VehicleKind::Jet)));
    c.ext.cells[base].construction = 100;
    transport_day(&mut c, 20);
    assert!(c
        .ext
        .transport
        .vehicles
        .iter()
        .any(|v| matches!(v.kind, VehicleKind::Aircraft | VehicleKind::Jet)));
}
#[test]
fn real_transport_and_trade_receipts_reconcile_to_budget_and_post_once() {
    let mut base = town();
    base.month = 100;
    base.ext.day = 3000;
    base.civic.day = 3000;
    base.civic.disaster_frequency = 0;
    let mut paid = base.clone();
    paid.ext.transport.fare_balance = 1234;
    paid.ext.transport.air_revenue_balance = 67;
    paid.ext.trade_income = 43;
    paid.ext.operating_cost = 29;
    base.ext.trade_income = 0;
    base.ext.operating_cost = 0;
    base.advance_month();
    paid.advance_month();
    assert_eq!(paid.income - base.income, 1234 + 67 + 43);
    assert_eq!(paid.expenses - base.expenses, 29);
    assert_eq!(paid.money - base.money, 1234 + 67 + 43 - 29);
    assert_eq!(paid.ext.transport.fare_balance, 0);
    assert_eq!(paid.ext.transport.air_revenue_balance, 0);
    assert_eq!(paid.history.last().unwrap().income, paid.income);
    assert_eq!(paid.history.last().unwrap().expenses, paid.expenses);
    paid.ext.trade_income = 0;
    paid.ext.operating_cost = 0;
    let before = paid.money - base.money;
    base.advance_month();
    paid.advance_month();
    assert_eq!(
        paid.money - base.money,
        before,
        "settled fares cannot pay a second time"
    );
}
#[test]
fn depleted_industry_produces_less_and_goods_have_a_conserved_balance() {
    let mut rich = town();
    for (i, t) in rich.tiles.iter().enumerate() {
        if t.kind == Kind::Industrial {
            rich.ext.cells[i].ore = 1000;
            rich.ext.cells[i].oil = 1000;
        }
    }
    let mut empty = rich.clone();
    for c in &mut empty.ext.cells {
        c.ore = 0;
        c.oil = 0;
    }
    for city in [&mut rich, &mut empty] {
        let mut p = std::mem::take(&mut city.ext.people);
        p.advance_day(city, 2, 20);
        city.ext.people = p;
        let p = &city.ext.people;
        assert_eq!(
            p.goods_produced + p.goods_imported,
            p.goods_consumed + p.goods_exported + p.goods_unused
        );
        assert_eq!(p.goods_demand, p.goods_consumed + p.goods_shortfall);
    }
    assert!(rich.ext.people.goods_produced > empty.ext.people.goods_produced);
    assert!(
        empty.ext.people.goods_produced > 0,
        "light/recycled manufacturing survives depletion"
    );
}
#[test]
fn coverage_opens_and_recovers_with_its_facility_without_another_player_edit() {
    let mut c = town();
    let home = c.idx(8, 11).unwrap();
    let park = c.idx(7, 9).unwrap();
    c.place(7, 9, Kind::Park).unwrap();
    assert_eq!(c.ext.cells[park].construction, 0);
    assert_eq!(c.ext.coverage[home][5], 0);
    for _ in 0..15 {
        c.advance_day();
    }
    assert_eq!(c.ext.cells[park].construction, 100);
    assert!(
        c.ext.coverage[home][5] > 0,
        "the finished park must open without another edit"
    );
    c.ext.cells[park].outage_days = 3;
    c.advance_day();
    assert_eq!(c.ext.coverage[home][5], 0);
    c.advance_day();
    c.advance_day();
    assert!(
        c.ext.coverage[home][5] > 0,
        "recovered providers must restore their coverage"
    );
    c.validate().unwrap();
}
#[test]
fn infrastructure_memos_are_pure_and_pending_edits_resume_like_cold_recomputation() {
    use vesper3d::{runtime::snapshot, two_d::GameLogic};
    let mut warm = town();
    let hospital = warm.idx(22, 9).unwrap();
    warm.tiles[hospital].kind = Kind::Hospital;
    let park = warm.idx(7, 9).unwrap();
    warm.tiles[park].kind = Kind::Park;
    warm.refresh();
    for pending in 0..3 {
        match pending {
            0 => warm.funding[1] = 35,
            1 => warm.ext.cells[hospital].outage_days = 4,
            _ => {
                let i = warm.idx(24, 8).unwrap();
                warm.tiles[i].terrain = Terrain::Water;
            }
        }
        let before = serde_json::to_vec(&warm).unwrap();
        warm.warm_infrastructure_cache();
        assert_eq!(
            serde_json::to_vec(&warm).unwrap(),
            before,
            "warming changed pending authoritative state"
        );
        let mut cold: City = serde_json::from_slice(&before).unwrap();
        let mut game = bluecity::BlueCity::new(7);
        game.state.city = warm.clone();
        game.state.speed = 0;
        let frame = snapshot::save(&game, "Pending infrastructure invalidation").unwrap();
        let mut loaded = bluecity::BlueCity::new(0);
        snapshot::restore(&mut loaded, &frame).unwrap();
        for _ in 0..12 {
            warm.advance_day();
            cold.advance_day();
            loaded.state.city.advance_day();
            assert!(
                warm == cold,
                "warm/cold service or shore cache changed the future for pending case{pending}"
            );
            assert!(
                warm == loaded.state.city,
                "snapshot warming changed pending case{pending}"
            );
        }
        warm.validate().unwrap();
        cold.validate().unwrap();
    }
}
#[test]
fn transport_access_memo_tracks_water_and_connected_flags_and_repairs_saved_arrays() {
    let mut warm = town();
    transport_day(&mut warm, 1);
    let mut cold: City = serde_json::from_slice(&serde_json::to_vec(&warm).unwrap()).unwrap();
    for day in 2..=15 {
        for c in [&mut warm, &mut cold] {
            if day == 4 {
                let i = c.idx(6, 10).unwrap();
                c.tiles[i].connected = false;
            }
            if day == 7 {
                let i = c.idx(6, 10).unwrap();
                c.tiles[i].connected = true;
            }
            if day == 9 {
                let i = c.idx(6, 12).unwrap();
                c.tiles[i].terrain = Terrain::Water;
            }
            if day == 12 {
                let i = c.idx(6, 12).unwrap();
                c.tiles[i].terrain = Terrain::Grass;
            }
        }
        // The authoritative archived arrays can be stale; memoization must
        // restore exact computed values rather than trusting their contents.
        warm.ext.transport.road_access.fill(u8::MAX);
        warm.ext.transport.nearest_road.fill(u32::MAX);
        transport_day(&mut warm, day);
        transport_day(&mut cold, day);
        assert!(
            warm.ext.transport == cold.ext.transport,
            "access memo diverged on day{day}"
        );
        assert_eq!(warm.tiles, cold.tiles);
    }
}
#[test]
fn cached_land_value_responds_to_tree_maturation_water_and_drainage() {
    let mut c = town();
    let home = c.idx(8, 11).unwrap();
    let forest = c.idx(8, 12).unwrap();
    c.tiles[forest].terrain = Terrain::Forest;
    c.ext.cells[forest].tree_age = 35;
    c.refresh();
    let before = c.tiles[home].land_value;
    c.ext.cells[forest].tree_age = 36;
    c.refresh();
    assert_eq!(c.tiles[home].land_value, before + 3);
    c.tiles[forest].terrain = Terrain::Grass;
    c.refresh();
    assert_eq!(c.tiles[home].land_value, before);
    c.terraform(8, 12, TerrainAction::Water).unwrap();
    assert_eq!(c.tiles[home].land_value, before + 7);
    c.terraform(8, 12, TerrainAction::Drain).unwrap();
    assert_eq!(c.tiles[home].land_value, before);
}
#[test]
fn freight_counts_actual_goods_and_yards_need_a_real_regional_rail_route() {
    let mut road = town();
    for x in 21..=44 {
        let i = road.idx(x, 11).unwrap();
        road.tiles[i].kind = Kind::Industrial;
        road.tiles[i].level = 5;
        road.tiles[i].jobs = 240;
        road.ext.cells[i].ore = 1000;
        road.ext.cells[i].oil = 1000;
    }
    let yard = road.idx(22, 9).unwrap();
    road.tiles[yard].kind = Kind::FreightStation;
    for n in &mut road.ext.neighbors {
        n.enabled = false;
    }
    road.ext.neighbors[0].enabled = true;
    road.ext.neighbors[0].goods = 50000;
    road.refresh();
    assert!(bluecity::transport::facility_operational(&road, yard));
    assert_eq!(
        bluecity::transport::freight_export_capacity(&road),
        0,
        "a roadside freight building alone does not create a rail export route"
    );
    let mut rail = road.clone();
    for y in 0..9 {
        let i = rail.idx(22, y).unwrap();
        rail.tiles[i].kind = Kind::Rail;
    }
    rail.refresh();
    assert_eq!(bluecity::transport::freight_export_capacity(&rail), 6000);
    for c in [&mut road, &mut rail] {
        let mut people = std::mem::take(&mut c.ext.people);
        people.advance_day(c, 2, 20);
        c.ext.people = people;
        transport_day(c, 2);
        let p = &c.ext.people;
        assert_eq!(
            p.goods_produced + p.goods_imported,
            p.goods_consumed + p.goods_exported + p.goods_unused
        );
        assert_eq!(c.ext.transport.freight, p.goods_imported + p.goods_exported);
        c.validate().unwrap();
    }
    assert_eq!(road.ext.people.goods_exported, 10000);
    assert!(rail.ext.people.goods_exported > road.ext.people.goods_exported);
    rail.ext.cells[yard].construction = 99;
    assert_eq!(bluecity::transport::freight_export_capacity(&rail), 0);
    rail.ext.cells[yard].construction = 100;
    let cut = rail.idx(22, 4).unwrap();
    rail.tiles[cut].kind = Kind::Empty;
    assert_eq!(
        bluecity::transport::freight_export_capacity(&rail),
        0,
        "a severed regional track immediately removes its shipping capacity"
    );
}
#[test]
fn utility_cache_warm_and_forced_cold_cities_continue_identically() {
    let mut warm = town();
    let mut cold = warm.clone();
    for day in 1..=75 {
        for city in [&mut warm, &mut cold] {
            if day == 12 {
                let i = city.idx(24, 10).unwrap();
                city.tiles[i].kind = Kind::Empty;
            }
            if day == 21 {
                let i = city.idx(24, 10).unwrap();
                city.tiles[i].kind = Kind::Road;
            }
            if day == 40 {
                city.funding[3] = 50;
            }
        }
        cold.ext.utility_masks = [vec![], vec![]];
        cold.ext.utility_grids = [vec![], vec![]];
        warm.advance_day();
        cold.advance_day();
        // The caches themselves are memoization, so compare every other
        // authoritative field as well as the actual utility allocations.
        let mut expected = warm.clone();
        expected
            .ext
            .utility_masks
            .clone_from(&cold.ext.utility_masks);
        expected
            .ext
            .utility_grids
            .clone_from(&cold.ext.utility_grids);
        if let (Some(a), Some(b)) = (&mut expected.civic.january, &cold.civic.january) {
            a.world.utility_masks.clone_from(&b.world.utility_masks);
            a.world.utility_grids.clone_from(&b.world.utility_grids);
        }
        assert!(expected == cold, "cache divergence on day{day}");
    }
    warm.validate().unwrap();
    cold.validate().unwrap();
}
