from pathlib import Path
import binascii,gzip,importlib.util,json,struct,unittest
spec=importlib.util.spec_from_file_location('reworks',Path(__file__).with_name('migrate-champion-reworks.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def packed(entries):
 payload=b'career-history-STAYS\x00'+b''.join(b'1HCMABCD'+struct.pack('<Q',len(e))+e for e in entries)+b'athlete-records-STAY';c=gzip.compress(payload,mtime=0);h=bytearray(b'TFM2'+b'\0'*21);struct.pack_into('<Q',h,13,len(c));struct.pack_into('<I',h,21,binascii.crc32(c)&0xffffffff);return bytes(h)+c
class Migration(unittest.TestCase):
 def test_fields_and_idempotence(self):
  defs=m.definitions();entries=[]
  for id,e in defs.items():
   e=json.loads(json.dumps(e));e['stat']['hp']=2345;e['growth']['hp']=173;e['passive_ult']=None
   if id=='tfm2_frieren_frieren':e.pop('passive');e['attack']['cooltime']=140;e['skill']['effect']={'type':'Combine','effects':[]}
   entries.append(json.dumps(e).encode())
  unrelated=b'{"id":"other","stat":{"hp":765},"history":[1,2,3]}';source=packed(entries+[unrelated]);out,report=m.migrate(source);self.assertEqual(sum(report['changed'].values()),3);self.assertFalse(report['conflicts']);_,payload=m.container.unpack(out);self.assertIn(unrelated,payload);self.assertTrue(payload.startswith(b'career-history-STAYS\x00'));self.assertTrue(payload.endswith(b'athlete-records-STAY'));self.assertEqual(payload.count(b'"hp":2345'),3);self.assertEqual(payload.count(b'"hp":173'),3);again,report=m.migrate(out);self.assertEqual(again,out);self.assertEqual(sum(report['changed'].values()),0)
 def test_conflicting_controller_and_bad_checksum_never_write(self):
  e=m.definitions()['tfm2_custom_magnus'];e['passive']={'passive_ref':'other:controller'};source=packed([json.dumps(e).encode()]);out,report=m.migrate(source);self.assertEqual(out,source);self.assertTrue(report['conflicts']);bad=bytearray(source);bad[-3]^=1
  with self.assertRaises(ValueError):m.migrate(bytes(bad))
if __name__=='__main__':unittest.main()
