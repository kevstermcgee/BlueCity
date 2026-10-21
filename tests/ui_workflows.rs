//! Player-facing workflows: every game action follows the public pointer/Intent route.
use bluecity::{
    city::{Kind, MapStyle, Terrain, SIZE},
    layout, BlueCity, Tool,
};
use vesper3d::two_d::{GameLogic, Intent, Point, Rect, Simulation, Snapshot};

fn click(game: &mut BlueCity, x: i32, y: i32) {
    game.step(&Intent {
        pointer: Some(Point::new(x, y)),
        action: true,
        ..Intent::default()
    });
}
fn tap(game: &mut BlueCity, rect: Rect) {
    click(game, rect.x + rect.w / 2, rect.y + rect.h / 2);
}
fn tab(game: &mut BlueCity, id: u8) {
    if !game.state.panel_open || game.state.tab != id {
        tap(game, layout::tab(id));
    }
}
fn tool(game: &mut BlueCity, category: u8, id: usize) {
    if !game.state.palette_open || game.state.category != category {
        tap(game, layout::category(category));
    }
    tap(game, layout::palette_tool(category, id));
    assert!(!game.state.palette_open);
}
fn block(game: &mut BlueCity, x: i32, y: i32) {
    let p = game.project(x, y);
    assert_eq!(
        game.pick(p),
        Some([x, y]),
        "visible tile centers must select the rendered tile"
    );
    click(game, p.x, p.y);
}
fn paused() -> BlueCity {
    let mut game = BlueCity::new(7);
    tap(&mut game, layout::speed(0));
    assert_eq!(game.state.speed, 0);
    game
}
fn blank() -> BlueCity {
    let mut game = paused();
    tab(&mut game, 0);
    tap(&mut game, layout::NEW_CITY);
    assert!(game.state.confirm_new);
    tap(&mut game, layout::CONFIRM_NEW);
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.speed, 0);
    game
}
fn generate(game: &mut BlueCity, style: usize, seed: &str) {
    tab(game, 0);
    tap(game, layout::NEW_CITY);
    tap(game, layout::map_style(style));
    tap(game, layout::SEED_FIELD);
    for digit in seed.bytes() {
        tap(game, layout::seed_digit(digit - b'0'));
    }
    tap(game, layout::CONFIRM_NEW);
    assert!(!game.state.confirm_new);
}
fn grass_rectangle(game: &BlueCity, width: i32, height: i32) -> [i32; 2] {
    for y in 12..36 {
        for x in 12..36 {
            let clear = (y..y + height).all(|ty| {
                (x..x + width).all(|tx| {
                    let p = game.project(tx, ty);
                    let tile = game.state.city.tile(tx, ty).unwrap();
                    game.pick(p) == Some([tx, ty])
                        && tile.kind == Kind::Empty
                        && tile.terrain == Terrain::Grass
                })
            });
            if clear {
                return [x, y];
            }
        }
    }
    panic!("the generated valley should have a visible open grass site");
}

#[test]
fn a_park_builds_with_one_click_and_cannot_be_bought_twice() {
    let mut game = blank();
    let [x, y] = grass_rectangle(&game, 1, 1);
    tool(&mut game, 0, 3);
    assert_eq!(game.state.tool, Tool::Build(Kind::Park));
    let treasury = game.state.city.money;
    block(&mut game, x, y);
    assert_eq!(game.state.city.tile(x, y).unwrap().kind, Kind::Park);
    assert_eq!(game.state.city.money, treasury - Kind::Park.cost());
    assert_eq!(game.state.anchor, None);
    block(&mut game, x, y);
    assert_eq!(game.state.city.money, treasury - Kind::Park.cost());
    assert!(game.state.notice.contains("already"));

    let water = (0..SIZE as i32)
        .flat_map(|ty| (0..SIZE as i32).map(move |tx| [tx, ty]))
        .find(|[tx, ty]| {
            let p = game.project(*tx, *ty);
            game.pick(p) == Some([*tx, *ty])
                && game.state.city.tile(*tx, *ty).unwrap().terrain == Terrain::Water
        })
        .expect("the river should be visible");
    let treasury = game.state.city.money;
    block(&mut game, water[0], water[1]);
    assert_eq!(game.state.city.money, treasury);
    assert_eq!(
        game.state.city.tile(water[0], water[1]).unwrap().kind,
        Kind::Empty
    );
    assert!(game.state.notice.contains("dry land"));
}

