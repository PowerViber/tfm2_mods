# Contender, the Shadow Boxer (contender-4.0)

Rebuilt in round 192 from Reihan's design. All numbers live in `rules/contender.json`; the full kit and the reasons are in
`docs/main-review-round-192.md`.

| Move | Rules |
|---|---|
| Shadows (passive) | 10 shadows, one back every 3 s (faster per landed hit while low). An enemy champion hit spends one to be dodged, and the shadow counterpunches (20 + 30% Attack). |
| Shadow Rhythm (passive) | Up to 10 stacks per enemy for 4 s; +1.5% damage taken from him per stack. |
| Shadowborne (passive) | Every shadow flurry ends with a 1 s knock-up within 60,000 of its last target; his own flurries also teleport him behind it. |
| Q. Shadow Grab | 150,000 through walls: hook, drag, dash, Haymaker (2 s knock-up). A miss makes a shadow flurry. |
| E. Shadow Anchor | A shadow ≤80,000 away fears within 50,000 for 1 s, then flurries; press again (or fall to 30% HP) to teleport to it. |
| S3. Smother | Self-cast line: silence, slow, Smothered (−20% Armour, −15% attack speed); haste and a shield for him; then a flurry. |
| R. Clone Army | Refills shadows; 3 clones in a kunai fan (plus 3 around the anchor) mimic his grab and Smother for 6 s at 50% damage. |

**Shadow Grab art** (Codex, round 190): a reaching hand, a latch, a pull ribbon and an upward burst, packed by `tools/build-contender-grab-art.py`. Smother and the clones still need their own art from Codex.

## Who decides his casts
The **game's own AI** presses S1 (cooltime 720, range 150,000), S2 (a 30-tick slot, range 80,000, so it can be pressed twice) and the ult (4,800, range 80,000). The Native effects drop `ct_cast:<slot>:<x>:<y>` markers, and the passive runs the order on its next update. **S3 Smother has no engine slot**: the core casts it at the nearest enemy champion in reach once it's off cooldown (level 3). The core's `decide()` only runs in the Lab.
