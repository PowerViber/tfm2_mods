# Round 180: level unlocks for native skills (native 0.10.115, tfm2_custom 0.2.118)

**Reihan:** "since the start of adding the 3 skill characters … i havent made level unlock limitation, so everyone can use any skill they want for the new champs".

## The problem
- **The engine's lock:** the engine locks its own skill slots: skill2 until level 3, and the ult until level 5 (`CooldownV1.unlocked`, `mod-api-stable/src/values.rs`).
- **Why native kits escaped it:** they park the engine slots on placeholder cooldowns and cast their skills themselves, so that lock never reached them.
- **Ult only:** most of them checked only the ult, through the ult-learned marker.
- **No checks at all:** Coder and Scribble checked nothing, so even their ult worked from level 1.

## The rule (Reihan)
| | Unlocks at |
|---|---|
| S1 | level 1 |
| S2 | level 2 |
| S3 | level 3 |
| Ult | level 5 |

Passives are always on.

- **Shared rule:** `native/tfm2_custom_ai/src/unlocks.rs`.
- **Bridges:** each bridge passes the champion's level (`entity.level()`) to its core.
- **Cores:** a locked skill is held "on cooldown" until its level, so the AI skips it and manual orders are refused.
- **Level 0:** Labs and older tests that send no level treat everything as unlocked.

## Per kit
| Kit | S2 (level 2) | S3 (level 3) | Ult (level 5) |
|---|---|---|---|
| Contender | Shadow Cyclone | Shadow Tether | Shadow Flurry (also needs 40 banked) |
| Akane | Kunai Flip | Crimson Edge | Crimson Night |
| Grimhald | Stone Bulwark | Skull Bash | Warden's Verdict |
| Thalassa | Dive | Tide Twin | Trident |
| Wright | Carpet Bomber | Repair Wing | Sky Carrier |
| Kratos | Leviathan Rampage | Spartan Guard and Chain Dash | Fenrir |
| Kicker | Rock Throw | Homecoming / Skyfall Kick (both charges arrive at level 3) | Second Wind |
| Kagami | Kindred Charm | Moonlit Mend | Last Reflection |
| Basalt | S2 | S3 (all variants) | Boulder / Meteor |
| Magnus | Develop a Piece | — | Grandmaster Board |
| Unified | Curie's six skills | Einstein's six skills (Newton's six are S1) | Theory of Everything |
| Coder | — | — | Activate AI |
| Scribble | 3-dot spells | 4-dot spells (1–2-dot spells are S1) | 5–6-dot spells |

**Notes:**
- **Vesper:** her kit is passive only (no active skills, no ult), so there's nothing to lock.
- **Gojo v2, Raze, Rally, Frieren, Steve and Omen:** they already used the engine's own slot locks, so their skill2 stays at level 3, the engine's rule.
- **Engine-cast kits:** Minato, DIO, David, Vader, V1, Gundam, Isliid and Levi are locked by the engine itself.
- **Tooltips:** each affected champion's skill text now ends with "Level unlocks: S2 at level 2, S3 at level 3, the ult at level 5."
- **Labs:** they gain a **Level** box (default 18).

## Checks
- **Native tests:** 1,416 passed, 18 ignored. New tests:
  - the threshold table;
  - every 3-skill core holds S2 until 2 and S3 until 3 (level 0 unlocks all);
  - Kicker's kick charges arrive at level 3;
  - Contender refuses locked orders, its AI at level 1 never uses S2/S3, and the ult waits for level 5 even with the bank full;
  - Scribble's spell sizes.
- **Lab crates:** tests pass for Akane, Basalt, Grimhald, Kagami, Kicker, Kratos, Thalassa, Wright and Contender; their wasm is rebuilt.
- **Contender's Lab, driven directly:**
  - level 1 refuses S2 and S3;
  - level 2 casts S2, and level 3 casts S3;
  - the ult is blocked at level 4 and casts at level 5.
- **Other checks:** the generators are current; the DLL is built; the Mod Manager tests (15) pass.
- **Not run here:** the real-map Lab browser checks and Codex's Windows checks.
