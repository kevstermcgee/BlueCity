//! Real command-line errors must not silently start a different simulation.
use std::process::Command;
const SIM: &str = env!("CARGO_BIN_EXE_bluecity-sim");
#[test]
fn invalid_or_conflicting_requests_fail_with_a_useful_error() {
    for args in [
        vec!["--years"],
        vec!["--yeras", "100"],
        vec!["--days", "0"],
        vec!["--days", "1", "--years", "1"],
        vec!["--days", "1", "--days", "2"],
        vec!["--map", "7"],
        vec!["--climate", "3"],
        vec!["--scenario", "8"],
        vec!["--scenario", "256"],
        vec!["--years", "4294967295"],
        vec!["--load", "missing.bcsave", "--stress"],
    ] {
        let result = Command::new(SIM).args(&args).output().unwrap();
        assert!(!result.status.success(), "accepted {args:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("BlueCity simulation:"));
    }
}
#[test]
fn climate_only_generation_produces_a_loadable_exact_checkpoint() {
    use vesper3d::{runtime::snapshot, two_d::GameLogic};
    let path = std::env::temp_dir().join(format!("bluecity-cli-{}.bcsave", std::process::id()));
    let result = Command::new(SIM)
        .args(["--climate", "2", "--days", "1", "--save"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let mut game = bluecity::BlueCity::new(0);
    snapshot::restore(&mut game, &bytes).unwrap();
    assert_eq!(game.state.city.width, 128);
    assert_eq!(
        game.state.city.ext.climate,
        bluecity::world::Climate::Alpine
    );
    assert_eq!(game.state.city.ext.day, 1);
}
