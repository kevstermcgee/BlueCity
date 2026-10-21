# BlueCity

Build infrastructure. Grow neighborhoods. Shape a thriving city.

BlueCity is an original, open-ended city management game made with BlueEngine. Build a riverside town into a busy city by balancing homes, jobs, transportation, utilities, public services and the budget. Its compact grid simulation draws on the classic SimCity approach and newer city builders: zone land, make it useful, and watch the community develop.

## Play

For Windows, install and play BlueCity through the [BlueEngine Games launcher](https://kevstermcgee.github.io/BlueEngineGames/games/bluecity/). A [standalone Windows x64 installer](https://github.com/kevstermcgee/BlueEngineGames/releases/download/bluecity-v0.2.0/bluecity-setup-windows-x64.exe) is also available. Installed saves and settings live in your user data folder.

The earlier Linux development build has a BlueCity Desktop shortcut. To use its archive, extract `dist/BlueCity-linux-x86_64.zip` and run `./bluecity` from that folder.

To run from source in the project directory:

```sh
cargo run --release
```

The launcher edition is a Windows x64 installer. It installs for your user and keeps progress in your user data directory. The initial Linux and browser development packages are retained locally; the BlueEngine Games launcher distributes Windows installers.

The city fills the window, with compact, translucent controls floating over the map. Management drawers and the tool palette start closed. Click **City**, **Maps**, **Budget** or **Guide** at the upper right to open a drawer; click the active tab again or its **×** to close it. Along the bottom, click a category to open its tools. Choosing a tool closes the palette so you can place it on the map.

The starter town gives you an existing street network and a small community to improve. For a fresh landscape, open **City** or **Guide** and choose **New map…**. Select **River**, **Coast**, **Islands** or **Highlands**, enter a seed, then click **Generate map**. This replaces the current town with a blank region, $60,000 and the calendar paused. **Keep this town** cancels. Save a manual checkpoint first if you want to keep the current town. There is no fixed ending: keep growing, repair a disaster, or redesign a neighborhood at your own pace.

Each style offers different geography: a winding river, a broad coastline, an archipelago or highlands with lakes and ridges. Seeds change shorelines and topography as well as vegetation. Enter a whole-number seed from **0** through **18446744073709551615**, using the keyboard or on-screen number buttons. Clicking the seed field makes the next digit replace its contents. **Clear** or **Ctrl+A** clears it; **Back** or Backspace removes the last digit. **New seed** chooses another seed. Reuse the same style and seed to recreate identical terrain. Generated maps retain their original style and seed in saves; open **Guide** to read that recipe later.

## First neighborhood

1. Pause the calendar while you plan. Extend a road from the existing network, then reserve room for homes, shops, industry and public buildings.
2. Open **Zones** in the bottom dock and choose residential, commercial or industrial zoning. The palette closes; click one corner and then the opposite corner to paint a rectangle. Buildings develop over time when access, demand and utilities support them.
3. Give the neighborhood connected roads, electricity, water and sewage capacity. Use utility overlays to find gaps. Keep polluting facilities away from homes. Connect a road to the map edge to give a new blank town regional access.
4. Resume time and open **City** to check development demand. More industry creates jobs; more housing brings workers and shoppers. Build a little at a time so maintenance costs stay manageable.
5. Add parks and services as the town grows. Check traffic, pollution, crime and land value before choosing where to invest next.
6. Open **Budget** regularly. Adjust taxes, service funding and policies, and keep a reserve for emergencies. A loan can fund infrastructure, but repayments reduce future flexibility. Close the drawer when you want a clear view of the city.

## Controls

| Action | Control |
| --- | --- |
| Start / resume | Click or Enter |
| Pan the map | WASD, arrow keys, or controller directions |
| Choose a build tool | Open a bottom category, then choose a tool; the palette closes |
| Open or close a management drawer | Click City, Maps, Budget or Guide at the upper right; click the active tab again or × to close |
| Build or inspect a tile | Select a tool, then click the map |
| Zone a rectangle | Click the first corner, then the opposite corner |
| Build a transport or utility line | Click the start, then the endpoint; different rows and columns produce an L-shaped route |
| Zoom / recenter | Click - / + or Center at the bottom right |
| Cancel a placement | Click Inspect in the bottom dock |
| Inspect a block | Choose Inspect, click a block, then open City for its details; Overview returns to city statistics |
| Bulldoze | Choose Bulldoze in the bottom dock, then select the area with two clicks |
| Change speed or pause the calendar | Speed controls at the upper right of the top bar |
| Change a map layer | Open Maps, choose a layer, then close the drawer if desired |
| Generate a seeded landscape | City or Guide → New map… → choose style and seed → Generate map |
| Manual save / load | K / L |
| Pause menu | Esc |
| Restart | R |
| Toggle sound / music | M / N |
| Toggle fullscreen | F / F11 |

Opening a management drawer closes the tool palette, and opening a category closes the drawer. Clicking the same category again closes its palette. **Inspect** and **Bulldoze** remain directly available in the bottom dock. Calendar pause lets you inspect, build and change the budget while monthly growth is stopped.

Controller directions pan the camera. A activates the current pointer location. With no pointer, it applies the tool to the selected tile or camera-center tile; choose tools with the pointer. The in-game Guide explains placement and the first steps for a connected town.

## What you can build and manage

- **Neighborhoods and growth:** residential, commercial and industrial zones, development from low-rise blocks to denser towers, demand, employment, growth and abandonment.
- **Transportation:** roads, avenues, bridges, rail and buses; traffic pressure and access influence development. Airports and seaports support the larger city.
- **Utilities:** power generation and distribution, water supply, sewage and waste capacity. Service shortages constrain growth.
- **Public life:** police, fire protection, healthcare, education, parks and a stadium. Coverage and funding affect neighborhood conditions.
- **City finances:** taxes, construction and maintenance costs, service budgets, loans and policies. The Budget drawer lets you adjust spending and revenue while the top bar keeps treasury funds visible.
- **City conditions:** traffic, pollution, crime, land value and utility overlays reveal problems that a normal map can hide. Maps also shows electricity and water supply totals and a compact population history graph.
- **Emergencies:** fire, flooding, earthquakes, tornadoes and power-plant meltdowns can damage the city. Reconnect infrastructure, restore services and rebuild.
- **Long-running towns:** pause and speed controls, a starter town, four procedural map styles with reproducible seeds, automatic progress and manual engine snapshot saves.

The presentation uses smooth embedded fonts, softened edges and detailed terrain, shorelines and buildings. Road surfaces stay clear of foliage. Translucent drawers leave more room for the growing city. Version 1 saves migrate to version 2 while preserving existing towns and their future simulation.

BlueCity uses an aggregate, tile-based simulation. Residents, jobs, traffic and service coverage are calculated at the city or neighborhood level; the game does not simulate a separate persistent agent for each resident or vehicle. It is its own city builder, with a broad feature set inspired by the genre, rather than an exact reproduction of another game's rules or artwork. The map is 48 by 48 tiles. Campaign scenarios, regional cities and monster/UFO disasters are outside this version.

## Build and verify

```sh
cargo test --no-default-features
cargo clippy --locked --all-targets -- -D warnings
scripts/blue ship --no-install
```

The project declares Linux and Windows native targets. Building requires the exact BlueEngine revision pinned by `scripts/tooling.json`, checked out beside this repository as `../bluecity-engine`. The Windows release workflow runs formatting, game tests, Clippy, package/resource/startup checks and installer install/upgrade/uninstall tests before publication. Current validation and known limitations are recorded in [STATUS.md](STATUS.md).

An original, one-minute Riverlight soundtrack pairs warm chord pads, soft arpeggios and a sparse melody; music and sound have separate toggles.

The opening BlueEngine Games title card uses centered text. The skyline icon and game artwork are original; the retired white-rat branding is not used.
