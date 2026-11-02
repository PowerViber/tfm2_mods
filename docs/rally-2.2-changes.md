# Rally 2.2: implemented changes after the 2.1 review snapshot

Rally 2.1 was published in draft PR #63 at commit 7f14d78 before these requests were implemented. This document records the new 2.2 behavior; [the original handoff](rally-review-handoff.md) and the original verification files remain the 2.1 baseline.

## Behavior comparison

| Feature | Published 2.1 | Implemented 2.2 |
|---|---|---|
| Released spike return | Hit returns; miss, wall and range expiry ground the ball | Every released spike returns automatically; return has no damage. Ordinary basic-volley retrieval is unchanged |
| Attack tier | Black required an armed finisher; White used Serve/apex flags | Exact actual contact height alone selects the attack |
| Permanent form | Required a delivered hit | Successful timed contact/release earns the form immediately, even if the projectile later misses |
| Ball speed | Uniform 270k units/s before Set | Follows permanent form independently of current attack height |
| Run-up | 1.4x for 1.5 s; one initial gauge | 2x for 3 s; repeating 36-tick gauge, optional takeoff; 5 s cooldown retained |
| White-capable flight | Low hold then forced apex blink | Continuous reachable arc with earlier contacts; White blink only after a valid 10 m+ contact |
| Serve walls | Stopped at walls | Passes map and construction walls, keeping first-unit collision, map bounds and realm filtering |
| Full-V vision | Absent | Reveal request emitted at V opening; clearly labelled Lab simulation. **Native execution unavailable in the inspected SDK** |

The separate Black command still spends its existing resources and supplies the higher jump. It no longer controls the attack's tier or grants extra stacked damage. A failed contact, interrupted toss or expired boost grants no permanent power.

## Exact contact bands and permanent forms

| Unrounded contact height | Attack |
|---|---|
| Below 4 m | Normal spike, no thunder |
| 4 m to below 6 m | Yellow Thunder |
| 6 m to below 10 m | Black Thunder |
| 10 m or higher | White Storm |

The existing damage prototypes remain: normal 1x; yellow 1.8x increasing to its existing 2.3x cap; Black 5x; White 6x. Self Set applies once. Black and White replace the ordinary height multiplier.

The highest permanent form remains until death and does not downgrade on lower contacts. For example, White Rally contacting at 3.5 m produces a normal attack with White-form speed and retains the White body/role bonuses. Contact at 5 m produces yellow damage and effects, not White damage. Jump height is snapshotted at takeoff; earning a form cannot retroactively raise that contact.

New speed defaults are **prototypes**, not separately approved numerical choices:

| Permanent state | Units/second | Units/tick |
|---|---:|---:|
| Normal Yellow | 180,000 | 3,000 |
| Yellow Thunder | 360,000 | 6,000 |
| Black Thunder | 600,000 | 10,000 |
| White Storm | 840,000 | 14,000 |

The first qualifying contact earns its state before projectile speed is snapshotted. Self Set's existing 20% speed bonus then applies once. Yellow Thunder changes speed, not the Black/White role-bonus values. Death clears every earned state.

Height labels round down; a 5.999 m contact must not display 6.0 m. Body appearance, exact attack tier and projectile speed are reported separately in the Lab.

## Directional run-up and earlier contacts

Run-up remains independent of mouse spike aim. It lasts up to 180 ticks at 200% base movement, with terrain/control checks and normalized direction. It does not force takeoff or toss the ball.

Gauge age repeats modulo 36. Perfect remains ticks 24-29, and each new cycle requires fresh actual displacement; movement from a prior cycle cannot pay for a stationary later takeoff. Serve retains its stationary takeoff exception.

All jumps use a continuous shared arc. The clone toss reaches the contact area earlier, continues moving through it while rising/falling and permits yellow/Black contacts before a White-capable apex. No invulnerability or global hit-stop is added. The existing vulnerable contact hold remains.

## Serve penetration and vision limit

Serve's committed projectile ignores map/construction wall rectangles. Ordinary spikes and basic volleys retain wall collision. Serve still stops on its first eligible unit, respects bounds and can damage only eligible realm members. It does not destroy walls or allow Rally to move through them.

At V opening, the shared core emits one full-sector reveal request. Its intended lifetime is the open Serve V: the active firing/flight state until normal completion, interruption, timeout or death. This duration is an implementation choice for review.

The current stable SDK has:
- A read-only team/entity visibility query.
- Global hide/banish operations.
- No verified team-scoped fog/sector reveal operation.

See [SDK contract](../native/mod-api-stable/src/contract.rs), [simulation API](../native/mod-api-stable/src/sim.rs) and [capability record](rally-results/rally-2.2/vision-capability.json). Global invisibility and controller-only visibility tags are not substitutes.

The native adapter fails closed: it records the unavailable capability without changing invisibility or feeding concealed targets to the planner. This part is **not complete native gameplay**, and stealth precedence remains unresolved.

The Lab offers an explicitly labelled simulation switch. While the V is open, it exposes its entire in-range sector even across walls, leaves enemies outside it concealed and removes that exposure immediately when Serve ends. This is evidence of the intended geometry and lifecycle, not proof of host vision.

## Verification and review

- **918 native tests passed**, 0 failed, 11 ignored; four pre-existing Windows test-helper warnings.
- **77 Lab tests passed** and **20 browser checks passed**, including manual keyboard input, recordings and native/WASM parity.
- Actual editor routes, including the current review page/capture, passed in an isolated inert-game fixture.
- 24 development and 12 holdout scripted scenarios; five 128-unit timing runs. These are not live-game balance or host timings.
- Canonical data and existing preview art validation passed. Decoded art remains 31.4609375 MiB; atlas/source pixels are unchanged.

New regression cases cover exact 4/6/10 m boundaries; contact-earned forms despite misses; no receipt-created forms; lower attacks retaining White/Black speed and role power; failed contact; all spike returns; unchanged basic retrieval; later perfect cycles; no prior-cycle displacement farming; early Yellow/Black contacts in White-capable jumps; wall traversal with unit/boundary collision retained; and one reveal request at opening.

Current reports, hashes and refreshed animated observations are under [rally-results/rally-2.2](rally-results/rally-2.2). Original 2.1 reports and artwork captures remain intact. Rules revision is rally-2.2; transport remains v1. Older recordings can be inspected, but replay requires their original physics revision instead of silently using 2.2.

## Remaining gates

Native sector reveal needs host support or a separately authorized and verified bridge. Incoming caught-power and ally-support Flow receipts, actual-game movement/impacts/items, final renderer acceptance, crowded performance and matched peer balance remain pending.

This continues the draft review prototype only. Artwork approval, shipped definition/game-art integration, release versions, DLL packaging, installation and real-career migration have not been performed.
