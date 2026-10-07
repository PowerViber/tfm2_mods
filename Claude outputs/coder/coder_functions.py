"""Round 101: the Coder's functions, the one source of truth for what he types.

Every function is real code in each language (line 0 is its signature; he types it too). The art (coder_code.py)
draws these lines over his head as he types them; this script also writes native/tfm2_custom_ai/src/coder_code.rs
with each line's length and syntax risk, so the native typing time and typo odds come from the very text shown.

Phase 1: 10 functions (tiers 1-3) in Python, C++ and Rust. Phase 2 adds JavaScript, Assembly and tiers 4-5.

Run from the repo root: python3 "Claude outputs/coder/coder_functions.py"
"""
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
LANGS = ['py', 'cpp', 'rust']

# name, tier, {lang: [lines]}; keep lines within MAX_COLS so the terminal stays readable
MAX_COLS = 28
FUNCS = [
    ('ping', 1, {
        'py': ['def ping(t):', '  t.hp -= 35 + ap // 2'],
        'cpp': ['void ping(Unit* t) {', '  t->hp -= 35 + ap / 2;', '}'],
        'rust': ['fn ping(t: &mut Unit) {', '  t.hp -= 35 + ap / 2;', '}'],
    }),
    ('shield', 1, {
        'py': ['def shield(a):', '  a.shield += 120', '  a.timer = 180'],
        'cpp': ['void shield(Unit* a) {', '  a->shield += 120;', '  a->timer = 180;', '}'],
        'rust': ['fn shield(a: &mut Unit) {', '  a.shield += 120;', '  a.timer = 180;', '}'],
    }),
    ('heal', 1, {
        'py': ['def heal(a):', '  a.hp = min(a.hp + 80,', '             a.max)'],
        'cpp': ['void heal(Unit* a) {', '  a->hp = std::min(', '    a->hp + 80, a->max);', '}'],
        'rust': ['fn heal(a: &mut Unit) {', '  a.hp = (a.hp + 80)', '    .min(a.max);', '}'],
    }),
    ('scan', 1, {
        'py': ['def scan():', '  seen = [e for e in', '    enemies if e.vis]', '  return seen'],
        'cpp': ['auto scan() {', '  std::vector<Unit*> v;', '  for (auto* e : enemies)', '    if (e->vis)',
                '      v.push_back(e);', '  return v;', '}'],
        'rust': ['fn scan(es: &[Unit])', '    -> Vec<&Unit> {', '  es.iter()', '    .filter(|e| e.vis)',
                 '    .collect()', '}'],
    }),
    ('spray', 2, {
        'py': ['def spray():', '  for e in near(me, 250):', '    e.hp -= 25'],
        'cpp': ['void spray() {', '  for (auto* e :', '       near(me, 250))', '    e->hp -= 25;', '}'],
        'rust': ['fn spray(w: &mut World) {', '  for e in w.near_mut(', '      w.me, 250) {', '    e.hp -= 25;', '  }', '}'],
    }),
    ('blink', 2, {
        'py': ['def blink(p):', '  me.pos = away(p, 300)', '  me.cpu -= 10'],
        'cpp': ['void blink(Vec2 p) {', '  me->pos = away(p, 300);', '  me->cpu -= 10;', '}'],
        'rust': ['fn blink(me: &mut Unit,', '         p: Vec2) {', '  me.pos = away(p, 300);', '  me.cpu -= 10;', '}'],
    }),
    ('slow', 2, {
        'py': ['def slow(t):', '  t.speed *= 0.6', '  t.timer = 120'],
        'cpp': ['void slow(Unit* t) {', '  t->speed *= 0.6f;', '  t->timer = 120;', '}'],
        'rust': ['fn slow(t: &mut Unit) {', '  t.speed *= 0.6;', '  t.timer = 120;', '}'],
    }),
    ('cache', 2, {
        'py': ['def cache():', '  me.cpu = min(me.cpu', '    + 40, 100)', '  me.idle(30)'],
        'cpp': ['void cache() {', '  me->cpu = std::min(', '    me->cpu + 40, 100);', '  me->idle(30);', '}'],
        'rust': ['fn cache(me: &mut Unit) {', '  me.cpu = (me.cpu + 40)', '    .min(100);', '  me.idle(30);', '}'],
    }),
    ('chain', 3, {
        'py': ['def chain(t):', '  for i in range(4):', '    t.hp -= 30', '    t = t.nearest()'],
        'cpp': ['void chain(Unit* t) {', '  for (int i = 0; i < 4;', '       ++i) {', '    t->hp -= 30;',
                '    t = t->nearest();', '  }', '}'],
        'rust': ['fn chain(w: &mut World,', '         mut t: Id) {', '  for _ in 0..4 {', '    w[t].hp -= 30;',
                 '    t = w.nearest(t);', '  }', '}'],
    }),
    ('firewall', 3, {
        'py': ['def firewall(a, b):', '  for x in line(a, b):', '    grid[x] = BURN', '  grid.ttl = 240'],
        'cpp': ['void firewall(Vec2 a,', '              Vec2 b) {', '  for (auto x : line(a,b))',
                '    grid[x] = BURN;', '  grid.ttl = 240;', '}'],
        'rust': ['fn firewall(g: &mut Grid,', '    a: Vec2, b: Vec2) {', '  for x in line(a, b) {',
                 '    g[x] = Cell::Burn;', '  }', '  g.ttl = 240;', '}'],
    }),
]

