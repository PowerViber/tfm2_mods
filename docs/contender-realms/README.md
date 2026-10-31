# Windows realm investigation — evidence and handoff

Date: 2026-10-31. Branch: `codex/contender-windows-bridge`, based on main
`5c7a9fc` (native 0.10.81). Claude's proposed Stage 1 bridge/probe was not present
in the inspected main checkout. This work supplies Windows discovery and an
independent observer, leaving his controller work separate.

**A true realm is not enabled or proven.** The production stopgap remains
unchanged and retains its limitations. No runtime isolation detour, global
invisibility workaround or damage restoration is introduced by this branch.

## Target and provenance

The installed executable was inspected locally. The read-only collector also
matched all 89 SDK pointers and sampled entry bytes against an owned disposable
process. That means loaded-byte identity, not observed combat execution.

| Identity | Value |
|---|---|
| Whole executable SHA-256 | `d678e01285b7a9edba2c7ee1067cea93a54f979d769eaae9f0d8309d18ff7d30` |
| PE timestamp / image size | `0x6ae1b2c7` / `0x58f8000` |
| `.text` VirtualSize file bytes SHA-256 | `d07606ef55d7117dfa5c28c44c2ed52cbdf0261303db09f939154ab754b7654b` |
| `.text` RawSize file bytes SHA-256 | `203668d078009b5636d3babbe8ce6b0a61a09c4123d2f02c317589949c0a3aec` |
| Installed SDK | base 0.7.0; size-prefixed SimVtableV1: 720 bytes |
| SDK table RVA | `0x4262988` |
| Tools | iced-x86 1.21.0; Ghidra 12.1.4; local REA 6.8.0 |

Whole-file and section hashes are deliberately separate. Section file hashes
are not ASLR-relocated live-memory hashes. Every address below is an RVA for
this build only. No executable or bundled game assets are committed.

## Findings ledger

| Finding | Evidence | Confidence and remaining limits |
|---|---|---|
| Unique SDK table matches the installed layout and loaded pointers | `windows/realm_probe.json`: SDK source hash, 89 entries, live comparisons | Observed bytes. Table membership does not prove execution or stock coverage. |
| SDK visibility wrapper `0x33cb210` uses internal trait slot `+0xf8` | Decoded instructions and focused pseudocode | Static, not a validated visibility hook. Direct-call-only tracing misses this dispatch. |
| Six unbound internal tables share visibility `0x16aa0c0`, entity lookup `0x16ad6e0`, queue `0x16ad0d0` | Table candidates and focused pseudocode | Receiver not yet observed in a real match. One other candidate is a forwarding table. |
| Internal visibility candidate reads a team-indexed entity tag | `0x16aa0c0`: Sim entity vector `+0x730`, lookup `+0x748`, stride `0x6b8`, tag `entity+0x38+team*0x18` | Static interpretation. No observing champion parameter. Cannot safely implement per-realm visibility here alone. |
| Position setter directly stores entity coordinates | SDK `0x33cd200`, internal lookup slot `+0x210`, writes `+0x658/+0x660` | Static. Teleport/readback success says nothing about stock pathing, map bounds or collision. |
| Invisibility setter extends a timer with max(existing,ticks) | SDK `0x33cfd80`, field `+0x690` | Static. Passing zero does not clear a prior timer. Global hiding is not selective realm vision. |
| SDK damage constructs a dynamic effect rather than immediately subtracting HP | `0x33cb980` → helper `0x33d1140` → internal slot `+0x258`; table `0x4260600`, debug string `ModDamageEffect` | Static. Must validate queue receiver and execution order. |
| Damage effect method `0x1e130a0` calls a shared damage candidate `0x18c0480` separately for AD/AP | Focused pseudocode; `damage-callers.json` has 23 decoded call sites into the shared candidate | Promising broader coverage, still no observed stock impact. Private ABI and nested/proc behaviour unverified. |
| Shared damage candidate performs reductions, RNG-related work and effect callbacks before its final entity application helper | `0x18c0480` focused pseudocode; final helper candidate `0x167a710` | Inferred semantics. A future gate must precede these side effects; blocking only the final HP write would be insufficient. |
| `0x16ad0d0` appends 64-byte work records at Sim `+0xb228/+0xb230/+0xb238` | Focused pseudocode | Inferred queue layout; do not treat it as the final interaction gate. |
| Shield application bypasses the damage queue | SDK `0x33cebb0` appends a 56-byte layer to the entity's shield vector | Static. Shield/support paths need separate source/owner provenance. |
| Public CC wrapper converts an SDK descriptor then calls `0x33d0a10` | `0x33cabd0` pseudocode | Static. Does not prove all stock CC uses this path. |
| Runtime observer is a separate opt-in mod | Experiment source, tests, reproducible build report | Built. In-game code-mod permission is pending; no observed match callbacks or combat claims. |

