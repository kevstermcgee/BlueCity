# BlueCity

BlueCity is a deterministic city management game using the native portable client.
The runtime and shared project tooling are pinned to `../bluecity-engine` in
Cargo.toml and scripts/tooling.json. Start work through that engine's be2.py with
`--project` pointing here. Read its AGENTS.md and the game's src/lib.rs first.

- City rules in src/city.rs remain rendering-free and deterministic. Input passes
  through GameLogic/Simulation; drawing reads state only.
- Engine Snapshot owns saves. Preserve KIND and include every authoritative field.
  Released layout changes need a version/migration contract and save evidence.
- Maintain meaningful simulation and pointer-workflow tests. Real-device review
  stays separate from deterministic tests, software captures and audio counters.
- scripts/check.py, ship.py, dev.py and project.py are thin entry points into the
  exact shared tooling pin. Use engine/tools/game_tooling.py for reviewed upgrades;
  preserve any explicit project hooks.
- Distribution is a native Windows x64 installer through the BlueEngine Games
  launcher. Browser artifacts from the initial development build are archival;
  do not publish browser play into the Windows-only launcher/site.
- Keep the studio opening card centered text. Do not reuse the retired white-rat
  or rat-on-blue-tile artwork in game screens, icons, logos or other artwork.

Useful checks: cargo fmt --check; cargo test --locked; cargo clippy --locked
--all-targets -- -D warnings; python3 scripts/check.py. Native packaging uses
scripts/blue ship --no-install. On Windows run installer prepare/build/test via
scripts/ship.py. .github/workflows/release.yml pins the dependency, verifies game
behavior, package/resources/startup and installer lifecycle, then optionally
publishes a new immutable release. Public catalog entries must pin that verified
installer's release URL, source commit, byte size and SHA256.
