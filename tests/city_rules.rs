use bluecity::city::{City, Disaster, Kind, MapStyle, Terrain, SIZE};
use std::collections::VecDeque;

fn plain() -> City {
    let mut city = City::new(71, false);
    for tile in &mut city.tiles {
        tile.terrain = Terrain::Grass;
    }
    city.money = 200_000;
    city.refresh();
    city
}

fn put(city: &mut City, x: i32, y: i32, kind: Kind) {
    city.place(x, y, kind).unwrap();
}

fn little_town(outside: bool) -> City {
    let mut city = plain();
    city.tiles[City::index(13, 8).unwrap()].terrain = Terrain::Water;
    for x in (if outside { 0 } else { 10 })..=24 {
        put(&mut city, x, 10, Kind::Road);
    }
    put(&mut city, 11, 9, Kind::Wind);
    put(&mut city, 13, 9, Kind::Pump);
    put(&mut city, 15, 9, Kind::Sewage);
    put(&mut city, 17, 9, Kind::Landfill);
    for x in 11..=15 {
        put(&mut city, x, 11, Kind::Residential);
    }
    for x in 17..=20 {
        put(&mut city, x, 11, Kind::Industrial);
    }
    city
}

#[test]
fn starter_grows_and_remains_a_viable_city_for_ten_years() {
    let mut city = City::new(7, true);
    let initial = city.population;
    assert!(initial > 1000);
    assert!(city
        .tiles
        .iter()
        .filter(|t| t.population > 0)
        .all(|t| t.connected && t.powered && t.watered));
    assert_eq!(city.garbage, 0);
    assert!(
        city.income > city.expenses,
        "starter budget {} / {}",
        city.income,
        city.expenses
    );
    for month in 1..=120 {
        city.advance_month();
        city.validate().unwrap();
        if [6, 24, 120].contains(&month) {
            eprintln!("month {month}: pop={} jobs={} happy={} money={} net={} power={}/{} water={}/{} education={}",
                city.population, city.jobs, city.happiness, city.money, city.income-city.expenses,
                city.power_demand, city.power_supply, city.water_demand, city.water_supply, city.education);
        }
        if month == 6 {
            assert!(city.population > initial);
        }
    }
    assert!(city.population > initial);
    assert!(city.money > 0);
    assert_eq!(city.history.len(), 120);
}

#[test]
fn same_seed_replays_and_saved_future_is_exact() {
    let mut uninterrupted = City::new(983, true);
    let mut same = City::new(983, true);
    assert_eq!(uninterrupted, same);
    for month in 0..48 {
        if month == 3 {
            uninterrupted.policies[2] = true;
            same.policies[2] = true;
        }
        if month == 11 {
            uninterrupted.disaster(Disaster::Earthquake);
            same.disaster(Disaster::Earthquake);
        }
        uninterrupted.advance_month();
        same.advance_month();
    }
    assert_eq!(uninterrupted, same);
    let bytes = serde_json::to_vec(&uninterrupted).unwrap();
    let mut restored: City = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    for _ in 0..24 {
        uninterrupted.advance_month();
        restored.advance_month();
    }
    assert_eq!(uninterrupted, restored);
    assert_ne!(City::new(983, true).tiles, City::new(984, true).tiles);
}

#[test]
fn a_road_must_reach_the_outside_before_a_town_can_grow() {
    let mut city = little_town(false);
    for _ in 0..4 {
        city.advance_month();
    }
    assert_eq!(city.population, 0);
    assert!(!city.tile(12, 11).unwrap().connected);
    assert_eq!(city.power_supply, 0);
    for x in 0..10 {
        put(&mut city, x, 10, Kind::Road);
    }
    for _ in 0..6 {
        city.advance_month();
    }
    assert!(city.population > 0);
    let home = city.tile(12, 11).unwrap();
    assert!(home.connected && home.powered && home.watered);
}

