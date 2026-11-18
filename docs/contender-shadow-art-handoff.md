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

**Wanted when there's time:** a tether line or latched shadow on enemies, a dodge after-image, a Break burst, and pips for the bank. The bridge publishes `ct_bank:<0..40>` (no dedicated native bank artwork yet), and Merge now uses the approved `ct_mb_` body bindings; `ct_charges:<n>` stays at 0..5 (merged dodges above 5 show as 5).


## Approved fused Merge completion (18 November 2026)

Reihan approved the fused charcoal/violet form. The full 24-drawing animation extension, mirrored atlas, native pose-driven artwork buffs and default Lab integration are complete in this PR. Normal body/effect pixels and combat rules are unchanged. See `docs/contender-merge-integration-review/README.md` for hashes, timings, current 2.3 checks and pending actual-game alignment. Older embedded definitions use their existing fallback until a separately backed-up migration.


## `contender-2.4` art request (completed below)
2.4 changes gameplay only. Every cue it plays is already packaged, so nothing is unbound. These are the visuals the new rules would benefit from; none of them may change combat, movement or control.

| Wanted | When it plays | Current stand-in | Notes |
|---|---|---|---|
| **Shadow Break burst** (`ct_break_t18` or similar) | On the enemy, on the hit after it reaches 20 Rhythm stacks: 6% max HP + 30% Attack, and its stacks reset to 0 | `ct_haymaker_t18` (shared with the post-ult haymaker) | It is now rare (one per 20 stacks), so it can be big: a violet implosion and shatter that reads as "the stacks were spent". |
| **Full-rhythm aura on him** (a named buff such as `ct_rhythm_full`) | For the 8 s after a Break, while his Rhythm buffs stay at the 20-stack level even though the target is at 0 | none (the core emits a `Rhythm` effect each refresh; the bridge has no named buff for it yet) | A low-key violet pulse at his feet or fists, readable over the normal body and the fused Merge body. Claude will add the named buff once a binding exists. |
| **Enemy Rhythm stack pips in the game** (`ct_rhythm:<0..20>` on the enemy) | While an enemy carries his stacks | Lab only (the violet fist icon with a count; gold at 20) | Match the Lab icon; gold at 20 warns that the next hit Breaks. It needs a bridge buff on enemies, which Claude can add alongside the binding. |
| **Bank pips** (`ct_bank:<0..40>` on him) | Always; the ult is ready at 40 | none (the buff is published unbound) | For example 8 pips of 5 stacks, or a ring that fills; it should be clearly "ready" at 40. |

**Budgets:** the same as the rest of his art. The Break and the aura must also read on the fused Merge body (`ct_mb_*`).

### Approved 2.4 completion — 18 November 2026

Reihan approved the new effects and indicators. The separate `contender_rhythm`
atlas and native/Lab bindings are integrated: `ct_break_t18` (and mirrored facing),
stable `ct_rhythm_full_<phase>` artwork buffs, owner-scoped
`ct_rhythm_p<player>:<1..20>` enemy markers, and existing `ct_bank:<0..40>` buffs.
The native presentation hooks are complete; no follow-up Claude hook addition is
needed. Break is now an explicit visual core cue, preserving all damage and state.
Cyclone also shows its canonical 25,000/35,000 circles.

See [the 2.4 review](contender-24-art-review/README.md) for approved sources,
reproducible packing, refreshed Windows/Lab evidence and pending actual-game
acceptance. The request table above records the former stand-ins, not remaining work.

## `contender-2.7` notes (for Codex)
- **Bank pips:** the 40-stack bank is gone (the ult is a 120 s cooldown again). The bridge no longer publishes `ct_bank:<n>`, so the bank-pip bindings are unused and can be removed or repurposed.
- **Dodge pips:** he has **one** dodge now, so `ct_charges:<n>` is only 0 or 1.
- **The Flurry banishes him:** it uses `entity_banish`, renewed every tick. Please confirm on Windows that:
  - enemies can't target or hit him;
  - the fused Merge body and the normal body are both hidden until he reappears;
  - the Flurry's punches still land while he's banished.
