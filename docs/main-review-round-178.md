# Round 178: Contender reworked into the Shadow Boxer (native 0.10.113, tfm2_custom 0.2.116, `contender-2.0`)

**Reihan:** "rework how contender works, i want him to be a shadow boxer in the liter means of shadow boxer … remove the whole kit from skills to ult". The kit was settled over several plan revisions: charges become recasts, Shadow Pact becomes a passive, Projection is S1, Cyclone and Slam & Uppercut back it, and the ult is a Hayabusa-style flurry. The full kit is in `docs/contender-guide.md`.

## What changed
- **The new kit:**
  - **Passives:** Shadows (5 charges, recasts, the ult's cooldown cut by every hit), Shadow Pact (echoes) and Shadow Rhythm (stacks).
  - **Skills:** S1 Shadow Projection, S2 Shadow Cyclone, S3 Slam & Uppercut.
  - **Ult:** Shadow Flurry.
- **Code:**
  - a new deterministic core, `contender_shadow_core.rs`, generated from `rules/contender.json`;
  - a rewritten bridge, `contender.rs`;
  - a new Lab crate, `native/contender_shadow_lab`, and `editor/contender.html`.
- **Retired:**
  - **The old kit:** stamina, Combination, Slip / Guard and Main Event.
  - **Its code:** `contender_core.rs` and `contender_arena.rs` remain for the realm code that shares their types; they read `rules/contender-1-legacy.json`, and no ring is opened.
  - **Old tests:** the old bridge's glue tests and one test that read the old bridge's source were removed.
  - **Old careers:** they keep their round-176 data. Its `contender_basic` and `contender_ultimate` refs still resolve, as a plain basic and the shared ult marker.
- **The champion template:** the other generators read `tools/native-champion-template.json` (a frozen copy of the old data shell) instead of Contender's data file. All their outputs are unchanged (`--check`).

## `contender-2.1`: the ult's echoes come from the projection (Reihan)
"the ult echo is triggered when contender is in projection mode, not on cyclone".
- **Old rule:** Shadow Cyclone cast from the projected shadow primed the ult for 8 s.
- **New rule:** the flurry echoes every punch when it's **cast while his shadow is projected**. It then starts at the shadow (the target nearest it), and the projection ends as he vanishes. Cast on foot, there are no echoes.
- **Cyclone:** it primes nothing now (`cyclone.primeTicks` is removed).
- **The AI:** it also casts the ult on a lone champion by the projected shadow, for the echoes.
- **Tests:** three new or changed: the projected flurry echoes every punch and starts at the shadow, S2 no longer primes, and the AI flurries from the projection.

## Merged with main
- **Codex's Basalt continuity fix:** PR #90 (Basalt effect continuity) merged into `main` first and took 0.10.112 / 0.2.115, so this round moved to 0.10.113 / 0.2.116.
- **The DLL:** rebuilt from the merged source, so it carries both #90's fix and this rework.

## `contender-2.3`: the elusive shadow (Reihan)
"ult hits cant reduce the ult cd itself, the shadows are useless … i want him to be like a shadow, really hard to hit since he dodges but his punches dont do a lot bot focusing on the stacks". Built on top of Codex's `contender-2.2` commits (flurry alternation, eight-direction flurry art, the 1vX Lab), which are kept unchanged.
- **Shadows:** the charges become **dodges**: an enemy champion's hit is undone and he slips aside (5 dodges, one every 6 s, at most one per 0.4 s). Skills cost nothing; each recast is free, once per cast.
- **Shadow Rhythm:** +4% damage taken per stack (was 1.5%), and a **Shadow Break** at 10 stacks (6% max HP + 30% Attack, back to 5 stacks). Base skill damage is down about 40%, and Mod Power Attack goes 30 → 15.
- **S1 Shadow Merge** replaces the projection (and its carrier, relay and launch): 6 s of double dodge recharge, +2 dodges, double echoes and double stacks; the recast steps out with a cross.
- **S2 Shadow Cyclone:** the recast is aimed anywhere within 60,000, in a bigger 35,000 circle, and each hit pulls enemies in.
- **S3 Shadow Tether** replaces Slam & Uppercut: every enemy within 50,000 (up to 5) takes 30% copies of his hits; the recast yanks them 20,000 toward him and uppercuts.
- **Ult:** no cooldown and no cooldown cuts; it's cast from **20 banked shadow stacks** (every non-flurry hit banks one). Merged, the flurry echoes; tethered enemies take 15% copies of each punch; his dodges refill when he reappears.
- **Code:** the core, bridge (with the `on_damaged` heal-back dodge), the Lab transport and both Lab pages are updated; the wasm and DLL are rebuilt.
- **Tests:** `contender_shadow_tests.rs` is rewritten for 2.3: 31 tests (Codex's alternation and directional tests kept), covering dodges, free recasts, the bank, Merge, the aimed pulling recast, Breaks, Tether copies and the yank, the AI, and the glue (including the dodge heal-back).
- **For Codex:** the Windows parity fixtures (`tools/test-contender-shadow-parity.cjs`) and the real-map Lab checks need a refresh against 2.3; see `docs/contender-shadow-art-handoff.md` for the effect uses.

## Checks
- **Native tests (2.3):** 1,400 passed, 18 ignored, including 31 Contender tests (core, AI and glue).
- **Wasm (2.3):** driven directly: a champion hit spends a dodge and slips him; 10 stacks fire a Break; 20 basics fill the bank, the flurry spends it, and the bank stays at 0 through the flurry.
- **Lab:** the Lab crate's transport tests pass (2.3: dodges through `incoming`, the bank, then the flurry).
- **Other checks:** the generators are current; the DLL is built; the Mod Manager tests (15) pass.
- **Not run here:** the real-map Lab (it needs the extracted game reference) and Codex's Windows checks.
