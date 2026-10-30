# Contender: required host proof — release held

This is the first milestone of the approved Contender plan, **not an implemented/releasable champion**. The canonical kit is frozen in `rules/contender.json`. No Contender definition, Mod Power line, final artwork, Boxing Lab, migration, production native version or production DLL has been added or changed.

The required host proof has **not passed**. Repository/fake-host tests cannot certify the real executable's navigation, global effects, camera or shield callback behaviour. In accordance with the plan, full integration and release are held. There is no on-map substitute, frozen-target duel, cast injection or separate native damage simulation.

## Checked baseline and evidence

- Implementation base: main `777204f37743e84cef0a1ea86680c1230079cdce` (PR #52); production native 0.10.78. PR #53's separate Minato fix was still unmerged when this branch was created and is not copied into this work.
- Installed target: `C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2\TeamfightManager2.exe`, PE x86-64, 93,235,712 bytes, SHA-256 `d678e01285b7a9edba2c7ee1067cea93a54f979d769eaae9f0d8309d18ff7d30`.
- REA 6.8.0 Rust metadata identity: `ev_51c45c3380a1fc6d5516e40ce7d278355cd0dc1cd989eed9845c5954132e4d74`; summary `ev_a1673455ab94c7e4f185ad8edd17fb7da47841515b1e29dc8b69f2760f2bff8a`. This does not recover gameplay functions.
- REA GNU objdump symbol inventory: `ev_362be957a5b6cb50c81344e73deca13017a6368e10167a26eafd66c017ebb927`. The symbol table reports no symbols. The target hash remained unchanged. Complete retained evidence is in `rea-symbols-evidence.json`.
- REA's Ghidra provider is not configured (`GHIDRA_INSTALL_DIR` absent). No native function analysis, game execution, installed-file patch or runtime interception was performed. No binary session was opened.

The supported API provides position mutation and rendering-camera positioning, but does not promise off-map movement or champion-follow. The checked passive's damage notification is post-damage and has no pre-impact veto. Custom-action gates do not wrap arbitrary stock casts. Banish is documented as untargetable, invisible and input-locked. Map customization explicitly retains original offline navigation/visibility geometry. These observations leave the eight gates in `gate.json` unresolved; they are not proof that every possible implementation is impossible.

Primary API reference: <https://raw.githubusercontent.com/teamsamoyed/TeamfightManager2Mod/main/docs/stable-api-reference.md>. Yin's [official spotlight](https://www.youtube.com/watch?v=M-VppZdcgXY) is the isolation reference; [community interaction notes](https://mobile-legends.fandom.com/wiki/Yin) describe return positions. Neither establishes MLBB's internal off-map implementation.

## Experimental probe

`experiments/contender-host-probe` is a separate native crate, outside the production `mods/` and `native/` build roots. Its default build registers **no hooks or champions**. Exactly one explicit feature enables an experimental match hook:

| Feature | Measurement |
|---|---|
| `off-map-probe` | At tick 300, records the lowest-ID living champion on each team, teleports both outside the battlefield, reads positions back, then observes their ordinary engine AI. Returns survivors to exact recorded coordinates after 480 ticks, a death, or escape/clamping. |
| `dodge-probe` | At tick 300, applies a stationary 12-tick banish + 100% reduction + CC-immunity **candidate** to the lowest-ID living blue champion. Records targetability, HP, shielding and CC before/during/after. This is not the moving Slip skill. |
| `shield-probe` | At tick 300, grants that champion a 1,000-point, 90-tick diagnostic shield and observes HP/shield changes. Aggregate changes do not establish attacker identity or a fully absorbed hit callback. |

All features preserve ordinary engine attacks/casts and add no damage. Off-map return coordinates are duplicated as scalar markers on both participants, with no global mutable combat state. Readback failures trigger rollback; failed returns retain the ledger for retry. Death of the passive owner cannot strand the other participant because cleanup runs in the match hook. Missing marker support rejects entry. This probe does **not** enforce ring boundaries, reserve construction footprints or filter global effects: those capabilities must still be proven, not simulated.

Raw bounded JSONL observations go to `mods/tfm2_contender_probe/logs` beside the running game. They include seed, simulation origin, tick, participant positions/HP/shields/control and slot cooldowns. Simulation pointers are diagnostic file labels only and never influence commands. Cooldown movement is not proof of delivered damage; distinguish selection, accepted execution and actual contribution.

Build and verify without installation:

```powershell
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --locked --manifest-path experiments/contender-host-probe/Cargo.toml --features off-map-probe
rtk proxy python tools/build-contender-probe.py --repeat
rtk proxy python tools/check-contender-host.py
rtk proxy python tools/check-contender-host.py --require-release
```

The last command deliberately exits **3** while release is held. Package output lives only under the ignored experiment `target/packages/`. The builder never installs, enables mods or edits a career. The experimental package is 0.1.0; production versions/dependencies remain unchanged, avoiding a collision with other release work.

## Live proof procedure — pending, not performed

Use only an isolated disposable game installation/config and a fresh throwaway match, with the game closed before staging a probe package. Enable one probe at a time. Do not enable this diagnostic in a real career or install it through the normal production package. Installation is deliberately separate from this PR.

1. Observe stock melee/ranged champions and custom champions with their original kits. Verify off-map movement, basic hits, skill delivery and targetable summons without steering/cast injection. Check an outside champion continues fighting.
2. Select both duelists using normal champion-follow; switch to an outside champion; replay and seek across entry/exit. Camera setter existence alone is insufficient. Test an optional Ring View only if ordinary follow fails.
3. Exercise targeted, area, aura, global, periodic, healing, shielding and control effects across memberships. Distance is not an interaction filter. Require real host isolation support before implementing Main Event.
4. Verify timeout, either death, missing entity, rejected/clamped transfer, interruption, exact return and independent simulations. Verify real construction routes can reserve both return footprints without obstructing unit movement.
5. For the dodge candidate, schedule targeted/AoE/physical/magic/fixed/DoT and new control on ticks 299–313. Existing effects keep their clocks. Test before/inside/after the exact interval and check no immunity tail.
6. For shielding, verify a shield-only champion hit has reliable attacker/impact identity. HP/shield polling alone cannot satisfy Counterpunch attribution.

If any essential property cannot be provided by supported hooks, document the required host extension and keep release held. Do not patch the installed executable or bypass the stable ABI. Required extensions may include arena-aware navigation/camera, cross-membership interaction gates, construction placement validation and attributed impact receipts including shields.

## Remaining implementation after the gate

The full boxing core, 13-stat planner, Mod Power registration, arena membership/boundaries, reserved returns, cooldown acceleration, Boxing Lab/native parity, approved body/effect art, 24 development/12 holdout scenarios and live balance acceptance remain pending. The prototype tests and package checks do not replace those acceptance requirements.

## Automated results

- Off-map feature: **20 tests passed**, including real SDK wrappers connected to a fake host. Dodge/shield features each pass 12 tests; the inert build passes eight. These suites reuse tests; they are not 52 distinct gameplay scenarios. All four builds have zero compiler warnings.
- Rust formatting passes. Conflicting probe features are rejected at compilation. Clippy was unavailable in the installed GNU toolchain; no toolchain components were installed.
- Two independent release builds per probe produce matching DLL bytes, using rustc 1.98.1 / GNU and the baseline's `SOURCE_DATE_EPOCH`. Exact hashes and source hashes are in `builds.json`. An earlier timestamp-only reproducibility difference was corrected before these final results.
- Mod Manager verifies the **unchanged production package** in a disposable fixture: **836 files compared, zero differences/missing**, exit 0. It does not verify experimental probe compatibility. The unchanged production DLL SHA-256 is `2001b1727b64b8de1f980aaab14b410a12abb55517456bd6fe75908e5641371d`.
- `--require-release` exits **3**, as required while all eight mandatory host proofs remain unproven. The installed game executable has not changed.
- No live game was launched, no experimental package was installed/enabled, and no real career was migrated. Test fixtures must retain the standard 60-tick simulation clock.
