# Round 144: Raze as a MOBA champion

Native 0.10.77; champion pack 0.2.77; Valorant pack 0.1.26. Rules revision `raze-3.6`. This builds on main after PR #50.

## What you reported

1. Make Raze viable as a MOBA champion, with all four directions: lane and wave clear, a reward for fighting, sustain and scaling, and a teamfight and ultimate identity.
2. You have never seen her ultimate; the animation doesn't show.
3. Give her a shield while airborne that lingers after landing, like the damage reduction.
4. Give her new skills that suit her personality.
5. Make her explosives go off sooner, and her projectiles reach farther and fly faster.

## Why the ultimate was never seen

The visual chain in code was complete (equip, ready, release, warnings, flight phases, blast), and the AI picks the rocket whenever it is allowed. Two things stopped it in real matches:

- **It probably never unlocked.** The bridge read only the engine's unlock flag, which is false before level 5 and unreliable in careers. Gojo, Magnus and Unified needed the `ult_learned` passive for exactly this; Raze had none. The Lab and every test started with the ultimate unlocked, so nothing caught it.
- **A career keeps its own copy of her view list.** Effect names added since round 143 don't exist in an unmigrated career, and a failed effect was ignored silently.

**Fixes:**
- Her champion data has `passive_ult` → `tfm2_custom_ai:ult_learned`. The bridge treats the ultimate as unlocked when the slot flag is set or the `gojo_ult_learned` buff is present.
- A view effect that fails to play is logged once per game to `raze_log.txt`, with a hint to run the career migration. With diagnostics on, every rejected skill is logged with the unlock state and her level.
- **Readability:** she holds the armed pose 12 ticks before firing. The rocket and its ready marker are drawn 1.5× larger, and the impact leaves a scorch for 15 ticks.
- **Test:** the ultimate starts locked, unlocks through the marker, and fires.

### Migrating a career

Run this on a copy of your career first. Inspection is the default and writes nothing:

```
python tools/migrate-champion-reworks.py <career file>
```

When the report looks right, apply it in place. A verified backup is written first:

```
python tools/migrate-champion-reworks.py <career file> --in-place
```

This replaces Raze's view list and adds `passive_ult`. Nothing else in the career changes.

## Faster explosions, longer and faster projectiles

| Value | Before | Now |
|---|---|---|
| Paint Shells fuse | 27 ticks (0.45 s) | 18 (0.30 s) |
| Cluster fuse | 24 (0.40 s) | 15 (0.25 s) |
| Boom Bot fuse | 12 | 8 |
| Blast Pack arm time | 30 (0.5 s) | 18 (0.3 s) |
| Paint Shells range / speed | 120,000 / 180,000 | 150,000 / 260,000 |
| Blast Pack throw range | 24,000 | 32,000 |
| Boom Bot speed / chase / tracking | 80,000 / 120,000 / 65,000 | 110,000 / 160,000 / 85,000 |
| Showstopper range | 120,000 | 150,000 |
| Showstopper start / top speed | 120,000 / 600,000 | 180,000 / 760,000 |
| Bucky full / falloff range | 8,000 / 18,000 | 10,000 / 22,000 (weapon range 22,000) |
| Bucky slug (new) | — | 30,000 |

The plan's "300,000 → 380,000" Showstopper row pointed at her own boosted flight cap. That would have made her fly farther, which wasn't asked for, so the rocket's top speed went up instead and her flight cap is unchanged.

## Her new kit

All numbers are in `rules/raze.json`; the game, the Lab and the tests read the same values.

**Air shield.** While airborne she has a shield of 6% max HP plus 40% Attack. It lasts the flight, then lingers for the landing-defence window (3–12 s by the champions near her landing; none nearby ends it at once). Later launches only top up what was lost. It doubles during Carnaval and shows the armour icon. The 15% damage reduction is unchanged and stacks with it.

**Hype and Carnaval.**

| Source | Hype |
|---|---|
| Skill hit on a champion | +12 |
| Skill hit on a minion or camp | +3 |
| Launch | +8 |
| Takedown | +30 |

Hype decays 5 per second out of combat. At 100 she bursts into **Carnaval** for 6 s:
- skill cooldowns recover twice as fast;
- +15% move speed;
- a doubled air shield;
- a fifth cluster on Paint Shells.

Gold-or-better athletes hold a full meter for a fight or objective.

**Graffiti.** Paint Shells leaves paint for 2 s. Enemies in it are slowed 20% and take 10% more from her.

**Encore.** Her first landing after 3 explosions within 4 s sends a shockwave: radius 20,000, 40% Attack, an 18-tick knock-up. It has a 10 s cooldown.

