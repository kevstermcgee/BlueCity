//! Floating, screen-resolution UI over the full-screen city.
use super::*;
use crate::city::Disaster;
#[path = "panels.rs"]
mod panels;
use crate::layout::{self, Action};
use macroquad::prelude as mq;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem::ManuallyDrop;
use vesper3d::two_d::draw::{self, Color, Scene};

const INK: Color = Color::new(0.13, 0.21, 0.29, 1.);
const MUTED: Color = Color::new(0.38, 0.46, 0.53, 1.);
const PAPER: Color = Color::new(0.98, 0.99, 1., 0.84);
const BLUE: Color = Color::new(0.12, 0.40, 0.67, 1.);
const SELECTED: Color = Color::new(0.87, 0.94, 0.99, 0.88);
const RULE: Color = Color::new(0.86, 0.90, 0.93, 1.);
const GREEN: Color = Color::new(0.18, 0.57, 0.42, 1.);
const AMBER: Color = Color::new(0.75, 0.47, 0.15, 1.);
const RED: Color = Color::new(0.77, 0.28, 0.29, 1.);

struct Fonts {
    regular: mq::Font,
    bold: mq::Font,
}
// GPU handles must not be destroyed after Macroquad tears down its context.
thread_local! { static FONTS: RefCell<Option<ManuallyDrop<Fonts>>> = const { RefCell::new(None) }; }
#[derive(Default)]
struct SeedKeys {
    pending: VecDeque<SeedKey>,
    emitted_tick: Option<u32>,
}
enum SeedKey {
    Digit(u8),
    Backspace,
    Clear,
    Generate,
}
thread_local! { static SEED_KEYS: RefCell<SeedKeys> = RefCell::new(SeedKeys::default()); }
impl Fonts {
    fn load() -> Self {
        let mut regular = mq::load_ttf_font_from_bytes(include_bytes!(
            "../assets/fonts/LiberationSans-Regular.ttf"
        ))
        .expect("bundled regular font");
        let mut bold =
            mq::load_ttf_font_from_bytes(include_bytes!("../assets/fonts/LiberationSans-Bold.ttf"))
                .expect("bundled bold font");
        regular.set_filter(mq::FilterMode::Linear);
        bold.set_filter(mq::FilterMode::Linear);
        Self { regular, bold }
    }
    fn get(&self, bold: bool) -> &mq::Font {
        if bold {
            &self.bold
        } else {
            &self.regular
        }
    }
}
#[derive(Clone, Copy)]
enum Align {
    Left,
    Center,
    Right,
}
enum Mark {
    Box(Rect, f32, Color),
    Line(f32, f32, f32, f32, f32, Color),
    Circle(f32, f32, f32, Color),
    Label {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: Color,
        bold: bool,
        align: Align,
        max_width: f32,
    },
}
struct Ui {
    marks: Vec<Mark>,
    text_scale: f32,
}
impl Default for Ui {
    fn default() -> Self {
        Self {
            marks: Vec::new(),
            text_scale: 1.,
        }
    }
}
impl Ui {
    fn rect(&mut self, r: Rect, radius: f32, color: Color) {
        self.marks.push(Mark::Box(r, radius, color));
    }
    fn line(&mut self, x: f32, y: f32, x2: f32, y2: f32, color: Color) {
        self.marks.push(Mark::Line(x, y, x2, y2, 0.7, color));
    }
    fn circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.marks.push(Mark::Circle(x, y, radius, color));
    }
    fn label(
        &mut self,
        x: f32,
        y: f32,
        text: impl Into<String>,
        size: f32,
        color: Color,
        bold: bool,
    ) {
        self.fit(x, y, text, size, color, bold, Align::Left, 800.);
    }
    #[allow(clippy::too_many_arguments)]
    fn fit(
        &mut self,
        x: f32,
        y: f32,
        text: impl Into<String>,
        size: f32,
        color: Color,
        bold: bool,
        align: Align,
        max_width: f32,
    ) {
        let size = if size <= 14. {
            size * self.text_scale
        } else {
            size
        };
        self.marks.push(Mark::Label {
            x,
            y,
            text: text.into(),
            size,
            color,
            bold,
            align,
            max_width,
        });
    }
    fn panel(&mut self, r: Rect) {
        self.rect(
            Rect::new(r.x, r.y + 3, r.w, r.h),
            7.,
            Color::new(0.07, 0.16, 0.22, 0.13),
        );
        self.rect(r, 7., PAPER);
    }
    fn button(&mut self, r: Rect, label: &str, active: bool, small: bool) {
        self.rect(
            r,
            5.,
            if active {
                SELECTED
            } else {
                Color::new(0.94, 0.96, 0.98, 0.46)
            },
        );
        self.fit(
            (r.x + r.w / 2) as f32,
            (r.y + r.h / 2) as f32 + 4.,
            label,
            if small { 11. } else { 12. },
            if active { BLUE } else { INK },
            active,
            Align::Center,
            (r.w - if r.w <= 40 { 2 } else { 4 }) as f32,
        );
    }
    fn metric(&mut self, y: f32, name: &str, value: impl Into<String>, color: Color) {
        self.label(562., y, name, 12., MUTED, false);
        self.fit(774., y, value, 12., color, true, Align::Right, 95.);
    }
    fn bar(&mut self, y: i32, name: &str, value: u32, color: Color) {
        self.label(562., y as f32, name, 12., MUTED, false);
        self.rect(Rect::new(660, y - 8, 79, 5), 2.5, RULE);
        self.rect(
            Rect::new(660, y - 8, (value.min(100) * 79 / 100) as i32, 5),
            2.5,
            color,
        );
        self.fit(
            774.,
            y as f32,
            value.to_string(),
            11.,
            INK,
            true,
            Align::Right,
            30.,
        );
    }
    fn wrap(
        &mut self,
        x: f32,
        mut y: f32,
        text: &str,
        width: usize,
        size: f32,
        color: Color,
    ) -> f32 {
        for p in text.split('\n') {
            let mut line = String::new();
            for word in p.split_whitespace() {
                if line.len() + word.len() + 1 > width && !line.is_empty() {
                    self.label(x, y, &line, size, color, false);
                    line.clear();
                    y += 17.;
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            if !line.is_empty() {
                self.label(x, y, line, size, color, false);
                y += 17.;
            }
        }
        y
    }
    fn icon(&mut self, x: f32, y: f32, kind: u8, color: Color) {
        // Small geometric symbols stay sharp at every window size.
        match kind {
            0 => {
                for (dx, h) in [(0, 8), (5, 13), (10, 10)] {
                    self.rect(Rect::new(x as i32 + dx, y as i32 - h, 4, h), 0.8, color);
                }
            }
            1 => {
                self.line(x, y - 10., x + 12., y - 10., color);
                self.line(x, y - 2., x + 12., y - 2., color);
                for dx in [1., 6., 11.] {
                    self.line(x + dx, y - 6., x + dx + 1., y - 6., color);
                }
            }
            2 => {
                self.line(x + 8., y - 14., x + 3., y - 7., color);
                self.line(x + 3., y - 7., x + 10., y - 7., color);
                self.line(x + 10., y - 7., x + 5., y, color);
            }
            3 => {
                self.rect(Rect::new(x as i32 + 5, y as i32 - 13, 4, 14), 0.5, color);
                self.rect(Rect::new(x as i32, y as i32 - 8, 14, 4), 0.5, color);
            }
            4 => {
                self.line(x, y - 10., x + 14., y - 10., color);
                for dx in [2., 7., 12.] {
                    self.line(x + dx, y - 10., x + dx, y - 1., color);
                }
                self.line(x, y, x + 14., y, color);
                self.line(x, y - 12., x + 7., y - 15., color);
                self.line(x + 7., y - 15., x + 14., y - 12., color);
            }
            _ => {
                self.circle(x + 5., y - 7., 5., color);
                self.circle(x + 5., y - 7., 3.4, PAPER);
                self.line(x + 9., y - 3., x + 13., y + 1., color);
            }
        }
    }
    fn draw(mut self, viewport: (i32, i32, i32, i32)) {
        mq::set_default_camera();
        let dpi = mq::screen_dpi_scale();
        let scale = viewport.2 as f32 / (800. * dpi);
        let left = viewport.0 as f32 / dpi;
        let top = mq::screen_height() - (viewport.1 + viewport.3) as f32 / dpi;
        let snap = |v: f32| (v * dpi).round() / dpi;
        FONTS.with(|cache| {
            let mut cached = cache.borrow_mut();
            let fonts = cached.get_or_insert_with(|| ManuallyDrop::new(Fonts::load()));
            // Populate the font atlas before submitting text, including after a resize.
            for mark in &mut self.marks {
                if let Mark::Label {
                    text,
                    size,
                    bold,
                    max_width,
                    ..
                } = mark
                {
                    let pixels = (*size * scale).round().max(1.) as u16;
                    let factor = *size * scale / pixels as f32;
                    let font = Some(fonts.get(*bold));
                    if mq::measure_text(text, font, pixels, factor).width > *max_width * scale {
                        let mut shortened = text.clone();
                        while !shortened.is_empty()
                            && mq::measure_text(&format!("{shortened}…"), font, pixels, factor)
                                .width
                                > *max_width * scale
                        {
                            shortened.pop();
                        }
                        *text = format!("{shortened}…");
                    }
                    mq::measure_text(text, font, pixels, factor);
                }
            }
            for mark in self.marks {
                match mark {
                    Mark::Box(r, radius, c) => {
                        let x = snap(left + r.x as f32 * scale);
                        let y = snap(top + r.y as f32 * scale);
                        let w = r.w as f32 * scale;
                        let h = r.h as f32 * scale;
                        let radius = (radius * scale).min(w / 2.).min(h / 2.).max(0.);
                        if radius < 0.5 {
                            mq::draw_rectangle(x, y, w, h, c);
                        } else {
                            mq::draw_rectangle(x + radius, y, w - 2. * radius, h, c);
                            mq::draw_rectangle(x, y + radius, radius, h - 2. * radius, c);
                            mq::draw_rectangle(
                                x + w - radius,
                                y + radius,
                                radius,
                                h - 2. * radius,
                                c,
                            );
                            // Disjoint corners avoid seams from repeated alpha blending.
                            for (cx, cy, start) in [
                                (x + radius, y + radius, std::f32::consts::PI),
                                (x + w - radius, y + radius, -std::f32::consts::FRAC_PI_2),
                                (x + radius, y + h - radius, std::f32::consts::FRAC_PI_2),
                                (x + w - radius, y + h - radius, 0.),
                            ] {
                                for step in 0..8 {
                                    let a = start + step as f32 * std::f32::consts::FRAC_PI_2 / 8.;
                                    let b = start
                                        + (step + 1) as f32 * std::f32::consts::FRAC_PI_2 / 8.;
                                    mq::draw_triangle(
                                        mq::vec2(cx, cy),
                                        mq::vec2(cx + a.cos() * radius, cy + a.sin() * radius),
                                        mq::vec2(cx + b.cos() * radius, cy + b.sin() * radius),
                                        c,
                                    );
                                }
                            }
                        }
                    }
                    Mark::Line(x, y, x2, y2, width, c) => mq::draw_line(
                        snap(left + x * scale),
                        snap(top + y * scale),
                        snap(left + x2 * scale),
                        snap(top + y2 * scale),
                        width * scale,
                        c,
                    ),
                    Mark::Circle(x, y, r, c) => {
                        mq::draw_circle(left + x * scale, top + y * scale, r * scale, c)
                    }
                    Mark::Label {
                        x,
                        y,
                        text,
                        size,
                        color,
                        bold,
                        align,
                        ..
                    } => {
                        let desired = size * scale;
                        let pixels = desired.round().max(1.) as u16;
                        let font_scale = desired / pixels as f32;
                        let font = Some(fonts.get(bold));
                        let w = mq::measure_text(&text, font, pixels, font_scale).width;
                        let offset = match align {
                            Align::Left => 0.,
                            Align::Center => w / 2.,
                            Align::Right => w,
                        };
                        mq::draw_text_ex(
                            &text,
                            snap(left + x * scale - offset),
                            snap(top + y * scale),
                            mq::TextParams {
                                font,
                                font_size: pixels,
                                font_scale,
                                color,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        });
    }
}
fn number(value: u32) -> String {
    let raw = value.to_string();
    let mut out = String::new();
    for (i, c) in raw.chars().enumerate() {
        if i > 0 && (raw.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
fn money(v: i64) -> String {
    if v.unsigned_abs() >= 1_000_000 {
        format!("${:.1}m", v as f64 / 1_000_000.)
    } else {
        format!(
            "{}${}",
            if v < 0 { "−" } else { "" },
            number(v.unsigned_abs().min(u32::MAX as u64) as u32)
        )
    }
}
fn drawer(ui: &mut Ui, st: &State) {
    let c = &st.city;
    ui.panel(layout::DRAWER);
    let title = if st.confirm_new {
        "Create a new map"
    } else {
        [
            if st.selected.is_some() {
                "Block details"
            } else {
                "City overview"
            },
            "Map layers",
            "City budget",
            "Mayor’s guide",
        ][st.tab as usize]
    };
    ui.label(562., 121., title, 15., INK, true);
    if !st.confirm_new && st.tab == 2 {
        ui.fit(
            750.,
            121.,
            format!("Net {}", money(c.income - c.expenses)),
            11.,
            if c.income >= c.expenses { GREEN } else { RED },
            true,
            Align::Right,
            94.,
        );
    }
    ui.line(562., 131., 774., 131., RULE);
    if st.confirm_new {
        ui.fit(
            562.,
            141.,
            "Replaces this town. K saves a checkpoint.",
            10.,
            MUTED,
            false,
            Align::Left,
            212.,
        );
        ui.label(562., 239., "Map seed", 11.5, INK, true);
        ui.fit(
            774.,
            239.,
            "Type digits or use the keypad",
            9.,
            MUTED,
            false,
            Align::Right,
            140.,
        );
        ui.fit(
            562.,
            342.,
            "Same style + climate + seed = same landscape",
            10.,
            MUTED,
            false,
            Align::Left,
            212.,
        );
        return;
    }
    match st.tab {
        0 => {
            if let Some([x, y]) = st.selected {
                if let Some(t) = c.tile(x, y) {
                    ui.label(562., 153., t.kind.label(), 15., BLUE, true);
                    ui.label(
                        562.,
                        173.,
                        format!("Block {x}, {y}  ·  Level {}", t.level),
                        11.,
                        MUTED,
                        false,
                    );
                    ui.fit(
                        562.,
                        195.,
                        format!(
                            "{} residents · {} jobs",
                            number(t.population),
                            number(t.jobs)
                        ),
                        12.,
                        INK,
                        true,
                        Align::Left,
                        212.,
                    );
                    ui.fit(
                        562.,
                        215.,
                        format!(
                            "Road {} · Power {} · Water {}",
                            if t.connected { "OK" } else { "—" },
                            if t.powered { "OK" } else { "—" },
                            if t.watered { "OK" } else { "—" }
                        ),
                        11.,
                        INK,
                        false,
                        Align::Left,
                        212.,
                    );
                    let i = c.idx(x, y).unwrap();
                    if let Some(person) = st.ui.citizen.and_then(|id| {
                        c.civic
                            .chirper
                            .iter()
                            .rev()
                            .filter_map(|post| post.person.as_ref())
                            .find(|p| p.id as usize == id)
                    }) {
                        ui.label(
                            562.,
                            239.,
                            format!("Meet citizen {}", person.id),
                            12.,
                            BLUE,
                            true,
                        );
                        ui.wrap(
                            562.,
                            259.,
                            &format!(
                                "Household {} · Education {} · Health {} · Income group {}",
                                person.size, person.education, person.health, person.income_class
                            ),
                            32,
                            11.,
                            INK,
                        );
                    } else {
                        ui.label(562., 239., "Next level needs", 12., BLUE, true);
                        let needs = c.next_level_needs(i);
                        let text = if needs.is_empty() {
                            "All current requirements are met.".into()
                        } else {
                            needs.into_iter().take(3).collect::<Vec<_>>().join(". ")
                        };
                        ui.wrap(562., 259., &text, 33, 11., INK);
                    }
                    ui.fit(
                        562.,
                        325.,
                        format!(
                            "Land {} · Pollution {} · Traffic {}",
                            t.land_value, t.pollution, t.traffic
                        ),
                        10.,
                        MUTED,
                        false,
                        Align::Left,
                        212.,
                    );
                }
            } else {
                ui.label(562., 154., "Development demand & reasons", 12., INK, true);
                for (i, (name, color)) in
                    [("Homes", GREEN), ("Commerce", BLUE), ("Industry", AMBER)]
                        .into_iter()
                        .enumerate()
                {
                    let y = 176 + i as i32 * 54;
                    ui.bar(y, name, c.demand[i].max(0) as u32, color);
                    ui.wrap(
                        562.,
                        (y + 16) as f32,
                        &crate::demand::explanation(c, i),
                        38,
                        9.5,
                        MUTED,
                    );
                }
                ui.metric(
                    326.,
                    "Monthly balance",
                    money(c.income - c.expenses),
                    if c.income >= c.expenses { GREEN } else { RED },
                );
            }
        }
        1 => {
            if st.overlay == 8 {
                ui.fit(
                    562.,
                    367.,
                    format!("Coverage: {}", crate::city::SERVICE_LABELS[st.ui.service]),
                    10.,
                    INK,
                    true,
                    Align::Left,
                    212.,
                );
            }
        }
        2 => {
            ui.label(562., 148., "Taxes", 12., INK, true);
            for (i, name) in ["Residential", "Commercial", "Industrial"]
                .iter()
                .enumerate()
            {
                let y = 171. + i as f32 * 20.;
                ui.label(562., y, *name, 11.5, MUTED, false);
                ui.fit(
                    707.,
                    y,
                    format!("{}%", c.taxes[i]),
                    11.5,
                    INK,
                    true,
                    Align::Right,
                    45.,
                );
            }
            ui.label(562., 225., "Department funding", 12., INK, true);
            for (i, name) in ["Transport", "Services", "Education", "Utilities"]
                .iter()
                .enumerate()
            {
                let y = 246. + i as f32 * 18.;
                ui.label(562., y, *name, 11.5, MUTED, false);
                ui.fit(
                    707.,
                    y,
                    format!("{}%", c.funding[i]),
                    11.5,
                    INK,
                    true,
                    Align::Right,
                    45.,
                );
            }
            ui.label(
                562.,
                345.,
                format!("Policies  ·  Debt {}", money(c.debt)),
                10.5,
                MUTED,
                false,
            );
        }
        _ => {
            ui.wrap(562.,154.,"1. Connect roads to your town.\n2. Zone homes and workplaces.\n3. Supply electricity and water.\n4. Invest in services and parks.",34,12.,INK);
            ui.wrap(
                562.,
                231.,
                "Roads/zones: two clicks. Buildings: one. WASD: pan.",
                35,
                11.5,
                MUTED,
            );
            ui.label(562., 269., "Sandbox disasters", 11., RED, true);
            ui.label(
                562.,
                328.,
                "K save · L load · Space pause",
                11.,
                MUTED,
                false,
            );
            if let (Some(seed), Some(style)) = (st.world_seed, st.world_style) {
                ui.fit(
                    562.,
                    347.,
                    format!("{} · Seed {seed}", style.label()),
                    10.5,
                    MUTED,
                    false,
                    Align::Left,
                    212.,
                );
            } else {
                ui.label(
                    562.,
                    347.,
                    "Mayor's desk: scenarios and settings.",
                    11.,
                    MUTED,
                    false,
                );
            }
        }
    }
}
impl draw::Game for BlueCity {
    fn device_input(&mut self, input: Intent) -> Intent {
        // Native edges become explicit public intentions; fixed steps own changes.
        if let Some(index) = self.state.ui.rebind {
            if let Some((code, _)) = crate::client::KEY_CODES
                .iter()
                .enumerate()
                .find(|(_, (_, k))| mq::is_key_pressed(*k))
            {
                return Intent {
                    pointer: Some(Point::new(-1100 - index as i32, code as i32)),
                    action: true,
                    x: 0,
                    y: 0,
                    ..input
                };
            }
            return Intent {
                x: 0,
                y: 0,
                ..input
            };
        }
        if self.state.ui.window == Some(crate::ui_model::Window::Settings)
            && self.state.ui.page == 0
            && mq::is_mouse_button_down(mq::MouseButton::Left)
            && input
                .pointer
                .is_some_and(|p| (0..3).any(|i| crate::ui_model::volume_slider(i).contains(p)))
        {
            return Intent {
                action: true,
                x: 0,
                y: 0,
                ..input
            };
        }
        if !self.state.confirm_new {
            for (i, key) in self.state.ui.preferences.keys.iter().enumerate() {
                if matches!(
                    crate::ui_model::Hotkey::ALL[i],
                    crate::ui_model::Hotkey::PanLeft
                        | crate::ui_model::Hotkey::PanRight
                        | crate::ui_model::Hotkey::PanUp
                        | crate::ui_model::Hotkey::PanDown
                        | crate::ui_model::Hotkey::Save
                        | crate::ui_model::Hotkey::Load
                        | crate::ui_model::Hotkey::Fullscreen
                ) {
                    continue;
                }
                if crate::client::KEY_CODES
                    .iter()
                    .any(|(name, code)| name == key && mq::is_key_pressed(*code))
                {
                    return Intent {
                        pointer: Some(Point::new(-2100 - i as i32, 0)),
                        action: true,
                        x: 0,
                        y: 0,
                        ..input
                    };
                }
            }
        }
        SEED_KEYS.with(|keys| {
            let mut keys = keys.borrow_mut();
            if !self.state.confirm_new {
                keys.pending.clear();
                keys.emitted_tick = None;
                while mq::get_char_pressed().is_some() {}
                return input;
            }
            while let Some(ch) = mq::get_char_pressed() {
                if ch.is_ascii_digit() && keys.pending.len() < 32 {
                    keys.pending.push_back(SeedKey::Digit(ch as u8 - b'0'));
                }
            }
            if mq::is_key_pressed(mq::KeyCode::Backspace) {
                keys.pending.push_back(SeedKey::Backspace);
            }
            if mq::is_key_pressed(mq::KeyCode::Delete)
                || (mq::is_key_pressed(mq::KeyCode::A)
                    && (mq::is_key_down(mq::KeyCode::LeftControl)
                        || mq::is_key_down(mq::KeyCode::RightControl)))
            {
                keys.pending.push_back(SeedKey::Clear);
            }
            if mq::is_key_pressed(mq::KeyCode::Enter) {
                keys.pending.push_back(SeedKey::Generate);
            }
            if input.action {
                keys.pending.clear();
                return input;
            }
            if keys.emitted_tick == Some(self.state.tick) {
                return input;
            }
            if let Some(key) = keys.pending.pop_front() {
                let rect = match key {
                    SeedKey::Digit(d) => layout::seed_digit(d),
                    SeedKey::Backspace => layout::SEED_BACKSPACE,
                    SeedKey::Clear => layout::SEED_CLEAR,
                    SeedKey::Generate => layout::CONFIRM_NEW,
                };
                keys.emitted_tick = Some(self.state.tick);
                Intent {
                    pointer: Some(Point::new(rect.x + rect.w / 2, rect.y + rect.h / 2)),
                    action: true,
                    x: 0,
                    y: 0,
                    ..input
                }
            } else {
                input
            }
        })
    }

    fn start_automatically() -> bool {
        true
    }
    fn show_overlays() -> bool {
        false
    }
    fn show_hud() -> bool {
        false
    }
    fn menu_status(&self) -> String {
        "Build a city. Watch it grow.".into()
    }
    fn draw<'a>(&'a self, scene: &mut Scene<'a>) {
        if mq::is_key_pressed(mq::KeyCode::Escape) {
            SEED_KEYS.with(|keys| keys.borrow_mut().pending.clear());
        }
        let st = &self.state;
        let c = &st.city;
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
                underground: st.ui.underground,
                calm: st.ui.preferences.calm,
                photo: st.ui.photo,
                service: st.ui.service,
            },
            scene,
        );
        let mut ui = Ui {
            text_scale: f32::from(st.ui.preferences.text_scale) / 100.,
            ..Ui::default()
        };
        if st.ui.photo {
            panels::draw(&mut ui, st);
            scene.render_view(40, Rect::new(0, 0, 800, 450), move |viewport| {
                ui.draw(viewport);
                Ok(())
            });
            return;
        }
        ui.panel(layout::TOP_BAR);
        ui.icon(24., 40., 0, BLUE);
        ui.label(47., 39., "BlueCity", 21., INK, true);
        for x in [140., 264., 382., 500., 626.] {
            ui.line(x, 23., x, 43., RULE);
        }
        ui.label(152., 26., "TREASURY", 8.5, MUTED, true);
        ui.label(
            152.,
            43.,
            money(c.money),
            16.,
            if c.money >= 0 { INK } else { RED },
            true,
        );
        ui.label(276., 26., "RESIDENTS", 8.5, MUTED, true);
        ui.label(276., 43., number(c.population), 16., INK, true);
        ui.label(394., 26., "APPROVAL", 8.5, MUTED, true);
        ui.circle(
            398.,
            38.,
            2.5,
            if c.civic.approval > 55 { GREEN } else { AMBER },
        );
        ui.label(406., 43., format!("{}%", c.happiness), 16., INK, true);
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        ui.label(512., 26., "CITY CALENDAR", 8.5, MUTED, true);
        ui.label(
            512.,
            43.,
            format!("{} {}", months[c.month as usize % 12], 2026 + c.month / 12),
            15.,
            INK,
            true,
        );
        for (i, demand) in c.demand.iter().enumerate() {
            let colors = [GREEN, BLUE, AMBER];
            ui.rect(Rect::new(148 + i as i32 * 36, 48, 32, 4), 1., RULE);
            ui.rect(
                Rect::new(
                    148 + i as i32 * 36,
                    48,
                    (*demand).clamp(0, 100) * 32 / 100,
                    4,
                ),
                1.,
                colors[i],
            );
        }
        ui.panel(layout::NAV);
        ui.panel(Rect::new(12, 408, 480, 30));
        for r in [
            layout::INSPECT,
            layout::BULLDOZE,
            layout::ZOOM_MINUS,
            layout::ZOOM_PLUS,
            layout::CENTER,
        ] {
            ui.panel(r);
        }
        if st.palette_open {
            ui.panel(layout::PALETTE);
        }
        if st.panel_open || st.confirm_new {
            drawer(&mut ui, st);
        }
        for (r, action) in layout::controls(st) {
            match action {
                Action::Extra(_) => {}
                Action::Speed(i) => ui.button(
                    r,
                    ["II", "1×", "3×", "10×"][i as usize],
                    st.speed == i,
                    true,
                ),
                Action::Tab(i) => ui.button(
                    r,
                    ["City", "Maps", "Budget", "Guide"][i as usize],
                    st.panel_open && st.tab == i,
                    false,
                ),
                Action::Category(i) => {
                    let active = st.palette_open && st.category == i;
                    if active {
                        ui.rect(r, 5., SELECTED);
                    }
                    let ink = if active { BLUE } else { INK };
                    ui.icon(r.x as f32 + 9., r.y as f32 + 21., i, ink);
                    ui.fit(
                        r.x as f32 + 27.,
                        r.y as f32 + 19.,
                        ["Zones", "Transport", "Utilities", "Services", "Landmarks"][i as usize],
                        11.5,
                        ink,
                        active,
                        Align::Left,
                        (r.w - 30) as f32,
                    );
                }
                Action::SelectTool(tool) => {
                    if r.y < 400 {
                        let selected = st.tool == tool;
                        ui.rect(
                            r,
                            5.,
                            if selected {
                                SELECTED
                            } else {
                                Color::new(0.94, 0.96, 0.98, 1.)
                            },
                        );
                        let name = match tool {
                            Tool::Build(Kind::PowerLine) => "Power lines",
                            Tool::Build(Kind::BusDepot) => "Bus depot",
                            _ => tool.label(),
                        };
                        ui.fit(
                            (r.x + r.w / 2) as f32,
                            r.y as f32 + 17.,
                            name,
                            11.5,
                            if selected { BLUE } else { INK },
                            true,
                            Align::Center,
                            (r.w - 8) as f32,
                        );
                        ui.fit(
                            (r.x + r.w / 2) as f32,
                            r.y as f32 + 32.,
                            if let Tool::Build(k) = tool {
                                c.civic
                                    .can_build(k, c.population)
                                    .err()
                                    .unwrap_or_else(|| money(tool.cost()))
                            } else {
                                money(tool.cost())
                            },
                            10.,
                            MUTED,
                            false,
                            Align::Center,
                            (r.w - 8) as f32,
                        );
                    } else {
                        ui.button(r, tool.label(), st.tool == tool, false);
                    }
                }
                Action::Zoom(delta) => {
                    ui.button(r, if delta < 0 { "−" } else { "+" }, false, false)
                }
                Action::Center => ui.button(r, "Center", false, true),
                Action::ClosePanel => ui.button(r, "×", false, false),
                Action::CityOverview => ui.button(r, "Overview", false, true),
                Action::Overlay(i) => ui.button(r, OVERLAYS[i as usize], st.overlay == i, true),
                Action::Tax { delta, .. } | Action::Funding { delta, .. } => {
                    ui.button(r, if delta < 0 { "−" } else { "+" }, false, true)
                }
                Action::Borrow => ui.button(r, "Borrow $10k", false, true),
                Action::Repay => ui.button(r, "Repay $10k", false, true),
                Action::Policy(i) => ui.button(
                    r,
                    ["Clean air", "Free transit", "Recycling", "Grants"][i],
                    c.policies[i],
                    true,
                ),
                Action::Disaster(d) => ui.button(
                    r,
                    match d {
                        Disaster::Fire => "Fire",
                        Disaster::Flood => "Flood",
                        Disaster::Earthquake => "EQ",
                        Disaster::Tornado => "Storm",
                        Disaster::Meltdown => "Atom",
                        _ => "Alert",
                    },
                    false,
                    true,
                ),
                Action::MapStyle(style) => {
                    ui.button(r, style.label(), st.map_style == style, false)
                }
                Action::SelectSeed => {
                    ui.rect(
                        r,
                        5.,
                        if st.seed_replace {
                            SELECTED
                        } else {
                            Color::new(1., 1., 1., 0.55)
                        },
                    );
                    ui.fit(
                        r.x as f32 + 10.,
                        r.y as f32 + 18.,
                        if st.draft_seed.is_empty() {
                            "Enter a seed"
                        } else {
                            &st.draft_seed
                        },
                        14.,
                        INK,
                        true,
                        Align::Left,
                        (r.w - 20) as f32,
                    );
                }
                Action::SeedDigit(d) => ui.button(r, &d.to_string(), false, true),
                Action::ClearSeed => ui.button(r, "Clear", false, true),
                Action::BackspaceSeed => ui.button(r, "Back", false, true),
                Action::ShuffleSeed => ui.button(r, "New seed", false, true),
                Action::NewCity => ui.button(r, "New map…", false, false),
                Action::ConfirmNew => ui.button(r, "Generate map", true, false),
                Action::CancelNew => ui.button(r, "Keep this town", false, false),
            }
        }
        if !st.palette_open && st.ui.window.is_none() && !st.ui.full_map {
            let (notice, color) = if st.notice_ticks > 0 {
                (st.notice.clone(), INK)
            } else if st.tool != Tool::Inspect {
                (
                    format!(
                        "{}  ·  {}  ·  Choose Inspect to cancel",
                        st.tool.label(),
                        if st.anchor.is_some() {
                            "Click the endpoint"
                        } else if st.tool.area() {
                            "Click the first tile"
                        } else {
                            "Click a tile"
                        }
                    ),
                    INK,
                )
            } else if st.overlay != 0 {
                (format!("{} map", OVERLAYS[st.overlay as usize]), BLUE)
            } else {
                (String::new(), MUTED)
            };
            if !notice.is_empty() {
                ui.panel(Rect::new(12, 376, 480, 23));
                ui.fit(23., 391., notice, 11., color, false, Align::Left, 458.);
            }
        }
        if let Some(at) = st.hover.filter(|_| st.tool != Tool::Inspect) {
            let plan = crate::placement::plan(st, at);
            let (w, h) = crate::projection::tile_size(st.zoom);
            let color = if plan.refusal.is_none() {
                Color::new(0.15, 0.58, 0.8, 0.9)
            } else {
                Color::new(0.9, 0.54, 0.14, 0.9)
            };
            for cell in plan.cells {
                let p = crate::projection::project_tile(st, cell[0], cell[1]);
                for (a, b) in [
                    ([p.x, p.y - h], [p.x + w, p.y]),
                    ([p.x + w, p.y], [p.x, p.y + h]),
                    ([p.x, p.y + h], [p.x - w, p.y]),
                    ([p.x - w, p.y], [p.x, p.y - h]),
                ] {
                    ui.line(a[0] as f32, a[1] as f32, b[0] as f32, b[1] as f32, color);
                }
            }
        }
        panels::draw(&mut ui, st);
        scene.render_view(40, Rect::new(0, 0, 800, 450), move |viewport| {
            ui.draw(viewport);
            Ok(())
        });
    }
}
impl crate::client::ClientGame for BlueCity {
    fn preferences(&self) -> &crate::ui_model::Preferences {
        &self.state.ui.preferences
    }
    fn set_preferences(&mut self, prefs: crate::ui_model::Preferences) {
        self.state.ui.preferences = prefs;
    }
    fn take_requests(&mut self) -> Vec<crate::client::ClientRequest> {
        std::mem::take(&mut self.requests)
    }
    fn client_notice(&mut self, message: String) {
        self.message(message);
    }
    fn draw_postcard<'a>(&'a self, scene: &mut Scene<'a>) {
        let st = &self.state;
        render::map(
            &st.city,
            render::View {
                cam_x: st.cam_x,
                cam_y: st.cam_y,
                zoom: st.zoom,
                overlay: 0,
                hover: None,
                anchor: None,
                tick: st.tick,
                underground: st.ui.underground,
                calm: st.ui.preferences.calm,
                photo: true,
                service: st.ui.service,
            },
            scene,
        );
    }
    fn editing_text(&self) -> bool {
        self.state.confirm_new || self.state.ui.rebind.is_some()
    }
}
