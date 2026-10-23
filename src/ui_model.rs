//! Saved interface preferences and commands. All authoritative changes run in fixed steps.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    Council,
    Ordinances,
    Districts,
    Evaluation,
    Graphs,
    Advisors,
    Newspaper,
    Chirper,
    Transit,
    Scenarios,
    Settings,
    Notebook,
    Disasters,
    Projects,
    Budget,
    Trade,
}
impl Window {
    pub const ALL: [Self; 16] = [
        Self::Council,
        Self::Ordinances,
        Self::Districts,
        Self::Evaluation,
        Self::Graphs,
        Self::Advisors,
        Self::Newspaper,
        Self::Chirper,
        Self::Transit,
        Self::Scenarios,
        Self::Settings,
        Self::Notebook,
        Self::Disasters,
        Self::Projects,
        Self::Budget,
        Self::Trade,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Council => "Mayor's desk",
            Self::Ordinances => "Ordinances",
            Self::Districts => "Districts",
            Self::Evaluation => "City evaluation",
            Self::Graphs => "History graphs",
            Self::Advisors => "Advisors",
            Self::Newspaper => "The BlueCity Post",
            Self::Chirper => "Chirper",
            Self::Transit => "Transit lines",
            Self::Scenarios => "Scenarios",
            Self::Settings => "Settings & controls",
            Self::Notebook => "Mayor's Notebook",
            Self::Disasters => "Emergency management",
            Self::Projects => "Megaprojects",
            Self::Budget => "Advanced budget",
            Self::Trade => "Neighbors & trade",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Geometry {
    #[default]
    Line,
    Curve,
    Grid,
    Rectangle,
    Brush,
    Roadside,
    BusLine,
}
impl Geometry {
    pub const ALL: [Self; 7] = [
        Self::Line,
        Self::Curve,
        Self::Grid,
        Self::Rectangle,
        Self::Brush,
        Self::Roadside,
        Self::BusLine,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::Curve => "Curve",
            Self::Grid => "Grid",
            Self::Rectangle => "Rectangle",
            Self::Brush => "Brush",
            Self::Roadside => "Roadside",
            Self::BusLine => "Bus line",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hotkey {
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    Pause,
    Save,
    Load,
    Menu,
    ZoomIn,
    ZoomOut,
    Center,
    Inspect,
    Bulldoze,
    Photo,
    Map,
    Skip,
    Fullscreen,
}
impl Hotkey {
    pub const ALL: [Self; 17] = [
        Self::PanLeft,
        Self::PanRight,
        Self::PanUp,
        Self::PanDown,
        Self::Pause,
        Self::Save,
        Self::Load,
        Self::Menu,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::Center,
        Self::Inspect,
        Self::Bulldoze,
        Self::Photo,
        Self::Map,
        Self::Skip,
        Self::Fullscreen,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::PanLeft => "Pan left",
            Self::PanRight => "Pan right",
            Self::PanUp => "Pan up",
            Self::PanDown => "Pan down",
            Self::Pause => "Pause city",
            Self::Save => "Save",
            Self::Load => "Load",
            Self::Menu => "Menu",
            Self::ZoomIn => "Zoom in",
            Self::ZoomOut => "Zoom out",
            Self::Center => "Center",
            Self::Inspect => "Inspect",
            Self::Bulldoze => "Bulldoze",
            Self::Photo => "Photo mode",
            Self::Map => "Full map",
            Self::Skip => "Next event",
            Self::Fullscreen => "Fullscreen",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub keys: Vec<String>,
    pub music: u8,
    pub sound: u8,
    pub ambience: u8,
    pub text_scale: u8,
    pub display_scale: u8,
    pub calm: bool,
    pub autosave: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            keys: [
                "A", "D", "W", "S", "Space", "K", "L", "Escape", "Equal", "Minus", "Home", "I",
                "B", "F12", "Tab", "Period", "F11",
            ]
            .map(str::to_owned)
            .to_vec(),
            music: 60,
            sound: 70,
            ambience: 50,
            text_scale: 100,
            display_scale: 100,
            calm: false,
            autosave: true,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> bool {
        self.keys.len() == Hotkey::ALL.len()
            && self.keys.iter().all(|s| KEY_NAMES.contains(&s.as_str()))
            && self.music <= 100
            && self.sound <= 100
            && self.ambience <= 100
            && (80..=150).contains(&self.text_scale)
            && (80..=150).contains(&self.display_scale)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    pub bankruptcy_month: Option<u32>,
    pub card: Option<Card>,
    pub window: Option<Window>,
    pub page: u16,
    pub palette_page: usize,
    pub geometry: Geometry,
    pub minimap: bool,
    pub full_map: bool,
    pub underground: bool,
    pub photo: bool,
    pub climate: crate::world::Climate,
    pub preferences: Preferences,
    pub rebind: Option<usize>,
    pub selected_line: usize,
    pub transit_mode: crate::transport::TransitMode,
    pub pending_stops: Vec<usize>,
    pub graph: usize,
    pub service: usize,
    pub citizen: Option<usize>,
    pub popup_dismissed: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,
    pub title: String,
    pub body: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Details {
    Milestone(usize),
    Request(usize),
    Achievement(usize),
    Scenario(usize),
    Project(usize),
    Note(usize),
    Advisor(usize),
    Headline(usize),
    Cat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    RotateRoad,
    Details(Details),
    NeighborToggle(usize),
    NeighborPower(usize, i32),
    NeighborGoods(usize, i32),
    NeighborTariff(usize, i32),
    Repair(u32),
    CancelRepair(u32),
    Relief,
    RepayBond(usize),
    RepayLoan,
    BondAmount(i64),
    Bankruptcy,
    DistrictPolicy(u16),
    RenameDistrict(u16),
    RemoveDistrict(u16),
    RemoveHeritage(u32),
    CancelProject(u8),
    Dismiss,
    Open(Window),
    Close,
    Page(i32),
    PalettePage(i32),
    Shape(Geometry),
    Minimap,
    FullMap,
    Underground,
    Photo,
    Skip,
    Climate(crate::world::Climate),
    Ordinance(usize),
    Bond,
    Bailout,
    Restructure,
    Rewind,
    Scenario(usize),
    Project(usize),
    Disaster(usize),
    Frequency(i32),
    Drill,
    PlaceForMe,
    Service(i32),
    Graph(i32),
    Meet(usize),
    Fare(i32),
    SelectLine(i32),
    TransitMode(i32),
    FinishLine,
    DeleteLine,
    Heritage,
    District,
    Setting(usize, i32),
    SetVolume(usize, u8),
    ToggleCalm,
    ToggleAutosave,
    Rebind(usize),
    ResetKeys,
    NewMap,
    Save,
    Load,
    Pause,
    Surprise,
}

use crate::{city::Kind, State, Tool};
use vesper3d::two_d::Rect;
pub const WINDOW: Rect = Rect::new(130, 100, 520, 296);
pub struct Control {
    pub rect: Rect,
    pub command: Command,
    pub label: String,
    pub active: bool,
}
fn control(rect: Rect, command: Command, label: impl Into<String>, active: bool) -> Control {
    Control {
        rect,
        command,
        label: label.into(),
        active,
    }
}
pub fn row_rect(i: usize) -> Rect {
    Rect::new(546, 142 + i as i32 * 34, 88, 30)
}
pub fn volume_slider(i: usize) -> Rect {
    Rect::new(340, 142 + i as i32 * 32, 252, 28)
}
pub fn controls(st: &State) -> Vec<Control> {
    let mut out = vec![
        control(
            Rect::new(12, 64, 80, 26),
            Command::Open(Window::Council),
            "Mayor's desk",
            st.ui.window.is_some(),
        ),
        control(
            Rect::new(96, 64, 52, 26),
            Command::Minimap,
            "Mini",
            st.ui.minimap,
        ),
        control(
            Rect::new(152, 64, 52, 26),
            Command::FullMap,
            "Map",
            st.ui.full_map,
        ),
        control(
            Rect::new(208, 64, 72, 26),
            Command::Underground,
            "Below",
            st.ui.underground,
        ),
        control(
            Rect::new(284, 64, 58, 26),
            Command::Photo,
            "Photo",
            st.ui.photo,
        ),
        control(
            Rect::new(346, 64, 76, 26),
            Command::Skip,
            "Next event",
            false,
        ),
    ];
    if st.ui.photo {
        return vec![control(
            Rect::new(12, 12, 104, 26),
            Command::Photo,
            "Exit photo mode",
            true,
        )];
    }
    if st.palette_open {
        let pages = crate::tools(st.category).len().div_ceil(8);
        if pages > 1 {
            out.push(control(
                Rect::new(674, 321, 50, 22),
                Command::PalettePage(-1),
                "Prev",
                false,
            ));
            out.push(control(
                Rect::new(730, 321, 50, 22),
                Command::PalettePage(1),
                "Next",
                false,
            ));
        }
        for (i, g) in Geometry::ALL.into_iter().enumerate() {
            out.push(control(
                Rect::new(12 + i as i32 * 88, 321, 84, 22),
                Command::Shape(g),
                g.label(),
                st.ui.geometry == g,
            ));
        }
    }
    if st.confirm_new {
        for (i, climate) in crate::world::Climate::ALL.into_iter().enumerate() {
            out.push(control(
                Rect::new(560 + i as i32 * 73, 220, 69, 20),
                Command::Climate(climate),
                climate.label(),
                st.ui.climate == climate,
            ));
        }
    }
    if st.panel_open && !st.confirm_new && st.tab == 0 {
        out.push(control(
            Rect::new(560, 334, 104, 22),
            Command::PlaceForMe,
            "Place for me",
            false,
        ));
        if let Some(i) = st
            .selected
            .and_then(|[x, y]| st.city.idx(x, y))
            .filter(|&i| crate::placement::directional_road(st.city.tiles[i].kind))
        {
            let direction = st.city.ext.cells.get(i).map_or(0, |c| c.orientation);
            out.push(control(
                Rect::new(672, 334, 104, 22),
                Command::RotateRoad,
                format!("{} · Rotate", crate::placement::direction_label(direction)),
                false,
            ));
        } else {
            out.push(control(
                Rect::new(672, 334, 104, 22),
                Command::Heritage,
                "Heritage site",
                false,
            ));
        }
    }
    if st.panel_open && !st.confirm_new && st.tab == 1 {
        out.push(control(
            Rect::new(560, 370, 104, 20),
            Command::Service(-1),
            "Prev service",
            false,
        ));
        out.push(control(
            Rect::new(672, 370, 104, 20),
            Command::Service(1),
            "Next service",
            false,
        ));
    }
    if st.panel_open && !st.confirm_new && st.tab == 2 {
        out.push(control(
            Rect::new(560, 133, 216, 19),
            Command::Open(Window::Budget),
            "Advanced budget & finance",
            false,
        ));
    }
    if let Some(window) = st.ui.window {
        out.push(control(
            Rect::new(616, 108, 24, 22),
            Command::Close,
            "×",
            false,
        ));
        if window == Window::Council {
            for (i, w) in Window::ALL
                .into_iter()
                .filter(|w| *w != Window::Council)
                .enumerate()
            {
                out.push(control(
                    Rect::new(
                        144 + (i % 3) as i32 * 164,
                        142 + (i / 3) as i32 * 44,
                        158,
                        38,
                    ),
                    Command::Open(w),
                    w.label(),
                    false,
                ));
            }
            for (i, (command, label)) in [
                (Command::NewMap, "New city"),
                (Command::Surprise, "Surprise me"),
                (Command::Save, "Save"),
                (Command::Load, "Load"),
                (
                    Command::Pause,
                    if st.speed == 0 { "Resume" } else { "Pause" },
                ),
            ]
            .into_iter()
            .enumerate()
            {
                out.push(control(
                    Rect::new(144 + i as i32 * 98, 366, 94, 22),
                    command,
                    label,
                    false,
                ));
            }
            append_dismiss(&mut out, st);
            return out;
        }
        if page_count(st) > 1 {
            out.push(control(
                Rect::new(144, 366, 62, 22),
                Command::Page(-1),
                "Previous",
                false,
            ));
            out.push(control(
                Rect::new(572, 366, 62, 22),
                Command::Page(1),
                "Next",
                false,
            ));
        }
        let start = st.ui.page as usize * 6;
        if let Some(rows) = crate::civic_ui::records(st, window) {
            for (i, row) in rows.into_iter().skip(start).take(6).enumerate() {
                if let Some((cmd, label)) = row.action {
                    out.push(control(row_rect(i), cmd, label, false));
                }
            }
            append_dismiss(&mut out, st);
            return out;
        }
        match window {
            Window::Ordinances => {
                for (i, item) in crate::content::ORDINANCES
                    .iter()
                    .enumerate()
                    .skip(start)
                    .take(6)
                {
                    out.push(control(
                        row_rect(i - start),
                        Command::Ordinance(i),
                        if st.city.civic.active(item.id) {
                            "Repeal"
                        } else {
                            "Enact"
                        },
                        st.city.civic.active(item.id),
                    ));
                }
            }
            Window::Scenarios => {
                for (i, _item) in crate::content::SCENARIOS
                    .iter()
                    .enumerate()
                    .skip(start)
                    .take(6)
                {
                    out.push(control(
                        row_rect(i - start),
                        Command::Scenario(i),
                        "Start",
                        false,
                    ));
                }
            }
            Window::Projects => {
                for (i, _item) in crate::content::MEGAPROJECTS
                    .iter()
                    .enumerate()
                    .skip(start)
                    .take(6)
                {
                    out.push(control(
                        row_rect(i - start),
                        Command::Project(i),
                        "Commission",
                        false,
                    ));
                }
            }
            Window::Settings => {
                out.push(control(
                    Rect::new(216, 366, 344, 22),
                    Command::ResetKeys,
                    "Reset keyboard bindings",
                    false,
                ));
                let settings = [
                    ("Music volume", 0),
                    ("Sound effects", 1),
                    ("City ambience", 2),
                    ("Text size", 3),
                    ("Display scale", 4),
                ];
                if st.ui.page == 0 {
                    for (i, (name, id)) in settings.into_iter().enumerate() {
                        out.push(control(
                            Rect::new(144, 142 + i as i32 * 32, if i < 3 { 30 } else { 370 }, 28),
                            Command::Setting(id, -10),
                            if i < 3 {
                                "−".into()
                            } else {
                                format!(
                                    "− {name}: {}%",
                                    if i == 3 {
                                        st.ui.preferences.text_scale
                                    } else {
                                        st.ui.preferences.display_scale
                                    }
                                )
                            },
                            false,
                        ));
                        out.push(control(
                            Rect::new(
                                if i < 3 { 604 } else { 524 },
                                142 + i as i32 * 32,
                                if i < 3 { 30 } else { 110 },
                                28,
                            ),
                            Command::Setting(id, 10),
                            "+",
                            false,
                        ));
                        if i < 3 {
                            out.push(control(
                                volume_slider(i),
                                Command::SetVolume(i, 0),
                                "",
                                false,
                            ));
                        }
                    }
                    out.push(control(
                        Rect::new(144, 308, 238, 28),
                        Command::ToggleCalm,
                        "Calm view",
                        st.ui.preferences.calm,
                    ));
                    out.push(control(
                        Rect::new(394, 308, 240, 28),
                        Command::ToggleAutosave,
                        "Autosave",
                        st.ui.preferences.autosave,
                    ));
                } else {
                    for (i, key) in Hotkey::ALL
                        .into_iter()
                        .enumerate()
                        .skip((st.ui.page as usize - 1) * 6)
                        .take(6)
                    {
                        out.push(control(
                            Rect::new(
                                144,
                                142 + (i - (st.ui.page as usize - 1) * 6) as i32 * 34,
                                490,
                                30,
                            ),
                            Command::Rebind(i),
                            format!("{}: {}", key.label(), st.ui.preferences.keys[i]),
                            st.ui.rebind == Some(i),
                        ));
                    }
                }
            }
            Window::Disasters => {
                let names = [
                    "Earthquake",
                    "Flood",
                    "Meteor",
                    "Tornado",
                    "Hostile UFO",
                    "Friendly UFO",
                    "Plane crash",
                    "Monster",
                    "Riot",
                    "Nuclear meltdown",
                ];
                for (i, _name) in names.into_iter().enumerate().skip(start).take(6) {
                    out.push(control(
                        row_rect(i - start),
                        Command::Disaster(i),
                        "Schedule",
                        false,
                    ));
                }
                out.push(control(
                    Rect::new(216, 366, 100, 22),
                    Command::Frequency(-1),
                    "Less frequent",
                    false,
                ));
                out.push(control(
                    Rect::new(322, 366, 100, 22),
                    Command::Frequency(1),
                    "More frequent",
                    false,
                ));
                out.push(control(
                    Rect::new(428, 366, 132, 22),
                    Command::Drill,
                    "Sandbox drill",
                    false,
                ));
            }
            Window::Budget => {
                for (i, (cmd, label)) in [
                    (Command::Bond, "Issue $25,000 bond"),
                    (Command::Bailout, "Emergency bailout"),
                    (Command::Restructure, "Restructure debt"),
                    (Command::Rewind, "Rewind to January"),
                    (Command::Save, "Save checkpoint"),
                    (Command::Load, "Load checkpoint"),
                ]
                .into_iter()
                .enumerate()
                {
                    out.push(control(
                        Rect::new(144, 142 + i as i32 * 34, 490, 30),
                        cmd,
                        label,
                        false,
                    ));
                }
            }
            Window::Graphs => {
                out.push(control(
                    Rect::new(216, 366, 160, 22),
                    Command::Graph(-1),
                    "Previous graph",
                    false,
                ));
                out.push(control(
                    Rect::new(392, 366, 168, 22),
                    Command::Graph(1),
                    "Next graph",
                    false,
                ));
            }
            Window::Transit => {
                out.push(control(
                    Rect::new(216, 366, 106, 22),
                    Command::SelectLine(1),
                    "Next line",
                    false,
                ));
                out.push(control(
                    Rect::new(330, 366, 228, 22),
                    Command::TransitMode(1),
                    format!("Mode: {:?}", st.ui.transit_mode),
                    false,
                ));
                for (i, (cmd, label)) in [
                    (Command::Shape(Geometry::BusLine), "Draw line"),
                    (Command::FinishLine, "Open line"),
                    (Command::DeleteLine, "Close line"),
                    (Command::Fare(-1), "Fare −"),
                    (Command::Fare(1), "Fare +"),
                ]
                .into_iter()
                .enumerate()
                {
                    out.push(control(row_rect(i), cmd, label, false));
                }
            }
            Window::Districts => {
                out.push(control(
                    Rect::new(144, 330, 238, 26),
                    Command::District,
                    "Create district at selected area",
                    false,
                ));
                out.push(control(
                    Rect::new(394, 330, 240, 26),
                    Command::Heritage,
                    "Protect selected building",
                    false,
                ));
            }
            Window::Chirper => {
                for i in 0..6 {
                    out.push(control(
                        Rect::new(574, 142 + i as i32 * 34, 60, 30),
                        Command::Meet(start + i),
                        "Meet",
                        false,
                    ));
                }
            }
            _ => {}
        }
    }
    append_dismiss(&mut out, st);
    out
}
fn append_dismiss(out: &mut Vec<Control>, st: &State) {
    if st.ui.card.is_some() || !st.city.civic.popups.is_empty() {
        out.push(control(
            Rect::new(502, card_rect(st).y + 5, 22, 22),
            Command::Dismiss,
            "×",
            false,
        ));
    }
}
pub fn page_count(st: &State) -> u16 {
    if let Some(w) = st.ui.window {
        if let Some(rows) = crate::civic_ui::records(st, w) {
            return rows.len().div_ceil(6).max(1).min(u16::MAX as usize) as u16;
        }
        if w == Window::Settings {
            return 1 + Hotkey::ALL.len().div_ceil(6) as u16;
        }
    }
    1
}
pub fn extra_tools(category: u8) -> Vec<Tool> {
    use Kind::*;
    match category {
        0 => vec![
            Farm,
            Hotel,
            Playground,
            Plaza,
            DogPark,
            SkatePark,
            WaterfrontPark,
            NatureReserve,
        ]
        .into_iter()
        .map(Tool::Build)
        .collect(),
        1 => vec![
            OneWay,
            Highway,
            Ramp,
            Roundabout,
            TunnelRoad,
            BusStop,
            Subway,
            SubwayStation,
            FreightStation,
            Airfield,
            Airport,
            Seaport,
            Marina,
            MaintenanceDepot,
        ]
        .into_iter()
        .map(Tool::Build)
        .collect(),
        2 => vec![
            Gas,
            Solar,
            Hydro,
            Fusion,
            WaterTower,
            SewageOutlet,
            Incinerator,
            Recycling,
        ]
        .into_iter()
        .map(Tool::Build)
        .collect(),
        3 => vec![Prison, FireHQ, Clinic, Cemetery, Library, CityHall]
            .into_iter()
            .map(Tool::Build)
            .collect(),
        _ => {
            let mut result = vec![
                BotanicalGarden,
                Market,
                Fountain,
                Monument,
                ClockTower,
                ConcertHall,
                Observatory,
                ScienceCenter,
                Aquarium,
                Zoo,
                SportsCenter,
                Beach,
                Campground,
                Fairground,
                HeritageMuseum,
                Arcology,
                Spaceport,
                WorldExpo,
            ]
            .into_iter()
            .map(Tool::Build)
            .collect::<Vec<_>>();
            result.extend(
                crate::world::TerrainAction::ALL
                    .into_iter()
                    .map(Tool::Shape),
            );
            result
        }
    }
}

pub fn shortcut(st: &mut State, key: usize) -> Option<Command> {
    let key = Hotkey::ALL.get(key)?;
    Some(match key {
        Hotkey::Pause => Command::Pause,
        Hotkey::Menu => {
            if st.ui.window.is_some() {
                Command::Close
            } else {
                st.speed = 0;
                Command::Open(Window::Council)
            }
        }
        Hotkey::Photo => Command::Photo,
        Hotkey::Map => Command::FullMap,
        Hotkey::Skip => Command::Skip,
        _ => return None,
    })
}
pub const KEY_NAMES: &[&str] = &[
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "Space", "Escape", "Tab", "Home", "Equal", "Minus",
    "Period", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "Left",
    "Right", "Up", "Down",
];

pub fn card_rect(st: &State) -> Rect {
    let body = st
        .ui
        .card
        .as_ref()
        .map(|c| c.body.as_str())
        .or_else(|| st.city.civic.popups.last().map(|p| p.text.as_str()));
    let lines = body
        .map(|s| {
            s.lines()
                .map(|line| line.chars().count().max(1).div_ceil(49))
                .sum::<usize>()
        })
        .unwrap_or(1);
    let height = (48 + lines as i32 * 16).clamp(72, 210);
    Rect::new(208, 366 - height, 324, height)
}
