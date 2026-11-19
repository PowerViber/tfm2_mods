# Contender 3.0 Shadow Link review

Based on merged 2.9 at `5e56bfc67c4919d145b0c8b640be6b11d1974813`. The review
branch includes the playable Lab, safer Contender AI, and a gated SDK carrier
prototype. The installed game package still uses the 2.9 DLL.

Open [the real-map Lab](http://localhost:7310/editor/contender.html?demo=shadow-link).
The fixture includes a stock Knight tank, stock Ninja assassin, Rhino creep and
one optional simulated ally. Press **C** to send the shadow. Aim at the shadow
and use **Space** to hit it. Tick the ally box to see the full four-second bank
resolve without issuing extra commands. The HUD shows the raw payout and the
observed HP and shield losses in the Lab host. The shadow uses the approved
Contender sprite and a thin arrival ring. A link is represented by a small icon.

## Reproduce the checks

- Generate the data with `rtk proxy python tools/generate-contender.py` and
  `rtk proxy python tools/generate-roster.py`. The target catalogue is generated
  with `rtk proxy python tools/generate-lab-targets.py`.
- Build `native/contender_shadow_lab` with the Windows GNU toolchain in release,
  including its `trace` binary. Build its library for `wasm32-unknown-unknown`
  and copy the resulting `contender_shadow_lab.wasm` to `editor/contender.wasm`.
- Run `rtk proxy node tools/check-contender-shadow-link.cjs` for 2.9 comparison and
  3.0 parity. The baseline WASM comes directly from the pinned Git revision.
- With the local, ignored game-reference assets available, start
  `rtk proxy node tools/serve-contender-shadow-review.cjs 7310`. Run
  `rtk proxy node tools/test-contender-shadow-link.cjs` for the browser fixture.

The fixed-position comparison casts S3 and then one Cyclone. It feeds the 3.0
carrier a deterministic training damage schedule. At 161 Attack, Cyclone's own
raw damage was identical for one, three and five enemies. Across the five-target
trial, queued visuals fell from **110 to 27**, and peak queued work in one tick
fell from **21 to 6**. The new delayed hit reached the 600 raw cap on each
2,000-HP target. That payout is team-assisted, and this fixed-position check
does not establish competitive balance.

Native/WASM outputs match through **1,590 frames** across one, three and five
enemies and Aggressive 20/80. Both old unsafe decisions were reproduced in 2.9
and blocked in 3.0. The supported Flurry fixture still commits at both profiles.
The real-map browser registers all 900 cells. The training ally fed 1,610 damage
into the shadow. Payout requests were 900/360/195 raw against the tank,
assassin and creep, and the map host delivered 428/206/166 HP damage.

## Automated test results

Lab WASM SHA-256:
`a61ea4ef9816215d95b63a2d7da355b44dac308ce8919972f2dc9fd3315fc770`.

- Windows release suite: 1,421 passed, 18 ignored.
- Shadow Lab transport: 3 passed. Mod Manager: 15 passed.
- The comparison and real-map fixture passed, including inspection of a 2.9
  recording. Replay of a recording requires its own rules revision.

## Live release gate

The `contender-shadow-link-probe` Cargo feature is for a disposable native build.
The default DLL build leaves Shadow Link disabled. The prototype uses one
opposing-team SDK summon and samples its HP once per active tick. It depends on
the host honouring targetability, immunity and silent cleanup. It confirms that
the slow buff exists before linking. The owner may not start a probe inside a
cage or realm because the existing host cage system ejects summons. This needs
live coverage before a release can claim arena compatibility.

In a disposable match, verify that stock friendly basics and eligible skills
reduce the carrier's HP, that unrelated sources cannot feed the bank, and that
the final HP loss is readable before the host removes the carrier. Observe the
kill feed, gold and experience at depletion and expiry. Cover stationary CC
immunity, original-target combat, visibility, link breaks, multiple Contenders,
and owner death. Measure native frame-time spikes with the same one-, three- and
five-target fights and compare without a cast. A missing terminal entity gives
no fabricated HP receipt. Game timing, attribution, artwork, packaging and
actual-game acceptance are still pending.
