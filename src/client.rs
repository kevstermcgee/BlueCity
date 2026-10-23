//! BlueCity native client, adapted from pinned BlueEngine portable client.
//! Game-owned bindings, volume preferences and postcards; engine owns storage/focus/snapshots.
use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use vesper3d::runtime::{
    storage::{self, PlatformStorage},
    FixedStepper, InputAccumulator,
};
use vesper3d::two_d::draw::{GOLD, WHITE};
use vesper3d::two_d::{draw::*, Intent, Point, Rect};

use crate::ui_model::{Hotkey, Preferences};
#[derive(Clone, Copy, Debug)]
pub enum ClientRequest {
    Save,
    Load,
    Postcard,
}
pub trait ClientGame: Game {
    fn preferences(&self) -> &Preferences;
    fn set_preferences(&mut self, prefs: Preferences);
    fn take_requests(&mut self) -> Vec<ClientRequest>;
    fn client_notice(&mut self, message: String);
    fn editing_text(&self) -> bool;
    fn draw_postcard<'a>(&'a self, scene: &mut Scene<'a>);
}
fn key(name: &str) -> Option<KeyCode> {
    KEY_CODES.iter().find(|(s, _)| *s == name).map(|(_, k)| *k)
}
pub const KEY_CODES: &[(&str, KeyCode)] = &[
    ("A", KeyCode::A),
    ("B", KeyCode::B),
    ("C", KeyCode::C),
    ("D", KeyCode::D),
    ("E", KeyCode::E),
    ("F", KeyCode::F),
    ("G", KeyCode::G),
    ("H", KeyCode::H),
    ("I", KeyCode::I),
    ("J", KeyCode::J),
    ("K", KeyCode::K),
    ("L", KeyCode::L),
    ("M", KeyCode::M),
    ("N", KeyCode::N),
    ("O", KeyCode::O),
    ("P", KeyCode::P),
    ("Q", KeyCode::Q),
    ("R", KeyCode::R),
    ("S", KeyCode::S),
    ("T", KeyCode::T),
    ("U", KeyCode::U),
    ("V", KeyCode::V),
    ("W", KeyCode::W),
    ("X", KeyCode::X),
    ("Y", KeyCode::Y),
    ("Z", KeyCode::Z),
    ("Space", KeyCode::Space),
    ("Escape", KeyCode::Escape),
    ("Tab", KeyCode::Tab),
    ("Home", KeyCode::Home),
    ("Equal", KeyCode::Equal),
    ("Minus", KeyCode::Minus),
    ("Period", KeyCode::Period),
    ("F1", KeyCode::F1),
    ("F2", KeyCode::F2),
    ("F3", KeyCode::F3),
    ("F4", KeyCode::F4),
    ("F5", KeyCode::F5),
    ("F6", KeyCode::F6),
    ("F7", KeyCode::F7),
    ("F8", KeyCode::F8),
    ("F9", KeyCode::F9),
    ("F10", KeyCode::F10),
    ("F11", KeyCode::F11),
    ("F12", KeyCode::F12),
    ("Left", KeyCode::Left),
    ("Right", KeyCode::Right),
    ("Up", KeyCode::Up),
    ("Down", KeyCode::Down),
];
fn binding<G: ClientGame>(game: &G, action: Hotkey) -> Option<KeyCode> {
    game.preferences()
        .keys
        .get(action as usize)
        .and_then(|s| key(s))
}
fn pressed<G: ClientGame>(game: &G, action: Hotkey) -> bool {
    binding(game, action).is_some_and(is_key_pressed)
}
fn held<G: ClientGame>(game: &G, action: Hotkey) -> bool {
    binding(game, action).is_some_and(vesper3d::viewer::focus::key_down)
}
#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    sound: bool,
    music: bool,
    fullscreen: Option<bool>,
    preferences: Preferences,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: true,
            music: true,
            fullscreen: None,
            preferences: Preferences::default(),
        }
    }
}
#[derive(Default, Serialize)]
pub struct AudioEvidence {
    pub loaded: u32,
    pub submitted: u32,
    pub enabled: bool,
    pub activated: bool,
}
#[derive(Default, Serialize)]
struct LoopEvidence {
    loaded: usize,
    submitted: usize,
    playing: bool,
    active_layers: usize,
}
struct LoopSound {
    bank: &'static str,
    layer: String,
    music: bool,
    sound: Sound,
    volume: f32,
}
struct Audio {
    sounds: Vec<Sound>,
    evidence: AudioEvidence,
    muted: bool,
    loops: Vec<LoopSound>,
    loop_evidence: LoopEvidence,
}
impl Audio {
    async fn new<G: ClientGame>(muted: bool) -> Result<Self, String> {
        if muted {
            return Ok(Self {
                sounds: vec![],
                evidence: AudioEvidence::default(),
                muted,
                loops: vec![],
                loop_evidence: LoopEvidence::default(),
            });
        }
        use vesper3d::runtime::synth::{self, Preset};
        let mut sounds = Vec::new();
        for preset in [Preset::Coin, Preset::Hit, Preset::Success] {
            let bytes = synth::wav_bytes(&synth::render(preset, 0, 7), synth::RATE);
            sounds.push(
                load_sound_from_bytes(&bytes)
                    .await
                    .map_err(|e| format!("Audio decode failed: {e}"))?,
            );
        }
        let mut loops = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for bank in G::audio_banks() {
            if !ids.insert(bank.id) {
                return Err(format!("Duplicate audio bank ID {}", bank.id));
            }
            let bytes = macroquad::file::load_file(&format!("{}/bank.json", bank.root))
                .await
                .map_err(|e| {
                    format!(
                        "Audio bank {} unavailable: {e}; declare it in identity.package",
                        bank.id
                    )
                })?;
            let metadata = vesper3d::runtime::audio_project::AudioBundle::parse(&bytes)?;
            if !metadata.effects.is_empty() {
                return Err(
                    "Portable loop banks must contain only music/ambience; effects use take_cues"
                        .into(),
                );
            }
            for (layer, info) in metadata.music {
                let path = format!("{}/{}", bank.root, info.file);
                let bytes = macroquad::file::load_file(&path)
                    .await
                    .map_err(|e| format!("Audio asset {path} unavailable: {e}"))?;
                vesper3d::runtime::audio_project::AudioBundle::verify_file(&info, &bytes)?;
                let sound = load_sound_from_bytes(&bytes)
                    .await
                    .map_err(|e| format!("Audio asset {path} failed decoding: {e}"))?;
                loops.push(LoopSound {
                    bank: bank.id,
                    layer,
                    music: bank.music,
                    sound,
                    volume: 0.,
                });
                next_frame().await;
            }
        }
        Ok(Self {
            loop_evidence: LoopEvidence {
                loaded: loops.len(),
                ..Default::default()
            },
            loops,
            evidence: AudioEvidence {
                loaded: 3,
                ..Default::default()
            },
            sounds,
            muted,
        })
    }
    fn update<G: ClientGame>(
        &mut self,
        game: &G,
        active: bool,
        settings: &Settings,
        dt: f32,
    ) -> Result<(), String> {
        let active = active && !self.muted && platform::audio_requested();
        if active && !self.loop_evidence.playing && !self.loops.is_empty() {
            for track in &self.loops {
                play_sound(
                    &track.sound,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.,
                    },
                );
            }
            self.loop_evidence.submitted += self.loops.len();
            self.loop_evidence.playing = true;
        } else if !active && self.loop_evidence.playing {
            for track in &mut self.loops {
                stop_sound(&track.sound);
                track.volume = 0.;
            }
            self.loop_evidence.playing = false;
        }
        self.loop_evidence.active_layers = 0;
        for track in &mut self.loops {
            let level = game.audio_level(track.bank, &track.layer);
            if !level.is_finite() || !(0. ..=1.).contains(&level) {
                return Err(format!(
                    "Audio level {}/{} must be finite 0..1",
                    track.bank, track.layer
                ));
            }
            let enabled = if track.music {
                settings.music
            } else {
                settings.sound
            };
            let volume = if track.music {
                game.preferences().music
            } else {
                game.preferences().ambience
            };
            let target = if active && enabled {
                level * f32::from(volume) / 100.
            } else {
                0.
            };
            // Disable immediately; ordinary day/night transitions fade.
            track.volume = if !enabled {
                0.
            } else {
                track.volume + (target - track.volume) * (dt * 3.).clamp(0., 1.)
            };
            set_sound_volume(&track.sound, track.volume);
            if track.volume > 0.001 {
                self.loop_evidence.active_layers += 1;
            }
        }
        Ok(())
    }
    fn play(&mut self, cue: usize, enabled: bool, volume: u8) {
        self.evidence.enabled = enabled && !self.muted;
        self.evidence.activated = platform::audio_requested();
        if self.evidence.enabled && self.evidence.activated {
            if let Some(sound) = self.sounds.get(cue) {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.3 * f32::from(volume) / 100.,
                    },
                );
                self.evidence.submitted += 1;
            }
        }
    }
}
pub fn config(title: &str) -> macroquad::conf::Conf {
    #[allow(unused_mut)]
    let mut size = (960_u32, 540_u32);
    {
        let args: Vec<_> = std::env::args().collect();
        if vesper3d::runtime::playback::has_flag(&args, "--size") {
            size = vesper3d::runtime::playback::flag_value(&args, "--size")
                .and_then(vesper3d::runtime::playback::parse_size)
                .filter(|(w, h)| *w >= 160 && *h >= 120 && *w <= 4096 && *h <= 4096)
                .unwrap_or_else(|| {
                    eprintln!("--size requires WIDTHxHEIGHT (160×120 through 4096×4096)");
                    std::process::exit(2)
                });
        }
    }
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: vesper3d::viewer::focus::window_title(title),
            window_width: size.0 as i32,
            window_height: size.1 as i32,
            fullscreen: vesper3d::viewer::startup::default_fullscreen(),
            high_dpi: true,
            ..Default::default()
        },
        ..Default::default()
    }
}
pub async fn run<G: ClientGame>() {
    if let Err(error) = run_inner::<G>().await {
        platform::error(&error);
        std::process::exit(1);
    }
}
async fn run_inner<G: ClientGame>() -> Result<(), String> {
    let capture = {
        let args: Vec<_> = std::env::args().collect();
        let plan = vesper3d::runtime::playback::CapturePlan::from_args(&args)
            .map_err(|e| e.to_string())?;
        if let Some(plan) = &plan {
            plan.create_dir().map_err(|e| e.to_string())?;
        }
        plan
    };
    let store = PlatformStorage::new(G::ID)?;
    let (mut settings, mut notice) = match storage::read_settings::<Settings>(&store) {
        Ok(settings) => (settings, String::new()),
        Err(error) => (
            Settings::default(),
            format!("{error}; playing with session defaults"),
        ),
    };
    if !settings.preferences.validate() {
        settings.preferences = Preferences::default();
        notice = "Invalid control preferences; defaults restored.".into();
    }
    let launch_args: Vec<_> = std::env::args().collect();
    let initial_fullscreen =
        vesper3d::viewer::startup::fullscreen_preference(&launch_args, settings.fullscreen);
    macroquad::miniquad::window::set_fullscreen(initial_fullscreen);
    vesper3d::viewer::startup::show().await;
    let mut game = G::new(7);
    game.set_preferences(settings.preferences.clone());
    if let Some(path) = vesper3d::runtime::playback::flag_value(&launch_args, "--city") {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        vesper3d::runtime::snapshot::restore(&mut game, &bytes).map_err(|e| e.to_string())?;
    }
    let verification = platform::verify_mode();
    #[allow(unused_mut)]
    let mut persist_progress = !verification;
    {
        persist_progress &= capture.is_none();
    }
    if persist_progress && vesper3d::runtime::playback::flag_value(&launch_args, "--city").is_none()
    {
        match storage::load_slot(&store, "progress", &mut game) {
            Ok(true) => notice = "Progress restored. Start to continue.".into(),
            Ok(false) => {}
            Err(error) => notice = error,
        }
    }
    let mut autosave_time = 0.;
    let mut autosave_hash = None;
    let mut started = verification || G::start_automatically();
    {
        started |= capture.is_some();
    }
    let mut verification_tick = 0;
    let paused = false;
    let mut audio = Audio::new::<G>(platform::muted()).await?;
    let mut stepper = FixedStepper::new();
    let mut inputs = InputAccumulator::<Intent>::new();
    let mut pointer_actions = ActionPointer::default();
    let mut renderer = Renderer::default();
    let mut frame = 0;
    let mut last_pointer = None;
    let mut focus_gate = vesper3d::viewer::focus::NeutralGate::default();
    let mut fullscreen = initial_fullscreen;
    #[cfg(feature = "client")]
    let mut pads = vesper3d::viewer::gamepad::Gamepads::new().ok();
    loop {
        frame += 1;
        let mut observation =
            vesper3d::viewer::focus::sample().ok_or("Native focus adapter is unavailable")?;
        if vesper3d::viewer::startup::automated_input(&launch_args) {
            observation.focused = true;
            observation.lost = false;
            observation.neutral = true;
        }
        #[allow(unused_mut)]
        let mut neutral = observation.neutral && get_keys_pressed().is_empty();

        if observation.focused && !observation.lost && pressed(&game, Hotkey::Fullscreen) {
            fullscreen = !fullscreen;
            macroquad::miniquad::window::set_fullscreen(fullscreen);
            settings.fullscreen = Some(fullscreen);
            if capture.is_none() {
                let _ = storage::write_settings(&store, &settings);
            }
        }
        let dt = get_frame_time().clamp(0., 0.1);
        let view = Viewport::fit(800., 450., screen_width().max(1.), screen_height().max(1.));
        let (mx, my) = mouse_position();
        #[allow(unused_mut)]
        let mut pointer = view.pointer(mx, my);
        #[allow(unused_mut)]
        let mut digital = platform::touch();
        if let Some(point) = digital.pointer {
            pointer = Some(point);
        }
        #[allow(unused_mut)]
        let mut x = i32::from(held(&game, Hotkey::PanRight))
            - i32::from(held(&game, Hotkey::PanLeft))
            + digital.x;
        #[allow(unused_mut)]
        let mut y = i32::from(held(&game, Hotkey::PanDown)) - i32::from(held(&game, Hotkey::PanUp))
            + digital.y;
        #[allow(unused_mut)]
        let mut action = is_mouse_button_pressed(MouseButton::Left);
        #[allow(unused_mut)]
        let mut start =
            platform::command_key(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left);
        action |= digital.action;
        start |= digital.commands & 1 != 0;
        #[allow(unused_mut)]
        let mut look_native = [0.; 2];
        #[cfg(feature = "client")]
        if let Some(pads) = pads.as_mut() {
            let pad = pads.poll(observation.focused && !observation.lost);
            neutral &= pad.neutral();
            x += (pad.left_stick[0] * 1.5) as i32;
            y -= (pad.left_stick[1] * 1.5) as i32;
            action |= pad.menu_select();
            start |= pad.menu_select();
            if G::drag_look() {
                look_native = [pad.right_stick[0] * dt * 2., pad.right_stick[1] * dt * 2.];
            }
        }
        let focused = focus_gate.observe(observation.focused, observation.lost, neutral);
        let mut look = [0.; 2];
        if G::drag_look()
            && vesper3d::viewer::focus::mouse_down(MouseButton::Left)
            && pointer.is_some()
        {
            if let Some((x, y)) = last_pointer {
                look = [(mx - x) * 0.003, (y - my) * 0.003];
            }
            last_pointer = Some((mx, my));
        } else {
            last_pointer = None;
        }
        look[0] += look_native[0];
        look[1] += look_native[1];
        if focused && !started && start {
            started = true;
            audio.play(0, settings.sound, game.preferences().sound);
        }
        if focused && !game.editing_text() && pressed(&game, Hotkey::Save) {
            notice = match storage::save(&store, &game) {
                Ok(()) => "Checkpoint saved.".into(),
                Err(e) => e,
            };
        }
        if focused && !game.editing_text() && pressed(&game, Hotkey::Load) {
            notice = match storage::load(&store, &mut game) {
                Ok(true) => {
                    inputs.clear();
                    pointer_actions.clear();
                    "Checkpoint loaded.".into()
                }
                Ok(false) => "No checkpoint yet.".into(),
                Err(e) => e,
            };
        }

        let mut intent = if focused && started && !paused {
            game.device_input(Intent {
                x: x.clamp(-1, 1),
                y: y.clamp(-1, 1),
                pointer,
                action,
                sprint: platform::sprint_key(),
                look: [0.; 2],
            })
        } else {
            Intent::default()
        };
        action = intent.action;
        if G::pointer_target_only_on_press() {
            pointer_actions.feed(
                action,
                intent.pointer,
                is_mouse_button_pressed(MouseButton::Left) || intent.pointer != pointer,
                digital.action && digital.pointer.is_some(),
            );
        }
        intent.action = false;
        clear_inactive_input(
            focused && started && !paused,
            &mut inputs,
            &mut pointer_actions,
            &mut last_pointer,
        );
        inputs.feed(
            intent,
            u32::from(action && focused && started && !paused),
            if focused && started && !paused {
                look
            } else {
                [0.; 2]
            },
        );
        let ticks = if verification && started {
            // Replay uses the same fixed steps; graphics need not redraw after every eight ticks.
            // Real-device verification below runs normal timing independently.
            60
        } else {
            stepper.advance(if focused && started && !paused {
                dt
            } else {
                0.
            })
        };
        for _ in 0..ticks {
            if verification && verification_tick >= G::VERIFY_TICKS {
                break;
            }
            let tick = inputs.take_tick();
            let mut intent = tick.held;
            intent.action = tick.pressed(1);
            if G::pointer_target_only_on_press() {
                intent.pointer = pointer_actions.take(intent.action);
            }
            intent.look = tick.look;
            if verification {
                intent = G::verification_input(verification_tick);
                verification_tick += 1;
            }
            game.step(&intent);
        }
        for cue in game.take_cues() {
            audio.play(cue, settings.sound, game.preferences().sound);
        }
        audio.update(&game, started && !paused && focused, &settings, dt)?;
        autosave_time += dt;
        if persist_progress
            && started
            && game.preferences().autosave
            && game.tick() > 0
            && (autosave_time >= 30. || !focused || paused || game.outcome() != "playing")
        {
            autosave_time = 0.;
            let hash = game.state_hash();
            if autosave_hash != Some(hash) {
                match storage::save_slot(&store, "progress", &game) {
                    Ok(()) => autosave_hash = Some(hash),
                    Err(error) => {
                        notice = format!("Autosave failed: {error}");
                        autosave_hash = Some(hash);
                    }
                }
            }
        }
        if game.preferences() != &settings.preferences {
            if game.preferences().display_scale != settings.preferences.display_scale && !fullscreen
            {
                let scale = f32::from(game.preferences().display_scale) / 100.;
                request_new_screen_size(960. * scale, 540. * scale);
            }
            settings.preferences = game.preferences().clone();
            if persist_progress {
                if let Err(e) = storage::write_settings(&store, &settings) {
                    notice = e;
                }
            }
        }
        let mut postcard_requested = false;
        for request in game.take_requests() {
            match request {
                ClientRequest::Save => {
                    notice = match storage::save(&store, &game) {
                        Ok(()) => "Checkpoint saved.".into(),
                        Err(e) => e,
                    }
                }
                ClientRequest::Load => {
                    notice = match storage::load(&store, &mut game) {
                        Ok(true) => {
                            inputs.clear();
                            pointer_actions.clear();
                            "Checkpoint loaded.".into()
                        }
                        Ok(false) => "No checkpoint yet.".into(),
                        Err(e) => e,
                    }
                }
                ClientRequest::Postcard => postcard_requested = true,
            }
        }

        if !notice.is_empty() {
            game.client_notice(std::mem::take(&mut notice));
        }
        let mut scene = Scene::default();
        scene.rect(-100, Rect::new(0, 0, 800, 450), INK);
        if postcard_requested {
            game.draw_postcard(&mut scene);
        } else {
            game.draw(&mut scene);
        }
        if G::show_hud() {
            scene.text(100, G::TITLE, Point::new(24, 30), 26., WHITE);
            scene.text(100, G::CONTROLS, Point::new(24, 428), 16., WHITE);
            scene.text(
                100,
                "Esc pause · M sound · K save · L load · R restart",
                Point::new(24, 448),
                14.,
                TEAL,
            );
        }
        if G::show_overlays() && !notice.is_empty() {
            scene.text(101, &notice, Point::new(24, 395), 19., GOLD);
        }
        if G::show_overlays() && (!started || paused || game.outcome() != "playing") {
            scene.rect(
                200,
                Rect::new(145, 135, 510, 180),
                Color::new(0.1, 0.16, 0.22, 0.96),
            );
            let title = if !started {
                "CLICK OR ENTER TO START"
            } else if paused {
                "PAUSED"
            } else if game.outcome() == "won" {
                "COMPLETE!"
            } else {
                "TRY AGAIN"
            };
            scene.text(201, title, Point::new(180, 190), 34., GOLD);
            scene.text(201, game.menu_status(), Point::new(180, 274), 22., WHITE);
            scene.text(
                201,
                if !started {
                    "Sound activates with your input."
                } else {
                    "R restarts · K saves · L resumes"
                },
                Point::new(180, 238),
                22.,
                WHITE,
            );
        }
        clear_background(BLACK);
        scene.draw(view, Point::default(), &mut renderer)?;
        if postcard_requested {
            let folder = std::env::var_os("BLUECITY_PHOTOS")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::env::var_os("BLUEENGINE_DATA_DIR")
                        .or_else(|| std::env::var_os("LOCALAPPDATA"))
                        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
                        .map(std::path::PathBuf::from)
                        .or_else(|| {
                            std::env::var_os("HOME")
                                .map(|p| std::path::PathBuf::from(p).join(".local/share"))
                        })
                        .unwrap_or_else(std::env::temp_dir)
                        .join("BlueCity/Postcards")
                });
            match std::fs::create_dir_all(&folder) {
                Ok(()) => {
                    let stamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();
                    let path = folder.join(format!("bluecity-{}-{stamp}.png", game.tick()));
                    get_screen_data().export_png(&path.to_string_lossy());
                    notice = format!("Postcard saved: {}", path.display());
                }
                Err(e) => notice = format!("Could not save postcard: {e}"),
            }
        }

        if let Some(plan) = &capture {
            if plan.wants(frame) {
                let path = plan.path_for(frame);
                get_screen_data().export_png(path.to_str().ok_or("Capture path must be UTF-8")?);
                println!(
                    "{}",
                    serde_json::json!({"frame":frame,"path":path,"width":screen_width(),"height":screen_height()})
                );
            }
        }
        audio.evidence.enabled = settings.sound && !audio.muted;
        audio.evidence.activated = platform::audio_requested();

        {
            if capture.as_ref().is_some_and(|c| c.finished(frame))
                || (capture.is_none() && verification && verification_tick >= G::VERIFY_TICKS)
            {
                println!(
                    "{}",
                    serde_json::json!({"subsystem":"audio", "intent":if audio.muted {"muted"} else {"requested"}, "assets_loaded":audio.evidence.loaded,"playback_submissions":audio.evidence.submitted,"backend_health":"unobservable","audible_hardware":"unverified"})
                );
                println!(
                    "{}",
                    serde_json::json!({"tick":game.tick(),"hash":format!("{:016x}",game.state_hash()),"outcome":game.outcome()})
                );
                break;
            }
        }
        next_frame().await;
    }
    {
        Ok(())
    }
}
// Shared production path: inactive local clients cannot retain a press/target/look for later ticks.
fn clear_inactive_input(
    active: bool,
    inputs: &mut InputAccumulator<Intent>,
    targets: &mut ActionPointer,
    pointer: &mut Option<(f32, f32)>,
) {
    if !active {
        inputs.clear();
        targets.clear();
        *pointer = None;
    }
}

