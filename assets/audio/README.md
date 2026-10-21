# Riverlight

Riverlight is BlueCity's original 60-second city-building score: a warm D-major chord progression, soft alternating arpeggios and a sparse melodic phrase. It is authored as notes and synthesized by BlueEngine; it uses no recordings or third-party music. The three source instruments are rendered together into one stereo runtime layer to reduce loading and mixing overhead.

Runtime integration:

```rust
AudioBankSpec { id: "riverlight", root: "assets/audio/riverlight", music: true }
```

The music layer is `city`. A level around `0.45` leaves space for construction and warning cues. The shared portable client handles looping, menu/focus fading and the saved Music toggle.

The authored source is `source/riverlight.project.json`. Regenerate into a **new** directory with:

```sh
python3 tools/generate_score.py --tools /path/to/be2-tools --output assets/audio/riverlight-new
```

The script renders and checks the original instrument score, prepares the wrapped mix, then uses the engine's checked clip importer to create the single-layer runtime bank. It preserves existing output directories. Only `riverlight/bank.json` and `riverlight/music-city.wav` are required at runtime; `identity.package` includes that directory.

Validation:

```sh
be2-tools audio check assets/audio/riverlight
python3 /path/to/BlueEngine/tools/audio_report.py assets/audio/riverlight/music-city.wav --loop --fail-on-issues --json
```

`source/quality.json` records the numeric report for the committed runtime WAV. Checksums, waveform measurements and browser playback submissions establish asset integrity and software behavior; no physical listening test is claimed.
