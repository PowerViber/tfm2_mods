# Rally 2.1: draft review handoff

This is the **implemented Rally 2.1 snapshot**, published for Claude's review at the user's request on 2026-11-01. The later confirmed requests below are a separate backlog, not claims about this snapshot. Do not treat its Lab previews as native-game acceptance.

Publication does not approve artwork or authorize installation, career migration, game-art integration or release packaging. No final Rally champion definition has been added to the shipped roster. Native/passive wiring and generator preparation are included so the prototype can be reviewed.

## Implemented in this snapshot

| Area | Current 2.1 behavior |
|---|---|
| Core | Deterministic Rust core shared by the native adapter and WASM Lab; canonical rules; owner-scoped state; thirteen pinned athlete attributes, no mastery |
| Roles | Accepted Receive selects Receiver, Set selects Setter, Run-up selects Spiker; rejected actions do not switch roles |
| Receive / Set | Shield/reduction, visual clone Self Set, independent teammate support marker with buff linger; the helper is not a combat summon |
| Run-up | Directional 1.4x sprint for 90 ticks (1.5 seconds), optional jump, one initial 36-tick gauge; perfect takeoff at ticks 24-29 |
| Ball | Hybrid palm/volley basics; committed spike returns after a registered hit; a miss, wall or exhausted travel grounds it |
| Thunder | Ordinary yellow Thunder follows existing height scaling. Black requires its armed finisher and valid contact geometry; height alone does not select Black. White uses the current Serve/perfect-apex path |
| Permanent power | Black and White role bonuses/forms currently require delivered-hit confirmation, not release alone; retain highest earned form until death |
| Speed | All damaging spikes currently use 4,500 units/tick, plus the existing Self Set speed bonus; no permanent-form speed ladder yet |
| Serve | Fixed 60-degree V, 560,000 range, 75-second cooldown, 0.4-second preparation, one attempt, 70% added final height; every Serve impact/crater uses 90,000 radius |
| Serve collision / vision | Stops on walls; no true map reveal is implemented |
| Black command | Separate Shift+R, spending three charges and all Flow when at least 60; 30-second prototype lockout; one eight-second attempt |
| Lab | Manual controls, starting Yellow/Black/White presets, armed-Black checkbox, recordings, state inspection and shared-core parity |
| Art | Full-face yellow, black/red and white body proposals, clone setter, gold Thunder, angular Black/White lightning and crater proposals; review-only, not approved game assets |

Current White-capable flight holds at the low phase and blinks to its apex; it does not yet permit the newly requested earlier lower-height contacts. See [implementation details](rally-implementation.md) for current prototype values, command semantics and proof limits.

The farming-capability generator change prepares Contender/Rally mappings and validates the canonical roster instead of assuming a fixed count. Generated shipped-roster coverage is not claimed for an unregistered Rally definition.

## Confirmed changes NOT implemented

These requests came after the tested 2.1 snapshot. Publishing first takes priority; they must not be mistaken for completed code.

- [ ] Automatically return every released spike after hit, miss, wall collision or maximum travel. The request concerns spikes; changes to ordinary volley retrieval or interrupted tosses still need an explicit decision.
- [ ] Select attack tier from **actual contact height alone**, including earlier contacts during a higher-capable jump:

  | Actual height | Attack |
  |---|---|
  | Below 4 m | Normal spike, no thunder |
  | 4 m to below 6 m | Yellow Thunder |
  | 6 m to below 10 m | Black Thunder |
  | 10 m or higher | White Storm |

- [ ] Earn the corresponding permanent form and role bonuses on successful timed contact/release, even if the released projectile later misses. This replaces the current delivered-hit trigger.
- [ ] Keep the highest permanent appearance and bonuses until death. A White-form athlete can release a lower-tier spike when contacting lower; the body must not downgrade.
- [ ] Base the normal < yellow < black < white ball-speed ladder on **permanent form**, independently of the current attack tier. Exact speeds remain proposals, not approved defaults.
- [ ] Keep the separate paid Black command as a jump boost; it must no longer gate Black attack classification.
- [ ] Increase directional Run-up to **2x speed for three seconds**, retain its five-second cooldown and optional jump, and repeat the 36-tick gauge with the existing perfect window/displacement requirement.
- [ ] Allow earlier lower-height contacts in White-capable jumps. Apply the White apex teleport only to a qualifying 10 m-or-higher contact.
- [ ] Make **Serve only** penetrate walls. Ordinary spikes still collide. This does not imply unit piercing, wall destruction, out-of-map travel or bypassing realm rules.
- [ ] Reveal the full fixed V when Serve opens, subject to genuine native visibility support being proven.

