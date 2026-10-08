"""Round 101 / 106: the Coder's terminal: the code he types, in a little IDE window floating over his head.

Round 106 (Rian: "I don't really like all the green stuff"): every rank writes in its own editor theme (the green
hacker terminal is only the Script Kiddie's cliche), and the code text is shared by all of them:

  coder_code_<lang>_<k>   ln_<lang>_<func>_<line>_<step>: line <line> of <func>, step/3 of it typed (step 3: all of it),
                          text only on a transparent background (line number, syntax colours that read on every
                          theme's dark window, a cursor while it's being typed); one frame
  coder_theme             tw_t<theme>_<lang>: the IDE window of each theme (frame, a tab with the file name, the line
                          number gutter, a minimap strip, the status bar), one per language (the tab);
                          <ov tag>_t<theme>: the status bar lines in each theme (compiled, SyntaxError, the AI, the
                          shop, and "$ ./ping"-style lines when his program runs a function); ov_bsod: the blue screen

The native code (coder.rs show_term / step_say) places them as point effects over him every 6 ticks: the window,
up to three code lines (the two above and the one being typed) and the status line. Every frame is 0.11 s, just over
those 6 ticks (round 105: the game plays an effect's animation to its end whatever life it's given).

Layout (px from his centre, up is negative; coder.rs WIN_DY, ROW_DY, ROW_DX, STATUS_DY):
  window 172 x 48 centred at (0, -66): tab rows -90..-83, code rows centred at -77 / -67 / -57, status bar at -47
  a code line sprite is 160 x 10, its left edge on the window's inner left edge (centre x -5)

Run from the repo root: python3 "Claude outputs/coder/coder_code.py" [--preview]
"""
import colorsys
import json
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import coder_font as F  # noqa: E402
from coder_functions import FUNCS, LANGS, MAX_COLS, typed  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
OUT = os.path.join(ROOT, 'mods', 'tfm2_custom', 'vfx')
HERE = os.path.dirname(os.path.abspath(__file__))

# Round 105: the game plays an effect's animation to its end whatever life the native code gives it (a 6 s frame left
# every re-placed line on screen for 6 s: a trail). So every line and status frame is just over TERM_EVERY (6 ticks)
# long, and the native code re-places it every TERM_EVERY ticks for as long as it should show.
LONG = 0.11
STEPS = 3                      # round 106: reveal steps (1/3, 2/3, all of it), coder.rs STEPS

# the shared code palette: readable on every theme's dark window
NAME = (226, 232, 245)
KEY = (150, 205, 255)
NUM = (255, 178, 132)
STR = (242, 216, 130)
SYM = (152, 162, 180)
GUTTER = (104, 114, 132)
CURSOR = (240, 244, 255)
MAP = {F.GREEN: NAME, F.CYAN: KEY, F.ORANGE: NUM, F.YELLOW: STR, F.GREY: SYM}

TW, TH = 160, 10               # a code line sprite
WW, WH = 172, 48               # the window
GUT = 3 * F.CW                 # the gutter: two digits and a space
EXT = {'py': 'main.py', 'cpp': 'main.cpp', 'rust': 'main.rs', 'js': 'main.js', 'asm': 'main.asm'}
LANG_LABEL = {'py': 'Python', 'cpp': 'C++', 'rust': 'Rust', 'js': 'JavaScript', 'asm': 'x86 asm'}

