"""Starter playbook for the Map tab (Blue's view, mirrored for Red across the river diagonal)."""
import json, math, random
D = json.load(open('/home/claude/testenv/tfm2/Teamfight Manager 2/mods/tfm2_custom_ai/map_dump.json'))
W, B = D['walls'], D['bushes']
def wall(x, y): return x < 0 or y < 0 or x >= 960000 or y >= 960000 or W[int(y // 32000)][int(x // 32000)] != 0
def bush(x, y): return B[int(y // 32000)][int(x // 32000)] != 0
S, M = (672000, 672000), (288000, 288000)
plans = []
def add(kind, trigger, prio, a, note, b=None, r=None, who=None):
    k = {'id': 'pb' + format(len(plans), '02d'), 'kind': kind, 'champ': 'team', 'trigger': trigger, 'side': 'both', 'prio': prio, 'note': note,
         'a': list(a), 'b': list(b or a)}
    if r: k['r'] = r
    if who: k['who'] = who
    plans.append(k)
# --- Serpent (the bottom-right end of the river)
add('control', 'serpen_fight', 4, S, 'Serpent fight: everyone contests together', r=70000, who='all')
add('control', 'won_serpen', 5, S, 'Won a fight: go take the Serpent', r=70000, who='all')
add('control', 'ahead_serpen', 4, S, 'More of us alive: take the Serpent', r=70000, who='all')
add('avoid', 'behind_serpen', 4, S, "Fewer of us alive: don't force the Serpent", r=60000)
add('cover', 'serpen', 3, S, 'Smoke the pit while we take it', r=60000)
add('block', 'isolate', 4, (606000, 544000), 'Cut mid off from the Serpent fight (river choke)', b=(544000, 606000))
add('block', 'isolate', 3, (640000, 600000), 'Cut their jungle off from the Serpent', b=(940000, 600000))
add('ambush', 'serpen_fight', 3, (656000, 816000), 'Surprise from the bush south of the pit', r=40000, who='near1')
add('flank', 'serpen_fight', 2, (496000, 688000), 'Jungler flanks through the west bush', r=40000, who='jungle')
add('fake', 'serpen_fight', 1, M, 'Fake a Morgard take to split them', r=50000)
# --- Morgard (the top-left end of the river)
add('control', 'epic_fight', 4, M, 'Morgard fight: everyone contests together', r=70000, who='all')
add('control', 'won_epic', 5, M, 'Won a fight: go take Morgard', r=70000, who='all')
add('control', 'ahead_epic', 4, M, 'More of us alive: take Morgard', r=70000, who='all')
add('avoid', 'behind_epic', 4, M, "Fewer of us alive: don't force Morgard", r=60000)
add('cover', 'epic', 3, M, 'Smoke the pit while we take it', r=60000)
add('block', 'isolate', 4, (415000, 355000), 'Cut mid off from the Morgard fight (river choke)', b=(355000, 415000))
add('block', 'isolate', 3, (370000, 165000), 'Cut their jungle off from Morgard', b=(370000, 400000))
add('ambush', 'epic_fight', 3, (144000, 272000), 'Surprise from the top-lane bush', r=40000, who='near1')
add('flank', 'epic_fight', 2, (400000, 432000), 'Jungler flanks through the mid bush', r=40000, who='jungle')
add('fake', 'epic_fight', 1, S, 'Fake a Serpent take to split them', r=50000)
# --- ganks and roams
add('group', 'gank', 3, (430000, 530000), 'Gank mid', r=50000, who='jungle')
add('group', 'gank', 2, (80000, 420000), 'Gank top', r=60000, who='jungle')
add('group', 'gank', 2, (540000, 880000), 'Gank bot', r=60000, who='jungle')
add('group', 'gank', 1, (600000, 880000), 'Mid roams bot', r=60000, who='mid')
add('group', 'gank', 1, (80000, 360000), 'Mid roams top', r=60000, who='mid')
# --- defending
add('group', 'tower', 5, (150000, 810000), 'Defend the base', r=70000, who='all')
add('control', 'tower', 4, (368000, 592000), 'Defend the mid tower', r=60000, who='all')
add('control', 'tower', 3, (48000, 272000), 'Defend the top tower', r=60000, who='near2')
add('control', 'tower', 3, (688000, 912000), 'Defend the bot tower', r=60000, who='near2')
add('safe', 'always', 3, (240000, 720000), 'Low HP: fall back behind the inner mid tower', r=40000)
# --- fights and pushes
add('control', 'won', 2, (592000, 368000), 'Won a fight, nothing to take: push their mid tower', r=60000, who='all')
add('flank', 'teamfight', 2, (528000, 560000), 'One flanks teamfights through the river bush', r=30000, who='near1')
add('ambush', 'teamfight', 2, (400000, 432000), 'Omen lurks in the mid bush in teamfights', r=35000, who='omen')
# checks: spots not in walls, ambush/flank spots in bushes
bad = []
for k in plans:
    for p in (k['a'], k['b']):
        if wall(*p): bad.append((k['note'], p, 'wall'))
    if k['kind'] in ('ambush', 'flank') and not bush(*k['a']): bad.append((k['note'], k['a'], 'not a bush'))
print('problems:', bad)
json.dump({'version': 1, 'flipY': False, 'marks': plans, 'tokens': []}, open('tactics.json', 'w'), indent=2)
print(len(plans), 'plans')
