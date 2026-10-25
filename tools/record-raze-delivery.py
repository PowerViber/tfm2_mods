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
text=f'''# Raze — stronger sticky satchels and accelerating rockets

Native **{package['native']}**, champion pack **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**, Raze Lab **v3**. Continue PR #46 on `codex/raze-air-strafe`; main `{package['baseCommit']}` and its Levi jungle fix are incorporated. The original checkout's local Magnus changes are preserved. This release does not install mods or migrate a real career.

## Mechanics

Raze is an AP fighter, using the supported `Range` category and AP/Range/CC tags. Internal ID `tfm2_valorant_raze`. No mastery or ranked artwork. Guns use Omen's credit income, prices, modifiers and engine-owned primary attack identity. Paid packs, grenades and a real destructible Boom Bot do not regenerate automatically.

| Showstopper | Prototype value |
|---|---:|
| Base cooldown, starting at accepted equip | 60 seconds |
| Equip / armed window | 0.7 / 6 seconds |
| Maximum range / speed | 240,000 / 120,000 accelerating to 600,000 units per second over 18 ticks |
| Maximum raw magic damage | 180 + 120% AP |
| Radius | 50,000 |
| Falloff | 100% centre, 62.5% halfway, 25% boundary |

The rocket commits to a ground endpoint, clamped to range and map bounds, or explodes sooner on its first swept unit/wall contact. It does not home. Falloff uses integer `floor(raw × (4R − 3d) / 4R)`. There is one damage packet per victim, with no extra direct-hit packet. Unit contact can occur at the edge of the prototype 8,000-unit collision radius, before reaching the unit centre. Self-damage remains 25% after distance falloff, before normal mitigation. Explosives exclude towers/nexuses; allies are unaffected.

Firing causes a ten-tick grounded slide at 60,000 units/second, or adds 90,000 units/second opposite the shot during flight. Air recoil permits a 300,000 speed cap and adds eighteen remaining ticks, bounded to eighty-four from the first launch. Satchel impulse is now **200,000 units/second**, with **360,000 units/second²** bounded sideways acceleration; the ordinary cap remains 240,000. Flight durations and continuous height are preserved. Roots prevent propulsion; stun ends controlled travel. Backward rocket boosts spend real inventory and the ultimate. The third-boost metric requires backward recoil, not merely a rocket after two packs.

The rocket integrates midpoint acceleration deterministically, sweeping every segment. The 240,000-unit path takes 32 simulation ticks. One live warning tracks the predicted first observable collision, uses the same accelerating ETA and displays a baked countdown phase. The warning marker and its enemy guides are removed immediately on impact/death; native point visuals renew for one tick, with no lingering long warning. Hidden units do not alter the prediction before an actual collision.

After damage, surviving enemy champions, minions and summons receive one twelve-tick outward force move. Its distance falls from 12,000 at the centre to 3,000 at the boundary. Terrain, death and control immunity are respected; objectives, buildings, allies and Raze are excluded. Raze retains her separate firing recoil.

Enemy reaction is stored per Raze instance, projectile and defender. Recognition uses a visible caster or a conservative 24,000-unit allied champion/minion/tower sight zone with clear terrain and smoke checks. Control Speed sets the initial 24–6 tick deadline once; Dodge determines recognition success. Escape choices consider arrival time, legal routes, the shot path and other hazards. The movement-only hook consumes the resulting guide without replacing attacks/casts or snapping airborne Raze. A committed engine attack/cast may prevent timely movement; this limitation remains a live acceptance item.

**Airborne protection: 15% incoming physical/magic damage reduction during actual flight.** It uses one named engine modifier, including ordinary self-blast mitigation; no post-hit healing or control immunity. Ordered transition commands activate it on takeoff and remove it on landing, flight-ending collision/control, death or cleanup. Ground recoil and cosmetic height never grant protection. Lethal self-blasts stop subsequent non-cleanup host commands in that batch.

Paint Shells remains 300 credits, one stored grenade, eight-second cooldown and 0.35-second preparation. Throw range is now **120,000**, main radius **30,000**, and four children scatter diagonally by **18,000 per axis**, each with **18,000 radius**. Main damage remains 65 + 55% AP; children remain 15 + 12.5% AP, at most two hits per target. Child landing points are clipped and committed when scatter begins; subsequent terrain changes cannot silently move warned explosions.

Packs remain 200 each, two stored, 40 HP, five-second lifetime, arming after one second for 20 + 15% AP. Their blast/launch radius is **28,000**. Throws attach to the first ground/wall/solid structure contact, never a moving unit. Each pack stores a face, normal, support identity and local offset separately from its carrier position. Moving supports carry it; destroyed/invalid supports drop it to legal ground without restoring health, arming or lifetime. An impossible drop removes it harmlessly. Steve/Herobrine publishes read-only stable construction geometry for this attachment; construction mechanics are unchanged.

Boom Bot remains 300 credits, one stored, eighteen-second cooldown, five-second lifetime, 25% owner max HP and 45 + 45% AP explosion. Failed carrier allocation restores only the failed summon commitment.

## Judge and decisions

The shared Judge curve for Raze and Omen is 300% basic damage through 10,000 units, linearly declining to 70% at 20,000. Price 1,850, 18,000 base reach, −25% attack speed and 35% secondary splash remain. Native attacks, Omen's burst estimate, Raze planning, farming attack hooks and the Lab use the shared rules. Omen's purchasing preferences and all existing body/effect pixels remain unchanged.

Raze buys/keeps Judge for affordable reachable close entries, reserving a missing launch pack before armour; open or blocked approaches retain rifle choices. Full-price replacements give no refunds. Automatic replacements require twenty percent estimated improvement and fifteen seconds since the previous weapon purchase. Manual purchases ignore athlete biases and that AI-only hysteresis.

All thirteen pinned v2 attributes remain independent. Ego prices risk; Judgement estimates value; Positioning chooses approach/landing; Control Speed schedules 24–6 tick first responses; Accuracy gives 30–100% observed-motion lead and 6,000-to-zero placement error; Dodge reacts to visible hazards; Aggressive favours offensive entries; Order prices support; Roaming favours rotations; Macro prices waves/resources; Micro controls 12–3 tick steering and one sampled detonation timing error; Awareness reviews last-sighting uncertainty; Strategy chooses a one-to-three-action continuation.

Every athlete can consider shooting, grenades, bot, paid single/double launches, rocket entries, backward rocket boosts and escape. Search retains 64 evaluated candidates, beam width eight and three macro actions. Aim variants compare visible groups, ground centres and wall impacts using the same first-contact geometry as real flight. Objective rockets require a visible assigned Morgard/Serpen secure or useful champion overlap; ordinary wave/camp reserves remain. Uncommitted stages revalidate; committed effects stay real. Hidden current positions/resources do not enter planning. Hashing is separate from combat RNG.

## Lab, artwork and compatibility

The buy panel and floating HUD reuse Omen's unchanged money, weapon and armour icon pixels, with the same purchase feedback. The panel shows exact credits, equipped gun, utility stock/prices, affordability and an Auto-buy button using the existing gun-first purchase policy. Air strafing gains another 50% acceleration (240,000 → 360,000), without changing launch impulse, speed caps, flight duration or the athlete profile. Previous `raze-3` recordings remain inspectable and explicitly require their original rules for replay.

WASD moves on the ground and supplies bounded steering in the air; arrows remain alternatives. Q places a travel-led launch pack behind actual movement, then detonates it. Shift+Q throws at mouse aim, including solid surfaces. A quick second Q queues one detonation through preparation/carrier readiness. E throws a grenade, X equips, left click/F fires, C deploys the bot. Mouse movement aims independently; diagonal ground movement is normalized. Continuous steering is separate from ability commands. Keyboard gameplay starts live manual playback only with arena focus; focus loss clears held inputs.

The Lab shows cooldowns, acceleration, attachment state, cluster landings, impact prediction, enemy reaction/escape reasons, recoil, velocity, remaining flight and 15% protection. Enemy guides produce real Lab movement; timed knockback suppresses its ordinary movement/attacks until completion. Ordinary attacks, incoming damage and mitigation remain a simplified twenty-resistance host model, not engine attribution. Recordings carry v3 and `raze-3.1`; v1/v2/different-rule recordings remain inspectable, with incompatible replay explicitly rejected.

Approved body pixels, 36×44 canvas, foot anchor, original animation names/counts/timing and both facings are preserved. The user approved the revised **10×8 visible pack on a 12×10 canvas** after requesting a smaller preview. Its ground/four-face variants use the approved compact pixels. Every old approved effect frame remains unchanged; larger-area variants are mechanically resized. Full Valorant decoded storage: **{art['decodedValorantBytes']:,} bytes**; Raze atlas **1024×883**, below both limits.

The compact carrier adds a transparent, per-entity animation pose to the Alchemist animation catalogue; every original animation value remains identical and its sheet is not overridden. Only Raze satchels request that pose. No entity invisibility or targeting changes are used. Stable API fixtures verify a real 40-HP spawned unit, damage delivery, attack suppression and owner-only pose selection. These fixtures and kit-pixel comparisons do not establish actual host rendering/targeting acceptance, which remains explicitly pending.

[Body comparison](raze-review/raze-comparison.png) · [Effects](raze-review/effects-comparison.png) · [Air protection screenshot](raze-results/raze-air-protection.png) · [Research and timestamps](raze-entry-research.md)

## Existing careers

Existing saves can embed the v2 Raze visual lists. Before a later installation, inspect a disposable copy with `rtk proxy python -X utf8 tools/migrate-champion-reworks.py PATH`. Inspection does not write. `--output NEW_PATH` creates a separate migrated copy; `--in-place` requires and verifies a complete backup before replacement. Raze refreshes only native passive/skill shells and presentation lists, preserving the gun attack, customized stats/growth, items and career history. Eleven synthetic migration tests pass, including idempotence. No real career was migrated by this release.

## Recorded verification

- Native release: **{native[0]} passed, zero failed, {native[1]} pre-existing ignored**; [raw output](raze-results/native-tests.txt).
- Shared Raze Lab: **{tests} passed**, including endpoint/range/collision/falloff, offensive entry execution, paid backward boosts, continuous height, ground recoil, protection transition ordering, manual independence and purchase safeguards.
- Three command-trace fixture pairs per attribute at 20 versus 80 with the other twelve fixed at 50; [seeds](raze-results/athlete-pairs.json).
- **{len(lab['scenarios'])} scenarios: 24 development and 12 holdout; {lab['parityFrames']:,} complete native/WASM state-and-command comparisons**, identical replay, independent sessions and 390px light/dark layouts; [Lab results](raze-results/lab.json).
- Real keyboard/mouse tests cover WASD, normalized diagonals, queued Q–Q, air steering, grenade/equip/fire, ground recoil without protection, focus loss and legacy inspection; [results](raze-results/keyboard.json).
- V3 browser fixtures additionally demonstrate Shift+Q attachment, moving support, harmless detachment, faster versus slower enemy escape movement, warning cleanup and identical replay; [results](raze-results/v3-browser.json).
- Buy-panel browser checks verify Omen money/gun pixels, icon replacement after purchase, exact credit spending, equipped-weapon highlighting, safe-shop gating and replay; [results](raze-results/shop.json) and [preview](raze-results/raze-shop.png).
- Stable API bridge tests cover a real damageable carrier without invisibility, original pose preservation, exact warning/guide ownership cleanup, lethal-hit/control-immunity knockback exclusions and stable wall identities through growth, translation and rotation. These are API fixtures, not live game observations.
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
pr=f'''Adds Raze as an AP fighter with purchased Omen weapons, paid utility, momentum strafing and all thirteen athlete attributes. This update adds compact sticky packs, stronger movement and a broader, lower-damage accelerating rocket with enemy counterplay.

