//! Fixed-step command dispatch for the floating city windows.
use crate::{
    city::Kind,
    ui_model::{Command, Geometry},
    BlueCity, Tool,
};
impl BlueCity {
    pub fn command(&mut self, cmd: Command) {
        let mut result: Result<String, String> = Ok(String::new());
        match cmd {
            Command::RotateRoad => {
                result = self
                    .state
                    .selected
                    .ok_or_else(|| "Inspect a one-way street first.".to_owned())
                    .and_then(|[x, y]| {
                        let i = self.state.city.idx(x, y).ok_or("Outside the region.")?;
                        if !crate::placement::directional_road(self.state.city.tiles[i].kind) {
                            return Err("Inspect a one-way street first.".into());
                        }
                        let direction = (self
                            .state
                            .city
                            .ext
                            .cells
                            .get(i)
                            .map_or(0, |c| c.orientation)
                            + 1)
                            % 4;
                        self.state.city.set_road_direction(x, y, direction)?;
                        Ok(format!(
                            "One-way street now points {}. Rotation is free.",
                            crate::placement::direction_label(direction)
                        ))
                    });
            }
            Command::Dismiss => {
                if self.state.ui.card.is_some() {
                    self.state.ui.card = None;
                } else if let Some(id) = self.state.city.civic.popups.last().map(|p| p.id) {
                    result = self
                        .state
                        .city
                        .civic
                        .dismiss_popup(id)
                        .map(|_| String::new());
                }
            }
            Command::NeighborToggle(index)
            | Command::NeighborPower(index, _)
            | Command::NeighborGoods(index, _)
            | Command::NeighborTariff(index, _) => {
                result = if let Some(contract) = self.state.city.ext.neighbors.get(index).cloned() {
                    let mut enabled = contract.enabled;
                    let mut power = contract.power;
                    let mut goods = contract.goods;
                    let mut tariff = contract.tariff;
                    match cmd {
                        Command::NeighborToggle(_) => enabled = !enabled,
                        Command::NeighborPower(_, delta) => {
                            power = (i64::from(power) + i64::from(delta)).clamp(0, 100000) as u32
                        }
                        Command::NeighborGoods(_, delta) => {
                            goods = (i64::from(goods) + i64::from(delta)).clamp(0, 50000) as u32
                        }
                        Command::NeighborTariff(_, delta) => {
                            tariff = (i32::from(tariff) + delta).clamp(0, 25) as u16
                        }
                        _ => {}
                    }
                    self.state.city.set_neighbor_contract(index,enabled,power,goods,tariff).map(|_|"Neighbor agreement updated. Actual purchases and freight settle through connected routes.".into())
                } else {
                    Err("Unknown neighboring region.".into())
                };
            }
            Command::Details(details) => {
                result = crate::civic_ui::details(&self.state, details).map(|(title, body)| {
                    self.state.ui.card = Some(crate::ui_model::Card {
                        id: self.state.tick.wrapping_add(1),
                        title,
                        body,
                    });
                    String::new()
                });
            }
            Command::Repair(tile) => {
                let mut service = std::mem::take(&mut self.state.city.ext.services);
                result = service.queue_repair(&self.state.city, tile).map(|_| {
                    "Repair crew requested. Payment is due when the crew reaches the site.".into()
                });
                self.state.city.ext.services = service;
            }
            Command::CancelRepair(tile) => {
                let mut service = std::mem::take(&mut self.state.city.ext.services);
                result = service
                    .cancel_repair(&mut self.state.city, tile)
                    .map(|_| "Repair request cancelled.".into());
                self.state.city.ext.services = service;
            }
            Command::Relief
            | Command::RepayBond(_)
            | Command::RepayLoan
            | Command::BondAmount(_)
            | Command::Bankruptcy
            | Command::DistrictPolicy(_)
            | Command::RenameDistrict(_)
            | Command::RemoveDistrict(_)
            | Command::RemoveHeritage(_)
            | Command::CancelProject(_) => {
                let mut civic = std::mem::take(&mut self.state.city.civic);
                let action = match cmd {
                    Command::Relief => civic.request_relief(&mut self.state.city),
                    Command::RepayBond(i) => civic.repay_bond(&mut self.state.city, i),
                    Command::RepayLoan => self.state.city.repay_loan(),
                    Command::BondAmount(amount) => civic.issue_bond(&mut self.state.city, amount),
                    Command::Bankruptcy => civic.declare_bankruptcy(&mut self.state.city),
                    Command::DistrictPolicy(id) => {
                        let next = civic
                            .districts
                            .iter()
                            .find(|d| d.id == id)
                            .map(|d| (d.specialization + 1) % 5);
                        next.ok_or_else(|| "Unknown district.".to_string())
                            .and_then(|next| {
                                civic.set_district_specialization(&mut self.state.city, id, next)
                            })
                    }
                    Command::RenameDistrict(id) => {
                        let name = civic.districts.iter().find(|d| d.id == id).map(|d| {
                            let i = crate::civic::DISTRICT_NAMES
                                .iter()
                                .position(|&name| name == d.name)
                                .map(|i| (i + 1) % crate::civic::DISTRICT_NAMES.len())
                                .unwrap_or(
                                    id.saturating_sub(1) as usize
                                        % crate::civic::DISTRICT_NAMES.len(),
                                );
                            crate::civic::DISTRICT_NAMES[i].to_owned()
                        });
                        name.ok_or_else(|| "Unknown district.".to_string())
                            .and_then(|name| civic.rename_district(id, name))
                    }
                    Command::RemoveDistrict(id) => civic.remove_district(id),
                    Command::RemoveHeritage(tile) => {
                        civic.remove_heritage(&mut self.state.city, tile)
                    }
                    Command::CancelProject(id) => civic.cancel_project(&mut self.state.city, id),
                    _ => unreachable!(),
                };
                self.state.city.civic = civic;
                if matches!(cmd, Command::RemoveHeritage(_) | Command::DistrictPolicy(_)) {
                    self.state.city.refresh();
                }
                result = action.map(|_| "Civic decision recorded.".into());
            }
            Command::Open(window) => {
                self.state.ui.window = Some(window);
                self.state.ui.page = 0;
                self.state.panel_open = false;
                self.state.palette_open = false;
                self.state.confirm_new = false;
            }
            Command::Close => {
                self.state.ui.window = None;
                self.state.ui.rebind = None;
            }
            Command::Page(delta) => {
                self.state.ui.page = (i32::from(self.state.ui.page) + delta)
                    .rem_euclid(i32::from(crate::ui_model::page_count(&self.state)))
                    as u16
            }
            Command::PalettePage(delta) => {
                self.state.ui.palette_page = (self.state.ui.palette_page as i32 + delta)
                    .rem_euclid(crate::tools(self.state.category).len().div_ceil(8) as i32)
                    as usize
            }
            Command::Shape(shape) => {
                self.state.ui.geometry = shape;
                self.state.anchor = None;
                if shape == Geometry::BusLine {
                    self.state.ui.window = None;
                    self.state.tool = Tool::Inspect;
                    result = Ok(
                        "Click existing bus stops in order, then open Transit lines to finish."
                            .into(),
                    );
                }
            }
            Command::Minimap => self.state.ui.minimap = !self.state.ui.minimap,
            Command::FullMap => {
                self.state.ui.full_map = !self.state.ui.full_map;
                self.state.ui.window = None;
            }
            Command::Underground => self.state.ui.underground = !self.state.ui.underground,
            Command::Photo => {
                self.state.ui.photo = !self.state.ui.photo;
                #[cfg(feature = "client")]
                if self.state.ui.photo {
                    self.requests.push(crate::client::ClientRequest::Postcard);
                }
            }
            Command::Pause => self.state.speed = if self.state.speed == 0 { 1 } else { 0 },
            Command::Skip => {
                let day = self.state.city.ext.day;
                let monthly = day + 30 - day % 30;
                let target = self
                    .state
                    .city
                    .civic
                    .warnings
                    .iter()
                    .map(|w| u64::from(w.impact_day))
                    .filter(|d| *d > day)
                    .min()
                    .unwrap_or(monthly)
                    .min(monthly);
                while self.state.city.ext.day < target {
                    self.state.city.advance_day();
                }
                result = Ok(format!(
                    "Advanced {} days to the next scheduled event.",
                    target - day
                ));
            }
            Command::Climate(climate) => self.state.ui.climate = climate,
            Command::Graph(delta) => {
                self.state.ui.graph = (self.state.ui.graph as i32 + delta).rem_euclid(16) as usize
            }
            Command::Service(delta) => {
                self.state.ui.service =
                    (self.state.ui.service as i32 + delta).rem_euclid(13) as usize
            }
            Command::PlaceForMe => {
                let kind = match self.state.tool {
                    Tool::Build(k) if k.is_building() && !k.is_zone() => k,
                    _ => [
                        Kind::Police,
                        Kind::Fire,
                        Kind::Hospital,
                        Kind::School,
                        Kind::University,
                        Kind::Park,
                        Kind::BusDepot,
                        Kind::Library,
                        Kind::Prison,
                        Kind::Cemetery,
                        Kind::Pump,
                        Kind::Wind,
                        Kind::Recycling,
                    ][self.state.ui.service],
                };
                if let Some(at) = self.state.city.suggest_site(kind) {
                    self.state.cam_x = at[0];
                    self.state.cam_y = at[1];
                    self.state.selected = Some(at);
                    self.state.tool = Tool::Build(kind);
                    self.state.anchor = None;
                    self.state.panel_open = false;
                    result = Ok(format!(
                        "Suggested {} site. Click to confirm construction.",
                        kind.label()
                    ));
                } else {
                    result = Err("No suitable site. Add road access and check the terrain.".into());
                }
            }
            Command::Ordinance(i) => {
                let mut civic = std::mem::take(&mut self.state.city.civic);
                result = crate::content::ORDINANCES
                    .get(i)
                    .ok_or_else(|| "Unknown ordinance.".into())
                    .and_then(|d| civic.toggle_ordinance(&mut self.state.city, d.id))
                    .map(|_| "Ordinance updated.".into());
                self.state.city.civic = civic;
                self.state.city.refresh();
            }
            Command::Bond | Command::Bailout | Command::Restructure | Command::Rewind => {
                let mut civic = std::mem::take(&mut self.state.city.civic);
                result = match cmd {
                    Command::Bond => civic.issue_bond(&mut self.state.city, 25_000),
                    Command::Bailout => civic.bailout(&mut self.state.city),
                    Command::Restructure => civic.restructure(&mut self.state.city),
                    _ => civic.rewind_january(&mut self.state.city),
                }
                .map(|_| "Finance decision recorded.".into());
                self.state.city.civic = civic;
            }
            Command::Scenario(i) => {
                let mut civic = std::mem::take(&mut self.state.city.civic);
                result = u8::try_from(i)
                    .map_err(|_| "Unknown scenario.".to_string())
                    .and_then(|id| civic.start_scenario(&mut self.state.city, id))
                    .map(|_| {
                        "Scenario started. Review the Mayor's Notebook for goals and deadline."
                            .into()
                    });
                self.state.city.civic = civic;
                if result.is_ok() {
                    Self::prepare_city_runtime(&mut self.state.city);
                    self.state.ui.window = None;
                    self.state.ui.page = 0;
                    self.state.ui.card = None;
                    self.state.ui.pending_stops.clear();
                    self.state.ui.bankruptcy_month = None;
                    self.state.cam_x = i32::from(self.state.city.width) / 2;
                    self.state.cam_y = i32::from(self.state.city.height) / 2;
                    self.state.selected = None;
                    self.state.anchor = None;
                    self.state.hover = None;
                    self.state.world_seed = Some(self.state.city.ext.seed);
                    self.state.world_style = Some(self.state.city.ext.style);
                    self.state.ui.climate = self.state.city.ext.climate;
                    self.state.clock = 0;
                    self.state.speed = 1;
                }
            }
            Command::Project(i) => {
                if let Some([x, y]) = self.state.selected {
                    if let Some(tile) = self.state.city.idx(x, y) {
                        let mut civic = std::mem::take(&mut self.state.city.civic);
                        result = u8::try_from(i)
                            .map_err(|_| "Unknown project.".to_string())
                            .and_then(|id| {
                                civic.start_project(&mut self.state.city, id, tile as u32)
                            })
                            .map(|_| "Megaproject commissioned.".into());
                        self.state.city.civic = civic;
                    }
                } else {
                    result = Err("Select a clear, connected site on the map first.".into());
                }
            }
            Command::Frequency(delta) => {
                let level =
                    (i32::from(self.state.city.civic.disaster_frequency) + delta).clamp(0, 3) as u8;
                result = self
                    .state
                    .city
                    .civic
                    .set_disaster_frequency(level)
                    .map(|_| {
                        format!(
                            "Disaster frequency: {}.",
                            ["Off", "Low", "Normal", "High"][level as usize]
                        )
                    });
            }
            Command::Disaster(i) => {
                result = crate::civic::Hazard::ALL
                    .get(i)
                    .ok_or_else(|| "Unknown disaster.".into())
                    .and_then(|kind| self.state.city.civic.schedule_disaster(*kind, 3))
                    .map(|_| "Warning issued: disaster expected in three days.".into());
            }
            Command::Drill => {
                let mut civic = std::mem::take(&mut self.state.city.civic);
                result = civic
                    .drill(&mut self.state.city)
                    .map(|_| "Sandbox drill complete. The city was not damaged.".into());
                self.state.city.civic = civic;
            }
            Command::District => {
                let [cx, cy] = self
                    .state
                    .selected
                    .unwrap_or([self.state.cam_x, self.state.cam_y]);
                let mut tiles = Vec::new();
                for y in cy - 6..=cy + 6 {
                    for x in cx - 6..=cx + 6 {
                        if let Some(i) = self.state.city.idx(x, y) {
                            tiles.push(i as u32);
                        }
                    }
                }
                let name = format!("Ward {}", self.state.city.civic.districts.len() + 1);
                let mut civic = std::mem::take(&mut self.state.city.civic);
                result = civic
                    .create_district(&mut self.state.city, name, tiles, 0)
                    .map(|_| "District created around the selected area.".into());
                self.state.city.civic = civic;
            }
            Command::Heritage => {
                if let Some([x, y]) = self.state.selected {
                    if let Some(i) = self.state.city.idx(x, y) {
                        let mut civic = std::mem::take(&mut self.state.city.civic);
                        result = civic
                            .designate_heritage(&mut self.state.city, i as u32)
                            .map(|_| "Heritage protection recorded.".into());
                        self.state.city.civic = civic;
                    }
                } else {
                    result = Err("Select an existing building first.".into());
                }
            }
            Command::Meet(i) => {
                if let Some(post) = self.state.city.civic.chirper.iter().rev().nth(i).cloned() {
                    if let Some(tile) = post.tile {
                        let width = u32::from(self.state.city.width);
                        let at = [(tile % width) as i32, (tile / width) as i32];
                        self.state.selected = Some(at);
                        self.state.cam_x = at[0];
                        self.state.cam_y = at[1];
                        self.state.ui.citizen = post.citizen.map(|id| id as usize);
                        self.state.ui.window = None;
                        self.state.tab = 0;
                        self.state.panel_open = true;
                        let text = crate::content::CHIRPER[post.id as usize].text;
                        let profile=post.person.as_ref().map(|p|format!("Citizen {} · household {} · home {},{} · education {} · health {} · income tier {} · employer {}",p.id,p.size,p.home%width,p.home/width,p.education,p.health,p.income_class,p.employer.map(|j|j.to_string()).unwrap_or_else(||"Seeking work".into()))).unwrap_or_else(||"A neighborhood post; no individual household was recorded.".into());
                        self.state.ui.card=Some(crate::ui_model::Card{id:self.state.tick.wrapping_add(1),title:format!("Meet citizen {}",post.citizen.map(|n|n.to_string()).unwrap_or_else(||"Citywide".into())),body:format!("{text}\n{profile}\nProfile recorded in month {}. The camera shows the recorded home.",post.month)});
                    } else {
                        result = Err("This citywide post has no home address.".into());
                    }
                } else {
                    result = Err("That post is no longer available.".into());
                }
            }
            Command::FinishLine => {
                result = self
                    .state
                    .city
                    .create_line(
                        self.state.ui.transit_mode,
                        self.state
                            .ui
                            .pending_stops
                            .iter()
                            .map(|i| *i as u32)
                            .collect(),
                    )
                    .map(|id| {
                        self.state.ui.pending_stops.clear();
                        format!("Bus line {id} opened.")
                    });
            }
            Command::SelectLine(delta) => {
                let n = self.state.city.ext.transport.lines.len();
                if n > 0 {
                    self.state.ui.selected_line =
                        (self.state.ui.selected_line as i32 + delta).rem_euclid(n as i32) as usize;
                }
            }
            Command::TransitMode(delta) => {
                use crate::transport::TransitMode::*;
                let modes = [Bus, Rail, Metro, Ferry, Freight];
                let i = modes
                    .iter()
                    .position(|m| *m == self.state.ui.transit_mode)
                    .unwrap_or(0);
                self.state.ui.transit_mode = modes[(i as i32 + delta).rem_euclid(5) as usize];
                self.state.ui.pending_stops.clear();
            }
            Command::Fare(delta) => {
                if let Some(line) = self
                    .state
                    .city
                    .ext
                    .transport
                    .lines
                    .get(self.state.ui.selected_line)
                {
                    let id = line.id;
                    let fare = (i32::from(line.fare) + delta).clamp(0, 20) as u16;
                    result = self
                        .state
                        .city
                        .set_line_fare(id, fare)
                        .map(|_| format!("Fare set to ${fare}."));
                } else {
                    result = Err("Create a transit line first.".into());
                }
            }
            Command::DeleteLine => {
                if self.state.ui.selected_line < self.state.city.ext.transport.lines.len() {
                    self.state
                        .city
                        .ext
                        .transport
                        .lines
                        .remove(self.state.ui.selected_line);
                    result = Ok("Transit line closed.".into());
                }
            }
            Command::SetVolume(i, value) => {
                let p = &mut self.state.ui.preferences;
                match i {
                    0 => p.music = value.min(100),
                    1 => p.sound = value.min(100),
                    2 => p.ambience = value.min(100),
                    _ => result = Err("Unknown volume control.".into()),
                }
            }
            Command::Setting(i, delta) => {
                let p = &mut self.state.ui.preferences;
                let target = match i {
                    0 => &mut p.music,
                    1 => &mut p.sound,
                    2 => &mut p.ambience,
                    3 => &mut p.text_scale,
                    _ => &mut p.display_scale,
                };
                *target = (i32::from(*target) + delta)
                    .clamp(if i < 3 { 0 } else { 80 }, if i < 3 { 100 } else { 150 })
                    as u8;
            }
            Command::ToggleCalm => self.state.ui.preferences.calm = !self.state.ui.preferences.calm,
            Command::ToggleAutosave => {
                self.state.ui.preferences.autosave = !self.state.ui.preferences.autosave
            }
            Command::Rebind(i) => {
                self.state.ui.rebind = Some(i);
                result = Ok("Press a letter, arrow, function key, Space, Tab or Escape.".into());
            }
            Command::ResetKeys => {
                self.state.ui.preferences.keys = crate::ui_model::Preferences::default().keys
            }
            Command::Save => {
                #[cfg(feature = "client")]
                self.requests.push(crate::client::ClientRequest::Save);
            }
            Command::Load => {
                #[cfg(feature = "client")]
                self.requests.push(crate::client::ClientRequest::Load);
            }
            Command::NewMap => {
                self.state.ui.window = None;
                self.state.confirm_new = true;
                self.state.panel_open = true;
            }
            Command::Surprise => {
                self.state.draft_seed = vesper3d::runtime::StateHasher::new()
                    .u64(self.state.city.seed)
                    .u32(self.state.tick)
                    .finish()
                    .to_string();
                self.state.map_style = crate::city::MapStyle::ALL
                    [self.state.tick as usize % crate::city::MapStyle::ALL.len()];
                self.state.ui.climate = crate::world::Climate::ALL[self.state.tick as usize % 3];
                self.state.confirm_new = true;
                self.state.ui.window = None;
                self.state.panel_open = true;
            }
        }
        self.state.ui.page = self
            .state
            .ui
            .page
            .min(crate::ui_model::page_count(&self.state).saturating_sub(1));
        match result {
            Ok(message) if !message.is_empty() => self.message(message),
            Err(error) => {
                self.message(error);
                self.cues.push(1);
            }
            _ => {}
        }
    }
}
