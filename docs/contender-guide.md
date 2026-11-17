# Contender, the Shadow Boxer (`contender-2.0`)

Reihan's rework (round 178): Contender is a literal **shadow boxer**. His shadow steps off him and fights, and his shadow charges let the shadow recast any skill, stronger. The old kit is retired: Combination / Body Shot / Uppercut, Slip / Guard, stamina and the Main Event ring.

Category **Melee**, tags AD / Melee / CC. Basic attack **Jab / Cross** (23,000). Every number lives in `rules/contender.json`. `tools/generate-contender.py` writes the Rust table, the champion data, the tooltips, the editor data and the roster entry (`--check` only verifies). The body art is unchanged; the shadow effects are placeholders until Codex's art (`docs/contender-shadow-art-handoff.md`).

## Passives
- **Shadows (charges and recasts):**
  - **Charges:** up to **5**, one every **6 s**; each basic attack takes 0.5 s off the recharge.
  - **Recasts:** for **3 s** after S2 or S3 (and for as long as S1's projection lasts), casting it again **spends a charge** and his shadow recasts it, stronger.
  - **Ult cooldown:** **every hit of his, on anything** (champions, minions, camps), takes **1 s** off the ult's cooldown.
- **Shadow Pact (echoes):**
  - **The echo:** every punch and skill hits again **1 s later** for **40%**, carrying its crowd control at half length.
  - **Double echoes:** recast hits echo **twice**, and so does every hit while he's projected.
  - **Cap:** echoes add at most **1.5 s** of crowd control per target every 6 s.
  - **The ult's punches:** they echo only when primed (S2 cast from the projected shadow).
- **Shadow Rhythm (stacks):**
  - **Stacks:** every hit stacks on its target (**10** max, **4 s**).
  - **Buffs:** the most-stacked enemy gives him, per stack, **+3% attack speed, +2% Attack and +1.5% move speed**.
  - **Vulnerability:** that enemy takes **+1.5% damage from him** per stack.
  - **Carry-over:** a max-stacked creep carries **4 stacks** onto the next champion he hits.
  - **On a kill:** a stacked kill keeps the buffs **5 s** longer.

## Skills
| Slot | Cast | Recast (1 shadow charge) |
|---|---|---|
| S1 **Shadow Projection** (12 s) | His body holds still and can't be hurt, and his shadow is launched up to **100,000**. For **5 s** it fights as him: it boxes, and S2 and S3 come from it. **70%** of the damage it takes reaches him; if it falls, that's 70% of his max HP. | His body **swaps** to the shadow, and the shadow's **cross** (from where he stood) knocks the nearest enemy back 20,000. |
| S2 **Shadow Cyclone** (8 s) | A shadow clone cleaves a **25,000** circle at a spot within **60,000**: 4 hits and a slow. It can't be hit. Cast from the projection, it **primes the ult's echoes** for 8 s. | **He goes there** and cleaves **7 hits**, invulnerable; the last hit knocks up. |
| S3 **Slam & Uppercut** (10 s) | He grabs an enemy within 20,000 and **slams** it: a stun, plus an impact ring. | An **uppercut** knock-up. From the projection it **launches the target toward his body**. |
| Ult **Shadow Flurry** (120 s; not recastable) | He's **unseen and unhurt for 5 s**, and his charges refill. **36 punches** start slow and speed up, each chained to an enemy within 30,000 of the last one hit (creeps too). Every second the last enemy hit is knocked up. The flurry ends early when nothing is in reach, and he reappears beside the last enemy hit with a **haymaker** ready (his next basic within 2 s). | — |

**The engage:** S1 projects the shadow onto the target. S2 from the shadow cleaves while untouchable, builds stacks and primes the ult. Then either:
- **Pick:** S3 slams, and the recast uppercuts the target back to his body; or
- **Commit:** the S1 recast swaps him in, then the ult.

## AI (his own, Kicker's flat list)
1. **Recasts:** the uppercut on a slammed champion. The projection swap when the target is low or his team is there; the projection lapses early when the shadow is losing. The self-cleave on a champion in the circle.
2. **Fight (not outnumbered, not under a tower unless the target is low):**
   - the ult on 2+ champions, a low one, or when primed;
   - S1 onto a target 40,000–120,000 away;
   - S2 in reach, then S3 in grab range.
   - Otherwise it walks (or steers the shadow) to the target.
3. **Farm (calm):** S2 on a wave or camp, S3 on a camp monster, S1 onto a far camp with full charges, and recasts only with full charges. The ult goes on a big camp.

## Code
- **Core:** `native/tfm2_custom_ai/src/contender_shadow_core.rs`. It's deterministic, and the Lab runs this same file.
- **Bridge:** `contender.rs`.
  - **The projected shadow:** a hidden carrier unit (Thalassa's pattern) with his max HP; the core relays 70% of what it loses.
  - **His body during a projection:** a named Bind and attack block, plus an unhurt lease.
  - **The ult:** `entity_set_invisible`, plus an unhurt, CC-immune lease.
  - **Teleports:** the swap and the ult's reappearance use `entity_set_pos` (allowlisted).
  - **Basic attacks:** `on_attack` reports them to the core with the vulnerability.
  - **Cleave circles:** published as `ctw:` danger zones.
- **Tests:** `contender_shadow_tests.rs` (core, AI and glue).
- **Lab:** `native/contender_shadow_lab` builds `editor/contender.wasm`. `editor/contender.html` now uses the extracted real map and stock actors, with lane/gank/objective situations, fixed 60 Hz playback and approved Shadow Boxer artwork. Host movement, mitigation, visibility and stock combat are simulated. See [artwork and Lab verification](contender-shadow-review/README.md).
- **Retired, not deleted:** `contender_core.rs` and `contender_arena.rs` stay because the realm code shares their types (they now read `rules/contender-1-legacy.json`). No ring is opened any more.
- **Template:** the other champions' generators now read a frozen template, `tools/native-champion-template.json`, instead of Contender's data file.
