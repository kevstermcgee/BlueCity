//! A reproducible bot mayor and an explicitly constructed load-test fixture.
use crate::{
    city::{City, Kind, MapStyle, Terrain, Tile},
    world::Climate,
};
use serde::Serialize;
#[derive(Clone, Debug, Default, Serialize)]
pub struct MayorReport {
    pub days: u32,
    pub edits: u32,
    pub refusals: u32,
    pub minimum_funds: i64,
    pub maximum_population: u32,
    pub bankrupt_months: u32,
    pub budget_changes: u32,
    pub financial_rescues: u32,
    #[serde(skip)]
    pub day_samples_ns: Vec<u64>,
}
fn has_pending(city: &City, matches: impl Fn(Kind) -> bool) -> bool {
    city.tiles.iter().enumerate().any(|(i, t)| {
        matches(t.kind)
            && city
                .ext
                .cells
                .get(i)
                .is_some_and(|c| c.construction < 100 && !c.rubble && !c.abandoned)
    })
}
/// The bot pays the same placement costs and follows the same refusal rules as players.
fn build_best(city: &mut City, kind: Kind, report: &mut MayorReport) -> bool {
    if city.money < kind.cost() + 2_500 {
        return false;
    }
    let Some([x, y]) = city.suggest_site(kind) else {
        return false;
    };
    match city.place(x, y, kind) {
        Ok(_) => {
            report.edits += 1;
            true
        }
        Err(_) => {
            report.refusals += 1;
            false
        }
    }
}
fn renew_empty_sites(city: &mut City, district: i32, report: &mut MayorReport) {
    if city.money < 2_540 {
        return;
    }
    let registered: std::collections::BTreeSet<_> =
        city.civic.damage.iter().map(|d| d.tile).collect();
    let center = i32::from(city.width) / 2;
    let mut sites: Vec<_> = city
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            (t.population == 0
                && t.jobs == 0
                && !registered.contains(&(i as u32))
                && city.ext.cells.get(i).is_some_and(|c| {
                    (c.rubble || c.abandoned) && (t.kind != Kind::Empty || c.rubble)
                })
                && city.civic.can_bulldoze(i as u32).is_ok())
            .then_some(i)
        })
        .collect();
    sites.sort_by_key(|&i| {
        let x = (i % city.width as usize) as i32;
        let y = (i / city.width as usize) as i32;
        (u8::from(x != center && y != district), i)
    });
    for i in sites.into_iter().take(3) {
        if city.money < 2_540 {
            break;
        }
        let (x, y) = (
            (i % city.width as usize) as i32,
            (i / city.width as usize) as i32,
        );
        if city.bulldoze(x, y).is_ok() {
            report.edits += 1;
        } else {
            report.refusals += 1;
        }
    }
}
pub fn govern(city: &mut City, days: u32) -> MayorReport {
    let mut report = MayorReport {
        minimum_funds: city.money,
        ..Default::default()
    };
    let mut last_bankrupt_month = None;
    for d in 0..days {
        let defaults_before =
            city.civic.events[crate::content::CivicEvent::DeclaredBankruptcy as usize];
        if d.is_multiple_of(30) {
            // The same bounded tax/funding controls exposed by the Budget UI.
            // Forecast running costs before commissioning more facilities.
            let deficit = (city.expenses - city.income).max(0);
            let tight = city.money < 5_000 + deficit * 6;
            if tight && city.population > 200 {
                let old = city.taxes;
                city.taxes = city.taxes.map(|t| t.saturating_add(1).min(12));
                if old != city.taxes {
                    report.budget_changes += 1;
                }
                city.refresh();
            }
            if city.money < -10_000 {
                let mut civic = std::mem::take(&mut city.civic);
                let rescue = if city.income < city.expenses {
                    civic.declare_bankruptcy(city)
                } else {
                    civic.restructure(city).or_else(|_| civic.bailout(city))
                };
                report.financial_rescues += u32::from(rescue.is_ok());
                city.civic = civic;
            }
            if city.money < 8_000 && city.debt < 20_000 && city.income >= city.expenses {
                let _ = city.take_loan();
            }
            if city.money > 15_000 {
                let mut civic = std::mem::take(&mut city.civic);
                if let Some(index) = civic
                    .bonds
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| city.money > b.principal + 10_000)
                    .min_by_key(|(_, b)| b.months_left)
                    .map(|(i, _)| i)
                {
                    let _ = civic.repay_bond(city, index);
                }
                city.civic = civic;
            }
            let w = i32::from(city.width);
            let h = i32::from(city.height);
            let center = w / 2;
            let district = ((d / 30) % ((h as u32 / 4).max(1))) as i32 * 4;
            renew_empty_sites(city, district, &mut report);
            let mut streets = vec![];
            for y in 0..h {
                if city.tile(center, y).is_some_and(|t| t.kind == Kind::Empty) {
                    streets.push((center, y, Kind::Road));
                }
            }
            if city.money > 6_000 {
                for x in (center - 12).max(0)..(center + 13).min(w) {
                    if city
                        .tile(x, district)
                        .is_some_and(|t| t.kind == Kind::Empty)
                        && !streets
                            .iter()
                            .any(|(sx, sy, _)| *sx == x && *sy == district)
                    {
                        streets.push((x, district, Kind::Road));
                    }
                }
            }
            for outcome in city.edit_batch(&streets) {
                if outcome.is_ok() {
                    report.edits += 1;
                } else {
                    report.refusals += 1;
                }
            }
            let count = |city: &City, kind: Kind| {
                city.tiles
                    .iter()
                    .enumerate()
                    .filter(|(i, t)| {
                        t.kind == kind
                            && city
                                .ext
                                .cells
                                .get(*i)
                                .is_none_or(|c| !c.rubble && !c.abandoned)
                    })
                    .count()
            };
            // Supply reliable baseload before expecting a new neighborhood to grow.
            if city.power_supply < city.power_demand + 1_500
                && !has_pending(city, |k| {
                    matches!(k, Kind::Coal | Kind::Gas | Kind::Nuclear | Kind::Fusion)
                })
            {
                let _ = build_best(city, Kind::Coal, &mut report);
            }
            if city.water_supply < city.water_demand + 1_500
                && !has_pending(city, |k| matches!(k, Kind::Pump | Kind::WaterTower))
                && !build_best(city, Kind::Pump, &mut report)
            {
                let _ = build_best(city, Kind::WaterTower, &mut report);
            }
            for (kind, needed) in [
                (
                    Kind::Sewage,
                    (city.ext.sewage_overflow > 0 || city.population > 0)
                        && count(city, Kind::Sewage) < (city.population / 5000 + 1) as usize,
                ),
                (
                    Kind::Recycling,
                    (city.garbage > 15
                        || (city.population > 0
                            && count(city, Kind::Recycling) == 0
                            && count(city, Kind::Landfill) == 0))
                        && count(city, Kind::Recycling) < (city.population / 8000 + 1) as usize,
                ),
                (
                    Kind::School,
                    city.education < 55
                        && count(city, Kind::School) < (city.population / 4000 + 1) as usize,
                ),
                (
                    Kind::University,
                    city.population >= 4500 && count(city, Kind::University) == 0,
                ),
                (
                    Kind::Hospital,
                    (city.health < 55 || city.ext.services.ill > city.ext.services.beds + 30)
                        && count(city, Kind::Hospital) < (city.population / 12000 + 1) as usize,
                ),
                (
                    Kind::Fire,
                    city.population > 300
                        && count(city, Kind::Fire) + count(city, Kind::FireHQ)
                            < (city.population / 10000 + 1) as usize,
                ),
                (
                    Kind::Police,
                    city.population > 1_500
                        && count(city, Kind::Police) < (city.population / 15000 + 1) as usize,
                ),
                (
                    Kind::Cemetery,
                    city.ext.services.deaths_waiting > 10
                        && count(city, Kind::Cemetery) < (city.population / 20000 + 1) as usize,
                ),
                (
                    Kind::MaintenanceDepot,
                    city.population > 2_500 && count(city, Kind::MaintenanceDepot) == 0,
                ),
                (
                    Kind::Park,
                    city.happiness < 65
                        && count(city, Kind::Park) < (city.population / 1000 + 3) as usize,
                ),
            ] {
                if needed && !has_pending(city, |k| k == kind) {
                    let _ = build_best(city, kind, &mut report);
                }
            }
            if city.money > 20_000 && city.debt > 0 && city.income > city.expenses {
                let _ = city.repay_loan();
            }
            if !city.civic.damage.is_empty() {
                let mut civic = std::mem::take(&mut city.civic);
                let _ = civic.request_relief(city);
                city.civic = civic;
                let repairs: Vec<_> = city.civic.damage.iter().take(3).map(|d| d.tile).collect();
                let mut services = std::mem::take(&mut city.ext.services);
                for tile in repairs {
                    let _ = services.queue_repair(city, tile);
                }
                city.ext.services = services;
            }
            if city.money > 5_000
                && city.power_supply > city.power_demand
                && city.water_supply > city.water_demand
            {
                let mut zones = vec![];
                for y in district + 1..=(district + 2).min(h - 1) {
                    for x in (center - 10).max(0)..=(center + 10).min(w - 1) {
                        if city
                            .tile(x, y)
                            .is_some_and(|t| t.kind == Kind::Empty && t.terrain != Terrain::Water)
                        {
                            let kind = match (x + 2 * y).rem_euclid(6) {
                                0 | 1 => Kind::Industrial,
                                2 => Kind::Commercial,
                                _ => Kind::Residential,
                            };
                            zones.push((x, y, kind));
                        }
                    }
                }
                for outcome in city.edit_batch(&zones) {
                    if outcome.is_ok() {
                        report.edits += 1;
                    } else {
                        report.refusals += 1;
                    }
                }
            }
        }
        let start = std::time::Instant::now();
        city.advance_day();
        report
            .day_samples_ns
            .push(start.elapsed().as_nanos() as u64);
        report.days += 1;
        report.minimum_funds = report.minimum_funds.min(city.money);
        report.maximum_population = report.maximum_population.max(city.population);
        let defaulted = city.civic.events[crate::content::CivicEvent::DeclaredBankruptcy as usize]
            > defaults_before;
        if (city.money < -10_000 || defaulted) && last_bankrupt_month != Some(city.month) {
            report.bankrupt_months += 1;
            last_bankrupt_month = Some(city.month);
        }
    }
    report
}
/// Dense, reproducible 128² starting data. This is fixture construction, not bot play.
pub fn dense_fixture(seed: u64, minimum_buildings: usize) -> City {
    let mut city = City::world(seed, MapStyle::Flatlands, Climate::Temperate);
    city.money = 10_000_000;
    city.month = 120;
    city.ext.day = 3600;
    city.civic.day = 3600;
    city.civic.disaster_frequency = 0;
    let w = city.width as usize;
    for i in 0..city.tiles.len() {
        let x = i % w;
        let y = i / w;
        let mut t = Tile::new(Terrain::Grass);
        if x.is_multiple_of(4) || y.is_multiple_of(4) {
            t.kind = Kind::Avenue;
        }
        city.tiles[i] = t;
    }
    // A real utility bank follows the eastern boundary. Every facility has road
    // access, physical conduits and a river; capacity covers fully grown lots.
    for y in 0..128 {
        let i = y * w + 126;
        city.tiles[i].terrain = Terrain::Water;
    }
    for (y, kind, x) in (1..=80)
        .map(|y| (y, Kind::Pump, 125))
        .chain((1..=100).map(|y| (y, Kind::Sewage, 121)))
        .chain((1..=120).map(|y| (y, Kind::Recycling, 117)))
    {
        let i = y * w + x;
        city.tiles[i].kind = kind;
    }
    for y in [10usize, 34, 58, 82, 106] {
        for x in [13usize, 37, 61, 85, 109] {
            for (dx, dy, k) in [
                (0, 0, Kind::Police),
                (1, 0, Kind::FireHQ),
                (0, 1, Kind::Hospital),
                (1, 1, Kind::School),
                (0, 2, Kind::University),
                (1, 2, Kind::Park),
                (2, 0, Kind::Library),
                (2, 1, Kind::Prison),
                (2, 2, Kind::Cemetery),
            ] {
                let i = (y + dy) * w + x + dx;
                city.tiles[i].kind = k;
            }
        }
    }
    for (x, k) in [
        (1, Kind::Fusion),
        (5, Kind::Fusion),
        (9, Kind::Fusion),
        (13, Kind::MaintenanceDepot),
    ] {
        city.tiles[3 * w + x].kind = k;
    }
    let target_lots = minimum_buildings
        .saturating_sub(city.tiles.iter().filter(|t| t.kind.is_building()).count())
        .div_ceil(16);
    let mut placed = 0;
    for y in 1..127 {
        for x in 1..116 {
            let i = y * w + x;
            if city.tiles[i].kind != Kind::Empty {
                continue;
            }
            if placed >= target_lots {
                break;
            }
            let t = &mut city.tiles[i];
            t.kind = match (x / 4 + y / 4) % 4 {
                0 => Kind::Industrial,
                1 => Kind::Commercial,
                _ => Kind::Residential,
            };
            t.level = 5;
            if t.kind == Kind::Residential {
                t.population = 360;
            } else {
                t.jobs = if t.kind == Kind::Commercial { 360 } else { 210 };
            }
            placed += 1;
        }
    }
    city.population = city.tiles.iter().map(|t| t.population).sum();
    city.jobs = city.tiles.iter().map(|t| t.jobs).sum();
    let mut ext = std::mem::take(&mut city.ext);
    for (i, cell) in ext.cells.iter_mut().enumerate() {
        cell.elevation = 0;
        cell.construction = 100;
        cell.condition = 100;
        cell.built_month = 120;
        cell.age_days = 3600;
        cell.pipe = city.tiles[i].kind.is_road();
        cell.sewer = cell.pipe;
        cell.underground_power = cell.pipe;
    }
    ext.sync_buildings(&city);
    city.ext = ext;
    city.refresh();
    let mut people = std::mem::take(&mut city.ext.people);
    people.initialize_existing(&city);
    city.ext.people = people;
    let mut transport = std::mem::take(&mut city.ext.transport);
    transport.initialize_commutes(&city);
    city.ext.transport = transport;
    let mut civic = std::mem::take(&mut city.civic);
    civic.day = city.ext.day as u32;
    civic.save_january(&city);
    city.civic = civic;
    city
}
