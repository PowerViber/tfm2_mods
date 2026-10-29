# Raze — physical skills, positional satchels and landing defence

Native **0.10.76**, champion pack **0.2.76**, Valorant **0.1.25**. Raze Lab transport **v3**, rules **`raze-3.5`**, athlete profiles **v2**. Branch `codex/raze-attack-dome` starts at main `73c4274c7918bfb00a5998f60fa204ef28d464be` (0.10.74, merged PR #49). Original worktrees/local work are preserved. This release installs nothing and migrates no real career.

## Attack scaling and physical damage

Reihan corrected the earlier AP/magic direction on 29 October 2026: Raze's skills now scale with **current total Attack** and deliver attributed **physical skill damage**. Armour mitigates them. AP no longer increases Raze-owned skill payloads. Enemy AP/MR still describe incoming magic threats, and are not globally rewritten.

| Source | Untripled raw formula | Enemy raw at 65 Attack |
|---|---|---:|
| Armed satchel | 30 + 25% Attack | 138 |
| Grenade | 85 + 75% Attack | 399 |
| Each child | 20 + 20% Attack | 99 |
| Boom Bot | 70 + 70% Attack | 345 |
| Rocket centre | 240 + 150% Attack | 1,011 |

Preserve existing integer rounding and rocket falloff, then multiply eligible enemy damage by exactly three before engine mitigation. One packet per existing hit; no additional direct-hit packet or duplicate item activation. Self-damage remains 15% of the **untripled** grenade/rocket result and is now physical too: at 65 Attack, centre grenade/rocket self requests are 19/50 before mitigation. Existing airborne/landing reduction applies through the engine.

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

Approved body, Bucky icon and original effect pixels/anchors/timings remain unchanged. The existing satchel blast variant is mechanically resized to the larger area under the explicit area-change request; the compact satchel itself is unchanged. Strict reproducible art verification passes at **7,263,872 decoded bytes**, under 32 MiB, with every atlas within 2048 pixels. No new artwork approval is needed for these reused assets.

Optional backed-up career migration still defaults to inspection. Raze's stale AP tag becomes AD while unrelated customized tags, stats, growth, gun attacks, items and history remain intact. Native shells/presentation refresh as before. Eleven synthetic migration tests pass, including idempotence and backup behavior. No real career was opened or migrated.

[Dome placement guide](raze-results/raze-dome-guide.png) · [Landing defence](raze-results/raze-landing-defence.png) · [Attack damage](raze-results/raze-attack-damage.png)

## Automated results and matched comparison

- **741 native release tests passed**, zero failures, 8 pre-existing ignored; [output](raze-results/native-tests.txt).
- **115 shared-core tests passed**, including dome knots/bounds, central momentum, sweet-spot travel, exact protection durations, late entrants, interruption/death, snapshots, AP independence and single Attack application; [output](raze-results/lab-tests.txt).
- **6,480 complete native/WASM state-and-command comparisons**, 24 development/12 holdout scenarios, independent sessions/replay and 390px light/dark layouts. Three independent low/high fixture pairs for each of all 13 stats. [Parity](raze-results/lab.json), [pairs](raze-results/athlete-pairs.json).
- Actual keyboard/shop/bot/grenade/rocket/server browser checks pass, plus centre/sweet launches, 1/5-enemy landing durations, late-entry rejection and exact expiry. [Dome browser](raze-results/dome-browser.json), [balance](raze-results/balance-browser.json), [keyboard](raze-results/keyboard.json).
- Native bridge fixtures verify physical packets, armour/MR separation, shield/HP changes, one attack identity and cleanup. These are controlled API fixtures, not real-game observations.
- Five candidate release-core runs: median p95 **0.0029 ms**, p99 **0.0347 ms**. Magnus 128-piece p95 **0.715–1.151 ms** across five runs, below four milliseconds. Host/rendering costs are excluded.

Exact merged-main and candidate sources use one identical harness, 36 seeds × three profiles × both teams (**216 matched cases**), repeated five times. Controlled armour/MR is used according to each rules revision: main skills are magic; candidate skills are physical. Both use normal Mod Power stats (65 AP/60 Attack before weapon effects), so identical coefficients do not imply identical raw payloads. Source/harness hashes and full totals are in [comparison.json](raze-results/comparison.json).

| Measurement | Main 0.10.74 | Candidate |
|---|---:|---:|
| Requested raw damage, including modelled gunfire | 648,068 | 627,640 |
| Lab-estimated delivered damage | 511,276 | 497,550 |
| Exposure ticks | 29,488 | 30,568 |
| Objective damage estimate | 40,014 | 37,188 |
| Launches | 172 | 156 |
| Entries receiving allied follow-up | 78 | 76 |
| Allied follow-up damage | 3,552 | 3,710 |
| Airborne protection ticks | 5,920 | 5,696 |
| Landing protection ticks | 0 | 18,488 |
| Damage prevented estimate | 734 | 2,040 |
| Deaths | 0 | 0 |
| Wasted preparation | 0 | 0 |

Damage is slightly lower and exposure is higher under the nerfed movement and different scaling. Allied follow-up increases, and landing protection prevents more simulated damage. Both builds record zero deaths, so this does not establish a survival benefit or live win-rate improvement. Defence uptime and damage prevention are reported separately from damage output. The economy fixture remains identical: Judge at tick 6,001, 150 credits retained and no paid utility spending. This is not a live acquisition-time claim.

## Package and acceptance

Two independent native builds and the packaged DLL match:

`a5efc3c52c55a4fd6736efb8107de0857f7f696b2ab403e931c5217ae0b4b927`

Isolated Mod Manager verification passed: 830 pack files compared with zero differences, including supplementary Valorant hashes. [Package](raze-results/package.json) · [Manager](raze-results/mod-manager.json).

**Live acceptance remains pending:** actual launch/height readability, targeting, physical/item attribution, mitigation stacking, group-entry quality, full-match protection uptime and crowded host performance. Nothing installed; no real career migrated. Installation remains separate from PR publication.
