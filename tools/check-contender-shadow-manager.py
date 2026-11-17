"""Reuse the fully isolated manager fixture; source/game configs remain untouched."""
from pathlib import Path
import os,subprocess
root=Path(__file__).resolve().parents[1]
result=subprocess.run(['rtk','proxy','python','tools/check-basalt-art-manager.py'],cwd=root,
    env=dict(os.environ,BASALT_RESULTS='docs/contender-shadow-review'),check=True)
