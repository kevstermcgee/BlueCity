//! End-to-end portable intentions for the expanded map-first interface.
use bluecity::{
    city::{Kind, MapStyle, Terrain},
    layout::{self, Action},
    ui_model::{self, Command, Geometry, Window},
    world::{Climate, TerrainAction},
    BlueCity, State, Tool,
};
use vesper3d::{
    runtime::snapshot,
    two_d::{GameLogic, Intent, Point, Simulation, Snapshot},
};
fn tap(g: &mut BlueCity, x: i32, y: i32) {
    g.step(&Intent {
        pointer: Some(Point::new(x, y)),
        action: true,
        ..Default::default()
    });
}
fn press(g: &mut BlueCity, command: Command) {
    let (r, _) = layout::controls(&g.state)
        .into_iter()
        .find(|(_, a)| *a == Action::Extra(command))
        .unwrap_or_else(|| panic!("No visible control for {command:?}"));
    tap(g, r.x + r.w / 2, r.y + r.h / 2);
}
fn paused() -> BlueCity {
    let mut g = BlueCity::new(7);
    let r = layout::speed(0);
    tap(&mut g, r.x + 2, r.y + 2);
    g
}
fn open(g: &mut BlueCity, w: Window) {
    press(g, Command::Open(Window::Council));
    if w != Window::Council {
        press(g, Command::Open(w));
    }
}
fn grass(g: &BlueCity) -> [i32; 2] {
    for y in g.state.cam_y - 10..g.state.cam_y + 10 {
        for x in g.state.cam_x - 10..g.state.cam_x + 10 {
            if g.state
                .city
                .tile(x, y)
                .is_some_and(|t| t.kind == Kind::Empty && t.terrain == Terrain::Grass)
                && g.pick(g.project(x, y)) == Some([x, y])
            {
                return [x, y];
            }
        }
    }
    panic!("No visible open site")
}
#[test]
fn all_optional_windows_mask_their_own_area_and_leave_the_city_intact() {
    let mut g = paused();
    let before = g.state.city.clone();
    for w in Window::ALL {
        open(&mut g, w);
        assert_eq!(g.state.ui.window, Some(w));
        assert_eq!(g.pick(Point::new(160, 200)), None);
        press(&mut g, Command::Close);
        assert_eq!(g.state.ui.window, None);
    }
    assert_eq!(g.state.city, before);
}
#[test]
fn all_twenty_data_views_and_thirteen_coverages_are_reachable() {
    let mut g = paused();
    let r = layout::tab(1);
    tap(&mut g, r.x + 3, r.y + 3);
    for i in 0..20 {
        let r = layout::overlay(i);
        tap(&mut g, r.x + 3, r.y + 3);
        assert_eq!(g.state.overlay, i);
    }
    for i in 1..=13 {
        press(&mut g, Command::Service(1));
        assert_eq!(g.state.ui.service, i % 13);
    }
}
#[test]
fn seven_styles_and_three_climates_use_the_same_generation_controls() {
    for style in MapStyle::ALL {
        for climate in Climate::ALL {
            let mut g = paused();
            g.command(Command::NewMap);
            let (r, _) = layout::controls(&g.state)
                .into_iter()
                .find(|(_, a)| *a == Action::MapStyle(style))
                .unwrap();
            tap(&mut g, r.x + 2, r.y + 2);
            press(&mut g, Command::Climate(climate));
            let r = layout::CONFIRM_NEW;
            tap(&mut g, r.x + 2, r.y + 2);
            assert_eq!(g.state.city.width, 128);
            assert_eq!(g.state.city.ext.climate, climate);
            assert_eq!(g.state.world_style, Some(style));
            assert_eq!(
                g.state.city,
                bluecity::city::City::world(g.state.world_seed.unwrap(), style, climate)
            );
        }
    }
}
#[test]
fn catalog_pagination_exposes_new_infrastructure_and_terrain_tools() {
    let mut g = paused();
    for category in 0..5 {
        let r = layout::category(category);
        tap(&mut g, r.x + 4, r.y + 4);
        assert!(g.state.palette_open);
        let pages = bluecity::tools(category).len().div_ceil(8);
        let mut found = Vec::new();
        for page in 0..pages {
            assert_eq!(g.state.ui.palette_page, page);
            found.extend(layout::controls(&g.state).into_iter().filter_map(|(_, a)| {
                if let Action::SelectTool(t) = a {
                    Some(t)
                } else {
                    None
                }
            }));
            if page + 1 < pages {
                press(&mut g, Command::PalettePage(1));
            }
        }
        for tool in bluecity::tools(category) {
            assert!(found.contains(&tool), "Missing {tool:?}");
        }
        let r = layout::category(category);
        tap(&mut g, r.x + 4, r.y + 4);
    }
    assert_eq!(TerrainAction::ALL.len(), 8);
}
#[test]
fn exact_preview_price_includes_terraforming_and_refuses_atomic_invalid_edits() {
    let mut g = paused();
    for t in &mut g.state.city.tiles {
        *t = bluecity::city::Tile::new(Terrain::Grass);
    }
    g.state.city.refresh();
    let at = grass(&g);
    g.state.tool = Tool::Shape(TerrainAction::Water);
    g.state.ui.geometry = Geometry::Brush;
    let plan = bluecity::placement::plan(&g.state, at);
    assert_eq!(plan.cost, plan.cells.len() as i64 * 80);
    let before = g.state.city.clone();
    assert!(plan.refusal.is_some() || plan.cost > 0);
    g.state.ui.geometry = Geometry::Rectangle;
    g.state.tool = Tool::Build(Kind::Road);
    g.state.anchor = Some(at);
    let end = [at[0] + 2, at[1]];
    let i = g.state.city.idx(end[0], end[1]).unwrap();
    g.state.city.tiles[i].kind = Kind::Park;
    let plan = bluecity::placement::plan(&g.state, end);
    assert!(plan.refusal.is_some());
    let money = g.state.city.money;
    let p = g.project(end[0], end[1]);
    tap(&mut g, p.x, p.y);
    assert_eq!(g.state.city.money, money);
    assert_eq!(
        g.state.city.tile(at[0], at[1]).unwrap(),
        before.tile(at[0], at[1]).unwrap()
    );
}
#[test]
fn geometry_preview_is_the_same_paid_footprint_as_the_final_edit() {
    let mut g = paused();
    for t in &mut g.state.city.tiles {
        *t = bluecity::city::Tile::new(Terrain::Grass);
    }
    g.state.city.refresh();
    g.state.tool = Tool::Build(Kind::Road);
    let start = [17, 23];
    let end = [21, 26];
    g.state.ui.geometry = Geometry::Curve;
    g.state.anchor = Some(start);
    let plan = bluecity::placement::plan(&g.state, end);
    assert!(plan.refusal.is_none(), "{:?}", plan.refusal);
    assert!(plan.cells.contains(&start) && plan.cells.contains(&end));
    let money = g.state.city.money;
    let p = g.project(end[0], end[1]);
    tap(&mut g, p.x, p.y);
    assert_eq!(g.state.city.money, money - plan.cost);
    let actual: Vec<_> = g
        .state
        .city
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.kind == Kind::Road)
        .map(|(i, _)| [(i % 48) as i32, (i / 48) as i32])
        .collect();
    assert_eq!(actual, plan.cells);
}

