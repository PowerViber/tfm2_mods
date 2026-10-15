"""Record package facts and verification separately from unobserved game acceptance."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
for name in ["native/tfm2_custom_ai/mod.mod_info","mods/tfm2_custom_ai/mod.mod_info"]:
    p=ROOT/name;d=json.loads(p.read_text(encoding="utf-8"))
    if "Magnus" not in d["description"]:
        d["description"]+=" Magnus, the Grandmaster: seven destructible chess pieces, shared command timing, castling, tactical checkmate, six adapted historical challenges and athlete mastery."
        p.write_text(json.dumps(d,indent=2)+"\n",encoding="utf-8")
package=json.loads((ROOT/"docs/magnus-package.json").read_text())
scenarios=json.loads((ROOT/"docs/magnus-scenarios.json").read_text())
summary=[]
for holdout in [False,True]:
    g=[x for x in scenarios if x["holdout"]==holdout]
    summary.append(f"| {'Holdout' if holdout else 'Development'} | {len(g)} | {sum(x['pieceSurvival'] for x in g)/len(g):.3f} | {sum(x['checkmates'] for x in g)} | {sum(x['challengeCompletions'] for x in g)} | {sum(x['rawDamage'] for x in g)} | {max(x['maxStepUs'] for x in g)} |")
text=f"""# Magnus implementation and verification

Implemented on §codex/magnus-grandmaster§ in the attached isolated worktree. The original checkout's uncommitted files are preserved.

## Implementation

- §tfm2_custom_magnus§, Magician, AP / CC / Range, with the approved fixed baseline and growth.
- Seven real summoned-unit carriers, stable logical identities, square movement, destructible HP, recovery and owner-attributed damage. No untargetable substitute pieces.
- Cardinal 8x8 boards, fixed d1 rook / e1 Magnus / f1 queen, shuffled opposite-colour bishops, terrain and friendly blockers, shared command timing.
- Reset restoration, castling, projections, check/fork/pin/discovery/sacrifice, tactical checkmate, warning cancellation and persistent match cooldowns.
- Six pawn-free adaptations, selected challenge, ordered identities, failure reasons, eight-second attempts and Brilliant Recall.
- Bounded one-to-four-command planning, cached terrain geometry, visible velocity prediction and seeded mistakes. Mastery changes choices rather than numeric power.
- Reproducible body, piece, identity, health, coverage, recovery, warning and reward art, with enemy-role miniatures on the handheld board. Coverage uses persistent layers, at most one per square; transient effects have a 12-token, six-per-tick budget.
- Chess Lab in the editor and Skill Test workspace: manual controls, terrain, fog, moving enemies, captures, interruption, death/respawn, all ranks and six replay fixtures. The lab runs the compiled Rust combat core.

## Clarifications

1. Opera accepts any enemy capture in matches. The stable API does not identify who destroyed a summoned unit. The lab reference and fixture retain the original knight capture.
2. Marathon uses the user's chosen replacement for impossible mutual protection: the rook protects the knight, and the knight covers queen escape squares on the opening and closing rook strikes. The queen moves after the knight's landing strike.
3. Castling exchanges logical squares: the rook takes the centre of Magnus's former square, keeping pieces square-bound.
4. Brilliant Recall preserves total times exactly at 60 ticks/second. Phases round to the nearest tick: ordinary 14 + 10, queen 22 + 14.

## Automated verification

The combined release suite passes 298 tests, with three existing ignored tests. The Mod Manager's 13 tests pass. Browser checks confirm the editor's Chess Lab entry, seven-piece deployment, reference catalogue and a completed King Hunt reward.

Parity covers 4,542 complete frames: internal state, timers, logical identities, projections, event progress, random state, movement, damage, shields, stuns and effects. It includes six actual command trajectories and manual/autonomous cases at every rank. Independent browser instances and two-owner core scenarios are included.

Magnus tests cover seeded rosters, bishops, walls and thin walls, blockers, jumps, recovery, reset, castling commitment, visibility, interruptions, single hits, checkmate edges/cancellation/cooldown, six successes and multiple near misses, invalid sacrifices, duplicate rewards, death and clones. The existing Coder vector test normalises CRLF before comparing; its vectors and combat rules are unchanged.

The harness uses 24 development scenarios and 12 fresh holdout seeds after planner changes were frozen. Detailed results are in §magnus-scenarios.json§.

| Set | Scenarios | Living-piece fraction | Checkmates | Historical rewards | Raw piece damage | Slowest step (microseconds) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(summary)}

These controlled movement/attack fixtures omit the live item system and full danger avoidance. They verify determinism and exercise planning, rather than establishing live balance. A zero historical reward count remains a planner tuning limitation. All six manual fixtures earn their rewards.

## Package

- Native **{package['nativeVersion']}**, advanced from the rebased 0.10.50 baseline.
- Champion pack **{package['championVersion']}**, advanced from the rebased 0.2.51 baseline, requires native >={package['nativeVersion']}.
- Release DLL SHA-256: §{package['dllSha256']}§.
- The packaged DLL matches the release output byte for byte.
- Decoded art: **{package['decodedArtworkBytes']/1024/1024:.2f} MiB**, every atlas <=2048 pixels per side.
- Piece art covers 5,696 checked active carrier-frame / piece-frame / facing combinations.

## Game acceptance — pending

The actual game folder is staged with **one knight only**, via §knight_probe: 1§. Previous game files are backed up under this worktree's §backups/magnus-*§ folders. Normal worktree champion data enables all seven pieces.

Real-match appearance, targeting, HP loss, autonomous-attack suppression and cleanup have not been observed. Native game controls are unavailable in this session. A user-assisted 30-second practice match was requested; §mods/tfm2_custom_ai/magnus_log.txt§ records spawn, HP, position and targetability.

The first milestone remains open until the knight is observed in the game. Then install the full army and check seven-piece readability, scale/anchors, warnings, rewards, items, kill attribution, farming, custom-champion interactions and crowded-fight performance. Old-board visual removal/fading on reset and death needs a match check because point effects use the existing lifetime API.

Mod Manager compares the game probe against the full worktree, so its probe-parameter discrepancy is expected. Its actual result is recorded separately; a probe comparison is not full-build game acceptance.

After incorporating the latest Vesper changes and normalising only structured-data files that were already semantically identical, Mod Manager compares 643 files: zero missing, an identical native DLL, matching versions, and one intentional Magnus probe-parameter difference. The runtime-data comparison confirms no other content or formatting discrepancies.

## Reproduction

Use RTK-prefixed commands. This machine builds the native DLL in GNU release for §x86_64-pc-windows-gnu§. Build §native/magnus_lab§ for §wasm32-unknown-unknown§ and copy its output to §editor/magnus.wasm§. Build its §vectors§ binary for GNU, then run §tools/run-magnus-verification.py§, §tools/verify-magnus.js§, §tools/verify-magnus-art.py§ and §tools/package-magnus.py§.

The catalogue and generator are in §Claude outputs/magnus/§. Start the isolated editor on port 7275 and open §/magnus.html§, or use Chess Lab. After the knight milestone passes and the game is closed, §tools/package-magnus.py --install-full <game folder>§ stages the army with another backup.
""".replace("§",chr(96))
(ROOT/"docs/magnus-verification.md").write_text(text,encoding="utf-8",newline="\n")
print("Magnus metadata and verification notes recorded.")
