# Round 155: Grimhald joins the game (native 0.10.90, tfm2_custom 0.2.92)

You asked for a new champion. His ultimate throws an axe across half the map. If the landing knocks anyone up, a pentagon wall rises around the axe and he jumps in. Everyone inside is caged: they can attack and use skills, but they can't get out. After that you shaped the kit over several rounds, and Codex drew the art. This round puts him in the game.

## Grimhald, the Axe-Warden
He is an Original-family bruiser who plays top or jungle, and roams when the team needs CC. He has no mastery. His Mod Power is `[20, 0, 20, 15, 15, 0, 5]`. The full kit is in `docs/grimhald-guide.md`; every number lives in `rules/grimhald.json`.

| | Skill | In short |
|---|---|---|
| Passive | Bloodrage | More damage and move speed as his HP falls (capped at 30% HP). Basic attacks deal +20% to knocked-up or stunned enemies. |
| S1 | Iron Ghost | 1 s of untargetable smoke (cleanses CC; 22 s cooldown if it did, 14 s if not). Recast to reappear early, then a burst of speed. |
| S2 | Stone Bulwark | A 70,000 fissure that slows, ending in a rock pillar. The rising pillar nudges enemies away. Relocate it once, up to 40,000 away, leaving a slowing trail. |
| S3 | Skull Bash | A 52,000 charge with a 50,000 shove. Into a wall: a long stun. Into the pillar: it shatters, knocking up everyone around it. |
| Ult | Warden's Verdict | The axe flies up to 480,000. If it knocks up a champion: an 8 s pentagon cage he leaps into.<br>Inside, the axe is planted and he fights unarmed, with faster punches that knock back, lifesteal and damage reduction. The axe's aura shreds enemies inside.<br>Each kill adds 2 s to his buff. If it hits nobody, the axe comes back. |

The cage:
- Caged champions can't blink, Flash or be knocked out, and nobody can enter.
- Inside sees only inside, like Omen's smoke.
- Minions and camps are pushed out of reach.
- The AI never throws where the cage would touch a tower's range.

## Shipped this round
- Native **0.10.90** (DLL rebuilt for Windows) and **tfm2_custom 0.2.92**, which needs 0.10.90 or later. Other packs are unchanged.
- Codex's art: the body sheet (armed and unarmed poses), the effects sheet and the weapon layer.
- README row and pack description.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1027 passed, 0 failed. This includes the Grimhald core tests and the mock-host tests for the cage, his named CCs and punches.
- `native/grimhald_lab` tests: 38 passed. The Lab WASM was rebuilt from the same core.
- Mod Manager `the_repos_champions_all_load`: passed.
- Every one of his 189 view effects and 69 view buffs resolves to an animation in the shipped sheets, and the body has all 8 animations.
- `generate-grimhald.py --check` and `generate-roster.py --check`: current.
- Lab bug from round 3 (Bulwark leaving him stuck) fixed and checked in a headless browser.

## Not proven yet (please check in play)
- **Live acceptance:** nothing here has been seen in the real game yet.
- **Balance:** the harness against Contender, Raze and Rally is next round, as you chose. Tell me how he feels; damage goes down first if he's too strong.
- **Cage isolation:** the game has no hook to stop a stock ranged champion inside from shooting a unit outside. Units are pushed beyond reach instead.
- **Iron Ghost recast:** ends within 0.1 s rather than instantly. The game has no proven way to end a banish.
- **Art notes:** the body art is Codex's; report anything that looks off and it goes back to Codex.

## How to get him
1. Merge PR #67.
2. Close the game completely, run `TFM2 Mod Manager.exe`, and choose **1 Update everything**.
3. Start the game.

Grimhald appears in exhibition matches and **new careers**. Existing careers keep the champion list they started with; `tools/migrate-champion-reworks.py` updates champions a career already has, but it can't add new ones.
