//! Population cohorts carry age, schooling, employment and migration history.
use crate::city::{City, Kind};
use serde::{Deserialize, Serialize};

fn operating(city: &City, i: usize) -> bool {
    let t = &city.tiles[i];
    t.connected
        && t.powered
        && t.watered
        && t.fire == 0
        && city.ext.cells.get(i).is_none_or(|c| c.operational())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Household {
    pub id: u32,
    pub home: u32,
    pub size: u16,
    pub age_days: u32,
    pub education: u8,
    pub income_class: u8,
    pub health: u8,
    pub employer: Option<u32>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Employment {
    pub household: u32,
    pub employer: u32,
    pub workers: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PeopleState {
    pub jobs_by_education: [u32; 4],
    pub employed_by_education: [u32; 4],
    pub filled_jobs_by_education: [u32; 4],
    pub workforce: u32,
    pub employed: u32,
    pub overqualified: u32,
    pub employment: Vec<Employment>,
    pub employment_stamp: u64,
    pub households: Vec<Household>,
    pub population: u32,
    pub children: u32,
    pub adults: u32,
    pub seniors: u32,
    pub births: u64,
    pub deaths: u64,
    pub migrants_in: u64,
    pub migrants_out: u64,
    pub unemployment: u8,
    pub happiness_factors: [u8; 16],
    pub education_levels: [u32; 4],
    pub income_classes: [u32; 3],
    pub goods_demand: u32,
    pub goods_produced: u32,
    pub goods_imported: u32,
    pub goods_exported: u32,
    pub goods_consumed: u32,
    pub goods_unused: u32,
    pub goods_shortfall: u32,
    pub average_commute: u16,
    pub birth_accumulator: u64,
    pub death_accumulator: u64,
    pub next_id: u32,
}
impl PeopleState {
    pub fn reconcile(&mut self, city: &City) {
        let homes: Vec<_> = city
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind == Kind::Residential && t.population > 0)
            .collect();
        let previous = std::mem::take(&mut self.households);
        let mut cursor = 0usize;
        self.households = Vec::with_capacity(homes.len());
        for (i, t) in homes {
            while cursor < previous.len() && previous[cursor].home < i as u32 {
                cursor += 1;
            }
            let existing = previous.get(cursor).filter(|h| h.home == i as u32).copied();
            let mut h = existing.unwrap_or_else(|| {
                self.next_id += 1;
                Household {
                    id: self.next_id,
                    home: i as u32,
                    age_days: 20 * 360 + (i as u32 * 127) % 15000,
                    education: city.education,
                    ..Default::default()
                }
            });
            h.size = t.population.min(u16::MAX as u32) as u16;
            h.health = city.health;
            self.households.push(h);
        }
        self.population = city.population;
    }
    pub fn advance_day(&mut self, city: &mut City, day: u64, temperature: i16) {
        if day % 30 == 1 || self.households.is_empty() {
            self.reconcile(city);
        }
        for h in &mut self.households {
            h.age_days = h.age_days.saturating_add(1);
            if day.is_multiple_of(30) && city.coverage(3, h.home as usize) > 35 {
                h.education = h.education.saturating_add(2).min(100);
            }
            if day.is_multiple_of(30) && city.coverage(4, h.home as usize) > 35 {
                h.education = h.education.saturating_add(2).min(100);
            }
            h.income_class = if h.education > 65 {
                2
            } else if h.education > 25 {
                1
            } else {
                0
            };
            h.health = ((h.health as u16 * 7 + city.health as u16) / 8) as u8;
        }
        self.update_employment(city);
        let hot = (temperature - 32).max(0) as u32;
        let cold = (-temperature).max(0) as u32;
        self.birth_accumulator += city.population as u64 * 18;
        self.death_accumulator += city.population as u64
            * (8 + (100 - city.health as u64) / 8 + hot as u64 + cold as u64 / 2);
        let births = (self.birth_accumulator / 360000) as u32;
        let deaths = (self.death_accumulator / 360000) as u32;
        self.birth_accumulator %= 360000;
        self.death_accumulator %= 360000;
        let migrate = if city.happiness > 65 && city.demand[0] > 10 {
            (city.population / 1500 + 1).min(12)
        } else {
            0
        };
        let leave = if city.happiness < 35 {
            (city.population / 500 + 1).min(40)
        } else {
            0
        };
        let mut plus = births + migrate;
        let mut minus = deaths + leave;
        let housing_capacity_percent = city.civic.modifiers().housing_capacity_percent as u32;
        let mut natural_remaining = deaths;
        city.ext.services.dead.resize(city.tiles.len(), 0);
        for (i, t) in city.tiles.iter_mut().enumerate() {
            if t.kind != Kind::Residential {
                continue;
            }
            if minus > 0 {
                let n = minus.min(t.population / 20 + u32::from(t.population > 0));
                t.population -= n;
                minus -= n;
                let natural = n.min(natural_remaining);
                city.ext.services.dead[i] = city.ext.services.dead[i].saturating_add(natural);
                natural_remaining -= natural;
            }
            if plus > 0 && t.connected && t.powered && t.watered && t.fire == 0 {
                let cap: u32 = match t.level {
                    0 => 0,
                    1 => 24,
                    2 => 64,
                    3 => 144,
                    4 => 280,
                    _ => 480,
                };
                let cap = cap * housing_capacity_percent / 100;
                let n = plus.min(cap.saturating_sub(t.population));
                t.population += n;
                plus -= n;
            }
        }
        // Counters describe people who actually entered or left occupied homes.
        // A full city cannot report immigrants who never found housing.
        let admitted = births + migrate - plus;
        let removed = deaths + leave - minus;
        self.births += admitted.min(births) as u64;
        self.migrants_in += admitted.saturating_sub(births) as u64;
        self.deaths += removed.min(deaths) as u64;
        self.migrants_out += removed.saturating_sub(deaths) as u64;
        for h in &mut self.households {
            h.size = city.tiles[h.home as usize].population.min(u16::MAX as u32) as u16;
        }
        self.summarize_cohorts();
        self.population = city.tiles.iter().map(|t| t.population).sum();
        city.population = self.population;
        self.goods_demand = city.population / 3;
        self.goods_produced = city
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if !operating(city, i) {
                    0
                } else {
                    match t.kind {
                        Kind::Industrial => {
                            t.jobs * 2 * city.industrial_productivity(i) as u32 / 100
                        }
                        Kind::Farm => {
                            t.jobs * 3 * city.ext.cells.get(i).map_or(50, |c| c.fertility) as u32
                                / 100
                        }
                        _ => 0,
                    }
                }
            })
            .sum();
        let export_capacity = crate::transport::freight_export_capacity(city);
        let road_capacity = if city.tiles.iter().enumerate().any(|(i, t)| {
            t.kind.is_road()
                && t.connected
                && (i % city.width as usize == 0
                    || i % city.width as usize + 1 == city.width as usize
                    || i / city.width as usize == 0
                    || i / city.width as usize + 1 == city.height as usize)
        }) {
            10000
        } else {
            0
        };
        let region_capacity = city
            .ext
            .neighbors
            .iter()
            .filter(|n| n.enabled)
            .map(|n| n.goods)
            .sum::<u32>()
            .min(export_capacity + road_capacity);
        self.goods_exported = self
            .goods_produced
            .saturating_sub(self.goods_demand)
            .min(region_capacity);
        self.goods_imported = self
            .goods_demand
            .saturating_sub(self.goods_produced)
            .min(region_capacity);
        let available = self.goods_produced + self.goods_imported - self.goods_exported;
        self.goods_consumed = available.min(self.goods_demand);
        self.goods_unused = available - self.goods_consumed;
        self.goods_shortfall = self.goods_demand - self.goods_consumed;
        let (mut imports, mut exports) = (self.goods_imported, self.goods_exported);
        for n in &mut city.ext.neighbors {
            n.last_goods = 0;
            if !n.enabled {
                continue;
            }
            let incoming = n.goods.min(imports);
            imports -= incoming;
            let outgoing = (n.goods - incoming).min(exports);
            exports -= outgoing;
            n.last_goods = outgoing as i32 - incoming as i32;
            n.monthly_cost = n.last_power as i64 * n.unit_price as i64 / 100
                - n.last_goods as i64 * n.tariff as i64 / 100;
        }
        let mut sums = [0u32; 9];
        let mut count = 0u32;
        for t in &city.tiles {
            if t.population == 0 {
                continue;
            }
            count += 1;
            for (index, value) in [
                100 - t.pollution,
                100 - t.crime,
                100 - t.traffic,
                t.land_value,
                u8::from(t.powered) * 100,
                u8::from(t.watered) * 100,
                u8::from(t.connected) * 100,
                100 - t.fire,
                t.crime,
            ]
            .into_iter()
            .enumerate()
            {
                sums[index] += value as u32;
            }
        }
        let avg = |index: usize| -> u8 { (sums[index] / count.max(1)) as u8 };
        self.happiness_factors = [
            city.health,
            city.education,
            100 - self.unemployment,
            avg(0),
            avg(1),
            avg(2),
            avg(3),
            100 - city.garbage,
            avg(4),
            avg(5),
            avg(6),
            100 - city.taxes[0].min(20) * 5,
            (self.goods_produced.saturating_add(self.goods_imported) * 100
                / self.goods_demand.max(1))
            .min(100) as u8,
            city.transport_usage,
            city.funding[1].min(100),
            city.happiness,
        ];
    }
    /// Populate an already occupied town without advancing demographics,
    /// collecting fares, dispatching vehicles or consuming a simulated day.
    pub fn initialize_existing(&mut self, city: &City) {
        self.reconcile(city);
        for h in &mut self.households {
            h.income_class = if h.education > 65 {
                2
            } else if h.education > 25 {
                1
            } else {
                0
            };
        }
        self.summarize_cohorts();
        self.update_employment(city);
    }
    fn update_employment(&mut self, city: &City) {
        let mut jobs = [0u32; 4];
        let mut stamp = 1469598103934665603u64;
        for (i, t) in city.tiles.iter().enumerate() {
            if t.jobs == 0 {
                continue;
            }
            stamp = stamp.wrapping_mul(1099511628211) ^ ((i as u64) << 32 | t.jobs as u64);
            stamp ^= u64::from(operating(city, i)) << 63;
            let tier = match t.kind {
                Kind::Commercial | Kind::Hotel => 1,
                Kind::Industrial => {
                    if t.level >= 3 {
                        2
                    } else {
                        0
                    }
                }
                Kind::Farm => 0,
                Kind::Arcology | Kind::Spaceport | Kind::WorldExpo | Kind::ScienceCenter => 3,
                _ => continue,
            };
            jobs[tier] += t.jobs;
        }
        self.jobs_by_education = jobs;
        for h in &self.households {
            stamp = stamp.wrapping_mul(1099511628211)
                ^ ((h.id as u64) << 32 | h.size as u64 | ((h.education as u64) << 20));
        }
        if stamp != self.employment_stamp {
            self.employment_stamp = stamp;
            self.match_jobs(city);
        }
        self.unemployment = (self.workforce.saturating_sub(self.employed) * 100
            / self.workforce.max(1))
        .min(100) as u8;
    }
    fn match_jobs(&mut self, city: &City) {
        self.employment.clear();
        self.workforce = 0;
        self.employed = 0;
        self.overqualified = 0;
        self.employed_by_education = [0; 4];
        self.filled_jobs_by_education = [0; 4];
        // Spatial vacancy grids let each household inspect nearby vacancies,
        // rather than scanning every employer again for every allocation.
        let mut vacancies = vec![0u32; city.tiles.len()];
        let mut required = vec![0u8; city.tiles.len()];
        let mut available = [0u32; 4];
        for (i, t) in city.tiles.iter().enumerate() {
            if t.jobs == 0 || !operating(city, i) {
                continue;
            }
            required[i] = match t.kind {
                Kind::Commercial | Kind::Hotel => 1,
                Kind::Industrial if t.level >= 3 => 2,
                Kind::Arcology | Kind::Spaceport | Kind::WorldExpo | Kind::ScienceCenter => 3,
                _ => 0,
            };
            vacancies[i] = t.jobs;
            available[required[i] as usize] += t.jobs;
        }
        for h in &mut self.households {
            h.employer = None;
            let tier = (h.education / 26).min(3);
            let mut remaining = h.size as u32 * 55 / 100;
            self.workforce += remaining;
            while remaining > 0 {
                if available[..=tier as usize].iter().sum::<u32>() == 0 {
                    break;
                }
                let hx = h.home as i32 % city.width as i32;
                let hy = h.home as i32 / city.width as i32;
                let mut candidate = None;
                for radius in 0..city.width as i32 + city.height as i32 {
                    for dx in -radius..=radius {
                        let dy = radius - dx.abs();
                        for y in [hy - dy, hy + dy] {
                            let Some(i) = city.idx(hx + dx, y) else {
                                continue;
                            };
                            if vacancies[i] > 0
                                && required[i] <= tier
                                && candidate.is_none_or(|j: usize| {
                                    (tier - required[i], i) < (tier - required[j], j)
                                })
                            {
                                candidate = Some(i);
                            }
                            if dy == 0 {
                                break;
                            }
                        }
                    }
                    if candidate.is_some() {
                        break;
                    }
                }
                let Some(at) = candidate else {
                    break;
                };
                let required = required[at];
                let number = remaining.min(vacancies[at]);
                vacancies[at] -= number;
                available[required as usize] -= number;
                remaining -= number;
                h.employer = h.employer.or(Some(at as u32));
                self.employment.push(Employment {
                    household: h.id,
                    employer: at as u32,
                    workers: number,
                });
                self.employed += number;
                self.employed_by_education[tier as usize] += number;
                self.filled_jobs_by_education[required as usize] += number;
                if tier > required {
                    self.overqualified += number;
                }
            }
        }
    }
    pub fn month_end(&mut self, city: &mut City) {
        self.reconcile(city);
        self.summarize_cohorts();
        let score = self
            .happiness_factors
            .iter()
            .map(|&v| v as u32)
            .sum::<u32>()
            / 16;
        city.happiness = ((city.happiness as u32 * 3 + score) / 4) as u8;
        city.unemployment = (city.unemployment as u16 + self.unemployment as u16).div_ceil(2) as u8;
    }
    fn summarize_cohorts(&mut self) {
        self.children = 0;
        self.adults = 0;
        self.seniors = 0;
        self.education_levels = [0; 4];
        self.income_classes = [0; 3];
        for h in &self.households {
            let age = h.age_days / 360;
            let children = h.size as u32 * if age < 35 { 28 } else { 15 } / 100;
            let seniors = if age > 62 {
                h.size as u32 / 2
            } else {
                h.size as u32 / 12
            };
            self.children += children;
            self.seniors += seniors;
            self.adults += h.size as u32 - children - seniors;
            self.education_levels[(h.education / 26).min(3) as usize] += h.size as u32;
            self.income_classes[h.income_class.min(2) as usize] += h.size as u32;
        }
    }
}
