"""Keep Skill Lab from saving retired Omen/Dio controllers back to disk."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
def generate():
    result={}
    for key,id in [('dio','tfm2_jojo_dio'),('omen','tfm2_valorant_omen'),('aegis','tfm2_gundam_aegis_zero')]:
        p=next((ROOT/'mods').glob('*/champion/'+id+'.data_champion'));text=json.loads((p.parent.parent/'text/champion.i18n').read_text())
        result[key]=dict(json=json.loads(p.read_text()),text=text.get('en',{}).get('description',{}).get(id,{}))
    (ROOT/'editor/native-tactics-presets.js').write_text('window.NATIVE_TACTICS_PRESETS='+json.dumps(result,separators=(',',':'))+';\n')
if __name__=='__main__':generate()
