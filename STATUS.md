# BlueCity 0.3.0 release candidate — not published

BlueCity 0.3.0 is an unpublished native release candidate. The public BlueEngine Games launcher currently distributes the verified 0.2.0 release. Every item in the user's expansion checklist was treated as required regardless of its emoji. [FEATURES.md](FEATURES.md) records the feature evidence and limits.

Before the final bot-maintenance patch, the recorded Linux shipping harness passed its project check and isolated packaged-game smoke. The nested default-feature `cargo test --locked` log contains **148 passing Rust tests** (8 library, 140 integration), including native-camera matrix checks, old-save migration, invalid-save atomicity, actual services/transit, one-way pointer routing, government and exact future continuation. Strict all-target Clippy and both audio-bank/content checks passed. The outer shipping wrapper reports zero tests because the test process is nested; the count comes from the actual project log.

Recorded local receipts are `.blue-check/expansion/final-linux-shipping.log` and `final-clippy.log`; the project report/log is `.blue-check/20261023T135605006271Z-89u4d_5z/report.json` and `4.log`. The Linux package contains ten declared, hashed files. Its executable SHA256 is `12693cca1f0597ec475101ac02dd3b08353005baaba7b3ae080b772e069281d1`, matching the final native capture receipts. Windows results are still pending and are not inferred from Linux success.

The world camera is upright and agrees with off-center picking: a native click at (496, 333) selects Clock Tower block (77, 69). Final Settings automation saved music 0/effects 50/ambience 100, text 150% and display 100%, with the polished large-text capture inspected. A starter-town milestone produced a clean automatic postcard without an F12 action. Receipts are under `.blue-check/expansion/upright-picking`, `release-settings` and `release-automatic-postcard`. Captures use software rendering and synthetic input with muted audio; physical devices and audible output remain unverified.

The pre-maintenance-patch paid-rules bot built from a blank 128×128 River region for **100 years**: peak population 70,160; end population 9,425; treasury $6,460,673; zero bankruptcy months; exact 2,758,877-byte save round trip. A packaged-binary resume audit advanced it one more day and confirmed **zero financial defaults**, 9,379 residents, unchanged funds and another exact round trip. A separate dense 128² century run retained 208,235 residents, $75,073,282 and an exact 3,731,423-byte save. These are reliability/balance observations, not a human fun rating.

The latest quiet 15,009-building run on the four-core Intel N97 recorded **2.659 ms mean, 4.280 ms p99 and 4.829 ms maximum per simulated day** over 360 days, zero defaults and exact 3,688,655-byte save; the state hash is `e29c094ebf193923`. **The strict 4 ms target fails locally.** The measurement includes normal, periodic and monthly steps, with setup reported separately. See `.blue-check/expansion/world-env-facts-profile-360.json`. The Windows workflow now enforces an independent 128²/15,000-building 360-day strict timing run plus both 100-year runs before it can publish an installer.

A later mayor-only patch makes ruined maintenance depots stop blocking replacement and permits bounded paid clearing of unusable empty lots. The patch passed five focused mayor contract tests and strict focused Clippy. These older receipts do not certify the final planner.

Open release gates: stable final shipping evidence and a fresh final-planner century, all declared Windows checks and installer lifecycle, strict Windows performance, independent artifact/hash inspection, immutable release and canonical launcher publication, then engine feedback submission. The current 54-game catalog must retain all 53 other games and BlueCity's prior release history. No 0.3.0 installer or launcher publication is claimed here.

The sections below retain historical 0.2.0 and initial-development evidence; their old feature/test counts and browser artifacts do not certify the expansion. Browser packages are archival, not a current launcher product.

---

# BlueCity 0.2.0

## Full-screen map and seeded regions

The city now fills the window behind compact translucent controls. Management drawers and the build palette start closed and toggle on demand. Embedded Liberation Sans regular/bold fonts are rasterized for the window size; multisampling, quieter terrain colors, detailed facades and shorelines improve clarity. Tree trunks and canopies keep clear of roads, bridges and rail.

New map creation offers River, Coast, Islands and Highlands styles. Enter an unsigned 64-bit seed by keyboard or keypad, or choose New seed. The same style and seed reproduce the same initial landscape. Generation starts a blank, paused town with $60,000; its original recipe remains available after saving and after the simulation's random stream advances.

All 40 Rust tests pass, including generated-map usability/reproducibility, the new floating controls, and migration of actual version 1 manual and automatic save frames. Snapshot payload version 2 explicitly migrates older towns without changing their city state or future simulation. Native software captures were reviewed at 960×540, 1280×720 and 1920×1080, including typed seed entry and map creation. Current evidence is under `.blue-check/ui-polish/`. Physical device/audio review remains separate.


## Launcher delivery

BlueCity 0.2.0 is live in the [BlueEngine Games launcher catalog](https://kevstermcgee.github.io/BlueEngineGames/games/bluecity/). The live 52-game feed, installer URL, thumbnail, game page and all five guide captures were verified. All 51 other game packages are unchanged; BlueCity 0.1.1 and 0.1.0 remain in release history.

The installer is pinned to SHA256 `6307da6fef33244d3cb087773cb3420ede23124ca7944486e7842cf1a10f3db0` and runtime source `de063b7e4f4c298dbedbb699332771e435a8ce62`. The [Windows release workflow](https://github.com/kevstermcgee/BlueCity/actions/runs/39544750852) and [launcher deployment](https://github.com/kevstermcgee/BlueEngineGames/actions/runs/39545429016) passed. Independently inspected Windows artifacts confirm readable translucent gameplay, text-only studio cards, six correctly hashed package files and a successful installer lifecycle. Source and canonical launcher release bytes match exactly. Publication evidence is recorded in `.blue-check/ui-polish/publication.json`.

The launcher edition targets native Windows x64 installers and Linux verification. Its runtime and shared installer tooling are pinned to public BlueEngine revision `1a5b696ee653179c6898b1c4195cb1d3d2715d6e`, with a reproducible sibling dependency path. The complete guide includes five authentic gameplay captures and the original human creation request. The Windows workflow verifies formatting, tests, Clippy, declared package/resources, the text-only studio card, fullscreen controls and installer installation/upgrade/uninstallation before creating an immutable release. Launcher catalog publication pins the completed installer hash and exact source commit.

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
