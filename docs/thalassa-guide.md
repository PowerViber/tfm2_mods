# Thalassa, the mermaid harpist

Original family. A **mage carry** (mid or bot) who can support. She deals magic damage with ranged harp bolts. She has no mastery, her Mod Power is `[0, 25, 5, 5, 10, 0, 10]`, and she keeps the global Flash.

Every number lives in `rules/thalassa.json`, and `tools/generate-thalassa.py` regenerates the code tables, data, tooltips, roster entry and editor data from it.

## Water
Water means her pools (radius 35,000, lasting 10 s), her whirlpools, the twin's rescue pool and the trident's pool. She also counts as in water for 1.5 s after a slide on land ("wet").

| | Skill | What it does |
|---|---|---|
| Passive | **Tidebound** | In water: +20% move speed, +20% range, harp bolts bounce to a second enemy (50%), 1% max HP per second. At full HP the regen becomes a shield (overheal, max 10% HP). Only her own overheal that she still has counts toward the cap, so a hit that breaks it lets the regen refill it; it fades about 2 s after she leaves the water. No penalty on land. |
| S1 | **Tide Pool** (8 s) | **On land:** a pool forms below her.<br>**In water:** the pool she stands in surges up to 60,000 to the target as a whirlpool (same 35,000 radius), dealing 60 + 70% AP, slowing 30% and pulling inward. 2+ enemy champions caught **bump and are stunned 1 s**. A new whirlpool forms below her.<br>Whirlpools keep slowing and drawing enemies in. |
| S2 | **Dive** (10 s) | **In water:** opens a pool up to 75,000 away, then dives and surfaces there (an engine glide, unseen and taking no damage). Aimed into **one of her own pools** (the trident's included), she dives there at any distance, taking longer the farther it is.<br>**On land:** a short slide that leaves her **wet**. |
| S3 | **Tide Twin** (18 s) | **The clone:** a water twin for 12 s (20% of her HP, can be killed) copies her Tide Pool and harp bolts at 50%; Dive copies her committed destination and arrival clock, including far-pool dives. It doesn't copy the ult.<br>**Rescue:** if an ally (or she) is stunned within 40,000, it flies to them and becomes a pool: cleanse, a 80 + 50% AP shield, and 1.5 s CC immunity.<br>**Tide link:** spawned on water, she can swap with it once when she's engaged, leaving a whirlpool behind. |
| Ult | **Trident of the Deep** (100 s) | A slow trident (2,000 units/tick; 2,600 from water) with global range, steerable 25° in total (recast to steer). The first enemy champion hit is **stunned 3 s (4 s from water)** (V1 can parry it: no stun, no damage), and enemies nearby take 150 + 90% AP. It leaves a pool where it lands (a whirlpool, and a faster throw, from water). |

## AI
Her athlete attributes change her choices, never her power.

- **S1:** makes a pool before a fight; surges onto groups (2+ for the bump) or onto a lone enemy if aggressive enough.
- **S2:** dives or slides away when low and caught; follows the trident into its pool.
- **S3:** spawns the twin when 2+ enemies are close, on a pool when she can (for the tide link). She swaps when dived onto or hit hard.
- **Ult:** throws at groups or low enemies, leading their movement. Aim improves with `skill_hit`. She prefers throwing from water.
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
