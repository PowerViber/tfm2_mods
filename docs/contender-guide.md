# Contender, the Shadow Boxer (contender-3.1.3)

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

## AI and athlete stats (`contender-3.1.3`, round 184)
- **Every skill is used.** The ult (Shadow Link) peels or holds a supported group. S2 Shadow Flurry commits when the fight is safe, or trades a dodge for a charge. S1 Merge goes in on supported approaches.
- **Farming:** when calm, he farms with **S2 Shadow Flurry** (3+ creeps, or one big camp monster within 50,000, using real charges only, never a traded dodge) as well as with Merge.
- **Athlete stats shape his play.** Each bounds an existing check and adds no power:

| Stat | Effect |
|---|---|
| **Aggressive** | Commit HP and chase range (Codex, 3.0). |
| **Judgement** | Retreat line ±3% around 35%. Below 35 Judgement, he accepts one more enemy than his support covers. |
| **Skill Accuracy** | Below 40, Shadow Link goes on a single champion; otherwise it waits for two. |
| **Micro** | Dodges kept before trading one for a Flurry charge: 0 (below 40), 1, or 2 (70 and up). |
| **Control Speed** | Below 50, up to 12 ticks before committing S1 or S2 to a new target. |

- **Guard test:** his own AI in a 1v2 for 20 s at athlete 20/50/80 and levels 3 and 9 must stay at or below 1.9× a plain champion's basics and 25% dodged. It measures 1.59–1.79× and 14–18%.
