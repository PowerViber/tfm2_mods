"""Explicit ability migration; inspection is the default and in-place writes require a verified backup.

Updates Gojo/Frieren/Steve ability definitions, Unified's view lists (round 138), and the five
champions' native passive bindings. Career records, history, base statistics, growth and embedded balance patches
remain byte-for-byte unchanged outside those champion JSON fields.
"""
from pathlib import Path
import argparse,binascii,copy,gzip,hashlib,importlib.util,json,shutil,struct,time
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('magnus_container',Path(__file__).with_name('migrate-magnus-career.py'))
container=importlib.util.module_from_spec(spec);spec.loader.exec_module(container)
IDS=['tfm2_custom_unified_theory','tfm2_frieren_frieren','tfm2_custom_magnus','tfm2_custom_gojo','tfm2_blockcraft_steve']
def definitions():
 return {id:json.loads(next((ROOT/'mods').glob('*/champion/'+id+'.data_champion')).read_text()) for id in IDS}
def migrate(data:bytes,defs=None):
 defs=defs or definitions();header,payload=container.unpack(data);parts=[];cursor=0;report={'found':{k:0 for k in IDS},'changed':{k:0 for k in IDS},'conflicts':[]}
 while True:
  at=payload.find(b'1HCM',cursor)
  if at<0:parts.append(payload[cursor:]);break
  if at+16>len(payload):parts.append(payload[cursor:]);break
  size=struct.unpack_from('<Q',payload,at+8)[0];end=at+16+size
  try:
   if end>len(payload):raise ValueError('incomplete record')
   e=json.loads(payload[at+16:end]);id=e.get('id')
  except (ValueError,UnicodeError,AttributeError):parts.append(payload[cursor:at+4]);cursor=at+4;continue
  parts.append(payload[cursor:at]);record=payload[at:end]
  if id in defs:
   report['found'][id]+=1;before=copy.deepcopy(e);d=defs[id]
   allowed={'tfm2_custom_unified_theory':'tfm2_custom_ai:unified_theory','tfm2_custom_magnus':'tfm2_custom_ai:magnus','tfm2_frieren_frieren':'tfm2_custom_ai:frieren','tfm2_custom_gojo':'tfm2_custom_ai:gojo','tfm2_blockcraft_steve':'tfm2_custom_ai:steve'}[id]
   if (e.get('passive')or{}).get('passive_ref') not in (None,allowed,*(['tfm2_custom_ai:gojo_v2'] if id=='tfm2_custom_gojo' else ['tfm2_custom_ai:steve_craft'] if id=='tfm2_blockcraft_steve' else [])):report['conflicts'].append(id)
   else:
    e['passive']=d['passive'];e['passive_ult']=d['passive_ult']
    if id in ('tfm2_custom_gojo','tfm2_blockcraft_steve'):
     for key in ['attack','skill','skill2','ult','view_buffs','view_effects','view_projectiles']:e[key]=d.get(key,[])
    if id=='tfm2_custom_unified_theory':
     # Round 138: the equation kit replaced every recipe effect; the old views name sheets that no longer ship.
     for key in ['view_buffs','view_effects','view_projectiles']:e[key]=d.get(key,[])
    if id=='tfm2_frieren_frieren':
     for key in ['skill','skill2','ult']:e[key]=d[key]
     e['attack']['cooltime']=72
     attack=e['attack']['effect']
     if attack.get('type')!='LinearProjectile':report['conflicts'].append('Nonstandard Frieren basic attack')
     else:
      effects=attack.get('applied_effects',[])
      damage=next((v['effect'] for v in effects if v.get('effect',{}).get('type')=='ApAttack'),None)
      if damage is None:report['conflicts'].append('Missing Frieren basic damage')
      else:damage.update(damage=20,attack_ratio=45)
     for key in ['view_buffs','view_effects','view_projectiles']:
      views={v['name']:v for v in e.get(key,[])};views.update({v['name']:v for v in d.get(key,[])});e[key]=list(views.values())
    assert e.get('stat')==before.get('stat') and e.get('growth')==before.get('growth')
    if e!=before:
     encoded=json.dumps(e,ensure_ascii=False,separators=(',',':')).encode();record=payload[at:at+8]+struct.pack('<Q',len(encoded))+encoded;report['changed'][id]+=1
  parts.append(record);cursor=end
 if report['conflicts']:return data,report
 if not any(report['changed'].values()):return data,report
 compressed=gzip.compress(b''.join(parts),compresslevel=6,mtime=0);h=bytearray(header);struct.pack_into('<Q',h,13,len(compressed));struct.pack_into('<I',h,21,binascii.crc32(compressed)&0xffffffff);out=bytes(h)+compressed;container.unpack(out);return out,report
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);g=p.add_mutually_exclusive_group();g.add_argument('--output',type=Path);g.add_argument('--in-place',action='store_true');a=p.parse_args();original=a.source.read_bytes();out,report=migrate(original);report.update(sourceSha256=hashlib.sha256(original).hexdigest(),resultSha256=hashlib.sha256(out).hexdigest())
 if report['conflicts']:raise SystemExit(json.dumps(report,indent=2))
 if a.output:
  if a.output.exists() or a.output.resolve()==a.source.resolve():raise SystemExit('Output must be a new file')
  a.output.write_bytes(out);report['output']=str(a.output.resolve())
 elif a.in_place and out!=original:
  backup=a.source.with_name(a.source.name+'.before-reworks-'+str(time.time_ns())+'.bak');shutil.copy2(a.source,backup)
  if backup.read_bytes()!=original:raise SystemExit('Backup verification failed; original unchanged')
  temp=a.source.with_name(a.source.name+'.reworks.tmp');stream=temp.open('xb');stream.write(out);stream.close();temp.replace(a.source);report['backup']=str(backup)
 else:report['mode']='inspection; input unchanged'
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
