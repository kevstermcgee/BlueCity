# BlueCity 0.3.0 release candidate — not published

BlueCity 0.3.0 remains an unpublished native release candidate. The public BlueEngine Games launcher distributes 0.2.0. Every item in the user's expansion checklist is required regardless of its emoji; [FEATURES.md](FEATURES.md) records behavior, evidence and limits.

The stable-source Linux shipping gate for `5957e9471be711ad4447346e3544016cd79d75b1` passed **153 default-feature Rust tests**, strict all-target Clippy, content/audio checks and isolated packaged-game smoke. The successful display-enabled report records `stable_sources:true` (`.blue-check/expansion/frozen-linux-display-shipping.log`; actual nested test log `.blue-check/20261023T142804851772Z-xovuh695/4.log`). This supersedes the earlier 148-test pre-maintenance receipt and the first display-less smoke attempt, which skipped delivery evidence.

Older Windows run **39969258219** passed its 153-test suite, formatting/Clippy/content, isolated package smoke, icon/resource and clean text-only studio-card startup checks. The native Windows city was reviewed upright. Its 360-day timing result was 2.156 ms mean, 3.508 ms p99 and **4.050 ms maximum: strict failure**. The run was cancelled before completing century and installer gates. These partial results do not certify the newer source.

Later exact performance changes currently have **61 simulation, 21 service and 32 Civic focused tests passing**, strict focused Clippy and byte-identical January codec evidence. Fresh default-feature all-target Clippy also passed (`rc2-clippy.log`, 4.87 seconds). A fresh full combined run and new strict Windows gate are pending; an expected future test total is not a passing result. The timing gate remains unchanged.

The latest quiet 128×128, 15,009-building run on the four-core Intel N97 records **2.394 ms mean, 3.677 ms p99 and 4.084 ms maximum** over 360 days (`.blue-check/expansion/lru-performance-360.json`). **It still fails the strict 4 ms limit by 84 microseconds.** Steps 360 and 240 took 4.084 and 4.064 ms. The CLI now reports ten `slowest_steps` after timing. Normal, periodic, monthly and annual work are included, with setup separate; zero defaults, exact 3,688,655-byte save and state hash `e29c094ebf193923` are unchanged.

Native captures verify upright rendering and off-center picking: click (496, 333) selects Clock Tower block (77, 69). Frozen starter/night/underground/inspection captures identify executable SHA256 `861ba4c4b2dc8c15a6672a922d61e8a119ea94556594798e36c28a27b29a4b56`. The corrected generator now visibly fits seven map styles, three climates and seed 12345; actual pointer/keyboard input created and saved a 128² Delta/Alpine region with $60,000 and zero residents (`corrected-map-generator`). Earlier packaged Settings saves 0/50/100 audio levels and 150% text; an actual milestone automatically creates a clean postcard without F12. Each receipt identifies its executable separately. Software/synthetic captures with muted audio do not certify physical devices or audible playback.

The maintenance-aware paid-rules bot grew an empty 128² River region for **100 years**: peak population 70,377; end population 6,538; funds **$9,315,146**; zero financial defaults, bankruptcy months or rescues; exact **2,488,187-byte save** (`frozen-bot-century.json`). The ten-year checkpoint has 69,800 residents/$3,051,079 and a reviewed clean native postcard. The staged seven-image guide uses this automated growth as its thumbnail, keeps the starter first and labels constructed facilities honestly. Bot reliability does not imply optimal planning or a human fun rating; newer Windows century gates remain pending.

Open release gates: fresh full-source checks, new strict Windows timing and both centuries, Windows package/startup/resource reruns and installer lifecycle, independent final artifact/hash inspection, immutable release and canonical launcher publication, then engine feedback submission. The 54-game catalog must retain all 53 other games and BlueCity's history. No 0.3.0 publication is claimed.

Historical 0.2.0 and initial-development evidence follows; its old feature/test counts and browser artifacts do not certify the expansion. Browser packages are archival.

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
