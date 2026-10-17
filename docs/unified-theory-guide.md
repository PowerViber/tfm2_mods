# Unified: the equation kit

Einstein, Newton and Marie Curie share one body and **18 equations**, six each. Every number in an equation is its effect: Apple hits harder from higher, Orbit hits harder point-blank, Time Field slows fast movers most, Photoelectric only frees energy above a threshold.

Round 138 replaced the old kit, which was 75 recipes, 50 templates, contexts, Overcharge, styles and Grand Unification. The rules now live in one deterministic core, `native/tfm2_custom_ai/src/unified_core.rs`. The game passive (`unified_theory.rs`) and the **Equation Lab** in the editor (`editor/unified.html`, the same core compiled to WASM) both run it.

## The rules

| Rule | Value |
|---|---|
| Charges | 5, each refilling on its **own** 6 s timer once spent |
| A step | **Cast** a skill, or **Reserve** it (spends a charge; the skill becomes a trait of the next cast) |
| Combo | up to 4 reserved + 1 cast = 5 steps; order matters (`Prism ▸ Well` ≠ `Well ▸ Prism`) |
| Lent power | each reserved skill adds 80% of its own power to the cast's first hit |
| Link bonus | +25% per step after the first |
| Insight | a step within 0.4 s of the previous one: +15% |
| Doubling | reserving the same skill twice doubles its trait (never more) |
| Reserve timeout | 4 s; after that the reserve fizzles and its charges stay spent |
| Cooldowns | per skill, starting when the combo is **cast** (so one combo may reserve a skill twice) |
| Damage limit | at most 35% of a champion's max HP per 2 s from Unified |
| Control limit | at most 1 s of root, pull or knock per target per 3 s; crowd-control-immune targets ignore it |

## The 18 equations

| Scientist | Skill | Equation | What the numbers do | Cooldown |
|---|---|---|---|---:|
| Newton | Apple | `E = m·g·h` | a lob that lands where the target will be; +60% at full height (range); 0.5 s root | 5 s |
| Newton | Prism | `n₁·sinθ₁ = n₂·sinθ₂` | refracts into 3 rays (40% each) at the first hit; closer hits spread wider | 5 s |
| Newton | Orbit | `F = G·m₁m₂ / r²` | a planet circles its arrival point, pulling and striking; 160% point-blank, 50% from afar | 7 s |
| Newton | Cradle | `p = m·v` | hits the first enemy in a line; the momentum passes to the last (+15% per body between) and knocks it | 7 s |
| Newton | Third Law | `F₁₂ = −F₂₁` | 1.5 s shield; then enemies within 22000 are pushed and take 30% of the damage Newton took | 9 s |
| Newton | Calculus | `dHP/dt` | marks for 3 s; the next hit adds 35% of the HP the target just lost | 7 s |
| Curie | Radium | `N = N₀·e^(−λt)` | a zone whose dose halves every 2 s; recast to enrich (N₀ resets) | 9 s |
| Curie | Alpha | `⁴₂He²⁺` | heavy, short, stopped by the first body | 5 s |
| Curie | Beta | `F = q·v × B` | curves toward the nearest enemy, pierces one | 5 s |
| Curie | Gamma | `I = I₀·e^(−μx)` | long, through walls and every body, ×0.85 per body crossed | 7 s |
| Curie | Half-life | `Σ a·(½)ⁿ = 2a` | a, a/2, a/4, a/8, a/16: the total converges to 2a | 7 s |
| Curie | Chain Reaction | `k = n₁ / n₀` | jumps 4 times, × k each; k = 0.7, or 1.0 when it starts inside Radium | 7 s |
| Einstein | Time Field | `γ = 1/√(1 − v²/c²)` | slows enemies 15% + up to 30% more by speed; allies +15% | 9 s |
| Einstein | E = mc² | `E = m·c²` | 20 + 4% of max HP as true damage; through a Time Field it waits 1 s and gains 25% | 9 s |
| Einstein | Einstein Ring | `θ_E = √(4GM·d / c²)` | a pulse, then Unified's rays crossing it bend toward the nearest enemy (+15%) | 7 s |
| Einstein | Twin Paradox | `Δt′ = γ·Δt` | an ally returns to its place and HP of 3 s ago (up to 20% max HP); once a minute per ally | 15 s |
| Einstein | Spacetime Well | `g = G·M / r²` | pulls enemies to the centre, harder when closer | 9 s |
| Einstein | Photoelectric | `E = h·f − W` | 60 + 75% AP against a target carrying ≥ 2 of Unified's marks, otherwise 15% | 7 s |

Unified's marks are root, slow, decay, Calculus, irradiated, pulled, knocked and revealed. Photoelectric and the Threshold trait count them.

## Traits, signatures and Unification

The trait each skill gives when reserved, and **every one of the 324 two-step combos**, are listed in [unified-combos.md](unified-combos.md).

Examples:
- **Prism ▸ Spacetime Well** is a *Lasing Spacetime Well*: the well fires small lasers at what it pulls.
- **Spacetime Well ▸ Prism** is a *Gravitic Prism*: the rays pull toward the beam.

