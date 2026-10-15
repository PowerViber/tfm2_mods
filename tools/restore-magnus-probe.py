"""Restore a package-magnus probe backup, verifying every path before touching files."""
from pathlib import Path
import argparse,json,shutil,subprocess
p=argparse.ArgumentParser();p.add_argument('backup',type=Path);args=p.parse_args()
backup=args.backup.resolve();m=json.loads((backup/'manifest.json').read_text())
assert m.get('probe'), 'Only reversible probe installations are supported'
game=Path(m['game']).resolve();assert (game/'bundle.game_data').is_file()
running=subprocess.run(['tasklist','/FO','CSV','/NH'],capture_output=True,text=True).stdout.lower()
assert 'teamfightmanager2' not in running,'Close the game before restoring its loaded files'
entries=[]
for entry in m['files']:
    rel=Path(entry['path']);target=(game/rel).resolve();source=(backup/rel).resolve()
    assert target.is_relative_to(game/'mods') and source.is_relative_to(backup)
    assert not entry['existed'] or source.is_file()
    entries.append((entry,target,source))
for entry,target,source in entries:
    if entry['existed']:shutil.copy2(source,target)
    elif target.is_file():target.unlink()
print('Restored',len(entries),'previous game files from',backup)
