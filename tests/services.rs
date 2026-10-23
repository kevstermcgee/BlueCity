use bluecity::{
    city::{City, Kind, Terrain, Tile},
    civic::DamageRecord,
    services::{Detention, EmergencyKind, FacilityLoad, ServicePurpose},
    world::WeatherKind,
};

fn town() -> City {
    let mut c = City::new(920, false);
    c.money = 500_000;
    for t in &mut c.tiles {
        *t = Tile::new(Terrain::Grass);
    }
    for x in 0..30 {
        set(&mut c, x, 10, Kind::Road);
    }
    set(&mut c, 12, 9, Kind::Residential);
    let home = c.idx(12, 9).unwrap();
    c.tiles[home].level = 2;
    c.tiles[home].population = 120;
    c.population = 120;
    c.initialize_expansion();
    c.civic.disaster_frequency = 0;
    for t in &mut c.tiles {
        t.connected = true;
        t.powered = true;
        t.watered = true;
    }
    c.ext.weather.rain = 0;
    c.ext.weather.wind_x = 0;
    c.ext.weather.wind_y = 0;
    c.ext.weather.kind = WeatherKind::Clear;
    c.ext.services.waste = vec![0; c.tiles.len()];
    c.ext.services.sick = vec![0; c.tiles.len()];
    c.ext.services.dead = vec![0; c.tiles.len()];
    c
}

fn damaged_town() -> (City, u32) {
    let mut c = town();
    set(&mut c, 10, 9, Kind::MaintenanceDepot);
    let home = c.idx(12, 9).unwrap();
    let previous = c.tiles[home];
    c.tiles[home].kind = Kind::Empty;
    c.tiles[home].population = 0;
    c.population = 0;
    c.ext.cells[home].rubble = true;
    c.ext.cells[home].condition = 0;
    c.civic.damage.push(DamageRecord {
        tile: home as u32,
        previous,
        repair_cost: 500,
        day: 0,
    });
    (c, home as u32)
}
fn queue(c: &mut City, tile: u32) -> Result<(), String> {
    let mut services = std::mem::take(&mut c.ext.services);
    let result = services.queue_repair(c, tile);
    c.ext.services = services;
    result
}

#[test]
fn repair_requests_charge_only_after_a_real_maintenance_crew_arrives() {
    let (mut c, tile) = damaged_town();
    let money = c.money;
    queue(&mut c, tile).unwrap();
    assert_eq!(c.money, money);
    assert!(queue(&mut c, tile).is_err());
    day(&mut c);
    assert_eq!(c.money, money);
    assert_eq!(c.tiles[tile as usize].kind, Kind::Empty);
    assert!(c
        .ext
        .services
        .missions
        .iter()
        .any(|m| m.purpose == ServicePurpose::Repair));
    day(&mut c);
    assert_eq!(c.money, money - 500);
    assert_eq!(c.tiles[tile as usize].kind, Kind::Residential);
    assert_eq!(c.ext.services.completed_repairs, 1);
    assert!(c.ext.services.repair_requests.is_empty());
    assert!(c.civic.damage.is_empty());
}

#[test]
fn repair_cancellation_outages_and_insufficient_arrival_funds_never_charge() {
    let (mut c, tile) = damaged_town();
    let depot = c.idx(10, 9).unwrap();
    c.tiles[depot].powered = false;
    assert!(queue(&mut c, tile).is_err());
    c.tiles[depot].powered = true;
    queue(&mut c, tile).unwrap();
    let money = c.money;
    day(&mut c);
    let mut state = std::mem::take(&mut c.ext.services);
    state.cancel_repair(&mut c, tile).unwrap();
    c.ext.services = state;
    days(&mut c, 3);
    assert_eq!(c.money, money);
    assert_eq!(c.tiles[tile as usize].kind, Kind::Empty);
    queue(&mut c, tile).unwrap();
    day(&mut c);
    c.money = 499;
    day(&mut c);
    assert_eq!(c.money, 499);
    assert_eq!(c.tiles[tile as usize].kind, Kind::Empty);
    assert!(c.ext.services.repair_requests.is_empty());
    assert!(c
        .ext
        .services
        .notices
        .iter()
        .any(|n| n.message.contains("Treasury")));
}