**Lane and wave clear.**
- Paint Shells may target 2+ minions (was 3+). Clusters deal +50% to minions and camps.
- With no champion in sight, Boom Bot hunts the nearest wave, camp or tower. It deals +40% to structures and objectives.
- Blast Pack detonations damage minions.
- **Bucky slug:** 120% Attack, range 30,000, every 90 ticks. She fires it at targets beyond 12,000 when she is safe.

**Fight rewards.**
- **Hot Hands:** a takedown, or 2 Paint Shells main-blast hits on champions, returns the free Paint Shells charge.
- **Combustion:** skill hits add Burn. At 3 stacks the target takes 6% of max HP as true damage and is revealed for 2 s.
- **Reset:** a takedown within 3 s of her damage returns the free Blast Pack charge.
- Rewards never add a bought charge.

**Sustain and scaling.**
- Mod Power `[35, 0, 10, 10, 10, 0, 10]`: her dead AP bonus moved to Attack, and AP growth is 0.
- 10% omnivamp on skill damage, 30% against champions while landing defence or the air shield is up. She heals from the HP and shield the hit actually removed.
- Skills gain +4 base damage per level.
- Kills pay +100 credits, and takedowns restore 10% armour.
- She gets a free Sheriff at level 6 if she is still on the Bucky.

**Teamfight and ultimate.**
- **Primed:** a Boom Bot hit gives +25% to her next Paint Shells or Showstopper on that target.
- **Satchel combo:** pushing an enemy into a Paint Shells blast within 1 s stuns for 24 ticks.
- **Showstopper:**
  - leaves a burning zone for 3 s (radius 30,000, 8% Attack per second);
  - each champion takedown refunds 25% of its cooldown.
- **Mastery Iron → Radiant:** it only sharpens her planning (Skill Accuracy, Micro and Strategy), never her raw power.

## Is she viable? The harness

`native/tfm2_custom_ai/src/raze_viability.rs` drives the real shared core (the same code as the game and the Lab) in automatic mode. Run it with:

```
cargo test --release raze_viability -- --ignored --nocapture
```

A fast smoke test runs with the normal suite.

