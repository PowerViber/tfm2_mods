"""Lab target definitions from shipped champion data; no installed assets copied."""
from pathlib import Path
import json,re,sys
ROOT=Path(__file__).resolve().parents[1]
roster={c['id']:c['name'] for c in json.loads((ROOT/'rules/champions.json').read_text(encoding='utf8'))['champions']}
native=(ROOT/'native/tfm2_custom_ai/src/lib.rs').read_text(encoding='utf8')
table=native[native.index('const MOD_POWER:'):native.index('const MOD_POWER_BUFF:')]
literal={suffix:json.loads(array) for suffix,array in re.findall(r'\("([^"]+)",\s*(\[[^\]]+\])\)',table)}
rules={}
for p in (ROOT/'rules').glob('*.json'):
    d=json.loads(p.read_text(encoding='utf8'))
    if isinstance(d,dict) and d.get('id') and isinstance(d.get('modPower'),list):rules[d['id']]=d['modPower']
targets=[]
for p in sorted((ROOT/'mods').glob('*/champion/*.data_champion')):
    d=json.loads(p.read_text(encoding='utf8'));ident=d.get('id',p.stem);asset=d['sprite'];stem=ROOT/asset.replace('asset/','mods/')
    assert Path(str(stem)+'#sheet.png').exists() and Path(str(stem)+'#anim.fanim').exists(),asset
    power=next((v for suffix,v in literal.items() if ident.endswith(suffix)),rules.get(ident))
    assert power is not None,f'Unresolved Mod Power: {ident}'
    targets.append(dict(id=ident,name=roster.get(ident,ident),kind='champion',stat=d['stat'],growth=d['growth'],
        attack=d['attack'],modPower=power,asset=asset,prefix=d.get('anim_prefix',''),source=p.relative_to(ROOT).as_posix()))
payload={'version':1,'scope':'Configured target stats and shipped base bodies. Opponent abilities, items and passives are not simulated.', 'champions':targets}
out=ROOT/'editor/lab-custom-targets.json';text=json.dumps(payload,separators=(',',':'))+'\n'
if '--check' in sys.argv:
    assert out.read_text(encoding='utf8')==text,'Stale Lab target catalogue'
else:out.write_text(text,encoding='utf8')
print(f'Lab catalogue: {len(targets)} custom champions; Mod Power resolved once from canonical sources')
