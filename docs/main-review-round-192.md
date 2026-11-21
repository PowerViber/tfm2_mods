# Round 192: Contender 4.0, the Shadow Boxer rebuilt (native 0.10.128, tfm2_custom 0.2.131)

**Reihan:**
> "idk its hard to make him a good champ, lets start from 0, remove everything, but make the merged skill as a passive 'shadowborne' - any flurry hit at the last enemy, he can do an area knock up(s) and tp, keep the pull ... i dont want a flurry to be a skill"

He designed the rest over the follow-ups:
- the shadow pool and dodges stay (buffed, with Counterpunch);
- Rhythm is capped at 10;
- the grab is S1;
- S2 is a shadow anchor ("press press auto ... the shadow drop does a flurry ... before a flurry the shadow fears around him 50k for 1s");
- S3 is Smother ("shred haste shield");
- the ult is Clone Army ("3 shadow clones in front of him 50k and right of the shadow and left, like Minato kunais, and the placed shadow does that too, and all shadows mimic s1 s3");
- shadows' flurries knock up but never teleport.

## The kit (`contender-4.0`)
| | Rules |
|---|---|
| **Shadows** (passive) | 10 shadows (5 pips), one back every **3 s**, faster per landed hit while low. An enemy champion's hit spends one to be dodged (0.4 s gap), and the dodge shadow counterpunches (20 + 30% Attack, 45,000). |
| **Shadow Rhythm** (passive) | His hits stack an enemy up to **10** for 4 s; each stack means +1.5% damage taken from him. |
| **Shadowborne** (passive) | Every shadow flurry (30 punches over 1.5 s, 2 + 7% Attack each, from 50,000 and chaining within 30,000) ends on its last punch with a **1 s knock-up of every enemy within 60,000** of that enemy. Contender's own flurries also **teleport him behind it**. |
| **S1 Shadow Grab** | 12 s cooldown, 150,000 range, through walls. A hit (20 + 60%) drags the target to the meeting point; he dashes 3/8 of the range, and the Haymaker (15 + 20%) knocks up for 2 s. A miss: a shadow flurry where it lands. |
| **S2 Shadow Anchor** | 14 s cooldown. Drops a shadow ≤80,000 away, which **fears within 50,000 for 1 s**, then flurries. It lasts 8 s: a **second press** or **HP ≤ 30%** teleports him there. |
| **S3 Smother** (self-cast, level 3) | 14 s cooldown, a 70,000 × 10,000 shadow line at the nearest enemy champion. Enemies hit take 20 + 40%, are silenced 1.5 s and slowed 30% for 2 s, and are **Smothered** (−20% Armour, −15% attack speed) for 4 s. He gains +20% attack speed and +15% move speed and an **8% max HP shield per champion hit (≤24%)** for 4 s, then flurries. |
| **Clone Army** (ult) | 80 s cooldown; refills all shadows. Three clones appear 50,000 ahead and 30,000 to each side (the kunai fan), and the anchor sends three more. For 6 s **every shadow (anchor and clones) mimics his grab and Smother** from where it stands, at 50% damage and full CC. |

## Notes
- **Removed:** Shadow Merge, the Flurry skill, Pact echoes, Footwork, Body Shots, Shadow Break, lifesteal, Overflow and the 3.x Lab AI.
- **The tower guard (3.7)** stays for walks and basic attacks; his ranged skills are never blocked.
- **Slots:** the game's AI presses S1, S2 and the ult through cast markers that carry the aim. S3 has no engine button, so the core casts it itself. The S2 slot is 30 ticks so it can be pressed twice; the core enforces the 14 s cooldown, and press gating turns an S2 press during the cooldown into a basic attack.
- **Mimicry:** each enemy can be held by one grab at a time; a shadow's grab drags the target to that shadow, which steps in and Haymakers it.
- **The anchor's flurry** reaches its fear radius farther (100,000) for its first punch, since feared enemies run.
- **Art:**
  - every grab uses Codex's hand, latch, ribbon and burst, and the flurries use the directional cues;
  - the anchor keeps the dark shadow body;
  - **Smother and Clone Army use Codex's art** (see below).

## Checks
- **Native:** 1,400 passed, 18 ignored. The Contender suite was rewritten for 4.0 (84 tests), and every number comes from `d::`.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** `build-contender-grab-art`, `generate-contender`, `generate-roster`, `generate-lab-targets` and `build-contender-merge-art` pass `--check`.
- **Builds:** the DLL contains 0.10.128, and the wasm is rebuilt.
- **Live only:** how often the game AI presses the anchor twice; whether placing him (anchor, Shadowborne) looks clean in the engine; and Clone Army's 7 casters at once.

## Review changes on PR #112 (Reihan)
> "the ult kinda sucks, make the shadow be able to run and BA but when contender skills, it mimics, also lets not use the new art, but make the art as the shadow before - ult spawn range much longer"

> "add another passive where he charges his punches - at 4 punch and above he punches 2x faster and the stack goes 2 per punch (lingers), and does a little pull and lifesteal - apply with every skill and overheal is kept, and tp back from shadow heals and gives armor"

- **Clone Army clones fight on their own.**
  - Each clone runs (1,050 a tick, through walls) at the nearest enemy within 100,000, champions first and never towers. It punches every basic cooldown for 50% of his Attack (Rhythm-scaled, adding a stack, `HitSource::Clone`).
  - With nobody near, it returns to within 40,000 of him.
  - When he casts S1 or S3, every clone (and the anchor) still casts it from where it stands now.
  - The front clone now spawns **150,000** ahead (was 50,000), and the ult slot's range is 150,000.
- **Charged Punches (new passive):**
  - **Charges:** each of his own hits (basics, Counterpunch, grab, Haymaker, Smother, his flurry punches) adds a charge, up to 10. They last 3 s after his last hit. Shadows' hits don't count.
  - **At 4 or more he's Charged:**
    - +100% attack speed (a `ct_charged` buff);
    - 2 Rhythm stacks per hit;
    - a 5,000 pull toward him (never towers or CC-immune units);
    - 15% lifesteal (for a basic, 15% of his Rhythm-scaled Attack).
- **Overflow returns:** lifesteal past max HP becomes a shield of up to 15% of max HP, in 4 s layers. It covers Charged lifesteal and the anchor heal, plus item and buff vamp spill on his basics.
- **Teleporting to the anchor** (the second press or 30% HP) heals 15% of max HP (spilling into Overflow) and gives **+30% Armour for 3 s** (`ct_anchor_guard`).
- **Art:** back to the shadow body Reihan approved before.
  - **Clones:** `ct_body_run`, `ct_body_punch` or `ct_body_idle0`, facing their way.
  - **Smother:** the line's front is a running shadow, and the contact uses `ct_echo`.
  - **Removed from the game:** Codex's 4.0 sheet, its packer and its bindings. His preview stays in `docs/contender-40-art-preview/`.
  - **Kept:** the pixel-compare art check helper (`tools/contender_art_pack.py`), still used by the Grab art.
