//! Named contributions drive both zone demand and its explanation.
use crate::city::{City, Kind};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Factor {
    pub name: &'static str,
    pub points: i32,
}
struct Context {
    population: i64,
    jobs: i64,
    happiness: i64,
    taxes: [u8; 3],
    commercial: i64,
    industrial: i64,
    airport: i64,
    stadium: i64,
    port: i64,
    grants: i64,
}
fn context(c: &City) -> Context {
    let mut v = Context {
        population: i64::from(c.population),
        jobs: i64::from(c.jobs),
        happiness: i64::from(c.happiness),
        taxes: c.taxes,
        commercial: 0,
        industrial: 0,
        airport: 0,
        stadium: 0,
        port: 0,
        grants: if c.policies[3] { 20 } else { 0 },
    };
    for t in &c.tiles {
        match t.kind {
            Kind::Commercial => v.commercial += i64::from(t.jobs),
            Kind::Industrial => v.industrial += i64::from(t.jobs),
            _ => {}
        }
        if t.connected && t.powered && t.watered && t.fire == 0 {
            match t.kind {
                Kind::Airport => v.airport += 1,
                Kind::Stadium => v.stadium += 1,
                Kind::Seaport => v.port += 1,
                _ => {}
            }
        }
    }
    v
}
fn terms(c: &Context, zone: usize) -> [(&'static str, i64); 5] {
    let p = c.population;
    match zone {
        0 => [
            ("Base", 35),
            ("Jobs", (c.jobs - p * 55 / 100) * 110 / (p + 100)),
            ("Happiness", (c.happiness - 60) * 2),
            ("Tax", -(i64::from(c.taxes[0]) - 9) * 8),
            ("", 0),
        ],
        1 => [
            ("Base", 40),
            ("Customers", (p * 20 / 100 - c.commercial) * 150 / (p + 100)),
            ("Tax", -(i64::from(c.taxes[1]) - 9) * 9),
            ("Visitors", c.airport * 20 + c.stadium * 10),
            ("Grants", c.grants),
        ],
        _ => [
            ("Base", 40),
            ("Workers", (p * 40 / 100 - c.industrial) * 120 / (p + 100)),
            ("Tax", -(i64::from(c.taxes[2]) - 9) * 9),
            ("Exports", c.port * 25),
            ("Grants", c.grants),
        ],
    }
}
pub fn factors(c: &City, zone: usize) -> Vec<Factor> {
    terms(&context(c), zone)
        .into_iter()
        .filter(|(name, _)| !name.is_empty())
        .map(|(name, n)| Factor {
            name,
            points: n.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        })
        .collect()
}
pub fn values(c: &City) -> [i32; 3] {
    let ctx = context(c);
    std::array::from_fn(|zone| {
        terms(&ctx, zone)
            .iter()
            .map(|(_, n)| *n)
            .sum::<i64>()
            .clamp(-100, 100) as i32
    })
}
pub fn explanation(c: &City, zone: usize) -> String {
    factors(c, zone)
        .into_iter()
        .map(|f| format!("{} {:+}", f.name, f.points))
        .collect::<Vec<_>>()
        .join(" / ")
}
