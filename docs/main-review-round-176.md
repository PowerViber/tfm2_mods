# Round 176: Basalt, the Wallbreaker (native 0.10.110, tfm2_custom 0.2.113, `basalt-1.0`)

**Reihan:** "what if the character can spawn walls and play by or with them, he throws he slams etc". Built from the plan's choices:
- **Role and walls:** a **bruiser**; raise, throw and slam walls; **no cage**; chunks torn off terrain become walls.
- **Skills:** a **Jump**, and **8 skills** (S1 to S3 and the ult, each with a jump version). One is the **V slam** (two walls at the upper left and right, slammed together).
- **Ranges:** buffed 1.5× to 2×.
- **Weapon:** a **whip**.
- **Walls:** 5 walls that fade 3 s after he's 150,000 away. The punch launches a wall. Hurl and Slab Drop are throws (no lines).
- **Whip Swing:** 360° + x with a growing whip, x toward a teammate (a gank) or away from their team.
- **Air rules:** he passes every wall in the air, and walls have 3 integrity (only his hits count).
- **Passives:** Whip Pull and Jump are passives. Whip Pull is a slingshot, faster the farther he is. The combos follow Kicker's moving and targeting.
- **The ult (mid-build):** "he stops for 3 s charging up to throw a massive rock which is global… knocks up and slows; for the air… he punches the big rock and what lands is hot meteors that burn, slow and a little knock-up; both can be recast so he can whip the rocks and land where they landed".

The kit and AI are in `docs/basalt-guide.md`.

## Implementation
- **Data:** `rules/basalt.json` feeds `tools/generate-basalt.py`. It generates `basalt_data.rs`, `tfm2_custom_basalt.data_champion` (Melee, AD / Melee / CC), the tooltips, `editor/basalt-data.js` and the roster (88 champions).
  - **Kinds:** his skills aren't tagged `Construct`, so he doesn't join the map-plan "map changers" (team-composition plans are unchanged).
- **Core (`basalt_core.rs`):**
  - **Walls:** with integrity, the keep range and fade, slides, and clap rotation.
  - **Moves and slams:** pushes with wall and terrain stops (slams scheduled on arrival); airborne chains (charges, the double jump, wall kicks); the slingshot.
  - **The eight skills,** plus the ult's charge, boulder, meteors, burns, slows and recast.
  - **AI:** Kicker's flat priority list.
- **Bridge (`basalt.rs`):**
  - **Walls:** they're the shared `sbw:` walls (renewed or rewritten each move; `note_map_buff`), so everyone collides with them.
  - **Airborne:** `bs_air` (CC immune, `ignore_wall`) plus `place::MOVED`, so the shared wall code lets him over.
  - **Effects:** glides only; slows `bs_slow`; the charge is a self Bind.
- **Wiring:** `lib.rs` (the passive, Mod Power `[15, 0, 5, 10, 10, 0, 5]`), `steve.rs` (the `bs_go` steer), the bench and the roster count.
- **Lab:** `native/basalt_lab` builds `editor/basalt.wasm`; the page is `editor/basalt.html`, registered in `editor/server.js`.
- **Art:** placeholders, with the request in `docs/basalt-art-handoff.md`.

## Verification
- **Native tests:** `cargo test --release` gives **1,337 passed**, 0 warnings, including **25 Basalt** tests:
  - **Walls:** the 5-wall cap and rubble; the fade (and its cancel); integrity (only his hits, 3 shatter it); the 2 s slam-stun guard.
  - **Jumps:** the double jump, the wall kick (long, restores the double jump, once per wall); landing out of terrain.
  - **Slingshot:** faster from farther, overshoots the wall, fly-through knocks.
  - **The ground and air skills:**
    - Raise & Punch: the launched wall slams a pinned enemy;
    - Air Dash keeps the chain open;
    - Hurl lands at a point and Slab Drop knocks up an area;
    - Crushing Clap: the V, the crush, outside untouched, fused onto the centre line;
    - Whip Swing: 360° + x toward a teammate (or away from their team), and the raised wall;
    - the shared pair cooldowns.
  - **Ult:**
    - Boulder Throw: the 3 s charge with no casting, the global landing, the knock-up and slow, the standing rock, and the recast landing at the rock;
    - Meteor Punch: hangs airborne, then meteors with the knock-up, slow and burn ticks.
  - **AI:**
    - the clap on a group, jumping in to Whip Swing, slingshotting to help, and farming with Hurl;
    - no 1v4;
    - the boulder at a far teamfight, and never a charge with an enemy close.
  - **Glue:** `sbw:` walls appear, move and go on the mock host, and `steve::walls_up` sees them; the airborne buffs; never towers; a clean 120-tick run.
