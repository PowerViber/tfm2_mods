# Kagami, the Mirror Priestess (`kagami-2.0`)

A support built around **saves**. Three mirror clones guard the champions in most danger. Each clone is its champion's save point. Every swap stuns the danger spot for 2 s, and her ult is the global backup save for anyone without a clone.

Category **Util**, tags AD / Range / Heal / Shield. Numbers live in `rules/kagami.json`; `tools/generate-kagami.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies).

## Kit
| Slot | Name | What it does |
|---|---|---|
| Basic | Mirror Shard | A ranged light bolt (55,000), 100% Attack. |
| Passive | Reflection | **Three** mirror clones, each with 25% of her max HP, her Defence and **no time limit**. Each **guards one champion** on her team, her included; several can guard the same one, picked automatically by danger. There's **no range limit**: a clone more than 60,000 from its champion catches up at dash speed. Each shoots for 35% of her Attack and casts its own Moonlit Mend (50%) and Kindred Charm. They never cast Mirror Step or the ult. A lost clone comes back at her side **12 s later, each on its own timer**. |
| Passive | Clone Save | A guarded champion is warded. A hit that would drop them under 10% holds them at 1 HP and swap-locks them with their nearest guarding clone. Then they trade places (with the 2 s stun), and the champion gets back to **30% max HP** with a **20% max-HP shield for 4 s**. The clone is **spent**. |
| Passive | Guardian Angel (2 charges, 30 s each) | **Any** champion on her team crossing below 50% HP spends a charge. Enemies within 50,000 of **that champion** are knocked back, and they get a 15% max-HP shield for 3 s. The same champion can get it only once every **30 s**. |
| S1 | Mirror Step (two separate cooldowns: teammate 14 s, herself 14 s) | **On a teammate (anywhere):** they swap with her **safest** clone. **On a point within 120,000 of her:** her **nearest** clone dashes there, then she swaps with it. |
| S2 | Kindred Charm (12 s) | A teammate within 70,000: +25% attack speed, +15% move speed, +10% Attack and Magic Power for 5 s. |
| S3 | Moonlit Mend (10 s) | Heals a teammate within 60,000 (or herself) for 80 + 60% Attack. |
| Ult | Last Reflection (120 s, automatic, global, **the backup**) | While it's ready, every teammate except her with **no clone save ready** is warded. The first one who'd drop under 10% HP is sealed for 2.5 s: untargetable, unseen, unable to act. Then they revive at **full HP** with Kindred Charm's buff, a 20% max-HP shield, +30% Defence and CC immunity for 4 s, and **enemies within 50,000 are stunned for 2 s**. |

### Which champion each clone guards
- **When:** every 3 s, and at once when a new champion has an enemy champion close.
- **Score:** each champion on her team gets a need score: the danger around them, how low they are, and their Attack. Each clone already on them lowers it, so a second clone goes to a champion only under heavy fire.
- **Hysteresis:** a clone switches only for a clearly better pick (`clone.switchMargin`), so they don't flicker.
- **Out of fights:** one clone on her and two on the hardest hitter.

### Every swap takes time: the lock-on
- A swap takes **0.5 s**. When it's triggered, it locks on to both sides (a clone, and the teammate or Kagami).
- **Nobody is rooted.** When the time ends, they trade the places they hold *at that moment*.
- A thin mirror thread between them shows the swap coming.
- **The stun:** every swap (Mirror Step and Clone Save) stuns enemies within 50,000 of where the clone lands for **2 s**.
- **Cancel:** if a side dies, or the teammate becomes untargetable, the swap is cancelled and half of that Mirror Step's own cooldown is refunded.
- **Concurrency:** several swaps can run at once, one per clone.

## AI
Her own casts, in priority order (the athlete values still shape the thresholds):
1. **Heal:** Moonlit Mend on the lowest teammate (her included) under 60% HP.
2. **Rescue** (the teammate cooldown): Mirror Step a teammate who is low, with an enemy champion close, when one of her clones stands somewhere safer than they do.
3. **Escape** (her own cooldown): dived and hurt, Mirror Step to her safest clone in reach, else toward home.
4. **Offensive swap and buff:** an aggressive swap onto a fight her team can follow, and Kindred Charm on the hardest hitter who is fighting.
5. **Walk:** stay near the teammate she guards. The engine plays her basics.

**The clones:**
- **Shards:** champions first; minions only near the champion they guard. Never towers.
- **Casts:** each casts its own Mend on the most hurt champion near it, and its own Charm on a fighting teammate near it.
- **Position:** a step behind the champion it guards, away from the nearest enemy. Clones on the same champion fan out. After a swap into a fight a clone holds its spot for up to 3 s while it has an enemy champion to shoot.

## Code
- **Core:** `native/tfm2_custom_ai/src/kagami_core.rs`. It's deterministic: a Frame in, Effects out. The Lab runs this same file.
- **Bridge:** `kagami.rs`.
  - **The clones:** three summoned units (`spawn_unit`, the base `priest` body under Codex's Reflection art) with move speed 1 and attacks and skills blocked. The core glides them (`motion::glide`) and deals their shards. Each carries `kg_clone:<owner>:<slot>`; a spent clone is set to 0 HP.
  - **Swaps:** two `entity_set_pos` calls through `place::safe` (allowlisted in `motion_guard_tests.rs`).
  - **Wards:** `kg_ward` (the ult) and `kg_clone_ward` (Clone Save) are `undying` buffs, renewed a few ticks at a time.
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
