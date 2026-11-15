# Kagami: approved art, athlete decisions and actual-map Lab

Follow-up to PR #86 head 07e5d41cfa226bd11302cff58efe8d3bbc6f503d. The existing kit, Mod Power, damage, healing, shields and cooldowns are unchanged. Native 0.10.108 and champion pack 0.2.111 remain this open PR's reserved versions; main was still 0.10.107 at the collision check.

## Approved artwork

On 15 November 2026 Reihan approved the silver/lavender effects, then approved the third body revision after rejecting two proposals as blurry. The final design has opaque dark contours, violet hair, broad ivory/lavender sleeves, navy lower clothing and an octagonal mirror. Reflection derives from the same complete body drawings.

Eight poses in both facings: idle 4, walk 6, basic 4, Mirror Step 4, Charm/heal 4, ultimate 4, hit 1 and death 6. Basic timing remains 22 ticks with release at tick 12. The 48-tick walk has six distinct drawings. Both atlases are 1024×512, using **4 MiB decoded** together, below 32 MiB.

- [Body comparisons](kagami-art-review/body-comparison.png), [walking](kagami-art-review/run.gif)
- [Effects](kagami-art-review/effects-comparison.png)
- [Situation preview](kagami-art-review/lab.png), [full map](kagami-art-review/full-map.png)
- [Approvals](kagami-art-review/approvals.json), [art checks](kagami-art-review/verification.json)

Imagegen sources and prompts are frozen. AI generation is not deterministic; extraction, palette conversion, mirroring, animation clocks and packing reproduce from those sources.

Kagami retains her visible real body and engine-owned basic animation. Native casts add effects without cosmetic movement/action locks. The Lab inspects all casting drawings; it does not prove that the native host selects those body poses for instant native actions.

Reflection still uses the known base priest carrier: custom spawn names can crash the host. The approved-art parameter adds one lifetime cosmetic animation mask to that core-controlled summon only, with short following single-frame body leases. Its existing attack/skill blocks and glides remain. No Bind, global invisibility, global Priest override or owner-body hiding is added. Actual-game clone rendering and targetability remain pending.

## Athlete decisions and Global Flash

| Value | Decision influence |
|---|---|
| Ego | Risk cost for offensive swaps and escape willingness |
| Judgement | Rescue threshold, pressure assessment, bounded estimation error |
| Positioning | Clone staging and spacing behind the carry |
| Control Speed | 24–6 tick response to a new situation |
| Skill Accuracy | Visible-motion lead and deterministic point error |
| Skill Dodge | Visible warning costs and movement escapes |
| Aggressive | Offensive swap preference and pressure support |
| Order | Champion follow-up and coordinated rescue/buff value |
| Roaming | Distance at which to consider a teammate rotation |
| Macro | Overheal and low-value rotation costs |
| Micro | Steering/clone input cadence, preserving clone speed |
| Map Awareness | Uncertain last sightings; no hidden current position/resources |
| Strategy | Ranked authored follow-up comparisons and continuation hints |

The planner evaluates at most 64 candidates, retains eight ranked candidates and exposes up to three actions including the selected action. Continuations are hints, revalidated on the next decision, not an irreversible cast queue. Last Reflection, Guardian Angel and Mirror Swap still protect immediately. Manual commands retain identical effects across profiles. Disabled shared movement-assistance defaults remain unchanged.

Kagami already inherits **40,000-range, 120-second Global Flash**. The guard now recognises her seals and owner-scoped pending-swap leases on the actual two participants. They can still walk and act. Completion, cancellation, death and respawn clear only that Kagami's lease. Refused Flash does not spend its cooldown.

## Playable Lab

Start with tools/start-kagami-review.ps1:

- http://localhost:7307/kagami.html
- http://localhost:7307/kagami-review.html

Root and /editor/ routes are checked. The server extracts the locally installed game reference: 960,000 world units, 30×30 collision cells, 90 blocked cells, 1,000 units per art pixel. No estimated map fallback or game-asset redistribution is added.

Includes stock champions, waves, camps, towers, objectives, eight midgame situations, thirteen controls, WASD, aim, basic, Mirror Step, Charm and Mend; pause/step, zoom/pan, diagnostics and recordings. The ultimate remains automatic. Clone glides and the link follow current actors. Simulation runs at 60 ticks/s independently of display refresh.

Browser movement, combat, buffs, visibility, seals and enemy tactics are explicitly simulated host behaviour. Stock skill AI and live mitigation are not reproduced. Flash status is explained; its native global policy is not falsely recreated in JavaScript. Older recordings remain inspectable and require original rules for replay.

## Automated evidence

- Native GNU suite: **1,306 passed, 18 existing ignored**; 31 Kagami/AI tests pass. Four inherited unused-helper warnings remain in native bridge tests.
- Lab crate: **nine tests pass**, including shared AI checks and JSON rescue.
- Every attribute compared at 20/80 over three relevant fixtures; manual profile comparisons cover 39 fixtures.
- Complete native/WASM state and effects agree across **24 development/12 holdout snapshot scenarios, 4,320 ticks**. These are deterministic fixtures, not live balance results. [Parity](kagami-art-review/parity.json).
- **42 browser checks**: routes, map, actors, controls, rescue, seal/revive, eight situations, replay and 30/60/120/144/240 Hz clock checks. [Browser evidence](kagami-art-review/browser.json).
- Reproducible packing, frozen sources, mirrors, anchors, six walking drawings, bounds, durations, bindings and unchanged mechanical data pass.
- **15 Mod Manager tests** pass, including every shipped champion's load schema. Isolated comparison: **895 files**, zero differences, identical DLL. This is not a real installation/rendering proof.
- Canonical Kagami, Kicker, Akane, Kratos, Wright, Thalassa and Grimhald generators are current.
- Five 1,800-tick mock-host runs: about 5.3–6.0 microseconds/tick with art disabled. Browser/WASM timings are separate. Neither predicts real Proceed waiting time.

Two isolated builds produced the same packaged DLL SHA-256:

dd11fcff1114a600089461c8096b29ec1303142a57328910aec1a95cc1e81131

[Package report](kagami-art-review/package.json), [performance](kagami-art-review/performance.json), [Mod Manager](kagami-art-review/mod-manager.json).

Reproduce with tools/package-kagami.py, tools/verify-kagami-art.py, tools/test-kagami-parity.cjs and tools/test-kagami-lab.cjs.

## Pending actual-game acceptance

No live game was launched. Check Reflection rendering, targetability, motion/facing and cleanup; seal visibility under native banish; burst/turret/true-damage saves; independent owners and relevant realm/cage interactions. Revive balance and rescue frequency still need playtesting.

No real game installation or career migration was performed. Publication does not establish live acceptance.
