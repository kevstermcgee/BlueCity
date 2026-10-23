//! Saved, daily public services. Coverage describes access; vehicles deliver care.
use crate::{
    city::{City, Kind, Terrain},
    transport::{Vehicle, VehicleKind},
    world::hash,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServicePurpose {
    #[default]
    Fire,
    Medical,
    Police,
    Waste,
    Deathcare,
    Repair,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EmergencyKind {
    #[default]
    Fire,
    Medical,
    Outbreak,
    Crime,
    Riot,
    Waste,
    Deathcare,
    Repair,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emergency {
    pub kind: EmergencyKind,
    pub tile: u32,
    pub severity: u8,
    pub opened_day: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMission {
    pub vehicle: u32,
    pub purpose: ServicePurpose,
    pub facility: u32,
    pub target: u32,
    pub returning: bool,
    pub load: u32,
    pub started_day: u64,
    #[serde(default)]
    pub cargo: Vec<[u32; 2]>,
    #[serde(default)]
    pub stops: Vec<u32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patient {
    pub home: u32,
    pub facility: u32,
    pub count: u32,
    pub discharge_day: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detention {
    pub count: u32,
    pub release_day: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityLoad {
    pub tile: u32,
    pub kind: Kind,
    /// Permanent landfill occupancy or cemetery burials.
    pub used: u32,
    pub processed_today: u32,
}
impl Default for FacilityLoad {
    fn default() -> Self {
        Self {
            tile: 0,
            kind: Kind::Empty,
            used: 0,
            processed_today: 0,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteFailure {
    pub facility: u32,
    pub target: u32,
    pub until_day: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairRequest {
    pub tile: u32,
    pub cost: i64,
    pub requested_day: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceNotice {
    pub tile: u32,
    pub day: u64,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceState {
    pub day: u64,
    pub beds: u32,
    pub occupied_beds: u32,
    pub ill: u32,
    pub deaths_waiting: u32,
    pub prisoners: u32,
    pub prison_capacity: u32,
    pub landfill_used: u64,
    pub landfill_capacity: u64,
    pub waste_collected: u64,
    pub waste_generated: u64,
    pub waste_processed: u64,
    pub recycled: u64,
    pub burials: u64,
    pub recovered: u64,
    pub overflow: u32,
    pub dispatches: u64,
    pub emergencies: Vec<Emergency>,
    pub missions: Vec<ServiceMission>,
    pub patients: Vec<Patient>,
    pub detentions: Vec<Detention>,
    pub facilities: Vec<FacilityLoad>,
    pub failed_routes: Vec<RouteFailure>,
    /// Physical stocks, indexed by the authoritative city grid.
    pub waste: Vec<u32>,
    pub sick: Vec<u32>,
    pub dead: Vec<u32>,
    pub repair_requests: Vec<RepairRequest>,
    pub completed_repairs: u64,
    pub cancelled_repairs: u64,
    pub notices: Vec<ServiceNotice>,
    #[serde(skip)]
    cache: ServiceCache,
}

fn operational(city: &City, i: usize) -> bool {
    city.tiles.get(i).is_some_and(|t| {
        t.connected
            && t.powered
            && t.watered
            && t.fire == 0
            && city.funding[t.kind.funding()] > 0
            && city.ext.cells.get(i).is_none_or(|c| {
                c.construction == 100 && !c.abandoned && !c.rubble && c.outage_days == 0
            })
    })
}
fn capacity(city: &City, i: usize) -> u32 {
    if !operational(city, i) {
        return 0;
    }
    let k = city.tiles[i].kind;
    let base = match k {
        Kind::Hospital => 128,
        Kind::Clinic => 24,
        Kind::Prison => 200,
        Kind::Landfill => 12000,
        Kind::Incinerator => 320,
        Kind::Recycling => 192,
        Kind::Cemetery => 2000,
        _ => 0,
    };
    base * city.funding[k.funding()].min(150) as u32 / 100
}
fn provider_group(kind: Kind) -> Option<usize> {
    match kind {
        Kind::Fire | Kind::FireHQ => Some(0),
        Kind::Hospital | Kind::Clinic => Some(1),
        Kind::Police => Some(2),
        Kind::Landfill | Kind::Incinerator | Kind::Recycling => Some(3),
        Kind::Cemetery => Some(4),
        Kind::MaintenanceDepot => Some(5),
        Kind::Prison => Some(6),
        _ => None,
    }
}
fn serves(kind: Kind, purpose: ServicePurpose) -> bool {
    provider_group(kind) == Some(purpose as usize)
}

fn allowed(city: &City, a: usize, b: usize) -> bool {
    if city.tiles[a].kind != Kind::OneWay {
        return true;
    }
    let d = city.ext.cells.get(a).map_or(0, |c| c.orientation) % 4;
    let (dx, dy) = [(1, 0), (0, 1), (-1, 0), (0, -1)][d as usize];
    city.idx(
        (a % city.width as usize) as i32 + dx,
        (a / city.width as usize) as i32 + dy,
    ) == Some(b)
}
/// Curbside crews can walk two lots or run a hose across two dry cells.
fn walk_access(city: &City, road: usize, address: usize) -> bool {
    let d = city.distance(road, address);
    if d > 2 {
        return false;
    }
    if d <= 1 {
        return true;
    }
    city.neighbors(road).any(|j| {
        city.distance(j, address) == 1
            && (city.tiles[j].terrain != Terrain::Water || city.tiles[j].kind.is_road())
    })
}
#[derive(Clone, Debug)]
struct RoutingTree {
    parent: Vec<u32>,
    root: Vec<u32>,
    distance: Vec<u32>,
}
#[derive(Clone, Debug, Default)]
struct StreetGraph {
    outgoing: Vec<[u16; 4]>,
    incoming: Vec<[u16; 4]>,
    curbs: Vec<[u16; 13]>,
    addresses: Vec<[u16; 13]>,
}
#[derive(Clone, Debug)]
struct CachedTree {
    providers: Vec<u32>,
    tree: RoutingTree,
}
#[derive(Clone, Debug)]
struct RouteAddresses {
    route: Arc<[u32]>,
    walk: Arc<[u16]>,
    sorted: Arc<[u16]>,
}
#[derive(Clone, Debug)]
struct RouteAddressesEntry {
    data: Arc<RouteAddresses>,
    used: u64,
}
/// Pure derived memoization, never serialized or part of authoritative equality.
#[derive(Clone, Debug, Default)]
struct ServiceCache {
    key: u64,
    graph: Arc<StreetGraph>,
    trees: BTreeMap<(u8, bool), Arc<CachedTree>>,
    route_addresses: BTreeMap<u64, RouteAddressesEntry>,
    route_ages: BTreeSet<(u64, u64)>,
    route_use: u64,
}
impl PartialEq for ServiceCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ServiceCache {}
/// A day's exact destination reservations. Pending arrivals are deliberately
/// absent until they are retained, matching the order in which crews receive
/// capacity in `arrivals`; this is derived scratch, never saved authority.
#[derive(Default)]
struct ResourceLoad {
    reserved: [u32; 6],
    used: u32,
    processed_today: u32,
}
#[derive(Default)]
struct ResourceLedger(BTreeMap<u32, ResourceLoad>);
impl ResourceLedger {
    fn new(state: &ServiceState) -> Self {
        let mut ledger = Self::default();
        for f in &state.facilities {
            let load = ledger.0.entry(f.tile).or_default();
            load.used = f.used;
            load.processed_today = f.processed_today;
        }
        for p in &state.patients {
            ledger.reserve(p.facility, ServicePurpose::Medical, p.count);
        }
        for m in &state.missions {
            ledger.reserve(m.facility, m.purpose, m.load);
        }
        ledger
    }
    fn reserve(&mut self, facility: u32, purpose: ServicePurpose, count: u32) {
        if count > 0 {
            self.0.entry(facility).or_default().reserved[purpose as usize] += count;
        }
    }
    fn available(&self, city: &City, facility: u32, purpose: ServicePurpose) -> u32 {
        let i = facility as usize;
        let load = self.0.get(&facility);
        let used = match city.tiles.get(i).map(|t| t.kind) {
            Some(Kind::Landfill | Kind::Cemetery) => load.map_or(0, |f| f.used),
            Some(Kind::Incinerator | Kind::Recycling) => load.map_or(0, |f| f.processed_today),
            _ => 0,
        };
        capacity(city, i)
            .saturating_sub(used)
            .saturating_sub(load.map_or(0, |f| f.reserved[purpose as usize]))
    }
}
#[derive(Default)]
struct Routing {
    memo: ServiceCache,
    resources: ResourceLedger,
    touched: BTreeSet<(u8, bool)>,
    prepared: bool,
    marks: Vec<u16>,
    stamp: u16,
    providers: [Vec<u32>; 7],
}
impl Routing {
    fn valid_route(&mut self, city: &City, route: &[u32]) -> bool {
        if route.len() < 2 || route.iter().any(|&i| i as usize >= city.tiles.len()) {
            return false;
        }
        self.prepare(city);
        route.windows(2).enumerate().all(|(k, p)| {
            self.memo.graph.outgoing[p[0] as usize].contains(&(p[1] as u16))
                || (k + 2 == route.len()
                    && self.memo.graph.incoming[p[1] as usize].contains(&(p[0] as u16)))
        })
    }
    fn prepare(&mut self, city: &City) {
        if self.prepared {
            return;
        }
        let mut providers: [Vec<u32>; 7] = Default::default();
        let mut key = (u64::from(city.width) << 32) | u64::from(city.height);
        for (i, t) in city.tiles.iter().enumerate() {
            if let Some(group) = provider_group(t.kind) {
                providers[group].push(i as u32);
            }
            let mask = u64::from(t.kind.is_road())
                | u64::from(t.terrain == Terrain::Water) << 1
                | if t.kind == Kind::OneWay {
                    u64::from(city.ext.cells.get(i).map_or(0, |c| c.orientation) % 4 + 1) << 2
                } else {
                    0
                };
            key = (key ^ mask).wrapping_mul(0x100000001b3);
        }
        if providers != self.providers {
            self.providers = providers;
            self.touched.clear();
        }
        if key != self.memo.key || self.memo.graph.outgoing.len() != city.tiles.len() {
            let n = city.tiles.len();
            let mut graph = StreetGraph {
                outgoing: vec![[u16::MAX; 4]; n],
                incoming: vec![[u16::MAX; 4]; n],
                curbs: vec![[u16::MAX; 13]; n],
                addresses: vec![[u16::MAX; 13]; n],
            };
            let mut curb_counts = vec![0usize; n];
            let width = city.width as usize;
            for i in 0..n {
                for (slot, j) in city.neighbors(i).enumerate() {
                    if city.tiles[j].kind.is_road() {
                        if allowed(city, i, j) {
                            graph.outgoing[i][slot] = j as u16;
                        }
                        if allowed(city, j, i) {
                            graph.incoming[i][slot] = j as u16;
                        }
                    }
                }
                if !city.tiles[i].kind.is_road() {
                    continue;
                }
                let (x, y) = ((i % width) as i32, (i / width) as i32);
                let dry = |j: usize| {
                    city.tiles[j].terrain != Terrain::Water || city.tiles[j].kind.is_road()
                };
                let mut a = 0;
                for dy in -2i32..=2 {
                    for dx in -2i32..=2 {
                        if dx.abs() + dy.abs() <= 2 {
                            if let Some(j) = city.idx(x + dx, y + dy) {
                                let accessible = if dx.abs() + dy.abs() < 2 {
                                    true
                                } else if dx == 0 {
                                    dry((i as isize + dy as isize / 2 * width as isize) as usize)
                                } else if dy == 0 {
                                    dry((i as isize + dx as isize / 2) as usize)
                                } else {
                                    dry((i as isize + dx as isize) as usize)
                                        || dry((i as isize + dy as isize * width as isize) as usize)
                                };
                                if accessible {
                                    graph.addresses[i][a] = j as u16;
                                    a += 1;
                                    graph.curbs[j][curb_counts[j]] = i as u16;
                                    curb_counts[j] += 1;
                                }
                            }
                        }
                    }
                }
            }
            self.memo.key = key;
            self.memo.graph = Arc::new(graph);
            self.memo.trees.clear();
            self.memo.route_addresses.clear();
            self.memo.route_ages.clear();
            self.touched.clear();
        }
        self.prepared = true;
    }
    fn visit_addresses(
        &mut self,
        city: &City,
        route: &[u32],
        mut visit: impl FnMut(usize) -> bool,
    ) {
        self.prepare(city);
        if let Some(addresses) = self.cached_addresses(route, false) {
            for &address in addresses.iter() {
                if !visit(usize::from(address)) {
                    break;
                }
            }
            return;
        }
        self.marks.resize(city.tiles.len(), 0);
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            self.marks.fill(0);
            self.stamp = 1;
        }
        for &road in route {
            for &j in &self.memo.graph.addresses[road as usize] {
                if j == u16::MAX {
                    break;
                }
                let j = j as usize;
                if self.marks[j] != self.stamp {
                    self.marks[j] = self.stamp;
                    if !visit(j) {
                        return;
                    }
                }
            }
        }
    }
    fn route_key(route: &[u32]) -> u64 {
        route.iter().fold(route.len() as u64, |key, &tile| {
            (key ^ u64::from(tile)).wrapping_mul(0x100000001b3)
        })
    }
    fn cached_addresses(&mut self, route: &[u32], sorted: bool) -> Option<Arc<[u16]>> {
        self.memo.route_use = self.memo.route_use.wrapping_add(1);
        let key = Self::route_key(route);
        let entry = self.memo.route_addresses.get_mut(&key)?;
        if entry.data.route.as_ref() != route {
            return None;
        }
        self.memo.route_ages.remove(&(entry.used, key));
        entry.used = self.memo.route_use;
        self.memo.route_ages.insert((entry.used, key));
        Some(if sorted {
            entry.data.sorted.clone()
        } else {
            entry.data.walk.clone()
        })
    }
    fn addresses(&mut self, city: &City, route: &[u32]) -> Arc<[u16]> {
        self.prepare(city);
        if let Some(addresses) = self.cached_addresses(route, true) {
            return addresses;
        }
        let mut addresses = Vec::with_capacity(route.len() * 5);
        self.visit_addresses(city, route, |j| {
            addresses.push(j as u16);
            true
        });
        let walk = Arc::from(addresses.as_slice());
        addresses.sort_unstable();
        let sorted: Arc<[u16]> = addresses.into();
        if self.memo.route_addresses.len() >= 512 {
            if let Some(&(used, oldest)) = self.memo.route_ages.first() {
                self.memo.route_ages.remove(&(used, oldest));
                self.memo.route_addresses.remove(&oldest);
            }
        }
        let key = Self::route_key(route);
        let used = self.memo.route_use;
        if let Some(old) = self.memo.route_addresses.insert(
            key,
            RouteAddressesEntry {
                data: Arc::new(RouteAddresses {
                    route: Arc::from(route),
                    walk,
                    sorted: sorted.clone(),
                }),
                used,
            },
        ) {
            self.memo.route_ages.remove(&(old.used, key));
        }
        self.memo.route_ages.insert((used, key));
        sorted
    }
    fn route(
        &mut self,
        city: &City,
        purpose: ServicePurpose,
        target: usize,
        reverse: bool,
        fleet_room: Option<&[u16]>,
    ) -> Option<(u32, Vec<u32>)> {
        self.prepare(city);
        let key = (purpose as u8, reverse);
        if self.touched.insert(key) {
            let group = if reverse && purpose == ServicePurpose::Police {
                6
            } else {
                purpose as usize
            };
            let providers: Vec<_> = self.providers[group]
                .iter()
                .copied()
                .filter(|&i| {
                    operational(city, i as usize)
                        && (matches!(
                            purpose,
                            ServicePurpose::Fire | ServicePurpose::Police | ServicePurpose::Repair
                        ) || self.resources.available(city, i, purpose) > 0)
                })
                .collect();
            if self
                .memo
                .trees
                .get(&key)
                .is_none_or(|cached| cached.providers != providers)
            {
                let n = city.tiles.len();
                let mut tree = RoutingTree {
                    parent: vec![u32::MAX; n],
                    root: vec![u32::MAX; n],
                    distance: vec![u32::MAX; n],
                };
                let mut queue = VecDeque::new();
                for &i in &providers {
                    let j = i as usize;
                    tree.parent[j] = i;
                    tree.root[j] = i;
                    tree.distance[j] = 0;
                    queue.push_back(j);
                }
                let edges = if reverse {
                    &self.memo.graph.incoming
                } else {
                    &self.memo.graph.outgoing
                };
                while let Some(i) = queue.pop_front() {
                    for &j in &edges[i] {
                        if j == u16::MAX {
                            continue;
                        }
                        let j = j as usize;
                        if tree.parent[j] != u32::MAX {
                            continue;
                        }
                        tree.parent[j] = i as u32;
                        tree.root[j] = tree.root[i];
                        tree.distance[j] = tree.distance[i] + 1;
                        queue.push_back(j);
                    }
                }
                self.memo
                    .trees
                    .insert(key, Arc::new(CachedTree { providers, tree }));
            }
        }
        let tree = &self.memo.trees[&key].tree;
        let road = self.memo.graph.curbs[target]
            .iter()
            .copied()
            .take_while(|&j| j != u16::MAX)
            .map(usize::from)
            .filter(|&j| {
                tree.root[j] != u32::MAX
                    && fleet_room.is_none_or(|room| room[tree.root[j] as usize] > 0)
            })
            .min_by_key(|&j| (tree.distance[j], j))?;
        let facility = tree.root[road];
        let mut route = vec![road as u32];
        let mut i = road;
        while tree.parent[i] != i as u32 {
            i = tree.parent[i] as usize;
            route.push(i as u32);
        }
        if !reverse {
            route.reverse();
        }
        Some((facility, route))
    }
}
fn fleet(city: &City, i: usize) -> usize {
    let base = match city.tiles[i].kind {
        Kind::Hospital | Kind::FireHQ => 12,
        Kind::Landfill | Kind::Incinerator | Kind::Recycling => 16,
        Kind::Police => 8,
        _ => 4,
    };
    (base * city.funding[city.tiles[i].kind.funding()] as usize / 100).max(1)
}

impl ServiceState {
    /// Copy a checkpoint exactly while retaining existing stock and crew buffers.
    pub fn checkpoint_copy_from(&mut self, source: &Self) {
        let Self {
            day,
            beds,
            occupied_beds,
            ill,
            deaths_waiting,
            prisoners,
            prison_capacity,
            landfill_used,
            landfill_capacity,
            waste_collected,
            waste_generated,
            waste_processed,
            recycled,
            burials,
            recovered,
            overflow,
            dispatches,
            emergencies,
            missions,
            patients,
            detentions,
            facilities,
            failed_routes,
            waste,
            sick,
            dead,
            repair_requests,
            completed_repairs,
            cancelled_repairs,
            notices,
            cache,
        } = source;
        self.day = *day;
        self.beds = *beds;
        self.occupied_beds = *occupied_beds;
        self.ill = *ill;
        self.deaths_waiting = *deaths_waiting;
        self.prisoners = *prisoners;
        self.prison_capacity = *prison_capacity;
        self.landfill_used = *landfill_used;
        self.landfill_capacity = *landfill_capacity;
        self.waste_collected = *waste_collected;
        self.waste_generated = *waste_generated;
        self.waste_processed = *waste_processed;
        self.recycled = *recycled;
        self.burials = *burials;
        self.recovered = *recovered;
        self.overflow = *overflow;
        self.dispatches = *dispatches;
        self.completed_repairs = *completed_repairs;
        self.cancelled_repairs = *cancelled_repairs;
        self.emergencies.clone_from(emergencies);
        self.patients.clone_from(patients);
        self.detentions.clone_from(detentions);
        self.facilities.clone_from(facilities);
        self.failed_routes.clone_from(failed_routes);
        self.waste.clone_from(waste);
        self.sick.clone_from(sick);
        self.dead.clone_from(dead);
        self.repair_requests.clone_from(repair_requests);
        crate::world::checkpoint_vec_copy(&mut self.missions, missions, |target, source| {
            let ServiceMission {
                vehicle,
                purpose,
                facility,
                target: destination,
                returning,
                load,
                started_day,
                cargo,
                stops,
            } = source;
            target.vehicle = *vehicle;
            target.purpose = *purpose;
            target.facility = *facility;
            target.target = *destination;
            target.returning = *returning;
            target.load = *load;
            target.started_day = *started_day;
            target.cargo.clone_from(cargo);
            target.stops.clone_from(stops);
        });
        crate::world::checkpoint_vec_copy(&mut self.notices, notices, |target, source| {
            let ServiceNotice { tile, day, message } = source;
            target.tile = *tile;
            target.day = *day;
            target.message.clone_from(message);
        });
        self.cache.clone_from(cache);
    }

    /// Prepare derived street geometry and every current provider catchment while
    /// loading a city. Authoritative stocks, vehicles and missions are unchanged.
    pub fn warm_routes(&mut self, city: &City) {
        if !city.ext.enabled {
            return;
        }
        let mut routes = Routing {
            memo: std::mem::take(&mut self.cache),
            resources: ResourceLedger::new(self),
            ..Default::default()
        };
        for purpose in [
            ServicePurpose::Fire,
            ServicePurpose::Medical,
            ServicePurpose::Police,
            ServicePurpose::Waste,
            ServicePurpose::Deathcare,
            ServicePurpose::Repair,
        ] {
            let _ = routes.route(city, purpose, 0, false, None);
            if matches!(
                purpose,
                ServicePurpose::Medical
                    | ServicePurpose::Police
                    | ServicePurpose::Waste
                    | ServicePurpose::Deathcare
            ) {
                let _ = routes.route(city, purpose, 0, true, None);
            }
        }
        self.cache = routes.memo;
    }
    fn notice(&mut self, tile: u32, message: String) {
        self.notices.push(ServiceNotice {
            tile,
            day: self.day,
            message,
        });
        if self.notices.len() > 32 {
            self.notices.remove(0);
        }
    }
    /// A request authorizes the quoted work; payment happens only when a crew arrives.
    pub fn queue_repair(&mut self, city: &City, tile: u32) -> Result<(), String> {
        if self.repair_requests.len() >= 128 {
            return Err("The maintenance work queue is full.".into());
        }
        if self.repair_requests.iter().any(|r| r.tile == tile) {
            return Err("A repair crew is already requested for this lot.".into());
        }
        let damage = city
            .civic
            .damage
            .iter()
            .find(|d| d.tile == tile)
            .ok_or("No repairable damage is registered on that lot.")?;
        let t = city.tiles.get(tile as usize).ok_or("Outside the map.")?;
        if t.kind != Kind::Empty && t.kind != damage.previous.kind {
            return Err("A new structure occupies this lot.".into());
        }
        let reserved = self.repair_requests.iter().map(|r| r.cost).sum::<i64>();
        if city.money - reserved < damage.repair_cost {
            return Err("The treasury cannot cover this repair and already requested work.".into());
        }
        if Routing::default()
            .route(city, ServicePurpose::Repair, tile as usize, false, None)
            .is_none()
        {
            return Err("A working maintenance depot needs a street route to this lot.".into());
        }
        self.repair_requests.push(RepairRequest {
            tile,
            cost: damage.repair_cost,
            requested_day: city.ext.day,
        });
        Ok(())
    }
    pub fn cancel_repair(&mut self, city: &mut City, tile: u32) -> Result<(), String> {
        let before = self.repair_requests.len();
        self.repair_requests.retain(|r| r.tile != tile);
        if before == self.repair_requests.len() {
            return Err("No repair is queued for this lot.".into());
        }
        let ids: Vec<_> = self
            .missions
            .iter()
            .filter(|m| m.purpose == ServicePurpose::Repair && m.target == tile)
            .map(|m| m.vehicle)
            .collect();
        self.missions.retain(|m| !ids.contains(&m.vehicle));
        city.ext.transport.vehicles.retain(|v| !ids.contains(&v.id));
        self.cancelled_repairs += 1;
        self.notice(tile, "Repair cancelled. No payment was taken.".into());
        Ok(())
    }
    fn spawn_vehicle(&mut self, city: &mut City, route: Vec<u32>, load: u32) -> Option<u32> {
        if route.len() < 2 || city.ext.transport.vehicles.len() >= 4096 {
            return None;
        }
        let tr = &mut city.ext.transport;
        tr.next_id = tr.next_id.saturating_add(1);
        let id = tr.next_id;
        tr.vehicles.push(Vehicle {
            id,
            kind: VehicleKind::Service,
            tile: route[0],
            next_tile: route[1],
            route,
            passengers: load.min(u16::MAX as u32) as u16,
            ..Default::default()
        });
        Some(id)
    }
    fn restore_cargo(&mut self, m: &ServiceMission) {
        if !m.cargo.is_empty() {
            let stocks = match m.purpose {
                ServicePurpose::Waste => &mut self.waste,
                ServicePurpose::Medical => &mut self.sick,
                ServicePurpose::Deathcare => &mut self.dead,
                _ => return,
            };
            for &[tile, amount] in &m.cargo {
                if let Some(stock) = stocks.get_mut(tile as usize) {
                    *stock = stock.saturating_add(amount);
                }
            }
            return;
        }
        let values = match m.purpose {
            ServicePurpose::Waste => &mut self.waste,
            ServicePurpose::Medical => &mut self.sick,
            ServicePurpose::Deathcare => &mut self.dead,
            _ => return,
        };
        if let Some(v) = values.get_mut(m.target as usize) {
            *v = v.saturating_add(m.load);
        }
    }
    fn collect_route(
        &mut self,
        city: &City,
        m: &mut ServiceMission,
        route: &[u32],
        limit: u32,
        routes: &mut Routing,
    ) {
        if m.load >= limit {
            return;
        }
        let stocks = match m.purpose {
            ServicePurpose::Waste => &mut self.waste,
            ServicePurpose::Medical => &mut self.sick,
            ServicePurpose::Deathcare => &mut self.dead,
            _ => return,
        };
        routes.visit_addresses(city, route, |address| {
            let amount = stocks[address].min(limit.saturating_sub(m.load));
            if amount > 0 {
                stocks[address] -= amount;
                m.load += amount;
                m.cargo.push([address as u32, amount]);
                if m.purpose == ServicePurpose::Waste {
                    self.waste_collected += amount as u64;
                }
            }
            m.load < limit
        });
    }
    fn arrivals(&mut self, city: &mut City, routes: &mut Routing) {
        let mut pending = std::mem::take(&mut self.missions);
        routes.resources = ResourceLedger::new(self);
        pending.sort_by_key(|m| !m.returning);
        let vehicles: BTreeMap<_, _> = city
            .ext
            .transport
            .vehicles
            .iter()
            .enumerate()
            .filter(|(_, v)| v.kind == VehicleKind::Service)
            .map(|(i, v)| (v.id, i))
            .collect();
        let mut removed = BTreeSet::new();
        for mut m in pending {
            let vehicle = vehicles
                .get(&m.vehicle)
                .and_then(|&i| city.ext.transport.vehicles.get(i));
            let eligible = operational(city, m.facility as usize)
                && city
                    .tiles
                    .get(m.facility as usize)
                    .is_some_and(|t| serves(t.kind, m.purpose));
            let valid = vehicle.is_some_and(|v| routes.valid_route(city, &v.route));
            if !eligible || !valid || self.day.saturating_sub(m.started_day) > 90 {
                self.restore_cargo(&m);
                removed.insert(m.vehicle);
                continue;
            }
            let vehicle = vehicle.unwrap();
            let goal = *vehicle.route.last().unwrap();
            if !m.returning && !walk_access(city, goal as usize, m.target as usize) {
                removed.insert(m.vehicle);
                continue;
            }
            if vehicle.age_days < 90 || vehicle.tile != goal {
                routes.resources.reserve(m.facility, m.purpose, m.load);
                self.missions.push(m);
                continue;
            }
            let traveled = vehicle.route.clone();
            removed.insert(m.vehicle);
            if m.returning {
                if matches!(
                    m.purpose,
                    ServicePurpose::Waste | ServicePurpose::Medical | ServicePurpose::Deathcare
                ) {
                    let size = match m.purpose {
                        ServicePurpose::Waste => 1024,
                        ServicePurpose::Medical => 16,
                        _ => 8,
                    };
                    let limit = routes
                        .resources
                        .available(city, m.facility, m.purpose)
                        .min(size);
                    if limit < m.load {
                        self.restore_cargo(&m);
                        continue;
                    }
                    self.collect_route(city, &mut m, &traveled, limit, routes);
                }
                match m.purpose {
                    ServicePurpose::Medical => {
                        // The ambulance has delivered its patients to working beds.
                        if m.cargo.is_empty() {
                            m.cargo.push([m.target, m.load]);
                        }
                        for &[home, count] in &m.cargo {
                            routes
                                .resources
                                .reserve(m.facility, ServicePurpose::Medical, count);
                            self.patients.push(Patient {
                                home,
                                facility: m.facility,
                                count,
                                discharge_day: self.day + 3,
                            });
                        }
                    }
                    ServicePurpose::Waste | ServicePurpose::Deathcare => {
                        if let Some(f) = self.facilities.iter_mut().find(|f| f.tile == m.facility) {
                            match f.kind {
                                Kind::Landfill | Kind::Cemetery => {
                                    f.used = f.used.saturating_add(m.load)
                                }
                                _ => f.processed_today = f.processed_today.saturating_add(m.load),
                            }
                            let load = routes.resources.0.entry(f.tile).or_default();
                            load.used = f.used;
                            load.processed_today = f.processed_today;
                            if m.purpose == ServicePurpose::Deathcare {
                                self.burials += m.load as u64;
                            } else {
                                self.waste_processed += m.load as u64;
                                if f.kind == Kind::Recycling {
                                    self.recycled += m.load as u64;
                                }
                            }
                        } else {
                            self.restore_cargo(&m);
                        }
                    }
                    _ => {}
                }
                continue;
            }
            let target = m.target as usize;
            match m.purpose {
                ServicePurpose::Fire => {
                    city.tiles[target].fire = city.tiles[target].fire.saturating_sub(85);
                    continue;
                }
                ServicePurpose::Police => {
                    let room = if routes
                        .route(city, ServicePurpose::Police, target, true, None)
                        .is_some()
                    {
                        self.prison_capacity.saturating_sub(self.prisoners)
                    } else {
                        0
                    };
                    let arrests = room.min(1 + city.tiles[target].crime as u32 / 20);
                    city.tiles[target].crime = city.tiles[target]
                        .crime
                        .saturating_sub(if arrests > 0 { 45 } else { 8 });
                    if arrests > 0 {
                        self.detentions.push(Detention {
                            count: arrests,
                            release_day: self.day + 30,
                        });
                        self.prisoners += arrests;
                    }
                    continue;
                }
                ServicePurpose::Repair => {
                    let Some(request) = self
                        .repair_requests
                        .iter()
                        .find(|r| r.tile == m.target)
                        .cloned()
                    else {
                        continue;
                    };
                    self.repair_requests.retain(|r| r.tile != m.target);
                    let quoted = city
                        .civic
                        .damage
                        .iter()
                        .find(|d| d.tile == m.target)
                        .is_some_and(|d| d.repair_cost == request.cost);
                    let result = if !quoted {
                        Err("The damage changed before the crew arrived.".into())
                    } else {
                        let mut civic = std::mem::take(&mut city.civic);
                        let result = civic.repair(city, m.target);
                        city.civic = civic;
                        result
                    };
                    match result {
                        Ok(()) => {
                            routes.prepared = false;
                            self.completed_repairs += 1;
                            self.notice(m.target, "Crew completed the requested repair.".into());
                        }
                        Err(e) => {
                            self.cancelled_repairs += 1;
                            self.notice(m.target, e);
                        }
                    }
                    continue;
                }
                purpose => {
                    let Some((destination, route)) =
                        routes.route(city, purpose, target, true, None)
                    else {
                        continue;
                    };
                    m.facility = destination;
                    let available = routes.resources.available(city, m.facility, purpose);
                    let size = match purpose {
                        ServicePurpose::Waste => 1024,
                        ServicePurpose::Medical => 16,
                        _ => 8,
                    };
                    self.collect_route(city, &mut m, &traveled, available.min(size), routes);
                    if m.load == 0 {
                        continue;
                    }
                    if let Some(id) = self.spawn_vehicle(city, route, m.load) {
                        m.returning = true;
                        m.vehicle = id;
                        routes.resources.reserve(m.facility, m.purpose, m.load);
                        self.missions.push(m);
                    } else {
                        self.restore_cargo(&m);
                    }
                }
            }
        }
        city.ext
            .transport
            .vehicles
            .retain(|v| !removed.contains(&v.id));
    }
    fn facilities(&mut self, city: &City) {
        self.beds = 0;
        self.prison_capacity = 0;
        self.landfill_capacity = 0;
        self.facilities.retain(|f| {
            city.tiles
                .get(f.tile as usize)
                .is_some_and(|t| t.kind == f.kind)
        });
        self.facilities.sort_unstable_by_key(|f| f.tile);
        for f in &mut self.facilities {
            f.processed_today = 0;
        }
        for (i, t) in city.tiles.iter().enumerate() {
            match t.kind {
                Kind::Hospital | Kind::Clinic => self.beds += capacity(city, i),
                Kind::Prison => self.prison_capacity += capacity(city, i),
                Kind::Landfill => self.landfill_capacity += capacity(city, i) as u64,
                _ => {}
            }
            if matches!(
                t.kind,
                Kind::Landfill | Kind::Incinerator | Kind::Recycling | Kind::Cemetery
            ) {
                if let Err(at) = self
                    .facilities
                    .binary_search_by_key(&(i as u32), |f| f.tile)
                {
                    self.facilities.insert(
                        at,
                        FacilityLoad {
                            tile: i as u32,
                            kind: t.kind,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
    fn disease_and_waste(&mut self, city: &mut City) {
        let old_sick = self.sick.clone();
        for i in 0..city.tiles.len() {
            let t = city.tiles[i];
            let count = t.population;
            let occupants = count + t.jobs / 2;
            if occupants == 0 && self.waste[i] == 0 && self.sick[i] == 0 {
                if let Some(cell) = city.ext.cells.get_mut(i) {
                    if cell.garbage != 0 {
                        cell.garbage = 0;
                    }
                }
                continue;
            }
            if occupants > 0 {
                let amount = occupants.div_ceil(if city.policies[2] { 32 } else { 24 });
                self.waste[i] = self.waste[i].saturating_add(amount);
                self.waste_generated += amount as u64;
            }
            let garbage = self.waste[i].saturating_mul(100) / (occupants.max(8) * 3);
            if let Some(c) = city.ext.cells.get_mut(i) {
                c.garbage = garbage.min(100) as u8;
            }
            if count == 0 {
                self.sick[i] = 0;
                continue;
            }
            self.sick[i] = self.sick[i].min(count);
            let environmental = city.ext.cells.get(i).map_or(0, |c| {
                c.air_pollution as u32 / 12 + c.flood as u32 / 8 + c.radiation as u32 / 5
            });
            let nearby = city
                .neighbors(i)
                .any(|j| old_sick[j] > city.tiles[j].population / 4 && old_sick[j] > 0);
            let risk = 2
                + t.pollution as u32 / 10
                + garbage.min(100) / 8
                + environmental
                + u32::from(!t.watered) * 12
                + u32::from(nearby) * 8
                + self.dead[i].min(10) * 2;
            let r = hash(
                city.ext.seed,
                self.day.wrapping_mul(65537).wrapping_add(i as u64),
            );
            let infections = count as u64 * risk as u64 / 2000
                + u64::from(r % 2000 < count as u64 * risk as u64 % 2000);
            self.sick[i] = self.sick[i].saturating_add(infections as u32).min(count);
            if self.day.is_multiple_of(10) && self.sick[i] > 0 {
                self.sick[i] -= 1;
                self.recovered += 1;
            }
            if self.sick[i] > count / 3 && self.sick[i] >= 4 && r.is_multiple_of(7) {
                let deaths = (self.sick[i] / 50).max(1).min(count);
                self.sick[i] -= deaths;
                self.dead[i] = self.dead[i].saturating_add(deaths);
                city.tiles[i].population -= deaths;
                city.population = city.population.saturating_sub(deaths);
                city.ext.people.deaths += deaths as u64;
                city.ext.people.population = city.ext.people.population.saturating_sub(deaths);
                if let Some(h) = city
                    .ext
                    .people
                    .households
                    .iter_mut()
                    .find(|h| h.home == i as u32)
                {
                    h.size = h.size.saturating_sub(deaths.min(u16::MAX as u32) as u16);
                }
            }
        }
    }
    fn fires(&mut self, city: &mut City) -> bool {
        let mut routing_changed = false;
        let burning: Vec<_> = city
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.fire > 0)
            .map(|(i, t)| (i, t.fire))
            .collect();
        let mut ignite = vec![];
        let rain = city.ext.weather.rain;
        for (i, intensity) in burning {
            let x = (i % city.width as usize) as i32;
            let y = (i / city.width as usize) as i32;
            if intensity >= 55 && rain < 55 {
                for j in city.neighbors(i) {
                    let t = city.tiles[j];
                    let combustible = t.kind.is_building()
                        || (t.kind == Kind::Empty && t.terrain == Terrain::Forest);
                    let dry = city.ext.cells.get(j).is_none_or(|c| c.moisture < 65);
                    let downwind = (j % city.width as usize) as i32 - x
                        == city.ext.weather.wind_x as i32
                        && (j / city.width as usize) as i32 - y == city.ext.weather.wind_y as i32;
                    if t.fire == 0
                        && combustible
                        && dry
                        && (downwind
                            || hash(city.ext.seed, self.day + i as u64 + j as u64)
                                .is_multiple_of(23))
                    {
                        ignite.push(j);
                    }
                }
            }
            city.tiles[i].fire = if rain > 60 {
                intensity.saturating_sub(4)
            } else {
                intensity.saturating_add(3).min(100)
            };
            if city.tiles[i].fire >= 100 {
                let routing_before = (
                    city.tiles[i].kind,
                    city.tiles[i].terrain,
                    city.ext.cells.get(i).map(|c| c.orientation),
                );
                let population = city.tiles[i].population;
                city.ext.people.migrants_out = city
                    .ext
                    .people
                    .migrants_out
                    .saturating_add(population as u64);
                city.ext.people.population = city.ext.people.population.saturating_sub(population);
                if let Some(h) = city
                    .ext
                    .people
                    .households
                    .iter_mut()
                    .find(|h| h.home == i as u32)
                {
                    h.size = 0;
                }
                city.population = city.population.saturating_sub(population);
                city.tiles[i].kind = Kind::Empty;
                city.tiles[i].level = 0;
                city.tiles[i].population = 0;
                city.tiles[i].jobs = 0;
                city.tiles[i].fire = 0;
                if city.tiles[i].terrain == Terrain::Forest {
                    city.tiles[i].terrain = Terrain::Grass;
                }
                if let Some(c) = city.ext.cells.get_mut(i) {
                    c.rubble = true;
                    c.condition = 0;
                    c.tree_age = 0;
                }
                routing_changed |= routing_before
                    != (
                        city.tiles[i].kind,
                        city.tiles[i].terrain,
                        city.ext.cells.get(i).map(|c| c.orientation),
                    );
            }
        }
        for i in ignite {
            city.tiles[i].fire = 35;
        }
        // Individual industrial ignition risk, not a coverage-based extinguishing shortcut.
        if self.day.is_multiple_of(7) {
            for (i, t) in city.tiles.iter_mut().enumerate() {
                if matches!(t.kind, Kind::Industrial | Kind::Coal | Kind::Incinerator)
                    && t.fire == 0
                    && t.pollution > 65
                    && hash(city.ext.seed, self.day + i as u64).is_multiple_of(1700)
                {
                    t.fire = 25;
                }
            }
        }
        routing_changed
    }
    fn emergencies(&mut self, city: &mut City) -> bool {
        let n = city.tiles.len();
        let mut previous = vec![0u64; n * 8];
        for event in std::mem::take(&mut self.emergencies) {
            previous[event.kind as usize * n + event.tile as usize] = event.opened_day + 1;
        }
        for i in 0..city.tiles.len() {
            let t = city.tiles[i];
            if t.population > 0
                && (city.unemployment > 15 || self.waste[i] > 100)
                && self.day.is_multiple_of(3)
            {
                city.tiles[i].crime = t.crime.saturating_add(1).min(100);
            }
            let t = city.tiles[i];
            let mut add = |kind, severity| {
                let age = previous[kind as usize * n + i];
                let opened_day = if age == 0 { self.day } else { age - 1 };
                self.emergencies.push(Emergency {
                    kind,
                    tile: i as u32,
                    severity,
                    opened_day,
                });
            };
            if t.fire > 0 {
                add(EmergencyKind::Fire, t.fire);
            }
            if self.sick[i] > 0 {
                let severity = (self.sick[i] * 100 / t.population.max(1)).min(100) as u8;
                add(
                    if severity >= 25 {
                        EmergencyKind::Outbreak
                    } else {
                        EmergencyKind::Medical
                    },
                    severity,
                );
            }
            if t.population + t.jobs > 0 && t.crime >= 55 {
                let kind = if t.crime >= 90 {
                    EmergencyKind::Riot
                } else {
                    EmergencyKind::Crime
                };
                add(kind, t.crime);
                if kind == EmergencyKind::Riot && self.day.is_multiple_of(7) {
                    city.happiness = city.happiness.saturating_sub(1);
                    if let Some(c) = city.ext.cells.get_mut(i) {
                        c.condition = c.condition.saturating_sub(3);
                    }
                }
            }
            if t.kind == Kind::Prison && self.prisoners > self.prison_capacity {
                add(EmergencyKind::Riot, 95);
                city.happiness = city.happiness.saturating_sub(1);
            }
            if self.dead[i] > 0 {
                add(EmergencyKind::Deathcare, 100);
            }
            if self.waste[i] >= 8 {
                add(EmergencyKind::Waste, (self.waste[i] / 2).min(100) as u8);
            }
        }
        for r in &self.repair_requests {
            self.emergencies.push(Emergency {
                kind: EmergencyKind::Repair,
                tile: r.tile,
                severity: 90,
                opened_day: r.requested_day,
            });
        }
        self.emergencies.sort_unstable_by_key(|e| {
            let priority = match e.kind {
                EmergencyKind::Fire => 0,
                EmergencyKind::Riot => 1,
                EmergencyKind::Outbreak => 2,
                EmergencyKind::Medical => 3,
                EmergencyKind::Deathcare => 4,
                EmergencyKind::Crime => 5,
                EmergencyKind::Waste => 6,
                EmergencyKind::Repair => 1,
            };
            (
                priority,
                std::cmp::Reverse(e.severity),
                e.opened_day,
                e.tile,
            )
        });
        self.emergencies.truncate(4096);
        // Persistent, severe unrest becomes the same saved disaster/recovery event as
        // a scheduled riot. A cooldown in CivicState prevents repeated destruction.
        if let Some(e) = self
            .emergencies
            .iter()
            .find(|e| e.kind == EmergencyKind::Riot && self.day.saturating_sub(e.opened_day) >= 14)
        {
            if !city
                .civic
                .active_events
                .iter()
                .any(|v| v.kind == crate::civic::Hazard::Riot)
            {
                let mut civic = std::mem::take(&mut city.civic);
                let changed = civic
                    .trigger_disaster_at(city, crate::civic::Hazard::Riot, e.tile)
                    .is_ok();
                city.civic = civic;
                return changed;
            }
        }
        false
    }
    fn dispatch(&mut self, city: &mut City, routes: &mut Routing) {
        self.failed_routes.retain(|f| f.until_day > self.day);
        let mut dispatched = 0;
        let mut claimed = vec![0u8; city.tiles.len()];
        routes.prepare(city);
        let mut fleet_room = vec![0u16; city.tiles.len()];
        for group in &routes.providers[..6] {
            for &i in group {
                fleet_room[i as usize] = fleet(city, i as usize) as u16;
            }
        }
        let mut counts = [0usize; 6];
        for m in &self.missions {
            let p = m.purpose as usize;
            counts[p] += 1;
            fleet_room[m.facility as usize] = fleet_room[m.facility as usize].saturating_sub(1);
            claimed[m.target as usize] |= 1 << p;
            for &tile in &m.stops {
                claimed[tile as usize] |= 1 << p;
            }
        }
        for event in 0..self.emergencies.len() {
            let (kind, tile) = (self.emergencies[event].kind, self.emergencies[event].tile);
            if dispatched >= 128 || self.missions.len() >= 256 {
                break;
            }
            let purpose = match kind {
                EmergencyKind::Fire => ServicePurpose::Fire,
                EmergencyKind::Medical | EmergencyKind::Outbreak => ServicePurpose::Medical,
                EmergencyKind::Crime | EmergencyKind::Riot => ServicePurpose::Police,
                EmergencyKind::Waste => ServicePurpose::Waste,
                EmergencyKind::Deathcare => ServicePurpose::Deathcare,
                EmergencyKind::Repair => ServicePurpose::Repair,
            };
            let p = purpose as usize;
            let limit = match purpose {
                ServicePurpose::Waste => 128,
                ServicePurpose::Medical => 64,
                ServicePurpose::Fire => 64,
                _ => 32,
            };
            if counts[p] >= limit || claimed[tile as usize] & (1 << p) != 0 {
                continue;
            }
            let Some((facility, route)) =
                routes.route(city, purpose, tile as usize, false, Some(&fleet_room))
            else {
                continue;
            };
            let mut stops = vec![tile];
            if matches!(
                purpose,
                ServicePurpose::Waste | ServicePurpose::Medical | ServicePurpose::Deathcare
            ) {
                stops.clear();
                for &j in routes.addresses(city, &route).iter() {
                    let j = usize::from(j);
                    let stock = match purpose {
                        ServicePurpose::Waste => self.waste[j],
                        ServicePurpose::Medical => self.sick[j],
                        _ => self.dead[j],
                    };
                    if stock > 0 {
                        stops.push(j as u32);
                    }
                }
                if let Err(at) = stops.binary_search(&tile) {
                    stops.insert(at, tile);
                }
            }
            for &tile in &stops {
                claimed[tile as usize] |= 1 << p;
            }
            if let Some(id) = self.spawn_vehicle(city, route, 0) {
                self.missions.push(ServiceMission {
                    vehicle: id,
                    purpose,
                    facility,
                    target: tile,
                    started_day: self.day,
                    stops,
                    ..Default::default()
                });
                self.dispatches += 1;
                dispatched += 1;
                counts[p] += 1;
                fleet_room[facility as usize] = fleet_room[facility as usize].saturating_sub(1);
            }
        }
    }
    fn totals(&mut self, city: &mut City) {
        self.occupied_beds = self.patients.iter().map(|p| p.count).sum();
        self.ill = self.sick.iter().sum::<u32>()
            + self.occupied_beds
            + self
                .missions
                .iter()
                .filter(|m| m.purpose == ServicePurpose::Medical)
                .map(|m| m.load)
                .sum::<u32>();
        self.deaths_waiting = self.dead.iter().sum::<u32>()
            + self
                .missions
                .iter()
                .filter(|m| m.purpose == ServicePurpose::Deathcare)
                .map(|m| m.load)
                .sum::<u32>();
        self.prisoners = self.detentions.iter().map(|d| d.count).sum();
        self.landfill_used = self
            .facilities
            .iter()
            .filter(|f| f.kind == Kind::Landfill)
            .map(|f| f.used as u64)
            .sum();
        self.overflow = self.waste.iter().copied().fold(0, u32::saturating_add);
        let occupied = city
            .tiles
            .iter()
            .map(|t| t.population + t.jobs / 2)
            .sum::<u32>();
        city.garbage = (self.overflow.saturating_mul(100) / (occupied.max(8) * 3)).min(100) as u8;
        let target = 100u32
            .saturating_sub((self.ill as u64 * 300 / city.population.max(1) as u64).min(80) as u32)
            .saturating_sub(city.garbage as u32 / 5);
        city.health = ((city.health as u32 * 9 + target) / 10).min(100) as u8;
    }
    pub fn advance_day(&mut self, city: &mut City) {
        if !city.ext.enabled {
            return;
        }
        self.day = city.ext.day;
        let invalid: Vec<_> = self
            .repair_requests
            .iter()
            .filter(|r| {
                !city
                    .civic
                    .damage
                    .iter()
                    .any(|d| d.tile == r.tile && d.repair_cost == r.cost)
                    || city.tiles.get(r.tile as usize).is_none_or(|t| {
                        t.kind != Kind::Empty
                            && city
                                .civic
                                .damage
                                .iter()
                                .find(|d| d.tile == r.tile)
                                .is_none_or(|d| t.kind != d.previous.kind)
                    })
            })
            .map(|r| r.tile)
            .collect();
        for tile in invalid {
            let _ = self.cancel_repair(city, tile);
        }
        self.waste.resize(city.tiles.len(), 0);
        self.sick.resize(city.tiles.len(), 0);
        self.dead.resize(city.tiles.len(), 0);
        self.facilities(city);
        self.detentions.retain(|d| d.release_day > self.day);
        self.prisoners = self.detentions.iter().map(|d| d.count).sum();
        let patients = std::mem::take(&mut self.patients);
        let mut occupied = vec![0u32; city.tiles.len()];
        for m in &self.missions {
            if m.purpose == ServicePurpose::Medical {
                occupied[m.facility as usize] += m.load;
            }
        }
        for p in patients {
            if !operational(city, p.facility as usize)
                || !city
                    .tiles
                    .get(p.facility as usize)
                    .is_some_and(|t| serves(t.kind, ServicePurpose::Medical))
                || capacity(city, p.facility as usize).saturating_sub(occupied[p.facility as usize])
                    < p.count
            {
                self.sick[p.home as usize] = self.sick[p.home as usize].saturating_add(p.count);
            } else if p.discharge_day <= self.day {
                self.recovered += p.count as u64;
            } else {
                occupied[p.facility as usize] += p.count;
                self.patients.push(p);
            }
        }
        let mut routes = Routing {
            memo: std::mem::take(&mut self.cache),
            ..Default::default()
        };
        self.arrivals(city, &mut routes);
        self.disease_and_waste(city);
        let fire_changed = self.fires(city);
        self.totals(city);
        let riot_changed = self.emergencies(city);
        if fire_changed || riot_changed {
            routes.prepared = false;
        }
        self.dispatch(city, &mut routes);
        self.cache = routes.memo;
    }
    pub fn validate(&self, tiles: usize) -> Result<(), String> {
        let bad = || {
            Err("Invalid public-service stock, capacity, cargo or mission reference.".to_string())
        };
        if self.day > u32::MAX as u64
            || self.missions.len() > 256
            || self.emergencies.len() > 4096
            || self.facilities.len() > tiles
            || self.patients.len() > 65536
            || self.detentions.len() > 65536
            || self.failed_routes.len() > 128
            || self.repair_requests.len() > 128
            || self.notices.len() > 32
            || [&self.waste, &self.sick, &self.dead].iter().any(|v| {
                (!v.is_empty() && v.len() != tiles)
                    || v.iter().any(|&n| n > 1_000_000)
                    || v.iter().map(|&n| n as u64).sum::<u64>() > u32::MAX as u64
            })
            || [
                self.waste_generated,
                self.waste_collected,
                self.waste_processed,
                self.recycled,
                self.burials,
                self.recovered,
                self.dispatches,
                self.completed_repairs,
                self.cancelled_repairs,
            ]
            .iter()
            .any(|&v| v > 1_000_000_000_000)
            || self.waste_processed > self.waste_collected
            || self.recycled > self.waste_processed
            || self.beds as usize > tiles.saturating_mul(192)
            || self.prison_capacity as usize > tiles.saturating_mul(300)
            || self.landfill_capacity > tiles as u64 * 18000
        {
            return bad();
        }
        let mut ids = BTreeSet::new();
        for m in &self.missions {
            let size = match m.purpose {
                ServicePurpose::Waste => 1024,
                ServicePurpose::Medical => 16,
                ServicePurpose::Deathcare => 8,
                _ => 0,
            };
            if m.vehicle == 0
                || m.vehicle > 1_000_000_000
                || !ids.insert(m.vehicle)
                || m.facility as usize >= tiles
                || m.target as usize >= tiles
                || m.load > size
                || m.cargo.len() > 1024
                || m.stops.len() > tiles
                || m.cargo
                    .iter()
                    .any(|c| c[0] as usize >= tiles || c[1] == 0 || c[1] > size)
                || m.cargo.iter().map(|c| c[1] as u64).sum::<u64>() != m.load as u64
                || m.stops.iter().any(|&i| i as usize >= tiles)
                || m.stops.windows(2).any(|p| p[0] >= p[1])
                || (!m.returning && m.load != 0)
                || (m.returning && m.load == 0)
                || m.started_day > u32::MAX as u64
            {
                return bad();
            }
        }
        if self.patients.iter().any(|p| {
            p.home as usize >= tiles
                || p.facility as usize >= tiles
                || p.count == 0
                || p.count > 16
                || p.discharge_day > u32::MAX as u64
        }) || self
            .detentions
            .iter()
            .any(|d| d.count == 0 || d.count > 1_000_000 || d.release_day > u32::MAX as u64)
            || self.emergencies.iter().any(|e| {
                e.tile as usize >= tiles || e.severity > 100 || e.opened_day > u32::MAX as u64
            })
            || self.failed_routes.iter().any(|f| {
                f.facility as usize >= tiles
                    || f.target as usize >= tiles
                    || f.until_day > u32::MAX as u64
            })
            || self.notices.iter().any(|n| {
                n.tile as usize >= tiles || n.message.len() > 300 || n.day > u32::MAX as u64
            })
        {
            return bad();
        }
        let mut lots = BTreeSet::new();
        if self.facilities.iter().any(|f| {
            f.tile as usize >= tiles
                || !lots.insert(f.tile)
                || match f.kind {
                    Kind::Landfill => f.used > 18000 || f.processed_today != 0,
                    Kind::Cemetery => f.used > 3000 || f.processed_today != 0,
                    Kind::Incinerator => f.used != 0 || f.processed_today > 480,
                    Kind::Recycling => f.used != 0 || f.processed_today > 288,
                    _ => true,
                }
        }) {
            return bad();
        }
        lots.clear();
        if self.repair_requests.iter().any(|r| {
            r.tile as usize >= tiles
                || !lots.insert(r.tile)
                || !(0..=1_000_000).contains(&r.cost)
                || r.requested_day > u32::MAX as u64
        }) {
            return bad();
        }
        let beds = self.patients.iter().map(|p| p.count as u64).sum::<u64>();
        let incoming = |purpose| {
            self.missions
                .iter()
                .filter(|m| m.purpose == purpose)
                .map(|m| m.load as u64)
                .sum::<u64>()
        };
        if self.occupied_beds as u64 != beds
            || beds > self.beds as u64
            || self.ill as u64
                != self.sick.iter().map(|&v| v as u64).sum::<u64>()
                    + beds
                    + incoming(ServicePurpose::Medical)
            || self.deaths_waiting as u64
                != self.dead.iter().map(|&v| v as u64).sum::<u64>()
                    + incoming(ServicePurpose::Deathcare)
            || self.prisoners as u64 != self.detentions.iter().map(|d| d.count as u64).sum::<u64>()
            || self.landfill_used
                != self
                    .facilities
                    .iter()
                    .filter(|f| f.kind == Kind::Landfill)
                    .map(|f| f.used as u64)
                    .sum::<u64>()
            || self.overflow as u64 != self.waste.iter().map(|&v| v as u64).sum::<u64>()
        {
            return bad();
        }
        Ok(())
    }
    /// Check typed vehicle links without rejecting a legitimate pending road edit.
    pub fn validate_city(&self, city: &City) -> Result<(), String> {
        self.validate(city.tiles.len())?;
        let vehicles: BTreeMap<_, _> = city
            .ext
            .transport
            .vehicles
            .iter()
            .map(|v| (v.id, v))
            .collect();
        for m in &self.missions {
            let v = vehicles
                .get(&m.vehicle)
                .ok_or("A service mission refers to a missing vehicle.")?;
            if v.kind != VehicleKind::Service
                || v.line_id.is_some()
                || v.route.len() < 2
                || v.leg as usize >= v.route.len()
                || v.tile != v.route[v.leg as usize]
                || v.next_tile != v.route[(v.leg as usize + 1).min(v.route.len() - 1)]
                || (m.returning && v.route.last() != Some(&m.facility))
                || (!m.returning
                    && (v.route.first() != Some(&m.facility)
                        || city.distance(*v.route.last().unwrap() as usize, m.target as usize) > 2))
            {
                return Err(
                    "A service mission has an invalid vehicle, route or destination.".into(),
                );
            }
        }
        Ok(())
    }
}