- **Lab:** `basalt_lab` passes (3 tests). A headless run of `editor/basalt.html` showed the AI jumping in, Whip Swing, and a slam into a raised wall, with no page errors.
- **Bench:** about **5.2 µs** per tick on the mock host.
- **Other checks:** the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports **0.10.110**.
- **Not checked here** (it needs the real game):
  - how the engine's pathing reacts to his moving walls;
  - Whip Swing's circular glide legs on screen;
  - the long-range boulder flight timing.

## Install (Reihan)
1. Merge the PR and pull.
2. Run the Mod Manager and press **Install**.

Basalt is in exhibition matches and new careers.

## basalt-1.1: square walls (Reihan)
**Reihan:** "the wall isn't a rectangle wall, it's a square wall like Grimhald's wall but bigger in size".
- **Shape:** every wall of his is now a **square block, 32,000 a side** (`wall.size`), instead of a 70,000-long line. That's twice the shared walls' 16,000 thickness, about one terrain cell.
- **In the world:** each square is laid down as **two parallel shared `sbw:` walls**, 16,000 apart, which together fill it. Everyone still collides with it, and it's drawn as a 2×2 cluster of blocks.
- **In the core:** squares are axis-aligned. Collisions, the slingshot and wall kicks measure to the square's surface. The Clap's two squares ride the V's arms to the centre line and fuse. The punch carries enemies within the square's width.
- **Checks:** 1,338 native tests pass, including 26 Basalt tests: a new square-shape test, and a glue test that the two shared walls fill the whole square. The wasm and the DLL (0.10.110) are rebuilt.

## Approved original artwork and `basalt-1.2` — 16 November 2026

Reihan approved the red/yellow body, requested a smaller approximately 40-pixel
silhouette and smaller boulder, and then explicitly approved the corrected
effects. The V-clap blocks now start at the maximum-range corners (110,000)
and merge at the far centre. Damage, hit region, cooldowns and the 18-tick
closure are unchanged.

Body and effects are integrated. Native/WASM share a read-only presentation
projection for accepted blocks, tosses, charge/flight, meteors, impacts and
live-endpoint whip tethers. The Lab uses the real map and stock actors with
labelled host simulation. Full evidence and native pose/lease/direction limits
are in `docs/basalt-art-review/README.md`.

Current checks: **1,342 native tests**, 18 ignored, **7 Lab tests**, **13 browser
checks**, **36 exact native/WASM traces** (10,800 ticks), **15 Mod Manager tests**
and a **900-file** disposable Mod Manager comparison pass. Four existing unused
native-bridge test-helper warnings remain. Body/effects total **24 MiB** decoded.
Two independent builds reproduce packaged native **0.10.110**; champion pack
**0.2.113** remains reserved by this PR. SHA-256 and reproduction details are in
`docs/basalt-art-review/package.json`.

Actual-game visual/pathfinding/balance acceptance is pending. No installation
or career migration was performed.

## basalt-1.3: a 0.5 s ult charge (Reihan)
**Reihan:** "make the ult 0.5, but keep the boulder travel time".
- **Charge:** Boulder Throw and Meteor Punch now charge for **0.5 s** (`ult.chargeTicks` 180 → 30). The rock's flight is unchanged (`rockSpeed` 8,000 per tick, at least 0.5 s), and so are the meteor timings and the recast.
- **AI:** it still charges only with no enemy champion within 80,000 of him (a far-fight tool).
- **Checks:** 1,344 native tests pass; `basalt_lab` passes; the wasm and the DLL (0.10.110) are rebuilt; the Mod Manager tests (15) pass.
- **Not re-run here:** Codex's art evidence files (`docs/basalt-art-review/*`, `editor/basalt-art.js`) still record `basalt-1.2` as metadata. They don't gate anything, and the charge pose and warning follow the core's charge state.
