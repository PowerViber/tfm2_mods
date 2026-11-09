# Wright, the aircraft engineer

Wright is an Original-family **summoner mage**, designed by Reihan. He plays mid, and can jungle: his planes farm camps and waves when it's calm. His damage is magic (AP); he has no mastery, the mage Mod Power line `[0, 25, 5, 5, 10, 0, 10]`, and he keeps Flash. Every number lives in `rules/wright.json` (revision `wright-2`, after Reihan's Lab review); these are starting values to tune in the Lab and in play.

## Kit
| | Skill | What it does |
|---|---|---|
| Passive | **Workshop** | Builds an **airframe every 4 s**, holding **up to 3**. S1–S3 each use one, so they need stock as well as cooldown. Every **14 s** it launches a **scout drone** (below). |
| S1 | **Fighter Wing** (6 s) | **2 jets for 20 s**, at most **4** (a new pair replaces the oldest). They **dogfight**: fast patrol laps around him, then they **boost in** on attack runs at his target (else the nearest enemy champion, else minions and camps), **strafe**, overshoot, **break away** in a wide turn and come around again; the two of a pair attack from different sides.<br>**Machine gun:** during runs, every 0.25 s, 7 + 10% AP.<br>**Lock-on rocket:** at the start of a run every 4 s, a reticle for 0.5 s, then a homing rocket, 35 + 40% AP.<br>**Kamikaze:** when their 20 s run out or they drop below 25% HP, they dive into their target: 30 + 35% AP within 15,000, a 20,000 knockback and a burn of 2% max HP a second for 3 s.<br>They are real, always-targetable units with **15% of his HP**: enemies and turrets can shoot them down. |
| S2 | **Carpet Bomber** (10 s) | **Nana-style:** marks an area (45,000, range 110,000) with a **1 s ready-up**, then a bomber sweeps over dropping **3 carpet waves** 0.3 s apart. Every enemy in the area is hit once per wave for 45 + 40% AP; waves 1–2 slow 30% for 1 s, **wave 3 stuns 0.75 s**. It can't be targeted. |
| S3 | **Repair Wing** (14 s) | Two aircraft for 6 s, neither targetable. The **repair chopper** circles him (25,000) and every 1 s repairs the lowest-HP ally near him (him included). The **repair plane** flies to the **most hurt ally anywhere** and repairs them every 1 s. A kit is 1% max HP and a 15 + 10% AP shield (stacking up to 60 + 40% AP, 3 s). The chopper also restores 15% HP to his fighters near it. |
| Ult | **Sky Carrier** (90 s) | After a 1 s shadow, a carrier **drops on Wright**. The drop deals **no damage**: enemies within 100,000 are **knocked away, the closer the farther** (up to 100,000), and slowed 15% for 1.5 s. It then **hovers over him for 10 s, following him**, and **takes 80% of the damage he takes** (from a pool of 100% of his max HP; it leaves early when the pool runs out). It launches a jet every 1 s (at most 6) and a repair chopper every 3 s, and **readies a bomber** every 3 s (up to 3): while it's up, **Carpet Bomber uses a ready bomber** with no airframe or cooldown and only a 0.3 s mark. It can't be targeted. |

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
