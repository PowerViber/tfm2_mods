"""Reproduce the open Round 165 DLL in two targets; package without installing."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess, json, os, hashlib, shutil

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/round-165-art-review'
BASE = 'a0171ba35822a6726533076c790f564b9036d97c'

def run(args, env=None):
    p = subprocess.run(['rtk', 'proxy', *args], cwd=ROOT, capture_output=True,
                       text=True, encoding='utf8', errors='replace', env=env)
    if p.returncode:
        raise RuntimeError(p.stdout + p.stderr)
    return p.stdout.strip()

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def source_hash():
    sources = sorted((ROOT/'native/tfm2_custom_ai/src').rglob('*.rs'))
    sources += [ROOT/'native/tfm2_custom_ai/Cargo.toml', ROOT/'native/tfm2_custom_ai/Cargo.lock']
    return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode() +
                                  p.read_bytes().replace(b'\r\n', b'\n') for p in sources)).hexdigest()

before = source_hash()
epoch = run(['git', 'show', '-s', '--format=%ct', BASE])
env = dict(os.environ, SOURCE_DATE_EPOCH=epoch)
args = ['cargo', '+stable-x86_64-pc-windows-gnu', 'build', '--locked', '--release',
        '--manifest-path', 'native/tfm2_custom_ai/Cargo.toml']
secondary = ROOT/'backups/round-165-art-repro-target'
with ThreadPoolExecutor(max_workers=2) as pool:
    jobs = [pool.submit(run, args, env), pool.submit(run, args, dict(env, CARGO_TARGET_DIR=str(secondary)))]
    for job in jobs:
        job.result()
assert before == source_hash(), 'Native source changed during packaging'
built = ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll'
reproduced = secondary/'release/tfm2_custom_ai.dll'
assert sha(built) == sha(reproduced), 'Two target builds differ'
package = ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll'
shutil.copyfile(built, package)
meta = json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text())
assert meta == json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())
report = dict(native=meta['version'], baseCommit=BASE, sourceDateEpoch=epoch,
              nativeSourcesSha256=before,
              toolchain=run(['rustc', '+stable-x86_64-pc-windows-gnu', '--version']),
              builtDllSha256=sha(built), reproducedDllSha256=sha(reproduced),
              packagedDllSha256=sha(package),
              labs={n:sha(ROOT/f'editor/{n}.wasm') for n in ['akane','wright','thalassa']},
              packs={p.parent.name:json.loads(p.read_text())['version'] for p in sorted((ROOT/'mods').glob('*/mod.mod_info'))},
              installedByThisScript=False, careerMigrationApplied=False,
              liveAcceptance='Pending actual-game visual, carrier guard receipt, targeting and item observations.')
(OUT/'package.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
