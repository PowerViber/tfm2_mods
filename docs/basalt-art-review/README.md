# Basalt artwork review

Review-only work on `codex/basalt-art`, based on PR #88 head
`3790d7ddb9578f3fc051e67bfa5f8f0bbb1f200a` (`basalt-1.1`).

## Direction and approval state

- Reihan requested original Basalt artwork on 16 November 2026.
- The first grey/orange body proposal was rejected as too grey.
- Reihan requested a more recognisable **red/yellow body**. The new source is
  `body-source-v2.png`; this generated appearance still awaits approval.
- Reihan requested a visible V-clap cone originating at Basalt, like Vader's
  choke. The revised preview shades the existing core hit region and draws
  the V outline from the champion. The earlier response was a correction
  request, not approval of the effect pack.
- Body appearance and skill effects remain **unapproved**. Full animation
  completion and game-pack integration follow those approvals.

## Sources and reproduction

The built-in `image_gen` tool supplied original transparent source sheets.
Prompts are in `prompts.json` and `prompts-v2.json`; frozen source/output hashes
and frame rectangles are in `manifest.json`. Generation is not deterministic.
Extraction, scaling, palette quantization, mirroring and packing reproduce:

```text
rtk proxy python tools/prepare-basalt-art.py
rtk proxy node editor/extract-game-world.js "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2"
rtk proxy node tools/serve-basalt-review.cjs 7308
```

Open `http://localhost:7308/editor/basalt-review.html`. The server also accepts
the root-level editor routes. It has no installation or career endpoints.
Extracted game art stays in ignored `editor/game-reference/`, outside the
distributable mod. It requires the existing game-exported map dump and bundle;
the extractor refuses an estimated map.

The comparison sheets show native-size and enlarged pixels without CSS
shrinking. Both facings and light/dark backgrounds are available. The body
canvas is 128×80 with foot anchor (64,60), including transparent room for the
long whip. Idle body height is approximately 44 pixels, excluding the coiled
whip. Complete animation cycles have not yet been authored.

## Review Lab

`editor/basalt.html?art=proposal` uses the unchanged shared WASM skill core,
the extracted 5v5 map and stock champions/minions/camps/structures. The host's
movement, opponent attacks, physical mitigation and visibility are explicitly
simulated, not native acceptance evidence. It exposes all thirteen profile
values and the existing ground/air commands. The default setup is a mid-game
gank, with the shared objective/laning/tower scenarios available.

The cone boundary is derived from the canonical rules and unchanged Rust
Clap hit check. Its outline begins at Basalt; only the existing hit region is
filled. The small point-blank gap and edge tolerance are preserved. No cast
delay, damage, stun, range or targeting change was made.

```text
rtk proxy node tools/test-basalt-art-review.cjs
rtk proxy node tools/test-basalt-cone.cjs
rtk proxy python tools/generate-basalt.py --check
```

Evidence: `browser-review.json`, `cone-check.json`, `validation.json` and the
screenshots in this folder. The corrected proposal has ten browser checks,
nine cone/core boundary comparisons, and 30/60/120/144/240-Hz clock checks.
Manual profiles and presentation-enabled/disabled traces agree in the covered
fixtures. Review atlases decode to 1.5 MiB; each is below 2048×2048.

`native/`, `rules/`, `mods/`, the DLL and the WASM are unchanged by this review.
No installation, career migration or real-game visual acceptance occurred.
