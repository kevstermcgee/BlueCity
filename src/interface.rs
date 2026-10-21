use super::*;
use vesper3d::two_d::draw::{self, Color, Scene};
const BG: Color = Color::new(0.047, 0.086, 0.125, 1.);
const PANEL: Color = Color::new(0.071, 0.125, 0.169, 1.);
const BUTTON: Color = Color::new(0.106, 0.184, 0.224, 1.);
const MUTED: Color = Color::new(0.51, 0.65, 0.68, 1.);
const TEXT: Color = Color::new(0.88, 0.93, 0.91, 1.);
const CYAN: Color = Color::new(0.32, 0.85, 0.77, 1.);
const AMBER: Color = Color::new(0.98, 0.73, 0.37, 1.);
const RED: Color = Color::new(0.98, 0.43, 0.41, 1.);
fn text(s: &mut Scene, x: i32, y: i32, t: impl Into<String>, size: f32, c: Color) {
    s.text(51, t, Point::new(x, y), size, c);
}
fn button(s: &mut Scene, r: Rect, label: impl Into<String>, active: bool) {
    let label = label.into();
    let size = ((r.w - 10) as f32 / (label.chars().count().max(1) as f32 * 0.55)).clamp(9., 14.);
    s.rect(
        50,
        r,
        if active {
            Color::new(0.15, 0.36, 0.36, 1.)
        } else {
            BUTTON
        },
    );
    text(
        s,
        r.x + 6,
        r.y + r.h / 2 + 5,
        label,
        size,
        if active { CYAN } else { TEXT },
    );
    if active {
        s.rect(51, Rect::new(r.x, r.y + r.h - 2, r.w, 2), CYAN);
    }
}
fn line(s: &mut Scene, y: i32) {
    s.rect(50, Rect::new(610, y, 176, 1), BUTTON);
}
fn wrap(s: &mut Scene, x: i32, mut y: i32, t: &str, limit: usize, size: f32, c: Color) -> i32 {
    for paragraph in t.split('\n') {
        let mut out = String::new();
        for word in paragraph.split_whitespace() {
            if out.len() + word.len() + 1 > limit && !out.is_empty() {
                text(s, x, y, &out, size, c);
                out.clear();
                y += 17;
            }
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(word);
        }
        if !out.is_empty() {
            text(s, x, y, out, size, c);
            y += 17;
        }
    }
    y
}
fn metric(s: &mut Scene, y: i32, label: &str, value: impl Into<String>, color: Color) {
    text(s, 610, y, label, 14., MUTED);
    text(s, 716, y, value, 14., color);
}
fn bar(s: &mut Scene, y: i32, label: &str, value: u32, color: Color) {
    text(s, 610, y, label, 13., MUTED);
    s.rect(50, Rect::new(698, y - 9, 64, 6), BUTTON);
    s.rect(
        51,
        Rect::new(698, y - 9, (value.min(100) * 64 / 100) as i32, 6),
        color,
    );
    text(s, 765, y, format!("{}", value), 12., TEXT);
}
fn money(v: i64) -> String {
    if v.abs() >= 1_000_000 {
        format!("${:.1}m", v as f64 / 1_000_000.)
    } else {
        format!("${v}")
    }
}
impl draw::Game for BlueCity {
    fn show_hud() -> bool {
        false
    }
    fn menu_status(&self) -> String {
        "Build a city. Watch it grow.".into()
    }
    fn draw<'a>(&'a self, s: &mut Scene<'a>) {
        let st = &self.state;
        let c = &st.city;
        s.rect(-20, Rect::new(0, 0, 800, 450), BG);
        render::map(
            c,
            render::View {
                cam_x: st.cam_x,
                cam_y: st.cam_y,
                zoom: st.zoom,
                overlay: st.overlay,
                hover: st.hover,
                anchor: st.anchor,
                tick: st.tick,
            },
            s,
        );
        s.rect(40, Rect::new(0, 0, 800, 64), BG);
        s.rect(40, Rect::new(592, 64, 208, 386), PANEL);
        s.rect(42, Rect::new(592, 64, 2, 386), BUTTON);
        s.rect(40, Rect::new(0, 370, 592, 80), BG);
        text(s, 12, 27, "BLUECITY", 26., CYAN);
        text(s, 157, 13, "TREASURY", 10., MUTED);
        text(
            s,
            157,
            29,
            money(c.money),
            18.,
            if c.money >= 0 { TEXT } else { RED },
        );
        text(s, 284, 13, "RESIDENTS", 10., MUTED);
        text(s, 284, 29, format!("{}", c.population), 18., TEXT);
        text(s, 397, 13, "HAPPINESS", 10., MUTED);
        text(
            s,
            397,
            29,
            format!("{}%", c.happiness),
            18.,
            if c.happiness > 55 { CYAN } else { AMBER },
        );
        let months = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        text(
            s,
            515,
            27,
            format!("{} {}", months[c.month as usize % 12], 2026 + c.month / 12),
            18.,
            TEXT,
        );
        text(s, 683, 17, "BLUEENGINE GAMES", 10., MUTED);
        text(s, 683, 29, "A place to belong", 12., TEXT);
        for (i, label) in ["II", "1x", "3x", "10x"].iter().enumerate() {
            button(
                s,
                Rect::new(8 + i as i32 * 32, 38, 29, 23),
                *label,
                st.speed == i as u8,
            );
        }
        button(s, Rect::new(144, 38, 26, 23), "-", false);
        button(s, Rect::new(174, 38, 26, 23), "+", false);
        button(
            s,
            Rect::new(208, 38, 147, 23),
            format!("Map: {}", OVERLAYS[st.overlay as usize]),
            st.overlay != 0,
        );
        button(
            s,
            Rect::new(362, 38, 68, 23),
            "Cancel",
            st.tool == Tool::Inspect,
        );
        button(s, Rect::new(438, 38, 68, 23), "Center", false);
        for (i, label) in ["NW", "SE", "NE", "SW"].iter().enumerate() {
            button(s, Rect::new(514 + i as i32 * 31, 38, 28, 23), *label, false);
        }
        text(s, 644, 53, "K / L", 12., MUTED);
        button(s, Rect::new(694, 38, 94, 23), "Mayor's guide", st.tab == 3);
        for (i, label) in ["Zoning", "Transport", "Utilities", "Services", "Landmarks"]
            .iter()
            .enumerate()
        {
            button(
                s,
                Rect::new(i as i32 * 118, 374, 116, 21),
                *label,
                st.category == i as u8,
            );
        }
        let ts = tools(st.category);
        let w = 592 / ts.len() as i32;
        for (i, t) in ts.iter().enumerate() {
            let x = i as i32 * w;
            button(
                s,
                Rect::new(x + 2, 400, w - 4, 33),
                match t {
                    Tool::Build(Kind::Coal) => "Coal",
                    Tool::Build(Kind::Wind) => "Wind",
                    Tool::Build(Kind::Nuclear) => "Nuclear",
                    Tool::Build(Kind::Pump) => "Pump",
                    Tool::Build(Kind::Pipe) => "Pipes",
                    Tool::Build(Kind::Sewage) => "Sewage",
                    _ => t.label(),
                },
                st.tool == *t,
            );
            text(
                s,
                x + 8,
                430,
                if *t == Tool::Inspect {
                    "Select a tile".into()
                } else {
                    format!("${}", t.cost())
                },
                11.,
                MUTED,
            );
        }
        let status = if st.notice_ticks > 0 {
            st.notice.clone()
        } else {
            format!(
                "{}  |  {}  |  {}",
                st.tool.label(),
                if st.anchor.is_some() {
                    "Click the endpoint to build"
                } else if st.tool.area() {
                    "Click start, then endpoint"
                } else {
                    "Click a tile"
                },
                "Esc menu"
            )
        };
        text(
            s,
            8,
            446,
            status.chars().take(98).collect::<String>(),
            11.,
            if st.notice_ticks > 0 { CYAN } else { MUTED },
        );
        for (i, label) in ["City", "Maps", "Budget", "Guide"].iter().enumerate() {
            button(
                s,
                Rect::new(600 + i as i32 * 48, 69, 45, 22),
                *label,
                st.tab == i as u8,
            );
        }
        if st.confirm_new {
            text(s, 610, 123, "A NEW BEGINNING", 18., CYAN);
            wrap(s,610,151,"Start a blank valley with a fresh treasury. This replaces your current town. K saves a manual checkpoint; L loads it.",25,14.,TEXT);
            button(s, Rect::new(610, 330, 176, 25), "Start blank map", false);
            button(s, Rect::new(610, 360, 176, 25), "Keep this town", true);
            return;
        }
        match st.tab {
            0 => {
                if let Some([x, y]) = st.selected {
                    if let Some(tile) = c.tile(x, y) {
                        text(s, 610, 117, tile.kind.label().to_uppercase(), 18., CYAN);
                        text(
                            s,
                            610,
                            136,
                            format!("Block {}, {}  |  Level {}", x, y, tile.level),
                            13.,
                            MUTED,
                        );
                        line(s, 145);
                        metric(s, 165, "Residents", tile.population.to_string(), TEXT);
                        metric(s, 184, "Jobs", tile.jobs.to_string(), TEXT);
                        metric(
                            s,
                            203,
                            "Road access",
                            if tile.connected { "Yes" } else { "No" },
                            if tile.connected { CYAN } else { RED },
                        );
                        metric(
                            s,
                            222,
                            "Electricity",
                            if tile.powered { "Online" } else { "Missing" },
                            if tile.powered { CYAN } else { AMBER },
                        );
                        metric(
                            s,
                            241,
                            "Water",
                            if tile.watered { "Online" } else { "Missing" },
                            if tile.watered { CYAN } else { AMBER },
                        );
                        bar(s, 264, "Land value", tile.land_value as u32, CYAN);
                        bar(s, 284, "Traffic", tile.traffic as u32, AMBER);
                        bar(s, 304, "Pollution", tile.pollution as u32, RED);
                        bar(s, 324, "Crime", tile.crime as u32, RED);
                    }
                } else {
                    text(s, 610, 117, "YOUR CITY, ALIVE", 18., CYAN);
                    text(s, 610, 138, "Demand for new development", 13., MUTED);
                    for (i, (label, color)) in [
                        ("Homes", Color::new(0.42, 0.81, 0.44, 1.)),
                        ("Shops", Color::new(0.36, 0.68, 0.94, 1.)),
                        ("Industry", AMBER),
                    ]
                    .iter()
                    .enumerate()
                    {
                        bar(
                            s,
                            161 + i as i32 * 21,
                            label,
                            c.demand[i].max(0) as u32,
                            *color,
                        );
                    }
                    line(s, 213);
                    metric(s, 234, "Monthly tax", money(c.income), CYAN);
                    metric(s, 253, "Running costs", money(c.expenses), AMBER);
                    metric(
                        s,
                        272,
                        "Monthly net",
                        money(c.income - c.expenses),
                        if c.income >= c.expenses { CYAN } else { RED },
                    );
                    line(s, 284);
                    bar(s, 306, "Health", c.health as u32, CYAN);
                    bar(s, 327, "Education", c.education as u32, CYAN);
                    text(s, 610, 347, "Click any block to inspect it.", 12., MUTED);
                }
                button(
                    s,
                    Rect::new(610, 354, 176, 25),
                    "Mayor's guide + disasters",
                    false,
                );
                button(
                    s,
                    Rect::new(610, 386, 176, 25),
                    "New blank valley...",
                    false,
                );
                let news = c
                    .messages
                    .last()
                    .map(String::as_str)
                    .unwrap_or("Build a neighborhood. Watch it grow.");
                wrap(
                    s,
                    610,
                    429,
                    &news.chars().take(52).collect::<String>(),
                    29,
                    11.,
                    MUTED,
                );
            }
            1 => {
                text(s, 610, 115, "SEE WHAT MATTERS", 16., CYAN);
                for (i, label) in OVERLAYS.iter().enumerate() {
                    button(
                        s,
                        Rect::new(610, 125 + i as i32 * 24, 176, 21),
                        *label,
                        st.overlay == i as u8,
                    );
                }
                metric(
                    s,
                    359,
                    "Power",
                    format!("{}/{}", c.power_demand, c.power_supply),
                    if c.power_demand > c.power_supply {
                        RED
                    } else {
                        CYAN
                    },
                );
                metric(
                    s,
                    378,
                    "Water",
                    format!("{}/{}", c.water_demand, c.water_supply),
                    if c.water_demand > c.water_supply {
                        RED
                    } else {
                        CYAN
                    },
                );
                text(s, 610, 399, "POPULATION HISTORY", 11., MUTED);
                let h = &c.history;
                let max = h.iter().map(|h| h.population).max().unwrap_or(1).max(1);
                let n = h.len().min(40);
                let width = 176 / n.max(1) as i32;
                for (i, v) in h.iter().skip(h.len() - n).enumerate() {
                    let height = (v.population as u64 * 32 / max as u64) as i32;
                    s.rect(
                        51,
                        Rect::new(
                            610 + i as i32 * width,
                            438 - height,
                            (width - 1).max(1),
                            height.max(1),
                        ),
                        CYAN,
                    );
                }
            }
            2 => {
                text(s, 610, 116, "MAKE IT SUSTAINABLE", 16., CYAN);
                metric(
                    s,
                    137,
                    "Balance",
                    money(c.income - c.expenses),
                    if c.income >= c.expenses { CYAN } else { RED },
                );
                text(s, 610, 155, "TAXES                 - / +", 11., MUTED);
                for (i, label) in ["Residential", "Commercial", "Industrial"]
                    .iter()
                    .enumerate()
                {
                    button(
                        s,
                        Rect::new(610, 162 + i as i32 * 24, 176, 21),
                        format!("{}   -   {}%   +", label, c.taxes[i]),
                        false,
                    );
                }
                text(s, 610, 248, "FUNDING               - / +", 11., MUTED);
                for (i, label) in ["Transport", "Services", "Education", "Utilities"]
                    .iter()
                    .enumerate()
                {
                    button(
                        s,
                        Rect::new(610, 254 + i as i32 * 18, 176, 16),
                        format!("{}  -  {}%  +", label, c.funding[i]),
                        false,
                    );
                }
                button(s, Rect::new(610, 333, 84, 23), "Borrow", false);
                button(s, Rect::new(700, 333, 86, 23), "Repay", false);
                text(
                    s,
                    610,
                    367,
                    format!("Debt {} | POLICIES", money(c.debt)),
                    11.,
                    MUTED,
                );
                for (i, label) in ["Clean air", "Free transit", "Recycling", "Business grants"]
                    .iter()
                    .enumerate()
                {
                    text(
                        s,
                        614,
                        382 + i as i32 * 16,
                        format!("[{}] {}", if c.policies[i] { "x" } else { " " }, label),
                        13.,
                        if c.policies[i] { CYAN } else { MUTED },
                    );
                }
            }
            _ => {
                text(s, 610, 116, "WELCOME, MAYOR", 18., CYAN);
                wrap(s,610,140,"1. Extend connected roads.\n2. Zone homes and jobs.\n3. Supply power and water.\n4. Add services and parks.",27,13.,TEXT);
                wrap(s,610,220,"Roads/zones: click start, then end. Buildings: one click. Cancel clears. Top arrows pan; +/- zooms.",29,12.,MUTED);
                text(s, 610, 292, "SANDBOX DISASTERS", 12., AMBER);
                for (i, label) in ["FIRE", "FLOOD", "QUAKE", "WIND", "ATOM"]
                    .iter()
                    .enumerate()
                {
                    button(
                        s,
                        Rect::new(610 + i as i32 * 35, 302, 33, 28),
                        *label,
                        false,
                    );
                }
                wrap(
                    s,
                    610,
                    349,
                    "K saves. L loads. Esc opens the menu. R restarts the starter town.",
                    29,
                    12.,
                    MUTED,
                );
                button(
                    s,
                    Rect::new(610, 386, 176, 25),
                    "New blank valley...",
                    false,
                );
                text(s, 610, 430, "Build thoughtfully. Grow together.", 12., CYAN);
            }
        }
        if let Some([x, y]) = st.hover {
            if let Some(t) = c.tile(x, y) {
                let name = format!(
                    "{}, {}  {}  |  {}",
                    x,
                    y,
                    t.kind.label(),
                    if st.anchor.is_some() {
                        "Click to finish"
                    } else {
                        st.tool.label()
                    }
                );
                s.rect(
                    45,
                    Rect::new(10, 345, 400, 19),
                    Color::new(0.04, 0.1, 0.14, 0.9),
                );
                text(s, 17, 359, name, 12., TEXT);
            }
        }
    }
}
