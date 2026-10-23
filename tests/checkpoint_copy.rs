use bluecity::{
    city::City,
    transport::{TransitLine, TransitMode},
    world::WorldState,
};

fn working_world() -> WorldState {
    let mut city = City::new(17, true);
    city.initialize_expansion();
    city.civic.disaster_frequency = 0;
    for _ in 0..12 {
        city.advance_day();
    }
    let mut world = city.ext;
    assert!(!world.transport.commute_routes.is_empty());
    assert!(!world.transport.vehicles.is_empty());
    let route = world
        .transport
        .commute_routes
        .iter()
        .find(|r| r.path.len() > 2)
        .unwrap()
        .path
        .clone();
    // Include every variable-length line payload as well as actual saved
    // commuter and vehicle paths; no transit simulation is run on this fixture.
    world.transport.lines.push(TransitLine {
        id: 77,
        name: "A sufficiently long checkpoint line name".into(),
        mode: TransitMode::Bus,
        stops: vec![route[0], *route.last().unwrap()],
        reverse_route: route.iter().rev().copied().collect(),
        route,
        color: [12, 34, 56],
        fare: 4,
        enabled: true,
        running: true,
        ridership: 91,
    });
    world
}

fn assert_exact_copy(target: &mut WorldState, source: &WorldState) {
    let source_before = serde_json::to_vec(source).unwrap();
    let fresh = source.clone();
    target.checkpoint_copy_from(source);
    assert_eq!(*target, fresh);
    assert_eq!(serde_json::to_vec(target).unwrap(), source_before);
    assert_eq!(serde_json::to_vec(source).unwrap(), source_before);
}

#[test]
fn checkpoint_copy_matches_fresh_owned_clones_through_growth_shrink_and_empty_routes() {
    let initial = working_world();
    let mut source = initial.clone();
    let mut target = WorldState::default();
    assert_exact_copy(&mut target, &source);

    source.day += 360;
    source.minute_of_day = 773;
    source.weather.rain = 87;
    source.cells[5].condition = 23;
    source.buildings[0].occupants += 7;
    source.people.households[0].education = 67;
    source.people.births += 19;
    source.transport.commute_routes[0].planned_day += 1;
    source.transport.commute_routes[0].path.reverse();
    source.transport.vehicles[0].route.reverse();
    source.transport.vehicles[0].passengers = 17;
    source.transport.lines[0].name = "New line".into();
    source.transport.lines[0].route.reverse();
    source.transport.lines[0].reverse_route.clear();
    source.neighbors[0].region = "New name".into();
    source.neighbors[0].last_goods = -31;
    source.utility_grids[0].push(vec![2, 3, 4, 5]);
    assert_exact_copy(&mut target, &source);

    source.transport.commute_routes.truncate(2);
    source.transport.commute_routes[0].path.clear();
    source.transport.vehicles.clear();
    source.transport.lines.clear();
    source.people.households.clear();
    source.people.employment.clear();
    source.utility_grids[0].clear();
    source.neighbors.truncate(1);
    assert_exact_copy(&mut target, &source);

    // Re-expansion must append independent owners and copy all current values.
    source = initial;
    source.transport.commute_routes.extend_from_within(..3);
    source.transport.vehicles.extend_from_within(..3);
    source
        .transport
        .lines
        .push(source.transport.lines[0].clone());
    assert_exact_copy(&mut target, &source);
    source.transport.commute_routes[0].path[0] += 1;
    assert_ne!(source, target, "checkpoint paths must remain independent");
}

#[test]
fn repeated_checkpoints_retain_existing_hot_vector_and_route_allocations() {
    let mut source = working_world();
    let mut target = source.clone();
    let cells = target.cells.as_ptr();
    let buildings = target.buildings.as_ptr();
    let coverage = target.coverage.as_ptr();
    let households = target.people.households.as_ptr();
    let utility_grid = target.utility_grids[0][0].as_ptr();
    let commutes = target.transport.commute_routes.as_ptr();
    let paths: Vec<_> = target
        .transport
        .commute_routes
        .iter()
        .map(|r| r.path.as_ptr())
        .collect();
    let vehicles: Vec<_> = target
        .transport
        .vehicles
        .iter()
        .map(|v| v.route.as_ptr())
        .collect();
    let line = &target.transport.lines[0];
    let line_buffers = (
        line.route.as_ptr(),
        line.reverse_route.as_ptr(),
        line.stops.as_ptr(),
        line.name.as_ptr(),
    );
    let neighbor_name = target.neighbors[0].region.as_ptr();
    source.day += 360;
    source.transport.lines[0].name = "Updated name".into();
    source.neighbors[0].region = "Blue".into();
    assert_exact_copy(&mut target, &source);
    assert_eq!(target.cells.as_ptr(), cells);
    assert_eq!(target.buildings.as_ptr(), buildings);
    assert_eq!(target.coverage.as_ptr(), coverage);
    assert_eq!(target.people.households.as_ptr(), households);
    assert_eq!(target.utility_grids[0][0].as_ptr(), utility_grid);
    assert_eq!(target.transport.commute_routes.as_ptr(), commutes);
    for (route, pointer) in target.transport.commute_routes.iter().zip(paths) {
        assert_eq!(route.path.as_ptr(), pointer);
    }
    for (vehicle, pointer) in target.transport.vehicles.iter().zip(vehicles) {
        assert_eq!(vehicle.route.as_ptr(), pointer);
    }
    let line = &target.transport.lines[0];
    assert_eq!(
        (
            line.route.as_ptr(),
            line.reverse_route.as_ptr(),
            line.stops.as_ptr(),
            line.name.as_ptr(),
        ),
        line_buffers
    );
    assert_eq!(target.neighbors[0].region.as_ptr(), neighbor_name);
}
