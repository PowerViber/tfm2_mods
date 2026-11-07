# Grimhald, the Axe-Warden

**Status:** released in native 0.10.90 / tfm2_custom 0.2.92 (round 155). Automated checks passed; live acceptance and the balance harness are pending.

Original family. A bruiser who plays **top or jungle** and moves to **roam** when the team needs CC. He scales with Attack (physical). He has no mastery. His Mod Power is `[20, 0, 20, 15, 15, 0, 5]`, and he keeps the global Flash, which is blocked while caged.

Every number lives in `rules/grimhald.json`. `tools/generate-grimhald.py` writes `grimhald_data.rs`, the champion data, the tooltips and the roster entry; change the rules file and regenerate, never the outputs.

## Kit

| | Skill | What it does |
|---|---|---|
| Passive | **Bloodrage** | +0.5% damage and +0.3% move speed per 1% missing HP, capped at 30% HP (+35% / +21%). Aura tiers show at HP ≤ 80%, 55% and 30%. Basic attacks deal +20% to knocked-up or stunned enemies. |
| S1 | **Iron Ghost** | 1 s of grey smoke: banished (untargetable, no damage), moving up to 45,000 at +60% speed. Recast to reappear early. Then +35% move speed fading over 2.5 s. Cleanses CC on cast. Cooldown is 22 s if it cleansed, 14 s otherwise. Reappearing within 10,000 of an enemy cuts 3 s off Skull Bash. |
| S2 | **Stone Bulwark** | 0.6 s preparation, then an advancing 70,000 × 8,000 fissure over 0.3 s: 50 + 90% Attack, 50% slow for 2.5 s. After a further 0.2 s, a rock pillar (a wall for everyone) rises at its end for 4 s, early if terrain is in the way; as it rises it **nudges** enemies within 18,000 back 10,000 (no stun; Raze-satchel-like, stops at walls). **Recast once** to relocate the pillar up to 40,000 away (it nudges again when it re-rises). Relocating pushes nobody; it leaves a 3 s trail that slows 50%. |
| S3 | **Skull Bash** | Charge up to 52,000. The first enemy takes 60 + 100% Attack, is knocked back 50,000 over 0.33 s (a long, headbutt-style shove) and stunned 0.3 s. **Into a wall** (terrain, Steve, the pentagon): 80 + 130% Attack and a 1.6 s stun. **Into the pillar** (the enemy knocked into it, or Grimhald himself): the pillar is destroyed and shatters, dealing 40 + 70% Attack with a 1.1 s knock-up in 22,000. |
| Ult | **Warden's Verdict** | 0.35 s wind-up, then the axe flies up to 480,000 (half the map) in 1.1 s, with at most 10 degrees of total steering. Landing deals 80 + 100% Attack and a 1.1 s knock-up in 40,000. **If it knocked up a champion:** a pentagon (radius 62,000) rises around the axe, he leaps in (0.6 s, untouchable), and every champion inside is caged for 8 s. **Inside, the axe is planted at the centre and he fights unarmed:** +25% damage, +80% attack speed, 12% lifesteal and 15% damage reduction, and every punch knocks back 4,000 (no stun). The axe's aura gives enemies inside -20% armour and MR and a 15% slow. **Each kill while unarmed adds 2 s** to the unarmed buff and the aura; the cage still falls on time, the axe stays planted with its aura (no walls) until the buff ends, then returns to his hands. **If not:** the axe returns, with the full 90 s cooldown. |

## The cage

- Its five edges are `sbw:` walls on Grimhald. The shared wall code handles pathing, walking, dashes and knockbacks, blink landing, Levi's cables, and sight (`steve::wall_fog`, or `valorant::fog` while a smoke is up). Inside sees only inside, and outside never sees in, as with Omen's smoke.
- `grimhald_cage::tick` (match hook) adds the rest:
  - caged champions who blink or Flash out are put back in;
  - uncaged champions can't enter;
  - minions, camps and summons are put outside (minions 30,000 out, beyond their reach).
- Flash is blocked for caged champions (`flash.rs`).
- `grimhald_cage::can_interact` keeps native hits from crossing the wall.
- The AI never throws where the pentagon would touch a tower's range (85,000), so towers never shoot in.
- **Honest limits:**
  - The game has no hook to stop a stock ranged champion inside from shooting a unit outside. Minions are pushed beyond reach, but this isn't enforced by the engine.
  - Iron Ghost's recast stops renewing 6-tick banish slices, so it ends within 0.1 s, not instantly. The game has no proven "end banish" call.
  - Nothing here has been checked in the live game.

## AI (the athlete's attributes change choices, never power)

- **Ult:** fires when 2+ enemies are in the predicted knock-up, or one enemy is at ≤ 40% HP.
  - Never into tower range: the landing point slides away or he doesn't throw.
  - Not at ≤ 25% own HP with fewer than 3 allies near the landing point.
  - Aim error shrinks with `skill_hit`.
- **Iron Ghost:**
  - cleanses CC of 0.75 s or more near enemies (or when low or in danger);
  - dodges danger zones;
  - engages to reappear behind a target, ending the ghost once a bash would connect.
- **Stone Bulwark:** scored placement that favours the pillar landing behind the target, many enemies on the line or near the pillar, and pinning against the cage edge. `judgement` weighs the setup. When escaping, the pillar goes between him and the chaser.
- **Combo:**
  - bashes an enemy whose knockback line runs into the pillar;
  - otherwise relocates the pillar behind the target;
  - otherwise charges into the pillar himself when 2+ enemies stand near it.
- **Farming:** S2 on lines of 2+ farmable units or big camps, and S3 on big camps, only when calm. The ult is never used for farming.

## Code

| File | Role |
|---|---|
| `native/tfm2_custom_ai/src/grimhald_core.rs` | deterministic core: geometry, skills, AI |
| `native/tfm2_custom_ai/src/grimhald.rs` | bridge: frame from the sim, effects to the sim |
| `native/tfm2_custom_ai/src/grimhald_cage.rs` | match hook: the pentagon |
| `native/tfm2_custom_ai/src/grimhald_tests.rs` | core tests, plus mock-host glue tests |

Art: see `docs/grimhald-art-handoff.md`.


## Current Lab and artwork revision

The later November 7 request authorizes the range/timing/steering changes above. See [the Lab and art handoff](grimhald-lab-and-art.md). The current connected, slightly slimmer body and revised volcanic/headbutt materials are review-only, awaiting approval and complete animations. The earlier split-body rig is superseded. No final game assets or release DLL are ready for installation.
