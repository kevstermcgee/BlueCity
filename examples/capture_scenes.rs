//! Explicit visual regression fixtures, never used as player starting cities.
use bluecity::{
    city::{City, Kind, MapStyle, Terrain, Tile},
    world::{Climate, Weather, WeatherKind},
    BlueCity,
};
use vesper3d::{runtime::snapshot, two_d::GameLogic};
fn main() {
    let out = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&out).unwrap();
    // A real starter town immediately before its first monthly goal review.
    // Capture this without pressing Photo to verify automatic milestone cards.
    let mut milestone = BlueCity::new(42);
    milestone.state.city.ext.day = 29;
    milestone.state.city.ext.minute_of_day = 1260;
    milestone.state.city.civic.day = 29;
    milestone.state.clock = 239;
    std::fs::write(
        format!("{out}/automatic-postcard.bcsave"),
        snapshot::save(&milestone, "First monthly milestone").unwrap(),
    )
    .unwrap();
    let mut game = BlueCity::new(42);
    let mut city = City::world(42, MapStyle::Delta, Climate::Temperate);
    city.money = 2_000_000;
    city.month = 240;
    city.civic.day = 7200;
    city.ext.day = 7200;
    city.civic.disaster_frequency = 0;
    for i in 0..city.tiles.len() {
        city.tiles[i] = Tile::new(Terrain::Grass);
        city.ext.cells[i].elevation = 0;
        city.ext.cells[i].water_depth = 0;
        city.ext.cells[i].construction = 100;
        city.ext.cells[i].tree_age = 36;
        city.ext.cells[i].age_days = 1080;
    }
    for y in 0..128 {
        city.tiles[y * 128 + 64].kind = Kind::Avenue;
        city.ext.cells[y * 128 + 64].pipe = true;
        city.ext.cells[y * 128 + 64].sewer = true;
    }
    for y in 48..=80 {
        for x in 48..=80 {
            if x % 4 == 0 || y % 4 == 0 {
                city.tiles[y * 128 + x].kind = Kind::Avenue;
            } else if (x + y) % 5 == 0 {
                city.tiles[y * 128 + x].terrain = Terrain::Forest;
            }
        }
    }
    let facilities: Vec<_> = Kind::ALL
        .iter()
        .copied()
        .filter(|k| k.is_building() && !k.is_zone())
        .collect();
    for (n, kind) in facilities.into_iter().enumerate() {
        let x = 49 + (n % 8) * 4;
        let y = 49 + (n / 8) * 4;
        let i = y * 128 + x;
        city.tiles[i] = Tile::new(Terrain::Grass);
        city.tiles[i].kind = kind;
        if matches!(
            kind,
            Kind::Pump
                | Kind::Hydro
                | Kind::Seaport
                | Kind::Marina
                | Kind::SewageOutlet
                | Kind::Beach
                | Kind::WaterfrontPark
        ) {
            let at = (y - 1) * 128 + x;
            city.tiles[at].terrain = Terrain::Water;
            city.ext.cells[at].water_depth = 5;
        }
    }
    for (n, kind) in [
        Kind::Residential,
        Kind::Commercial,
        Kind::Industrial,
        Kind::Farm,
        Kind::Hotel,
    ]
    .into_iter()
    .enumerate()
    {
        for level in 1..=5 {
            let x = 54 + level as usize * 2;
            let y = 66 + n * 2;
            let i = y * 128 + x;
            city.tiles[i] = Tile::new(Terrain::Grass);
            city.tiles[i].kind = kind;
            city.tiles[i].level = level;
            match kind {
                Kind::Residential => {
                    city.tiles[i].population = [0, 24, 64, 144, 280, 480][level as usize]
                }
                _ => city.tiles[i].jobs = level as u32 * 10,
            };
        }
    }
    city.population = city.tiles.iter().map(|t| t.population).sum();
    city.jobs = city.tiles.iter().map(|t| t.jobs).sum();
    city.refresh();
    let mut ext = std::mem::take(&mut city.ext);
    ext.sync_buildings(&city);
    city.ext = ext;
    city.validate().unwrap();
    game.state.city = city;
    game.state.cam_x = 64;
    game.state.cam_y = 64;
    game.state.zoom = 2;
    game.state.speed = 0;
    game.state.world_seed = Some(42);
    game.state.world_style = Some(MapStyle::Delta);
    game.state.notice_ticks = 0;
    game.state.ui.card = None;
    for (name, tick, climate, underground) in [
        ("facilities", 1800, Climate::Temperate, false),
        ("night", 5400, Climate::Tropical, false),
        ("underground", 1800, Climate::Temperate, true),
        ("snow", 1800, Climate::Alpine, false),
        ("rain", 1800, Climate::Tropical, false),
        ("fog", 1800, Climate::Temperate, false),
    ] {
        game.state.tick = tick;
        game.state.city.ext.climate = climate;
        game.state.ui.underground = underground;
        game.state.city.ext.weather = Weather {
            kind: if name == "rain" {
                WeatherKind::Rain
            } else if climate == Climate::Alpine {
                WeatherKind::Snow
            } else {
                WeatherKind::Clear
            },
            temperature: if name == "fog" {
                3
            } else if climate == Climate::Alpine {
                -4
            } else {
                22
            },
            wind_x: 1,
            wind_y: 0,
            rain: if name == "rain" {
                70
            } else if climate == Climate::Alpine {
                60
            } else {
                0
            },
            cloud: if name == "fog" || name == "rain" {
                95
            } else {
                20
            },
        };
        std::fs::write(
            format!("{out}/{name}.bcsave"),
            snapshot::save(&game, name).unwrap(),
        )
        .unwrap();
    }
    game.state.city.ext.climate = Climate::Temperate;
    game.state.city.ext.weather = Weather {
        kind: WeatherKind::Clear,
        temperature: 22,
        wind_x: 1,
        wind_y: 0,
        rain: 0,
        cloud: 10,
    };
    game.state.ui.underground = false;
    for (i, cell) in game.state.city.ext.cells.iter_mut().enumerate() {
        if game.state.city.tiles[i].kind.is_building() {
            cell.construction = match i % 5 {
                0 => 0,
                1 => 35,
                2 => 75,
                _ => 100,
            };
            if i % 13 == 0 {
                cell.abandoned = true;
                cell.condition = 15;
            }
            if i % 19 == 0 {
                cell.rubble = true;
                cell.condition = 0;
            }
        }
    }
    std::fs::write(
        format!("{out}/lifecycle.bcsave"),
        snapshot::save(&game, "Lifecycle").unwrap(),
    )
    .unwrap();
}
