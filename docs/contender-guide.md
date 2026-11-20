# Contender, the Shadow Boxer (contender-3.2)

Contender retaliates when his passive dodges a champion hit, stacks Rhythm with a
short Flurry and sets up teammates with Shadow Link.

| Move | Rules |
|---|---|
| Shadows and Counterpunch | Five dodges, one recovered every 15 seconds, with a 0.4-second dodge gap. The dodge shadow punches the triggering champion once within 45,000 for 20 + 30% Attack physical damage. Terrain, cage and realm boundaries stop the counter. |
| Shadow Pact and Rhythm | A basic echoes after one second for 3% damage. Merge doubles these echoes. Basics and Flurry stack Rhythm, up to 20 per enemy. A basic or Counterpunch can trigger Shadow Break on a full stack. |
| Q. Shadow Merge | Gain movement speed for six seconds. Once per second, hit each eligible enemy within 50,000 of his current position for 12 + 15% Attack physical damage. Each landed Cross hit cuts 0.75 seconds from Flurry recharge. |
| E. Shadow Flurry | Unlocks at level two. Vanish and throw up to ten alternating punches in one second. Three charges, 16 seconds to recharge one and a 0.75-second gap after reappearing. Other landed non-Flurry hits cut the active recharge by 0.5 seconds. A successful burst primes a damage-only Haymaker. |
| R. Shadow Link | 75-second cooldown, 60,000 range, 85,000 radius. Refills all five dodges. After a 0.75-second warning, remaining links are pulled for 0.2 seconds, knocked up for 0.5 seconds, and stunned for 0.4 seconds. Zero damage. |

In the Lab, **Shift+E** or **Exchange dodge** spends one passive dodge to gain
one Flurry charge. The exchange needs an empty Flurry slot and leaves the
0.75-second cast gap in effect. When the in-game AI has zero Flurry charges,
it can spend one dodge as part of an otherwise safe Flurry cast.

Contender's AI checks allied follow-up, enemy numbers, towers and retreat paths
before committing Flurry. The projected Shadow Link serves a supported group
catch or a teammate peel. Manual Lab orders retain their own controls.

[Contender 3.1 review and verification](contender-31-review.md)


## Who decides his casts (`contender-3.2`, round 185)
The **game's own AI** casts his skills, like a base champion's (Reihan: "revert back to the basics, since its own game already has an ai").

| Slot | Cooltime | Range | Targets | Native effect |
|---|---|---|---|---|
| S1 Shadow Merge | 840 ticks | 60,000 | any enemy but towers | `contender_s1` |
| S2 Shadow Flurry | 45 ticks (the cast gap) | 50,000 | any enemy but towers | `contender_s2` |
| Ult Shadow Link | 4,500 ticks | 60,000 | an enemy champion | `contender_ult` |

- **How a cast runs:** each Native effect only drops a 5-tick marker (`ct_cast:s1` / `s2` / `ult:x:y`). His passive consumes it on its next update and runs the core's order, the same as a manual Lab order.
- **What the core still enforces:** the Merge and Link timers, the **3 Flurry charges** (the engine may press S2 every 45 ticks, and the press is refused with no charge), and the level unlocks.
- **Level unlocks:** the engine locks skill2 until level 3, so the Flurry unlocks at level 3.
- **The dodge trade:** the game AI never trades a dodge for a Flurry charge. That's a manual Lab action only (Shift+E).
- **Movement:** he has no native steering or hold hints. The engine moves him.
- **Passives still run natively:** dodges, Counterpunch, Shadow Pact and Shadow Rhythm.
- **The core's `decide()`:** it now runs only behind the Lab's "Lab AI (not used in game)" checkbox.
