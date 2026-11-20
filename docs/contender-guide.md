# Contender, the Shadow Boxer (contender-3.6)

Contender dodges with a shared pool of shadows, stacks Rhythm on one enemy until every passive is roaring, bursts with
Shadow Flurry and drags a target in with Shadow Grab. All numbers live in `rules/contender.json`.

| Move | Rules |
|---|---|
| Shadows and Counterpunch | 10 shadows (5 pips, 2 each), shared by dodges (1) and Flurry (2). One returns every 4 s; each landed non-Flurry hit takes 0.1 s off the wait per missing shadow. A dodge has a 0.4 s gap. The dodge shadow punches the attacking champion within 45,000 for 20 + 30% Attack. |
| Shadow Pact and Rhythm | A basic echoes after 1 s for 3% damage (twice while merged). Basics, Flurry and Grab build up to 20 stacks per enemy for 4 s. Each stack: the target takes +1.5% damage from him and loses 1.5% Armour (Body Shots), and he gains +1.5% attack speed and +0.5% move speed against it (Footwork). |
| Shadow Break | A basic or Counterpunch on 20 stacks deals 4% max HP + 20% Attack and knocks the enemy up for 1 s (at most once every 5 s per enemy). |
| Q. Shadow Merge | 14 s cooldown. For 6 s: +20% move speed, +30% attack speed, +15% Attack, CC immune; enemies that damage him are slowed 30% for 1.5 s. Every second a cross hits within 50,000 for 12 + 15% Attack. |
| E. Shadow Flurry | Unlocks at level 3. Costs 2 shadows. Vanish and throw up to 30 punches over 1.5 s for 2 + 7% Attack each, reappearing in place. Recast at once if he has the shadows. A successful burst primes a Haymaker. |
| R. Shadow Grab | Unlocks at level 5. 12 s cooldown. The shadow flies up to 90,000 in a line (8,000 wide). The first enemy it touches takes 20 + 60% Attack, is dragged to him and knocked up for 1 s. |

[Contender 3.1 review and verification](contender-31-review.md)


## Who decides his casts
The **game's own AI** casts his skills, like a base champion's (Reihan: "revert back to the basics, since its own game already has an ai").

| Slot | Cooltime | Range | Targets | Native effect |
|---|---|---|---|---|
| S1 Shadow Merge | 840 ticks | 60,000 | any enemy but towers | `contender_s1` |
| S2 Shadow Flurry | 1 tick | 50,000 | any enemy but towers | `contender_s2` |
| Ult Shadow Grab | 720 ticks | 90,000 | an enemy champion | `contender_ult` |

- **How a cast runs:** each Native effect only drops a 5-tick marker (`ct_cast:s1` / `s2` / `ult:x:y`). His passive consumes it on its next update and runs the core's order, the same as a manual Lab order.
- **What the core still enforces:** the Merge and Grab timers, the **shadow pool** (a Flurry needs 2) and active Flurry sequences, and the level unlocks.
- **Level unlocks:** the engine locks skill2 until level 3, so the Flurry unlocks at level 3.
- **Movement:** he has no native steering or hold hints. The engine moves him.
- **Passives still run natively:** dodges, Counterpunch, Shadow Pact and Shadow Rhythm.
- **The core's `decide()`:** it now runs only behind the Lab's "Lab AI (not used in game)" checkbox.
