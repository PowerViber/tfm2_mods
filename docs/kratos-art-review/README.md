# Kratos presentation

Original transparent pixel artwork generated with the built-in imagegen tool. Reihan approved Kratos and Fenrir separately, then approved the skill effects after the preview server was restored. Exact responses and appearance-source hashes are in `approvals.json`.

Kratos keeps his pale skin, red tattoo, full beard, leather armour and red waistcloth. Axe and Blades use rigid accessory layers over a complete, connected, visible body. The roster fallback is the real resting body with empty hands; weapons appear through the optional native presentation layer during matches. No cosmetic hiding, idle/basic Animation CC, stat change or extra movement restriction is introduced.

## Sources and references

Reference images were inspected before generation; no reference pixels were packaged:

- Official PlayStation Kratos cosplay guide: https://secure.cdn.us.playstation.com/god-of-war-ragnarok/cosplay/god-of-war-ragnarok-cosplay-guide-kratos.pdf — page 3 combat outfit and page 5 fur-cloak comparison. Download SHA-256 `e20cfcea4d09c5d03f264427a5eca4fe22c9b972fa31c0adee06b65b749bd15b`. The light combat outfit was selected for readability; the cloak was omitted.
- Source hub: https://www.playstation.com/en-us/god-of-war/community/cosplay/
- Fenrir game capture reproduced by Sohu: https://p5.itc.cn/q_70/images03/20221219/4a16658b4be54e65a98c3e66cf647209.png — grey-brown fur, silver facial markings and amber eyes. This is a third-party reproduction, not an official publication. SHA-256 `49ab6a9c9c057b7bae689a01973b9b43fb36fb7b7ce6f6946dc5606ccc3e7ea9`.

Initial prompts: `prompts-v1.json`. Complete whole-body animation edit prompts: `animation-prompts-v1.json`. Other prompts: `effects-prompt-v1.txt`, `props-prompt-v1.txt`, `fenrir-run-prompt-v1.txt`. Frozen original outputs use `*-source-v1.png`; their hashes are recorded in `art-manifest.json`. AI generation itself is not deterministic.

## Reproduce

From the repository root, run:

```powershell
rtk proxy python tools/build-kratos-art.py
rtk proxy python tools/generate-kratos.py
rtk proxy python tools/generate-kratos.py --check
```

The build extracts whole drawings, thresholds alpha, removes disconnected extraction specks, applies one scale per source sheet, transforms only rigid weapons, mirrors facings and deduplicates packed frames. Character anatomy is never sliced into separately animated parts. Group-specific approval gates prevent unapproved sources from entering the mod folders.

`rules/kratos-art.json` defines frame clocks, aliases, scale and anchors. The generated manifest supplies Lab selection; generated Rust clocks and champion bindings supply native selection. Twelve poses have eight frames each. The body canvas is 64×64 with feet 20 pixels below its centre; padding preserves the approximately 40-pixel body. The weapon canvas is 96×96 with the same world anchor. The 0.8-second movement cycle uses the same drawings and lighting as idle/casting.

The native body/weapon/FX allocation totals **16 MiB**, below 32 MiB. Atlases are at most 2048 pixels. `assembled-preview` is an offline contact-sheet aid and is neither shipped nor loaded by the Lab. The Lab draws the actual approved packaged body, weapon and FX atlases.

## Review and limitations

- `editor/kratos-review.html`: actual-size/3× comparisons, all poses/frames/facings and effect playback.
- `editor/kratos.html`: fixed-60-Hz WASM simulation, actual 5v5 map, full-map/follow views, supported manual commands, thirteen athlete values, demos and recordings.
- `tools/test-kratos-art.cjs`: browser/route, playback, recording, presentation independence, scenario and manual-profile checks.
- `tools/verify-kratos-art.py`: source hashes, atlas bounds, mirrors, anchors, unchanged combat data and complete native/WASM output comparisons.
- `tools/check-kratos-art-manager.py`: isolated file-comparison fixture only. It never installs into the real game.

The Lab models enemies, basic attacks, glides, mitigation, shields and targeting. It models Sight as avoidance; the native adapter instead refunds damage by healing, so healing reduction can affect it. Native Chain Dash glides to a wall and then hops across it. Live alignment, interruption, dodge receipts/healing reduction, cage behaviour and crowded-fight readability remain actual-game acceptance gates.

Release versioning and DLL packaging are deliberately separate. This work does not install mods or migrate a career, and the existing packaged DLL does not yet contain this presentation adapter.
