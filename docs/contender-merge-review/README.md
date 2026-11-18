# Shadow Merge: fused-body appearance proposal

Reihan requested that Contender actually look merged with his shadow. The current native presentation renews an idle shadow over the normal body. This proposal instead uses one connected whole body: charcoal shadow arms, violet seams through the wraps, bright fists and the same recognisable hooded face, scale and anchor.

**Approval is pending.** These are four appearance poses (idle, movement, straight punch, uppercut), in both facings. They are not a completed animation set. No shipped body/effect assets, DLL, WASM, rules or release versions are changed.

- Comparison/inspector: `http://localhost:7309/editor/contender-merge-review.html`
- Actual-map candidate: `http://localhost:7309/editor/contender.html?mergePreview=1&build=contender-merge-1`. Press Q. Only the opt-in candidate replaces the old offset shadow/body pair; the ordinary Lab retains the packaged presentation until approval.
- Candidate poses follow movement, attacks and facing. On expiry the approved normal body returns. No gameplay locks, stat changes, new effects or extra control are introduced.

## Source and reproduction

Generated with the **built-in imagegen tool**, editing the approved `docs/contender-shadow-review/body-source-v1.png`. The frozen proposal is `merge-source-v1.png`; its hash and the extracted sheet hash are in `review.json`. AI generation itself is not deterministic.

Run `rtk proxy python tools/prepare-contender-merge-review.py` to reproduce extraction, nearest-neighbour scale, facings, packing, data and comparisons. The 640×64 proposal sheet occupies 163,840 decoded bytes. It uses the existing 80×64 canvas, [40,52] foot anchor and approximately 42-pixel visible height. Extraction was repeated with identical source/sheet hashes. The normal body pixels remain unchanged.

Exact imagegen prompt:

> Use case: identity-preserve. Asset type: transparent pixel-art game sprite proposal for this EXACT approved hooded boxer, transformed by his Shadow Merge skill. Edit the supplied sprite sheet; keep the same four columns and poses (idle guard, run, straight punch, uppercut), the same proportions, connected anatomy, foot baseline, simple blocky pixel-art clusters and transparent background. Change ONLY the temporary merged form. He must visibly become one fused flesh-and-shadow boxer, not a normal boxer with a second idle ghost behind him. His exposed arms and most of his body turn charcoal-black shadow, with clear violet seams flowing through his torso and into the wrapped fists. Keep the same recognisable hood and human face under it; bright violet eyes, compact white-violet fists and ivory wrap highlights preserve readability at 42 visible game pixels. The shadow is part of his connected body. Keep all feet, hands and clothing attached and maintain consistent lighting/scale in every pose. Small shadow ribbons can trail from his hood and waist, but no separate clone, giant smoke cloud, circle/aura, weapons, new armour, extra limbs, text, labels or scenery. Preserve pose landmarks, existing canvas layout and large transparent gaps for reproducible extraction. Provide genuine alpha transparency.

## Observations

`browser-observations.json` records the current `contender-2.3` Lab check: complete 420-tick traces agree with the candidate enabled/disabled; only one body is drawn during the opt-in Merge; expiry returns to the normal presentation; four poses and both facings load without browser errors. `merge-lab.png` shows the candidate on the actual extracted map. This does not establish native rendering acceptance or full 2.3 gameplay/parity validation.

After appearance approval: complete the pose animations, add native pose/facing-aware presentation with safe effect lifetimes, verify packaged assets and compare actual-game alignment separately. Installation and career migration remain separate.
