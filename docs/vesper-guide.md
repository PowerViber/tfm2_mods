# Vesper, the Rift Marksman: design and player guide

Vesper is a ranged carry who tears space anywhere on the map. She has **no active skills and no ultimate**. Instead, a native passive (`tfm2_custom_ai:vesper`) gives her rift charges, a 100-dimension map atlas, 10 rift types and 28 techniques.

Everything is available at mastery rank 0. Rank only changes how well the athlete plans with it, plus how she and her tears look.

- [Every dimension, rift type and technique with its numbers](vesper-catalogue.md)
- [Verification](vesper-verification.md)
- Rift Lab: open the Database Editor and press **Rift Lab** to practise everything in this guide.

![Mastery ladder](vesper-mastery.png)

## Tears, charges and distance

| Rule | Value |
|---|---|
| Rift charges | 5. Each recovers on its own in 5 s (300 ticks) |
| Tear life | About 5 s (it depends on the rift type), then it closes. At most 5 rifts are open at once |
| Where | Anywhere on the map |
| Opening delay | The rip travels from her to the spot in `6 + distance / 12000` ticks, about 1.4 s across the whole map. Sky Rifts take 1.5× as long. A crack line telegraphs it, so far tears must be **predicted** |

## The chain

- Open rifts link in the order she opened them: Vesper → T1 → … → Tn.
- When the last rift (the exit) is within her attack range (66000) of an enemy, she plants her feet and aims.
- **Every basic attack** then leaves as a packet that hops from rift to rift and scales with her AD. The aiming pose shows when this is happening, and her body is the weak point while she aims.
- Without a chain she attacks normally.

**Damage per link.** A link of length `L` keeps

    keep(L) = clamp(90% − 30% · (L / 120000)², 10%, 90%)

The final hop from the exit to the target is free, and the efficiencies multiply. For 180000 units:

| Links | Kept |
|---|---|
| 1 | 22.5% |
| 2 | 53.5% |
| 3 | 56.1% |
| 4 | 54.2% |

So the best number of links is a real decision, paid for in charges, strain and opening delay.

Every hit goes through the shared damage window: at most 35% of the target's max HP per 2 s.

## The atlas: 100 dimensions

- At match start, a seeded shuffle places the 100 dimensions on a 10 × 10 grid of 96000-unit cells. Each dimension appears exactly once.
- Every 30 s the whole atlas **drifts** one cell north, east, south or west, wrapping around the edges. The cells shimmer 3 s before the drift.
- A rift takes the dimension of the cell it opens in. That dimension:
  - affects fighters in the rift's zone: enemy-side dimensions hit enemies, support dimensions help allies;
  - **imbues every packet** passing through. On hit, the packet applies each enemy-side effect it collected, scaled by its remaining efficiency.
- **Exit-pair bonus:** the families of the last two rifts a packet passed through trigger one of 55 family-pair twists. 100 × 99 ordered pairs gives 9,900 combinations; 4,950 if order is ignored.

The 10 families, 10 dimensions each:

| Family | Examples |
|---|---|
| Control | Void (banish), Silent Realm, Frozen Moment (stun), Anchor (root), Heavy World |
| Ruin | Static Realm (+damage taken), Ember (burn), True Rift, Shatter (strip shields) |
| Motion | Mirror Realm, Drift (pull), Gale (knockback), Swap Space |
| Haven | Shelter, Bloom (heal), Bulwark (shield), Last Breath |
| Passage | Folded Map (ally teleport), Ghost Plane, Far Shore (+range), Swift Plane |
| Time | Rewind, Echo, Haste Clock / Slow Clock, Afterimage |
| Veil | Hidden Side, Blind Fog, Clear Sky (cleanse) |
| Matter | Glass Wall, Prism (packet split), Quartz Lens, Bedrock |
| Hunger | Hungry (heal cut), Siphon, Tithe |
| Chaos | Rift Storm, Dice, Unstable |

## Rift types

Ten shapes, all available at rank 0:

| Rift | Charges | What the shape does |
|---|---|---|
| Tear | 1 | Basic chain node and portal |
| Slash | 1 | A line that throws back enemies crossing it, and fires fan volleys |
| Sinkhole | 1 | A ground ellipse that slows, and swallows anyone who stays 1 s |
| Sky Rift | 2 | Opens overhead: rains volleys and drops allies from above |
| Mirror Pane | 1 | Glass that bounces shots and divers |
| Twin Rift | 2 | Two rifts born linked: an instant portal pair, and swaps |
| Wandering Rift | 1 | Moves along the path she draws, carrying what it catches |
| Anchor Rift | 1 | Stuck to a unit: escorts, hooks, lifelines |
| Seed Rift | 1 | Hidden for 2 s, then a trap |
| Maw | 3 | Huge and brief: swallows everyone inside, then caves in |