#[test]
fn disconnected_utility_components_do_not_share_their_supply() {
    let mut city = little_town(true);
    // A second outside road and neighborhood, with no physical conduit to the plant.
    for x in 30..SIZE as i32 {
        put(&mut city, x, 32, Kind::Road);
    }
    put(&mut city, 35, 31, Kind::Residential);
    city.advance_month();
    let isolated = city.tile(35, 31).unwrap();
    assert!(isolated.connected);
    assert!(!isolated.powered && !isolated.watered);
    assert_eq!(isolated.population, 0);
    assert!(city.tile(12, 11).unwrap().powered);
}

#[test]
fn utility_shortages_are_finite_and_are_repaired_by_capacity() {
    let mut city = little_town(true);
    for x in 0..=24 {
        let i = City::index(x, 12).unwrap();
        city.tiles[i].kind = Kind::Residential;
        city.tiles[i].level = 5;
        city.tiles[i].population = 480;
    }
    city.refresh();
    assert!(city.power_demand > city.power_supply);
    assert!(city.tiles.iter().any(|t| t.population > 0 && !t.powered));
    put(&mut city, 20, 9, Kind::Nuclear);
    assert!(city.power_supply > city.power_demand);
    assert!(city
        .tiles
        .iter()
        .filter(|t| t.population > 0)
        .all(|t| t.powered));
}

#[test]
fn purchases_validate_land_and_funds_before_charging() {
    let mut city = plain();
    let original = city.money;
    city.tiles[City::index(4, 4).unwrap()].terrain = Terrain::Water;
    assert!(city.place(4, 4, Kind::Coal).is_err());
    assert!(city.place(30, 30, Kind::Pump).is_err());
    assert!(city.place(-1, 0, Kind::Road).is_err());
    assert_eq!(city.money, original);
    assert_eq!(city.place(4, 4, Kind::Road).unwrap(), Kind::Road.cost() * 3);
    let before = city.money;
    assert!(city.place(4, 4, Kind::Road).is_err());
    assert_eq!(city.money, before);
    assert!(city.bulldoze(4, 4).is_ok());
    assert_eq!(city.tile(4, 4).unwrap().terrain, Terrain::Water);
    city.money = 0;
    assert!(city.place(30, 30, Kind::Residential).is_err());
    assert_eq!(city.tile(30, 30).unwrap().kind, Kind::Empty);
}

#[test]
fn bonds_have_a_limit_interest_and_actual_principal_repayment() {
    let mut city = City::new(7, true);
    let initial_money = city.money;
    let initial_expenses = city.expenses;
    city.take_loan().unwrap();
    city.take_loan().unwrap();
    city.take_loan().unwrap();
    assert!(city.take_loan().is_err());
    assert_eq!(city.money, initial_money + 30000);
    assert_eq!(city.expenses, initial_expenses + 300);
    city.repay_loan().unwrap();
    assert_eq!(city.debt, 20000);
    assert_eq!(city.money, initial_money + 20000);
    city.money = 0;
    assert!(city.repay_loan().is_err());
}

#[test]
fn high_taxes_reduce_happiness_demand_and_growth() {
    let mut normal = City::new(7, true);
    let mut taxed = normal.clone();
    taxed.taxes = [20; 3];
    taxed.refresh();
    assert!(taxed.happiness < normal.happiness);
    assert!(taxed.demand[0] < normal.demand[0]);
    for _ in 0..12 {
        normal.advance_month();
        taxed.advance_month();
    }
    assert!(normal.population > taxed.population);
}

#[test]
fn landfill_and_treatment_protect_public_health() {
    let mut city = City::new(7, true);
    let clean_health = city.health;
    assert_eq!(city.garbage, 0);
    let works: Vec<_> = city
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.kind, Kind::Landfill | Kind::Sewage))
        .map(|(i, _)| i)
        .collect();
    for i in works {
        city.bulldoze((i % SIZE) as i32, (i / SIZE) as i32).unwrap();
    }
    assert_eq!(city.garbage, 100);
    assert!(city.health < clean_health);
}

#[test]
fn functioning_transit_and_clean_air_reduce_city_pressure() {
    let city = City::new(7, true);
    let mut green = city.clone();
    green.policies[0] = true;
    green.policies[1] = true;
    green.refresh();
    let traffic = |c: &City| c.tiles.iter().map(|t| t.traffic as u32).sum::<u32>();
    let pollution = |c: &City| c.tiles.iter().map(|t| t.pollution as u32).sum::<u32>();
    assert!(green.transport_usage > 0);
    assert!(traffic(&green) < traffic(&city));
    assert!(pollution(&green) < pollution(&city));
    assert!(green.expenses > city.expenses);
}

