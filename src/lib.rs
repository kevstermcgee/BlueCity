//! BlueCity: fixed-step city rules and a read-only portable presentation.
pub mod city;
pub mod layout;
#[cfg(feature = "client")]
mod render;
use city::{City, Disaster, Kind, MapStyle, Terrain, SIZE};
use serde::{Deserialize, Serialize};
use vesper3d::two_d::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tool {
    Inspect,
    Build(Kind),
    Bulldoze,
    Terrain(Terrain),
}
impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inspect => "Inspect",
            Self::Build(k) => k.label(),
            Self::Bulldoze => "Bulldoze",
            Self::Terrain(Terrain::Forest) => "Trees",
            Self::Terrain(Terrain::Water) => "Water",
            Self::Terrain(_) => "Land",
        }
    }
    fn cost(self) -> i64 {
        match self {
            Self::Build(k) => k.cost(),
            Self::Bulldoze => 15,
            Self::Terrain(_) => 20,
            _ => 0,
        }
    }
    fn area(self) -> bool {
        match self {
            Self::Build(k) => matches!(
                k,
                Kind::Road
                    | Kind::Avenue
                    | Kind::Rail
                    | Kind::PowerLine
                    | Kind::Pipe
                    | Kind::Residential
                    | Kind::Commercial
                    | Kind::Industrial
            ),
            Self::Bulldoze | Self::Terrain(_) => true,
            _ => false,
        }
    }
}
pub fn tools(category: u8) -> Vec<Tool> {
    use Kind::*;
    match category {
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
    }
}
pub const OVERLAYS: [&str; 9] = [
    "Natural",
    "Electricity",
    "Water",
    "Traffic",
    "Pollution",
    "Land value",
    "Crime",
    "Zoning",
    "Services",
];

