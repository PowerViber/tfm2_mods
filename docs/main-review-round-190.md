# Round 190: Contender 3.6 is back, with Codex's Shadow Grab art (native 0.10.126, tfm2_custom 0.2.129)

**Reihan reverted 3.6** ([#106](https://github.com/PowerViber/tfm2_mods/pull/106)) to wait for Codex's S3 art. Codex then pushed `codex/contender-grab-art` (`4f0b5c5`): four violet stills (reaching hand, latch, pull ribbon, upward burst) and an animated review page over the recorded 3.6 cast. Reihan approved the art, so 3.6 comes back exactly as merged in `c522b41` and is now drawn with Codex's effects.

## How the preview became game art
The game can't stretch, rotate or fade a view effect at runtime. An effect plays fixed frames from a sheet at a point, and the bridge renews it. So `tools/build-contender-grab-art.py` (with `--check`) bakes the review page's Canvas moves into `mods/tfm2_custom/vfx/contender_grab#sheet.png` / `#anim.fanim`. It uses Codex's crop boxes, game sizes, offsets and fades from `editor/contender-grab-review.html` (1 px = 1,000 units):

| Effect | Baked as | Played |
|---|---|---|
| Reaching hand (32×16) | `hand_d0..7`: 8 headings; left headings are mirrored, as the preview's left facing, never upside down | every 4 ticks during the flight, 14 px ahead of the flying shadow and 3 px up, beside the existing shadow body |
| Latch (24×20) | `latch`: 4 frames over 8 ticks, opacity 1 − t/12 | once on the grabbed enemy at contact (`GRAB_LATCH_TICKS`) |
| Pull ribbon (58×17, 13 px high when pulling) | `ribbon_L{2..9}_d{0..7}`: stretched to each 10,000 distance from `pullStop` (20,000) to `range` (90,000) × 8 headings | every 2 ticks during the 0.2 s pull, re-picked from Contender's and the target's **current** positions, so it stays connected and shortens as the target is dragged in |
| Upward burst (21×39) | `burst`: 6 frames over 18 ticks, opacity 1 − t/21 | once at the target when the knock-up starts (`GRAB_BURST_TICKS`) |

- **The ribbon:** it starts 10 px in front of Contender and is 6 px longer than the gap, as in the preview. It picks the length bucket nearest the live distance (within 5,000) and the eighth-turn heading toward the target.
- **A miss:** only the flying shadow and hand show, with no latch, ribbon or burst.
- **Bindings:** the generator binds every packed tag as `ct_grab_*`, read straight from the `.fanim`, so the bindings can't drift. The packer reads `pullStop` and `range` from `rules/contender.json`, so a balance change re-bakes the ribbon.
- **Lab:** it draws the same packed sheet. The Lab needs the local game-world extraction, so it couldn't run in this container. `docs/contender-grab-art-preview/packed-in-game-check.png` instead renders the packed frames, placed exactly as the bridge places them, over Codex's recorded hit and miss traces at both facings.

## Checks
- **Native:** 1,421 passed, 19 ignored. New tests:
  - a real engine-cast grab plays hand → latch → a renewed ribbon → burst, with the latch and burst once each;
  - a miss plays none of them;
  - every hand and ribbon name the helpers can pick, over 32 directions × every distance, is bound;
  - the latch and burst fade lengths match the `.fanim`;
  - the ribbon shortens and follows the target.
- **Lab, Mod Manager and generators:** 3 Lab tests and 15 Mod Manager tests pass. `build-contender-grab-art`, `generate-contender`, `generate-roster` and `generate-lab-targets` pass `--check`.
- **Builds:** the DLL contains 0.10.126, and the wasm is rebuilt.
- **Live only:** how the effects look in the game's renderer, especially the ribbon's 2-tick renewal during the 0.2 s pull.
