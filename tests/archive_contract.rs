//! Correctly framed, malicious city archives must fail before touching a live town.
use bluecity::BlueCity;
use serde_json::{json, Value};
use vesper3d::{
    runtime::{snapshot, Simulation, Snapshot},
    two_d::{GameLogic, Intent},
};
struct Forged(Value);
impl Simulation for Forged {
    type Input = Intent;
    fn step(&mut self, _: &Intent) {}
    fn state_hash(&self) -> u64 {
        0
    }
}
impl Snapshot for Forged {
    const KIND: &'static str = "bluecity";
    const VERSION: u32 = 3;
    type State = Value;
    fn capture(&self) -> Value {
        self.0.clone()
    }
    fn restore(&mut self, state: Value) -> Result<(), String> {
        self.0 = state;
        Ok(())
    }
}
fn archive(bytes: &[u8]) -> Value {
    let zipped = miniz_oxide::deflate::compress_to_vec_zlib(bytes, 1);
    let data: String = zipped.iter().map(|b| format!("{b:02x}")).collect();
    json!({"codec":"bluecity-deflate-hex-1", "data":data})
}
#[test]
fn malformed_archives_preserve_the_live_city_and_its_future() {
    let mut live = BlueCity::new(19);
    live.state.city.advance_day();
    let mut control = BlueCity::new(0);
    control.state = live.state.clone();
    let cases = [
        json!({"codec":"unrecognized", "data":""}),
        json!({"codec":"bluecity-deflate-hex-1"}),
        json!({"codec":"bluecity-deflate-hex-1", "data":42}),
        json!({"codec":"bluecity-deflate-hex-1", "data":"0"}),
        json!({"codec":"bluecity-deflate-hex-1", "data":"xy"}),
        json!({"codec":"bluecity-deflate-hex-1", "data":"789c"}),
        json!({"codec":"bluecity-deflate-hex-1", "data":"00000000"}),
        archive(b"not JSON"),
        archive(b"{\"width\":128}"),
        archive(b"null"),
    ];
    for bad in cases {
        let mut value = serde_json::to_value(live.capture()).unwrap();
        value["city"] = bad;
        let frame = snapshot::save(&Forged(value), "Invalid archive").unwrap();
        let before = live.state_hash();
        assert!(snapshot::restore(&mut live, &frame).is_err());
        assert_eq!(live.state_hash(), before);
        live.state.city.advance_day();
        control.state.city.advance_day();
        assert_eq!(live.state_hash(), control.state_hash());
    }
}
#[test]
fn compressed_bombs_are_bounded_before_city_deserialization() {
    let mut value = serde_json::to_value(BlueCity::new(7).capture()).unwrap();
    value["city"] = archive(&vec![b' '; 64 * 1024 * 1024 + 1]);
    assert!(serde_json::from_value::<bluecity::State>(value).is_err());
}
#[test]
fn hashing_does_not_change_the_save_format_or_omit_city_changes() {
    let mut game = BlueCity::new(7);
    let before = serde_json::to_vec(&game.capture()).unwrap();
    let hash = game.state_hash();
    assert_eq!(serde_json::to_vec(&game.capture()).unwrap(), before);
    game.state.city.money += 1;
    assert_ne!(game.state_hash(), hash);
    let value = serde_json::to_value(game.capture()).unwrap();
    assert_eq!(value["city"]["codec"], "bluecity-deflate-hex-1");
    let frame = snapshot::save(&game, "After hashing").unwrap();
    let mut loaded = BlueCity::new(0);
    snapshot::restore(&mut loaded, &frame).unwrap();
    assert_eq!(loaded.state_hash(), game.state_hash());
}
