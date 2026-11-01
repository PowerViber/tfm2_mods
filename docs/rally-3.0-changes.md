# Rally 3.0 — Pace, Aura Burst and Impossible Spike

This continues [draft PR #63](https://github.com/PowerViber/tfm2_mods/pull/63) after the published 2.1 and 2.2 snapshots. The 2.1 handoff and original reports, and the versioned 2.2 evidence, remain unchanged. Rules are `rally-3.0`; Lab transport is v2. This is a review prototype, not a packaged release.

## Implemented behavior

| System | Rally 2.2 | Rally 3.0 |
|---|---|---|
| Resource | Flow plus three regenerating charge pips | 0–100 Pace, displayed in five 20-point bars; no timed refill |
| Spending | Separate paid Black jump | Choose 1–5 complete bars and arm Aura Burst for eight seconds |
| Commitment | Paid jump preparation | Receive/Set acceptance or offensive takeoff; run-up alone spends nothing |
| Spike strength | Height multiplier and Set multiplier | Attack-derived base, separate additive Spike Power, height multiplier, then flat Receive force |
| Receive | Small maximum-HP bank, cleared by role changes | Calibrated force capped at 1000% of snapshotted base spike; survives role changes |
| Exceptional attack | White is the highest height tier | Impossible is a separate White-form attack requiring five-bar Self Set, Serve and perfect apex contact |
| Recordings | Transport v1 / 2.2 rules | Transport v2 / 3.0 rules; old recordings remain inspectable, never silently replayed under new rules |

A valid champion spike at four metres or higher, verified teammate support contribution, or qualifying perfect Receive earns 15 Pace once per action. Death clears Pace and the bank. Burst-origin actions, including a later return using force stored by Burst Receive, cannot pay their own Pace cost back.

### Aura Burst and exhaustion

For N selected bars:

- Spike takeoff: +0.1 m and +25 Power points per bar.
- Self Set: its existing +0.3 m, +15 Power and +10% approach speed; additionally +0.6 m and +15 Power per bar.
- Teammate Set: its existing support plus +3 percentage points Attack/AP and +1 percentage point movement per bar.
- Receive: +15% shield per bar, a 1 + 0.2N second capture window and a 0.15 + 0.03N second perfect window. Perfect protection stays capped at 95%.

Set and Spike can each receive a separately funded Burst. Costs come from actual available Pace. A completed Set can remain prepared during exhaustion.

Exhaustion lasts 1 + 0.4N seconds and applies one -20% movement modifier. It prevents new Burst commitment and Pace gains, while ordinary attacks and techniques remain usable. Set starts exhaustion when preparation resolves; Receive starts it when the capture window closes; a spike starts it at release or a failed/interrupted attempt. Failed commitments retain their spent cost.

### Damage, speed and permanent forms

`D = 70 + floor(1.2 × current Attack)`.

Power starts at 100. Add only the highest permanent-form bonus: yellow +10, Black +25, White +40. Add +15 Self Set, +15 per Burst Set bar, and +25 per Burst Spike bar.

Spike damage is `floor(D × Power/100 × contact multiplier)`, followed by eligible flat Receive force once. Contact multipliers remain normal ×1, yellow ×1.8 increasing to ×2.3, Black ×5 and White ×6. Impossible replaces White's multiplier with ×8. Multipliers do not multiply the stored force.

Actual contact height selects the attack: below 4 m normal; 4–<6 m gold Thunder; 6–<10 m Black; 10 m+ White. Successful timed contact earns the permanent form before computing that shot's Power and speed. Lower later contacts do not downgrade the permanent form.

Velocity uses the permanent-form base speed (180k / 360k / 600k / 840k world units per second), multiplied by Power/100 and a Receive fill bonus of up to ×2. It caps at 600 game-calibrated km/h using 30,720 world units per metre. Integer simulation ticks conservatively round travel speed down. Power and actual km/h are separate Lab readouts.

Basic palms and ordinary 75%-Attack volleys keep their existing primary-attack identity and cadence. A volley can consume the Receive bank and its velocity bonus, but it does not consume Aura Burst or adopt the spike damage formula. Gun/item/basic combat is otherwise unchanged.

### Receive proof boundary

When an empty bank receives a qualifying event, it snapshots D. Its maximum stored force is ten times that value. The displayed percentage is `floor(stored force × 100 / D)`, capped at 1000%. The bank expires eight seconds after the last qualifying receipt; Set, run-up and role changes preserve it.

On spike or volley release, the bank is reserved and removed. Its flat damage goes to the first enemy champion struck, once across the entire piercing/impact ledger. A miss has no refund. Burst provenance is retained until release.

The core accepts explicit pre-Receive-reduction force only with verified incoming/champion flags. It never reconstructs force from aggregate HP changes, shield disappearance or requested damage. The Lab models ordinary armour before its own Receive reduction and labels those receipts simulated.

**Native incoming-force and ally-support contribution remain disabled.** The current supported callback has not been calibrated for stage, shields, unrelated mitigation or rounding. There is no new hook, damage-healing workaround or assumed inversion. See [capability evidence](rally-results/rally-3.0/native-capabilities.json).

### Impossible lifecycle

Qualification requires White earned before takeoff, a five-bar Self Set, active Serve, perfect takeoff and contact within 0.2 m of that setup's maximum. Current maximum: `(4.8 + 2.5 + 0.3 + 3.0) × 1.7 = 18.02 m`.

A valid contact reserves Power, Set and Receive force and begins a five-second apex hold. Rally remains visible, targetable and stationary. One Rally-owned reduction state selects the strongest applicable protection, including the 50% charge reduction.

Aim can change for 4.4 seconds inside the original fixed V. The final 0.6 seconds lock the endpoint and publish the full line and endpoint warning. A committed Serve can extend once to finish this charge and descent; repeated updates do not refresh it. Damage alone does not cancel the charge. Enemy stun, airborne or silence does.

The released ball passes through units and walls to the committed in-bounds endpoint. It uses one hit ledger, so line and endpoint splash cannot hit the same enemy twice. Receive force reaches only the first champion. The endpoint creates a 120,000-radius crater, one 0.5-second knock-up and four seconds of 30% slow. The ball returns automatically. Release or interruption starts six seconds of exhaustion.

A committed speed of at least 300 km/h triggers Overpressure: 20% more warned radius and presentation changes, with no extra damage packet or stun. The demonstration's 230 Power and 1000% Receive bank produce 452.8 km/h and a 144,000-radius Impossible crater.

Realm changes cancel pending charge/projectiles/fields without cleanup damage. Native snapshots filter interactions through the existing realm model and use the realm's bounds; warning movement guidance filters duel membership. This does not upgrade the game's existing stopgap realm bridge into true host isolation.

## Lab and athlete decisions

Open `/editor/rally.html?build=rally-3.0`. Q/E/Space/F/R remain Receive/Set/run-up/contact/Serve. Keys 1–5 select cost; Shift+R arms Aura Burst; Escape cancels unspent arming.

The Lab shows Pace, cost, timers, bank/1000%, reserved force, Power, km/h, contact/maximum height and Impossible qualification. Practice presets can choose all permanent forms. “Play Impossible sequence” executes actual shared-core commands with a clearly declared White/Pace practice setup.

All thirteen pinned attributes remain decision/execution inputs. Manual commands remain profile-independent. Planning reserves room for Burst alternatives within 64 evaluated candidates, beam width eight and three-action continuations. Charge re-aim review is bounded and uses visible opponents only. A fixture demonstrates the automatic planner choosing and reaching Impossible with real resource commitments.

Old recordings show their version/revision/frame count and remain available for inspection. Replaying them requires their original rules; v1 is not relabelled v2.

## Presentation and delivery gates

[Review page](../editor/rally-review.html) · [actual Lab animation](rally-results/rally-3.0/impossible-spike-lab.gif) · [verification](rally-results/rally-3.0/verification.json).

New presentation is preview-only. Existing White/Thunder artwork supplies staged layers; no new body or effect source has been approved or installed. Frozen body, stroke and White atlases remain unchanged. The 3.0 preview repacks existing main-atlas frames, replaces the obsolete Flow/pips UI with five bars, and adds height labels through 19 m. Every retained frame and timing is checked against the frozen 2.2 atlas.

Native hazard timing/geometry is implemented, but enlarged contours and the final line currently use bounded existing markers pending continuous artwork approval and actual-host rendering review. The existing native effect-point clamp can also affect high poses near the map's upper edge. Lab camera behavior does not prove native 18 m visibility or clipping.

Full-V vision remains a labelled Lab simulation. The native SDK has no verified team-sector reveal setter; native AI never receives hidden positions through this feature.

No shipped Rally definition, roster, release version, installed mod, career or packaged DLL is changed by this update. Artwork approval, shipped integration, release packaging/Mod Manager verification, receipt calibration, matched-role balance and actual-game acceptance remain separate. Automated tests and scripted timings do not establish host-attributed damage, crowded-game performance or a 45–55% peer win rate.
