# Rally 3.2 — Impossible storm and impact frames

This is the completed first 3.2 checkpoint, before the subsequent request for a larger two-second impact sequence and enemy-centred lightning. The earlier 2.1/2.2/3.0/3.1 evidence remains unchanged. Lab transport and athlete profiles stay v2. New artwork is an unapproved proposal, not a shipped asset.

## Combat and aiming

Every ordinary Serve can aim in any direction within 560,000 units. Reachable-area outlines and a selected landing circle replace its V. Impossible needs White earned before takeoff, one five-bar play, perfect takeoff and valid contact. Non-Serve contact must additionally reach 10 m and is limited to 400,000 range with an 80,000 base radius. Serve Impossible reaches anywhere within its current map/realm with a 120,000 base radius. Neither route reveals hidden enemies.

Charge for 300 ticks: 240 adjustable ticks, then 60 locked ticks. Hold the actual contact position/height, retain strongest-applicable 50% protection, and cancel on enemy stun/airborne/silence. Power grows by `floor(200 × elapsed / 300)` from immutable contact snapshots. Speed is half the calculated Impossible velocity, capped at 600 km/h; ordinary velocity is unchanged. The 18.02 m, 104 Attack, no-bank example builds 470→670 Power, 231.3→329.7 displayed km/h and 10,398 raw centre damage. Speed displays use the shared downward tenth rounding.

Impossible travels harmlessly through units/walls. Its line is a guide, never a damaging or dodge hazard. At the endpoint, eligible victims receive one physical packet:

`floor(centreDamage × (100R − 75 × clamp(distance, 0, R)) / (100R))`

This yields 100%, 62.5% and 25% at the centre, halfway and edge. Footprint-overlap eligibility is preserved. Receive force is added afterward once to the nearest eligible champion, breaking ties by entity ID. Knock-up is still 0.5 seconds across the eligible footprint; crater slow is 30% for four seconds. Overpressure at 300+ km/h enlarges the warned/actual radius 20%. Ball return, 1.5-second exhaustion and resource commitments remain intact.

Non-Impossible Black-tier craters, including Black Serve, now slow 20% for two seconds; damage/radius/knock-up are unchanged. White and Impossible retain their stronger crater settings. Each crater stores its own slow and expiry.

## Storm and presentation

The first checkpoint scatters two fixed, warned patches every 30 ticks within the selected landing circle. Radius is 12,000, warning is 24 ticks and active lifetime is 36 ticks with 15% slow. They operate throughout charging/flight, add no damage/knock-up/Pace and never stack. Owner-scoped counters and decision hashing generate locations without combat RNG or hidden positions. Placement is bounded and legal within the current terrain/realm, with at most six simultaneous patches. Interruption, death, realm changes and impact remove remaining fields/warnings. Native owner ledgers preserve the strongest overlapping Rally slow.

The original effect proposals use three distinct impact drawings, two ticks each, at release and landing. Event identity, phase and start tick prevent renewal restarts. There is no hit-stop, camera shake, full-screen flash or added control. The next requested visual revision supersedes this 0.1-second duration; it is preserved here as checkpoint evidence.

References were inspected visually: [Ian Zhang around 15.2–15.8 seconds](https://www.youtube.com/watch?v=CtO_XE2Nrho&t=15s) and [ZiShen44’s construction/finalization/polish sections](https://www.youtube.com/watch?v=jJMDwMRycd4&t=139s). The guide had no transcript. Original drawings were produced with built-in imagegen, then frozen and mechanically extracted. AI generation is not claimed deterministic. [Source hashes and extraction](rally-results/rally-3.2/effect-source-manifest.json) record the distinction.

Pace is one overhead fill divided into three equal sections, without role text/icons or numeric Pace. Roles and exact values remain in the inspector. Height and changing km/h stay adjacent. The Lab has both global Serve and local five-bar Self Set demos, full-map aiming, optional presentation and light/dark review controls.

Lossless packing retains all 1,813 frozen main-atlas frames and White-frame pixels/timings. The preview allocation including the new effect atlas is **31.8205 MiB**, with every atlas at most 2048 pixels. No source body pixels changed.

## Automated checkpoint

- **948 native tests passed**, 11 explicitly ignored, with four existing helper warnings. The first parallel run exposed `tick_budget_tests::raze_frames_match`; that test passed in isolation and the complete serialized run passed. A cause was not established; no Raze code was changed.
- **105 Lab tests** and **20 browser checks** passed. The complete **600-frame native/WASM trace**, recorded replay and presentation on/off outputs match.
- Ten routes passed through the actual editor server using an inert isolated game fixture; no real career was accessed.
- 24 development and 12 holdout scripted scenarios ran with simplified mitigation and simulated receipts. They are not engine-attributed damage or matched-role win rates.
- Five release runs: median p95 **31.9 μs** for the 128-unit core fixture, **89.8 μs** for an active 128-unit Impossible storm and **888 μs** for the unchanged Magnus 128-piece fixture, below its 4,000 μs target. Host/rendering performance is excluded.
- Independently rebuilt WASM outputs match: `b64c7f02d2c6063128487bd5d6b399317c4ce006684180e5ab9d5b3f1dd0ff76`.

[Automated results](rally-results/rally-3.2/automated-checks.json) · [Browser/parity](rally-results/rally-3.2/browser.json) · [Scenarios](rally-results/rally-3.2/scenarios.json) · [Animation timing](rally-results/rally-3.2/animation-timing.json) · [Actual Lab animation](rally-results/rally-3.2/impossible-spike-lab.gif).

Reproduce with `rtk proxy python tools/prepare-rally-3-presentation.py`, `rtk proxy python tools/prepare-rally-32-effects.py`, `rtk proxy node tools/preview-rally-timing.cjs` (review server on port 7294), and `rtk proxy python tools/check-rally-32.py`. Extraction uses the frozen sources; the check pipeline never installs or migrates anything.

## Acceptance boundaries

Native incoming-force and ally-support receipts remain disabled until calibrated. Native high-altitude rendering, effect alignment, target reactions, shields/items/attribution and live balance have not been observed. No shipped champion/roster/version, installed mod, real career or packaged DLL was changed. Art integration, release packaging/Mod Manager verification, installation and career migration remain separate.
