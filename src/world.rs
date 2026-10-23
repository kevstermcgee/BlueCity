//! Saved daily environmental and building lifecycle rules. No presentation clock
//! or platform randomness participates in these systems.
use crate::{
    city::{City, Kind, MapStyle, Terrain},
    people::PeopleState,
    transport::TransportState,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Climate {
    #[default]
    Temperate,
    Tropical,
    Alpine,
}

impl Climate {
    pub const ALL: [Self; 3] = [Self::Temperate, Self::Tropical, Self::Alpine];
    pub fn label(self) -> &'static str {
        match self {
            Self::Temperate => "Temperate",
            Self::Tropical => "Tropical",
            Self::Alpine => "Alpine",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Heatwave,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weather {
    pub kind: WeatherKind,
    pub temperature: i16,
    pub wind_x: i8,
    pub wind_y: i8,
    pub rain: u8,
    pub cloud: u8,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingStage {
    #[default]
    Zoned,
    Foundation,
    Frame,
    Complete,
    Abandoned,
    Rubble,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub tile: u32,
    pub kind: Kind,
    pub stage: BuildingStage,
    pub progress: u8,
    pub condition: u8,
    pub era: u8,
    pub age_days: u32,
    pub occupants: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CellState {
    pub shore_distance: u8,
    pub pipe: bool,
    pub sewer: bool,
    pub underground_power: bool,
    pub subway: bool,
    pub elevation: i16,
    pub water_depth: u8,
    pub tree_age: u16,
    pub tree_species: u8,
    pub ore: u16,
    pub oil: u16,
    pub moisture: u8,
    pub fertility: u8,
    pub condition: u8,
    pub construction: u8,
    pub built_month: u32,
    pub abandoned: bool,
    pub rubble: bool,
    pub age_days: u32,
    pub heat: u8,
    pub noise: u8,
    pub radiation: u8,
    pub garbage: u8,
    pub flood: u8,
    pub orientation: u8,
    pub air_pollution: u8,
    pub outage_days: u8,
    pub abandoned_days: u16,
    pub density_stress: u16,
}
impl CellState {
    pub fn operational(&self) -> bool {
        self.construction == 100 && !self.rubble && !self.abandoned && self.outage_days == 0
    }
}
impl Default for CellState {
    fn default() -> Self {
        Self {
            shore_distance: u8::MAX,
            pipe: false,
            sewer: false,
            underground_power: false,
            subway: false,
            elevation: 0,
            water_depth: 0,
            tree_age: 0,
            tree_species: 0,
            ore: 0,
            oil: 0,
            moisture: 50,
            fertility: 50,
            condition: 100,
            construction: 100,
            built_month: 0,
            abandoned: false,
            rubble: false,
            age_days: 0,
            heat: 0,
            noise: 0,
            radiation: 0,
            garbage: 0,
            flood: 0,
            orientation: 0,
            air_pollution: 0,
            outage_days: 0,
            abandoned_days: 0,
            density_stress: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainAction {
    Raise,
    Lower,
    Level,
    Smooth,
    Water,
    Drain,
    Plant,
    Clear,
}
impl TerrainAction {
    pub const ALL: [Self; 8] = [
        Self::Raise,
        Self::Lower,
        Self::Level,
        Self::Smooth,
        Self::Water,
        Self::Drain,
        Self::Plant,
        Self::Clear,
    ];
    pub fn cost(self) -> i64 {
        match self {
            Self::Raise | Self::Lower => 40,
            Self::Level | Self::Smooth => 25,
            Self::Water | Self::Drain => 80,
            Self::Plant | Self::Clear => 20,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Raise => "Raise",
            Self::Lower => "Lower",
            Self::Level => "Level",
            Self::Smooth => "Smooth",
            Self::Water => "Water",
            Self::Drain => "Drain",
            Self::Plant => "Plant trees",
            Self::Clear => "Clear trees",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighborContract {
    pub region: String,
    pub enabled: bool,
    pub power: u32,
    pub goods: u32,
    pub unit_price: u16,
    pub tariff: u16,
    pub last_power: u32,
    pub last_goods: i32,
    pub monthly_cost: i64,
}
impl Default for NeighborContract {
    fn default() -> Self {
        Self {
            region: "Neighbor".into(),
            enabled: false,
            power: 0,
            goods: 0,
            unit_price: 2,
            tariff: 2,
            last_power: 0,
            last_goods: 0,
            monthly_cost: 0,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldState {
    #[serde(skip)]
    pub(crate) infrastructure_memo: crate::city::InfrastructureMemo,
    pub infrastructure_stamp: u64,
    pub utility_masks: [Vec<u8>; 2],
    pub utility_grids: [Vec<Vec<usize>>; 2],
    pub services: crate::services::ServiceState,
    pub neighbors: Vec<NeighborContract>,
    pub enabled: bool,
    pub seed: u64,
    pub style: MapStyle,
    pub climate: Climate,
    pub day: u64,
    pub minute_of_day: u16,
    pub season: u8,
    pub weather: Weather,
    pub cells: Vec<CellState>,
    pub buildings: Vec<Building>,
    pub next_building: u32,
    pub people: PeopleState,
    pub transport: TransportState,
    pub coverage: Vec<[u8; 13]>,
    pub power_reserve: i64,
    pub water_reserve: i64,
    pub sewage_overflow: u32,
    pub brownouts: u32,
    pub breakdowns: u32,
    pub tourism: u32,
    pub trade_income: i64,
    pub operating_cost: i64,
    pub maintenance: u8,
    pub rng: u64,
}
pub fn hash(seed: u64, n: u64) -> u64 {
    let mut v = seed.wrapping_add(n.wrapping_mul(0x9e3779b97f4a7c15));
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
    v ^ (v >> 31)
}
impl WorldState {
    pub fn new(city: &City, seed: u64, style: MapStyle, climate: Climate) -> Self {
        let cells = city
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let h = hash(seed, i as u64);
                let wet = t.terrain == Terrain::Water;
                CellState {
                    pipe: t.kind.is_road() || t.kind == Kind::Pipe,
                    sewer: t.kind.is_road() || t.kind == Kind::Pipe,
                    underground_power: t.kind.is_road() || t.kind == Kind::PowerLine,
                    subway: matches!(t.kind, Kind::Subway | Kind::SubwayStation),
                    age_days: if t.terrain == Terrain::Forest {
                        (36 + ((h >> 50) % 965) as u32) * 30
                    } else {
                        0
                    },
                    elevation: if wet {
                        -4
                    } else if t.terrain == Terrain::Hill {
                        24 + (h % 25) as i16
                    } else {
                        ((h >> 12) % 10) as i16
                    },
                    water_depth: if wet { 4 } else { 0 },
                    tree_age: if t.terrain == Terrain::Forest {
                        36 + ((h >> 50) % 965) as u16
                    } else {
                        0
                    },
                    tree_species: ((h >> 6) % 6) as u8,
                    ore: if (h >> 18).is_multiple_of(7) {
                        1000 + ((h >> 22) % 5000) as u16
                    } else {
                        0
                    },
                    oil: if (h >> 28).is_multiple_of(13) {
                        1000 + ((h >> 32) % 6000) as u16
                    } else {
                        0
                    },
                    fertility: if climate == Climate::Tropical {
                        80
                    } else {
                        40 + ((h >> 44) % 60) as u8
                    },
                    moisture: if wet { 100 } else { 50 },
                    built_month: city.month,
                    ..Default::default()
                }
            })
            .collect();
        let mut world = Self {
            neighbors: ["Bluevale", "Riverton", "Haven"]
                .into_iter()
                .map(|region| NeighborContract {
                    region: region.into(),
                    ..Default::default()
                })
                .collect(),
            enabled: true,
            seed,
            style,
            climate,
            day: city.month as u64 * 30,
            cells,
            coverage: vec![[0; 13]; city.tiles.len()],
            maintenance: 100,
            rng: hash(seed, 77),
            ..Default::default()
        };
        world.rebuild_shore(city);
        world
    }
    fn shoreline(city: &City) -> Vec<u8> {
        let mut distance = vec![u8::MAX; city.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        for (i, t) in city.tiles.iter().enumerate() {
            if t.terrain == Terrain::Water {
                distance[i] = 0;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let d = distance[i];
            if d >= 3 {
                continue;
            }
            for j in city.neighbors(i) {
                if distance[j] > d + 1 {
                    distance[j] = d + 1;
                    queue.push_back(j);
                }
            }
        }
        distance
    }
    pub(crate) fn warm_shore(&mut self, city: &City) {
        // A save can contain a pending terrain invalidation. Seed only when
        // its current derived distances already match the actual water mask.
        let distance = Self::shoreline(city);
        if self.cells.len() == distance.len()
            && self
                .cells
                .iter()
                .zip(distance)
                .all(|(c, d)| c.shore_distance == d)
        {
            self.infrastructure_memo.shore = Some(std::sync::Arc::new(
                city.tiles
                    .iter()
                    .map(|t| t.terrain == Terrain::Water)
                    .collect(),
            ));
        }
    }
    pub fn rebuild_shore(&mut self, city: &City) {
        if self.cells.len() != city.tiles.len() {
            return;
        }
        let water: Vec<bool> = city
            .tiles
            .iter()
            .map(|t| t.terrain == Terrain::Water)
            .collect();
        if self.infrastructure_memo.shore.as_deref() == Some(&water) {
            return;
        }
        self.infrastructure_memo.shore = Some(std::sync::Arc::new(water));
        for (c, d) in self.cells.iter_mut().zip(Self::shoreline(city)) {
            c.shore_distance = d;
        }
    }
    pub fn visual_tile(&self, i: usize) -> Option<&CellState> {
        self.cells.get(i)
    }
    pub fn register_edit(&mut self, i: usize, kind: Kind, month: u32) {
        if let Some(c) = self.cells.get_mut(i) {
            c.pipe = kind.is_road() || kind == Kind::Pipe;
            c.sewer = c.pipe;
            c.underground_power = kind.is_road() || kind == Kind::PowerLine;
            c.subway = matches!(kind, Kind::Subway | Kind::SubwayStation);
            c.condition = 100;
            c.age_days = 0;
            c.built_month = month;
            c.abandoned = false;
            c.rubble = false;
            c.abandoned_days = 0;
            c.density_stress = 0;
            c.construction = if kind.is_building() { 0 } else { 100 };
            if kind == Kind::Empty {
                self.buildings.retain(|b| b.tile != i as u32);
            }
        }
    }
    pub fn sync_buildings(&mut self, city: &City) {
        if !self.sync_existing_buildings(city) {
            self.sync_buildings_full(city);
        }
    }
    fn sync_existing_buildings(&mut self, city: &City) -> bool {
        let mut cursor = 0;
        for (i, t) in city.tiles.iter().enumerate() {
            if !t.kind.is_building() {
                continue;
            }
            let count = if t.kind.is_zone() {
                [1, 1, 2, 4, 8, 16][t.level.min(5) as usize]
            } else {
                1
            };
            let Some(lot) = self.buildings.get_mut(cursor..cursor + count) else {
                return false;
            };
            if lot.iter().any(|b| b.tile != i as u32 || b.kind != t.kind) {
                return false;
            }
            let c = &self.cells[i];
            for b in lot.iter_mut() {
                if c.construction < 100 {
                    b.progress = c.construction;
                }
                b.condition = c.condition;
                b.occupants = 0;
                b.stage = if c.rubble {
                    BuildingStage::Rubble
                } else if c.abandoned {
                    BuildingStage::Abandoned
                } else if b.progress < 20 {
                    BuildingStage::Foundation
                } else if b.progress < 100 {
                    BuildingStage::Frame
                } else if t.level == 0 && t.kind.is_zone() {
                    BuildingStage::Zoned
                } else {
                    BuildingStage::Complete
                };
            }
            let completed = lot.iter().filter(|b| b.progress == 100).count();
            if completed > 0 {
                for (ordinal, b) in lot.iter_mut().filter(|b| b.progress == 100).enumerate() {
                    b.occupants = (t.population + t.jobs) / completed as u32
                        + u32::from(
                            ordinal < ((t.population + t.jobs) % completed as u32) as usize,
                        );
                }
            }
            cursor += count;
        }
        cursor == self.buildings.len()
    }
    fn sync_buildings_full(&mut self, city: &City) {
        let previous = std::mem::take(&mut self.buildings);
        self.buildings = Vec::with_capacity(previous.len() + 128);
        let mut cursor = 0usize;
        for (i, t) in city.tiles.iter().enumerate() {
            if !t.kind.is_building() {
                continue;
            }
            let count = if t.kind.is_zone() {
                [1, 1, 2, 4, 8, 16][t.level.min(5) as usize]
            } else {
                1
            };
            while cursor < previous.len() && previous[cursor].tile < i as u32 {
                cursor += 1;
            }
            let start = cursor;
            while cursor < previous.len() && previous[cursor].tile == i as u32 {
                cursor += 1;
            }
            let existing = &previous[start..cursor];
            let c = &self.cells[i];
            let first = self.buildings.len();
            for sub in 0..count {
                let mut b = existing
                    .get(sub)
                    .filter(|b| b.kind == t.kind)
                    .copied()
                    .unwrap_or_else(|| {
                        self.next_building += 1;
                        Building {
                            id: self.next_building,
                            tile: i as u32,
                            kind: t.kind,
                            stage: BuildingStage::Foundation,
                            progress: if existing.is_empty() {
                                c.construction
                            } else {
                                0
                            },
                            condition: c.condition,
                            era: (city.month / 120).min(7) as u8,
                            age_days: c.age_days,
                            occupants: 0,
                        }
                    });
                if c.construction < 100 {
                    b.progress = c.construction;
                }
                b.condition = c.condition;
                if b.age_days == 0 && existing.is_empty() {
                    b.age_days = c.age_days;
                }
                b.occupants = 0;
                b.stage = if c.rubble {
                    BuildingStage::Rubble
                } else if c.abandoned {
                    BuildingStage::Abandoned
                } else if b.progress < 20 {
                    BuildingStage::Foundation
                } else if b.progress < 100 {
                    BuildingStage::Frame
                } else if t.level == 0 && t.kind.is_zone() {
                    BuildingStage::Zoned
                } else {
                    BuildingStage::Complete
                };
                self.buildings.push(b);
            }
            // Redevelopment keeps the existing occupants in completed houses
            // while additional buildings are foundations or frames.
            let lot = &mut self.buildings[first..];
            let completed = lot.iter().filter(|b| b.progress == 100).count();
            if completed > 0 {
                for (ordinal, b) in lot.iter_mut().filter(|b| b.progress == 100).enumerate() {
                    b.occupants = (t.population + t.jobs) / completed as u32
                        + u32::from(
                            ordinal < ((t.population + t.jobs) % completed as u32) as usize,
                        );
                }
            }
        }
    }
    pub fn advance_day(&mut self, city: &mut City) {
        self.day += 1;
        self.season = ((self.day / 90) % 4) as u8;
        let r = hash(self.seed, self.day / 21);
        let seasonal = [8, 19, 28, 13][self.season as usize];
        let offset = match self.climate {
            Climate::Temperate => 0,
            Climate::Tropical => 10,
            Climate::Alpine => -13,
        };
        let temp = seasonal + offset + (r % 11) as i16 - 5;
        let rain = match self.climate {
            Climate::Tropical => ((r >> 8) % 110).min(100),
            _ => ((r >> 8) % 120).saturating_sub(35).min(100),
        } as u8;
        self.weather = Weather {
            kind: if temp < 1 && rain > 25 {
                WeatherKind::Snow
            } else if rain > 78 {
                WeatherKind::Storm
            } else if rain > 35 {
                WeatherKind::Rain
            } else if temp > 34 {
                WeatherKind::Heatwave
            } else if rain > 12 {
                WeatherKind::Cloudy
            } else {
                WeatherKind::Clear
            },
            temperature: temp,
            wind_x: ((r >> 24) % 3) as i8 - 1,
            wind_y: ((r >> 28) % 3) as i8 - 1,
            rain,
            cloud: (rain as u16 + ((r >> 32) % 30) as u16).min(100) as u8,
        };
        let maintenance = city
            .tiles
            .iter()
            .any(|t| t.kind == Kind::MaintenanceDepot && t.connected && t.powered);
        self.maintenance = if maintenance {
            100
        } else {
            city.funding[0].min(100)
        };
        let old_elevation: Vec<_> = if self.day.is_multiple_of(30) && rain > 30 {
            self.cells.iter().map(|c| c.elevation).collect()
        } else {
            Vec::new()
        };
        let old_air: Vec<_> = self.cells.iter().map(|c| c.air_pollution).collect();
        let mut erode = vec![];
        self.brownouts = 0;
        let width = usize::from(city.width);
        let wind_x = i32::from(self.weather.wind_x);
        let wind_y = i32::from(self.weather.wind_y);
        for (y, row) in self.cells.chunks_mut(width).enumerate() {
            let upwind_y = y as i32 - wind_y;
            let upwind_row = (0..i32::from(city.height))
                .contains(&upwind_y)
                .then(|| upwind_y as usize * width);
            for (x, c) in row.iter_mut().enumerate() {
                let i = y * width + x;
                let t = city.tiles[i];
                let upwind_x = x as i32 - wind_x;
                let upwind = upwind_row.and_then(|row| {
                    (0..i32::from(city.width))
                        .contains(&upwind_x)
                        .then(|| row + upwind_x as usize)
                });
                c.age_days = c.age_days.saturating_add(1);
                if c.outage_days > 0 {
                    c.outage_days -= 1;
                    self.brownouts += 1;
                }
                let source =
                    match t.kind {
                        Kind::Coal => 85,
                        Kind::Gas => 45,
                        Kind::Industrial => 25 + t.level * 7,
                        Kind::Incinerator => 45,
                        Kind::Landfill => 40,
                        Kind::SewageOutlet => 60,
                        _ => 0,
                    } * u8::from(c.operational() && t.connected && t.powered && t.fire == 0);
                let inherited = upwind.map_or(0, |j| old_air[j] as u16 * 3 / 5);
                c.air_pollution = ((source as u16 + inherited) / 2)
                    .saturating_sub(
                        rain as u16 / 10
                        + u16::from(t.terrain == Terrain::Forest && c.tree_age >= 36) * 4
                        // Vegetation in accessible parks also scrubs the air.
                        + self.coverage.get(i).map_or(0, |v| v[5] as u16 / 20),
                    )
                    .min(100) as u8;
                let moisture = (c.moisture as i16 + rain as i16 / 9 - (temp.max(0) / 12) - 1)
                    .clamp(0, 100) as u8;
                c.moisture = if t.terrain == Terrain::Water {
                    100
                } else {
                    moisture
                };
                if t.terrain == Terrain::Forest {
                    c.tree_age = (c.age_days / 30).min(1200) as u16;
                    if c.tree_age >= 36 && c.moisture > 15 {
                        c.fertility = c.fertility.saturating_add(1).min(100);
                    }
                }
                if t.kind.is_building() && c.construction < 100 && city.money >= 0 {
                    c.construction = c
                        .construction
                        .saturating_add(8 + (hash(self.seed, self.day + i as u64) % 8) as u8)
                        .min(100);
                }
                if t.kind.is_building() {
                    if !t.connected || !t.powered || (!t.watered && t.kind != Kind::Pump) {
                        c.abandoned_days = c.abandoned_days.saturating_add(1);
                    } else {
                        c.abandoned_days = c.abandoned_days.saturating_sub(3);
                    }
                    c.abandoned = c.abandoned_days > 120;
                    if c.abandoned_days > 360 {
                        c.rubble = true;
                        self.people.migrants_out += city.tiles[i].population as u64;
                        city.tiles[i].population = 0;
                        city.tiles[i].jobs = 0;
                    }
                }
                if t.kind.is_road() && self.day.is_multiple_of(30) {
                    let wear = 1 + t.traffic / 25;
                    c.condition = if maintenance {
                        c.condition.saturating_add(8).min(100)
                    } else {
                        c.condition
                            .saturating_sub(wear * (150 - city.funding[0].min(150)) / 100)
                    };
                }
                if self.day.is_multiple_of(30)
                    && matches!(t.kind, Kind::Industrial | Kind::Gas)
                    && c.operational()
                    && t.connected
                    && t.powered
                    && t.watered
                    && t.fire == 0
                {
                    c.ore = c.ore.saturating_sub((t.jobs / 20).min(20) as u16);
                    c.oil = c.oil.saturating_sub((t.jobs / 30).min(12) as u16);
                }
                if t.kind.is_building() && c.age_days > 360 && self.day.is_multiple_of(30) {
                    c.condition = if maintenance {
                        c.condition.saturating_add(1).min(100)
                    } else {
                        c.condition.saturating_sub(1)
                    };
                }
                if c.condition < 35
                    && t.kind.is_building()
                    && hash(self.rng, self.day + i as u64).is_multiple_of(500)
                {
                    c.outage_days = 3 + (r % 5) as u8;
                    self.breakdowns += 1;
                }
                c.noise = (t.traffic as u16 / 2
                    + if matches!(t.kind, Kind::Airport | Kind::Airfield) {
                        60
                    } else if t.kind == Kind::Industrial {
                        35
                    } else {
                        0
                    })
                .min(100) as u8;
                c.heat = (temp.max(0) as u16 * 2 + t.level as u16 * 3 + c.air_pollution as u16 / 5)
                    .min(100) as u8;

                c.radiation = c
                    .radiation
                    .saturating_sub(u8::from(self.day.is_multiple_of(30)));
                if rain > 75
                    && c.elevation < 6
                    && t.terrain != Terrain::Water
                    && city
                        .neighbors(i)
                        .any(|j| city.tiles[j].terrain == Terrain::Water)
                {
                    c.flood = c.flood.saturating_add(rain / 8).min(100);
                    c.condition = c.condition.saturating_sub(2);
                } else {
                    c.flood = c.flood.saturating_sub(5);
                }
                if self.day.is_multiple_of(30)
                    && t.kind == Kind::Empty
                    && rain > 30
                    && c.elevation > 5
                {
                    if let Some(j) = city.neighbors(i).min_by_key(|&j| old_elevation[j]) {
                        if c.elevation > old_elevation[j] + 8 {
                            erode.push((i, j));
                        }
                    }
                }
            }
        }
        for (i, j) in erode {
            self.cells[i].elevation -= 1;
            self.cells[j].elevation = (self.cells[j].elevation + 1).min(48);
        }
        self.sewage_overflow = city.water_demand.saturating_sub(
            city.tiles
                .iter()
                .enumerate()
                .filter(|(i, t)| {
                    matches!(t.kind, Kind::Sewage | Kind::SewageOutlet)
                        && t.connected
                        && t.powered
                        && t.fire == 0
                        && city.funding[t.kind.funding()] > 0
                        && self.cells[*i].operational()
                })
                .map(|(_, t)| match t.kind {
                    Kind::Sewage => 16000,
                    Kind::SewageOutlet => 5000,
                    _ => 0,
                })
                .sum(),
        );
        self.power_reserve = city.power_supply as i64 - city.power_demand as i64;
        self.water_reserve = city.water_supply as i64 - city.water_demand as i64;
        self.tourism = city
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                matches!(
                    t.kind,
                    Kind::Hotel
                        | Kind::Airport
                        | Kind::WorldExpo
                        | Kind::Spaceport
                        | Kind::Stadium
                        | Kind::Zoo
                        | Kind::Aquarium
                        | Kind::ConcertHall
                ) && t.connected
                    && t.powered
                    && t.watered
                    && t.fire == 0
                    && city.funding[t.kind.funding()] > 0
                    && self.cells[*i].operational()
            })
            .map(|(_, t)| match t.kind {
                Kind::Hotel => t.jobs * 3,
                Kind::Airport => 500,
                Kind::WorldExpo => 3000,
                Kind::Spaceport => 2500,
                Kind::Stadium | Kind::Zoo | Kind::Aquarium | Kind::ConcertHall => 300,
                _ => 0,
            })
            .sum();
        // Goods are settled by each neighbor contract; this line is tourism
        // revenue only, so an export is never credited twice.
        self.trade_income =
            (self.tourism as i64 / 20) * city.civic.modifiers().trade_percent as i64 / 100;
        self.operating_cost =
            self.buildings.len() as i64 / 20 + self.transport.vehicles.len() as i64 / 30;
        for b in &mut self.buildings {
            b.age_days = b.age_days.saturating_add(1);
            if b.progress < 100 && city.money >= 0 {
                b.progress = b.progress.saturating_add(8).min(100);
                b.stage = if b.progress < 20 {
                    BuildingStage::Foundation
                } else if b.progress < 100 {
                    BuildingStage::Frame
                } else {
                    BuildingStage::Complete
                };
            }
            let c = &self.cells[b.tile as usize];
            b.condition = c.condition;
            if c.rubble {
                b.stage = BuildingStage::Rubble;
            } else if c.abandoned {
                b.stage = BuildingStage::Abandoned;
            }
        }
    }
    pub fn validate(&self, tiles: usize) -> Result<(), String> {
        for slot in 0..2 {
            if (!self.utility_masks[slot].is_empty() && self.utility_masks[slot].len() != tiles)
                || self.utility_masks[slot].iter().any(|&v| v > 1)
                || self.utility_grids[slot].iter().map(Vec::len).sum::<usize>() > tiles
                || self.utility_grids[slot].iter().any(|g| {
                    g.is_empty()
                        || g.iter().any(|&i| i >= tiles)
                        || g.windows(2).any(|p| p[0] >= p[1])
                })
            {
                return Err("Invalid cached utility components.".into());
            }
        }
        if self.minute_of_day >= 1440
            || self.day > 1_000_000_000
            || self.next_building > 1_000_000_000
            || (!self.cells.is_empty() && self.cells.len() != tiles)
            || self.buildings.len() > 300000
            || self.transport.vehicles.len() > 4096
            || self.transport.lines.len() > 128
        {
            return Err("Invalid expanded world dimensions or limits.".into());
        }
        if self
            .buildings
            .iter()
            .any(|b| b.tile as usize >= tiles || b.progress > 100 || b.condition > 100)
            || self
                .transport
                .lines
                .iter()
                .any(|l| l.stops.len() > 64 || l.stops.iter().any(|&s| s as usize >= tiles))
        {
            return Err("Invalid world entity references.".into());
        }
        if self.season > 3
            || !(-100..=100).contains(&self.weather.temperature)
            || !(-1..=1).contains(&self.weather.wind_x)
            || !(-1..=1).contains(&self.weather.wind_y)
            || self.weather.rain > 100
            || self.weather.cloud > 100
            || self.cells.iter().any(|c| {
                !(-16..=48).contains(&c.elevation)
                    || c.orientation > 3
                    || c.tree_species > 5
                    || [
                        c.moisture,
                        c.fertility,
                        c.condition,
                        c.construction,
                        c.heat,
                        c.noise,
                        c.radiation,
                        c.garbage,
                        c.flood,
                        c.air_pollution,
                    ]
                    .iter()
                    .any(|&v| v > 100)
            })
            || (!self.coverage.is_empty() && self.coverage.len() != tiles)
            || self.coverage.iter().flatten().any(|&v| v > 100)
            || self.neighbors.len() > 16
            || self.neighbors.iter().any(|n| {
                n.region.len() > 100
                    || n.power > 100000
                    || n.goods > 50000
                    || n.tariff > 25
                    || n.unit_price > 100
                    || n.last_power > n.power
            })
        {
            return Err("Invalid terrain, weather, service coverage or regional contract.".into());
        }
        let p = &self.people;
        if p.households.len() > tiles
            || [
                p.goods_demand,
                p.goods_produced,
                p.goods_imported,
                p.goods_exported,
                p.goods_consumed,
                p.goods_unused,
                p.goods_shortfall,
            ]
            .iter()
            .any(|&v| v > 100_000_000)
            || u64::from(p.goods_produced) + u64::from(p.goods_imported)
                != u64::from(p.goods_consumed)
                    + u64::from(p.goods_exported)
                    + u64::from(p.goods_unused)
            || u64::from(p.goods_demand)
                != u64::from(p.goods_consumed) + u64::from(p.goods_shortfall)
            || p.next_id > 1_000_000_000
            || p.birth_accumulator >= 360000
            || p.death_accumulator >= 360000
            || [p.births, p.deaths, p.migrants_in, p.migrants_out]
                .iter()
                .any(|&n| n > 1_000_000_000_000)
            || p.employment.len() > tiles.saturating_mul(16)
            || p.households.iter().any(|h| {
                h.home as usize >= tiles
                    || h.education > 100
                    || h.health > 100
                    || h.income_class > 2
                    || h.employer.is_some_and(|i| i as usize >= tiles)
            })
            || p.employment
                .iter()
                .any(|e| e.employer as usize >= tiles || e.workers > 65535)
            || p.unemployment > 100
            || p.happiness_factors.iter().any(|&v| v > 100)
        {
            return Err("Invalid population cohort or employment references.".into());
        }
        let t = &self.transport;
        let route_valid =
            |r: &[u32]| r.len() <= u16::MAX as usize && r.iter().all(|&i| (i as usize) < tiles);
        if t.next_id > 1_000_000_000
            || [t.fare_revenue, t.fare_balance, t.air_revenue_balance]
                .iter()
                .any(|&v| !(0..=1_000_000_000_000).contains(&v))
            || t.congestion > 100
            || t.road_wear > 100
            || t.average_commute > 600
            || t.trips as u64 != t.served_trips as u64 + t.unserved_trips as u64
            || t.ridership > t.served_trips
            || t.next_line > 1_000_000_000
            || t.vehicles.iter().any(|v| {
                v.tile as usize >= tiles
                    || v.next_tile as usize >= tiles
                    || v.heading > 3
                    || v.progress >= 1000
                    || v.age_days > 360
                    || !route_valid(&v.route)
                    || v.route.len() < 2
                    || v.leg as usize >= v.route.len()
            })
            || t.lines.iter().any(|l| {
                l.name.len() > 100
                    || l.fare > 25
                    || !route_valid(&l.route)
                    || !route_valid(&l.reverse_route)
            })
            || t.commute_routes.len() > tiles
            || t.commute_routes.iter().any(|r| {
                r.home as usize >= tiles || r.employer as usize >= tiles || !route_valid(&r.path)
            })
            || t.jams.iter().any(|&i| i as usize >= tiles)
            || (!t.road_access.is_empty() && t.road_access.len() != tiles)
            || [&t.nearest_road, &t.route_flow]
                .iter()
                .any(|v| !v.is_empty() && v.len() != tiles)
            || t.nearest_road
                .iter()
                .any(|&i| i != u32::MAX && i as usize >= tiles)
        {
            return Err("Invalid transport route, vehicle or cached access.".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        if self
            .buildings
            .iter()
            .any(|b| b.id == 0 || b.id > self.next_building || !ids.insert(b.id))
        {
            return Err("Invalid or duplicate building identifiers.".into());
        }
        self.services.validate(tiles)?;
        Ok(())
    }
}

#[cfg(test)]
mod building_sync_tests {
    use super::*;
    #[test]
    fn in_place_updates_and_partial_fallback_equal_the_complete_sorted_merge() {
        let mut city = City::new(17, true);
        city.initialize_world();
        let target = city
            .tiles
            .iter()
            .position(|t| t.kind == Kind::Residential)
            .unwrap();
        let mut fast = city.ext.clone();
        let mut full = fast.clone();
        for step in 0..10 {
            match step {
                1 => {
                    city.tiles[target].population = city.tiles[target].population.saturating_sub(1)
                }
                2 => city.tiles[target].level = 3,
                3 => city.tiles[target].level = 1,
                4 => {
                    fast.cells[target].construction = 35;
                    full.cells[target].construction = 35;
                }
                5 => {
                    fast.cells[target].condition = 23;
                    full.cells[target].condition = 23;
                    fast.cells[target].abandoned = true;
                    full.cells[target].abandoned = true;
                }
                6 => {
                    city.tiles[target].kind = Kind::Commercial;
                    city.tiles[target].population = 0;
                    city.tiles[target].jobs = 12;
                }
                7 => city.tiles[target] = crate::city::Tile::new(Terrain::Grass),
                8 => {
                    let last = city.tiles.len() - 1;
                    city.tiles[last] = crate::city::Tile::new(Terrain::Grass);
                    city.tiles[last].kind = Kind::Park;
                }
                9 => {
                    fast.buildings.swap(0, 2);
                    full.buildings.swap(0, 2);
                }
                _ => {}
            }
            city.month += 120;
            fast.sync_buildings(&city);
            full.sync_buildings_full(&city);
            assert!(
                fast == full,
                "building identity, stage or occupancy changed on step{step}"
            );
        }
    }
}
