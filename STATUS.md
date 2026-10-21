# BlueCity 0.2.0

## Full-screen map and seeded regions

The city now fills the window behind compact translucent controls. Management drawers and the build palette start closed and toggle on demand. Embedded Liberation Sans regular/bold fonts are rasterized for the window size; multisampling, quieter terrain colors, detailed facades and shorelines improve clarity. Tree trunks and canopies keep clear of roads, bridges and rail.

New map creation offers River, Coast, Islands and Highlands styles. Enter an unsigned 64-bit seed by keyboard or keypad, or choose New seed. The same style and seed reproduce the same initial landscape. Generation starts a blank, paused town with $60,000; its original recipe remains available after saving and after the simulation's random stream advances.

All 39 Rust tests pass, including generated-map usability/reproducibility, the new floating controls, and migration of actual version 1 manual and automatic save frames. Snapshot payload version 2 explicitly migrates older towns without changing their city state or future simulation. Native software captures were reviewed at 960×540, 1280×720 and 1920×1080, including typed seed entry and map creation. Current evidence is under `.blue-check/ui-polish/`. Physical device/audio review remains separate.

The launcher delivery below records version 0.1.1; 0.2.0 publication will use a separately verified immutable installer.

## Launcher delivery

BlueCity 0.1.1 is published in the [BlueEngine Games launcher catalog](https://kevstermcgee.github.io/BlueEngineGames/games/bluecity/). The live feed was verified with 52 games, preserving the packages of all 51 previous games. Its Windows installer is pinned to SHA256 `f56a15b54ed9f2de9dd02228ec2921742ff2778de298dc42165f0f67c97a3b57` and runtime source `7a7a79c81c3fb1007df4e7e8f131163eea66e854`.

The [Windows release workflow](https://github.com/kevstermcgee/BlueCity/actions/runs/39540610720) and [launcher deployment](https://github.com/kevstermcgee/BlueEngineGames/actions/runs/39541336825) passed. Release bytes were checked against both the original release and the trusted launcher release. Windows startup captures confirm the text-only studio card and a start-menu prompt that fits in fullscreen and windowed views. The live game page, thumbnail and three guide captures were fetched successfully. Local publication evidence is recorded in `.blue-check/launcher-publication.json`.

The launcher edition targets native Windows x64 installers and Linux verification. Its runtime and shared installer tooling are pinned to public BlueEngine revision `1a5b696ee653179c6898b1c4195cb1d3d2715d6e`, with a reproducible sibling dependency path. The complete guide includes three authentic gameplay captures and the original human creation request. The Windows workflow verifies formatting, tests, Clippy, declared package/resources, the text-only studio card, fullscreen controls and installer installation/upgrade/uninstallation before creating an immutable release. Launcher catalog publication pins the completed installer hash and exact source commit.

The evidence below records the original development build on engine `d7159cbc01ec`; its browser package is archival and is not a launcher product. New Windows delivery evidence is produced by `.github/workflows/release.yml`. Physical device/audio review remains separate from automated packaging.


BlueCity is a playable, original BlueEngine city builder for Linux desktop and the browser. It uses a deterministic 48 by 48 tile simulation and an isometric map with animated traffic, industry, water and wind turbines.

## Implemented

- Connected roads, avenues and bridges; working rail routes and road/rail crossings; bus transit.
- Residential, commercial and industrial zoning with demand, employment, abandonment and automatic density upgrades.
- Coal, wind and nuclear power; distribution networks, water pumps/pipes, sewage and waste capacity.
- Police, fire crews, healthcare, schools, universities, parks and stadiums, with coverage and funding effects.
- Taxes, construction and maintenance spending, three $10,000 bonds, interest, repayment, department funding and four policies.
- Traffic, pollution, crime, land value, health, education, utility shortages, milestone grants and 120 months of history.
- Airports, seaports and their economic effects; fire, flood, earthquake, tornado and reactor meltdown emergencies.
- Terrain and tree tools, starter town, confirmed blank-valley creation, time controls, zoom/pan, nine map views and block inspection.
- Engine-owned automatic progress and manual snapshot saves; original icons and Riverlight music. The BlueEngine Games opening card is centered text.

## Verified on 2026-10-20

All 28 Rust tests pass in the final locked builds: three game/replay/snapshot tests, eighteen simulation tests, six pointer-driven player workflow tests and one identity test. The starter remains solvent for ten simulated years; the observed baseline grows from 2,232 to about 7,600 residents.

`BE2_TOOLS=/srv/storage/cache/cargo/default/itest/be2-tools python3 scripts/check.py` passes the complete game check and native ship gate. `scripts/blue ship` passes identity, full icon format/art, declared package hashes, Desktop/application shortcuts and isolated executable smoke capture under Xvfb. Windows-specific resource/window checks are not applicable to the Linux target. The check tool override selects the current engine authoring binary because an older debug binary on this machine lacks the audio command.

`python3 tools/be2.py web build /home/kevin/bluecity` passes with separate real Chromium desktop and emulated touch profiles. Evidence includes matching native/browser replay hashes, pointer/keyboard/emulated-controller input, start/pause/restart, focus loss/return, manual and automatic saves, reload and failed-write preservation, audio decode/play submissions, offline installation/reload and interrupted-update recovery. The final package manifest records the source hash and every shipped file hash.

Native world captures, all diagnostic overlays, budget, guide, utility palette and browser portrait/landscape layouts were inspected. Riverlight passes checksum/PCM/loop-quality checks with no detected clipping or clicks. No physical controller/mobile-device or human listening certification is implied. Browser performance measurements used software rendering on this host, with concurrent verification load; hardware performance is unmeasured.

## Artifacts and evidence

- Desktop launcher: `/home/kevin/Desktop/BlueCity.desktop`.
- Linux program: `dist/bluecity`; run with the package folder as the working directory.
- Linux archive: `dist/BlueCity-linux-x86_64.zip`.
- Browser package: `dist/web/`; serve over localhost HTTP or HTTPS. Browser archive: `dist/BlueCity-browser.zip`.
- Native final check: `.blue-check/final-native-check.log` and its referenced report.
- Browser final check: `.blue-check/final-web-build.log`, `.blue-check/web/browser.json`, `.blue-check/web/mobile.json`.
- Visual captures: `.blue-check/native-preview`, `.blue-check/ui-review`, `.blue-check/web-preview` and `.blue-check/smoke-*`.

The model uses aggregate residents, jobs and commuting pressure rather than a persistent individual agent for every citizen or vehicle. It is a broad classic-style city simulator, with a compact map and five disaster types. Campaign scenarios, regional city simulation and monster/UFO events are not included. At the original development gate, Windows and macOS were not validated; the launcher delivery workflow now provides separate Windows evidence. macOS is not a declared target.