#[test]
fn invalidated_repair_requests_drop_without_spending_and_saved_crews_resume() {
    let (mut a, tile) = damaged_town();
    queue(&mut a, tile).unwrap();
    day(&mut a);
    let mut b: City = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    a.ext.services.validate_city(&a).unwrap();
    day(&mut a);
    day(&mut b);
    assert_eq!(a, b);
    let (mut c, tile) = damaged_town();
    queue(&mut c, tile).unwrap();
    day(&mut c);
    let money = c.money;
    c.civic.damage.clear();
    day(&mut c);
    assert_eq!(c.money, money);
    assert!(c.ext.services.repair_requests.is_empty());
    assert_eq!(c.ext.services.completed_repairs, 0);
}

#[test]
fn corrupt_service_stock_cargo_and_typed_vehicle_links_are_rejected() {
    let mut good = town();
    set(&mut good, 10, 9, Kind::Clinic);
    let home = good.idx(12, 9).unwrap();
    good.ext.services.sick[home] = 20;
    days(&mut good, 2);
    assert!(good
        .ext
        .services
        .missions
        .iter()
        .any(|m| m.returning && m.load > 0));
    good.ext.services.validate_city(&good).unwrap();
    let reject = |f: fn(&mut City)| {
        let mut bad = good.clone();
        f(&mut bad);
        assert!(bad.ext.services.validate_city(&bad).is_err());
    };
    reject(|c| c.ext.services.missions[0].cargo[0][0] = u32::MAX);
    reject(|c| c.ext.services.missions[0].cargo[0][1] += 1);
    reject(|c| c.ext.services.missions[0].vehicle = u32::MAX);
    reject(|c| c.ext.transport.vehicles.clear());
    reject(|c| c.ext.transport.vehicles[0].kind = bluecity::transport::VehicleKind::Car);
    reject(|c| c.ext.services.sick[0] = u32::MAX);
    reject(|c| c.ext.services.occupied_beds = 4000);
    reject(|c| c.ext.services.overflow += 1);
    reject(|c| {
        c.ext.services.detentions.push(Detention {
            count: u32::MAX,
            release_day: 100,
        })
    });
    reject(|c| {
        c.ext
            .services
            .failed_routes
            .push(bluecity::services::RouteFailure {
                facility: u32::MAX,
                target: 0,
                until_day: 10,
            })
    });
    reject(|c| {
        c.ext.services.facilities.push(FacilityLoad {
            tile: 0,
            kind: Kind::Park,
            used: 0,
            processed_today: 0,
        })
    });
}
fn set(c: &mut City, x: i32, y: i32, k: Kind) -> usize {
    let i = c.idx(x, y).unwrap();
    c.tiles[i].kind = k;
    c.tiles[i].connected = true;
    c.tiles[i].powered = true;
    c.tiles[i].watered = true;
    i
}
// Exercise the public daily transport/service phases with stable weather and
// utility supply, so an unrelated migration or seasonal event cannot mask delivery.
fn day(c: &mut City) {
    c.ext.day += 1;
    let mut transport = std::mem::take(&mut c.ext.transport);
    transport.advance_day(c, c.ext.day, c.ext.weather.rain);
    c.ext.transport = transport;
    let mut services = std::mem::take(&mut c.ext.services);
    services.advance_day(c);
    c.ext.services = services;
}
fn days(c: &mut City, n: usize) {
    for _ in 0..n {
        day(c);
    }
}

