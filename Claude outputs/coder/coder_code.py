"""Round 101: the Coder's terminal: the code he types, floating over his head.

Sheets (each at most 2048 x 2048, one per language, the round-100 lesson):
  coder_code_<lang>   ln_<lang>_<func>_<line>_<step>: line <line> of <func> with step/4 of it typed (step 4: all of it),
                      in a dark terminal panel with the line number and a cursor; one frame (the native code sets how
                      long it shows)
  coder_ui            ov_*: the status lines over the terminal (compiled, saved, SyntaxError, the borrow checker,
                      compiling, loading, debugging, out of memory, segfault, null, the infinite loop) and the blue
                      screen over his body

Every sprite is centred on him (that's how the game draws effects), so the panel sits at a fixed height above his head.

Run from the repo root: python3 "Claude outputs/coder/coder_code.py" [--preview]
"""
import json
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import coder_font as F  # noqa: E402
from coder_functions import FUNCS, LANGS, typed  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
OUT = os.path.join(ROOT, 'mods', 'tfm2_custom', 'vfx')
HERE = os.path.dirname(os.path.abspath(__file__))

ROW_Y = -56        # the code line's top, from his centre
OV_Y = -67         # the status line above it
PANEL = (8, 14, 18, 205)
EDGE = (60, 255, 140, 255)
DIM = (110, 120, 130)
LONG = 6.0         # one long frame: the native effect's own life ends it


def panel(line_im, gutter, y_top, frame=EDGE, cursor=False):
    """A terminal panel holding gutter + line, placed so its top is y_top px above (negative) his centre."""
    g = F.text(gutter, [DIM] * len(gutter)) if gutter else None
    gw = g.width + 2 if g else 0
    w = gw + line_im.width + (F.CW + 1 if cursor else 0) + 6
    h = F.CH + 4
    W = w + (w % 2)
    H = 2 * (-y_top) + 2
    im = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    px = im.load()
    x0, y0 = (W - w) // 2, H // 2 + y_top
    for y in range(h):
        for x in range(w):
            corner = (x in (0, w - 1)) and (y in (0, h - 1))
            if not corner:
                px[x0 + x, y0 + y] = PANEL
    for x in range(1, w - 1):   # a thin bright top edge, like a terminal tab
        px[x0 + x, y0] = frame[:3] + (170,)
    if g:
        im.alpha_composite(g, (x0 + 3, y0 + 2))
    im.alpha_composite(line_im, (x0 + 3 + gw, y0 + 2))
    if cursor:
        cx = x0 + 3 + gw + line_im.width + 1
        for y in range(F.CH - 1):
            px[cx, y0 + 2 + y] = (120, 255, 150, 230)
            px[cx + 1, y0 + 2 + y] = (120, 255, 150, 230)
    return im


