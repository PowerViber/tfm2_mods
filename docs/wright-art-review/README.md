# Wright — approved engineer and fleet artwork

Worktree branch: `codex/wright-art`. The original art pass used PR #72 head `2635f714c89d761df822e174ffad7c3984799bef`; wright-art-2 used updated kit head `57a75b8f8bf89d042acee94352cc98fdfb2759d2`. Current **wright-3 / wright-art-3** starts at `1f96d4805670a16f18c018b4001bb66fff715749` and implements Reihan's confirmed three-advancing-explosion correction.

On November 10, 2026, Reihan approved **“Approve Wright’s body”**, **“Approve the fleet and carrier”** and **“Approve these effects.”** `approval.json` records their exact scope and frozen source hashes. The direction is a navy/orange modern workshop engineer, original gunmetal/navy aircraft and teal repair markings. There is no mastery appearance. The later wright-2 review raises fighter HP to **15%**; this follow-up preserves it. The initial art pass held 10%, and the separate 20% proposal was not adopted. The drone provides no extra vision.

## Complete presentation

- Nine connected whole-body animations, eight drawings each: **72 right-facing frames and their mirrors**. Visible real roster idle; constant scale and feet 20 pixels below the world centre.
- Initial proposals use 48×56. The packaged frames have eight transparent pixels of horizontal padding on each side (**64×56**); visible body size, vertical origin and ground anchor stay unchanged.
- Idle 72 ticks; walk 48; attack 22 with its firing pose at tick 12; ordinary casts 18; carrier call 24; hit eight; death 48. No new idle/basic Animation CC.
- Fighter, bomber, helicopter, repair plane, drone and rocket in eight headings. Carrier body, falling-carrier/shadow warning, non-damaging downdraft, shockwave, machine-gun feedback, lock-on, bombs/crash/burn, repairs/shields, taunt and three-airframe indicator.
- **113** effect/buff/projectile bindings. Renewed aircraft/carrier tags are single-frame, one-tick effects. Boost and banking variants reuse approved fighter/rocket materials and a stable birth-tick clock. Native and Lab select the same variant, including when planes overlap. The old aircraft aliases remain available. Lock-on follows its target through an SDK entity target.
- Bomber: **three large explosions advance along a fixed line**, nominally 40,000 apart, each with **45,000 radius**. Aim selects the middle/direction; the last centre remains within 110,000. Map edges shorten the line without bending it. Three markers warn their own 60/78/96-tick impacts (18/36/54 for ready bombers). Each strike has one 14-tick explosion; the small-blast composite is superseded. Damage, spacing in time, slow and final stun remain unchanged. Legacy warning and compact 14/16/18-tick rocket/crash aliases remain.
- Carrier shadow/drop and knockback ring cover **100,000 radius**. The one-shot descent warning follows Wright while its existing clock continues; it is not restarted each tick.
- Body atlas **512×1008**, FX atlas **1920×1280**. Total decoded storage **11,894,784 bytes (11.34 MiB)**, below 32 MiB. No atlas exceeds 2048. The approved body sheet and timing are unchanged.

Local review: http://localhost:7300/editor/wright-review.html. Playable Lab: http://localhost:7300/editor/wright.html. The editor server also serves `/wright-review.html` and `/wright.html`; its atlas aliases are verified against packaged PNG hashes. The review inspects the real packaged assets, every pose/frame/facing and aircraft heading. GIFs and `complete-poses.png` show the complete set.

## Frozen sources and reproduction

The original PNGs were produced through the built-in imagegen tool. Every source and prompt is retained here, including the nine eight-frame animation sheets. Generation itself is not deterministic. Source selection was approved before completing the animations; those approvals cover animation completion without another appearance question.

Extraction, nearest-neighbour scaling, mirroring, ground registration, effects geometry and atlas packing are deterministic. Each animation sheet uses its neutral first frame as the scale reference; later wide or falling poses are not independently shrunk. The approved original roster idle is preserved exactly.

```powershell
rtk proxy python -X utf8 tools/prepare-wright-proposals.py
rtk proxy python -X utf8 tools/build-wright-art.py --integrate
rtk proxy python -X utf8 tools/generate-wright.py --check
rtk proxy python -X utf8 tools/verify-wright-art.py
```

