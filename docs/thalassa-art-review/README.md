# Thalassa — approved pearl-and-seafoam artwork

Artwork and presentation on `codex/thalassa-art`, based on PR #70 head `68f3d8048bcbed7500921743f740e3821b580c60`.

The user approved both sources on November 9, 2026: **“Approve this appearance”** and **“Approve these effects.”** See `approval.json`. The appearance is silver-white hair, a true teal mermaid tail, navy-and-pearl clothing and a compact muted-gold harp. The approved roster-idle pixels remain exact. All 37 whole-body frames and their mirrored counterparts are complete, along with the derived translucent twin and 126 effect/presentation bindings.

Open `/editor/thalassa-review.html` on a server rooted at this checkout, or `/thalassa-review.html` on the editor server. The review inspects actual packaged frames, every pose/frame/facing, the water twin and the original stock comparisons. `idle.gif`, `run.gif`, `attack.gif`, the four skill/ultimate GIFs, `hit.gif` and `dead.gif` preview complete animations. The playable Lab uses these same atlases.

Local review: `http://localhost:7299/editor/thalassa-review.html`; playable Lab: `http://localhost:7299/editor/thalassa.html`.

## Sources and extraction

Both source PNGs came from the built-in imagegen tool. Their hashes, source cells, native footprints and mechanical reduction are in `body-proposal.json` and `effects-proposal.json`. Generation itself is not deterministic. `tools/prepare-thalassa-proposals.py` reproduces review outputs from those frozen sources:

```powershell
python tools/prepare-thalassa-proposals.py --body-source docs/thalassa-art-review/body-source-v1.png --effect-source docs/thalassa-art-review/effects-source-v1.png
```

The proposal script writes only this review directory. Complete-animation sources are frozen separately; `tools/pack-thalassa-art.py` reproduces game atlases, frame metadata, the native presentation-clock table, the editor manifest and the GIFs. Whole-body drawings are uniformly sampled, registered to the tail-ground anchor, mirrored and packed without separately sliced torso/leg animation. The twin is a deterministic tint of those body frames.

## Presentation and geometry

- Body atlas: **512×560**. Effects atlas: **1536×1536**. Total decoded storage: **10,584,064 bytes**, below 32 MiB.
- Pool/whirlpool art uses the canonical 25,000/35,000 radii at 950 world units per pixel.
- Pools use stable eight-tick renewal clocks, with at most four pool renewals per update and a nine-tick cancellation tail. Twin and trident body phases last one tick. Existing aliases remain registered.
- Trident orientation follows its committed heading; Dive's exit splash is presented at actual arrival. The twin follows its real carrier, with mirrored movement/attack/cast poses.
- No new idle/basic Animation CC, movement restrictions, planning or combat randomness. The combat core, rules, Lab transport, WASM and champion combat fields are unchanged from the reviewed PR head.

## Verification

- **742** artwork/definition/reproducibility checks; all 37 frames visible, correct mirrors, tail anchors, atlas bounds and unchanged approved roster pixels.
- Full native suite **1,052 passed, 13 ignored**, with serial execution. The first parallel run failed the unchanged Levi corridor fixture and then a poisoned terrain lock; serial passes do not establish that parallel-test isolation is fixed. Four existing native-bridge test-helper warnings remain.
- Thalassa Lab **22 passed**; original browser suite **8 passed**; artwork/browser suite **34 passed**.
- **954** identical complete combat-state frames with presentation on/off, and **954** complete native/WASM state and command-array comparisons.
- Champion-load test passed. Disposable Mod Manager comparison: **864 files**, zero differences. This did not run the game or prove runtime registration.
- Five Magnus stress runs: 128-piece p95 **729 / 711 / 750 / 702 / 654 µs**, median **711 µs**, below the 4,000 µs target.
- New-browser-context loading and sustained canvas drawing are measured separately in `browser-verification.json`. They are not game-host FPS or freeze acceptance.

Reports and hashes: `verification.json`, `browser-verification.json`, `magnus-performance.json`, `mod-manager.json` and `validation-summary.json`.

## Remaining delivery gates

- Release version synchronisation and reproducible packaged DLL: reserved native **0.10.92**, champion pack **0.2.95**. Existing packaged native DLL is unchanged from main and must not be treated as the Thalassa release.
- Actual-game roster visibility, pool lifetime/alignment, clone targeting/facings, Dive arrival and crowded-combat acceptance.
- Installation and real-career migration remain separate and were not performed.
