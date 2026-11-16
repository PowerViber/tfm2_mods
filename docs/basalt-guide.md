# Basalt, the Wallbreaker (`basalt-1.4`)

An original stone-shaper brawler with a **stone whip**. He raises walls, punches them, throws them, claps them shut and slams enemies into them. He leaps over every wall, and his whip slingshots him around the map.

Category **Melee** (bruiser), tags AD / Melee / CC. Numbers live in `rules/basalt.json`; `tools/generate-basalt.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies).

## Passives (no skill slot; they fire on their own)
- **Bedrock.**
  - **Walls:** up to **5**, each a **big square block, 32,000 a side** (like Grimhald's blocks, but bigger; `wall.size`). A wall stands while he's within **150,000** of it and fades **3 s** after he leaves that range (coming back cancels the fade).
  - **Integrity:** only he can hit his walls. Each takes **3** of his hits (the punch, a throw landing, an enemy slammed into it) and then shatters into rubble: **+15%** on his next whip crack.
  - **Slam:** an enemy his skills drive into **any** wall (his, the map's, Steve's, Grimhald's) is **slammed**: a 1 s stun plus 40 + 60% Attack. The stun lands once every 2 s per target.
- **Jump** (2 charges, 6 s each):
  - **The leap:** a 60,000 leap, **airborne 0.8 s**: crowd-control immune, basics miss, and he **passes every wall**. He never lands inside terrain or a wall.
  - **Double jump:** one free 40,000 double jump.
  - **Wall kick:** kicking off one of his walls goes **80,000** and restores the double jump (each wall once per airborne chain).
- **Whip Pull, the slingshot** (7 s): his whip catches any wall within 80,000 and **slingshots him over it**. The **farther he is, the faster he flies** (1,200 + distance ÷ 100 per tick, at most 2,000). He lands 20% of the distance past the wall (up to 20,000), airborne all the way. Enemies on his path are knocked aside, harder at speed.
- **Cast recovery** (`basalt-1.4`): after any skill, jump or whip pull he waits **0.5 s** (0.35 s in the air) before the next one. The ult's recast is exempt.

## The eight skills
Each slot has a **ground** version and a **jump** (airborne) version, sharing one cooldown.

| Slot | Ground | Airborne |
|---|---|---|
| S1 (2 charges, 8 s) | **Raise & Punch:** a wall rises in front of him and his punch launches it 80,000 forward (0.9 s), pushing enemies (50 + 60% Attack). Anyone pinned against a wall is slammed. | **Air Dash:** a 50,000 dash (0.6 s) that **keeps him airborne**. **Anything can follow it.** |
| S2 (9 s) | **Hurl:** his whip lashes his nearest wall (or **rips a chunk off terrain**) within 120,000 and **throws it onto a point** within 160,000. Impact: 80 + 90% Attack and an outward knock (into walls: slammed). The wall stands where it lands. | **Slab Drop:** a wall thrown down within 90,000. An **area knock-up** (1 s) and 80 + 80% Attack. |
| S3 (12 s) | **Crushing Clap (the V slam):** two walls rise at the **upper left and upper right** of a 70° V and his whip **snaps them shut**. Enemies inside are swept to the middle, take 110 + 110% Attack and are stunned 1.25 s; the walls fuse into one. | **Whip Swing:** his whip snares an enemy champion within 45,000 and **grows to 2×** while he swings them **360° + x**. x points **toward his nearest teammate** (a gank) or else **away from their team**. They're slammed into a wall there (one is raised if none) for 120 + 150% Attack. |
| Ult (90 s) | **Boulder Throw:** he **stops and charges for 0.5 s**, then throws a massive rock **anywhere on the map**. Within 60,000 of the impact: a 1 s knock-up, a 40% slow (2 s), and 150 + 120% Attack. The rock stands as a wall. | **Meteor Punch:** he **hangs in the air charging for 0.5 s**, then **punches the rock apart**. **Six hot meteors** rain on the area: each one damages, knocks up a little, slows 30% and **burns** for 3 s. |
| Ult recast (5 s after the landing) | **Whip to the rock:** his whip catches the rock and slingshots him **to where it landed** (at most 4,000 per tick), airborne. | Same. |

## Combos (the AI plays these)
- **Air Dash** never ends the airborne chain: after it come air skills, the double jump, a wall kick, a Whip Pull, or a ground skill the instant he lands.
- **Slingshot & Swing:** whip a far wall beyond the target, fly over it, then Whip Swing them into that same wall.
- **Over the top:** slingshot over terrain, Air Dash onto the group, Slab Drop, then land into Crushing Clap.
- **Throw and follow:** Boulder Throw or Meteor Punch onto a far fight, then the recast whips him there.

## AI (Kicker's flat list)
Re-run every tick, with no lock-on:
0. **Ult:**
   - **Recast:** to follow up a hit at the rock (not outnumbered, his HP at 40%+), or to escape to it.
   - **Charge:** only with no enemy champion within 80,000 of him, onto a group of three, two his team is fighting, or a low champion in a fight. It's Meteor Punch when he's airborne.
1. **Escape** (real damage, under 45% HP, pressed): slingshot or jump toward his teammates or home; else punch a wall into the diver.
2. **Peel:** engage the diver on a hurt teammate.
3. **Follow-up:** after a slam, go hit that target.
4. **Help:** a teammate's fight within 250,000 (slingshot or jump toward it, else walk).
5. **Best target** (`pick` / `score` / `would_start` / `outnumbered`, never under a tower):
   - **Airborne:** Whip Swing in reach, Slab Drop on a group, else Air Dash in.
   - **Ground:** Crushing Clap on a group or a low target in the cone, Raise & Punch with a wall behind them, Hurl. Then mobility: jump in (for Whip Swing) or slingshot over a wall toward them.
6. **Farm** (`basalt-1.4`, when no enemy champion is in his engage range): every skill but the ult works on farmable minions and jungle monsters.
   - **Ground:** Crushing Clap on two creeps or a camp monster, Raise & Punch into a camp, Hurl on two creeps or a monster.
   - **Opening the air skills:** a jump onto a camp, only with both charges.
   - **Airborne:** Whip Swing on a camp monster, Slab Drop on a group or a monster, Air Dash onto a camp.

While he recovers between skills, the AI walks on toward its target.

## Code
- **Core:** `native/tfm2_custom_ai/src/basalt_core.rs`. It's deterministic, and the Lab runs this same file.
- **Bridge:** `basalt.rs`.
  - **Walls:** they become the shared `sbw:` walls (each square is two parallel shared walls on him that together fill it, renewed every 40 ticks and rewritten when it moves), so everyone collides with them; `steve::note_map_buff` switches the shared pathing on. Approved artwork draws one composite square at its current core position; legacy definitions retain the four-stamp fallback. Walls and ult effects are drawn first, outside the per-tick cap, and standing ones live two renew periods, so they never blink (`basalt-1.4`).
  - **Airborne:** `bs_air` (`cc_immune`, `ignore_wall`, basics miss) plus `place::MOVED`.
  - **Moves:** every move is a glide (no `entity_set_pos`).
  - **Slows and charge:** `bs_slow`; the ult's charge is a named Bind on himself (`bs_charge`).
- **Tests:** `basalt_tests.rs` (core, AI, pace, creeps and glue) and the flicker runs in `basalt_presentation.rs`.
- **Lab:** `native/basalt_lab` builds `editor/basalt.wasm`; `editor/basalt.html` uses the extracted game map and stock champions, minions, camps, towers and objectives, with mid-game scenarios, thirteen-stat controls, fixed 60 Hz playback and every skill on a key. Host movement/combat/visibility are labelled simulations. Shared Rust presentation drives the actual packaged effects.
- **Art:** approved original red/yellow body at approximately 40 visible pixels, whole-body animations in both facings, square blocks, whips, smaller boulder and meteors. Crushing Clap's blocks start at its 110,000-range corners and fuse at the far centre. Native ordinary poses are supported; every extra passive-skill owner pose is not claimed. See `docs/basalt-art-review/README.md` for approvals, reproducible hashes and native/Lab limits. Installation and real-career migration remain separate.
