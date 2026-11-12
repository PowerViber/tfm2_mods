# Round 165: Akane buff, Wright's carrier powers his skills, Thalassa's tsunami bolt (native 0.10.99, tfm2_custom 0.2.102)

## Akane (`akane-1.2`)
Reihan wanted a stack of her own on her basic attacks; attacking and casting inside the invulnerability, where a 2nd basic attack ends it; and higher damage when it ends.
- **Blood Rhythm (new passive):**
  - Each basic attack on a champion, minion or creep adds a stack, up to 5, for 4 s.
  - Each stack is +6% basic-attack damage, so up to +30%.
- **Dismiss & Devour, reworked:**
  - Still 1.5 s of invulnerability, +60% speed and the heal.
  - She can **cast skills** now. Her 1st basic attack is allowed; the 2nd ends it.
  - She's **invisible until she acts**: a cast or a basic attack reveals her.
- **Ambush:** when Dismiss ends (naturally or on the 2nd hit), +20% damage for 3 s, on her skills and her basic attacks.
- **Her AI:** inside Dismiss she now engages a target she can kill with Crimson Edge or the Kunai, counting the Ambush. Otherwise she slips away.
- **Level 10:** her basic attack goes from about 180 to about 234 at 5 stacks, and about 281 with Ambush.

### Akane wave clear (`akane-1.3`, added before merge)
Reihan: she can't clear waves fast, with no area damage skill. He picked three of the ideas:
- **Rhythm cleave:** at 3+ Blood Rhythm stacks, her basic attacks cleave enemies within 12,000 of the target: 80% of the hit to minions and creeps, 40% to champions. Never towers.
- **The Kunai pierces:**
  - it passes through up to 4 minions or creeps, damaging each;
  - it stops at the first champion, who is marked;
  - with no champion, it marks the last one it went through (the recast still crosses walls on a camp).
  - The AI only needs no *other champion* in front, not a clear line.
- **Crimson Edge splash:**
  - each of the 5 empowered hits also deals 50% to enemies within 14,000 of the target;
  - on a minion or creep the 4th hit always bursts; champions still need the 4 stacks;
  - the splash deals damage only, never the slow, vulnerable or knock-up.
- **Tests:** 3 new, and the kunai test is updated (a minion in front is now pierced). 1,200 native tests.

## Wright (`wright-6`)
Reihan: the ult spawned so much that no AI could react, and it one-tapped.
- **Before:** the carrier launched a jet every 1 s (up to 6 on top of his own 4), repair choppers, and up to 3 ready bombers with a 0.3 s mark.
- **Now:** **the Sky Carrier launches nothing.** It still drops on him (knockback, no damage), follows him and takes 80% of his damage for 10 s. While it's up, it **empowers his skills**:
  - no airframe cost, and half the cooldown;
  - jets rearm 2Ã— faster with +1 strike, still capped at 4 jets;
  - Carpet Bomber explosions deal 125% (Lab note below);
  - repair kits are 150%.
- **One change from the plan:** the bomber keeps its **original size**. The plan said +20% radius, but the warning art is drawn at 30,000. A bigger explosion than its warning would hit people outside the circle they were shown, and readability was the whole point. If you want the bigger size, Codex has to redraw the warnings.

## Thalassa (`thalassa-1.6`)
Reihan: "the bounce is too much", "make her BA slower but [it] does an area rectangle, a small tsunami with a knock-up".
- **No more water bounce.**
- **Basic attack:** one every 2 s instead of every 1.3 s.
  - The bolt still hits its target, towers included.
  - A tsunami sweeps from her through the target (8,000 past it, 14,000 wide). Other enemies in it take 25 + 35% AP.
  - Everyone in it is knocked up for 0.35 s, each at most once every 3 s. Never towers or allies.
- **Clones:** her Twin and the Trident clone copy single bolts only, with no wave.
- **Not done:** the plan's "aim bias" for her AI. The game engine picks which target her basic attacks go to, so her AI can't steer the wave without overriding the engine's attack choice. I left it out.
- **Art:** the wave reuses the `surge` and `bump` effects until Codex draws a tsunami (`docs/thalassa-art-handoff.md`).

## Verification
- `cargo test --release`: **1,197 passed**, 0 failed, 0 warnings.
  - **Akane:** 6 new or rewritten tests: the stacks, one hit allowed, the end on the 2nd hit, Ambush on skills and basics, the natural end, and the bridge's hiding, reveal and damage scaling.
  - **Wright:** 2 rewritten: the carrier spawns nothing and doesn't hurt allies or turrets, the 4-jet cap holds, and the empowerment covers no airframe, half the cooldown, the full 1 s warning, 125% bombs and +1 strike.
  - **Thalassa:** 2 new: the rectangle in, beside and past the target, the knock-up lock, no towers or allies, no wave from a bolt on a tower.