/// The action target travels with its press edge across frames with no fixed tick.
#[derive(Default)]
struct ActionPointer {
    target: Option<Point>,
}
impl ActionPointer {
    fn feed(&mut self, action: bool, pointer: Option<Point>, clicked: bool, tapped: bool) {
        if action {
            self.target = if clicked || tapped { pointer } else { None };
        }
    }
    fn take(&mut self, action: bool) -> Option<Point> {
        if action {
            self.target.take()
        } else {
            None
        }
    }
    fn clear(&mut self) {
        self.target = None;
    }
}

mod platform {
    #[derive(Default)]
    pub struct Digital {
        pub x: i32,
        pub y: i32,
        pub action: bool,
        pub commands: i32,
        pub pointer: Option<super::Point>,
    }
    pub fn touch() -> Digital {
        Digital::default()
    }
    pub fn muted() -> bool {
        let args: Vec<_> = std::env::args().collect();
        vesper3d::runtime::playback::has_flag(&args, "--mute")
            || (vesper3d::runtime::playback::has_flag(&args, "--capture")
                && !vesper3d::runtime::playback::has_flag(&args, "--audible"))
    }
    pub fn verify_mode() -> bool {
        std::env::args().any(|a| a == "--verify")
    }
    pub fn sprint_key() -> bool {
        use macroquad::prelude::*;
        vesper3d::viewer::focus::key_down(KeyCode::LeftShift)
            || vesper3d::viewer::focus::key_down(KeyCode::RightShift)
    }
    pub fn command_key(key: macroquad::prelude::KeyCode) -> bool {
        {
            macroquad::prelude::is_key_pressed(key)
        }
    }
    pub fn audio_requested() -> bool {
        true
    }
    pub fn error(error: &str) {
        eprintln!("2D client: {error}");
    }
}
