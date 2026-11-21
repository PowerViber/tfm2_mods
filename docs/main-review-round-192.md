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

## Codex's Smother and Clone Army art (in PR #112)
Codex previewed the art in `docs/contender-40-art-preview/` and `editor/contender-40-review.html`. `tools/build-contender-40-art.py` (with `--check`) bakes the preview's Canvas moves into `mods/tfm2_custom/vfx/contender_40#sheet.png` and `#anim.fanim`, bound as `ct_40_*`.

| Effect | Baked as | Played |
|---|---|---|
| Smother trail | `trail_L{1..7}_d{0..7}`: stretched to every 10,000 up to the range, 20 px wide (2 × `smother.width`), in 8 headings; `trail_s_…` at a shadow's 0.7 alpha | every 2 ticks from the cast origin to the line's front (nearest length, heading, midpoint) |
| Smother contact | `contact`: 24×20, fading 1 − 0.8 t over 12 ticks (`SMOTHER_CONTACT_TICKS`) | once on each enemy hit |
| Clone | `clone_in` (0.55 → 0.8 alpha over 8 ticks) then `clone`, 30×44; `_l` faces left | every 4 ticks at each clone, facing the nearest enemy champion |

- **The art checks** (`build-contender-40-art.py`, `build-contender-grab-art.py`, and the shared `tools/contender_art_pack.py`) now compare decoded pixels and animation data instead of PNG bytes, so they pass on Windows too (Codex saw a byte mismatch there).
- **Visual check:** `docs/contender-40-art-preview/packed-in-game-check.png` renders the packed frames, placed as the bridge places them, over 4.0 WASM casts.
- **Tests:**
  - every trail the bridge can pick (32 directions × all lengths × Contender or shadow) is bound;
  - the contact and fade-in lengths match the core;
  - a host run plays clone fade-in → clone, his trail, the shadows' paler trails, the contact, and the anchor's shadow body.
