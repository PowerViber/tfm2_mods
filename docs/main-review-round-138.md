# Review of main after PRs #33–#37 (round 138)

What was checked on `origin/main` (e4405e8), what was wrong, and what this round changed.

| Area | Finding | Status |
|---|---|---|
| Build | 391 tests passed, but 24 dead-code warnings | Fixed: 0 warnings |
| Magnus | `plan()` ran on every tick while nothing was pending; `plan_at` was never read | Fixed: `PLAN_RETRY` = 12 ticks after a fruitless search; 128-piece p95 2925 → 910 µs |
| Magnus | Full `Core` clone every tick (spawn rollback); `validate_preview` replays every tick | Left: about 1 clone per tick, small beside the planner |
| Frieren | Prep warning drawn on Frieren, not the target | Fixed: the target is marked for the whole preparation |
| Frieren | Spellcraft not gated on Skill 1 | Fixed |
| Frieren | Stark/Fern/ray/radius literals bypassed `rework.json`; Fern reached 200000 against a 150000 skill range | Fixed: 13 constants in `rework.json`; Fern 150000 |
| Frieren | Tooltips silent on piercing minions and Stark's damage attribution | Fixed |
| Frieren | Parity tool Windows-only | Fixed: runs on Linux too |
| Frieren | Kit design: no crowd control, about 3× basic attack damage | Reported, not changed (a balance call) |
| Unified | Review limit penalised catalogue order; Rust and JS disagreed on Grand Unification; stale JS constants and docs | Superseded: the kit was replaced (round 138 equation kit) |
| Vesper | Main added `farm_only`, a champion-only takedown flag and an observer; nothing reverted | OK |

The Unified rework itself is described in [unified-theory-guide.md](unified-theory-guide.md).