#[test]
fn working_beds_and_prisons_have_finite_capacity_and_stop_during_outages() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Clinic);
    set(&mut c, 14, 9, Kind::Hospital);
    set(&mut c, 16, 9, Kind::Prison);
    day(&mut c);
    assert_eq!(c.ext.services.beds, 152);
    assert_eq!(c.ext.services.prison_capacity, 200);
    c.funding[1] = 50;
    day(&mut c);
    assert_eq!(c.ext.services.beds, 76);
    assert_eq!(c.ext.services.prison_capacity, 100);
    c.funding[1] = 0;
    day(&mut c);
    assert_eq!(
        (c.ext.services.beds, c.ext.services.prison_capacity),
        (0, 0)
    );
}

#[test]
fn ambulances_must_collect_and_deliver_patients_before_beds_treat_them() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Clinic);
    let home = c.idx(12, 9).unwrap();
    c.ext.services.sick[home] = 20;
    day(&mut c);
    assert!(c
        .ext
        .services
        .missions
        .iter()
        .any(|m| m.purpose == ServicePurpose::Medical));
    assert_eq!(
        (c.ext.services.occupied_beds, c.ext.services.recovered),
        (0, 0)
    );
    let mut admitted = false;
    for _ in 0..60 {
        day(&mut c);
        admitted |= c.ext.services.occupied_beds > 0;
        assert!(c.ext.services.occupied_beds <= c.ext.services.beds);
    }
    assert!(admitted, "a reachable clinic must admit delivered patients");
    assert!(
        c.ext.services.recovered >= 16,
        "treatment must produce actual recoveries"
    );
}

#[test]
fn high_coverage_cannot_send_ambulances_across_a_missing_street() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Clinic);
    let home = c.idx(12, 9).unwrap();
    set(&mut c, 11, 10, Kind::Empty);
    c.ext.services.sick[home] = 10;
    c.ext.coverage.resize(c.tiles.len(), [100; 13]);
    days(&mut c, 8);
    assert_eq!(c.ext.services.dispatches, 0);
    assert_eq!(c.ext.services.occupied_beds, 0);
    assert_eq!(c.ext.services.recovered, 0);
    assert!(c.ext.services.ill >= 10);
}

#[test]
fn removing_a_road_mid_response_cancels_care_without_losing_patients() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Clinic);
    let home = c.idx(12, 9).unwrap();
    c.ext.services.sick[home] = 20;
    day(&mut c);
    assert_eq!(c.ext.services.dispatches, 1);
    set(&mut c, 11, 10, Kind::Empty);
    days(&mut c, 5);
    assert!(c.ext.services.missions.is_empty());
    assert_eq!(c.ext.services.occupied_beds, 0);
    assert_eq!(c.ext.services.recovered, 0);
    assert!(c.ext.services.ill >= 20);
}

#[test]
fn a_street_burning_during_arrivals_rebuilds_routes_before_new_dispatches() {
    let mut warm = town();
    set(&mut warm, 10, 9, Kind::Clinic);
    let first = warm.idx(12, 9).unwrap();
    warm.ext.services.sick[first] = 8;
    day(&mut warm);
    assert_eq!(warm.ext.services.dispatches, 1);
    let mut cold: City = serde_json::from_slice(&serde_json::to_vec(&warm).unwrap()).unwrap();
    for c in [&mut warm, &mut cold] {
        let remote = set(c, 25, 9, Kind::Residential);
        c.tiles[remote].population = 120;
        c.tiles[remote].level = 2;
        c.population += 120;
        c.ext.services.sick[remote] = 20;
        let bridge = c.idx(11, 10).unwrap();
        c.tiles[bridge].fire = 99;
        day(c);
        assert_eq!(c.tiles[bridge].kind, Kind::Empty);
        assert_eq!(
            c.ext.services.dispatches, 1,
            "the response already in transit must not dispatch a second ambulance across the street that burned down later that day"
        );
        assert!(c.ext.services.sick[remote] >= 20);
    }
    assert_eq!(
        warm, cold,
        "a restored street memo must produce the same arrivals and post-fire dispatch decisions"
    );
}

