//! Headless save, scenario, bot and performance tooling for BlueCity.
use bluecity::{
    city::{City, MapStyle},
    world::Climate,
    BlueCity,
};
use vesper3d::{
    runtime::snapshot,
    two_d::{GameLogic, Simulation},
};
fn value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|p| p[0] == flag).map(|p| p[1].clone())
}
fn number(args: &[String], flag: &str, default: u32) -> Result<u32, String> {
    value(args, flag)
        .map(|s| {
            s.parse()
                .map_err(|_| format!("{flag} requires a positive integer"))
        })
        .unwrap_or(Ok(default))
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help") {
        println!("bluecity-sim [--seed N] [--map 0..6] [--climate 0..2] [--days N | --years N] [--bot | --stress] [--load FILE] [--save FILE] [--report FILE]\n--stress constructs a dense 128×128 fixture with at least 15,000 building instances, then measures actual daily steps. --bot uses paid public construction rules. --scenario 0..7 starts an authored scenario.");
        return Ok(());
    }
    let mut at = 0;
    while at < args.len() {
        match args[at].as_str() {
            "--bot" | "--stress" => at += 1,
            "--seed" | "--map" | "--climate" | "--days" | "--years" | "--scenario" | "--load"
            | "--save" | "--report" => {
                if args.get(at + 1).is_none_or(|s| s.starts_with("--")) {
                    return Err(format!("{} requires a value", args[at]));
                }
                if args[..at].contains(&args[at]) {
                    return Err(format!("{} was specified twice", args[at]));
                }
                at += 2;
            }
            flag => return Err(format!("Unknown argument: {flag}")),
        }
    }
    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--days") && has("--years") {
        return Err("Choose either --days or --years".into());
    }
    if has("--load")
        && ["--map", "--climate", "--stress", "--scenario"]
            .iter()
            .any(|f| has(f))
    {
        return Err(
            "A loaded city cannot also select a new map, scenario or stress fixture".into(),
        );
    }
    if has("--stress") && ["--map", "--climate", "--scenario"].iter().any(|f| has(f)) {
        return Err("A stress fixture cannot also select a new map or scenario".into());
    }
    let seed = value(&args, "--seed")
        .unwrap_or_else(|| "7".into())
        .parse::<u64>()
        .map_err(|_| "Invalid seed")?;
    let setup_started = std::time::Instant::now();
    let mut game = BlueCity::new(seed);
    game.state.speed = 0;
    if let Some(path) = value(&args, "--load") {
        snapshot::restore(&mut game, &std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    } else if args.iter().any(|a| a == "--stress") {
        game.state.city = bluecity::mayor::dense_fixture(seed, 15000);
    } else if has("--map") || has("--climate") {
        let style = *MapStyle::ALL
            .get(number(&args, "--map", 0)? as usize)
            .ok_or("Map must be 0..6")?;
        let climate = *Climate::ALL
            .get(number(&args, "--climate", 0)? as usize)
            .ok_or("Climate must be 0..2")?;
        game.state.city = City::world(seed, style, climate);
        game.state.world_seed = Some(seed);
        game.state.world_style = Some(style);
        game.state.ui.climate = climate;
        game.state.cam_x = 64;
        game.state.cam_y = 64;
    }
    if let Some(id) = value(&args, "--scenario") {
        let id: usize = id.parse().map_err(|_| "Invalid scenario")?;
        if id >= bluecity::content::SCENARIOS.len() {
            return Err("Scenario must be 0..7".into());
        }
        game.command(bluecity::ui_model::Command::Scenario(id));
        if game
            .state
            .city
            .civic
            .scenario
            .as_ref()
            .is_none_or(|s| s.id as usize != id)
        {
            return Err(game.state.notice.clone());
        }
    }
    let days = if value(&args, "--years").is_some() {
        number(&args, "--years", 1)?
            .checked_mul(360)
            .ok_or("Year count overflow")?
    } else {
        number(&args, "--days", 360)?
    };
    if days == 0 || days > 360000 {
        return Err("Choose 1..360000 days".into());
    }
    BlueCity::prepare_city_runtime(&mut game.state.city);
    let setup_us = setup_started.elapsed().as_micros();
    let initial_buildings = game.state.city.ext.buildings.len();
    let start = std::time::Instant::now();
    let mut samples = Vec::new();
    let bot = if args.iter().any(|a| a == "--bot") {
        Some(bluecity::mayor::govern(&mut game.state.city, days))
    } else {
        for _ in 0..days {
            let t = std::time::Instant::now();
            game.state.city.advance_day();
            samples.push(t.elapsed().as_nanos() as u64);
        }
        None
    };
    let elapsed = start.elapsed();
    game.state.city.validate().map_err(|e| {
        format!(
            "{e} (month {}, treasury {}, loans {})",
            game.state.city.month, game.state.city.money, game.state.city.debt
        )
    })?;
    if let Some(mayor) = &bot {
        samples.clone_from(&mayor.day_samples_ns);
    }
    let mean = samples.iter().map(|&n| u128::from(n)).sum::<u128>() / u128::from(days);
    // Analyze the already collected samples after timing ends, so reports show
    // which actual steps caused spikes without instrumenting simulation work.
    let mut slowest: Vec<_> = samples.iter().copied().enumerate().collect();
    slowest.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let slowest: Vec<_> = slowest
        .into_iter()
        .take(10)
        .map(|(step, ns)| serde_json::json!({"measured_step":step+1,"microseconds":ns/1000}))
        .collect();
    samples.sort_unstable();
    let p99 = samples
        .get(samples.len().saturating_sub(1) * 99 / 100)
        .copied()
        .unwrap_or(0);
    let max = samples.last().copied().unwrap_or(0);
    let bytes =
        snapshot::save(&game, "BlueCity command-line checkpoint").map_err(|e| e.to_string())?;
    let mut resumed = BlueCity::new(0);
    snapshot::restore(&mut resumed, &bytes).map_err(|e| e.to_string())?;
    if resumed.state_hash() != game.state_hash() {
        return Err("Snapshot continuation mismatch".into());
    }
    let report = serde_json::json!({"seed":seed,"days":days,"setup_us":setup_us,"width":game.state.city.width,"height":game.state.city.height,"initial_buildings":initial_buildings,"buildings":game.state.city.ext.buildings.len(),"population":game.state.city.population,"money":game.state.city.money,"elapsed_ms":elapsed.as_millis(),"mean_day_us":mean/1000,"p99_day_us":p99/1000,"max_day_us":max/1000,"target_day_us":4000,"target_met":max<4000000,"save_bytes":bytes.len(),"hash":format!("{:016x}",game.state_hash()),"bot":bot,"snapshot_roundtrip":true,"financial_defaults":game.state.city.civic.events[bluecity::content::CivicEvent::DeclaredBankruptcy as usize]});
    let mut report = report;
    report["slowest_steps"] = serde_json::json!(slowest);
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if let Some(path) = value(&args, "--report") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap())
            .map_err(|e| e.to_string())?;
    }
    if let Some(path) = value(&args, "--save") {
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("BlueCity simulation: {e}");
        std::process::exit(1);
    }
}
