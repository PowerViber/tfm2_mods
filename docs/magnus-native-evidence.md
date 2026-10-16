# Magnus native boundary investigation

17 October 2026. Read-only investigation using the user-requested REA workflow, supplemented by focused Ghidra disassembly. No game executable was patched. Decompiled control flow is treated as a hypothesis and is not reproduced here.

## Target and tools

| Item | Identity |
| --- | --- |
| Game | Teamfight Manager 2, version 0.6.4, Windows x86-64 PE |
| Executable size | 87,228,416 bytes |
| Executable SHA-256 | `a1d05045405a12e2bed182daeafcad8c8a40cf29ab68ba6ea5bfd30257439582` |
| REA | 6.5.0 |
| Ghidra | Portable 12.1.4 |
| JDK | Portable 21.0.12.1+1 |
| Ghidra download SHA-256 | `ddac49f903da9d5bac833e5cc79395098b9c33cfd3279be5f31bd00387d2d4db` |
| JDK download SHA-256 | `f9d6e191ab098c0d416e7d588a24420a8621cd2f4720dab2459b8b7b2d2d8b4e` |

No default Java/Node path or system client configuration was changed. REA identified Rust build metadata: evidence **`ev_f732eedfdff88cd390eb51531a70f3e08f4235ffe7cd7e2cc56f679bb1e2c17b`**, rustc path hash `23a3312d92a1c4ba0373f1e25277be20ba8bb28c`, file offset `60937556`. That evidence supports Rust identity, not recovered source symbols.

Full REA analysis/import timed out twice and one host restarted. A small supplementary Ghidra project used an immutable hash-checked copy, no full auto-analysis, and only selected boundary addresses. These focused results have no REA evidence ID and are recorded separately below. The first REA session closed normally; the restarted session's closure could not be acknowledged after its host disappeared. Final cleanup found no remaining owned REA helper or Java process. The focused project remains an ignored local investigation artifact.

## Address ledger (this executable only)

| Observed address / offset | Observation | Confidence / limit |
| --- | --- | --- |
| VA `0x143d53258`, raw `0x3d51858` | Exported simulation-vtable candidate, 720-byte shape matching shipped SDK ABI 10 | Boundary layout match, not full symbol recovery |
| Vtable slot 38, byte offset 304 → `0x142e1f150` | Supported `deal_damage` wrapper; checks read-only flag at context +0x18 and calls `0x142e24810` | Direct boundary/call observation |
| `0x142e24810` | Checks entities and constructs a payload carrying damage fields; dispatches through internal trait slot +600 (`0x258`) with zero delay and source/target IDs | Consistent with a deferred damage queue; internal semantic names are inferred |
| Vtable slot 81 → `0x142e22220` | Typed damage wrapper validates input, creates a typed payload and uses the same +600 dispatch; returns acceptance | Return value is not an applied-damage receipt |
| Payload vtable `0x143d50ef8` → `0x142e26430` | Examined payload function serializes data | It is not the damage-application function; it does not prove downstream combat semantics |

The [official API](https://raw.githubusercontent.com/teamsamoyed/TeamfightManager2Mod/main/docs/stable-api-reference.md) documents mitigation, statistics, items and kill credit for `deal_damage`. The mod retains that supported route. The typed variant does not solve synchronous acknowledgement; raw HP writes would bypass required combat behaviour.

## Runtime corroboration and implementation consequence

In the QA run, immediate HP/shield deltas were zero while subsequent ticks often showed loss and the completed match credited Magnus with champion damage. A bounded log snapshot contained 417 requests and 417 next-tick observations, of which 416 were positive. Their total observed loss was 16,075, but it includes unrelated attacks, healing, shields and non-champion targets. It is **not** an attributable Magnus damage measurement.

That corroborates deferred execution and invalidates the old immediate-zero interpretation. It does not establish an exact per-command callback. The adapter now exposes pending requests and unknown later observations. Only positive, correlated applied receipts advance historical damaging steps or pawn qualification. Native acceptance of those features remains open; the lab can supply exact deterministic receipts.

## Farming AI: verified scope

Repository inspection and official action API documentation show target eligibility and farming/jungle gates on individual skills. The existing player-input hook can observe the engine's selected inputs. It does not provide a supported mutable wrapper around every stock skill action.

Runtime observations showed stock skill 1/skill 2 orders targeting visible non-champion enemies on both teams, as well as existing custom kits with broad targeting. Therefore stock farming is present. The implementation fixes champion-only custom target lists and adds conservative ordinary-skill farming, without injecting or holding stock orders. This does not constitute full decompilation of the stock farming utility/scoring system; the complete stock scoring path remains unknown.

Raw game executables, save data, logs, Ghidra databases and pseudocode dumps are not included in the PR. [Delivery report and remaining acceptance gates](magnus-grandmaster-farming.md).
