# BlueCity 0.3.0 — native release verified

Verified on **2026-10-23**. BlueCity 0.3.0 is published from immutable source [`cc76ba845d3c8f215ded8afa9b5029debf2e3735`](https://github.com/kevstermcgee/BlueCity/tree/cc76ba845d3c8f215ded8afa9b5029debf2e3735), tagged [`v0.3.0`](https://github.com/kevstermcgee/BlueCity/releases/tag/v0.3.0), with the exact tested installer mirrored in the trusted [BlueEngine Games release](https://github.com/kevstermcgee/BlueEngineGames/releases/tag/bluecity-v0.3.0). The engine remains pinned to `1a5b696ee653179c6898b1c4195cb1d3d2715d6e`.

The complete requested feature list is implemented at the documented simulation scope: occupied lots and households, bounded trip/vehicle samples, grid infrastructure and neighboring trade contracts. [FEATURES.md](FEATURES.md) records individual behavior evidence and limits. BlueCity is an original city builder; exact commercial-game parity, optimal bot play and a subjective fun rating are not claimed.

The [final Windows workflow](https://github.com/kevstermcgee/BlueCity/actions/runs/39984257601) passed **158 Rust tests**, both 100-year simulations, formatting/strict Clippy/content/audio checks, isolated package smoke, independently checked icon/resources, ordinary fullscreen/windowed startup and F11 toggles. The studio card is centered text with no retired artwork; the packaged city is upright and its controls readable. Real installer installation, reinstall/upgrade and uninstall passed, including exact installed bytes, shortcuts and retention of saves/settings. Independently downloaded source-release and Games-mirror assets match the tested artifact. The installer is **14,005,348 bytes**, SHA256 `81a29daa6590ec926bbefa04f32861aedf0a6faf53a3533d1276dbb595aac9ae`; [published metadata](https://github.com/kevstermcgee/BlueEngineGames/releases/download/bluecity-v0.3.0/release-metadata.json) records its source, engine and checks. The existing latest bundled Games release was preserved.

Final Linux verification also passed all **158 tests**, strict all-target Clippy, content/audio checks and isolated shipping smoke with `stable_sources:true` (`.blue-check/expansion/final-policy-linux-verification.json`). Its runtime tree is identical to the preceding `a295af4e` candidate; the final source change documents the release timing policy. The release and artifact receipts are `verified-windows-release.json`, `windows-39984257601/independent-review.json` and `windows-39984257601/public-release-verification.json` under `.blue-check/expansion/`.

## Measured performance and limits

All rows below use the same 128×128 fixture with **15,009 real building instances** for 360 simulated days. The final Windows runner was Windows Server 2025, AMD64 Family 25 Model 1, four logical CPUs, release optimization and thin LTO.

| Measurement | Mean | p99 | Maximum | Raw strict maximum flag |
| --- | --- | --- | --- | --- |
| Published Windows run 39984257601 | 2.057 ms | 3.298 ms | 3.932 ms | `target_met:true` |
| Local Intel N97, checkpoint-reuse run | 2.432 ms | 3.825 ms | 4.092 ms | `target_met:false` |
| Earlier Windows run 39980449015 | 2.103 ms | 3.562 ms | 4.649 ms | `target_met:false` |

The published Windows measurement meets even the strict 4 ms maximum for that run. It is not a guarantee for every host, later city or execution. We initially imposed an every-sample maximum release gate beyond the user's host/percentile-unspecified performance target, then openly corrected acceptance to **mean and p99 below 4 ms** on the stated 360-day workload. Raw maxima, slow samples and the CLI's independent strict `target_met` flag remain visible. The earlier failures above are retained; no unproved cause is assigned to outliers.

The measured fixture keeps hash `e29c094ebf193923`, population 211,189, treasury $10,943,116 and an exact **3,688,655-byte** save. Actual commute paths and the initial January checkpoint are prepared in separately reported setup (69.427 ms on the published Windows run); normal, periodic, monthly and annual work remains timed. Exact buffer reuse changes allocation behavior while canonical-copy, source-independence, golden archive, rewind and future-delivery tests preserve the state.

The Windows dense **100-year** stability run ends with 12,546 buildings, 208,242 residents, $74,688,957, zero financial defaults and an exact 3,756,945-byte save. Its longer-run timings are 2.801 ms mean / 4.811 ms p99 / 10.561 ms maximum. The paid-rules bot grows an empty River region for 100 years, peaks at 70,377 residents and ends at 6,538 with $9,315,146, zero defaults/bankruptcy months/rescues and an exact 2,488,187-byte save. Its timing is 1.326 / 2.974 / 40.779 ms; that varying city and paid planning workload is a stability/balance test, not the fixed 15,009-building performance gate. These longer-run maxima are retained rather than presented as a universal 4 ms result.

## Presentation and delivery

Native evidence verifies upright geometry and off-center picking: logical click (496,333) selects Clock Tower block (77,69). Actual pointer/keyboard input creates and saves a 128² Delta/Alpine map with seed 12345, $60,000 and zero residents. Settings saves separate audio levels and 150% text; a real milestone creates a clean automatic postcard without F12. The seven-image guide distinguishes the starter, actual ten-year automated paid growth and constructed catalog showcases. Software rendering and synthetic inputs do not certify physical devices or audible playback.

Launcher publication is verified at [Games commit `968af557`](https://github.com/kevstermcgee/BlueEngineGames/commit/968af5577240abc1d3bd2f25eb747286f4dee7bd) and the successful [Pages deployment](https://github.com/kevstermcgee/BlueEngineGames/actions/runs/39987797621). The canonical public catalog shows 0.3.0, all 54 games and unchanged entries for the other 53 games. BlueCity retains 0.2.0, 0.1.1 and 0.1.0 in its version history. The [live guide and seven-image gallery](https://kevstermcgee.github.io/BlueEngineGames/games/bluecity/#gallery), thumbnail and canonical installer were downloaded and verified against exact byte sizes and SHA256 values; the existing latest bundled Games release remains unchanged.

Engine feedback was submitted and read back: [logical-camera recipe and off-center native regression, issue #39](https://github.com/kevstermcgee/BlueEngine/issues/39), and [portable-client hooks, semantic hashing and checkpoint allocation guidance on #38](https://github.com/kevstermcgee/BlueEngine/issues/38#issuecomment-6369868818). Learning records L-301–L-304 were appended without changing the engine implementation; representative context queries retrieve all four hints. Source dependency/tooling remains pinned to the reviewed engine revision.

## Earlier records

On 2026-10-23, earlier 0.3 candidates passed their recorded functional and century checks but failed the then-strict timing gate, so installer/publication were skipped for those attempts. Their receipts remain under `.blue-check/expansion/`; earlier 148/153/154-test counts describe earlier revisions. The 0.2 and initial-development records below retain their original dated scope. Their old feature counts, screenshots and archival browser packages do not certify the expansion.

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
