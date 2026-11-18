# Contender 2.0: shadow art handoff (for Codex)

**Completed 18 November 2026:** Reihan superseded the retained ring-boxer appearance and approved a new hooded shadow-boxer body plus separate violet/ivory effects. See [the completed artwork, reproduction and verification record](contender-shadow-review/README.md). Both bodies and effects now use their own approved packed assets; installation and native visual acceptance remain separate. The table below is the historical placeholder handoff.

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

## `contender-2.3` uses (for Codex)
Every 2.3 effect reuses a packaged cue at a packaged length, so nothing is unbound; better-fitting art is welcome.

| Cue | 2.3 use |
|---|---|
| `ct_swap_t18` | A dodge: the shadow left where he stood |
| `ct_launch_t18` | Shadow Merge starting |
| `ct_mb_<tile>` / `_l` | Approved opaque Merge body slots, selected by the native pose clock; old embedded definitions keep their prior fallback |
| `ct_cross_t18` | The step-out's cross |
| `ct_cleave_t14` / `_t12` | Cyclone clone / the aimed self-cleave (now a 35,000 circle) |
| `ct_echo_t12` | Shadow Pact echoes, Tether latching, each tether copy, and a flash on each tethered enemy every 12 ticks |
| `ct_uppercut_t20` | The Tether yank's uppercut |
| `ct_haymaker_t18` | A Shadow Break (and the haymaker) |
| `ct_slam_t20` | No longer used |

**Wanted when there's time:** a tether line or latched shadow on enemies, a dodge after-image, a Break burst, and pips for the bank. The bridge publishes `ct_bank:<0..20>` (no dedicated native bank artwork yet), and Merge now uses the approved `ct_mb_` body bindings; `ct_charges:<n>` stays at 0..5 (merged dodges above 5 show as 5).


## Approved fused Merge completion (18 November 2026)

Reihan approved the fused charcoal/violet form. The full 24-drawing animation extension, mirrored atlas, native pose-driven artwork buffs and default Lab integration are complete in this PR. Normal body/effect pixels and combat rules are unchanged. See `docs/contender-merge-integration-review/README.md` for hashes, timings, current 2.3 checks and pending actual-game alignment. Older embedded definitions use their existing fallback until a separately backed-up migration.
