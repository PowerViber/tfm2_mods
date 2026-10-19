"""Read-only PE/SDK boundary check; this is not a private projectile hook."""
from pathlib import Path
import hashlib,json,re,struct
ROOT=Path(__file__).resolve().parents[1]
TARGET=Path('C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2/TeamfightManager2.exe')
data=TARGET.read_bytes();sha=hashlib.sha256(data).hexdigest()
assert sha=='a1d05045405a12e2bed182daeafcad8c8a40cf29ab68ba6ea5bfd30257439582','Different game build; do not reuse private boundary addresses'
pe=struct.unpack_from('<I',data,0x3c)[0];assert data[pe:pe+4]==b'PE\0\0'
assert struct.unpack_from('<H',data,pe+4)[0]==0x8664
opt=pe+24;assert struct.unpack_from('<H',data,opt)[0]==0x20b
base=struct.unpack_from('<Q',data,opt+24)[0];sections=opt+struct.unpack_from('<H',data,pe+20)[0]
def offset(va):
 rva=va-base
 for i in range(struct.unpack_from('<H',data,pe+6)[0]):
  at=sections+40*i;size,start,raw_size,raw=struct.unpack_from('<IIII',data,at+8)
  if start<=rva<start+max(size,raw_size):return raw+rva-start
 raise ValueError('Unmapped VA')
api=(ROOT/'native/mod-api-stable/src/host.rs').read_text();body=api.split('pub struct SimVtableV1 {',1)[1].split('\n}',1)[0]
fields=re.findall(r'^    pub (\w+):',body,re.M);assert len(fields)==90,len(fields)
table=0x143d53258;raw=offset(table);assert struct.unpack_from('<Q',data,raw)[0]==len(fields)*8
names=['projectile_count','projectile_at','projectile_is_valid','projectile_info','entity_set_pos','spawn_projectile']
slots=[{'name':n,'slot':fields.index(n),'functionVa':hex(struct.unpack_from('<Q',data,raw+8*fields.index(n))[0])} for n in names]
report={'target':str(TARGET),'sha256':sha,'format':'PE32+ x86-64','bytes':len(data),'simulationVtableVa':hex(table),'vtableFileOffset':hex(raw),'tableBytes':720,'observedSlots':slots,'publicProjectileFields':[n for n in fields if 'projectile' in n],'privateKinematicsRecovered':False,'stockShotBendingEnabled':False,'scope':'Static read-only API boundary. The absence of a public setter does not prove an internal hook impossible. No live projectile proof and no executable changes.'}
(ROOT/'docs/gojo-results/v3-projectile-boundary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
