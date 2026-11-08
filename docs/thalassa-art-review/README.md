# Thalassa — approved pearl-and-seafoam artwork

Artwork and presentation on `codex/thalassa-art`, based on PR #70 head `68f3d8048bcbed7500921743f740e3821b580c60`.

The user approved both sources on November 9, 2026: **“Approve this appearance”** and **“Approve these effects.”** See `approval.json`. The appearance is silver-white hair, a true teal mermaid tail, navy-and-pearl clothing and a compact muted-gold harp. The approved roster-idle pixels remain exact. All 37 whole-body frames and their mirrored counterparts are complete, along with the derived translucent twin and 126 original effect/presentation bindings, plus 224 baked link variants in the follow-up.

Open `/editor/thalassa-review.html` on a server rooted at this checkout, or `/thalassa-review.html` on the editor server. The review inspects actual packaged frames, every pose/frame/facing, the water twin and the original stock comparisons. `idle.gif`, `run.gif`, `attack.gif`, the four skill/ultimate GIFs, `hit.gif` and `dead.gif` preview complete animations. The playable Lab uses these same atlases.

Local review: `http://localhost:7299/editor/thalassa-review.html`; playable Lab: `http://localhost:7299/editor/thalassa.html`.

## Sources and extraction

Both source PNGs came from the built-in imagegen tool. Their hashes, source cells, native footprints and mechanical reduction are in `body-proposal.json` and `effects-proposal.json`. Generation itself is not deterministic. `tools/prepare-thalassa-proposals.py` reproduces review outputs from those frozen sources:

```powershell
python tools/prepare-thalassa-proposals.py --body-source docs/thalassa-art-review/body-source-v1.png --effect-source docs/thalassa-art-review/effects-source-v1.png
```

The proposal script writes only this review directory. Complete-animation sources are frozen separately; `tools/pack-thalassa-art.py` reproduces game atlases, frame metadata, the native presentation-clock table, the editor manifest and the GIFs. Whole-body drawings are uniformly sampled, registered to the tail-ground anchor, mirrored and packed without separately sliced torso/leg animation. The twin is a deterministic tint of those body frames.

## Presentation and geometry

- Body atlas: **512×560**. Effects atlas: **1536×1536**; tracked-link atlas: **2048×2048**. Total decoded storage: **27,361,280 bytes (~26.1 MiB)**, below 32 MiB.
- Pool/whirlpool art uses the canonical matching 35,000 radii at 950 world units per pixel.
- Pools use stable eight-tick renewal clocks, with at most four pool renewals per update and a nine-tick cancellation tail. Twin and trident body phases last one tick. Existing aliases remain registered.
- Trident orientation follows its committed heading; Dive's exit splash is presented at actual arrival. The twin follows its real carrier, with mirrored movement/attack/cast poses.
- No new idle/basic Animation CC, movement restrictions, planning or combat randomness. The original art-only commit 3849991 kept gameplay byte-identical to 68f3d80; the authorized rules-1.1 follow-up changes pool radius, Dive range/echo and trident velocity. Lab transport/profile formats, body pixels and animation durations remain unchanged.

## Original art-only verification (3849991)

- **742** artwork/definition/reproducibility checks; all 37 frames visible, correct mirrors, tail anchors, atlas bounds and unchanged approved roster pixels.
- Full native suite **1,052 passed, 13 ignored**, with serial execution. The first parallel run failed the unchanged Levi corridor fixture and then a poisoned terrain lock; serial passes do not establish that parallel-test isolation is fixed. Four existing native-bridge test-helper warnings remain.
- Thalassa Lab **22 passed**; original browser suite **8 passed**; artwork/browser suite **34 passed**.
- **954** identical complete combat-state frames with presentation on/off, and **954** complete native/WASM state and command-array comparisons.
- Champion-load test passed. Disposable Mod Manager comparison: **864 files**, zero differences. This did not run the game or prove runtime registration.
- Five Magnus stress runs: 128-piece p95 **729 / 711 / 750 / 702 / 654 µs**, median **711 µs**, below the 4,000 µs target.
- New-browser-context loading and sustained canvas drawing are measured separately in `browser-verification.json`. They are not game-host FPS or freeze acceptance.

Reports and hashes: `verification.json`, `browser-verification.json`, `magnus-performance.json`, `mod-manager.json` and `validation-summary.json`.

## Rules 1.1 follow-up (November 9, 2026)

Reihan requested the twin to follow the real water Dive, a tracking Tide Link, pools matching the existing whirlpool, farther Dive and a slower trident. He clarified **"More range only"**: Dive increases from 50,000 to 75,000 without changing its travel formula. Owned-pool dives remain unlimited in distance.

- Pools increase to 35,000; whirlpool remains 35,000 (scale becomes 100%, avoiding accidental 49,000 whirlpools).
- Twin Dive copies the owner's committed legal destination and arrival clock. Each approach still validates terrain. Both get departure/arrival feedback.
- Tide Link reads live endpoints every tick, using at most six one-tick approved-pixel segments, 32 heading buckets and seven baked lengths. It has no six-tick frozen midpoint or lingering trail. Rotation is quantized to 11.25 degrees; final length rounds to an available small segment. No gameplay geometry depends on it.
- Trident becomes 2,000 units/tick, 2,600 from water. Grimhald's axe is a fixed 66-tick flight, so this matches its average speed at a 132,000-unit land throw, not at every possible distance. Other ultimate rules remain unchanged.
- Lab playback was incorrectly one tick per display frame. It now runs 60 ticks/sec at every tested refresh rate, with bounded catch-up, pause/step and background reset. Existing body frame durations are unchanged.

Current reports supersede earlier counts: verification.json (1,415 checks), browser-verification.json (34 checks, 1,154 complete native/WASM and visual-on/off trace frames), followup-verification.json (16 targeted browser checks including 30/60/120/144/240 Hz), and mod-manager.json (866 disposable files, zero differences). The full serial native suite has 1,056 passing tests and 13 ignored; 29 Thalassa native tests pass. Thalassa Lab has 25 passing tests; the original gameplay-browser suite has eight. Actual Windows game observations remain pending.

Reproduce the additional browser checks with tools/test-thalassa-followup.py after building native/thalassa_lab's parity binary and serving the checkout on port 7299. The updated WASM is built from that same revised core. Older recordings still require their original rules.

## Remaining delivery gates

- Release version synchronisation and reproducible packaged DLL: reserved native **0.10.92**, champion pack **0.2.95**. Existing packaged native DLL is unchanged from main and must not be treated as the Thalassa release.
- Actual-game roster visibility, pool lifetime/alignment, clone targeting/facings, Dive arrival and crowded-combat acceptance.
- Installation and real-career migration remain separate and were not performed.
