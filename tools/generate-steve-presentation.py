"""Reuse original effects through canonical native/editor bindings. No new effects."""
from pathlib import Path
import json,shutil
ROOT=Path(__file__).resolve().parents[1]
def generate():
    data=json.loads((ROOT/'rules/steve-presentation.json').read_text())
    p=ROOT/'mods/tfm2_blockcraft/champion/tfm2_blockcraft_steve.data_champion';champ=json.loads(p.read_text())
    for key,tag in data['bindings'].items():
        name='tfm2_blockcraft_craft_'+key
        view=next((v for v in champ['view_effects']if v['name']==name),None)
        if view is None:
            view=dict(type='Animation',name=name,z=2,is_follow=False);champ['view_effects'].append(view)
        view.update(anim=data['legacyAtlas'],tag=tag)
    for tag in ['pearl','tnt_fly','apple']:
        name='tfm2_blockcraft_steve_'+tag
        if not any(v['name']==name for v in champ['view_effects']):champ['view_effects'].append(dict(type='Animation',name=name,anim=data['legacyAtlas'],tag=tag,z=2,is_follow=False))
    p.write_text(json.dumps(champ,indent=2)+'\n')
    (ROOT/'editor/steve-preset.js').write_text('window.STEVE_PRESET='+json.dumps(champ,separators=(',',':'))+';\n')
    (ROOT/'editor/steve-presentation.js').write_text('window.STEVE_PRESENTATION='+json.dumps(data,separators=(',',':'))+';\n')
    for src,dest in [('blockcraft#sheet.png','steve-legacy.png'),('blockcraft#anim.fanim','steve-legacy.fanim')]:shutil.copyfile(ROOT/'mods/tfm2_blockcraft/vfx'/src,ROOT/'editor'/dest)
    import runpy
    runpy.run_path(str(ROOT/'tools/generate-steve-eye-alignment.py'))['generate']()
    print('Restored',len(data['bindings']),'legacy presentation bindings; original atlas bytes unchanged.')
if __name__=='__main__':generate()