#[test]
fn a_two_click_road_and_neighborhood_commit_only_after_the_endpoint() {
    let mut game = blank();
    let [x, y] = grass_rectangle(&game, 3, 3);
    tool(&mut game, 1, 0); // Road.
    assert_eq!(game.state.tool, Tool::Build(Kind::Road));
    let treasury = game.state.city.money;
    block(&mut game, x, y);
    assert_eq!(game.state.anchor, Some([x, y]));
    assert_eq!(game.state.city.money, treasury);
    assert_eq!(game.state.city.tile(x, y).unwrap().kind, Kind::Empty);
    block(&mut game, x + 2, y + 1);
    for [tx, ty] in [[x, y], [x + 1, y], [x + 2, y], [x + 2, y + 1]] {
        assert_eq!(game.state.city.tile(tx, ty).unwrap().kind, Kind::Road);
    }
    assert_eq!(
        game.state.city.tile(x + 1, y + 1).unwrap().kind,
        Kind::Empty
    );
    assert_eq!(game.state.city.money, treasury - 4 * Kind::Road.cost());
    assert_eq!(game.state.anchor, None);

    tool(&mut game, 0, 0); // Homes.
    let treasury = game.state.city.money;
    block(&mut game, x, y + 1);
    assert_eq!(game.state.city.money, treasury);
    block(&mut game, x + 1, y + 2);
    for ty in y + 1..=y + 2 {
        for tx in x..=x + 1 {
            assert_eq!(
                game.state.city.tile(tx, ty).unwrap().kind,
                Kind::Residential
            );
        }
    }
    assert_eq!(
        game.state.city.money,
        treasury - 4 * Kind::Residential.cost()
    );

    // Canceling an unfinished expansion leaves both land and treasury untouched.
    block(&mut game, x + 2, y + 2);
    let treasury = game.state.city.money;
    tap(&mut game, layout::INSPECT);
    assert_eq!(game.state.anchor, None);
    assert_eq!(game.state.tool, Tool::Inspect);
    block(&mut game, x + 2, y + 2);
    assert_eq!(
        game.state.city.tile(x + 2, y + 2).unwrap().kind,
        Kind::Empty
    );
    assert_eq!(game.state.city.money, treasury);
}

#[test]
fn pan_zoom_and_tile_inspection_work_while_the_city_is_paused() {
    let mut game = paused();
    let month = game.state.city.month;
    for _ in 0..12 {
        game.step(&Intent {
            x: 1,
            ..Intent::default()
        });
    }
    assert_eq!([game.state.cam_x, game.state.cam_y], [26, 22]);
    assert_eq!(game.state.city.month, month);
    tap(&mut game, layout::CENTER);
    assert_eq!([game.state.cam_x, game.state.cam_y], [24, 24]);
    for _ in 0..3 {
        tap(&mut game, layout::ZOOM_PLUS);
    }
    assert_eq!(game.state.zoom, 3);
    for zoom in (1..=3).rev() {
        assert_eq!(game.state.zoom, zoom);
        let p = game.project(23, 24);
        click(&mut game, p.x - 1, p.y - 1); // Inside the same centered diamond.
        assert_eq!(game.state.selected, Some([23, 24]));
        tap(&mut game, layout::ZOOM_MINUS);
    }
    assert_eq!(game.state.zoom, 1);
    tap(&mut game, layout::ZOOM_MINUS);
    assert_eq!(game.state.zoom, 1);
    let selected = game.state.selected;
    click(&mut game, 106, 423); // The gap between floating category buttons is masked.
    assert_eq!(game.state.selected, selected);
    assert_eq!(game.state.city.month, month);
}