SYMBOLS = set('()[]{}<>:;,.&|*=+-/%!?\'"_#@^~')


def typed(line: str) -> str:
    """What he actually types: the editor indents for him."""
    return line.strip()


def risk(line: str) -> int:
    """Syntax risk of a line, x10: every character counts 10, a symbol 25 (brackets, ->, ::, &mut are where typos
    bite)."""
    t = typed(line)
    return sum(25 if ch in SYMBOLS else 10 for ch in t if ch != ' ') + 5 * t.count(' ')


def check():
    for name, tier, langs in FUNCS:
        assert set(langs) == set(LANGS), name
        for lang, lines in langs.items():
            for ln in lines:
                assert len(ln) <= MAX_COLS, (name, lang, ln, len(ln))
                assert all(32 <= ord(c) < 127 for c in ln), (name, ln)


def write_rust():
    """native/tfm2_custom_ai/src/coder_code.rs: per function its tier and, per language, each line's typed length
    and risk."""
    out = ['//! Generated by Claude outputs/coder/coder_functions.py: do not edit. The Coder\'s functions as the art',
           '//! draws them: per language, each line\'s typed length (characters) and syntax risk (x10).', '',
           f'pub const LANGS: [&str; {len(LANGS)}] = [{", ".join(repr(l).replace(chr(39), chr(34)) for l in LANGS)}];', '',
           '/// (name, tier, per language: [(typed length, risk)] by line)',
           f'pub const FUNCS: [(&str, usize, [&[(usize, usize)]; {len(LANGS)}]); {len(FUNCS)}] = [']
    for name, tier, langs in FUNCS:
        per = []
        for lang in LANGS:
            per.append('&[' + ', '.join(f'({len(typed(ln))}, {risk(ln)})' for ln in langs[lang]) + ']')
        out.append(f'    ("{name}", {tier}, [{", ".join(per)}]),')
    out.append('];')
    path = os.path.join(ROOT, 'native', 'tfm2_custom_ai', 'src', 'coder_code.rs')
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write('\n'.join(out) + '\n')
    return path


if __name__ == '__main__':
    check()
    print('wrote', write_rust())
    for name, tier, langs in FUNCS:
        print(f'{name:9} t{tier}', '  '.join(f'{l}:{sum(len(typed(x)) for x in langs[l])}c/{len(langs[l])}l' for l in LANGS))
