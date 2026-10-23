# BlueCity

Build infrastructure. Grow neighborhoods. Shape a thriving city.

BlueCity is an original BlueEngine city management game inspired by classic SimCity and Cities: Skylines. Roads, homes, workplaces and public services share a saved, deterministic simulation. The city fills the window, with readable, translucent overlays that open when you need them.

BlueCity **0.3.0** is published with a verified Windows x64 installer. The requested city-building systems are implemented at the model scope described below; [FEATURES.md](FEATURES.md) records behavior and limits, and [STATUS.md](STATUS.md) records release and launcher deployment evidence.

## Play

Download the [BlueCity 0.3.0 Windows installer](https://github.com/kevstermcgee/BlueEngineGames/releases/download/bluecity-v0.3.0/bluecity-setup-windows-x64.exe), or use the [BlueEngine Games launcher and player guide](https://kevstermcgee.github.io/BlueEngineGames/games/bluecity/). The installer is per-user; saves and settings live in your user data directory. Launcher rollout verification is recorded in [STATUS.md](STATUS.md).

For a source build, check out the exact BlueEngine revision in `scripts/tooling.json` beside this repository as `../bluecity-engine`, then run:

```sh
cargo run --locked --release --bin bluecity
```

## Your first neighborhood

1. Start with the existing town, or open the Mayor's desk and choose **New map** or **Surprise me**. Fresh regions have 128×128 tiles, seven map styles and three climates. Reusing the seed, style and climate recreates the landscape. New maps start paused so you can plan.
2. Connect a road to the map edge. Add reliable electricity and water, leave room for sewage and garbage collection, then zone a small mix of homes, shops and industry beside connected streets.
3. Resume the calendar. Demand bars explain what encourages or prevents development. Inspect a building to see its problems and the requirements for its next level. Homes need useful jobs and services; employers need workers with suitable education.
4. Expand in small steps. The twenty data views reveal shortages, desirability, pollution and traffic. Thirteen service coverage views and **Place for me** help choose sites. Coverage and actual routed service vehicles both matter.
5. Keep a cash reserve. Taxes, department funding, utility upkeep, loans, bonds, ordinances and trade agreements affect the budget. The yearly review, advisors and Mayor's Notebook explain how the city is doing.

Save a manual checkpoint before replacing a town. Released version 1 and 2 towns migrate to the expansion format; malformed saves are refused without replacing the live city. Automatic progress and manual checkpoints are separate.

## Controls

The Settings window has one rebindable table for all seventeen shortcuts, plus music, effects and ambience volume, text size, display scaling, calm view and autosave.

| Action | Default |
| --- | --- |
| Pan | WASD or controller directions; arrows can be assigned in Settings |
| Build or inspect | Select a tool and click the map |
| Pause or resume calendar | Space or top speed controls |
| Save / load | K / L |
| Mayor's desk / pause menu | Escape |
| Zoom | Equal / Minus, or on-screen controls |
| Center camera | Home |
| Inspect / bulldoze | I / B |
| Photo and postcard | F12 |
| Full-screen map | Tab |
| Underground view | Below button |
| Skip to next event | Period |
| Fullscreen | F11 |

Open a category along the bottom to see its paginated catalog. Tools close after selection. Choose line, curve, grid, rectangle, brush or roadside zoning in the geometry strip. Most multi-tile tools use a start and endpoint. The cursor shows the exact footprint, cost and any refusal before you pay. For a transit line, choose the bus-line geometry, click existing stops in order and finish in Transit. Management windows, cards and the minimap mask their own area so clicks do not build through them.

## What the expansion contains

- **Land and growth:** rivers, lakes, erosion, forests, ore and oil; eight landscaping tools; tree maturation; four desirability views; wind-carried pollution and cleanup; construction, five density levels, building eras, decay, abandonment and rebuilding.
- **People and economy:** households, age groups, schooling, births, deaths and migration; four job education tiers; sixteen happiness factors; commercial trade, tourism, hotels, farms, resource industry and neighboring contracts.
- **Infrastructure:** seven power technologies and physical distribution, purchased power, breakdowns and brownouts; water and sewage; waste storage, recycling and incineration; finite hospital, prison and burial capacity; real routed emergency and collection missions.
- **Transport:** directed streets, avenues, bridges, tunnels, roundabouts and ramps; congestion-aware trips, wear and repair; buses, rail, subway and freight; airports, seaport and marina. Lines have fares, capacity and saved colors.
- **Government:** seventeen ordinances, election-earned slots, districts and Heritage protection; budget views, loans, bonds, overdraft and three bankruptcy choices; evaluation, medals and sixteen history graphs.
- **Emergencies and story:** ten scheduled disaster types, warnings, relief and routed repair crews; twenty-five milestones, twelve reward requests, eight achievements and eight scenarios; advisors, hints, newspaper, Mayor's Cat, Chirper and citizen profiles; phased megaprojects and the Mayor's Notebook.
- **Presentation:** original procedural isometric buildings and facilities, construction cranes, illuminated windows, climate trees, vehicles and emergency lights, weather and seasons, underground networks, original Riverlight music, city ambience and effects. Photo mode writes clean postcards; milestones can capture them automatically.

The simulation uses occupied lots and households, with bounded samples of trips and vehicles. Roads and large facilities follow a grid, and neighboring cities are trade contracts. The complete requested feature list is implemented within that declared scope; this original game does not promise exact commercial-game parity. [FEATURES.md](FEATURES.md) distinguishes behavior tests, source review and presentation limits.

## Verify and balance

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/check.py --content-only
scripts/blue ship --no-install
cargo run --locked --release --no-default-features --bin bluecity-sim -- --map 0 --bot --years 100 --report bot.json --save town.bcsave
cargo run --locked --release --no-default-features --bin bluecity-sim -- --stress --days 360 --report performance.json
```

`bluecity-sim --help` also documents seeds, climates, scenarios, checkpoints and reports. The dense performance fixture is explicitly constructed test data; the bot builds through paid public rules. Reports separate elapsed run time from actual daily simulation samples and include mean, p99, maximum, building count, save size and snapshot round-trip results. Timing depends on hardware and load.

The [published Windows run](https://github.com/kevstermcgee/BlueCity/actions/runs/39984257601) passed all **158 Rust tests**, both 100-year simulations, formatting/Clippy/content checks, isolated package launch, executable resources, the centered text-only studio opening and installer install/reinstall-upgrade/uninstall. The 128×128, 15,009-building, 360-day benchmark measured **2.057 ms mean, 3.298 ms p99 and 3.932 ms maximum** on that runner. Other hosts and longer stress runs can exceed 4 ms; [STATUS.md](STATUS.md) preserves those results and the disclosed release criterion. This is measured evidence, not a universal timing guarantee.

Linux and Windows are declared native targets. Software capture does not certify audible playback or physical controller hardware. Artwork and the skyline icon are original; the retired rat branding is absent.
