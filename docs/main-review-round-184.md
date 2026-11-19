# Round 184: ship Contender 3.1.x to the game (native 0.10.119, tfm2_custom 0.2.122, `contender-3.1.3`)

**Reihan:**
> "check from PR 96 above whats changed and update the game following that change … i checked the live gameplay, but codex dont update it in game"

Also: "any changes made on the skill … the AI of that champ must change too … and how the ai stats change how this champ be played".

## What changed after #96 (Codex)
- **#97, 3.0:** Shadow Link, and safer fight selection (health, support, threats, terrain, towers, an exit).
- **#98, 3.1:**
  - **Counterpunch:** a passive dodge strikes back.
  - **Merge:** its cross follows him for 6 s.
  - **S2 is Shadow Flurry:** 10 punches in 1 s, 3 charges, a 16 s recharge.
  - **Shadow Link (ult):** refills the 5 dodges, then pulls, knocks up and stuns.
  - **Retired:** Cyclone, recasts and the S3 slot.
- **#99, 3.1.1 / 3.1.2:**
  - a dodge can be traded for a Flurry charge;
  - the Cross radius is 50,000;
  - hits shorten the Flurry recharge.

## Why the game still played 2.9
- **What Codex updated:** the source and the champion data (data_champion, tooltips, wasm).
- **What it left behind:** it never rebuilt the shipped DLL (`mods/tfm2_custom_ai/tfm2_custom_ai.dll`, last built in 2.9's `88fe62a`) and never bumped the versions (still 0.10.118 / 0.2.121). #98 says so: "the prior packaged DLL and manifest versions remain".
- **What the Mod Manager does:** Update builds the DLL only when Rust is installed; otherwise it installs the committed DLL.
- **The result:** the game ran the 3.1.2 champion data on the 2.9 native code.
- **Codex's local install:** its "installed locally with a rollback backup" was on its own machine, so it never reached Reihan's game.

## This round
- **Packaging:**
  - the DLL is rebuilt from the 3.1.x source (the string `0.10.119` is inside it);
  - native **0.10.119** and pack **0.2.122** (dependency `>=0.10.119`);
  - the wasm is rebuilt from the same core.
- **AI (`contender-3.1.3`):**
  - **S2 Shadow Flurry farms** calm waves and big camps, with real charges only. This is the skills-on-creeps rule: before, he farmed with Merge alone.
  - **Judgement, Skill Accuracy, Micro and Control Speed** now shape his play (table in `docs/contender-guide.md`). Before, only Aggressive did.
  - At the default 50 for every stat he plays exactly as Codex's 3.1.2.
- **Balance (measured, flagged to Reihan):** his own AI, 1v2, 20 s, against a plain champion's basic attacks.

  | | Level 3 | Level 9 | Dodged |
  |---|---|---|---|
  | Codex 3.1.2 (any athlete) | 1.73× | 1.65× | 14% |
  | 3.1.3, athlete 20 | 1.79× | 1.70× | 14% |
  | 3.1.3, athlete 50 | 1.73× | 1.65× | 14% |
  | 3.1.3, athlete 80 | 1.66× | 1.59× | 18% |

  2.8 / 2.9 sat at about 1.3×, so the 3.x redesign is stronger. The balance is shipped as Codex designed it. A guard test keeps it at or below 1.9× and 25% dodged, so it can't creep further; Reihan decides whether to bring it back to about 1.3×.

## Checks
- **Native tests:** 1,417 passed, 18 ignored. New tests:
  - Micro's dodge reserve;
  - Control Speed's reaction delay;
  - Judgement's retreat line;
  - Skill Accuracy's single-champion Link;
  - Flurry farming without trading a dodge;
  - the ported 1v2 guard at athletes 20/50/80.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** `generate-contender`, `generate-roster` and `generate-lab-targets` are current.
- **Not run here:** Codex's Windows parity, the browser checks and the live match.
