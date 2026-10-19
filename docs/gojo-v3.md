# Gojo: original effects, curved Blue and active combat

This is the current PR #40 implementation on `codex/gojo-six-eyes-black-flash`: native **0.10.60**, JJK **0.1.31**, custom pack **0.2.61** for synchronized dependencies. It supersedes the presentation and six-second-orb portions of the earlier v2 notes. Champion IDs, saved identity, statistics, growth, native bonus and the ordinary attack clock remain unchanged. Objective/Stark PR #39 remains separate. No game installation or career migration was performed.

## Combat changes

| Technique | Current behaviour |
|---|---|
| Maximum Blue | 140,000 targeting range; straight, left or right committed integer curve. Preparation, damage and costs stay unchanged. The path is checked at acceptance, release and during travel. No homing or transit damage. |
| Maximum Blue orb | Three seconds from arrival. No periodic free damage. Late entrants' whirlpool and movement hold cannot outlast the orb. |
| Maximum Red at orb | 140,000 targeting reach, with its existing preparation and real projectile collision. It must arrive before expiry; body blocking still prevents fusion. Ordinary Red remains 70,000. |
| Returning Red | An additional ordinary-Red choice sharing cost, strain, preparation and cooldown. Initial direct hit has no knockback. A 48-tick curved return ends at the first impact's fixed, warned position, dealing the existing explosion component and 24,000 knockback. The route does not track an escaping target. A missed outbound shot does not create a free return blast. |
| Existing techniques | Six Eyes discounts, damage penalty and exhaustion, Black Flash gates/amplification, Infinity, RCT amounts, Purple/Nuke payloads and both Domains' radii/control remain intact. Ordinary Blue retains its existing wall behaviour. |

Curved Blue uses a quadratic path with a 32,000-unit lateral control-point offset, a minimum 12-tick flight and an 8,000-unit-per-tick path-length bound. Its destination is fixed at commitment. The actual integer path drives collision, exported lab geometry and projectile centres. Returning Red uses a fixed cubic loop, revalidates terrain and can be interrupted by new obstructions. One cast identity binds its direct and delayed components, preventing a second Black Flash combination from the same cast. Death clears owned flights and returns.

The AI evaluates up to 16 Purple firing directions through the complete 400,000-unit line. A valuable single enemy can justify Purple or either Domain; invulnerable targets do not count as Purple value, and control-immune targets do not count as Domain value. The fixed guest and extra-allied-stun cost remain respected. A stalled melee combination waits at most one second without punch progress rather than occupying the full four-second window. Maximum output considers available Infinity protection. Returning Red is reserved for healthier, protected duels; ordinary Red still supplies separation when surrounded. RCT starts at 55% HP, or 65% with multiple nearby enemies. Spatial Step validates the actual proposed destination.

## Original animation style and hitboxes

The retained pre-PR40 body and VFX frames remain byte-equivalent at the decoded-frame level to main `a4098f1`. Existing Blue, Red, Purple, blink and Infinity artwork supplies the live presentation. New Black Flash, healing, orb, fusion and Nuke effects use the same compact pixel style. Original body poses supply their own hands; the previous added gesture layer is transparent. Active Infinity uses its original layer once rather than two overlapping active designs.

Projectile and impact silhouettes are sized to their actual collision or effect areas. Domain field/ring artwork is the original cosmic design, scaled for 64,000 and 96,000 radii, without the later star repaint. Seventy-two one-tick expansion-ring variants follow the actual expanding boundaries. Warning circles remain authoritative. Hollow Nuke shows its complete damage boundary from impact while interior decoration expands. Effects never consume gameplay randomness or allocate combat events. Native visual requests stay bounded to 48 per update, with one-tick flight samples and bounded continuous-effect tails.

![Original-style effects](gojo-results/anime-all-skills.png)

[Animated before/after](gojo-results/v3-before-after.gif) · [Both facings and poses](gojo-results/anime-gesture-facings.png) · [Curved Blue in the lab](gojo-results/v3-curved-blue.png) · [Hollow Nuke warning](gojo-results/v3-nuke-warning.png)

## Lab and recording compatibility

Gojo Lab and the Skill Test iframe open in **Combat** mode with automatic techniques, incoming attacks, scene movement and a fixture containing enemies plus one guest. Scene movement is an explicit approximation of host movement, not a claim that the host's entire movement AI is reproduced. **Manual** mode keeps technique choices with the user. Hovering or submitting a technique explains availability; counters expose accepted casts, and the orb shows remaining time versus Red's predicted arrival time.

**Demonstration** mode has individually labelled scenes for the complete kit. The Nuke and Maximum Red scenes first earn Black Flash through two clock-spaced punches. Actor retreat and sidestepping are visible scripted inputs; costs, gates, cooldowns, damage and the three-second orb remain real. Starting a new demonstration explicitly resets the scene. Ordinary Blue/Red, both maximums, Purple, returning Red, RCT, Spatial Step and both Domains are exposed manually. Allied stuns suppress scene attacks and movement.

Transport **v3** keeps numeric command meanings and adds curve action codes, accepted-cast counts, per-technique rejection reasons and committed paths. Optional presentation metadata is version 2. The current decoder refuses old transport versions rather than silently replaying them under new rules. `/gojo-v1.html` and `/gojo-v2.html` retain their original readers and WASM; the v2 art archive is editor-only and is not loaded by the current lab or installed as another mod atlas. Complete v3 input replay compares full state, commands, identities and presentation.

## Manga reference boundary and projectile feasibility

