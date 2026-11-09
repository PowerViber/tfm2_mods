# Wright — approved engineer and fleet artwork

Worktree branch: `codex/wright-art`, based on PR #72 head `2635f714c89d761df822e174ffad7c3984799bef`.

On November 10, 2026, Reihan approved **“Approve Wright’s body”**, **“Approve the fleet and carrier”** and **“Approve these effects.”** `approval.json` records their exact scope and frozen source hashes. The direction is a navy/orange modern workshop engineer, original gunmetal/navy aircraft and teal repair markings. There is no mastery appearance. Fighter HP remains 10%; 20% was not adopted. The drone provides no extra vision.

## Complete presentation

- Nine connected whole-body animations, eight drawings each: **72 right-facing frames and their mirrors**. Visible real roster idle; constant scale and feet 20 pixels below the world centre.
- Initial proposals use 48×56. The packaged frames have eight transparent pixels of horizontal padding on each side (**64×56**); visible body size, vertical origin and ground anchor stay unchanged.
- Idle 72 ticks; walk 48; attack 22 with its firing pose at tick 12; ordinary casts 18; carrier call 24; hit eight; death 48. No new idle/basic Animation CC.
- Fighter, bomber, helicopter, repair plane, drone and rocket in eight headings. Carrier body, falling-carrier/shadow warning, non-damaging downdraft, shockwave, machine-gun feedback, lock-on, bombs/crash/burn, repairs/shields, taunt and three-airframe indicator.
- **70** effect/buff/projectile bindings. Renewed aircraft/carrier tags are single-frame, one-tick effects. Lock-on follows its target through an SDK entity target. Bomb aliases preserve the three existing 14/16/18-tick visual lifetimes instead of relying on a lifetime argument to stretch animation.
- Body atlas **512×1008**, FX atlas **1920×448**. Total decoded storage **5,505,024 bytes (5.25 MiB)**, below 32 MiB. No atlas exceeds 2048.

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

- Full serial native suite: **1,095 passed, 13 ignored**; **22 Wright tests**. The Windows GNU test build reports four existing native-bridge detour-test-helper warnings. **18 Lab tests** pass (17 unit/transport plus independent-owner/session/replay isolation).

- `verification.json`: **818** asset/source/anchor/timing/binding/budget and reproducible-packing checks. The combat core, canonical rules/data, Lab transport and shipped WASM remain byte-identical to the PR baseline (line-ending-normalised for text).
- `browser-verification.json`: **52** new checks and **2,010** complete native/WASM state-and-command comparisons; artwork-on/off host/combat traces agree. Manual mechanics agree across all-20 and all-80 thirteen-stat profiles. Five cold browser loads and five sustained 16-plane canvas scenes are recorded separately from host FPS.
- The original gameplay browser suite passes **eight** checks. `routes.json` adds **five** actual editor-server checks.
- `mod-manager.json`: disposable installation fixture and isolated AppData; **871** files compared with no differences. Champion-load validation passes. These do not prove native DLL registration or live rendering.
- `magnus-performance.json`: five unchanged 128-piece core p95 runs, **1400 / 949 / 1014 / 940 / 847 µs**, median **949 µs**, all below 4000 µs. No host/crowded-game FPS claim is made.

Build the parity runner with `rtk cargo +stable-x86_64-pc-windows-gnu build --release --bin parity` in `native/wright_lab`, serve the repository on port 7300, then run `tools/test-wright-art.py`. The original WASM stays unchanged. Use the GNU toolchain on this PC; the default MSVC linker is unavailable.

Two narrow native adapter corrections accompany presentation: a known rejected unit spawn is removed immediately, without refund, rather than surviving the receipt grace as an invisible attacker; dead-owner updates clear owned real carriers/presentation state while preserving remaining cooldowns. Successful combat mechanics and numerical rules are unchanged. Mock-host evidence is separate from actual-game callback/render observations.

## Release and live gates

**No release version bump, new packaged DLL, installation or career migration was performed.** The existing packaged DLL remains unchanged from the PR baseline, SHA-256 `97ce14369d999da605a80dbe0618fec3bf192c9595cf606564ae25a00565a628`; it is not a Wright release. Keep PR #72 draft until release packaging and required host acceptance are completed.

Still pending in the Windows game: roster visibility, carrier-unit appearance/targetability and HP markers, target-following reticles, shadow/drop alignment, owner-death callback cleanup, crowded combat and actual host performance. The Lab models host movement and enemy fire; shield, burn and taunt indicators are visual receipts and do not establish native mitigation or taunt execution. No additional drone vision is claimed.
