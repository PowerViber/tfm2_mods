# Four-character ranked body redesign

The proposal stage contains twelve original appearance concepts: base, middle and
highest for Vesper, Emperor Isliid, Coder and Magnus. **Appearance approval is
pending.** No game-body replacement, completed animation set, release package,
installation or career migration is claimed.

The user requires approval before completing the new animations. After approval,
the approved source hashes become the appearance baseline; the remaining poses
and rank variations can be completed without repeating that appearance question.

## Workspace and source

- Branch: `codex/vesper-isliid-coder-magnus-redesign`.
- Current-main baseline: `1dec6d890c523af10e9cbe7879db50e8ece81e2c`.
- Existing edits and uncommitted Magnus proposals in the main checkout are
  preserved. Their filenames do not establish approval and none was imported as
  an approved source.
- Canonical proposal manifest: `rules/ranked-body-art.json`.
- Built-in imagegen source PNGs: `sources/`; exact prompts:
  `concept-prompts.json`.
- Official Emperor reference images, page URLs and hashes:
  `reference-pages.json`. The original Emperor and Beyond Fate references govern
  clothing/hair/posture; the TFM2 flying swords remain separate existing assets.
- `baseline-contract.json` captures the four real champion definitions, their
  complete frame/duration contracts, 322 declared packed assets and native DLL
  hash. It refuses accidental regeneration over a different production baseline.

The proposals preserve each character's existing visible idle height during
registration. Vesper and Magnus use additional transparent width, retaining
their existing vertical origin/foot relationship. Final animation registration
must retain the individual anchors, including wide attacks and fallen poses.
No body is sliced into independently animated anatomy.

## Review

Run from the isolated checkout:

```powershell
rtk proxy python -X utf8 tools/build-ranked-body-review.py
rtk proxy node tools/serve-ranked-body-review.cjs 7297
rtk proxy node tools/test-ranked-body-review.cjs
```

Open `http://localhost:7297/editor/ranked-body-review.html`. The page provides
1×/3× sprites, light/dark terrain, both facings, stock/current comparisons and
rank/pose/frame inspection. Concept poses are explicitly labelled as concepts;
uncompleted ranks show an unavailable message rather than a misleading reuse.
The current-package inspector loads the actual PNG/FANIM files. It does not
substitute editor-only thumbnails or pretend to reconstruct native layers.

Recorded browser checks: **48 proposal/facing/terrain combinations, 64 packaged
pose/facing cases, 426 packaged frame/facing cases (213 unique frames), zero
console errors and zero failed asset requests**.
The interactive review's cold/warm numbers concern browser review drawing only.
They are not native rendering benchmarks or actual-game acceptance.

Cropping, alpha-fringe cleanup (threshold 180), nearest-neighbour registration,
mirroring and review packing are deterministic from saved sources. Imagegen
generation itself is not deterministic. New raster anatomy is generated through
imagegen, not drawn procedurally by the review builder.

## Rank coverage

| Character | Ordinary | Shared Top 10 | Distinct podium | Total |
| --- | ---: | ---: | ---: | ---: |
| Vesper | 7 | #4–10 | #3, #2, #1 | 11 |
| Emperor Isliid | 7 | #2–10 | #1 | 9 |
| Coder | 7 | #2–10 | Zero-Day #1 | 9 |
| Magnus | 7 | #4–10 | #3, #2, #1 | 11 |

Rank selectors retain the existing index orders. In particular, Vesper's #1 is
look **8**, #3 is **9**, #2 is **10**; Magnus's #3/#2/#1 are **8/9/10**. Do not
silently unify those two historical orders. Names, thresholds and badges remain
the existing systems' responsibility.

## Presentation audit and remaining proof

The checked stable entity API supplies position, attack interval, cooldowns and
buff/control state; it does not expose the host body's current animation frame
or facing. Existing presentation uses inferred facing and owned action state:

- Vesper: `style_names`/`sync_visuals` in `vesper.rs`; current outfit/gauntlet
  poses follow the core's Shatter/swipe/punch/jab/block/guard priorities.
- Magnus: `sync` in `magnus.rs` selects attack/hit/run/idle overlays; actual PLAN
  marks are generated independently from `core.plans` and must remain so.
- Coder: `coder.rs` uses persistent `cd_fitN` and `cd_kbN`; the old clothing art
  assumes a fixed torso. A moving whole-body replacement cannot safely inherit
  that alignment assumption.
- Isliid currently changes sword/badge art rather than providing nine body
  outfits. His hovering movement must remain a glide.

Before full rank animation production, prove the shared pose references and
native selection on idle, movement and attacks. Keep the real base sprite
visible, add no hiding or Animation CC, and retain the coherent base fallback
where alignment cannot be established. No exact host-frame synchronisation or
in-game layer alignment is claimed by this prototype review.

After approval, integration must add `rank_art_v2: 1` without losing existing
parameters; generate native/Lab mappings from the art manifest; preserve badges,
gear and independent effects; remove only incompatible owned legacy clothing
layers for old embedded definitions. Migration stays inspect-first and separate.

## Memory audit

| Character | Existing declared unique sheets | Existing decoded total |
| --- | ---: | ---: |
| Vesper | 111 | 165.16 MiB |
| Emperor Isliid | 119 | 292.23 MiB |
| Coder | 18 | 62.52 MiB |
| Magnus | 74 | 30.34 MiB |

These inherited totals include skill FX; they are not new-body allocations.
The new body/outfit allocation stays below 32 MiB per character, atlas edges at
most 2048. **Magnus's complete presentation remains limited to 32 MiB**, so his
remaining headroom must be checked before final packing. The concept PNGs are
review assets, not packaged atlases.

## Delivery gates

Pending: user appearance approval; complete authored animation/rank coverage;
idle/run/attack alignment proof; generated native/Lab mappings and fallback;
native/parity/regression suites; five-run presentation/performance measurements;
approved-asset Mod Manager load verification; new PR separate from Grimhald #67.

Game code, champion definitions, packed textures/FANIM and native DLL remain
unchanged at this stage. Therefore no combat-change or release claim is made.
Installation, actual-career migration and live-game acceptance stay separate.