## 28 techniques

| Group | Techniques |
|---|---|
| Offense | Chain Fire, Crossfire, Bank Shot, Rift Rain, Slash Volley, Implode, Domino, Overload Shot |
| Engage | Rift Step, Ally Gate, Sky Drop, Kidnap, Sinkhole Trap, Swap, Fling, Hook |
| Defense | Phase Out, Rift Curtain, Reflect Wall, Rescue, Pocket, Lifeline |
| Team and atlas | Escort, Rotation Gate, Seed Trap, Sweep, Borrow, Stitch |

Every technique:
- spends charges and adds strain;
- applies the dimensions of every rift it uses. A Kidnap that exits in a Static Realm cell delivers an amplified enemy.

Displacements (Kidnap, Swap, Fling, Hook and the rest) respect `cc_immune`, per-target cooldowns, the 2-per-2-s displacement window and walls. Stuns respect the 60-ticks-per-3-s control window.

A Worldbreaker sequence, for example: **Seed Trap** on the healer's path → **Sky Drop** the tank onto it → **Maw** → **Domino** of implosions through Static and Ember cells → **Rift Step** out, with a **Lifeline** set beforehand.

## Counterplay and risk

- **Breaking.** An enemy standing on a rift's core for 1 s breaks it. This:
  - cuts the chain;
  - fizzles packets past that rift;
  - costs Vesper +10 strain.

  The game's AI doesn't hunt rifts, so breaks come from rifts left in fight paths. Avoiding that is part of the skill.
- **Rift strain** runs from 0 to 100:
  - +12 per rift, plus each technique's strain;
  - +8 per long link;
  - +10 per broken rift;
  - it decays while at most one rift is open.
- **Breach** happens at 100 strain:
  - the dimension under her erupts on **everyone** nearby, her allies and herself included;
  - she loses 6% max HP (never lethal) and is staggered for 0.5 s;
  - every rift collapses;
  - strain resets to 40.

The HUD above her head shows the 5 charges and a strain bar that cracks from 70 and flashes from 90.

## Mastery: same toolbox, a far better brain

Each rank adds one habit. The rank tables live in `vesper_data.rs`; they control look-ahead, how many spots and techniques she compares, noise, combo depth, link count, atlas routing, strain cap, a spare charge and how long she waits for allies.

| Rank | Name | New habit |
|---|---|---|
| 0 | Spark | Reflex chaos: random rift types and techniques from what's legal; ignores strain and enemy paths |
| 1 | Tear | Leads targets, including opening delays |
| 2 | Rift | Safety: watches strain, keeps rifts off enemy paths, checks exits, faces mirrors correctly |
| 3 | Gate | Geometry: chain solver, bank and slash angles, Crossfire splits, Implode and Domino timing |
| 4 | Wanderer | Survival: a spare charge, Lifeline first, escapes when dived, Pocket and Rescue |
| 5 | Cartographer | Team engage: Gates and Sky Drops synced with the frontliner, Escort, Kidnaps into her team |
| 6 | Riftwalker | Priority and atlas: healer or carry over tanks, chokepoints, Borrow and Stitch |
| 7 | Worldbreaker | Five-step sequences, a standing chain, drift foresight, strain riding into Overload |
| #1 | Between Worlds | Same brain as rank 7. Visual podium only |

## Rank looks

- **Outfit.** Every rank adds one readable layer: a patched coat and taped crystal at Spark, then:
  - a glowing seam;
  - dimension pockets and a strain gauge;
  - gate rings and orbiting shards;
  - a starfield lining;
  - a glowing atlas sash;
  - a hem dissolving into rift fragments;
  - rift-glass armour and a rift lance.
- **Between Worlds (#1).** A cracked sky with the atlas visible through it, ten family shards, and gold-white threads. #2 and #3 get silver and bronze versions.
- **Tears.** Each rank has its own tear look: a sputtering hairline at Spark, a rune ring at Gate, atlas stitching at Cartographer, a cathedral rip with a glass rim at Worldbreaker, and a gold fracture at #1. Link beams, packets and every technique's animation change with it.

## Logs

- `mods/tfm2_custom_ai/vesper_log.txt`: a summary every game minute, with rank, strain, Breaches, breaks, Kidnaps, gates, traps and technique use.
- `vesper_damage_log.txt`: damage traces.

Both appear under **Show logs** in the manager.
