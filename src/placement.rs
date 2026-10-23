//! One deterministic footprint used by the placement preview and the actual edit.
use crate::{city::Kind, ui_model::Geometry, State, Tool};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacementPlan {
    pub cells: Vec<[i32; 2]>,
    pub road_directions: Vec<([i32; 2], u8)>,
    pub cost: i64,
    pub refusal: Option<String>,
}
fn segment(out: &mut Vec<[i32; 2]>, from: [i32; 2], to: [i32; 2]) {
    let [mut x, mut y] = from;
    let dx = (to[0] - x).abs();
    let sx = if x < to[0] { 1 } else { -1 };
    let dy = -(to[1] - y).abs();
    let sy = if y < to[1] { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        out.push([x, y]);
        if [x, y] == to {
            break;
        }
        let twice = 2 * err;
        if twice >= dy {
            err += dy;
            x += sx;
        }
        if twice <= dx {
            // Rules networks use cardinal neighbors. Keep the corner cell when
            // Bresenham steps on both axes so curved roads never have diagonal gaps.
            if twice >= dy {
                out.push([x, y]);
            }
            err += dx;
            y += sy;
        }
    }
}
fn ordered_cells(st: &State, start: [i32; 2], end: [i32; 2]) -> Vec<[i32; 2]> {
    let mut cells = Vec::new();
    let network = matches!(st.tool,Tool::Build(k) if k.is_network());
    match st.ui.geometry {
        Geometry::Brush => {
            for y in end[1] - 2..=end[1] + 2 {
                for x in end[0] - 2..=end[0] + 2 {
                    if (x - end[0]).pow(2) + (y - end[1]).pow(2) <= 4 {
                        cells.push([x, y]);
                    }
                }
            }
        }
        Geometry::Curve if network => {
            let control = [end[0], start[1]];
            let n = ((end[0] - start[0]).abs() + (end[1] - start[1]).abs()).max(1) * 3;
            let mut previous = start;
            for t in 0..=n {
                let v = n - t;
                let at = [
                    (v * v * start[0] + 2 * v * t * control[0] + t * t * end[0] + n * n / 2)
                        / (n * n),
                    (v * v * start[1] + 2 * v * t * control[1] + t * t * end[1] + n * n / 2)
                        / (n * n),
                ];
                segment(&mut cells, previous, at);
                previous = at;
            }
        }
        Geometry::Line if network => {
            segment(&mut cells, start, [end[0], start[1]]);
            segment(&mut cells, [end[0], start[1]], end);
        }
        Geometry::BusLine => segment(&mut cells, start, end),
        _ => {
            for y in start[1].min(end[1])..=start[1].max(end[1]) {
                for x in start[0].min(end[0])..=start[0].max(end[0]) {
                    if st.ui.geometry != Geometry::Grid
                        || (x - start[0]).rem_euclid(4) == 0
                        || (y - start[1]).rem_euclid(4) == 0
                    {
                        cells.push([x, y]);
                    }
                }
            }
        }
    }
    cells
}
pub fn directional_road(kind: Kind) -> bool {
    // Matches the existing transport rule: other road kinds are bidirectional.
    kind == Kind::OneWay
}
pub fn direction_label(direction: u8) -> &'static str {
    ["East", "South", "West", "North"][usize::from(direction % 4)]
}
fn stroke_directions(st: &State, start: [i32; 2], end: [i32; 2]) -> Vec<([i32; 2], u8)> {
    if !matches!(st.tool, Tool::Build(k) if directional_road(k))
        || !matches!(st.ui.geometry, Geometry::Line | Geometry::Curve)
    {
        return Vec::new();
    }
    let mut path = ordered_cells(st, start, end);
    path.dedup(); // Remove repeated samples without losing stroke order.
    path.iter()
        .enumerate()
        .map(|(i, &at)| {
            let [dx, dy] = if let Some(next) = path.get(i + 1) {
                [next[0] - at[0], next[1] - at[1]]
            } else if i > 0 {
                [at[0] - path[i - 1][0], at[1] - path[i - 1][1]]
            } else {
                [1, 0]
            };
            let direction = match [dx, dy] {
                [0, 1] => 1,
                [-1, 0] => 2,
                [0, -1] => 3,
                _ => 0,
            };
            (at, direction)
        })
        .collect()
}
pub fn footprint(st: &State, start: [i32; 2], end: [i32; 2]) -> Vec<[i32; 2]> {
    let mut cells = ordered_cells(st, start, end);
    cells.sort_unstable_by_key(|p| (p[1], p[0]));
    cells.dedup();
    if st.ui.geometry == Geometry::Roadside {
        cells.retain(|[x, y]| {
            (-2..=2).any(|dy: i32| {
                (-2..=2).any(|dx: i32| {
                    !st.city.tile(*x, *y).is_some_and(|t| t.kind.is_road())
                        && dx.abs() + dy.abs() <= 2
                        && st
                            .city
                            .tile(x + dx, y + dy)
                            .is_some_and(|t| t.kind.is_road())
                })
            })
        });
    }
    cells
}
pub fn plan(st: &State, end: [i32; 2]) -> PlacementPlan {
    let road_directions = stroke_directions(st, st.anchor.unwrap_or(end), end);
    let mut cells = footprint(st, st.anchor.unwrap_or(end), end);
    if let Tool::Build(kind) = st.tool {
        if kind.is_network() {
            cells.retain(|[x, y]| !st.city.tile(*x, *y).is_some_and(|t| t.kind == kind));
        }
    }

    let mut plan = PlacementPlan {
        cells,
        road_directions,
        cost: 0,
        refusal: None,
    };
    if plan.cells.is_empty() {
        plan.refusal = Some("Choose land beside a road.".into());
        return plan;
    }
    if plan.cells.len() > 1024 {
        plan.refusal = Some("Choose up to 1,024 tiles at once.".into());
        return plan;
    }
    for [x, y] in &plan.cells {
        let result = match st.tool {
            Tool::Inspect => Ok(0),
            Tool::Build(k) => st.city.placement_cost(*x, *y, k),
            Tool::Bulldoze => st.city.placement_cost(*x, *y, Kind::Empty),
            Tool::Shape(action) => st.city.terraform_cost(*x, *y, action),
            Tool::Terrain(t) => st.city.terraform_cost(
                *x,
                *y,
                match t {
                    crate::city::Terrain::Forest => crate::world::TerrainAction::Plant,
                    crate::city::Terrain::Water => crate::world::TerrainAction::Water,
                    _ => crate::world::TerrainAction::Drain,
                },
            ),
        };
        match result {
            Ok(cost) => plan.cost += cost,
            Err(reason) => {
                plan.refusal = Some(reason);
                break;
            }
        }
    }
    if plan.refusal.is_none() && plan.cost > st.city.money {
        plan.refusal = Some(format!("Need ${}; treasury ${}.", plan.cost, st.city.money));
    }
    plan
}
