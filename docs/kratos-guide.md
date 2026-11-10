# Kratos, the God of War

Kratos is a **God of War**-family **engage bruiser (diver)**, designed by Reihan (Nov 10–11). He plays top, jungle or support (not bot, and not really mid): he dives, survives and locks people down, but doesn't one-shot. His damage is physical and scales on Attack; he has no mastery, the bruiser Mod Power line `[15, 0, 5, 10, 10, 0, 5]`, and he keeps Flash. Every number lives in `rules/kratos.json` (revision `kratos-1`); these are starting values to tune in the Lab and in play.

## Kit
| | Skill | What it does |
|---|---|---|
| Passive | **Frost & Flame + Spartan Armor** | He holds the **Leviathan Axe** (frost) or the **Blades of Chaos** (fire), starting with the axe. **He changes weapons only through S3** (Spartan Guard → axe, Chain Dash → Blades); the Rampage uses both but leaves him on the one he had.<br>**Axe:** each hit adds a Frost stack (up to 3, 4 s); at 3 the target **freezes 0.6 s**, then can't freeze again for 3 s.<br>**Blades:** each hit **burns** 8 + 15% Attack a second for 3 s (refreshes, never stacks); his basic attacks reach **30,000** instead of 22,000.<br>**Spartan Armor:** every weapon swap gives a **shield of 5% of his max HP** for 5 s (the white bar over his HP), **at most 10%** from swaps at once. It isn't the armor stat or damage reduction. |
| S1 | **Heimdall's Sight** (13 s) | **1.5 s** of **CC immunity**, and he **dodges the first hit from each enemy champion** (no damage). He stays visible and targetable. |
| S2 | **Leviathan Rampage** (9 s) | About **1.1 s**, three advancing stages (about 28,000 in all, **stopped by walls**): an **axe cleave** (35 + 50% Attack, Frost, 20% slow 1 s), a **Blades sweep** (35 + 50% Attack, burn; the next advance can be **redirected**) and a **frost-and-fire smash** (radius 22,000, 70 + 110% Attack, **0.5 s knock-up**, Frost and burn). Each swing hits each enemy once. It **cleanses** him, and during it he's **CC-immune** and takes **25% less damage**; he can still die. |
| S3a | **Spartan Guard** (12 s, swaps to the axe) | **Cleanses**, a **shield of 15% max HP** for 3 s and **1 s** of CC immunity. |
| S3b | **Chain Dash** (10 s, swaps to the Blades) | Throws the Blades in a line (**55,000**). On a champion, minion or creep (**never a tower**) he **dashes to it, over any wall or terrain**, landing on open ground, and strikes for 30 + 50% Attack (V1 can parry it). A miss means no dash (the cooldown and the swap still happen). |
| S3 link | **Guard ↔ Dash** within 3 s | **Guard → Dash:** the throw reaches **90,000**. **Dash → Guard:** **1.8 s** of immunity and a **25%** shield. |
| Ult | **Fenrir's Charge** (90 s) | Fenrir appears **110,000** away in the aimed direction and **charges back to where Kratos stood**: 80 + 80% Attack and a 40% slow for 1.5 s to enemies on the way. **Recast within 3 s** (4 again): Kratos leaps into the wolf (up to 15,000) and **slams it back the way it came** (80,000): 100 + 100% Attack and a **1 s knock-up**. Once per enemy per leg. No recast: the wolf fades. Fenrir is drawn as effects and can't be targeted. |

**CC-immunity cap:** Sight, the Rampage and Guard share one clock. Windows **never stack** (an overlap only lasts to the longer end), and he is immune for **at most 3 s in any 6 s**; past that the immunity is refused but the rest of the skill (dodges, damage reduction, shields, cleanses) still works.

**Towers:** his skills, burns, freezes and Fenrir **never touch towers**; his basic attacks hit them normally, so he can push. **Walls:** the Chain Dash crosses terrain and built walls (Steve's, Rally's); Grimhald's cage is a shared rule nobody crosses, so it's respected (caged units are out of reach).

**Stats:** category Melee, tags AD/Melee/CC/Tank. Attack 80 + 8, HP 1,050 + 95, armor 26 + 5, MR 24 + 4, move speed 1,030 + 9.

## AI
- **Fenrir** down the line catching the most champions (2+, or 1 that's low, or 1 for a very aggressive athlete); the **recast** when 2+ are on the slam line, or 1 at the end of the window.
- **Heimdall's Sight** when focused (2+ champions close) or hurt with one close.
- **Guard → Dash** to engage from 55,000–90,000; **Dash → Guard** when he lands in a crowd or hurt.
- **Rampage** once a champion is in reach, or 2+ enemies in a fight; it steers its next advance at the nearest champion.
- **Farming when calm** (no enemy champion within 90,000): the Rampage on a wave or camp, and the Chain Dash onto a far camp or wave (it's movement too).
- His athlete attributes change his choices (how hurt he'll engage at), never his power.

## Code
- `native/tfm2_custom_ai/src/kratos_core.rs` (core), `kratos.rs` (bridge), `kratos_tests.rs` (core + mock-host glue tests). Generator: `tools/generate-kratos.py` (data, tooltips, roster, the **God of War** family).
- Moves are engine glides through `crate::motion`; the Chain Dash's hop over a wall is the one hand-set position (a blink across terrain, allowlisted in `motion_guard_tests.rs`). Shields use `entity_add_shield`; CC immunity and the Rampage's reduction are named buffs (`krt_immune`, `krt_reduce`); his own CCs are named `krt_` so he never reads himself as stunned.
- **Lab:** `native/kratos_lab` → `editor/kratos.html`, using the installed map layers, native-exported collision/object coordinates and original stock champion/minion/creep/structure sprites. The corrected registration is 1 art pixel = 1,000 world units, with a 160-pixel border around the playable source rectangle; all 900 wall cells agree. Choose lane, gank, being-ganked, camp, tower or objective situations, replace stock champions, and inspect through full-map/follow/1× views. Stock movement, combat, mitigation and visibility remain labelled host models; full stock skills/items are not emulated. See [battlefield evidence and reproduction](kratos-game-world-review/README.md). Tests: `node tools/test-kratos-lab.cjs` and `node tools/test-kratos-situations.cjs`.
- **Art:** `docs/kratos-art-handoff.md` (Codex).

## Known limits
- **Release packaging pending:** approved body, weapon and Fenrir/effect assets are integrated and champion-load checks pass. The packaged native DLL is still inherited from main; build/version a new release separately before claiming installed-game readiness.
- **Dodges** refund the hit through `heal` from `on_damaged`; a heal-reduction debuff on him would shrink the refund. Confirm in play.
- The engine's own handling of a glide that meets terrain is unverified; the dash plans its glides to stop at the wall and hops over it.
- Balance is unplayed; numbers are starting values.

## After the art lands
1. `python3 tools/generate-kratos.py --check` passes (update its effect list if a tag is renamed).
2. Mod Manager `the_repos_champions_all_load` passes.
3. Release: the next unused native / tfm2_custom versions in all four places, the DLL, and a review doc.
