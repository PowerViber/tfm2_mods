"""Pack Codex's approved Shadow Grab artwork into a game sheet (round 190, contender-3.6).

Source: docs/contender-grab-art-preview/shadow-grab-source.png (Codex, approved by Reihan). The game can't stretch,
rotate or fade a view effect at runtime, so the Canvas moves of editor/contender-grab-review.html are baked here:

- hand: the reaching hand ahead of the flying shadow, 8 headings (`hand_d{h}`);
- ribbon: the pull ribbon stretched to the live distance, baked per length bucket and heading (`ribbon_L{k}_d{h}`);
  the bridge picks the bucket closest to the current distance every few ticks, so it stays connected as the
  target is dragged in;
- latch: the contact flash, one fading sequence (`latch`);
- burst: the knock-up burst on the ground, one fading sequence (`burst`).

Sizes, offsets and fades follow the review page (1 px = 1,000 units). Ranges come from rules/contender.json.
`--check` only verifies.
"""
import argparse
import json
from pathlib import Path

from PIL import Image

from contender_art_pack import TICK, faded, heading, pack, write_or_check

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/contender-grab-art-preview/shadow-grab-source.png"
ASSET = ROOT / "mods/tfm2_custom/vfx"
RULES = json.loads((ROOT / "rules/contender.json").read_bytes())
GRAB = RULES["shadowGrab"]

# the review page's crops and game sizes
HAND = ((127, 239, 424, 215), (32, 16))
LATCH = ((768, 215, 312, 254), (24, 20))
RIBBON = ((99, 855, 463, 144), (58, 17))
BURST = ((816, 684, 214, 398), (21, 39))
RIBBON_HEIGHT = 13          # the review squashes the ribbon to 13 px while pulling
RIBBON_START = 10_000       # it starts 10 px in front of Contender
RIBBON_SLACK = 6_000        # and is 6 px longer than the gap
LATCH_TICKS, LATCH_FADE = 8, 12      # opacity 1 - t / 12 for 8 ticks
BURST_TICKS, BURST_FADE = 18, 21     # opacity 1 - t / 21 for 18 ticks
STEP = 10_000


def ribbon_lengths():
    """Distances (Contender to target) the ribbon is baked for: pullStop..range in 10,000 steps."""
    lo = GRAB["pullStop"] // STEP * STEP
    hi = -(-GRAB["range"] // STEP) * STEP
    return list(range(lo, hi + 1, STEP))


def cell(src, part, size=None):
    (x, y, w, h), base = part
    return src.crop((x, y, x + w, y + h)).resize(size or base, Image.NEAREST)


def build():
    src = Image.open(SOURCE).convert("RGBA")
    anims = {}       # tag -> [(image, duration)]
    hand = cell(src, HAND)
    for h in range(8):
        anims[f"hand_d{h}"] = [(heading(hand, h), 0.1)]
    for length in ribbon_lengths():
        width = max(1, (length - RIBBON_START + RIBBON_SLACK) // 1000)
        ribbon = cell(src, RIBBON, (width, RIBBON_HEIGHT))
        for h in range(8):
            anims[f"ribbon_L{length // STEP}_d{h}"] = [(heading(ribbon, h), 0.1)]
    latch = cell(src, LATCH)
    anims["latch"] = [(faded(latch, 1 - t / LATCH_FADE), 2 * TICK) for t in range(0, LATCH_TICKS, 2)]
    burst = cell(src, BURST)
    anims["burst"] = [(faded(burst, 1 - t / BURST_FADE), 3 * TICK) for t in range(0, BURST_TICKS, 3)]

    return pack(anims)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    sheet, fanim = build()
    write_or_check(ASSET / "contender_grab#sheet.png", ASSET / "contender_grab#anim.fanim", sheet, fanim, args.check, "Shadow Grab")
