"""Freeze canonical Git bytes for the declared art integration baseline, not local edits."""
from pathlib import Path
import importlib.util,subprocess,tarfile,io,json,hashlib
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('art',ROOT/'tools/integrate-character-redesign.py');art=importlib.util.module_from_spec(spec);spec.loader.exec_module(art)
data=subprocess.check_output(['rtk','proxy','git','archive','--format=tar',art.BASE,'mods'],cwd=ROOT)
with tarfile.open(fileobj=io.BytesIO(data))as archive:
 files={m.name:hashlib.sha256(archive.extractfile(m).read()).hexdigest()for m in archive.getmembers()if m.isfile()}
path=art.OUT/'integration-baseline.json';report=json.loads(path.read_text());report.update(base=art.BASE,byteConvention='Canonical Git bytes; working-tree CRLF accepted only when normalizing exactly reproduces these bytes',files=files);path.write_text(json.dumps(report,indent=2)+'\n')
print('Frozen baseline',art.BASE,len(files),'mod files')
