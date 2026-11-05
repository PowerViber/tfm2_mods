# Rally 3.5 rules (round 152, native 0.10.87)

Approved by Reihan on 2026-11-05. Damage values are the round-153 retune (native 0.10.88), set after the 1.2 s jump; round 152 had spike 42, Thunder 130, Black 160, White 190. `rules/rally.json` keeps the revision name `rally-3.3`, because the Rally Lab's art paths and recordings use it. The rule values below are what changed.

| Rule | 3.4 | 3.5 |
|---|---|---|
| `IMPOSSIBLE_MAX_HP_PERCENT` | 70 (centre packet cap) | removed: uncapped |
| `IMPOSSIBLE_MULTIPLIER` | 800 | 600 (uncapped) |
| `SPIKE_PERCENT` | 50 | 65 |
| `THUNDER_MULTIPLIER` (new) | 180 + height bonus, up to 230 | 155 + height bonus, at most `BLACK_MULTIPLIER - 10` (185) |
| `BLACK_MULTIPLIER` | 180 | 195 |
| `WHITE_MULTIPLIER` | 220 | 235 |
| `FLIGHT_TICKS` | 48 (0.8 s) | 72 (1.2 s) |
| `TOSS_TICKS` | 48 | 72 (matches the flight, so the ball meets her hand at the top) |
| `BALL_GIVE_UP_TICKS` (new) | none | 90: a grounded ball she makes no progress toward returns to her hand |

## Behaviour
- **Flight:** the ball remembers its intended target. A spike or volley passes other units, and is stopped by that target or by any enemy champion in the path.
- **Grounded ball:** it rests where her body can stand. She walks around walls, keeping to one side.
- **Contact:** the AI makes contact at the top of the jump, ±7 ticks for an imperfect athlete. The AI prefers champions in range over minions and camps.
