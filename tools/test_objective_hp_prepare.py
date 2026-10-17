from pathlib import Path
import binascii,gzip,importlib.util,json,struct,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('prepare',Path(__file__).with_name('prepare-objective-hp-career.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def packed():
    c=gzip.compress(b'career and history remain unchanged',mtime=0);h=bytearray(b'TFM2'+b'\0'*21);struct.pack_into('<Q',h,13,len(c));struct.pack_into('<I',h,21,binascii.crc32(c)&0xffffffff);return bytes(h)+c
class Prepare(unittest.TestCase):
    def fixture(self):
        parent=ROOT/'backups';parent.mkdir(exist_ok=True);d=tempfile.TemporaryDirectory(prefix='objective-prepare-',dir=parent);self.addCleanup(d.cleanup);root=Path(d.name).resolve();assert root.is_relative_to(parent.resolve());p=root/'career.data';p.write_bytes(packed());return root,p
    def test_inspection_never_writes_and_apply_verifies_complete_backup(self):
        root,p=self.fixture();old=p.read_bytes();report=m.prepare(p);self.assertEqual(report['mode'],'inspect');self.assertEqual(list(root.iterdir()),[p]);report=m.prepare(p,'apply');self.assertEqual(p.read_bytes(),old);self.assertEqual(Path(report['backup']).read_bytes(),old);self.assertFalse(report['saveRewritten'])
    def test_restore_stages_only_the_selected_native_folder_and_preserves_save(self):
        root,p=self.fixture();native=root/'tfm2_custom_ai';native.mkdir();(native/'mod.mod_info').write_text(json.dumps({'mod_type':'native'}));old=p.read_bytes();report=m.prepare(p,'restore',native);self.assertEqual(p.read_bytes(),old);self.assertTrue(Path(report['restoreFlag']).is_file());self.assertEqual(Path(report['backup']).read_bytes(),old)
    def test_bad_container_is_rejected_before_any_backup(self):
        root,p=self.fixture();p.write_bytes(b'corrupt');self.assertRaises(ValueError,m.prepare,p,'apply');self.assertEqual(list(root.iterdir()),[p])
if __name__=='__main__':unittest.main()
