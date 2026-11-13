# Round 173: Kicker's Wall Bounce and toughness, Akane's damage nerf (native 0.10.107, tfm2_custom 0.2.110, `kicker-2.0`, `akane-1.6`)

**Reihan, after round 172:** "so I think the wall is too complicated for the AI". Of the options offered, Reihan chose the **Wall Bounce**:
- no standing on walls;
- S1 leaps to a wall within reach and springs straight off it onto an enemy or a point, with the Drop-in;
- a short hidden moment at the touch;
- "wall" versions of his skills right after.

**On the plan, Reihan added:**
- "buff the damage reduction, armor, and add a CC immunity, and nerf overall damage of Akane";
- "since he can't jump over all walls, remove the CD for ground-to-wall jumps".

## Why
Rounds 168-172 each patched the wall stance: the leash, the edge stop, the game's wish table, perch timers, idle drops, gank climbs, and wall escapes removed and then restored. Every fix added state for the AI to manage, and live play still found him stuck on walls or sitting on them. A bounce has no state to manage: he's on a wall for 0.25 s and the move finishes itself.

## Kicker `kicker-2.0`
- **S1 Wall Bounce (no cooldown with a wall in reach):**
  1. A leap to the best wall point within 80,000. That's one with the aim within the 100,000 spring, by the shortest way round; otherwise the nearest.
  2. A 0.25 s touch: hidden, with `ignore_wall` and 25% less damage.
  3. The spring onto the aim. On an enemy at the aim he lands behind it (from the wall's side) and kicks it for 60 + 80% Attack.
  - **Protection:** invulnerable to damage and CC immune from the cast to the landing.
  - **No wall in reach:** the short 20,000 hop (2 charges, 7 s).
- **Landing:**
  - the Drop-in (6 s cooldown);
  - the landing shield, 8% → **12%** max HP, every 10 s → **8 s**;
  - **Bounced for 3 s:** 25% less damage (was 15% on walls), CC immune for the first second, 45,000 rock basics, Rock Rain, and one Skyfall.
- **Armor and CC:**
  - Marked Dash shield 4% → **7%** max HP, 2 s → **2.5 s**;
  - Defence 24 + 4 → **30 + 5**;
  - **CC immune** (`kk_unstoppable`, `cc_immune`) during every leap and dash: the bounce, the short hop, Marked Dash, the kick's dash-in and Skyfall. A shorter window never cuts a longer one short.
- **Removed:**
  - the wall stance (`kk_roof` is only the touch now): the leash, the edge stop (`roof_walk` in the input layer), the game's-wish table (`note_want`/`want`), perch and idle timers, gank climbs, wall-to-wall hops, Drop Kick and the mount;
  - the +25% wall speed and the 40,000 hide.
- **AI:** one flat list with no wall state:
  1. death cancel and teamfight tag;
  2. escape (a Marked Dash away, else a bounce toward safety when the landing is safer);
  3. peel;
  4. the kick follow-up;
  5. help (a Marked Dash, else a bounce onto the enemy on the teammate, else a walk);
  6. the target. A kick if it's ready. Otherwise a **bounce onto it**: wall first, with the Drop-in ready or the target out of plain reach. Then, while Bounced, **Skyfall** or **Rock Rain**. Then Rock Throw, a Marked Dash, or a purposeful walk. A target fighting his team, low or alone is considered up to 180,000 away (the bounce's reach);
  7. farming with S2 only.
  - **Checks kept:** outnumbered, under its tower and V1's parry all still apply.

## Akane `akane-1.6` (about 15% less damage)
- **Base Attack:** 78 → 72.
- **Blood Rhythm:** +6% → +5% per stack.
- **Ambush:** +20% → +15%.
- **Kunai:** 30 + 60% → 25 + 50%; the slash: 40 + 80% → 35 + 70%.
- **Crimson Edge:**
  - bleed: 6 + 10% → 5 + 8%;
  - burst: 8% max HP + 40% → 6% + 35%;
  - crimson bleed: 1% → 0.8% max HP a second.
- **Crimson Night:** 100 + 90% → 85 + 75%.
- **Unchanged:** cooldowns, ranges, CC and the AI.
- **Test adjustments:**
  - Ego test: its target sat just short of lethal for her old damage, so its HP moves from 700 to 650.
  - Lab kunai test: it now reads the damage from the data.

## Verification
- **Native tests:** `cargo test --release` gives **1,277 passed**, 0 warnings. That includes **60 Kicker tests**, after removing the 24 wall-stance tests and adding the bounce tests:
  - the touch length and hide;
  - landing behind the target with the Drop-in;
  - no charge used;
  - back-to-back bounces;
  - Rock Rain and one Skyfall while Bounced;
  - CC immunity and armor on every leap and dash;
  - the AI's wall-first engage then Skyfall, no bounce into a crowd or under a tower, the bounce over a wall, the escape bounce, the help bounce;
  - the bridge's touch, Bounced and CC-immune buffs.
- **Bench:** about 6 µs per tick.
- **Other checks:**
  - `kicker_lab` (3) and `akane_lab` (9) pass, and both wasm files are rebuilt;
  - the generators are current;
  - the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports **0.10.107**.
- **Lab page:** Q is the Wall Bounce, and C is the kick (the core makes it a Skyfall while Bounced).
- **Not run here:** `tools/verify-kicker-art.py` compares against Codex's art baseline, which already predates rounds 169-172.

## Live acceptance (pending)
- No more stuck-on-wall states.
- He engages off walls, then Skyfalls or rains rocks.
- He escapes by bouncing toward his team.
- Is the tankiness right, with no cooldown on the bounce?
- Akane still threatens kills, just slower.
