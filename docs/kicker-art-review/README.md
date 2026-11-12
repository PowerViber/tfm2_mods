# Kicker appearance proposal / actual-map review

Based on open PR #80 head 2b7da261b1078269b484aa7ac59561fa64c1f023 (kicker-1.0, native 0.10.102, champion pack 0.2.105).

**Review-only.** No appearance/effects approval has been received for these new sources. No replacement body or effect enters the native pack, and gameplay, AI, rules, the DLL and Contender’s placeholder remain unchanged.

The proposed original street fighter wears a rust-red track jacket, charcoal cargo trousers, taped bare feet and a jade spray can. Source: built-in imagegen, eight whole-body appearance poses. Exact prompt is in prompts.json; body-source-v1.png is the generated source. AI generation is not deterministic. prepare-kicker-art.py reproduces extraction/downscaling, mirrored facings, atlas and comparison sheets. The concept uses one uniform anatomical scale, a 56×48 canvas and foot anchor (28,44); standing height is approximately 39 visible pixels.

The candidate review is editor/kicker-review.html; the playable shared-core Lab is editor/kicker.html. The Lab’s original isolated artificial wall grid and dots have a new actual-map view using the extracted map and stock champions/minions/camps/towers/objectives at the verified world scale. Eight mid-game scenario presets, follow/full-map views, collider overlay, editable walls and all thirteen profile controls are available. A changed wall grid is a custom fixture, not unchanged game terrain. Candidate poses are inspection drawings; full animations await appearance approval.

The Rust/WASM core remains authoritative for Kicker actions. The browser host still models glides, damage, kicks and returns. This update does not prove native undying, true-damage handling, kill attribution, real Flash use or stock targeting. It does not claim a full stock-engine simulation. The real-game checks listed in status.json remain pending.

Automated checks: three Kicker Lab tests pass. The browser review verifies loaded assets, 900-cell registration, all thirteen controls and stock actors without missing sprite keys. Native integration, animation completion, all-effect bindings, packaged-art budgets and actual-game acceptance remain separate gates.

Use tools/start-kicker-review.ps1 to open the isolated preview server on port 7306. The ignored game-reference cache is extracted locally from the installed game and is not redistributed as source art.

A disposable copy of the installed 0.7.2 game is prepared for the four requested host checks. Its process alone uses the existing filesystem-isolation harness; real career data and the installed mod configuration are not modified. The native code permission is deliberately left for the user to approve in the game UI. No match has been observed yet.

