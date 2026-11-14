# Round 174: Kagami, the Mirror Priestess (native 0.10.108, tfm2_custom 0.2.111, `kagami-1.0`)

**Reihan:** "I have an idea where the ult is a global revive: when a teammate is on the verge of low, it triggers a revive, and the friend becomes invulnerable and can't be hit, can't hit, can't move, can't skill, then revives at full HP."

**Reihan's choices along the way:**
- **A new support champion**, with the ult triggering **automatically**: a 2.5 s seal and a 120 s cooldown.
- **A clone as a passive:**
  - it has unlimited lifetime;
  - it basic-attacks for less damage than hers;
  - it can heal, and **can also cast the buff**;
  - it can't ult.
- **A skill that teleports the clone to trigger a swap**, which also gets an endangered teammate out.
- **The swap:**
  - "no 1 HP, but when swapping it triggers a stun area" (50k radius);
  - "any swap needs time to swap… lock both the clone and the targeted ally";
  - "no root, but lock on, so even if the target or the clone walks away they still swap".
- **Her own passives:**
  - **Guardian Angel**: below 50% HP, an area knockback and a shield;
  - **Mirror Swap** back as her self-save.
- **The revive** comes "with buffs and armor and anti-CC".
- Kindred Charm is the ally power-up.

## Kagami `kagami-1.0`
The full kit and AI are in `docs/kagami-guide.md`. In short:
- **Reflection:**
  - **The clone:** 35% of her HP and her Defence, with no time limit;
  - **Fighting:** it shoots for 40% of her Attack, and casts its own Mend (50%) and Charm;
  - **Death:** it comes back 15 s after dying.
- **Guardian Angel (30 s):** crossing below 50% HP knocks back enemies within 50,000 and gives a 15% shield.
- **Mirror Swap (90 s):** at 1 HP with a live clone, she's swap-locked with the clone, trades places, is sealed 1 s, and revives at 40%.
- **Every swap:**
  - **The lock:** a 0.5 s lock-on to both sides, with no root. They trade the places they hold when it ends.
  - **The stun:** a 1 s stun within 50,000 of where the clone lands.
  - **Cancel:** a cancelled swap refunds half of Mirror Step's cooldown.
- **Skills:**
  - **Mirror Step (14 s):** a teammate swaps with the clone, or the clone dashes to a point and she swaps with it;
  - **Kindred Charm (12 s):** +25% attack speed, +15% move speed, +10% damage for 5 s;
  - **Moonlit Mend (10 s):** 80 + 60% Attack.
- **Last Reflection (120 s, automatic, global):**
  - **The ward:** every teammate but her is warded (`undying`).
  - **The seal:** the first one dropping under 10% is sealed 2.5 s (banish).
  - **The revive:** at full HP with Charm, a 20% shield, +30% Defence and CC immunity for 4 s.

## Implementation
- **Data:** `rules/kagami.json` feeds `tools/generate-kagami.py`. It generates `kagami_data.rs`, `tfm2_custom_kagami.data_champion`, the tooltips, `editor/kagami-data.js` and the roster entry; the roster is regenerated (87 champions).
- **Core:** `kagami_core.rs`. It's deterministic, and the Lab runs the same file.
- **Bridge:** `kagami.rs`.
  - **The clone:** a summoned unit that the core glides and fires for.
  - **Swaps:** two `entity_set_pos` calls through `place::safe` (the `motion_guard_tests.rs` allowlist gets `kagami.rs`: 2).
  - **Wards and seal:** undying wards renewed a few ticks at a time; the seal is the engine's banish plus `kg_sealed`.
- **Wiring:** `lib.rs` (the passive, Mod Power), `steve.rs` (her `kg_go` walk, walks only), the bench roster, and the roster count test (86 → 87).
- **Mock host:** `glue_tests.rs` now records `entity_banish` and `entity_knockback`.
- **Lab:** `native/kagami_lab` builds `editor/kagami.wasm`; the page is `editor/kagami.html`, registered in `editor/server.js`.
- **Art:** placeholders bound to Thalassa's sheet and the base `priest` body. The request is in `docs/kagami-art-handoff.md`.

**Caught while building:**
- **Tags:** the Mod Manager test rejected the tags `Ranged`/`Support`, which the game would have skipped. They are now AD/Range/Heal/Shield.
- **The Lab:** it panicked on the first log effect (serde can't write an internally tagged newtype of a string). `Effect::Log` is now a struct variant, with a test that every effect serializes.

## Verification
- **Native tests:** `cargo test --release` gives **1,300 passed**, 0 warnings, including **23 Kagami tests**:
  - core: the clone, the swaps (the lock-on with no root, cancel and refund, the stun), Guardian Angel, Mirror Swap, Last Reflection, and the AI;
  - glue: the clone body, the swap and tower-safe stun, the ward, seal and blessed revive, charm, knockback and shots, and a whole rescue through the bridge.
- **Lab:** `kagami_lab` passes (transport, a rescue through JSON, and every effect serializing). A headless run of the Lab page showed Charm, Mend from her and the clone, a rescue swap with the stun, then Last Reflection sealing the carry, with no page errors.
- **Bench:** Kagami at about **6.5 µs** per tick (Kicker 6.4). The clone lookup goes straight to the known id.
- **Other checks:**
  - generators: Kagami, Kicker, Akane, Kratos, Wright, Thalassa, Grimhald and the roster are all current;
  - Mod Manager tests: 15 passed.
- **DLL:** the Windows DLL reports **0.10.108**.
- **Not checked here:** how the summoned `priest` clone looks and moves in the real game, and whether a banished ally keeps the `kg_sealed` buff visual.


## Codex follow-up, 15 November 2026

Approved outlined original body, Reflection and silver/lavender effects are integrated; all thirteen athlete decisions, seal/swap Global Flash guards and the actual-map playable Lab are included. Kit numbers are unchanged. See [the art/Lab review](kagami-art-lab-review.md) for previews, hashes, verification and live limitations. Earlier placeholder notes describe the original PR head. Installation and career migration remain separate.