#[test]
fn industry_hurts_nearby_land_and_education_has_coverage() {
    let city = City::new(7, true);
    assert!(city.tile(28, 22).unwrap().pollution > city.tile(16, 16).unwrap().pollution);
    assert!(city.education > 20);
    let mut uneducated = city.clone();
    uneducated.funding[2] = 0;
    uneducated.refresh();
    assert_eq!(uneducated.education, 0);
    assert!(uneducated.expenses < city.expenses);
}

#[test]
fn freight_airport_and_landmarks_provide_real_revenue_and_demand() {
    let mut city = City::new(7, true);
    let before = city.income;
    let demand = city.demand;
    put(&mut city, 8, 26, Kind::Seaport);
    put(&mut city, 28, 18, Kind::Airport);
    assert!(city.tile(8, 26).unwrap().powered && city.tile(8, 26).unwrap().watered);
    assert!(city.tile(28, 18).unwrap().powered && city.tile(28, 18).unwrap().watered);
    assert!(city.income > before);
    assert!(city.demand[1] > demand[1]);
    assert!(city.demand[2] > demand[2]);
}

#[test]
fn funded_fire_crews_save_homes_and_absent_crews_lose_them() {
    let mut safe = City::new(7, true);
    let mut unsafe_city = safe.clone();
    let i = City::index(21, 17).unwrap();
    safe.tiles[i].fire = 65;
    unsafe_city.tiles[i].fire = 65;
    unsafe_city.funding[1] = 0;
    for _ in 0..4 {
        safe.advance_month();
        unsafe_city.advance_month();
    }
    assert_eq!(safe.tiles[i].kind, Kind::Residential);
    assert_eq!(safe.tiles[i].fire, 0);
    assert_eq!(unsafe_city.tiles[i].kind, Kind::Empty);
}

#[test]
fn disasters_are_reproducible_and_saves_remain_valid() {
    for disaster in [
        Disaster::Fire,
        Disaster::Flood,
        Disaster::Earthquake,
        Disaster::Tornado,
        Disaster::Meltdown,
    ] {
        let mut a = City::new(91, true);
        put(&mut a, 29, 18, Kind::Nuclear);
        let mut b = a.clone();
        a.disaster(disaster);
        b.disaster(disaster);
        assert_eq!(a, b);
        a.validate().unwrap();
        for _ in 0..6 {
            a.advance_month();
            a.validate().unwrap();
        }
    }
}

#[test]
fn malformed_save_statistics_are_rejected_without_panics() {
    let original = City::new(7, true);
    let mut c = original.clone();
    c.tiles.pop();
    assert!(c.validate().is_err());
    let mut c = original.clone();
    c.money = i64::MIN;
    assert!(c.validate().is_err());
    let mut c = original.clone();
    c.taxes[0] = 255;
    assert!(c.validate().is_err());
    let mut c = original.clone();
    c.population += 1;
    assert!(c.validate().is_err());
    let mut c = original.clone();
    c.tiles[0].fire = 255;
    assert!(c.validate().is_err());
    let mut c = original;
    c.tiles[0].population = 1;
    assert!(c.validate().is_err());
}

#[test]
fn painting_a_batch_matches_individually_validated_edits() {
    let mut batch = plain();
    let mut single = batch.clone();
    let edits: Vec<_> = (0..30)
        .map(|x| (x, 20, Kind::Road))
        .chain([
            (4, 20, Kind::Avenue),
            (5, 20, Kind::Empty),
            (-1, 20, Kind::Road),
            (2, 20, Kind::Residential),
        ])
        .collect();
    let expected: Vec<_> = edits
        .iter()
        .map(|&(x, y, k)| single.place(x, y, k))
        .collect();
    assert_eq!(batch.edit_batch(&edits), expected);
    assert_eq!(batch, single);
}

