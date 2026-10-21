//! Player-facing workflows: every game action follows the public pointer/Intent route.
use bluecity::{
    city::{Kind, Terrain, SIZE},
    BlueCity, Tool,
};
use vesper3d::two_d::{GameLogic, Intent, Point, Simulation, Snapshot};

fn click(game: &mut BlueCity, x: i32, y: i32) {
    game.step(&Intent {
        pointer: Some(Point::new(x, y)),
        action: true,
        ..Intent::default()
    });
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
    click(&mut game, 20, 50);
    assert_eq!(game.state.speed, 0);
    game
}
fn blank() -> BlueCity {
    let mut game = paused();
    click(&mut game, 650, 398);
    assert!(game.state.confirm_new);
    click(&mut game, 650, 343);
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.speed, 0);
    game
}
fn grass_rectangle(game: &BlueCity, width: i32, height: i32) -> [i32; 2] {
    for y in 12..36 {
        for x in 12..36 {
            let clear = (y..y + height).all(|ty| {
                (x..x + width).all(|tx| {
                    let p = game.project(tx, ty);
                    let tile = game.state.city.tile(tx, ty).unwrap();
                    (12..580).contains(&p.x)
                        && (76..358).contains(&p.y)
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
    click(&mut game, 343, 414);
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
    click(&mut game, 150, 384); // Transport category.
    click(&mut game, 49, 414); // Road.
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

    click(&mut game, 50, 384); // Zoning category.
    click(&mut game, 49, 414); // Homes.
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
    click(&mut game, 395, 50);
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
    click(&mut game, 470, 50); // Recenter.
    assert_eq!([game.state.cam_x, game.state.cam_y], [24, 24]);
    for _ in 0..3 {
        click(&mut game, 185, 50);
    }
    assert_eq!(game.state.zoom, 3);
    for zoom in (1..=3).rev() {
        assert_eq!(game.state.zoom, zoom);
        let p = game.project(23, 24);
        click(&mut game, p.x - 1, p.y - 1); // Inside the same centered diamond.
        assert_eq!(game.state.selected, Some([23, 24]));
        click(&mut game, 155, 50);
    }
    assert_eq!(game.state.zoom, 1);
    click(&mut game, 155, 50);
    assert_eq!(game.state.zoom, 1);
    let selected = game.state.selected;
    click(&mut game, 590, 373); // Gap between map and tool categories.
    assert_eq!(game.state.selected, selected);
    assert_eq!(game.state.city.month, month);
}

#[test]
fn map_layers_and_municipal_budget_controls_follow_pointer_input() {
    let mut game = paused();
    click(&mut game, 672, 80); // Maps tab.
    assert_eq!(game.state.tab, 1);
    click(&mut game, 650, 279); // Crime map.
    assert_eq!(game.state.overlay, 6);
    click(&mut game, 280, 50); // Cycle the map layer.
    assert_eq!(game.state.overlay, 7);
    click(&mut game, 650, 159); // Electricity map.
    assert_eq!(game.state.overlay, 1);

    click(&mut game, 720, 80); // Budget tab.
    assert_eq!(game.state.tab, 2);
    let tax = game.state.city.taxes[0];
    click(&mut game, 770, 170);
    assert_eq!(game.state.city.taxes[0], tax + 1);
    click(&mut game, 620, 170);
    assert_eq!(game.state.city.taxes[0], tax);
    for _ in 0..7 {
        click(&mut game, 770, 277);
    }
    assert_eq!(game.state.city.funding[1], 150);
    for _ in 0..20 {
        click(&mut game, 620, 277);
    }
    assert_eq!(game.state.city.funding[1], 0);
    for y in [376, 392, 408, 424] {
        click(&mut game, 650, y);
    }
    assert_eq!(game.state.city.policies, [true; 4]);
    click(&mut game, 650, 408);
    assert_eq!(game.state.city.policies, [true, true, false, true]);

    let treasury = game.state.city.money;
    for _ in 0..3 {
        click(&mut game, 650, 343);
    }
    assert_eq!(game.state.city.money, treasury + 30_000);
    assert_eq!(game.state.city.debt, 30_000);
    click(&mut game, 650, 343);
    assert_eq!(game.state.city.money, treasury + 30_000);
    assert!(game.state.notice.contains("limit"));
    for _ in 0..3 {
        click(&mut game, 745, 343);
    }
    assert_eq!(game.state.city.money, treasury);
    assert_eq!(game.state.city.debt, 0);
    click(&mut game, 745, 343);
    assert_eq!(game.state.city.money, treasury);
    assert!(game.state.notice.contains("no outstanding"));
}

#[test]
fn starting_a_new_valley_requires_confirmation_and_starts_paused() {
    let mut game = BlueCity::new(7);
    let population = game.state.city.population;
    let seed = game.state.city.seed;
    assert!(population > 0);
    click(&mut game, 650, 398);
    assert!(game.state.confirm_new);
    assert_eq!(game.state.city.population, population);
    click(&mut game, 650, 373); // Keep this town.
    assert!(!game.state.confirm_new);
    assert_eq!(game.state.city.seed, seed);
    assert_eq!(game.state.city.population, population);
    click(&mut game, 650, 398);
    click(&mut game, 650, 343); // Start the blank valley.
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
    click(&mut game, 720, 80);
    click(&mut game, 770, 170);
    click(&mut game, 650, 376);
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