#[test]
fn paid_curved_roads_form_real_routes_in_both_directions() {
    use bluecity::transport::{route_between, TransitMode};
    for (start, end) in [
        ([17, 23], [21, 26]),
        ([21, 26], [17, 23]),
        ([17, 26], [22, 23]),
    ] {
        let mut g = paused();
        for t in &mut g.state.city.tiles {
            *t = bluecity::city::Tile::new(Terrain::Grass);
        }
        g.state.city.refresh();
        g.state.tool = Tool::Build(Kind::Road);
        g.state.ui.geometry = Geometry::Curve;
        g.state.anchor = Some(start);
        let plan = bluecity::placement::plan(&g.state, end);
        assert!(plan.refusal.is_none());
        let cash = g.state.city.money;
        let at = g.project(end[0], end[1]);
        tap(&mut g, at.x, at.y);
        assert_eq!(g.state.city.money, cash - plan.cost);
        let a = g.state.city.idx(start[0], start[1]).unwrap();
        let b = g.state.city.idx(end[0], end[1]).unwrap();
        for (from, to) in [(a, b), (b, a)] {
            let route = route_between(&g.state.city, from, to, TransitMode::Bus)
                .expect("a paid curve must carry a real street route");
            assert_eq!(route.first(), Some(&(from as u32)));
            assert_eq!(route.last(), Some(&(to as u32)));
            assert!(route
                .windows(2)
                .all(|p| g.state.city.distance(p[0] as usize, p[1] as usize) == 1));
            assert!(route
                .iter()
                .all(|&i| g.state.city.tiles[i as usize].kind == Kind::Road));
        }
    }
}

