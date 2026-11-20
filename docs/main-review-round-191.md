# Round 191: Contender 3.7, no tower dives, a 150,000 grab through walls into a dash and Haymaker, Flurry lifesteal and Overflow (native 0.10.127, tfm2_custom 0.2.130)

**Reihan:**
> "contenders Ai tends to push anywhere he goes even in turret, also buff his pull range greatly, its too short and can pass through wall, flurry lifesteal and add a passive where he can overheal from any lifesetal with max 15% armor, after a pull he dashes a little(3/8 of the pull range) to the pulled champ delivering a haymaker knock up for 2s"

Then: "if S2 don't hit, it spawns a shadow there and does 1 flurry".

**His choices:** range 150,000; the grab passes through walls; Flurry lifesteal 25%; overheal becomes a shield capped at 15% of max HP; the dash + Haymaker replaces the 1 s knock-up; on a **missed grab** the shadow throws one Flurry.

## Changes (`contender-3.7`)
- **No tower dives.** The shared input layer (`steve.rs`, `contender_tower_guard`) rewrites any engine Move, Attack, S1 or S2 order that would take him into an enemy tower's range (`ai.towerRange`). It becomes a walk to just outside that range, and he walks out if he's already inside.
  - **Allowed:** diving when one of his minions is inside that tower's range.
  - **Never blocked:** the grab (his ult).
  - **Still the engine's choice:** where he fights and pushes.
- **Shadow Grab** reaches **150,000** (`flightTicks` 40 keeps the shadow's speed). It **passes through walls**: the flight is straight, and the target is dragged through terrain and Steve's walls. The bridge places it each tick (`Effect::Drag`) instead of using an engine force-move.
- **Dash and Haymaker** (replacing the 1 s knock-up):
  - the pull leaves the target `GRAB_MEET` = dash + Haymaker reach (56,250 + 15,000) from him, at a point he can reach;
  - he dashes up to **3/8 of the range** (56,250) toward it while it's held, then the Haymaker hits for 15 + 20% Attack (Rhythm-scaled, building a stack) and **knocks up for 2 s**;
  - a close grab skips the pull and dashes only the gap;
  - a wall stopping the dash short means no Haymaker.
- **A missed grab** leaves the shadow where it landed, throwing **one full Flurry** (up to 30 punches, 2 + 7% Attack each) on enemies within 50,000 of it.
  - It costs nothing, moves nothing and gives no lifesteal.
  - It uses a new `HitSource::ShadowFlurry`, which isn't a landed hit.
- **Flurry lifesteal:** each punch heals **25%** of its damage (the native amount, before the target's Armour).
- **Overflow (new passive):** lifesteal past his max HP becomes a **shield**, topped up to at most **15% of his max HP** (counting his current shield), with each layer lasting 4 s.
  - It covers the Flurry's lifesteal, and item and buff vamp on his basic attacks; the engine heals the part that fits.
  - The engine's own vamp on native skill damage isn't visible to the mod, so only basics are covered there.
- **Art and Lab:**
  - Codex's grab art re-bakes for the new pull distances (ribbon lengths 70,000–150,000).
  - The missed grab's shadow stands where it landed and uses the directional Flurry cues.
  - The Lab shows the dash, the Haymaker, the shadow Flurry, the lifesteal and the Overflow shield.

## Checks
- **Native:** 1,426 passed, 19 ignored. New tests:
  - a grab through a wall at 140,000 → meeting point → dash ≤ 3/8 range → Haymaker 2 s knock-up, with no 1 s knock-up;
  - a close grab dashes only the gap;
  - a wall stops the dash, so there's no Haymaker;
  - a missed grab's shadow Flurry: no move, cost or lifesteal; nobody near means nothing happens;
  - every Flurry punch's lifesteal;
  - Overflow heals first, then shields up to the cap;
  - the tower guard (uncovered, covered, outside, already inside).
- **Lab, Mod Manager and generators:** 3 Lab tests and 15 Mod Manager tests pass. `build-contender-grab-art`, `generate-contender`, `generate-roster` and `generate-lab-targets` pass `--check`.
- **Builds:** the DLL contains 0.10.127, and the wasm is rebuilt.
- **Live only:**
  - how the engine handles placing a target through a wall (the meeting point is always open ground);
  - whether the engine glide always lands the dash on time (the Haymaker also counts reach from the dash's aim point).
