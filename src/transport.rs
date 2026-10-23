//! Routed trips, maintained roads and player-authored transit services.
use crate::city::{City, Kind};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitMode {
    #[default]
    Bus,
    Rail,
    Metro,
    Ferry,
    Freight,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleKind {
    Pedestrian,
    Helicopter,
    Jet,
    #[default]
    Car,
    Bus,
    Train,
    Metro,
    Truck,
    Service,
    Aircraft,
    Ferry,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitLine {
    #[serde(default)]
    pub reverse_route: Vec<u32>,
    #[serde(default)]
    pub running: bool,
    pub route: Vec<u32>,
    pub id: u32,
    pub name: String,
    pub mode: TransitMode,
    pub stops: Vec<u32>,
    pub color: [u8; 3],
    pub fare: u16,
    pub enabled: bool,
    pub ridership: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: u32,
    pub kind: VehicleKind,
    pub tile: u32,
    pub next_tile: u32,
    pub progress: u16,
    pub passengers: u16,
    pub line_id: Option<u32>,
    pub heading: u8,
    pub route: Vec<u32>,
    pub leg: u16,
    pub age_days: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommuteRoute {
    pub home: u32,
    pub employer: u32,
    pub path: Vec<u32>,
    pub planned_day: u64,
    pub topology: u64,
}
#[derive(Clone, Debug)]
struct AccessMemo {
    width: u16,
    height: u16,
    markers: Vec<u8>,
    road_access: Vec<u8>,
    nearest_road: Vec<u32>,
}
#[derive(Clone, Debug, Default)]
struct AccessMemoCache(Option<std::sync::Arc<AccessMemo>>);
impl PartialEq for AccessMemoCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for AccessMemoCache {}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TransportState {
    #[serde(skip)]
    access_memo: AccessMemoCache,
    pub air_revenue_balance: i64,
    pub served_trips: u32,
    pub unserved_trips: u32,
    pub commute_routes: Vec<CommuteRoute>,
    pub lines: Vec<TransitLine>,
    pub vehicles: Vec<Vehicle>,
    pub next_id: u32,
    pub next_line: u32,
    pub trips: u32,
    pub completed_trips: u32,
    pub average_commute: u16,
    pub congestion: u8,
    pub ridership: u32,
    pub fare_revenue: i64,
    pub fare_balance: i64,
    pub freight: u32,
    pub road_wear: u8,
    pub road_access: Vec<u8>,
    pub nearest_road: Vec<u32>,
    pub route_flow: Vec<u32>,
    pub jams: Vec<u32>,
}
fn allowed(city: &City, from: usize, to: usize) -> bool {
    if !city.tiles[from].kind.is_road() {
        return false;
    }
    if city.tiles[from].kind != Kind::OneWay {
        return true;
    }
    let dir = city.ext.cells.get(from).map_or(0, |c| c.orientation) % 4;
    let w = city.width as usize;
    match dir {
        0 => to == from + 1 && to / w == from / w,
        1 => to == from + w,
        2 => from > 0 && to == from - 1 && to / w == from / w,
        _ => from >= w && to == from - w,
    }
}
fn road_topology(city: &City) -> u64 {
    city.tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.kind.is_road())
        .fold(0u64, |h, (i, t)| {
            h.wrapping_mul(1099511628211)
                ^ ((i as u64) << 8 | t.kind as u64)
                ^ city.ext.cells.get(i).map_or(0, |c| c.orientation as u64)
        })
}
fn track(city: &City, i: usize, mode: TransitMode) -> bool {
    let t = &city.tiles[i];
    match mode {
        TransitMode::Bus => t.kind.is_road() || matches!(t.kind, Kind::BusStop | Kind::BusDepot),
        TransitMode::Rail => matches!(t.kind, Kind::Rail | Kind::Station) || t.rail_crossing,
        TransitMode::Metro => {
            matches!(t.kind, Kind::Subway | Kind::SubwayStation)
                || city.ext.cells.get(i).is_some_and(|c| c.subway)
        }
        TransitMode::Ferry => {
            t.terrain == crate::city::Terrain::Water
                || matches!(t.kind, Kind::Marina | Kind::Seaport)
        }
        TransitMode::Freight => {
            matches!(
                t.kind,
                Kind::Rail | Kind::Station | Kind::FreightStation | Kind::Seaport
            ) || t.rail_crossing
        }
    }
}
pub fn facility_operational(city: &City, i: usize) -> bool {
    let Some(t) = city.tiles.get(i) else {
        return false;
    };
    t.connected
        && t.powered
        && t.watered
        && t.fire == 0
        && city.funding[t.kind.funding()] > 0
        && city.ext.cells.get(i).is_none_or(|c| c.operational())
}
fn depot_capacities(city: &City, operational: bool) -> [u32; 5] {
    let mut counts = [0u32; 5];
    for (i, t) in city.tiles.iter().enumerate() {
        let mode = match t.kind {
            Kind::BusDepot => 0,
            Kind::Station => 1,
            Kind::SubwayStation => 2,
            Kind::Marina | Kind::Seaport => 3,
            Kind::FreightStation => 4,
            _ => continue,
        };
        if !operational || facility_operational(city, i) {
            counts[mode] += 1;
        }
    }
    for (count, capacity) in counts.iter_mut().zip([1800, 6000, 10000, 2500, 8000]) {
        *count *= capacity;
    }
    counts
}
fn depot_capacity(city: &City, mode: TransitMode, operational: bool) -> u32 {
    depot_capacities(city, operational)[mode as usize]
}
pub fn route_between(city: &City, start: usize, end: usize, mode: TransitMode) -> Option<Vec<u32>> {
    if start >= city.tiles.len() || end >= city.tiles.len() {
        return None;
    }
    let n = city.tiles.len();
    let mut parent = vec![u32::MAX; n];
    let mut q = VecDeque::from([start]);
    parent[start] = start as u32;
    while let Some(i) = q.pop_front() {
        if i == end {
            break;
        }
        for j in city.neighbors(i) {
            let connector = j == end;
            if parent[j] != u32::MAX || (!track(city, j, mode) && !connector) {
                continue;
            }
            if mode == TransitMode::Bus && city.tiles[i].kind.is_road() && !allowed(city, i, j) {
                continue;
            }
            parent[j] = i as u32;
            q.push_back(j);
        }
    }
    if parent[end] == u32::MAX {
        return None;
    }
    let mut at = end;
    let mut result = vec![at as u32];
    while at != start {
        at = parent[at] as usize;
        result.push(at as u32);
    }
    result.reverse();
    Some(result)
}
fn route_line(city: &City, mode: TransitMode, stops: &[u32]) -> Option<Vec<u32>> {
    let mut route = vec![];
    for pair in stops.windows(2) {
        let leg = route_between(city, pair[0] as usize, pair[1] as usize, mode)?;
        if route.is_empty() {
            route.extend(leg);
        } else {
            route.extend(leg.into_iter().skip(1));
        }
        if route.len() > u16::MAX as usize {
            return None;
        }
    }
    Some(route)
}
/// Regional cargo needs a working port or a real railway from its freight yard
/// to the boundary. A player line can instead bring a yard's goods to a port.
pub fn freight_export_capacity(city: &City) -> u32 {
    let ports: Vec<_> = city
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            (t.kind == Kind::Seaport && facility_operational(city, i)).then_some(i)
        })
        .collect();
    let stations: Vec<_> = city
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            (t.kind == Kind::FreightStation && facility_operational(city, i)).then_some(i)
        })
        .collect();
    if stations.is_empty() {
        return ports.len() as u32 * 6000;
    }
    let mut reached = vec![false; city.tiles.len()];
    let mut queue = VecDeque::new();
    let width = usize::from(city.width);
    for (i, hit) in reached.iter_mut().enumerate() {
        if (i % width == 0
            || i % width + 1 == width
            || i / width == 0
            || i / width + 1 == usize::from(city.height))
            && track(city, i, TransitMode::Freight)
        {
            *hit = true;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        for j in city.neighbors(i) {
            if !reached[j] && track(city, j, TransitMode::Freight) {
                reached[j] = true;
                queue.push_back(j);
            }
        }
    }
    for line in &city.ext.transport.lines {
        if line.mode != TransitMode::Freight
            || !line.enabled
            || !line.stops.iter().any(|&s| ports.contains(&(s as usize)))
            || !line
                .stops
                .iter()
                .all(|&s| facility_operational(city, s as usize))
            || route_line(city, TransitMode::Freight, &line.stops).is_none()
        {
            continue;
        }
        for &s in &line.stops {
            reached[s as usize] = true;
        }
    }
    (ports.len() + stations.into_iter().filter(|&s| reached[s]).count()) as u32 * 6000
}
fn walking_catchments(
    city: &City,
    lines: &[TransitLine],
) -> std::collections::BTreeMap<u32, Vec<usize>> {
    let mut result = std::collections::BTreeMap::new();
    if !lines.iter().any(|l| l.running) {
        return result;
    }
    let mut seen = vec![0u32; city.tiles.len()];
    let mut generation = 0;
    for stop in lines
        .iter()
        .filter(|l| l.running)
        .flat_map(|l| l.stops.iter())
        .copied()
    {
        if result.contains_key(&stop) {
            continue;
        }
        generation += 1;
        seen[stop as usize] = generation;
        let mut found = vec![stop as usize];
        let mut q = VecDeque::from([(stop as usize, 0u8)]);
        while let Some((i, steps)) = q.pop_front() {
            if steps == 6 {
                continue;
            }
            for j in city.neighbors(i) {
                let t = city.tiles[j];
                if seen[j] == generation
                    || (t.terrain == crate::city::Terrain::Water && !t.kind.is_road())
                {
                    continue;
                }
                seen[j] = generation;
                found.push(j);
                // Building doors are destinations. Walkers do not pass through
                // buildings, and open water requires an actual road bridge.
                if !t.kind.is_building()
                    || matches!(
                        t.kind,
                        Kind::Park
                            | Kind::Plaza
                            | Kind::Playground
                            | Kind::WaterfrontPark
                            | Kind::NatureReserve
                            | Kind::Beach
                    )
                {
                    q.push_back((j, steps + 1));
                }
            }
        }
        found.sort_unstable();
        result.insert(stop, found);
    }
    result
}
impl TransportState {
    /// Prepare authentic cached commute plans for an existing occupied city.
    /// This does not advance time, charge fares or create fictional trips.
    pub fn initialize_commutes(&mut self, city: &City) {
        self.rebuild_access(city);
        let topology = road_topology(city);
        let mut search = RouteSearch::new(city.tiles.len());
        self.commute_routes.clear();
        for h in &city.ext.people.households {
            let Some(employer) = h.employer else { continue };
            let start = self.nearest_road[h.home as usize];
            let end = self.nearest_road[employer as usize];
            if start == u32::MAX || end == u32::MAX {
                continue;
            }
            let path = road_route_with(city, start as usize, end as usize, &mut search)
                .unwrap_or_default();
            self.commute_routes.push(CommuteRoute {
                home: h.home,
                employer,
                path,
                planned_day: city.ext.day,
                topology,
            });
        }
    }
    pub fn create_line(
        &mut self,
        city: &City,
        mode: TransitMode,
        stops: Vec<u32>,
    ) -> Result<u32, String> {
        if stops.len() < 2 || stops.len() > 64 {
            return Err("A transit line needs 2 to 64 stops.".into());
        }
        if self.lines.len() >= 128 {
            return Err("This city supports 128 transit lines.".into());
        }
        let unique: std::collections::BTreeSet<_> = stops.iter().collect();
        if unique.len() != stops.len() {
            return Err("Choose at least two distinct stops, without duplicates.".into());
        }
        if depot_capacity(city, mode, true) == 0 {
            return Err("Complete and fund an operational matching depot or station first.".into());
        }
        for &s in &stops {
            let Some(t) = city.tiles.get(s as usize) else {
                return Err("Stop is outside the map.".into());
            };
            let valid = match mode {
                TransitMode::Bus => matches!(t.kind, Kind::BusStop | Kind::BusDepot),
                TransitMode::Rail => t.kind == Kind::Station,
                TransitMode::Metro => t.kind == Kind::SubwayStation,
                TransitMode::Ferry => matches!(t.kind, Kind::Marina | Kind::Seaport),
                TransitMode::Freight => matches!(t.kind, Kind::FreightStation | Kind::Seaport),
            };
            if !valid {
                return Err("Choose stops that match this transit mode.".into());
            }
        }
        let route = route_line(city, mode, &stops).ok_or(
            "Stops must be connected by the matching road, rail, subway or water network.",
        )?;
        let reverse_route =
            route_line(city, mode, &stops.iter().rev().copied().collect::<Vec<_>>())
                .unwrap_or_default();
        self.next_line += 1;
        let id = self.next_line;
        let color = [
            [48, 130, 220],
            [230, 110, 65],
            [90, 175, 90],
            [155, 90, 200],
        ][id as usize % 4];
        self.lines.push(TransitLine {
            route,
            reverse_route,
            running: false,
            id,
            name: format!("{:?} {}", mode, id),
            mode,
            stops,
            color,
            fare: 2,
            enabled: true,
            ridership: 0,
        });
        Ok(id)
    }
    fn rebuild_access(&mut self, city: &City) {
        let n = city.tiles.len();
        let markers: Vec<u8> = city
            .tiles
            .iter()
            .map(|t| {
                u8::from(t.kind.is_road() && t.connected)
                    | u8::from(t.terrain == crate::city::Terrain::Water) << 1
            })
            .collect();
        if let Some(cache) =
            self.access_memo.0.as_ref().filter(|m| {
                m.width == city.width && m.height == city.height && m.markers == markers
            })
        {
            self.road_access.clone_from(&cache.road_access);
            self.nearest_road.clone_from(&cache.nearest_road);
            return;
        }
        let mut q = VecDeque::new();
        self.road_access.resize(n, u8::MAX);
        self.road_access.fill(u8::MAX);
        self.nearest_road.resize(n, u32::MAX);
        self.nearest_road.fill(u32::MAX);
        for (i, t) in city.tiles.iter().enumerate() {
            if t.kind.is_road() && t.connected {
                self.road_access[i] = 0;
                self.nearest_road[i] = i as u32;
                q.push_back(i);
            }
        }
        while let Some(i) = q.pop_front() {
            if self.road_access[i] >= 6 {
                continue;
            }
            for j in city.neighbors(i) {
                if self.road_access[j] > self.road_access[i] + 1
                    && city.tiles[j].terrain != crate::city::Terrain::Water
                {
                    self.road_access[j] = self.road_access[i] + 1;
                    self.nearest_road[j] = self.nearest_road[i];
                    q.push_back(j);
                }
            }
        }
        self.access_memo.0 = Some(std::sync::Arc::new(AccessMemo {
            width: city.width,
            height: city.height,
            markers,
            road_access: self.road_access.clone(),
            nearest_road: self.nearest_road.clone(),
        }));
    }
    pub fn advance_day(&mut self, city: &mut City, day: u64, rain: u8) {
        let n = city.tiles.len();
        self.rebuild_access(city);
        self.route_flow.resize(n, 0);
        self.route_flow.fill(0);
        let mut distances = vec![u32::MAX; n];
        let mut parent = vec![u32::MAX; n];
        let mut destination = vec![u32::MAX; n];
        let mut heap = BinaryHeap::new();
        for (i, t) in city.tiles.iter().enumerate() {
            if t.jobs > 0 && self.nearest_road[i] != u32::MAX {
                let r = self.nearest_road[i] as usize;
                if distances[r] > 0 {
                    distances[r] = 0;
                    destination[r] = i as u32;
                    heap.push(Reverse((0u32, r)));
                }
            }
        }
        let mut needed = vec![false; n];
        let mut remaining = 0usize;
        for (i, t) in city.tiles.iter().enumerate() {
            if t.population > 0 && self.nearest_road[i] != u32::MAX {
                let road = self.nearest_road[i] as usize;
                if !needed[road] {
                    needed[road] = true;
                    remaining += 1;
                }
            }
        }
        let fallback_needed = city.ext.people.households.is_empty()
            || city
                .ext
                .people
                .households
                .iter()
                .any(|h| h.employer.is_none());
        while let Some(Reverse((cost, i))) = if fallback_needed { heap.pop() } else { None } {
            if cost != distances[i] {
                continue;
            }
            if needed[i] {
                needed[i] = false;
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
            for j in city.neighbors(i) {
                if !city.tiles[j].kind.is_road() || !allowed(city, j, i) {
                    continue;
                }
                let condition = city.ext.cells.get(j).map_or(100, |c| c.condition);
                let step = 10
                    + city.tiles[i].traffic as u32 / 5
                    + (100 - condition) as u32 / 5
                    + u32::from(city.tiles[i].kind == Kind::TunnelRoad) * 2;
                let next = cost.saturating_add(step);
                if next < distances[j] {
                    distances[j] = next;
                    parent[j] = i as u32;
                    destination[j] = destination[i];
                    heap.push(Reverse((next, j)));
                }
            }
        }
        let capacities = depot_capacities(city, true);
        let mut line_capacity = Vec::with_capacity(self.lines.len());
        for line in &mut self.lines {
            line.route = route_line(city, line.mode, &line.stops).unwrap_or_default();
            line.reverse_route = route_line(
                city,
                line.mode,
                &line.stops.iter().rev().copied().collect::<Vec<_>>(),
            )
            .unwrap_or_default();
            let capacity = capacities[line.mode as usize];
            line.running = line.enabled
                && !line.route.is_empty()
                && capacity > 0
                && line
                    .stops
                    .iter()
                    .all(|&s| facility_operational(city, s as usize));
            line_capacity.push(if line.running { capacity } else { 0 });
        }
        let walking = walking_catchments(city, &self.lines);
        self.trips = 0;
        self.served_trips = 0;
        self.unserved_trips = 0;
        let mut distance_sum = 0u64;
        // Hold cache indices until a vehicle is actually spawned. Most commute
        // paths only contribute flow; cloning every path needlessly copies the
        // same geometry even when the visual fleet is already full.
        let mut origins = vec![];
        self.ridership = 0;
        self.fare_revenue = 0;
        for line in &mut self.lines {
            line.ridership = 0;
        }
        self.commute_routes.retain(|r| {
            city.tiles
                .get(r.home as usize)
                .is_some_and(|t| t.population > 0)
                && city
                    .tiles
                    .get(r.employer as usize)
                    .is_some_and(|t| t.jobs > 0)
        });
        let mut cache_index = vec![usize::MAX; n];
        for (i, r) in self.commute_routes.iter().enumerate() {
            cache_index[r.home as usize] = i;
        }
        let mut employer = vec![u32::MAX; n];
        for h in &city.ext.people.households {
            if let Some(at) = h.employer {
                employer[h.home as usize] = at;
            }
        }
        let topology = road_topology(city);
        let mut capacity_pool = capacities;
        let mut route_search = None;
        for (i, t) in city.tiles.iter().enumerate() {
            if t.population == 0 || self.nearest_road[i] == u32::MAX {
                continue;
            }
            let origin = self.nearest_road[i] as usize;
            let mut dest = employer[i];
            let mut route = Vec::new();
            let mut cached_path = None;
            if dest != u32::MAX {
                let at = cache_index[i];
                let current = if at == usize::MAX {
                    None
                } else {
                    self.commute_routes.get(at)
                };
                let valid = current.is_some_and(|r| {
                    r.employer == dest
                        && r.topology == topology
                        && day < r.planned_day + 7
                        && !(day > r.planned_day && (day + u64::from(r.home)).is_multiple_of(7))
                });
                if valid {
                    cached_path = Some(at);
                } else {
                    let goal = self
                        .nearest_road
                        .get(dest as usize)
                        .copied()
                        .unwrap_or(u32::MAX);
                    if goal != u32::MAX {
                        route = road_route_with(
                            city,
                            origin,
                            goal as usize,
                            route_search.get_or_insert_with(|| RouteSearch::new(n)),
                        )
                        .unwrap_or_default();
                    }
                    let record = CommuteRoute {
                        home: i as u32,
                        employer: dest,
                        path: route,
                        planned_day: day,
                        topology,
                    };
                    if at == usize::MAX {
                        cache_index[i] = self.commute_routes.len();
                        self.commute_routes.push(record);
                    } else {
                        self.commute_routes[at] = record;
                    }
                    cached_path = Some(cache_index[i]);
                    route = Vec::new();
                }
            } else if distances[origin] != u32::MAX {
                dest = destination[origin];
                let mut at = origin;
                route.push(at as u32);
                for _ in 0..n {
                    if parent[at] == u32::MAX {
                        break;
                    }
                    at = parent[at] as usize;
                    route.push(at as u32);
                }
            }
            if dest == u32::MAX {
                continue;
            }
            let path = cached_path
                .map(|at| self.commute_routes[at].path.as_slice())
                .unwrap_or(&route);
            let road_available = !path.is_empty();
            let trips = t.population / 3;
            self.trips += trips;
            let mut transit = 0u32;
            for (line_index, line) in self.lines.iter_mut().enumerate() {
                let mode = line.mode as usize;
                if line_capacity[line_index] == 0
                    || capacity_pool[mode] == 0
                    || line.mode == TransitMode::Freight
                {
                    continue;
                }
                let reachable = line.stops.iter().enumerate().find_map(|(a, &s)| {
                    if !walking.get(&s).is_some_and(|v| v.binary_search(&i).is_ok()) {
                        return None;
                    }
                    line.stops.iter().enumerate().find_map(|(b, &d)| {
                        if a == b
                            || (b < a && line.reverse_route.is_empty())
                            || !walking
                                .get(&d)
                                .is_some_and(|v| v.binary_search(&(dest as usize)).is_ok())
                        {
                            return None;
                        }
                        Some((
                            s,
                            d,
                            if b > a {
                                line.route.len()
                            } else {
                                line.reverse_route.len()
                            },
                        ))
                    })
                });
                let Some((boarding, alighting, line_distance)) = reachable else {
                    continue;
                };
                let share = if !road_available {
                    100
                } else if city.policies[1] {
                    70
                } else {
                    (60i32 - line.fare as i32 * 5).clamp(5, 60) as u32
                };
                let users = (trips.saturating_sub(transit) * share / 100)
                    .min(line_capacity[line_index])
                    .min(capacity_pool[mode]);
                line.ridership += users;
                line_capacity[line_index] -= users;
                capacity_pool[mode] -= users;
                transit += users;
                distance_sum += users as u64
                    * (line_distance
                        + city.distance(i, boarding as usize)
                        + city.distance(dest as usize, alighting as usize))
                        as u64;
            }
            let walking_trips = if road_available && path.len() <= 5 {
                trips.saturating_sub(transit) / 4
            } else {
                0
            };
            let driving = if road_available {
                trips.saturating_sub(transit + walking_trips)
            } else {
                0
            };
            self.served_trips += transit + walking_trips + driving;
            self.unserved_trips += trips - transit - walking_trips - driving;
            distance_sum += path.len() as u64 * (driving + walking_trips) as u64;
            for &at in path {
                self.route_flow[at as usize] += driving;
            }
            if driving > 0 && path.len() > 1 && origins.len() < 768 {
                origins.push((cached_path, route));
            }
        }
        for line in &self.lines {
            self.ridership += line.ridership;
            if !city.policies[1] {
                self.fare_revenue += line.ridership as i64 * line.fare as i64 / 30;
            }
        }
        self.fare_balance += self.fare_revenue;
        self.average_commute = (distance_sum / self.served_trips.max(1) as u64 * 2).min(600) as u16;
        city.ext.people.average_commute = self.average_commute;
        self.jams.clear();
        let mut congested = 0;
        let mut roads = 0;
        for (i, t) in city.tiles.iter_mut().enumerate() {
            if !t.kind.is_road() {
                continue;
            }
            roads += 1;
            let capacity = match t.kind {
                Kind::Highway => 1100,
                Kind::Avenue => 420,
                Kind::OneWay => 260,
                Kind::Ramp => 320,
                Kind::Roundabout => 350,
                _ => 150,
            };
            let condition = city.ext.cells.get(i).map_or(100, |c| c.condition).max(20) as u32;
            let traffic =
                (self.route_flow[i] * 100 / (capacity * condition / 100).max(1)).min(100) as u8;
            t.traffic = traffic;
            if traffic > 80 {
                congested += 1;
                self.jams.push(i as u32);
            }
        }
        self.congestion = (congested * 100 / roads.max(1)).min(100) as u8;
        self.freight = city
            .ext
            .people
            .goods_imported
            .saturating_add(city.ext.people.goods_exported);
        self.vehicles.retain(|v| {
            v.age_days
                < if matches!(
                    v.kind,
                    VehicleKind::Helicopter | VehicleKind::Jet | VehicleKind::Aircraft
                ) {
                    360
                } else {
                    90
                }
                && v.route.len() > 1
                && v.route.iter().all(|&i| (i as usize) < n)
                && if matches!(
                    v.kind,
                    VehicleKind::Helicopter | VehicleKind::Jet | VehicleKind::Aircraft
                ) {
                    true
                } else if let Some(id) = v.line_id {
                    self.lines.iter().find(|l| l.id == id).is_some_and(|l| {
                        l.running && (v.route == l.route || v.route == l.reverse_route)
                    })
                } else if v.kind == VehicleKind::Service {
                    v.route.len() > 1
                        && v.route[1..v.route.len() - 1]
                            .iter()
                            .all(|&i| city.tiles[i as usize].kind.is_road())
                        && v.route.windows(2).enumerate().all(|(n, p)| {
                            n == 0
                                || n + 2 == v.route.len()
                                || allowed(city, p[0] as usize, p[1] as usize)
                        })
                } else {
                    v.route
                        .iter()
                        .all(|&i| city.tiles[i as usize].kind.is_road())
                        && v.route
                            .windows(2)
                            .all(|p| allowed(city, p[0] as usize, p[1] as usize))
                }
        });
        for v in &mut self.vehicles {
            v.age_days += 1;
            let speed = match v.kind {
                VehicleKind::Jet | VehicleKind::Aircraft => 4000,
                VehicleKind::Helicopter => 2000,
                VehicleKind::Pedestrian => 180,
                // A rules day is a whole day of work, so an emergency or waste
                // crew can cross a normal city catchment before tomorrow.
                VehicleKind::Service => 128000,
                _ => (380 - city.tiles[v.tile as usize].traffic as u32 * 2 - rain as u32).max(60),
            };
            let mut progress = v.progress as u32 + speed;
            while progress >= 1000 {
                progress -= 1000;
                v.leg += 1;
                if v.leg as usize >= v.route.len() - 1 {
                    let destination = *v.route.last().unwrap() as usize;
                    if v.kind == VehicleKind::Helicopter {
                        city.tiles[destination].fire =
                            city.tiles[destination].fire.saturating_sub(60);
                        city.health = city.health.saturating_add(1).min(100);
                    }
                    if matches!(v.kind, VehicleKind::Jet | VehicleKind::Aircraft) {
                        self.air_revenue_balance += v.passengers as i64 / 10;
                        city.ext.tourism = city.ext.tourism.saturating_add(v.passengers as u32);
                    }
                    v.tile = destination as u32;
                    v.next_tile = v.tile;
                    progress = 0;
                    v.leg = (v.route.len() - 1) as u16;
                    v.age_days = if matches!(
                        v.kind,
                        VehicleKind::Helicopter | VehicleKind::Jet | VehicleKind::Aircraft
                    ) {
                        360
                    } else {
                        90
                    };
                    self.completed_trips += 1;
                    break;
                }
                v.tile = v.route[v.leg as usize];
                v.next_tile = v.route[v.leg as usize + 1];
                v.heading = if v.next_tile % city.width as u32 > v.tile % city.width as u32 {
                    0
                } else if v.next_tile / city.width as u32 > v.tile / city.width as u32 {
                    1
                } else if v.next_tile % (city.width as u32) < v.tile % (city.width as u32) {
                    2
                } else {
                    3
                };
            }
            v.progress = progress as u16;
        }
        if self.vehicles.len() < 1024 {
            for (cached_path, route) in origins.into_iter().step_by(3).take(64) {
                let route = cached_path.map_or(route, |at| self.commute_routes[at].path.clone());
                self.next_id += 1;
                self.vehicles.push(Vehicle {
                    id: self.next_id,
                    tile: route[0],
                    next_tile: route[1],
                    kind: if route.len() <= 6 {
                        VehicleKind::Pedestrian
                    } else {
                        VehicleKind::Car
                    },
                    route,
                    passengers: 1,
                    ..Default::default()
                });
            }
        }
        for line in &self.lines {
            if !line.running
                || line.stops.len() < 2
                || self.vehicles.iter().any(|v| v.line_id == Some(line.id))
            {
                continue;
            }
            let route = line.route.clone();
            if route.len() < 2 {
                continue;
            }
            self.next_id += 1;
            self.vehicles.push(Vehicle {
                id: self.next_id,
                kind: match line.mode {
                    TransitMode::Bus => VehicleKind::Bus,
                    TransitMode::Rail => VehicleKind::Train,
                    TransitMode::Metro => VehicleKind::Metro,
                    TransitMode::Ferry => VehicleKind::Ferry,
                    TransitMode::Freight => VehicleKind::Truck,
                },
                tile: route[0],
                next_tile: route[1],
                route,
                passengers: line.ridership.min(255) as u16,
                line_id: Some(line.id),
                ..Default::default()
            });
        }
        let flight_path = |start: usize, end: usize| {
            let mut path = vec![start as u32];
            let (mut x, mut y) = (
                (start % city.width as usize) as i32,
                (start / city.width as usize) as i32,
            );
            let (tx, ty) = (
                (end % city.width as usize) as i32,
                (end / city.width as usize) as i32,
            );
            while x != tx || y != ty {
                if x != tx {
                    x += (tx - x).signum();
                } else {
                    y += (ty - y).signum();
                }
                path.push((y as usize * city.width as usize + x as usize) as u32);
            }
            path
        };
        if self.vehicles.len() < 1200 {
            if let Some(base) = city.tiles.iter().enumerate().find_map(|(i, t)| {
                (matches!(t.kind, Kind::FireHQ | Kind::Hospital) && facility_operational(city, i))
                    .then_some(i)
            }) {
                let target = city.tiles.iter().position(|t| t.fire > 0).or_else(|| {
                    if city.health < 40 {
                        city.tiles.iter().position(|t| t.population > 0)
                    } else {
                        None
                    }
                });
                if let Some(target) = target {
                    if target != base
                        && !self.vehicles.iter().any(|v| {
                            v.kind == VehicleKind::Helicopter
                                && v.route.last() == Some(&(target as u32))
                        })
                    {
                        let route = flight_path(base, target);
                        self.next_id += 1;
                        self.vehicles.push(Vehicle {
                            id: self.next_id,
                            kind: VehicleKind::Helicopter,
                            tile: route[0],
                            next_tile: route[1],
                            route,
                            passengers: 4,
                            ..Default::default()
                        });
                    }
                }
            }
            if day.is_multiple_of(10) {
                if let Some(airport) = city.tiles.iter().enumerate().find_map(|(i, t)| {
                    (matches!(t.kind, Kind::Airport | Kind::Airfield)
                        && facility_operational(city, i))
                    .then_some(i)
                }) {
                    let edge = airport / city.width as usize * city.width as usize;
                    let route = flight_path(airport, edge);
                    if route.len() > 1 {
                        self.next_id += 1;
                        self.vehicles.push(Vehicle {
                            id: self.next_id,
                            kind: if city.tiles[airport].kind == Kind::Airfield {
                                VehicleKind::Aircraft
                            } else {
                                VehicleKind::Jet
                            },
                            tile: route[0],
                            next_tile: route[1],
                            route,
                            passengers: 120,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        city.transport_usage = (self.ridership * 100 / self.trips.max(1)).min(100) as u8;
    }
}
/// A congestion- and wear-weighted directed road route. Surface building
/// endpoints connect to their nearest street within the municipal access radius.
struct RouteSearch {
    costs: Vec<u32>,
    parents: Vec<u32>,
    seen: Vec<u32>,
    generation: u32,
    heap: BinaryHeap<Reverse<(u32, u32, usize)>>,
}
impl RouteSearch {
    fn new(n: usize) -> Self {
        Self {
            costs: vec![0; n],
            parents: vec![0; n],
            seen: vec![0; n],
            generation: 0,
            heap: BinaryHeap::new(),
        }
    }
}
pub fn road_route(city: &City, start: usize, end: usize) -> Option<Vec<u32>> {
    road_route_with(city, start, end, &mut RouteSearch::new(city.tiles.len()))
}
fn road_route_with(
    city: &City,
    start: usize,
    end: usize,
    search: &mut RouteSearch,
) -> Option<Vec<u32>> {
    fn access(city: &City, i: usize) -> Option<usize> {
        if i >= city.tiles.len() {
            return None;
        }
        if city.tiles[i].kind.is_road() {
            return Some(i);
        }
        let x = (i % city.width as usize) as i32;
        let y = (i / city.width as usize) as i32;
        (-3i32..=3)
            .flat_map(|dy| (-3i32..=3).map(move |dx| (dx, dy)))
            .filter(|(dx, dy)| dx.abs() + dy.abs() <= 3)
            .filter_map(|(dx, dy)| city.idx(x + dx, y + dy))
            .filter(|&j| city.tiles[j].kind.is_road() && city.tiles[j].connected)
            .min_by_key(|&j| (city.distance(i, j), j))
    }
    let source = access(city, start)?;
    let goal = access(city, end)?;
    search.generation += 1;
    let generation = search.generation;
    let costs = &mut search.costs;
    let parents = &mut search.parents;
    let seen = &mut search.seen;
    let heap = &mut search.heap;
    heap.clear();
    seen[source] = generation;
    costs[source] = 0;
    heap.push(Reverse((
        city.distance(source, goal) as u32 * 10,
        u32::MAX,
        source,
    )));
    // Prefer the deeper path among equal A* lower bounds. A rectangular grid
    // otherwise expands the entire equally good rectangle before the goal.
    while let Some(Reverse((_, reverse_cost, i))) = heap.pop() {
        let cost = u32::MAX - reverse_cost;
        if cost != costs[i] {
            continue;
        }
        if i == goal {
            break;
        }
        for j in city.neighbors(i) {
            if !city.tiles[j].kind.is_road() || !allowed(city, i, j) {
                continue;
            }
            let condition = city.ext.cells.get(j).map_or(100, |c| c.condition);
            let step = 10 + city.tiles[j].traffic as u32 / 5 + (100 - condition) as u32 / 5;
            let next = cost + step;
            if seen[j] != generation || next < costs[j] {
                seen[j] = generation;
                costs[j] = next;
                parents[j] = i as u32;
                heap.push(Reverse((
                    next + city.distance(j, goal) as u32 * 10,
                    u32::MAX - next,
                    j,
                )));
            }
        }
    }
    if seen[goal] != generation {
        return None;
    }
    let mut at = goal;
    let mut path = vec![at as u32];
    while at != source {
        at = parents[at] as usize;
        path.push(at as u32);
    }
    path.reverse();
    if start != source {
        path.insert(0, start as u32);
    }
    if end != goal {
        path.push(end as u32);
    }
    Some(path)
}
