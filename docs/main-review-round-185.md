# Round 185: the game's own AI casts Contender (native 0.10.120, tfm2_custom 0.2.123, `contender-3.2`)

**Reihan:** "nevermind the AI you made sucks, revert back to the basics, since its own game already has an ai".

## What changed
- **Round 184's AI is reverted:**
  - the athlete-stat hooks (Judgement, Skill Accuracy, Micro, Control Speed);
  - Flurry farming;
  - their rules keys and tests;
  - the custom-AI fight guard.
- **The game's AI casts his skills.** His three data slots were never-castable placeholders. They are now real engine skills, and each Native effect only drops a cast marker (Isliid's pattern):
  - **S1 Merge:** 840 ticks, range 60,000, `contender_s1`;
  - **S2 Flurry:** 45 ticks, range 50,000, `contender_s2`;
  - **Ult Link:** 4,500 ticks, range 60,000, enemy champion, `contender_ult`.
- **The bridge:** it consumes the marker and runs the core in its existing manual mode (order in, no `decide()`, no steering, no dodge trade).
- **The core still enforces:** the Merge and Link timers, the 3 Flurry charges, and the level unlocks.
- **Steering removed:** the `_contender` `ct_go` / `ct_hold` steering in `steve.rs` is gone, along with the two helpers it used. The engine moves him.
- **The Flurry unlocks at level 3,** because of the engine's skill2 lock (it was level 2).
- **Lab:** the AI checkbox is now "Lab AI (not used in game)", and manual keys still work.

## Checks
- **Native tests:** 1,414 passed, 18 ignored. New tests:
  - no native decision, cast or steering happens without the engine;
  - an engine S1 marker starts Merge on the next update, and an S2 marker spends a Flurry charge;
  - a no-charge press is refused and never trades a dodge;
  - the ult marker carries the engine's aim;
  - the data slots are real engine skills with `Native` effects.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the DLL contains 0.10.120, and the wasm is rebuilt.
- **Not verified here:** how well the game's AI uses these slots in a real match. Only a live game shows that.
