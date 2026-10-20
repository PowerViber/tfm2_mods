from pathlib import Path
import binascii,gzip,importlib.util,json,struct,unittest,tempfile,subprocess
spec=importlib.util.spec_from_file_location('reworks',Path(__file__).with_name('migrate-champion-reworks.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def packed(entries):
 payload=b'career-history-STAYS\x00'+b''.join(b'1HCMABCD'+struct.pack('<Q',len(e))+e for e in entries)+b'athlete-records-STAY';c=gzip.compress(payload,mtime=0);h=bytearray(b'TFM2'+b'\0'*21);struct.pack_into('<Q',h,13,len(c));struct.pack_into('<I',h,21,binascii.crc32(c)&0xffffffff);return bytes(h)+c
class Migration(unittest.TestCase):
 def test_tactical_controllers_and_aegis_refresh_without_balance_or_history_changes(self):
  for id,old in [('tfm2_jojo_dio','dio'),('tfm2_valorant_omen','omen'),('tfm2_gundam_aegis_zero','gundam')]:
   with self.subTest(id=id):
    e=json.loads(json.dumps(m.definitions()[id]));e['passive']={'passive_ref':'tfm2_custom_ai:'+old,'params':{}};e['ult']['effect']={'type':'Combine','effects':[]};e['stat']['hp']=9876;e['growth']['hp']=123;e['items']=['keep:item'];e['career_history']=[11,22,33]
    source=packed([json.dumps(e).encode()]);result,report=m.migrate(source);self.assertEqual(report['changed'][id],1);_,payload=m.container.unpack(result)
    for value in [b'"hp":9876',b'"hp":123',b'"items":["keep:item"]',b'"career_history":[11,22,33]']:self.assertIn(value,payload)
    self.assertEqual(m.migrate(result)[0],result)
    if id=='tfm2_gundam_aegis_zero':self.assertIn(b'gdm_shield_v2',payload)
 def test_gojo_current_controller_refreshes_stale_art_without_career_changes(self):
  e=json.loads(json.dumps(m.definitions()['tfm2_custom_gojo']));e['view_effects']=[];e['view_buffs']=[];e['stat']['hp']=4321;e['career_history']=[7,9];e['items']=['keep']
  source=packed([json.dumps(e).encode()]);out,report=m.migrate(source);self.assertEqual(report['changed']['tfm2_custom_gojo'],1)
  _,payload=m.container.unpack(out);self.assertIn(b'"hp":4321',payload);self.assertIn(b'"items":["keep"]',payload);self.assertIn(b'"career_history":[7,9]',payload);self.assertIn(b'void_full_expand0',payload);self.assertEqual(m.migrate(out)[0],out)
 def test_gojo_v1_to_v2_then_idempotent(self):
  e=json.loads(json.dumps(m.definitions()['tfm2_custom_gojo']));e['passive']={'passive_ref':'tfm2_custom_ai:gojo','params':{}};e['attack']['effect']={'type':'Native','effect_ref':'tfm2_custom_ai:gojo_basic'};e['items']=['preserved'];e['career_history']=[1,2,3]
  out,report=m.migrate(packed([json.dumps(e).encode()]));self.assertEqual(report['changed']['tfm2_custom_gojo'],1);_,payload=m.container.unpack(out);self.assertIn(b'tfm2_custom_ai:gojo_v2',payload);self.assertIn(b'tfm2_custom_ai:gojo_basic_v2',payload);again,report=m.migrate(out);self.assertEqual(out,again);self.assertFalse(any(report['changed'].values()))
 def test_gojo_legacy_kit_preserves_stats_items_and_history(self):
  e=m.definitions()['tfm2_custom_gojo'];e=json.loads(json.dumps(e));e.pop('passive');e['attack']['effect']={'type':'AdAttack','damage':0,'attack_ratio':100};e['skill']['effect']={'type':'AddCasterBuff','buff_state':{'name':'gojo_blue'}};e['career_history']=[4,8,15];e['items']=['test:item'];e['stat']['hp']=4321
  out,report=m.migrate(packed([json.dumps(e).encode()]));self.assertEqual(report['changed']['tfm2_custom_gojo'],1);_,payload=m.container.unpack(out);self.assertIn(b'"hp":4321',payload);self.assertIn(b'"career_history":[4,8,15]',payload);self.assertIn(b'"items":["test:item"]',payload);self.assertIn(b'tfm2_custom_ai:gojo_basic',payload);self.assertNotIn(b'"buff_state":{"name":"gojo_blue"',payload)
 def test_gojo_unknown_controller_aborts_the_complete_update(self):
  e=json.loads(json.dumps(m.definitions()['tfm2_custom_gojo']));e['passive']={'passive_ref':'other:gojo'};source=packed([json.dumps(e).encode()]);out,report=m.migrate(source);self.assertEqual(out,source);self.assertEqual(report['conflicts'],['tfm2_custom_gojo'])
 def test_file_inspection_and_backup_are_idempotent(self):
  e=json.loads(json.dumps(m.definitions()['tfm2_custom_gojo']));e.pop('passive');original=packed([json.dumps(e).encode()])
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'fixture.tfm2db';p.write_bytes(original);command=['rtk','proxy','python',str(Path(__file__).with_name('migrate-champion-reworks.py')),str(p)]
   subprocess.run(command,check=True,capture_output=True);self.assertEqual(p.read_bytes(),original);self.assertEqual(list(Path(d).glob('*.bak')),[])
   subprocess.run(command+['--in-place'],check=True,capture_output=True);backups=list(Path(d).glob('*.bak'));self.assertEqual(len(backups),1);self.assertEqual(backups[0].read_bytes(),original);updated=p.read_bytes();self.assertNotEqual(updated,original)
   subprocess.run(command+['--in-place'],check=True,capture_output=True);self.assertEqual(p.read_bytes(),updated);self.assertEqual(len(list(Path(d).glob('*.bak'))),1)
 def test_fields_and_idempotence(self):
  defs=m.definitions();entries=[]
  for id,e in defs.items():
   e=json.loads(json.dumps(e));e['stat']['hp']=2345;e['growth']['hp']=173;e['passive_ult']=None
   if id=='tfm2_frieren_frieren':e.pop('passive');e['attack']['cooltime']=140;e['skill']['effect']={'type':'Combine','effects':[]}
   if id=='tfm2_blockcraft_steve':e['passive']={'passive_ref':'tfm2_custom_ai:steve','params':{}};e['ult']['effect']={'type':'AddCasterBuff','buff_state':{'name':'stv_boat'}}
   entries.append(json.dumps(e).encode())
  unrelated=b'{"id":"other","stat":{"hp":765},"history":[1,2,3]}';source=packed(entries+[unrelated]);out,report=m.migrate(source);self.assertEqual(sum(report['changed'].values()),5);self.assertFalse(report['conflicts']);_,payload=m.container.unpack(out);self.assertIn(unrelated,payload);self.assertTrue(payload.startswith(b'career-history-STAYS\x00'));self.assertTrue(payload.endswith(b'athlete-records-STAY'));self.assertEqual(payload.count(b'"hp":2345'),5);self.assertEqual(payload.count(b'"hp":173'),5);again,report=m.migrate(out);self.assertEqual(again,out);self.assertEqual(sum(report['changed'].values()),0)
 def test_conflicting_controller_and_bad_checksum_never_write(self):
  e=m.definitions()['tfm2_custom_magnus'];e['passive']={'passive_ref':'other:controller'};source=packed([json.dumps(e).encode()]);out,report=m.migrate(source);self.assertEqual(out,source);self.assertTrue(report['conflicts']);bad=bytearray(source);bad[-3]^=1
  with self.assertRaises(ValueError):m.migrate(bytes(bad))
if __name__=='__main__':unittest.main()