#[test]
fn a_railway_station_needs_a_real_route_and_can_connect_the_outside() {
    let mut city = little_town(false);
    put(&mut city, 10, 9, Kind::Station);
    assert_eq!(city.transport_usage, 0);
    let tracks: Vec<_> = (0..9).map(|y| (10, y, Kind::Rail)).collect();
    assert!(city.edit_batch(&tracks).iter().all(Result::is_ok));
    for _ in 0..5 {
        city.advance_month();
    }
    assert!(city.population > 0);
    assert!(city.transport_usage > 0);
    assert!(city.tile(12, 11).unwrap().connected);
    city.bulldoze(10, 4).unwrap();
    assert!(!city.tile(12, 11).unwrap().connected);
    assert_eq!(city.transport_usage, 0);
}

#[test]
fn rail_crossings_preserve_both_street_and_track_connections() {
    let mut city = little_town(true);
    let tracks: Vec<_> = (0..16).map(|y| (16, y, Kind::Rail)).collect();
    assert!(city.edit_batch(&tracks).iter().all(Result::is_ok));
    let crossing = city.tile(16, 10).unwrap();
    assert_eq!(crossing.kind, Kind::Road);
    assert!(crossing.rail_crossing);
    assert!(city.tile(23, 10).unwrap().connected);
    assert!(city.place(16, 10, Kind::Rail).is_err());
    put(&mut city, 16, 16, Kind::Station);
    let branch: Vec<_> = (12..=20).map(|x| (x, 17, Kind::Road)).collect();
    assert!(city.edit_batch(&branch).iter().all(Result::is_ok));
    put(&mut city, 15, 18, Kind::Residential);
    for _ in 0..4 {
        city.advance_month();
    }
    let home = city.tile(15, 18).unwrap();
    assert!(home.connected && home.powered && home.watered && home.population > 0);
    city.place(16, 10, Kind::Avenue).unwrap();
    assert!(city.tile(16, 10).unwrap().rail_crossing);
    city.validate().unwrap();
    city.bulldoze(16, 10).unwrap();
    assert!(!city.tile(16, 10).unwrap().rail_crossing);
}

fn major_terrain(terrain: Terrain) -> u8 {
    match terrain {
        Terrain::Grass | Terrain::Forest => 0,
        Terrain::Water => 1,
        Terrain::Hill => 2,
    }
}

fn major_terrain_difference(a: &City, b: &City) -> usize {
    a.tiles
        .iter()
        .zip(&b.tiles)
        .filter(|(a, b)| major_terrain(a.terrain) != major_terrain(b.terrain))
        .count()
}

fn largest_landmass(city: &City) -> usize {
    let mut visited = vec![false; SIZE * SIZE];
    let mut largest = 0;
    for first in 0..visited.len() {
        if visited[first] || city.tiles[first].terrain == Terrain::Water {
            continue;
        }
        let mut queue = VecDeque::from([first]);
        visited[first] = true;
        let mut size = 0;
        while let Some(i) = queue.pop_front() {
            size += 1;
            let (x, y) = ((i % SIZE) as i32, (i / SIZE) as i32);
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if let Some(j) = City::index(nx, ny) {
                    if !visited[j] && city.tiles[j].terrain != Terrain::Water {
                        visited[j] = true;
                        queue.push_back(j);
                    }
                }
            }
        }
        largest = largest.max(size);
    }
    largest
}

#[test]
fn generated_regions_reproduce_the_entire_city_and_serialize_their_styles() {
    assert_eq!(MapStyle::default(), MapStyle::River);
    assert_eq!(
        MapStyle::ALL.map(MapStyle::label),
        [
            "River",
            "Coast",
            "Islands",
            "Highlands",
            "Lakes",
            "Delta",
            "Flatlands"
        ]
    );
    for style in MapStyle::ALL {
        let encoded = serde_json::to_string(&style).unwrap();
        assert_eq!(serde_json::from_str::<MapStyle>(&encoded).unwrap(), style);
        for seed in [0, 7, 1984, 1 << 32, 1 << 63, u64::MAX] {
            let city = City::generated(seed, style);
            assert_eq!(city, City::generated(seed, style));
            assert_eq!(city.seed, seed, "generation must not advance disaster RNG");
            assert_eq!(city.money, 60000);
            assert_eq!((city.month, city.population, city.jobs), (0, 0, 0));
            assert_eq!(city.history.len(), 1);
            assert_eq!(city.history[0].money, city.money);
            assert_eq!(city.history[0].population, city.population);
            assert_eq!(city.history[0].income, city.income);
            assert_eq!(city.history[0].expenses, city.expenses);
            assert!(city.tiles.iter().all(|t| t.kind == Kind::Empty));
            city.validate().unwrap();
            let encoded = serde_json::to_vec(&city).unwrap();
            let restored: City = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(restored, city);
            let mut refreshed = restored;
            refreshed.refresh();
            assert_eq!(refreshed, city);
        }
    }
}

