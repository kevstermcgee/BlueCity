#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use vesper3d::two_d::{client, GameLogic};
fn config() -> macroquad::conf::Conf {
    let identity =
        vesper3d::viewer::identity::Identity::parse(include_str!("../assets/identity.json"))
            .unwrap();
    let mut config = client::config(&identity.title);
    config.miniquad_conf.sample_count = 4;
    config.miniquad_conf.icon = Some(macroquad::miniquad::conf::Icon {
        small: *include_bytes!("../assets/icon_16.rgba"),
        medium: *include_bytes!("../assets/icon_32.rgba"),
        big: *include_bytes!("../assets/icon_64.rgba"),
    });
    assert_eq!(
        identity.title,
        bluecity::BlueCity::TITLE,
        "Game title and identity disagree"
    );
    config
}
#[macroquad::main(config)]
async fn main() {
    client::run::<bluecity::BlueCity>().await;
}
