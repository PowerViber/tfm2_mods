# Approved original base bodies and ranked-art gate

This draft integrates approved original base bodies for Vesper, Emperor Isliid,
Coder and Magnus. **The full ranked-outfit redesign is not complete.** Remaining
ranked animations are gated by safe native layer alignment, as required by the
user's plan. Native presentation uses a coherent, visible base fallback.

## Approval and baseline

On November 8, 2026 the user replied **“Approve all four appearance directions”**.
Complete poses derive from those approved sources; no repeated appearance
question is required.

- Vesper: brown wolf cut, real glasses, gunmetal street-fighter clothes and chrome gauntlets.
- Isliid: official original Grand Summoners Emperor toward Beyond Fate, preserving floating movement.
- Coder: dark-haired human programmer, graphite clothing and typing gestures.
- Magnus: Carlsen-like face, swept brown hair, stubble, navy/ivory tailoring and handheld board.

Branch: `codex/vesper-isliid-coder-magnus-redesign`.
Current-main baseline: `de10d3f7de9a8d471c4bab407f6f61ddb47907f9` (native **0.10.91**).
The branch includes the merged movement fixes. Champion pack **0.2.94** follows
main's 0.2.93 and depends on native >=0.10.91. The native DLL is inherited
**byte-identically from main**, not rebuilt/replaced by this art pass.
Existing main-folder edits and uncommitted Magnus proposals were untouched;
filenames containing 'approval' do not establish approval.

## Integrated body artwork

| Body | Frames | Poses | New body atlas |
| --- | ---: | ---: | ---: |
| Vesper | 64 | 8 | 1.014 MiB |
| Emperor Isliid | 33 | 8 | 0.443 MiB |
| Coder | 52 | 8 | 0.619 MiB |
| Magnus | 64 | 8 | 0.643 MiB |

All **213 frames** retain existing names, counts and individual durations.
Whole-body drawings use fixed world/foot offsets, transparent padding and
distinct movement frames. Isliid glides without walking leg contacts.
The actual default roster idle is visible. There is no body hiding, animation
CC or movement restriction.

Old clothing aliases and Magnus's body-only cast aliases retain their dimensions,
frame counts, durations and names but contain transparent presentation frames.
They no longer paint an incompatible old body on top. Existing runtime buff
and effect command identities stay unchanged, including old embedded definitions.

Coder's keyboard drawings move four pixels upward to meet the typing hands;
their decoded design pixels remain identical. Other rig/effect frames in that
mixed atlas are exact. Badges, rifts, engraving, flying swords, code windows,
chess pieces, board geometry and actual PLAN calculations are preserved.
Vesper's header and legacy preview cells use actual candidate body frames.

New definitions add only `rank_art_v2: 1`, preserving existing parameters.
It marks this presentation generation; it does not enable an unproven renderer.

## Ranked animation capability gate

The stable entity API has no current body animation-frame or facing query.
Existing Vesper/Magnus layers infer poses; Coder's old clothes assume a fixed
torso. These assumptions cannot prove exact alignment with moving whole bodies.
A sprite draw API does not establish such synchronisation.

The approved plan's visible fallback is therefore used. **The remaining
11/9/9/11 ranked animation sets were not authored past the alignment gate.**
Twelve approved single-pose rank references remain inspectable. No completed
ranks or safe host-frame synchronisation are fabricated. Full ranked outfits
require a supported pose/sprite mechanism and idle/movement/attack proof.

## Review and reproduction

- [Combined review](../../editor/ranked-body-review.html): before/stock/candidate
  comparisons, approved rank references, 1x/3x, light/dark terrain, both facings,
  actual packed pose/frame inspection and complete base animation GIFs.
- [Vesper animation](staged/vesper-right.gif),
  [Isliid animation](staged/isliid-right.gif),
  [Coder animation](staged/coder-right.gif),
  [Magnus animation](staged/magnus-right.gif). Left-facing GIFs are adjacent.
- [Canonical art manifest](../../rules/ranked-body-art.json): frozen source
  hashes, exact animation contracts, anchors and native/Lab asset paths.
- [Concept prompts](concept-prompts.json), [animation prompts](animation-prompts.json)
  and [official references](reference-pages.json). Imagegen itself is not deterministic.

Open `http://localhost:7298/ranked-body-review.html`. All four existing Labs
read actual package frames. Standalone wrappers expose the existing Isliid/Coder
Labs without recreating their rules. The editor server uses isolated AppData.

```powershell
rtk proxy python tools/serve-ranked-editor.py 7298
rtk proxy python -X utf8 tools/verify-ranked-body-generation.py
rtk proxy node tools/test-ranked-body-review.cjs
rtk proxy node tools/test-ranked-body-labs.cjs
rtk proxy python -X utf8 tools/check-ranked-body-package-manager.py
```

Registration, mirroring and packing reproduce from frozen sources. Legacy
procedural body/outfit generators are historical; the new manifest-driven
scripts are authoritative for these candidate bodies and compatibility aliases.

## Automated evidence

- Native: **1,027 passed, 0 failed, 13 ignored**. Four existing Windows bridge
  test-helper warnings remain. The initial MSVC attempt lacked link.exe; the
  supported GNU toolchain completed successfully. See [native-tests.txt](native-tests.txt).
- Vesper: **40 math + 938 simulation vectors**, zero mismatches.
- Magnus: **9,819 complete native/editor frames**, including concurrent instances.
- Coder: **558 deterministic runs**, all 5,500 view references checked.
- Isliid: **5,307 visual references**, Lab tables and grades passed. Its verifier
  now accepts approved transparent padding while preserving timing and hashes.
- Browser: **48 concept/facing/terrain cases**, **64 packaged pose/facing cases**,
  **426 frame/facing pixel comparisons** against an independent Pillow oracle.
  All four actual Labs load without page errors.
- **284 complete independent assets** are byte-identical. **198 mixed-atlas
  frames** preserve their designs, including the authorised keyboard translation.
- Champion-load test passed. Isolated Mod Manager: **859 files**, zero
  differences/missing files. No real installation or real AppData logs were used.
- Native/WASM sources and gameplay definitions are unchanged apart from the
  inert art marker; see [art-verification.json](art-verification.json).

[Five Magnus stress runs](magnus-performance.json) retain the under-4-ms
128-piece p95 target: median **0.740 ms**. This excludes host/rendering and is
not an upper bound on every individual step.

[Browser measurements](lab-browser-results.json) separate five cold body loads
from five warm runs. Warm numbers normalise 32-draw batches with a small readback
to avoid timer-resolution zeros. They are browser timings, not game-host
performance or evidence that a freeze is fixed.

## Memory, compatibility and remaining delivery

All new body atlases are <=2048 pixels and under 32 MiB individually.
Magnus's **complete** candidate presentation is **28.85 MiB**, below 32 MiB.
Baseline inherited totals were Vesper 165.16 MiB, Isliid 292.23 MiB and Coder
62.52 MiB. Their large independent FX allocations remain memory debt; this
body-only pass does not claim to resolve it. Candidate totals are in the review.

[Generation hashes](generation-results.json) record byte-identical rebuilding
and the unchanged packaged DLL. DLL SHA-256:
`1e74f3f5c9128f8dd20cd1e6b95734246681c83590082b105107a3c15a047916`.

No install, real-career migration, released package or actual-game acceptance
is claimed. Live roster, crowding, movement/gear alignment, interruption, death
and respawn observations remain pending. The PR stays draft while the ranked
animation capability gate is unresolved.