#[test]
fn generation_seeds_change_water_and_topography_rather_than_only_tree_scatter() {
    for style in MapStyle::ALL {
        for (a, b) in [(7, 8), (101, 9182), (0, u64::MAX), (7, 7 + (1 << 32))] {
            let a = City::generated(a, style);
            let b = City::generated(b, style);
            assert!(
                major_terrain_difference(&a, &b) > SIZE * SIZE / 12,
                "{style:?} seeds should materially change water or hills"
            );
        }
    }
}

#[test]
fn map_styles_provide_distinct_region_shapes_for_the_same_seed() {
    for seed in [7, 1984, u64::MAX] {
        let cities = MapStyle::ALL.map(|style| City::generated(seed, style));
        for a in 0..cities.len() {
            for b in (a + 1)..cities.len() {
                assert!(
                    major_terrain_difference(&cities[a], &cities[b]) > SIZE * SIZE / 12,
                    "{:?} and {:?} need distinct geography for seed {seed}",
                    MapStyle::ALL[a],
                    MapStyle::ALL[b]
                );
            }
        }
    }
}

#[test]
fn generated_regions_leave_room_for_towns_regional_access_and_water_pumps() {
    for style in MapStyle::ALL {
        for seed in [0, 1, 7, 1984, 1 << 63, u64::MAX] {
            let mut city = City::generated(seed, style);
            let count = |terrain| city.tiles.iter().filter(|t| t.terrain == terrain).count();
            assert!(count(Terrain::Water) >= 40, "{style:?} seed {seed}: water");
            assert!(count(Terrain::Grass) >= 192, "{style:?} seed {seed}: grass");
            assert!(
                largest_landmass(&city) >= 288,
                "{style:?} seed {seed}: landmass"
            );
            for y in 21..=27 {
                for x in 21..=27 {
                    assert_eq!(city.tile(x, y).unwrap().terrain, Terrain::Grass);
                }
            }
            let edge_land: Vec<_> = city
                .tiles
                .iter()
                .enumerate()
                .filter(|(i, t)| {
                    t.terrain != Terrain::Water
                        && (i % SIZE == 0
                            || i % SIZE == SIZE - 1
                            || i / SIZE == 0
                            || i / SIZE == SIZE - 1)
                })
                .map(|(i, _)| i)
                .collect();
            assert!(edge_land.len() >= 7, "{style:?} seed {seed}: edge access");
            let i = edge_land[0];
            city.place((i % SIZE) as i32, (i / SIZE) as i32, Kind::Road)
                .unwrap();
            assert!(city.tiles[i].connected);
            let shore = city
                .tiles
                .iter()
                .enumerate()
                .find(|(i, t)| {
                    let (x, y) = ((i % SIZE) as i32, (i / SIZE) as i32);
                    t.terrain != Terrain::Water
                        && t.kind == Kind::Empty
                        && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                            .into_iter()
                            .any(|(x, y)| {
                                city.tile(x, y).is_some_and(|t| t.terrain == Terrain::Water)
                            })
                })
                .map(|(i, _)| i)
                .expect("a generated region needs usable shoreline");
            city.place((shore % SIZE) as i32, (shore / SIZE) as i32, Kind::Pump)
                .unwrap();
            assert_eq!(city.tiles[shore].kind, Kind::Pump);
            city.validate().unwrap();
        }
    }
}
