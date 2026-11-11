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

## Wright (`wright-6`)
Reihan: the ult spawned so much that no AI could react, and it one-tapped.
- **Before:** the carrier launched a jet every 1 s (up to 6 on top of his own 4), repair choppers, and up to 3 ready bombers with a 0.3 s mark.
- **Now:** **the Sky Carrier launches nothing.** It still drops on him (knockback, no damage), follows him and takes 80% of his damage for 10 s. While it's up, it **empowers his skills**:
  - no airframe cost, and half the cooldown;
  - jets rearm 2× faster with +1 strike, still capped at 4 jets;
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