fn empty_street_town() -> BlueCity {
    let mut game = paused();
    game.state.city = bluecity::city::City::new(7, false);
    for tile in &mut game.state.city.tiles {
        *tile = bluecity::city::Tile::new(Terrain::Grass);
    }
    game.state.city.initialize_world();
    for cell in &mut game.state.city.ext.cells {
        cell.elevation = 0;
        cell.water_depth = 0;
    }
    game.state.city.refresh();
    game
}
fn choose_one_way(game: &mut BlueCity, geometry: Geometry) {
    let category = layout::category(1);
    tap(game, category.x + 3, category.y + 3);
    press(game, Command::Shape(geometry));
    for _ in 0..bluecity::tools(1).len().div_ceil(8) {
        if let Some((r, _)) = layout::controls(&game.state)
            .into_iter()
            .find(|(_, a)| *a == Action::SelectTool(Tool::Build(Kind::OneWay)))
        {
            tap(game, r.x + 3, r.y + 3);
            assert_eq!(game.state.tool, Tool::Build(Kind::OneWay));
            return;
        }
        press(game, Command::PalettePage(1));
    }
    panic!("One-way tool must be reachable in Transport");
}

#[test]
fn pointer_one_way_strokes_follow_curves_verticals_and_reversed_endpoints() {
    use bluecity::transport::{route_between, TransitMode};
    for (start, end, geometry) in [
        ([17, 23], [22, 27], Geometry::Curve),
        ([22, 27], [17, 23], Geometry::Curve),
        ([20, 20], [20, 27], Geometry::Line),
        ([20, 27], [20, 20], Geometry::Line),
        ([17, 24], [25, 24], Geometry::Line),
        ([25, 24], [17, 24], Geometry::Line),
    ] {
        let mut game = empty_street_town();
        choose_one_way(&mut game, geometry);
        let point = game.project(start[0], start[1]);
        tap(&mut game, point.x, point.y);
        assert_eq!(game.state.anchor, Some(start));
        let quote = bluecity::placement::plan(&game.state, end);
        assert!(quote.refusal.is_none());
        assert_eq!(quote.cost, quote.cells.len() as i64 * Kind::OneWay.cost());
        let cash = game.state.city.money;
        let point = game.project(end[0], end[1]);
        tap(&mut game, point.x, point.y);
        assert_eq!(game.state.city.money, cash - quote.cost);
        let a = game.state.city.idx(start[0], start[1]).unwrap();
        let b = game.state.city.idx(end[0], end[1]).unwrap();
        assert!(route_between(&game.state.city, a, b, TransitMode::Bus).is_some());
        assert!(route_between(&game.state.city, b, a, TransitMode::Bus).is_none());
        for (at, direction) in quote.road_directions {
            let i = game.state.city.idx(at[0], at[1]).unwrap();
            assert_eq!(game.state.city.ext.cells[i].orientation, direction);
        }
        let frame = snapshot::save(&game, "Directed street from public pointer input").unwrap();
        let mut restored = BlueCity::new(0);
        snapshot::restore(&mut restored, &frame).unwrap();
        for _ in 0..3 {
            game.state.city.advance_day();
            restored.state.city.advance_day();
        }
        assert_eq!(game.state_hash(), restored.state_hash());
        assert!(route_between(&restored.state.city, a, b, TransitMode::Bus).is_some());
        assert!(route_between(&restored.state.city, b, a, TransitMode::Bus).is_none());
    }
}

