//! Compact optional city windows drawn over the map.
use super::*;
use crate::ui_model::{self, Window};
pub(super) fn draw(ui: &mut Ui, st: &State) {
    if st.ui.photo {
        for c in ui_model::controls(st) {
            ui.button(c.rect, &c.label, c.active, true);
        }
        return;
    }
    if st.ui.minimap {
        minimap(ui, st, Rect::new(12, 100, 132, 104));
    }
    if st.ui.full_map {
        minimap(ui, st, Rect::new(80, 100, 640, 296));
    }
    if let Some(window) = st.ui.window {
        ui.panel(ui_model::WINDOW);
        ui.label(144., 124., window.label(), 16., INK, true);
        ui.fit(
            598.,
            123.,
            format!("Page {}", st.ui.page + 1),
            10.,
            MUTED,
            false,
            Align::Right,
            95.,
        );
        ui.line(144., 133., 634., 133., RULE);
        if window == Window::Graphs {
            graph(ui, st);
        } else if window != Window::Council && window != Window::Settings {
            for (i, (title, body)) in rows(st, window)
                .into_iter()
                .skip(st.ui.page as usize * 6)
                .take(6)
                .enumerate()
            {
                let y = 154. + i as f32 * 34.;
                ui.fit(
                    150.,
                    y,
                    title,
                    11.5,
                    INK,
                    true,
                    Align::Left,
                    window_width(window),
                );
                ui.fit(
                    150.,
                    y + 14.,
                    body,
                    10.,
                    MUTED,
                    false,
                    Align::Left,
                    window_width(window),
                );
            }
        }
        if window == Window::Settings && st.ui.page == 0 {
            for (i, (name, value)) in [
                ("Music", st.ui.preferences.music),
                ("Effects", st.ui.preferences.sound),
                ("Ambience", st.ui.preferences.ambience),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 158 + i as i32 * 32;
                ui.fit(184., y as f32 + 2., name, 11., INK, true, Align::Left, 92.);
                ui.fit(
                    332.,
                    y as f32 + 2.,
                    format!("{value}%"),
                    11.,
                    INK,
                    true,
                    Align::Right,
                    52.,
                );
                ui.rect(Rect::new(344, y - 4, 244, 4), 2., RULE);
                if value > 0 {
                    ui.rect(
                        Rect::new(344, y - 4, i32::from(value) * 244 / 100, 4),
                        2.,
                        BLUE,
                    );
                }
                ui.circle(344. + f32::from(value) * 2.44, y as f32 - 2., 4.5, BLUE);
            }
        }
    }
    for c in ui_model::controls(st) {
        if matches!(c.command, ui_model::Command::SetVolume(_, _)) {
            continue;
        }
        ui.button(c.rect, &c.label, c.active, true);
    }
    if st.overlay > 0 {
        let r = Rect::new(12, 212, 186, 76);
        ui.panel(r);
        ui.fit(
            22.,
            228.,
            OVERLAYS[st.overlay as usize],
            11.,
            INK,
            true,
            Align::Left,
            164.,
        );
        if st.overlay == 7 {
            for (i, (name, kind)) in [
                ("Homes", Kind::Residential),
                ("Shops", Kind::Commercial),
                ("Work", Kind::Industrial),
                ("Farms", Kind::Farm),
            ]
            .into_iter()
            .enumerate()
            {
                ui.rect(
                    Rect::new(22 + i as i32 * 42, 238, 32, 7),
                    0.,
                    render::zoning_color(kind),
                );
                ui.label(22. + i as f32 * 42., 259., name, 8.5, INK, false);
            }
            ui.label(22., 277., "Grey: municipal / open land", 8.5, MUTED, false);
        } else {
            let stops = render::legend_colors(st.overlay);
            for i in 0..20 {
                let at = i as f32 / 19. * 4.;
                let lo = (at.floor() as usize).min(3);
                let t = at - lo as f32;
                let a = stops[lo];
                let b = stops[lo + 1];
                let color = Color::new(
                    a.r + (b.r - a.r) * t,
                    a.g + (b.g - a.g) * t,
                    a.b + (b.b - a.b) * t,
                    0.95,
                );
                ui.rect(Rect::new(22 + i * 8, 238, 9, 6), 0., color);
            }
            let (left, right, units) = match st.overlay {
                1 => ("Offline", "Online", "Electricity connection"),
                2 => ("Unserved", "Supplied", "Water connection"),
                3 => ("0%", "100%+", "Road capacity in use"),
                4 => ("Clean", "100", "Pollution index"),
                5 => ("0", "100", "Land value"),
                6 => ("0", "100", "Crime risk"),
                8 => ("0%", "100%", "Service coverage"),
                9 => ("0", "48", "Ground height"),
                10 => ("0", "10,000+", "Combined ore and oil reserves"),
                11 => ("0%", "100%", "Sewage overflow"),
                12 => ("0", "100", "Uncollected waste load"),
                13 => ("0", "100", "Health and access"),
                14 => ("0%", "100%", "School / university access"),
                15..=18 => ("0", "100", "Development desirability"),
                19 => ("0%", "100%", "Transit accessibility"),
                _ => ("0", "100", "Index"),
            };
            ui.label(22., 259., left, 9., MUTED, false);
            ui.fit(184., 259., right, 9., MUTED, false, Align::Right, 80.);
            ui.fit(22., 277., units, 8.5, MUTED, false, Align::Left, 164.);
        }
    }

    if let Some(at) = st.hover {
        if st.tool != Tool::Inspect {
            let plan = crate::placement::plan(st, at);
            let p = crate::projection::project_tile(st, at[0], at[1]);
            let text = plan.refusal.unwrap_or_else(|| {
                format!(
                    "{} · {} tiles · {}",
                    st.tool.label(),
                    plan.cells.len(),
                    money(plan.cost)
                )
            });
            let x = (p.x + 16).clamp(12, 496);
            let y = (p.y + 18).clamp(100, 369);
            ui.panel(Rect::new(x, y, 292, 24));
            ui.fit(
                (x + 8) as f32,
                (y + 16) as f32,
                text,
                11.,
                INK,
                false,
                Align::Left,
                276.,
            );
        }
    }
    if let Some(i) = st
        .city
        .civic
        .next_milestone()
        .filter(|_| crate::layout::next_unlock_visible(st))
    {
        let m = &crate::content::MILESTONES[i];
        ui.panel(crate::layout::NEXT_UNLOCK);
        ui.fit(
            22.,
            108.,
            format!("Next: {} · {} residents", m.title, number(m.population)),
            10.,
            INK,
            false,
            Align::Left,
            390.,
        );
    }
    let popups = popup(st);
    if let Some((_id, title, body)) = popups {
        {
            let r = ui_model::card_rect(st);
            ui.panel(r);
            ui.fit(
                220.,
                (r.y + 21) as f32,
                title,
                13.,
                BLUE,
                true,
                Align::Left,
                278.,
            );
            ui.wrap(220., (r.y + 40) as f32, &body, 49, 11., INK);
            ui.button(Rect::new(502, r.y + 5, 22, 22), "×", false, true);
        }
    }
}
fn window_width(_w: Window) -> f32 {
    386.
}