#[test]
fn an_interrupted_return_journey_restores_each_collected_patient_or_load() {
    for kind in [Kind::Clinic, Kind::Landfill, Kind::Cemetery] {
        let mut c = town();
        set(&mut c, 10, 9, kind);
        let home = c.idx(12, 9).unwrap();
        let stock = match kind {
            Kind::Clinic => &mut c.ext.services.sick,
            Kind::Landfill => &mut c.ext.services.waste,
            _ => &mut c.ext.services.dead,
        };
        stock[home] = if kind == Kind::Landfill { 300 } else { 8 };
        days(&mut c, 2);
        assert!(c
            .ext
            .services
            .missions
            .iter()
            .any(|m| m.returning && m.load > 0));
        set(&mut c, 10, 10, Kind::Empty);
        day(&mut c);
        let state = &c.ext.services;
        match kind {
            Kind::Clinic => {
                assert!(state.sick[home] >= 8);
                assert_eq!(state.recovered, 0);
                assert_eq!(state.occupied_beds, 0);
            }
            Kind::Landfill => {
                assert!(state.waste[home] >= 300);
                assert_eq!(state.waste_processed, 0);
                assert_eq!(state.landfill_used, 0);
            }
            _ => {
                assert_eq!(state.dead[home], 8);
                assert_eq!(state.burials, 0);
            }
        }
    }
}

#[test]
fn garbage_trucks_fill_landfills_only_after_delivery_and_respect_remaining_space() {
    let mut c = town();
    let landfill = set(&mut c, 10, 9, Kind::Landfill);
    let home = c.idx(12, 9).unwrap();
    c.ext.services.waste[home] = 1000;
    c.ext.services.facilities.push(FacilityLoad {
        tile: landfill as u32,
        kind: Kind::Landfill,
        used: 11980,
        processed_today: 0,
    });
    day(&mut c);
    assert_eq!(c.ext.services.landfill_used, 11980);
    assert_eq!(c.ext.services.waste_collected, 0);
    days(&mut c, 50);
    assert_eq!(c.ext.services.landfill_used, 12000);
    assert_eq!(c.ext.services.waste_processed, 20);
    assert_eq!(c.ext.services.waste_collected, 20);
    assert!(
        c.ext.services.overflow > 1000,
        "full landfill must leave real uncollected waste"
    );
}

#[test]
fn incineration_and_recycling_process_delivered_waste_instead_of_filling_landfills() {
    for kind in [Kind::Incinerator, Kind::Recycling] {
        let mut c = town();
        set(&mut c, 10, 9, kind);
        let home = c.idx(12, 9).unwrap();
        c.ext.services.waste[home] = 300;
        days(&mut c, 65);
        assert!(
            c.ext.services.waste_processed >= 64,
            "{kind:?} must receive and process a truckload"
        );
        assert_eq!(c.ext.services.landfill_used, 0);
        assert_eq!(c.ext.services.recycled > 0, kind == Kind::Recycling);
    }
}

#[test]
fn police_arrests_need_prison_space_and_sentences_eventually_end() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Police);
    set(&mut c, 14, 9, Kind::Prison);
    let home = c.idx(12, 9).unwrap();
    c.tiles[home].crime = 85;
    day(&mut c);
    assert_eq!(c.ext.services.prisoners, 0);
    days(&mut c, 15);
    assert!(c.ext.services.prisoners > 0);
    assert!(c.tiles[home].crime < 55);
    let prisoners = c.ext.services.prisoners;
    days(&mut c, 35);
    assert!(c.ext.services.prisoners < prisoners);
    let mut full = town();
    set(&mut full, 10, 9, Kind::Police);
    set(&mut full, 14, 9, Kind::Prison);
    full.tiles[home].crime = 85;
    full.ext.services.detentions.push(Detention {
        count: 200,
        release_day: 100,
    });
    day(&mut full);
    for _ in 0..20 {
        day(&mut full);
        if full.tiles[home].crime < 85 {
            break;
        }
    }
    assert_eq!(full.ext.services.prisoners, 200);
    assert!(
        full.tiles[home].crime >= 55,
        "overcrowding cannot create extra prison beds"
    );
}

