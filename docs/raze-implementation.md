# Raze — Bucky start, 3× skill damage and worthwhile entries

Native **0.10.74**, champion pack **0.2.74**, Valorant **0.1.23**. Raze Lab transport **v3**, rules **`raze-3.3`**, pinned athlete profiles **v2**. Branch `codex/raze-bucky-entry-balance` starts at main `7b5ab117ab6ef3ae94a54f8180b971d028d7f67e` (merged 0.10.73), with only the unmerged Lab route repair cherry-picked before this rework. PR #46 is already merged; this work belongs in a new PR. The original worktree and its uncommitted changes are preserved.

## Damage and identity

Raze remains an AP fighter using the supported `Range` category. AP scaling and magic damage describe different properties: AP determines raw skill strength, and magic resistance mitigates the resulting damage. Her existing stats, growth and +30% Mod Power bonus remain unchanged. The SDK current-stat snapshot already includes that bonus and items; it is not multiplied again here.

Satchel, grenade, child, Boom Bot and rocket enemy damage is exactly three times the original integer result, after AP rounding and distance falloff and before mitigation. All currently eligible enemy types retain their original eligibility. Gunfire, displacement, control, movement and self-damage do not receive the multiplier.

| Source at 65 current AP | Merged main raw | Candidate raw |
|---|---:|---:|
| Armed satchel | 46 | 138 |
| Grenade | 133 | 399 |
| Each child | 33 | 99 |
| Boom Bot | 115 | 345 |
| Rocket centre | 337 | 1,011 |

Grenade and rocket self-damage remain 15% of the original unbuffed result, with original rounding, distance falloff and ordinary mitigation. At 65 AP a centre grenade requests 19 self-damage and a centre rocket 50. Native skill packets remain attributed magic damage. A bounded, opt-in native receipt records source, requested raw, submission status and observed HP/shield changes. This separates unavailable/untargetable victims from weak formulas. Readbacks can include engine item interactions and are not a replacement for live attribution checks.

## Bucky and protected savings

Raze starts each new match with a free Bucky (appended ID 6) and 1,000 credits. IDs 0–5, Omen's Classic default, Omen's weapon choices/prices and his gun-first policy are unchanged. Weapons persist through death; respawning never replaces an upgraded weapon with the starter.

Bucky has 18,000 range, 300% AD through 8,000, linear falloff to 60% at 18,000 and a −40% weapon attack-speed modifier. Ordinary engine bonuses still apply. Its physical primary attack keeps one engine attack identity, the approved pellet effect and existing 35% secondary shotgun splash. It is distinct from Judge.

Raze reserves the full Judge price (1,850) for viable close entries or Vandal price (2,900) for safer ranged pressure. She spends only surplus on extra utility or armour, and buys the selected weapon first when affordable. She skips automatic intermediate Sheriff/Spectre purchases. Paid automatic replacements require at least 20% estimated improvement, 15 seconds since the previous purchase and no tier downgrade. The goal is reviewed at most every 15 seconds unless its tactical approach becomes invalid. Manually triggering Auto-buy uses a neutral athlete profile and the same automatic purchasing safeguards.

Main's free utility charges, paid second charges, cooldowns, death rules and credit income are preserved. In the identical 6,500-tick scripted utility-demand test, main ends on Sheriff with 0 credits and 1200 utility spending; the candidate buys Judge at tick 6001, keeps 150 credits and spends 0 on paid utility. This is a controlled economy fixture, not a claim about weapon acquisition time in a live match.

[Bucky icon comparison](raze-review/bucky/comparison.png) · [Shop and protected savings](raze-results/raze-shop.png)

## Offensive entry review

The planner and every uncommitted offensive pack/detonation recheck a clear legal route, useful landing and conservative 1.5-second survival estimate leaving at least 20% maximum HP. Visible champions, buildings, warnings and expected unbuffed rocket self-damage contribute to the estimate. Unknown routes or missing movement/range data cannot qualify a teammate as timely support.

An entry then requires either an allied **champion** able to contribute within 1.5 seconds of arrival, or a retained usable satchel and clear legal exit. Minions and summons never count as champion follow-up. Spending both satchels needs timely support, except for an isolated predicted kill with a clear walking retreat and no other visible enemy champion within 80,000. Kill estimates use conservative accuracy/mitigation and actually available follow-up attacks. A backward boost is not counted as forward rocket damage.

Support loss or a dangerous landing cancels remaining uncommitted work, including the second launch. Released flight/effects continue under the existing mechanics. Each queued continuation retains its own landing rather than inheriting the first action's landing. Manual Lab commands bypass these decision gates while retaining normal resources and terrain validation.

All thirteen athlete attributes retain their roles and all legal action families remain available. Search limits remain 64 candidates, beam width eight and three actions. Visibility and deterministic decision hashing are unchanged. This work preserves main's movement fixes, 155° steering limit, held ground guidance, free-utility lifecycle, rocket cooldown starting on firing and one-frame renewal visuals.

## Lab, artwork and compatibility