fn rows(st: &State, w: Window) -> Vec<(String, String)> {
    if let Some(rows) = crate::civic_ui::rows(st, w) {
        return rows;
    }
    let c = &st.city;
    let civic = &c.civic;
    let page = st.ui.page as usize;
    match w {
        Window::Ordinances => crate::content::ORDINANCES
            .iter()
            .map(|d| {
                (
                    format!(
                        "{} · {} · {} / month",
                        if civic.active(d.id) { "ON" } else { "OFF" },
                        d.title,
                        money(d.monthly_base)
                    ),
                    d.description.into(),
                )
            })
            .collect(),
        Window::Scenarios => crate::content::SCENARIOS
            .iter()
            .map(|d| {
                (
                    format!("{} · {} months", d.title, d.deadline_months),
                    d.briefing.into(),
                )
            })
            .collect(),
        Window::Projects => crate::content::MEGAPROJECTS
            .iter()
            .enumerate()
            .map(|(i, d)| {
                (
                    format!(
                        "{} · {} · {} months{}",
                        d.title,
                        money(d.cost),
                        d.months,
                        if civic.completed_projects & (1 << i) != 0 {
                            " · COMPLETE"
                        } else {
                            ""
                        }
                    ),
                    d.description.into(),
                )
            })
            .collect(),
        Window::Evaluation => vec![
            (
                format!("Approval {}% · Happiness {}%", civic.approval, c.happiness),
                format!(
                    "Population {} · Jobs {} · Unemployment {}%",
                    number(c.population),
                    number(c.jobs),
                    c.unemployment
                ),
            ),
            (
                "Treasury and credit".into(),
                format!(
                    "{} · Debt {} · Bonds {}",
                    money(c.money),
                    money(c.debt),
                    civic.bonds.len()
                ),
            ),
            (
                "Utilities".into(),
                format!(
                    "Power {} / {} · Water {} / {}",
                    c.power_supply, c.power_demand, c.water_supply, c.water_demand
                ),
            ),
            (
                "Public services".into(),
                format!(
                    "Health {} · Education {} · Waste {}",
                    c.health, c.education, c.garbage
                ),
            ),
            (
                "Elections".into(),
                format!(
                    "{} terms recorded · {} ordinance slots",
                    civic.elections.len(),
                    civic.ordinance_slots
                ),
            ),
            (
                "Achievements".into(),
                format!(
                    "{} / 8 · {} / 25 milestones · {} / 12 civic requests",
                    civic.achievement_bits.count_ones(),
                    civic.milestone_bits.count_ones(),
                    civic.request_bits.count_ones()
                ),
            ),
        ],
        Window::Transit => vec![
            (
                format!(
                    "{} lines · {} riders · {} min commute",
                    c.ext.transport.lines.len(),
                    c.ext.transport.ridership,
                    c.ext.transport.average_commute
                ),
                format!(
                    "Trips {} · Congested roads {}% · Fare income {}",
                    c.ext.transport.trips,
                    c.ext.transport.congestion,
                    money(c.ext.transport.fare_revenue)
                ),
            ),
            (
                "Line builder".into(),
                format!(
                    "{} stops selected. Add a depot and at least two stops.",
                    st.ui.pending_stops.len()
                ),
            ),
        ]
        .into_iter()
        .chain(c.ext.transport.lines.iter().map(|l| {
            (
                format!("{} · ${} · {} riders", l.name, l.fare, l.ridership),
                format!(
                    "{:?} · {} stops · {}",
                    l.mode,
                    l.stops.len(),
                    if l.enabled { "Running" } else { "Closed" }
                ),
            )
        }))
        .collect(),
        Window::Chirper => civic
            .chirper
            .iter()
            .rev()
            .map(|post| {
                let text = crate::content::CHIRPER
                    .get(post.id as usize)
                    .map_or("", |d| d.text);
                (
                    format!(
                        "Citizen {} · Month {}",
                        post.citizen.unwrap_or(0),
                        post.month
                    ),
                    text.into(),
                )
            })
            .collect(),
        Window::Advisors => civic
            .advisors
            .iter()
            .rev()
            .map(|post| {
                let d = &crate::content::ADVISORS[post.id as usize % 76];
                (d.name.into(), d.text.into())
            })
            .chain(if civic.advisors.is_empty() {
                vec![(
                    "Your advisors are reviewing the city.".into(),
                    "They react to 90 conditions as your city changes.".into(),
                )]
            } else {
                vec![]
            })
            .collect(),
        Window::Newspaper => civic
            .headlines
            .iter()
            .rev()
            .map(|post| {
                (
                    format!("BlueCity Post · Month {}", post.month),
                    crate::content::HEADLINES
                        .get(post.id as usize)
                        .map_or("", |d| d.text)
                        .into(),
                )
            })
            .chain(vec![(
                "The Mayor's Cat".into(),
                "Our whiskered correspondent checks the parks, benches and warm windows.".into(),
            )])
            .collect(),
        Window::Notebook => {
            let mut rows = vec![(
                "What's next".into(),
                civic
                    .next_milestone()
                    .map(|i| {
                        format!(
                            "{}: reach {} residents.",
                            crate::content::MILESTONES[i].title,
                            crate::content::MILESTONES[i].population
                        )
                    })
                    .unwrap_or_else(|| "Build the city you want to live in.".into()),
            )];
            rows.extend(crate::content::REQUESTS.iter().enumerate().map(|(i, r)| {
                (
                    format!(
                        "{} {}",
                        if civic.request_bits & (1 << i) != 0 {
                            "Complete:"
                        } else {
                            "Request:"
                        },
                        r.title
                    ),
                    r.description.into(),
                )
            }));
            rows.extend(
                crate::content::ACHIEVEMENTS
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        (
                            format!(
                                "{} {}",
                                if civic.achievement_bits & (1 << i) != 0 {
                                    "Earned:"
                                } else {
                                    "Achievement:"
                                },
                                r.title
                            ),
                            r.description.into(),
                        )
                    }),
            );
            rows
        }
        Window::Districts => civic
            .districts
            .iter()
            .map(|d| {
                (
                    d.name.clone(),
                    format!(
                        "{} residents · {} jobs · Happiness {}% · {} tiles",
                        number(d.population),
                        number(d.jobs),
                        d.happiness,
                        d.tiles.len()
                    ),
                )
            })
            .chain(vec![(
                "Districts follow the land you select.".into(),
                "Select a tile, then create a ward around it. Protect a building with Heritage."
                    .into(),
            )])
            .collect(),
        Window::Settings => {
            if page == 0 {
                vec![
                    (
                        format!(
                            "Music {}% · Sound {}% · Ambience {}%",
                            st.ui.preferences.music,
                            st.ui.preferences.sound,
                            st.ui.preferences.ambience
                        ),
                        "Volumes apply immediately and are saved.".into(),
                    ),
                    (
                        format!(
                            "Text {}% · Display {}%",
                            st.ui.preferences.text_scale, st.ui.preferences.display_scale
                        ),
                        "Resize the window freely; text stays crisp.".into(),
                    ),
                ]
            } else {
                vec![]
            }
        }
        Window::Disasters => vec![(
            format!(
                "Frequency: {}",
                ["Off", "Low", "Normal", "High"][civic.disaster_frequency as usize]
            ),
            format!(
                "{} warnings · {} damaged sites",
                civic.warnings.len(),
                civic.damage.len()
            ),
        )],
        Window::Budget => vec![(
            "Credit and recovery".into(),
            format!(
                "Cash {} · Loans {} · Bonds {} · Net {}",
                money(c.money),
                money(c.debt),
                civic.bonds.len(),
                money(c.income - c.expenses)
            ),
        )],
        _ => vec![],
    }
}
fn graph(ui: &mut Ui, st: &State) {
    let names = crate::civic::GRAPH_NAMES;
    let i = st.ui.graph % 16;
    ui.label(148., 155., names[i], 13., BLUE, true);
    let h = &st.city.civic.history;
    let values: Vec<_> = h
        .iter()
        .rev()
        .take(120)
        .map(|s| s.values[i])
        .rev()
        .collect();
    let lo = values.iter().min().copied().unwrap_or(0).min(0);
    let hi = values.iter().max().copied().unwrap_or(100).max(lo + 1);
    ui.label(
        148.,
        176.,
        format!("{} → {} · {} months", lo, hi, values.len()),
        10.,
        MUTED,
        false,
    );
    ui.line(150., 338., 628., 338., RULE);
    for (n, pair) in values.windows(2).enumerate() {
        let x = 150. + n as f32 * 478. / values.len().max(2) as f32;
        let x2 = 150. + (n + 1) as f32 * 478. / values.len().max(2) as f32;
        let y = 336. - (pair[0] - lo) as f32 * 146. / (hi - lo) as f32;
        let y2 = 336. - (pair[1] - lo) as f32 * 146. / (hi - lo) as f32;
        ui.line(x, y, x2, y2, BLUE);
    }
}
fn minimap(ui: &mut Ui, st: &State, r: Rect) {
    ui.panel(r);
    let c = &st.city;
    let sx = r.w as f32 / f32::from(c.width);
    let sy = r.h as f32 / f32::from(c.height);
    for (y, row) in c.tiles.chunks(c.width as usize).enumerate() {
        for (x, t) in row.iter().enumerate() {
            let color = if t.terrain == Terrain::Water {
                Color::new(0.23, 0.56, 0.72, 0.85)
            } else if t.kind.is_road() {
                Color::new(0.43, 0.47, 0.49, 0.9)
            } else {
                match t.kind {
                    Kind::Residential => Color::new(0.52, 0.68, 0.35, 0.9),
                    Kind::Commercial => Color::new(0.31, 0.57, 0.75, 0.9),
                    Kind::Industrial => Color::new(0.8, 0.67, 0.31, 0.9),
                    _ => Color::new(0.67, 0.76, 0.56, 0.85),
                }
            };
            ui.rect(
                Rect::new(
                    r.x + (x as f32 * sx) as i32,
                    r.y + (y as f32 * sy) as i32,
                    sx.ceil() as i32,
                    sy.ceil() as i32,
                ),
                0.,
                color,
            );
        }
    }
    let x = r.x as f32 + st.cam_x as f32 * sx;
    let y = r.y as f32 + st.cam_y as f32 * sy;
    ui.circle(x, y, 3., BLUE);
}
fn popup(st: &State) -> Option<(u32, String, String)> {
    st.ui
        .card
        .as_ref()
        .map(|c| (c.id, c.title.clone(), c.body.clone()))
        .or_else(|| {
            st.city.civic.popups.last().map(|p| {
                (
                    p.id.min(u64::from(u32::MAX)) as u32,
                    p.title.clone(),
                    p.text.clone(),
                )
            })
        })
}
