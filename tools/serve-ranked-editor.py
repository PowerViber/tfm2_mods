"""Serve candidate Labs with empty, isolated AppData; never opens a game/window."""
from pathlib import Path
import os,sys
root=Path(__file__).resolve().parents[1]
fixture=root/'backups/ranked-body-editor'
fixture.mkdir(parents=True,exist_ok=True)
(fixture/'appdata').mkdir(exist_ok=True)
(fixture/'bundle.game_data').touch(exist_ok=True)
os.environ['APPDATA']=str(fixture/'appdata')
os.chdir(root)
os.execvp('node',['node','editor/server.js','--port',sys.argv[1] if len(sys.argv)>1 else '7298','--game',str(fixture),'--no-open'])
