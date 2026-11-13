# Akane, the crimson katana assassin

Akane is an Original-family **assassin**, designed by Reihan (Nov 11): **fast and deadly, but easy to kill**. She plays jungle or mid, scales on Attack, keeps Flash, and has no mastery. Mod Power is `[20, 0, 0, 5, 5, 0, 10]`, with no HP bonus. Her loop is Reyna-style: kill, absorb the soul, reset, go again. Every number lives in `rules/akane.json` (revision `akane-1`); these are starting values.

## Kit
| | Skill | What it does |
|---|---|---|
| Passive | **Soul Harvest** | A **champion kill or assist** leaves a **Soul Orb** where the enemy died, for **15 s**. After **0.5 s**, staying within **25,000** of it for **0.75 s** absorbs it. She can move meanwhile; leaving the range or being stunned cancels it.<br>Absorbing gives back **one Soul charge** (no limit on how often) and **resets Kunai Flip**. It also **resets Crimson Edge at most once every 30 s**. It never resets the ult. |
| Passive | **Blood Rhythm** (round 165) | Each basic attack on a champion, minion or creep (never a tower) adds a stack, up to **5**, for **4 s**: **+5% basic-attack damage** per stack (+25% at 5). It multiplies with Ambush. |
| S1 | **Dismiss & Devour** (2 charges, 10 s each) | Not a dash. For **1.5 s** she's **invisible and invulnerable** (100% damage reduction, CC-immune) and **60% faster**, but she **can only move** (no attacks or skills) until it ends. She **heals 6% max HP + 30% Attack**. Windows never stack: a second charge extends to its own end. |
| S2 | **Kunai Flip** (10 s) | She **flips back 30,000 over 0.5 s** and throws a kunai (**120,000** line at 180,000 units/s; it flies over walls). The first enemy hit takes **25 + 50% Attack** and is **marked for 3 s**; it can be a champion, minion or creep, never a tower. **Recast:** she dashes to the mark, **over walls too**, and slashes for **35 + 70% Attack**. V1 can parry the slash. A miss means no recast. |
| S3 | **Crimson Edge** (12 s) | Her **next 5 basic attacks** within 6 s:<br>1. bleed;<br>2. bleed and **30% slow**;<br>3. bleed and **+15% damage taken** for 3 s;<br>4. the 4th stack **bursts** for **6% of the target's max HP + 35% Attack**, half of that to enemies within 18,000, plus a **crimson bleed** of 0.8% max HP a second for 3 s;<br>5. she's **invulnerable for 0.4 s**, **teleports behind the target** and **knocks up** enemies within **20,000** for **0.75 s**.<br>Bleed: 5 + 8% Attack a second per stack, up to 4. Towers take her basic attacks but no debuffs, and don't use up a hit. |
| Ult | **Crimson Night** (100 s) | For **4 s** every enemy is **nearsighted**: her whole team is hidden from them except within **20,000** of an enemy champion, minion or tower. **Recast once** to dash, over walls, to a visible enemy champion within **240,000** and slash for **85 + 75% Attack**; the slash sets off a **lesser crimson explosion** (50% of Crimson Edge's burst, half that within 18,000, with the crimson bleed) that **fears** everyone hit for **0.75 s**. |

**Stats (fragile):** category Assassin; tags AD, Melee, CC, Dot. Attack **72** + 8 (78 before `akane-1.6`), HP **850 + 85**, armor 18 + 3, MR 20 + 3, move speed **1,080 + 10**. Basic range 22,000, a fast swing (48 ticks).

**Shared rules:** her skills never touch towers; cage and realm gates apply; V1 can parry the Kunai slash and the ult slash.

## Her AI (her own; the global AI is a separate project)
She reads, through her frame, each enemy champion's:
- Flash readiness (the `flash_cd` buff);
- skill readiness (`entity.cooldown`; a native kit's data slot reads as unknown, so ready);
- abilities from the roster (Dash, Blink, CC, Heal, Shield);
- protections (CC immunity, damage reduction, V1's parry);
- HP, shield, armor and velocity.

She also reads towers and whether something of hers is tanking them, plus walls and bushes. She remembers who used Flash and who has gone missing.

- **Retreating is the game's job** (round 166, `akane-1.5`). She has no disengage of her own: the engine's AI walks her and decides when to back off. Her own steering is only toward her target while Dismiss is up, and toward an orb between fights. A steer never replaces an attack or a cast the engine wanted (`akane::steer_input`).
- **Her athlete's ego decides whether she starts a fight:**

  | | Low ego (below 35) | 50 | High ego (70+) |
  |---|---|---|---|
  | Starts a fight | only when a teammate is already on the target, or the target is isolated **and** lethal | a near-kill (20% short), an isolated target, or a teammate's fight while she's at 60%+ HP | a near-kill down to 35% short, and into a 2v1 if she has an out |
  | Danger she accepts | −15 points of HP | as before | +15 points |
  | Counts as hurt below | 45% HP | 35% | 25% |
  | After a takedown | starts nothing new in that fight | re-scores with the chain bar | the same |

  A modest athlete still leaves a kill to a closer teammate. Each teammate on the target (up to two) also adds 15 points to the danger she accepts.
- **Engage:** the Kunai on a target her ego will fight, with an out (a Soul charge or Flash) and a landing she can stand. Her combo's estimated damage is weighed against the target's effective HP plus a margin for its outs: +25% for Flash, +15% for a ready dash or blink, +15% for a ready heal or shield. CC immunity means not now; V1's parry means she never slashes into it.
- **Readiness per slot:** an enemy's CC counts only if its CC slot is off cooldown, so a ready damage skill is no longer read as a ready stun. Native kits' slots can't be read, so they count half.
- **Danger is weighted by distance:** an enemy within a short step counts fully; one that needs a dash or Flash to reach her counts half; the target she's going in on doesn't count against its own landing.
- **Dismiss & Devour:**
  - **As a dodge:** only against something real: damage just taken (8% in half a second with an enemy around), a tower on her, being hurt (the ego line above), or danger over her limit below 70% HP. A ready CC nearby alone is no longer a reason.
  - **As a fighting window:** invulnerable and hidden, she goes to her target. In reach, the engine swings (the 2nd swing ends it into Ambush); she empowers it with Crimson Edge, or uses the Kunai on a near-kill. If she's really hurt, she issues no steer and the game's AI walks her out.
  - **As an engage:** when the Kunai can't reach the target (on cooldown, or a wave in the way), a healthy Akane may Dismiss in on a near-kill within 60,000. An average athlete keeps one charge in reserve unless the kill is sure; a high-ego one spends it.
- **Kunai recast**, re-judged in the moment:
  - not into an untanked tower, unless it kills and she has a charge to leave;
  - **held** when the target Flashes away, and used if it lands back in reach.
- **Orbs:** she walks to one only between fights (no enemy champion within her reach plus 20,000), and only when the 0.75 s is safe.
- **Crimson Night:** cast to pick an isolated kill, or to hide her team when she's hurt with no charge left (the game walks her). The recast dash waits for the target's Flash, or for the end of the window.
- **The 13 athlete values** change her choices, never her damage:

| Stat | What it changes |
|---|---|
| aggressive | the margin she accepts, and how far she chains kills |
| judgement | how accurate her damage and danger estimates are |
| positioning | where the finale lands her (target between her and its allies), and her danger tolerance |
| control_speed, skill_avoid | her reaction time for dodges |
| skill_hit | how far the Kunai's aim leads the target |
| ego | **whether she starts a fight** (see the table above), the danger she accepts, when she counts as hurt, Dismiss as an engage, and leaving a kill to a closer teammate |
| order | whether she joins the team's fight |
| roaming | how far she hunts |
| macro | pushing on after a kill rather than resetting |
| micro | when she commits Crimson Edge |
| awareness | the threat radius, and how long a missing enemy stays a worry |
| strategy | how long she holds the ult's dash, and when she ults to pick |

## Code
- **Native:** `akane_core.rs` (core and AI), `akane.rs` (bridge), `akane_tests.rs` (24 core and 6 mock-host tests). Generator: `tools/generate-akane.py`.
- **Movement:**
  - every visible move is an engine glide (`crate::motion`);
  - a dash's hop over a wall and the Crimson Edge finale teleport are the only hand-set positions, both allowlisted;
  - the AI walks her through a short `akn_go:x:y` buff that the shared input AI (`steve::WallAi`) turns into a move order, for her only.
- **Lab:** `native/akane_lab` → `editor/akane.html`, on the real 5v5 map. Browser test: `node tools/test-akane-lab.cjs`.
- **Art:** `docs/akane-art-handoff.md`.

## Wave clear (round 165, `akane-1.3`)
- **Rhythm cleave:** at 3+ Blood Rhythm stacks, her basic attacks cleave within 12,000 of the target: 80% to minions and creeps, 40% to champions.
- **Kunai Flip pierces** up to 4 minions or creeps. It marks the first champion behind them, or the last one it went through.
- **Crimson Edge** splashes 50% around each empowered hit, and on a minion or creep the 4th hit always bursts.
- **No extra CC** on champions from any of these, and never towers.

## Crimson Night explosion (round 165, `akane-1.4`)
- The ult's slash sets off a lesser crimson explosion: **3% max HP + 17% Attack** on the target (`akane-1.6`), half that within 18,000, crimson bleed, and a **0.75 s fear** away from her.
- **Kept:** the target's bleed stacks, so a following 4th empowered hit still bursts.
- **Never:** towers or allies. A V1 parry cancels it.

## Known limits
- **Released in round 164** (native 0.10.98, tfm2_custom 0.2.101) with Codex's approved art and the `akane-1.1` ranges. Live acceptance is still pending.
- **Healing reduction:** Dismiss's heal is a normal heal, so healing reduction on her shrinks it.
- **Skill readiness:** native kits' skills read as "ready" (their data slots never come off cooldown), so she treats them as dangerous.
- **Balance:** unplayed. The level-10 table goes in the release review.

### Range and travel revision (2026-11-11)
Reihan requested farther backflip, kunai and ultimate dash with slower movement, then confirmed visible kunai flight. Rules are now `akane-1.1`: backflip 30,000 over 30 ticks; kunai 120,000 at 3,000/tick; recast dash 3,000/tick; Night dash 240,000 at 4,000/tick. The kunai keeps a fixed endpoint, snapshots Attack and applies one hit/mark on swept contact. Damage formulas, cooldowns, mark duration and other kit mechanics are unchanged. Old recordings require their original rules.

## Damage nerf (round 173, `akane-1.6`)
Reihan asked for a nerf to her overall damage. Every damage number is about 15% lower; cooldowns, ranges, CC, protections and her AI are unchanged.
- Base Attack 78 → 72.
- Blood Rhythm: +6% → +5% per stack.
- Ambush: +20% → +15%.
- Kunai: 30 + 60% → 25 + 50% Attack; the recast slash: 40 + 80% → 35 + 70%.
- Crimson Edge:
  - bleed: 6 + 10% → 5 + 8% Attack a second per stack;
  - burst: 8% max HP + 40% → 6% max HP + 35% Attack;
  - crimson bleed: 1% → 0.8% max HP a second.
- Crimson Night: 100 + 90% → 85 + 75% Attack.
