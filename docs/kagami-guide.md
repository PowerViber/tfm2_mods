# Kagami, the Mirror Priestess (`kagami-1.0`)

A support built around a **global revive**. Her mirror clone fights, heals and buffs beside her, every swap with it stuns the danger spot, and her ult saves the first teammate who would die, anywhere on the map.

Category **Util**, tags AD / Range / Heal / Shield. Numbers live in `rules/kagami.json`; `tools/generate-kagami.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies).

## Kit
| Slot | Name | What it does |
|---|---|---|
| Basic | Mirror Shard | A ranged light bolt (55,000), 100% Attack. |
| Passive | Reflection | A mirror clone: 35% of her max HP, her Defence, **no time limit**. It follows her within 120,000. It shoots for 40% of her Attack and casts its own Moonlit Mend (50%) and Kindred Charm, each on its own cooldown. It never casts Mirror Step or the ult. A dead clone comes back at her side 15 s later. |
| Passive | Guardian Angel (30 s) | Dropping below 50% HP knocks back enemies within 50,000 and shields her for 15% max HP (3 s). |
| Passive | Mirror Swap (90 s) | A hit that would drop her under 10% with her clone alive holds her at 1 HP and swap-locks her with the clone. Then they trade places, she's sealed for 1 s, and she revives at 40% HP. |
| S1 | Mirror Step (14 s) | **On a teammate within 120,000 of the clone:** the teammate and the clone swap. **On a point within 120,000 of her:** the clone dashes there, then she swaps with it. |
| S2 | Kindred Charm (12 s) | A teammate within 70,000: +25% attack speed, +15% move speed, +10% Attack and Magic Power for 5 s. |
| S3 | Moonlit Mend (10 s) | Heals a teammate within 60,000 (or herself) for 80 + 60% Attack. |
| Ult | Last Reflection (120 s, automatic, global) | While it's ready, every teammate but her is warded. The first one who'd drop under 10% HP is sealed for 2.5 s: untargetable, unseen, unable to act. Then they revive at **full HP** with Kindred Charm's buff, a 20% max-HP shield, +30% Defence and CC immunity for 4 s. One teammate per use. |

### Every swap takes time: the lock-on
- A swap takes **0.5 s**. When it's triggered, it locks on to both sides (the clone, and the teammate or Kagami).
- **Nobody is rooted.** When the time ends, they trade the places they hold *at that moment*, even if one walked or was knocked past the swap's range. The range is checked only when the swap is cast.
- A thin mirror thread between them shows the swap coming.
- **The stun:** every swap stuns enemies within 50,000 of where the clone lands for 1 s.
- **Cancel:** if a side dies, or the teammate becomes untargetable, the swap is cancelled and half of Mirror Step's cooldown is refunded.

## AI
In priority order, each tick:
1. **Heal:** Moonlit Mend on the lowest teammate (her included) under 60% HP.
2. **Rescue:** Mirror Step a teammate who is low (35%, moved up to ±10 by Judgement) with an enemy champion within 40,000. Only when the clone's spot is safer than theirs, so nobody is swapped into a worse fight.
3. **Escape:** dived (an enemy champion within 25,000) and hurt: Mirror Step to the clone when it stands somewhere safer, else to a point toward home.
4. **Buff:** Kindred Charm on the hardest hitter who is fighting.
5. **Walk:** keep within 45,000 of the teammate she guards (the hardest hitter nearby). The engine plays her basics and the rest of her walking.

**The clone:**
- **Shards:** champions first; minions only when no enemy champion is near her or the minion is near her. Never towers.
- **Casts:** its own Mend on the most hurt teammate near it, and its own Charm on a fighting teammate near it.
- **Position:** it keeps a step behind her, away from the nearest enemy (or toward home). After a swap into a fight it holds its spot for up to 3 s while it has an enemy champion to shoot.

## Code
- **Core:** `native/tfm2_custom_ai/src/kagami_core.rs`. It's deterministic: a Frame in, Effects out. The Lab runs this same file.
- **Bridge:** `kagami.rs`.
  - **The clone:** a summoned unit (`spawn_unit`, the base `priest` body until Codex's art) with move speed 1 and attacks and skills blocked. The core glides it (`motion::glide`) and deals its shards. It carries `kg_clone:<owner>`.
  - **Swaps:** two `entity_set_pos` calls through `place::safe` (allowlisted in `motion_guard_tests.rs`).
  - **Wards:** `kg_ward` (the ult) and `kg_self` (Mirror Swap) are `undying` buffs, renewed a few ticks at a time.
  - **Seal:** the engine's banish plus `kg_sealed`, so every Kagami's frame sees it. **Revive:** `entity_set_hp`, plus `kg_revived` and a shield.
  - **Walk:** `kg_go:x:y`, read by the shared input AI in `steve.rs` (walks only).
- **Tests:** `kagami_tests.rs` (core and glue). The mock host now records banishes and knockbacks.
- **Lab:** `native/kagami_lab` builds `editor/kagami.wasm`; the page is `editor/kagami.html`. It's a sandbox: dive the carry or her, hurt either, kill the clone, and toggle the AI and the ult. With the AI off, click to cast.

## Art
Until Codex's sheet arrives, her body is Thalassa's sheet. Her effects are bound to existing Thalassa effects (`tools/generate-kagami.py`, `PLACEHOLDER`), and the clone is the base `priest` body. See `docs/kagami-art-handoff.md`.

## Known limits
- **Ult while she's dead:** Last Reflection wards nobody while she's dead. A teammate already sealed still revives.
- **Clone body:** the clone's placeholder body slides when it moves (it's glided, not walked).


## Codex follow-up, 15 November 2026

Approved outlined original body, Reflection and silver/lavender effects are integrated; all thirteen athlete decisions, seal/swap Global Flash guards and the actual-map playable Lab are included. Kit numbers are unchanged. See [the art/Lab review](kagami-art-lab-review.md) for previews, hashes, verification and live limitations. Earlier placeholder notes describe the original PR head. Installation and career migration remain separate.