- Satchels: approved **10×8 visible pixels**, ground/four-face attachment, destructible 40-HP carriers, stable construction identities and harmless drops. Launch **200,000 units/s**, air acceleration **360,000 units/s²**, blast/launch radius **28,000**; speed caps and flight duration unchanged.
- Paint Shells: **120,000 throw range**, **30,000 main radius**, four committed diagonal clusters at ±18,000 per axis with **18,000 radius**. Damage, fuses and two-child-hit cap unchanged.
- Rocket: **60s cooldown, 240,000 range, 120,000 → 600,000 units/s over 0.3s, 180 + 120% AP, 50,000 radius**. Fixed endpoint/first swept contact; existing centre-to-edge falloff and self-damage. Surviving eligible enemies are pushed outward over twelve ticks.
- One impact warning and countdown follows the observable collision prediction. Stock/custom enemies receive once-per-projectile athlete reaction and legal movement-only escape guidance; no cast/attack injection or hidden-target prediction.
- Ground recoil is separate from flight. Two packs plus a real backward rocket can create a third boost; protection ends on landing/control/death and does not grant immunity.
- Shared Raze/Omen Judge: **3× damage through 10,000**, falling to 0.7× at 20,000; existing price/range/speed/splash. Raze chooses Judge for viable entries and rifles for open approaches, with paid-swap safeguards.
- Omen's original money/gun/armour icons now render above Raze and in a buy panel; exact balance, inventory, prices and safe-shop state reflect the shared core. Native purchase feedback uses the same Omen effect.
- **Lab v3: WASD, Q–Q, Shift+Q surface throw, E, X, mouse/F fire**, attachment/support fixtures, acceleration, enemy movement, knockback and escape explanations. Legacy recordings remain inspectable without silent rule changes.
- All original Alchemist animation values and sheet pixels are retained; an additional transparent pose is requested only by Raze satchel entities. No entity invisibility. Native carrier appearance/targeting still needs real-game acceptance. Main's Levi jungle fix is merged.

