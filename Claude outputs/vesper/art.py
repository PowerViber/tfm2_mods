"""Vesper and the Rift Gauntlets: regenerate every runtime sheet, then the editor previews and docs images.

    python "Claude outputs/vesper/art.py"     then     python "Claude outputs/vesper/data.py"

Drawing lives in vesper_art.py (body, gauntlets, outfits, rift-punch poses, emblems, auras) and rift_art.py
(rifts, cores, glyphs, links, packets, techniques, bonuses, holes, swipes, Shatter, gauntlet and module effects). This file packs them into <=2048 px atlases the same way
The Unified Theory does, adds the HUD bars, and replaces files atomically so re-runs are byte-identical.
"""
import importlib.util
import json
import math
import os
import tempfile
from pathlib import Path
from types import SimpleNamespace

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MOD = ROOT / 'mods/tfm2_custom'


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, HERE / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


V = load('vesper_art_common', 'vesper_art.py')
RA = load('vesper_rift_art', 'rift_art.py')


def centred(im, w, h):
    out = Image.new('RGBA', (w, h))
    out.alpha_composite(im, ((w - im.width) // 2, (h - im.height) // 2))
    return out


def atlas(folder, name, entries, w, h, cols=16, trim=False, palette=False):
    """One sheet + fanim; with trim, every animation gets one shared symmetric crop (no wobble)."""
    frames = sum(len(v) for v in entries.values())
    anims = {}
    n = 0
    sheet = Image.new('RGBA', (2048, 2048) if trim else (cols * w, math.ceil(frames / cols) * h))
    cx = cy = row = used = 0
    for tag, seq in entries.items():
        fw, fh = w, h
        if trim:    # smaller canvases are centred on the shared anchor before the symmetric crop
            seq = [(im if im.size == (w, h) else centred(im, w, h), duration) for im, duration in seq]
        if trim:
            boxes = [im.getbbox() for im, _ in seq if im.getbbox()]
            if boxes:
                fw = min(w, 2 * math.ceil(max(max(w / 2 - b[0], b[2] - w / 2) for b in boxes) + 1))
                fh = min(h, 2 * math.ceil(max(max(h / 2 - b[1], b[3] - h / 2) for b in boxes) + 1))
            else:
                fw = fh = 2
        out = []
        for im, duration in seq:
            if trim:
                if cx + fw + 2 > 2048:
                    cx = 0
                    cy += row
                    row = 0
                x, y = cx + 1, cy + 1
                assert y + fh + 1 <= 2048, (name, tag)
                im = im.crop(((w - fw) // 2, (h - fh) // 2, (w + fw) // 2, (h + fh) // 2))
                cx += fw + 2
                row = max(row, fh + 2)
                used = max(used, cx)
            else:
                x, y = n % cols * w, n // cols * h
            sheet.alpha_composite(im, (x, y))
            out.append({'duration': round(duration, 6), 'data': {'x': x, 'y': y, 'w': fw, 'h': fh}})
            n += 1
        anims[tag] = {'frames': out}
    if trim:
        sheet = sheet.crop((0, 0, max(used, 1), max(cy + row, 1)))
    if palette:
        sheet = sheet.quantize(colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
    folder.mkdir(parents=True, exist_ok=True)
    assert max(sheet.size) <= 2048, (name, sheet.size)
    sheet.save(folder / (name + '#sheet.png'))
    (folder / (name + '#anim.fanim')).write_text(json.dumps({'anims': anims}, indent=2) + '\n')


def bounded(name, entries, w, h, cols=16):
    """Pack into one trimmed 2048 px sheet, splitting into a/b halves when it does not fit."""
    folder = MOD / 'vfx'
    try:
        atlas(folder, name, entries, w, h, cols, trim=True, palette=True)
        return [name]
    except AssertionError:
        for suffix in ['#anim.fanim', '#sheet.png']:
            (folder / (name + suffix)).unlink(missing_ok=True)
        items = list(entries.items())
        half = len(items) // 2
        assert half, (name, 'single animation exceeds 2048 px')
        return bounded(name + 'a', dict(items[:half]), w, h, cols) + bounded(name + 'b', dict(items[half:]), w, h, cols)


# ------------------------------------------------------------------ HUD (buff layers above her head)
HUD = (48, 96)          # centred on the body; the bars sit above her hood


def orbs(ready, sockets, f):
    """Left gauntlet: one 3 x 3 socket per orb (5 base, up to 4 overload sockets rimmed orange)."""
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    if sockets is None:          # broken: grey cracked sockets and a red slash
        for k in range(5):
            x = 14 + k * 4
            d.rectangle((x, 7, x + 2, 9), fill=V.INK, outline=(90, 90, 100, 255))
        d.line((13, 10, 34, 6), fill=(255, 93, 93, 255) if f % 2 else (200, 60, 60, 255))
        return im
    width = sockets * 4 - 1
    x0 = 24 - width // 2
    for k in range(sockets):
        x = x0 + k * 4
        over = k >= 5
        rim_ = (255, 160, 90, 255) if over else (130, 96, 210, 255) if k < ready else V.INK
        d.rectangle((x, 7, x + 2, 9), fill=V.INK, outline=rim_)
        if k < ready:
            pulse = 200 + (55 if (f + k) % 8 < 2 else 0)
            d.point((x + 1, 8), fill=V.a((255, 200, 150) if over else V.RIFT_PALE, pulse))
        else:
            d.point((x + 1, 8), fill=V.RIFT_DEEP)
    return im


def hexes(ready, slots, f):
    """Right gauntlet: hexorb slots (2 base, 6 at 90% completion), small cyan hexes."""
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    if slots is None:
        for k in range(2):
            x = 20 + k * 5
            d.rectangle((x, 12, x + 3, 15), fill=V.INK, outline=(110, 60, 110, 255))
        d.line((19, 16, 30, 11), fill=(220, 70, 170, 255))
        return im
    width = slots * 5 - 1
    x0 = 26 - width // 2
    for k in range(slots):
        x = x0 + k * 5
        d.polygon([(x + 1, 12), (x + 2, 12), (x + 3, 13), (x + 3, 14), (x + 2, 15), (x + 1, 15), (x, 14), (x, 13)], fill=V.INK)
        if k < ready:
            d.rectangle((x + 1, 13, x + 2, 14), fill=V.a((110, 240, 255), 210 + (45 if (f + k) % 8 < 2 else 0)))
        else:
            d.point((x + 1, 14), fill=(40, 120, 150, 255))
    return im


def hexcore(level, f, awake=None, broken=False):
    """The Hexcore: a 7 px hexagon left of the hexorb row, filling from 70% (0) to 100% (6);
    white-hot while Awakened with red-violet corruption pips; cracked when the right gauntlet is down."""
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    cx, cy = 10, 13
    pts = [(cx - 2, cy - 3), (cx + 2, cy - 3), (cx + 4, cy), (cx + 2, cy + 3), (cx - 2, cy + 3), (cx - 4, cy)]
    d.polygon(pts, fill=V.INK, outline=(170, 110, 60, 255))
    if broken:
        d.line((cx - 3, cy - 2, cx + 3, cy + 2), fill=(220, 70, 170, 255))
        d.point((cx + f % 3 - 1, cy - 1), fill=(120, 60, 110, 255))
        return im
    if awake is not None:
        hot = (255, 255, 255, 255) if f % 2 else (200, 250, 255, 255)
        d.rectangle((cx - 2, cy - 2, cx + 2, cy + 2), fill=hot)
        for k in range(awake):
            d.point((cx - 2 + k, cy + 5), fill=(220, 70, 170, 255))
        d.point((cx, cy - 5 - f % 2), fill=(110, 240, 255, 255))
        return im
    rows = round(5 * level / 6)
    for k in range(rows):
        y = cy + 2 - k
        d.line((cx - 2, y, cx + 2, y), fill=(110, 240, 255, 255) if level < 6 else (230, 255, 255, 255))
    if level >= 3:
        d.point((cx + 3, cy - 1 + f % 3), fill=(255, 255, 255, 255))
    return im


def stress(b, f):
    """Gauntlet stress 0..100 in tenths; cracks appear from 70."""
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    d.rectangle((10, 2, 37, 4), fill=V.INK)
    fill = min(26, round(26 * b / 10))
    col = (255, 93, 93, 255) if b >= 9 else (255, 184, 77, 255) if b >= 7 else (127, 209, 255, 255)
    if b >= 9 and f % 2:
        col = (255, 220, 200, 255)
    if fill:
        d.rectangle((11, 3, 10 + fill, 3), fill=col)
    if b >= 7:
        for k in range(b - 6):
            x = 14 + k * 6 + (f % 2)
            d.line((x, 1, x + 1, 5), fill=V.a(V.WHITE, 220))
    return im


def _generate():
    for old in (MOD / 'vfx').glob('vesper_*#*'):
        old.unlink()
    # the registered body: rank-0 look, eight frames per animation
    entries = {kind: [(V.body(kind, f), .1) for f in range(8)] for kind in ['idle', 'run', 'attack', 'skill1', 'skill2', 'ult', 'hit', 'dead']}
    atlas(MOD / 'champions', 'tfm2_custom_vesper', entries, 48, 64)
    vfx = MOD / 'vfx'
    # rank layers (view buffs): the guard outfit and the five animated pose overlays for every look (0..7, 8 = #1)
    layers = {f'{pose}_r{l}': [(V.costume(l, f, pose), V.SECONDS[pose]) for f in range(8)] for pose in V.POSES for l in range(9)}
    atlas(vfx, 'vesper_outfits', layers, 48, 64, trim=True, palette=True)
    badges = {f'rank{r}': [(V.badge(r, f), .12) for f in range(8)] for r in range(7)}
    badges.update({f'top{p}': [(V.badge(7, f, p), .12) for f in range(8)] for p in range(1, 11)})
    atlas(vfx, 'vesper_badges', badges, *V.BADGE, 16)
    backs = {f'back_r{r}': [(V.back(r, f), .12) for f in range(8)] for r in range(7)}
    backs.update({f'back_r7_p{p}': [(V.back(7, f, p), .12) for f in range(8)] for p in range(1, 5)})
    bounded('vesper_backs', backs, *V.AURA)
    hud = {f'orbs{r}_{n}': [(orbs(r, n, f), .12) for f in range(8)] for n in range(5, 10) for r in range(n + 1)}
    hud['orbs_x'] = [(orbs(0, None, f), .12) for f in range(8)]
    hud.update({f'hex{r}_{n}': [(hexes(r, n, f), .12) for f in range(8)] for n in range(2, 7) for r in range(n + 1)})
    hud['hex_x'] = [(hexes(0, None, f), .12) for f in range(8)]
    hud.update({f'core{c}': [(hexcore(c, f), .12) for f in range(8)] for c in range(7)})
    hud.update({f'core_aw{c}': [(hexcore(6, f, awake=c), .12) for f in range(8)] for c in range(5)})
    hud['core_x'] = [(hexcore(0, f, broken=True), .12) for f in range(8)]
    hud.update({f'stress{b}': [(stress(b, f), .12) for f in range(8)] for b in range(11)})
    bounded('vesper_hud', hud, *HUD)
    RA.generate(SimpleNamespace(MOD=MOD, bounded=bounded, atlas=atlas))
    V.previews(SimpleNamespace(MOD=MOD))
    print('Vesper body, gauntlets, rank layers, HUD, rifts, techniques, effects and previews generated')


def generate():
    """Replace every file atomically (Windows previews can memory-map old atlases)."""
    original, original_text = Image.Image.save, Path.write_text

    def atomic_text(destination, data, encoding=None, errors=None, newline=None):
        fd, temporary = tempfile.mkstemp(dir=destination.parent, suffix=destination.suffix)
        os.close(fd)
        try:
            result = original_text(Path(temporary), data, encoding=encoding or 'utf-8', errors=errors, newline='\n')
            os.replace(temporary, destination)
            return result
        finally:
            Path(temporary).unlink(missing_ok=True)

    def atomic_save(im, destination, format=None, **options):
        if not isinstance(destination, (str, Path)):
            return original(im, destination, format=format, **options)
        destination = Path(destination)
        fd, temporary = tempfile.mkstemp(dir=destination.parent, suffix=destination.suffix)
        os.close(fd)
        try:
            original(im, temporary, format=format, **options)
            os.replace(temporary, destination)
        finally:
            Path(temporary).unlink(missing_ok=True)

    Image.Image.save, Path.write_text = atomic_save, atomic_text
    try:
        _generate()
    finally:
        Image.Image.save, Path.write_text = original, original_text


if __name__ == '__main__':
    generate()
