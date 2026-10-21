#!/usr/bin/env python3
"""Render BlueCity's original Riverlight score with BlueEngine's authored audio tools.

The source score has three instrument layers. They are mixed into one checked
60-second stereo loop to keep browser loading and runtime mixing modest.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import wave

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/audio/source/riverlight.project.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tools", default=os.environ.get("BE2_TOOLS") or shutil.which("be2-tools"), help="Path to BlueEngine be2-tools executable")
    parser.add_argument("--output", type=Path, default=ROOT / "assets/audio/riverlight", help="New bundle directory; existing directories are preserved")
    args = parser.parse_args()
    if not args.tools:
        parser.error("Supply --tools /path/to/be2-tools or set BE2_TOOLS")
    if args.output.exists():
        parser.error(f"Output already exists: {args.output}; choose a new directory")

    def audio(*parts):
        subprocess.run([args.tools, "audio", *map(str, parts)], check=True)

    audio("validate", SOURCE)
    with tempfile.TemporaryDirectory(prefix="bluecity-score-") as folder:
        temp = Path(folder)
        stems = temp / "stems"
        audio("render", SOURCE, stems)
        audio("check", stems)
        # Appending the exact opening allows the checked clip importer to perform
        # its wrap crossfade without changing an already wrapped score seam.
        with wave.open(str(stems / "preview-mix.wav"), "rb") as source:
            params = source.getparams()
            pcm = source.readframes(source.getnframes())
        width = params.nchannels * params.sampwidth
        with wave.open(str(temp / "riverlight.wav"), "wb") as destination:
            destination.setparams(params)
            destination.writeframes(pcm + pcm[:params.framerate * width])
        clips = {
            "version": 1,
            "seed": 20261020,
            "headroom": 0.65,
            "effects": {},
            "music": {
                "kind": "clips", "seconds": 60, "crossfade_seconds": 1,
                "layers": {"city": {"file": "riverlight.wav", "gain": 1}}
            }
        }
        project = temp / "loop.project.json"
        project.write_text(json.dumps(clips, indent=2) + "\n")
        audio("validate", project)
        audio("render", project, args.output)
        audio("check", args.output)
        # The checked runtime asset is music-city.wav. Its preview mix is an
        # identical convenience duplicate and is unnecessary in the game package.
        (args.output / "preview-mix.wav").unlink()
        audio("check", args.output)
    print(json.dumps({"ok": True, "bundle": str(args.output), "bank_id": "riverlight", "layer": "city", "seconds": 60}))


if __name__ == "__main__":
    main()
