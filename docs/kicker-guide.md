# Kicker, the rooftop street fighter

**What he is:** an original melee fighter for any lane (jungle, top, support or roam, mid). He engages, makes picks and peels by **kicking an enemy champion**: out from under its tower, into a teammate's gank, or away from his carry.
- **Working name:** "Kicker". The theme, name and art come from Codex later; the brief is in `docs/kicker-art-handoff.md`.
- **Until that art exists:** he uses Contender's body sheet as a placeholder, so the game never skips him.
- **Revision:** `kicker-1.0` (`rules/kicker.json`). Game category Melee (a fighter), physical, scaling with Attack. **No CC immunity anywhere.**

## Kit
| Slot | Name | What it does |
|---|---|---|
| P1 | **Rooftop Runner** | He stands and runs on wall cells. **Stats on a wall:** +25% speed and 15% less damage taken. **High ground:** hidden from enemies farther than 40,000. **Drop-in** (every jump down; 6 s): invulnerable to damage while dropping, knocks up enemies within 15,000 of the landing for 1 s, and his next hit deals +15%. **Ground shield:** landing gives a shield of 8% max HP (3 s, once every 10 s). **Never walks off a wall:** a walk toward open ground stops at the wall's edge, and a step the engine takes off it is pulled straight back onto the wall (no Drop-in). Only a skill (Drop Kick, Skyfall, a Marked Dash) or hard control takes him down; far from any wall (a teleport) the wall stance ends. Hard control also interrupts a wall hop and drops him to open ground. |
| P2 | **Marked Dash** | **Charges:** 2, 8 s each. **Marks:** his basic attacks and skills mark enemies, minions and creeps for 4 s (never towers). **Dash:** to a marked unit within 75,000, always ending **behind** it (the far side from where he started), on open ground. **Armor:** every dash and leap of his (Marked Dash, the kick's dash-in, Skyfall, Drop Kick, mount and hops) gives a 4% max HP shield for 2 s, never stacked. |
| P3 | **Bruised** | **Who:** everyone he crowd-controls is Bruised for 4 s. **Effect:** no HP regeneration and 40% less healing. **Stacks:** each of his hits adds +3% damage taken, up to 5 stacks (15%); they end with Bruised. |
| S1 | **Wall Hop / Drop Kick** | **Mount:** onto a wall within 40,000 (its own 7 s cooldown). **Wall to wall:** within 300,000 (4 charges; 3 s each on a wall, 6 s on the ground). **Drop Kick:** off a wall onto an enemy or point within 50,000 for 60 + 80% Attack; no charge, with a Drop-in. **No wall near:** a short 20,000 hop. |
| S2 | **Rock Throw / Rock Rain** | **Throw** (9 s): a rock flies 110,000 in a line, through everything (never towers), for 40 + 70% Attack, marking each unit hit. **From a wall:** three rocks in a narrow fan (90,000), 40% damage each. |
| S3 | **Homecoming Kick / Skyfall Kick** | **Charges:** 2, 20 s each. **On the ground:** he dashes up to 15,000 to stand behind an enemy champion and kicks it 75,000 away from him for 70 + 110% Attack. The flying body hits up to 2 other enemy champions (50% damage, 0.4 s knock-up). A wall stops it and stuns for 0.5 s; never through a wall or off the map. **Flash:** just before the kick, or during its short wind-up, it re-aims the kick from the new spot. **From a wall: Skyfall Kick.** An 80,000 dive that carries the first enemy champion in its line to the end, through walls, for 90 + 100% Attack. He lands 8,000 past it (behind it) with a Drop-in, both on open ground. |
| Ult | **Second Wind** | **Cast** (90 s): he tags where he stands and fights freely for 8 s. **A death in that time:** he doesn't die; he's back at the tag with full HP, and nobody gets a kill. **The time runs out:** he's back at the tag with the HP he had at cast (no free heal). |

## His AI (his own, `kicker-1.3`)
**Who drives it:** the athlete's 13 values. Ego decides whether he *starts* a fight, aim sets kick-direction accuracy, and reaction sets dive answers. There is no fixed personality (Reihan).

**Targets:**
- **Choosing:** a score re-read every tick. Nearer, lower, alone, or already fighting his team scores better; enemies around it score worse. So he switches targets instead of chasing one.
- **Under its tower:** a target there counts only when a **tower kick-out is ready**: a kick charge, a safe way out, the kick spot within reach, and his HP at 50% or more.
- **Movement:** he never "locks on". He only walks with a purpose (a fight, a low target, Second Wind, or a wall to climb); otherwise the game moves him.
- **Joining fights:** a fight his team is already in needs no Ego (Ego only decides who starts).

**Walls:**
- **Between fights:** with enemies around, he walks to a wall within 120,000 and climbs it (Roaming 40+; 70+ or high Ego looks farther).
- **Climbing to reach a target:** when a wall stands between him and a target, he climbs it.
- **On a wall:** his basic is a 45,000 rock throw. He rains rocks on any enemy in range, Skyfalls (one charge is enough in a fight, on a low target, or on one alone near his teammates), Drop Kicks into fights, and throws basics at anything in range. A target out of reach gets a 300,000 hop to the wall nearest it. After 3 s with nothing to do, he drops toward his teammates.

**Second Wind:**
- **Death cancel:** at low HP with an enemy close, real damage, or an enemy projectile closing in, Flash ready or not. The HP line is `30 - Ego/5` % (lower for a high Ego). Then he goes **all in**: no escape, the held kick charge is free, and picks inside groups are fair game.
- **Fearless engage:** right before a kick, Skyfall, Drop Kick or Marked Dash onto a champion, he tags first (never from under an enemy tower). How freely depends on Ego:
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
- **Farming:** S2 through the wave only. S3 and the ult never touch minions.

**Kick direction score:**
- **In order:** toward a teammate, out of the enemy tower, toward home, then into a wall for the stun.
- **Rejected:** any direction that lands the target under an enemy tower or among its team.
- **Charges:** one is held for picks. The second only finishes a target he just kicked, or peels.
- **Never:** at a body still flying from his last kick, or into a V1 parry.

## Code
- **Data:** `rules/kicker.json` feeds `tools/generate-kicker.py` (`--check`), which writes `kicker_data.rs`, `tfm2_custom_kicker.data_champion`, the tooltips, `editor/kicker-data.js` and the roster entry.
- **Core:** `native/tfm2_custom_ai/src/kicker_core.rs` is deterministic (Frame in, Effects out). The Lab runs the same file.
- **Bridge:** `kicker.rs`.
  - **Walls:** `kk_roof` (`ignore_wall`, speed, 15% reduction), plus `kk_pass` during leaps.
  - **Kick and carry:** they go through `crate::motion::glide`; the carry holds `kk_carry` (`ignore_wall`).
  - **Status buffs:** `kk_bruised:<n>` (`heal_reduce`, `damaged_amplify`, regen cancelled) and `kk_second_wind` (`undying`).
  - **Flash and steering:** Flash goes through `flash::kit_blink` (the shared cooldown). Steering is `kk_go:` and rewrites walks only.
  - **Teleport:** the ult's return is the one `entity_set_pos` (allowlisted).
- **Tests:** 36 core and glue tests in `kicker_tests.rs`. He is also in `perf_bench_tests.rs`, at about 18 µs per tick on the mock host, with Contender and Steve.
- **Lab:** the `native/kicker_lab` crate builds `editor/kicker.wasm`. `editor/kicker.html` has editable walls, units, manual orders and the AI.

## Known limits
- **Vision:** "+50% vision on a wall" isn't possible through the API (no sight stat). High ground hides him instead.
- **Shields:** "no shield regeneration" under Bruised has no engine counterpart (shields don't regenerate in TFM2). Bruised does stop HP regen and cuts healing.
- **Live checks pending:**
  - standing still on a wall cell with `ignore_wall`;
  - enemies can target him there;
  - the overlay draws right;
  - `undying` holds against every damage type (towers, true damage) with no kill credit.