This implementation uses the [REA](https://github.com/morluto/rea) workflow and reuses the matching artifact identity documented in [native evidence](magnus-native-evidence.md). The installed 0.6.4 executable is PE32+ x86-64, SHA-256 `a1d05045405a12e2bed182daeafcad8c8a40cf29ab68ba6ea5bfd30257439582`. REA Rust identity evidence is `ev_f732eedfdff88cd390eb51531a70f3e08f4235ffe7cd7e2cc56f679bb1e2c17b`; that evidence identifies metadata, not private projectile behaviour.

[The read-only boundary report](gojo-results/v3-projectile-boundary.json) confirms the SDK's 720-byte simulation table at VA `0x143d53258` / file offset `0x3d51858`: projectile inspection is in slots 50–53; spawning is slot 66. Slot 55 changes entity position and is not a projectile redirect operation. No projectile direction/velocity setter is exposed. Private kinematics and a safe replay-compatible hook were not recovered. **Stock enemy-projectile bending is unsupported and disabled**, as approved when the user chose to ship the proven changes if no safe route was established. No visual-only bend, substitute damage reduction, private offset writes or executable patch is shipped. No new native REA session required cleanup in this implementation.

The official [VIZ chapter index](https://www.viz.com/shonenjump/chapters/jujutsu-kaisen) gates relevant chapters behind access requirements. This is not a claim to have read the whole manga. Chapter-linked summaries distinguish inspiration from custom rules:

| Reference | Supported inspiration / adaptation boundary |
|---|---|
| [Six Eyes](https://jujutsu-kaisen.fandom.com/wiki/Six_Eyes) | Innate perception and extreme CE efficiency. The 90% discount, cooldown acceleration, damage penalty and 250-point exhaustion model are mod balance. |
| [Blue](https://jujutsu-kaisen.fandom.com/wiki/Cursed_Technique_Lapse%3A_Blue) | Controlled attraction centres and movement assistance. The fixed curve choices and whirlpool are combat adaptations. Maximum output is not canonically locked behind Black Flash. |
| [Chapter 232](https://jujutsu-kaisen.fandom.com/wiki/Chapter_232) | Red strikes, returns around a building and attacks from behind before Black Flash. The fixed return geometry and unchanged split damage budget are adaptations. |
| [Chapter 235](https://jujutsu-kaisen.fandom.com/wiki/Chapter_235) | Persistent Blue and Red combine into explosive Purple; Blue diverts an incoming water attack. General stock-shot bending is not established by the mod API. Exact orb duration and 50%-current-HP self-cost are custom. |
| [Black Flash](https://jujutsu-kaisen.fandom.com/wiki/Black_Flash) | An exceptional timed physical/CE impact. Guaranteed two-punch triggering and the eight-second 20% buff are adaptations. |
| [Unlimited Void](https://jujutsu-kaisen.fandom.com/wiki/Unlimited_Void) | Information overload and brief 0.2-second exposure; contact with Gojo supplies immunity. The larger quick radius and fixed nearest allied guest are custom. |
| [Limitless](https://jujutsu-kaisen.fandom.com/wiki/Limitless), [RCT](https://jujutsu-kaisen.fandom.com/wiki/Reverse_Cursed_Technique) | Short-range movement and self-healing inform Spatial Step/RCT. No unrestricted map teleport or free brain-reset cooldown refresh is added. |

Simple Domain and Falling Blossom Emotion remain future defensive suggestions, not extra abilities in this update.

## Verification and delivery

Current results: [native/shared/migration/art/browser checks](gojo-results/verification.json), [complete parity](gojo-results/parity.json), [new combat/demonstration/reader checks](gojo-results/v3-browser.json), [geometry and frozen-v2 identity](gojo-results/v3-geometry.json), [phase windows](gojo-results/anime-presentation-verification.json), [lossless legacy frames and reproducibility](gojo-results/art-verification.json), [art storage](gojo-results/v2-art.json), [matched raw combat comparison](gojo-results/v3-comparison.json), [Gojo core timings](gojo-results/performance.json), [Magnus stress regression](gojo-results/magnus-stress.json), [packaged hashes](gojo-results/package.json), [isolated Mod Manager](gojo-results/mod-manager.json).

These are automated core/lab/package checks. Engine-attributed damage, items, native animation anchoring, projectile interception and crowded native rendering have not been observed. Live acceptance stays pending. No general freeze fix or unconditional balance buff is claimed.

Career migration remains a separate explicit action. Inspect a copy first with `rtk proxy python tools/migrate-champion-reworks.py PATH_TO_CAREER`; `--output NEW_COPY` writes a separate copy, while `--in-place` verifies a backup before replacement. That existing tool handles the supported champion reworks, so inspect its complete report. It refreshes stale Gojo visual bindings even when the passive ID is already current, preserves career statistics/items/history and is idempotent. Unknown controllers abort. Nothing here migrates a replay or previously created match snapshot.

Reproduce from the implementation checkout. Run the editor server in a separate terminal while executing the browser checks:

```text
rtk proxy python tools/generate-gojo-v2-rules.py
rtk proxy python tools/generate-gojo-v2-art.py
rtk proxy python tools/generate-gojo-champion.py
rtk proxy node tools/verify-gojo-parity.cjs
rtk proxy node editor/server.js --port 7281 --no-open
rtk proxy python tools/verify-gojo-release.py
rtk proxy node tools/test-gojo-v3.cjs
rtk proxy node tools/test-gojo-presentation.cjs
rtk proxy python tools/verify-gojo-presentation.py
rtk proxy python tools/verify-gojo-v3-geometry.py
rtk proxy node tools/check-gojo-domain-size.cjs
rtk proxy node tools/compare-gojo-v2.cjs
rtk proxy python tools/verify-gojo-magnus-regression.py
rtk proxy python tools/package-gojo.py
rtk proxy python tools/check-gojo-package-manager.py
```
