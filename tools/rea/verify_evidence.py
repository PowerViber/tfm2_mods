"""Verify the finite diagnostic evidence contract; never enables a realm backend."""
import argparse,hashlib,json
from pathlib import Path

def verify(folder):
    probe=json.loads((folder/'realm_probe.json').read_text())
    hooks=json.loads((folder/'realm-hooks.json').read_text())
    focused=json.loads((folder/'ghidra-focused.json').read_text())
    run=json.loads((folder/'ghidra-focused-run.json').read_text())
    builds=json.loads((folder/'observer-builds.json').read_text())
    cleanup=json.loads((folder/'disposable-cleanup.json').read_text())
    integrity=json.loads((folder/'profile-integrity.json').read_text())
    manager=json.loads((folder/'mod-manager.json').read_text())
    digest=probe['exe']['sha256']
    assert hooks['exeSha256']==focused['exeSha256']==run['exeSha256']==run['exeBeforeSha256']==digest,'Build identity mismatch'
    assert probe['simVtable']['size']==8*len(probe['sdk']['simFields']),'SDK size mismatch'
    assert len(probe['simVtable']['entries'])+1==len(probe['sdk']['simFields']),'SDK entry mismatch'
    assert probe['readOnly'] and not hooks['nativeBackendEnabled'] and not any(hooks['capabilities'].values()),'Unsupported native capability claim'
    live=probe.get('live')
    if live:
        assert live['sdkTableMatches'] and all(x['pointerMatches'] and x['codeBytesMatch'] and not x['executionObserved']for x in live['checks'])
        assert not any(live[k]for k in ['injection','processPaused','memoryWrites','scannedHeapOrCareerData'])
    for f in hooks['functions']:
        assert not f['hookValidated'] and not f['runtimeExecutionObserved']
        assert f['buildIdentityRef']=='realm_probe.json#/exe'
        assert f['callingConventionEvidence'] and f['signatureLimit']
    assert not run['exitCode'] and not run['executableChanged'] and run['projectDeletedByHeadless']
    assert focused['functions'] and all(x.get('decompileCompleted') and not x.get('error')for x in focused['functions']),'Incomplete focused decompilation'
    assert builds['independentBuilds']>=2 and builds['identical'] and not builds['productionRelease'] and not builds['installationPerformed']
    assert cleanup['exactDisposablePathVerified'] and cleanup['ownedTestProcessExited'] and not cleanup['installedGameTerminated']
    assert integrity['allExistingFilesUnchanged'] and not integrity['newFileCount']
    assert manager['exitCode']==0 and manager['packageUnchanged'] and manager['productionUnchanged']
    assert manager['dllSha256']==builds['dllSha256']
    return {'version':1,'scope':'Finite diagnostic evidence only; no realm behaviour acceptance',
        'exeSha256':digest,'sdkFunctionCount':len(probe['simVtable']['entries']),
        'decodedCandidates':len(hooks['functions']),'focusedDecompilations':len(focused['functions']),
        'observedCombatExecution':False,'nativeBackendEnabled':False,
        'disposableProcessClosed':True,'realProfileUnchanged':True,
        'modManagerScope':'Unchanged production baseline and diagnostic file identity only','checksPassed':True}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--reports',type=Path,default=Path(__file__).resolve().parents[2]/'docs/contender-realms/windows');a=p.parse_args()
    result=verify(a.reports);(a.reports/'verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
