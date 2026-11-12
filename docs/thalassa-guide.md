# Thalassa, the mermaid harpist

Original family. A **mage carry** (mid or bot) who can support. She deals magic damage with ranged harp bolts. She has no mastery, her Mod Power is `[0, 10, 5, 5, 10, 0, 10]` and she gains 12 AP per level (round 160; were 25% and 26), and she keeps the global Flash.

Every number lives in `rules/thalassa.json`, and `tools/generate-thalassa.py` regenerates the code tables, data, tooltips, roster entry and editor data from it.

## Water
Water means her pools (radius 35,000, lasting 10 s), her whirlpools, the twin's rescue pool and the trident's pool. She also counts as in water for 1.5 s after a slide on land ("wet").

| | Skill | What it does |
|---|---|---|
| Passive | **Tidebound** | In water: +20% move speed, +50% range (82,500; clones retain +20% range), 1% max HP per second. At full HP the regen becomes a shield (overheal, max 10% HP). Only her own overheal that she still has counts toward the cap, so a hit that breaks it lets the regen refill it; it fades about 2 s after she leaves the water. **Every pool slows enemies in it 20%** (a swirling whirlpool 30%; never stacked). No penalty on land. |
| Basic | **Tsunami bolt** (round 165) | Slower: one every **2 s** (was 1.3 s). The harp bolt hits its target as before (towers too), and **only while she is in water or wet**, a **small tsunami** sweeps from her through the target (**8,000** past it, **14,000** wide): other enemies in it take **25 + 35% AP**, and everyone in it, the target too, is **knocked up 0.35 s**, each at most **once every 3 s**. Never towers or allies. The water **bounce is gone**. Her Twin and the Trident clone still copy single bolts (no wave). |
| S1 | **Tide Pool** (8 s) | **On land:** a pool forms below her.<br>**In water:** the pool she stands in surges up to 60,000 to the target as a whirlpool (same 35,000 radius), dealing 60 + 70% AP and **pulling enemies inward once** (never through walls). If 2+ enemy champions are caught, **up to 2 (the closest to the center) bump and are stunned 1 s**. A new whirlpool forms below her.<br>A whirlpool **swirls for 2 s** (30% slow), then calms into a normal pool for the rest of its 10 s. |
| S2 | **Dive** (10 s) | **In water:** opens a pool up to 75,000 away, then dives and surfaces there (an engine glide, unseen and taking no damage). Always within 75,000, even into her own pools (the trident's clone holds the far fight instead).<br>**On land:** a short slide that leaves her **wet**. |
| S3 | **Tide Twin** (18 s) | **The clone:** a water twin for 12 s (20% of her HP, can be killed) that **plays on its own** like someone playing her: it walks (at her speed, +20% in water), kites and fires its own harp bolts at 40% (round 160), staying within 60,000 of her. It uses skills **only when she does**: Tide Pool to the same point, and Dive to her committed destination and arrival clock, including far-pool dives. It doesn't copy the ult.<br>**Rescue:** if an ally (or she) is stunned within 40,000, it flies to them and becomes a pool: cleanse, a 80 + 50% AP shield, and 1.5 s CC immunity. **A twin spawned on water rescues Thalassa at any range, instantly** (the pool appears under her); allies keep the 40,000 range.<br>**Tide link:** spawned on water, she can swap with it once when she's engaged, leaving a whirlpool behind. |
| Ult | **Trident of the Deep** (100 s) | A slow trident (2,000 units/tick; 2,600 from water) with global range, steerable 25° in total (recast to steer). The first enemy champion hit is **stunned 3 s (4 s from water)** (V1 can parry it: no stun, no damage), and enemies nearby take 120 + 70% AP (round 160). It leaves a pool where it lands (a whirlpool, and a faster throw, from water) and **a clone of her** there: 8 s, 25% of her HP, fighting **on its own** with harp bolts, Tide Pool and Dive at 40% damage (round 160; crowd control at full strength). She stays where she is. A new trident replaces it; it never rescues or tide-links. |

## AI
Her athlete attributes change her choices, never her power.

- **S1:** makes a pool before a fight; surges onto groups (2+ for the bump) or onto a lone enemy if aggressive enough.
- **S2:** dives or slides away when low and caught; follows up into one of her pools in range that holds an enemy champion.
- **S3:** spawns the twin when 2+ enemies are close, on a pool when she can (for the tide link). She swaps when dived onto or hit hard. The twin's own AI: nearest visible enemy champion within 70,000 (a farmable unit when no champion is near), walk into range, back off below 60% of its range, otherwise return near her.
- **Ult:** throws at groups or low enemies, leading their movement. Aim improves with `skill_hit`. She prefers throwing from water.
- **The trident's clone:** fights like the twin (target, range, kiting) but within 80,000 of where it landed, and casts on its own: Tide Pool onto the spot that catches the most champions (a pool below it on land), Dive onto a champion beyond its bolts, or away when below 40% HP.
- **Farming:** surges whirlpools onto waves and camps when it's calm.

## Code
- `native/tfm2_custom_ai/src/thalassa_core.rs` (core), `thalassa.rs` (bridge), `thalassa_core_tests.rs` + `thalassa_tests.rs`.
- **Lab:** `native/thalassa_lab` → `editor/thalassa.html` (the "Thalassa Lab" link in the editor). Browser test: `node tools/test-thalassa-lab.cjs`.
- **Art:** `docs/thalassa-art-handoff.md`.

## Known limits
- The twin is a hidden carrier unit drawn as an effect, so its look depends on Codex's `twin_p*` frames.
- The twin's spawn, flights and the tide-link swap never land on terrain or walls (checked in the core and again with `place::safe`). Its flights and dive echo are engine glides, drawn every tick in motion; only the swap is an instant teleport.
- Grimhald's thrown axe has the same missing V1 parry the trident had; not fixed yet.
- Engine motion (dives, pulls) follows the round-156 rule. Live smoothness still has to be confirmed in play.

## Lab playback (rules thalassa-1.1)
The Lab advances at 60 simulation ticks per second independently of monitor refresh. Pausing and manual stepping retain one-tick semantics; background tabs do not accumulate catch-up. The separate artwork inspector was already time-based. This corrects the high-refresh playback bug without changing body frame durations or Dive travel timing.

Trident tuning uses Grimhald's 66-tick axe flight as a typical-range reference: 132,000 units takes 66 trident flight ticks on land (about 51 from water). Grimhald's own flight duration is fixed, so this is not an equal-speed claim at every distance. Damage, stun, steering allowance, windup and cooldown are unchanged.