Native **{package['native']}**, champion pack **{package['packs']['tfm2_custom']}**, Valorant **{package['packs']['tfm2_valorant']}**.

### Verification
- Native **{native[0]} passed / {native[1]} existing ignored**; Lab **{tests} passed**.
- **{lab['parityFrames']:,} native/WASM parity frames**, 24 development + 12 holdout scenarios, independent replay/sessions, real keyboard/mouse tests and three low/high comparisons per athlete attribute.
- Five core runs: p95 **{perf['medianP95Ms']:.4f} ms**, p99 **{perf['medianP99Ms']:.4f} ms**; Magnus 128-piece p95 **{min(mp):.3f}–{max(mp):.3f} ms**.
- Reproducible art; {art['decodedValorantBytes']:,} decoded bytes. Two DLL builds match. Isolated manager verification plus all-pack SHA-256 checks: **{manager['allPackFilesCompared']} files, zero differences**.

DLL SHA-256: `{sha}`

![Airborne protection and keyboard Lab](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/raze-air-strafe/docs/raze-results/raze-air-protection.png)

![Approved smaller satchel](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/raze-air-strafe/docs/raze-review/sticky-satchel/comparison.png)

![Rocket impact warning and enemy response](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/raze-air-strafe/docs/raze-results/raze-rocket-warning-80-true.png)

![Omen-style money, gun icons and buy panel](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/raze-air-strafe/docs/raze-results/raze-shop.png)

[Implementation/results/limits](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-implementation.md) · [Timestamped research](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-entry-research.md) · [Approved body](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-review/raze-comparison.png) · [Effects](https://github.com/PowerViber/tfm2_mods/blob/codex/raze-air-strafe/docs/raze-review/effects-comparison.png)

**Draft — live-game checks remain pending.** Native targeting, protection/mitigation stacking, shields/items, attribution, warning tails and crowded host/rendering need observation. Lab damage is a simplified estimate; short harness counts are not a win-rate or balance proof. Projectile visibility/collision use documented conservative approximations. **Nothing installed; no real career migrated; original local work preserved.**
'''
(root/'backups/raze-pr-body.md').write_text(pr,encoding='utf8')
print(json.dumps({'nativeTests':native[0],'labTests':tests,'dll':sha,'totals':totals}))
