"""Round 60: let mod champions' damage skills hit monsters and minions (SwitchByBuff mod_farm). Run on the PC."""
import json, glob, os, shutil, sys
MODS = os.path.expanduser('~/mnt/Teamfight Manager2/mods')
spec = json.load(open(os.path.join(os.path.dirname(__file__), 'farm_spec.json')))
bk = os.path.expanduser('~/mnt/tfm2/Claude outputs/backup-before-round60')
dry = '--dry' in sys.argv
touched = set()
for f in glob.glob(MODS + '/*/champion/*.data_champion'):
    j = json.load(open(f, encoding='utf-8'))
    sp = spec.get(j.get('id'))
    if not sp: continue
    mod = f.split('/mods/')[1].split('/')[0]
    changed = []
    for slot, v in sp.items():
        a = j[slot]
        if a.get('casting_target') != 'EnemyWithoutTower':
            a['casting_target'] = 'EnemyWithoutTower'; changed.append(slot + ' target')
        e = a['effect']
        if v['farm'] and not (e.get('type') == 'SwitchByBuff' and e.get('buff_name') == 'mod_farm'):
            a['effect'] = {'type': 'SwitchByBuff', 'buff_name': 'mod_farm', 'effect_none': e, 'effect_buff': v['farm']}
            changed.append(slot + ' farm')
    print(mod, j['id'], changed)
    if changed and not dry:
        os.makedirs(os.path.join(bk, mod, 'champion'), exist_ok=True)
        shutil.copy2(f, os.path.join(bk, mod, 'champion', os.path.basename(f)))
        json.dump(j, open(f, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
        touched.add(mod)
    # text
    t = os.path.join(MODS, mod, 'text', 'champion.i18n')
    if os.path.exists(t):
        d = json.load(open(t, encoding='utf-8'))
        tc = []
        for lang, L in d.items():
            desc = L.get('description', {}).get(j['id'])
            if not desc: continue
            for slot, v in sp.items():
                if v['text'] and slot in desc and 'Farming:' not in desc[slot]:
                    desc[slot] = desc[slot] + ' ' + v['text']; tc.append(lang + ':' + slot)
        if tc and not dry:
            os.makedirs(os.path.join(bk, mod, 'text'), exist_ok=True)
            if not os.path.exists(os.path.join(bk, mod, 'text', 'champion.i18n')):
                shutil.copy2(t, os.path.join(bk, mod, 'text', 'champion.i18n'))
            json.dump(d, open(t, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
            touched.add(mod)
        print('   text', tc)
print('touched', sorted(touched))
