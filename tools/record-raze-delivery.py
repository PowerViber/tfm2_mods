"""Write the current Raze implementation/PR notes from recorded verification."""
from pathlib import Path
import json,re
root=Path(__file__).resolve().parents[1];results=root/'docs/raze-results'
read=lambda n:json.loads((results/n).read_text(encoding='utf8'))
package=read('package.json');perf=read('performance.json');lab=read('lab.json');art=read('art.json');manager=read('mod-manager.json');keyboard=read('keyboard.json');magnus=read('magnus-performance.json')
native=re.search(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored',(results/'native-tests.txt').read_text()).groups()
tests=re.search(r'test result: ok\. (\d+) passed; 0 failed',(results/'lab-tests.txt').read_text()).group(1)
totals={k:sum(s['core'][k]for s in perf['scenarios'])for k in ['rocket_shots','rocket_hits','rocket_entries','triple_boosts','rocket_objective_hits','protected_ticks']}
sha=package['packagedDllSha256'];mp=[r['stress'][-1]['p95StepUs']/1000 for r in magnus]
table='\n'.join('| '+level+' | '+' | '.join(str(d[k])for k in ['requestedRaw','labEstimatedDelivered','exposureTicks','objectiveEstimatedDamage','launches'])+' |'for level,d in perf['aggregates'].items())
text=f'''# Raze — rocket entries, airborne protection and thirteen athlete attributes

Native **{package['native']}**, champion pack **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**, Raze Lab **v2**. Continue PR #46 on `codex/raze-air-strafe`; main `{package['baseCommit']}` and its Levi jungle fix are incorporated. The original checkout's local Magnus changes are preserved. This release does not install mods or migrate a real career.

## Mechanics

Raze is an AP fighter, using the supported `Range` category and AP/Range/CC tags. Internal ID `tfm2_valorant_raze`. No mastery or ranked artwork. Guns use Omen's credit income, prices, modifiers and engine-owned primary attack identity. Paid packs, grenades and a real destructible Boom Bot do not regenerate automatically.

| Showstopper | Prototype value |
|---|---:|
| Base cooldown, starting at accepted equip | 60 seconds |
| Equip / armed window | 0.7 / 6 seconds |
| Maximum range / speed | 240,000 / 600,000 units per second |
| Maximum raw magic damage | 240 + 150% AP |
| Radius | 40,000 |
| Falloff | 100% centre, 62.5% halfway, 25% boundary |

The rocket commits to a ground endpoint, clamped to range and map bounds, or explodes sooner on its first swept unit/wall contact. It does not home. Falloff uses integer `floor(raw × (4R − 3d) / 4R)`. There is one damage packet per victim, with no extra direct-hit packet. Unit contact can occur at the edge of the prototype 8,000-unit collision radius, before reaching the unit centre. Self-damage remains 25% after distance falloff, before normal mitigation. Explosives exclude towers/nexuses; allies are unaffected.

Firing causes a ten-tick grounded slide at 60,000 units/second, or adds 90,000 units/second opposite the shot during flight. Air recoil permits a 300,000 speed cap and adds eighteen remaining ticks, bounded to eighty-four from the first launch. The original satchel impulse (160,000), ordinary cap (240,000), bounded strafing and terrain rules remain. Height stays continuous. Roots prevent propulsion; stun ends controlled travel. Backward rocket boosts spend real inventory and the ultimate. The third-boost metric requires backward recoil, not merely a rocket after two packs.

**Airborne protection: 15% incoming physical/magic damage reduction during actual flight.** It uses one named engine modifier, including ordinary self-blast mitigation; no post-hit healing or control immunity. Ordered transition commands activate it on takeoff and remove it on landing, flight-ending collision/control, death or cleanup. Ground recoil and cosmetic height never grant protection. Lethal self-blasts stop subsequent non-cleanup host commands in that batch.

Paint Shells remains 300 credits, one stored grenade, eight-second cooldown, 0.35-second preparation and 80,000 range: 65 + 55% AP main damage and four 15 + 12.5% AP clusters, at most two secondary hits per target. Packs remain 200 each, two stored, 40 HP, five-second lifetime, arming after one second for 20 + 15% AP and displacement. Boom Bot remains 300 credits, one stored, eighteen-second cooldown, five-second lifetime, 25% owner max HP and 45 + 45% AP explosion. Failed carrier allocation restores only the failed summon commitment.

## Judge and decisions

The shared Judge curve for Raze and Omen is 300% basic damage through 10,000 units, linearly declining to 70% at 20,000. Price 1,850, 18,000 base reach, −25% attack speed and 35% secondary splash remain. Native attacks, Omen's burst estimate, Raze planning, farming attack hooks and the Lab use the shared rules. Omen's purchasing preferences and all existing body/effect pixels remain unchanged.

Raze buys/keeps Judge for affordable reachable close entries, reserving a missing launch pack before armour; open or blocked approaches retain rifle choices. Full-price replacements give no refunds. Automatic replacements require twenty percent estimated improvement and fifteen seconds since the previous weapon purchase. Manual purchases ignore athlete biases and that AI-only hysteresis.

All thirteen pinned v2 attributes remain independent. Ego prices risk; Judgement estimates value; Positioning chooses approach/landing; Control Speed schedules 24–6 tick first responses; Accuracy gives 30–100% observed-motion lead and 6,000-to-zero placement error; Dodge reacts to visible hazards; Aggressive favours offensive entries; Order prices support; Roaming favours rotations; Macro prices waves/resources; Micro controls 12–3 tick steering and one sampled detonation timing error; Awareness reviews last-sighting uncertainty; Strategy chooses a one-to-three-action continuation.

Every athlete can consider shooting, grenades, bot, paid single/double launches, rocket entries, backward rocket boosts and escape. Search retains 64 evaluated candidates, beam width eight and three macro actions. Aim variants compare visible groups, ground centres and wall impacts using the same first-contact geometry as real flight. Objective rockets require a visible assigned Morgard/Serpen secure or useful champion overlap; ordinary wave/camp reserves remain. Uncommitted stages revalidate; committed effects stay real. Hidden current positions/resources do not enter planning. Hashing is separate from combat RNG.

## Lab, artwork and compatibility

WASD moves on the ground and supplies bounded steering in the air; arrows remain alternatives. Q places a travel-led launch pack behind actual movement, then detonates it. A quick second Q queues one detonation through preparation/carrier readiness. E throws a grenade, X equips, left click/F fires, C deploys the bot. Mouse movement aims independently; diagonal ground movement is normalized. Continuous steering is separate from ability commands. Keyboard gameplay starts live manual playback only with arena focus; focus loss clears held inputs.

The Lab shows cooldown, preparation, firing-window expiry, falloff, ground/air recoil, velocity, remaining flight, 15% protection and estimated prevented damage. Its ordinary attacks, incoming damage and mitigation are a simplified twenty-resistance host model, not actual engine damage, shields/items or verified native stacking order. Recordings carry v2 and `raze-2`; v1/different-rule recordings remain inspectable, with replay explicitly rejected rather than silently reinterpreted.

Approved body pixels, 36×44 canvas, foot anchor, original animation names/counts/timing and both facings are preserved. Eight source poses are reused across those sequences. Existing frames remain pixel-identical; the new blast/warning variants are nearest-neighbour resizes authorized by the plan. Full Valorant decoded storage: **{art['decodedValorantBytes']:,} bytes**; largest Raze atlas **1024×488**, below both limits. The public view API has no individual point-effect cancellation handle; native warning visuals have bounded tails, while owned danger markers clean up separately. Live presentation acceptance remains pending.

[Body comparison](raze-review/raze-comparison.png) · [Effects](raze-review/effects-comparison.png) · [Air protection screenshot](raze-results/raze-air-protection.png) · [Research and timestamps](raze-entry-research.md)

## Recorded verification

- Native release: **{native[0]} passed, zero failed, {native[1]} pre-existing ignored**; [raw output](raze-results/native-tests.txt).
- Shared Raze Lab: **{tests} passed**, including endpoint/range/collision/falloff, offensive entry execution, paid backward boosts, continuous height, ground recoil, protection transition ordering, manual independence and purchase safeguards.
- Three command-trace fixture pairs per attribute at 20 versus 80 with the other twelve fixed at 50; [seeds](raze-results/athlete-pairs.json).
- **{len(lab['scenarios'])} scenarios: 24 development and 12 holdout; {lab['parityFrames']:,} complete native/WASM state-and-command comparisons**, identical replay, independent sessions and 390px light/dark layouts; [Lab results](raze-results/lab.json).
- Real keyboard/mouse tests cover WASD, normalized diagonals, queued Q–Q, air steering, grenade/equip/fire, ground recoil without protection, focus loss and legacy inspection; [results](raze-results/keyboard.json).
- Canonical generation, original approved frame pixels/timings and atlas limits verified. Farming metadata covers 78 champions, 234 slots and 11,700 role/team/target combinations; this is not proof of live casts.
- Five matched Raze release-core timing runs: median p95 **{perf['medianP95Ms']:.4f} ms**, p99 **{perf['medianP99Ms']:.4f} ms**, 194,400 steps per run. Magnus 128-piece p95: **{', '.join(str(v) for v in mp)} ms**, all below four milliseconds. Host/rendering timing is excluded.

The first matched case set recorded {totals['rocket_shots']} rocket shots, {totals['rocket_hits']} enemy hit requests, {totals['rocket_entries']} airborne rocket entries, {totals['triple_boosts']} backward third boosts, {totals['rocket_objective_hits']} objective hit requests and {totals['protected_ticks']} protected ticks. These are harness events, not engine-attributed damage or proof of live match quality.

| Uniform profile | Requested raw | Lab estimated delivered | Exposure ticks | Objective estimate | Launches |
|---|---:|---:|---:|---:|---:|
{table}

The short harness still has limited balance coverage: inspect its deaths, wave kills and missed-wave counts separately. Higher Ego/all-high profiles do not guarantee greater damage or wins. Earlier v1 benchmark output did not model Judge falloff or air protection; it is not a clean balance baseline for this corrected harness.

## Delivery and outstanding observations

No actual game match was observed for this release. Pending: carrier targeting/coverage, in-flight attacks, native mitigation/other reductions/shields/items, interruption and death ordering in the host, attribution, actual objective fights, warning tails and crowded rendering/performance. Public projectile snapshots lack exact hitboxes and per-projectile visibility; dodge guidance retains its conservative visible-caster/clear-line approximation. Prototype core collision is not proof of exact engine geometry.

Reproduce with `rtk proxy python -X utf8 tools/check-raze-release.py` while serving the checkout at `http://localhost:7286`; then run `rtk proxy python -X utf8 tools/package-raze.py` and `rtk proxy python -X utf8 tools/record-raze-delivery.py`. The pipeline builds native/WASM, tests, records five-run timings, reproduces the DLL in two directories and checks an isolated package fixture. No Steam installation or real-career migration occurs.

Manager verified its custom-pack/native scope; supplementary hashes checked **{manager['allPackFilesCompared']} total files, including {manager['valorantFilesCompared']} Valorant files**, with zero differences. Toolchain/source/WASM fingerprints are in [package.json](raze-results/package.json). Both independently built DLLs and the package have SHA-256:

`{sha}`
'''
(root/'docs/raze-implementation.md').write_text(text,encoding='utf8')
pr=f'''Adds Raze as an AP fighter with purchased Omen weapons, paid utility, momentum strafing and all thirteen athlete attributes. This update strengthens Showstopper and entry decisions and adds **15% actual-airborne damage reduction**.

- Rocket: **60s cooldown, 240,000 range, 600,000 units/s, 240 + 150% AP, 40,000 radius**, continuous centre-to-edge falloff; fixed ground endpoint or first swept collision.
- Ground recoil is separate from flight. Two packs plus a real backward rocket can create a third boost; protection ends on landing/control/death and does not grant immunity.
- Shared Raze/Omen Judge: **3× damage through 10,000**, falling to 0.7× at 20,000; existing price/range/speed/splash. Raze chooses Judge for viable entries and rifles for open approaches, with paid-swap safeguards.
- **Lab v2: WASD, Q–Q, E, X, mouse/F fire**, simultaneous steering/casts, visible cooldown/protection and versioned recordings. Legacy recordings can be inspected but are not silently replayed with new rules.
- Approved body/effect frames retained; only authorized mechanical explosion/warning resizes. Main's Levi jungle fix is merged.

Native **{package['native']}**, champion pack **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**.

### Verification
- Native **{native[0]} passed / {native[1]} existing ignored**; Lab **{tests} passed**.
- **{lab['parityFrames']:,} native/WASM parity frames**, 24 development + 12 holdout scenarios, independent replay/sessions, real keyboard/mouse tests and three low/high comparisons per athlete attribute.
- Five core runs: p95 **{perf['medianP95Ms']:.4f} ms**, p99 **{perf['medianP99Ms']:.4f} ms**; Magnus 128-piece p95 **{min(mp):.3f}–{max(mp):.3f} ms**.
- Reproducible art; {art['decodedValorantBytes']:,} decoded bytes. Two DLL builds match. Isolated manager verification plus all-pack SHA-256 checks: **{manager['allPackFilesCompared']} files, zero differences**.

DLL SHA-256: `{sha}`

![Airborne protection and keyboard Lab](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/raze-air-strafe/docs/raze-results/raze-air-protection.png)

[Implementation/results/limits](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-implementation.md) · [Timestamped research](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-entry-research.md) · [Approved body](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-review/raze-comparison.png) · [Effects](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-review/effects-comparison.png)

**Draft — live-game checks remain pending.** Native targeting, protection/mitigation stacking, shields/items, attribution, warning tails and crowded host/rendering need observation. Lab damage is a simplified estimate; short harness counts are not a win-rate or balance proof. Projectile visibility/collision use documented conservative approximations. **Nothing installed; no real career migrated; original local work preserved.**
'''
(root/'backups/raze-pr-body.md').write_text(pr,encoding='utf8')
print(json.dumps({'nativeTests':native[0],'labTests':tests,'dll':sha,'totals':totals}))