# ------------------------------------------------------------------ the rank themes (coder.rs theme_of)
# bg, frame, tab strip, active tab, gutter, status bar, accent (prompt / ok lines), alert, the run line's format
THEMES = [
    dict(name='kiddie', bg=(6, 10, 8), frame=(60, 255, 140), strip=(10, 18, 13), tab=(16, 30, 21), gutter=(9, 15, 11),
         status=(12, 34, 20), accent=(90, 255, 150), alert=(255, 90, 90), run='$ ./{}', prompt='$'),
    dict(name='crt', bg=(30, 18, 8), frame=(255, 170, 60), strip=(42, 26, 11), tab=(58, 36, 14), gutter=(36, 22, 9),
         status=(70, 40, 10), accent=(255, 196, 100), alert=(255, 110, 80), run='C:\\> {}.exe', prompt='>', scan=True),
    dict(name='dark', bg=(26, 30, 42), frame=(70, 86, 120), strip=(20, 23, 33), tab=(26, 30, 42), gutter=(30, 34, 48),
         status=(36, 92, 196), accent=(235, 242, 255), alert=(255, 120, 120), run='> {}()', prompt='', tabline=(90, 160, 255)),
    dict(name='gold', bg=(28, 28, 30), frame=(214, 172, 74), strip=(20, 20, 22), tab=(38, 37, 34), gutter=(34, 33, 31),
         status=(62, 50, 22), accent=(255, 216, 120), alert=(255, 120, 100), run='* {}() ok', prompt='', tabline=(255, 210, 110)),
    dict(name='blueprint', bg=(18, 50, 108), frame=(228, 244, 255), strip=(14, 40, 88), tab=(20, 58, 122), gutter=(16, 44, 96),
         status=(10, 34, 80), accent=(140, 236, 255), alert=(255, 150, 130), run='{}() -> run', prompt='', grid=(34, 74, 140)),
    dict(name='root', bg=(8, 4, 6), frame=(255, 56, 76), strip=(18, 6, 9), tab=(30, 9, 14), gutter=(14, 5, 8),
         status=(64, 10, 20), accent=(255, 96, 108), alert=(255, 200, 90), run='root# ./{}', prompt='#'),
    dict(name='zeroday', bg=(6, 6, 8), frame=None, strip=(14, 13, 10), tab=(24, 22, 14), gutter=(12, 12, 10),
         status=(30, 25, 10), accent=(255, 222, 128), alert=(255, 120, 150), run='<{}>', prompt='', iridescent=True),
]
THEME_RANKS = ['Script Kiddie', 'Intern / Junior', 'Developer / Senior', 'Staff', 'Architect', 'Root #10-#2', 'Zero-Day']


def A(c, a=255):
    return tuple(c[:3]) + (a,)


def hsv(h, s, v, a=255):
    r, g, b = colorsys.hsv_to_rgb(h % 1.0, s, v)
    return (int(r * 255), int(g * 255), int(b * 255), a)


def palette(line):
    """The shared syntax colours of a line."""
    return [MAP.get(c, NAME) for c in F.colours(line)]


# ------------------------------------------------------------------ code lines (text only)

