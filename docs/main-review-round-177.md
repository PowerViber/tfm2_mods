# Round 177: Basalt slowed down, no more flicker, and every champion skills creeps (native 0.10.111, tfm2_custom 0.2.114, `basalt-1.4`)

**Reihan, after the first live clip of Basalt:**
- "the walls are flickering, and his slingshot is too far, like he has abnormal speed";
- "make him able to skill the creeps", then "make the newer champs to be able to skill on creeps" and "can be all champions, even roam made jungle, like kagami";
- "basalt overall skill execution is too fast, we have to nerf him greatly", then "the cooldowns are fine but the range of what he can travel and the speed is abnormal";
- "basalt ult also flickers".

## Basalt `basalt-1.4`
### Slower, shorter moves (cooldowns unchanged)
| Move | Was | Now |
|---|---|---|
| Jump | 90,000 in 0.6 s | **60,000 in 0.8 s** |
| Double jump | another 90,000 | **40,000** |
| Wall kick | 140,000 | **80,000** |
| Air Dash | 90,000 in 0.3 s | **50,000 in 0.6 s** (extra air 0.5 s) |
| Whip Pull reach | 120,000 | **80,000** |
| Whip Pull speed | 2,000 + d/40 per tick, no cap | **1,200 + d/100, capped at 2,000** (about 2× his walk) |
| Whip Pull landing past the wall | 50%, up to 60,000 | **20%, up to 20,000** |
| Punch slide | 110,000 in 0.6 s | **80,000 in 0.9 s** |
| Ult recast whip | up to 9,000 per tick | **4,000 per tick** |

- **Cast recovery (new):** after any skill, jump or whip pull he waits **0.5 s** (**0.35 s** in the air) before the next one. Before this, his AI chained a jump, Air Dash, Slab Drop and Whip Swing on back-to-back ticks. The ult's recast is exempt. While recovering, his AI walks on toward its target.
- **More time to react:** Hurl flies 0.8 s (was 0.5), Slab Drop 0.6 s (0.3), the Clap closes in 0.7 s (0.3), Whip Swing spins 1 s (0.6).
- **Result:** a full airborne chain (jump, double jump, Air Dash, Slab Drop) now takes over a second and covers about 150,000, where it used to cover 400,000+ in about 2 s.

### The flicker (walls and ult)
- **Cause:** the presentation renews each drawn effect just before it expires, with at most 4 renewals per tick shared by everything. The whip strips (up to 16 pieces), cones and warnings took those slots, so a standing wall, or the ult's rock, meteors and warnings (up to 14 at once), missed its renewal and blinked off for a tick.
- **Fix** (`basalt_presentation.rs`):
  - walls and ult effects go first and no longer count toward the 4-per-tick cap;
  - effects that stay put (walls, warnings, the charging rock) now last two renew periods, so a late renewal never leaves a gap;
  - moving ones (the flying boulder, falling meteors) keep their short life, so there's no trailing double image.
- **The no-art fallback** (2×2 blocks) gets the same overlap.
- **Tests:** 600-tick runs with all 5 walls, a clap and its whips, and an air ult with six meteors check that every wall and ult effect is covered on every tick. Both runs fail on the old code.

### Skills on creeps
- **His frame** now marks farmable minions and jungle monsters (the shared farming rules, calm only), like Kicker's.
- **Farming (no enemy champion in his engage range)** uses all six non-ult skills:
  - **Ground:** Crushing Clap on two creeps or a camp monster, Raise & Punch into a camp, Hurl on two creeps or a camp.
  - **To open the air skills:** a jump onto a camp, only with both charges so one stays for fights.
  - **Airborne:** Whip Swing on a camp monster, Slab Drop on a group or a monster, Air Dash onto a camp.
- **Whip Swing** takes a creep when no enemy champion is in reach. Champions always come first.
- **Never on creeps:** the ult.

## The creeps rule for every champion
**Reihan's rule:** every skill except the ult can be used on minions and jungle creeps, roamers and supports included.
- **Kicker (`kicker_core.rs`):**
  - **The Kick** takes a creep (a camp monster), into a wall or toward home, never under an enemy tower.
  - **Skyfall** carries the first creep in its line when no champion is in it.
  - **Farming adds:** Wall Bounce onto a camp or wave, Skyfall through it while Bounced, the Kick, and Marked Dash onto a marked monster (one dash kept).
- **Thalassa (`thalassa_core.rs`):** farming adds Tide Twin sent at a camp or wave, and Dive onto one from water.
- **Akane (`akane_core.rs`):** farming adds Dismiss & Devour, to heal up below 60% HP or close on a camp monster. A charge is always kept.
- **Grimhald (`grimhald_core.rs`):** Skull Bash goes on any camp monster (1,000+ HP, was 1,500) or a bunch of three; Iron Ghost closes onto a camp out of Bash reach.
- **Kagami (`kagami_core.rs`, `kagami_ai.rs`, `kagami.rs`):** she can jungle.
  - Her frame marks farmable units, and her clones guarding her already shoot them.
  - When calm she Charms herself at a camp and Mirror Steps onto a far camp. Mend already heals her below 60%.
  - Ally-only uses stay on allies.
- **Wright:** the Carpet Bomber also goes on a lone camp monster (it needed 3 units). Fighters already farmed; Repair stays for allies.
- **Kratos:** already farmed with Rampage and the Chain Dash. A test confirms it. Sight and Spartan Guard are utility.
- **Tests:** per champion, a calm run beside a camp (and a wave) checks that every listed skill is used and the ult never is.

### Audit of the other champions
| Champion | Farming today | Notes |
|---|---|---|
| Contender, Rally, Frieren, Steve (craft) | every skill aims at "a champion, else a farmable unit" | Champions first. Nothing to add. |
| Gojo | Blue and Red on farmable units when calm | Both non-ult skills. Complete. |
| Raze | a farm window over every skill target | Complete. |
| Coder, Vesper | a farm-only mode with their damage functions | Utility functions are held back by design. |
| Scribble | farming recipes (a fixed list) | Large recipes reserved by design. |
| Isliid | a farm sword | Formation swords and empowerment are reserved. |
| Levi | sweeps through a wave or camp | Complete. |
| Magnus, Unified Theory | eligible units in their planners | Complete. |
| Omen | Paranoia as wave clear | Dark Cover and Shadow Step are utility. |
| Aegis Zero | marked-enemy slices | The ult is never used for farming. |
| Minato, David, Dio, Vader, V1 | the universal AI's capability table (`docs/universal-farming-coverage.md`) | Only "offense" slots farm; live execution is still pending there. A per-skill pass is a follow-up. |

## Checks
- **Native tests:** 1,362 passed, 18 ignored. That includes the new pace, flicker and creeps tests.
- **Lab tests:** the Basalt, Kicker, Thalassa, Akane, Grimhald, Kagami, Kratos and Wright Lab tests pass. The Basalt transport test now spaces its orders past the recovery.
- **Lab wasm:** rebuilt for Basalt, Kicker, Thalassa, Akane, Grimhald, Kagami and Wright.
- **Other checks:** the generators are current; the DLL (0.10.111) is built; the Mod Manager tests (15) pass.
- **Not run here:** Codex's Windows-only parity, packaging and browser checks.