**36 signatures** (3–5 steps) add a bonus (+40 / +60 / +90%) and an effect of their own. Examples:
- *Criticality* (Radium ▸ Chain Reaction ▸ Gamma): k = 1, through walls.
- *Curie's Notebook*: a zone that fires alpha, beta and gamma.
- *Annus Mirabilis*: the bolt fires, rewinds and fires again.
- *Time Loop*: the orbit replays three times.
- *Theory of Everything*: four reserved, then the ultimate.

The full list is in the combos document, and the Equation Lab plays any of them.

A combo that uses all three scientists fires a **Unification** (10 s cooldown). Which one depends on the order in which the scientists first appear:

| Order | Unification | Payoff |
|---|---|---|
| Newton → Curie → Einstein | Unified Burst | a second strike for 40% of the combo, +30% |
| Curie → Einstein → Newton | Unified Field | enemies near the target pulled together and rooted 0.4 s |
| Einstein → Newton → Curie | Unified Life | heals 30% of the combo's damage |
| Newton → Einstein → Curie | Unified Shield | 12% max-HP shield for 3 s |
| Curie → Newton → Einstein | Unified Proof | true damage of 8% of the target's missing HP |
| Einstein → Curie → Newton | Unified Motion | +30% speed for 2 s and the oldest charge returns |

**Theory of Everything** (the ultimate, 60 s) warns for 0.6 s. It then replays the last combo at 125% without spending charges, plus whatever is reserved now.

## Mastery

Rank never adds power. It changes how the athlete plays.

| Rank | Plans up to | Step pace (Insight ≤ 24 ticks) | Aim leads by | Reads the equations | Knows signatures |
|---|---:|---:|---:|---:|---|
| Student | 1 step | 34 ticks | 0% | 0% | none |
| Lab Assistant | 2 | 30 | 0% | 15% | none |
| Researcher | 2 | 26 | 30% | 35% | none |
| Scientist | 3 | 22 | 50% | 55% | none |
| Professor | 4 | 18 | 70% | 70% | 3-step |
| Fellow | 4 | 14 | 85% | 85% | 3-step |
| Laureate | 5 | 10 | 100% | 95% | all |
| Unified Mind | 5 | 8 | 100% | 100% | all |

**How each rank behaves:**
- **Reading the equations.** A Student believes every equation is always worth its full value: it fires Photoelectric with no marks, drops Apple point-blank and throws Orbit from afar. A Unified Mind reads distance, marks, motion, grouping and the damage limit.
- **Noise and mistakes.** Lower ranks carry estimation noise (45% down to 0%) and a chance of a random legal play (30% down to 0%).
- **Banking.** From Professor up, the athlete saves charges for a combo worth more per charge than what it could cast now.

**The rank ladder** from `cargo test --release unified_core_tests::rank_` at 40 seeds (`tools/record-rework-comparison.py` writes it to `docs/rework-results/unified-comparison.json`):

| Rank | Damage | Avg steps | Signatures | Insights | Misplays |
|---|---:|---:|---:|---:|---:|
| Student | 2711 | 1.00 | 0 | 0 | 115 |
| Lab Assistant | 2891 | 1.17 | 0 | 0 | 44 |
| Researcher | 3252 | 1.13 | 0 | 0 | 27 |
| Scientist | 3329 | 1.14 | 0 | 54 | 11 |
| Professor | 3796 | 1.88 | 29 | 371 | 0 |
| Fellow | 3730 | 1.92 | 47 | 380 | 0 |
| Laureate | 4804 | 2.00 | 140 | 405 | 0 |
| Unified Mind | 4793 | 2.10 | 156 | 421 | 0 |

**Head to head,** Unified Mind beats Student in 40 of 40 duels.

**Planner cost:** about 235 µs per search at the top rank, roughly twice a second (about 8 µs per tick on average).

## What you see in a match

Above the head, bottom to top:
1. the 5-pip charge bar;
2. the reserved traits;
3. the equation of the cast;
4. its live value;
5. a signature or Unification banner.

Each projectile flies in its own colour, impacts are drawn per skill, and the four zones are drawn at their game radius. The scientist's outfit, cast poses, morphs, rank emblems and cosmic auras are unchanged. Art: `Claude outputs/unified_theory/equation_art.py` produces the sheets `science_eq_labels`, `science_eq_hud` and `science_eq_fx`.

## Equation Lab

Open the editor and choose **Equation Lab**, or open `/unified.html`. It shows:
- the 18 equations, each with Reserve and Cast;
- per-charge timers and the reserve queue;
- the name the next cast would take, and the signatures the reserve can still complete;
- a signature player, the AI at any rank, and live statistics.

## Files and checks

- Data: `Claude outputs/unified_theory/equations.json`. `python3 tools/generate-unified-equations.py` writes `unified_eq_data.rs`, `editor/unified-equations.js` and `docs/unified-combos.md`; add `--check` to only verify them.
- Native: `cargo test --release` in `native/tfm2_custom_ai`, covering every skill, every pair, every signature, the limits, determinism, the rank ladder, the duel, planner cost, and every drawn effect being shipped.
- Art and data: `python3 tools/verify_unified_theory.py` (regeneration byte-identical, lifetimes match the core).
- Lab: `node tools/verify_unified_lab_browser.cjs` (needs Playwright).
- Careers: `tools/migrate-champion-reworks.py` replaces Unified's embedded view lists with the shipped ones.
