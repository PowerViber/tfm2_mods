# Round 150: Rally plays and balance, Raze far units (native 0.10.86)

Native 0.10.86, tfm2_custom 0.2.87, Valorant 0.1.29 (both require 0.10.86). Built on PR #63 (Rally 0.10.85).

## What you asked
- Give Rally's AI named plays, a Pace spend or save policy, an Impossible safety gate, and jump timing from athlete stats, plus a viability harness.
- Cap Rally's effects so two Rallys don't flood a tick.
- Let Raze skip far units each tick.
- Your decisions:
  - White form and local Impossible at **9.5 m** (was 10 m);
  - **peer-like stats**, balanced through multipliers instead.

## Rally
- **Plays:** Quick Spike, Set Spike, Receive, Serve, Impossible and Support.
  - Danger is relative: a spike pays a quarter of the incoming danger, because palming or Receiving stands in the same danger.
  - Under pressure, the run-up heads away or to a clear side so a perfect takeoff is possible.
- **Pace ladder:**
  - Before any form, she saves 3 bars to reach Black (6 m); once Black, 5 bars to reach White (9.5 m).
  - Bursts are valued by the height threshold they cross, so she never arms 1-bar Bursts any more.
  - Impatient athletes still spend as they go.
- **Impossible safety gate:** she starts only if no enemy can reach her during the 5 s charge, two allies are near, or the target is controlled or low. Otherwise she spikes White and keeps her Pace.
- **Timing:** perfect takeoff and contact chances come from control_speed, skill_hit and Micro, deterministic per seed. Manual play is exact.
- **Rules (rally-3.4):**
  - White form and local Impossible need a 9.5 m contact, approved by you on 2026-11-02.
  - Stats: base attack 80, growth 8, armour and magic-resistance growth 6.
  - Multipliers: spike 120 → 50, Black 500 → 180, White 600 → 220.
- **Effect budget:**
  - Her body, ball, helper, Pace bar, height label and crater ground sprites always draw.
  - Bursts share 12 tokens (+2 per tick, at most 6 per tick, ambient bursts only above a reserve of 4).
  - Two Rallys with storms: bursts peak at 6 per tick (was 176), and gameplay is identical with and without the budget.

### Harness (scripted peer models)

Average duel win rate: **56.4%** when her Receive bank fills from verified hits (as in the Lab), and **44.4%** in the game as shipped, where native Receive receipts stay off until calibrated.

| Opponent | L1 | L6 | L10 | L15 |
|---|---|---|---|---|
| Levi | 71 | 0 | 0 | 0 |
| V1 | 100 | 46 | 4 | 0 |
| Minato | 100 | 96 | 54 | 42 |
| David | 83 | 13 | 8 | 0 |
| Raze | 100 | 100 | 100 | 100 |
| Contender | 100 | 71 | 96 | 71 |

- **Why the extremes:**
  - Levi and David are modelled with large estimated ultimates and gap-closers; Raze's own harness scores 8–17% against them.
  - The Raze model kites away and never closes, so Rally always wins.
- **Impossible:**
  - With allies at level 15, she arms 17–37 three-bar and 6 five-bar Bursts per 3-minute fight.
  - In those fights she still dies before chaining Black → White → Impossible, which needs about 90 s alive. Her form resets on death.
  - A dedicated test shows a surviving top athlete completes the whole chain and attempts it with the gate allowing.
  - Whether it shows up in your matches depends on how long she survives.

## Raze
- **Far units:** minions, camps and structures beyond 170,000 are read every 30 ticks. Champions, units within range and her own targets are read every tick. Camps are only read in the farming window.

| Match | Host calls per tick | µs per tick |
|---|---|---|
| One Raze | 7,652 → 5,959 | 373 → 258 |
| A Raze on each team | 11,304 → 7,974 | 512 → 399 |

- Two Razes now cost about 2.0× a match without Raze, down from 2.8×.
- Equivalence test: across 600 ticks, decisions differ only when a unit more than about 150,000 away is involved.

## Verification
- `cargo test --release`: 975 passed, 0 failed, 12 ignored. No new warnings; the 2 old ones are in `valorant_guns.rs`.
- `rally_lab`: 126 passed. `raze_lab`: 136 passed.
- `generate-rally.py --check` and `verify-rally.py` pass.
- Rally Lab browser parity: 600 frames on Linux Chromium.
- The Windows DLL reports 0.10.86.

## Still open
- Native Receive and support receipts are not calibrated, so they stay off in game.
- Live checks: Rally's play variety, Impossible frequency, no lag with two Rallys, Raze unchanged.
- Contender realm hooks wait for your `realm_trace.jsonl`.
