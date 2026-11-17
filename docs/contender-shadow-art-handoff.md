# Contender 2.0: shadow art handoff (for Codex)

Contender keeps his approved boxer body. His new kit (`docs/contender-guide.md`) needs **shadow** art. Until it arrives, every name he plays is bound to an existing effect (`tools/generate-contender.py`, `PLACEHOLDER`).

| Effect name | Used for | Placeholder |
|---|---|---|
| `ct_shadow` | The projected shadow's body (drawn every 10 ticks at its carrier), and the cleaving clone | kagami `clone_in` |
| `ct_launch` | The projection's launch | kicker `dash` |
| `ct_swap` | The projection swap (both ends) | kagami `swap` |
| `ct_cross` | The shadow's punches, and the swap's cross | kicker `kick` |
| `ct_cleave` | One spin of Shadow Cyclone (25,000 radius) | grimhald `slash` |
| `ct_slam` | Slam | grimhald `slam` |
| `ct_uppercut` | Uppercut | kicker `drop_in` |
| `ct_echo` | Shadow Pact's echo hit (1 s later) | kagami `shard_hit` |
| `ct_vanish`, `ct_appear` | The ult's vanish and reappearance | grimhald `ghost` and `ghost_end` |
| `ct_flurry` | One flurry punch | akane `dash_slash` |
| `ct_haymaker` | The post-ult haymaker | kicker `wall_slam` |

**Wanted:**
- **A shadow silhouette:** a dark, translucent version of his body in both facings for idle, walk, jab, cross, cleave spin, slam, uppercut and flurry punches, at his body's scale.
- **Charge pips:** up to 5, under him; the core publishes a `ct_charges:<n>` buff.
- **Small hit flashes:** for the echo, the stun and the knock-up.
- **Budgets:** the same as the other champions.
