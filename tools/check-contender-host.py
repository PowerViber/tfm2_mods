"""Inspect the checked SDK and keep the Contender release gate closed without live-host proof.

This checks repository/package integrity. It does not infer runtime support from unit tests.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/contender-host-proof'
BASE = '777204f37743e84cef0a1ea86680c1230079cdce'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def git_bytes(path):
    return subprocess.run(['rtk', 'proxy', 'git', 'show', BASE + ':' + path], cwd=ROOT,
                          check=True, capture_output=True).stdout

def inspect():
    rules = json.loads((ROOT / 'rules/contender.json').read_text(encoding='utf-8'))
    files = ['native/mod-api-stable/src/traits.rs', 'native/mod-api-stable/src/host.rs',
             'native/mod-api-stable/src/sim.rs', 'native/mod-api-stable/src/values.rs',
             'native/mod-api-stable/src/client_ctx.rs']
    sdk = {}
    for name in files:
        data = (ROOT / name).read_bytes()
        sdk[name] = {'sha256': digest(data), 'bytes': len(data)}
    host = (ROOT / files[1]).read_text(encoding='utf-8')
    traits = (ROOT / files[0]).read_text(encoding='utf-8')
    inventory = sorted(set(re.findall(r'pub ([a-z][a-z_0-9]+)\s*:\s*Option<', host)))
    hooks = sorted(set(re.findall(r'\bfn ([a-z][a-z_0-9]+)\s*\(', traits)))
    production = ['mods/tfm2_custom_ai/tfm2_custom_ai.dll', 'mods/tfm2_custom_ai/mod.mod_info',
                  'native/tfm2_custom_ai/mod.mod_info', 'mods/tfm2_custom/mod.mod_info',
                  'native/tfm2_custom_ai/src/lib.rs']
    unchanged = {}
    for name in production:
        before, after = git_bytes(name), (ROOT / name).read_bytes()
        binary = name.endswith('.dll')
        equal = before == after if binary else before.replace(b'\r\n', b'\n') == after.replace(b'\r\n', b'\n')
        unchanged[name] = {'sha256': digest(after), 'comparison': 'exact bytes' if binary else 'text with checkout line endings normalized', 'matchesImplementationBaseline': equal}
    loaded = []
    for path in (ROOT / 'mods').rglob('*.data_champion'):
        value = json.loads(path.read_text(encoding='utf-8-sig'))
        if value.get('id') == rules['id']:
            loaded.append(path.relative_to(ROOT).as_posix())
    source_checks = {
        'productionFilesUnchanged': all(v['matchesImplementationBaseline'] for v in unchanged.values()),
        'championNotRegisteredInProduction': not loaded,
        'canonicalReleaseGateClosed': rules['releaseStatus'] == 'blocked-host-proof',
        'approvedModPower': rules['modPower'] == {'attack': 30, 'magicPower': 0, 'hp': 5, 'defence': 10, 'magicResistance': 10, 'moveSpeed': 0, 'attackSpeed': 10},
        'approvedCooldownRecovery': (rules['ultimate']['ordinaryCooldownRecoveryNumerator'], rules['ultimate']['ordinaryCooldownRecoveryDenominator']) == (4, 3),
        'exactReservedReturn': rules['ultimate']['return'] == 'exact-entry-position-before-transfer' and rules['ultimate']['reserveReturnFootprints'],
    }
    gates = [
        ('offMapWorld', 'Stock/custom movement, targeting, ordinary attacks, casts and summons outside map bounds'),
        ('interactionIsolation', 'Block new cross-arena damage, support, control and global effects'),
        ('boundaryEnforcement', 'All movement, forced movement and stock/custom teleports respect ring geometry'),
        ('exactReturn', 'Live death/timeout/partial failure restore exact recorded coordinates'),
        ('constructionReservations', 'All relevant solid construction routes reject reserved footprints'),
        ('cameraReplay', 'Existing champion-follow or optional Ring View, plus replay seek'),
        ('fullDodge', 'Targeting, physical/magic/fixed damage, DoT and new control at exact window boundaries'),
        ('shieldHitIdentity', 'Reliable attacker/impact identity including fully shield-absorbed hits'),
    ]
    return {
        'schema': 1, 'implementationBaseline': BASE, 'rulesRevision': rules['revision'], 'releaseReady': False,
        'scope': 'Repository and checked SDK inspection; no live game assertions',
        'sourceChecks': source_checks, 'production': unchanged, 'loadedContenderDefinitions': loaded,
        'sdk': sdk, 'sdkHostSlots': inventory, 'sdkTraitMethods': hooks,
        'sdkFindings': [
            {'observation': 'The checked surface provides entity_set_pos and a render camera setter.', 'limitation': 'No promise of off-map navigation, rendering or camera-follow behaviour.'},
            {'observation': 'The checked StablePassive::on_damaged receives an immutable damage value after damage.', 'limitation': 'It is not a universal pre-impact cancellation or shield-hit identity hook.'},
            {'observation': 'The checked banish operation is documented as untargetable, invisible and input-locked.', 'limitation': 'Full dodge and normal playable isolated duels cannot be assumed from that operation.'},
            {'observation': 'The map customizer documents original offline pathfinding/visibility geometry.', 'limitation': 'Extending a wall document alone does not establish an off-map navigable arena.'},
        ],
        'mandatoryHostProof': [{'id': key, 'requirement': text, 'status': 'unproven'} for key, text in gates],
        'notImplementedBeforeGate': ['production champion registration', 'boxing combat controller', 'athlete planner', 'Boxing Lab', 'final body/effect artwork', 'career migration'],
    }

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--require-release', action='store_true')
    args = parser.parse_args()
    report = inspect()
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'gate.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'releaseReady': report['releaseReady'], 'sourceChecks': report['sourceChecks'],
                      'mandatoryHostProof': report['mandatoryHostProof']}, indent=2))
    if not all(report['sourceChecks'].values()):
        raise SystemExit(1)
    if args.require_release:
        raise SystemExit(3)

if __name__ == '__main__':
    main()
