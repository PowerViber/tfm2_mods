# Round 164: Akane joins the game (native 0.10.98, tfm2_custom 0.2.101)

Reihan: "art is done now put her in the game." (Round 163 is the Fight Sense docs from the global-AI chat. They have no release and no game effect.)

## What's released
- **Akane**, the original crimson katana assassin, rules **`akane-1.1`**:
  - my kit and her own AI (`docs/akane-guide.md`);
  - Codex's approved body, weapon and skill art;
  - the Lab on the verified game map, with stock situations.
- **Codex's range and travel revision (`akane-1.1`, approved by Reihan):**
  - Kunai Flip: the flip is 30,000 over 0.5 s; the kunai flies 120,000 at 3,000 a tick (a delayed flight); the dash moves at 3,000 a tick.
  - Crimson Night: the dash reaches **240,000**, at 4,000 a tick.
- **Versions:** native 0.10.98 and tfm2_custom 0.2.101, in all four places. The Windows DLL is rebuilt and reports 0.10.98, with 0 warnings.

## Akane at level 10
| | HP | Attack | Sample burst on a 2,000 HP bruiser (before armor) |
|---|---|---|---|
| **Akane** | about **1,615** | about **180** | Kunai 138 + slash 184 + Crimson Edge (bleeds about 144, burst about 267 with vulnerable, crimson bleed 60) + Night slash 262 + about 5 basic attacks: **about 2,000**, everything she has |
| Kratos | about 2,000 | about 175 | about 1,120 |
| Contender | about 2,000 | about 228 | |
| Rally | about 1,950 | about 198 | |

She's the frailest of the four: about 20% less HP, and Mod Power gives her no HP or shield. With every skill up, she can kill a bruiser. That fits "fast and deadly, easy to kill".

## Flags for Reihan
- **Reach:** a 120,000 kunai plus a 240,000 night dash (hidden by the darkness) lets her pick off targets from very far away. If she feels unfair, cut the night dash range first (`night.dashRange`).
- **Resets:** orbs reset the Kunai every time. Crimson Edge resets at most once every 30 s, which limits the chain.

## Verification
- `cargo test --release`: **1,193 passed**, 0 failed, 0 warnings. Codex's revision updated my tests for the delayed kunai.
- The `akane_lab` transport tests pass. The packaged `editor/akane.wasm` is Codex's `akane-1.1` build, made from the same commit as the core and rules. A rebuild here is the same revision but not byte-identical (a different toolchain).
- The generators (akane, roster) are current, and the Mod Manager load test passes.
- **Not run here:** Codex's browser tests and art verifiers (`test-akane-lab`, `test-akane-art`, `verify-akane-art`) now need the local `editor/game-reference/` cache (extracted from the installed game, not in git) and Edge. They run on Reihan's PC; their recorded runs pass (see `docs/akane-art-review/`).

## Live acceptance (pending)
- **Dismiss:** that it really hides her and makes her invulnerable and move-only; and the size of its heal when she has healing reduction on her.
- **Kunai:** the delayed flight, and the hop over walls.
- **Crimson Edge:** the finale teleport, and the knock-up.
- **Crimson Night:** that it hides her team, and its long dash.
- **Her AI:** the walks it asks for through the shared input AI.
- **Balance.**
