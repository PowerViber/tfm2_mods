# Emperor Isliid: first sprite pass and engraving lab

Open the editor's **Skill Test → Engraving lab (Isliid)**. The lab is an isolated design prototype for comparing how mastery changes sword placement time and precision. It does not register Isliid as a playable match champion yet.

## Controls

- Choose any of the seven swords in the palette, then click the canvas to launch and plant it. Its travel time follows the selected mastery rank. There is no rotation.
- Drag a planted sword to redraw that anchor. Right-click it to recall it.
- Press **R** (with the canvas focused), or click **Manifest**, to grade the drawing. Press **Esc** or click **Reset** to clear it.
- Choose a formation, mastery rank, size, and playback speed. **Auto draw this rank** animates one route. **Compare all ranks** evaluates 50 identical seeded routes per rank.
- **Auto includes Emperor sword** substitutes that sword into a formation so its 12% effect amplification can be compared.
- **Preview floating glide** plays the movement row in the editor. The body rises and falls while his straight legs remain above the ground; the cape and oversized blades trail with the motion.
- The seven orbiting swords are independent sprites. Selecting a sword and placing it removes that exact sword from the orbit and plants the same design at the chosen point. Redrawing and recall preserve its identity.
- Changing mastery switches sword art: Base through Engraver, Engraved at Tactician and Swordmaster, Sovereign at Regent and Sovereign, Imperial at Imperial. The top two tiers add large energy crowns, lightning fins, orbiting fragments and dimensional rings. These are visual tiers for review; the sword's effect and slot stay the same.

The rank decision delays, travel speeds, aim errors, and shape effects are provisional tuning values. The displayed formation grade compares planted anchors with the cyan target. It does not yet detect arbitrary overlapping drawings, resolve shared edges, simulate BA throws or recalls, or apply game damage and buffs.

## Sprite files

- `isliid#sheet.png`: transparent 64×72 pixel frames on a 384×576 sheet. The `run` row is a floating glide, retained under that name for game animation compatibility.
- `isliid#anim.fanim`: animation metadata for idle, run, attack, skill1, skill2, ult, hit, and dead.
- `isliid_sprite_review.png`, `isliid_float.gif`, and `isliid_idle.gif`: enlarged review previews.
- `isliid_sprite.py`: reproducible Pillow generator for this reworked pixel-art pass.
- `isliid_swords#sheet.png` and `isliid_swords#anim.fanim`: separate seven-sword sprite layer with two shimmer frames per sword and four mastery art tiers, following Levi's separate cape-sheet pattern.
- `isliid_weapons.py` and `isliid_swords_review.png`: reproducible weapon art and review board.
- `isliid_ranks#sheet.png` and `isliid_ranks#anim.fanim`: seven original, numberless badge families for Bearer through Sovereign, plus a separate Imperial family with Imperial 1–10 subdivisions. They are placed upper-right of Isliid in the editor like Levi and Scribble.
- `isliid_ranks.py` and `isliid_ranks_review.png`: reproducible badge layer and review board.

The editor layers `editor/isliid-weapons.png` and `editor/isliid-ranks.png` around the sword-free character sheet in `editor/isliid-sprite-sheet.png`. Idle and movement hands hang at his sides; attack frames swing the right hand through a four-frame slash. Throws rotate the exact selected sword toward the target and add a color-matched trail and particles before it becomes the planted anchor. The character follows the slim white/navy/gold uniform, split cape, cyan gems and eyes. Native gameplay, effects, and final balance still need implementation.
