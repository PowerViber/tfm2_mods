# Original character bodies

Based on merged main `02dba54` (PR #50). Work is isolated on `codex/original-character-bodies`.

Approved original bodies replace Gojo, Frieren, Fern, Stark, DIO, David, V1 and Minato. The World and golden KCM Minato were added and approved. Vader remains unchanged after two imagegen failures; the original nine-character scope is partially complete. Gameplay and skill-effect designs are preserved. Original approved Omen/Steve/Raze bodies provide the simple blocky style reference.

## Appearance and approval

Gojo uses his black blindfold and high-collar uniform. David has a lean build and yellow jacket. Minato uses his canonical Hokage cloak instead of the old Storm Runner concept. The other appearances follow the source-character references recorded in `original-body-review/references.json`.

All integrated appearances are approved. The user corrected undersized initial fitting and explicitly allowed transparent padding while retaining original game-size idle height, exact world foot anchors and timing. Source generation uses imagegen; approved sources are frozen and mechanical extraction/packing is reproducible. AI generation itself is not deterministic.

## Boundaries

- Keep every existing duration, tag, frame count and foot baseline; approved symmetric transparent padding replaces the exact-dimension constraint.
- Held weapons in body frames are included. Separate skill effects, resources and attack geometry retain their designs. The World is an explicitly approved additional body replacement.
- Update body-derived afterimages, companion bodies and casting composites so old recolours cannot leak through abilities.
- Do not modify Rust gameplay, stats, AI or save/replay formats. The user approved a visual-only native Minato KCM presenter, relaxing the byte-identical DLL constraint for that hook.
- No installation or career migration in this work.

## Verification

See [the release evidence](original-body-review/README.md) and machine-readable reports for automated results, package hashes, reproduction and pending live acceptance. Existing native gameplay source, all Lab WASM binaries and non-body effect pixels are unchanged. The new native presenter only selects artwork from observed existing state. Installation and real-career migration were not performed.
