"""Round 101: the Coder's pixel monospace font (printable ASCII), 5 px a character, 8 px a line.

The glyphs were taken once from DejaVu Sans Mono at 8 px (no anti-aliasing) and are kept in coder_font.json, so the
generators don't depend on the fonts installed on the machine. Run this file only to rebuild that table.

text(...) draws a line of code with syntax colours: keywords cyan, numbers orange, strings yellow, symbols grey,
names green.
"""
import json
import os
import re

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = os.path.join(HERE, 'coder_font.json')
CW, CH = 5, 8          # cell
BASE = 6               # rows above the baseline

KEYWORDS = {
    'def', 'for', 'in', 'if', 'else', 'return', 'while', 'True', 'None', 'min', 'max', 'range', 'and', 'or', 'not',
    'void', 'auto', 'int', 'float', 'std', 'const', 'struct', 'fn', 'let', 'mut', 'impl', 'pub', 'use', 'self', 'me',
    'Vec', 'Vec2', 'Unit', 'World', 'Grid', 'Cell', 'Id', 'push_back', 'iter', 'filter', 'collect', 'vector',
}
GREEN = (120, 255, 150)
CYAN = (110, 220, 255)
ORANGE = (255, 180, 90)
YELLOW = (255, 230, 120)
GREY = (190, 200, 210)
PINK = (255, 140, 200)


def build_table():
    """(Round 101: x, m, a, w, V and v were then redrawn by hand in the table; rebuilding drops that.)"""
    from PIL import ImageDraw, ImageFont
    font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf', 8)
    glyphs = {}
    for o in range(32, 127):
        ch = chr(o)
        im = Image.new('L', (12, 12), 0)
        d = ImageDraw.Draw(im)
        d.fontmode = '1'
        d.text((0, -1), ch, font=font, fill=255)
        rows = []
        for y in range(CH):
            rows.append(''.join('#' if im.getpixel((x, y)) > 127 else '.' for x in range(CW)))
        glyphs[ch] = rows
    # round 108: the font draws '_' below the cell; put it on the descender row (ddos_all read as "ddos all")
    glyphs['_'] = ['.....'] * (CH - 1) + ['#####']
    with open(TABLE, 'w', encoding='utf-8') as fh:
        json.dump(glyphs, fh, indent=0)
    return glyphs


_G = None


def glyphs():
    global _G
    if _G is None:
        if not os.path.exists(TABLE):
            build_table()
        with open(TABLE, encoding='utf-8') as fh:
            _G = json.load(fh)
    return _G


def colours(line):
    """A colour per character: syntax highlighting."""
    out = [GREY] * len(line)
    for m in re.finditer(r'[A-Za-z_][A-Za-z_0-9]*|\d+(\.\d+)?f?|"[^"]*"', line):
        tok = m.group(0)
        c = CYAN if tok in KEYWORDS else ORANGE if tok[0].isdigit() else YELLOW if tok[0] == '"' else GREEN
        for i in range(m.start(), m.end()):
            out[i] = c
    return out


def text(line, cols=None, alpha=255, shade=None):
    """The line as an RGBA image, CW px a character, CH tall; `shade` (0..1) dims it."""
    g = glyphs()
    cols = cols or colours(line)
    im = Image.new('RGBA', (max(1, CW * len(line)), CH), (0, 0, 0, 0))
    px = im.load()
    for i, ch in enumerate(line):
        rows = g.get(ch, g['?'])
        c = cols[i]
        if shade is not None:
            c = tuple(int(v * shade) for v in c)
        for y, row in enumerate(rows):
            for x, bit in enumerate(row):
                if bit == '#':
                    px[i * CW + x, y] = c + (alpha,)
    return im


if __name__ == '__main__':
    g = build_table()
    sample = text('def firewall(a, b):  t->hp -= 35 + ap / 2; {}[]|& "pwned"')
    sample.resize((sample.width * 4, sample.height * 4), Image.NEAREST).save(os.path.join(HERE, 'preview', 'font.png'))
    print(len(g), 'glyphs')