#[test]
fn police_cannot_book_people_into_a_prison_on_a_disconnected_street() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Police);
    set(&mut c, 20, 9, Kind::Prison);
    set(&mut c, 16, 10, Kind::Empty);
    let home = c.idx(12, 9).unwrap();
    c.tiles[home].crime = 85;
    days(&mut c, 2);
    assert_eq!(c.ext.services.prison_capacity, 200);
    assert_eq!(c.ext.services.prisoners, 0);
    assert!(c.tiles[home].crime >= 55);
}

#[test]
fn wind_carries_fire_and_rain_prevents_spread() {
    let mut east = town();
    let source = east.idx(20, 20).unwrap();
    let target = east.idx(21, 20).unwrap();
    for i in [source, target] {
        east.tiles[i].terrain = Terrain::Forest;
        east.ext.cells[i].moisture = 10;
    }
    east.tiles[source].fire = 60;
    east.ext.weather.wind_x = 1;
    let mut wet = east.clone();
    wet.ext.weather.rain = 90;
    day(&mut east);
    day(&mut wet);
    assert!(east.tiles[target].fire > 0);
    assert_eq!(wet.tiles[target].fire, 0);
    assert!(wet.tiles[source].fire < 60);
}

#[test]
fn firefighters_extinguish_on_arrival_and_coverage_cannot_cross_a_broken_route() {
    let mut connected = town();
    set(&mut connected, 10, 9, Kind::Fire);
    let home = connected.idx(12, 9).unwrap();
    connected.tiles[home].fire = 35;
    let mut blocked = connected.clone();
    set(&mut blocked, 11, 10, Kind::Empty);
    blocked.ext.coverage.resize(blocked.tiles.len(), [100; 13]);
    day(&mut connected);
    day(&mut blocked);
    assert!(
        connected.tiles[home].fire > 35,
        "dispatch is not instant suppression"
    );
    assert!(connected
        .ext
        .services
        .missions
        .iter()
        .any(|m| m.purpose == ServicePurpose::Fire));
    days(&mut connected, 15);
    days(&mut blocked, 15);
    assert_eq!(connected.tiles[home].fire, 0);
    assert!(blocked.tiles[home].fire > 35);
}

#[test]
fn deathcare_waits_for_a_real_round_trip_and_outbreaks_are_saved() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Cemetery);
    let home = c.idx(12, 9).unwrap();
    c.ext.services.dead[home] = 4;
    day(&mut c);
    assert_eq!(c.ext.services.deaths_waiting, 4);
    assert_eq!(c.ext.services.burials, 0);
    days(&mut c, 40);
    assert_eq!(c.ext.services.burials, 4);
    assert_eq!(c.ext.services.deaths_waiting, 0);
    c.ext.services.sick[home] = 60;
    day(&mut c);
    assert!(c
        .ext
        .services
        .emergencies
        .iter()
        .any(|e| e.kind == EmergencyKind::Outbreak));
}

#[test]
fn in_flight_services_survive_a_city_save_and_continue_identically() {
    let mut a = town();
    set(&mut a, 10, 9, Kind::Clinic);
    set(&mut a, 14, 9, Kind::Recycling);
    let home = a.idx(12, 9).unwrap();
    a.ext.services.sick[home] = 20;
    a.ext.services.waste[home] = 300;
    days(&mut a, 2);
    assert!(!a.ext.services.missions.is_empty());
    let bytes = serde_json::to_vec(&a).unwrap();
    let mut b: City = serde_json::from_slice(&bytes).unwrap();
    for _ in 0..45 {
        day(&mut a);
        day(&mut b);
    }
    assert_eq!(a, b);
    a.ext.services.validate(a.tiles.len()).unwrap();
}