fn default_draft_seed() -> String {
    "7".into()
}
fn default_seed_replace() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub city: City,
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
}
impl BlueCity {
    fn message(&mut self, s: impl Into<String>) {
        self.state.notice = s.into();
        self.state.notice_ticks = 360;
    }
    pub fn project(&self, x: i32, y: i32) -> Point {
        let (w, h) = match self.state.zoom {
            1 => (8, 4),
            2 => (12, 6),
            _ => (18, 9),
        };
        let dx = x - self.state.cam_x;
        let dy = y - self.state.cam_y;
        Point::new(400 + (dx - dy) * w, 225 + (dx + dy) * h)
    }
    pub fn pick(&self, p: Point) -> Option<[i32; 2]> {
        if !Rect::new(0, 0, 800, 450).contains(p) || layout::blocked_map(&self.state, p) {
            return None;
        }
        let (w, h) = match self.state.zoom {
            1 => (8., 4.),
            2 => (12., 6.),
            _ => (18., 9.),
        };
        let a = (p.x - 400) as f32 / w;
        let b = (p.y - 225) as f32 / h;
        let x = ((a + b) / 2. + 0.5).floor() as i32 + self.state.cam_x;
        let y = ((b - a) / 2. + 0.5).floor() as i32 + self.state.cam_y;
        if x >= 0 && y >= 0 && x < SIZE as i32 && y < SIZE as i32 {
            Some([x, y])
        } else {
            None
        }
    }
    fn apply(&mut self, x: i32, y: i32) -> Result<i64, String> {
        match self.state.tool {
            Tool::Inspect => Ok(0),
            Tool::Build(k) => self.state.city.place(x, y, k),
            Tool::Bulldoze => self.state.city.bulldoze(x, y),
            Tool::Terrain(t) => {
                let idx = y as usize * SIZE + x as usize;
                if self.state.city.tiles[idx].kind != Kind::Empty {
                    return Err("Bulldoze the building before shaping terrain.".into());
                }
                if self.state.city.money < 20 {
                    return Err("Not enough funds for landscaping.".into());
                }
                self.state.city.tiles[idx].terrain = t;
                self.state.city.money -= 20;
                Ok(20)
            }
        }
    }
    fn build(&mut self, at: [i32; 2]) {
        self.state.selected = Some(at);
        if self.state.tool == Tool::Inspect {
            self.state.anchor = None;
            return;
        }
        if self.state.tool.area() && self.state.anchor.is_none() {
            self.state.anchor = Some(at);
            self.message("Choose the other end. Click the same tile for one tile.");
            return;
        }
        let start = self.state.anchor.take().unwrap_or(at);
        let line = matches!(
            self.state.tool,
            Tool::Build(Kind::Road | Kind::Avenue | Kind::Rail | Kind::PowerLine | Kind::Pipe)
        );
        let mut cells = vec![];
        if line {
            for x in start[0].min(at[0])..=start[0].max(at[0]) {
                cells.push([x, start[1]]);
            }
            for y in start[1].min(at[1])..=start[1].max(at[1]) {
                if y != start[1] {
                    cells.push([at[0], y]);
                }
            }
        } else {
            for y in start[1].min(at[1])..=start[1].max(at[1]) {
                for x in start[0].min(at[0])..=start[0].max(at[0]) {
                    cells.push([x, y]);
                }
            }
        }
        if cells.len() > 256 {
            self.message("Select up to 256 tiles at once.");
            return;
        }
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
        use layout::Action;
        if let Some(action) = layout::hit(&self.state, p) {
            match action {
                Action::Speed(speed) => self.state.speed = speed,
                Action::Zoom(delta) => self.state.zoom = (self.state.zoom + delta).clamp(1, 3),
                Action::Center => {
                    self.state.cam_x = 24;
                    self.state.cam_y = 24;
                }
                Action::Tab(tab) => {
                    self.state.panel_open = !self.state.panel_open || self.state.tab != tab;
                    self.state.tab = tab;
                    self.state.palette_open = false;
                    self.state.confirm_new = false;
                    self.state.anchor = None;
                }
                Action::Category(category) => {
                    self.state.palette_open =
                        !self.state.palette_open || self.state.category != category;
                    self.state.category = category;
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
                    self.message(format!(
                        "{} selected. ${} per tile.",
                        tool.label(),
                        tool.cost()
                    ));
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
                Action::Policy(row) => {
                    self.state.city.policies[row] = !self.state.city.policies[row];
                    self.state.city.refresh();
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
                    self.state.city = City::generated(seed, style);
                    self.state.world_seed = Some(seed);
                    self.state.world_style = Some(style);
                    self.state.confirm_new = false;
                    self.state.panel_open = false;
                    self.state.palette_open = false;
                    self.state.anchor = None;
                    self.state.selected = None;
                    self.state.hover = None;
                    self.state.speed = 0;
                    self.state.cam_x = 24;
                    self.state.cam_y = 24;
                    self.state.overlay = 0;
                    self.message(format!("{} map · Seed {}. Connect a road to the map edge, then add power, water and zones.",style.label(),seed));
                }
            }
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
        Self{state:State{city:City::new(seed,true),tick:0,clock:0,speed:1,tool:Tool::Inspect,category:0,cam_x:24,cam_y:24,zoom:2,overlay:0,tab:0,hover:None,selected:None,anchor:None,notice:"Welcome, Mayor. Your town is ready to grow. Add roads and zone new neighborhoods.".into(),notice_ticks:600,confirm_new:false,panel_open:false,palette_open:false,draft_seed:default_draft_seed(),map_style:MapStyle::default(),seed_replace:true,world_seed:None,world_style:None},cues:vec![]}
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
            12 => Some(Point::new(460, 374)),
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
        &[AudioBankSpec {
            id: "riverlight",
            root: "assets/audio/riverlight",
            music: true,
        }]
    }
    fn audio_level(&self, _bank: &str, _layer: &str) -> f32 {
        if self.state.city.happiness < 35 {
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
                .clamp(0, SIZE as i32 - 1);
            self.state.cam_y = (self.state.cam_y - input.x.clamp(-1, 1) + input.y.clamp(-1, 1))
                .clamp(0, SIZE as i32 - 1);
        }
        self.state.hover = input.pointer.and_then(|p| self.pick(p));
        if input.action {
            if let Some(p) = input.pointer {
                self.click(p);
            } else if let Some(p) = self.state.selected {
                self.build(p);
            } else {
                self.build([self.state.cam_x, self.state.cam_y]);
            }
        }
        self.state.clock += match self.state.speed {
            0 => 0,
            1 => 1,
            2 => 3,
            _ => 10,
        };
        if self.state.clock >= 240 {
            self.state.clock -= 240;
            let before = self.state.city.population;
            self.state.city.advance_month();
            if (before / 500) < self.state.city.population / 500 {
                self.cues.push(2);
                self.message(format!(
                    "Milestone: {} residents now call BlueCity home!",
                    self.state.city.population
                ));
            }
        }
    }
    fn state_hash(&self) -> u64 {
        hash_json(&self.state)
    }
}
impl Snapshot for BlueCity {
    const KIND: &'static str = "bluecity";
    const VERSION: u32 = 2;
    // Version 1 is released. Keep this migration immutable: future state
    // changes add another step rather than changing how old towns are loaded.
    const MIGRATIONS: &'static [vesper3d::runtime::snapshot::Migration] =
        &[vesper3d::runtime::snapshot::Migration {
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
        }];
    type State = State;
    fn capture(&self) -> State {
        self.state.clone()
    }
    fn restore(&mut self, state: State) -> Result<(), String> {
        state.city.validate()?;
        if state.speed > 3
            || state.zoom < 1
            || state.zoom > 3
            || state.overlay > 8
            || state.tab > 3
            || state.category > 4
            || state.clock >= 240
            || state.draft_seed.len() > 20
            || !state.draft_seed.bytes().all(|b| b.is_ascii_digit())
            || state.world_seed.is_some() != state.world_style.is_some()
            || state.cam_x < 0
            || state.cam_y < 0
            || state.cam_x >= SIZE as i32
            || state.cam_y >= SIZE as i32
            || [state.hover, state.selected, state.anchor]
                .into_iter()
                .flatten()
                .any(|[x, y]| x < 0 || y < 0 || x >= SIZE as i32 || y >= SIZE as i32)
        {
            return Err("Invalid city view in save.".into());
        }
        self.state = state;
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