`realm-hooks.json` records instruction bytes, bounded control-flow edges,
unresolved indirect calls, exact unwind ownership when available, ABI evidence
and unknowns per function. Prefix signatures are **not** safe detour recipes.
`ghidra-focused.json` distinguishes installed-SDK prototypes from private ABI
inference. Pseudocode is an analysis hypothesis, not recovered original source.

## Capability status

| Required native capability | Status | What must be demonstrated next |
|---|---|---|
| Per-actor AI/render context | Unresolved | Bound receiver and observing actor, scoped per simulation with nested-call restoration |
| Stock target filtering | Unresolved | Champions, minions and towers reject cross-realm targets before committing |
| Clean-floor navigation | Unresolved | Duelists walk across underlying battlefield walls; outsiders keep their original paths |
| Collision separation | Unresolved | Only duelists meet the ropes; outsiders cross without seeing or colliding with them |
| Pre-impact interaction gates | Candidate damage path only | Damage, CC, healing, shields and auras all gate before application |
| Projectile/field isolation | Unresolved | Existing and new projectiles/fields respect realm membership and source ownership |
| Rendering and selected-champion view | Unresolved | Actor-specific visibility, ring selection, replay seeking and independent rings |
| Concurrent-safe hook lifecycle | Not implemented | Reviewed relocation, installation, rollback and teardown before any backend enablement |

Native backend: **disabled**. Diagnostic runtime observations do not change
this status automatically. A `stopgap` status is a separate, explicitly limited
playtest capability and must not imply any of these native capabilities.

## Bridge requirements before a mutable prototype

Do not write a handwritten byte-count trampoline. Decode complete overwritten
instructions and relocate RIP-relative addressing, relative calls/jumps,
conditional branches and targets inside the relocated block. Reject unsupported
instructions or ambiguous ownership. Preserve the actual private calling
convention, registers, stack alignment, unwind and exception requirements.

Install/remove only with a verified concurrent-thread protocol. Account for
instruction pointers inside the patch span or trampoline, page-protection
restoration, instruction-cache flushes, partial failure and ownership when
another mod has changed the bytes. Matching the original first bytes is only
one admission check. No bridge is installed here because these conditions and
the required engine contexts are not yet proven.

A future capability contract must name each hook family's build digest,
expected bytes, observed receiver/actor/source context, stock/custom coverage,
thread/reentrancy behaviour and disposable fixture results. Enable `native`
only when the complete required set passes; one damage hook is insufficient.

## Local tooling and test scope

Ghidra's official 12.1.4 archive was verified against its release SHA-256.
The private Java/Python/tool environment changes no system PATH or MCP config.
Two REA whole-program attempts did not complete global analysis in the usable
window. The first hit the provider's 20-minute deadline; cancellation/cleanup
of the second was not confirmed through the provider, so verified owned
processes were stopped explicitly. No game process was killed in that cleanup.
The successful focused route uses `-noanalysis`, decoded seeds and a disposable
Ghidra project. Its report checks source-executable hashes and actual project
deletion. It does not claim complete global xrefs or function recovery.

The Python tests cover decoder false positives, leaf/unwind ownership, loops,
indirect dispatch and SDK table admission. Observer tests cover inert default
registration and null-host rejection. Independent feature builds are compared
byte-for-byte in `observer-builds.json`. These are diagnostic/tooling checks,
not realm gameplay acceptance. The isolated Mod Manager fixture passes against
841 unchanged production files with both observer files separately compared.
That manager checks the existing production pack; it is not a generic observer
ABI/host verifier. No real installation or career migration was performed.

## Runtime acceptance still pending

The disposable copy reached the game's native-code warning. It contains only
the diagnostic mod and an empty private profile. The observer has not been
enabled by the user and no match-observer file has been produced. Initial
checks found all 38 pre-existing real user-data files unchanged; final cleanup
also confirmed the same 38 files unchanged with no new files
(`windows/profile-integrity.json`). The owned disposable process was closed
before its REA filesystem instrumentation was detached. No owned game,
Ghidra or private REA analysis process is left running. The unrelated Autopsy
Java process was left untouched. The native-code permission remains pending;
continuation will restart the disposable copy.

After approval, first observe real match receivers and stock call stacks with
no gameplay gates. Then validate one family at a time: mutual visibility,
target enumeration, pre-impact interactions, projectiles, clean-floor
navigation/collision, and rendering. Use stock/custom opponents, outsiders,
turrets, minions, splash/support, summons and simultaneous rings. Separately
test exact returns, interrupted entry, deaths, spectator selection, seeking
and independent simulations. Compare a no-Main-Event run for identical
commands/state/RNG before calling the native bridge reliable.

Controller changes, stopgap removal and production release/version bumps are
intentionally left to the separately coordinated Stage 1 integration after
the required Windows evidence exists. Installation and real-career migration
remain separate from publication.
