"""Record reproducible package facts separately from observed game acceptance."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads((ROOT/p).read_text(encoding='utf-8'))
package=read('docs/magnus-package.json');scenarios=read('docs/magnus-scenarios.json');stress=read('docs/magnus-stress.json')
rows=[]
for holdout in [False,True]:
    g=[x for x in scenarios if x['holdout']==holdout]
    rows.append(f"| {'Holdout' if holdout else 'Development'} | {len(g)} | {sum(x['pieceSurvival'] for x in g)/len(g):.3f} | {sum(x['checkmates'] for x in g)} | {sum(x['challengeCompletions'] for x in g)} | {sum(x['rawDamage'] for x in g)} |")
perf='\n'.join(f"| {r['pieces']} | {r['p95StepUs']/1000:.3f} | {r['p95PlanningUs']/1000:.3f} | {r['maxStepUs']/1000:.3f} |" for r in stress)
text=f'''# Magnus Slim A / persistent army verification

Implementation branch: `codex/magnus-slim-army`, isolated from the original checkout at `af600c8`. Original uncommitted artwork and scratch files remain in that checkout. The canonical generator now owns Slim A's drawing logic.

## Result

- Native **{package['nativeVersion']}**, champion pack **{package['championVersion']}**, dependency **>={package['nativeVersion']}**.
- Persistent owner-scoped registry with stable piece IDs and seven designated board slots. Dynamic version 2 transport runs exactly the same Rust rules in native gameplay and Chess Lab.
- Ultimate deployment, whole-board random placement, adjacent queen, opposite-colour bishops, free Magnus movement and queued yielding. Redeployment restores the queen/captured slots and preserves surviving non-queen HP; skill summons retain their positions.
- No automatic capture recovery. A five-second departure countdown fades the board; surviving pieces keep fighting at field strength. Army dilution and per-hit territory multipliers apply to piece attacks, leaving normal basic attacks unchanged.
- Allied-sight pawn and minor-piece summons, map-wide chess movement, pawn first double advances, diagonal attacks, continuously observed en passant interceptions and combat-qualified promotion preserving identity and HP percentage.
- Repeatable board and field traps, continuous 0.4-second warning validation, terrain/safe-exit cancellation, shared 12-second target immunity, and castling legality checked again before the exchange.
- Twelve selected historical adaptations, bound identities, real damage receipts, enemy captures, different five-second rewards, 20-second shared reward cooldown and 60-second per-reference repeat cooldown. Damage and pawn-tempo charges apply to piece moves; summoning does not consume them. Immortal protection keeps its original expiry through a rebuild.
- Bounded mastery planning: at most 16 tactical pieces, 64 scored command candidates and beam width eight; the considered roster rotates deterministically. Every rank retains the same manual legal actions and numeric power.

## Artwork

![Previous artwork and Slim A](magnus-slim-comparison.png)

[All poses and both facings](magnus-poses.png). All 48×56 canvases, animation names, frame counts and durations match the baseline. Casting overlays use the canonical Slim A poses. Enemy miniatures use the handheld board's new coordinates in both facings; pawns and weakened bases have distinct art.

Identical atlas frames are deduplicated without changing animation metadata. Decoded artwork is **{package['decodedArtworkBytes']/1024/1024:.2f} MiB** across {len(package['atlases'])} atlases, each at most 2048 pixels per side. Generation reproduces all **54 generated files** byte for byte. The piece masking check passes **7,120** carrier-frame / piece-frame / facing combinations.

## Automated checks

- Final native release suite: **337 passed, three pre-existing ignored**. Magnus contributes **65** focused tests.
- Mod Manager tests: **13 passed**.
- **8,428 complete native/WASM parity frames**, including all twelve legal historical replays, every mastery rank, manual and autonomous paths, and independent concurrent sessions. Comparisons include internal state, random state, identities, cooldowns, events and all emitted effects.
- Twelve successful histories and multiple near misses per reference. Coverage includes hidden or lost targets, interruption, missing required events, cleanup, duplicates, ordinary movement, sacrifices, promotion and expiry.
- Dynamic rosters, 96 formation seeds, map occupancy and yielding, smoke/wall sight, permanent captures, rebuilt slots, fading/re-entry, scaling, serialization, real-hit qualification, immunity across owners and independent clones.
- Regression checks reject occupied summon landings, teleports as walking progress, fog/blinks as interception history, castling after leaving the board, and pawn/knight coverage through invalid terrain.

### Browser observations

The standalone Chess Lab loaded version 2 and rendered the new controls. Birthday Queen completed and granted its rook reward. Opera completed, applied board checkmate and displayed 1.25-second stun / 12-second immunity. With moving enemies and attacks enabled, all three enemy positions and attack deadlines remained unchanged during a half-second step inside the stun. After expiry, positions and attack deadlines advanced while immunity continued counting down.

The lab includes map navigation, full-army selection, field coverage from Rust, summon direction and promotion, allied sight / smoke / walls, departure countdown, dead slots, multipliers, 32/64/128-piece setups and all twelve reference replays. Setup-only edits clear challenge progress. UI testing ended when the user stopped computer use.

### Development / holdout scenarios

| Set | Scenarios | Mean living fraction | Checkmates | Historical rewards | Raw damage |
| --- | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(rows)}

The 24 development seeds and 12 holdout seeds are distinct. These controlled movement/attack scenarios omit full host item interactions and rendering. Historical rewards remain absent in these autonomous scenarios: **recognition works in all twelve manual replays; autonomous combination execution needs more tuning**.

### Release-core stress

| Living pieces | p95 step (ms) | p95 planning decision (ms) | Slowest step (ms) |
| ---: | ---: | ---: | ---: |
{perf}

Measurements use the development machine's GNU release core with exact fixed army sizes. Both ordinary steps and planning decisions meet the <4 ms p95 target. They exclude native snapshot collection, engine entities, renderer and other champion hooks. Native-host and crowded-match performance remain unmeasured.

## Package and Mod Manager

DLL SHA-256: `{package['dllSha256']}`. The shipped DLL matches the built release output byte for byte; metadata and dependencies are synchronized.

The real Mod Manager also checks an **isolated full-package installation fixture**, with its result in [magnus-manager-package.json](magnus-manager-package.json). This checks packaging and byte equality without replacing files in the running game.

At actual-game probe staging, Mod Manager compared 643 champion files: zero missing, matching versions, identical DLL and one intentional `carrier_probe` data difference. That observation predates final castling / reward / terrain repairs. It is distinct from the final package check and from gameplay acceptance.

## Actual game acceptance — partial

The game loaded native 0.10.52 in version 0.6.4. A separate fresh campaign with all starting champions enabled offered Magnus with Slim A. A one-knight composition test against Archer completed; the native log recorded a 200-HP summoned carrier, and its distinct knight silhouette appeared in the actual game replay.

**The destructive knight and pawn milestone is still open.** Enemy targeting/HP loss, pawn rendering and movement, suppression of autonomous attacks, cleanup, full-army warnings/rewards, items, kill attribution, farming and crowded fights have not been observed. The user stopped computer use with physical Escape before the second carrier probe could be inspected. No full-army game acceptance is claimed.

The temporary redesign probe install was rolled back after the game closed. Previous installed files and their original Magnus settings were restored byte for byte from `backups/magnus-1792075411869719900`. The PR enables the full army with no probe parameters. The original campaign matches its pre-test backup byte for byte. A separate test campaign, Magnus Acceptance, remains available.

Other host limits: conservative ally sight replaces arbitrary-point fog queries; summons use the supported long-lived carrier convention. The stock champion card still displays inert-shell cooldown fields; actual command and ability cooldowns are in the compact tooltips and Chess Lab. Point effects can persist for their remaining half-second lifetime after a board removal.

## Reproduction

Prefix shell commands with RTK. Build native and vector tools in GNU release for `x86_64-pc-windows-gnu`. Build `native/magnus_lab`'s library for `wasm32-unknown-unknown` and copy it to `editor/magnus.wasm`.

Run `tools/run-magnus-verification.py`, `tools/verify-magnus.js`, `tools/verify-magnus-art.py`, `tools/verify-magnus-generation.py`, `tools/review-magnus-art.py`, `tools/package-magnus.py`, `tools/check-magnus-package-manager.py` and this recorder. Serve the worktree's editor and open `/magnus.html`; Chess Lab is also available from the editor / Skill Test workspace.

The canonical source catalogue and generator are in `Claude outputs/magnus/`. Saavedra and Réti are labelled study adaptations; reference FEN/moves are stored separately from combat steps. Opera accepts any enemy capture, and Marathon uses rook protection of the knight plus knight coverage of the enemy queen's escapes, as approved.
'''
(ROOT/'docs/magnus-verification.md').write_text(text,encoding='utf-8',newline='\n')
print('Recorded Magnus v2 verification and partial game acceptance.')
