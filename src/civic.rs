//! Saved civic authority. Daily hazards and monthly government share the city's
//! integer clock. Call `advance_day` once per day and `month_end` once AFTER the
//! ordinary monthly growth/budget step. The latter is idempotent by city.month.
use crate::city::{City, History, Kind, Terrain, Tile};
use crate::content::{self, CivicEvent, Facts, Metric, OrdinanceId, Perk};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Hazard {
    Earthquake,
    Flood,
    Meteor,
    Tornado,
    HostileUfo,
    FriendlyUfo,
    PlaneCrash,
    Monster,
    Riot,
    Meltdown,
}
impl Hazard {
    pub const ALL: [Self; 10] = [
        Self::Earthquake,
        Self::Flood,
        Self::Meteor,
        Self::Tornado,
        Self::HostileUfo,
        Self::FriendlyUfo,
        Self::PlaneCrash,
        Self::Monster,
        Self::Riot,
        Self::Meltdown,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Earthquake => "Earthquake",
            Self::Flood => "Flood",
            Self::Meteor => "Meteor",
            Self::Tornado => "Tornado",
            Self::HostileUfo => "Hostile UFO",
            Self::FriendlyUfo => "Friendly UFO",
            Self::PlaneCrash => "Plane crash",
            Self::Monster => "Monster",
            Self::Riot => "Riot",
            Self::Meltdown => "Meltdown",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bond {
    pub principal: i64,
    pub months_left: u16,
    pub monthly_interest: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectionResult {
    pub month: u32,
    pub votes: u8,
    pub won: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSample {
    pub month: u32,
    pub values: [i64; 16],
}
pub const GRAPH_NAMES: [&str; 16] = [
    "Population",
    "Treasury",
    "Tax income",
    "Running costs",
    "Monthly balance",
    "Happiness",
    "Health",
    "Education",
    "Unemployment",
    "Traffic",
    "Pollution",
    "Crime",
    "Land value",
    "Power coverage",
    "Water coverage",
    "Approval",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitizenSnapshot {
    pub id: u32,
    pub home: u32,
    pub size: u16,
    pub education: u8,
    pub health: u8,
    pub income_class: u8,
    pub employer: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: u16,
    pub month: u32,
    pub citizen: Option<u32>,
    pub tile: Option<u32>,
    #[serde(default)]
    pub person: Option<CitizenSnapshot>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookEntry {
    pub month: u32,
    pub title: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct District {
    pub id: u16,
    pub name: String,
    pub tiles: Vec<u32>,
    pub specialization: u8,
    pub population: u32,
    pub jobs: u32,
    /// Population-weighted local livability proxy, not citywide approval.
    pub happiness: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageSite {
    pub tile: u32,
    pub designated_month: u32,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRun {
    pub id: u8,
    pub tile: u32,
    pub months_done: u16,
    pub paid: i64,
    pub paused: bool,
    pub complete: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioResult {
    Running,
    Won,
    Lost,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioRun {
    pub id: u8,
    pub started_month: u32,
    pub deadline_month: u32,
    pub streak: u16,
    pub result: ScenarioResult,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisasterWarning {
    pub kind: Hazard,
    pub impact_day: u32,
    pub tile: u32,
    pub announced: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualEvent {
    pub kind: Hazard,
    pub tile: u32,
    pub started_day: u32,
    pub until_day: u32,
    pub intensity: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Celebration {
    pub project: u8,
    /// A real milestone award; absent on legacy and current project records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u8>,
    pub tile: u32,
    pub started_day: u32,
    pub until_day: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageRecord {
    pub tile: u32,
    pub previous: Tile,
    pub repair_cost: i64,
    pub day: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JanuarySnapshot {
    pub year: u32,
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<Tile>,
    pub values: [i64; 24],
    pub taxes: [u8; 3],
    pub funding: [u8; 4],
    pub policies: [bool; 4],
    pub history: Vec<History>,
    #[serde(default)]
    pub messages: Vec<String>,
    pub seed: u64,
    #[serde(with = "packed_world")]
    pub world: crate::world::WorldState,
    pub government: Vec<u8>,
}
impl JanuarySnapshot {
    /// Decode into an isolated city. Neither a failed load nor a failed rewind
    /// can change the running city, and checkpoints cannot contain checkpoints.
    fn candidate(&self) -> Result<City, String> {
        if self.government.len() > 8 * 1024 * 1024 {
            return Err("January government archive is too large.".into());
        }
        let value: serde_json::Value = serde_json::from_slice(&unpack(&self.government)?)
            .map_err(|e| format!("January civic snapshot: {e}"))?;
        if value.get("january").is_some_and(|v| !v.is_null()) {
            return Err("A January checkpoint cannot contain another checkpoint.".into());
        }
        let government: CivicState =
            serde_json::from_value(value).map_err(|e| format!("January civic snapshot: {e}"))?;
        government.validate(self.tiles.len())?;
        let v = self.values;
        let u32_at = |i: usize| {
            u32::try_from(v[i]).map_err(|_| "Invalid January unsigned statistic.".to_owned())
        };
        let percent_at = |i: usize| -> Result<u8, String> {
            if !(0..=100).contains(&v[i]) {
                return Err("Invalid January percentage.".into());
            }
            u8::try_from(v[i]).map_err(|_| "Invalid January percentage.".to_owned())
        };
        let demand_at = |i: usize| -> Result<i32, String> {
            if !(-100..=100).contains(&v[i]) {
                return Err("Invalid January demand.".into());
            }
            i32::try_from(v[i]).map_err(|_| "Invalid January demand.".to_owned())
        };
        let month = u32_at(1)?;
        if self.year != month / 12
            || v[20..].iter().any(|&n| n != 0)
            || [v[5], v[6]]
                .iter()
                .any(|n| !(0..=1_000_100_000).contains(n))
            || self.history.iter().any(|h| h.happiness > 100)
        {
            return Err("Invalid January calendar or budget statistics.".into());
        }
        let city = City {
            width: self.width,
            height: self.height,
            ext: self.world.clone(),
            civic: government,
            tiles: self.tiles.clone(),
            money: v[0],
            month,
            population: u32_at(2)?,
            jobs: u32_at(3)?,
            happiness: percent_at(4)?,
            income: v[5],
            expenses: v[6],
            power_supply: u32_at(7)?,
            power_demand: u32_at(8)?,
            water_supply: u32_at(9)?,
            water_demand: u32_at(10)?,
            debt: v[11],
            health: percent_at(12)?,
            education: percent_at(13)?,
            garbage: percent_at(14)?,
            unemployment: percent_at(15)?,
            transport_usage: percent_at(16)?,
            demand: [demand_at(17)?, demand_at(18)?, demand_at(19)?],
            taxes: self.taxes,
            funding: self.funding,
            policies: self.policies,
            history: self.history.clone(),
            messages: self.messages.clone(),
            seed: self.seed,
        };
        city.validate()?;
        Ok(city)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CivicModifiers {
    pub power_demand_percent: u16,
    pub water_demand_percent: u16,
    pub transit_bonus: u8,
    pub pollution_reduction: u8,
    pub crime_reduction: u8,
    pub disaster_reduction: u8,
    pub housing_capacity_percent: u16,
    pub education_bonus: u8,
    pub health_bonus: u8,
    pub trade_percent: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupKind {
    Milestone,
    Request,
    Achievement,
    ScenarioWon,
    ScenarioLost,
    DisasterWarning,
    Election,
    AnnualReview,
    Megaproject,
    Bankruptcy,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Popup {
    pub id: u64,
    pub month: u32,
    pub kind: PopupKind,
    pub title: String,
    pub text: String,
    pub tile: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Medal {
    FirstCommunity,
    CityOfCare,
    LearningCapital,
    GreenCity,
    ConnectedCity,
    SoundStewardship,
    ResilientCity,
    MasterBuilder,
}
impl Medal {
    pub const ALL: [Self; 8] = [
        Self::FirstCommunity,
        Self::CityOfCare,
        Self::LearningCapital,
        Self::GreenCity,
        Self::ConnectedCity,
        Self::SoundStewardship,
        Self::ResilientCity,
        Self::MasterBuilder,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::FirstCommunity => "First Community",
            Self::CityOfCare => "City of Care",
            Self::LearningCapital => "Learning Capital",
            Self::GreenCity => "Green City",
            Self::ConnectedCity => "Connected City",
            Self::SoundStewardship => "Sound Stewardship",
            Self::ResilientCity => "Resilient City",
            Self::MasterBuilder => "Master Builder",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub score: u32,
    pub grade: &'static str,
    pub medals: Vec<Medal>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CivicState {
    pub day: u32,
    pub rng: u64,
    pub ordinances: u32,
    pub ordinance_slots: u8,
    pub bonds: Vec<Bond>,
    pub bankruptcy_until: u32,
    pub last_bankruptcy_month: Option<u32>,
    pub approval: u8,
    pub elections: Vec<ElectionResult>,
    pub milestone_bits: u32,
    pub request_bits: u16,
    pub achievement_bits: u8,
    pub perks: u16,
    pub unlocks: u16,
    pub scenario: Option<ScenarioRun>,
    pub projects: Vec<ProjectRun>,
    pub completed_projects: u8,
    pub districts: Vec<District>,
    pub next_district: u16,
    pub heritage: Vec<HeritageSite>,
    pub history: Vec<GraphSample>,
    pub chirper: Vec<MessageRecord>,
    pub advisors: Vec<MessageRecord>,
    pub headlines: Vec<MessageRecord>,
    pub notebook: Vec<NotebookEntry>,
    pub warnings: Vec<DisasterWarning>,
    pub damage: Vec<DamageRecord>,
    pub active_events: Vec<VisualEvent>,
    pub celebrations: Vec<Celebration>,
    pub disaster_frequency: u8,
    pub drill_until: u32,
    pub last_drill_day: Option<u32>,
    pub last_disaster_day: Option<u32>,
    pub relief_claimed: bool,
    pub events: [u32; 12],
    pub milestone_streaks: Vec<u16>,
    pub request_streaks: Vec<u16>,
    pub achievement_streaks: Vec<u16>,
    pub last_month: Option<u32>,
    pub january: Option<Box<JanuarySnapshot>>,
    pub rewind_used: bool,
    pub hint_seen: u32,
    pub medal_bits: u8,
    pub popups: Vec<Popup>,
    pub next_popup: u64,
}
fn random(s: &mut u64) -> u64 {
    if *s == 0 {
        *s = 0xd1b54a32d192ed03;
    }
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
fn bounded_push<T>(v: &mut Vec<T>, item: T, cap: usize) {
    v.push(item);
    if v.len() > cap {
        v.remove(0);
    }
}
fn spend(city: &mut City, amount: i64) -> Result<(), String> {
    if amount < 0 || city.money < amount {
        return Err(format!("Treasury needs ${amount}."));
    }
    city.money -= amount;
    Ok(())
}
/// Available municipal overdraft before creditors require court protection.
/// It grows with the tax base but is finite even for the largest city.
pub fn overdraft_limit(city: &City) -> i64 {
    (20_000 + i64::from(city.population) * 10).min(100_000)
}
pub fn financial_emergency_text(city: &City) -> String {
    let review = city.month.saturating_add(1);
    format!(
        "Treasury ${}; overdraft limit -${}. Review due {}/{}. Open Advanced budget for bailout, restructuring or this year's January rewind. If the settled treasury exceeds its overdraft, court protection clears debt, resets cash to $5,000 and imposes twelve months of austerity and closed credit, with a major loss of trust.",
        city.money, overdraft_limit(city), review % 12 + 1, 2026 + review / 12
    )
}
fn note(city: &mut City, s: impl Into<String>) {
    bounded_push(&mut city.messages, s.into(), 16);
}
fn district_stats(city: &City, tiles: &[u32]) -> (u32, u32, u8) {
    let (mut population, mut jobs, mut weighted) = (0u32, 0u32, 0u64);
    for &i in tiles {
        let Some(t) = city.tiles.get(i as usize) else {
            continue;
        };
        population += t.population;
        jobs += t.jobs;
        let score = (u64::from(t.land_value)
            + u64::from(100 - t.pollution)
            + u64::from(100 - t.crime)
            + u64::from(100 - t.traffic)
            + u64::from(t.powered) * 100
            + u64::from(t.watered) * 100
            + u64::from(t.connected) * 100
            + u64::from(city.coverage(2, i as usize))
            + u64::from(city.coverage(5, i as usize)))
            / 9;
        weighted += u64::from(t.population) * score;
    }
    (
        population,
        jobs,
        weighted.checked_div(u64::from(population)).unwrap_or(0) as u8,
    )
}

fn scenario_lot(city: &mut City, x: i32, y: i32, kind: Kind, level: u8, occupants: u32) {
    let i = city.idx(x, y).expect("authored scenario coordinate");
    let mut t = Tile::new(Terrain::Grass);
    t.kind = kind;
    t.level = level;
    if kind == Kind::Residential {
        t.population = occupants;
    } else {
        t.jobs = occupants;
    }
    city.tiles[i] = t;
    if kind.is_building()
        && !kind.is_zone()
        && !city.neighbors(i).any(|j| city.tiles[j].kind.is_road())
    {
        // Established public facilities include a real one-tile driveway when
        // set back from the block's street. Vehicles cannot teleport to a road.
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if city
                .tile(x + dx * 2, y + dy * 2)
                .is_some_and(|t| t.kind.is_road())
                && city
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.kind == Kind::Empty)
            {
                scenario_lot(city, x + dx, y + dy, Kind::Road, 0, 0);
                break;
            }
        }
    }
}

/// Scenario fixtures are authored starting towns, never free edits performed by
/// the player. Roads, jobs, supply and damage are real rules data from day zero.
fn scenario_town(id: u8, def: &content::ScenarioDef) -> City {
    use Kind::*;
    let mut city = City::world(def.seed, def.style, def.climate);
    for y in 40..=88 {
        for x in 40..=88 {
            scenario_lot(&mut city, x, y, Empty, 0, 0);
        }
    }
    let stride = if id == 0 { 8 } else { 4 };
    for y in 44..=84 {
        for x in 44..=84 {
            if (x - 44) % stride == 0 || (y - 44) % stride == 0 {
                scenario_lot(&mut city, x, y, if id == 1 { Avenue } else { Road }, 0, 0);
            }
        }
    }
    // A single western outside connection is deliberate: the harbor's isolated
    // eastern wharf must not accidentally be served from the far map edge.
    for x in 0..=44 {
        scenario_lot(&mut city, x, 64, Road, 0, 0);
    }
    let power: &[Kind] = match id {
        0 => &[Wind, Wind, Wind, Wind],
        6 => &[Coal, Coal, Coal],
        _ => &[Gas, Gas],
    };
    for (k, &kind) in power.iter().enumerate() {
        scenario_lot(&mut city, 46, 46 + k as i32 * 4, kind, 0, 0);
    }
    let towers = match id {
        0 => 1,
        2 => 2,
        4..=6 => 3,
        _ => 5,
    };
    for k in 0..towers {
        scenario_lot(&mut city, 50 + k * 4, 46, WaterTower, 0, 0);
    }
    for (x, kind) in [
        (46, Police),
        (50, Fire),
        (54, Clinic),
        (58, School),
        (62, Library),
        (66, Cemetery),
        (70, Recycling),
        (74, Landfill),
        (78, Park),
        (82, MaintenanceDepot),
    ] {
        scenario_lot(&mut city, x, if x <= 62 { 58 } else { 82 }, kind, 0, 0);
    }
    scenario_lot(
        &mut city,
        82,
        74,
        if id == 2 { SewageOutlet } else { Sewage },
        0,
        0,
    );
    if id == 1 || id == 3 || id == 7 {
        scenario_lot(&mut city, 82, 78, Sewage, 0, 0);
    }
    if id == 0 {
        for (x, y) in [(54, 54), (62, 62), (70, 70)] {
            scenario_lot(&mut city, x, y, Park, 0, 0);
        }
    }
    if id == 1 {
        for y in 40..=88 {
            let i = city.idx(63, y).unwrap();
            city.tiles[i] = Tile::new(Terrain::Water);
        }
        scenario_lot(&mut city, 63, 64, Road, 0, 0);
        city.tiles[64 * city.width as usize + 63].terrain = Terrain::Water;
        scenario_lot(&mut city, 58, 50, BusDepot, 0, 0);
        scenario_lot(&mut city, 58, 49, BusStop, 0, 0);
        scenario_lot(&mut city, 70, 49, BusStop, 0, 0);
    }
    if id == 2 {
        for x in [41, 61, 85] {
            for y in 40..=88 {
                let i = city.idx(x, y).unwrap();
                city.tiles[i] = Tile::new(Terrain::Water);
            }
            scenario_lot(&mut city, x, 64, Road, 0, 0);
            city.tiles[64 * city.width as usize + x as usize].terrain = Terrain::Water;
        }
    }
    if id == 3 {
        for x in 85..=88 {
            scenario_lot(&mut city, x, 64, Road, 0, 0);
        }
        for y in 56..=76 {
            for x in 90..=104 {
                scenario_lot(&mut city, x, y, Empty, 0, 0);
            }
            for x in 96..=104 {
                let i = city.idx(x, y).unwrap();
                city.tiles[i].terrain = Terrain::Water;
            }
        }
        for x in 91..=95 {
            scenario_lot(&mut city, x, 64, Road, 0, 0);
        }
        scenario_lot(&mut city, 94, 65, Seaport, 0, 0);
        scenario_lot(&mut city, 94, 61, FreightStation, 0, 0);
        scenario_lot(&mut city, 94, 69, Marina, 0, 0);
        scenario_lot(&mut city, 90, 64, Empty, 0, 0);
        scenario_lot(&mut city, 89, 64, Empty, 0, 0);
    }
    if id == 4 {
        scenario_lot(&mut city, 66, 54, University, 0, 0);
        scenario_lot(&mut city, 62, 54, School, 0, 0);
        scenario_lot(&mut city, 58, 54, Library, 0, 0);
    }
    if id == 6 {
        scenario_lot(&mut city, 50, 54, Wind, 0, 0);
    }
    if id == 7 {
        for (x, y) in [(70, 78), (74, 78), (78, 78)] {
            scenario_lot(&mut city, x, y, Stadium, 0, 0);
        }
        city.taxes = [15, 14, 14];
        city.funding = [150; 4];
        city.debt = 30000;
    }
    let mut residents = def.start_population;
    let mut jobs = def.start_population * 55 / 100;
    let home_level = if id == 0 { 1 } else { 3 };
    let home_capacity = if id == 0 { 24 } else { 144 };
    for y in 45..=83 {
        for x in 45..=83 {
            let i = city.idx(x, y).unwrap();
            if city.tiles[i].kind != Empty || city.tiles[i].terrain == Terrain::Water {
                continue;
            }
            if id == 0 && (x - 44) % stride == 4 && (y - 44) % stride == 4 {
                continue;
            }
            if x <= 61 && residents > 0 {
                let n = residents.min(home_capacity);
                scenario_lot(&mut city, x, y, Residential, home_level, n);
                residents -= n;
            } else if x >= 65 && jobs > 0 {
                let kind = if id == 0 {
                    if (x + y) % 5 == 0 {
                        Commercial
                    } else {
                        Farm
                    }
                } else if (x + y) % 4 == 0 {
                    Commercial
                } else {
                    Industrial
                };
                let level = if id == 0 { 2 } else { 3 };
                let capacity = match kind {
                    Farm => 24,
                    Commercial if level == 2 => 48,
                    Commercial => 120,
                    _ => 100,
                };
                let n = jobs.min(capacity);
                scenario_lot(&mut city, x, y, kind, level, n);
                jobs -= n;
            }
        }
    }
    assert_eq!((residents, jobs), (0, 0), "authored scenario capacity");
    city.money = def.start_money;
    city.ext = crate::world::WorldState::new(&city, def.seed, def.style, def.climate);
    for y in 40..=88 {
        for x in 40..=88 {
            let i = city.idx(x, y).unwrap();
            let c = &mut city.ext.cells[i];
            c.construction = 100;
            c.condition = 100;
            c.age_days = 720;
            // The established town includes buried municipal conduits. Street
            // access remains a separate physical network, including harbor gaps.
            c.pipe = true;
            c.sewer = true;
            c.underground_power = true;
            if city.tiles[i].terrain != Terrain::Water {
                c.elevation = if id == 2 { 2 } else { 8 };
            }
        }
    }
    if id == 2 {
        let i = city.idx(50, 46).unwrap();
        city.ext.cells[i].outage_days = 45;
    }
    if id == 3 {
        city.ext.neighbors[0].enabled = true;
        city.ext.neighbors[0].goods = 8000;
    }
    city.refresh();
    let mut people = crate::people::PeopleState::default();
    people.reconcile(&city);
    if id == 4 {
        for h in &mut people.households {
            h.education = 15;
        }
        city.education = 20;
        city.unemployment = 30;
    }
    city.ext.people = people;
    city.history.clear();
    city.messages.clear();
    city
}

/// Empty tiles never dilute residential health or congestion measurements.
pub fn facts(city: &City) -> Facts {
    use Metric::*;
    let mut f = Facts::default();
    f.counts.reserve(Kind::ALL.len());
    // Preserve first-encounter order without searching the kind list for each
    // map tile. The enum/catalog is dense; the slot table is transient only.
    let mut count_index = [usize::MAX; Kind::ALL.len()];
    for (m, v) in [
        (Population, city.population as i64),
        (Treasury, city.money),
        (NetIncome, city.income - city.expenses),
        (Happiness, city.happiness as i64),
        (Health, city.health as i64),
        (Education, city.education as i64),
        (Unemployment, city.unemployment as i64),
        (Garbage, city.garbage as i64),
        (Transit, city.transport_usage as i64),
        (Debt, city.debt),
        (Months, city.month as i64),
    ] {
        f.set(m, v);
    }
    let (
        mut residents,
        mut pollution,
        mut crime,
        mut value,
        mut served,
        mut powered,
        mut watered,
        mut traffic,
        mut roads,
        mut green,
        mut total,
    ) = (
        0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64,
    );
    for (i, t) in city.tiles.iter().enumerate() {
        let slot = &mut count_index[t.kind as usize];
        let at = if *slot != usize::MAX {
            *slot
        } else {
            *slot = f.counts.len();
            f.counts.push((t.kind, 0, 0));
            *slot
        };
        f.counts[at].1 = f.counts[at].1.saturating_add(1);
        if crate::transport::facility_operational(city, i) {
            f.counts[at].2 = f.counts[at].2.saturating_add(1);
        }
        if t.kind == Kind::Residential {
            let n = t.population as u64;
            residents += n;
            pollution += n * t.pollution as u64;
            crime += n * t.crime as u64;
            value += n * t.land_value as u64;
            f.values[ResidentialTiles as usize] += 1;
        }
        if t.kind == Kind::Commercial {
            f.values[CommercialJobs as usize] += t.jobs as i64;
        }
        if t.kind == Kind::Industrial {
            f.values[IndustrialJobs as usize] += t.jobs as i64;
        }
        if t.kind == Kind::Park {
            f.values[Parks as usize] += 1;
        }
        if t.terrain == Terrain::Forest {
            f.values[ForestTiles as usize] += 1;
        }
        let n = (t.population + t.jobs) as u64;
        if n > 0 {
            served += n;
            if t.powered {
                powered += n;
            }
            if t.watered {
                watered += n;
            }
        }
        if t.kind.is_road() && t.connected {
            traffic += t.traffic as u64;
            roads += 1;
        }
        if t.connected && t.fire == 0 {
            let output = u64::from(city.power_output(i));
            total += output;
            if matches!(
                t.kind,
                Kind::Wind | Kind::Solar | Kind::Hydro | Kind::Fusion
            ) {
                green += output;
            }
        }
    }
    for (m, n, d) in [
        (Pollution, pollution, residents),
        (Crime, crime, residents),
        (LandValue, value, residents),
        (Traffic, traffic, roads),
        (PowerCoverage, powered * 100, served),
        (WaterCoverage, watered * 100, served),
        (RenewableShare, green * 100, total),
    ] {
        f.set(
            m,
            n.checked_div(d).map(|v| v as i64).unwrap_or_else(|| {
                if matches!(m, PowerCoverage | WaterCoverage) {
                    100
                } else {
                    0
                }
            }),
        );
    }
    f.set(OperationalAirport, f.count(Kind::Airport, true) as i64);
    f.set(OperationalPort, f.count(Kind::Seaport, true) as i64);
    f
}
impl CivicState {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seed ^ 0x72c4a910ab37d221,
            ordinance_slots: 3,
            approval: 50,
            disaster_frequency: 1,
            next_district: 1,
            ..Default::default()
        }
    }
    pub fn active(&self, id: OrdinanceId) -> bool {
        self.ordinances & (1 << id as u8) != 0
    }
    pub fn note_event(&mut self, event: CivicEvent, n: u32) {
        self.events[event as usize] = self.events[event as usize].saturating_add(n);
    }
    pub fn next_milestone(&self) -> Option<usize> {
        (0..25).find(|i| self.milestone_bits & (1 << i) == 0)
    }
    pub fn current_facts(&self, city: &City) -> Facts {
        let mut f = facts(city);
        f.events = self.events;
        f.ordinances = self.ordinances;
        f.set(Metric::Approval, self.approval as i64);
        f.set(Metric::HeritageSites, self.heritage.len() as i64);
        f.set(Metric::ActiveWarnings, self.warnings.len() as i64);
        f.set(Metric::UnrepairedDamage, self.damage.len() as i64);
        f.set(
            Metric::ReliefAvailable,
            i64::from(
                !self.relief_claimed
                    && !self.damage.is_empty()
                    && self
                        .last_disaster_day
                        .is_some_and(|d| self.day.saturating_sub(d) <= 30),
            ),
        );
        f.set(
            Metric::CompletedProjects,
            self.completed_projects.count_ones() as i64,
        );
        f.set(
            Metric::ElectionsWon,
            self.events[CivicEvent::WonElection as usize] as i64,
        );
        f.set(
            Metric::DisastersSurvived,
            self.events[CivicEvent::SurvivedDisaster as usize] as i64,
        );
        f.set(
            Metric::BondsOutstanding,
            self.bonds.iter().map(|b| b.principal).sum(),
        );
        f.set(
            Metric::SuccessfulScenarios,
            self.events[CivicEvent::WonScenario as usize] as i64,
        );
        f
    }
    pub fn toggle_ordinance(&mut self, city: &mut City, id: OrdinanceId) -> Result<(), String> {
        let bit = 1 << id as u8;
        if self.ordinances & bit == 0 {
            if self.ordinances.count_ones() >= self.ordinance_slots.max(3) as u32 {
                return Err("All ordinance slots are occupied. Repeal one first.".into());
            }
            spend(city, 100)?;
        }
        self.ordinances ^= bit;
        for i in 0..4 {
            city.policies[i] = self.ordinances & (1 << i) != 0;
        }
        city.refresh();
        Ok(())
    }
    pub fn modifiers(&self) -> CivicModifiers {
        let mut m = CivicModifiers {
            power_demand_percent: 100,
            water_demand_percent: 100,
            housing_capacity_percent: 100,
            trade_percent: 100,
            ..Default::default()
        };
        if self.active(OrdinanceId::EnergyEfficiency) {
            m.power_demand_percent = 88;
        }
        if self.active(OrdinanceId::WaterConservation) {
            m.water_demand_percent = 85;
        }
        if self.active(OrdinanceId::CleanAir) {
            m.pollution_reduction = 12;
        }
        if self.active(OrdinanceId::FreeTransit) {
            m.transit_bonus += 15;
        }
        if self.active(OrdinanceId::TruckRoutes) {
            m.transit_bonus += 5;
        }
        if self.active(OrdinanceId::NoiseControl) {
            m.trade_percent = m.trade_percent.saturating_sub(10);
        }
        if self.active(OrdinanceId::PoliceAccountability) {
            m.crime_reduction = 8;
        }
        if self.active(OrdinanceId::DisasterPreparedness) {
            m.disaster_reduction += 30;
        }
        if self.active(OrdinanceId::IndustrialSafety) {
            m.disaster_reduction += 15;
        }
        if self.perks & (1 << Perk::ResilienceOffice as u8) != 0 {
            m.disaster_reduction += 20;
        }
        if self.drill_until > self.day {
            m.disaster_reduction += 20;
        }
        if self.completed_projects & 1 != 0 {
            m.housing_capacity_percent = 125;
            m.power_demand_percent = m.power_demand_percent.saturating_sub(5);
            m.water_demand_percent = m.water_demand_percent.saturating_sub(5);
        }
        if self.completed_projects & 2 != 0 {
            m.trade_percent += 30;
        }
        if self.completed_projects & 4 != 0 {
            m.trade_percent += 20;
        }
        if self.completed_projects & 8 != 0 {
            m.education_bonus += 12;
            m.health_bonus += 8;
        }
        m
    }
    pub fn issue_bond(&mut self, city: &mut City, amount: i64) -> Result<(), String> {
        if ![10000, 25000, 50000].contains(&amount) {
            return Err("Bond issues are $10,000, $25,000 or $50,000.".into());
        }
        if self.bonds.len() >= 3 {
            return Err("The city may have three outstanding bond issues.".into());
        }
        let total: i64 = self.bonds.iter().map(|b| b.principal).sum();
        let ceiling = (20000 + city.population as i64 * 12).min(200000);
        if total + city.debt + amount > ceiling {
            return Err("This bond exceeds the city's debt capacity.".into());
        }
        if city.month < self.bankruptcy_until {
            return Err("Credit markets remain closed during restructuring.".into());
        }
        city.money += amount - amount / 100;
        self.bonds.push(Bond {
            principal: amount,
            months_left: 60,
            monthly_interest: (amount / 200).max(1),
        });
        Ok(())
    }
    pub fn repay_bond(&mut self, city: &mut City, index: usize) -> Result<(), String> {
        let amount = self
            .bonds
            .get(index)
            .ok_or("No such bond issue.")?
            .principal;
        spend(city, amount)?;
        self.bonds.remove(index);
        self.note_event(CivicEvent::RepaidDebt, 1);
        Ok(())
    }
    pub fn declare_bankruptcy(&mut self, city: &mut City) -> Result<(), String> {
        if self
            .last_bankruptcy_month
            .map(|m| city.month.saturating_sub(m) < 60)
            .unwrap_or(false)
        {
            return Err("Bankruptcy protection is available once per five years.".into());
        }
        if city.debt == 0 && self.bonds.is_empty() && city.money >= 0 {
            return Err("The city has no insolvency to restructure.".into());
        }
        self.enter_bankruptcy_protection(city, false);
        Ok(())
    }
    fn enter_bankruptcy_protection(&mut self, city: &mut City, court_ordered: bool) {
        let prior_cash = city.money;
        let discharged = city.debt + self.bonds.iter().map(|b| b.principal).sum::<i64>();
        let limit = overdraft_limit(city);
        city.debt = 0;
        self.bonds.clear();
        city.money = 5000;
        self.bankruptcy_until = city.month.saturating_add(12);
        self.last_bankruptcy_month = Some(city.month);
        city.funding = city.funding.map(|f| f.min(70));
        self.approval = self.approval.saturating_sub(30);
        city.happiness = city.happiness.saturating_sub(20);
        self.note_event(CivicEvent::DeclaredBankruptcy, 1);
        let title = if court_ordered {
            "Court receivership"
        } else {
            "Bankruptcy protection"
        };
        let text = format!(
            "{} Treasury was ${prior_cash}; overdraft limit -${limit}. Creditors discharged ${discharged} of loans/bonds and reset cash to $5,000. Approval fell 30 points, happiness 20, and department funding is capped at 70%. Credit remains closed until {}/{}.",
            if court_ordered { "The settled treasury exceeded its credit limit." } else { "The city elected bankruptcy protection." },
            self.bankruptcy_until % 12 + 1, 2026 + self.bankruptcy_until / 12
        );
        self.journal(city.month, title, &text);
        self.popup(city.month, PopupKind::Bankruptcy, title, &text, None);
        note(
            city,
            format!("{title}: debts discharged; twelve months of austerity and closed credit."),
        );
    }
    pub fn bailout(&mut self, city: &mut City) -> Result<(), String> {
        if self.bonds.len() >= 3 {
            return Err(
                "Repay or restructure an outstanding bond before regional aid can be issued."
                    .into(),
            );
        }
        if self
            .last_bankruptcy_month
            .map(|m| city.month.saturating_sub(m) < 60)
            .unwrap_or(false)
        {
            return Err("Regional financial aid has a five-year cooldown.".into());
        }
        if city.money >= 0 && city.income >= city.expenses {
            return Err("A solvent city is not eligible for an emergency bailout.".into());
        }
        city.money += 25000;
        self.bonds.push(Bond {
            principal: 20000,
            months_left: 120,
            monthly_interest: 100,
        });
        city.taxes = city.taxes.map(|t| t.max(12));
        self.bankruptcy_until = city.month + 6;
        self.last_bankruptcy_month = Some(city.month);
        self.approval = self.approval.saturating_sub(15);
        self.journal(
            city.month,
            "Regional bailout",
            "Aid arrived with tax conditions and a temporary credit restriction.",
        );
        Ok(())
    }
    pub fn restructure(&mut self, city: &mut City) -> Result<(), String> {
        if city.debt == 0 && self.bonds.is_empty() {
            return Err("There is no debt to restructure.".into());
        }
        if self
            .last_bankruptcy_month
            .map(|m| city.month.saturating_sub(m) < 60)
            .unwrap_or(false)
        {
            return Err("A recent financial settlement prevents another restructuring.".into());
        }
        let total = city.debt + self.bonds.iter().map(|b| b.principal).sum::<i64>();
        if total * 4 / 5 > 200_000 {
            return Err("This debt exceeds the negotiated restructuring limit. Bankruptcy protection can discharge it.".into());
        }
        city.debt = 0;
        self.bonds = vec![Bond {
            principal: total * 4 / 5,
            months_left: 120,
            monthly_interest: (total / 250).max(1),
        }];
        city.money = city.money.max(0) + 3000;
        self.bankruptcy_until = city.month + 6;
        self.last_bankruptcy_month = Some(city.month);
        self.approval = self.approval.saturating_sub(10);
        self.journal(
            city.month,
            "Debt restructuring",
            "Creditors accepted a twenty-percent principal reduction and a ten-year payment term.",
        );
        Ok(())
    }
    pub fn create_district(
        &mut self,
        city: &mut City,
        name: String,
        mut tiles: Vec<u32>,
        specialization: u8,
    ) -> Result<u16, String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 24 || name.chars().any(|c| c.is_control()) {
            return Err("Use a district name of one to twenty-four printable characters.".into());
        }
        if self.districts.len() >= 16 || specialization > 4 {
            return Err("District limit is sixteen; specializations are 0 through 4.".into());
        }
        tiles.sort_unstable();
        tiles.dedup();
        if tiles.len() < 4
            || tiles.len() > 4096
            || tiles.iter().any(|&i| {
                i as usize >= city.tiles.len()
                    || self
                        .districts
                        .iter()
                        .any(|d| d.tiles.binary_search(&i).is_ok())
            })
        {
            return Err("Choose 4..4096 unassigned tiles inside the map.".into());
        }
        let id = self.next_district.max(1);
        if id == u16::MAX {
            return Err("District identifiers are exhausted.".into());
        }
        spend(city, 100 + tiles.len() as i64 * 2)?;
        self.next_district = id + 1;
        let (population, jobs, happiness) = district_stats(city, &tiles);
        self.districts.push(District {
            id,
            name: name.into(),
            tiles,
            specialization,
            population,
            jobs,
            happiness,
        });
        Ok(id)
    }
    pub fn designate_heritage(&mut self, city: &mut City, tile: u32) -> Result<(), String> {
        let t = *city.tiles.get(tile as usize).ok_or("Outside the map.")?;
        if city.ext.enabled
            && city
                .ext
                .cells
                .get(tile as usize)
                .map(|c| city.month.saturating_sub(c.built_month) < 12)
                .unwrap_or(true)
        {
            return Err("Heritage buildings must be at least one year old.".into());
        }
        if !t.kind.is_building() || t.fire > 0 || (t.kind.is_zone() && t.level == 0) {
            return Err("Choose an occupied or civic building in sound condition.".into());
        }
        if self.heritage.len() >= 64 || self.heritage.iter().any(|h| h.tile == tile) {
            return Err("The heritage register is full or already includes that lot.".into());
        }
        spend(city, 500)?;
        self.heritage.push(HeritageSite {
            tile,
            designated_month: city.month,
            name: format!("{} historic site", t.kind.label()),
        });
        self.note_event(CivicEvent::SavedHeritage, 1);
        Ok(())
    }
    pub fn protected(&self, tile: u32) -> bool {
        self.active(OrdinanceId::Heritage) && self.heritage.iter().any(|h| h.tile == tile)
    }
    pub fn demolition_permission(
        &self,
        tile: u32,
        override_protection: bool,
    ) -> Result<i64, String> {
        if self.protected(tile) {
            if !override_protection {
                return Err(
                    "Heritage protection prevents demolition; revoke the designation explicitly."
                        .into(),
                );
            }
            Ok(10000)
        } else {
            Ok(0)
        }
    }
    pub fn remove_heritage(&mut self, city: &mut City, tile: u32) -> Result<(), String> {
        let i = self
            .heritage
            .iter()
            .position(|h| h.tile == tile)
            .ok_or("That lot is not registered.")?;
        spend(city, 1000)?;
        self.heritage.remove(i);
        self.approval = self.approval.saturating_sub(3);
        Ok(())
    }
    pub fn set_disaster_frequency(&mut self, value: u8) -> Result<(), String> {
        if value > 3 {
            return Err("Disaster frequency is 0..3.".into());
        }
        self.disaster_frequency = value;
        Ok(())
    }
    pub fn schedule_disaster(&mut self, kind: Hazard, delay_days: u16) -> Result<(), String> {
        if self.warnings.len() >= 8 || !(1..=3600).contains(&delay_days) {
            return Err("At most eight events may be scheduled, 1..3600 days ahead.".into());
        }
        self.warnings.push(DisasterWarning {
            kind,
            impact_day: self.day.saturating_add(delay_days as u32),
            tile: 0,
            announced: false,
        });
        Ok(())
    }
    pub fn drill(&mut self, city: &mut City) -> Result<(), String> {
        if self
            .last_drill_day
            .map(|d| self.day.saturating_sub(d) < 180)
            .unwrap_or(false)
        {
            return Err("Emergency drills have a 180-day cooldown.".into());
        }
        spend(city, 1000 + city.population as i64 / 10)?;
        self.last_drill_day = Some(self.day);
        self.drill_until = self.day + 360;
        self.note_event(CivicEvent::HeldDrill, 1);
        Ok(())
    }
    pub fn request_relief(&mut self, city: &mut City) -> Result<(), String> {
        if self.relief_claimed
            || self
                .last_disaster_day
                .map(|d| self.day.saturating_sub(d) > 30)
                .unwrap_or(true)
            || self.damage.is_empty()
        {
            return Err(
                "Relief is available once within thirty days of a damaging emergency.".into(),
            );
        }
        let grant = (self.damage.iter().map(|d| d.repair_cost).sum::<i64>() / 2).clamp(1000, 25000);
        city.money += grant;
        self.relief_claimed = true;
        self.journal(
            city.month,
            "Disaster relief",
            &format!("The region awarded ${grant} toward verified damage."),
        );
        Ok(())
    }
    pub fn repair(&mut self, city: &mut City, tile: u32) -> Result<(), String> {
        let at = self
            .damage
            .iter()
            .position(|d| d.tile == tile)
            .ok_or("No repairable damage is registered on that lot.")?;
        let damage = self.damage[at].clone();
        let t = city.tiles.get(tile as usize).ok_or("Outside the map.")?;
        if t.kind != Kind::Empty && t.kind != damage.previous.kind {
            return Err("A new structure occupies this lot. Remove it before restoration.".into());
        }
        spend(city, damage.repair_cost)?;
        let mut restored = damage.previous;
        restored.population = 0;
        restored.jobs = 0;
        restored.fire = 0;
        city.tiles[tile as usize] = restored;
        if let Some(c) = city.ext.cells.get_mut(tile as usize) {
            c.rubble = false;
            c.condition = 75;
            c.construction = 50;
            c.radiation = 0;
            c.flood = 0;
        }
        self.damage.remove(at);
        self.note_event(CivicEvent::RepairedDamage, 1);
        city.refresh();
        Ok(())
    }
    fn journal(&mut self, month: u32, title: &str, text: &str) {
        bounded_push(
            &mut self.notebook,
            NotebookEntry {
                month,
                title: title.into(),
                text: text.into(),
            },
            120,
        );
    }
}
impl CivicState {
    pub fn can_build(&self, kind: Kind, population: u32) -> Result<(), String> {
        let gate = match kind {
            Kind::University => 4500,
            Kind::Seaport => 6000,
            Kind::Airport => 8000,
            Kind::Nuclear => 12500,
            Kind::Stadium => 3000,
            Kind::Arcology | Kind::Spaceport | Kind::WorldExpo | Kind::ScienceCenter => 25000,
            _ => 0,
        };
        if population < gate {
            return Err(format!("{} unlocks at {gate} residents.", kind.label()));
        }
        let project = match kind {
            Kind::Arcology => Some(0),
            Kind::Spaceport => Some(1),
            Kind::WorldExpo => Some(2),
            Kind::ScienceCenter => Some(3),
            _ => None,
        };
        if project
            .map(|i| self.completed_projects & (1 << i) == 0)
            .unwrap_or(false)
        {
            return Err("Commission this megaproject through the civic project program.".into());
        }
        Ok(())
    }
    pub fn reserved(&self, index: u32) -> bool {
        self.projects.iter().any(|p| p.tile == index && !p.complete)
    }
    pub fn can_edit_tile(&self, index: u32) -> Result<(), String> {
        if self.reserved(index) {
            Err("This lot is reserved for an active megaproject.".into())
        } else {
            Ok(())
        }
    }
    pub fn can_bulldoze(&self, index: u32) -> Result<(), String> {
        self.can_edit_tile(index)?;
        self.demolition_permission(index, false).map(|_| ())
    }
    pub fn cancel_project(&mut self, city: &mut City, id: u8) -> Result<(), String> {
        let i = self
            .projects
            .iter()
            .position(|p| p.id == id && !p.complete)
            .ok_or("No active project with that identifier.")?;
        let run = self.projects.remove(i);
        city.money += run.paid / 4;
        self.journal(city.month,"Project cancelled","A quarter of paid construction costs was recovered. The site and construction slot are available again.");
        Ok(())
    }
    pub fn start_project(&mut self, city: &mut City, id: u8, tile: u32) -> Result<(), String> {
        let def = content::MEGAPROJECTS
            .get(id as usize)
            .ok_or("Unknown megaproject.")?;
        if self.projects.iter().filter(|p| !p.complete).count() >= 2 {
            return Err("Both megaproject construction slots are occupied.".into());
        }
        if self.projects.iter().any(|p| p.id == id) || self.completed_projects & (1 << id) != 0 {
            return Err("That unique megaproject is already commissioned.".into());
        }
        let t = city.tiles.get(tile as usize).ok_or("Outside the map.")?;
        if t.kind != Kind::Empty
            || t.terrain == Terrain::Water
            || self.projects.iter().any(|p| p.tile == tile)
        {
            return Err("Choose an unoccupied land lot for the project.".into());
        }
        if !self.current_facts(city).meets(def.goal) {
            return Err("The city does not meet this project's published prerequisites.".into());
        }
        let deposit = def.cost / 4;
        spend(city, deposit)?;
        self.projects.push(ProjectRun {
            id,
            tile,
            months_done: 0,
            paid: deposit,
            paused: false,
            complete: false,
        });
        self.journal(
            city.month,
            def.title,
            "Site commissioned; monthly installments begin next month.",
        );
        Ok(())
    }
    pub fn start_scenario(&mut self, city: &mut City, id: u8) -> Result<(), String> {
        let def = content::SCENARIOS
            .get(id as usize)
            .ok_or("Unknown scenario.")?;
        let mut fresh = scenario_town(id, def);
        let mut government = Self::new(def.seed);
        government.scenario = Some(ScenarioRun {
            id,
            started_month: 0,
            deadline_month: u32::from(def.deadline_months),
            streak: 0,
            result: ScenarioResult::Running,
        });
        government.journal(0, def.title, def.briefing);
        let first_steps = match id {
            0 => "This quiet highland farming village has four wind turbines and low-density homes. Extend village roads, add jobs and parks, and keep pollution below 30 while growing toward 4,000 residents.",
            1 => "Homes west of the river and employers east of it share one road bridge at 63,64. A bus depot and two stops exist but no line runs. Create a useful service or another crossing, then sustain 35% transit use and traffic below 45.",
            2 => "The delta is low and wet. One water tower is broken for 45 days and an outlet cannot handle all sewage. Restore capacity, reserve flood-recovery funds, and prepare for warnings in months 6 and 18.",
            3 => "The eastern wharf is cut off: build streets across 89,64 and 90,64 to reach the port at 94,65. Its freight station and marina need connections too. An 8,000-unit goods agreement is ready; grow industrial employment and a profitable clean harbor.",
            4 => "An alpine university town has lost its skilled workforce. The campus and schools exist, but residents start with little schooling and advanced industrial vacancies cannot be filled. Invest in education while providing entry-level work.",
            5 => "Eight commercial and industrial lots were damaged by a recent fire. A working maintenance depot can dispatch paid repair crews. Keep their routes clear, add a second fire station, and restore confidence without losing the surviving neighborhoods.",
            6 => "Three coal plants power this lakeside town. One wind turbine is only a small start. Replace the dirty generation with sufficient clean capacity while preserving water, jobs and your cash reserve.",
            _ => "The capital owes 30,000 in municipal loans and 25,000 in bonds. High taxes, maximum department funding and three stadiums strain confidence. Retire both debts, build 20,000 in reserves and sustain a credible budget and 65% approval for a year.",
        };
        government.journal(0, "Your first decisions", first_steps);
        if id == 2 {
            government.schedule_disaster(Hazard::Flood, 180)?;
            government.schedule_disaster(Hazard::Flood, 540)?;
        }
        if id == 5 {
            let damaged: Vec<_> = fresh
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    matches!(t.kind, Kind::Commercial | Kind::Industrial) && t.jobs > 0
                })
                .take(8)
                .map(|(i, t)| (i, *t))
                .collect();
            for (i, previous) in damaged {
                government.damage.push(DamageRecord {
                    tile: i as u32,
                    previous,
                    repair_cost: (previous.kind.cost() / 2).max(250),
                    day: 0,
                });
                fresh.tiles[i] = Tile::new(previous.terrain);
                fresh.ext.cells[i].rubble = true;
                fresh.ext.cells[i].condition = 0;
                fresh.ext.cells[i].construction = 0;
            }
            government.last_disaster_day = Some(0);
            fresh.refresh();
        }
        if id == 7 {
            government.bonds.push(Bond {
                principal: 25000,
                months_left: 60,
                monthly_interest: 125,
            });
        }
        fresh.civic = government.clone();
        fresh.validate()?;
        *self = government;
        *city = fresh;
        Ok(())
    }
    pub fn advance_day(&mut self, city: &mut City) {
        let first_day_checkpoint = self.january.is_none();
        self.day = self.day.saturating_add(1);
        self.active_events.retain(|e| e.until_day > self.day);
        self.celebrations.retain(|e| e.until_day > self.day);
        let warning_days = if self.active(OrdinanceId::DisasterPreparedness) {
            7
        } else {
            3
        };
        let mut announced = Vec::new();
        for w in &mut self.warnings {
            if !w.announced && w.impact_day <= self.day + warning_days {
                w.announced = true;
                announced.push((w.kind, w.impact_day, w.tile));
                note(
                    city,
                    format!(
                        "{} warning: impact expected in {} days.",
                        w.kind.label(),
                        w.impact_day.saturating_sub(self.day)
                    ),
                );
            }
        }
        for (kind, day, _tile) in announced {
            self.popup(city.month,PopupKind::DisasterWarning,kind.label(),&format!("Impact warning: {} days remain. Run a drill, keep repair funds available and check utility connections.",day.saturating_sub(self.day)),None);
        }
        let mut due = vec![];
        self.warnings.retain(|w| {
            if w.impact_day <= self.day {
                due.push(w.kind);
                false
            } else {
                true
            }
        });
        for kind in due {
            self.impact(city, kind);
        }
        let interval = match self.disaster_frequency {
            1 => 720,
            2 => 360,
            3 => 120,
            _ => 0,
        };
        if interval > 0 && self.day.is_multiple_of(interval) && self.warnings.len() < 8 {
            let mut kind = Hazard::ALL[(random(&mut self.rng) % 10) as usize];
            if kind == Hazard::Meltdown && !city.tiles.iter().any(|t| t.kind == Kind::Nuclear) {
                kind = Hazard::Earthquake;
            }
            if kind == Hazard::PlaneCrash
                && !city
                    .tiles
                    .iter()
                    .any(|t| matches!(t.kind, Kind::Airport | Kind::Airfield))
            {
                kind = Hazard::Tornado;
            }
            let _ = self.schedule_disaster(kind, 7);
        }
        if self.day.is_multiple_of(30)
            && self
                .last_disaster_day
                .map(|d| self.day.saturating_sub(d) >= 30)
                .unwrap_or(false)
            && self.active_events.is_empty()
        {
            self.note_event(CivicEvent::SurvivedDisaster, 1);
            self.last_disaster_day = None;
        }
        if first_day_checkpoint {
            self.capture_january(city);
        }
    }
    fn impact(&mut self, city: &mut City, kind: Hazard) {
        self.impact_at(city, kind, None);
    }
    fn impact_at(&mut self, city: &mut City, kind: Hazard, target: Option<usize>) {
        let candidates: Vec<usize> = city
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| match kind {
                Hazard::Meltdown => t.kind == Kind::Nuclear,
                Hazard::PlaneCrash => matches!(t.kind, Kind::Airport | Kind::Airfield),
                Hazard::Flood => t.terrain != Terrain::Water && t.kind != Kind::Empty,
                _ => t.kind != Kind::Empty,
            })
            .map(|(i, _)| i)
            .collect();
        if candidates.is_empty() {
            note(
                city,
                format!("{} passed without a local impact.", kind.label()),
            );
            return;
        }
        let center = target
            .filter(|i| candidates.contains(i))
            .unwrap_or_else(|| {
                candidates[(random(&mut self.rng) % candidates.len() as u64) as usize]
            });
        let w = city.width as usize;
        let (cx, cy) = (center % w, center / w);
        let reduction = self.modifiers().disaster_reduction.min(75) as usize;
        self.active_events.push(VisualEvent {
            kind,
            tile: center as u32,
            started_day: self.day,
            until_day: self.day + if kind == Hazard::Riot { 12 } else { 5 },
            intensity: (100 - reduction) as u8,
        });
        if kind == Hazard::FriendlyUfo {
            city.money += 5000;
            city.health = city.health.saturating_add(8).min(100);
            city.education = city.education.saturating_add(8).min(100);
            for t in &mut city.tiles {
                t.pollution = t.pollution.saturating_sub(10);
            }
            self.journal(
                city.month,
                "Visitors from above",
                "Friendly visitors shared medical knowledge and a clean-energy grant.",
            );
            return;
        }
        let radius = match kind {
            Hazard::Meteor => 3,
            Hazard::Meltdown => 5,
            Hazard::Earthquake | Hazard::Flood => 7,
            _ => 4,
        };
        let maximum = (match kind {
            Hazard::Earthquake => 32,
            Hazard::Flood => 28,
            Hazard::Meteor => 20,
            Hazard::Tornado => 25,
            Hazard::HostileUfo => 16,
            Hazard::PlaneCrash => 10,
            Hazard::Monster => 40,
            Hazard::Riot => 20,
            Hazard::Meltdown => 36,
            Hazard::FriendlyUfo => 0,
        }) * (100 - reduction)
            / 100;
        let mut targets: Vec<usize> = city
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                t.kind != Kind::Empty && (i % w).abs_diff(cx) + (i / w).abs_diff(cy) <= radius
            })
            .map(|(i, _)| i)
            .collect();
        targets.sort_by_key(|&i| ((i % w).abs_diff(cx) + (i / w).abs_diff(cy), i));
        let mut damaged = 0;
        for i in targets.into_iter().take(maximum) {
            if self.damage.len() >= 512 {
                break;
            }
            let before = city.tiles[i];
            if self.damage.iter().any(|d| d.tile == i as u32) {
                continue;
            }
            let cost = (before.kind.cost() * 3 / 5).max(15);
            self.damage.push(DamageRecord {
                tile: i as u32,
                previous: before,
                repair_cost: cost,
                day: self.day,
            });
            let t = &mut city.tiles[i];
            t.kind = Kind::Empty;
            t.level = 0;
            t.population = 0;
            t.jobs = 0;
            t.fire = 0;
            t.powered = false;
            t.watered = false;
            t.connected = false;
            if kind == Hazard::Meteor {
                t.terrain = Terrain::Hill;
            }
            if let Some(c) = city.ext.cells.get_mut(i) {
                c.rubble = true;
                c.condition = 0;
                c.construction = 0;
                if kind == Hazard::Flood {
                    c.flood = 100;
                }
                if kind == Hazard::Meltdown {
                    c.radiation = 100;
                }
            }
            damaged += 1;
        }
        if damaged > 0 {
            self.last_disaster_day = Some(self.day);
            self.relief_claimed = false;
            city.happiness = city.happiness.saturating_sub(12);
            self.journal(city.month,kind.label(),&format!("{damaged} lots damaged. Relief is available for thirty days; damaged lots can be repaired individually."));
            note(
                city,
                format!("{}: {damaged} damaged lots need recovery.", kind.label()),
            );
            city.population = city.tiles.iter().map(|t| t.population).sum();
            city.jobs = city.tiles.iter().map(|t| t.jobs).sum();
            city.refresh();
        }
    }
    pub fn month_end(&mut self, city: &mut City) {
        if self.last_month == Some(city.month) {
            return;
        }
        self.last_month = Some(city.month);
        self.day = self.day.max(city.month.saturating_mul(30));
        // A mandate is permanent: growth alone cannot expand the council agenda.
        // Preserve any grandfathered capacity supplied by a released-save migration.
        self.ordinance_slots = self.ordinance_slots.clamp(3, 8);
        // The first four policies remain compatible with saves from the original city.
        for i in 0..4 {
            if city.policies[i] {
                self.ordinances |= 1 << i;
            } else {
                self.ordinances &= !(1 << i);
            }
        }
        let mut charges = 0i64;
        for def in content::ORDINANCES {
            if self.active(def.id) {
                charges += def.monthly_base + def.per_thousand * city.population as i64 / 1000;
            }
        }
        for bond in &mut self.bonds {
            charges += bond.monthly_interest;
            bond.months_left = bond.months_left.saturating_sub(1);
            if bond.months_left == 0 {
                if city.money - charges >= bond.principal {
                    charges += bond.principal;
                    bond.principal = 0;
                } else {
                    bond.months_left = 12;
                    bond.monthly_interest = (bond.monthly_interest * 3 / 2).clamp(1, 10_000);
                    self.approval = self.approval.saturating_sub(8);
                    note(city,"A bond matured without enough reserves; a costly twelve-month extension was imposed.");
                }
            }
        }
        self.bonds.retain(|b| b.principal > 0);
        if city.month < self.bankruptcy_until {
            city.funding = city.funding.map(|f| f.min(70));
        }
        charges += self.heritage.len() as i64 * 25;
        city.money -= charges;
        city.expenses += charges;
        let health = [
            OrdinanceId::Childcare,
            OrdinanceId::PublicHealth,
            OrdinanceId::SchoolMeals,
        ]
        .into_iter()
        .filter(|&o| self.active(o))
        .count() as u8
            * 3;
        city.health = city
            .health
            .saturating_add(health)
            .saturating_add(if self.perks & (1 << Perk::MedicalResearch as u8) != 0 {
                5
            } else {
                0
            })
            .min(100);
        city.education = city
            .education
            .saturating_add(if self.active(OrdinanceId::SchoolMeals) {
                4
            } else {
                0
            })
            .saturating_add(if self.perks & (1 << Perk::CentralLibrary as u8) != 0 {
                5
            } else {
                0
            })
            .min(100);
        let happy = (if self.active(OrdinanceId::RentSupport) {
            5
        } else {
            0
        }) + (if self.active(OrdinanceId::NoiseControl) {
            3
        } else {
            0
        }) + (if self.active(OrdinanceId::Childcare) {
            2
        } else {
            0
        });
        city.happiness = city.happiness.saturating_add(happy).min(100);
        if self.active(OrdinanceId::Recycling) {
            city.garbage = city.garbage.saturating_sub(8);
        }
        if self.perks & (1 << Perk::TransitAuthority as u8) != 0 {
            city.transport_usage = city.transport_usage.saturating_add(10).min(100);
        }
        let visitor_bonus = (if self.perks & (1 << Perk::CivicMuseum as u8) != 0 {
            150
        } else {
            0
        }) + (if self.perks & (1 << Perk::ConventionCentre as u8) != 0 {
            (city.jobs as i64 / 100).min(1000)
        } else {
            0
        });
        city.money += visitor_bonus;
        city.income += visitor_bonus;
        let mods = self.modifiers();
        city.transport_usage = city
            .transport_usage
            .saturating_add(mods.transit_bonus)
            .min(100);
        for t in &mut city.tiles {
            if self.active(OrdinanceId::TruckRoutes) && t.kind.is_road() {
                t.traffic = t.traffic.saturating_sub(8);
            }
            t.pollution = t.pollution.saturating_sub(mods.pollution_reduction);
            t.crime = t.crime.saturating_sub(mods.crime_reduction).saturating_sub(
                if self.perks & (1 << Perk::Courthouse as u8) != 0 {
                    5
                } else {
                    0
                },
            );
            if self.perks & (1 << Perk::TransitAuthority as u8) != 0 {
                t.traffic = t.traffic.saturating_sub(5);
            }
            if self.perks & (1 << Perk::CivicMuseum as u8) != 0 {
                t.land_value = t.land_value.saturating_add(2).min(100);
            }
            if self.perks & (1 << Perk::BotanicalGarden as u8) != 0 {
                t.pollution = t.pollution.saturating_sub(3);
            }
        }
        for h in &self.heritage {
            if let Some(t) = city.tiles.get_mut(h.tile as usize) {
                t.land_value = t.land_value.saturating_add(10).min(100);
            }
        }
        if self.active(OrdinanceId::SmallBusinessGrants) {
            let boost = (city.jobs as i64 / 100).min(1500);
            city.money += boost;
            city.income += boost;
        }
        if self.active(OrdinanceId::TourismPromotion) {
            let visitors =
                (self.heritage.len() as i64 * 45 + facts(city).get(Metric::Parks) * 15).min(2000);
            city.money += visitors;
            city.income += visitors;
        }
        if self.perks & (1 << Perk::TradeCommission as u8) != 0 {
            let trade = (city.jobs as i64 / 80).min(2500);
            city.money += trade;
            city.income += trade;
        }
        for district in &mut self.districts {
            if district.specialization > 0 {
                let cost = district.tiles.len() as i64 / 8 + district.population as i64 / 100;
                city.money -= cost;
                city.expenses += cost;
            }
            for &i in &district.tiles {
                if let Some(t) = city.tiles.get_mut(i as usize) {
                    match district.specialization {
                        1 if t.kind == Kind::Residential => {
                            t.land_value = t.land_value.saturating_add(5).min(100)
                        }
                        2 if t.kind == Kind::Commercial => {
                            let grant = t.jobs as i64 / 20;
                            city.money += grant;
                            city.income += grant;
                        }
                        3 if t.kind == Kind::Industrial => t.traffic = t.traffic.saturating_sub(8),
                        4 => t.pollution = t.pollution.saturating_sub(8),
                        _ => {}
                    }
                }
            }
            (district.population, district.jobs, district.happiness) =
                district_stats(city, &district.tiles);
        }
        self.approval = ((city.happiness as u16 * 3
            + city.health as u16
            + (100 - city.unemployment.min(100)) as u16
            + (100 - city.garbage.min(100)) as u16)
            / 6) as u8;
        self.approval = self
            .approval
            .saturating_add(if self.perks & (1 << Perk::MayorsHouse as u8) != 0 {
                1
            } else {
                0
            })
            .saturating_add(if self.perks & (1 << Perk::BroadcastingHouse as u8) != 0 {
                3
            } else {
                0
            })
            .min(100);
        if city.money < 0 {
            self.approval = self.approval.saturating_sub(15);
        }
        if city.month > 0 && city.month.is_multiple_of(48) {
            let votes =
                (self.approval as i16 + (random(&mut self.rng) % 9) as i16 - 4).clamp(0, 100) as u8;
            let won = votes >= 50;
            bounded_push(
                &mut self.elections,
                ElectionResult {
                    month: city.month,
                    votes,
                    won,
                },
                24,
            );
            self.note_event(
                if won {
                    CivicEvent::WonElection
                } else {
                    CivicEvent::LostElection
                },
                1,
            );
            if won {
                city.money += 5000;
                self.ordinance_slots = self.ordinance_slots.saturating_add(1).min(8);
            } else {
                city.money -= 3000;
                self.approval = self.approval.saturating_sub(5);
            }
            self.popup(city.month,PopupKind::Election,if won{"Election won"}else{"Election lost"},&format!("The mayor received {votes}% support. A {} result has been recorded in the civic notebook.",if won{"renewed-mandate"}else{"reform"}),None);
            self.journal(
                city.month,
                if won { "Election won" } else { "Election lost" },
                &format!(
                    "The mayor received {votes}% support. {}",
                    if won {
                        "The region awarded a renewed-mandate grant."
                    } else {
                        "An independent audit and reform program begins; sandbox play continues."
                    }
                ),
            );
        }
        if city.income >= city.expenses {
            self.note_event(CivicEvent::BalancedMonth, 1);
        }
        if city.population > 0 {
            self.events[CivicEvent::BuiltTown as usize] = 1;
        }
        self.advance_projects(city);
        let mut f = self.current_facts(city);
        self.advance_goals(city, &f);
        f.set(Metric::Treasury, city.money);
        self.advance_scenario(city, &f);
        // Settle every recurring charge and grant before enforcing the credit
        // boundary. An ignored crisis has an explicit, saved default outcome;
        // it never silently truncates cash or compounds beyond save bounds.
        if city.money < -overdraft_limit(city) {
            self.enter_bankruptcy_protection(city, true);
            f = self.current_facts(city);
        }
        f.set(Metric::Treasury, city.money);
        f.set(Metric::Happiness, city.happiness as i64);
        f.set(Metric::Approval, self.approval as i64);
        f.set(
            Metric::SuccessfulScenarios,
            self.events[CivicEvent::WonScenario as usize] as i64,
        );
        self.publish(city, &f);
        bounded_push(
            &mut self.history,
            GraphSample {
                month: city.month,
                values: [
                    city.population as i64,
                    city.money,
                    city.income,
                    city.expenses,
                    city.income - city.expenses,
                    city.happiness as i64,
                    city.health as i64,
                    city.education as i64,
                    city.unemployment as i64,
                    f.get(Metric::Traffic),
                    f.get(Metric::Pollution),
                    f.get(Metric::Crime),
                    f.get(Metric::LandValue),
                    f.get(Metric::PowerCoverage),
                    f.get(Metric::WaterCoverage),
                    self.approval as i64,
                ],
            },
            240,
        );
        let rated = evaluate_with(city, self, &f);
        for medal in rated.medals {
            self.medal_bits |= 1 << medal as u8;
        }
        if city.month.is_multiple_of(12) {
            if city.month > 0 {
                let net: i64 = self
                    .history
                    .iter()
                    .rev()
                    .take(12)
                    .map(|h| h.values[4])
                    .sum();
                let text=format!("Year {} closed with a ${net} operating balance. Treasury ${}; municipal debt ${}; bonds ${}. Evaluation: {} / 1000, grade {}.",city.month/12,city.money,city.debt,self.bonds.iter().map(|b|b.principal).sum::<i64>(),rated.score,rated.grade);
                self.journal(city.month, "Annual Budget Review", &text);
                self.popup(
                    city.month,
                    PopupKind::AnnualReview,
                    "Annual Budget Review",
                    &text,
                    None,
                );
            }
            self.capture_january(city);
        }
        let _ = f;
    }
    fn advance_projects(&mut self, city: &mut City) {
        let mut complete = vec![];
        for run in &mut self.projects {
            if run.complete {
                continue;
            }
            let def = content::MEGAPROJECTS[run.id as usize];
            let left = def.months.saturating_sub(run.months_done).max(1) as i64;
            let payment = (def.cost - run.paid + left - 1) / left;
            if city
                .tiles
                .get(run.tile as usize)
                .map(|t| t.kind != Kind::Empty)
                .unwrap_or(true)
                || city.money < payment
                || city.power_supply < city.power_demand
                || city.water_supply < city.water_demand
            {
                run.paused = true;
                continue;
            }
            run.paused = false;
            city.money -= payment;
            run.paid += payment;
            run.months_done += 1;
            if run.months_done >= def.months {
                run.complete = true;
                complete.push((run.id, run.tile));
            }
        }
        for (id, tile) in complete {
            self.completed_projects |= 1 << id;
            self.note_event(CivicEvent::CompletedProject, 1);
            if let Some(t) = city.tiles.get_mut(tile as usize) {
                city.jobs = city.jobs.saturating_sub(t.jobs).saturating_add(250);
                city.population = city.population.saturating_sub(t.population);
                t.kind = match id {
                    0 => Kind::Arcology,
                    1 => Kind::Spaceport,
                    2 => Kind::WorldExpo,
                    _ => Kind::ScienceCenter,
                };
                t.level = 1;
                t.population = 0;
                t.jobs = 250;
                t.fire = 0;
            }
            if let Some(c) = city.ext.cells.get_mut(tile as usize) {
                c.construction = 100;
                c.condition = 100;
                c.rubble = false;
            }
            self.popup(
                city.month,
                PopupKind::Megaproject,
                content::MEGAPROJECTS[id as usize].title,
                "Construction is complete. Permanent civic benefits are now active.",
                Some(tile),
            );
            bounded_push(
                &mut self.celebrations,
                Celebration {
                    project: id,
                    milestone: None,
                    tile,
                    started_day: self.day,
                    until_day: self.day + 10,
                },
                4,
            );
            self.journal(
                city.month,
                content::MEGAPROJECTS[id as usize].title,
                "Construction complete. The project's permanent civic benefits are active.",
            );
        }
    }
    fn advance_goals(&mut self, city: &mut City, f: &Facts) {
        fn streak(v: &mut Vec<u16>, i: usize, yes: bool, months: u16) -> bool {
            if v.len() <= i {
                v.resize(i + 1, 0);
            }
            v[i] = if yes { v[i].saturating_add(1) } else { 0 };
            v[i] >= months.max(1)
        }
        let mut grants = vec![];
        let mut celebration_site = None;
        for (i, def) in content::MILESTONES.iter().enumerate() {
            if self.milestone_bits & (1 << i) == 0
                && streak(
                    &mut self.milestone_streaks,
                    i,
                    f.meets(def.goal),
                    def.goal.months,
                )
            {
                self.milestone_bits |= 1 << i;
                grants.push((def.title, def.reward));
                let site = *celebration_site.get_or_insert_with(|| Self::milestone_site(city));
                self.celebrate_milestone(site, i as u8);
            }
        }
        for (i, def) in content::REQUESTS.iter().enumerate() {
            if self.request_bits & (1 << i) == 0
                && streak(
                    &mut self.request_streaks,
                    i,
                    f.meets(def.goal),
                    def.goal.months,
                )
            {
                self.request_bits |= 1 << i;
                grants.push((def.title, def.reward));
            }
        }
        for (i, def) in content::ACHIEVEMENTS.iter().enumerate() {
            if self.achievement_bits & (1 << i) == 0
                && streak(
                    &mut self.achievement_streaks,
                    i,
                    f.meets(def.goal),
                    def.goal.months,
                )
            {
                self.achievement_bits |= 1 << i;
                self.journal(city.month, def.title, def.description);
                self.popup(
                    city.month,
                    PopupKind::Achievement,
                    def.title,
                    def.description,
                    None,
                );
                note(city, format!("Achievement: {}.", def.title));
            }
        }
        for (title, reward) in grants {
            self.popup(city.month,if reward.perk.is_some(){PopupKind::Request}else{PopupKind::Milestone},title,&format!("Goal complete. The ${} grant and any permanent civic benefits have been awarded once. {}",reward.cash,if reward.unlock.is_some(){"A new infrastructure tier is unlocked."}else{"See the civic notebook for the record."}),None);
            city.money += reward.cash;
            if let Some(p) = reward.perk {
                if p == Perk::CityHall && self.perks & (1 << p as u8) == 0 {
                    self.ordinance_slots = self.ordinance_slots.saturating_add(1).min(8);
                }
                self.perks |= 1 << p as u8;
            }
            if let Some(u) = reward.unlock {
                self.unlocks |= 1 << u as u8;
            }
            self.journal(city.month,title,&format!("Civic goal completed. Regional grant: ${}. Permanent benefits and unlocks are recorded for this city.",reward.cash));
            note(city, format!("{}: ${} civic grant.", title, reward.cash));
        }
    }
    fn milestone_site(city: &City) -> Option<u32> {
        let width = usize::from(city.width);
        let hall = city
            .tiles
            .iter()
            .enumerate()
            .find(|(i, t)| {
                t.kind == Kind::CityHall && crate::transport::facility_operational(city, *i)
            })
            .map(|(i, _)| i);
        let tile = hall.or_else(|| {
            city.tiles
                .iter()
                .enumerate()
                .filter(|(i, t)| {
                    t.kind.is_building()
                        && t.fire == 0
                        && t.population + t.jobs > 0
                        && city
                            .ext
                            .cells
                            .get(*i)
                            .is_none_or(|c| c.construction == 100 && !c.abandoned && !c.rubble)
                })
                .min_by_key(|(i, _)| {
                    (i % width).abs_diff(width / 2)
                        + (i / width).abs_diff(usize::from(city.height) / 2)
                })
                .map(|(i, _)| i)
        });
        tile.map(|i| i as u32)
    }
    fn celebrate_milestone(&mut self, site: Option<u32>, milestone: u8) {
        if let Some(tile) = site {
            bounded_push(
                &mut self.celebrations,
                Celebration {
                    project: 0,
                    milestone: Some(milestone),
                    tile,
                    started_day: self.day,
                    until_day: self.day + 10,
                },
                4,
            );
        }
    }
    fn advance_scenario(&mut self, city: &mut City, f: &Facts) {
        let mut result = None;
        if let Some(run) = &mut self.scenario {
            if run.result != ScenarioResult::Running {
                return;
            }
            let def = content::SCENARIOS[run.id as usize];
            run.streak = if f.meets(def.goal) {
                run.streak.saturating_add(1)
            } else {
                0
            };
            if run.streak >= def.goal.months {
                run.result = ScenarioResult::Won;
                result = Some((true, run.id));
            } else if city.month >= run.deadline_month {
                run.result = ScenarioResult::Lost;
                result = Some((false, run.id));
            }
        }
        if let Some((won, id)) = result {
            let def = content::SCENARIOS[id as usize];
            if won {
                city.money += def.prize;
                self.note_event(CivicEvent::WonScenario, 1);
            }
            self.popup(
                city.month,
                if won {
                    PopupKind::ScenarioWon
                } else {
                    PopupKind::ScenarioLost
                },
                def.title,
                if won {
                    "Scenario goals were sustained. The regional prize has been awarded."
                } else {
                    "The scenario deadline passed. Sandbox play can continue."
                },
                None,
            );
            self.journal(city.month,def.title,if won{"Scenario complete: its goals remained true for the required period. The regional prize was awarded."}else{"The deadline passed before the objectives were sustained. Sandbox play remains available."});
        }
    }
    fn publish(&mut self, city: &City, f: &Facts) {
        let residents: Vec<_> = city
            .ext
            .people
            .households
            .iter()
            .filter(|h| h.size > 0)
            .collect();
        let resident = if residents.is_empty() {
            None
        } else {
            Some(residents[(random(&mut self.rng) % residents.len() as u64) as usize])
        };
        let person = resident.map(|h| CitizenSnapshot {
            id: h.id,
            home: h.home,
            size: h.size,
            education: h.education,
            health: h.health,
            income_class: h.income_class,
            employer: h.employer,
        });
        let citizen = resident.map(|h| h.id);
        let tile = resident.map(|h| h.home).or_else(|| {
            city.tiles
                .iter()
                .position(|t| t.population > 0)
                .map(|i| i as u32)
        });
        let choose = |defs: &[content::MessageDef],
                      recent: &[MessageRecord],
                      seed: &mut u64|
         -> Option<u16> {
            let mut eligible: Vec<usize> = defs
                .iter()
                .enumerate()
                .filter(|(i, d)| {
                    f.meets(d.goal) && !recent.iter().rev().take(12).any(|r| r.id == *i as u16)
                })
                .map(|(i, _)| i)
                .collect();
            if eligible.is_empty() {
                eligible = defs
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| f.meets(d.goal))
                    .map(|(i, _)| i)
                    .collect();
            }
            if eligible.is_empty() {
                None
            } else {
                Some(eligible[(random(seed) % eligible.len() as u64) as usize] as u16)
            }
        };
        if let Some(id) = choose(&content::CHIRPER, &self.chirper, &mut self.rng) {
            bounded_push(
                &mut self.chirper,
                MessageRecord {
                    id,
                    month: city.month,
                    citizen,
                    tile,
                    person: person.clone(),
                },
                24,
            );
        }
        if let Some(id) = choose(&content::HEADLINES, &self.headlines, &mut self.rng) {
            bounded_push(
                &mut self.headlines,
                MessageRecord {
                    id,
                    month: city.month,
                    citizen: None,
                    tile: None,
                    person: None,
                },
                16,
            );
        }
        let mut trigger: Vec<_> = content::ADVISOR_TRIGGERS
            .iter()
            .filter(|d| f.meets(d.goal))
            .collect();
        trigger.sort_by_key(|t| std::cmp::Reverse(t.priority));
        if let Some(t) = trigger
            .iter()
            .find(|t| {
                !self
                    .advisors
                    .iter()
                    .rev()
                    .take(4)
                    .any(|r| r.id == t.line as u16)
            })
            .or_else(|| trigger.first())
        {
            bounded_push(
                &mut self.advisors,
                MessageRecord {
                    id: t.line as u16,
                    month: city.month,
                    citizen: None,
                    tile: None,
                    person: None,
                },
                12,
            );
        }
    }
    pub fn next_hint(&mut self, city: &City) -> Option<&'static str> {
        let f = self.current_facts(city);
        let i = content::HINTS
            .iter()
            .enumerate()
            .find(|(i, h)| self.hint_seen & (1 << i) == 0 && f.meets(h.goal))
            .map(|(i, _)| i)?;
        self.hint_seen |= 1 << i;
        Some(content::HINTS[i].text)
    }
    fn capture_january(&mut self, city: &City) {
        // The previous checkpoint owns a complete world. Detach it before
        // cloning government so an annual capture does not copy that world
        // only to discard it immediately. Preserve it if encoding fails.
        let previous = self.january.take();
        let mut clean = self.clone();
        clean.rewind_used = false;
        let government = match serde_json::to_vec(&clean) {
            Ok(v) => pack(&v),
            Err(_) => {
                self.january = previous;
                return;
            }
        };
        let world = city.ext.clone();
        let values = [
            city.money,
            city.month as i64,
            city.population as i64,
            city.jobs as i64,
            city.happiness as i64,
            city.income,
            city.expenses,
            city.power_supply as i64,
            city.power_demand as i64,
            city.water_supply as i64,
            city.water_demand as i64,
            city.debt,
            city.health as i64,
            city.education as i64,
            city.garbage as i64,
            city.unemployment as i64,
            city.transport_usage as i64,
            city.demand[0] as i64,
            city.demand[1] as i64,
            city.demand[2] as i64,
            0,
            0,
            0,
            0,
        ];
        self.january = Some(Box::new(JanuarySnapshot {
            year: city.month / 12,
            width: city.width,
            height: city.height,
            tiles: city.tiles.clone(),
            values,
            taxes: city.taxes,
            funding: city.funding,
            policies: city.policies,
            history: city.history.clone(),
            messages: city.messages.clone(),
            seed: city.seed,
            world,
            government,
        }));
        self.rewind_used = false;
    }
    pub fn save_january(&mut self, city: &City) {
        self.capture_january(city);
    }
    pub fn rewind_january(&mut self, city: &mut City) -> Result<(), String> {
        if self.rewind_used {
            return Err("January rewind has already been used this year.".into());
        }
        let snap = self
            .january
            .as_ref()
            .ok_or("No January checkpoint is available yet.")?
            .as_ref()
            .clone();
        if snap.year != city.month / 12 {
            return Err("The checkpoint belongs to another calendar year.".into());
        }
        let mut candidate = snap.candidate()?;
        candidate.civic.january = Some(Box::new(snap));
        candidate.civic.rewind_used = true;
        candidate.validate()?;
        *self = candidate.civic.clone();
        *city = candidate;
        Ok(())
    }
    pub fn validate(&self, tiles: usize) -> Result<(), String> {
        if self.popups.len() > 32
            || self.popups.iter().any(|p| {
                p.id == 0
                    || p.id >= self.next_popup
                    || p.title.len() > 100
                    || p.text.len() > 600
                    || p.tile.map(|t| t as usize >= tiles).unwrap_or(false)
            })
            || self.popups.windows(2).any(|p| p[0].id >= p[1].id)
            || self.next_popup == u64::MAX
        {
            return Err("Invalid civic popup queue.".into());
        }
        if self.ordinances >> 17 != 0
            || self.milestone_bits >> 25 != 0
            || self.request_bits >> 12 != 0
            || self.perks >> 12 != 0
            || self.unlocks >> 7 != 0
            || self.completed_projects >> 4 != 0
            || self.hint_seen >> 20 != 0
            || self.ordinance_slots > 8
            || self.ordinances.count_ones() > self.ordinance_slots.max(3) as u32
            || self.approval > 100
            || self.day > u32::MAX - 3600
            || self.disaster_frequency > 3
            || self.bonds.len() > 3
            || self.bonds.iter().any(|b| {
                b.principal <= 0
                    || b.principal > 200000
                    || b.months_left == 0
                    || b.months_left > 120
                    || b.monthly_interest <= 0
                    || b.monthly_interest > 10000
            })
        {
            return Err("Invalid civic finance or progression values.".into());
        }
        if self.districts.len() > 16
            || self.heritage.len() > 64
            || self.projects.len() > 4
            || self.warnings.len() > 8
            || self.damage.len() > 512
            || self.history.len() > 240
            || self.notebook.len() > 120
            || self.chirper.len() > 24
            || self.advisors.len() > 12
            || self.headlines.len() > 16
            || self.active_events.len() > 16
            || self.celebrations.len() > 4
            || self.elections.len() > 24
            || self.milestone_streaks.len() > 25
            || self.request_streaks.len() > 12
            || self.achievement_streaks.len() > 8
        {
            return Err("Civic collections exceed their saved limits.".into());
        }
        if self.districts.iter().any(|d| {
            d.tiles.len() > 4096
                || d.tiles.len() < 4
                || d.name.is_empty()
                || d.name.chars().count() > 24
                || d.name.chars().any(char::is_control)
                || d.id == 0
                || d.id >= self.next_district
                || d.specialization > 4
                || d.happiness > 100
                || d.tiles.iter().any(|&i| i as usize >= tiles)
                || d.tiles.windows(2).any(|a| a[0] >= a[1])
        }) || self
            .heritage
            .iter()
            .any(|h| h.tile as usize >= tiles || h.name.len() > 100)
            || self.projects.iter().any(|p| {
                p.id > 3
                    || p.tile as usize >= tiles
                    || p.months_done > content::MEGAPROJECTS[p.id.min(3) as usize].months
                    || p.paid < 0
                    || p.paid > content::MEGAPROJECTS[p.id.min(3) as usize].cost
                    || (p.complete
                        && (p.months_done != content::MEGAPROJECTS[p.id.min(3) as usize].months
                            || p.paid != content::MEGAPROJECTS[p.id.min(3) as usize].cost
                            || self.completed_projects & (1 << p.id.min(3)) == 0))
            })
            || self.damage.iter().any(|d| {
                d.tile as usize >= tiles
                    || !(0..=1_000_000).contains(&d.repair_cost)
                    || d.previous.level > 5
                    || [
                        d.previous.pollution,
                        d.previous.traffic,
                        d.previous.crime,
                        d.previous.land_value,
                        d.previous.fire,
                    ]
                    .iter()
                    .any(|&v| v > 100)
            })
            || self
                .notebook
                .iter()
                .any(|n| n.title.len() > 100 || n.text.len() > 500)
        {
            return Err("Invalid civic map references or project progress.".into());
        }
        if self.chirper.iter().any(|r| {
            if r.tile.is_some_and(|t| t as usize >= tiles) || r.citizen == Some(0) {
                return true;
            }
            r.person
                .as_ref()
                .map(|p| {
                    p.home as usize >= tiles
                        || Some(p.id) != r.citizen
                        || Some(p.home) != r.tile
                        || p.health > 100
                        || p.education > 100
                        || p.income_class > 2
                        || p.employer.is_some_and(|t| t as usize >= tiles)
                        || p.size == 0
                })
                .unwrap_or(false)
        }) {
            return Err("Invalid archived citizen profile.".into());
        }
        if self.chirper.iter().any(|r| r.id >= 150)
            || self.advisors.iter().any(|r| r.id >= 76)
            || self.headlines.iter().any(|r| r.id >= 61)
            || self
                .scenario
                .as_ref()
                .map(|s| {
                    s.id >= 8
                        || s.started_month.checked_add(u32::from(
                            content::SCENARIOS[s.id.min(7) as usize].deadline_months,
                        )) != Some(s.deadline_month)
                        || s.streak > content::SCENARIOS[s.id.min(7) as usize].goal.months
                })
                .unwrap_or(false)
        {
            return Err("Invalid saved content reference.".into());
        }
        if self.warnings.iter().any(|w| w.tile as usize >= tiles)
            || self.active_events.iter().any(|e| {
                e.tile as usize >= tiles || e.intensity > 100 || e.until_day < e.started_day
            })
            || self.celebrations.iter().any(|e| {
                e.project > 3
                    || e.milestone.is_some_and(|m| m >= 25 || e.project != 0)
                    || e.tile as usize >= tiles
                    || e.started_day > self.day
                    || e.started_day.checked_add(10) != Some(e.until_day)
            })
            || self
                .elections
                .iter()
                .any(|e| e.votes > 100 || e.won != (e.votes >= 50))
            || self.history.iter().any(|h| {
                h.values[0] < 0
                    || h.values[0] > tiles as i64 * 1000
                    || h.values[1..5]
                        .iter()
                        .any(|&n| !(-2_000_200_000..=2_000_200_000).contains(&n))
                    || h.values[5..].iter().any(|&n| !(0..=100).contains(&n))
            })
        {
            return Err("Invalid civic event, election or graph statistics.".into());
        }
        for i in 0..self.districts.len() {
            if self.districts[i + 1..].iter().any(|d| {
                d.id == self.districts[i].id
                    || d.tiles
                        .iter()
                        .any(|tile| self.districts[i].tiles.binary_search(tile).is_ok())
            }) {
                return Err("Duplicate district ownership.".into());
            }
        }
        if self
            .heritage
            .iter()
            .enumerate()
            .any(|(i, h)| self.heritage[i + 1..].iter().any(|x| x.tile == h.tile))
            || self.projects.iter().enumerate().any(|(i, p)| {
                self.projects[i + 1..]
                    .iter()
                    .any(|x| x.id == p.id || x.tile == p.tile)
            })
        {
            return Err("Duplicate civic unique asset.".into());
        }
        if let Some(s) = &self.january {
            if s.width as usize * s.height as usize != s.tiles.len()
                || s.tiles.len() != tiles
                || s.government.len() > 8 * 1024 * 1024
            {
                return Err("Invalid January checkpoint size.".into());
            }
            s.candidate()?;
        }
        Ok(())
    }
}

// A small bounded LZ stream keeps January's repeated JSON keys compact without
// introducing a platform compressor or serializing City recursively.
#[derive(Default)]
struct CodecKeyHasher(u64);
impl std::hash::Hasher for CodecKeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write_u32(&mut self, value: u32) {
        // All codec keys are full u32 triples. Mixing them gives hashbrown good
        // bucket/fingerprint dispersion without SipHash's per-key overhead.
        let mut x = u64::from(value) ^ 0x9e3779b97f4a7c15;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        self.0 = x ^ (x >> 31);
    }
    fn write(&mut self, bytes: &[u8]) {
        let mut x = 0xcbf29ce484222325u64;
        for byte in bytes {
            x = (x ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
        self.0 = x;
    }
}
fn pack(input: &[u8]) -> Vec<u8> {
    use std::collections::HashMap;
    use std::hash::BuildHasherDefault;
    let mut out = Vec::with_capacity(input.len() / 4);
    // Only get/insert are used: hash bucket order never enters the wire codec.
    let mut last =
        HashMap::<u32, usize, BuildHasherDefault<CodecKeyHasher>>::with_capacity_and_hasher(
            input.len().min(8192),
            BuildHasherDefault::default(),
        );
    let (mut i, mut literal) = (0, Vec::with_capacity(128));
    let flush = |out: &mut Vec<u8>, v: &mut Vec<u8>| {
        for c in v.chunks(128) {
            out.push((c.len() - 1) as u8);
            out.extend_from_slice(c);
        }
        v.clear();
    };
    while i < input.len() {
        let key = if i + 2 < input.len() {
            Some(input[i] as u32 | ((input[i + 1] as u32) << 8) | ((input[i + 2] as u32) << 16))
        } else {
            None
        };
        let mut found = None;
        if let Some(k) = key {
            if let Some(&p) = last.get(&k) {
                let dist = i - p;
                if dist <= 65535 {
                    let mut len = 0;
                    while len < 130 && i + len < input.len() && input[p + len] == input[i + len] {
                        len += 1;
                    }
                    if len >= 4 {
                        found = Some((dist, len));
                    }
                }
            }
            last.insert(k, i);
        }
        if let Some((dist, len)) = found {
            flush(&mut out, &mut literal);
            out.push(0x80 | ((len - 3) as u8));
            out.extend_from_slice(&(dist as u16).to_le_bytes());
            i += len;
        } else {
            literal.push(input[i]);
            i += 1;
            if literal.len() == 128 {
                flush(&mut out, &mut literal);
            }
        }
    }
    flush(&mut out, &mut literal);
    out
}
fn unpack(input: &[u8]) -> Result<Vec<u8>, String> {
    let (mut out, mut i) = (Vec::new(), 0);
    while i < input.len() {
        let tag = input[i];
        i += 1;
        if tag & 0x80 == 0 {
            let len = tag as usize + 1;
            if i + len > input.len() {
                return Err("Truncated January literal.".into());
            }
            out.extend_from_slice(&input[i..i + len]);
            i += len;
        } else {
            if i + 2 > input.len() {
                return Err("Truncated January match.".into());
            }
            let dist = u16::from_le_bytes([input[i], input[i + 1]]) as usize;
            i += 2;
            let len = (tag & 0x7f) as usize + 3;
            if dist == 0 || dist > out.len() {
                return Err("Invalid January back-reference.".into());
            }
            for _ in 0..len {
                let b = out[out.len() - dist];
                out.push(b);
            }
        }
        if out.len() > 64 * 1024 * 1024 {
            return Err("January snapshot exceeds its decoded limit.".into());
        }
    }
    Ok(out)
}

mod packed_world {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        world: &crate::world::WorldState,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let bytes = serde_json::to_vec(world).map_err(serde::ser::Error::custom)?;
        pack(&bytes).serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<crate::world::WorldState, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(serde::de::Error::custom(
                "January world archive is too large.",
            ));
        }
        let decoded = unpack(&bytes).map_err(serde::de::Error::custom)?;
        serde_json::from_slice(&decoded).map_err(serde::de::Error::custom)
    }
}

/// A score is a current assessment. Medals already earned remain in the record.
pub fn evaluation(city: &City) -> Evaluation {
    evaluate(city, &city.civic)
}
fn evaluate(city: &City, g: &CivicState) -> Evaluation {
    let f = g.current_facts(city);
    evaluate_with(city, g, &f)
}
fn evaluate_with(city: &City, g: &CivicState, f: &Facts) -> Evaluation {
    let pop = city.population;
    let finance = (if city.income >= city.expenses { 80 } else { 0 })
        + (if city.money >= city.expenses.max(1) * 6 {
            60
        } else if city.money >= 0 {
            30
        } else {
            0
        })
        + (if city.debt + g.bonds.iter().map(|b| b.principal).sum::<i64>() == 0 {
            60
        } else {
            0
        });
    let score = if pop == 0 {
        0
    } else {
        ((pop.min(100000) as u64 * 150 / 100000) as u32
            + city.happiness.min(100) as u32 * 3 / 2
            + city.health.min(100) as u32
            + city.education.min(100) as u32
            + ((f.get(Metric::PowerCoverage) + f.get(Metric::WaterCoverage)).clamp(0, 200) / 2)
                as u32
            + (100 - city.unemployment.min(100)) as u32
            + ((200 - f.get(Metric::Pollution) - f.get(Metric::Crime)).clamp(0, 200) / 2) as u32
            + finance)
            .min(1000)
    };
    let balanced_year =
        g.history.len() >= 12 && g.history.iter().rev().take(12).all(|h| h.values[4] >= 0);
    let earned = [
        pop >= 500,
        pop >= 5000 && city.health >= 80 && city.happiness >= 75,
        pop >= 10000 && city.education >= 80 && city.unemployment <= 5,
        pop >= 8000
            && f.get(Metric::Pollution) <= 20
            && f.get(Metric::RenewableShare) >= 70
            && f.get(Metric::PowerCoverage) >= 95,
        pop >= 10000 && city.transport_usage >= 50 && f.get(Metric::Traffic) <= 40,
        pop >= 1000 && balanced_year && city.debt == 0 && g.bonds.is_empty(),
        g.events[CivicEvent::SurvivedDisaster as usize] >= 3
            && g.events[CivicEvent::RepairedDamage as usize] >= 20,
        g.completed_projects.count_ones() >= 2 && pop >= 50000,
    ];
    let medals = Medal::ALL
        .into_iter()
        .filter(|m| earned[*m as usize] || g.medal_bits & (1 << *m as u8) != 0)
        .collect();
    Evaluation {
        score,
        grade: match score {
            850..=1000 => "A",
            700..=849 => "B",
            550..=699 => "C",
            400..=549 => "D",
            _ => "F",
        },
        medals,
    }
}
impl CivicState {
    fn popup(&mut self, month: u32, kind: PopupKind, title: &str, text: &str, tile: Option<u32>) {
        let id = self.next_popup.max(1);
        self.next_popup = id.saturating_add(1);
        bounded_push(
            &mut self.popups,
            Popup {
                id,
                month,
                kind,
                title: title.into(),
                text: text.into(),
                tile,
            },
            32,
        );
    }
    pub fn dismiss_popup(&mut self, id: u64) -> Result<(), String> {
        let index = self
            .popups
            .iter()
            .position(|p| p.id == id)
            .ok_or("This notification has already been dismissed.")?;
        self.popups.remove(index);
        Ok(())
    }
    pub fn add_note(&mut self, month: u32, title: String, text: String) -> Result<(), String> {
        if title.trim().is_empty()
            || title.chars().count() > 60
            || text.trim().is_empty()
            || text.chars().count() > 400
            || title.chars().any(|c| c.is_control())
        {
            return Err(
                "Notebook entries need a title of 1..60 and text of 1..400 characters.".into(),
            );
        }
        self.journal(month, title.trim(), text.trim());
        Ok(())
    }
}

pub fn ordinance_monthly_cost(city: &City, id: OrdinanceId) -> i64 {
    let d = content::ORDINANCES[id as usize];
    let legacy = match id {
        OrdinanceId::CleanAir => 30 + city.population as i64 / 100,
        OrdinanceId::FreeTransit => 20 + city.population as i64 / 70,
        OrdinanceId::Recycling => 25 + city.population as i64 / 100,
        OrdinanceId::SmallBusinessGrants => 80 + city.jobs as i64 / 80,
        _ => 0,
    };
    d.monthly_base + d.per_thousand * city.population as i64 / 1000 + legacy
}
pub const DISTRICT_SPECIALIZATIONS: [&str; 5] = [
    "Balanced",
    "Residential",
    "Commercial",
    "Industrial",
    "Green",
];
pub const DISTRICT_NAMES: [&str; 12] = [
    "Old Town",
    "Riverside",
    "University Quarter",
    "Market Ward",
    "Harbor View",
    "Garden District",
    "Industrial Heights",
    "Civic Center",
    "Maple Grove",
    "Eastbank",
    "Westhaven",
    "Founders' Square",
];
impl CivicState {
    pub fn rename_district(&mut self, id: u16, name: String) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 24 || name.chars().any(|c| c.is_control()) {
            return Err("District names use 1..24 printable characters.".into());
        }
        self.districts
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or("Unknown district.")?
            .name = name.into();
        Ok(())
    }
    pub fn set_district_specialization(
        &mut self,
        city: &mut City,
        id: u16,
        value: u8,
    ) -> Result<(), String> {
        if value > 4 {
            return Err("Unknown district specialization.".into());
        }
        let i = self
            .districts
            .iter()
            .position(|d| d.id == id)
            .ok_or("Unknown district.")?;
        if self.districts[i].specialization == value {
            return Ok(());
        }
        spend(city, 500)?;
        self.districts[i].specialization = value;
        Ok(())
    }
    pub fn remove_district(&mut self, id: u16) -> Result<(), String> {
        let i = self
            .districts
            .iter()
            .position(|d| d.id == id)
            .ok_or("Unknown district.")?;
        self.districts.remove(i);
        Ok(())
    }
}
impl Medal {
    pub fn description(self) -> &'static str {
        match self{Self::FirstCommunity=>"Welcome at least 500 residents.",Self::CityOfCare=>"Reach 5,000 residents, health 80 and happiness 75.",Self::LearningCapital=>"Reach 10,000 residents, education 80 and unemployment at most 5%.",Self::GreenCity=>"Reach 8,000 residents, clean power 70%, pollution at most 20 and power coverage 95%.",Self::ConnectedCity=>"Reach 10,000 residents, transit use 50 and road traffic at most 40.",Self::SoundStewardship=>"Reach 1,000 residents, twelve balanced months and no loan or bond debt.",Self::ResilientCity=>"Recover from three disasters and repair twenty damaged lots.",Self::MasterBuilder=>"Complete two megaprojects and serve 50,000 residents."}
    }
}

impl CivicState {
    pub fn trigger_disaster_at(
        &mut self,
        city: &mut City,
        kind: Hazard,
        tile: u32,
    ) -> Result<(), String> {
        if tile as usize >= city.tiles.len() || city.tiles[tile as usize].kind == Kind::Empty {
            return Err("A live disaster target must be an occupied map lot.".into());
        }
        if self.active_events.len() >= 16
            || self
                .active_events
                .iter()
                .any(|e| e.kind == kind && e.tile == tile && e.until_day > self.day)
        {
            return Err("This local event is already active or the event limit is reached.".into());
        }
        self.impact_at(city, kind, Some(tile as usize));
        Ok(())
    }
}

#[cfg(test)]
mod codec_compatibility {
    #[test]
    fn indexed_compressor_preserves_actual_saved_january_bytes() {
        let raw = include_bytes!("../tests/fixtures/january-government-reference.json");
        let saved = include_bytes!("../tests/fixtures/january-government-reference.lz");
        assert_eq!(super::unpack(saved).unwrap().as_slice(), raw.as_slice());
        assert_eq!(super::pack(raw).as_slice(), saved.as_slice());
    }
}