def line_frames():
    """{lang: {tag: image}}"""
    out = {}
    for lang in LANGS:
        d = {}
        for name, tier, langs in FUNCS:
            for i, ln in enumerate(langs[lang]):
                t = typed(ln)
                ind = ln[:len(ln) - len(ln.lstrip())]
                for step in range(1, 5):
                    n = len(t) if step == 4 else max(1, -(-len(t) * step // 4))
                    shown = ind + t[:n]
                    cols = F.colours(ind + t)[:len(shown)]
                    d[f'ln_{lang}_{name}_{i}_{step}'] = panel(F.text(shown, cols), f'{i + 1:>2}', ROW_Y, cursor=step < 4)
        out[lang] = d
    return out


RED = (255, 90, 90)
GREEN = (120, 255, 150)
BLUE = (110, 200, 255)
AMBER = (255, 200, 90)

OVERLAYS = {
    'ov_compiled': ('[ok] compiled', GREEN),
    'ov_saved': ('[ok] compiled + saved', GREEN),
    'ov_syntax': ('SyntaxError!', RED),
    'ov_borrow': ('error[E0502]: borrow', RED),
    'ov_rustc': ('rustc: compiling...', AMBER),
    'ov_compile': ('g++ -O2: compiling...', AMBER),
    'ov_load': ('loading from disk...', BLUE),
    'ov_debug': ('debugging...', AMBER),
    'ov_oom': ('Out of memory!', RED),
    'ov_segv': ('Segmentation fault', RED),
    'ov_null': ('NullReference!', RED),
    'ov_loop': ('while(True): ...', RED),
}


def bsod():
    """The blue screen over his body (centred on him)."""
    W, H = 44, 40
    im = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    px = im.load()
    for y in range(4, 36):
        for x in range(4, 40):
            px[x, y] = (24, 92, 200, 240)
    face = F.text(':(', [(255, 255, 255)] * 2)
    im.alpha_composite(face, (8, 8))
    for k, w in enumerate((26, 20, 23)):
        for x in range(8, 8 + w):
            px[x, 21 + k * 4] = (200, 225, 255, 230)
    return im


def ui_frames():
    d = {}
    for tag, (msg, col) in OVERLAYS.items():
        d[tag] = panel(F.text(msg, [col] * len(msg)), '', OV_Y, frame=col + (255,))
    d['ov_bsod'] = bsod()
    return d


def pack(frames, width=2048):
    items = sorted(frames.items(), key=lambda kv: (-kv[1].height, -kv[1].width))
    x = y = row = 0
    pos = {}
    for tag, im in items:
        if x + im.width > width:
            x, y, row = 0, y + row + 1, 0
        pos[tag] = (x, y)
        x += im.width + 1
        row = max(row, im.height)
    sheet = Image.new('RGBA', (width, y + row), (0, 0, 0, 0))
    anims = {}
    for tag, im in frames.items():
        px, py = pos[tag]
        sheet.paste(im, (px, py))
        anims[tag] = {'frames': [{'duration': LONG, 'data': {'x': px, 'y': py, 'w': im.width, 'h': im.height}}]}
    assert sheet.height <= 2048, (sheet.size, len(frames))
    return sheet, {'anims': anims}


def build():
    sheets = {}
    for lang, d in line_frames().items():
        sheets[f'coder_code_{lang}'] = pack(d)
    sheets['coder_ui'] = pack(ui_frames())
    return sheets


def preview(folder):
    bg = (54, 74, 60, 255)
    lf = line_frames()
    rows = []
    for lang in LANGS:
        tags = [f'ln_{lang}_firewall_{i}_{s}' for i in range(len(dict((n, l) for n, _, l in FUNCS)['firewall'][lang])) for s in (2, 4)]
        ims = [lf[lang][t].crop((0, 0, lf[lang][t].width, 14)) for t in tags]
        w = max(i.width for i in ims)
        col = Image.new('RGBA', (w, 15 * len(ims)), bg)
        for k, i in enumerate(ims):
            col.alpha_composite(i, (0, k * 15))
        rows.append(col)
    uf = ui_frames()
    ov = [uf[t].crop((0, 0, uf[t].width, 14)) for t in OVERLAYS]
    w = max(i.width for i in ov)
    col = Image.new('RGBA', (w, 15 * len(ov) + 44), bg)
    for k, i in enumerate(ov):
        col.alpha_composite(i, (0, k * 15))
    col.alpha_composite(uf['ov_bsod'], (0, 15 * len(ov)))
    rows.append(col)
    W = sum(r.width + 8 for r in rows)
    H = max(r.height for r in rows)
    out = Image.new('RGBA', (W, H), bg)
    x = 0
    for r in rows:
        out.alpha_composite(r, (x, 0))
        x += r.width + 8
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'terminal.png'))


if __name__ == '__main__':
    S = build()
    for name, (sheet, fan) in S.items():
        sheet.save(os.path.join(OUT, name + '#sheet.png'), optimize=True)
        with open(os.path.join(OUT, name + '#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fan, fh, separators=(',', ':'))
        print(name, sheet.size, len(fan['anims']))
    if '--preview' in sys.argv:
        preview(os.path.join(HERE, 'preview'))