def code_line(gutter, shown, cols, cursor):
    im = Image.new('RGBA', (TW, TH), (0, 0, 0, 0))
    if gutter:
        im.alpha_composite(F.text(gutter, [GUTTER] * len(gutter)), (1, 1))
    if shown:
        im.alpha_composite(F.text(shown, cols), (1 + GUT, 1))
    if cursor:
        px = im.load()
        cx = 1 + GUT + len(shown) * F.CW + 1
        for y in range(1, 1 + F.CH - 1):
            for dx in (0, 1):
                if cx + dx < TW:
                    px[cx + dx, y] = CURSOR + (230,)
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
                cols_all = palette(ind + t)
                for step in range(1, STEPS + 1):
                    n = len(t) if step == STEPS else max(1, -(-len(t) * step // STEPS))
                    shown = ind + t[:n]
                    d[f'ln_{lang}_{name}_{i}_{step}'] = code_line(f'{i + 1:>2}', shown, cols_all[:len(shown)], step < STEPS)
        out[lang] = d
    return out


# ------------------------------------------------------------------ the theme windows

def frame_colour(th, x, y, f=0):
    if th.get('iridescent'):
        # gold with a colour shimmer running round the frame
        t = (x / WW + y / WH * 0.5 + f * 0.11) % 1.0
        return hsv(0.13 + 0.5 * abs(t - 0.5), 0.55, 1.0)
    return A(th['frame'])


def window(th, lang):
    im = Image.new('RGBA', (WW, WH), (0, 0, 0, 0))
    px = im.load()
    for y in range(WH):
        for x in range(WW):
            corner = (x in (0, WW - 1)) and (y in (0, WH - 1))
            if corner:
                continue
            c = th['bg']
            if y < 8:
                c = th['strip']
            elif y >= 38:
                c = th['status']
            elif x < 1 + GUT + 1:
                c = th['gutter']
            if th.get('scan') and 8 <= y < 38 and y % 2:
                c = tuple(int(v * 0.8) for v in c)
            if th.get('grid') and 8 <= y < 38 and x > GUT + 1 and (x % 10 == 0 or y % 10 == 3):
                c = th['grid']
            px[x, y] = A(c, 238)
    # the frame
    for x in range(1, WW - 1):
        px[x, 0] = frame_colour(th, x, 0)
        px[x, WH - 1] = frame_colour(th, x, WH - 1)
    for y in range(1, WH - 1):
        px[0, y] = frame_colour(th, 0, y)
        px[WW - 1, y] = frame_colour(th, WW - 1, y)
    # the tab: a file name; an accent line on top of the active tab
    label = EXT[lang]
    tw = len(label) * F.CW + 6
    for y in range(1, 8):
        for x in range(2, 2 + tw):
            px[x, y] = A(th['tab'], 245)
    line = th.get('tabline') or th['frame'] or (255, 214, 120)
    for x in range(2, 2 + tw):
        px[x, 1] = A(line)
    im.alpha_composite(F.text(label, [th['accent']] * len(label)), (5, 0))
    # traffic lights on the right of the tab strip
    for k, c in enumerate(((255, 95, 86), (255, 189, 46), (39, 201, 63))):
        px[WW - 18 + k * 5, 4] = A(c)
        px[WW - 17 + k * 5, 4] = A(c)
    # the minimap: a few dim strokes on the right edge
    for k in range(9):
        y = 10 + k * 3
        w = 2 + (k * 5 + len(lang)) % 5
        for x in range(WW - 9, WW - 9 + w):
            px[x, y] = A(th['accent'], 70)
    # the idle status bar: the language on the right
    lab = LANG_LABEL[lang]
    im.alpha_composite(F.text(lab, [th['accent']] * len(lab)), (WW - 4 - len(lab) * F.CW, 39))
    return im


# ------------------------------------------------------------------ status lines (per theme)

RED = 'alert'
OK = 'accent'
OVERLAYS = {
    'ov_compiled': ('[ok] compiled', OK),
    'ov_saved': ('[ok] compiled + saved', OK),
    'ov_syntax': ('SyntaxError!', RED),
    'ov_borrow': ('error[E0502]: borrow', RED),
    'ov_rustc': ('rustc: compiling...', OK),
    'ov_compile': ('g++ -O2: compiling...', OK),
    'ov_load': ('loading from disk...', OK),
    'ov_debug1': ('debug: fixed 1 bug', OK),
    'ov_debug2': ('debug: fixed 2 bugs', OK),
    'ov_debug3': ('debug: fixed 3+ bugs', OK),
    'ov_oom': ('Out of memory!', RED),
    'ov_segv': ('Segmentation fault', RED),
    'ov_null': ('NullReference!', RED),
    'ov_loop': ('while(true): hung', RED),
    'ov_thinking': ('Claude: Thinking...', (255, 170, 110)),
    'ov_reasoning': ('ChatGPT: Reasoning...', (140, 230, 190)),
    'ov_diff': ('Gemini: 3 files changed', (150, 170, 255)),
    'ov_ratelimit': ('429: rate limited', RED),
    'ov_switch_claude': ('switching to Claude', (255, 170, 110)),
    'ov_switch_gpt': ('switching to ChatGPT', (140, 230, 190)),
    'ov_switch_gemini': ('switching to Gemini', (150, 170, 255)),
    'ov_install': ('installing...', OK),
    'ov_buy_ram1': ('BTC: +32 GB RAM', OK),
    'ov_buy_ram2': ('BTC: +64 GB RAM', OK),
    'ov_buy_disk1': ('BTC: 16 save slots', OK),
    'ov_buy_disk2': ('BTC: 32 save slots', OK),
    'ov_buy_ssd1': ('BTC: SSD installed', OK),
    'ov_buy_ssd2': ('BTC: NVMe installed', OK),
    'ov_buy_cool1': ('BTC: air cooler', OK),
    'ov_buy_cool2': ('BTC: liquid cooling', OK),
    'ov_buy_cpu1': ('BTC: CPU 3.6 GHz', OK),
    'ov_buy_cpu2': ('BTC: CPU 4.2 GHz', OK),
}
RUN = [name for name, _, _ in FUNCS]


def status_line(th, msg, col):
    """The status bar with a message: the bar itself (it covers the idle one) and the text."""
    w, h = WW - 2, 10
    im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    px = im.load()
    for y in range(h):
        for x in range(w):
            px[x, y] = A(th['status'])
    c = th[col] if isinstance(col, str) else col
    msg = msg[:(w - 6) // F.CW]
    im.alpha_composite(F.text(msg, [c] * len(msg)), (3, 1))
    if col == RED:   # an alert marker on the left edge
        for y in range(h):
            px[0, y] = A(th['alert'])
    return im


def bsod():
    """The blue screen over his body (centred on him; theme-independent)."""
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


def theme_frames():
    d = {}
    for k, th in enumerate(THEMES):
        for lang in LANGS:
            d[f'tw_t{k}_{lang}'] = window(th, lang)
        for tag, (msg, col) in OVERLAYS.items():
            d[f'{tag}_t{k}'] = status_line(th, msg, col)
        for name in RUN:
            d[f'ov_run_{name}_t{k}'] = status_line(th, th['run'].format(name), OK)
    d['ov_bsod'] = bsod()
    return d


# ------------------------------------------------------------------ packing

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


def pack_many(frames, name):
    """Round 102: as many sheets as it takes (each at most 2048 tall): name_0, name_1, ..."""
    out, cur, k = {}, {}, 0
    for tag, im in frames.items():
        cur[tag] = im
        try:
            pack(cur)
        except AssertionError:
            del cur[tag]
            out[f'{name}_{k}'] = pack(cur)
            cur, k = {tag: im}, k + 1
    out[f'{name}_{k}'] = pack(cur)
    return out


def build():
    sheets = {}
    for lang, d in line_frames().items():
        sheets.update(pack_many(d, f'coder_code_{lang}'))
    sheets['coder_theme'] = pack(theme_frames())
    return sheets


# ------------------------------------------------------------------ preview: every theme's window with code in it

def compose(theme, lang, func, rows, status=None):
    """The window as the game draws it: window, up to 3 rows (line, step), a status line."""
    lf = line_frames.cache[lang]
    tf = theme_frames.cache
    im = Image.new('RGBA', (WW, WH), (0, 0, 0, 0))
    im.alpha_composite(tf[f'tw_t{theme}_{lang}'], (0, 0))
    for r, (line, step) in enumerate(rows):
        im.alpha_composite(lf[f'ln_{lang}_{func}_{line}_{step}'], (1, 8 + r * 10))
    if status:
        im.alpha_composite(tf[f'{status}_t{theme}'], (1, 38))
    return im


def preview(folder):
    line_frames.cache = line_frames()
    theme_frames.cache = theme_frames()
    bg = (64, 86, 70, 255)
    shots = []
    for k in range(len(THEMES)):
        lang = ['py', 'js', 'py', 'cpp', 'rust', 'asm', 'rust'][k]
        st = [None, 'ov_compile', 'ov_run_ping', 'ov_saved', 'ov_rustc', 'ov_segv', 'ov_run_kill9'][k]
        shots.append(compose(k, lang, 'firewall' if lang != 'asm' else 'chain', [(0, 3), (1, 3), (2, 2)], st))
    W = WW + 12
    out = Image.new('RGBA', (W * 4, (WH + 18) * 2), bg)
    for i, s in enumerate(shots):
        x, y = (i % 4) * W + 6, (i // 4) * (WH + 18) + 4
        out.alpha_composite(s, (x, y))
        lab = THEME_RANKS[i]
        out.alpha_composite(F.text(lab, [(240, 240, 240)] * len(lab)), (x, y + WH + 3))
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'themes.png'))


if __name__ == '__main__':
    S = build()
    import glob
    for old in glob.glob(os.path.join(OUT, 'coder_code_*')) + glob.glob(os.path.join(OUT, 'coder_ui#*')):
        os.remove(old)
    for name, (sheet, fan) in S.items():
        sheet.save(os.path.join(OUT, name + '#sheet.png'), optimize=True)
        with open(os.path.join(OUT, name + '#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fan, fh, separators=(',', ':'))
        print(name, sheet.size, len(fan['anims']))
    if '--preview' in sys.argv:
        preview(os.path.join(HERE, 'preview'))
