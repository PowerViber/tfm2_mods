"""Generate Raze 3.6 implementation and PR notes from verified release evidence (round 144: see docs/main-review-round-144.md)."""
from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parents[1];out=root/'docs/raze-results'
read=lambda n:json.loads((out/n).read_text(encoding='utf8'))
package,manager,art,perf,lab,compare,magnus=[read(n)for n in ['package.json','mod-manager.json','art.json','performance.json','lab.json','comparison.json','magnus-performance.json']]
assert package['native']=='0.10.82' and art['approved'] and manager['exitCode']==0
sha=package['packagedDllSha256'];assert sha==package['builtDllSha256']==package['reproducedDllSha256']==manager['packagedDllSha256']==hashlib.sha256((root/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest()
build={k:package[k]for k in ['native','baseCommit','nativeSourcesSha256','sourceDateEpoch','toolchain','builtDllSha256','reproducedDllSha256']}
build.update(releasePackageUpdated=True,reason='Original approved artwork is preserved; the satchel blast is mechanically enlarged. Independent builds match the packaged Raze 3.6 DLL; isolated Mod Manager verification passed.')
(out/'dll-build.json').write_text(json.dumps(build,indent=2)+'\n',encoding='utf8')
native=re.search(r'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored',(out/'native-tests.txt').read_text()).groups()
shared=re.search(r'test result: ok\. (\d+) passed; 0 failed',(out/'lab-tests.txt').read_text()).group(1)
a,b=compare['baseline'],compare['candidate'];mp=[r['stress'][-1]['p95StepUs']/1000 for r in magnus]
metrics=[('Requested raw damage, including modelled gunfire','requestedRaw'),('Lab-estimated delivered damage','labEstimatedDelivered'),('Exposure ticks','exposureTicks'),('Objective damage estimate','objectiveEstimatedDamage'),('Launches','launches'),('Entries receiving allied follow-up','entriesWithFollowup'),('Allied follow-up damage','followupDamage'),('Airborne protection ticks','airborneTicks'),('Landing protection ticks','landingGuardTicks'),('Damage prevented estimate','airDamagePreventedEstimate'),('Deaths','deaths'),('Wasted preparation','wastedPreparation')]
table='\n'.join(f'| {label} | {a[key]:,} | {b[key]:,} |'for label,key in metrics)
status=dict(rules='raze-3.6',nativeTarget=package['native'],state='packaged_live_acceptance_pending',buckyIconApproved=True,releasePackageUpdated=True,testDllSha256=sha,trackedDllSha256=sha,modManager='Passed in isolated fixture',installationApplied=False,realCareerMigrationApplied=False,liveAcceptance='Pending')
(out/'release-status.json').write_text(json.dumps(status,indent=2)+'\n',encoding='utf8')
text=f'''# Raze — physical skills, positional satchels and landing defence

Native **{package['native']}**, champion pack **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**. Raze Lab transport **v3**, rules **`raze-3.6`**, athlete profiles **v2**. Branch `codex/raze-attack-dome` starts at main `{compare['baselineCommit']}` (0.10.74, merged PR #49). Original worktrees/local work are preserved. This release installs nothing and migrates no real career.

## Attack scaling and physical damage

Reihan corrected the earlier AP/magic direction on 29 October 2026: Raze's skills now scale with **current total Attack** and deliver attributed **physical skill damage**. Armour mitigates them. AP no longer increases Raze-owned skill payloads. Enemy AP/MR still describe incoming magic threats, and are not globally rewritten.

| Source | Raw formula before the enemy multiplier | Enemy raw at 65 Attack |
|---|---|---:|
| Armed satchel | 30 + 25% Attack | 92 |
| Grenade | 85 + 75% Attack | 266 |
| Each child | 20 + 20% Attack | 66 |
| Boom Bot | 70 + 70% Attack | 230 |
| Rocket centre | 240 + 150% Attack | 674 |

Preserve existing integer rounding and rocket falloff, then multiply eligible enemy damage by 200% (300% before round 144's viability tuning) before engine mitigation. One packet per existing hit; no additional direct-hit packet or duplicate item activation. Self-damage remains 15% of the **untripled** grenade/rocket result and is now physical too: at 65 Attack, centre grenade/rocket self requests are 19/50 before mitigation. Existing airborne/landing reduction applies through the engine.

Grenades/Bot snapshot Attack at cast acceptance, rockets at firing, and armed satchels at detonation. The native adapter reads the SDK's current Attack, including items/weapon buffs, once. Current-gun estimates no longer multiply that already-applied weapon bonus again; Omen's existing base-stat helper and all weapon mechanics remain unchanged. Lab inputs supply effective Attack once, while its displayed base-unit values and gun bonus generate that snapshot. Hypothetical weapon upgrades remain estimates.

Raze's tag changes AP→AD, retaining the supported Range category and CC/Range tags. Base stats, growth and Mod Power values remain unchanged; unused AP is not reassigned. Bucky, credits, free/paid utility, protected savings, damage types for gunfire, cooldowns, warnings and knockback retain their rules.

## Enlarged satchels, underfoot placement and weaker strafe

Air acceleration is **600,000 units/second²**, half the previous 1,200,000. Velocity changes gradually, with the existing 155° steering-input limit and 240,000/300,000 normal/rocket speed caps. Full turns remain possible with enough airtime; steering does not snap or discard momentum.

The satchel blast/launch radius is now **56,000**, doubled from 28,000: four times the area. Damage per target and maximum launch strength are unchanged. Its visible attachment point is the blast/launch centre. Carrier placement remains separate so a wall-attached destructible pack stays outside solid terrain. Existing ground/wall/construction attachments and support cleanup remain supported.

| Distance from centre | Added horizontal speed | Fresh arc peak | Fresh airtime |
|---|---:|---:|---:|
| 0–2,800 | 0 | 24,000 | 39 ticks |
| 14,000 sweet spot | 200,000 outward | 20,000 | 48 ticks |
| 56,000 boundary | 0 | 0 | None |

Interpolate between the knots using deterministic integer arithmetic. A centred stationary Raze initially rises straight up; a moving Raze keeps existing sideways velocity. Steering can then add motion normally. The sweet spot gives longer outward travel; forces diminish toward the boundary. A zero-force boundary detonation does not begin flight or grant protection.

This uses the existing two-dimensional ground-plane travel and visual-height model. Raze stays targetable at her shadow; vertical lift grants no evasion, terrain crossing or crowd-control immunity. Second satchels preserve current velocity and visual-height continuity, subject to the existing 66-tick combined flight and 84-tick rocket limit. Rocket recoil/triple boosts remain.

AI placement chooses among nine bounded dome offsets to approximate its selected travel rather than secretly reducing launch strength for a goal. Manual and athlete inputs use the same physics. Q and Place launch satchel use an explicit underfoot command: placement follows Raze through preparation and drops at her current ground position on release, then stays there. Walking and airborne momentum are preserved. Aimed throws remain available for the 14,000 sweet spot; default placement never silently offsets behind her. Existing survival/support gates revalidate uncommitted entries. Search remains 64 evaluated candidates, beam eight and three actions; all 13 athlete attributes remain active.

## Landing defence

Keep one owner-scoped, non-stacking **15% physical/magic reduction**, including self-damage through ordinary mitigation. At the end of actual flight, snapshot living visible enemy champions within 80,000 units, including the boundary:

| Champions | Extension |
|---:|---:|
| 0 | None |
| 1 | 3 seconds / 180 ticks |
| 2 | 5.25 seconds / 315 ticks |
| 3 | 7.5 seconds / 450 ticks |
| 4 | 9.75 seconds / 585 ticks |
| 5+ | 12 seconds / 720 ticks |

Minions, summons, objectives, dead and concealed champions do not count. Snapshot once at normal expiry, terrain collision or flight-ending control. Late arrivals do not refresh protection, and later concealment/death does not alter that snapshot. Another flight keeps the same buff active; its landing can extend expiry but cannot stack strength or shorten an existing expiry. Death/cleanup removes it immediately. Grounded rocket recoil cannot activate or refresh protection, but does not erase an already-earned landing buffer.

## Lab, compatibility and artwork

The Lab exposes Attack, physical estimates using target armour, launch distance/impulse/peak/airtime, centre/sweet-spot/56,000-edge guides, snapshot enemy count and defence time remaining. Ordered core transitions govern reduction for damage on landing/expiry ticks; incoming Lab attacks are applied after core commands. Native and WASM share the same combat core. The additive `drop_pack` manual command resolves underfoot at release; existing aimed `pack` commands keep their semantics. New saved-state fields have defaults. Older recordings remain inspectable and explicitly require their original rules to replay.

Approved body, Bucky icon and original effect pixels/anchors/timings remain unchanged. The existing satchel blast variant is mechanically resized to the larger area under the explicit area-change request; the compact satchel itself is unchanged. Strict reproducible art verification passes at **{art['decodedValorantBytes']:,} decoded bytes**, under 32 MiB, with every atlas within 2048 pixels. No new artwork approval is needed for these reused assets.

Optional backed-up career migration still defaults to inspection. Raze's stale AP tag becomes AD while unrelated customized tags, stats, growth, gun attacks, items and history remain intact. Native shells/presentation refresh as before. Eleven synthetic migration tests pass, including idempotence and backup behavior. No real career was opened or migrated.

[Dome placement guide](raze-results/raze-dome-guide.png) · [Landing defence](raze-results/raze-landing-defence.png) · [Attack damage](raze-results/raze-attack-damage.png)

## Automated results and matched comparison

- **{native[0]} native release tests passed**, zero failures, {native[1]} pre-existing ignored; [output](raze-results/native-tests.txt).
- **{shared} shared-core tests passed**, including dome knots/bounds, central momentum, sweet-spot travel, exact protection durations, late entrants, interruption/death, snapshots, AP independence and single Attack application; [output](raze-results/lab-tests.txt).
- **{lab['parityFrames']:,} complete native/WASM state-and-command comparisons**, 24 development/12 holdout scenarios, independent sessions/replay and 390px light/dark layouts. Three independent low/high fixture pairs for each of all 13 stats. [Parity](raze-results/lab.json), [pairs](raze-results/athlete-pairs.json).
- Actual keyboard/shop/bot/grenade/rocket/server browser checks pass, plus centre/sweet launches, 1/5-enemy landing durations, late-entry rejection and exact expiry. [Dome browser](raze-results/dome-browser.json), [balance](raze-results/balance-browser.json), [keyboard](raze-results/keyboard.json).
- Native bridge fixtures verify physical packets, armour/MR separation, shield/HP changes, one attack identity and cleanup. These are controlled API fixtures, not real-game observations.
- Five candidate release-core runs: median p95 **{perf['medianP95Ms']:.4f} ms**, p99 **{perf['medianP99Ms']:.4f} ms**. Magnus 128-piece p95 **{min(mp):.3f}–{max(mp):.3f} ms** across five runs, below four milliseconds. Host/rendering costs are excluded.

Exact merged-main and candidate sources use one identical harness, 36 seeds × three profiles × both teams (**216 matched cases**), repeated five times. Controlled armour/MR is used according to each rules revision: main skills are magic; candidate skills are physical. Both use normal Mod Power stats (65 AP/60 Attack before weapon effects), so identical coefficients do not imply identical raw payloads. Source/harness hashes and full totals are in [comparison.json](raze-results/comparison.json).

| Measurement | Main 0.10.74 | Candidate |
|---|---:|---:|
{table}

Damage is slightly lower and exposure is higher under the nerfed movement and different scaling. Allied follow-up increases, and landing protection prevents more simulated damage. Both builds record zero deaths, so this does not establish a survival benefit or live win-rate improvement. Defence uptime and damage prevention are reported separately from damage output. The economy fixture remains identical: Judge at tick 6,001, 150 credits retained and no paid utility spending. This is not a live acquisition-time claim.

## Package and acceptance

Two independent native builds and the packaged DLL match:

`{sha}`

Isolated Mod Manager verification passed: {manager['allPackFilesCompared']} pack files compared with zero differences, including supplementary Valorant hashes. [Package](raze-results/package.json) · [Manager](raze-results/mod-manager.json).

**Live acceptance remains pending:** actual launch/height readability, targeting, physical/item attribution, mitigation stacking, group-entry quality, full-match protection uptime and crowded host performance. Nothing installed; no real career migrated. Installation remains separate from PR publication.
'''
(root/'docs/raze-implementation.md').write_text(text,encoding='utf8')
branch='codex/raze-attack-dome';url='https://github.com/PowerViber/tfm2_mods/blob/'+branch;raw='https://raw.githubusercontent.com/PowerViber/tfm2_mods/'+branch
pr=f'''Raze now has a **56,000-radius satchel** (four times the previous area), with Q and the launch button dropping directly beneath her at release. Her explosives scale with Attack and deal physical damage. Satchel placement determines lift/travel, air steering is weaker, and the existing 15% protection lingers for 3–12 seconds according to nearby enemy champions at landing.

- Apply the 200% enemy skill multiplier (300% before round 144), coefficient/rounding/falloff rules and 15% untripled self-damage. Current total Attack is used once; gunfire and Omen retain their mechanics.
- Centre adds lift without erasing momentum; 14,000 offset gives 200,000 outward speed/48 ticks; forces fade at the 56,000 edge. Q/the launch button drop underfoot at release; aimed throws remain separate. Air acceleration halves to 600,000. AI and manual movement use the same dome curve.
- Landing snapshots visible champions within 80,000: 0→none; 1–5→3/5.25/7.5/9.75/12 seconds. No minion/summon counting, strength stacking or late-arrival refresh; death clears the buff.
- Lab v3 / `raze-3.6` exposes physical damage, profile guides and defence timers. Optional inspected/backed-up migration updates the AD tag while preserving career data. Original approved artwork is preserved; satchel blast artwork is mechanically resized.

Native **{package['native']}**, custom **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**, based on current main/merged PR #49.

### Verification
- **{native[0]} native / {shared} shared tests passed**, {native[1]} existing ignored; 6,480 parity frames; 24 development + 12 holdout scenarios; all 13 stat comparisons; browser/keyboard/replay and 11 migration tests.
- 216 matched cases: estimated delivered damage {a['labEstimatedDelivered']:,}→{b['labEstimatedDelivered']:,}; exposure {a['exposureTicks']:,}→{b['exposureTicks']:,}; allied follow-up {a['followupDamage']:,}→{b['followupDamage']:,}; damage prevented estimate {a['airDamagePreventedEstimate']:,}→{b['airDamagePreventedEstimate']:,}. Both zero deaths; no live survival/win-rate claim.
- Five-run core p95 **{perf['medianP95Ms']:.4f} ms**, p99 **{perf['medianP99Ms']:.4f} ms**; Magnus 128-piece p95 **{min(mp):.3f}–{max(mp):.3f} ms**.
- Reproduced/package DLL SHA-256: `{sha}`. Isolated Mod Manager and supplementary hashes: **{manager['allPackFilesCompared']} files, zero differences**.

![Dome placement guide]({raw}/docs/raze-results/raze-dome-guide.png)
![Landing defence]({raw}/docs/raze-results/raze-landing-defence.png)

[Implementation/results/migration/limits]({url}/docs/raze-implementation.md) · [Matched evidence]({url}/docs/raze-results/comparison.json)

**Draft: live-game acceptance remains pending.** Nothing installed, no real career migrated, and original local work preserved. Installation remains separate.
'''
(root/'backups/raze-pr-body.md').write_text(pr,encoding='utf8')
print(json.dumps({'nativeTests':native[0],'sharedTests':shared,'rules':'raze-3.6','dll':sha}))
