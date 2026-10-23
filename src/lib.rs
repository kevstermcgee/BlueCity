//! BlueCity: fixed-step city rules and a read-only portable presentation.
pub mod city;
pub mod civic;
pub mod civic_ui;
#[cfg(feature = "client")]
pub mod client;
mod commands;
pub mod content;
pub mod demand;
pub mod layout;
pub mod mayor;
pub mod people;
pub mod placement;
pub mod projection;
#[cfg(feature = "client")]
mod render;
pub mod services;
pub mod transport;
pub mod ui_model;
mod wire;
pub mod world;
use city::{City, Disaster, Kind, MapStyle, Terrain};
use serde::{Deserialize, Serialize};
use vesper3d::two_d::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tool {
    Inspect,
    Build(Kind),
    Bulldoze,
    Terrain(Terrain),
    Shape(world::TerrainAction),
}
impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inspect => "Inspect",
            Self::Build(k) => k.label(),
            Self::Bulldoze => "Bulldoze",
            Self::Shape(action) => action.label(),
            Self::Terrain(Terrain::Forest) => "Trees",
            Self::Terrain(Terrain::Water) => "Water",
            Self::Terrain(_) => "Land",
        }
    }
    fn cost(self) -> i64 {
        match self {
            Self::Build(k) => k.cost(),
            Self::Bulldoze => 15,
            Self::Shape(action) => action.cost(),
            Self::Terrain(Terrain::Water) => 80,
            Self::Terrain(_) => 20,
            _ => 0,
        }
    }
    fn area(self) -> bool {
        match self {
            Self::Build(k) => k.is_network() || k.is_zone(),
            Self::Bulldoze | Self::Terrain(_) | Self::Shape(_) => true,
            _ => false,
        }
    }
}
pub fn tools(category: u8) -> Vec<Tool> {
    use Kind::*;
    let mut result = match category {
        0 => vec![
            Tool::Build(Residential),
            Tool::Build(Commercial),
            Tool::Build(Industrial),
            Tool::Build(Park),
            Tool::Inspect,
            Tool::Bulldoze,
        ],
        1 => [Road, Avenue, Rail, Station, BusDepot]
            .into_iter()
            .map(Tool::Build)
            .chain([Tool::Bulldoze])
            .collect(),
        2 => [Coal, Wind, Nuclear, Pump, PowerLine, Pipe, Sewage, Landfill]
            .into_iter()
            .map(Tool::Build)
            .collect(),
        3 => [Police, Fire, Hospital, School, University, Park]
            .into_iter()
            .map(Tool::Build)
            .collect(),
        _ => vec![
            Tool::Build(Stadium),
            Tool::Build(Seaport),
            Tool::Build(Airport),
            Tool::Terrain(Terrain::Forest),
            Tool::Terrain(Terrain::Water),
            Tool::Terrain(Terrain::Grass),
        ],
    };
    result.extend(ui_model::extra_tools(category));
    result
}
pub const OVERLAYS: [&str; 20] = [
    "Natural",
    "Electricity",
    "Water",
    "Traffic",
    "Pollution",
    "Land value",
    "Crime",
    "Zoning",
    "Services",
    "Elevation",
    "Resources",
    "Sewage",
    "Garbage",
    "Health",
    "Education",
    "Residential desirability",
    "Commercial desirability",
    "Industrial desirability",
    "Tourism desirability",
    "Transit",
];

