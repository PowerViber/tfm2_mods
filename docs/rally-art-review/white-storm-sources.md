# White Storm proposal sources — 2026-11-01

These are **unapproved** original Rally assets created with the built-in imagegen tool. They are used only in the review and Lab. Source/derived hashes are recorded by `source-manifest.json`, `effects-manifest.json` and `rally-results/art-validation.json`. AI generation is not deterministic; extraction and packing are reproducible from these frozen sources.

## Body

Frozen file: `body-white-source.png`. Generated image: `exec-7be8453f-bec9-4026-8010-aaceb38727b0.png`. Edit target: `body-yellow-source.png`.

Prompt: preserve the same ten poses in two rows of five, same athlete/anatomy/hair/clothing geometry and full-face mask top row/unmasked bottom row. Change yellow uniform/footwear to white/ivory and navy trim to cool grey; preserve skin, dark hair, dark visor and crisp outlines. Transparent background, separated complete bodies, restrained blocky pixels, no lightning baked into the body, no accessories or text. Idle/run/spike/receive/set in each row.

## Effects

Frozen file: `effects-white-source.png`. Generated image: `exec-8017f8dd-bb12-4e0b-baaa-5d5dc844e875.png`. Style reference: `effects-thunder-source.png`.

Prompt: exactly six isolated transparent pixel-art effects in a three-column/two-row grid: tall hollow white branching lightning aura; white-lightning volleyball; long horizontal white corkscrew trail; vertical white bolt into an elliptical shock ring; compressed white teleport/afterimage burst; shallow charcoal crater with black/crimson cracks and a readable elliptical rim. Coarse angular strokes, silver/pale icy-blue edging, restrained palette. Generous margins, no character, text, opaque backdrop, full-screen flash or scene.

## Mechanical processing and storage

- Extract cells/components, threshold alpha, nearest-neighbor resize, quantize body palette, align the foot at 52 on a 64x64 canvas, mirror facings and pack matching frames.
- Impact frames remove only verified empty lower padding; a canonical −106-pixel effect offset preserves world alignment. The generator asserts removed phase areas contain no visible pixels.
- Identical stroke frames share storage. Fully transparent stroke frames are canonical blanks; visible pixels and phase durations are preserved.
- Current four-sheet proposal totals about 30.8 MiB decoded; every atlas side is at most 2048 pixels.
- Serve's sector, resource bars and danger boundaries are code-defined geometry, separate from creative body/effect artwork.

White Storm's blink, ten-metre trigger and permanent role tier are original TFM2 adaptations requested here, not verified claims about a SUNCYAN character's exact mechanics.
