# Contender, the Shadow Boxer (`contender-2.8`)

Reihan's rework (round 178): Contender is a literal **shadow boxer**. Since `contender-2.3` he is **elusive**: a shadow that is hard to hit because he dodges, whose punches are light, and whose damage comes from his stacks. His skills never cost anything; his ult is on a 120 s cooldown (`contender-2.4`: 20 Rhythm stacks, a slower Shadow Break and a 40-stack ult; `contender-2.5`: Reihan's emergency damage nerf, see `docs/main-review-round-179.md`; `contender-2.6`: a heavy nerf to his skills and dodges, see `docs/main-review-round-181.md`; `contender-2.7`: one basic-attack dodge, a 120 s ult, the ult combo cut and a banished flurry, see `docs/main-review-round-182.md`; `contender-2.8`: his skill damage cut to about 1.3x, same round). The old kit is retired: Combination / Body Shot / Uppercut, Slip / Guard, stamina and the Main Event ring.

Category **Melee**, tags AD / Melee / CC. Basic attack **Jab / Cross** (23,000). Attack **76** (+8 per level). Every number lives in `rules/contender.json`. `tools/generate-contender.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies). The approved hooded shadow-boxer body and effects are integrated; see `docs/contender-shadow-review/README.md`.

## Passives
- **Shadows (dodges and free recasts):**
  - **Dodge:** **1**, recharging every **12 s**.
  - **The dodge:** when an **enemy champion's basic attack** hits him, his dodge is spent: the hit is undone and he slips **8,000** aside, leaving a shadow where he stood. **Skills and ults always land.** (The game doesn't tell his passive what kind of hit it was, so a hit within 1.5 s of the attacker casting an engine skill counts as a skill; native champions' own skills can still be dodged.)
  - **No costs:** his skills never spend dodges. Each skill's recast is **free, once per cast**, within its window (S2 3 s; S1 while merged; S3 while tethered).
- **Shadow Pact (echoes):**
  - **The echo:** every punch and skill hits again **1 s later** for **3%**, carrying its crowd control at half length. Echoes **never add Rhythm stacks** (so they never set off a Break).
  - **Double echoes:** every hit while he's **merged** (S1) echoes **twice**; recast hits echo once, like any other hit.
  - **Cap:** echoes add at most **1.5 s** of crowd control per target every 6 s.
  - **The ult's punches:** they echo only when the flurry is **cast while merged**.
- **Shadow Rhythm (stacks, his damage engine):**
  - **Stacks:** every hit stacks on its target (**20** max, **4 s**).
  - **Buffs:** the most-stacked enemy gives him, per stack, **+0.5% attack speed and +0.4% move speed** (+10% / +8% at 20); no Attack bonus.
  - **Vulnerability:** a stacked enemy takes **+0.5% damage from him** per stack (+10% at 20).
  - **Shadow Break:** at 20 stacks his next hit on it detonates for **1% of its max HP + 5% Attack**, and its stacks **reset to 0**. His Rhythm buffs then **stay at full (the 20-stack level) for 3 s**, even with the target at 0.
  - **Carry-over:** a max-stacked creep carries **8 stacks** onto the next champion he hits.
  - **On a kill:** a stacked kill keeps the buffs **3 s** longer.

## Skills
| Slot | Cast | Free recast |
|---|---|---|
| S1 **Shadow Merge** (14 s) | He **merges with his shadow** for **6 s**: every hit **echoes twice** (each hit still adds **1 stack**), and +10% move speed. | He **steps out** early: the shadow left behind throws a **cross** at the nearest enemy (within 30,000; 12 + 15% Attack, knocked back 20,000) and he slips 15,000 away, unhurt for a moment. |
| S2 **Shadow Cyclone** (8 s) | A shadow clone cleaves a **25,000** circle at a spot within **60,000**: 4 hits (3 + 4% Attack) and a slow. It can't be hit. | **He cleaves himself anywhere within 60,000 of him** (not necessarily the clone's spot), invulnerable, in a **35,000** circle: 3 hits (4 + 5% Attack), **each pulling enemies 5,000 toward the centre**; the last hit knocks up. |
| S3 **Shadow Tether** (10 s) | A shadow latches onto **every enemy within 50,000** (up to 5, champions first, creeps too, never towers) for **4 s**. Each hit he lands is **copied onto every tethered enemy** at **3%** (flurry punches at 2%), with a Rhythm stack each and no crowd control. A tether breaks past 90,000. | Every tethered enemy is **yanked 20,000 toward him** (stopping at walls) and uppercut: 12 + 15% Attack, 0.5 s airborne. The tethers end. |
| Ult **Shadow Flurry** (**120 s**) | **The cooldown:** every hit of his outside the flurry (basics, skills, recasts, echoes and Tether copies) takes **0.01 s** off it; flurry punches and their copies don't. **The flurry:** he's **banished for 5 s**: untargetable, unseen and unhurt, so enemies can't hit him. **36 punches** start slow and speed up, each chained to an enemy within 30,000 of the last one hit (creeps and summons too); each punch prefers a different eligible enemy. Flurry punches **never echo** (even merged) and **never set off a Shadow Break** (a full target Breaks on his next normal hit). Every second the last enemy hit is knocked up. The flurry ends early when nothing is in reach, and he reappears beside the last enemy hit with a **haymaker** ready (his next basic within 2 s). | — |

**The big combo (Reihan asked):** S1 Merge, S3 Tether the group, recast to yank them together, then the ult. The clump keeps the flurry chaining, every punch copies onto the whole group at 2%, and the Rhythm stacks race to Shadow Breaks across the group.

## AI (his own, Kicker's flat list)
1. **Recasts:** the yank once the tethered group is spread, a tethered champion is low, or the tether is about to lapse with 2+ on it; the step-out to escape when he's under 30% HP with a champion on him; the self-cleave onto the lowest champion within reach.
2. **Fight (not outnumbered, not under a tower unless the target is low):**
   - the ult when it's off cooldown, on 2+ champions, a tethered group, a low one, or any champion while merged;
   - S1 Merge with a champion within 60,000;
   - S3 Tether on 2+ enemies within 50,000;
   - S2 in reach. Otherwise it walks to the target.
   - Dodges are automatic.
3. **Farm (calm):** S2 (and its recast) on a wave or camp, S3 on 2+ creeps; both bank stacks for the ult. The ult goes on a big camp.

## Code
- **Core:** `native/tfm2_custom_ai/src/contender_shadow_core.rs`. It's deterministic, and the Lab runs this same file.
- **Bridge:** `contender.rs`.
  - **The dodge:** `on_damaged` reports an enemy champion's hit after it lands (since `contender-2.7` only basic attacks: a hit within 1.5 s of the attacker casting an engine skill is treated as a skill and lands); if the core spends a dodge, the bridge heals the damage straight back (Kratos's pattern) and the core's slip glides him aside. A lethal hit can still kill him before the heal-back, as with Kratos.
  - **Shadow Merge:** a `ct_merge` move-speed buff, and his shadow body drawn on him while merged.
  - **Tethers:** an echo flash on each tethered enemy every 12 ticks.
  - **The ult:** `entity_banish` (untargetable, unseen, input-locked), renewed every flurry tick, plus `entity_set_invisible` and an unhurt, CC-immune lease as a backstop (`contender-2.7`).
  - **Teleports:** only the ult's reappearance uses `entity_set_pos` (allowlisted).
  - **Basic attacks:** `on_attack` reports them to the core with the vulnerability.
  - **Cleave circles:** published as `ctw:` danger zones.
- **Tests:** `contender_shadow_tests.rs` (core, AI and glue).
- **Lab:** `native/contender_shadow_lab` builds `editor/contender.wasm`. `editor/contender.html` now uses the extracted real map and stock actors, with lane/gank/objective situations, fixed 60 Hz playback and approved Shadow Boxer artwork. Host movement, mitigation, visibility and stock combat are simulated. See [artwork and Lab verification](contender-shadow-review/README.md).
- **Retired, not deleted:** `contender_core.rs` and `contender_arena.rs` stay because the realm code shares their types (they now read `rules/contender-1-legacy.json`). No ring is opened any more.
- **Template:** the other champions' generators now read a frozen template, `tools/native-champion-template.json`, instead of Contender's data file.
