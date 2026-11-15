# Basalt, the Wallbreaker (`basalt-1.1`)

An original stone-shaper brawler with a **stone whip**. He raises walls, punches them, throws them, claps them shut and slams enemies into them. He leaps over every wall, and his whip slingshots him around the map.

Category **Melee** (bruiser), tags AD / Melee / CC. Numbers live in `rules/basalt.json`; `tools/generate-basalt.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies).

## Passives (no skill slot; they fire on their own)
- **Bedrock.**
  - **Walls:** up to **5**, each a **big square block, 32,000 a side** (like Grimhald's blocks, but bigger; `wall.size`). A wall stands while he's within **150,000** of it and fades **3 s** after he leaves that range (coming back cancels the fade).
  - **Integrity:** only he can hit his walls. Each takes **3** of his hits (the punch, a throw landing, an enemy slammed into it) and then shatters into rubble: **+15%** on his next whip crack.
  - **Slam:** an enemy his skills drive into **any** wall (his, the map's, Steve's, Grimhald's) is **slammed**: a 1 s stun plus 40 + 60% Attack. The stun lands once every 2 s per target.
- **Jump** (2 charges, 6 s each):
  - **The leap:** a 90,000 leap, **airborne 0.6 s**: crowd-control immune, basics miss, and he **passes every wall**. He never lands inside terrain or a wall.
  - **Double jump:** one free double jump.
  - **Wall kick:** kicking off one of his walls goes **140,000** and restores the double jump (each wall once per airborne chain).
- **Whip Pull, the slingshot** (7 s): his whip catches any wall within 120,000 and **slingshots him over it**. The **farther he is, the faster he flies** (2,000 + distance ÷ 40 per tick). He lands 50% of the distance past the wall (up to 60,000), airborne all the way. Enemies on his path are knocked aside, harder at speed.

## The eight skills
Each slot has a **ground** version and a **jump** (airborne) version, sharing one cooldown.

| Slot | Ground | Airborne |
|---|---|---|
| S1 (2 charges, 8 s) | **Raise & Punch:** a wall rises in front of him and his punch launches it 110,000 forward, pushing enemies (50 + 60% Attack). Anyone pinned against a wall is slammed. | **Air Dash:** a 90,000 dash that **keeps him airborne**. **Anything can follow it.** |
| S2 (9 s) | **Hurl:** his whip lashes his nearest wall (or **rips a chunk off terrain**) within 120,000 and **throws it onto a point** within 160,000. Impact: 80 + 90% Attack and an outward knock (into walls: slammed). The wall stands where it lands. | **Slab Drop:** a wall thrown down within 90,000. An **area knock-up** (1 s) and 80 + 80% Attack. |
| S3 (12 s) | **Crushing Clap (the V slam):** two walls rise at the **upper left and upper right** of a 70° V and his whip **snaps them shut**. Enemies inside are swept to the middle, take 110 + 110% Attack and are stunned 1.25 s; the walls fuse into one. | **Whip Swing:** his whip snares an enemy champion within 45,000 and **grows to 2×** while he swings them **360° + x**. x points **toward his nearest teammate** (a gank) or else **away from their team**. They're slammed into a wall there (one is raised if none) for 120 + 150% Attack. |
| Ult (90 s) | **Boulder Throw:** he **stops and charges for 3 s**, then throws a massive rock **anywhere on the map**. Within 60,000 of the impact: a 1 s knock-up, a 40% slow (2 s), and 150 + 120% Attack. The rock stands as a wall. | **Meteor Punch:** he **hangs in the air charging for 3 s**, then **punches the rock apart**. **Six hot meteors** rain on the area: each one damages, knocks up a little, slows 30% and **burns** for 3 s. |
| Ult recast (5 s after the landing) | **Whip to the rock:** his whip catches the rock and slingshots him **to where it landed**, airborne. | Same. |

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
6. **Farm:** Hurl through a wave (3 minions or more).

## Code
- **Core:** `native/tfm2_custom_ai/src/basalt_core.rs`. It's deterministic, and the Lab runs this same file.
- **Bridge:** `basalt.rs`.
  - **Walls:** they become the shared `sbw:` walls (each square is two parallel shared walls on him that together fill it, renewed every 40 ticks and rewritten when it moves), so everyone collides with them; `steve::note_map_buff` switches the shared pathing on. Each is drawn as a 2×2 cluster of stone blocks.
  - **Airborne:** `bs_air` (`cc_immune`, `ignore_wall`, basics miss) plus `place::MOVED`.
  - **Moves:** every move is a glide (no `entity_set_pos`).
  - **Slows and charge:** `bs_slow`; the ult's charge is a named Bind on himself (`bs_charge`).
- **Tests:** `basalt_tests.rs` (25: core, AI and glue).
- **Lab:** `native/basalt_lab` builds `editor/basalt.wasm`; the page is `editor/basalt.html`. It's a sandbox with terrain, the AI toggle, and every skill on a key.
- **Art:** placeholders (Grimhald's body; Grimhald's cage blocks and pillars, and Kicker's rocks and slams). See `docs/basalt-art-handoff.md`.