#[test]
fn map_layers_and_municipal_budget_controls_follow_pointer_input() {
    let mut game = paused();
    tab(&mut game, 1);
    assert_eq!(game.state.tab, 1);
    tap(&mut game, layout::overlay(6));
    assert_eq!(game.state.overlay, 6);
    tap(&mut game, layout::overlay(7));
    assert_eq!(game.state.overlay, 7);
    tap(&mut game, layout::overlay(1));
    assert_eq!(game.state.overlay, 1);

    tab(&mut game, 2);
    assert_eq!(game.state.tab, 2);
    let tax = game.state.city.taxes[0];
    tap(&mut game, layout::tax_plus(0));
    assert_eq!(game.state.city.taxes[0], tax + 1);
    tap(&mut game, layout::tax_minus(0));
    assert_eq!(game.state.city.taxes[0], tax);
    for _ in 0..7 {
        tap(&mut game, layout::funding_plus(1));
    }
    assert_eq!(game.state.city.funding[1], 150);
    for _ in 0..20 {
        tap(&mut game, layout::funding_minus(1));
    }
    assert_eq!(game.state.city.funding[1], 0);
    for policy in 0..4 {
        tap(&mut game, layout::policy(policy));
    }
    assert_eq!(game.state.city.policies, [true; 4]);
    tap(&mut game, layout::policy(2));
    assert_eq!(game.state.city.policies, [true, true, false, true]);

    let treasury = game.state.city.money;
    for _ in 0..3 {
        tap(&mut game, layout::BORROW);
    }
    assert_eq!(game.state.city.money, treasury + 30_000);
    assert_eq!(game.state.city.debt, 30_000);
    tap(&mut game, layout::BORROW);
    assert_eq!(game.state.city.money, treasury + 30_000);
    assert!(game.state.notice.contains("limit"));
    for _ in 0..3 {
        tap(&mut game, layout::REPAY);
    }
    assert_eq!(game.state.city.money, treasury);
    assert_eq!(game.state.city.debt, 0);
    tap(&mut game, layout::REPAY);
    assert_eq!(game.state.city.money, treasury);
    assert!(game.state.notice.contains("no outstanding"));
}

#[test]
fn starting_a_new_valley_requires_confirmation_and_starts_paused() {
    let mut game = BlueCity::new(7);
    let population = game.state.city.population;
    let seed = game.state.city.seed;
    assert!(population > 0);
    tab(&mut game, 0);
    tap(&mut game, layout::NEW_CITY);
    assert!(game.state.confirm_new);
    assert_eq!(game.state.city.population, population);
    tap(&mut game, layout::CANCEL_NEW); // Keep this town.
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.city.seed, seed);
    assert_eq!(game.state.city.population, population);
    tab(&mut game, 0);
    tap(&mut game, layout::NEW_CITY);
    tap(&mut game, layout::CONFIRM_NEW); // Start the blank valley.
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.city.population, 0);
    assert_eq!(game.state.city.money, 60_000);
    assert_eq!(game.state.city.seed, seed.wrapping_add(1));
    assert_eq!(game.state.speed, 0);
    assert!(game
        .state
        .city
        .tiles
        .iter()
        .all(|tile| tile.kind == Kind::Empty));
    for _ in 0..480 {
        game.step(&Intent::default());
    }
    assert_eq!(game.state.city.month, 0);
    assert_eq!(game.state.city.money, 60_000);
}

#[test]
fn a_valid_save_round_trips_and_corrupt_saves_leave_the_running_city_intact() {
    let mut game = paused();
    tab(&mut game, 2);
    tap(&mut game, layout::tax_plus(0));
    tap(&mut game, layout::policy(0));
    let encoded = serde_json::to_string(&game.capture()).unwrap();
    let saved = serde_json::from_str(&encoded).unwrap();
    let mut loaded = BlueCity::new(99);
    loaded.restore(saved).unwrap();
    assert_eq!(game.state_hash(), loaded.state_hash());
    let hash = loaded.state_hash();

    let mut invalid = loaded.capture();
    invalid.city.tiles.pop();
    assert!(loaded.restore(invalid).is_err());
    assert_eq!(loaded.state_hash(), hash);
    let mut invalid = loaded.capture();
    invalid.anchor = Some([-1, 24]);
    assert!(loaded.restore(invalid).is_err());
    assert_eq!(loaded.state_hash(), hash);
    let mut invalid = loaded.capture();
    invalid.city.taxes[0] = 21;
    assert!(loaded.restore(invalid).is_err());
    assert_eq!(loaded.state_hash(), hash);
    let mut invalid = loaded.capture();
    invalid.city.population += 1;
    assert!(loaded.restore(invalid).is_err());
    assert_eq!(loaded.state_hash(), hash);
}

