# Vader, Aegis Zero and Scribble — appearance proposals

Requested on 2026-10-31. These are new original pixel proposals generated with the built-in imagegen workflow, then extracted, sized and foot-aligned mechanically. They are **not integrated into the game packages**. Approval of the appearance directions is pending.

## Direction

- **Darth Vader:** flared helmet and mask, readable black cape/armour, chest controls and red lightsaber. The previous game body is a stock-derived silhouette. The proposal gives Vader a recognisable original body.
- **Aegis Zero:** retain the existing Destiny Gundam basis—white/blue/red armour, V-fin, green sensors, folded red wing pack and existing weapon identity. This is not a switch to another Gundam model.
- **Scribble:** Gear 5-like cloud hair/scarf, white open jacket and shorts, purple sash, sandals, straw hat and elastic pose language. The user explicitly selected **“Appearance only; keep Scribble’s kit.”** No spell, stat, mastery or AI rework is authorized by this art pass.

The review includes idle, movement, attacks, two cast poses, ultimate and hit keys in both facings. These are appearance keys, not completed final movement/death animations. Game-size heights are 40 px for Vader, 46 px for Aegis Zero and 45 px for Scribble. The current bodies are shown beside them; source weapons are allowed transparent space instead of shrinking the whole character to fit the widest pose.

## Reviewed primary references

- [Star Wars Databank — Darth Vader](https://www.starwars.com/databank/darth-vader), and the official suit image listed in `reference-images.json`.
- [Official Destiny Gundam mechanical catalogue](https://en.gundam-official.com/seed/destiny/mecha/7/), particularly its full-body front illustration. The repository's existing Aegis generator also identifies Destiny as its design basis.
- [Bandai's official Gear 5 Luffy product reference](https://p-bandai.jp/press/2025/04/1000015825/), showing the white outfit, cloud forms, purple sash and straw hat.

The official images were inspected as identity references; they are not copied into the game or committed in this proposal. Local reference downloads are ignored. Original imagegen source outputs and prompts are frozen here. Source generation is not claimed deterministic; the extraction/review composition is repeatable.

`scribble-source.png` preserves the first output. `scribble-source-v2.png` corrects an extended hand crossing its packing cell. Only v2 is used by the final review. The correction prompt is recorded separately.

## Review

Start a static server rooted at this checkout and open `editor/character-redesign-review.html`. The current session serves it at <http://localhost:7292/editor/character-redesign-review.html>. Comparisons include actual size and 3× on light/dark backgrounds; wide sheets scroll rather than shrink.

`tools/preview-vader-aegis-scribble.py` regenerates the review from the frozen sources. `review.json` records source hashes, scale and approval state. `verification.json` confirms every packaged mod file, including the native DLL, remains byte-identical to the branch baseline. No installation or career migration was performed.

After appearance approval, complete the existing animation timelines and derived body presentations, verify hand/weapon/effect alignment, preserve skill-effect artwork and run the normal art-package verification before publication. Do not treat these proposal GIFs as completed in-game animation acceptance.