#[test]
fn inspected_one_way_has_a_free_rotation_button_that_changes_real_routing() {
    use bluecity::transport::{route_between, TransitMode};
    let mut game = empty_street_town();
    choose_one_way(&mut game, Geometry::Line);
    for at in [[20, 20], [20, 27]] {
        let point = game.project(at[0], at[1]);
        tap(&mut game, point.x, point.y);
    }
    let inspect = layout::INSPECT;
    tap(&mut game, inspect.x + 3, inspect.y + 3);
    let at = [20, 24];
    let point = game.project(at[0], at[1]);
    tap(&mut game, point.x, point.y);
    let tab = layout::tab(0);
    tap(&mut game, tab.x + 3, tab.y + 3);
    let cash = game.state.city.money;
    let i = game.state.city.idx(at[0], at[1]).unwrap();
    let a = game.state.city.idx(20, 20).unwrap();
    let b = game.state.city.idx(20, 27).unwrap();
    for (name, next) in [("South", 2), ("West", 3), ("North", 0), ("East", 1)] {
        let button = ui_model::controls(&game.state)
            .into_iter()
            .find(|c| c.command == Command::RotateRoad)
            .unwrap();
        assert!(button.label.contains(name));
        press(&mut game, Command::RotateRoad);
        assert_eq!(game.state.city.ext.cells[i].orientation, next);
        assert_eq!(game.state.city.money, cash);
        assert_eq!(
            route_between(&game.state.city, a, b, TransitMode::Bus).is_some(),
            next == 1
        );
    }
    assert!(route_between(&game.state.city, b, a, TransitMode::Bus).is_none());
}
#[test]
fn hotkey_rebinding_is_a_saved_fixed_step_action_and_bad_views_are_rejected() {
    let mut g = paused();
    open(&mut g, Window::Settings);
    press(&mut g, Command::Page(1));
    press(&mut g, Command::Rebind(0));
    let key = ui_model::KEY_NAMES.iter().position(|s| *s == "Y").unwrap();
    tap(&mut g, -1100, key as i32);
    assert_eq!(g.state.ui.preferences.keys[0], "Y");
    assert!(g.state.ui.rebind.is_none());
    let bytes = snapshot::save(&g, "Rebound controls").unwrap();
    let mut resumed = BlueCity::new(2);
    snapshot::restore(&mut resumed, &bytes).unwrap();
    assert_eq!(resumed.state_hash(), g.state_hash());
    let before = resumed.state_hash();
    let mut bad = resumed.capture();
    bad.ui.service = 13;
    assert!(resumed.restore(bad).is_err());
    assert_eq!(resumed.state_hash(), before);
    let mut value = serde_json::to_value(g.capture()).unwrap();
    value["city"]["codec"] = "unsupported".into();
    assert!(serde_json::from_value::<State>(value).is_err());
}
#[test]
fn minimap_recenters_without_building_and_photo_mode_has_one_exit() {
    let mut g = paused();
    g.state.tool = Tool::Build(Kind::Park);
    press(&mut g, Command::Minimap);
    let money = g.state.city.money;
    tap(&mut g, 78, 152);
    assert_eq!([g.state.cam_x, g.state.cam_y], [24, 24]);
    assert_eq!(g.state.city.money, money);
    press(&mut g, Command::Photo);
    assert!(g.state.ui.photo);
    assert_eq!(layout::controls(&g.state).len(), 1);
    let before = g.state.city.clone();
    tap(&mut g, 400, 225);
    assert_eq!(g.state.city, before);
    press(&mut g, Command::Photo);
    assert!(!g.state.ui.photo);
}
#[test]
fn daily_clock_accounts_for_one_month_at_high_speed_and_resumes_exactly() {
    let mut g = paused();
    let r = layout::speed(3);
    tap(&mut g, r.x + 2, r.y + 2);
    for _ in 0..23 {
        g.step(&Intent::default());
    }
    assert_eq!(g.state.city.month, 1);
    assert_eq!(g.state.city.ext.day, 30);
    assert_eq!(g.state.city.civic.history.len(), 1);
    let bytes = snapshot::save(&g, "Monthly boundary").unwrap();
    let mut restored = BlueCity::new(0);
    snapshot::restore(&mut restored, &bytes).unwrap();
    for _ in 0..24 {
        g.step(&Intent::default());
        restored.step(&Intent::default());
    }
    assert_eq!(g.state_hash(), restored.state_hash());
    assert_eq!(g.state.city.month, 2);
}

#[test]
fn volume_sliders_use_pointer_position_save_and_respect_cards() {
    let mut game = paused();
    open(&mut game, Window::Settings);
    let town = game.state.city.clone();
    for (i, expected) in [0, 50, 100].into_iter().enumerate() {
        let r = ui_model::volume_slider(i);
        tap(
            &mut game,
            r.x + 4 + expected * (r.w - 8) / 100,
            r.y + r.h / 2,
        );
    }
    let p = &game.state.ui.preferences;
    assert_eq!((p.music, p.sound, p.ambience), (0, 50, 100));
    assert_eq!(game.state.city, town);
    game.state.ui.card = Some(ui_model::Card {
        id: 1,
        title: "Notice".into(),
        body: "This card masks the slider below it. ".repeat(20),
    });
    let r = ui_model::volume_slider(2);
    let before = game.state.ui.preferences.clone();
    tap(&mut game, r.x + 2, r.y + 5);
    assert_eq!(game.state.ui.preferences, before);
    let frame = snapshot::save(&game, "Volumes").unwrap();
    let mut loaded = BlueCity::new(0);
    snapshot::restore(&mut loaded, &frame).unwrap();
    assert_eq!(loaded.state.ui.preferences, game.state.ui.preferences);
}