Integration refuses missing approvals or mismatched approved source hashes. `art-manifest.json` is the presentation source for native bindings, generated Rust clocks and Lab atlas selection. Gameplay rules and transports remain separate.

## Lab and verification

The Lab now runs at 60 simulation ticks/sec independently of 30/60/120/144/240 Hz displays. Pause, step, background reset and held-input cleanup are covered. The separate incoming-fire toggle and low-fighter-HP button are inspection fixtures; native plane HP and resource rules remain unchanged. Seven preview scenarios use actual core orders, including the real 14-second scout timer.

Reports:

- Current full serial native suite: **1,105 passed, 13 ignored**; **32 Wright tests**. The Windows GNU test build reports four existing native-bridge detour-test-helper warnings. **24 Lab tests** pass (23 unit/transport plus independent-owner/session/replay isolation).

- `verification.json`: **1,088** asset/source/anchor/timing/binding/budget and reproducible-packing checks. Canonical verification permits only the intended bomber spacing/revision change and removal of obsolete decorative bomb count. Damage, costs, cooldowns and approved body pixels/timing are preserved. Gameplay input shape and Lab transport schema remain unchanged; bomber state adds its accepted centres. Old recordings require their original rules revision.
- `browser-verification.json`: **63** checks and **2,010** complete current native/WASM state-and-command comparisons; artwork-on/off host/combat traces agree. A separate 2,010-frame non-bomber fixture matches the pre-change wright-2 state and nonvisual commands (excluding only revision metadata and the additive empty strike field). Bomber outcomes intentionally differ with its new geometry. Manual mechanics agree across all-20 and all-80 thirteen-stat profiles. Five cold browser loads and five sustained 16-plane canvas scenes are recorded separately from host FPS. `bomber-line.gif` is actual shared-core playback, with three isolated targets taking one hit each and the stun only at the third footprint.
- The updated gameplay browser suite passes **12** checks. `routes.json` adds **five** actual editor-server checks.
- `mod-manager.json`: disposable installation fixture and isolated AppData; **871** files compared with no differences. Champion-load validation passes. These do not prove native DLL registration or live rendering.
- `magnus-performance.json`: five unchanged 128-piece core p95 runs, **937 / 909 / 810 / 1026 / 872 µs**, median **909 µs**, all below 4000 µs. No host/crowded-game FPS claim is made.
- Earlier evidence remains in `validation-summary-wright-art-1.json`, `validation-summary-wright-art-2.json` and Git history; it is not relabelled as evidence for the revised kit.

Build the parity runner with `rtk cargo +stable-x86_64-pc-windows-gnu build --release --bin parity` in `native/wright_lab`, serve the repository on port 7300, then run `tools/test-wright-art.py`. Current native/WASM parity includes the new bomber geometry. Historical comparison is limited to a fixture without bomber casts, rather than claiming unchanged damage footprints. Use the GNU toolchain on this PC; the default MSVC linker is unavailable.

The earlier art pass already supplied two narrow native adapter corrections: a known rejected unit spawn is removed immediately, without refund, rather than surviving the receipt grace as an invisible attacker; dead-owner updates clear owned real carriers/presentation state while preserving remaining cooldowns. Successful combat mechanics and numerical rules are unchanged. Mock-host evidence is separate from actual-game callback/render observations.

## Release and live gates

**No release version bump, new packaged DLL, installation or career migration was performed.** The existing packaged DLL remains unchanged from the PR baseline, SHA-256 `97ce14369d999da605a80dbe0618fec3bf192c9595cf606564ae25a00565a628`; it is not a Wright release. Keep PR #72 draft until release packaging and required host acceptance are completed.

Still pending in the Windows game: roster visibility, carrier-unit appearance/targetability and HP markers, actual turret targeting, target-following reticles, shadow/drop alignment, owner-death callback cleanup, crowded combat and actual host performance. The carrier pool assumes post-reduction on_damaged receipts; calibrating that callback stage remains a required live gate. The Lab models host movement and enemy fire; shield, burn and taunt indicators are visual receipts and do not establish native mitigation or taunt execution. No additional drone vision is claimed.
