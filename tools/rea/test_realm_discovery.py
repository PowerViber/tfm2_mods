import importlib.util,struct,tempfile,unittest
from pathlib import Path
from iced_x86 import Decoder,FlowControl
spec=importlib.util.spec_from_file_location('realm_discovery',Path(__file__).with_name('realm_discovery.py'))
import sys
module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module;spec.loader.exec_module(module)

def image():
    b=bytearray(0xa00);b[:2]=b'MZ';struct.pack_into('<I',b,60,0x80);b[0x80:0x84]=b'PE\0\0'
    struct.pack_into('<HHI',b,0x84,0x8664,3,1);struct.pack_into('<H',b,0x94,0xf0)
    o=0x98;struct.pack_into('<H',b,o,0x20b);struct.pack_into('<Q',b,o+24,0x140000000);struct.pack_into('<I',b,o+56,0x4000)
    for i,(name,rva,offset,flags)in enumerate([(b'.text',0x1000,0x400,0x60000020),(b'.rdata',0x2000,0x600,0x40000040),(b'.pdata',0x3000,0x800,0x40000040)]):
        q=o+0xf0+i*40;struct.pack_into('<8sIIII',b,q,name,0x200,rva,0x200,offset);struct.pack_into('<I',b,q+36,flags)
    struct.pack_into('<III',b,0x800,0x1000,0x1006,0x300c)
    return b

class Discovery(unittest.TestCase):
    def setUp(self):self.temp=tempfile.TemporaryDirectory();self.path=Path(self.temp.name)/'test.exe'
    def tearDown(self):self.temp.cleanup()
    def pe(self,b):self.path.write_bytes(b);return module.PE(self.path)
    def test_leaf_does_not_inherit_previous_unwind(self):
        pe=self.pe(image());self.assertIsNone(pe.extent(0x1020));self.assertEqual(pe.extent(0x1000)[:2],(0x1000,0x1006))
    def test_opcode_byte_inside_immediate_is_not_a_call(self):
        b=image();b[0x400:0x406]=bytes.fromhex('b8e8000000c3');d=module.decode(self.pe(b),0x1000)
        self.assertEqual(d['edges'],[]);self.assertEqual(len(d['instructions']),2)
    def test_indirect_dispatch_is_preserved_unresolved(self):
        b=image();b[0x420:0x428]=bytes.fromhex('41ff91f8000000c3');d=module.decode(self.pe(b),0x1020)
        self.assertEqual(d['edges'][0]['kind'],'indirect-call');self.assertEqual(d['edges'][0]['displacement'],0xf8)
        self.assertFalse(d['edges'][0]['resolved']);self.assertIsNone(d['unwindRange'])
    def test_short_loop_terminates(self):
        b=image();b[0x420:0x422]=b'\xeb\xfe';d=module.decode(self.pe(b),0x1020)
        self.assertEqual(len(d['instructions']),1);self.assertEqual(d['edges'][0]['targetRva'],'0x1020')
    def test_table_requires_all_pointers_to_be_executable(self):
        b=image();struct.pack_into('<QQQ',b,0x600,24,0x140001000,0x140001020)
        pe=self.pe(b);self.assertEqual(module.find_table(pe,['size','a','b'])['rva'],'0x2000')
        struct.pack_into('<Q',b,0x610,0x140002000)
        with self.assertRaises(ValueError):module.find_table(self.pe(b),['size','a','b'])
    def test_unknown_file_backing_fails(self):
        pe=self.pe(image())
        with self.assertRaises(ValueError):pe.offset(0x2500)
    def test_sha_is_exact_bytes(self):self.assertNotEqual(module.sha(b'a\r\n'),module.sha(b'a\n'))

if __name__=='__main__':unittest.main()
