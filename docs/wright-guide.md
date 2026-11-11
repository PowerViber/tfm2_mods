# Wright, the aircraft engineer

Wright is an Original-family **summoner mage**, designed by Reihan. He plays mid, and can jungle: his planes farm camps and waves when it's calm. His damage is magic (AP); he has no mastery, Mod Power `[0, 10, 5, 5, 10, 0, 10]` and 12 AP per level (round 160; were 25% and 26), and he keeps Flash. Every number lives in `rules/wright.json` (revision `wright-5`, after Reihan's Lab reviews and the round 160 nerf); these are starting values to tune in the Lab and in play.

## Kit
| | Skill | What it does |
|---|---|---|
| Passive | **Workshop** | Builds an **airframe every 4 s**, holding **up to 3**. S1–S3 each use one, so they need stock as well as cooldown. Every **14 s** it launches a **scout drone** (below). |
| S1 | **Fighter Wing** (6 s) | **2 jets for 20 s**, at most **4** (a new pair replaces the oldest). Each jet is **its own unit with 3 strikes of ammo** (round 160): it picks enemies **near itself** (champions first, then minions and creeps; never turrets; up to 110,000 from him), commits to where each one is (no homing), strikes, picks the next one near it, and after the 3rd strike **flies back to him to rearm (2 s)**. Jets **cruise at 2,400** and use **afterburners (1.5×)** on the final 25,000 of each approach.<br>**Gun strike:** 12 + 14% AP. **Every 3rd strike a rocket:** 24 + 26% AP and half to enemies within 8,000 (a direct hit V1 can parry).<br>**Kamikaze:** when their 20 s run out or they drop below 25% HP, they dive at the nearest enemy's spot: 20 + 22% AP within 15,000, a 20,000 knockback and a burn of 1% max HP a second for 3 s.<br>They are real, always-targetable units with **15% of his HP**. |
| S2 | **Carpet Bomber** (10 s) | **Three advancing explosions:** commits three overlapping 30,000-radius circles, normally 28,000 apart (round 160: smaller). Aim chooses the middle strike/direction; the furthest centre stays within **110,000**. Map edges shorten spacing along the same line. The first warning lasts **1 s**, then the explosions advance **0.6 s apart** (round 160: slower), each with its own countdown. Each deals **30 + 25% AP** once per enemy in that circle; the first two slow 30% for 1 s, **the third stuns 0.75 s**. Overlap can hit a unit more than once, up to once per strike. The bomber can't be targeted. |
| S3 | **Repair Wing** (14 s) | Two aircraft for 6 s, neither targetable. The **repair chopper** circles him (25,000) and every 1 s repairs the lowest-HP ally near him (him included). The **repair plane** flies to the **most hurt ally anywhere** and repairs them every 1 s. A kit is 1% max HP and a 15 + 10% AP shield (stacking up to 60 + 40% AP, 3 s). The chopper also restores 15% HP to his fighters near it. |
| Ult | **Sky Carrier** (90 s) | After a 1 s shadow, a carrier **drops on Wright**: no damage, enemies within 100,000 **knocked away** (the closer, the farther) and slowed 15% for 1.5 s. It **hovers over him for 10 s, following him**, and **takes 80% of the damage he takes** (pool: 100% of his max HP). **Round 165: it launches nothing.** While it's up it **empowers his skills**: they use **no airframe** and recharge in **half the time**; his jets **rearm 2× faster with +1 strike** (still at most 4 jets); Carpet Bomber's explosions deal **125%** (same 1 s warning and size); repair kits are **150%**. |

**Scout drone:** flies to the nearest enemy champion (or where one was last seen) within 160,000, hovers 4 s and, on arrival, **taunts** enemy champions within 20,000 onto itself for 0.75 s, once. It is a real unit with 15% of his HP.

## AI
- **Ult:** when he's under pressure (2+ enemy champions within 60,000, or below 50% HP with one close): it's a peel and a guard.
- **Ready bombers:** used on the spot catching the most enemies whenever one is on deck.
- **S3** first when an ally (or he) is below 60% HP.
- **S2** onto the spot catching the most champions; when calm, onto a camp or wave of 3+.
- **S1** when champions are near, or (calm) to farm.
- His athlete attributes change his choices, never his power.

## Code
- `native/tfm2_custom_ai/src/wright_core.rs` (core), `wright.rs` (bridge), `wright_core_tests.rs` + `wright_tests.rs`. Generator: `tools/generate-wright.py`.
- Fighters and the drone are hidden carrier units (Raze's pattern) moved by engine glides; the bomber, repair aircraft, rockets and the carrier are simulated in the core and drawn as effects. Direct strikes (rockets, kamikaze) can be parried by V1. No hand-set positions.
- **Lab:** `native/wright_lab` → `editor/wright.html` ("Wright Lab" in the editor). Browser test: `node tools/test-wright-lab.cjs`.
- **Art:** `docs/wright-art-handoff.md`.

## Known limits
- Approved original body, fleet and effects are integrated, with actual-asset inspection at `editor/wright-review.html`. See `docs/wright-art-review/README.md` for approvals, hashes and evidence. The new release DLL and actual-game acceptance remain separate; the existing packaged DLL is not a Wright release.
- The drone gives no extra vision yet (the host has no proven vision hook); it scouts by being there and taunting.
- **Summons and turrets:** no summon ever damages a turret; jets and the drone are units turrets can shoot (the game decides turret targeting, so confirm in play).
- The guard trusts `on_damaged` to report the damage after the 80% reduction; confirm the pool drains right in play.
- Jet HP (15% of his) is a starting value; tune `fighter.hpPct` after playing.