#[test]
fn the_map_uses_the_full_viewport_and_floating_panels_mask_only_their_areas() {
    let mut game = paused();
    assert!(!game.state.panel_open && !game.state.palette_open);
    assert_eq!(game.project(24, 24), Point::new(400, 225));
    let east = game.project(36, 14);
    assert!(east.x > 592, "the old sidebar area must be usable map");
    assert_eq!(game.pick(east), Some([36, 14]));
    assert_eq!(
        game.pick(Point::new(256, 381)),
        Some([31, 43]),
        "passive status text must not eat map input"
    );
    tool(&mut game, 0, 3);
    let city = game.state.city.clone();
    tab(&mut game, 2);
    assert!(game.state.panel_open);
    assert_eq!(game.pick(east), None);
    for p in [
        Point::new(580, 230),
        Point::new(744, 167),
        Point::new(577, 78),
        Point::new(106, 423),
    ] {
        assert_eq!(game.pick(p), None);
        click(&mut game, p.x, p.y);
    }
    assert_eq!(
        game.state.city, city,
        "clicking panel spacing must never build behind it"
    );
    assert_eq!(game.state.anchor, None);
    tap(&mut game, layout::tab(2));
    assert!(!game.state.panel_open);
    assert_eq!(game.pick(east), Some([36, 14]));
    tap(&mut game, layout::category(0));
    assert!(game.state.palette_open);
    click(&mut game, 143, 374); // Spacing between palette buttons.
    assert_eq!(game.state.city, city);
    assert!(game.state.palette_open);
    tap(&mut game, layout::palette_tool(0, 0));
    assert_eq!(game.state.tool, Tool::Build(Kind::Residential));
    assert!(!game.state.palette_open);
    assert_eq!(game.pick(Point::new(256, 381)), Some([31, 43]));
}

#[test]
fn legacy_json_without_panel_fields_restores_the_city_and_its_exact_future() {
    let mut original = paused();
    tab(&mut original, 2);
    tap(&mut original, layout::tax_plus(0));
    tap(&mut original, layout::policy(2));
    tap(&mut original, layout::CLOSE_PANEL);
    tap(&mut original, layout::speed(2));
    let mut legacy = serde_json::to_value(original.capture()).unwrap();
    legacy.as_object_mut().unwrap().remove("panel_open");
    legacy.as_object_mut().unwrap().remove("palette_open");
    let legacy_state: bluecity::State = serde_json::from_value(legacy).unwrap();
    assert!(!legacy_state.panel_open && !legacy_state.palette_open);
    let mut loaded = BlueCity::new(111);
    loaded.restore(legacy_state).unwrap();
    assert_eq!(loaded.state.city, original.state.city);
    assert_eq!(loaded.state_hash(), original.state_hash());
    for _ in 0..960 {
        loaded.step(&Intent::default());
        original.step(&Intent::default());
    }
    assert_eq!(loaded.state_hash(), original.state_hash());
}

// Frozen release-1 schema, including field order: the original game hashed the
// serialized struct bytes. Do not replace this with the current State or Value,
// which would change the hash or quietly include future fields in the fixture.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct LegacyState {
    city: bluecity::city::City,
    tick: u32,
    clock: u32,
    speed: u8,
    tool: Tool,
    category: u8,
    cam_x: i32,
    cam_y: i32,
    zoom: i32,
    overlay: u8,
    tab: u8,
    hover: Option<[i32; 2]>,
    selected: Option<[i32; 2]>,
    anchor: Option<[i32; 2]>,
    notice: String,
    notice_ticks: u32,
    confirm_new: bool,
}
struct LegacyBlueCity {
    game: BlueCity,
}
impl Simulation for LegacyBlueCity {
    type Input = Intent;
    fn step(&mut self, input: &Intent) {
        self.game.step(input);
    }
    fn state_hash(&self) -> u64 {
        vesper3d::two_d::hash_json(&self.capture())
    }
}
impl Snapshot for LegacyBlueCity {
    const KIND: &'static str = "bluecity";
    const VERSION: u32 = 1;
    type State = LegacyState;
    fn capture(&self) -> LegacyState {
        let s = &self.game.state;
        LegacyState {
            city: s.city.clone(),
            tick: s.tick,
            clock: s.clock,
            speed: s.speed,
            tool: s.tool,
            category: s.category,
            cam_x: s.cam_x,
            cam_y: s.cam_y,
            zoom: s.zoom,
            overlay: s.overlay,
            tab: s.tab,
            hover: s.hover,
            selected: s.selected,
            anchor: s.anchor,
            notice: s.notice.clone(),
            notice_ticks: s.notice_ticks,
            confirm_new: s.confirm_new,
        }
    }
    fn restore(&mut self, state: LegacyState) -> Result<(), String> {
        let value = serde_json::to_value(state).map_err(|e| e.to_string())?;
        self.game
            .restore(serde_json::from_value(value).map_err(|e| e.to_string())?)
    }
}