- **Lab crates:** Thalassa 44, Wright 28 and Akane 2 tests pass. The `thalassa`, `wright` and `akane` wasm are rebuilt.
- **Browser (Chromium copies):** the Wright Lab's 12 checks and the Thalassa Lab's 10 checks pass.
  - Akane's Lab test is updated: casting during Dismiss now reveals her. It needs the local game-reference cache, so it runs on Reihan's PC.
  - `verify-thalassa-art.py` now expects `thalassa-1.6`.
- **Generators:** akane, wright, thalassa and roster are current. The Mod Manager load test passes.
- **The Windows DLL** builds with 0 warnings and reports 0.10.99.

## Live acceptance (pending)
- **Akane:** that the reveal and the end of Dismiss really work in the game.
- **Wright:** that the ult reads as fair now.
- **Thalassa:** that the knock-ups don't chain; the 3 s lock should stop that.

## Codex approved-art follow-up (2026-11-12)

Reihan approved the corrected Round 165 effects. The tsunami is **water-only** (wet/Dive included), with prototype owner water range **82,500** and unchanged **120-tick** basic cadence; land stays **55,000** and clone range stays +20%. Akane has Rhythm/Ambush/Dismiss indicators. Wright has a carrier-empowerment indicator and corrected **30,000-radius / 60/96/132-tick** bomber warnings. Claude’s newer **akane-1.3** changes are included.

See [approved sources, previews, budgets, reproduction and separate evidence](round-165-art-review/README.md). Final Windows suite: **1,202 passed, 13 ignored**; four inherited test-fixture dead-code warnings. All three existing browser suites and the new 23-check art suite pass; complete native/WASM traces match. All 252 old Thalassa FX frames retain exact pixels/timing, and presentation stays below 32 MiB per owner. The rebuilt 0.10.99 DLL reproduces byte-for-byte; isolated Mod Manager verification passes. No installation or real-career migration was performed. Actual-game acceptance remains pending.

## Crimson Night explosion (`akane-1.4`, added after Codex's art follow-up)
Reihan: the ult's dash should trigger the 4th-hit crimson explosion, as a lesser burst with fear.
- **The slash sets off a lesser crimson explosion:**
  - the target takes **50%** of Crimson Edge's burst (**4% max HP + 20% Attack**);
  - enemies within **18,000** take half of that;
  - everyone hit gets the crimson bleed and is **feared for 0.75 s**, running away from her (`akn_fear`, speed 700).
- **Unchanged:** the target's bleed stacks are kept (only the 4th empowered hit consumes them), and the Kunai recast slash doesn't explode.
- **Never hits:** towers or allies.
- **V1 parry:** a parried slash sets off nothing, and its crimson bleed never ticks.
- **Art:** it reuses Codex's approved `akn_burst`, so no new art is needed. The Lab shows feared units running away.
- **Decision:** Reihan keeps Codex's `thalassa-1.7` (water-only tsunami, 82,500 water range).
- **Verification:**
  - **Tests:** `cargo test --release` gives **1,206 passed**, 13 ignored, 0 warnings. That includes 2 new tests: the core explosion, and the glue fear direction with the parry cancel.
  - **Lab:** the Akane Lab crate's kunai test is updated (it now pierces the minion and marks the champion behind it). The `akane` wasm is rebuilt.
  - **Generators and Mod Manager:** the generators are current, and the Mod Manager load test passes.
  - **DLL:** the Windows DLL is rebuilt and reports 0.10.99. Codex's `round-165-art-review/package.json` describes the earlier `akane-1.3` DLL; I verified that one by source hash before this change.
  - **Not run here:** the Akane browser test, which needs msedge and the local game-reference cache.

## Load-time check (2026-11-12)
Reihan: the game launch, and mostly Proceed, felt slower than in round 164. Nothing in round 165 measures heavier.
- **Native per-tick cost:** `bench_tick_cost` (ignored tests, run with `cargo test --release bench_tick_cost -- --ignored --nocapture`) runs each core on a crowded 5v5 with 12 minions for 36,000 ticks with her AI on.

  | Champion | Round 164 (1d119b6) | Round 165 |
  |---|---|---|
  | Akane | 1,473 ns/tick | **976 ns/tick** |
  | Thalassa | 713 ns/tick | **335 ns/tick** |
  | Wright | 1,793 ns/tick | **1,033 ns/tick** |

  All three are about 1 microsecond per tick, so they are nowhere near the cost of a Proceed.
- **Startup data:**
  - the mod text grows by 1.4% (+160 KB);
  - there are 302 more view definitions (19,296 in all);
  - decoded VFX grows by 0.6% (896 to 902 MiB);
  - the DLL imports and exports are identical, and `init()` and the server hooks are unchanged.
- **The tsunami art plays at most 2 segments per bolt** at the 82,500 water range, so it isn't an effect flood.
- **Gundam review images:** `mods/tfm2_gundam/source` review PNGs are never installed (that folder has no mod_info).
- **Likely cause:** the first launch after an update runs cold. Windows rescans the new 11 MB DLL and the replaced files are no longer in the disk cache. To confirm, time a second launch, or put `perf.flag` in `mods/tfm2_custom_ai` and send `perf_log.txt` after a Proceed.
