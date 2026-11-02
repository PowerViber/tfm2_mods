# Rally 3.1 — global Impossible Spike and height-driven Power

This implements the approved 3.1 redesign on draft PR #63. [Rally 3.0](rally-3.0-changes.md), its reports, and the earlier 2.1/2.2 evidence remain unchanged. The current canonical revision is `rally-3.1`; Lab transport and pinned athlete profiles remain v2.

## Resources and one Burst per play

Choose 1, 3 or 5 bars, costing 20/60/100 Pace. Unsupported costs and incomplete balances are rejected. Arming still lasts eight seconds and does not spend resources. Run-up alone also spends nothing.

A stable, owner-scoped Burst-play identity records the payment at Self Set/Receive/teammate Set acceptance or offensive takeoff. A paid Self Set carries that payment into the prepared spike; another Burst cannot be armed or spent on the same play. Ordinary techniques remain available where their existing lifecycle permits.

Exhaustion is always 90 ticks (1.5 seconds), with one -20% movement modifier. It blocks new Burst spending and Pace gains. It starts at spike release/failure, paid setup expiry, Receive capture closure, or teammate Set delivery. Set preparation itself does not exhaust Rally before the prepared attack. Interrupted commitments retain their cost; resolution cannot repeatedly extend exhaustion.

Receive stores at most 300% of the ordinary base spike snapshotted when the empty bank begins. At 104 Attack, D is 194 and the cap is 582 force. The bank survives Set, run-up and role changes, expires eight seconds after the last valid receipt, and is consumed on the next spike/volley release. Its flat damage reaches the first enemy champion once; a miss has no refund. Source validation and duplicate/Burst-origin checks remain internal.

Native incoming-force and teammate-contribution receipts remain disabled pending host calibration. The Lab explicitly simulates receipts; it does not prove native mitigation or shield semantics.

## Height and charge formulas

For actual contact height h in millimetres:

- Ordinary base spike: `D = 70 + floor(1.2 × current Attack)`.
- Height Power: `floor(max(0, h - 2000) × 15 / 1000)`.
- Starting Power: 100 + highest form bonus (0/10/25/40) + 15 Self Set + the chosen Burst bonus + Height Power.
- Burst Set adds 15 Power and 0.6 m per bar; Burst Spike adds 25 Power and 0.1 m per bar. These are alternatives within one play.
- Damage: `floor(D × Power × multiplier / 10000)`, using multipliers expressed as 100/180–230/500/600/800.
- Velocity: permanent-form base speed × Power/100 × (1 + Receive percentage/300).
- Ordinary balls cap at 600 km/h; Impossible caps at 1200 km/h. Conversion uses 30,720 world units per metre and integer simulation ticks.

Impossible adds `floor(200 × elapsedTicks / 150)` Power throughout its 150-tick charge. Attack, starting Power, base form speed, contact height and reserved force are immutable contact snapshots. Current previews and final release are recomputed from those snapshots, never by multiplying the previous tick's result.

| Setup, at 104 Attack and no Receive bank | Contact | Starting → final Power | Starting → final speed | Final raw damage |
|---|---:|---:|---:|---:|
| White + five-bar Spike; no Set or Serve | 7.8 m | 352 → 552 | 346.5 → 543.4 km/h | 8,567 |
| White + five-bar Self Set + optional Serve | 18.02 m | 470 → 670 | 462.7 → 659.5 km/h | 10,398 |

A full Receive bank adds 582 damage once. It doubles the velocity before the cap: the full-height example reaches the 1200 km/h cap. These are prototype combat values, not established live-balance results.

## Global Impossible lifecycle

White must have been earned before takeoff. A five-bar payment through Self Set or takeoff, a perfect takeoff and valid contact qualify. Serve, Self Set, ultimate availability and near-apex contact are not extra requirements.

Rally holds the actual contact position and height for 2.5 seconds. Early contact does not snap to the apex. The strongest applicable Rally reduction includes 50% charge protection. Damage alone does not cancel; stun, airborne or silence does.

Aim may change for 60 ticks. At tick 60 it locks the last accepted point, then remains fixed for 90 ticks. Provisional public line/circle warnings update while aiming; final warnings freeze at lock. Radius and arrival predictions use the final charged speed, including future Overpressure.

Aim can be anywhere within the current map/realm, including behind Rally or outside an active Serve V. Endpoint clipping follows the committed ray. The released ball pierces units and walls and uses one victim ledger across line and endpoint. It applies ×8 damage instead of the ordinary height multiplier, with Receive force on the first champion only.

The endpoint retains a 120000-radius crater, one 0.5-second knock-up and four seconds of 30% slow. At 300+ km/h, Overpressure enlarges the warned radius by 20% without another damage/control packet. Released spikes return automatically. Realm changes and death cancel owned unresolved work without cleanup damage.

Impossible warnings are public attack telegraphs rather than vision grants. Native danger guidance is not gated on caster visibility and still filters realm membership. Provisional updates do not restart the Lab's reaction delay. Actual native warning rendering and enemy avoidance remain live-acceptance items, with existing movement-assist defaults preserved.

Serve retains its ordinary 75-second cooldown, fixed V, long range, wall penetration and +70% height. Its higher actual contact increases Height Power; it has no additional damage multiplier. Reveal requests, reveal promises and the Lab's simulated V vision were removed.

## Lab, presentation and compatibility

The main toolbar provides Self Set + Serve and Spike-only demo routes. Both use real shared-core commands and disclose their White/Pace practice presets. `?demo=impossible&route=spike` starts the simpler route automatically.

A continuous overhead resource bar replaces the numeric Pace label. The current role stays above Rally. Adjacent height/km/h badges track the actual contact and changing charge speed; exact resource and Power values remain in the inspector. Only costs 1/3/5 are offered.

The clickable full-map panel sets aim without attacking. It shows allies, legitimately visible enemies, warnings and the endpoint. Serve no longer exposes concealed enemies. Old recordings remain inspectable and require their own rules for replay.

Existing body/effect sources remain unchanged. Code-generated continuous bars and readable native role labels are packed as previews; existing White effects supply the staged charge. Every retained frame/anchor/timing is checked against the frozen source. Total preview allocation remains 31.5859375 MiB, with atlases no larger than 2048 pixels. Artwork approval and native high-altitude/continuous-warning visual acceptance remain pending.

## Verification and release boundaries

[Verification/hashes](rally-results/rally-3.1/verification.json) · [browser/parity](rally-results/rally-3.1/browser.json) · [scenarios/timings](rally-results/rally-3.1/scenarios.json) · [actual Lab animation](rally-results/rally-3.1/impossible-spike-lab.gif).

Coverage includes costs and play locks, expiry/exhaustion, bank caps, both qualification routes, early contacts, immutable charge calculations, global lock/warnings, no-Serve wall piercing, victim deduplication, realms, fog, profile independence, recordings and native/WASM parity. Development scenarios and five-run core timings are scripted observations, not engine-attributed damage or matched-role win rates.

No shipped champion/roster/version, installed mod, career or packaged DLL changes in this draft update. Release packaging/Mod Manager verification, installation, backed-up migration, native receipt calibration, native rendering and actual-game balance/acceptance remain separate.
