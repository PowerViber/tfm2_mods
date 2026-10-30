"""Build experimental host-probe packages outside mods/. Never installs or enables anything."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / 'experiments/contender-host-probe'
TARGET = CRATE / 'target'
REPORT = ROOT / 'docs/contender-host-proof'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def sources():
    paths = [ROOT / 'rules/contender.json', CRATE / 'Cargo.toml', CRATE / 'Cargo.lock', CRATE / 'build.rs']
    paths.extend(sorted((CRATE / 'src').glob('*.rs')))
    paths.extend(sorted((ROOT / 'native/mod-api-stable/src').glob('*.rs')))
    return {p.relative_to(ROOT).as_posix(): sha(p) for p in paths}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repeat', action='store_true', help='Second independent build and hash comparison')
    args = parser.parse_args()
    REPORT.mkdir(parents=True, exist_ok=True)
    before = sources()
    epoch = subprocess.run(['rtk', 'proxy', 'git', 'show', '-s', '--format=%ct', '777204f37743e84cef0a1ea86680c1230079cdce'], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    env = dict(os.environ, SOURCE_DATE_EPOCH=epoch)
    results = []
    for mode in ['off-map-probe', 'dodge-probe', 'shield-probe']:
        hashes = []
        for run in range(2 if args.repeat else 1):
            out = TARGET / f'repro-{mode}-{run}'
            cmd = ['rtk', 'proxy', 'cargo', '+stable-x86_64-pc-windows-gnu', 'build', '--locked', '--release',
                   '--manifest-path', str(CRATE / 'Cargo.toml'), '--target-dir', str(out), '--features', mode]
            result = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
            (REPORT / f'build-{mode}-{run}.txt').write_text(result.stdout + result.stderr, encoding='utf-8')
            if result.returncode:
                raise SystemExit(result.stdout + result.stderr)
            dll = out / 'release/tfm2_contender_probe.dll'
            hashes.append(sha(dll))
            if run == 0:
                package = TARGET / 'packages' / mode / 'tfm2_contender_probe'
                package.mkdir(parents=True, exist_ok=True)
                shutil.copy2(dll, package / dll.name)
                info = {'name': 'EXPERIMENTAL Contender host probe — ' + mode, 'author': 'TFM2 Database Editor',
                        'version': '0.1.0', 'mod_type': 'native', 'contains_code': True,
                        'description': 'Not a champion. Mutates disposable test matches at tick 300. Release is BLOCKED; no arena isolation guarantee.'}
                (package / 'mod.mod_info').write_text(json.dumps(info, indent=2) + '\n', encoding='utf-8')
                (package / 'RELEASE_BLOCKED.txt').write_text('HOST PROBE ONLY. Do not use in a real career. No Contender champion is registered. See the repository docs/contender-host-proof/README.md.\n', encoding='utf-8')
        if len(set(hashes)) != 1:
            raise SystemExit(f'Non-reproducible {mode}: {hashes}')
        results.append({'mode': mode, 'dllSha256': hashes[0], 'independentBuilds': len(hashes), 'identical': True,
                        'scope': 'Experimental, never installed or enabled by this script'})
    if sources() != before:
        raise SystemExit('Sources changed during the build; discard these results and rebuild.')
    report = {'schema': 1, 'productionRelease': False, 'sourceDateEpoch': epoch, 'sources': before, 'rulesSha256': sha(ROOT / 'rules/contender.json'),
              'cargoLockSha256': sha(CRATE / 'Cargo.lock'), 'packages': results}
    (REPORT / 'builds.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))

if __name__ == '__main__':
    main()
