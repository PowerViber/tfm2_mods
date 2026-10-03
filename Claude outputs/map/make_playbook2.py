"""Starter playbook v2 for the Map tab: only the map-altering champions (Omen smokes, Steve walls), per line-up."""
import json
D = json.load(open('/home/claude/testenv/tfm2/Teamfight Manager 2/mods/tfm2_custom_ai/map_dump.json'))
W = D['walls']
def wall(x, y): return x < 0 or y < 0 or x >= 960000 or y >= 960000 or W[int(y // 32000)][int(x // 32000)] != 0
S, M = (672000, 672000), (288000, 288000)
plans = []
def add(comp, kind, trigger, prio, a, note, b=None, fake=False):
    plans.append({'id': 'p2_' + format(len(plans), '02d'), 'kind': kind, 'trigger': trigger, 'side': 'both', 'prio': prio, 'note': note,
                  'fake': fake, 'comp': sorted(comp), 'a': list(a), 'b': list(b or a)})
def smokes(comp, chokes):
    add(comp, 'smoke', 'serpen', 3, S, 'Smoke the Serpent pit while we take it')
    add(comp, 'smoke', 'epic', 3, M, 'Smoke the Morgard pit while we take it')
    add(comp, 'smoke', 'serpen_fight', 1, M, 'Fake a Morgard take to split them', fake=True)
    add(comp, 'smoke', 'epic_fight', 1, S, 'Fake a Serpent take to split them', fake=True)
    add(comp, 'smoke', 'teamfight', 2, (480000, 480000), 'Smoke the mid crossing in a teamfight')
    add(comp, 'smoke', 'tower', 3, (420000, 560000), 'Hide our defence of the mid tower')
    if chokes:   # without Steve, Omen covers the chokes himself
        add(comp, 'smoke', 'serpen_fight', 4, (575000, 575000), 'Blind the river from mid in the Serpent fight')
        add(comp, 'smoke', 'serpen_fight', 3, (800000, 600000), "Blind their jungle's way to the Serpent")
        add(comp, 'smoke', 'epic_fight', 4, (385000, 385000), 'Blind the river from mid in the Morgard fight')
        add(comp, 'smoke', 'epic_fight', 3, (440000, 230000), "Blind their jungle's way to Morgard")
def walls(comp):
    add(comp, 'wall', 'isolate', 4, (606000, 544000), 'Cut mid off from the Serpent fight (river choke)', b=(544000, 606000))
    add(comp, 'wall', 'isolate', 3, (640000, 600000), 'Cut their jungle off from the Serpent', b=(940000, 600000))
    add(comp, 'wall', 'isolate', 4, (415000, 355000), 'Cut mid off from the Morgard fight (river choke)', b=(355000, 415000))
    add(comp, 'wall', 'isolate', 3, (370000, 165000), 'Cut their jungle off from Morgard', b=(370000, 400000))
smokes(['omen'], True)
walls(['steve'])
walls(['omen', 'steve']); smokes(['omen', 'steve'], False)
bad = [(k['note'], p) for k in plans for p in (k['a'], k['b']) if wall(*p)]
print('problems:', bad, len(plans), 'plans')
json.dump({'version': 2, 'flipY': False, 'comp': ['omen', 'steve'], 'marks': plans}, open('tactics2.json', 'w'), indent=2)
