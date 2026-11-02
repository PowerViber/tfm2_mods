# Rally 3.3 — one contact impact, stronger charge aura and enemy-centred storm

This is the follow-up to the preserved [3.2 checkpoint](rally-3.2-changes.md). The latest user correction replaces the intervening two-second and map-wide impact proposals. Neither rejected proposal is part of this revision.

## Current presentation

- One **300×300 contact drawing**, held for **six ticks / 0.1 seconds**, centred at the airborne ball when Impossible releases. One cue per shot; no renewal restart, repeated contrast sequence, map overlay or landing impact frame.
- Charge remains **five seconds**, with four seconds of aiming and one locked second. The ball stays its normal size. Seven visual stages strengthen surrounding compression rings and increase lightning strokes from two to eight.
- A single-play Lab capture replaces the looping preview. Restart through the Lab demo controls. The ordinary colour landing effect and gameplay crater remain.
- Simple geometric fractures replace the generated-looking crater texture. Actual attack tier selects yellow/Black/White/Impossible styling, independently of the highest permanent body. Normal/yellow sample styles are available for review; existing gameplay still determines which attacks create persistent craters.
- No hit-stop, camera shake, vision, extra damage, control or gameplay pause comes from the contact frame or charge aura.

The approved 3.2 timing, range, endpoint-only damage, falloff and slow changes remain. Serve Impossible can aim globally within its current map/realm. Local Impossible requires 10 m contact and retains 400,000 reach. Their base blast radii remain 120,000 and 80,000; Overpressure adds 20%. Reach is separate from blast size.

## Enemy-centred storm

New patches mark visible, living, targetable enemy champions within legal spike reach, rather than following the cursor. Serve covers the current map/realm; local Impossible uses its 400,000 reach from the fixed release origin. Hidden positions, allies, ordinary units and other realms cannot supply new targets.

Prototype defaults: up to five new patches every 30 ticks, maximum ten live patches; 18,000 radius and deterministic ±6,000 placement jitter; 24-tick warning, 36-tick active interval and 15% slow. Warned positions stay fixed. Overlapping slow selects the strongest applicable value, not additive strengths. There is no lightning damage, knock-up or Pace generation.

The user requested visible enemies; the pending scope question has not separately confirmed whether ordinary units should attract patches. This implementation uses enemy champions. Native and Lab use the same bounded placement and target identities.

## Shared implementation and compatibility

- Canonical rules `rally-3.3`; Lab transport v2 and pinned athlete profiles v2 remain.
- Shared state exposes immutable charge-presentation stages and one contact-event clock. Rendering reads completed state without combat randomness or additional planning.
- Craters store their actual attack tier and Impossible identity for presentation.
- The Lab loads the same packed effect rectangles as the native adapter. The real editor server now explicitly serves these preview atlas files, fixing a timeout found during isolated route verification.
- Previous recordings remain inspectable; replay requires their original rules. Earlier 2.1, 2.2, 3.0, 3.1 and 3.2 evidence is retained.

## Artwork provenance and verification

The contact drawing is the frozen original reversed-contrast panel from the [3.2 source manifest](rally-results/rally-3.2/effect-source-manifest.json). Existing storm bolts are reused. Compression rings and crater geometry are editable code-native presentation. The discarded growing-ball proposal is not packed.

Source references were visually inspected, not fully transcribed: [Ian Zhang around 0:15](https://www.youtube.com/watch?v=CtO_XE2Nrho&t=15s), and [ZiShen44's visual construction/finalization guide](https://www.youtube.com/watch?v=jJMDwMRycd4&t=139s). One-frame timing is a TFM2 prototype choice. Image generation itself is not deterministic; extraction from frozen sources and atlas packing are reproducible.

Current test counts, reproducible WASM hash and five-run performance measurements are in [automated checks](rally-results/rally-3.3/automated-checks.json). See also [browser/parity](rally-results/rally-3.3/browser.json), [real editor routes](rally-results/rally-3.3/editor-route.json), [art validation](rally-results/rally-3.3/art-validation.json), [packing reproducibility](rally-results/rally-3.3/art-reproducibility.json), [scenarios](rally-results/rally-3.3/scenarios.json) and [Magnus timings](rally-results/rally-3.3/magnus-performance.json).

Repeat with `rtk proxy python tools/check-rally-33.py`. Tests cover contact expiry without landing cues; five-second charge and fixed-size ball; enemy-based storm selection, concealment and fixed warnings; ordinary units/allies/local-range exclusion; crater styles; original 3.2 gameplay; full native/WASM replay and presentation independence. Preview art stays under 32 MiB decoded storage and 2048 pixels per atlas. Frozen legacy main/White frame pixels, dimensions and timings are retained by packing checks.

The first real-editor attempt exposed missing preview-atlas routes and a timeout. Explicit static routes were added, then the isolated editor test passed. This was an editor loading issue, not a gameplay or native-runtime observation. A prior native parallel-suite failure is preserved in the 3.2 checkpoint; current full verification is serialized.

## Delivery boundaries

Draft [PR #63](https://github.com/PowerViber/tfm2_mods/pull/63) is updated only while open. The user approved the corrected effects on 2026-11-02; exact approved files are frozen in [effect approval](rally-results/rally-3.3/effect-approval.json). Body/helper approval status is unchanged. No shipped Rally registration/art, release package or DLL hash, Mod Manager release verification, installation or career migration is claimed here.

Native Receive and support-contribution receipts still fail closed until host calibration. Lab receipts are explicitly simulated. Native rendering, warning visibility, attribution/items, crowded host performance and actual-game acceptance remain pending. Core timings and browser screenshots are not host proof or matched-role balance results.
