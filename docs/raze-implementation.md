# Raze — rocket entries, airborne protection and thirteen athlete attributes

Native **0.10.69**, champion pack **0.2.69**, Valorant **0.1.18**, Raze Lab **v2**. Continue PR #46 on `codex/raze-air-strafe`; main `6415d0b` and its Levi jungle fix are incorporated. The original checkout's local Magnus changes are preserved. This release does not install mods or migrate a real career.

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

Approved body pixels, 36×44 canvas, foot anchor, original animation names/counts/timing and both facings are preserved. Eight source poses are reused across those sequences. Existing frames remain pixel-identical; the new blast/warning variants are nearest-neighbour resizes authorized by the plan. Full Valorant decoded storage: **4,781,696 bytes**; largest Raze atlas **1024×488**, below both limits. The public view API has no individual point-effect cancellation handle; native warning visuals have bounded tails, while owned danger markers clean up separately. Live presentation acceptance remains pending.

[Body comparison](raze-review/raze-comparison.png) · [Effects](raze-review/effects-comparison.png) · [Air protection screenshot](raze-results/raze-air-protection.png) · [Research and timestamps](raze-entry-research.md)

## Recorded verification

- Native release: **671 passed, zero failed, 8 pre-existing ignored**; [raw output](raze-results/native-tests.txt).
- Shared Raze Lab: **57 passed**, including endpoint/range/collision/falloff, offensive entry execution, paid backward boosts, continuous height, ground recoil, protection transition ordering, manual independence and purchase safeguards.
- Three command-trace fixture pairs per attribute at 20 versus 80 with the other twelve fixed at 50; [seeds](raze-results/athlete-pairs.json).
- **36 scenarios: 24 development and 12 holdout; 6,480 complete native/WASM state-and-command comparisons**, identical replay, independent sessions and 390px light/dark layouts; [Lab results](raze-results/lab.json).
- Real keyboard/mouse tests cover WASD, normalized diagonals, queued Q–Q, air steering, grenade/equip/fire, ground recoil without protection, focus loss and legacy inspection; [results](raze-results/keyboard.json).
- Canonical generation, original approved frame pixels/timings and atlas limits verified. Farming metadata covers 78 champions, 234 slots and 11,700 role/team/target combinations; this is not proof of live casts.
- Five matched Raze release-core timing runs: median p95 **0.0118 ms**, p99 **0.0521 ms**, 194,400 steps per run. Magnus 128-piece p95: **1.915, 1.872, 2.184, 2.659, 1.492 ms**, all below four milliseconds. Host/rendering timing is excluded.

The first matched case set recorded 114 rocket shots, 150 enemy hit requests, 14 airborne rocket entries, 0 backward third boosts, 36 objective hit requests and 5250 protected ticks. These are harness events, not engine-attributed damage or proof of live match quality.

| Uniform profile | Requested raw | Lab estimated delivered | Exposure ticks | Objective estimate | Launches |
|---|---:|---:|---:|---:|---:|
| 20 | 44866 | 36988 | 15356 | 2032 | 28 |
| 50 | 52604 | 43324 | 20926 | 2236 | 56 |
| 80 | 44384 | 36582 | 15748 | 1804 | 62 |

The short harness still has limited balance coverage: inspect its deaths, wave kills and missed-wave counts separately. Higher Ego/all-high profiles do not guarantee greater damage or wins. Earlier v1 benchmark output did not model Judge falloff or air protection; it is not a clean balance baseline for this corrected harness.

## Delivery and outstanding observations

No actual game match was observed for this release. Pending: carrier targeting/coverage, in-flight attacks, native mitigation/other reductions/shields/items, interruption and death ordering in the host, attribution, actual objective fights, warning tails and crowded rendering/performance. Public projectile snapshots lack exact hitboxes and per-projectile visibility; dodge guidance retains its conservative visible-caster/clear-line approximation. Prototype core collision is not proof of exact engine geometry.

Reproduce with `rtk proxy python -X utf8 tools/check-raze-release.py` while serving the checkout at `http://localhost:7286`; then run `rtk proxy python -X utf8 tools/package-raze.py` and `rtk proxy python -X utf8 tools/record-raze-delivery.py`. The pipeline builds native/WASM, tests, records five-run timings, reproduces the DLL in two directories and checks an isolated package fixture. No Steam installation or real-career migration occurs.

Manager verified its custom-pack/native scope; supplementary hashes checked **829 total files, including 15 Valorant files**, with zero differences. Toolchain/source/WASM fingerprints are in [package.json](raze-results/package.json). Both independently built DLLs and the package have SHA-256:

`ba6ddcd3d2eeba03b801775693b7ce6b2d07186d93d12a8cfa4b0eb7bfa13a1a`
