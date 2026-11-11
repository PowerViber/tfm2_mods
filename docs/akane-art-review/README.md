# Akane — approved original artwork and actual-map Lab

Appearance approval: **“Approve Akane’s appearance.”** Effects approval: **“Approve these effects.”** Both were supplied by Reihan on 2026-11-11. Exact source hashes and approvals are in `approvals.json` and `frozen-sources.json`.

## Artwork

Built-in imagegen produced the original connected whole-body sources, followed by reproducible extraction and packing. Prompts are frozen in `prompts.json`; AI generation itself is not deterministic. `concept-source-v1.png` is the approved appearance reference. Motion, techniques, finale/death and night/hit source sheets complete that appearance.

Nine poses have eight whole-body drawings each, in both facings: idle, movement, katana attack, kunai backflip, Edge stance, fifth-hit finale, Night call, hit and death. `dash` and `night_dash` are retimed compatibility aliases of the attack drawings. No torso/leg slicing, idle/basic animation locks or body hiding is introduced.

Canvas starts at 48×56, with transparent padding for wide poses. Every foot contact remains 20 pixels below the frame’s world centre. One per-source standing calibration preserves crouch/death proportions; one shared palette preserves lighting. Native idle remains a real visible sprite for roster cards.

Crimson soul/mist/slash artwork preserves the 18,000-radius burst and 20,000-radius finale. Eight directional kunai/slash variants retain original aliases. Orbs select single-frame phases from their birth tick; three-tick renewals bound cancellation tails and share the existing effect budget. Night uses a short opening and an owner-following cosmetic buff. Existing accepted pose CC names are remapped; their durations and mechanics are preserved by the art-only change.

Decoded body plus effects: **4 MiB**, within 32 MiB. Largest atlas edge: **1,024 pixels**, within 2,048.

## Reproduce and inspect

1. `rtk proxy node editor/extract-game-world.js` (reads a local installed game; extracted stock files are ignored).
2. `rtk proxy python tools/build-akane-art.py`
3. `rtk proxy python tools/generate-akane.py`
4. `rtk proxy powershell -NoProfile -File tools/start-akane-review.ps1`

Open `http://localhost:7304/akane-review.html` or the playable `http://localhost:7304/akane.html`. Root and `/editor/` routes work. The review uses the packaged atlases and includes both facings, frame/pose inspection, actual-size/3× comparisons and animated GIFs.

## Lab and limits

The installed 960,000×960,000 battlefield, original map layers, 90 blocked cells, structures, stock champions, waves and camps share 1,000 world units per art pixel. All 900 wall-grid cells match the registered source map. Eight mid-game situations cover laning, ganks, counterganks, a jungle camp, Morgard, Serpen, tower defence and diving. Stock actor replacement, native-size inspection, pan/zoom/minimap, recordings and thirteen athlete controls are available.

The Rust/WASM core owns Akane’s kit. Stock combat/navigation, physical mitigation, visibility, cooldown-readiness hints and incoming control are **modeled host behaviour**, not a stock-engine emulator. Bush stealth and native assist receipts are not proven by the Lab. The lab’s `acceptedBasic` test hook represents a host-accepted impact; ordinary F attacks preserve their release/cadence. No proximity-only assist model is presented as an authoritative receipt.

Playback uses 60 simulation ticks per second independently of 30/60/120/144/240 Hz display refresh. Pause/background resets prevent catch-up; focus loss clears held controls.

`art-only-verification.json` and `art-only-browser-results.json` preserve evidence before the later requested range/travel changes. They must not be read as proof that subsequent balance edits leave gameplay unchanged. Current results live in `verification.json`, `browser-results.json` and the range review.

The current `akane-1.1` ranges, flight rules, tests and limitations are recorded in [range-review.md](range-review.md). The backflip drawings now follow the requested 30-tick travel, and the kunai uses single-frame directional flight artwork renewed at its actual core position.

Release versions, packaged Windows DLL, installation, real-career migration and actual-game visual/combat acceptance remain separate gates. The existing packaged DLL is unchanged; Mod Manager’s disposable fixture does not prove the new adapter runs in that DLL.