**The opponents are scripted models, not their native brains.** Each is built from its shipped data (stats, growth, basic attack, data-driven skill damage) and its Mod Power line. Damage that only lives in native code (Levi's cuts and Rampage, the V1, Minato and David ultimates) is estimated and marked in the file.
- Peers react after 15–60 ticks and land 80% of their skills.
- Ranged peers kite to 80% of their range.
- Nobody dodges her grenades.
- 24 seeded starts per matchup, with a 30 s cap; on a draw, the side with more HP left wins.

### Before tuning (the round-144 kit at 300% skill damage)

| Opponent | Lv 1 | Lv 6 | Lv 10 | Lv 15 |
|---|---:|---:|---:|---:|
| Levi | 100.0 | 41.7 | 20.8 | 45.8 |
| V1 | 95.8 | 100.0 | 100.0 | 100.0 |
| Minato | 100.0 | 100.0 | 100.0 | 100.0 |
| David | 95.8 | 29.2 | 29.2 | 33.3 |
| Stock ranged | 100.0 | 100.0 | 100.0 | 100.0 |

Average against her peers: **74.5%** (target 45–55%).

### Tuning

As you asked, damage came down first, never range, speed or fuse times. The Showstopper decided almost every duel at level 5+.

| Skill damage to enemies | Average against peers |
|---:|---:|
| 300% | 74.5% |
| 250% | 66.4% |
| 225% | 61.2% |
| 210% | 56.0% |
| **200% (shipped)** | **54.7%** |

A lighter Showstopper alone (250% skills with the rocket at 200 + 110% Attack) still gave 63.0%. Lowering Bucky's close-range damage barely moved the duels. So the shipped change is one number: `SKILL_DAMAGE_PERCENT` 300 → 200. That is still double her pre-round-143 damage. Self-damage is unchanged.

### After tuning (shipped)

| Opponent | Level | Win % | Kills/min | Deaths/min | Damage share |
|---|---:|---:|---:|---:|---:|
| Levi | 1 | 87.5 | 10.68 | 1.53 | 54.5 |
| Levi | 6 | 12.5 | 2.24 | 15.67 | 37.5 |
| Levi | 10 | 12.5 | 2.09 | 14.63 | 35.6 |
| Levi | 15 | 8.3 | 1.24 | 13.68 | 33.6 |
| V1 | 1 | 100.0 | 13.30 | 0.00 | 62.9 |
| V1 | 6 | 91.7 | 13.52 | 1.23 | 56.0 |
| V1 | 10 | 79.2 | 9.14 | 2.41 | 52.0 |
| V1 | 15 | 66.7 | 6.66 | 3.33 | 51.7 |
| Minato | 1 | 70.8 | 5.96 | 2.45 | 56.3 |
| Minato | 6 | 66.7 | 6.98 | 3.49 | 52.4 |
| Minato | 10 | 62.5 | 6.18 | 3.71 | 53.2 |
| Minato | 15 | 87.5 | 8.39 | 1.20 | 55.4 |
| David | 1 | 70.8 | 7.78 | 3.20 | 48.9 |
| David | 6 | 16.7 | 4.14 | 20.72 | 36.5 |
| David | 10 | 16.7 | 4.14 | 20.71 | 34.3 |
| David | 15 | 25.0 | 3.57 | 10.71 | 39.1 |
| Stock ranged | 1–15 | 100.0 | 16.5–19.4 | 0.00 | 74–80 |

Average against her peers: **54.7%**, inside the 45–55% target. The matchups are lopsided, though:
- She beats the ranged peers (V1, Minato) because her explosives outrange their trades.
- She loses to the divers (Levi, David) once their burst arrives at level 6.

The average lands in the band, but the individual matchups do not.

**Lane** (one minute against the stock ranged champion, damage to champions):

| Level | Raze | Stock | Ratio |
|---:|---:|---:|---:|
| 1 | 802 | 828 | 0.97 |
| 6 | 1,245 | 461 | 2.70 |
| 10 | 1,585 | 512 | 3.10 |
| 15 | 2,010 | 611 | 3.29 |

From level 6 she kills the stock champion inside the minute (her damage equals its HP). The ±15% target holds only at level 1. The stock model is a base-class champion without Mod Power, so no damage cut that kept her near her peers could meet it. I stopped tuning on the peer target.

**Wave clear and objective** (seconds):

| Level | Raze wave | Levi wave | Stock wave | Raze objective | Levi objective | Stock objective |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1.4 | 6.0 | 33.0 | 32.3 | 22.0 | 77.0 |
| 6 | 1.4 | 2.7 | 21.0 | 36.2 | 8.3 | 60.0 |
| 10 | 1.2 | 2.6 | 19.0 | 34.0 | 6.7 | 55.0 |
| 15 | 1.2 | 2.1 | 15.0 | 31.3 | 5.8 | 53.0 |

She clears a wave faster than Levi (the "within 20%" target is missed on the fast side), because one Paint Shells and its clusters cover a whole six-minion wave. Minion HP in the harness is an estimate (450 melee, 300 ranged, +22 per level). Objectives take her much longer than Levi, because Levi's Rampage is modelled as a full channel on the objective.

## Verification

- **Native tests:** `cargo test --release` 761 passed, 0 failed, 10 ignored, 0 warnings in the test build. The release library build shows two warnings about unused gun helpers (`basic_raw`, `Gun::upgrade`); both are unused on main too.
- **Shared core:** `native/raze_lab` 132 passed, and `editor/raze.wasm` is rebuilt.
- **Browser tests:** all pass (lab parity of 6,480 native and wasm frames, v32, v3, shop, keyboard, dome, balance). I ran them with Linux Chromium and the Linux vector binary instead of Edge and the `.exe`, and updated the balance test to the 200% numbers.
- **Other checks:** `verify-raze.py` passes with approved pixels unchanged. The 12 migration tests pass. `generate-roster.py --check` is up to date.
- **Windows DLL:** built with `SOURCE_DATE_EPOCH` set to the commit time; SHA-256 `9afab43333f4483c90dc9e0e0c7aafcc6b6e872f74c3bf90e9610e89248fa3c7`.

**Not run here:** `check-raze-release.py` and `test-raze-server.cjs`. Both need Windows, Edge and the Mod Manager. `tools/record-raze-delivery.py` now expects 0.10.77 and `raze-3.6`, so it will fail until those Windows checks regenerate its inputs.

## Limits

- **Combustion's reveal** shows the target only to Raze's own AI and the Lab. The engine has no team-vision API.
- **Shield linger:** if she lands early (a wall or crowd control), her air shield can outlive the landing by the remaining flight ticks. The API can't remove one shield layer.
- **Her normal gun tracer** (`tfm2_valorant_raze_tracer`) is not defined in any view list, so her gun shots probably draw nothing. This predates this round. The slug uses Omen's tracer, which is defined.
- **The harness is a model.** Peer damage that lives only in native code is estimated, and nobody dodges. Treat the win rates as relative, and check them against live games.

## Please check in game

1. After migrating your career, does the ultimate fire at level 5+ and is it visible?
2. Does the shield show in flight and linger after landing?
3. Does the Hype meter fill, and does Carnaval trigger in fights?
4. Does she clear waves and survive skirmishes, and do divers (Levi, David) still feel like her hard matchups?
