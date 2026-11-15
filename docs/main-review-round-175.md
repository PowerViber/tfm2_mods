# Round 175: Kagami 2.0, three guarding clones (native 0.10.109, tfm2_custom 0.2.112, `kagami-2.0`)

**Reihan, after round 174:** "she is too weak". What they asked for:
- **Three clones** with no maximum range. Each follows a certain champion, and one champion can have more than one.
- **Each clone is a save point** for its champion. Instead of the ult, the clone swaps in, then heals and shields the champion it swapped out.
- **The swap stun becomes 2 s.**

**Reihan's answers on the plan:**
- **Spent clones:** a save spends the clone, and every clone has **its own 12 s timer**. For example, clone 1 dies, and 4 s later clone 2 dies: clone 2 has 12 s left while clone 1 has 8 s.
- **The ult** is the backup save, and its revive stuns the area.
- **Assignment** is automatic, by danger.
- **Numbers:** 25% HP and 35% damage per clone; a save restores 30% HP and gives a 20% shield.
- **Guardian Angel** works on teammates, with 2 charges, and the same champion has its own 30 s cooldown.

## Kagami `kagami-2.0`
- **Reflection: three clones.**
  - **Stats:** each has 25% of her HP and her Defence, shoots for 35% of her Attack, and casts its own Mend (50%) and Charm.
  - **Guarding:** each guards one champion, her included, and several can guard the same one. There's no range limit: past 60,000 a clone catches up at dash speed.
  - **Respawn:** each comes back 12 s after it's lost, on its own timer.
- **Assignment:** a need score (danger, low HP, Attack, minus the clones already there), re-run every 3 s and at once when someone new is in danger.
  - **Hysteresis:** a margin, so the clones don't flicker.
  - **Calm:** one clone on her and two on the hardest hitter.
- **Clone Save** (replaces Mirror Swap; works for any guarded champion):
  - **The ward:** `kg_clone_ward` (`undying`).
  - **The save:** a 0.5 s lock-on at 1 HP, then the swap with the **2 s stun**, then 30% max HP and a 20% shield for 4 s. The clone is spent.
- **Guardian Angel:**
  - **Who:** any champion on her team crossing below 50% HP.
  - **Effect:** a knockback away from *them* and a 15% shield for them.
  - **Limits:** 2 charges (30 s each), and 30 s per champion.
- **Mirror Step:** a teammate anywhere swaps with her safest clone; her own swap uses her nearest clone. It keeps the two 14 s cooldowns and the half refund.
- **Every swap stuns for 2 s** (was 1 s).
- **Last Reflection:** it wards only teammates with no clone save ready. Its revive stuns enemies within 50,000 for 2 s.

## Implementation
- **Core (`kagami_core.rs`):**
  - **Clones:** `clones: Vec<CloneSlot>` (id, guard, own timers and cooldowns per slot), and the frame's `clones: Vec<CloneView>` (with `slot`).
  - **Swaps:** `swaps: Vec<PendingSwap>`, so several run at once; `SwapKind::Save` replaces `SelfSave`.
  - **Passes:** `assign_clones` (the new assignment), `run_clone_saves`, and the team `run_angel` (`angel_charges`, `angel_ready`).
  - **Effects:** `SpawnClone { slot }`, `DespawnClone`, `CloneMove/CloneShot { clone }`, `Knockback { from }`, `CloneWard`.
- **AI (`kagami_ai.rs`):** its gating moves to `stepping()` and `self_swapping()`. Offensive swaps check the clone that would serve them. The single-clone `clone_spot` is gone (clones follow their own champion).
- **Bridge (`kagami.rs`):**
  - **Clones:** three tagged units `kg_clone:<owner>:<slot>`, and `find_clones`, which looks up the known ids first and searches only for a missing clone.
  - **Despawn:** it sets HP to 0.
  - **Body leases:** one per slot (Codex's Reflection art).
  - **Flash guards:** they cover every swap side.
  - **Knockback:** it pushes away from the protected champion. The FX budget goes from 12 to 18.
- **Lab:**
  - `kagami_lab` validates `clones`;
  - `editor/kagami-lab.js` runs three clones, draws each one's guard line, and lists each clone's champion or timer, all pending swaps and the Angel charges;
  - the parity fixtures use three clones;
  - the wasm is rebuilt.

## Verification
- **Native tests:** `cargo test --release` gives **1,311 passed**, 0 warnings, including **34 Kagami** core, AI and glue tests:
  - three clones and per-clone timers (two lost 4 s apart come back 4 s apart);
  - per-clone Mend and Charm;
  - the no-range catch-up;
  - danger assignment and stacking;
  - no flicker;
  - Clone Save for a teammate and for her (the 2 s stun, 30% and 20%, the clone spent, no ult);
  - Guardian Angel's charges and per-champion cooldown, including A → B → A;
  - Mirror Step's clone choice;
  - the ult as the backup, with the revive stun;
  - the AI's heal, rescue and escape;
  - the Flash guard and art glue.
- **Lab crate:** `kagami_lab` passes (3 tests).
- **Bench:** about **9.3 µs** per tick on the mock host (was 6.5 with one clone).
- **Other checks:** the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports **0.10.109**.
- **Not run here:** `tools/test-kagami-lab.cjs`, `tools/test-kagami-parity.cjs` and `tools/package-kagami.py`. They need Codex's Windows set-up (Edge, the extracted game map, `rtk`). Their fixtures and checks are updated for three clones.

## Install (Reihan)
1. Merge the PR and pull.
2. Run the Mod Manager and press **Install**.

Kagami is in exhibition matches and new careers; existing careers can't add new champions.