#[test]
fn release_one_manual_and_automatic_frames_migrate_and_resume_exactly() {
    use vesper3d::{
        runtime::snapshot,
        viewer::savestate::{self, SaveSlots},
    };
    assert_eq!(<BlueCity as Snapshot>::KIND, "bluecity");
    assert_eq!(<BlueCity as Snapshot>::VERSION, 2);
    let mut game = paused();
    tab(&mut game, 2);
    tap(&mut game, layout::tax_plus(0));
    tap(&mut game, layout::policy(2));
    tap(&mut game, layout::BORROW);
    tap(&mut game, layout::CLOSE_PANEL);
    tap(&mut game, layout::speed(2));
    for _ in 0..487 {
        game.step(&Intent::default());
    }
    assert!(game.state.city.month >= 6);
    let legacy = LegacyBlueCity { game };
    assert_ne!(
        legacy.state_hash(),
        legacy.game.state_hash(),
        "adding the UI fields changes the authoritative hash, requiring migration"
    );
    let folder = std::env::temp_dir().join(format!(
        "bluecity-v1-save-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let slots = SaveSlots::new(&folder);
    snapshot::save_to_slot(&legacy, &slots, "manual", "Manual checkpoint").unwrap();
    snapshot::autosave(&legacy, &slots, 3, "Automatic checkpoint").unwrap();
    for (slot, label) in [
        ("manual", "Manual checkpoint"),
        ("auto", "Automatic checkpoint"),
    ] {
        let bytes = std::fs::read(slots.path(slot).unwrap()).unwrap();
        let frame = savestate::decode(&bytes).unwrap();
        assert_eq!(frame.header.kind, "bluecity");
        assert_eq!(frame.header.version, 1);
        let envelope: serde_json::Value = serde_json::from_slice(frame.payload).unwrap();
        assert!(envelope["state"].get("panel_open").is_none());
        assert!(envelope["state"].get("palette_open").is_none());
        for field in [
            "draft_seed",
            "map_style",
            "seed_replace",
            "world_seed",
            "world_style",
        ] {
            assert!(envelope["state"].get(field).is_none());
        }
        assert_eq!(envelope["hash"], format!("{:016x}", legacy.state_hash()));
        // The engine verifies the old frame and then applies the game's
        // migration; a raw JSON deserialize does not exercise this contract.
        let mut upgraded = BlueCity::new(99);
        let (header, _) = snapshot::load_from_slot(&mut upgraded, &slots, slot).unwrap();
        assert_eq!(header.label, label);
        assert!(!upgraded.state.panel_open && !upgraded.state.palette_open);
        assert_eq!(upgraded.state.draft_seed, "7");
        assert_eq!(upgraded.state.map_style, MapStyle::River);
        assert!(upgraded.state.seed_replace);
        assert_eq!(upgraded.state.world_seed, None);
        assert_eq!(upgraded.state.world_style, None);
        assert_eq!(upgraded.state.city, legacy.game.state.city);
        let mut reference = BlueCity::new(111);
        reference.restore(legacy.game.capture()).unwrap();
        assert_eq!(upgraded.state_hash(), reference.state_hash());
        for _ in 0..960 {
            upgraded.step(&Intent::default());
            reference.step(&Intent::default());
        }
        assert_eq!(upgraded.state.city, reference.state.city);
        assert_eq!(upgraded.state_hash(), reference.state_hash());
        let current_bytes = snapshot::save(&upgraded, "Migrated checkpoint").unwrap();
        let current_frame = savestate::decode(&current_bytes).unwrap();
        assert_eq!(current_frame.header.version, 2);
        let mut round_trip = BlueCity::new(123);
        snapshot::restore(&mut round_trip, &current_bytes).unwrap();
        assert_eq!(round_trip.state_hash(), upgraded.state_hash());
    }
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn seed_and_style_buttons_generate_reproducible_distinct_maps() {
    let mut first = paused();
    let mut same = paused();
    generate(&mut first, 0, "18446744073709551615");
    generate(&mut same, 0, "18446744073709551615");
    assert_eq!(first.state.city, same.state.city);
    assert_eq!(first.state.world_seed, Some(u64::MAX));
    assert_eq!(first.state.world_style, Some(MapStyle::River));
    assert_eq!(first.state.city.money, 60_000);
    assert_eq!(first.state.speed, 0);
    let terrain = |g: &BlueCity| {
        g.state
            .city
            .tiles
            .iter()
            .map(|t| t.terrain)
            .collect::<Vec<_>>()
    };
    let mut maps = vec![terrain(&first)];
    for (style_id, style) in MapStyle::ALL.into_iter().enumerate().skip(1) {
        let mut game = paused();
        generate(&mut game, style_id, "18446744073709551615");
        assert_eq!(game.state.world_style, Some(style));
        let next = terrain(&game);
        assert!(
            maps.iter().all(|old| *old != next),
            "each map style should shape a different region"
        );
        maps.push(next);
    }
}

#[test]
fn invalid_seed_input_never_replaces_the_existing_town_and_zero_is_valid() {
    let mut game = paused();
    let town = game.state.city.clone();
    tab(&mut game, 0);
    tap(&mut game, layout::NEW_CITY);
    tap(&mut game, layout::SEED_FIELD);
    for _ in 0..25 {
        tap(&mut game, layout::seed_digit(9));
    }
    assert_eq!(game.state.draft_seed, "99999999999999999999");
    tap(&mut game, layout::CONFIRM_NEW);
    assert!(game.state.confirm_new);
    assert_eq!(game.state.city, town);
    assert!(game.state.notice.contains("seed"));
    tap(&mut game, layout::SEED_BACKSPACE);
    assert_eq!(game.state.draft_seed.len(), 19);
    tap(&mut game, layout::SEED_CLEAR);
    assert_eq!(game.state.draft_seed, "");
    tap(&mut game, layout::CONFIRM_NEW);
    assert!(game.state.confirm_new);
    assert_eq!(game.state.city, town);
    tap(&mut game, layout::SEED_FIELD);
    tap(&mut game, layout::seed_digit(0));
    tap(&mut game, layout::CONFIRM_NEW);
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.world_seed, Some(0));
    assert_eq!(game.state.city.population, 0);
}

#[test]
fn seed_shuffle_and_first_digit_replacement_follow_deterministic_input() {
    let mut a = paused();
    let mut b = paused();
    for game in [&mut a, &mut b] {
        tab(game, 0);
        tap(game, layout::NEW_CITY);
        tap(game, layout::SEED_SHUFFLE);
        assert!(game.state.draft_seed.parse::<u64>().is_ok());
        assert!(game.state.seed_replace);
    }
    assert_eq!(a.state.draft_seed, b.state.draft_seed);
    tap(&mut a, layout::seed_digit(4));
    assert_eq!(a.state.draft_seed, "4");
    tap(&mut a, layout::seed_digit(2));
    assert_eq!(a.state.draft_seed, "42");
    tap(&mut a, layout::SEED_FIELD);
    tap(&mut a, layout::seed_digit(7));
    assert_eq!(a.state.draft_seed, "7");
}

#[test]
fn generated_world_recipe_survives_rng_changes_and_framed_saves() {
    use vesper3d::runtime::snapshot;
    let mut game = paused();
    generate(&mut game, 0, "3456");
    let [x, y] = grass_rectangle(&game, 1, 1);
    tool(&mut game, 0, 3);
    block(&mut game, x, y);
    let random_before = game.state.city.seed;
    tab(&mut game, 3);
    tap(&mut game, layout::disaster(0));
    assert_ne!(
        game.state.city.seed, random_before,
        "the emergency advances the simulation RNG"
    );
    assert_eq!(game.state.world_seed, Some(3456));
    assert_eq!(game.state.world_style, Some(MapStyle::River));
    let bytes = snapshot::save(&game, "Generated city").unwrap();
    let mut restored = BlueCity::new(99);
    snapshot::restore(&mut restored, &bytes).unwrap();
    assert_eq!(restored.state_hash(), game.state_hash());
    assert_eq!(restored.state.world_seed, Some(3456));
    tab(&mut restored, 0);
    tap(&mut restored, layout::NEW_CITY);
    assert_eq!(
        restored.state.draft_seed, "3457",
        "new map defaults use the original recipe seed"
    );
    assert_eq!(restored.state.world_style, Some(MapStyle::River));
}
