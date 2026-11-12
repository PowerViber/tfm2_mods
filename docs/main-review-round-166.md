# Round 166: Akane engages while healthy (native 0.10.100, tfm2_custom 0.2.103, rules `akane-1.5`)

Reihan: "Akane doesn't engage fights even when she's healthy; she behaves very differently from the other AIs." Then:
- She's invulnerable and unhurt in Dismiss, yet she disengages, even though she can attack and cast in it. A waste.
- She should engage according to her athlete's ego.
- Her own disengage isn't useful: the game's AI already has its own retreat.

## Why (reproduced with her real core from main 823773d)
**Setup:** average athlete, full HP (850), enemies with 900 HP and 70 Attack.

| Scenario | Before | After |
|---|---|---|
| A. One CC enemy 35,000 away, not attacking | Kunai, then a Dismiss "dodge" 30 ticks later, then 2 s of running home. Recast, then Dismiss again: both charges gone, steering away for 213 of 300 ticks. | Kunai, then recast. Both charges kept, no steering away. |
| B. The same, with an ally next to it | Kunai, then recast. | Kunai, then recast. |
| C. Two CC enemies 60,000 away (a 2v2 lane) | Dismiss at once, again at 200, and ~3.5 s running home. | No Dismiss and no steering. The game's AI walks and fights her; she casts when her ego's gate passes. |

**Causes:**
1. **The alarm fired on potential danger, not on anything hitting her.**
   - Every enemy within 105,000 (walk + dash + Flash) counted fully, plus 200 for a "ready CC".
   - One CC enemy within 40,000, or any two enemies near her, crossed the line at full HP.
   - She then spent a Dismiss charge and ran for 2 s.
2. **"CC ready" was almost always true.** Readiness was "any slot ready", and abilities were "anything in the kit". A ready damage skill therefore read as a ready stun.
3. **Her steering cancelled attacks.** The input AI turned her `akn_go` buff into a walk whatever the engine wanted, so an attack or cast the engine had chosen was dropped. During every retreat, orb walk and Dismiss she couldn't swing back.
4. **The orb walk ran mid-fight.**
5. **After any kill or assist she ran for 2 s** unless the next kill was certain.
6. **Dismiss was spent running.**
   - She is invulnerable in it and may cast and swing once (round 165).
   - But her AI only used it on a target already killable with the Ambush bonus; otherwise it steered her to a "safe spot".

## Changes (Akane only; her kit and damage numbers are unchanged)
- **No retreat of her own.** `disengage_until` and `safe_spot` are gone. The game's AI walks her and decides when to back off.
  - Her steering is now only toward her target in Dismiss, and toward an orb between fights.
  - It never replaces an attack or a cast: `akane::steer_input`, used by `steve::WallAi`.
- **Ego decides whether she starts a fight** (`AI_MODEST_EGO` 35, `AI_BOLD_EGO` 70). An average athlete (50) picks fights.

  | | ego < 35 | 50 | ego ≥ 70 |
  |---|---|---|---|
  | Starts a fight | joins a teammate on the target, or takes a sure isolated kill | a near-kill (20% short), an isolated target, or a teammate's fight at 60%+ HP | a near-kill down to 35% short, into a 2v1 with an out |
  | Danger she accepts | −15 points of HP | as before | +15 points |
  | Hurt below | 45% | 35% | 25% |
  | After a takedown | starts nothing new in that fight | re-scores (chain bar) | re-scores (chain bar) |

  Each teammate on the target (up to two) adds 15 points to the danger she accepts.
- **The dodge needs something real:**
  - 8% of her HP lost in half a second with an enemy around;
  - an untanked tower on her;
  - being hurt;
  - or danger over her limit below 70% HP.
- **Dismiss is a fighting window.**
  - Healthy, she goes to her target. In reach she lets the engine swing (the 2nd swing ends it into Ambush), empowers it with Crimson Edge, or uses the Kunai on a near-kill.
  - Really hurt, she issues no steer.
  - A healthy athlete may also Dismiss **in** when the Kunai can't reach: a near-kill within 60,000. She keeps one charge unless ego ≥ 70 or the kill is sure.
- **Per-slot readiness.** The bridge reads each enemy slot's cooldown with that slot's roster kinds. Native kits' unreadable slots count half.
- **Danger by distance.** Within a short step (35,000) an enemy counts fully; within dash or Flash reach, half. The target she dives doesn't count against its own landing.
- **Orbs** wait until no enemy champion is within her reach plus 20,000.
- All new numbers are under `ai` in `rules/akane.json`: `closeReach`, `ccReach`, `modestEgo`, `boldEgo`, `healthyPct`, `hurtLossPct`, `dodgeLossPct`, `soulEngageRange`, `allySharePct`.

## Verification
- **`cargo test --release`:** 1,217 passed, 0 failed, 0 warnings.
- **New and updated Akane tests:**
  - scenarios A and C;
  - ego 20 vs 80 for starting a fight, and for Dismiss as an engage;
  - the ego lines;
  - Dismiss as a fighting window, and hurt in Dismiss;
  - no retreat steer after a takedown, at every ego;
  - per-slot CC;
  - danger by distance;
  - orbs waiting for the fight to end;
  - the steer passing attacks and casts through;
  - the dodge now needs a real hit.
- **Windows DLL:** rebuilt with `x86_64-pc-windows-gnu` and `SOURCE_DATE_EPOCH`; it reports 0.10.100.
- **Lab:** `editor/akane.wasm` was rebuilt from the same core, and the `native/akane_lab` tests pass (9).
- **Generators:** `generate-akane.py --check` and `generate-roster.py --check` are current.
- **Not run here:** `tools/test-akane-lab.cjs`. The page needs `editor/game-reference/world.json`, extracted from the installed game (`node editor/extract-game-world.js`), and that file isn't in this container. Run it on Reihan's PC.

## Open
- **Live acceptance:** does she take healthy 1v1s and 2v2s, and Dismiss only out of real burst? Watch the 8% dodge threshold in long melee trades: it may spend charges quickly.
- **Native kits** read as "unknown" (half weight) for CC. Their real cooldowns aren't exposed by their data shells.
