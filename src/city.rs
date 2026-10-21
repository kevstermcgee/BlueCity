//! Deterministic, integer-only BlueCity simulation. Roads contain municipal
//! power and water conduits; separate lines and pipes extend those networks.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const SIZE: usize = 48;
const AREA: usize = SIZE * SIZE;
const FAR: u16 = u16::MAX;

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
}

impl MapStyle {
    pub const ALL: [Self; 4] = [Self::River, Self::Coast, Self::Islands, Self::Highlands];

    pub fn label(self) -> &'static str {
        match self {
            Self::River => "River",
            Self::Coast => "Coast",
            Self::Islands => "Islands",
            Self::Highlands => "Highlands",
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
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "Open land",
            Self::Road => "Road",
            Self::Avenue => "Avenue",
            Self::Rail => "Rail",
            Self::PowerLine => "Power line",
            Self::Pipe => "Water pipe",
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
            Self::Residential | Self::Commercial | Self::Industrial => 45,
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
        }
    }
    pub fn upkeep(self) -> i64 {
        match self {
            Self::Empty | Self::Residential | Self::Commercial | Self::Industrial => 0,
            Self::Road | Self::PowerLine | Self::Pipe => 1,
            Self::Avenue | Self::Rail => 2,
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
            Self::BusDepot | Self::Station => 35,
            Self::Seaport => 100,
            Self::Airport => 150,
            Self::Landfill => 30,
        }
    }
    pub fn is_zone(self) -> bool {
        matches!(
            self,
            Self::Residential | Self::Commercial | Self::Industrial
        )
    }
    fn is_road(self) -> bool {
        matches!(self, Self::Road | Self::Avenue)
    }
    fn is_network(self) -> bool {
        matches!(
            self,
            Self::Road | Self::Avenue | Self::Rail | Self::PowerLine | Self::Pipe
        )
    }
    fn is_building(self) -> bool {
        self != Self::Empty && !self.is_network()
    }
    fn funding(self) -> usize {
        match self {
            Self::Road
            | Self::Avenue
            | Self::Rail
            | Self::PowerLine
            | Self::Pipe
            | Self::BusDepot
            | Self::Station
            | Self::Seaport
            | Self::Airport => 0,
            Self::School | Self::University => 2,
            Self::Coal
            | Self::Wind
            | Self::Nuclear
            | Self::Pump
            | Self::Sewage
            | Self::Landfill => 3,
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
    fn new(terrain: Terrain) -> Self {
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

pub fn index(x: i32, y: i32) -> Option<usize> {
    if x >= 0 && y >= 0 && x < SIZE as i32 && y < SIZE as i32 {
        Some(y as usize * SIZE + x as usize)
    } else {
        None
    }
}
fn neighbors(i: usize) -> impl Iterator<Item = usize> {
    let x = (i % SIZE) as i32;
    let y = (i / SIZE) as i32;
    [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
        .into_iter()
        .filter_map(|(x, y)| index(x, y))
}
fn distance(a: usize, b: usize) -> usize {
    (a % SIZE).abs_diff(b % SIZE) + (a / SIZE).abs_diff(b / SIZE)
}
fn capacity(kind: Kind, level: u8) -> u32 {
    let l = level.min(5) as usize;
    match kind {
        Kind::Residential => [0, 24, 64, 144, 280, 480][l],
        Kind::Commercial => [0, 18, 48, 120, 250, 450][l],
        Kind::Industrial => [0, 28, 60, 100, 160, 240][l],
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

fn generated_terrain(seed: u64, style: MapStyle) -> Vec<Tile> {
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
    (0..AREA)
        .map(|i| {
            let x = (i % SIZE) as i32;
            let y = (i / SIZE) as i32;
            let (u, v) = match rotation {
                0 => (x, y),
                1 => (47 - y, x),
                2 => (47 - x, 47 - y),
                _ => (y, 47 - x),
            };
            let coast_noise = terrain_noise(seed, u, v, 7, 40) - 512;
            let water = match style {
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
                MapStyle::Highlands => lakes.iter().any(|&(cx, cy, radius)| {
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
            let terrain = if (21..=27).contains(&x) && (21..=27).contains(&y) {
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
        city.tiles = generated_terrain(seed, style);
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
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        index(x, y).map(|i| &self.tiles[i])
    }
    fn message(&mut self, text: impl Into<String>) {
        self.messages.push(text.into());
        if self.messages.len() > 16 {
            self.messages.remove(0);
        }
    }
    fn make_starter(&mut self) {
        let mut set = |x, y, kind: Kind| {
            let i = index(x, y).unwrap();
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
            self.tiles[index(7, y).unwrap()] = Tile::new(Terrain::Water);
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
        let i = index(x, y).ok_or("That location is outside the region.")?;
        let old = self.tiles[i];
        if old.kind == kind {
            return Err("That tile already has this development.".into());
        }
        let upgrade = matches!(
            (old.kind, kind),
            (Kind::Road, Kind::Avenue) | (Kind::Avenue, Kind::Road)
        );
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
        if matches!(kind, Kind::Pump | Kind::Seaport)
            && !self.near_water(i, if kind == Kind::Pump { 3 } else { 1 })
        {
            return Err("This facility needs a river or coastline nearby.".into());
        }
        if kind == Kind::Airport && old.terrain == Terrain::Hill {
            return Err("An airport needs flat land.".into());
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
        let i = index(x, y).ok_or("That location is outside the region.")?;
        let t = self.tiles[i];
        if t.kind == Kind::Empty && t.terrain != Terrain::Forest && t.terrain != Terrain::Hill {
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
        let terrain = if self.tiles[i].terrain == Terrain::Water {
            Terrain::Water
        } else {
            Terrain::Grass
        };
        self.tiles[i] = Tile::new(terrain);
    }
    fn near_water(&self, i: usize, radius: usize) -> bool {
        let x = (i % SIZE) as i32;
        let y = (i / SIZE) as i32;
        let radius = radius as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() > radius {
                    continue;
                }
                if let Some(j) = index(x + dx, y + dy) {
                    if self.tiles[j].terrain == Terrain::Water {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn road_network(&self) -> (Vec<bool>, Vec<u16>) {
        let mut roads = vec![false; AREA];
        let mut queue = VecDeque::new();
        // A connected rail line can import people and goods through a station.
        let mut rail = vec![false; AREA];
        let mut rail_queue = VecDeque::new();
        for i in 0..AREA {
            let edge =
                i % SIZE == 0 || i % SIZE == SIZE - 1 || i / SIZE == 0 || i / SIZE == SIZE - 1;
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
            for j in neighbors(i) {
                if !rail[j] && self.rail_track(j) {
                    rail[j] = true;
                    rail_queue.push_back(j);
                }
            }
        }
        if self.funding[0] >= 25 {
            for (i, &connected) in rail.iter().enumerate() {
                if connected && self.tiles[i].kind == Kind::Station {
                    for (j, road) in roads.iter_mut().enumerate() {
                        if distance(i, j) <= 2 && self.tiles[j].kind.is_road() && !*road {
                            *road = true;
                            queue.push_back(j);
                        }
                    }
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            for j in neighbors(i) {
                if !roads[j] && self.tiles[j].kind.is_road() {
                    roads[j] = true;
                    queue.push_back(j);
                }
            }
        }
        let mut access = vec![FAR; AREA];
        for i in 0..AREA {
            if roads[i] {
                access[i] = 0;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            if access[i] >= 3 {
                continue;
            }
            for j in neighbors(i) {
                if access[j] > access[i] + 1 && self.tiles[j].terrain != Terrain::Water {
                    access[j] = access[i] + 1;
                    queue.push_back(j);
                }
            }
        }
        (roads, access)
    }
    fn rail_track(&self, i: usize) -> bool {
        matches!(self.tiles[i].kind, Kind::Rail | Kind::Station) || self.tiles[i].rail_crossing
    }
    fn station_active(&self, start: usize) -> bool {
        if self.funding[0] < 25 {
            return false;
        }
        let mut seen = vec![false; AREA];
        let mut queue = VecDeque::from([start]);
        seen[start] = true;
        while let Some(i) = queue.pop_front() {
            let edge =
                i % SIZE == 0 || i % SIZE == SIZE - 1 || i / SIZE == 0 || i / SIZE == SIZE - 1;
            if i != start
                && ((self.tiles[i].kind == Kind::Station
                    && self.tiles[i].connected
                    && self.tiles[i].powered
                    && self.tiles[i].watered)
                    || edge)
            {
                return true;
            }
            for j in neighbors(i) {
                if !seen[j] && self.rail_track(j) {
                    seen[j] = true;
                    queue.push_back(j);
                }
            }
        }
        false
    }
    fn service(
        &self,
        roads: &[bool],
        kinds: &[Kind],
        range: u16,
        funding: usize,
        require_utilities: bool,
    ) -> Vec<u8> {
        let mut dist = vec![FAR; AREA];
        let mut queue = VecDeque::new();
        let budget = self.funding[funding].min(150) as u16;
        let range = range * budget / 100;
        if range == 0 {
            return vec![0; AREA];
        }
        for (i, t) in self.tiles.iter().enumerate() {
            if !kinds.contains(&t.kind)
                || !t.connected
                || t.fire > 0
                || (require_utilities && (!t.powered || !t.watered))
                || (t.kind == Kind::Station && !self.station_active(i))
            {
                continue;
            }
            for j in 0..AREA {
                let d = distance(i, j) as u16;
                if roads[j] && d <= 3 && d < dist[j] {
                    dist[j] = d;
                    queue.push_back(j);
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            if dist[i] >= range {
                continue;
            }
            for j in neighbors(i) {
                if roads[j] && dist[j] > dist[i] + 1 {
                    dist[j] = dist[i] + 1;
                    queue.push_back(j);
                }
            }
        }
        let mut result = vec![0; AREA];
        // Service vehicles travel on roads; coverage spills the last three tiles
        // into city blocks. Disconnected parallel streets receive no coverage.
        for (i, coverage) in result.iter_mut().enumerate() {
            let x = (i % SIZE) as i32;
            let y = (i / SIZE) as i32;
            let mut best = FAR;
            for dy in -3i32..=3 {
                for dx in -3i32..=3 {
                    if dx.abs() + dy.abs() > 3 {
                        continue;
                    }
                    if let Some(j) = index(x + dx, y + dy) {
                        if dist[j] != FAR {
                            best = best.min(dist[j].saturating_add((dx.abs() + dy.abs()) as u16));
                        }
                    }
                }
            }
            if best <= range {
                *coverage = (100 - best as u32 * 45 / (range as u32 + 1))
                    .min(budget as u32)
                    .min(100) as u8;
            }
        }
        result
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
            if matches!(t.kind, Kind::Coal | Kind::Wind | Kind::Nuclear) {
                0
            } else {
                inhabitants / 3 + 8
            }
        }
    }
    fn utility(&mut self, water: bool) -> (u32, u32) {
        let mut visited = vec![false; AREA];
        let mut total_supply = 0u32;
        let mut total_demand = 0u32;
        for start in 0..AREA {
            if visited[start] || !self.conducts(start, water) {
                continue;
            }
            let mut queue = VecDeque::from([start]);
            let mut members = vec![];
            visited[start] = true;
            let mut supply = 0u32;
            while let Some(i) = queue.pop_front() {
                members.push(i);
                let t = self.tiles[i];
                if t.connected && t.fire == 0 {
                    let output = if water {
                        if t.kind == Kind::Pump && t.powered && self.near_water(i, 3) {
                            14000
                        } else {
                            0
                        }
                    } else {
                        match t.kind {
                            Kind::Coal => 7500,
                            Kind::Wind => 1300,
                            Kind::Nuclear => 35000,
                            _ => 0,
                        }
                    };
                    supply += output * self.funding[3].min(150) as u32 / 100;
                }
                for j in neighbors(i) {
                    if !visited[j] && self.conducts(j, water) {
                        visited[j] = true;
                        queue.push_back(j);
                    }
                }
            }
            let demand: u32 = members
                .iter()
                .map(|&i| Self::utility_demand(&self.tiles[i], water))
                .sum();
            total_supply += supply;
            total_demand += demand;
            // Rotate brownouts each month instead of permanently starving one
            // district. Each disconnected component has its own finite capacity.
            members.sort_unstable_by_key(|&i| (i + self.month as usize * 37) % AREA);
            let mut remaining = supply;
            for i in members {
                let need = Self::utility_demand(&self.tiles[i], water);
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
        // A road's underground infrastructure feeds the neighboring lots.
        // Empty lots are deliberately not conductors: remote development still
        // needs a road, a power line or a pipe all the way to its block.
        (total_supply, total_demand)
    }
    fn conducts(&self, i: usize, water: bool) -> bool {
        let kind = self.tiles[i].kind;
        kind != Kind::Empty
            && if water {
                kind != Kind::PowerLine
            } else {
                kind != Kind::Pipe
            }
    }
    fn infrastructure(&mut self) -> Networks {
        let (roads, access) = self.road_network();
        for (i, &distance) in access.iter().enumerate() {
            self.tiles[i].connected = distance <= 3;
            self.tiles[i].powered = false;
            self.tiles[i].watered = false;
        }
        (self.power_supply, self.power_demand) = self.utility(false);
        (self.water_supply, self.water_demand) = self.utility(true);
        let police = self.service(&roads, &[Kind::Police], 32, 1, true);
        let fire = self.service(&roads, &[Kind::Fire], 34, 1, true);
        let health = self.service(&roads, &[Kind::Hospital], 34, 1, true);
        let school = self.service(&roads, &[Kind::School], 28, 2, true);
        let university = self.service(&roads, &[Kind::University], 42, 2, true);
        let park = self.service(&roads, &[Kind::Park, Kind::Stadium], 14, 1, false);
        let transit = self.service(&roads, &[Kind::BusDepot, Kind::Station], 36, 0, true);
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
        self.population = self.tiles.iter().map(|t| t.population).sum();
        self.jobs = self.tiles.iter().map(|t| t.jobs).sum();
        let network = self.infrastructure();
        self.update_environment(&network);
        self.update_economy();
    }
    fn operational(&self, kind: Kind) -> u32 {
        self.tiles
            .iter()
            .filter(|t| {
                t.kind == kind
                    && t.connected
                    && t.powered
                    && t.watered
                    && t.fire == 0
                    && self.funding[t.kind.funding()] > 0
            })
            .count() as u32
    }
    fn update_environment(&mut self, n: &Networks) {
        let landfill =
            self.operational(Kind::Landfill) * 5000 * self.funding[3].min(150) as u32 / 100;
        let garbage_capacity = landfill * if self.policies[2] { 2 } else { 1 };
        self.garbage = ((self.population.saturating_sub(garbage_capacity)) * 100
            / self.population.max(1))
        .min(100) as u8;
        let sewage = self.operational(Kind::Sewage) * 16000 * self.funding[3].min(150) as u32 / 100;
        let sewage_penalty =
            self.water_demand.saturating_sub(sewage) * 25 / self.water_demand.max(1);
        let workforce = self.population * 55 / 100;
        self.unemployment =
            (workforce.saturating_sub(self.jobs) * 100 / workforce.max(1)).min(100) as u8;
        let mut pollution = vec![0u16; AREA];
        for i in 0..AREA {
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
            let x = (i % SIZE) as i32;
            let y = (i / SIZE) as i32;
            for dy in -5i32..=5 {
                for dx in -5i32..=5 {
                    let d = (dx.abs() + dy.abs()) as u16;
                    if d > 5 {
                        continue;
                    }
                    if let Some(j) = index(x + dx, y + dy) {
                        // Multiple factories do not cause an instant 100% blanket.
                        pollution[j] = pollution[j].saturating_add(source / (d * d + 2));
                    }
                }
            }
        }
        let mut health_total = 0u64;
        let mut school_total = 0u64;
        let mut happy_total = 0u64;
        let mut transit_total = 0u64;
        let mut traffic_sources = vec![0u32; AREA];
        // Jobs and residents make trips to their nearest connected street. Trip
        // pressure then spreads along the road graph, showing real bottlenecks.
        for i in 0..AREA {
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
                    if let Some(j) = index((i % SIZE) as i32 + dx, (i / SIZE) as i32 + dy) {
                        if n.roads[j] && nearest.is_none_or(|k| distance(i, j) < distance(i, k)) {
                            nearest = Some(j);
                        }
                    }
                }
            }
            if let Some(j) = nearest {
                let reduction = n.transit[i] as u32 * if self.policies[1] { 65 } else { 40 } / 100;
                traffic_sources[j] += trips * (100 - reduction) / 100;
            }
        }
        let old_traffic = traffic_sources.clone();
        for (i, trips) in traffic_sources.iter_mut().enumerate() {
            if n.roads[i] {
                for j in neighbors(i) {
                    if n.roads[j] {
                        *trips += old_traffic[j] / 2;
                    }
                }
            }
        }
        let mut traffic = vec![0u8; AREA];
        for i in 0..AREA {
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
        for i in 0..AREA {
            let t = self.tiles[i];
            if !n.roads[i] {
                traffic[i] = neighbors(i).map(|j| traffic[j]).max().unwrap_or(0) / 2;
            }
            let trees = neighbors(i)
                .filter(|&j| self.tiles[j].terrain == Terrain::Forest)
                .count() as i32
                * 3;
            let p = (pollution[i] as i32 + self.garbage as i32 / 3 + traffic[i] as i32 / 9 - trees)
                .clamp(0, 100);
            let crime = (12
                + self.unemployment as i32 / 2
                + t.level as i32 * 5
                + (self.taxes[0] as i32 - 9).max(0) * 2
                - n.police[i] as i32 * 45 / 100
                - n.school[i] as i32 / 10)
                .clamp(0, 100);
            let scenic = if t.terrain == Terrain::Hill { 8 } else { 0 };
            let water = if self.near_water(i, 2) { 7 } else { 0 };
            let value = (38
                + n.park[i] as i32 / 3
                + n.school[i] as i32 / 8
                + n.health[i] as i32 / 10
                + scenic
                + water
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
                let education =
                    (n.school[i] as u32 * 65 / 100 + n.university[i] as u32 * 35 / 100).min(100);
                let happy =
                    (35 + value / 2 + health / 6 + education as i32 / 10 + n.park[i] as i32 / 12
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
        let commercial: u32 = self
            .tiles
            .iter()
            .filter(|t| t.kind == Kind::Commercial)
            .map(|t| t.jobs)
            .sum();
        let industrial: u32 = self
            .tiles
            .iter()
            .filter(|t| t.kind == Kind::Industrial)
            .map(|t| t.jobs)
            .sum();
        let p = self.population as i64;
        let incentives = if self.policies[3] { 20 } else { 0 };
        self.demand = [
            (35 + (self.jobs as i64 - p * 55 / 100) * 110 / (p + 100)
                + (self.happiness as i64 - 60) * 2
                - (self.taxes[0] as i64 - 9) * 8)
                .clamp(-100, 100) as i32,
            (40 + (p * 20 / 100 - commercial as i64) * 150 / (p + 100)
                - (self.taxes[1] as i64 - 9) * 9
                + self.operational(Kind::Airport) as i64 * 20
                + self.operational(Kind::Stadium) as i64 * 10
                + incentives)
                .clamp(-100, 100) as i32,
            (40 + (p * 40 / 100 - industrial as i64) * 120 / (p + 100)
                - (self.taxes[2] as i64 - 9) * 9
                + self.operational(Kind::Seaport) as i64 * 25
                + incentives)
                .clamp(-100, 100) as i32,
        ];
    }
    fn update_economy(&mut self) {
        let mut income = 0i64;
        let mut expenses = 0i64;
        for t in &self.tiles {
            income += match t.kind {
                Kind::Residential => t.population as i64 * self.taxes[0] as i64 / 40,
                Kind::Commercial => t.jobs as i64 * self.taxes[1] as i64 / 20,
                Kind::Industrial => t.jobs as i64 * self.taxes[2] as i64 / 25,
                _ => 0,
            };
            expenses += t.kind.upkeep() * self.funding[t.kind.funding()].min(150) as i64 / 100;
            if t.rail_crossing {
                expenses += Kind::Rail.upkeep() * self.funding[0].min(150) as i64 / 100;
            }
        }
        // Freight, visitors and fares are useful revenue streams, not decorations.
        income += self.operational(Kind::Seaport) as i64 * (80 + self.jobs as i64 / 20);
        income += self.operational(Kind::Airport) as i64 * (100 + self.population as i64 / 25);
        income += self.operational(Kind::Stadium) as i64 * (35 + self.population as i64 / 80);
        if !self.policies[1] {
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
        self.income = income;
        self.expenses = expenses;
    }
    pub fn advance_month(&mut self) {
        self.month = self.month.saturating_add(1);
        let old_population = self.population;
        let n = self.infrastructure();
        self.update_environment(&n);
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
        for i in 0..AREA {
            let t = self.tiles[i];
            if !t.kind.is_zone() {
                continue;
            }
            let which = match t.kind {
                Kind::Residential => 0,
                Kind::Commercial => 1,
                _ => 2,
            };
            let livable = t.connected && t.powered && t.watered && t.fire == 0;
            let local = match t.kind {
                Kind::Residential => {
                    t.land_value as i32 / 3 - t.pollution as i32 / 2 - t.crime as i32 / 3
                }
                Kind::Commercial => {
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
                let cap = capacity(t.kind, self.tiles[i].level);
                let current = if which == 0 { t.population } else { t.jobs };
                let growth =
                    (3 + cap / 14 + score.max(0) as u32 / 18).min(cap.saturating_sub(current));
                if which == 0 {
                    self.tiles[i].population += growth;
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
                } else {
                    self.tiles[i].jobs = next;
                }
                if next == 0 {
                    self.tiles[i].level = 0;
                }
            }
        }
        self.process_fires(&n.fire);
        self.refresh();
        self.money =
            (self.money + self.income - self.expenses).clamp(-1_000_000_000, 1_000_000_000);
        if self.money < 0 && self.month.is_multiple_of(6) {
            self.message(
                "The treasury is in deficit. Adjust taxes, reduce spending or issue a bond.",
            );
        }
        for milestone in [500, 1500, 5000, 15000, 35000] {
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
                    for j in neighbors(i) {
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
                for i in 0..AREA {
                    if distance(i, center) <= 5
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
                let x = (center % SIZE) as i32;
                let y = (center / SIZE) as i32;
                let mut lost = 0;
                for step in -7..=7 {
                    for width in -1..=1 {
                        if let Some(i) = index(x + step, y + step / 2 + width) {
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
                for i in 0..AREA {
                    if distance(i, reactor) <= 4 && self.tiles[i].kind.is_building() {
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
        if self.tiles.len() != AREA {
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
        for t in &self.tiles {
            if t.level > 5
                || t.population > capacity(Kind::Residential, t.level)
                || t.jobs > capacity(t.kind, t.level)
                || (t.population > 0 && t.kind != Kind::Residential)
                || (t.jobs > 0 && !matches!(t.kind, Kind::Commercial | Kind::Industrial))
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
        Ok(())
    }
}
