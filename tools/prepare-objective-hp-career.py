"""Back up a career before the supported on-load objective-settings migration.

Default: inspect only. --apply backs up the complete file without rewriting its
binary records; the native GameSetting API applies the journalled policy on load.
--restore also stages a restore flag in the explicitly supplied native mod folder.
The flag affects careers loaded while it exists; remove it after restoration.
"""
from pathlib import Path
import argparse, hashlib, importlib.util, json, shutil, time
spec=importlib.util.spec_from_file_location('container',Path(__file__).with_name('migrate-magnus-career.py'))
container=importlib.util.module_from_spec(spec);spec.loader.exec_module(container)

def prepare(source,mode='inspect',native_dir=None):
    source=Path(source);original=source.read_bytes();container.unpack(original)
    report={'mode':mode,'sourceSha256':hashlib.sha256(original).hexdigest(),
            'saveRewritten':False,'application':'Supported native GameSetting API when this career loads'}
    if mode!='inspect':
        backup=source.with_name(source.name+'.before-objective-hp-'+str(time.time_ns())+'.bak')
        with backup.open('xb') as f:f.write(original)
        if backup.read_bytes()!=original:raise ValueError('Backup verification failed')
        report['backup']=str(backup.resolve())
        if mode=='restore':
            if native_dir is None:raise ValueError('--restore requires --native-dir')
            native_dir=Path(native_dir).resolve()
            if not (native_dir/'mod.mod_info').is_file():raise ValueError('Not a native mod directory')
            meta=json.loads((native_dir/'mod.mod_info').read_text())
            if meta.get('mod_type')!='native':raise ValueError('Not a native mod')
            flag=native_dir/'objective_hp_restore.flag';flag.write_text('Restore journalled objective HP on career load.\n')
            report['restoreFlag']=str(flag)
            report['next']='Load the intended career, inspect objective-hp/v1 log result, then remove this flag.'
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path)
    g=p.add_mutually_exclusive_group();g.add_argument('--apply',action='store_true');g.add_argument('--restore',action='store_true')
    p.add_argument('--native-dir',type=Path);a=p.parse_args()
    print(json.dumps(prepare(a.source,'restore' if a.restore else 'apply' if a.apply else 'inspect',a.native_dir),indent=2))