#[test]
fn reused_service_checkpoints_preserve_bytes_buffers_and_future_deliveries() {
    let mut a = town();
    for (x, kind) in [
        (8, Kind::MaintenanceDepot),
        (10, Kind::Clinic),
        (14, Kind::Landfill),
        (16, Kind::Cemetery),
    ] {
        set(&mut a, x, 9, kind);
    }
    let library = set(&mut a, 18, 9, Kind::Library);
    let previous = a.tiles[library];
    a.tiles[library].kind = Kind::Empty;
    a.ext.cells[library].rubble = true;
    a.ext.cells[library].condition = 0;
    a.civic.damage.push(DamageRecord {
        tile: library as u32,
        previous,
        repair_cost: 500,
        day: 0,
    });
    queue(&mut a, library as u32).unwrap();
    days(&mut a, 2);
    assert_eq!(a.ext.services.completed_repairs, 1);
    assert!(!a.ext.services.notices.is_empty());
    // Repair rebuilds actual utility coverage. Restore this service fixture's
    // supplied utilities before starting the independent collection journeys.
    for tile in &mut a.tiles {
        tile.connected = true;
        tile.powered = true;
        tile.watered = true;
    }
    let home = a.idx(12, 9).unwrap();
    a.ext.services.sick[home] = 20;
    a.ext.services.waste[home] = 300;
    a.ext.services.dead[home] = 8;
    days(&mut a, 2);
    let mut copied = bluecity::services::ServiceState::default();
    copied.checkpoint_copy_from(&a.ext.services);
    let loaded = copied
        .missions
        .iter()
        .position(|m| !m.cargo.is_empty())
        .unwrap();
    let stocks = (
        copied.waste.as_ptr(),
        copied.sick.as_ptr(),
        copied.dead.as_ptr(),
    );
    let crew_buffers = (
        copied.missions[loaded].cargo.as_ptr(),
        copied.missions[loaded].stops.as_ptr(),
    );
    let notice = copied.notices[0].message.as_ptr();
    copied.checkpoint_copy_from(&a.ext.services);
    assert_eq!(
        stocks,
        (
            copied.waste.as_ptr(),
            copied.sick.as_ptr(),
            copied.dead.as_ptr()
        )
    );
    assert_eq!(
        crew_buffers,
        (
            copied.missions[loaded].cargo.as_ptr(),
            copied.missions[loaded].stops.as_ptr()
        )
    );
    assert_eq!(notice, copied.notices[0].message.as_ptr());
    let mut b = a.clone();
    b.ext.services = bluecity::services::ServiceState::default();
    b.ext.services.checkpoint_copy_from(&a.ext.services);
    for _ in 0..15 {
        assert_eq!(copied, a.ext.services);
        assert_eq!(
            serde_json::to_vec(&copied).unwrap(),
            serde_json::to_vec(&a.ext.services).unwrap()
        );
        day(&mut a);
        day(&mut b);
        assert_eq!(
            a, b,
            "copied in-flight loads must arrive, recover and charge identically"
        );
        copied.checkpoint_copy_from(&a.ext.services);
    }
}

#[test]
fn warm_service_routes_match_cold_restores_after_water_and_one_way_edits() {
    let mut a = town();
    let old = a.idx(12, 9).unwrap();
    a.tiles[old].kind = Kind::Empty;
    a.tiles[old].population = 0;
    let home = set(&mut a, 10, 8, Kind::Residential);
    a.tiles[home].population = 120;
    a.tiles[home].level = 2;
    let clinic = set(&mut a, 10, 9, Kind::Clinic);
    a.ext.services.sick[home] = 20;
    days(&mut a, 2);
    let before = serde_json::to_vec(&a).unwrap();
    let mut services = std::mem::take(&mut a.ext.services);
    services.warm_routes(&a);
    a.ext.services = services;
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        before,
        "warming must not change a single saved authoritative value"
    );
    let mut b: City = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    for c in [&mut a, &mut b] {
        c.tiles[clinic].terrain = Terrain::Water;
    }
    day(&mut a);
    day(&mut b);
    assert_eq!(a, b);
    for c in [&mut a, &mut b] {
        c.tiles[clinic].terrain = Terrain::Grass;
        let road = set(c, 10, 10, Kind::OneWay);
        c.ext.cells[road].orientation = 1;
    }
    for _ in 0..4 {
        day(&mut a);
        day(&mut b);
        assert_eq!(a, b);
    }
    for c in [&mut a, &mut b] {
        let road = c.idx(10, 10).unwrap();
        c.ext.cells[road].orientation = 3;
    }
    for _ in 0..4 {
        day(&mut a);
        day(&mut b);
        assert_eq!(a, b);
    }
}

