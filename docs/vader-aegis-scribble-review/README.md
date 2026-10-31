# Vader, Aegis Zero and floating Scribble — original bodies

Art-only integration for [PR #57](https://github.com/PowerViber/tfm2_mods/pull/57), requested on 2026-10-31. The candidate includes every existing body animation and refreshes the body-derived Gundam flight composites. Installation and real-career migration remain separate. Actual-game visual acceptance is pending.

## Approved direction

- **Darth Vader:** flared helmet and mask, black cape/armour, chest controls and red lightsaber. The user accepted the proposal with “i love vader.”
- **Aegis Zero:** preserve the repository's Destiny Gundam basis: white/blue/red armour, V-fin, green sensors, folded red wings and existing weapons. Enlarge the existing Wings of Light by 30% in each dimension, with fixed shoulder roots. Deployment, ascent, flight, dive and afterimages use the new body.
- **Scribble:** floating/reclining Gear 5-style body, cloud hair/scarf, white clothing, purple sash, sandals and straw hat. Gathering, weaving and invoking poses fit his actual five-element, six-dot, 35-spell system. The six-pixel hover is cosmetic. The user explicitly selected **“Appearance only; keep Scribble’s kit.”**

The user requested the floating pose and larger wings after the first previews, then said **“continue”** after revised comparisons. `decisions.json` preserves that approval basis; it does not invent an explicit “approve all” quote.

## References and frozen sources

- [Star Wars Databank — Darth Vader](https://www.starwars.com/databank/darth-vader), with its official suit image identified in `reference-images.json`.
- [Official Destiny Gundam mechanical catalogue](https://en.gundam-official.com/seed/destiny/mecha/7/), particularly the front illustration. The existing Aegis generator confirms this model identity.
- [Bandai's official Gear 5 Luffy product reference](https://p-bandai.jp/press/2025/04/1000015825/), showing clothing, cloud forms, purple sash and straw hat.
- [User-supplied floating pose reference](https://i.pinimg.com/736x/17/9b/95/179b95ac5e45304e16c5935612d09dbf.jpg). Authorship was not independently verified.

Reference downloads are ignored and are not game assets. Original sprites were generated through the built-in imagegen workflow; the sources and prompts are frozen here. Image generation itself is not deterministic. Extraction, sizing, frame selection and packing from those sources reproduce byte-for-byte.

The standing `scribble-source.png` and `scribble-source-v2.png` are historical proposals. **`scribble-source-floating.png` and `scribble-motion-source.png` are the final sources.** Each character's motion source supplies additional action/hit/death keys. `final-verification.json` records source hashes.

## Compatibility and scope

The implementation baseline is `00fe08dbbf2f09c560121e11a1c972dff8d92ad3`, including the newer merged Contender/Minato fixes. Native remains **0.10.81**, byte-identical to that baseline; no DLL build was needed. Only affected art packs advance: Custom **0.2.82**, Star Wars **0.1.14**, Toon **0.1.7**.

Champion definitions, skills, AI, statistics, hitboxes, animation names, frame counts and durations remain unchanged. Transparent frame padding protects wider poses while preserving centre/foot registration; Scribble's deliberate visual hover is the exception. Vader's pure `skill1_effect` and `ult_area` frames remain exact, and his existing saber arc is separated from the retired body before compositing. Gundam effect aliases remain stable. Unchanged effect frames retain identical decoded pixels after restoring symmetric transparent padding; warnings and combat circles are unchanged.

Body animation coverage: Vader **16 animations / 82 frames**, Aegis **8 / 33**, Scribble **8 / 52**. Maximum atlas side is 2048 pixels. Decoded body plus owned presentation totals are below 32 MiB: Vader about **5.92 MiB**, Aegis **10.43 MiB**, Scribble **10.90 MiB**. The enlarged Gundam sheet is repacked to **1304×2048**, replacing the previous oversized sheet without removing effects.

Editor body starters and the Gundam VFX library point to the packaged artwork. Existing other-character entries are preserved. No replay or save format changes are required.

## Review and reproduction

The current static review is <http://localhost:7292/editor/character-redesign-review.html>. Serve this checkout at port 7292 to reopen it later. The page includes actual-size and enlarged comparisons, both facings, light/dark backgrounds, packaged animation frames, the orb presentation and packaged Aegis flight phases. Wide sheets scroll without shrinking.

Run from the repository root:

```powershell
rtk proxy python tools/preview-vader-aegis-scribble.py
rtk proxy python tools/integrate-character-redesign.py
rtk proxy python tools/reproduce-character-art.py
rtk proxy python tools/verify-character-redesign.py
rtk proxy python tools/preview-aegis-wings.py
rtk proxy python tools/preview-vader-aegis-scribble.py
rtk proxy node tools/test-character-redesign.cjs
rtk proxy python tools/check-character-art-package-manager.py
```

The first preview command extracts canonical pose keys required by the integrator. Source extraction and packing need Pillow. The browser check needs Playwright/Edge. The Mod Manager check uses a disposable game directory and isolated AppData, restores `manager.cfg`, and never examines real-game logs as evidence of this candidate.

## Evidence and limits

- `integration-baseline.json`: hashes from canonical Git bytes for all 841 baseline mod files. Windows CRLF is accepted only when normalizing it exactly reproduces the canonical bytes. The older `baseline.json` / `verification.json` retain the initial preview-only audit as history.
- `final-verification.json`: DLL identity, unchanged champion definitions and timing, all atlas rectangles in bounds, visible actual idle frames, 224 frozen Gundam effect frames, aliases, editor bindings and art budgets.
- `reproducibility.json`: second-generation runtime asset/binding hashes.
- `browser.json`: all 20 comparison/contact/animation images decoded without page errors.
- `mod-manager.json` and `mod-manager.txt`: isolated package consistency, separate from actual gameplay.
- `package-hashes.json`: affected-pack file manifests and aggregate hashes, plus the unchanged native DLL hash.

Automated raster/metadata/browser/package checks do not certify host rendering, crowded-fight readability or actual-game cleanup. Those observations remain pending. No new combat suite is claimed: combat inputs and the native DLL were not changed.
