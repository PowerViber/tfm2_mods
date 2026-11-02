# Rally 3.4 changes (round 150)

## Rules

- **White Storm and the local (non-Serve) Impossible need a 9.5 m contact, down from 10 m** (`WHITE_MIN_HEIGHT` and
  `IMPOSSIBLE_MIN_HEIGHT` 10,000 → 9,500). Approved by Reihan 2026-11-02.
  - Why: without a Serve, Rally's highest contact in Black form was 9.6 m (perfect 4.8 m + Black role jump 1.5 m + Set
    0.3 m + five Burst bars × 0.6 m). Under the 10 m rule White Storm needed a perfect Serve, so the athlete AI almost
    never reached White or Impossible. At 9.5 m a perfect five-bar Self Set in Black reaches White, and in White
    (role jump 2.5 m) a five-bar Self Set reaches a local Impossible even from a good takeoff.
- **Damage multipliers lowered for balance:** `SPIKE_PERCENT` 120 → 50, `BLACK_MULTIPLIER` 500 → 180,
  `WHITE_MULTIPLIER` 600 → 220. Tests that pin exact damage now derive from the rules or carry the new values.
- **Peer-like stats restored:** base attack 80 and growth 8 (as in Rally 3.3), hp growth 90; armour and magic
  resistance growth 3 → 6 (Levi 8, Contender 4).

## Athlete AI (decisions only; manual Lab mechanics unchanged)

- Bursts are valued by the form threshold they cross (Black 6 m, White 9.5 m) at the contact this athlete would reach
  (perfect or good takeoff weighted by her perfect-takeoff chance), not linearly per bar.
- Pace policy: before any form, save toward three bars (a three-bar Self Set reaches Black); once Black, save toward
  five. Saved Pace goes only into a Self Set outside a Serve. Impatient athletes (ego and aggression above judgement
  and Strategy) spend as they go. The Impossible safety gate is unchanged.
- The run-up heads where there is room for a perfect takeoff (away from, or beside, a target that is pressing her).
- Danger is relative: a spike, a Local Impossible start and walking to a grounded ball pay only the extra exposure
  (`SPIKE_EXPOSURE_PERCENT` of the incoming danger) over palming or receiving in the same spot.
