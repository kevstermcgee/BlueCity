//! Deterministic, integer-only BlueCity simulation. Roads contain municipal
//! power and water conduits; separate lines and pipes extend those networks.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

pub const SIZE: usize = 48;
const AREA: usize = SIZE * SIZE;
const FAR: u16 = u16::MAX;
fn legacy_dimension() -> u16 {
    SIZE as u16
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Water,
    Forest,
    Hill,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapStyle {
    #[default]
    River,
    Coast,
    Islands,
    Highlands,
    Lakes,
    Delta,
    Flatlands,
}

impl MapStyle {
    pub const ALL: [Self; 7] = [
        Self::River,
        Self::Coast,
        Self::Islands,
        Self::Highlands,
        Self::Lakes,
        Self::Delta,
        Self::Flatlands,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::River => "River",
            Self::Coast => "Coast",
            Self::Islands => "Islands",
            Self::Highlands => "Highlands",
            Self::Lakes => "Lakes",
            Self::Delta => "Delta",
            Self::Flatlands => "Flatlands",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Empty,
    Road,
    Avenue,
    Rail,
    PowerLine,
    Pipe,
    Residential,
    Commercial,
    Industrial,
    Coal,
    Wind,
    Nuclear,
    Pump,
    Sewage,
    Police,
    Fire,
    Hospital,
    School,
    University,
    Park,
    Stadium,
    BusDepot,
    Station,
    Seaport,
    Airport,
    Landfill,
    Gas,
    Solar,
    Hydro,
    Fusion,
    WaterTower,
    SewageOutlet,
    Incinerator,
    Recycling,
    Prison,
    FireHQ,
    Clinic,
    Cemetery,
    Library,
    BusStop,
    SubwayStation,
    FreightStation,
    Airfield,
    Marina,
    MaintenanceDepot,
    OneWay,
    Ramp,
    Highway,
    Roundabout,
    TunnelRoad,
    Subway,
    Playground,
    Plaza,
    BotanicalGarden,
    DogPark,
    SkatePark,
    WaterfrontPark,
    NatureReserve,
    CityHall,
    Market,
    Fountain,
    Monument,
    ClockTower,
    ConcertHall,
    Observatory,
    ScienceCenter,
    Aquarium,
    Zoo,
    SportsCenter,
    Beach,
    Campground,
    Fairground,
    HeritageMuseum,
    Arcology,
    Spaceport,
    WorldExpo,
    Farm,
    Hotel,
}

impl Kind {
    pub const ALL: &'static [Self] = &[
        Self::Empty,
        Self::Road,
        Self::Avenue,
        Self::Rail,
        Self::PowerLine,
        Self::Pipe,
        Self::Residential,
        Self::Commercial,
        Self::Industrial,
        Self::Coal,
        Self::Wind,
        Self::Nuclear,
        Self::Pump,
        Self::Sewage,
        Self::Police,
        Self::Fire,
        Self::Hospital,
        Self::School,
        Self::University,
        Self::Park,
        Self::Stadium,
        Self::BusDepot,
        Self::Station,
        Self::Seaport,
        Self::Airport,
        Self::Landfill,
        Self::Gas,
        Self::Solar,
        Self::Hydro,
        Self::Fusion,
        Self::WaterTower,
        Self::SewageOutlet,
        Self::Incinerator,
        Self::Recycling,
        Self::Prison,
        Self::FireHQ,
        Self::Clinic,
        Self::Cemetery,
        Self::Library,
        Self::BusStop,
        Self::SubwayStation,
        Self::FreightStation,
        Self::Airfield,
        Self::Marina,
        Self::MaintenanceDepot,
        Self::OneWay,
        Self::Ramp,
        Self::Highway,
        Self::Roundabout,
        Self::TunnelRoad,
        Self::Subway,
        Self::Playground,
        Self::Plaza,
        Self::BotanicalGarden,
        Self::DogPark,
        Self::SkatePark,
        Self::WaterfrontPark,
        Self::NatureReserve,
        Self::CityHall,
        Self::Market,
        Self::Fountain,
        Self::Monument,
        Self::ClockTower,
        Self::ConcertHall,
        Self::Observatory,
        Self::ScienceCenter,
        Self::Aquarium,
        Self::Zoo,
        Self::SportsCenter,
        Self::Beach,
        Self::Campground,
        Self::Fairground,
        Self::HeritageMuseum,
        Self::Arcology,
        Self::Spaceport,
        Self::WorldExpo,
        Self::Farm,
        Self::Hotel,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "Open land",
            Self::Road => "Road",
            Self::Avenue => "Avenue",
            Self::Rail => "Rail",
            Self::PowerLine => "Power line",
            Self::Pipe => "Pipe",
            Self::Residential => "Residential",
            Self::Commercial => "Commercial",
            Self::Industrial => "Industrial",
            Self::Coal => "Coal plant",
            Self::Wind => "Wind turbine",
            Self::Nuclear => "Nuclear plant",
            Self::Pump => "Water pump",
            Self::Sewage => "Sewage works",
            Self::Police => "Police station",
            Self::Fire => "Fire station",
            Self::Hospital => "Hospital",
            Self::School => "School",
            Self::University => "University",
            Self::Park => "Park",
            Self::Stadium => "Stadium",
            Self::BusDepot => "Bus depot",
            Self::Station => "Rail station",
            Self::Seaport => "Seaport",
            Self::Airport => "Airport",
            Self::Landfill => "Landfill",
            Self::Gas => "Gas plant",
            Self::Solar => "Solar farm",
            Self::Hydro => "Hydroelectric dam",
            Self::Fusion => "Fusion plant",
            Self::WaterTower => "Water tower",
            Self::SewageOutlet => "Sewage outlet",
            Self::Incinerator => "Incinerator",
            Self::Recycling => "Recycling",
            Self::Prison => "Prison",
            Self::FireHQ => "Fire headquarters",
            Self::Clinic => "Clinic",
            Self::Cemetery => "Cemetery",
            Self::Library => "Library",
            Self::BusStop => "Bus Stop",
            Self::SubwayStation => "Subway Station",
            Self::FreightStation => "Freight Station",
            Self::Airfield => "Airfield",
            Self::Marina => "Marina",
            Self::MaintenanceDepot => "Maintenance depot",
            Self::OneWay => "One-way road",
            Self::Ramp => "Ramp",
            Self::Highway => "Highway",
            Self::Roundabout => "Roundabout",
            Self::TunnelRoad => "Road tunnel",
            Self::Subway => "Subway track",
            Self::Playground => "Playground",
            Self::Plaza => "Plaza",
            Self::BotanicalGarden => "Botanical Garden",
            Self::DogPark => "Dog Park",
            Self::SkatePark => "Skate Park",
            Self::WaterfrontPark => "Waterfront Park",
            Self::NatureReserve => "Nature Reserve",
            Self::CityHall => "City Hall",
            Self::Market => "Market",
            Self::Fountain => "Fountain",
            Self::Monument => "Monument",
            Self::ClockTower => "Clock Tower",
            Self::ConcertHall => "Concert Hall",
            Self::Observatory => "Observatory",
            Self::ScienceCenter => "Science Center",
            Self::Aquarium => "Aquarium",
            Self::Zoo => "Zoo",
            Self::SportsCenter => "Sports Center",
            Self::Beach => "Beach",
            Self::Campground => "Campground",
            Self::Fairground => "Fairground",
            Self::HeritageMuseum => "Heritage Museum",
            Self::Arcology => "Arcology",
            Self::Spaceport => "Spaceport",
            Self::WorldExpo => "World Expo",
            Self::Farm => "Farm",
            Self::Hotel => "Hotel",
        }
    }
    pub fn cost(self) -> i64 {
        match self {
            Self::Empty => 15,
            Self::Road => 25,
            Self::Avenue => 65,
            Self::Rail => 45,
            Self::PowerLine => 15,
            Self::Pipe => 12,
            Self::Residential => 45,
            Self::Commercial => 45,
            Self::Industrial => 45,
            Self::Coal => 3200,
            Self::Wind => 950,
            Self::Nuclear => 18000,
            Self::Pump => 1400,
            Self::Sewage => 1700,
            Self::Police => 1800,
            Self::Fire => 1600,
            Self::Hospital => 3200,
            Self::School => 1800,
            Self::University => 6500,
            Self::Park => 200,
            Self::Stadium => 6500,
            Self::BusDepot => 1800,
            Self::Station => 2400,
            Self::Seaport => 5500,
            Self::Airport => 12000,
            Self::Landfill => 1000,
            Self::Gas => 4800,
            Self::Solar => 2200,
            Self::Hydro => 8500,
            Self::Fusion => 55000,
            Self::WaterTower => 1100,
            Self::SewageOutlet => 450,
            Self::Incinerator => 4200,
            Self::Recycling => 2600,
            Self::Prison => 3600,
            Self::FireHQ => 5200,
            Self::Clinic => 1100,
            Self::Cemetery => 1500,
            Self::Library => 1800,
            Self::BusStop => 80,
            Self::SubwayStation => 3800,
            Self::FreightStation => 3600,
            Self::Airfield => 4000,
            Self::Marina => 2200,
            Self::MaintenanceDepot => 1600,
            Self::OneWay => 35,
            Self::Ramp => 85,
            Self::Highway => 110,
            Self::Roundabout => 85,
            Self::TunnelRoad => 160,
            Self::Subway => 110,
            Self::Playground => 400,
            Self::Plaza => 400,
            Self::BotanicalGarden => 4200,
            Self::DogPark => 400,
            Self::SkatePark => 400,
            Self::WaterfrontPark => 4200,
            Self::NatureReserve => 4200,
            Self::CityHall => 4200,
            Self::Market => 4200,
            Self::Fountain => 400,
            Self::Monument => 4200,
            Self::ClockTower => 4200,
            Self::ConcertHall => 4200,
            Self::Observatory => 4200,
            Self::ScienceCenter => 15000,
            Self::Aquarium => 4200,
            Self::Zoo => 4200,
            Self::SportsCenter => 4200,
            Self::Beach => 400,
            Self::Campground => 400,
            Self::Fairground => 4200,
            Self::HeritageMuseum => 4200,
            Self::Arcology => 60000,
            Self::Spaceport => 80000,
            Self::WorldExpo => 65000,
            Self::Farm => 65,
            Self::Hotel => 120,
        }
    }
    pub fn upkeep(self) -> i64 {
        match self {
            Self::Empty => 0,
            Self::Road => 1,
            Self::Avenue => 2,
            Self::Rail => 2,
            Self::PowerLine => 1,
            Self::Pipe => 1,
            Self::Residential => 0,
            Self::Commercial => 0,
            Self::Industrial => 0,
            Self::Coal => 90,
            Self::Wind => 15,
            Self::Nuclear => 380,
            Self::Pump => 40,
            Self::Sewage => 35,
            Self::Police => 45,
            Self::Fire => 40,
            Self::Hospital => 65,
            Self::School => 45,
            Self::University => 110,
            Self::Park => 4,
            Self::Stadium => 60,
            Self::BusDepot => 35,
            Self::Station => 35,
            Self::Seaport => 100,
            Self::Airport => 150,
            Self::Landfill => 30,
            Self::Gas => 130,
            Self::Solar => 25,
            Self::Hydro => 80,
            Self::Fusion => 650,
            Self::WaterTower => 18,
            Self::SewageOutlet => 10,
            Self::Incinerator => 70,
            Self::Recycling => 50,
            Self::Prison => 85,
            Self::FireHQ => 110,
            Self::Clinic => 35,
            Self::Cemetery => 20,
            Self::Library => 35,
            Self::BusStop => 2,
            Self::SubwayStation => 60,
            Self::FreightStation => 65,
            Self::Airfield => 65,
            Self::Marina => 35,
            Self::MaintenanceDepot => 30,
            Self::OneWay => 1,
            Self::Ramp => 3,
            Self::Highway => 4,
            Self::Roundabout => 2,
            Self::TunnelRoad => 5,
            Self::Subway => 3,
            Self::Playground => 8,
            Self::Plaza => 8,
            Self::BotanicalGarden => 50,
            Self::DogPark => 8,
            Self::SkatePark => 8,
            Self::WaterfrontPark => 50,
            Self::NatureReserve => 50,
            Self::CityHall => 50,
            Self::Market => 50,
            Self::Fountain => 8,
            Self::Monument => 50,
            Self::ClockTower => 50,
            Self::ConcertHall => 50,
            Self::Observatory => 50,
            Self::ScienceCenter => 140,
            Self::Aquarium => 50,
            Self::Zoo => 50,
            Self::SportsCenter => 50,
            Self::Beach => 8,
            Self::Campground => 8,
            Self::Fairground => 50,
            Self::HeritageMuseum => 50,
            Self::Arcology => 450,
            Self::Spaceport => 700,
            Self::WorldExpo => 400,
            Self::Farm => 2,
            Self::Hotel => 2,
        }
    }
    pub fn category(self) -> u8 {
        match self {
            Self::Empty => 4,
            Self::Road => 1,
            Self::Avenue => 1,
            Self::Rail => 1,
            Self::PowerLine => 2,
            Self::Pipe => 2,
            Self::Residential => 0,
            Self::Commercial => 0,
            Self::Industrial => 0,
            Self::Coal => 2,
            Self::Wind => 2,
            Self::Nuclear => 2,
            Self::Pump => 2,
            Self::Sewage => 2,
            Self::Police => 3,
            Self::Fire => 3,
            Self::Hospital => 3,
            Self::School => 3,
            Self::University => 3,
            Self::Park => 0,
            Self::Stadium => 4,
            Self::BusDepot => 1,
            Self::Station => 1,
            Self::Seaport => 1,
            Self::Airport => 1,
            Self::Landfill => 2,
            Self::Gas => 2,
            Self::Solar => 2,
            Self::Hydro => 2,
            Self::Fusion => 2,
            Self::WaterTower => 2,
            Self::SewageOutlet => 2,
            Self::Incinerator => 2,
            Self::Recycling => 2,
            Self::Prison => 3,
            Self::FireHQ => 3,
            Self::Clinic => 3,
            Self::Cemetery => 3,
            Self::Library => 3,
            Self::BusStop => 1,
            Self::SubwayStation => 1,
            Self::FreightStation => 1,
            Self::Airfield => 1,
            Self::Marina => 1,
            Self::MaintenanceDepot => 1,
            Self::OneWay => 1,
            Self::Ramp => 1,
            Self::Highway => 1,
            Self::Roundabout => 1,
            Self::TunnelRoad => 1,
            Self::Subway => 1,
            Self::Playground => 0,
            Self::Plaza => 0,
            Self::BotanicalGarden => 4,
            Self::DogPark => 0,
            Self::SkatePark => 0,
            Self::WaterfrontPark => 0,
            Self::NatureReserve => 0,
            Self::CityHall => 3,
            Self::Market => 4,
            Self::Fountain => 4,
            Self::Monument => 4,
            Self::ClockTower => 4,
            Self::ConcertHall => 4,
            Self::Observatory => 4,
            Self::ScienceCenter => 4,
            Self::Aquarium => 4,
            Self::Zoo => 4,
            Self::SportsCenter => 4,
            Self::Beach => 4,
            Self::Campground => 4,
            Self::Fairground => 4,
            Self::HeritageMuseum => 4,
            Self::Arcology => 4,
            Self::Spaceport => 4,
            Self::WorldExpo => 4,
            Self::Farm => 0,
            Self::Hotel => 0,
        }
    }
    pub fn is_zone(self) -> bool {
        matches!(
            self,
            Self::Residential | Self::Commercial | Self::Industrial | Self::Farm | Self::Hotel
        )
    }
    pub fn is_road(self) -> bool {
        matches!(
            self,
            Self::Road
                | Self::Avenue
                | Self::OneWay
                | Self::Ramp
                | Self::Highway
                | Self::Roundabout
                | Self::TunnelRoad
        )
    }
    pub fn is_network(self) -> bool {
        matches!(
            self,
            Self::Road
                | Self::Avenue
                | Self::OneWay
                | Self::Ramp
                | Self::Highway
                | Self::Roundabout
                | Self::TunnelRoad
                | Self::Rail
                | Self::PowerLine
                | Self::Pipe
                | Self::Subway
        )
    }
    pub fn is_building(self) -> bool {
        self != Self::Empty && !self.is_network()
    }
    pub fn funding(self) -> usize {
        match self {
            Self::Road
            | Self::Avenue
            | Self::OneWay
            | Self::Ramp
            | Self::Highway
            | Self::Roundabout
            | Self::TunnelRoad
            | Self::Rail
            | Self::Subway
            | Self::BusDepot
            | Self::BusStop
            | Self::Station
            | Self::SubwayStation
            | Self::FreightStation
            | Self::Seaport
            | Self::Airport
            | Self::Airfield
            | Self::Marina
            | Self::MaintenanceDepot
            | Self::PowerLine
            | Self::Pipe => 0,
            Self::School | Self::University | Self::Library => 2,
            Self::Coal
            | Self::Wind
            | Self::Nuclear
            | Self::Gas
            | Self::Solar
            | Self::Hydro
            | Self::Fusion
            | Self::Pump
            | Self::WaterTower
            | Self::Sewage
            | Self::SewageOutlet
            | Self::Landfill
            | Self::Incinerator
            | Self::Recycling => 3,
            _ => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub kind: Kind,
    pub level: u8,
    pub population: u32,
    pub jobs: u32,
    pub powered: bool,
    pub watered: bool,
    pub connected: bool,
    pub pollution: u8,
    pub traffic: u8,
    pub crime: u8,
    pub land_value: u8,
    pub fire: u8,
    /// A road and railway share this tile as a level crossing.
    pub rail_crossing: bool,
}
impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain,
            kind: Kind::Empty,
            level: 0,
            population: 0,
            jobs: 0,
            powered: false,
            watered: false,
            connected: false,
            pollution: 0,
            traffic: 0,
            crime: 0,
            land_value: 40,
            fire: 0,
            rail_crossing: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disaster {
    Meteor,
    HostileUfo,
    FriendlyUfo,
    PlaneCrash,
    Monster,
    Riot,
    Fire,
    Flood,
    Earthquake,
    Tornado,
    Meltdown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    pub population: u32,
    pub money: i64,
    pub income: i64,
    pub expenses: i64,
    pub happiness: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct City {
    #[serde(default = "legacy_dimension")]
    pub width: u16,
    #[serde(default = "legacy_dimension")]
    pub height: u16,
    #[serde(default)]
    pub ext: crate::world::WorldState,
    #[serde(default)]
    pub civic: crate::civic::CivicState,
    pub tiles: Vec<Tile>,
    pub money: i64,
    pub month: u32,
    pub population: u32,
    pub jobs: u32,
    pub happiness: u8,
    pub demand: [i32; 3],
    pub income: i64,
    pub expenses: i64,
    pub power_supply: u32,
    pub power_demand: u32,
    pub water_supply: u32,
    pub water_demand: u32,
    pub debt: i64,
    pub taxes: [u8; 3],
    pub funding: [u8; 4],
    pub policies: [bool; 4],
    pub history: Vec<History>,
    pub messages: Vec<String>,
    pub seed: u64,
    pub health: u8,
    pub education: u8,
    pub garbage: u8,
    pub unemployment: u8,
    pub transport_usage: u8,
}

struct Networks {
    roads: Vec<bool>,
    access: Vec<u16>,
    police: Vec<u8>,
    fire: Vec<u8>,
    health: Vec<u8>,
    school: Vec<u8>,
    university: Vec<u8>,
    park: Vec<u8>,
    transit: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct RoadLayout {
    width: u16,
    height: u16,
    rail_imports: bool,
    cells: Vec<(Kind, Terrain, bool)>,
}
#[derive(Clone, Debug)]
struct RoadAccessMemo {
    layout: RoadLayout,
    roads: Vec<bool>,
    access: Vec<u16>,
}
#[derive(Clone, Debug)]
struct EnvironmentMemo {
    width: u16,
    key: Vec<u8>,
    trees: Vec<u8>,
    scenic_water: Vec<u8>,
}
/// Derived exact coverage/shore memoization. Immutable shared entries make
/// snapshots cheap; none of this cache is authoritative or serialized.
#[derive(Clone, Debug, Default)]
pub(crate) struct InfrastructureMemo {
    pub(crate) shore: Option<Arc<Vec<bool>>>,
    roads: Option<Arc<Vec<bool>>>,
    services: Vec<CoverageEntry>,
    rasters: Vec<Option<Arc<Vec<u16>>>>,
    raster_count: usize,
    kinds: Option<Arc<Vec<Kind>>>,
    by_kind: Arc<Vec<Vec<usize>>>,
    road_access: Option<Arc<RoadAccessMemo>>,
    environment: Option<Arc<EnvironmentMemo>>,
}
impl PartialEq for InfrastructureMemo {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for InfrastructureMemo {}
#[derive(Clone, Debug)]
struct CoverageEntry {
    kinds: Vec<Kind>,
    roads: Arc<Vec<bool>>,
    providers: Vec<usize>,
    range: u16,
    budget: u16,
    values: Arc<Vec<u8>>,
}

pub fn index(x: i32, y: i32) -> Option<usize> {
    if x >= 0 && y >= 0 && x < SIZE as i32 && y < SIZE as i32 {
        Some(y as usize * SIZE + x as usize)
    } else {
        None
    }
}
fn capacity(kind: Kind, level: u8) -> u32 {
    let l = level.min(5) as usize;
    match kind {
        Kind::Residential => [0, 24, 64, 144, 280, 480][l],
        Kind::Commercial | Kind::Hotel => [0, 18, 48, 120, 250, 450][l],
        Kind::Industrial => [0, 28, 60, 100, 160, 240][l],
        Kind::Farm => [0, 12, 24, 36, 48, 64][l],
        Kind::Arcology | Kind::Spaceport | Kind::WorldExpo | Kind::ScienceCenter => 250,
        _ => 0,
    }
}
fn random(seed: &mut u64) -> u64 {
    // An explicit nonzero substitute makes seed 0 a useful reproducible world.
    if *seed == 0 {
        *seed = 0x9e3779b97f4a7c15;
    }
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

// Coordinate hashing is separate from the simulation's random stream. Generation
// reads the entire u64 seed, including zero, without advancing disaster randomness.
fn terrain_hash(seed: u64, x: i32, y: i32, layer: u64) -> u64 {
    let mut value = seed
        .wrapping_add(layer.wrapping_mul(0x9e3779b97f4a7c15))
        .wrapping_add((x as u64).wrapping_mul(0x632be59bd9b4e019))
        .wrapping_add((y as u64).wrapping_mul(0x85157af5d66d8cbd));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

// Integer smoothstep interpolation keeps coastlines coherent and reproducible on
// every target rather than scattering independent random water tiles.
fn terrain_noise(seed: u64, x: i32, y: i32, scale: i32, layer: u64) -> i32 {
    let gx = x.div_euclid(scale);
    let gy = y.div_euclid(scale);
    let sx = x.rem_euclid(scale);
    let sy = y.rem_euclid(scale);
    let tx = sx * sx * (3 * scale - 2 * sx);
    let ty = sy * sy * (3 * scale - 2 * sy);
    let denominator = scale * scale * scale;
    let sample = |dx, dy| (terrain_hash(seed, gx + dx, gy + dy, layer) & 1023) as i32;
    let top = (sample(0, 0) * (denominator - tx) + sample(1, 0) * tx) / denominator;
    let bottom = (sample(0, 1) * (denominator - tx) + sample(1, 1) * tx) / denominator;
    (top * (denominator - ty) + bottom * ty) / denominator
}

fn generated_terrain(seed: u64, style: MapStyle, size: usize) -> Vec<Tile> {
    let parameter =
        |layer, minimum, range| minimum + (terrain_hash(seed, 0, 0, layer) % range as u64) as i32;
    let rotation = parameter(1, 0, 4);
    let river_center = parameter(2, 8, 6);
    let river_radius = parameter(3, 2, 2);
    let coast = parameter(4, 10, 8);
    let islands = [
        (
            parameter(10, 21, 7),
            parameter(11, 21, 7),
            parameter(12, 11, 4),
        ),
        (
            parameter(13, 5, 8),
            parameter(14, 5, 8),
            parameter(15, 5, 4),
        ),
        (
            parameter(16, 35, 8),
            parameter(17, 5, 8),
            parameter(18, 5, 4),
        ),
        (
            parameter(19, 5, 8),
            parameter(20, 35, 8),
            parameter(21, 5, 4),
        ),
        (
            parameter(22, 35, 8),
            parameter(23, 35, 8),
            parameter(24, 5, 4),
        ),
        (0, parameter(25, 12, 24), parameter(26, 5, 3)),
    ];
    let lakes = [
        (
            parameter(30, 7, 10),
            parameter(31, 7, 10),
            parameter(32, 4, 3),
        ),
        (
            parameter(33, 33, 8),
            parameter(34, 33, 8),
            parameter(35, 3, 3),
        ),
    ];
    (0..(size * size))
        .map(|i| {
            let x = (i % size) as i32;
            let y = (i / size) as i32;
            let gx = x * 48 / size as i32;
            let gy = y * 48 / size as i32;
            let (u, v) = match rotation {
                0 => (gx, gy),
                1 => (47 - gy, gx),
                2 => (47 - gx, 47 - gy),
                _ => (gy, 47 - gx),
            };
            let coast_noise = terrain_noise(seed, u, v, 7, 40) - 512;
            let water = match style {
                MapStyle::Flatlands => u < 3 || (u > 40 && v > 38),
                MapStyle::Delta => {
                    let branch = (v - 24).abs() / 6;
                    (u - river_center - branch).abs() <= 2
                        || (u - river_center + branch).abs() <= 2
                        || v > 43
                }
                MapStyle::River => {
                    let bend = (terrain_noise(seed, 0, v, 10, 41) - 512) / 128;
                    (u - river_center - bend).abs() <= river_radius
                }
                MapStyle::Coast => {
                    let shoreline = coast + (terrain_noise(seed, 0, v, 9, 42) - 512) / 120;
                    u < shoreline
                }
                MapStyle::Islands => !islands.iter().any(|&(cx, cy, radius)| {
                    let dx = u - cx;
                    let dy = v - cy;
                    (radius * radius - dx * dx - dy * dy) * 16 + coast_noise / 5 >= 0
                }),
                MapStyle::Lakes | MapStyle::Highlands => lakes.iter().any(|&(cx, cy, radius)| {
                    let dx = u - cx;
                    let dy = v - cy;
                    (radius * radius - dx * dx - dy * dy) * 16 + coast_noise / 5 >= 0
                }),
            };
            let elevation =
                terrain_noise(seed, x, y, 12, 50) * 2 + terrain_noise(seed, x, y, 5, 51);
            let hills = if style == MapStyle::Highlands {
                1400
            } else {
                2250
            };
            let terrain = if ((size as i32 / 2 - 3)..=(size as i32 / 2 + 3)).contains(&x)
                && ((size as i32 / 2 - 3)..=(size as i32 / 2 + 3)).contains(&y)
            {
                // A small arrival clearing leaves a usable first building site.
                Terrain::Grass
            } else if water {
                Terrain::Water
            } else if elevation > hills {
                Terrain::Hill
            } else if terrain_noise(seed, x, y, 5, 52) > 620 {
                Terrain::Forest
            } else {
                Terrain::Grass
            };
            Tile::new(terrain)
        })
        .collect()
}

impl City {
    /// Generate an undeveloped region without changing the legacy starter map.
    pub fn generated(seed: u64, style: MapStyle) -> Self {
        let mut city = Self::new(seed, false);
        city.tiles = generated_terrain(seed, style, SIZE);
        city.history.clear();
        city.messages.clear();
        city.refresh();
        city.record_history();
        city.message(format!(
            "{} region · Seed {seed}. Connect a road to the map edge, then add power, water and zones.",
            style.label()
        ));
        city
    }

    pub fn new(seed: u64, starter: bool) -> Self {
        let mut terrain_seed = seed.wrapping_add(0x6a09e667f3bcc909);
        let tiles = (0..AREA)
            .map(|i| {
                let x = i % SIZE;
                let y = i / SIZE;
                let coast = 5 + (24usize.abs_diff(y).min(24) / 8);
                let noise = random(&mut terrain_seed) % 100;
                let terrain = if x <= coast || ((39..=41).contains(&y) && x < 33) {
                    Terrain::Water
                } else if noise < 16 {
                    Terrain::Forest
                } else if noise > 94 {
                    Terrain::Hill
                } else {
                    Terrain::Grass
                };
                Tile::new(terrain)
            })
            .collect();
        let mut city = Self {
            width: SIZE as u16,
            height: SIZE as u16,
            ext: Default::default(),
            civic: Default::default(),
            tiles,
            money: if starter { 45000 } else { 60000 },
            month: 0,
            population: 0,
            jobs: 0,
            happiness: 65,
            demand: [70, 45, 55],
            income: 0,
            expenses: 0,
            power_supply: 0,
            power_demand: 0,
            water_supply: 0,
            water_demand: 0,
            debt: 0,
            taxes: [9; 3],
            funding: [100; 4],
            policies: [false; 4],
            history: vec![],
            messages: vec![],
            seed,
            health: 70,
            education: 0,
            garbage: 0,
            unemployment: 0,
            transport_usage: 0,
        };
        if starter {
            city.make_starter();
        }
        city.refresh();
        city.record_history();
        city.message(if starter {
            "Welcome, Mayor! Your riverside town is connected and growing. Expand zones beside roads."
        } else { "A fresh region awaits. Connect a road to the map edge, then add power, water and zones." });
        city
    }
    pub fn index(x: i32, y: i32) -> Option<usize> {
        index(x, y)
    }
    pub fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }
    #[inline]
    pub fn neighbor_array(&self, i: usize) -> [Option<usize>; 4] {
        let w = self.width as usize;
        let x = if w == 128 { i & 127 } else { i % w };
        [
            if x > 0 { Some(i - 1) } else { None },
            if x + 1 < w { Some(i + 1) } else { None },
            if i >= w { Some(i - w) } else { None },
            if i + w < self.tiles.len() {
                Some(i + w)
            } else {
                None
            },
        ]
    }
    #[inline]
    pub fn neighbors(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        self.neighbor_array(i).into_iter().flatten()
    }
    #[inline]
    pub fn distance(&self, a: usize, b: usize) -> usize {
        if self.width == 128 {
            return (a & 127).abs_diff(b & 127) + (a >> 7).abs_diff(b >> 7);
        }
        (a % self.width as usize).abs_diff(b % self.width as usize)
            + (a / self.width as usize).abs_diff(b / self.width as usize)
    }
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        self.idx(x, y).map(|i| &self.tiles[i])
    }
    fn message(&mut self, text: impl Into<String>) {
        self.messages.push(text.into());
        if self.messages.len() > 16 {
            self.messages.remove(0);
        }
    }
    fn make_starter(&mut self) {
        let mut set = |x, y, kind: Kind| {
            let i = self.idx(x, y).unwrap();
            self.tiles[i] = Tile::new(Terrain::Grass);
            self.tiles[i].kind = kind;
            if kind.is_zone() {
                self.tiles[i].level = 1;
                if kind == Kind::Residential {
                    self.tiles[i].population = capacity(kind, 1);
                } else {
                    self.tiles[i].jobs = capacity(kind, 1);
                }
            }
        };
        for y in [13, 19, 25, 31] {
            for x in 14..=32 {
                set(x, y, Kind::Road);
            }
        }
        for x in [14, 20, 26, 32] {
            for y in 13..=31 {
                set(x, y, Kind::Road);
            }
        }
        for y in 0..=13 {
            set(23, y, Kind::Avenue);
        }
        for x in 8..=14 {
            set(x, 25, Kind::Road);
        }
        for y in 14..=18 {
            for x in 15..=25 {
                if x != 20 {
                    set(x, y, Kind::Residential);
                }
            }
        }
        for y in 20..=24 {
            for x in 15..=19 {
                set(x, y, Kind::Commercial);
            }
        }
        for y in 26..=30 {
            for x in 15..=25 {
                if x != 20 {
                    set(x, y, Kind::Residential);
                }
            }
        }
        for y in 20..=30 {
            for x in 27..=31 {
                if y != 25 {
                    set(x, y, Kind::Industrial);
                }
            }
        }
        for (x, y, kind) in [
            (30, 14, Kind::Coal),
            (31, 16, Kind::Landfill),
            (8, 24, Kind::Pump),
            (9, 24, Kind::Sewage),
            (21, 20, Kind::Police),
            (22, 20, Kind::Fire),
            (21, 26, Kind::Hospital),
            (16, 26, Kind::School),
            (18, 16, Kind::Park),
            (23, 16, Kind::Park),
            (18, 28, Kind::Park),
            (24, 28, Kind::Park),
            (24, 30, Kind::BusDepot),
        ] {
            set(x, y, kind);
        }
        // The starter pump is on an actual bank, connected by the waterfront road.
        for y in 22..=26 {
            let i = self.idx(7, y).unwrap();
            self.tiles[i] = Tile::new(Terrain::Water);
        }
    }
    pub fn place(&mut self, x: i32, y: i32, kind: Kind) -> Result<i64, String> {
        let result = self.place_internal(x, y, kind);
        if result.is_ok() {
            self.refresh();
        }
        result
    }
    /// Paint a road or zoning brush with one infrastructure recomputation.
    /// Each edit is individually validated and charged, in the supplied order.
    pub fn edit_batch(&mut self, edits: &[(i32, i32, Kind)]) -> Vec<Result<i64, String>> {
        let results: Vec<_> = edits
            .iter()
            .map(|&(x, y, k)| self.place_internal(x, y, k))
            .collect();
        if results.iter().any(Result::is_ok) {
            self.refresh();
        }
        results
    }
    fn place_internal(&mut self, x: i32, y: i32, kind: Kind) -> Result<i64, String> {
        if kind == Kind::Empty {
            return self.bulldoze_internal(x, y);
        }
        let cost = self.placement_cost(x, y, kind)?;
        let i = self.idx(x, y).unwrap();
        if self.ext.enabled
            && self.tiles[i].kind != Kind::Empty
            && matches!(kind, Kind::Pipe | Kind::PowerLine | Kind::Subway)
        {
            self.money -= cost;
            let c = &mut self.ext.cells[i];
            match kind {
                Kind::Pipe => {
                    c.pipe = true;
                    c.sewer = true;
                }
                Kind::PowerLine => c.underground_power = true,
                Kind::Subway => c.subway = true,
                _ => {}
            }
            return Ok(cost);
        }
        let old = self.tiles[i];
        let upgrade = old.kind.is_road() && kind.is_road();
        let crossing = (old.kind.is_road() && kind == Kind::Rail)
            || (old.kind == Kind::Rail && kind.is_road());
        self.money -= cost;
        let terrain = if old.terrain == Terrain::Water {
            Terrain::Water
        } else {
            Terrain::Grass
        };
        self.tiles[i] = Tile::new(terrain);
        self.tiles[i].kind = if crossing && old.kind.is_road() {
            old.kind
        } else {
            kind
        };
        self.tiles[i].rail_crossing = crossing || (upgrade && old.rail_crossing);
        self.ext.register_edit(i, self.tiles[i].kind, self.month);
        Ok(cost)
    }
    pub fn bulldoze(&mut self, x: i32, y: i32) -> Result<i64, String> {
        let result = self.bulldoze_internal(x, y);
        if result.is_ok() {
            self.refresh();
        }
        result
    }
    fn bulldoze_internal(&mut self, x: i32, y: i32) -> Result<i64, String> {
        let i = self
            .idx(x, y)
            .ok_or("That location is outside the region.")?;
        let t = self.tiles[i];
        if self.ext.enabled {
            self.civic.can_bulldoze(i as u32)?;
        }
        if t.kind == Kind::Empty
            && t.terrain != Terrain::Forest
            && t.terrain != Terrain::Hill
            && !self.ext.cells.get(i).is_some_and(|c| c.rubble)
        {
            return Err("There is nothing to bulldoze here.".into());
        }
        let cost = if t.terrain == Terrain::Hill { 40 } else { 15 };
        if self.money < cost {
            return Err("Not enough treasury funds to bulldoze.".into());
        }
        self.money -= cost;
        self.clear(i);
        Ok(cost)
    }
    fn clear(&mut self, i: usize) {
        if self.ext.enabled {
            self.ext.people.migrants_out += self.tiles[i].population as u64;
        }
        let terrain = if self.tiles[i].terrain == Terrain::Water {
            Terrain::Water
        } else {
            Terrain::Grass
        };
        self.tiles[i] = Tile::new(terrain);
        self.ext.register_edit(i, Kind::Empty, self.month);
    }
    fn near_water(&self, i: usize, radius: usize) -> bool {
        if self.ext.enabled && self.ext.cells.len() == self.tiles.len() && radius <= 3 {
            return self.ext.cells[i].shore_distance as usize <= radius;
        }
        let x = (i % (self.width as usize)) as i32;
        let y = (i / (self.width as usize)) as i32;
        let radius = radius as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() > radius {
                    continue;
                }
                if let Some(j) = self.idx(x + dx, y + dy) {
                    if self.tiles[j].terrain == Terrain::Water {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn road_network(&self) -> (Vec<bool>, Vec<u16>) {
        let mut roads = vec![false; self.tiles.len()];
        let mut queue = VecDeque::new();
        // A connected rail line can import people and goods through a station.
        let mut rail = vec![false; self.tiles.len()];
        let mut rail_queue = VecDeque::new();
        for i in 0..self.tiles.len() {
            let edge = i % (self.width as usize) == 0
                || i % (self.width as usize) == (self.width as usize) - 1
                || i / (self.width as usize) == 0
                || i / (self.width as usize) == (self.width as usize) - 1;
            if edge && self.rail_track(i) {
                rail[i] = true;
                rail_queue.push_back(i);
            }
            if edge && self.tiles[i].kind.is_road() {
                roads[i] = true;
                queue.push_back(i);
            }
        }
        while let Some(i) = rail_queue.pop_front() {
            for j in self.neighbor_array(i).into_iter().flatten() {
                if !rail[j] && self.rail_track(j) {
                    rail[j] = true;
                    rail_queue.push_back(j);
                }
            }
        }
        if self.funding[0] >= 25 {
            for (i, &connected) in rail.iter().enumerate() {
                if connected && matches!(self.tiles[i].kind, Kind::Station | Kind::FreightStation) {
                    let x = (i % self.width as usize) as i32;
                    let y = (i / self.width as usize) as i32;
                    for dy in -2i32..=2 {
                        for dx in -2i32..=2 {
                            if dx.abs() + dy.abs() > 2 {
                                continue;
                            }
                            if let Some(j) = self.idx(x + dx, y + dy) {
                                if self.tiles[j].kind.is_road() && !roads[j] {
                                    roads[j] = true;
                                    queue.push_back(j);
                                }
                            }
                        }
                    }
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            for j in self.neighbor_array(i).into_iter().flatten() {
                if !roads[j] && self.tiles[j].kind.is_road() {
                    roads[j] = true;
                    queue.push_back(j);
                }
            }
        }
        let mut access = vec![FAR; self.tiles.len()];
        for i in 0..self.tiles.len() {
            if roads[i] {
                access[i] = 0;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            if access[i] >= 3 {
                continue;
            }
            for j in self.neighbor_array(i).into_iter().flatten() {
                if access[j] > access[i] + 1 && self.tiles[j].terrain != Terrain::Water {
                    access[j] = access[i] + 1;
                    queue.push_back(j);
                }
            }
        }
        (roads, access)
    }
    fn cached_road_network(&mut self) -> (Vec<bool>, Vec<u16>) {
        let layout = RoadLayout {
            width: self.width,
            height: self.height,
            rail_imports: self.funding[0] >= 25,
            cells: self
                .tiles
                .iter()
                .map(|t| (t.kind, t.terrain, t.rail_crossing))
                .collect(),
        };
        if let Some(cache) = self
            .ext
            .infrastructure_memo
            .road_access
            .as_ref()
            .filter(|m| m.layout == layout)
        {
            return (cache.roads.clone(), cache.access.clone());
        }
        let (roads, access) = self.road_network();
        self.ext.infrastructure_memo.road_access = Some(Arc::new(RoadAccessMemo {
            layout,
            roads: roads.clone(),
            access: access.clone(),
        }));
        (roads, access)
    }
    fn rail_track(&self, i: usize) -> bool {
        matches!(
            self.tiles[i].kind,
            Kind::Rail | Kind::Station | Kind::FreightStation
        ) || self.tiles[i].rail_crossing
    }
    fn station_active(&self, start: usize) -> bool {
        if self.funding[0] < 25 {
            return false;
        }
        let mut seen = vec![false; self.tiles.len()];
        let mut queue = VecDeque::from([start]);
        seen[start] = true;
        while let Some(i) = queue.pop_front() {
            let edge = i % (self.width as usize) == 0
                || i % (self.width as usize) == (self.width as usize) - 1
                || i / (self.width as usize) == 0
                || i / (self.width as usize) == (self.width as usize) - 1;
            if i != start
                && ((self.tiles[i].kind == Kind::Station
                    && self.tiles[i].connected
                    && self.tiles[i].powered
                    && self.tiles[i].watered)
                    || edge)
            {
                return true;
            }
            for j in self.neighbor_array(i).into_iter().flatten() {
                if !seen[j] && self.rail_track(j) {
                    seen[j] = true;
                    queue.push_back(j);
                }
            }
        }
        false
    }
    fn service(
        &mut self,
        roads: &[bool],
        kinds: &[Kind],
        range: u16,
        funding: usize,
        require_utilities: bool,
    ) -> Vec<u8> {
        let budget = self.funding[funding].min(150) as u16;
        let range = range * budget / 100;
        if range == 0 {
            return vec![0; self.tiles.len()];
        }
        let mut providers = Vec::new();
        let mut candidates: Vec<usize> = kinds
            .iter()
            .flat_map(|&k| {
                self.ext.infrastructure_memo.by_kind[k as usize]
                    .iter()
                    .copied()
            })
            .collect();
        candidates.sort_unstable();
        for &i in &candidates {
            let t = &self.tiles[i];
            if !t.connected
                || t.fire > 0
                || self.ext.cells.get(i).is_some_and(|c| !c.operational())
                || (require_utilities && (!t.powered || !t.watered))
                || (t.kind == Kind::Station && !self.station_active(i))
                || (self.ext.enabled
                    && matches!(
                        t.kind,
                        Kind::BusDepot | Kind::BusStop | Kind::Station | Kind::SubwayStation
                    )
                    && !self
                        .ext
                        .transport
                        .lines
                        .iter()
                        .any(|l| l.enabled && !l.route.is_empty() && l.stops.contains(&(i as u32))))
            {
                continue;
            }
            providers.push(i);
        }
        if self
            .ext
            .infrastructure_memo
            .roads
            .as_deref()
            .is_none_or(|r| r.as_slice() != roads)
        {
            self.ext.infrastructure_memo.roads = Some(Arc::new(roads.to_vec()));
            self.ext.infrastructure_memo.rasters = vec![None; self.tiles.len()];
            self.ext.infrastructure_memo.raster_count = 0;
        }
        let shape = self.ext.infrastructure_memo.roads.as_ref().unwrap().clone();
        // A source's exact distance influence is independent of its operating
        // state and funding. Precompute unfinished/closed sources too, then an
        // outage only changes a minimum merge. Bound memory and retain exact
        // combined BFS as the fallback for unusually dense service catalogs.
        let missing = candidates
            .iter()
            .filter(|&&i| self.ext.infrastructure_memo.rasters[i].is_none())
            .count();
        let memoize = candidates.len() <= 128
            && (self.ext.infrastructure_memo.raster_count + missing)
                .saturating_mul(self.tiles.len())
                .saturating_mul(2)
                <= 32 * 1024 * 1024;
        if memoize {
            for &i in &candidates {
                if self.ext.infrastructure_memo.rasters[i].is_none() {
                    let raster = self.provider_distances(roads, &[i]);
                    self.ext.infrastructure_memo.rasters[i] = Some(Arc::new(raster));
                    self.ext.infrastructure_memo.raster_count += 1;
                }
            }
        }
        if let Some(entry) = self.ext.infrastructure_memo.services.iter().find(|e| {
            e.kinds == kinds
                && Arc::ptr_eq(&e.roads, &shape)
                && e.providers == providers
                && e.range == range
                && e.budget == budget
        }) {
            let values = entry.values.as_ref().clone();
            return values;
        }
        if providers.is_empty() {
            let result = vec![0; self.tiles.len()];
            self.cache_coverage(kinds, shape, providers, range, budget, &result);
            return result;
        }
        let best = if memoize {
            let mut best = vec![FAR; self.tiles.len()];
            for &provider in &providers {
                let raster = self.ext.infrastructure_memo.rasters[provider]
                    .as_ref()
                    .unwrap();
                for (value, &influence) in best.iter_mut().zip(raster.iter()) {
                    *value = (*value).min(influence);
                }
            }
            best
        } else {
            self.provider_distances(roads, &providers)
        };
        let mut result = vec![0; self.tiles.len()];
        for (i, &d) in best.iter().enumerate() {
            if d <= range {
                result[i] = (100 - d as u32 * 45 / (range as u32 + 1))
                    .min(budget as u32)
                    .min(100) as u8;
            }
        }
        self.cache_coverage(kinds, shape, providers, range, budget, &result);
        result
    }
    fn provider_distances(&self, roads: &[bool], providers: &[usize]) -> Vec<u16> {
        let mut dist = vec![FAR; self.tiles.len()];
        let mut queue = VecDeque::new();
        for &i in providers {
            let x = (i % self.width as usize) as i32;
            let y = (i / self.width as usize) as i32;
            for dy in -3i32..=3 {
                for dx in -3i32..=3 {
                    let d = (dx.abs() + dy.abs()) as u16;
                    if d > 3 {
                        continue;
                    }
                    if let Some(j) = self.idx(x + dx, y + dy) {
                        if roads[j] && d < dist[j] {
                            dist[j] = d;
                            queue.push_back(j);
                        }
                    }
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            for j in self.neighbor_array(i).into_iter().flatten() {
                if roads[j] && dist[j] > dist[i] + 1 {
                    dist[j] = dist[i] + 1;
                    queue.push_back(j);
                }
            }
        }
        // Three synchronous relaxation passes are exactly the final <=3-tile
        // walk from a service road, without per-cell 7x7 searches.
        let mut best = dist;
        let mut scratch = best.clone();
        let w = self.width as usize;
        for _ in 0..3 {
            for i in 0..best.len() {
                let mut v = best[i];
                if i % w > 0 {
                    v = v.min(best[i - 1].saturating_add(1));
                }
                if i % w + 1 < w {
                    v = v.min(best[i + 1].saturating_add(1));
                }
                if i >= w {
                    v = v.min(best[i - w].saturating_add(1));
                }
                if i + w < best.len() {
                    v = v.min(best[i + w].saturating_add(1));
                }
                scratch[i] = v;
            }
            std::mem::swap(&mut best, &mut scratch);
        }
        best
    }
    fn cache_coverage(
        &mut self,
        kinds: &[Kind],
        roads: Arc<Vec<bool>>,
        providers: Vec<usize>,
        range: u16,
        budget: u16,
        values: &[u8],
    ) {
        let entry = CoverageEntry {
            kinds: kinds.to_vec(),
            roads,
            providers,
            range,
            budget,
            values: Arc::new(values.to_vec()),
        };
        if let Some(existing) = self
            .ext
            .infrastructure_memo
            .services
            .iter_mut()
            .find(|e| e.kinds == kinds)
        {
            *existing = entry;
        } else {
            self.ext.infrastructure_memo.services.push(entry);
        }
    }
    fn utility_demand(t: &Tile, water: bool) -> u32 {
        if !t.kind.is_building() {
            return 0;
        }
        let inhabitants = t.population + t.jobs;
        if water {
            if t.kind == Kind::Pump {
                0
            } else {
                inhabitants + 10
            }
        } else {
            if matches!(
                t.kind,
                Kind::Coal
                    | Kind::Wind
                    | Kind::Nuclear
                    | Kind::Gas
                    | Kind::Solar
                    | Kind::Hydro
                    | Kind::Fusion
            ) {
                0
            } else {
                inhabitants / 3 + 8
            }
        }
    }
    fn utility(&mut self, water: bool) -> (u32, u32) {
        let modifiers = self.civic.modifiers();
        let demand_percent = if water {
            modifiers.water_demand_percent
        } else {
            modifiers.power_demand_percent
        } as u32;
        let mut import_remaining = if !water {
            self.ext
                .neighbors
                .iter()
                .filter(|n| n.enabled)
                .map(|n| n.power)
                .sum::<u32>()
        } else {
            0
        };
        let mut imported = 0u32;
        let grids = self.utility_grid(water);
        let mut total_supply = 0u32;
        let mut total_demand = 0u32;
        for members in &grids {
            let mut supply = 0u32;
            let mut demand = 0u32;
            let mut outside = false;
            for &i in members {
                let t = self.tiles[i];
                demand += Self::utility_demand(&t, water) * demand_percent / 100;
                if !water
                    && !outside
                    && (t.kind.is_road() || matches!(t.kind, Kind::Rail | Kind::PowerLine))
                    && (i % self.width as usize == 0
                        || i % self.width as usize + 1 == self.width as usize
                        || i / self.width as usize == 0
                        || i / self.width as usize + 1 == self.height as usize)
                {
                    outside = true;
                }
                if water {
                    if matches!(t.kind, Kind::Pump | Kind::WaterTower)
                        && t.connected
                        && t.fire == 0
                        && t.powered
                        && self.ext.cells.get(i).is_none_or(|c| c.operational())
                        && (t.kind == Kind::WaterTower || self.near_water(i, 3))
                    {
                        supply += if t.kind == Kind::Pump { 14000 } else { 4500 }
                            * self.funding[3].min(150) as u32
                            / 100;
                    }
                } else {
                    supply += self.power_output(i);
                }
            }
            if outside && !water {
                let buy = import_remaining.min(demand.saturating_sub(supply));
                import_remaining -= buy;
                imported += buy;
                supply += buy;
            }
            total_supply += supply;
            total_demand += demand;
            if supply == 0 || supply >= demand {
                let served = supply > 0;
                for &i in members {
                    if water {
                        self.tiles[i].watered = served;
                    } else {
                        self.tiles[i].powered = served;
                    }
                }
                continue;
            }
            // Rotate brownouts each month instead of permanently starving one
            // district. Each disconnected component has its own finite capacity.
            let pivot = (self.tiles.len() - (self.month as usize * 37) % self.tiles.len())
                % self.tiles.len();
            let cut = members.partition_point(|&i| i < pivot);
            let mut remaining = supply;
            for &i in members[cut..].iter().chain(&members[..cut]) {
                let need = Self::utility_demand(&self.tiles[i], water) * demand_percent / 100;
                let served = supply > 0 && remaining >= need;
                if served {
                    remaining -= need;
                }
                if water {
                    self.tiles[i].watered = served;
                } else {
                    self.tiles[i].powered = served;
                }
            }
        }
        if self.ext.enabled {
            // Return the borrowed-out component storage unchanged. Brownout
            // rotation only changes iteration order, never the saved topology.
            self.ext.utility_grids[usize::from(water)] = grids;
        }
        // A road's underground infrastructure feeds the neighboring lots.
        // Empty lots are deliberately not conductors: remote development still
        // needs a road, a power line or a pipe all the way to its block.
        if !water {
            for neighbor in &mut self.ext.neighbors {
                neighbor.last_power = if neighbor.enabled {
                    let n = neighbor.power.min(imported);
                    imported -= n;
                    n
                } else {
                    0
                };
                neighbor.monthly_cost = neighbor.last_power as i64 * neighbor.unit_price as i64
                    / 100
                    - neighbor.last_goods as i64 * neighbor.tariff as i64 / 100;
            }
        }
        (total_supply, total_demand)
    }
    /// Local ore and oil provide the high-productivity extraction/manufacturing
    /// mix. Exhausted deposits retain 40% light/recycled-goods productivity.
    pub fn industrial_productivity(&self, i: usize) -> u8 {
        self.ext.cells.get(i).map_or(100, |c| {
            (40 + (c.ore as u32 + c.oil as u32).min(200) * 60 / 200) as u8
        })
    }
    /// Actual generated supply before network allocation or neighbor imports.
    pub fn power_output(&self, i: usize) -> u32 {
        let Some(t) = self.tiles.get(i) else {
            return 0;
        };
        let output = match t.kind {
            Kind::Coal => 7500,
            Kind::Wind => 1300,
            Kind::Nuclear => 35000,
            Kind::Gas => 12000,
            Kind::Solar => 3200,
            Kind::Hydro => 18000,
            Kind::Fusion => 90000,
            Kind::Incinerator => 1600,
            _ => 0,
        };
        if output == 0
            || !t.connected
            || t.fire > 0
            || self.ext.cells.get(i).is_some_and(|c| !c.operational())
        {
            return 0;
        }
        let output = if self.ext.enabled {
            match t.kind {
                Kind::Wind => {
                    output
                        * (35
                            + (self.ext.weather.wind_x.unsigned_abs() as u32
                                + self.ext.weather.wind_y.unsigned_abs() as u32)
                                * 35)
                        / 100
                }
                Kind::Solar => {
                    output
                        * (100 - self.ext.weather.cloud as u32)
                        * if (360..=1080).contains(&self.ext.minute_of_day) {
                            100
                        } else {
                            15
                        }
                        / 10000
                }
                Kind::Hydro => {
                    if self.near_water(i, 2) {
                        output * (50 + self.ext.weather.rain as u32) / 100
                    } else {
                        0
                    }
                }
                _ => output,
            }
        } else {
            output
        };
        output * self.funding[3].min(150) as u32 / 100
    }
    fn utility_grid(&mut self, water: bool) -> Vec<Vec<usize>> {
        let slot = usize::from(water);
        let mask: Vec<u8> = (0..self.tiles.len())
            .map(|i| u8::from(self.conducts(i, water)))
            .collect();
        if self.ext.enabled && self.ext.utility_masks[slot] == mask {
            return std::mem::take(&mut self.ext.utility_grids[slot]);
        }
        let mut visited = vec![false; self.tiles.len()];
        let mut grids = Vec::new();
        for start in 0..self.tiles.len() {
            if visited[start] || mask[start] == 0 {
                continue;
            }
            visited[start] = true;
            let mut members = vec![];
            let mut queue = VecDeque::from([start]);
            while let Some(i) = queue.pop_front() {
                members.push(i);
                for j in self.neighbors(i) {
                    if !visited[j] && mask[j] != 0 {
                        visited[j] = true;
                        queue.push_back(j);
                    }
                }
            }
            members.sort_unstable();
            grids.push(members);
        }
        if self.ext.enabled {
            self.ext.utility_masks[slot] = mask;
        }
        grids
    }
    fn conducts(&self, i: usize, water: bool) -> bool {
        let kind = self.tiles[i].kind;
        (self
            .ext
            .cells
            .get(i)
            .is_some_and(|c| if water { c.pipe } else { c.underground_power }))
            || (kind != Kind::Empty
                && if water {
                    kind != Kind::PowerLine
                } else {
                    kind != Kind::Pipe
                })
    }
    fn cache_kind_indices(&mut self) {
        let kinds: Vec<Kind> = self.tiles.iter().map(|t| t.kind).collect();
        if self.ext.infrastructure_memo.kinds.as_deref() == Some(&kinds) {
            return;
        }
        let mut buckets = vec![Vec::new(); Kind::ALL.len()];
        for (i, &kind) in kinds.iter().enumerate() {
            buckets[kind as usize].push(i);
        }
        self.ext.infrastructure_memo.kinds = Some(Arc::new(kinds));
        self.ext.infrastructure_memo.by_kind = Arc::new(buckets);
    }
    fn service_maps(&mut self, roads: &[bool], extended: bool) -> [Vec<u8>; 10] {
        self.cache_kind_indices();
        let police = self.service(roads, &[Kind::Police, Kind::Prison], 32, 1, true);
        let fire = self.service(roads, &[Kind::Fire, Kind::FireHQ], 34, 1, true);
        let health = self.service(roads, &[Kind::Hospital, Kind::Clinic], 34, 1, true);
        let school = self.service(roads, &[Kind::School, Kind::Library], 28, 2, true);
        let university = self.service(roads, &[Kind::University], 42, 2, true);
        let park = self.service(
            roads,
            &[
                Kind::Park,
                Kind::Stadium,
                Kind::Playground,
                Kind::Plaza,
                Kind::BotanicalGarden,
                Kind::DogPark,
                Kind::SkatePark,
                Kind::WaterfrontPark,
                Kind::NatureReserve,
                Kind::Fountain,
                Kind::Beach,
                Kind::Campground,
            ],
            14,
            1,
            false,
        );
        let transit = self.service(
            roads,
            &[
                Kind::BusDepot,
                Kind::BusStop,
                Kind::Station,
                Kind::SubwayStation,
            ],
            36,
            0,
            true,
        );
        let (library, prison, cemetery) = if extended {
            let library = self.service(roads, &[Kind::Library], 26, 2, true);
            let prison = self.service(roads, &[Kind::Prison], 44, 1, true);
            let cemetery = self.service(roads, &[Kind::Cemetery], 36, 1, true);
            (library, prison, cemetery)
        } else {
            (vec![], vec![], vec![])
        };
        [
            police, fire, health, school, university, park, transit, library, prison, cemetery,
        ]
    }
    /// Populate derived memoization without changing coverage, utility flags,
    /// pending readiness changes, counters or any other authoritative field.
    pub fn warm_infrastructure_cache(&mut self) {
        if !self.ext.enabled {
            return;
        }
        let mut ext = std::mem::take(&mut self.ext);
        ext.warm_shore(self);
        self.ext = ext;
        let _ = self.cached_road_network();
        let roads: Vec<bool> = self
            .tiles
            .iter()
            .map(|t| t.kind.is_road() && t.connected)
            .collect();
        let _ = self.service_maps(&roads, true);
        let _ = self.environment_neighborhood();
    }
    fn infrastructure(&mut self) -> Networks {
        let (roads, access) = self.cached_road_network();
        for (i, &distance) in access.iter().enumerate() {
            self.tiles[i].connected = distance <= 3;
            self.tiles[i].powered = false;
            self.tiles[i].watered = false;
        }
        (self.power_supply, self.power_demand) = self.utility(false);
        (self.water_supply, self.water_demand) = self.utility(true);
        let [police, fire, health, school, university, park, transit, library, prison, cemetery] =
            self.service_maps(&roads, self.ext.enabled);
        if self.ext.enabled {
            self.ext.coverage.resize(self.tiles.len(), [0; 13]);
            for i in 0..self.tiles.len() {
                self.ext.coverage[i] = [
                    police[i],
                    fire[i],
                    health[i],
                    school[i],
                    university[i],
                    park[i],
                    transit[i],
                    library[i],
                    prison[i],
                    cemetery[i],
                    u8::from(self.tiles[i].watered) * 100,
                    u8::from(self.tiles[i].powered) * 100,
                    100 - self.garbage,
                ];
            }
        }
        Networks {
            roads,
            access,
            police,
            fire,
            health,
            school,
            university,
            park,
            transit,
        }
    }
    /// Recompute infrastructure and statistics after budget or policy changes;
    /// no time passes and the treasury is not charged.
    pub fn refresh(&mut self) {
        if self.ext.enabled {
            let mut ext = std::mem::take(&mut self.ext);
            ext.rebuild_shore(self);
            self.ext = ext;
        }
        self.population = self.tiles.iter().map(|t| t.population).sum();
        self.jobs = self.tiles.iter().map(|t| t.jobs).sum();
        let network = self.infrastructure();
        self.update_environment(&network);
        self.update_economy();
        if self.ext.enabled {
            let mut ext = std::mem::take(&mut self.ext);
            ext.sync_buildings(self);
            self.ext = ext;
        }
        self.ext.infrastructure_stamp = self.infrastructure_stamp();
    }
    fn infrastructure_stamp(&self) -> u64 {
        let mut stamp = 0xcbf29ce484222325u64;
        let mut add = |v: u64| {
            stamp = (stamp ^ v).wrapping_mul(0x100000001b3);
        };
        for (i, t) in self.tiles.iter().enumerate() {
            let provider = t.kind.is_building() && !t.kind.is_zone();
            add(t.kind as u64
                | (t.terrain as u64) << 8
                | u64::from(t.rail_crossing) << 10
                | if provider {
                    u64::from(t.connected) << 11
                        | u64::from(t.powered) << 12
                        | u64::from(t.watered) << 13
                        | u64::from(t.fire > 0) << 14
                        | u64::from(self.ext.cells.get(i).is_none_or(|c| c.operational())) << 15
                } else {
                    0
                });
        }
        for &f in &self.funding {
            add(u64::from(f));
        }
        for line in &self.ext.transport.lines {
            add(line.id as u64
                | u64::from(line.enabled) << 32
                | u64::from(!line.route.is_empty()) << 33);
            for &stop in &line.stops {
                add(u64::from(stop));
            }
        }
        stamp
    }
    fn operational(&self, kind: Kind) -> u32 {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| t.kind == kind && crate::transport::facility_operational(self, *i))
            .count() as u32
    }
    fn environment_neighborhood(&mut self) -> Option<Arc<EnvironmentMemo>> {
        if !self.ext.enabled || self.ext.cells.len() != self.tiles.len() {
            return None;
        }
        let key: Vec<u8> = self
            .tiles
            .iter()
            .zip(&self.ext.cells)
            .map(|(t, c)| {
                t.terrain as u8
                    | u8::from(t.terrain == Terrain::Forest && c.tree_age >= 36) << 2
                    | u8::from(c.shore_distance <= 2) << 3
            })
            .collect();
        if let Some(cached) = self
            .ext
            .infrastructure_memo
            .environment
            .as_ref()
            .filter(|m| m.width == self.width && m.key == key)
        {
            return Some(cached.clone());
        }
        let trees = (0..self.tiles.len())
            .map(|i| self.neighbors(i).filter(|&j| key[j] & 4 != 0).count() as u8 * 3)
            .collect();
        let scenic_water = key
            .iter()
            .map(|&k| u8::from(k & 3 == Terrain::Hill as u8) * 8 + u8::from(k & 8 != 0) * 7)
            .collect();
        let cached = Arc::new(EnvironmentMemo {
            width: self.width,
            key,
            trees,
            scenic_water,
        });
        self.ext.infrastructure_memo.environment = Some(cached.clone());
        Some(cached)
    }
    fn update_environment(&mut self, n: &Networks) {
        let neighborhood = self.environment_neighborhood();
        let (mut waste_capacity, mut sewage_capacity) = (0u32, 0u32);
        for (i, t) in self.tiles.iter().enumerate() {
            let (waste, sewage) = match t.kind {
                Kind::Landfill => (5000, 0),
                Kind::Incinerator => (9000, 0),
                Kind::Recycling => (7500, 0),
                Kind::Sewage => (0, 16000),
                Kind::SewageOutlet => (0, 5000),
                _ => continue,
            };
            if crate::transport::facility_operational(self, i) {
                waste_capacity += waste;
                sewage_capacity += sewage;
            }
        }
        let landfill = waste_capacity * self.funding[3].min(150) as u32 / 100;
        let garbage_capacity = landfill * if self.policies[2] { 2 } else { 1 };
        if !self.ext.enabled || self.ext.services.day == 0 {
            self.garbage = ((self.population.saturating_sub(garbage_capacity)) * 100
                / self.population.max(1))
            .min(100) as u8;
        }
        let sewage = sewage_capacity * self.funding[3].min(150) as u32 / 100;
        let sewage_penalty =
            self.water_demand.saturating_sub(sewage) * 25 / self.water_demand.max(1);
        let workforce = self.population * 55 / 100;
        self.unemployment = if self.ext.enabled && !self.ext.people.households.is_empty() {
            self.ext.people.unemployment
        } else {
            (workforce.saturating_sub(self.jobs) * 100 / workforce.max(1)).min(100) as u8
        };
        let mut pollution = if self.ext.enabled {
            Vec::new()
        } else {
            vec![0u16; self.tiles.len()]
        };
        // Expanded worlds already transport pollution with saved wind/weather.
        if !self.ext.enabled {
            for i in 0..self.tiles.len() {
                let t = self.tiles[i];
                let mut source = match t.kind {
                    Kind::Industrial if t.jobs > 0 => 24 + t.level as u16 * 10,
                    Kind::Coal if t.connected => 95,
                    Kind::Landfill => 65,
                    Kind::Sewage => 25,
                    Kind::Airport => 45,
                    _ => 0,
                };
                if self.policies[0] {
                    source = source * 60 / 100;
                }
                if source == 0 {
                    continue;
                }
                let x = (i % (self.width as usize)) as i32;
                let y = (i / (self.width as usize)) as i32;
                for dy in -5i32..=5 {
                    for dx in -5i32..=5 {
                        let d = (dx.abs() + dy.abs()) as u16;
                        if d > 5 {
                            continue;
                        }
                        if let Some(j) = self.idx(x + dx, y + dy) {
                            // Multiple factories do not cause an instant 100% blanket.
                            pollution[j] = pollution[j].saturating_add(source / (d * d + 2));
                        }
                    }
                }
            }
        }
        let mut health_total = 0u64;
        let mut school_total = 0u64;
        let mut happy_total = 0u64;
        let mut transit_total = 0u64;
        // The expanded transport simulation has already routed actual trips.
        // Keep its congestion rather than replacing it with a second estimate.
        let mut traffic = if self.ext.enabled {
            self.tiles.iter().map(|t| t.traffic).collect::<Vec<_>>()
        } else {
            let mut traffic_sources = vec![0u32; self.tiles.len()];
            // Jobs and residents make trips to their nearest connected street. Trip
            // pressure then spreads along the road graph, showing real bottlenecks.
            for i in 0..self.tiles.len() {
                let t = self.tiles[i];
                let trips = t.population / 3 + t.jobs / 2;
                if trips == 0 || n.access[i] > 3 {
                    continue;
                }
                let mut nearest = None;
                for dy in -3i32..=3 {
                    for dx in -3i32..=3 {
                        if dx.abs() + dy.abs() > 3 {
                            continue;
                        }
                        if let Some(j) = self.idx(
                            (i % (self.width as usize)) as i32 + dx,
                            (i / (self.width as usize)) as i32 + dy,
                        ) {
                            if n.roads[j]
                                && nearest.is_none_or(|k| self.distance(i, j) < self.distance(i, k))
                            {
                                nearest = Some(j);
                            }
                        }
                    }
                }
                if let Some(j) = nearest {
                    let reduction =
                        n.transit[i] as u32 * if self.policies[1] { 65 } else { 40 } / 100;
                    traffic_sources[j] += trips * (100 - reduction) / 100;
                }
            }
            let old_traffic = traffic_sources.clone();
            for (i, trips) in traffic_sources.iter_mut().enumerate() {
                if n.roads[i] {
                    for j in self.neighbor_array(i).into_iter().flatten() {
                        if n.roads[j] {
                            *trips += old_traffic[j] / 2;
                        }
                    }
                }
            }
            let mut traffic = vec![0u8; self.tiles.len()];
            for i in 0..self.tiles.len() {
                if n.roads[i] {
                    let capacity = if self.tiles[i].kind == Kind::Avenue {
                        320
                    } else {
                        125
                    };
                    let budget_capacity = capacity * self.funding[0].clamp(15, 150) as u32 / 100;
                    traffic[i] = (traffic_sources[i] * 100 / budget_capacity.max(1)).min(100) as u8;
                }
            }
            traffic
        };
        let width = usize::from(self.width);
        for row in (0..self.tiles.len()).step_by(width) {
            for i in row..(row + width).min(self.tiles.len()) {
                let t = self.tiles[i];
                if !n.roads[i] {
                    let x = i - row;
                    let mut adjacent = 0;
                    if x > 0 {
                        adjacent = adjacent.max(traffic[i - 1]);
                    }
                    if x + 1 < width {
                        adjacent = adjacent.max(traffic[i + 1]);
                    }
                    if row > 0 {
                        adjacent = adjacent.max(traffic[i - width]);
                    }
                    if i + width < traffic.len() {
                        adjacent = adjacent.max(traffic[i + width]);
                    }
                    traffic[i] = adjacent / 2;
                }
                let (trees, scenic_water) = if let Some(m) = &neighborhood {
                    (m.trees[i] as i32, m.scenic_water[i] as i32)
                } else {
                    (
                        self.neighbors(i)
                            .filter(|&j| {
                                self.tiles[j].terrain == Terrain::Forest
                                    && self.ext.cells.get(j).is_none_or(|c| c.tree_age >= 36)
                            })
                            .count() as i32
                            * 3,
                        if self.tiles[i].terrain == Terrain::Hill {
                            8
                        } else {
                            0
                        } + if self.near_water(i, 2) { 7 } else { 0 },
                    )
                };
                let p = (if self.ext.enabled {
                    0
                } else {
                    pollution[i] as i32
                } + self
                    .ext
                    .cells
                    .get(i)
                    .map_or(0, |c| c.air_pollution as i32 + c.radiation as i32 / 2)
                    + self.garbage as i32 / 3
                    + traffic[i] as i32 / 9
                    - trees)
                    .clamp(0, 100);
                let crime = (12
                    + self.unemployment as i32 / 2
                    + t.level as i32 * 5
                    + (self.taxes[0] as i32 - 9).max(0) * 2
                    - n.police[i] as i32 * 45 / 100
                    - n.school[i] as i32 / 10)
                    .clamp(0, 100);
                let value = (38
                    + n.park[i] as i32 / 3
                    + n.school[i] as i32 / 8
                    + n.health[i] as i32 / 10
                    + scenic_water
                    + trees
                    - p * 2 / 3
                    - crime / 3
                    - traffic[i] as i32 / 6
                    - self.garbage as i32 / 5)
                    .clamp(0, 100);
                self.tiles[i].pollution = p as u8;
                self.tiles[i].crime = crime as u8;
                self.tiles[i].traffic = traffic[i];
                self.tiles[i].land_value = value as u8;
                if t.population > 0 {
                    let health = (65 + n.health[i] as i32 * 30 / 100
                        - p / 3
                        - sewage_penalty as i32
                        - if t.watered { 0 } else { 35 }
                        - self.garbage as i32 / 5)
                        .clamp(0, 100);
                    let education = (n.school[i] as u32 * 65 / 100
                        + n.university[i] as u32 * 35 / 100)
                        .min(100);
                    let happy = (35
                        + value / 2
                        + health / 6
                        + education as i32 / 10
                        + n.park[i] as i32 / 12
                        - self.unemployment as i32 / 4
                        - (self.taxes[0] as i32 - 9) * 3
                        - if t.powered { 0 } else { 25 }
                        - if t.watered { 0 } else { 25 }
                        - if t.connected { 0 } else { 20 }
                        - if t.fire == 0 { 0 } else { 35 })
                    .clamp(0, 100);
                    health_total += health as u64 * t.population as u64;
                    school_total += education as u64 * t.population as u64;
                    happy_total += happy as u64 * t.population as u64;
                    transit_total += n.transit[i] as u64 * t.population as u64 / 2;
                }
            }
        }
        if self.population > 0 {
            self.health = (health_total / self.population as u64) as u8;
            self.education = (school_total / self.population as u64) as u8;
            self.happiness = (happy_total / self.population as u64) as u8;
            self.transport_usage = (transit_total / self.population as u64).min(100) as u8;
        } else {
            self.health = 75;
            self.education = 0;
            self.happiness = 65;
            self.transport_usage = 0;
        }
        self.demand = crate::demand::values(self);
    }
    fn update_economy(&mut self) {
        let mut income = 0i64;
        let mut expenses = 0i64;
        let mut regional = [0i64; 3];
        for (i, t) in self.tiles.iter().enumerate() {
            let venue = match t.kind {
                Kind::Seaport => Some(0),
                Kind::Airport => Some(1),
                Kind::Stadium => Some(2),
                _ => None,
            };
            if let Some(venue) = venue.filter(|_| crate::transport::facility_operational(self, i)) {
                regional[venue] += 1;
            }
            income += match t.kind {
                Kind::Residential => t.population as i64 * self.taxes[0] as i64 / 40,
                Kind::Commercial => t.jobs as i64 * self.taxes[1] as i64 / 20,
                Kind::Industrial | Kind::Farm => t.jobs as i64 * self.taxes[2] as i64 / 25,
                Kind::Hotel => t.jobs as i64 * self.taxes[1] as i64 / 20,
                _ => 0,
            };
            expenses += t.kind.upkeep() * self.funding[t.kind.funding()].min(150) as i64 / 100;
            if t.rail_crossing {
                expenses += Kind::Rail.upkeep() * self.funding[0].min(150) as i64 / 100;
            }
        }
        // Freight, visitors and fares are useful revenue streams, not decorations.
        income += regional[0] * (80 + self.jobs as i64 / 20);
        income += regional[1] * (100 + self.population as i64 / 25);
        income += regional[2] * (35 + self.population as i64 / 80);
        if !self.ext.enabled && !self.policies[1] {
            income += self.transport_usage as i64 * self.population as i64 / 1800;
        }
        expenses += self.debt / 100; // 1% monthly interest; principal is repaid explicitly.
        if self.policies[0] {
            expenses += 30 + self.population as i64 / 100;
        }
        if self.policies[1] {
            expenses += 20 + self.population as i64 / 70;
        }
        if self.policies[2] {
            expenses += 25 + self.population as i64 / 100;
        }
        if self.policies[3] {
            expenses += 80 + self.jobs as i64 / 80;
        }
        for neighbor in &self.ext.neighbors {
            if neighbor.monthly_cost >= 0 {
                expenses += neighbor.monthly_cost;
            } else {
                income -= neighbor.monthly_cost;
            }
        }
        if self.ext.enabled {
            income += self.ext.trade_income
                + self.ext.transport.fare_balance
                + self.ext.transport.air_revenue_balance;
            expenses += self.ext.operating_cost;
        }
        self.income = income;
        self.expenses = expenses;
    }
    fn cached_networks(&self) -> Networks {
        let col = |n: usize| self.ext.coverage.iter().map(|r| r[n]).collect::<Vec<_>>();
        Networks {
            roads: self
                .tiles
                .iter()
                .map(|t| t.kind.is_road() && t.connected)
                .collect(),
            access: self
                .tiles
                .iter()
                .map(|t| {
                    if t.kind.is_road() && t.connected {
                        0
                    } else if t.connected {
                        3
                    } else {
                        FAR
                    }
                })
                .collect(),
            police: col(0),
            fire: col(1),
            health: col(2),
            school: col(3),
            university: col(4),
            park: col(5),
            transit: col(6),
        }
    }
    fn refresh_statistics(&mut self) {
        if !self.ext.enabled || self.ext.coverage.len() != self.tiles.len() {
            self.refresh();
            return;
        }
        self.population = self.tiles.iter().map(|t| t.population).sum();
        self.jobs = self.tiles.iter().map(|t| t.jobs).sum();
        (self.power_supply, self.power_demand) = self.utility(false);
        (self.water_supply, self.water_demand) = self.utility(true);
        if self.ext.infrastructure_stamp != self.infrastructure_stamp() {
            self.refresh();
            return;
        }
        for (t, c) in self.tiles.iter().zip(&mut self.ext.coverage) {
            c[10] = u8::from(t.watered) * 100;
            c[11] = u8::from(t.powered) * 100;
            c[12] = 100 - self.garbage;
        }
        let n = self.cached_networks();
        self.update_environment(&n);
        self.update_economy();
    }
    pub fn advance_month(&mut self) {
        self.month = self.month.saturating_add(1);
        let old_population = self.population;
        let legacy_fire = if self.ext.enabled && self.ext.coverage.len() == self.tiles.len() {
            None
        } else {
            let n = self.infrastructure();
            self.update_environment(&n);
            Some(n.fire)
        };
        let max_level = if self.population >= 35000 {
            5
        } else if self.population >= 15000 {
            4
        } else if self.population >= 5000 {
            3
        } else if self.population >= 1400 {
            2
        } else {
            1
        };
        let port = self.operational(Kind::Seaport) > 0;
        let airport = self.operational(Kind::Airport) > 0;
        let university = self.operational(Kind::University) > 0;
        let demand = self.demand;
        let housing_capacity_percent = self.civic.modifiers().housing_capacity_percent as u32;
        for i in 0..self.tiles.len() {
            let t = self.tiles[i];
            if !t.kind.is_zone() {
                continue;
            }
            let which = match t.kind {
                Kind::Residential => 0,
                Kind::Commercial | Kind::Hotel => 1,
                _ => 2,
            };
            let livable = t.connected
                && t.powered
                && t.watered
                && t.fire == 0
                && self
                    .ext
                    .cells
                    .get(i)
                    .is_none_or(|c| c.construction == 100 && !c.rubble && !c.abandoned);
            let local = match t.kind {
                Kind::Residential => {
                    t.land_value as i32 / 3 - t.pollution as i32 / 2 - t.crime as i32 / 3
                }
                Kind::Commercial | Kind::Hotel => {
                    t.land_value as i32 / 5 - t.traffic as i32 / 4 + if airport { 15 } else { 0 }
                }
                _ => 8 + if port { 20 } else { 0 } - t.traffic as i32 / 3,
            };
            let score = demand[which] + local;
            let should_grow = livable && score > 0 && self.money > -10000;
            if should_grow {
                let quality = t.kind == Kind::Industrial || t.land_value >= 35;
                let educated =
                    t.level < 2 || self.education >= 25 || (t.kind == Kind::Industrial && port);
                let highrise = t.level < 3 || university || (t.kind == Kind::Commercial && airport);
                if t.level == 0
                    || (t.level < max_level
                        && quality
                        && educated
                        && highrise
                        && self.month % 4 == (i % 4) as u32
                        && score > 20)
                {
                    self.tiles[i].level = t.level.saturating_add(1).min(max_level);
                }
                let cap = capacity(t.kind, self.tiles[i].level)
                    * if t.kind == Kind::Residential {
                        housing_capacity_percent
                    } else {
                        100
                    }
                    / 100;
                let current = if which == 0 { t.population } else { t.jobs };
                let growth =
                    (3 + cap / 14 + score.max(0) as u32 / 18).min(cap.saturating_sub(current));
                if which == 0 {
                    self.tiles[i].population += growth;
                    if self.ext.enabled {
                        self.ext.people.migrants_in += growth as u64;
                    }
                } else {
                    self.tiles[i].jobs += growth;
                }
            } else {
                let current = if which == 0 { t.population } else { t.jobs };
                let loss = if !livable {
                    (current / 5).max(3)
                } else if score < -5 {
                    (current / 12).max(2)
                } else {
                    0
                };
                let next = current.saturating_sub(loss);
                if which == 0 {
                    self.tiles[i].population = next;
                    if self.ext.enabled {
                        self.ext.people.migrants_out += (current - next) as u64;
                    }
                } else {
                    self.tiles[i].jobs = next;
                }
                if next == 0 {
                    self.tiles[i].level = 0;
                }
            }
            if let Some(cell) = self.ext.cells.get_mut(i) {
                let tile = &mut self.tiles[i];
                let occupied = tile.population + tile.jobs;
                let cap = capacity(tile.kind, tile.level)
                    * if tile.kind == Kind::Residential {
                        housing_capacity_percent
                    } else {
                        100
                    }
                    / 100;
                if !should_grow && (score < 0 || !livable) && occupied < cap / 3 {
                    cell.density_stress = cell.density_stress.saturating_add(1);
                } else {
                    cell.density_stress = cell.density_stress.saturating_sub(1);
                }
                if cell.density_stress >= 3 && tile.level > 1 {
                    tile.level -= 1;
                    cell.density_stress = 0;
                }
            }
        }
        if !self.ext.enabled {
            self.process_fires(legacy_fire.as_ref().unwrap());
        }
        self.refresh_statistics();
        self.money =
            (self.money + self.income - self.expenses).clamp(-1_000_000_000, 1_000_000_000);
        if self.ext.enabled {
            self.ext.transport.fare_balance = 0;
            self.ext.transport.air_revenue_balance = 0;
        }
        if self.money < 0 && self.month.is_multiple_of(6) {
            self.message(
                "The treasury is in deficit. Adjust taxes, reduce spending or issue a bond.",
            );
        }
        for milestone in if self.ext.enabled {
            &[][..]
        } else {
            &[500, 1500, 5000, 15000, 35000][..]
        } {
            let milestone = *milestone;
            if old_population < milestone && self.population >= milestone {
                let grant = milestone as i64 / 2 + 1000;
                self.money += grant;
                self.message(format!(
                    "Population milestone: {milestone} residents! Regional grant: ${grant}."
                ));
            }
        }
        if self.month.is_multiple_of(12) {
            if self.power_supply < self.power_demand {
                self.message("Rolling blackouts are slowing development. Increase power capacity or utility funding.");
            } else if self.water_supply < self.water_demand {
                self.message("Water demand exceeds supply. Add a powered pump near the river.");
            } else if self.garbage > 30 {
                self.message(
                    "Garbage is piling up. A connected landfill or recycling policy will help.",
                );
            } else if self.unemployment > 20 {
                self.message(
                    "Residents need more jobs. Zone commerce and industry near connected roads.",
                );
            }
        }
        // Rare fire risk is tied to actual pollution and underfunded protection.
        if self.month > 24 && self.funding[1] < 75 && random(&mut self.seed).is_multiple_of(120) {
            self.disaster(Disaster::Fire);
        }
        if self.ext.enabled {
            let mut people = std::mem::take(&mut self.ext.people);
            people.month_end(self);
            self.ext.people = people;
            let mut ext = std::mem::take(&mut self.ext);
            ext.sync_buildings(self);
            self.ext = ext;
        }
        let mut civic = std::mem::take(&mut self.civic);
        civic.month_end(self);
        self.civic = civic;
        self.record_history();
    }
    fn process_fires(&mut self, coverage: &[u8]) {
        let mut spread = vec![];
        for (i, &protection) in coverage.iter().enumerate() {
            if self.tiles[i].fire == 0 {
                continue;
            }
            if protection > 30 {
                self.tiles[i].fire = self.tiles[i].fire.saturating_sub(35 + protection / 4);
            } else {
                self.tiles[i].fire = self.tiles[i].fire.saturating_add(18).min(100);
                if self.tiles[i].fire > 65 {
                    for j in self.neighbor_array(i).into_iter().flatten() {
                        if self.tiles[j].kind.is_building()
                            && self.tiles[j].fire == 0
                            && random(&mut self.seed).is_multiple_of(4)
                        {
                            spread.push(j);
                        }
                    }
                }
                if self.tiles[i].fire >= 100 {
                    self.clear(i);
                    if let Some(c) = self.ext.cells.get_mut(i) {
                        c.rubble = true;
                        c.condition = 0;
                        c.construction = 0;
                    }
                }
            }
        }
        for i in spread {
            self.tiles[i].fire = 45;
        }
    }
    fn record_history(&mut self) {
        self.history.push(History {
            population: self.population,
            money: self.money,
            income: self.income,
            expenses: self.expenses,
            happiness: self.happiness,
        });
        if self.history.len() > 120 {
            self.history.remove(0);
        }
    }
    pub fn take_loan(&mut self) -> Result<(), String> {
        if self.debt >= 30000 {
            return Err(
                "The bond limit is $30,000. Repay principal before borrowing again.".into(),
            );
        }
        self.debt += 10000;
        self.money += 10000;
        self.message("Issued a $10,000 city bond. Monthly interest is $100 per bond.");
        self.update_economy();
        Ok(())
    }
    pub fn repay_loan(&mut self) -> Result<(), String> {
        if self.debt == 0 {
            return Err("The city has no outstanding bonds.".into());
        }
        let amount = self.debt.min(10000);
        if self.money < amount {
            return Err(format!("You need ${amount} to repay a bond."));
        }
        self.debt -= amount;
        self.money -= amount;
        self.message(format!("Repaid ${amount} of city bonds."));
        self.update_economy();
        Ok(())
    }
    pub fn disaster(&mut self, disaster: Disaster) {
        let buildings: Vec<usize> = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind.is_building())
            .map(|(i, _)| i)
            .collect();
        if buildings.is_empty() {
            self.message("No developed districts are threatened.");
            return;
        }
        let center = buildings[(random(&mut self.seed) as usize) % buildings.len()];
        match disaster {
            Disaster::FriendlyUfo => {
                self.money += 5000;
                self.health = self.health.saturating_add(10).min(100);
                self.message(
                    "A friendly visitor delivered clean technology and a $5,000 research grant.",
                );
            }
            Disaster::Meteor
            | Disaster::HostileUfo
            | Disaster::PlaneCrash
            | Disaster::Monster
            | Disaster::Riot => {
                let radius = match disaster {
                    Disaster::Meteor => 5,
                    Disaster::HostileUfo => 4,
                    Disaster::Monster => 6,
                    _ => 2,
                };
                let affected: Vec<_> = (0..self.tiles.len())
                    .filter(|&i| {
                        self.distance(i, center) <= radius && self.tiles[i].kind.is_building()
                    })
                    .collect();
                for i in affected {
                    if random(&mut self.seed).is_multiple_of(3) {
                        self.clear(i);
                    } else {
                        self.tiles[i].fire = 65;
                    }
                    if let Some(c) = self.ext.cells.get_mut(i) {
                        c.condition = c.condition.saturating_sub(50);
                        c.rubble = c.condition == 0;
                        c.radiation = if disaster == Disaster::Meteor {
                            70
                        } else {
                            c.radiation
                        };
                    }
                }
                self.message(format!(
                    "{:?} emergency: crews are responding to affected districts.",
                    disaster
                ));
            }
            Disaster::Fire => {
                self.tiles[center].fire = 65;
                self.message("FIRE! Fire crews are responding along connected streets.");
            }
            Disaster::Flood => {
                let mut lost = 0;
                let vulnerable: Vec<usize> = buildings
                    .iter()
                    .copied()
                    .filter(|&i| self.near_water(i, 3))
                    .collect();
                for i in vulnerable {
                    if !random(&mut self.seed).is_multiple_of(3) {
                        self.clear(i);
                        lost += 1;
                    }
                }
                self.message(format!("River flood! {lost} waterfront developments were lost. Rebuild farther from the bank."));
            }
            Disaster::Earthquake => {
                let mut lost = 0;
                for i in 0..self.tiles.len() {
                    if self.distance(i, center) <= 5
                        && self.tiles[i].kind != Kind::Empty
                        && random(&mut self.seed).is_multiple_of(3)
                    {
                        self.clear(i);
                        lost += 1;
                    }
                }
                self.message(format!(
                    "EARTHQUAKE! {lost} developments destroyed. Check broken roads and utilities."
                ));
            }
            Disaster::Tornado => {
                let x = (center % (self.width as usize)) as i32;
                let y = (center / (self.width as usize)) as i32;
                let mut lost = 0;
                for step in -7..=7 {
                    for width in -1..=1 {
                        if let Some(i) = self.idx(x + step, y + step / 2 + width) {
                            if self.tiles[i].kind != Kind::Empty
                                && !random(&mut self.seed).is_multiple_of(4)
                            {
                                self.clear(i);
                                lost += 1;
                            }
                        }
                    }
                }
                self.message(format!("TORNADO! A storm tore through {lost} developments. Reconnect affected districts."));
            }
            Disaster::Meltdown => {
                let nuclear: Vec<usize> = self
                    .tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.kind == Kind::Nuclear)
                    .map(|(i, _)| i)
                    .collect();
                if nuclear.is_empty() {
                    self.message(
                        "Nuclear safety drill complete: the city has no nuclear reactors.",
                    );
                    return;
                }
                let reactor = nuclear[(random(&mut self.seed) as usize) % nuclear.len()];
                for i in 0..self.tiles.len() {
                    if self.distance(i, reactor) <= 4 && self.tiles[i].kind.is_building() {
                        if i == reactor || random(&mut self.seed).is_multiple_of(2) {
                            self.clear(i);
                        } else {
                            self.tiles[i].fire = 80;
                        }
                    }
                }
                self.message("REACTOR MELTDOWN! Nearby districts were evacuated. Restore emergency services and power.");
            }
        }
        self.refresh();
    }
    pub fn validate(&self) -> Result<(), String> {
        if ![48, 128].contains(&self.width)
            || self.height != self.width
            || self.tiles.len() != self.width as usize * self.height as usize
        {
            return Err("Invalid map dimensions.".into());
        }
        if self.taxes.iter().any(|&t| t > 20) || self.funding.iter().any(|&f| f > 150) {
            return Err("Invalid taxes or department funding.".into());
        }
        if !(-1_000_100_000..=1_000_100_000).contains(&self.money)
            || self.debt < 0
            || self.debt > 30000
            || self.debt % 10000 != 0
        {
            return Err("Invalid treasury or bond balance.".into());
        }
        if [
            self.happiness,
            self.health,
            self.education,
            self.garbage,
            self.unemployment,
            self.transport_usage,
        ]
        .iter()
        .any(|&v| v > 100)
            || self.demand.iter().any(|&d| !(-100..=100).contains(&d))
        {
            return Err("Invalid city indicators.".into());
        }
        let housing_capacity_percent = self.civic.modifiers().housing_capacity_percent as u32;
        for t in &self.tiles {
            if t.level > 5
                || t.population
                    > capacity(Kind::Residential, t.level) * housing_capacity_percent / 100
                || t.jobs > capacity(t.kind, t.level)
                || (t.population > 0 && t.kind != Kind::Residential)
                || (t.jobs > 0 && !t.kind.is_building())
                || [t.pollution, t.traffic, t.crime, t.land_value, t.fire]
                    .iter()
                    .any(|&v| v > 100)
            {
                return Err("Invalid district statistics.".into());
            }
            if t.terrain == Terrain::Water && t.kind != Kind::Empty && !t.kind.is_network() {
                return Err("Invalid development on open water.".into());
            }
            if t.rail_crossing && !t.kind.is_road() {
                return Err("Invalid railway crossing.".into());
            }
        }
        if self.population != self.tiles.iter().map(|t| t.population).sum::<u32>()
            || self.jobs != self.tiles.iter().map(|t| t.jobs).sum::<u32>()
        {
            return Err("City totals do not match its districts.".into());
        }
        if self.history.len() > 120
            || self.messages.len() > 16
            || self.messages.iter().any(|s| s.len() > 400)
        {
            return Err("Invalid city history.".into());
        }
        self.ext.validate(self.tiles.len())?;
        for slot in 0..2 {
            let mask = &self.ext.utility_masks[slot];
            let grids = &self.ext.utility_grids[slot];
            if mask.is_empty() {
                if !grids.is_empty() {
                    return Err("Utility component cache has no membership mask.".into());
                }
                continue;
            }
            let mut group = vec![usize::MAX; self.tiles.len()];
            for (id, members) in grids.iter().enumerate() {
                for &i in members {
                    if group[i] != usize::MAX || mask[i] == 0 {
                        return Err("Utility components overlap or contain open land.".into());
                    }
                    group[i] = id;
                }
            }
            if mask
                .iter()
                .enumerate()
                .any(|(i, &v)| (v != 0) != (group[i] != usize::MAX))
            {
                return Err("Utility component cache omits conductors.".into());
            }
            let mut visited = vec![false; self.tiles.len()];
            for (id, members) in grids.iter().enumerate() {
                let mut q = VecDeque::from([members[0]]);
                visited[members[0]] = true;
                let mut count = 0;
                while let Some(i) = q.pop_front() {
                    count += 1;
                    for j in self.neighbors(i) {
                        if mask[j] == 0 {
                            continue;
                        }
                        if group[j] != id {
                            return Err(
                                "Adjacent conductors have inconsistent utility components.".into(),
                            );
                        }
                        if !visited[j] {
                            visited[j] = true;
                            q.push_back(j);
                        }
                    }
                }
                if count != members.len() {
                    return Err("Disconnected conductors share a utility component.".into());
                }
            }
        }
        self.ext.services.validate_city(self)?;
        self.civic.validate(self.tiles.len())?;
        Ok(())
    }
}

pub const SERVICE_LABELS: [&str; 13] = [
    "Police",
    "Fire",
    "Healthcare",
    "Schools",
    "University",
    "Parks",
    "Transit",
    "Library",
    "Prisons",
    "Cemetery",
    "Water",
    "Electricity",
    "Waste",
];

impl Kind {
    pub fn footprint(self) -> [u8; 2] {
        [1, 1]
    }
}
impl City {
    pub fn world(seed: u64, style: MapStyle, climate: crate::world::Climate) -> Self {
        let mut city = Self::new(seed, false);
        city.width = 128;
        city.height = 128;
        city.tiles = generated_terrain(seed, style, 128);
        city.ext = crate::world::WorldState::new(&city, seed, style, climate);
        city.civic = crate::civic::CivicState::new(seed);
        city.history.clear();
        city.messages.clear();
        city.refresh();
        city.record_history();
        city.message(format!(
            "A 128×128 {} region in a {} climate. Connect an outside road to begin.",
            style.label(),
            climate.label()
        ));
        city
    }
    pub fn initialize_world(&mut self) {
        if self.ext.cells.len() != self.tiles.len() {
            let day = self.ext.day;
            let minute = self.ext.minute_of_day;
            self.ext = crate::world::WorldState::new(
                self,
                self.seed,
                MapStyle::River,
                crate::world::Climate::Temperate,
            );
            if self.width == SIZE as u16 {
                for c in &mut self.ext.cells {
                    c.elevation = 0;
                }
            }
            self.ext.day = day.max(self.month as u64 * 30);
            self.ext.minute_of_day = minute;
        }
    }
    pub fn initialize_expansion(&mut self) {
        self.initialize_world();
    }
    pub fn advance_time(&mut self, minutes: u16) {
        self.initialize_world();
        let total = self.ext.minute_of_day as u32 + minutes as u32;
        self.ext.minute_of_day = (total % 1440) as u16;
        for _ in 0..total / 1440 {
            self.advance_day();
        }
    }
    pub fn advance_day(&mut self) {
        self.initialize_world();
        let mut ext = std::mem::take(&mut self.ext);
        ext.advance_day(self);
        self.ext = ext;
        let rebuild = self.ext.infrastructure_stamp != self.infrastructure_stamp();
        if rebuild {
            self.refresh();
        }
        let day = self.ext.day;
        let temperature = self.ext.weather.temperature;
        let mut people = std::mem::take(&mut self.ext.people);
        people.advance_day(self, day, temperature);
        self.ext.people = people;
        let rain = self.ext.weather.rain;
        let mut transport = std::mem::take(&mut self.ext.transport);
        transport.advance_day(self, day, rain);
        self.ext.transport = transport;
        let mut services = std::mem::take(&mut self.ext.services);
        services.advance_day(self);
        self.ext.services = services;
        let mut civic = std::mem::take(&mut self.civic);
        civic.advance_day(self);
        self.civic = civic;
        if day.is_multiple_of(30) {
            self.advance_month();
        } else if day.is_multiple_of(5) {
            self.refresh_statistics();
        }
    }
    pub fn placement_cost(&self, x: i32, y: i32, kind: Kind) -> Result<i64, String> {
        let i = self
            .idx(x, y)
            .ok_or("That location is outside the region.")?;
        let old = self.tiles[i];
        if kind == Kind::Empty {
            if self.ext.enabled {
                self.civic.can_bulldoze(i as u32)?;
            }
            if old.kind == Kind::Empty && !matches!(old.terrain, Terrain::Forest | Terrain::Hill) {
                return Err("There is nothing to bulldoze here.".into());
            }
            let cost = if old.terrain == Terrain::Hill { 40 } else { 15 };
            return if self.money < cost {
                Err("Not enough treasury funds to bulldoze.".into())
            } else {
                Ok(cost)
            };
        }
        if self.ext.enabled
            && old.kind != Kind::Empty
            && matches!(kind, Kind::Pipe | Kind::PowerLine | Kind::Subway)
        {
            let c = &self.ext.cells[i];
            let existing = match kind {
                Kind::Pipe => c.pipe,
                Kind::PowerLine => c.underground_power,
                Kind::Subway => c.subway,
                _ => false,
            };
            if existing {
                return Err("This underground network already exists here.".into());
            }
            return if self.money >= kind.cost() {
                Ok(kind.cost())
            } else {
                Err("Not enough funds for this network.".into())
            };
        }
        if self.ext.cells.get(i).is_some_and(|c| c.rubble) {
            return Err("Clear the rubble before rebuilding this lot.".into());
        }
        if old.kind == kind {
            return Err("That tile already has this development.".into());
        }
        let upgrade = old.kind.is_road() && kind.is_road();
        let crossing = (old.kind.is_road() && kind == Kind::Rail)
            || (old.kind == Kind::Rail && kind.is_road());
        if crossing && old.rail_crossing {
            return Err("This street already has a railway crossing.".into());
        }
        if old.kind != Kind::Empty && !upgrade && !crossing {
            return Err("Bulldoze the existing development first.".into());
        }
        if old.terrain == Terrain::Water && !kind.is_network() {
            return Err("Build on dry land. Roads and rail can bridge water.".into());
        }
        if matches!(
            kind,
            Kind::Pump
                | Kind::Hydro
                | Kind::Seaport
                | Kind::Marina
                | Kind::WaterfrontPark
                | Kind::Beach
                | Kind::SewageOutlet
        ) && !self.near_water(i, if kind == Kind::Pump { 3 } else { 2 })
        {
            return Err("This facility needs a river or coastline nearby.".into());
        }
        if matches!(kind, Kind::Airport | Kind::Airfield) && old.terrain == Terrain::Hill {
            return Err("An airport needs flat land.".into());
        }
        if self.ext.enabled {
            self.civic.can_build(kind, self.population)?;
            let threshold = match kind {
                Kind::Nuclear => 12500,
                Kind::Fusion => 50000,
                Kind::Airport => 8000,
                Kind::Seaport => 6000,
                Kind::University => 4500,
                Kind::Arcology | Kind::Spaceport | Kind::WorldExpo => 22000,
                _ => 0,
            };
            if self.population < threshold {
                return Err(format!(
                    "{} unlocks at {} residents.",
                    kind.label(),
                    threshold
                ));
            }
            if matches!(
                kind,
                Kind::CityHall
                    | Kind::Monument
                    | Kind::ClockTower
                    | Kind::Arcology
                    | Kind::Spaceport
                    | Kind::WorldExpo
            ) && self.tiles.iter().any(|t| t.kind == kind)
            {
                return Err("This landmark is unique to the city.".into());
            }
        }
        let mut cost = if upgrade {
            (kind.cost() - old.kind.cost()).max(15)
        } else {
            kind.cost()
        };
        if old.terrain == Terrain::Water {
            cost *= 3;
        }
        if old.terrain == Terrain::Forest {
            cost += 15;
        }
        if old.terrain == Terrain::Hill {
            cost += 40;
        }
        if self.money < cost {
            return Err(format!("Not enough treasury funds: ${cost} required."));
        }
        Ok(cost)
    }
    pub fn terraform_cost(
        &self,
        x: i32,
        y: i32,
        action: crate::world::TerrainAction,
    ) -> Result<i64, String> {
        use crate::world::TerrainAction as A;
        let i = self.idx(x, y).ok_or("Outside the region.")?;
        let t = &self.tiles[i];
        if t.kind != Kind::Empty {
            return Err("Bulldoze development before terraforming this tile.".into());
        }
        if self.ext.enabled {
            self.civic.can_bulldoze(i as u32)?;
        }
        if action == A::Plant && t.terrain == Terrain::Water {
            return Err("Drain open water before planting trees.".into());
        }
        if action == A::Water && t.terrain == Terrain::Water {
            return Err("This tile is already open water.".into());
        }
        if action == A::Drain && t.terrain != Terrain::Water {
            return Err("Select open water to drain.".into());
        }
        if matches!(action, A::Level | A::Smooth) && t.terrain == Terrain::Water {
            return Err("Drain the water before leveling this tile.".into());
        }
        if let Some(c) = self.ext.cells.get(i) {
            if action == A::Raise && c.elevation >= 48 {
                return Err("Maximum terrain elevation reached.".into());
            }
            if action == A::Lower && c.elevation <= -16 {
                return Err("Minimum terrain elevation reached.".into());
            }
        }
        let cost = action.cost();
        if self.money < cost {
            return Err("Not enough funds for terraforming.".into());
        }
        Ok(cost)
    }
    pub fn terraform(
        &mut self,
        x: i32,
        y: i32,
        action: crate::world::TerrainAction,
    ) -> Result<i64, String> {
        use crate::world::TerrainAction as A;
        let cost = self.terraform_cost(x, y, action)?;
        self.initialize_world();
        let i = self.idx(x, y).unwrap();
        let neighbor_heights: Vec<_> = self
            .neighbors(i)
            .map(|j| self.ext.cells[j].elevation)
            .collect();
        let c = &mut self.ext.cells[i];
        match action {
            A::Raise => c.elevation = (c.elevation + 4).min(48),
            A::Lower => c.elevation = (c.elevation - 4).max(-16),
            A::Level => c.elevation = 8,
            A::Smooth => {
                c.elevation =
                    neighbor_heights.iter().sum::<i16>() / neighbor_heights.len().max(1) as i16
            }
            A::Water => {
                c.elevation = -4;
                c.water_depth = 4;
            }
            A::Drain => {
                c.elevation = 4;
                c.water_depth = 0;
            }
            A::Plant => {
                c.tree_age = 0;
                c.age_days = 0;
                c.tree_species = (crate::world::hash(self.ext.seed, i as u64) % 6) as u8;
            }
            A::Clear => {
                c.tree_age = 0;
                c.age_days = 0;
            }
        }
        self.tiles[i].terrain = if action == A::Plant {
            Terrain::Forest
        } else if c.elevation < 0 {
            Terrain::Water
        } else if c.elevation >= 24 {
            Terrain::Hill
        } else {
            Terrain::Grass
        };
        c.water_depth = if c.elevation < 0 {
            (-c.elevation).min(100) as u8
        } else {
            0
        };
        self.money -= cost;
        self.refresh();
        Ok(cost)
    }
    pub fn coverage(&self, service: usize, i: usize) -> u8 {
        self.ext
            .coverage
            .get(i)
            .and_then(|r| r.get(service))
            .copied()
            .unwrap_or_else(|| {
                let Some(t) = self.tiles.get(i) else {
                    return 0;
                };
                match service {
                    10 => u8::from(t.watered) * 100,
                    11 => u8::from(t.powered) * 100,
                    12 => 100 - self.garbage,
                    _ => 0,
                }
            })
    }
    pub fn service_coverage(&self, i: usize, service: usize) -> Option<u32> {
        self.tiles.get(i)?;
        Some(self.coverage(service, i) as u32)
    }
    pub fn overlay_value(&self, i: usize, mode: u8) -> Option<u32> {
        let t = self.tiles.get(i)?;
        let c = self.ext.cells.get(i);
        let desirability = |bonus: i32, penalty: i32| -> u32 {
            (t.land_value as i32 + bonus - penalty).clamp(0, 100) as u32
        };
        Some(match mode {
            0 => 0,
            1 => u32::from(t.powered) * 100,
            2 => u32::from(t.watered) * 100,
            3 => t.traffic as u32,
            4 => t.pollution as u32,
            5 => t.land_value as u32,
            6 => t.crime as u32,
            7 => {
                if t.kind.is_zone() {
                    100
                } else {
                    0
                }
            }
            8 => self.coverage(0, i) as u32,
            9 => c.map_or(0, |c| c.elevation.max(0) as u32 * 100 / 48),
            10 => c.map_or(0, |c| ((c.ore as u32 + c.oil as u32) / 100).min(100)),
            11 => {
                if t.watered {
                    self.ext.sewage_overflow.min(100)
                } else {
                    100
                }
            }
            12 => c.map_or(self.garbage, |c| c.garbage) as u32,
            13 => ((self.health as u32 + self.coverage(2, i) as u32) / 2).min(100),
            14 => ((self.coverage(3, i) as u32 + self.coverage(4, i) as u32) / 2).min(100),
            15 => desirability(
                self.coverage(5, i) as i32 / 4,
                t.pollution as i32 / 2 + t.crime as i32 / 3,
            ),
            16 => desirability(self.coverage(6, i) as i32 / 3, t.traffic as i32 / 2),
            17 => desirability(30, t.traffic as i32 / 2),
            18 => desirability(self.coverage(5, i) as i32 / 3, t.pollution as i32),
            19 => self.coverage(6, i) as u32,
            _ => return None,
        })
    }
    pub fn suggest_site(&self, kind: Kind) -> Option<[i32; 2]> {
        let service = match kind {
            Kind::Police => 0,
            Kind::Fire | Kind::FireHQ => 1,
            Kind::Hospital | Kind::Clinic => 2,
            Kind::School => 3,
            Kind::University => 4,
            Kind::Park => 5,
            Kind::BusDepot => 6,
            Kind::Library => 7,
            Kind::Prison => 8,
            Kind::Cemetery => 9,
            _ => 0,
        };
        self.tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                t.kind == Kind::Empty
                    && t.terrain != Terrain::Water
                    && t.connected
                    && self
                        .placement_cost(
                            (*i % self.width as usize) as i32,
                            (*i / self.width as usize) as i32,
                            kind,
                        )
                        .is_ok()
            })
            .max_by_key(|(i, _)| {
                let need = self
                    .neighbors(*i)
                    .map(|j| self.tiles[j].population + self.tiles[j].jobs)
                    .sum::<u32>();
                (
                    need * (100 - self.coverage(service, *i) as u32),
                    std::cmp::Reverse(*i),
                )
            })
            .map(|(i, _)| {
                [
                    (i % self.width as usize) as i32,
                    (i / self.width as usize) as i32,
                ]
            })
    }
    pub fn problems(&self, i: usize) -> Vec<String> {
        let Some(t) = self.tiles.get(i) else {
            return vec!["Outside the region".into()];
        };
        let mut out = vec![];
        if t.kind != Kind::Empty {
            if !t.connected {
                out.push("No route to the outside".into());
            }
            if !t.powered {
                out.push("No electricity".into());
            }
            if !t.watered {
                out.push("No clean water".into());
            }
        }
        if t.traffic > 75 {
            out.push("Traffic congestion".into());
        }
        if t.pollution > 45 {
            out.push("Air pollution".into());
        }
        if t.crime > 35 {
            out.push("High crime".into());
        }
        if t.fire > 0 {
            out.push("Active fire".into());
        }
        if let Some(c) = self.ext.cells.get(i) {
            if c.rubble {
                out.push("Clear rubble before rebuilding".into());
            }
            if c.abandoned {
                out.push("Abandoned after service failure".into());
            }
            if c.condition < 40 {
                out.push("Building or road needs maintenance".into());
            }
            if c.flood > 0 {
                out.push("Flooding".into());
            }
            if c.radiation > 20 {
                out.push("Radiation hazard".into());
            }
        }
        out
    }
    pub fn next_level_needs(&self, i: usize) -> Vec<String> {
        let Some(t) = self.tiles.get(i) else {
            return vec![];
        };
        if !t.kind.is_zone() {
            return vec![];
        }
        let mut out = self.problems(i);
        if self.education < 25 && t.level >= 2 {
            out.push("More educated workers".into());
        }
        if t.land_value < 35 && t.kind == Kind::Residential {
            out.push("Higher land value: add parks and services".into());
        }
        if self.demand[if t.kind == Kind::Residential {
            0
        } else if matches!(t.kind, Kind::Commercial | Kind::Hotel) {
            1
        } else {
            2
        }] <= 20
        {
            out.push("More demand or lower taxes".into());
        }
        if t.level >= 3
            && !self
                .tiles
                .iter()
                .any(|t| t.kind == Kind::University && t.powered)
        {
            out.push("An operational university".into());
        }
        out
    }
    pub fn create_line(
        &mut self,
        mode: crate::transport::TransitMode,
        stops: Vec<u32>,
    ) -> Result<u32, String> {
        let mut transport = std::mem::take(&mut self.ext.transport);
        let result = transport.create_line(self, mode, stops);
        self.ext.transport = transport;
        result
    }
    pub fn set_line_fare(&mut self, id: u32, fare: u16) -> Result<(), String> {
        if fare > 25 {
            return Err("Fares range from 0 to25.".into());
        }
        let line = self
            .ext
            .transport
            .lines
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or("Unknown line.")?;
        line.fare = fare;
        Ok(())
    }
    pub fn set_road_direction(&mut self, x: i32, y: i32, orientation: u8) -> Result<(), String> {
        self.initialize_world();
        let i = self.idx(x, y).ok_or("Outside the region.")?;
        if !self.tiles[i].kind.is_road() {
            return Err("Select a road.".into());
        }
        self.ext.cells[i].orientation = orientation % 4;
        Ok(())
    }
}
impl City {
    pub fn underground_connections(&self, i: usize) -> [u8; 4] {
        if i >= self.tiles.len() {
            return [0; 4];
        }
        let present = |j: usize, layer: usize| {
            let t = &self.tiles[j];
            let c = self.ext.cells.get(j);
            match layer {
                0 => self.conducts(j, false),
                1 => self.conducts(j, true),
                2 => c.is_some_and(|c| c.sewer) || t.kind.is_road() || t.kind == Kind::Pipe,
                3 => {
                    c.is_some_and(|c| c.subway)
                        || matches!(t.kind, Kind::Subway | Kind::SubwayStation)
                }
                _ => false,
            }
        };
        let x = (i % self.width as usize) as i32;
        let y = (i / self.width as usize) as i32;
        std::array::from_fn(|layer| {
            if !present(i, layer) {
                return 0;
            }
            [(1, 0), (0, 1), (-1, 0), (0, -1)]
                .into_iter()
                .enumerate()
                .filter_map(|(bit, (dx, dy))| {
                    self.idx(x + dx, y + dy)
                        .filter(|&j| present(j, layer))
                        .map(|_| 1u8 << bit)
                })
                .fold(0, |a, b| a | b)
        })
    }
}
impl City {
    pub fn outside_connected(&self) -> bool {
        self.tiles.iter().enumerate().any(|(i, t)| {
            t.connected
                && (t.kind.is_road() || matches!(t.kind, Kind::Rail | Kind::PowerLine))
                && (i % self.width as usize == 0
                    || i % self.width as usize + 1 == self.width as usize
                    || i / self.width as usize == 0
                    || i / self.width as usize + 1 == self.height as usize)
        })
    }
    pub fn set_neighbor_contract(
        &mut self,
        index: usize,
        enabled: bool,
        power: u32,
        goods: u32,
        tariff: u16,
    ) -> Result<(), String> {
        if power > 100000 || goods > 50000 || tariff > 25 {
            return Err(
                "Neighbor contracts allow 0..100,000 power, 0..50,000 goods and 0..25 tariff."
                    .into(),
            );
        }
        self.initialize_world();
        let n = self
            .ext
            .neighbors
            .get_mut(index)
            .ok_or("Unknown neighboring region.")?;
        n.enabled = enabled;
        n.power = power;
        n.goods = goods;
        n.tariff = tariff;
        if !enabled {
            n.last_power = 0;
            n.last_goods = 0;
            n.monthly_cost = 0;
        }
        self.refresh();
        Ok(())
    }
}

#[cfg(test)]
mod coverage_memo_tests {
    use super::*;
    #[test]
    fn merged_provider_influences_equal_full_bfs_with_funding_outages_and_road_edits() {
        let mut city = City::new(7, true);
        city.initialize_world();
        let mut providers = Vec::new();
        for x in [15, 21, 27] {
            let i = city.idx(x, 18).unwrap();
            city.tiles[i] = Tile::new(Terrain::Grass);
            city.tiles[i].kind = Kind::Police;
            city.ext.cells[i].construction = 100;
            providers.push(i);
        }
        for step in 0..6 {
            city.funding[1] = [35, 100, 150][step % 3];
            city.ext.cells[providers[1]].outage_days = u8::from(step >= 2);
            if step == 4 {
                let i = city.idx(20, 19).unwrap();
                city.tiles[i].kind = Kind::Empty;
            }
            city.refresh();
            let roads: Vec<bool> = city
                .tiles
                .iter()
                .map(|t| t.kind.is_road() && t.connected)
                .collect();
            city.ext.infrastructure_memo.services.clear();
            let merged = city.service(&roads, &[Kind::Police, Kind::Prison], 32, 1, true);
            assert!(merged.iter().any(|&v| v > 0));
            city.ext.infrastructure_memo.services.clear();
            // Exercise the bounded-memory exact combined-BFS path with the
            // identical providers, rather than using the raster merge again.
            city.ext.infrastructure_memo.raster_count =
                32 * 1024 * 1024 / (city.tiles.len() * 2) + 1;
            let combined = city.service(&roads, &[Kind::Police, Kind::Prison], 32, 1, true);
            assert_eq!(
                merged, combined,
                "distance decomposition changed step{step}"
            );
            city.ext.infrastructure_memo.raster_count = city
                .ext
                .infrastructure_memo
                .rasters
                .iter()
                .filter(|r| r.is_some())
                .count();
        }
    }
}
