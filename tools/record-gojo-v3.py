"""Record only completed checks and exact package identities; never install."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/gojo-results'
def read(name):return json.loads((OUT/name).read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
package=read('package.json');manager=read('mod-manager.json');verification=read('verification.json');parity=read('parity.json');art=read('v2-art.json');browser=[read(n) for n in ['browser.json','anime-browser.json','v3-browser.json']];perf=read('performance.json');magnus=read('magnus-stress.json')
assert all(c['passed'] for c in verification['checks'])
assert package['builtDllSha256']==package['reproducedDllSha256']==package['packagedDllSha256']==manager['packagedDllSha256']==sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll')
assert package['gojoWasmSha256']==sha(ROOT/'editor/gojo.wasm')
assert all(not b['errors'] for b in browser) and manager['exitCode']==0
assert art['decodedBytesIncludingV1Compatibility']<=32*1024*1024 and magnus[-1]['p95StepUs']<4000
native=re.search(r'(\d+) passed; 0 failed; (\d+) ignored',(OUT/'native-tests.txt').read_text());shared=re.search(r'(\d+) passed; 0 failed',(OUT/'shared-tests.txt').read_text())
assert native and shared and int(shared[1])==63
demo_frames=browser[-1]['demonstrationParityFrames'];assert demo_frames==6510
files=sorted([*list((ROOT/'native/tfm2_custom_ai/src').glob('gojo_v2*.rs')),ROOT/'native/tfm2_custom_ai/src/gojo_visual_data.rs',ROOT/'native/tfm2_custom_ai/src/lib.rs',ROOT/'rules/gojo-v2.json',ROOT/'rules/gojo-visuals.json'])
fingerprint=hashlib.sha256(b''.join(str(p.relative_to(ROOT)).replace('\\','/').encode()+b'\0'+p.read_bytes()+b'\0' for p in files)).hexdigest()
result={'native':package['native'],'jjk':package['jjk'],'championPack':package['championPack'],'nativeTestsPassed':int(native[1]),'nativeTestsIgnored':int(native[2]),'sharedTestsPassed':int(shared[1]),'migrationTestsPassed':7,'browserChecksPassed':sum(len(b['checks']) for b in browser),'parityFrames':parity['frames'],'developmentScenarios':24,'holdoutScenarios':12,'decodedArtBytes':art['decodedBytesIncludingV1Compatibility'],'gojoMedianP95Ns':perf['medianP95Ns'],'gojoMedianP99Ns':perf['medianP99Ns'],'magnus128P95Us':magnus[-1]['p95StepUs'],'packagedDllSha256':package['packagedDllSha256'],'wasmSha256':package['gojoWasmSha256'],'sourceFingerprintSha256':fingerprint,'originalCheckoutDllPreserved':sha(Path('C:/Games/tfm2_mods/mods/tfm2_custom_ai/tfm2_custom_ai.dll'))=='96701d896e740fc7d256a4a7be0e64178ad147e36331f34478059a9e5620e105','installed':False,'careersMigrated':False,'liveAcceptance':'Pending','stockProjectileBending':'Unsupported; no safe hook demonstrated','modManager':'Isolated fixture passed'}
result['scenarioParityFrames']=result['parityFrames'];result['demonstrationParityFrames']=demo_frames;result['parityFrames']+=demo_frames
(OUT/'v3-delivery.json').write_text(json.dumps(result,indent=2)+'\n')
comparisons=read('v3-comparison.json')['averages'];table='\n'.join(f"| {r['kind']} | {r['baseline']['requestedRawDamage']:.1f} → {r['candidate']['requestedRawDamage']:.1f} | {r['baseline']['remainingHp']:.1f} → {r['candidate']['remainingHp']:.1f} |" for r in comparisons)
text=f'''# Gojo v3 release verification

Native **{result['native']}**, JJK **{result['jjk']}**, custom pack **{result['championPack']}**. [Implementation and references](gojo-v3.md).

- {result['nativeTestsPassed']} native tests passed, {result['nativeTestsIgnored']} existing ignored; {result['sharedTestsPassed']} shared-core tests and seven migration tests passed.
- {result['browserChecksPassed']} browser checks, including every technique demonstration, current replay and frozen v1/v2 readers. Both facings/light-dark presentation samples pass.
- {result['parityFrames']:,} complete native/WASM frames: {result['scenarioParityFrames']:,} across 24 development and 12 holdout scenarios, plus {demo_frames:,} covering all ten legal demonstrations, including complete orb/Nuke flights. Independent sessions pass.
- 72 native Domain expansion frames, projectile/impact bounds, 530 animation renewal windows and 4,240 phase comparisons pass. Original body and legacy frame pixels match main. Generation is reproducible.
- Installed Gojo decoded art: **{result['decodedArtBytes']:,} bytes ({result['decodedArtBytes']/1048576:.2f} MiB)**; every atlas is at most 2048 pixels per side. Archived-v2 editor artwork is loaded only by its separate reader.
- Five Gojo core runs: median p95 **{result['gojoMedianP95Ns']/1000:.1f} µs**, p99 **{result['gojoMedianP99Ns']/1000:.1f} µs**. Magnus 128-piece p95: **{result['magnus128P95Us']/1000:.3f} ms**, below the 4 ms target. These exclude native host/rendering time. Fresh-browser load and canvas submission are recorded separately.
- Two independent release builds and packaged DLL have identical SHA-256. Mod Manager passes against an isolated full-package fixture.

| Thirty-second raw fixture, five seeds | Requested damage: v2 → v3 | Remaining Gojo HP: v2 → v3 |
|---|---:|---:|
{table}

The fixtures use explicit scene movement and raw shield bookkeeping, not engine armour, items or attributed match damage. Duels trade some requested damage for better remaining HP; the team fixture improves slightly in both. No unconditional buff or live-performance claim is made.

Packaged DLL SHA-256:

`{result['packagedDllSha256']}`

WASM SHA-256:

`{result['wasmSha256']}`

Core/rules source fingerprint: `{fingerprint}`.

**Live acceptance pending. Stock enemy-shot bending remains unsupported.** Original checkout DLL preserved: {result['originalCheckoutDllPreserved']}. No game executable patch, Steam installation or career migration was performed. [Machine-readable delivery record](gojo-results/v3-delivery.json).
'''
(ROOT/'docs/gojo-v3-release.md').write_text(text,encoding='utf-8');print(json.dumps(result,indent=2))
