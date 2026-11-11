# Akane, the crimson katana assassin

Akane is an Original-family **assassin**, designed by Reihan (Nov 11): **fast and deadly, but easy to kill**. She plays jungle or mid, scales on Attack, keeps Flash, and has no mastery. Mod Power is `[20, 0, 0, 5, 5, 0, 10]`, with no HP bonus. Her loop is Reyna-style: kill, absorb the soul, reset, go again. Every number lives in `rules/akane.json` (revision `akane-1`); these are starting values.

## Kit
| | Skill | What it does |
|---|---|---|
| Passive | **Soul Harvest** | A **champion kill or assist** leaves a **Soul Orb** where the enemy died, for **15 s**. After **0.5 s**, staying within **25,000** of it for **0.75 s** absorbs it. She can move meanwhile; leaving the range or being stunned cancels it.<br>Absorbing gives back **one Soul charge** (no limit on how often) and **resets Kunai Flip**. It also **resets Crimson Edge at most once every 30 s**. It never resets the ult. |
| S1 | **Dismiss & Devour** (2 charges, 10 s each) | Not a dash. For **1.5 s** she's **invisible and invulnerable** (100% damage reduction, CC-immune) and **60% faster**, but she **can only move** (no attacks or skills) until it ends. She **heals 6% max HP + 30% Attack**. Windows never stack: a second charge extends to its own end. |
| S2 | **Kunai Flip** (10 s) | She **flips back 30,000 over 0.5 s** and throws a kunai (**120,000** line at 180,000 units/s; it flies over walls). The first enemy hit takes **30 + 60% Attack** and is **marked for 3 s**; it can be a champion, minion or creep, never a tower. **Recast:** she dashes to the mark, **over walls too**, and slashes for **40 + 80% Attack**. V1 can parry the slash. A miss means no recast. |
| S3 | **Crimson Edge** (12 s) | Her **next 5 basic attacks** within 6 s:<br>1. bleed;<br>2. bleed and **30% slow**;<br>3. bleed and **+15% damage taken** for 3 s;<br>4. the 4th stack **bursts** for **8% of the target's max HP + 40% Attack**, half of that to enemies within 18,000, plus a **crimson bleed** of 1% max HP a second for 3 s;<br>5. she's **invulnerable for 0.4 s**, **teleports behind the target** and **knocks up** enemies within **20,000** for **0.75 s**.<br>Bleed: 6 + 10% Attack a second per stack, up to 4. Towers take her basic attacks but no debuffs, and don't use up a hit. |
| Ult | **Crimson Night** (100 s) | For **4 s** every enemy is **nearsighted**: her whole team is hidden from them except within **20,000** of an enemy champion, minion or tower. **Recast once** to dash, over walls, to a visible enemy champion within **240,000** and slash for **100 + 90% Attack**. |

**Stats (fragile):** category Assassin; tags AD, Melee, CC, Dot. Attack 78 + 8, HP **850 + 85**, armor 18 + 3, MR 20 + 3, move speed **1,080 + 10**. Basic range 22,000, a fast swing (48 ticks).

**Shared rules:** her skills never touch towers; cage and realm gates apply; V1 can parry the Kunai slash and the ult slash.

## Her AI (her own; the global AI is a separate project)
She reads, through her frame, each enemy champion's:
- Flash readiness (the `flash_cd` buff);
- skill readiness (`entity.cooldown`; a native kit's data slot reads as unknown, so ready);
- abilities from the roster (Dash, Blink, CC, Heal, Shield);
- protections (CC immunity, damage reduction, V1's parry);
- HP, shield, armor and velocity.

She also reads towers and whether something of hers is tanking them, plus walls and bushes. She remembers who used Flash and who has gone missing.

- **Engage** only on a lethal or near-lethal champion, with a real way out (a Soul charge or Flash) and a landing she can survive.
  - Her combo's estimated damage has to cover the target's effective HP plus a margin for its own outs: +25% if Flash is ready, +15% if a dash or blink is ready, +15% if a heal or shield is ready.
  - CC immunity means not now; V1's parry means she never slashes into it.
- **Kunai recast**, re-judged in the moment:
  - not into an untanked tower, unless it kills and she has a charge to leave;
  - **held** when the target Flashes away, and used if it lands back in reach.
- **Chaining:** after a kill she re-scores the next target with her current HP and charges. Each extra kill in the fight raises the bar; otherwise she **disengages** to the safest spot.
- **Soul charges:** she uses one to dodge a ready CC or burst, or a tower turning on her, after her reaction time. While diving she keeps one in reserve unless the kill is certain.
- **Orbs:** she absorbs one only when the 0.75 s is safe.
- **Crimson Night:** cast to pick an isolated kill or to escape. The recast dash waits for the target's Flash, or for the end of the window.
- **The 13 athlete values** change her choices, never her damage:

| Stat | What it changes |
|---|---|
| aggressive | the margin she accepts, and how far she chains kills |
| judgement | how accurate her damage and danger estimates are |
| positioning | where the finale lands her (target between her and its allies), and her danger tolerance |
| control_speed, skill_avoid | her reaction time for dodges |
| skill_hit | how far the Kunai's aim leads the target |
| ego | whether she leaves a kill to a closer teammate |
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

## Known limits
- **Released in round 164** (native 0.10.98, tfm2_custom 0.2.101) with Codex's approved art and the `akane-1.1` ranges. Live acceptance is still pending.
- **Healing reduction:** Dismiss's heal is a normal heal, so healing reduction on her shrinks it.
- **Skill readiness:** native kits' skills read as "ready" (their data slots never come off cooldown), so she treats them as dangerous.
- **Balance:** unplayed. The level-10 table goes in the release review.

### Range and travel revision (2026-11-11)
Reihan requested farther backflip, kunai and ultimate dash with slower movement, then confirmed visible kunai flight. Rules are now `akane-1.1`: backflip 30,000 over 30 ticks; kunai 120,000 at 3,000/tick; recast dash 3,000/tick; Night dash 240,000 at 4,000/tick. The kunai keeps a fixed endpoint, snapshots Attack and applies one hit/mark on swept contact. Damage formulas, cooldowns, mark duration and other kit mechanics are unchanged. Old recordings require their original rules.