fn default_draft_seed() -> String {
    "7".into()
}
fn default_seed_replace() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(with = "wire")]
    pub city: City,
    #[serde(default)]
    pub ui: ui_model::UiState,
    pub tick: u32,
    pub clock: u32,
    pub speed: u8,
    pub tool: Tool,
    pub category: u8,
    pub cam_x: i32,
    pub cam_y: i32,
    pub zoom: i32,
    pub overlay: u8,
    pub tab: u8,
    pub hover: Option<[i32; 2]>,
    pub selected: Option<[i32; 2]>,
    pub anchor: Option<[i32; 2]>,
    pub notice: String,
    pub notice_ticks: u32,
    pub confirm_new: bool,
    #[serde(default)]
    pub panel_open: bool,
    #[serde(default)]
    pub palette_open: bool,
    #[serde(default = "default_draft_seed")]
    pub draft_seed: String,
    #[serde(default)]
    pub map_style: MapStyle,
    #[serde(default = "default_seed_replace")]
    pub seed_replace: bool,
    #[serde(default)]
    pub world_seed: Option<u64>,
    #[serde(default)]
    pub world_style: Option<MapStyle>,
}
pub struct BlueCity {
    pub state: State,
    cues: Vec<usize>,
    #[cfg(feature = "client")]
    requests: Vec<client::ClientRequest>,
}
impl BlueCity {
    /// Prepare derived route memoization during city creation/loading, outside
    /// fixed-step rules. Authoritative stocks, routes and money are unchanged.
    pub fn prepare_city_runtime(city: &mut City) {
        city.warm_infrastructure_cache();
        let mut services = std::mem::take(&mut city.ext.services);
        services.warm_routes(city);
        city.ext.services = services;
    }
    fn message(&mut self, s: impl Into<String>) {
        self.state.notice = s.into();
        self.state.notice_ticks = 360;
    }
    pub fn project(&self, x: i32, y: i32) -> Point {
        projection::project_tile(&self.state, x, y)
    }
    pub fn pick(&self, p: Point) -> Option<[i32; 2]> {
        if !Rect::new(0, 0, 800, 450).contains(p) || layout::blocked_map(&self.state, p) {
            None
        } else {
            projection::pick(&self.state, p)
        }
    }
    fn apply(&mut self, x: i32, y: i32) -> Result<i64, String> {
        match self.state.tool {
            Tool::Inspect => Ok(0),
            Tool::Build(k) => self.state.city.place(x, y, k),
            Tool::Bulldoze => self.state.city.bulldoze(x, y),
            Tool::Shape(action) => self.state.city.terraform(x, y, action),
            Tool::Terrain(t) => self.state.city.terraform(
                x,
                y,
                match t {
                    Terrain::Forest => world::TerrainAction::Plant,
                    Terrain::Water => world::TerrainAction::Water,
                    _ => world::TerrainAction::Drain,
                },
            ),
        }
    }
    fn build(&mut self, at: [i32; 2]) {
        if self.state.ui.geometry == ui_model::Geometry::BusLine {
            if let Some(i) = self.state.city.idx(at[0], at[1]) {
                if match self.state.ui.transit_mode {
                    transport::TransitMode::Bus => matches!(
                        self.state.city.tiles[i].kind,
                        Kind::BusStop | Kind::BusDepot
                    ),
                    transport::TransitMode::Rail => self.state.city.tiles[i].kind == Kind::Station,
                    transport::TransitMode::Metro => {
                        self.state.city.tiles[i].kind == Kind::SubwayStation
                    }
                    transport::TransitMode::Ferry => {
                        matches!(self.state.city.tiles[i].kind, Kind::Marina | Kind::Seaport)
                    }
                    transport::TransitMode::Freight => matches!(
                        self.state.city.tiles[i].kind,
                        Kind::FreightStation | Kind::Seaport
                    ),
                } {
                    if self.state.ui.pending_stops.last() != Some(&i) {
                        self.state.ui.pending_stops.push(i);
                    }
                    self.message(format!(
                        "{} stops selected. Open Transit lines to finish.",
                        self.state.ui.pending_stops.len()
                    ));
                } else {
                    self.message("Choose an existing bus stop or depot.");
                }
            }
            return;
        }
        self.state.selected = Some(at);
        if self.state.tool == Tool::Inspect {
            self.state.anchor = None;
            return;
        }
        if self.state.tool.area()
            && self.state.ui.geometry != ui_model::Geometry::Brush
            && self.state.anchor.is_none()
        {
            self.state.anchor = Some(at);
            self.message("Choose the other end. Click the same tile for one tile.");
            return;
        }
        let plan = placement::plan(&self.state, at);
        if let Some(reason) = plan.refusal {
            self.message(reason);
            self.cues.push(1);
            return;
        }
        self.state.anchor = None;
        let cells = plan.cells;
        let road_directions = plan.road_directions;
        let mut count = 0;
        let mut cost = 0;
        let mut last_error = String::new();
        let attempted = cells.len();
        let results = match self.state.tool {
            Tool::Build(kind) => self.state.city.edit_batch(
                &cells
                    .iter()
                    .map(|[x, y]| (*x, *y, kind))
                    .collect::<Vec<_>>(),
            ),
            Tool::Bulldoze => self.state.city.edit_batch(
                &cells
                    .iter()
                    .map(|[x, y]| (*x, *y, Kind::Empty))
                    .collect::<Vec<_>>(),
            ),
            _ => {
                let results = cells.into_iter().map(|[x, y]| self.apply(x, y)).collect();
                self.state.city.refresh();
                results
            }
        };
        for result in results {
            match result {
                Ok(c) => {
                    count += 1;
                    cost += c;
                }
                Err(e) => last_error = e,
            }
        }
        if count > 0 {
            for ([x, y], direction) in road_directions {
                if self
                    .state
                    .city
                    .tile(x, y)
                    .is_some_and(|t| placement::directional_road(t.kind))
                {
                    if let Err(error) = self.state.city.set_road_direction(x, y, direction) {
                        self.message(error);
                        self.cues.push(1);
                        return;
                    }
                }
            }
            self.message(format!(
                "{}: {}/{} tiles built for ${}.",
                self.state.tool.label(),
                count,
                attempted,
                cost
            ));
            self.cues.push(0);
        } else {
            self.message(last_error);
            self.cues.push(1);
        }
    }
    fn click(&mut self, p: Point) {
        for (rect, full) in [
            (Rect::new(80, 100, 640, 296), true),
            (Rect::new(12, 100, 132, 104), false),
        ] {
            if ((full && self.state.ui.full_map) || (!full && self.state.ui.minimap))
                && rect.contains(p)
            {
                self.state.cam_x = ((p.x - rect.x) * i32::from(self.state.city.width) / rect.w)
                    .clamp(0, i32::from(self.state.city.width) - 1);
                self.state.cam_y = ((p.y - rect.y) * i32::from(self.state.city.height) / rect.h)
                    .clamp(0, i32::from(self.state.city.height) - 1);
                return;
            }
        }
        use layout::Action;
        if let Some(action) = layout::hit(&self.state, p) {
            match action {
                Action::Speed(speed) => self.state.speed = speed,
                Action::Zoom(delta) => self.state.zoom = (self.state.zoom + delta).clamp(1, 3),
                Action::Center => {
                    self.state.cam_x = i32::from(self.state.city.width) / 2;
                    self.state.cam_y = i32::from(self.state.city.height) / 2;
                }
                Action::Tab(tab) => {
                    self.state.panel_open = !self.state.panel_open || self.state.tab != tab;
                    self.state.tab = tab;
                    self.state.ui.window = None;
                    self.state.palette_open = false;
                    self.state.confirm_new = false;
                    self.state.anchor = None;
                }
                Action::Category(category) => {
                    self.state.palette_open =
                        !self.state.palette_open || self.state.category != category;
                    self.state.category = category;
                    self.state.ui.palette_page = 0;
                    self.state.panel_open = false;
                    self.state.confirm_new = false;
                    self.state.anchor = None;
                }
                Action::SelectTool(tool) => {
                    self.state.tool = tool;
                    self.state.anchor = None;
                    self.state.palette_open = false;
                    self.state.panel_open = false;
                    self.state.confirm_new = false;
                    let reason = if let Tool::Build(k) = tool {
                        self.state
                            .city
                            .civic
                            .can_build(k, self.state.city.population)
                            .err()
                    } else {
                        None
                    };
                    self.message(reason.unwrap_or_else(|| {
                        format!("{} selected. ${} per tile.", tool.label(), tool.cost())
                    }));
                }
                Action::ClosePanel => {
                    self.state.panel_open = false;
                    self.state.confirm_new = false;
                }
                Action::CityOverview => self.state.selected = None,
                Action::Overlay(overlay) => self.state.overlay = overlay,
                Action::Tax { row, delta } => {
                    self.state.city.taxes[row] =
                        (self.state.city.taxes[row] as i32 + delta).clamp(0, 20) as u8;
                    self.state.city.refresh();
                }
                Action::Funding { row, delta } => {
                    self.state.city.funding[row] =
                        (self.state.city.funding[row] as i32 + delta).clamp(0, 150) as u8;
                    self.state.city.refresh();
                }
                Action::Borrow => {
                    let result = self.state.city.take_loan();
                    self.message(
                        result
                            .map(|_| "Loan received: $10,000.".to_string())
                            .unwrap_or_else(|e| e),
                    );
                }
                Action::Repay => {
                    let result = self.state.city.repay_loan();
                    self.message(
                        result
                            .map(|_| "Loan repaid.".to_string())
                            .unwrap_or_else(|e| e),
                    );
                }
                Action::Policy(row) => self.command(ui_model::Command::Ordinance(row)),
                Action::Disaster(Disaster::Flood) => self.command(ui_model::Command::Disaster(1)),
                Action::Disaster(Disaster::Earthquake) => {
                    self.command(ui_model::Command::Disaster(0))
                }
                Action::Disaster(Disaster::Tornado) => self.command(ui_model::Command::Disaster(3)),
                Action::Disaster(Disaster::Meltdown) => {
                    self.command(ui_model::Command::Disaster(9))
                }
                Action::Disaster(disaster) => {
                    self.state.city.disaster(disaster);
                    self.message(
                        self.state
                            .city
                            .messages
                            .last()
                            .cloned()
                            .unwrap_or_else(|| "Emergency services are responding.".into()),
                    );
                    self.cues.push(1);
                }
                Action::NewCity => {
                    self.state.confirm_new = true;
                    self.state.panel_open = true;
                    self.state.palette_open = false;
                    self.state.draft_seed = self
                        .state
                        .world_seed
                        .unwrap_or(self.state.city.seed)
                        .wrapping_add(1)
                        .to_string();
                    self.state.seed_replace = true;
                }
                Action::CancelNew => self.state.confirm_new = false,
                Action::Extra(command) => self.command(command),
                Action::MapStyle(style) => self.state.map_style = style,
                Action::SelectSeed => self.state.seed_replace = true,
                Action::ClearSeed => {
                    self.state.draft_seed.clear();
                    self.state.seed_replace = false;
                }
                Action::BackspaceSeed => {
                    self.state.draft_seed.pop();
                    self.state.seed_replace = false;
                }
                Action::SeedDigit(digit) => {
                    if self.state.seed_replace {
                        self.state.draft_seed.clear();
                        self.state.seed_replace = false;
                    }
                    if digit <= 9 && self.state.draft_seed.len() < 20 {
                        self.state.draft_seed.push((b'0' + digit) as char);
                    }
                }
                Action::ShuffleSeed => {
                    let seed = StateHasher::new()
                        .u64(self.state.city.seed)
                        .u32(self.state.tick)
                        .bytes(self.state.draft_seed.as_bytes())
                        .finish();
                    self.state.draft_seed = seed.to_string();
                    self.state.seed_replace = true;
                }
                Action::ConfirmNew => {
                    let Ok(seed) = self.state.draft_seed.parse::<u64>() else {
                        self.message("Enter a whole-number seed from 0 to 18446744073709551615.");
                        return;
                    };
                    let style = self.state.map_style;
                    self.state.city = City::world(seed, style, self.state.ui.climate);
                    Self::prepare_city_runtime(&mut self.state.city);
                    self.state.clock = 0;
                    self.state.ui.window = None;
                    self.state.ui.page = 0;
                    self.state.ui.card = None;
                    self.state.ui.pending_stops.clear();
                    self.state.ui.selected_line = 0;
                    self.state.ui.bankruptcy_month = None;
                    self.state.ui.full_map = false;
                    self.state.ui.underground = false;
                    self.state.world_seed = Some(seed);
                    self.state.world_style = Some(style);
                    self.state.confirm_new = false;
                    self.state.panel_open = false;
                    self.state.palette_open = false;
                    self.state.anchor = None;
                    self.state.selected = None;
                    self.state.hover = None;
                    self.state.speed = 0;
                    self.state.cam_x = i32::from(self.state.city.width) / 2;
                    self.state.cam_y = i32::from(self.state.city.height) / 2;
                    self.state.overlay = 0;
                    self.message(format!("{} map · Seed {}. Connect a road to the map edge, then add power, water and zones.",style.label(),seed));
                }
            }
            return;
        }
        if self.state.ui.photo {
            return;
        }
        if layout::blocked_map(&self.state, p) {
            return;
        }
        if let Some(at) = self.pick(p) {
            self.build(at);
        }
    }
}
impl GameLogic for BlueCity {
    const ID: &'static str = "bluecity";
    const TITLE: &'static str = "BlueCity";
    const CONTROLS: &'static str = "Click to build | WASD/arrows pan | K save | L load | Esc menu";
    const VERIFY_TICKS: u32 = 1440;
    fn new(seed: u64) -> Self {
        Self{state:State{city:{let mut city=City::new(seed,true);city.initialize_world();city.civic=civic::CivicState::new(seed);Self::prepare_city_runtime(&mut city);city},ui:ui_model::UiState::default(),tick:0,clock:0,speed:1,tool:Tool::Inspect,category:0,cam_x:24,cam_y:24,zoom:2,overlay:0,tab:0,hover:None,selected:None,anchor:None,notice:"Welcome, Mayor. Your town is ready to grow. Add roads and zone new neighborhoods.".into(),notice_ticks:600,confirm_new:false,panel_open:false,palette_open:false,draft_seed:default_draft_seed(),map_style:MapStyle::default(),seed_replace:true,world_seed:None,world_style:None},cues:vec![],#[cfg(feature="client")] requests:vec![]}
    }
    fn pointer_target_only_on_press() -> bool {
        true
    }
    fn tick(&self) -> u32 {
        self.state.tick
    }
    fn outcome(&self) -> &'static str {
        "playing"
    }
    fn verification_input(t: u32) -> Intent {
        let p = match t {
            10 => Some(Point::new(58, 423)),
            12 => {
                let r = layout::palette_tool(0, 3);
                Some(Point::new(r.x + r.w / 2, r.y + r.h / 2))
            }
            14 => Some(Point::new(388, 219)),
            16 => Some(Point::new(534, 423)),
            _ => None,
        };
        Intent {
            pointer: p,
            action: p.is_some(),
            ..Default::default()
        }
    }
    fn probe_input() -> Intent {
        Intent {
            pointer: Some(Point::new(708, 423)),
            action: true,
            ..Default::default()
        }
    }
    fn probe_success(&self) -> bool {
        self.state.zoom > 2
    }
    fn take_cues(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.cues)
    }
    fn audio_banks() -> &'static [AudioBankSpec] {
        &[
            AudioBankSpec {
                id: "riverlight",
                root: "assets/audio/riverlight",
                music: true,
            },
            AudioBankSpec {
                id: "city",
                root: "assets/audio/city-ambience",
                music: false,
            },
        ]
    }
    fn audio_level(&self, bank: &str, layer: &str) -> f32 {
        if bank == "city" {
            let rain = self.state.city.ext.weather.rain > 35;
            let night = self.state.tick % 7200 > 4500;
            match layer {
                "rain" => {
                    if rain {
                        0.65
                    } else {
                        0.
                    }
                }
                "night" => {
                    if night {
                        0.6
                    } else {
                        0.
                    }
                }
                "day" if !night => {
                    (0.3 + self.state.city.population.min(20000) as f32 / 40000.).min(0.8)
                }
                _ => 0.,
            }
        } else if self.state.city.happiness < 35 {
            0.3
        } else {
            0.45
        }
    }
}
impl Simulation for BlueCity {
    type Input = Intent;
    fn step(&mut self, input: &Intent) {
        self.state.tick = self.state.tick.wrapping_add(1);
        self.state.notice_ticks = self.state.notice_ticks.saturating_sub(1);
        if self.state.tick.is_multiple_of(6) {
            self.state.cam_x = (self.state.cam_x + input.x.clamp(-1, 1) + input.y.clamp(-1, 1))
                .clamp(0, i32::from(self.state.city.width) - 1);
            self.state.cam_y = (self.state.cam_y - input.x.clamp(-1, 1) + input.y.clamp(-1, 1))
                .clamp(0, i32::from(self.state.city.height) - 1);
        }
        self.state.hover = input.pointer.and_then(|p| self.pick(p));
        let mut synthetic = false;
        if input.action {
            if let Some(p) = input.pointer {
                if (-1116..=-1100).contains(&p.x) {
                    synthetic = true;
                    let target = (-1100 - p.x) as usize;
                    if let Some(name) = ui_model::KEY_NAMES.get(p.y as usize) {
                        let old = self.state.ui.preferences.keys[target].clone();
                        if let Some(other) = self
                            .state
                            .ui
                            .preferences
                            .keys
                            .iter()
                            .position(|k| k == name)
                        {
                            self.state.ui.preferences.keys[other] = old;
                        }
                        self.state.ui.preferences.keys[target] = name.to_string();
                        self.state.ui.rebind = None;
                    }
                }
                if (-2116..=-2100).contains(&p.x) {
                    synthetic = true;
                    let i = (-2100 - p.x) as usize;
                    match ui_model::Hotkey::ALL[i] {
                        ui_model::Hotkey::ZoomIn => self.state.zoom = (self.state.zoom + 1).min(3),
                        ui_model::Hotkey::ZoomOut => self.state.zoom = (self.state.zoom - 1).max(1),
                        ui_model::Hotkey::Center => {
                            self.state.cam_x = i32::from(self.state.city.width) / 2;
                            self.state.cam_y = i32::from(self.state.city.height) / 2;
                        }
                        ui_model::Hotkey::Inspect => {
                            self.state.tool = Tool::Inspect;
                            self.state.anchor = None;
                        }
                        ui_model::Hotkey::Bulldoze => {
                            self.state.tool = Tool::Bulldoze;
                            self.state.anchor = None;
                        }
                        _ => {
                            if let Some(cmd) = ui_model::shortcut(&mut self.state, i) {
                                self.command(cmd);
                            }
                        }
                    }
                }
            }
        }
        if input.action && !synthetic {
            if let Some(p) = input.pointer {
                self.click(p);
            } else if let Some(p) = self.state.selected {
                self.build(p);
            } else {
                self.build([self.state.cam_x, self.state.cam_y]);
            }
        }
        let before_month = self.state.city.month;
        let before_milestones = self.state.city.civic.milestone_bits;
        let rate = match self.state.speed {
            0 => 0,
            1 => 1,
            2 => 3,
            _ => 10,
        };
        self.state.clock = (self.state.clock + rate) % 240;
        self.state.city.advance_time((rate * 180) as u16);
        if self.state.ui.window.is_some() {
            self.state.ui.page = self
                .state
                .ui
                .page
                .min(ui_model::page_count(&self.state).saturating_sub(1));
        }
        if self.state.city.money >= 0 {
            self.state.ui.bankruptcy_month = None;
        }
        let mut card = None;
        if self.state.city.civic.milestone_bits != before_milestones {
            self.cues.push(2);
            #[cfg(feature = "client")]
            self.requests.push(client::ClientRequest::Postcard);
        }
        if self.state.city.money < -5000
            && self.state.ui.card.is_none()
            && self.state.ui.bankruptcy_month != Some(self.state.city.month)
        {
            self.state.ui.bankruptcy_month = Some(self.state.city.month);
            card = Some((
                "Financial emergency".into(),
                civic::financial_emergency_text(&self.state.city),
            ));
        }
        if self.state.city.month != before_month
            && self.state.city.month.is_multiple_of(3)
            && card.is_none()
        {
            let mut civic = std::mem::take(&mut self.state.city.civic);
            if let Some(hint) = civic.next_hint(&self.state.city) {
                card = Some(("Mayor's hint".into(), hint.into()));
            }
            self.state.city.civic = civic;
        }
        if let Some((title, body)) = card {
            self.state.ui.card = Some(ui_model::Card {
                id: self.state.tick,
                title,
                body,
            });
        }
    }
    fn state_hash(&self) -> u64 {
        wire::state_hash(&self.state)
    }
}
impl Snapshot for BlueCity {
    const KIND: &'static str = "bluecity";
    const VERSION: u32 = 3;
    // Version 1 is released. Keep this migration immutable: future state
    // changes add another step rather than changing how old towns are loaded.
    const MIGRATIONS: &'static [vesper3d::runtime::snapshot::Migration] = &[
        vesper3d::runtime::snapshot::Migration {
            from: 1,
            step: |mut state| {
                let fields = state
                    .as_object_mut()
                    .ok_or("Version 1 BlueCity state must be an object.")?;
                fields.insert("panel_open".into(), false.into());
                fields.insert("palette_open".into(), false.into());
                fields.insert("draft_seed".into(), "7".into());
                fields.insert("map_style".into(), "River".into());
                fields.insert("seed_replace".into(), true.into());
                fields.insert("world_seed".into(), serde_json::Value::Null);
                fields.insert("world_style".into(), serde_json::Value::Null);
                Ok(state)
            },
        },
        vesper3d::runtime::snapshot::Migration {
            from: 2,
            step: |mut state| {
                let fields = state
                    .as_object_mut()
                    .ok_or("Version 2 BlueCity state must be an object.")?;
                fields.insert(
                    "ui".into(),
                    serde_json::to_value(ui_model::UiState::default())
                        .map_err(|e| e.to_string())?,
                );
                let old_clock = fields
                    .get("clock")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let recipe_seed = fields.get("world_seed").and_then(serde_json::Value::as_u64);
                let recipe_style = fields
                    .get("world_style")
                    .filter(|v| !v.is_null())
                    .cloned()
                    .map(serde_json::from_value::<MapStyle>)
                    .transpose()
                    .map_err(|e| e.to_string())?;
                let city = fields
                    .get_mut("city")
                    .and_then(serde_json::Value::as_object_mut)
                    .ok_or("Missing legacy city.")?;
                city.insert("width".into(), 48.into());
                city.insert("height".into(), 48.into());
                let mut legacy: City =
                    serde_json::from_value(serde_json::Value::Object(city.clone()))
                        .map_err(|e| e.to_string())?;
                legacy.initialize_world();
                for cell in &mut legacy.ext.cells {
                    cell.elevation = 0;
                }
                legacy.ext.seed = recipe_seed.unwrap_or(legacy.seed);
                legacy.ext.style = recipe_style.unwrap_or_default();
                legacy.ext.day = u64::from(legacy.month) * 30 + old_clock / 8;
                legacy.ext.minute_of_day = ((old_clock % 8) * 180) as u16;
                legacy.civic = civic::CivicState::new(legacy.seed);
                legacy.civic.day = legacy.ext.day as u32;
                for (i, enabled) in legacy.policies.iter().enumerate() {
                    if *enabled {
                        legacy.civic.ordinances |= 1 << i;
                    }
                }
                legacy.civic.ordinance_slots = legacy
                    .civic
                    .ordinance_slots
                    .max(legacy.civic.ordinances.count_ones() as u8);
                *city = serde_json::to_value(legacy)
                    .map_err(|e| e.to_string())?
                    .as_object()
                    .ok_or("Legacy city conversion failed")?
                    .clone();
                Ok(state)
            },
        },
    ];
    type State = State;
    fn capture(&self) -> State {
        self.state.clone()
    }
    fn restore(&mut self, state: State) -> Result<(), String> {
        state.city.validate()?;
        if state.speed > 3
            || state.zoom < 1
            || state.zoom > 3
            || state.overlay > 19
            || !state.ui.preferences.validate()
            || state
                .ui
                .card
                .as_ref()
                .is_some_and(|c| c.title.len() > 120 || c.body.len() > 1200)
            || (state.ui.window.is_some() && state.ui.page >= ui_model::page_count(&state))
            || state.ui.graph >= 16
            || state.ui.service >= 13
            || state.ui.selected_line >= 128
            || state.ui.palette_page >= tools(state.category).len().div_ceil(8)
            || state.ui.pending_stops.len() > 64
            || state
                .ui
                .pending_stops
                .iter()
                .any(|i| *i >= state.city.tiles.len())
            || state
                .ui
                .rebind
                .is_some_and(|i| i >= ui_model::Hotkey::ALL.len())
            || state.tab > 3
            || state.category > 4
            || state.clock >= 240
            || state.draft_seed.len() > 20
            || !state.draft_seed.bytes().all(|b| b.is_ascii_digit())
            || state.world_seed.is_some() != state.world_style.is_some()
            || state.cam_x < 0
            || state.cam_y < 0
            || state.cam_x >= i32::from(state.city.width)
            || state.cam_y >= i32::from(state.city.height)
            || [state.hover, state.selected, state.anchor]
                .into_iter()
                .flatten()
                .any(|[x, y]| {
                    x < 0
                        || y < 0
                        || x >= i32::from(state.city.width)
                        || y >= i32::from(state.city.height)
                })
        {
            return Err("Invalid city view in save.".into());
        }
        self.state = state;
        Self::prepare_city_runtime(&mut self.state.city);
        self.cues.clear();
        Ok(())
    }
}
#[cfg(feature = "client")]
mod interface;

