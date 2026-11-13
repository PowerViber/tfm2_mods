# Kicker, the rooftop street fighter

**What he is:** an original melee fighter for any lane (jungle, top, support or roam, mid). He engages, makes picks and peels by **kicking an enemy champion**: out from under its tower, into a teammate's gank, or away from his carry.
- **Working name:** "Kicker". The theme, name and art come from Codex later; the brief is in `docs/kicker-art-handoff.md`.
- **Until that art exists:** he uses Contender's body sheet as a placeholder, so the game never skips him.
- **Revision:** `kicker-2.0` (`rules/kicker.json`, round 173). Game category Melee (a fighter), physical, scaling with Attack.
- **Round 173:** the wall stance is gone. He no longer stands on walls: S1 is a **Wall Bounce** off a wall onto an enemy. He's also tougher: more damage reduction, more armor, and CC immunity.

**Stats:** Defence **30 + 5** per level (was 24 + 4).

## Kit
| Slot | Name | What it does |
|---|---|---|
| P1 | **Wall Runner** | **Drop-in** (every Wall Bounce landing; 6 s): enemies within 15,000 of the landing are knocked up for 1 s, and his next hit deals +15%. **Landing shield:** 12% max HP for 3 s, once every 8 s. **Bounced** (3 s after each bounce): **25% less damage taken**, **CC immune for the first 1 s**, his basic attack is a **45,000-range rock throw** (it marks), and his next S2 and S3 are their wall versions. **CC immune** during every leap and dash of his (the bounce, Marked Dash, the kick's dash-in, Skyfall). Back-to-back bounces only refresh these, never stack them. |
| P2 | **Marked Dash** | **Charges:** 2, 8 s each. **Marks:** his basic attacks and skills mark enemies, minions and creeps for 4 s (never towers). **Dash:** to a marked unit within 75,000, always ending **behind** it, on open ground. **Armor:** every dash and leap gives a **7% max HP shield for 2.5 s**, never stacked. |
| P3 | **Bruised** | **Who:** everyone he crowd-controls is Bruised for 4 s. **Effect:** no HP regeneration and 40% less healing. **Stacks:** each of his hits adds +3% damage taken, up to 5 stacks (15%); they end with Bruised. |
| S1 | **Wall Bounce** | **No cooldown** whenever a wall is in reach. **Cast** at an enemy or a point: he leaps to a wall within **80,000**, **touches it for 0.25 s** (hidden, 25% less damage), then **springs** onto the enemy or point within **100,000** of that wall. On an enemy he lands **behind it** and kicks it for **60 + 80% Attack**. Invulnerable to damage and CC immune from the cast to the landing. **No wall in reach:** a short 20,000 hop (2 charges, 7 s each). |
| S2 | **Rock Throw / Rock Rain** | **Throw** (9 s): a rock flies 110,000 in a line, through everything (never towers), for 40 + 70% Attack, marking each unit hit. **Bounced:** Rock Rain, three rocks in a narrow fan (90,000), 40% damage each. |
| S3 | **Homecoming Kick / Skyfall Kick** | **Charges:** 2, 20 s each. **On the ground:** he dashes up to 15,000 to stand behind an enemy champion and kicks it 75,000 away from him for 70 + 110% Attack. The flying body hits up to 2 other enemy champions (50% damage, 0.4 s knock-up). A wall stops it and stuns for 0.5 s; never through a wall or off the map. **Flash:** just before the kick, or during its short wind-up, it re-aims the kick from the new spot. **Bounced (once per bounce): Skyfall Kick.** An 80,000 dive that carries the first enemy champion in its line to the end, through walls, for 90 + 100% Attack. He lands 8,000 past it (behind it) with a Drop-in, both on open ground. |
| Ult | **Second Wind** | **Cast** (90 s): he tags where he stands and fights freely for 8 s. **A death in that time:** he doesn't die; he's back at the tag with full HP, and nobody gets a kill. **The time runs out:** he's back at the tag with the HP he had at cast (no free heal). |

## His AI (his own, `kicker-2.0`)
**Who drives it:** the athlete's 13 values. Ego decides whether he *starts* a fight, aim sets kick-direction accuracy, and reaction sets dive answers. There is no fixed personality (Reihan).

**Targets:**
- **Choosing:** a score re-read every tick. Nearer, lower, alone, or already fighting his team scores better; enemies around it score worse. So he switches targets instead of chasing one.
- **Never outnumbered:** no target, fight or bounce where enemy champions outnumber his side by two (or three against fewer than two of his).
- **Under its tower:** a target there counts only when a **tower kick-out is ready**: a kick charge, a safe way out, the kick spot within reach, and his HP at 50% or more.
- **Movement:** he never "locks on". He only walks with a purpose (a fight, a low target, Second Wind, or while Bounced); otherwise the game moves him.
- **Joining fights:** a fight his team is already in needs no Ego (Ego only decides who starts).

**The Wall Bounce (no wall state to manage):** he's never on a wall except for the 0.25 s touch, so there is nothing to get stuck on.
- **Engage, wall first:** on his target (within 90,000, or up to 180,000 when it's fighting his team, low, or alone), with a wall in reach and the target in spring range, he bounces onto it whenever the Drop-in is ready (or the target is out of plain reach). Then, while Bounced, he **Skyfalls** it (one charge is enough in a fight, on a low target, or on one alone near his teammates) or **rains rocks** on it, and his basics are 45,000 rock throws.
- **Helping:** a teammate in a fight within 200,000: a Marked Dash on anything marked that way, else a bounce onto the enemy on it, else a walk.
- **Escape:** only after real damage (or a tower on him), never under the tag: a Marked Dash away, else a bounce toward safety (his teammates away from the danger, otherwise home) when the landing is safer.
- **Kicks** keep their round 172 rules (see below); the bounce never replaces a kick that's ready.

**Second Wind:**
- **Death cancel:** at low HP with an enemy close, real damage, or an enemy projectile closing in, Flash ready or not. The HP line is `30 - Ego/5` % (lower for a high Ego). Then he goes **all in**: no escape, the held kick charge is free, and picks inside groups are fair game.
- **Fearless engage:** right before a kick, Skyfall, Wall Bounce or Marked Dash onto a champion, he tags first (never from under an enemy tower). How freely depends on Ego:
  - **Ego 60+:** any engage, even a 1v2 or a tower dive;
  - **Ego 35-59:** real picks or fights only;
  - **Ego under 35:** teamfights of 3+ only.
- **Teamfight:** when 2+ enemies are fighting his team (3+ at Ego under 35).

**Combos:**
- **Tower kick-out:** Rock Throw marks, then Marked Dash in, then the kick out of tower range toward his team.
- **Flash kick:** only for a real pick (a low target, a peel, or a teammate at the landing), never next to a tower or into a crowd, Judgement 35+.
- **Gank into an ally:** the kick goes toward the nearest teammate.
- **Peel:** the diver on a hurt teammate is kicked away from them (this may spend the held charge).
- **Escape:** only after real damage, never under the tag.
- **Farming:** S2 through the wave only. S1, S3 and the ult never go to minions.

**Kick direction score:**
- **In order:** toward a teammate, out of the enemy tower, toward home, then into a wall for the stun.
- **Rejected:** any direction that lands the target under an enemy tower or among its team.
- **Charges:** one is held for picks. The second only finishes a target he just kicked, or peels.
- **Never:** at a body still flying from his last kick, or into a V1 parry.

## Code
- **Data:** `rules/kicker.json` feeds `tools/generate-kicker.py` (`--check`), which writes `kicker_data.rs`, `tfm2_custom_kicker.data_champion`, the tooltips, `editor/kicker-data.js` and the roster entry.
- **Core:** `native/tfm2_custom_ai/src/kicker_core.rs` is deterministic (Frame in, Effects out). The Lab runs the same file.
- **Bridge:** `kicker.rs`.
  - **Walls:** `kk_roof` (`ignore_wall`, 25% reduction) only for the 0.25 s touch, plus `kk_pass` during leaps. `kk_bounced` (25% reduction, the rock basic range) and `kk_unstoppable` (`cc_immune`, never shortened by a later, shorter window).
  - **Kick and carry:** they go through `crate::motion::glide`; the carry holds `kk_carry` (`ignore_wall`).
  - **Status buffs:** `kk_bruised:<n>` (`heal_reduce`, `damaged_amplify`, regen cancelled) and `kk_second_wind` (`undying`).
  - **Flash and steering:** Flash goes through `flash::kit_blink` (the shared cooldown). Steering is `kk_go:` and rewrites walks only.
  - **Teleport:** the ult's return is the one `entity_set_pos` (allowlisted).
- **Tests:** 60 core and glue tests in `kicker_tests.rs`. He is also in `perf_bench_tests.rs`, at about 6 µs per tick on the mock host.
- **Lab:** the `native/kicker_lab` crate builds `editor/kicker.wasm`. `editor/kicker.html` has editable walls, units, manual orders and the AI.

## Known limits
- **Shields:** "no shield regeneration" under Bruised has no engine counterpart (shields don't regenerate in TFM2). Bruised does stop HP regen and cuts healing.
- **Live checks pending:**
  - the 0.25 s touch on a wall cell with `ignore_wall`;
  - `cc_immune` blocks every stock CC during the leaps;
  - the overlay draws right;
  - `undying` holds against every damage type (towers, true damage) with no kill credit.
