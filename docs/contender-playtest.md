# Contender playtest candidate

This continues merged PRs #54 and #55 from `6688f74`. The user explicitly asked to finish a playable character before their first actual-game test, and approved both the original boxer and navy/red/blue ring. The previous pre-integration gate is therefore superseded for this **draft playtest**, not for stable release acceptance.

## Included

- A visible original default body, mirrored poses and idle/run/punch/cast/hit/KO animations. Frozen generation sources, approval and deterministic extraction hashes are in `contender-art-review/`.
- One shared Rust boxing core, native adapter, current-stat Mod Power, stamina, Combination/Body Shot, Uppercut, Slip, Guard and a target-bound Counterpunch in the primary physical attack packet.
- One accepted technique at a time; engine-owned ordinary attack scheduling; native skill shells do not hold or inject casts. Preparation, interruption, cooldown/resource spending and rejection reasons are explicit.
- A bounded planner using the thirteen pinned attributes, seeded-independent decision hashes, at most 64 evaluated branches, beam eight and up to three actually applied forecast actions. All techniques remain available to every athlete. Farming uses the existing role-aware context.
- An experimental real-entity transfer bridge: reserve participants, record integer positions at transfer, persist paired ledgers, validate both transfers/readback, roll back partial failures and return survivors exactly after timeout/death. Separate rings use owner-specific coordinates. No frozen-opponent substitute or separate native duel simulation.
- Shared ring geometry and five-tick presentation renewals; radius-aware post-tick rope clamping. Short warning effects use the same cast clock and shared danger-zone input.
- Steve rejects constructions overlapping reserved return areas; complete Boat Wall paths are checked at acceptance and commitment. New reservations refund a blocked wall's charge/cooldown. Empty reservation inputs preserve existing behavior.
- Ordinary cooldowns accumulate fractional recovery at 4/3 speed only for elapsed active-ring time; Main Event is excluded.
- Boxing Lab v1, editor/Skill Test links, keyboard controls, profile controls, real dummy stun suppression, state/rejection inspection, recorded frames and native/WASM parity.

## Important incomplete host work

These are release limitations, not successful checks:

1. The checked SDK has no universal pre-impact veto for every stock/global effect. Contender's own damage/target snapshots enforce membership, but that does **not** establish isolation for every other controller or stock global heal, shield, control, aura or projectile.
2. Arena-born summon membership/return and complete projectile/field cleanup are not implemented across the roster. Existing summons stay physically outside. Do not claim their interactions are isolated.
3. Native off-map navigation, visibility, ordinary stock casts and champion-follow/replay rendering require the actual Windows host. Position readback proves a transfer call only. Rope enforcement currently corrects positions after a tick; it is not a universal pre-movement interception.
4. Construction reservations cover the active Steve native routes, not every possible stock/legacy solid builder.
5. Slip requests banishment, full reduction and CC immunity. The API documents banishment as invisible/input-locked; fixed damage, DoT, existing debuff clocks and exact transition behavior need host proof. The Lab implements the intended rules directly and cannot certify the host.
6. The damage callback must still be observed for fully shield-absorbed hits before Guard counter attribution is accepted in-game.

These limitations keep `check-contender-host.py --require-release` failing with exit 3. The ordinary checker validates a playable candidate and reports every host gate as unproven. This is **not ready to merge as a certified private-realm implementation**.

## Try the Lab

Run `node editor/server.js --port 7291 --no-open` from this checkout, then open `/boxing.html` or `/editor/boxing.html`. The editor toolbar has **Boxing Lab (playtest)** and selecting Contender in Skill Test opens it.

Focus the arena first. WASD moves, Space punches, Q/Shift+Q use Combination/Body Shot, E/Shift+E use Slip/Guard, C uses Uppercut, R uses Main Event. Click an enemy to select it; mouse position aims Slip. Buttons, Pause, stepping and recording/replay remain available.

## Actual-game acceptance still needed

Keep the game closed while installing this branch with Mod Manager; publication does not install it. Use a disposable test career/match before migrating real saves. No real career migration is supplied or applied in this change.

1. Confirm Contender's portrait/body, facings, punches, Guard, stamina and damage attribution.
2. Unlock the normal ultimate. Test transfers against one stock melee and one stock caster, and confirm real movement/attacks/casts in off-map space.
3. Select each participant with normal champion-follow; switch to an outside champion and back. Check replay seek, timeout and death returns.
4. Exercise walls, forced movement, teleports, concealed targets, failed transfers and two simultaneous Contenders. Record exact before/after positions.
5. Check global damage/support/auras, existing and newly summoned entities, and pending projectiles across the boundary. Any leakage fails isolation acceptance.
6. Attempt straight/curved solid construction over each return point and test full shield absorption during Guard.
7. Test Slip against physical, magic, fixed and damage-over-time impacts plus new control on takeoff/exit ticks. Existing debuffs must keep their clocks.

Report logs/screenshots and the exact native version. A failed host gate requires a real implementation fix or missing-host-support report; it does not justify quietly substituting another ultimate.

## Verification records

See `contender-results/` for the browser checks, 43-frame native/WASM comparison, 24 development/12 holdout development-simulator cases, thirteen-stat pairs, five-run core timing, art hashes, package reproduction and isolated Mod Manager results. Synthetic enemy movement/damage in the development simulator is explicitly not a game observation or balance certification.

The initial debug sweep exposed overflow in Unified's unmodified planner at `unified_core.rs:2089`; the normalized source matches the implementation baseline. Release-suite results are recorded separately. No unrelated Unified mechanics were changed to make this pass.