#[test]
fn public_city_day_advances_service_capacity_and_waste() {
    let mut c = town();
    set(&mut c, 10, 9, Kind::Landfill);
    c.advance_day();
    assert_eq!(c.ext.services.day, c.ext.day);
    assert!(c.ext.services.waste_generated > 0);
}

#[test]
fn a_dense_two_hundred_thousand_resident_city_has_a_working_collection_fleet() {
    let mut c = town();
    c.width = 128;
    c.height = 128;
    c.tiles = vec![Tile::new(Terrain::Grass); 128 * 128];
    for y in 0..128 {
        for x in 0..128 {
            let i = y * 128 + x;
            let t = &mut c.tiles[i];
            t.connected = true;
            t.powered = true;
            t.watered = true;
            if x % 4 == 0 || y % 4 == 0 {
                t.kind = Kind::Road;
            } else {
                t.kind = Kind::Residential;
                t.population = 24;
                t.level = 2;
            }
        }
    }
    for y in (1..128).step_by(16) {
        for x in (1..128).step_by(16) {
            set(&mut c, x, y, Kind::Landfill);
            set(&mut c, x + 1, y, Kind::Clinic);
            let clinic = c.idx(x + 1, y).unwrap();
            c.tiles[clinic].population = 0;
            let landfill = c.idx(x, y).unwrap();
            c.tiles[landfill].population = 0;
        }
    }
    c.population = c.tiles.iter().map(|t| t.population).sum();
    assert!(c.population > 200_000);
    c.ext.cells = vec![Default::default(); c.tiles.len()];
    c.ext.services = Default::default();
    c.ext.people = Default::default();
    c.ext.transport = Default::default();
    let started = std::time::Instant::now();
    days(&mut c, 20);
    let mut resumed: City = serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
    for _ in 0..25 {
        day(&mut c);
        day(&mut resumed);
    }
    assert_eq!(
        c, resumed,
        "dense collection must continue exactly with a cold memo after restoring"
    );
    let elapsed = started.elapsed();
    eprintln!(
        "dense routed services and transport: {:.2} ms/day; dispatched={}, missions={}, medical={}, waste={}, ill={}, health={}, garbage={}",
        elapsed.as_secs_f64() * 1000.0 / 70.0
        ,c.ext.services.dispatches,c.ext.services.missions.len(),
        c.ext.services.missions.iter().filter(|m|m.purpose==ServicePurpose::Medical).count(),
        c.ext.services.missions.iter().filter(|m|m.purpose==ServicePurpose::Waste).count(),
        c.ext.services.ill,c.health,c.garbage
    );
    assert!(
        c.ext.services.dispatches > 200,
        "collection must scale beyond four global trucks"
    );
    assert!(c.ext.services.waste_processed>c.ext.services.waste_generated/2,
        "a funded, reachable fleet must keep up with most generated waste: processed={} generated={}",
        c.ext.services.waste_processed,c.ext.services.waste_generated);
    assert!(
        c.garbage < 35,
        "a fully served city must not collapse under uncollected waste"
    );
    assert!(c.health > 60);
    c.ext.services.validate(c.tiles.len()).unwrap();
}