Classify height using the unrounded value; the UI must not label a sub-threshold contact as the next tier by rounding. Snapshot form-dependent projectile speed after a successful contact earns its form, without retroactively increasing that contact's height. Retain existing damage prototypes unless a later balance decision changes them.

### Native vision feasibility remains open

The inspected stable SDK exposes a team/target visibility query and global invisibility controls, but no verified team-scoped sector-reveal setter. An AI marker treated as visible by one mod controller is **not** shared engine/map vision.

Verify a supported execution route and actual host behavior before claiming that Serve opens the map. Do not silently replace this with hidden-enemy knowledge, clear unrelated invisibility or add an unapproved native hook. Reveal duration and stealth precedence also remain unresolved.

## Recorded verification

The following are the recorded results for the unchanged 2.1 source/WASM hashes, rechecked for this publication:

| Check | Recorded result |
|---|---|
| Native suite | **904 passed**, 0 failed, 11 ignored; four pre-existing Windows test-helper warnings |
| Rally Lab suite | **63 passed**, 0 failed |
| Browser checks | **15 passed** |
| Real editor routing | Passed in an isolated inert-game fixture; no real career access |
| Canonical rule consistency | Passed again before publication |
| Preview artwork | 31.4609375 MiB decoded; atlas dimensions within 2048 pixels; deterministic extraction/packing reports included |
| Scenario runs | 24 development / 12 holdout scripted seeds plus five core timing runs; not a host balance result |

See [verification and hashes](rally-results/verification.json), [browser observations](rally-results/browser.json), [route checks](rally-results/editor-route.json), [scenarios](rally-results/scenarios.json) and [art reproduction](rally-results/art-reproduction.json). The incomplete intermediate native console capture is excluded rather than presented as a complete test log. Tests were not rerun merely to publish unchanged code.

An earlier parallel native test run had a nonreproducing Raze parity failure; the focused rerun and subsequent serial full suite passed. These counts do not establish live-game acceptance or the requested 45-55% matched-role balance.

### Native acceptance still pending

- Hybrid basic reach, accepted impacts and one primary item-proc identity.
- Individually attributable incoming receipts for caught power and ally champion-hit receipts for support Flow. Both native receipt routes currently fail closed; Lab receipts are simulated.
- Actual movement/contact holds, learning gates, items, walls/smoke/parry, realm isolation, ownership and death cleanup.
- Native shared vision for Serve, requested-versus-delivered damage, crowded-game performance and peer balance.
- Final artwork approval and complete pose/facing review, shipped definition/roster integration, release versions and reproducible packaged DLL.
- Installation, real-career migration and live-game visual/combat acceptance.

## Review entry points

- [Canonical rules](../rules/rally.json)
- [Shared core](../native/tfm2_custom_ai/src/rally_core.rs), [native adapter](../native/tfm2_custom_ai/src/rally.rs), [native fixtures](../native/tfm2_custom_ai/src/rally_tests.rs)
- [Lab transport and parity tests](../native/rally_lab/src/lib.rs)
- [Lab UI](../editor/rally.html), [art review](../editor/rally-review.html)
- [Ordinary Thunder animation](rally-art-review/ordinary-thunder-lab.gif), [Black Thunder animation](rally-art-review/black-thunder-lab.gif), [White Storm animation](rally-art-review/white-storm-lab.gif)

For the read-only review server, run:

~~~powershell
rtk proxy node tools/serve-rally-review.cjs
~~~

Then open http://localhost:7294/editor/rally.html and http://localhost:7294/editor/rally-review.html. Build/test instructions are in the implementation document. The normal editor-route fixture starts a separate server with isolated test paths.

Frozen AI source images are included so extraction/packing is reproducible; image generation itself is not deterministic. Rejected source copies, intermediate logs, target directories and raw videos are excluded. No installed mod files, release DLL or packaged art were changed by publication.

The user will share the draft PR with Claude. No direct Claude message or review request is part of this handoff.
