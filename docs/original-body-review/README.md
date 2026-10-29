# Original body release evidence

Merged-main baseline: `02dba54` (PR #50). Isolated branch: `codex/original-character-bodies`.

## Artwork and approvals

Gojo, Frieren, Fern, Stark, DIO, David, V1, normal Minato, The World and KCM Minato are approved. Approval quotes and frozen source hashes are in `approval.json` and `source-manifest.json`. The user explicitly approved transparent padding to preserve game-size height, and a visual-only native KCM hook. Vader remains unchanged after two imagegen failures; see `generation-limitations.json`.

Exact official reference image URLs, dates and hashes are in `references.json`. Reference caches are excluded from Git; generated source sheets and prompts are checked in. Mechanical extraction, nearest-neighbour scaling and packing are reproducible from frozen images; AI generation itself is not deterministic.

Review `editor/original-body-review.html`: native 1x and 3x, dark/light terrain, both facings and every completed animation. Body-derived Fern/Stark layers, the Ghoul Stark carrier, The World's poses, David's tinted afterimages and Minato's teleport silhouettes use the approved bodies. Pure projectile, explosion, Domain, Stand-fist and other effect pixels/timings are preserved. Current Gojo gesture bindings are transparent compatibility slots; these remain unchanged and the body supplies its gestures.

## Minato presentation

`minato_body.rs` observes cooldown increases, position, visible facing information, control and the existing `minato_kcm` buff. It selects one normal/KCM frame layer, immediately removing its previous layer. Form changes preserve the pose clock; mirrored slots are explicit. Death emits one fixed-position animation.

Only current definitions opt into this passive and its transparent in-match base. The original visible sprite alias remains for embedded legacy definitions, editor starters and thumbnails. Existing definitions retain aura-only KCM until explicitly refreshed; no career was migrated.

The API has no current-action animation getter. Pose selection observes accepted-action cooldown transitions rather than changing the engine's action state. Host pose fidelity, invisibility/fog rendering, draft/HUD appearance and cleanup need actual-game inspection. The presenter issues no input, damage, control, movement, stat or RNG operations. Existing native gameplay modules are byte-identical; `lib.rs` only adds its module and passive registration.

Skill Test uses visible body starters and a visible KCM atlas when its existing buff activates. Embedded sprite/VFX copies are refreshed. Save/replay formats and old aliases remain available.

## Automated checks

- Native release suite: **745 passed, zero failed, eight diagnostic/benchmark tests ignored**, including four presentation-clock tests.
- **552 replaced body/derivative frames** and **554 unchanged non-body frames** in repacked atlases. Tags, counts, durations, symmetric padding and world foot anchors checked. All 18 champion combat definitions compare identically after excluding Minato presentation fields.
- All existing Lab WASM files remain byte-identical to main. This is source/data equivalence plus existing regression suites, not an observed native-host replay comparison.
- All atlases fit 2048x2048. Decoded totals: Gojo 33,069,056 B; Frieren party 6,791,168 B; DIO/The World 6,475,776 B; David 2,617,344 B; V1 1,150,976 B; Minato 7,958,528 B. Each is below 32 MiB.
- Frozen-source integration reproduced identical hashes for **836 runtime files**. Approval hashes and editor starter equivalence checked.
- Headless Edge: **740 decoded images, 80 full-animation GIFs, 320 mirrored previews**, zero broken images or browser errors. Review panels accommodate large transparent frames without clipping. JavaScript syntax checks passed.
- Unchanged Magnus 24 development/12 holdout scenarios and 32/64/128-piece stress rerun. 128-piece p95 **796 microseconds**; maximum 5,681 microseconds. Core-only measurement; no native host rendering claim.
- Two independent GNU release builds match the packaged native 0.10.77 DLL: `de53bf0593148edd801adf14137cfc99666b6c1c7b2950b3338521f0e79067c7`.
- Isolated Mod Manager: **836 copied files hash-compared**, **723 champion files compared**, zero differences/missing, native DLL identical. Its “game” result refers to a temporary fixture, not the installed game.

Machine-readable reports: `verification.json`, `native-tests.json`, `browser.json`, `packing-reproducibility.json`, `package.json`, `mod-manager.json`, `magnus-stress.json` and `magnus-scenarios.json`.

## Reproduce

```powershell
rtk proxy python tools/original_bodies.py complete gojo frieren fern stark dio david v1 minato minato_kcm
rtk proxy python tools/integrate_original_bodies.py
rtk proxy python tools/verify-original-bodies.py
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --manifest-path native/tfm2_custom_ai/Cargo.toml --lib
rtk proxy python tools/package-original-bodies.py
```

Python requires Pillow/NumPy. The browser test expects the repository server on localhost:7290. Package verification creates an isolated fixture under ignored `backups/`; it does not install or migrate.

Versions: native 0.10.77, custom 0.2.77, JJK 0.1.35, Frieren 0.1.23, JoJo 0.1.20, Cyberpunk 0.1.15, ULTRAKILL 0.1.17. Vader's pack is unchanged. Dependencies are synchronized.

## Pending live acceptance

Vader needs a successful proposal and approval before replacement. Actual-game visual checks remain pending for every new body: both facings, weapons/eyes/overlays, Stark's leap/carrier, Fern's casts, David's trails, Minato's teleport/KCM switch, Gojo's overlays, fog/invisibility, draft/HUD representations, cleanup and crowded host rendering. No live-game acceptance, installation or real-career migration is claimed.
