# Reference games

Each `*.json` file here describes one real, well-documented match (a reviewed pro game, a VOD with a
caster breakdown) as timestamped macro events. `tools/evaluate-game-sense.py` measures the same events
in the athlete-stats rollout and reports which rating level plays most like each side.

Copy `TEMPLATE.json`, fill it in from the video or a match-history site, and name it after the match.
Files whose name starts with `TEMPLATE` are ignored.

| Field | Meaning |
|---|---|
| `title`, `source.url`, `source.notes` | What the game is and where the timestamps come from. |
| `durationS` | Game length in seconds. |
| `stronger` | The side with the better macro, by the review (`blue` or `red`). |
| `events[]` | `{ "t": seconds, "type": ..., "side": "blue" or "red", ... }` |

Event types and their extra fields:
- `objective`: `kind` (`epic`, `serpent`, `dragon`, `baron`, `herald`), `contested` (bool).
- `tower`: `lane`.
- `teamfight`: `numbers` (`[blue, red]` at the start), `winner`.
- `roam`: a champion leaves its lane for another lane or an objective. `afterWave` is true when it left right after pushing its wave.
- `death`: `unseen` is true when the killer was not visible to the dying side just before.

The evaluator turns each side into the same metrics the rollout reports: objective share, fights
started with more players, roams per 10 minutes, and the share of deaths to unseen enemies.