#[cfg(test)]
mod tests {
    use super::*;
    use vesper3d::runtime::{assert_deterministic, snapshot};
    #[test]
    fn city_replays_and_saves_exactly() {
        let inputs: Vec<_> = (0..BlueCity::VERIFY_TICKS)
            .map(BlueCity::verification_input)
            .collect();
        assert_deterministic(|| BlueCity::new(7), &inputs);
        snapshot::assert_resumes_exactly(|| BlueCity::new(7), &inputs, 651);
        let mut city = BlueCity::new(7);
        let initial = city.state.city.population;
        for input in &inputs {
            city.step(input);
        }
        assert_eq!(city.state.city.tile(23, 24).unwrap().kind, Kind::Park);
        assert!(city.state.city.month >= 6);
        assert!(
            city.state.city.population > initial,
            "starter town must grow"
        );
        let (hash, outcome) = verify::<BlueCity>();
        assert_eq!(outcome, "playing");
        if let Ok(path) = std::env::var("BE2_VERIFY_REPORT") {
            std::fs::write(path,serde_json::json!({"hash":format!("{hash:016x}"),"outcome":outcome,"ticks":BlueCity::VERIFY_TICKS,"purpose":"Grow a connected starter town through six monthly budgets and construct a park using public pointer input."}).to_string()).unwrap();
        }
    }
    #[test]
    fn pointer_selects_tools_and_two_click_zoning() {
        let mut g = BlueCity::new(7);
        g.state.speed = 0;
        g.click(Point::new(58, 423));
        g.click(Point::new(80, 374));
        assert_eq!(g.state.tool, Tool::Build(Kind::Residential));
        let target = (15..35)
            .flat_map(|y| (15..35).map(move |x| [x, y]))
            .find(|[x, y]| {
                g.state
                    .city
                    .tile(*x, *y)
                    .map(|t| t.kind == Kind::Empty && t.terrain == Terrain::Grass)
                    .unwrap_or(false)
            })
            .unwrap();
        let p = g.project(target[0], target[1]);
        let p = Point::new(p.x, p.y + 3);
        assert!(Rect::new(0, 0, 800, 450).contains(p));
        assert_eq!(g.pick(p), Some(target));
        g.click(p);
        assert_eq!(g.state.anchor, Some(target));
        g.click(p);
        assert_eq!(
            g.state.city.tile(target[0], target[1]).unwrap().kind,
            Kind::Residential
        );
    }
    #[test]
    fn pause_budget_and_invalid_save() {
        let mut g = BlueCity::new(7);
        g.click(Point::new(652, 33));
        let month = g.state.city.month;
        for _ in 0..300 {
            g.step(&Intent::default());
        }
        assert_eq!(month, g.state.city.month);
        g.click(Point::new(683, 78));
        let tax = g.state.city.taxes[0];
        g.click(Point::new(763, 167));
        assert_eq!(g.state.city.taxes[0], tax + 1);
        let mut save = g.capture();
        save.zoom = 0;
        assert!(g.restore(save).is_err());
    }
}
