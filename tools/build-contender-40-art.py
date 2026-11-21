"""Pack Codex's approved Contender 4.0 art (round 192): the Smother trail and contact, and the Clone Army clone.

Source: docs/contender-40-art-preview/ (Codex; crops and sizes in sources.json). The review page
(editor/contender-40-review.js) stretches and rotates the trail from the cast origin to its front, fades the contact and
fades the clones in; the game can't, so the moves are baked (1 px = 1,000 units):

- `trail_L{k}_d{h}`: the trail stretched to every 10,000 up to `smother.range`, in 8 headings, 2 x `smother.width` wide;
  `trail_s_...` is the same at a shadow's 0.7 alpha. The bridge re-picks the nearest length every 2 ticks.
- `contact`: the contact flash, fading 1 - 0.8 t over CONTACT_TICKS.
- `clone_in` / `clone`: a clone fading in from 0.55 to 0.8 over CLONE_IN_TICKS, then steady (`_l`: facing left).

`--check` compares decoded pixels and the animation data.
"""
import argparse
import json
from pathlib import Path

from PIL import Image

from contender_art_pack import TICK, faded, heading, pack, write_or_check

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "docs/contender-40-art-preview"
ASSET = ROOT / "mods/tfm2_custom/vfx"
SOURCES = json.loads((ART / "sources.json").read_text())
RULES = json.loads((ROOT / "rules/contender.json").read_bytes())
SMOTHER = RULES["smother"]
STEP = 10_000
CONTACT_TICKS = 12      # the core's Smother contact cue (`SMOTHER_CONTACT_TICKS`)
CLONE_IN_TICKS = 8
SHADOW_ALPHA = 0.7


def trail_lengths():
    return list(range(STEP, -(-SMOTHER["range"] // STEP) * STEP + 1, STEP))


def crop(img, box, size):
    x, y, w, h = box
    return img.crop((x, y, x + w, y + h)).resize(size, Image.NEAREST)


def build():
    smother = Image.open(ART / SOURCES["smother"]["file"]).convert("RGBA")
    clone_src = Image.open(ART / SOURCES["clone"]["file"]).convert("RGBA")
    anims = {}
    breadth = 2 * SMOTHER["width"] // 1000
    for length in trail_lengths():
        trail = crop(smother, SOURCES["smother"]["trail"], (length // 1000, breadth))
        for h in range(8):
            turned = heading(trail, h)
            anims[f"trail_L{length // STEP}_d{h}"] = [(turned, 0.1)]
            anims[f"trail_s_L{length // STEP}_d{h}"] = [(faded(turned, SHADOW_ALPHA), 0.1)]
    contact = crop(smother, SOURCES["smother"]["contact"], tuple(SOURCES["smother"]["contactFrame"]))
    anims["contact"] = [(faded(contact, 1 - 0.8 * t / CONTACT_TICKS), 2 * TICK) for t in range(0, CONTACT_TICKS, 2)]
    clone = crop(clone_src, SOURCES["clone"]["crop"], tuple(SOURCES["clone"]["frame"]))
    alpha = SOURCES["clone"]["alpha"]
    for suffix, img in (("", clone), ("_l", clone.transpose(Image.FLIP_LEFT_RIGHT))):
        anims["clone_in" + suffix] = [(faded(img, alpha * min(1, 0.55 + t / CLONE_IN_TICKS)), 2 * TICK) for t in range(0, CLONE_IN_TICKS, 2)]
        anims["clone" + suffix] = [(faded(img, alpha), 0.1)]
    return pack(anims)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    sheet, fanim = build()
    write_or_check(ASSET / "contender_40#sheet.png", ASSET / "contender_40#anim.fanim", sheet, fanim, args.check, "Contender 4.0")