The Lab exposes Bucky and its falloff, protected goal/reserve/reason, entry rejection, predicted HP, retained escape, allied identity/ETA and raw damage by source. It retains WASD, Q–Q, Shift+Q, grenade, rocket and bot controls. The carried-forward routing repair supports both `/editor/raze.html` and the older `/raze.html` bookmark while preserving queries/fragments.

Older recordings remain inspectable. A different rules revision explicitly requires its original rules for replay; no silent combat conversion is attempted. New observation/presentation fields have compatible defaults. Existing profile v2 persistence and independent owner/simulation state are preserved.

Approved body/effect pixels, timing, anchors and the 36×44 body remain unchanged. The only new drawing is the distinct Bucky HUD icon, approved by the user on 26 October 2026 and verified against its recorded hash. Preview generation is separated from strict art/package checks. Total decoded Valorant art is **6,215,296 bytes**, below 32 MiB, and all atlases remain within 2048 pixels per side. Existing Omen pixels are verified unchanged.

The existing backed-up career migration still defaults to inspection. It refreshes Raze native shells and visual bindings while preserving custom stats, growth, attacks, items and history. Eleven synthetic tests cover idempotence and backup safety. No real career was read for migration or changed. Installation remains separate.

## Automated verification

- **730 native release tests passed**, zero failures, 8 pre-existing ignored. [Output](raze-results/native-tests.txt).
- **104 shared-core tests passed**, including exact enemy/self damage, AP snapshots, Bucky cadence/falloff/death, Omen defaults, saving/replacements, champion-only follow-up, blocked/late support, survival, solo picks and second-launch cancellation. [Output](raze-results/lab-tests.txt).
- Three independent 20-vs-80 fixture pairs for **each of all 13 stats**, with the other twelve held at 50. [Seeds](raze-results/athlete-pairs.json).
- **6,480 full native/WASM state-and-command parity frames** across 24 development and 12 holdout scenarios, independent sessions, identical replay and 390px light/dark layouts. [Results](raze-results/lab.json).
- Actual browser keyboard/mouse, shop, damage and entry checks pass. These include Bucky's 90 raw / 75 estimated mitigated hit at 13,000 and a 399 raw / 332 estimated mitigated grenade hit at 65 AP. [Balance](raze-results/balance-browser.json), [Shop](raze-results/shop.json), [Keyboard](raze-results/keyboard.json), [V3](raze-results/v3-browser.json), [Movement/bot](raze-results/v32-browser.json), [Server routes](raze-results/server.json).
- Native bridge fixtures verify one attributed magic submission, observed HP/shield changes under a controlled host, rejected targets and one Bucky primary identity. These are API fixtures, not observations in the real game.
- Canonical generation, approved old pixels, icon binding, atlas limits and replay compatibility checked. Farming metadata covers 78 champions/234 slots/11,700 role-team-target combinations; it does not prove live casts.
- Five release-core runs: median p95 **0.0021 ms**, p99 **0.0350 ms**. Five Magnus 128-piece p95 measurements: **0.813, 0.805, 0.685, 0.820, 0.800 ms**, all below four milliseconds. Host/rendering performance is excluded.

## Matched main comparison

The exact main Raze core and the candidate use the same harness, current Mod Power stats, simplified 20-resistance mitigation, visible allied movement/basic follow-up, 36 seeds, three profiles and both teams: **216 matched cases**, repeated five times. Baseline observation-only fields allow the same fixture structure; candidate gameplay is not backported into main. Source and harness hashes are recorded in [comparison.json](raze-results/comparison.json). [Reproduction tool](../tools/compare-raze.py).

| Measurement | Merged main | Candidate |
|---|---:|---:|
| Requested raw skill damage | 281,606 | 653,936 |
| Lab-estimated delivered damage | 232,666 | 514,992 |
| Landing exposure ticks | 50,172 | 29,042 |
| Objective damage estimate | 11,036 | 39,468 |
| Launches | 248 | 186 |
| Entries receiving allied follow-up | 196 | 82 |
| Allied follow-up damage | 9,934 | 3,798 |
| Deaths | 0 | 0 |
| Wasted preparations | 0 | 0 |

More requested/delivered damage and fewer exposure ticks are promising harness results. Fewer launches also produce fewer allied-follow-up events; those counts are reported rather than described as a blanket improvement. Both builds record zero deaths, so these cases do not establish improved survival. The comparison is not a win-rate study, and different combat outcomes make timing differences unsuitable as proof of a behavior-preserving optimization. Utility spending in fight fixtures is zero because they use equal practice stocks; the separate economy fixture tests real purchasing pressure.

## Package and live acceptance

Two independent DLL builds and the packaged DLL match: `0ec98d71536a73f7ae9a51fb1f48cbbb24c8f2c167cf79bccd71bb97c22ec879`. Isolated Mod Manager verification passed; supplementary hashes compared all 830 pack files with zero differences. Nothing was installed and no real career was migrated.

**Live game acceptance remains pending:** actual Bucky cadence/splash/items; Raze requested versus delivered damage; native carrier rendering/targeting; champion follow-up and retreat quality; shields/mitigation/attribution; crowded matches and host/render performance. Automated fixtures and Lab estimates are reported separately. No freeze or match balance claim is made from these tests.
