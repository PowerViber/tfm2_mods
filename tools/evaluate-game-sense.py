"""Do the athlete stats play like real teams? Compare the stats rollout with reference games.

Runs the athlete-stats rollout (`cargo test --release --lib athlete_stats_table -- --ignored`), which
plays team A at 20, 50 and 80 in one rating or profile value against an average team B. Then:

1. writes the table to docs/round-139-results/athlete-stats.json;
2. checks the direction: a higher rating should not take fewer objectives, die more to unseen enemies,
   or start fewer fights with numbers;
3. when tools/reference-games/*.json holds documented games, reports for each metric which level of
   each value is closest to the stronger side's numbers.

`--from FILE` reuses saved STATS lines instead of running cargo.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/round-139-results/athlete-stats.json'
REFS = ROOT / 'tools/reference-games'
# metric: (label, higher is better for the stronger macro side)
METRICS = {
    'objectiveShare': ('Objective share', True),
    'fightsAheadShare': ('Fights started with more players', True),
    'roamsPer10': ('Roams per 10 minutes', None),
    'unseenDeathShare': ('Deaths to unseen enemies', False),
}
GAME_MINUTES = 20


def stats_lines():
    if '--from' in sys.argv:
        return Path(sys.argv[sys.argv.index('--from') + 1]).read_text().splitlines()
    p = subprocess.run(['cargo', 'test', '--release', '--lib', 'athlete_stats_table', '--', '--ignored', '--nocapture'],
                       cwd=ROOT / 'native/tfm2_custom_ai', capture_output=True, text=True)
    if p.returncode:
        raise SystemExit(p.stdout[-4000:] + p.stderr[-4000:])
    return p.stdout.splitlines()


def share(pair, side=0):
    total = sum(pair)
    return round(pair[side] / total, 4) if total else None


def sim_metrics(row, seeds):
    return {
        'objectiveShare': share(row['objectives']),
        'fightsAheadShare': share(row['fightsAhead']),
        'roamsPer10': round(row['rotations'][0] / seeds / GAME_MINUTES * 10, 2),
        'unseenDeathShare': round(row['unseenDeaths'][0] / max(1, row['deaths'][0]), 4),
        'deathsPerGame': round(row['deaths'][0] / seeds, 2),
        'wavesPerGame': round(row['waves'][0] / seeds, 2),
    }


def ref_metrics(game, side):
    ev = game['events']
    other = 'red' if side == 'blue' else 'blue'
    mine = lambda kind: [e for e in ev if e['type'] == kind and e.get('side') == side]
    theirs = lambda kind: [e for e in ev if e['type'] == kind and e.get('side') == other]
    fights = [e for e in ev if e['type'] == 'teamfight']
    i = 0 if side == 'blue' else 1
    ahead = sum(1 for f in fights if f['numbers'][i] > f['numbers'][1 - i])
    behind = sum(1 for f in fights if f['numbers'][i] < f['numbers'][1 - i])
    deaths = mine('death')
    minutes = max(1, game['durationS'] / 60)
    obj = (len(mine('objective')), len(theirs('objective')))
    return {
        'objectiveShare': share(obj),
        'fightsAheadShare': share((ahead, behind)),
        'roamsPer10': round(len(mine('roam')) / minutes * 10, 2),
        'unseenDeathShare': round(sum(1 for d in deaths if d.get('unseen')) / len(deaths), 4) if deaths else None,
    }


def main():
    rows = [json.loads(l.split('STATS ', 1)[1]) for l in stats_lines() if l.startswith('STATS ')]
    if not rows:
        raise SystemExit('no STATS lines')
    seeds = 96
    table = {}
    for r in rows:
        table.setdefault(r['stat'], {})[r['level']] = sim_metrics(r, seeds)
    # direction: 80 against 20 for the three metrics with a known good direction
    checks = []
    for stat, levels in table.items():
        lo, hi = levels[20], levels[80]
        for m, (label, up) in METRICS.items():
            if up is None or lo[m] is None or hi[m] is None:
                continue
            delta = hi[m] - lo[m]
            ok = delta >= -0.01 if up else delta <= 0.01
            checks.append({'stat': stat, 'metric': label, 'at20': lo[m], 'at80': hi[m], 'rightWay': ok})
    refs = [p for p in sorted(REFS.glob('*.json')) if not p.name.startswith('TEMPLATE')]
    calibration = []
    for p in refs:
        game = json.loads(p.read_text(encoding='utf-8'))
        strong = ref_metrics(game, game['stronger'])
        for stat, levels in table.items():
            for m, (label, _) in METRICS.items():
                if strong[m] is None:
                    continue
                best = min(levels, key=lambda lv: abs((levels[lv][m] or 0) - strong[m]))
                calibration.append({'game': game['title'], 'stat': stat, 'metric': label, 'reference': strong[m], 'closestLevel': best})
    OUT.parent.mkdir(parents=True, exist_ok=True)
    doc = {
        'model': 'athlete_stats_fixture: 5v5, 20 minutes, mirrored three-lane map, real team planner; '
                 'fights go through athlete_play::decide. Team A at the level, team B at 50; '
                 f'{seeds} seeds, sides alternate. A model of the macro layer, not the engine.',
        'table': table, 'directionChecks': checks,
        'referenceGames': [p.name for p in refs], 'calibration': calibration,
    }
    OUT.write_text(json.dumps(doc, indent=1) + '\n', encoding='utf-8')
    wrong = [c for c in checks if not c['rightWay']]
    print(f'{len(rows)} rows; {len(checks) - len(wrong)}/{len(checks)} direction checks the right way; '
          f'{len(refs)} reference games')
    for c in wrong:
        print(f"  wrong way: {c['stat']} {c['metric']}: {c['at20']} at 20 -> {c['at80']} at 80")
    if not refs:
        print('  no reference games yet: add tools/reference-games/*.json (see its README)')


if __name__ == '__main__':
    main()
