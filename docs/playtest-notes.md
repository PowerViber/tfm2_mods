# Skill Lab playtest notes

## Confirmed engine behaviour
- **SwitchByBuff on caster buffs works.** Confirmed Sep 30: Hollow Purple fires when Red is cast while the `gojo_blue` flag is up.
- **Projectile end_effects carry the landing / stop spot.** Confirmed Sep 30: Minato's seal (ViewEffect) appeared where the kunai landed, and his Delayed → Teleport went back to that spot.
- **A ParabolicProjectile cast with Targeting follows a moving target.** Rian saw the kunai "home" once. Use a LinearProjectile when the landing spot should be fixed at the throw.
- **The AI casts every skill the moment its real cooldown is ready**, in slot order: skill (Ability 1) before skill2. With equal cooldowns, Ability 1 always comes first, so "Ability 2 then Ability 1" combos never happen. Offset the cooldowns to get both orders.
- **WithSelf does NOT redirect CC to the caster.** Gojo's `WithSelf → Stun` (cast lock) stunned the enemy target instead. Assume every effect inside WithSelf still hits the action's target.
  - Use `RangeEffect AroundCaster, target AllyOnlySelf, radius 1000` for "apply to myself" (Shield, Stun, ShrinkingBarrier). This replacement is itself unconfirmed.
- Every cast plays its animation, even when the effect is an empty branch. Gate normal cooldowns with the real `cooltime`.
- **Buff-check lockouts on top of a real cooldown are predictable:** a locked cast is empty and restarts the real cooldown. So a lock that ends between the Nth and (N+1)th natural recharge gives exactly N+1 × the cooldown.
- **Multi-charge skills without empty casts:** use a real cooltime equal to the gap between charges, and stage buffs that pick what the next cast does. A lapsed stage just falls through to stage 1.
- **AI targeting as a condition:** the action's `casting_target` decides when the AI will cast. `EnemyChampionInCC` + a huge `range` means "only cast at a crowd-controlled enemy champion anywhere on the map". Slows are buffs, not CC, so they don't count.
- **Not available to data effects:**
  - kill detection (no on-kill trigger, no target HP check)
  - checking or removing buffs on *other* units: RemoveCasterBuff only affects the caster, and SwitchByBuff only reads the caster
- Careers keep the copy of each mod champion made when the career was started. Mod edits reach exhibition matches and new careers only.

## Sep 30, round 1 (Minato, Gojo)
- **Observed:** Minato spammed kunais, his Rasengan charges stacked, and Gojo's Blue/Red were spammable.
- **Cause:** fake cooldowns (60-tick real cooltime + `SwitchByBuff "<x>_cd"`).
- **Fix:**
  - **Minato, Flying Raijin:** one cast = throw → RushMoveToBack strike → teleport back. Real cooldown 720.
  - **Minato, Rasengan:** real cooltime = charge 360 + 480.
  - **Minato, KCM:** +50 skill_cooldown_mult.
  - **Gojo, ult:** real cooltime 7200 slow / 5400 fast / 6300 auto.

## Sep 30, round 2 (Gojo)
- **Observed:** Blue → Red instantly became Purple every time (Red→Blue never happened), the domain cast stunned the enemy, and players could walk into the domain.
- **Fix:**
  - Blue real cooldown 960 (flag 300 + 660), Red 720 (300 + 420).
  - Combo freshness window (first flag must be a minimum age).
  - Purple radius 26000, speed 4200, sprite 72 px.
  - Self-effects moved to the AllyOnlySelf RangeEffect pattern.
  - Domain wall: ShrinkingBarrier (radius + 4000, shrink 0, edge 5000, Knockback 2600/8).
- **Not possible:** hiding the inside of the domain from outsiders.

## Sep 30, round 3 (Gojo)
- **Done:**
  - skill2 (Red/Purple) targets `EnemyChampionInCC` at range 400000. Blue's Pull counts as CC.
  - comboWindow 30 ticks.
  - Combo locks `gojo_blue_lock` / `gojo_red_lock`:
    - after Purple: blue 2010 ticks, red 1800
    - after Red→Blue: blue 2400, red 1620
  - This gives exactly 3× on both skills.
- **Not done:** "2× if Purple kills" — kills can't be detected.

## Sep 30, round 4 (Minato)
- Dropped: moving the Hiraishin mark onto enemies via basic attacks. It needs target-side buff checks/removal and "target a marked enemy" AI selection.
- The kunai became a LinearProjectile (penetrate false, speed 5000, applied_target EnemyChampion, so it flies past minions). It stops at the first enemy champion hit, otherwise at full range.
- Workspace presets.js re-synced from C:\Games\tfm2\editor\presets.js. An unfinished Position-cast draft was discarded.
- Tried but never tested in game: "one cast: throw → teleport onto kunai → slash" with range 70000.

## Sep 30, round 5 (Minato): 3 charges again, done without fake cooldowns
- **Flying Raijin now:**
  - Real cooltime 240 (4 s) between casts, so there are never empty casts.
  - Stage buffs: `raijin_2` (480, cast 2 must come within 8 s of the throw) and `raijin_3` (240).
    1. Cast 1: LinearProjectile kunai (range 70000); end_effects = `kunai_seal` flash + 48 sibling Delayed checks every 15 ticks for 720 (12 s).
    2. Cast 2 sets `tp_now` (30 ticks); the next check teleports him to the kunai spot.
    3. Cast 3 sets `tp_now` + `tp_last`; after that teleport the check adds `kunai_gone`, so the seal stops.
  - The seal is redrawn every 30 ticks (`kunai_planted`, 0.5 s pulse) while not gone.
  - Arrival (each teleport): flash, Teleport, flash, hiraishin 720, RangeEffect 18000 → Fear 6 ticks (0.1 s). No damage slash.
- **Known edge case:** if stage 2 lapses (no enemy in range for 8 s), the old kunai's checks keep running until 12 s. A new throw in that window can leave two seals, and a teleport request may be answered by the old kunai.
- Rian saw the ORIGINAL 3-charge version skip teleports: stages timed out while no enemy was in range, and its fake cooldowns caused empty casts.

## Sep 30, round 6 (Minato, Gojo)
- **Minato:**
  - Rasengan stun window after a teleport is now 2 s (primeTicks 120).
  - Fear is on the 1st teleport only.
  - Look changed by Rian in the Skill Lab to Lightning Mage (tags attack/skill1/skill2/ult); now the preset default.
- **Minato cooldown-speed EXPERIMENT** (resolved later, see "cooldown-speed leak"):
  - Real cooltime 360; the throw adds `raijin_haste` (skill_cooldown_mult +700) and cast 2 removes it.
  - Expected under "recovery rate" semantics: teleport ready ~0.75 s after the throw; the return and the next throw each 6 s.
  - If skill_cooldown_mult is instead applied when a cooldown starts, the first gap stays 6 s. If it does nothing, all gaps are 6 s.
  - Windows: raijin_2 480, raijin_3 660, kunai 1200 (80 checks).
- **Gojo:**
  - Look changed by Rian to Dark Mage (skill → skill1).
  - Purple now needs `gojo_blue_orb`: the Blue orb hovers 60 ticks after landing before becoming the black hole, and Purple uses it up (no black hole).
  - Blue/Red flags now power ONE basic attack, then they're removed. Red→Blue uses its own `gojo_red_combo` (5 s).
- **Careers keep old copies:**
  - The save stores each mod champion's JSON (without view_* keys) at career creation.
  - The game reads mods only at startup, so a career made right after a mod edit, without restarting, gets the previous version.
  - Rian's saves save_20260930_023746 (Monk Gojo, old) and save_20260930_052956 (Dark Mage Gojo, 0.1.19) both hold old copies. Offered to replace them (needs the game closed).
- Old seal visual name `kunai_marker` is re-registered, because older career copies still use it.

## Sep 30: Skill Lab overwrite incident
- At 06:05 Rian clicked **Save mod** in a Skill Lab page opened earlier. It wrote back the page's in-memory Gojo, undoing the later updates (cooldown buff, one-pulse Blue, domain fix); cooldowns went back to 16 / 12. The backup in editor\backups\mods\tfm2_custom.20260930-060531 held the lost version.
- **Fixed:**
  - Gojo was redeployed (mod 0.1.28).
  - skills.js now remembers the mod version it loaded (`L.diskVersion`). Before saving, it re-reads /api/mods; if the version on disk changed, it asks whether to reload instead of overwriting.
  - Opening the Skill Lab tab reloads from disk when there are no unsaved edits (`TFM2Skills.show`).
- **Process rule:** when Claude updates mods\tfm2_custom directly, always bump mod_info.version, which is what the Skill Lab check compares.

## Sep 30: native AI mod `tfm2_custom_ai` (built, not yet tested in game)
- **Why:** the data format has only 4 actions (attack, skill, skill2, ult), and the built-in AI casts them the moment they're ready. A stable-ABI native mod (`StablePlayerAi`) now decides when Gojo and Minato cast.
- **Source:** C:\Games\tfm2\native (`tfm2_custom_ai/src/lib.rs`, with the SDK copy `mod-api-stable` next to it). `build.bat` runs `cargo build --release` and copies the DLL and mod.mod_info to `mods\tfm2_custom_ai`. Compiles cleanly on Linux; the Windows DLL has to be built on Rian's PC because the Windows Rust target can't be downloaded in the cloud.
- **How it works:**
  - Each tick it reads the sim (own buffs and cooldowns; visible, targetable enemies; allies).
  - It returns None to keep the built-in input, or replaces it: a retarget, a deliberate cast, or a "hold", which turns the skill into a basic attack.
  - It attaches to players whose champion id ends with gojo or minato.
- **Gojo:**
  - Purple when `gojo_blue_orb` is up and Red is ready, aimed through the most enemies.
  - Red→Blue when `gojo_red_combo` is up and `gojo_red_new` is not.
  - Domain only when it catches 2+ enemies and more enemies than allies, or one enemy at ≤40% HP with no allies inside.
  - Plain Red at a CC'd enemy, at the nearest enemy when Blue is ≤60 ticks from ready, or at an enemy at ≤25% HP; otherwise held.
  - Blue retargets to the densest enemy group.
- **Minato:**
  - Stage 2 (`raijin_2`) casts at once.
  - Stage 3 (`raijin_3`) is saved for HP ≤45%, 3+ enemies within 30000, or ≤45 ticks left.
  - The throw targets the lowest-HP enemy in range.
  - Rasengan is cast during `hiraishin` with an enemy within 30000, and held when nobody is near.
  - KCM needs 2+ enemies within 50000, or himself ≤60% HP, or an enemy ≤50% HP.
- **Follow-up once the DLL is confirmed loaded:**
  - Switch Gojo's skill2 `casting_target` from EnemyChampionInCC to EnemyChampion, so the AI's non-CC Red and Purple shots pass `is_valid_input`. Until then, Red and Purple only go at CC'd enemies (Blue's pull counts).
  - Check log.log for "tfm2_custom_ai loaded".
- The data mod on disk was at 0.1.33 (Rian's Skill Lab edits), with cooltimes Blue 780, Red 600, and ult 5250.

## Sep 30: Gojo stopped loading (fixed in mod 0.1.34)
- **Symptom:** log.log said `data_champion load error: invalid value: integer -60000, expected usize`. Gojo was missing from the new career save_20260930_072941.
- **Cause:** the Void Overload buff had `range: -60000`, and the game only accepts non-negative values for `range`.
- **Fix:** removed that field (8 copies in the mod file, plus presets.js on the PC and in the workspace). Backup: editor\backups\mods\tfm2_custom.20260930-003132.
- **Rule:** in BuffState, only these are signed (i32): attack, magic_power, defence, hp, hp_regen, magic_resistance, vamp, crit_chance and every *_mult. These are unsigned (usize) and must never be negative: range, toughness, heal_reduce, damage_reflect, damaged_amplify/reduce, penetrations, dot_amplify, the max_hp_damage fields, base_attack/skill_damaged_reduce.
- **After any mod edit, check log.log for `load error`.** A champion that fails to load is silently skipped.

## Sep 30: "no fights can be simulated" (under investigation)
- **Symptom:** log.log shows panics in the game's own team AI: `called Option::unwrap() on a None value` at game-ai plan_legacy sub_plan line_defense.rs:968, jungle.rs:243/245 and battle.rs:368, on parallel sim threads.
- **First time both new pieces ran:** the native AI mod and Gojo's Void Overload (never loaded before, because of the range bug).
- **Leading guess:** Void Overload had move_speed_mult −100 and skill/ult_cooldown_mult −100. Zero move speed or zero cooldown recovery can give None from a checked division in the AI's time estimates. Base game data never goes below −35.
- **Change:** all four set to −90 (attack_speed_mult −99 → −90 too). Mod 0.1.36, backup editor\backups\mods\tfm2_custom.20260930-003745. A Skill Lab save at 07:32 had made 0.1.35.
- **If it still panics:** disable the native AI mod to isolate it. Its matches() only attaches to Gojo/Minato players.
- **Result:** still panicked at −90 (AI log line confirmed, 07:39), and Rian pinned it on Gojo. Void Overload was removed entirely (mod 0.1.37, backup tfm2_custom.20260930-004221). Gojo data is now identical to 0.1.28, the last version that played fine. If it still panics, the Gojo part of the native AI is next.
- **07:47:** still panicked (line_defense.rs:968) with Gojo data = 0.1.28, so the native AI is the prime suspect. The only thing unique to Gojo's AI was creating or swapping skill2 inputs: Red/Purple is a Direction cast whose casting_target is EnemyChampionInCC, and `InputTargetV1::dir` leaves target_id 0. AI 0.1.1 never touches skill2 and leaves it to the built-in AI (returns None). Next step if it still panics: disable tfm2_custom_ai to confirm.
- **08:15:** Rian says Gojo is still the problem (AI 0.1.1 DLL built 07:58) and Minato is fine. Gojo's only AI-relevant setting not used anywhere in the base game is skill2 `casting_target: EnemyChampionInCC` (range 400000). The game's legacy planner probably unwraps a target lookup for it. Changed it to EnemyChampion with range 70000 (Red's real flight range). Gojo now lives in folder tfm2_jjk (0.1.2), currently switched off (.off). The preset default is now redTargetsCC false. Built-in AI now fires Red whenever it's ready.
- **08:20:** still 10 planner panics (line_defense 968, jungle 243, battle 368) with no InCC left. Rian: Gojo sometimes 'casts' (cooldown spent, nothing fires) and then gets stuck, only in Gojo vs Minato, the only pairing where both native AIs are active. Leading theory: the native AI injects casts (and holds) that the game's planner didn't start. The planner's next input cancels the cast (cooldown spent) and its plan state no longer matches, so it unwraps None and the player freezes. Asked Rian to disable tfm2_custom_ai to confirm. If confirmed, any future AI may only retarget the planner's own casts: no injecting, no holding.
- **CONFIRMED (08:30):** with tfm2_custom_ai off, the game simulates fine. **Rule: a StablePlayerAi must never inject or hold casts.** The legacy planner (plan_legacy line_defense/jungle/battle) breaks and freezes the player. The Void Overload and EnemyChampionInCC removals were misdiagnoses, so both were restored: tfm2_jjk 0.1.4 = the 0.1.36 Gojo with the −90 overload (no range stat) and Red at InCC/400000, with asset paths pointing at tfm2_jjk. The preset was restored the same way.

## Skill Lab folders and on/off switches (Sep 30)
- **Folders:** every editor-made data mod in `mods\` shows up as a folder in the Skill Lab sidebar. The editor-made ones are those with author "TFM2 Database Editor", not native, and without contains_code. Each folder is its own mod, so it can be switched on/off separately.
  - "+ New folder" creates `tfm2_<name>`. It is written on the first Save.
  - "Save" writes every changed folder, and each save bumps that folder's version.
- **Champion switch:** the tick on a champion row, or the On/Off chip in its header. Switching a champion off saves its file as `champion/<id>.data_champion.off`, which the game doesn't load (assumed from the extension; unconfirmed in game). Careers keep their own copies.
- **Folder switch:** the tick in the folder header. It edits `config\game\mods.json` enabled_mods (backup: mods.json.bak) through `POST /api/mod-enabled`, and the server refuses while the game is running.
- **Move to folder:**
  - Rewrites `asset/<old>/` → `asset/<new>/` in the champion JSON.
  - Copies its VFX sheets (via `GET /api/mod-file`) into the new folder and removes the file from the old one.
  - The champion id stays the same (e.g. tfm2_custom_gojo).
- A champion id can live in only one folder: reworks and presets check every folder.
- Tested end to end in a copy of Rian's mod (moved Gojo into a new folder with its VFX, switched Minato off, enabled the folder).

## Sep 30: Gojo v2, the flag kit (tfm2_jjk 0.1.5)
- **Rian's design:**
  - Blue / Red casts only raise a flag.
  - The next basic attack fires that skill: the Blue orb (→ black hole 1/3 s after it lands) or the Red blast. The flag is then used up.
  - Both flags up → the next basic attack is Hollow Purple: 34-tick (0.56 s) wind-up with a self-stun and the purple_charge visual (caster, follows, tag "purple"), then the Purple shot + MoveBack recoil 3000×12.
  - Blue and Purple can no longer coexist.
- **Flag casts:** Targeting EnemyChampion, range 30000, flag 150 ticks. Real cooltime = 150 + blueCd 480 / redCd 300 = 630 / 450.
- **After Purple:** gojo_purple_strain gives skill_cooldown_mult −50 for 1260 ticks, which is exactly 2× on both cooldowns. No lock buffs, so no empty casts.
- **Dropped:** Red→Blue teleport combo, orb-hover Purple, combo locks, mini-shots, EnemyChampionInCC targeting.
- **Unchanged:** Infinity and the domain with the −90 overload.
- **"If it's worth it":** there is no custom AI (see the player-AI rule above). Purple happens when both flags are up together, i.e. when the game's AI finds both skills ready at once and casts them back to back before the next swing.
- **Limit:** the flagged attack fires at whatever the attack targets (the basic attack casting_target is Enemy, so it can be a minion).
- **To verify in game:**
  - LinearProjectile / MoveBack inside a Delayed on a basic attack;
  - the self-stun during the wind-up;
  - whether the AI really casts both flags before the next swing.

## Sep 30: tweaks (tfm2_jjk 0.1.6, tfm2_custom 0.1.41)
- **Blue:** the orb collapses into the black hole the instant it lands (no hover, no blue_orb effect).
- **Red:**
  - The path hit is 40 + 50% AP + push.
  - It is penetrating, so it always reaches max range (70000). There it explodes: a one-pulse RangePeriodProjectile of radius 20000 dealing 60 + 70% AP with a 3000×8 knockback, plus the new `red_burst` VFX (gen.py, 60 px).
- **Cooldowns:** Blue 10 s and Red 8 s after the flag (real cooltime 750 / 630). Domain 60 s for slow, fast and auto. Minato KCM 60 s (was 50; preset option kcmCd).
- Only Minato's ult cooltime and ult tooltip were changed in his deployed file, so his other data was left as it was.

## Sep 30: domain rules via a native match hook (tfm2_custom_ai 0.2.0 = "Gojo domain rules", tfm2_jjk 0.1.7)
- The player AI was removed from the DLL. The old source is kept at /home/claude/native/player_ai_v0.1.1.rs.bak (cloud only).
- **Data side:**
  - At domain open, Gojo gets `gojo_void_active` (for the stun duration) and `gojo_void_guard` (base_attack_damaged_reduce 100, skill_damaged_reduce 100, cc_immune).
  - domainHitsAllies is now false, so the data zone only affects enemies (stun + −90 overload).
- **Native `StableMatchHook::on_match_tick`** (every tick, every match), for each champion with gojo_void_active:
  - **First tick:** stores the centre in the buff name `void_c<gid>:<x>:<y>`. Every champion inside gets `void_trapped<gid>`, except the closest allied champion, who gets `void_guest<gid>` + `void_guard<gid>` (damage reduction 100/100, cc_immune).
  - **Each tick:**
    - trapped and not stunned → strip cc_immune (remove the buff and re-add it with cc_immune false), then apply_cc stun 12;
    - inside but neither trapped nor guest (an intruder) → strip immunity and apply ForceMove outward at 4000 until R + 3000.
- **Projectiles:** there's no API to delete projectiles. "Blocking" means Gojo and the guest take no damage and no CC while the domain is open.
- **Untested:** the match hook in career sims; ForceMove dx/dy normalisation; apply_cc after the strip; buff-name centre encoding (64-char cap). Rian must run build.bat and enable the mod.

## Sep 30: Void seals from the press; Minato kunai buff (tfm2_jjk 0.1.8, tfm2_custom 0.1.42, native 0.2.1)
- Rian saw KCM Minato moving in the domain. The native 0.2 had loaded fine on game 0.6.2 with no panics. Most likely he moved during the conjuring time, when nobody was locked yet.
- **Domain:**
  - Everything now starts on the press. Enemies inside get the seal for cast + stun: Stun + Bind + BlockAttack + BlockSkill + BlockMoveSkill (the Taoist's talismans combined) + overload, with move −100 again and the rest −90.
  - The zone re-seals every 10 ticks for the same total, and gojo_void_active / gojo_void_guard last the same total.
  - Gojo's own lock is now selfLock: slow 30, fast 15 (was the whole cast: 150 / 45).
  - The full domain visual + Gojo's power buff come at `cast`.
  - Sealed from the press: slow 6.5 s, fast 2.75 s.
- **Native 0.2.1:** trapped champions get the same five CC kinds (12 ticks) after the immunity strip. mod.mod_info in the game folder was still 0.1.0 (build.bat's copy didn't update it) and was replaced by hand.
- **Minato:** kunai throw speed 7500 and range 110000 (was 5000 / 70000), and skill cast range 110000. The edit was targeted in his deployed file. Preset defaults were updated too.

## Sep 30: domain follow-ups (tfm2_jjk 0.1.9, tfm2_custom 0.1.43, native 0.2.2)
- **Gojo:**
  - Flag casts now have range 70000 (was 30000).
  - While gojo_void_active is up, Blue/Red skip the flag and fire straight at the target (sealed enemies). With the other flag up, the cast is Purple.
  - Purple now also knocks back everyone hit (4500×10).
  - The domain buff `void` gets +150 skill_cooldown_mult, plus slow 50/50/25/25 or fast 25/25/40/40 (damage / speed).
- **Native 0.2.2:**
  - The guest gets a copy of Gojo's `void` buff (`void_ally<gid>`).
  - Trapped champions get `void_anchor<gid>`, a move_speed_mult that brings their total to exactly −100 whatever speed buffs they have. It is a 30-tick buff, re-added when its value changes or it expires.
  - The seal is re-applied every 10 ticks even while they're stunned.
- **Why KCM Minato could walk:**
  - Speed can't beat a stun, so if he walked, no CC was on him: the seal missed (e.g. his 2 s kcm_burst immunity at the press) or he wasn't trapped.
  - The only thing left was Void Overload: −100 + KCM's +30 = −70%, i.e. a slow walk.
  - The anchor removes that, and the native strip handles the immunity.
- **Minato:**
  - Kunai speed 9000 / range 160000, and skill range 160000.
  - The seal VFX (kunai_seal / planted / marker) is now blue (gen.py B1/B2).

## Sep 30: growing seal + Purple-first in the domain (tfm2_jjk 0.1.10, native 0.2.3)
- **The seal spreads with the ring:**
  - At the press, one-pulse RangePeriodProjectile zones are spawned via the self pattern (centred on the press spot), one every growStep = 5 ticks. The radius grows from growStart = 8000 to domainRadius at `cast`, and each zone seals for total − t (39 zones across slow + fast).
  - The full re-seal zone (period 10) now starts at `cast`.
  - Gojo carries `gojo_void_conj<cast>` while conjuring.
- **Native 0.2.3:**
  - Its edge grows the same way (GROW_START 8000 → 60000 over the conj buff): anyone the edge reaches is trapped and sealed.
  - The guest is chosen at the press among teammates in the full radius.
  - Intruders are only pushed out after the domain is conjured.
- **Purple-first in the domain:** a cast with the other flag up → Purple. Otherwise the flag goes up and a Delayed 30 fires the skill by itself if the flag is still unused.
- **Target choice:** "focus the sealed enemies" is still the game AI's pick within range 70000. There's no safe way to steer it (see the player-AI rule), but inside the domain nearly everything in reach is sealed.

## Sep 30: domain wall (native 0.2.4, tfm2_jjk 0.1.11 tooltip only)
- Once conjured, trapped champions who end up past R − 2000 (up to R + 20000, so knockback overshoot is caught but a respawn at base is ignored) are entity_set_pos'd back onto R − 2000 along their line from the centre every tick. They look like they hit a wall.
- Purple / Red knockbacks can no longer push trapped enemies out. The guest can still leave freely.

## Sep 30: Blue/Red conjure time (tfm2_jjk 0.1.12)
- Rian: Gojo cast Blue/Red without stopping.
- The flag actions had duration 12 / start_timing 6, a 0.2 s cast that is invisible in play. They now use conjureTicks 36 / conjureAt 24: he stands for 0.6 s and the flag (or in-domain shot) comes at 0.4 s. can_use_with_move stays false.
- domainPurpleWait is now 45, because the 2nd cast's effect lands 36 ticks after the 1st.

## Sep 30: wall bug (native 0.2.5)
- **Rian saw:** an enemy pushed, pulled in, pushed, and then out of the domain.
- **Cause:** the native edge only reached the full radius on the conjured tick, and 0.2.4 treated "conjured and not trapped" as an intruder that same tick. So enemies in the outer band were ForceMoved out; Blue's black hole pulled them back in; then they were pushed out again.
- **Fix:**
  - On the first conjured tick (marker `void_sealed<gid>` on Gojo), everyone inside R is trapped. Only after that are non-trapped champions inside treated as intruders.
  - Wall hits now also call entity_clear_cc (cancelling the rest of the knockback) and immediately re-apply the seal.

## Sep 30: Minato teleported to a used-up kunai (tfm2_custom 0.1.44)
- **Rian saw:** throw → teleport → (return / mark gone) → ult → new throw → teleport back to the OLD, vanished kunai.
- **Cause:** the old checks were gated only on a shared `kunai_gone` flag, which every new throw cleared. The used-up kunai's Delayed checks were still scheduled (kunaiLife 20 s), so they revived and could answer tp_now before the new kunai did.
- **Fix:** 3 rotating kunai slots.
  - Each throw removes every `kunai_live1..3` / `kunai_last1..3` and sets `kunai_live<n>` (life) + `kunai_last<n>` (marks the slot for rotation).
  - A kunai's checks require its own `kunai_live<n>`, and the 3rd cast removes it.
  - A slot is reused only after 3 throws (≥ ~25 s even with KCM haste), which is past kunaiLife.
- checkEvery is now 20 (was 15), so 3 × 60 checks; the file is 1.1 MB pretty-printed / 257 KB compact.
- Kunai range 120000 (the Ninja's skill range), speed 9000, skill cast range 120000.

## Sep 30: balance pass: Gojo up, Minato basic attacks down (tfm2_jjk 0.1.13, tfm2_custom 0.1.45)
- **Why Minato beat Gojo** (level 9, no items):
  - Basic attacks: Minato attack 339 (115 + 28/lvl) vs Gojo 126 (70 + 7/lvl), so ~400 vs ~135 dmg/s.
  - Gojo's skills only fire through melee basic attacks, and his flags lapsed after 2.5 s.
  - Red exploded at max range, 50000 behind a melee target.
  - Infinity only blocked skills, while Minato's damage is basic attacks (incl. Rasengan).
- **Gojo:**
  - Basic attacks: + ApAttack 0 + 60% AP.
  - Magic power growth 18 → 25 (base mages max at 20).
  - Flags last 5 s. blueCd 450 / redCd 330 keep the real cooltimes at 750 / 630.
  - Red: penetrate false, so it stops at the first enemy hit and explodes there (path hit 40 + 50%, burst 60 + 70%).
  - Infinity: + base_attack_damaged_reduce 30.
- **Minato:** attack 95 + 20/lvl (was 115 + 28). Base assassins are 120 + 30, median 85 + 15.
- On-disk files matched the previous deploys (no Skill Lab edits), so both were rebuilt from the preset.

## Sep 30: Gojo damage trim (tfm2_jjk 0.1.14)
- Rian: "he hurts too much now". Estimate at level 9 without items: ~380 dps + Purple ~830 + domain +50% dmg, vs Minato ~350.
- **Trimmed:**
  - Basic-attack AP ratio 60 → 35.
  - Magic power growth 25 → 20 (the mage norm).
  - Red burst 60 + 70% → 40 + 50% (it now lands every time).
  - Purple 260 + 220% → 200 + 160%.
  - Domain buff: slow 50 → 30% damage, fast 25 → 15%, cooldown speed 150 → 100% (the guest copy follows automatically).
- New estimate: ~280 dps + Purple ~550.
- **Unchanged:** flags 5 s, Red exploding on hit, Infinity 30% basic-attack block.

## Sep 30: Gojo growth tweak (tfm2_jjk 0.1.15)
- Rian asked for "higher ap/lvl, lower magic/lvl". Read as magic power growth 20 → 23 and magic resistance growth 4 → 2.
- For reference, base mages are MP 40 + 20/lvl and MR 20 + 3/lvl.
- About +8% damage at level 9. Against AD champions like Minato the lower MR costs nothing; it only matters against mages.

## Sep 30: Gojo growth shift + Purple lockout (tfm2_jjk 0.1.16)
- Rian meant AP = attack power. Now attack growth 7 → 14 and magic power growth 23 → 14 (magic resistance stays 2).
- At level 9: attack 182 (was 126), MP 172 (was 244). Basic attacks ~same (+5%); Purple 590 → 475; Blue/Red ~−25%.
- **Purple spam:** the domain buff's +100% cooldown speed cancelled the −50% strain, and 5 s flags raised from 70000 leave time for both flags to be up.
- **New `gojo_purple_rest` (20 s):**
  - both flags + rest → the swing fires Blue and keeps the Red flag for the next swing;
  - in the domain, the other flag + rest → direct shot instead of Purple.

## Sep 30: domain visuals anchored to the press spot (tfm2_jjk 0.1.17)
- The rings, seal zones and native centre were already fixed at the press. Two visuals were not:
  - `void_cast_*` was bound with is_follow true;
  - the finished-domain picture (`void_slow` / `void_fast`) was a CasterViewEffect inside Delayed(cast), so it drew wherever Gojo was by then (his lock is only 0.25–0.5 s).
- **Fix:**
  - void_cast is now is_follow false.
  - A self-spawned marker zone `void_anchor` (life = cast, never pulses) plays fx(void_<tag>) in its end_effects, at the press spot. This relies on end_effects carrying the spot, confirmed for projectiles and assumed for RangePeriodProjectile.
  - Gojo's `void` buff is still applied at cast.

## Sep 30: flags auto-fire; Purple gated on the ult (tfm2_jjk 0.1.18, native 0.2.6)
- **Rian saw:** the AI running (low HP, chased) with both flags up, and Purple at level 2.
- **Flags:** raising one now schedules Delayed(flagAutoFire = 75 ticks) → if the flag is still up, fire that skill at the cast target (domain wait is still 45). A cast with the other flag up → purpleOr(own shot).
- **Purple:** `purpleOr(x) = ifBuff(gojo_ult_learned, ifBuff(gojo_purple_rest, x, purple), x)`.
- **`gojo_ult_learned` is set by:**
  1. the native passive `tfm2_custom_ai:ult_learned`, attached via `passive_ult` (the game activates it once the ult is learned). It re-adds a permanent buff on spawn/update. Uses add_native_passive (game ≥ 0.6.0 full); unverified in game.
  2. the ult itself as a data fallback (a permanent AddCasterBuff on the first cast; lost on death if buffs are cleared).
- Without the native mod: a warning in the log, the passive is skipped, and Purple is only available after his first domain.
- The game data doesn't expose which level unlocks the ult (SwitchByLevel3 hints at level 3).

## Sep 30: Gojo missing again: load error (fixed in tfm2_jjk 0.1.19); Minato buff (tfm2_custom 0.1.46)
- **Log:** `data_champion load error: missing field effect` (line 6236 = the ult's effect).
- **Cause:** the 0.1.17 `void_anchor` RangePeriodProjectile had raw end_effects.
  - **Rule:** RangePeriodProjectile.end_effects use `{effect, casting_type}` entries, and they apply to the units inside the area, not the spot.
  - Gojo failed to load from 0.1.17 to 0.1.18.
- **Fix:** the anchor is now a tiny LinearProjectile (range 1000) from the ult. Its end_effects hold Delayed(cast − 1) → fx(void_<tag>), keeping the landing spot the way Minato's kunai does.
- skills.js validate() now flags raw RangePeriodProjectile end_effects and raw applied_effects (except RushMoveToBack).
- **Minato:** attack 105 + 24/lvl (was 95 + 20), 297 at level 9. Rian feels the gap is mostly AI skill.
- **Careers** made while Gojo failed to load don't contain him.

## Sep 30: HP pass (tfm2_jjk 0.1.20, tfm2_custom 0.1.47)
- The 14:21 log has no load error, so the 0.1.19 fix loads.
- Gojo base HP 1000 → 850 (Infinity is his shield); growth stays 85, so 1530 at level 9.
- Minato HP 880 + 80 → 950 + 90, so 1670 at level 9 (assassin median 1540).

## Sep 30: HP follow-up (tfm2_jjk 0.1.21, tfm2_custom 0.1.48)
- **Gojo:** 850 + 95/lvl HP (1610 at level 9), armor growth 5 → 6 (73 at level 9). He fights in melee, where base melee is 1000 + 100 HP and 30 + 9 armor, and Infinity breaks to basic attacks.
- **Minato:** HP growth 90 → 85 (1630 at level 9, vs assassins 1540). A bit above the median to offset his lower attack.
- **Still open:** suggested but not applied: Infinity's 100% skill block → 60–70%, and the domain seal 6.5 s → 4 s.

## Sep 30: domain freeze + hard wall (native 0.2.7)
- **Rian saw:** only one champion stunned; the others walked freely and got "sucked in" / jittered at the edge.
- **Cause (0.2.5–0.2.6):**
  - The wall clamp for trapped champions ran whenever dist > R − 2000. After a set_pos to exactly R − 2000, rounding re-triggered it about every other tick, and each clamp called entity_clear_cc, wiping the seal. So they walked, hit the wall, got snapped back, and repeated.
  - Intruders were ForceMoved, which read as pushes.
- **Fix:**
  - No entity_clear_cc anywhere.
  - Trapped champions are frozen at `void_p<gid>:<x>:<y>` (their catch spot, moved in to R − 2000 if needed). If they drift more than 400 (and less than 120000, so a respawn at base is ignored), they are set_pos'd back. The five-CC seal is re-applied when not stunned or every 10 ticks; the anchor is kept.
  - Outsiders within R + 1500 after conjure are set_pos'd onto R + 1500 along their line: a hard wall.
- Proven primitives used: set_pos (the clamp visibly worked) and apply_cc (ForceMove pushes visibly worked).
- Possibly also relevant: the data seal zones spawned via self() might be centred on the ult target rather than Gojo, which would explain "only one stunned". The native freeze no longer depends on them.

## Sep 30: Flying Raijin moved into the native mod (native 0.3.0 "Gojo & Minato rules", tfm2_custom 0.1.49)
- **Rian's design:**
  - a 3-kunai fan;
  - the middle kunai homes on the target with a lifespan: it drops where it hits, or where its range runs out;
  - the side kunai are straight teleport spots;
  - smart teleport choice;
  - Kurama Mode refreshes the skill, with 2× range and 1.5× speed.
- **Data (Minato):**
  - The throw sets `raijin_throw` (6 ticks) on Minato and `raijin_target` (AddBuff) on the aimed enemy, keeping the stage buffs and haste.
  - The 2nd/3rd casts set `tp_now` (+`tp_last`). No Delayed checks remain, so the file went from 1.1 MB to 14 KB.
  - Kurama Mode adds `raijin_refresh` (skill_cooldown_mult +5000 for 10 ticks) and removes raijin_2/3.
- **Native `mod raijin`** (run for any champion with raijin_*/rk*/tp_now buffs):
  - **Throw:** clear old kunai.
    - Middle: `rkm:x:y:travelled:tid:dx:dy`. It moves speed 7000/tick and turns at most 0.17 rad/tick toward the target. Every 5 ticks it spawns a visual Linear segment (effect `tfm2_custom_ai:noop`, casting_target None).
    - It hits within 11000 (deal_damage 40 + 60% AD, Skill) or drops at 160000 travelled, becoming `rkl0:x:y` plus the seal fx.
    - Sides: ±0.38 rad, straight 120000 at 9000/tick, visual projectile, `rkl1/2` plus `rka1/2` while in the air.
    - Kurama Mode: range ×2, speed ×1.5.
  - **Seals:** kunai_planted fx every 30 ticks for landed kunai.
  - **tp_now (with landed kunai):**
    - HP ≤ 35% → the spot furthest from enemies;
    - otherwise → the spot nearest the weakest enemy that has fewer than 3 enemies within 25000. If none qualifies, no teleport.
    - The teleport: set_pos, flash fx ×2, remove that kunai, hiraishin 120, fear 6 ticks within 18000 on the 1st teleport (ForceMove-style dx/dy away). Then tp_now/tp_last are removed.
- **Untested APIs:** spawn_projectile + add_native_effect, play_view_effect, deal_damage, Fear via apply_cc, entity name() format (fallback to tfm2_custom_minato).
- Without the native mod Minato's first skill does nothing.

## Sep 30: Minato kunai life and fear (native 0.3.1, tfm2_custom 0.1.50)
- Kunai stay on the ground 12 s (KUNAI_LIFE 720, preset kunaiLife 720).
- Fear on arrival: teleporting to the homing/middle kunai (rkl0) always fears 30 ticks (0.5 s); a side kunai fears 6 ticks on the first teleport only. The tooltip was updated.

## Sep 30: cooldown-speed leak → infinite Minato ult (tfm2_custom 0.1.51)
- **Observed:** the +5000% "refresh" burst in Kurama Mode let him ult almost endlessly.
- **Engine rule (inferred from play):**
  - A cooldown's length is fixed when it starts, from the current skill_cooldown_mult.
  - skill_cooldown_mult also affects the ult.
  - So a buff shortens only cooldowns that start while it's up, never a running one.
- **Leaks fixed:**
  - `raijin_haste` +700 now lasts 12 ticks (just the throw's own cooldown start); it used to last until the teleport, up to 8 s, and sped Rasengan/ult casts.
  - Kurama Mode buffs (+50 cooldown speed) are applied via Delayed kcmDelay = 15 after the effect, so they don't shorten the ult (it had effectively been 40 s).
  - The refresh burst is removed.
- **New Kurama "refresh":** Delayed 16 → raijin_2 + raijin_throw, so the native mod throws a free Kurama fan (nearest enemy within 200000) and his next cast teleports.
- **To keep in mind:**
  - Gojo's purple strain (−50) would lengthen an ult cast during it.
  - The domain `void` buff (+100) is applied after the ult's cooldown has started, so the ult is fine.

## Sep 30: teleport waits for the Rasengan (native 0.3.2, tfm2_custom 0.1.52 tooltip)
- **Attacking teleports** (HP > 35%) only fire when `minato_rasengan` is up or the skill2 cooldown is ≤ 10 ticks.
  - The cooldown is read via player_at(i).cooldowns().2, matching the player by champion id.
  - Otherwise tp_now/tp_last become `tp_wait` / `tp_wait_last` (360 ticks), and the request lapses if the Rasengan isn't ready by then.
- **Escapes** go immediately.
- A new throw cancels held requests.
- The hook now picks Minato up on any `tp_*` buff.

## Sep 30: domain warning + danger zone + Minato reflex (tfm2_jjk 0.1.22, native 0.3.3)
- **Warning (domainWarn = 60):**
  - The press shows the void_cast charge-up only, plus `gojo_void_warn` (W+2, overlapping conj so the native side never sees a gap).
  - The seal steps spread from W to W+cast, then the full zone from W+cast. total = W + cast + stun.
  - The conj buff comes via Delayed(W). Ring fx at W−1 and domain fx at W+cast−1 come from the anchor LinearProjectile's end_effects (press spot).
- **Danger zone:** the anchor's end_effects also launch a ParabolicProjectile `void_warn` (radius = R, travel W, 10 AP damage) at the press spot. It's a real incoming area the game's AI may dodge (skill_avoid). Best effort.
- **Native:**
  - While warn is up and conj absent: only fix the centre.
  - The guest is picked once after the warning (`void_gpick<gid>`).
- **Minato reflex:** inside an enemy domain that's still warning or conjuring, not trapped or stunned, with a landed kunai beyond R + 5000 from the centre → free teleport to the farthest such kunai. Once per domain (`raijin_evade<gid>`).

## Sep 30: Flying Raijin on the move (tfm2_custom 0.1.53)
- The skill action is now duration 10, start_timing 3, can_use_with_move true. It's one action for throw and teleports, so both no longer stop him.
- Max 3 kunai on the ground: every throw (incl. the free Kurama throw) clears the old ones (clear_kunai).

## Sep 30: domain picture ≠ real area → exact native edge (native 0.3.4)
- **Rian:** champions visibly inside the domain art weren't affected.
- **What calibration showed:** base sprites put the scale near 500–800 units/px (Nightmare projectile: 19 px frame for radius 5000; champion idle frames 25–41 px for collision radius ~10000). So 750 is plausible.
- **More likely causes:** the effect anchor (centre vs feet: a feet anchor shifts the circle up by R) or screen foreshortening (circles shown as ellipses). Neither is verified.
- **Fix now:** run_domain draws the true edge with `debug_draw_circle` (world coordinates, 0xRRGGBBAA):
  - warning: red at R;
  - spreading: cyan at the current r plus a faint full-R outline;
  - conjured: cyan at R plus a soft inner line.
- **Next:** compare the art to that circle (screenshot) and adjust gen.py DOMAIN_R, the anchor offset, or an ellipse ratio. It's also unknown whether debug drawing shows in normal play.
- The data seal zones spawned via self() might not be centred on the press spot (unverified). The native trap/freeze uses the true centre.

## Sep 30: two kunai sets, max 6 (native 0.3.5, tfm2_custom 0.1.54)
- **Bug:** the Kurama free throw set raijin_2 (and raijin_throw), so his next Flying Raijin cast was a teleport request that looked like an empty throw. A real throw within 12 ticks could also be swallowed by `raijin_seen`.
- **Fix:**
  - Kurama now adds `raijin_kthrow` only (no stage change).
  - Native keeps two sets: set 0 (his throws: `rkm0`, rkl0–2, rka1–2, `raijin_seen`) and set 1 (Kurama: `rkm1`, rkl3–5, `raijin_kseen`). A throw clears only its own set (clear_set).
  - Teleports consider all 6 slots. The homing-kunai fear applies to slots 0 and 3.
  - Used kunai are removed as before.
- A careless slice while editing ate the run() fn; lib.rs was restored from lib-v34.rs and redone. Check that a build passes before staging.

## Sep 30: teleport cast with nowhere to go → new throw (native 0.3.6)
- **Rian saw:** Minato "wanted to throw another set of kunai" but nothing was thrown.
- **Cause:** the data stages (raijin_2 8 s, raijin_3 11 s) turn the 2nd/3rd casts after a throw into teleport requests. With no kunai left (all used / expired), or every kunai among 3+ enemies, the request did nothing and the cast was wasted until the stage lapsed.
- **Fix (native only, no data change):**
  - A fresh `tp_now` with no landed kunai, nothing in flight and no throw in the last 12 ticks → a set-0 fan at the nearest enemy (no range limit). The stage resets to `raijin_2` (480), so the next two casts teleport to the new fan.
  - Every landed kunai is too dangerous (3+ enemies within 25000) → the same rethrow instead of skipping.
  - Idle clean-up: no kunai anywhere and no request → raijin_2/raijin_3/tp_wait* are removed, so the next cast is a normal data throw (with haste).
  - The throw code is now `throw_fan()`, shared by all three paths.
- **Unchanged:** an attacking teleport still waits up to 6 s (tp_wait) for the Rasengan. That wait also looks like "nothing happened". It could become a rethrow if Rian prefers.
- A rethrow's own cooldown is the normal 6 s (the throw haste can't be applied after the cooldown starts).

## Sep 30: Minato avoids enemy towers (native 0.3.7)
- **Rian saw:** Minato teleporting under an enemy turret.
- **Data:** game_setting tower (5v5): attack 600, defence 70, attack range 75000, a shot every 40 ticks. 2v2: 200 atk; 3v3: 300.
- **Fix:** each tick, enemy towers are read via tower_count / tower_id_at / get_entity (alive, other team, pos, stat().attack).
  - Reach = 75000 + 10000 (a champion's radius). One shot on Minato is estimated as attack × 100 / (100 + his armor). This is the usual curve; the game's exact formula is unverified.
  - `tower_ok(spot)`: outside every tower, or HP > 1.2 × the shot.
  - **Engage:** tower-free kunai first. A tower kunai is used only as the last resort and only if tower_ok. Otherwise it's a rethrow.
  - **Escape and domain reflex:** tower_ok only, and tower-free spots preferred.
- **Untested:** tower_id_at and stat() on towers.

## Sep 30: champion sprites + sprite editor (editor, tfm2_jjk 0.1.23, tfm2_custom 0.1.55)
- **Rian asked** for sprites "as close as possible" to the anime characters. Declined: no drawing of copyrighted characters. Instead Claude made two original designs matched to the kits, plus an editor Rian can use to change them.
  - **Starweaver** (Gojo slot): cream robe, teal scarf, copper side tail. Blue/red orb conjures; ult: seal with a violet ground ring.
  - **Storm Courier** (Minato slot): plum hood, teal face cloth. Knife slash, 3-knife throw, spinning wind orb; ult: teal storm aura.
- **Format** (SDK assets-and-sprite-sheets.md):
  - Files: `mods/<mod>/champions/<id>#sheet.png` + `#anim.fanim`; the champion has `sprite: asset/<mod>/champions/<id>` and `anim_prefix: ""`.
  - Base frames are trimmed symmetrically around the anchor, so the frame centre is the champion's position. Measured on dark_mage / lightning_mage / ninja / swordman: the feet are 10–12 px below the centre, and characters are about 34 px tall with a 1-px black outline.
  - Our frames: uniform 64×64, anchor at the centre, feet at y = 43.
  - Tags: idle 4, run 6, attack 5, skill1 6, skill2 5, ult 7, hit 1, dead 10.
- **Generator:** `claude/sprite-gen.py` (a pixel puppet: posed limbs, head/hair templates, each part outlined separately). Its output is bundled in editor `sprites-data.js` (`TFM2_SPRITES`).
- **Sprite editor** (`sprites.js`, Skill Lab → Identity → "✎ Edit sprite…", or "★ Own sprite" in Looks like):
  - Animations: add/rename/delete; missing required tags are flagged (idle, run, hit, dead + the slots' action_names).
  - Frames: blank/duplicate/move/delete, per-frame duration, import a PNG.
  - Tools: pencil (Shift = line), eraser, fill, picker (Alt), move/arrow keys, size 1–4, mirror.
  - Palette of sprite colours: right-click replaces a colour in every frame.
  - Flip frame/animation, auto outline, onion skin, grid, anchor + feet guides, canvas resize, undo/redo (Ctrl+Z/Y), live preview, "Start from" originals or blank, Download sheet.
  - "Use this sprite" packs a sheet (a row per animation) into c.assets and marks the folder dirty. Save writes it.
- **Server:**
  - `/api/champ-art` also lists mod sprites (key = full asset path, `url` = `/api/mod-png`).
  - `/api/mod-png` serves them; `/api/base-anim?key=` returns a base sprite's full anims, so the editor can open a base look.
  - art.js `register()`/`get()` handle full-path keys and `url`.
- Moving a champion to another folder carries `champions/` sprite files as well as `vfx/`. Presets now use their own sprites; if sprites-data.js is missing they fall back to a base look.
- **Deployed:** the editor (server.js changed, so the editor must be restarted); both mods get their own sprite (backups: editor\backups\mods\*.20260930-140720-sprites).
- **Unverified in game:** that a champion .fanim loads from a mod with uniform 64×64 frames, and the anchor assumption. Check log.log. If the sprites stand too high or low, shift all frames in the editor or change FEET/GROUND in the generator.
- **Paused:** David Martinez and DIO kits (ideas are in chat).

## Oct 1: batch 2: DIO, David, V1, Vader, Frieren (native 0.4.0; new folders tfm2_jojo / tfm2_cyberpunk / tfm2_ultrakill / tfm2_starwars / tfm2_frieren, 0.1.1 each)
- **Sprites:** Rian asked for "the base sprite with a different colour". recolor.py (project: claude/recolor.py) recolours base-game bodies by hue/sat/value rules, keeping skin and outline: DIO = Vampire (gold), David = Hitman (yellow jacket), V1 = Android (blue/yellow), Vader = Inquisitor (black, red blade), Frieren = White Mage (green tint); summons Fern = Dark Mage (violet), Stark = Berserker (navy). The Vampire's coat is almost black, so it stays dark. All are bundled in sprites-data.js and editable in the sprite editor.
- **VFX:** vfx/gen2.py (project: claude/vfx-gen2.py) → sheets jojo / cyberpunk / ultrakill / starwars / frieren, bundled in vfx2.js.
- **Presets** (presets.js): each has `folder: [id, name]` (addPreset creates or switches to that folder) and `extraSprites` (Fern/Stark bodies under champions/). The Skill Lab's PASSIVES list now includes tfm2_custom_ai:v1 / vader / david / ult_learned. An unknown passive_ref used to crash champHTML.
- **Kits (marker buffs → native):**
  - **V1** (Assassin, pistol 60000). S1 coin: `v1_coin` → the coin hangs 24 ticks, then the shot splits into every enemy champion within 70000 of it; (attack + 60 + 80% AD)/n each. S2 parry: `v1_parry` 30 ticks = damaged/base/skill reduce 100 + cc_immune; the passive's on_damaged counters (50 + 100% AD, heal 40). Unlimited Void strips cc_immune, so it can't be parried. Ult: 3 railgun BAs (`v1_rail3→2→1`, +200000 range buff, penetrate LinearProjectile, 260000 range). Passive: pools on BA/coin hits (max 6, 10 s); stand in one → heal 70 fixed, once per 8 s. 820 + 78 HP.
  - **Vader** (Melee tank). BA: forward rect cleave. S1: saber throw out and back (BackToCaster), self BlockAttack 30. S2: forward rect marks `vader_choke`; the passive stuns each for 120/n (min 30), and Vader gets BlockAttack + −40% move for the same time. Passive: +1 stack per damage event (one per attacker per tick), max 10 → `vader_full`. Ult: casting_target EnemyChampionInCC. At full stacks the passive Binds the nearest enemy 2 ticks every 30, so the AI can press it. Rage (+40% attack, +100% cdr, 8 s) needs vader_full, else an empty cast (5 s real cd).
  - **Frieren** (Magician). S1: `frieren_fern` → native Fern beside her (fern_idle/fern_cast view effects from the recoloured sheet), fires a piercing Zoltraak (native effect, 80 + 110% AP) from her spot after 90 ticks. S2: Stark ParabolicProjectile (90 travel; follows the target) → small circle Airborne 50 + 60 + 90% AP, big circle (30000) −40% slow 2 s. Ult: Limiter release 8 s (+60% MP, +100% cdr, +30% AS, +15000 range; applied via Delayed 15).
  - **DIO** (Assassin). Passive vamp 15 (permanent buff upkeep). S1: 3 straight knives (native, 160000, 30 + 60% AD). In time stop: 5 hanging knives that home in when time resumes. S2: marker 12000 behind the target, resolve at +30 ticks: an enemy within 15000 → DIO blinks behind them, 60 + 100% AD, ForceMove toward his origin (3000×12), stun 60 at the end, DIO returns. Whiff → `dio_refund` skill_cooldown_mult +100 for 14 ticks. EXPERIMENTAL: only halves the cd if the cooldown starts at the action's end. In time stop: MUDA (8 hits of 15 + 35% AD). Ult: `dio_timestop` 240 (cc_immune) → the native freezes every other champion map-wide (freeze helper: pos buff + 5-CC seal + strip immunity), plus free knives at +12 and a free MUDA at +60. A real cooldown refresh is impossible (no API), so the refresh is these free casts.
  - **David** (Assassin). Passive struct holds cp/fuel (×100). Sandevistan buff `dv_sande` (+60% MS, +80% AS) drains fuel 20/s (refill 8/s; dry until 30), cp +6/s, −4/s after 2 s idle; slows enemies within 30000 (−35% MS/−25% AS). Tiers → `dv_t1/t2/t3` for S2 (barrage / MoveToTarget slam / grab + explosion + `dv_selfboom` raw self-damage / nothing). Caution: tier 2 −25%, tier 3 −50% skill_cooldown_mult. ≥70: 25% of BAs redirected (raw) to an ally or himself. 100: psycho (sande forced, −1% max HP per 0.5 s, BA ×0.8 + spill 50% onto an ally 40%). Death resets. Ult: crosshair 60 ticks → gravity 180 ticks, radius 30000, freezes every champion inside but David.
- **Deployed:** the editor (presets, skills, server, index, sprites-data, vfx2); native src lib.rs + batch2.rs + mod.mod_info 0.4.0 (Rian must run build.bat); five mod folders (made through the Skill Lab save path in the test env, then copied); mods.json enables all five (backup mods.json.bak).
- **Unverified in game:**
  - do native passives registered by add_native_passive fire on_attack/on_damaged, and does on_damaged fire at 0 damage (parry)?
  - do deal_damage_raw hits on allies/self work?
  - does the Rect/Forward cleave orient along the aim?
  - will the AI press Vader's ult via the Bind trick?
  - view effects using another champion's sheet (Fern/Stark);
  - does ParabolicProjectile with a view_projectiles Animated body sprite display?
  - does the DIO whiff refund work?

## Oct 1: Gojo, Minato and Stark bodies (tfm2_jjk 0.1.24, tfm2_custom 0.1.56, tfm2_frieren 0.1.2)
- Rian asked for Gojo and Minato to get the recoloured-base-body treatment too, and for Stark to be "a slim guy with a red coat".
  - **Gojo** = Dark Mage body (his original look pick), outfit navy-black, accents bright blue, white hair kept. Tags attack/skill1/skill2/ult are unchanged.
  - **Minato** = Ninja body: blond hair (the Ninja's two warm hair greys only), navy outfit, red scarf → white. Action names changed to the Ninja's tags: skill_pre / skill2_attack / ult_pre (deployed file + preset).
  - **Stark** = Exorcist body: red hair, red coat. The leap projectile anim tag changed from ult_dash to skill1 (the Exorcist has no ult_dash).
- sprites-data.js adds 'gojo' / 'minato' and updates 'stark'. The presets now use them; Starweaver and Storm Courier remain as "Start from…" options in the sprite editor.
- Backups: editor\backups\mods\*.20261001-*-recolor.

## Oct 1: Stark = Spellbreaker with an axe; David psycho drain 7%/s (native 0.4.1, tfm2_frieren 0.1.3, tfm2_cyberpunk 0.1.2)
- Stark's body is now the base Spellbreaker (a data champion sprite in the bundle) via spell_axe.py (project: claude/spell-axe.py): exact-colour recolour (hair → red, coat purples → red, suit/shoes → black). The greatsword is replaced per frame: blade found by its exact colours (largest 8-connected component), erased with its orphaned outline, then an axe (wooden haft + bearded steel head on the side away from the body) is drawn along the blade's principal axis from the hilt end. Frames are padded 8 px (anchor unchanged) and repacked.
- Spellbreaker tags: attack, dead, skill, skill2, ult, hit, idle, run. The leap projectile uses 'skill'.
- David at 100 cyberpsychosis: 3.5% max HP every 30 ticks = 7% per second (PSYCHO_DRAIN_PER_HALF_S). Tooltip updated.

## Oct 1: first battle feedback (native 0.4.3; jojo 0.1.2, cyberpunk 0.1.3, ultrakill 0.1.2, starwars 0.1.3, frieren 0.1.4)
- **Vader:**
  - The choke visual is now played by the native on each caught enemy (fx_on). The data ViewEffect inside a RangeEffect had drawn it under Vader.
  - Choke = Airborne shared (120/n, min 30) and no longer blocks his basic attacks (slow −40% kept).
  - Saber throw is now native: cast range 105000 (1.5x); speed 3500, homing (0.12 rad/tick), 160000 of flight (the kunai lifespan); hits every enemy within 10000 (50 + 90% AD out, 25 + 45% back); returns at 5000 speed. BlockAttack is refreshed while it's away. Data S1 = `vader_throw` + `vader_saber_target` markers.
- **David:**
  - Melee (range 23000, plain Attack).
  - HUD via view_buffs: `dv_bar0..10` (bar_0..bar_10: 24x4 bar 32 px above his centre; cyan <50, yellow <70, red flickering ≥70) and `dv_t0..3` icons (pistol / fist slam / grab + spark / red X unusable).
  - Sandevistan afterimages: the passive keeps a 12-tick position trail; every 4 ticks while dv_sande it plays after_{g,m,c}_{r,l} (a tinted copy of his run frame, flipped by movement direction) where he was 6 ticks ago. The old 'sande' buff visual was removed.
- **Frieren:**
  - BA = piercing LinearProjectile Zoltraak (thin continuous white line 70 px, speed 12000, 20 + 60% AP, cooltime 110).
  - Fern's Zoltraak: 'zoltraak_fern' 130 px thick line, speed 20000 (was 9000), radius 13000, range 260000.
  - Stark: travel 50 ticks (was 90). The native draws him jumping along the arc (stark_up / stark_down poses cut from his sheet, fx every 2 ticks, height sin·36000) plus a big red 'stark_mark' (radius 30000) on the target until he lands. The red circle art moved here from DIO as Rian asked.
- **DIO:**
  - The World marker is now a yellow "!" with the hit ring. Hit radius 22000, measured on champion centres. The old 15000 missed enemies whose bodies visibly overlapped it: the likely reason he "didn't teleport".
  - Resolves after 20 ticks (was 30); S2 action duration 12 (was 36), so he isn't mid-cast when repositioned.
  - Time stop: 3 s, dome radius 90000 around DIO that follows him (only champions inside freeze); timestop buff visual = yellow half-bubble; no cc_immune any more.
  - Knife charges in stopped time: 2 charges of 4 straight knives (hang, then fly straight; no homing), used by his S1 casts (`dio_knives4`); if S1 is on cooldown they fire themselves at +30 / +100.
  - One free MUDA at +60 if S2 is on cooldown and unused.
  - Frozen-first: a 30-tick Taunt toward the nearest frozen enemy every 45 ticks (experimental).
- **V1:** parry window 1 s (60 ticks; it was 0.5 s, the 5 s cooldown made it feel permanent), cooldown 10 s. Railgun 60 + 110% AD (was 80 + 160%), radius 10500 (1.5x), sprite 1.5x, EnemyWithoutTower.

## Oct 1: Sandevistan pacing (native 0.4.4, tfm2_cyberpunk 0.1.4)
- **Rian saw:** the AI used Sandevistan until the fuel hit 0, then only again much later.
- **Cause:** each press = 3 s on + a 3 s real cooldown (4.5 s with caution), and a press on an empty tank still spent the full cooldown; fuel had to climb back to 30.
- **Fix:**
  - Short pulses: sandeTicks 60, sandeCd 45. The AI keeps re-pressing while fighting, so it stays on while there's fuel and lapses 1 s after he stops fighting (saves fuel).
  - An empty press only costs 0.75 s. FUEL_BACK 1500 (usable again from 15).
  - HUD: a fuel bar (dv_fuel0..10, green) under the cyberpsychosis bar; dv_fueldry flashes red while it recharges after running dry.

## Oct 1: battle feedback round 2 (native 0.4.5; jojo 0.1.3, cyberpunk 0.1.5, ultrakill 0.1.3, starwars 0.1.4, frieren 0.1.5)
- **David:**
  - Psycho (100): a 40-tick Taunt toward the nearest enemy champion (within 200000) every 30 ticks, so he charges in instead of backing off or recalling.
  - Sandevistan cost: cyberpsychosis +14/s while on (was 6), drains only after 3 s idle (was 2), fuel drain 25/s (was 20).
  - Escape: HP ≤ 45%, an enemy within 50000, moving away from it (trail 6 ticks ago was closer), fuel ≥ 20, cyberpsychosis < 60 → the native adds dv_sande (60 ticks) itself.
  - Ult: a single smash-down at +60: whoever is inside right then is pinned 180 ticks (tracked by id); gravity fx 30 ticks; nothing lingers.
- **Vader:**
  - Saber straight (no homing), SABER_LIFE 140000 = 2x the original 70000, cast range 140000.
  - Choke is fully native: a cone of 55000 × ±0.6 rad toward the `vader_choke_aim` enemy, 40 + 70% AD, Airborne shared. A red V (choke_v0..7, 8 directions, placed half-way along the aim) shows the area.
- **DIO:**
  - "!" is red; the mark resolves after 33 ticks (0.55 s); knockback 3000×20 (was ×12).
  - Time stop: flat 2D field (timestop_field fx, 180 ticks, clock design) fixed at the cast spot. State "dts:t0:charges:x:y"; freeze within TS_R of that spot. The follow-buff dome is gone.
- **Frieren:**
  - Fern: target within 70000 → zoltraak_big (speed 7000, radius 18000, range 150000); else zoltraak_fern (11000 / 6000 / 260000).
  - Stark is fully native: fixed landing spot (the target's position at the jump), inner 20000 (Airborne 50 + 60 + 90% AP), outer 50000 (−40% 2 s). The mark is fx_at (not follow). The data ParabolicProjectile was removed.
  - Limiter release (`frieren_limit`): "flp:t0:x:y". Fern stays beside her (fern_idle every 20 ticks) and fires a homing zoltraak_big (shoot_homing, zoltraak effect) every 100 ticks. Stark stands where he last landed (stark_idle = stark body idle) and jumps homing onto the nearest enemy every 150 ticks.
- **V1:**
  - Coin is thrown toward the target (0.75 × distance, max 90000), arcing over 20 ticks (coin fx every 2 ticks), shot at +24; ricochet within 40000, damage split.
  - Parry reflex: an enemy projectile (projectile_at: pos/caster/team only) within 26000 that is closer to V1 than to its caster → if no v1_parry_cd, apply the parry buff (60 ticks) + v1_parry_cd 600 + 'alert_parry' (yellow "!!"); otherwise 'alert_red' (red "!"), throttled to 1 per 30 ticks. Data S2 checks v1_parry_cd, so the two never stack. "Depends on the skill inputs" was read as "only when the parry is ready"; the athlete's own reaction stats aren't exposed to mods.

## Oct 1: Stark in Limiter release fights on foot (native 0.4.6, tfm2_frieren 0.1.6)
- **Rian:** Fern's behaviour is fine. Stark should jump once onto each different enemy that comes within reach, never the same one twice, and otherwise basic-attack with a longer cooldown; he switches to a new enemy by jumping on it when the jump is ready.
- **Native:**
  - Stark's spot is "flp:t0:x:y" (updated as he walks and when he lands).
  - "fsj<id>" = enemies already jumped on this ult; "fst:<id>" = current target; "fsjt:<tick>" = last jump; "fsba:<tick>" = last swing.
  - Jump when a not-yet-jumped enemy is within 90000 of Stark and 120 ticks have passed since the last jump (homing landing).
  - Otherwise he walks toward his target at 900/tick (stark_idle fx every 6 ticks) and within 22000 swings every 90 ticks: 40 + 50% of Frieren's AP as physical damage, 'stark_down' pose.
  - All of it is cleared when the ult ends.

## Oct 1: battle feedback round 3 (native 0.4.7; jjk 0.1.25, jojo 0.1.4, cyberpunk 0.1.6, ultrakill 0.1.4, starwars 0.1.5, frieren 0.1.7)
- **Scale:** VFX art is 950 world units/px, measured from base area art (Priest heal zone 32000 = 66 px, Exorcist ult 60000 = 128 px). The old 750 made art about 27% too big. gen.py UNITS_PER_PX / gen2.py U = 950.
  - Rian asked to keep the Gojo and DIO domains at their old on-screen size. So their real areas grew to match the art instead: Gojo DOMAIN_R 60000 → 76000 (80 px), DIO TS_R 90000 → 114000 (120 px). The data side matches (domainRadius, tsRadius), and gen.py's Gojo sheet came out byte-identical to the pre-scale one.
  - The other batch-2 art (Stark rings, zoltraak, etc.) is drawn at 950.
- **Gojo:** domainWarn 0. No warning second: the seal starts spreading on the press and the whole timeline is 60 ticks shorter.
- **Vader:** the saber's projectile visual didn't render. It is now fx_at `saber_spin` every 2 ticks (time 2) at the saber's position.
- **David:**
  - Ult cost is by mode, chosen at cast:
    - Area (+25): picked if other enemies are within GRAV_R.
    - Lock-on (+40, crosshair_lock on the target, the centre follows it until the smash): picked if the target is alone and worth it (≤55% HP, or attack + MP ≥ David's attack).
    - Otherwise area.
  - S2 tiers: pistol 0-19, slam 20-29, claw grab 30-74, off 75+.
  - Sandevistan: move +40 / attack speed +50 (slower); cyberpsychosis +10/s, fuel 22/s, drains after 2.5 s idle.
- **Frieren:**
  - Duplicates / "cut up" came from overlapping view effects. Fix: an fx replayed every 2 ticks has time 2; Fern's S1 is one fx (`fern_s1`: idle loop + cast); Fern at her side in the ult is a buff visual (`frieren_fern_side`, idle frames offset 14 px).
  - Stark in the ult is a real unit: spawn_unit("tfm2_frieren_stark"), with StatV1 (attack 40 + MP/2, HP 500 + max HP/2) and UnitAttackV1 (range 22000, cooltime 90), plus buff visual `stark_body` (Stark sheet, run).
    - He moves and basic-attacks by the game's own AI; the native part only does the jump (once per new enemy within 90000, 120-tick cd). During a jump he is invisible and stunned, then set to the landing spot.
    - **Untested:** whether buff visuals render on spawned units, and whether spawn_unit works at all in 0.6.2.
  - Fern fires every 150 ticks (2.5 s).
- **DIO:**
  - Freeze = inside the field or already carrying the frozen buff. The freeze() helper snaps anyone knocked out back to their frozen spot, like Purple in Gojo's domain.
  - The field art now matches TS_R exactly (see Scale).
- **V1:**
  - S1: one coin per cast ("v1_coin_seen"), airborne for 50 ticks, 14000 ahead and 26000 up. While it's up, his next basic attack shoots the coin: the attack + 60 + 80% AD, split among enemies within 60000 of it, and the direct hit is set to 0. Once the coin drops, it's gone.
  - S2 parry: a 6-tick window (0.1 s), 300-tick cd. On a parry: "spellbreaker_attack" sfx (the ting), yellow "!!", and the v1_parried buff for 180 ticks (+30% attack, +30% attack speed, +20% move). No shot back.
  - AI: it parries an incoming enemy projectile within 16000 only when it is risky, i.e. V1 is at ≤40% HP, the caster has gojo_purple_strain / frieren_limit, or the caster's attack + MP ≥ 1.2× V1's attack. Otherwise it shows the red "!".
  - Blood pool only on champion hits, 12 s cd. Basic-attack range 30000 (halved).
- Backup of everything replaced: `C:\Games\tfm2\Claude outputs\backup-before-battle3\`.

## Oct 1: battle feedback round 4 (native 0.4.8; custom 0.1.57, jojo 0.1.5, cyberpunk 0.1.7, ultrakill 0.1.5, starwars 0.1.6, frieren 0.1.8)
- **Frieren crash:** the likely cause was the spawned Stark unit (spawn_unit + entity_set_pos / set_invisible on a unit id that could be dead or stale). It's gone, along with every unit call.
  - Stark in Limiter release is now fully native and drawn pose by pose: fx `stark_{i|r|a}{r|l}{n}` (idle 4 / run 8 / attack 5 frames, right or mirrored left), one every 2 ticks, time 2.
  - State "flp:t0:x:y:face", "fsba:<tick>" = swing start. He walks 900/tick toward the nearest enemy and swings every 90 ticks; the hit lands at +10 for 30 + 35% AP. He jumps once per new enemy (35 + 50% AP landing). He can't be targeted.
- **V1:**
  - S2 now arms the parry: Permanent buff `v1_guard` (yellow diamond visual), cast range 200000 so the AI arms it as soon as it's ready, 10 s cd. The native side enforces `v1_parry_cd` 600 per parry.
  - Parry triggers:
    - A risky projectile within 16000 (estimated hit ≥ 0.9× his attack, Purple / Limiter, or V1 ≤ 40% HP).
    - An enemy that moved ≥ 35000 in one tick and landed within 35000 of him (any teleport).
    - `batch2::try_parry` called by DIO's The World strike, DIO's MUDA start and Minato's teleport slash before they hit.
  - Reward: attack +15..70% (15 + estimate × 120 / max HP), three quarters of that as attack speed, +20% move, for 180 ticks. Then a Taunt (120 ticks) onto the attacker, plus a `parried_on` aura.
  - Damage up: stats 108/870 (+27/+82 per level), BA cd 50, coin 70 + 100% AD, rail 70 + 120%.
- **DIO:**
  - Mark radius 24200 (+10%), delay 36 ticks; knife speed 6500 (was 9000); MUDA 10 + 22% AD per punch (was 15 + 35%).
  - Time stop 120 ticks, radius 76000 (= Gojo), 1 knife charge, auto knives at +25, auto MUDA at +50. V1 can parry the strike and MUDA.
  - Laning fix: HP 930 / growth 85, def 25, lifesteal 10% (was 15), strike cd 10 s.
- **Vader:** S1 cd 12 s, saber range 112000 (−20%); S2 cd 13 s.
- **David:**
  - Claw overlay `dv_claw` (claw_on: hooked steel talons on both hands) while cyberpsychosis ≥ 30.
  - S2 nerfs: pistol 6 × (8 + 18%), slam 50 + 95%, grab 40 + 70% then boom 60 + 90%.
  - Gravity pin 150 ticks (was 180).
- **Minato** (Rian picked the teleport slash): every Flying Raijin teleport hits enemies within 20000 of the landing spot for 50 + 80% AD.
  - Less BA-dependent: attack 96 / growth 21 (was 105 / 24), BA 90% AD (KCM 110%), Rasengan 110 + 180%.
- **Frieren:** Fern 70 + 100% AP; Fern in the ult uses its own effect `zoltraak_party` (45 + 65% AP); S2 Stark landing 55 + 80%.
- **Balance target:** base-game level by role (gamedata: assassin 120/900 +30/+80; bruiser 100/1000; tank 80/1100; mage 80 atk / 40 MP / 900; skills about 40-60 + 70-100%, ults 80-200 + 60-150%).
- Backup: `C:\Games\tfm2\Claude outputs\backup-before-battle4\`.

## Oct 1: Minato dodge system (native 0.4.9, tfm2_custom 0.1.58)
- **Rian's idea:** skill expression through dodging, with kunai as the dodge charges.
- **S1:** all three kunai fly straight and slower (middle 5000/tick, range 150000; sides 6500). The middle one hits the first enemy champion in its path (40 + 60% AD) and drops there. Ground state is "rkl<slot>:x:y:<hit>"; a kunai that hit an enemy gives the 0.5 s fear on teleport.
- **Kurama Mode throw** (set 1, slots 3-7): 5 kunai straight onto the ground, 3 ahead toward the target (±0.45 rad, distance to target + 8000, clamped 30000-110000) and 2 at 80000 away from the enemies' centroid (±0.55 rad). Separate from the S1 set (slots 0-2), so up to 8 kunai can lie on the ground.
- **Dodge:**
  - Trigger: a projectile from an enemy champion within 15000, approaching (closer to Minato than to its caster).
  - He teleports to the nearest kunai 12000-160000 away (not into 3+ enemies or a deadly tower) and uses it up. With no usable kunai, a 20000 sidestep across the shot's path (`hr_blink_cd` 180).
  - Afterimage fx after_r / after_l (0.5 s) at the old spot, plus `hr_after` 30 ticks of full damage immunity. `hr_dodge_cd` 90.
  - Not while stunned or void_trapped; melee can't be dodged.
- **Stacks:** "hrs:<n>" plus `minato_flow<n>` (pips flow_1..5), 420 ticks, max 5. Each stack +8% attack / +6% attack speed / +5% move speed; at 5 stacks also 10% lifesteal.
- **Limit:** projectiles can only be read by mods, not retargeted, so a homing shot isn't really redirected. The 0.5 s immunity is what makes the afterimage "take" it; a slow homing shot still chasing after 0.5 s can hit.
- **Stats:** attack 92 / growth 20 (−4%) to pay for the stacks.
- **Lightning charges (tfm2_custom 0.1.59):** planted kunai (dodge charges) crackle with jagged lightning arcs; the stack pips flow_1..5 are small flickering lightning bolts.
- **Storm Runner body (tfm2_custom 0.1.60):** an original look Claude proposed and Rian adjusted. Charcoal hood with electric-yellow piping on its front rim, spiky silver fringe under the hood, charcoal cloak (the Ninja's scarf) with a yellow hem, teal bodysuit, red shoes. Built by storm_runner.py from the base Ninja sheet (in-place recolour plus a few pixel edits, same layout); recolor.py no longer makes 'minato'. The dodge afterimage uses the new body.
- **Sprite request declined twice:** spiky hair + headband + green suit + white cape = that anime character's signature look. Offered an original look, or Rian editing his own copy in the sprite editor.

## Oct 1: Minato skill share + KCM cloak (native 0.4.10, tfm2_custom 0.1.61)
- **Rian's design for the stacks** (the dodge stacks are now a resource):
  - Teleport slash with 3+ stacks: big lightning slash 75 + 115% AD (fx slash_big), spends 2 stacks (only if it hit someone). Otherwise a weak slash 35 + 55% AD.
  - Rasengan on arrival: after the slash, if minato_rasengan is charged and ≥1 stack is left, it fires on the nearest enemy within 25000 (110 + 180% AD, stun 60, V1 can parry) and spends 1 stack. Otherwise Rasengan still needs his BA.
  - Stacks are decremented with their remaining duration kept (set_flow / flow_count).
- **Kurama Mode dodge (+50%):** detection 22500, afterimage / immunity 45 ticks, dodge cd 60.
- **KCM look:** the body sheet can't be swapped mid-match (the API has no sprite/animation control; CasterAnimation only plays one tag for N ticks). So the `kcm` buff visual (z −1, behind him) is now a glowing orange chakra cloak: it hangs from the shoulders past the knees, flares out symmetrically and has flame tips at the hem.
- **BA share estimate** before this change: ~70-75% of his damage in a 10 s fight at mid level.

## Oct 1: David's claw overlay removed (native 0.4.11, tfm2_cyberpunk 0.1.8)
- Rian asked to remove it. The `dv_claw` buff visual, its native toggle and the gen2 `claw_on` art are all gone. The S2 grab tier (30-74) works as before.

## Oct 1: Kurama throw rework + walls (native 0.4.12, tfm2_custom 0.1.62)
- **Kurama Mode free throw (set 1):** one homing kunai "rkm1:x:y:trav:tid:dx:dy", speed 9000, range 320000, turn ≤0.17 rad/tick, through walls. On a hit (60 + 80% AD) or at max range it splits into 5 kunai that fly through walls: 3 ahead (±0.45 rad, 120000) and 2 back (±0.4 rad, 90000). A landing spot inside a wall is pulled back to the nearest free point (walls::pull_back). Slots 3-7.
- **Walls:**
  - New `WallReader` map customizer reads `walls` (30×30, 32000 per cell) at match creation into a static grid. Row order is undocumented, so it picks rows = y or rows = x by whichever puts fewer towers inside walls.
  - Normal S1 kunai now stop at walls: side kunai via walls::clip; the middle kunai drops where it meets a wall.
  - The no-kunai sidestep never lands in a wall.
  - If the map document can't be read, there are no walls (same as before).
- **Unverified:** the walls JSON shape and the orientation guess.

## Oct 1: Stark as a targetable companion (native 0.4.13, tfm2_frieren 0.1.9, editor skills.js)
- **Rian's choice:** a real unit like the Druid's bear, 12 s, with the summon sprite swapped. He accepted the side effect that Necromancer ghouls look like Stark while the mod is on.
- **How mod units are drawn:** the exe shows that mod-spawned units are the built-in summon type (ghoul-style AI) and use the ghoul's view. The spawn_unit name doesn't pick a sprite.
- **Sprite swap:** mod.override_info overrides `asset/base/aseprite_resources/champions/ghoul` with `asset/tfm2_frieren/champions/stark_ghoul`. That is Stark's sheet with the ghoul's tags (idle, run, attack, dead, hit + berserk_idle / run / attack / dead). The ghoul ref is in `C:\Games\tfm2\Claude outputs\ghoul_ref`.
- **Editor:** presets can declare `overrides(modId)`, and Skill Lab Save writes them into mod.override_info for any folder holding that preset's champion.
- **Native:**
  - At the start of Limiter release: spawn_unit("tfm2_frieren_stark", Frieren, team, 720 ticks). Stats: attack 30 + 35% AP, HP 500 + half Frieren's max HP, def 30 / mr 25 / ms 1000. Attack: range 22000, cooltime 90.
  - "fsu:<id>" lives 720 ticks. The unit is used only if it's alive, on Frieren's team and not a champion.
  - Leap per new enemy (cd 120): stark_up fx, entity_set_pos next to the target (walls::pull_back), stark_land 35 + 50% AP.
  - No buffs, invisibility or CC are ever put on the unit (round-3 suspects for the crash).
  - If spawn_unit returns None, the fallback is the old fx-drawn Stark.
- **Risk:** the round-3 crash happened with a spawned unit. Watch for crashes when Limiter release starts.

## Oct 1: round 6 (native 0.4.14; custom 0.1.63, starwars 0.1.7, ultrakill 0.1.6)
- **Vader:**
  - SABER_SPEED 2600 (was 3500).
  - New 'choke' animation added to his body by vader_choke.py (run after recolor.py): free arm thrust out, fist clenched and rising, dark-red specks.
  - S2 action_name 'choke', plus CasterAnimation 'choke' for chokeTotal (120 ticks), so he holds the pose while choking.
- **Minato:**
  - `batch2::danger_zones(all, team)` lists DIO's red mark (`dmk:`, strikes at t0 + MARK_DELAY, MARK_R), DIO's time-stop field while it's on (`dts:`, TS_R) and David's pending gravity. David now publishes the gravity as "dvz:<x>:<y>:<smash tick>:<lock+1>" each tick before the smash.
  - lib.rs adds every open Unlimited Void (centre_of, DOMAIN_R).
  - Teleports (engage and escape) and kunai dodges only use spots outside all zones + 4000.
  - He dodges out of a zone that strikes within 14 ticks: to a safe kunai, or a step straight out of the zone (no-kunai blink rules). A gravity locked on him is skipped, since it follows him.
  - S1 fan: side angle 0.55 rad (was 0.38), side range 145000 (was 120000), middle range 170000 (was 150000).
- **V1:**
  - A parry gives `v1_shotgun` (180 ticks). His next BA is 6 pellets over ±0.35 rad, range 45000; each pellet hits the first enemy champion within 6000 of its line, for 28% of the BA at point blank, falling to 25% of that at max range. Damage is dealt as Skill; the BA itself does 0.
  - Projectiles from non-champions are never parried.
  - While armed (guard up, no parry cd, HP > 50%, not hunting): Taunt 24 ticks onto the nearest enemy champion within 70000, every 60 ticks.
- **DIO rework:** Rian wants him to be a teamfight brawler with time powers and a Stand, not a 1v1 specialist, with 2 modes. Options were offered; waiting on his pick.

## Oct 1: DIO rework — Stand Out / Stand In (native 0.4.15, tfm2_jojo 0.1.6, tfm2_ultrakill 0.1.7)
- **Rian's design:** a 1v1 brawler, not an assassin, with long cooldowns across his 5 skills. I chose the mode switch: automatic, at most every 5 s (MODE_CD 300), also in stopped time. Target ≤ 28000 → Stand In; ≥ 45000 → Stand Out.
- **Stand look (stand.py, original):** the base Android's torso and arms + the Nightmare's face (hood and hair removed), translucent gold, no legs (it fades into a wisp below the waist). Poses: idle 4, punch 4, bar 4 (fist barrage), world 4 (arm out + starburst), lunge 1, grab 1. 56×56 frames with the anchor at the centre. The Android and Nightmare refs are in `C:\Games\tfm2\Claude outputs\stand_ref`.
- **Native passive `tfm2_custom_ai:dio`:**
  - Mode buffs: `dio_out` (range +32000, no lifesteal) and `dio_in` (vamp 12).
  - Stand drawing every 2 ticks: fx stand_<pose><r|l><n>, 9000 behind him and 3000 up, facing the nearest enemy. It only shows in Stand Out, and never during the time stop (then it's at the field's centre via stand_world_ult).
  - Actions: Punch (Out BA, from on_attack), Block, Counter, Windup → Lunge → Grab → Return, Pop (In S2).
  - Out S1 = Permanent `dio_guard`, used by:
    - a champion projectile within 16000 → `dio_block` 8 ticks (damaged_reduce 25, cc_immune);
    - on_damaged by an enemy champion within 30000 → counter: stun 45 + 6 × (6 + 12% AD) over 36 ticks (V1 can parry it).
  - Out S2 lunge: 60-tick wind-up, then 9000/tick for 80000 through walls, hit radius 14000. Grab: 0.7 s, 5 × (10 + 14% AD), −40% move speed; then the Stand returns over 14 ticks.
  - In S2 dash: stand_pop + vanish at the start, entity_set_pos behind the target (walls::pull_back), appear, 60 + 100% AD, stun 36, V1 parry check.
- **Match hook (run_dio):** knives only (3 straight; 4 hanging in stopped time) plus the time stop.
  - Time stop: 120 ticks, TS_R 76000, freeze, taunt onto the frozen; mono timestop_field + stand_world_ult at the centre.
  - The data adds `dio_ts_cut` (attack −40%) for the time stop.
  - Removed: the mark/strike, MUDA, the auto knives/MUDA and the old lifesteal stack.
- **Art:** the time stop field and the ZA WARUDO burst are black and white now. New fx: stand_fist (ranged punch), stand_block, stand_swap (mode change), vanish/appear (DIO dissolving to black-and-white specks).
- **Data:** S1/S2 cooldowns 660 each, shared between the modes (only 4 slots exist).
- **V1 shotgun art:** while `v1_shotgun` is loaded, fx shotgun_<r|l> at his hand pointing at the nearest enemy; muzzle_<r|l> when it fires; pellets use the 'pellet' projectile.
- **Editor:** skills.js PASSIVES knows tfm2_custom_ai:dio.

## Oct 1: DIO Stand Out punch reworked (native 0.4.16, tfm2_jojo 0.1.7)
- Rian wanted the Stand to go to the target and punch there, "locked on" like a shadow, instead of a flying fist.
- Native: on_attack in Stand Out starts Punch { t0, tid, sx, sy }. The Stand glides from where it was to the target's near side over 8 ticks (lunge pose), punches for 12 ticks (punch poses), hovers by the target for 24 ticks, then drifts home (Return). A new attack while it's out restarts from its current spot (`spos`).
- Data: the Stand Out BA is now an invisible TargetProjectile `stand_hit` (no view binding) at speed 4500, so the damage lands about when the fist does. The stand_fist binding is gone.

## Oct 1: ghoul override fixed (tfm2_frieren 0.1.10)
- The game warned "Only 7/8 asset override(s) were applied". log.log said: "Override source 'asset/tfm2_frieren/champions/stark_ghoul' ... was not found, skipping 'asset/base/aseprite_resources/champions/ghoul'".
- Cause: a manual sprite is two assets, `<name>#sheet` and `<name>#anim`, so a path without the suffix doesn't exist. The override is now two entries: ghoul#sheet → stark_ghoul#sheet and ghoul#anim → stark_ghoul#anim (the presets.js overrides() was updated the same way).
- Lesson: override sprite pairs per half (#sheet / #anim). The SDK's example without a suffix doesn't apply to manual sheet pairs.

## Oct 1: DIO polish (native 0.4.17, tfm2_jojo 0.1.8)
- **Vampire blast on the BA:** the "projectile" Rian saw was the golden blast drawn inside the Vampire's own attack frames 3-5. dio_attack.py re-points the 'attack' tag at the clean frames 0,1,2,2,1,0 (same sheet, pixels unchanged; run after recolor.py). Stand In's melee hit adds a 'strike' burst on the target instead.
- **Stand Out:** `dio_out` now also gives +45% attack speed, and the punch drops to 60% AD.
- **Knives with the Stand out:** run_dio ignores a knife marker while `dio_out` is up and arms the guard instead. Mode switches are also held while a skill marker (dio_knives*, dio_lunge, dio_dash) is pending, the cast → effect window.
- **Choppy Stand:** it's now drawn every tick with fx time 1 (was every 2 ticks, time 2). Idle follows by closing 1/4 of the gap per tick, with a sine bob. The punch rush is eased out over 10 ticks.

## Oct 1: round 8 (native 0.4.18, tfm2_custom 0.1.64, tfm2_starwars 0.1.8)
- **Frieren crash (again, root cause):** log.log panicked with "animation info not found: asset/base/aseprite_resources/champions/tfm2_frieren_stark#anim". The match view draws a spawn_unit unit from the base sprite named after the unit, so an unknown name crashes the host (not caught by the mod guard). Stark is now spawned as `spawn_unit("ghoul", …)` for 720 ticks, and the ghoul#sheet/#anim override draws him as Stark. If spawning fails, the old fx Stark is the fallback.
  - Lesson: spawn_unit names must be existing base champions; restyle them with asset overrides.
- **DIO trail:** fx time 1 plays the whole Stand animation each tick, so copies stacked into a trail. Back to drawing every 2 ticks with time 2 (the smoother lerp/bob from 0.4.17 stays).
- **Vader S2 reworked:** the choke holds 60 ticks (target Airborne, Vader self-rooted: Bind/BlockAttack/BlockSkill so he can't walk away and cut the choke animation). At tick 60 slash 1 (20 + 40% AD), at tick 78 slash 2 (30 + 60% AD) plus a ForceMove knockback away from him. Data plays CasterAnimation choke 60, then attack twice.
- **Minato:**
  - Escape teleport only to a kunai ≥ 55000 from enemies and ≥ 25000 farther than where he stands; otherwise no teleport (stops the low-HP suicide jumps onto kunai near enemies).
  - Engage only to a kunai within 45000 of an enemy; otherwise he throws a fresh fan that replaces the old one (fixes "can't throw while old kunai still live"). The old Rasengan wait and TP hold are gone.
  - Backpack (2 charges): he picks up an untouched landed kunai within 12000 when walking over it. When none is on the ground he throws one pack kunai (70% range, half damage, 6 s life) that he can teleport to. Icons pack_1/pack_2 over his head; buffs rbp:<n>, minato_pack1/2.
- **Opinion given:** the backpack isn't too strong (pickups need positioning, the pack kunai is weaker, max 2). It's moderately complex, but the native AI does it automatically.

## Oct 1: round 9 (native 0.4.19, tfm2_custom 0.1.65, tfm2_frieren 0.1.11, tfm2_ultrakill 0.1.8, tfm2_cyberpunk 0.1.9)
- **Build path:** the data files are generated by node straight from presets.js (same serialization as the Skill Lab's buildFiles; verified byte-identical against round 8), not through the Skill Lab: the testenv mods are stale and the Skill Lab refuses a preset that already exists. mod_info versions are bumped in place on the device.
- **Minato:**
  - Old kunai are replaced only when no kunai lies near an enemy AND the backpack is empty. A kunai near an enemy that's just unsafe is kept. Otherwise the teleport cast throws a pack kunai.
  - Pickup only of a kunai with no enemy within 45000, on a safe, tower-free spot.
  - Chasing (an enemy 35000+ away, running from him, or at 40% HP or less), with no kunai by an enemy → pack throw first. A pack kunai landing on or next to an enemy auto-flashes him there (arrive(): slash/Rasengan/fear), unless it's a crowd, a tower, a zone or he's escaping. Otherwise it's planted and raijin_3 is set.
  - S1 cooldown 6 s → 5 s.
  - The arrival logic is now `arrive()`.
  - Rasengan scales with stacks: 50 + 80% AD, +20% AD per stack (native and data via minato_flow1..5); stun 1 s → 0.75 s. Big Ball 60 + 90%, +15% per stack.
  - KCM: +20% move speed, +15% attack speed, 10% less damage taken; BA 95% with a 15% splash.
  - **Melee dodge:** detected from an enemy champion's attack cooldown jumping up (player.cooldowns().0, kept per enemy in "rac<id>:<cd>" buffs) within 32000, when he's that enemy's closest target. Costs 1 stack; 3 s cooldown. He flashes to a kunai or steps 30000 away.
- **Frieren:**
  - Nerfs: BA 15 + 50% AP; Fern 55 + 80%; party Fern 35 + 50% every 3 s; Stark landing 45 + 65%, leaps 25 + 40%; slow −25% for 1.25 s in a 40000 circle; Limiter +40% magic power and no cooldown speed.
  - Zoltraak and party Zoltraak skip towers (is_tower).
  - Stark unit: 300 + 25% of max HP; dies after 3 hits ("fsh:<hp>:<hits>", an HP drop ≥ 10 counts as a hit; killed with deal_damage_raw credited to the nearest enemy).
  - While frieren_limit or the Stark unit is up, she gets BlockSkill (re-applied every 10 ticks), and the native side ignores fern/stark markers.
- **V1:**
  - Double pump: the shotgun BA fires a second blast 12 ticks later.
  - The coin is auto-shot at its peak (20 ticks) if an enemy is near it. With the shotgun loaded the coin shot is a split shot: every enemy near the coin takes the full ricochet.
  - No parry while v1_rail1-3 is up.
  - Hunt and nudges only if worth_chase: within 60000, not under an enemy tower (unless the target ≤ 25% and V1 ≥ 60%), not into 3+ foes without 2 allies, and V1 above 35%. After a parry he presses every 0.5 s.
- **David:** 75+ (and full psychosis) S2 = Sandevistan rush grab (MoveToTarget 16000, dv_rush afterimages, dv_s2r = +15 psychosis), the same grab + explosion + self-damage. New icon ico_rush (gen2.py).

## Oct 1: round 10 (native 0.4.20, tfm2_custom 0.1.66, tfm2_cyberpunk 0.1.10, tfm2_starwars 0.1.9, tfm2_ultrakill 0.1.9)
- **Vader:**
  - CasterAnimation never showed the choke pose. Rian saw the 'choke' tag only for the 20-tick cast; Nightmare uses CasterAnimation with a real tag, so the likely cause is the self-CC (Bind/BlockAttack/BlockSkill) or the action ending.
  - Now the skill2 action itself is 96 ticks with the tag 'choke_seq': choke frames for 62 ticks, then the skill2 spin-cut frames twice, timed to the native slashes at 68/86. The tag is built by vader_seq.py from existing frame rects, applied to sprites-data.js and to the device fanim. The self-CC is removed.
  - BA: data start_timing 2 sets the vader_ba marker plus vader_ba_target, and the RangeEffect is Delayed 10 (it lands at tick 12 as before). Native draws ba_line0..7 (32000x12000 box, 8 directions) at 16000 ahead for 10 ticks.
- **David:**
  - S2 tiers: 0-34 pistols; 35-69 cyber dash (MoveToTarget 12000) slam with dv_minigrav (entity_pull 2400x10 within 25000, grav_mini fx); 70+ / psycho = rush grab. dv_t3 and the 30-74 grab are gone; dv_t2 = ico_rush.
  - Caution: -25% at 70+, -50% in psycho.
  - Sandevistan versions while on: v2 at 80+ (dv_sv2: +20 move speed, +20 attack speed, -15 attack, red fire_red on the bar), v3 in psycho (dv_sv3: +45, +40, -25, blue fire_blue).
- **V1:** the coin flies to 60% of the distance toward the target (14000-40000) over 18 ticks; coin_from is stored for the flight.
- **Minato:** S1 cooldown 7 s.
- **Roster guide PDF:** TFM2_Skill_Lab_Roster_Guide.pdf, built by build.py + roster.py (content) in the session scratch. It has a balance review page:
  - Gojo strong; V1 strong-leaning (split shot); David, DIO and Vader balanced; Minato and Frieren weak-leaning.

## Oct 1: round 11 (native 0.4.21, tfm2_cyberpunk 0.1.11): David vs Gojo
- Rian: David can't beat Gojo at all. Cause: every David S2 tier and the rush grab explosion is skill damage, which Infinity blocks 100%; the explosion still hurts David, and each wasted S2 adds +10/+15 cyberpsychosis (faster hallucinations). Only his basic attacks worked, at -30%, through a 250 + 60% AP shield. The domain then seals him (Void Overload ignores CC immunity).
- Fix (native, David passive): while dv_sande is on, BA damage on a gojo_infinity target is x10/7 (undoes the 30% cut). The 3rd Sandevistan hit on that Gojo within 180 ticks shatters Infinity: entity_clear_shield + remove gojo_infinity, so Gojo's data marks it broken (15 s down) on his next action. The gravity smash shatters it too. Neither applies while Gojo's domain is open (gojo_void_active).

## Oct 1: round 12-13 (tfm2_jjk 0.1.26, tfm2_ultrakill 0.1.10)
- **Gojo Infinity nerf:** skill_damaged_reduce 100 -> 75 (skills chip the shield now); shield 250 + 60% AP -> 200 + 50% AP (preset infinityShield/infinityAp). Gojo lives in tfm2_jjk as tfm2_custom_gojo (generator: id tfm2_custom_gojo, modId tfm2_jjk; byte-identical check passed).
- **V1 parry only while S2 is ready** (Rian: "why does v1 parry when s2 is on cd"; the guard armed by a cast stayed up through that cast's 10 s cooldown).
  - S2 now: duration 4, start 1. Effect: if v1_parry_cd, nothing; else arm v1_guard (if not already) plus v1_parry_haste (skill_cooldown_mult +1100 for 5 ticks), so that cast's own cooldown is ~50 ticks. The AI re-casts about every 0.9 s while armed.
  - After a parry (v1_parry_cd 600) the next cast does nothing and gets the full 600 cooldown = the lockout.
  - The haste is 5 ticks, too short for any other action's cooldown start (cooldowns start after the action).

## Oct 1: round 14 (native 0.4.22, tfm2_custom 0.1.67): Minato fights at range
- Rian: the AI still mostly basic-attacks with Minato. The game AI casts skills when they're ready and basic-attacks otherwise, so the fix is to make the BA itself play like Minato (Rian picked "kunai BA + hit-and-run").
- **BA:** TargetProjectile kunai, range 50000, speed 7000, 70% AD, plus AddBuff minato_mark (240 ticks, shown with the kunai_marker art). With the Rasengan charged it also adds minato_ras_go (6). In KCM: a piercing LinearProjectile (85% AD, 65000). Tags are now AD/CC/Range.
- **Native:**
  - minato_ras_go on a champion while minato_rasengan is up → flash behind them (BEHIND 9000, pull_back), Rasengan (RAS formula with stacks, stun 45, V1 parry check), schedule the return.
  - A teleport request (not escaping) prefers the weakest marked enemy within 160000 whose spot behind them is safe, tower_ok and has fewer than 3 enemies around; it uses the mark up and runs arrive(). This comes before the kunai pick and the rethrow.
  - Hit-and-run: arrive() (when not low) stores "rret:<origin x>:<y>:<due>" and shows a seal at the origin. 24 ticks later he flashes back if the origin is safe, tower_ok, has fewer than 2 enemies around and isn't a wall.

## Oct 1: round 15 (native 0.4.23, tfm2_jojo 0.1.9): DIO lunge visible, no tower siege
- Rian had never seen the lunge. LUNGE_SPEED 9000 crossed its 80000 range in about 9 ticks (4-5 drawn frames), and the wind-up looked like an ordinary punch.
  - Now LUNGE_SPEED is 4500 (~18 ticks). The wind-up draws the 'world' poses 0..3 (arm back, the flare at the fist growing, trembling at the end), plus a stand_charge fx (gold streaks gathering) at the fist for 60 ticks.
  - The lunge leaves a stand_ghostr/l afterimage every 2 ticks (4 fading frames, 9 ticks) so it reads as a streak.
- Rian: with Stand Out's long BA range he sieged towers from far away. Fix: on_attack at a tower (is_tower) switches to Stand In at once (bypassing MODE_CD) and sets tower_until = tick + 90; the mode rule holds Stand In while tower_until is in the future.

## Oct 1: round 16 (native 0.4.24, tfm2_jojo 0.1.10): DIO Stand Out lifesteal, lunge area
- **Lifesteal in Stand Out:** the only mod lifesteal is dio_in (vamp 12), which set_mode removes. The rest came from items: item_setting has vamp on ruinous_blade 5, conquerors_greatsword 10 and warlords_final_judgement 15.
  - Now dio_out carries vamp = -(his items' vamp). item_vamp() reads player.item_keys(). refresh_out_vamp re-applies the buff every 30 ticks if the total changed (after a purchase), so Stand Out stays at 0 lifesteal.
- **Lunge area:** during the wind-up, every 6 ticks the native draws lunge_area<k>_<stage> (8 directions x 6 stages). It is a gold strip 80000 x 28000 from under the Stand toward the target, filling outward, with a ring under the Stand. It is centred 40000 out and re-aimed each time, so it tracks the target.

## Oct 1: round 17 (native 0.4.25, tfm2_jojo 0.1.11): DIO per-mode cooldowns, dash "!"
- **Dash "!" is back:** Stand In S2 puts the red 'mark' fx on the target (fx_on, bound with follow) and publishes "dmk:x:y:t0" (danger zone for Minato). The dash/strike fires DASH_DELAY = MARK_DELAY (36 ticks) later. Automatic mode switches wait while it's pending.
- **Per-mode cooldowns** (Rian: "2 different cd for each mode", combo S2 dash, S1 knife, change, S2 lunge, then defense):
  - The native keeps rdy[4] (S1 In/Out 600, S2 In/Out 660) and shows them as named buffs dio_r1in/r1out/r2in/r2out while ready. The data reports uses with dio_c* markers.
  - Data branch: this mode's version ready → use it (+ dio_haste skill_cooldown_mult +1100 for 10 ticks if the other mode's version is also ready, so the slot's real cooldown is ~50-55 ticks). Otherwise, if the other mode's version is ready → dio_sw_in/out (the native switches stance at once, bypassing MODE_CD) + that version. Otherwise nothing.
  - Slot cooltimes: S1 600, S2 660.
  - run_dio's knife check ignores dio_out when dio_sw_in / dio_in is present (same-tick switch order).

## Oct 1: round 18 (native 0.4.26, tfm2_frieren 0.1.12): Frieren mid damage
- Rian's scoreboard: Frieren mid (level 12, 85 CS, 8/0/5) did 28,497 magic damage; the next best on the board was 14,853 and most players had about 5-6k.
- **Suspected cause:** Fern's native beams are spawn_projectile with penetrate: true. If the engine re-applies a piercing projectile's effect on every tick it overlaps a target, the big beam (radius 18000, speed 7000) hits a close target ~5 times. The data BA LinearProjectile is engine-run and assumed to hit once.
- **Fix:** first_hit() puts a marker buff on the target ("zlt<caster>" 45 ticks / "zlp<caster>" 30 ticks); Zoltraak and ZoltraakParty skip targets that already carry it.
- **Plus nerfs:**
  - Fern 55 + 80% → 50 + 70% AP.
  - Party Fern 35 + 50% → 30 + 45%.
  - BA 15 + 50% → 15 + 40%.
  - Magic power growth 20 → 16 per level.
- If she is suddenly weak after this, the multi-hit was the real cause and the ratio nerfs can be rolled back.

## Oct 1: round 19 (native 0.4.27, tfm2_ultrakill 0.1.11): V1 nerf
- Scoreboard after round 18: Frieren mid was back to 12,841 (in line with the board). V1 mid against her did 26,970 at 11/1/4, level 12. She feeds him parries: her shots count as "dangerous" during Limiter, and each parry gives power-ups plus a shotgun.
- **Nerfs:**
  - Coin ricochet 70 + 100% AD → 40 + 60% AD on top of the shot. The auto-shot uses 60% of his attack instead of 100%. The split shot gives each enemy 60% (was 100%).
  - Shotgun pellets 28% → 20%; the second pump hits at 60%.
  - Parry power-up +15..70% attack → +10..40%; attack speed is now half of that (was 3/4).
  - AD growth 27 → 24.
- **Unverified:** whether on_attack's *damage = 0 actually stops the pistol projectile's own hit when the shot goes into the coin or shotgun. If it doesn't, those shots double-dip.

## Oct 1: round 20 (native 0.4.28, tfm2_ultrakill 0.1.12): V1 Railgun nerf, shotgun restored
- Rian: "the shotgun is fine but the ult is too much".
  - Shotgun restored: pellets 28%, second pump at full damage.
  - Railgun nerfed: 70 + 120% AD → 40 + 80% AD; width (radius) 10500 → 8000; cooldown 40 s → 55 s. Still 3 piercing shots within 15 s, range 260000.
- The round 19 coin, parry and AD-growth nerfs stay.

## Oct 1: round 21 (native 0.5.0, new mod tfm2_blockcraft 0.1.1): Steve
- **New champion:** Rian's "Steve", a tank support. The look is an original blocky builder made by sprites/blocky.py: square head, rust-red shirt, charcoal trousers, pickaxe. Rian asked for a Roblox/Lego-like blocky style; nothing is copied from Minecraft.
  - Category Util; tags Tank/CC/Shield/Melee. Stats 62 AD, 1150 HP, 40 armour, 35 MR, 1000 move speed.
  - BA: pickaxe 100% AD, no tower bonus.
- **Native module steve.rs** (passive tfm2_custom_ai:steve; data markers stv_s1 / stv_rod / stv_boat):
  - **S1 cycle** pearl → TNT → golden apple (icon stv_tool0..2 over his head).
    - Pearl: 70000 max, lands 9000 short of the target. When low he flees 60000 instead; with a tethered enemy he throws it toward his team and the pull restarts, so the enemy is dragged along.
    - TNT: lands 12000 behind the target (seen from Steve), so the blast knocks them toward his team; it lands in front (peel) when he's at 50% HP or less or outnumbered within 40000. It lies unlit (tnt_idle) up to 15 s and is lit when Steve or a teammate starts a basic attack within 22000 (detected from the attack cooldown jumping), then a 60-tick fuse with a red ring, then 40 + 6% max HP in radius 26000 plus a 3000x12 knockback.
    - Apple: 60 + 8% max HP heal plus a 10% max HP shield for 4 s, on the ally he just hooked or the lowest-HP% ally within 60000.
  - **S2 rod**, charged: 50000 instantly up to 110000 over 60 ticks; -40% move speed while charging; charge bar stv_ch0..5. It releases once the reach covers the distance to the aim + 8000 (tracking a moving aim).
    - Aim priority: his landed TNT near a fight (thrown like a bomb, blows up as it lands) → an ally at 40% HP or less with an enemy within 25000 (yanked to him; his next apple goes to them) → when low, a wall away from the enemies (grapple) → the enemy target (20 + 2% max HP, entity_pull 2800x30, tether).
  - **Ult boat:**
    - The cut line runs 18000 behind the target, across its way home (toward its team's towers), 100000 long. He hops onto the line if its start is within 45000; otherwise he rides from where he stands toward the line's centre.
    - The ride is 2500/tick with cc_immune (stv_riding). A wall builds behind him, from the start to 8000 short of the end, and stands 5 s after it's built ("sbw:ax:ay:bx:by:t0:tb:tend" on Steve). Ice under it gives allies +25% move speed.
  - **Match hook enforce_walls:** keeps each unit's last position (keyed per match by sim.seed(), since matches simulate in parallel, and only while a wall is up). A unit that crosses or ends inside the built wall is set back; one the wall rose under is pushed out to its side. It also marks swb_blk.
  - **Input AI WallAi:** only retargets MOVE inputs whose straight path crosses an active wall, to a point past the nearer useful end (checked with is_valid_input and the terrain grid). It never touches casts or attacks (the Sep 30 plan_legacy panic rule) and does nothing when no wall is up.
- **Mod files:** produced through the Skill Lab e2e (preset 'steve', folder ['tfm2_blockcraft','Blockcraft']); validation passed. Rian has to enable "Blockcraft" in the Mod Manager.

## Oct 1: round 22 (native 0.5.1, tfm2_blockcraft 0.1.2): crossing the wall, map awareness
- **Crossing:** enforce_walls lets any per-tick jump of BLINK_PASS (6000)+ through, so teleports and fast dashes go over the wall like a blink over terrain. Walking (under ~2500/tick), knockbacks (3000-3500) and pulls (2800-3500) are still blocked.
- **wall_need(sim, all, me, tick, reach)** (steve.rs) returns the most urgent thing on the other side of a standing wall:
  1. A teammate at 55% HP or less with an enemy within 30000, or outnumbered within 35000 (lowest HP first).
  2. An allied tower with an enemy champion within 30000.
  3. A neutral non-minion entity on no champion's team with 2500+ max HP and more enemies than allies within 30000 (a big objective).
- **Users:**
  - Minato (raijin::run, before the Void reflex; rwx_cd 240): flashes to a landed kunai within 45000 of the need on the far side (safe, tower_ok, fewer than 3 enemies), with arrive() including the hit-and-run. Otherwise he throws a pack kunai at the enemy nearest the need; it flies over the boat wall and auto-flashes on landing.
  - DIO: the Stand In dash targets the enemy nearest the need (within 40000 of it, 100000 of DIO).
  - Steve: the pearl throws toward the need (when not low and not tethering).
- No AI casts are injected: these all happen inside abilities the game already decided to use, or in Minato's own native auto-moves.

## Oct 1: round 23 (native 0.5.2, tfm2_blockcraft 0.1.3): Steve's S1 is situational
- Request: "can steve s1 be conditional, not in a cycle".
- steve.rs `choose_tool()` replaces the pearl→TNT→apple cycle. It is evaluated on every cast and every 10 ticks for the icon over his head, which now shows the tool he'd use right now. Priority:
  1. Enemy on his fishing line → pearl (drags them toward his team).
  2. wall_need (an ally / tower / big neutral in trouble behind a wall) and he isn't low → pearl.
  3. Ally he hooked in the last 120 ticks, or an ally at ≤45% HP within 60000 with an enemy within 30000 of them → golden apple.
  4. He's low with enemies near → pearl away if outnumbered within 35000, otherwise apple.
  5. Target with 2+ enemies bunched within 30000 of it, or target within 60000 → TNT.
  6. Target farther → pearl; no target → TNT.
- S1 tooltip rewritten to match. The data_champion is unchanged; only the i18n changed.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round23. Rian needs to run build.bat.

## Oct 1: round 24 (native 0.5.3, tfm2_blockcraft 0.1.4): Steve's boat boxes in fights, jumps off midway; rod ×2 through walls; ult ×3 size, 10 s, ×4 range
- Requests: "make sure steve ult can jump midway so doesnt have to do it along the wall, and make sure that steve actually uses it to encapsulate fights or objectives"; then "2x range for fishing rod and pierce wall"; then "3x size for steve ult, and it stays for 10, and 4x the range".
- **When he casts:** the data ult now targets EnemyChampionInCC (range 480000). While the ult is ready, every 20 ticks plan_ult looks for a setup; if it finds one, the key enemy gets a 2-tick Bind so the AI presses it. This is Vader's trick, with no injected casts. Any other CC on an enemy can also trigger it; then the plan is relaxed.
- **plan_ult (centre ≤ 340000 away):**
  - **Fight:** an enemy group his side can take: as many of his team within 90000 as there are enemies, a tower dive, or a lone enemy at ≤50% HP with 2 of his team. The box is three walls, open toward his team (`our_side`). R = 3 × (spread + 14000), clamped to 72000–96000.
  - **Objective:** a big neutral (2500+ max HP). Lock it when his team is on it and an enemy is within 250000; trap the enemies when they're on it and as many of his team are within 120000. R = 84000.
  - **Peel:** he's outnumbered or a teammate is dying: a straight 270000 wall between the teams.
- **The ride:**
  - A path of several segments, each its own `sbw:` buff, timed as the boat passes. RIDE_SPEED is 6000; the walls last WALL_LIFE 600 ticks (10 s).
  - He hops to the start when it's within 60000 or the way is blocked; otherwise the boat rides there first and builds no wall on the way.
- **Jumping off midway:**
  - At the path point nearest the jump spot (inside the box, behind the enemies or by the monster), or at once to a teammate ≤35% HP with an enemy on them. JUMP_T 14, JUMP_MAX 60000.
  - He stays aboard when he's low, and always on Peel. The empty boat finishes the wall.
- **WallAi:** detour_all picks a wall end reachable without crossing any wall, so units leave a box through its open side.
- **Rod:**
  - HOOK_MIN 100000 and HOOK_RANGE 220000 (data range 220000). The hook flies through terrain and boat walls; only the low-HP grapple still catches walls.
  - Whatever it catches is reeled in to about 10000 from him. That's the game's pull when the line is clear of terrain; through terrain it's moved by hand and set down on free ground. The pull lasts dist/speed ticks, capped at 60 (enemy) or 50 (ally).
  - Reeled units carry stv_reeled, which enforce_walls lets through boat walls.
- Unit test steve::tests::box_and_detour (box layout, back-middle point, detour out through the open side).
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round24. Rian needs to run build.bat.

## Oct 1: round 25 (native 0.5.4; tfm2_blockcraft 0.1.5, tfm2_ultrakill 0.1.13, tfm2_frieren 0.1.13)
- **Requests:**
  - "make the block visuals stay on the map until it wears off, and simulate the fog of war … the wall its a straight and cant bend, so its a very strategic skill which needs deep understanding of macros". For the fog question Rian picked "Both": the wall blocks sight, and the AI respects fog.
  - "steves pull make it larger but slower and remove 30% range, faster grab, and if using pearl, dont pearl alone into the enemies, prioritize supporting teammates, and make the AI really smart in macro".
  - "nerf v1 damage overall … and nerf frieren, … constant barrage of zoltrakk".
- **Engine finding:** an Animation view effect lasts as long as its frames, whatever `time` it's played with. wall_block (0.2 s) and ice (0.1 s) had vanished almost at once. Now both are 0.5 s and every built block is re-placed every 30 ticks until the wall falls (stops 12 ticks before), and all blocks crumble (wall_rise) at tend.
- **Steve ult, one straight wall** (no more box):
  - **Laying it:** lay_line runs each half out from the centre to terrain or the map edge, up to WALL_LEN 300000. An end resting on terrain counts toward `sealed`; sealed 2 means the passage is shut. Score +10 per sealed end, and a fully sealed wall may be as short as 25000.
  - **Candidates:**
    - Cut: behind a favourable enemy group, across its way home; +3 per unseen enemy, since they would come that way.
    - Split: between a fight his side wins and the enemy reinforcements.
    - Lock: between a monster his team is on and the visible threats, or their base when 2+ enemies are unseen.
    - Trap: enemies on a monster, wall across their way home.
    - Peel.
  - **Plan fields:** UltPlan.axis points from his side to the far side. He jumps off at the jump point (Cut: behind the group; Split: fight side; Lock: monster; Peel: his side), or to a dying teammate. When low he jumps 25000 to his own side at once.
- **Fog of war:**
  - plan_ult, pearl_spot, choose_tool and wall_need (Minato / DIO / Steve crossings) count only enemies with sim.is_visible(team, id).
  - wall_fog runs in the match hook every 2 ticks: a champion whose every enemy eye within SIGHT_R 110000 (champions, towers, minions of the champion teams) is across a standing wall gets entity_set_invisible(3). Steve riding the boat is exempt.
- **Rod:** HOOK_MIN 70000, HOOK_RANGE 154000 (−30%). HOOK_SPEED 4200 (was 6000), HOOK_HIT_R 16000 (was 9000), hook sprite drawn 3× with a catch ring. Reel PULL (4200, 40), RESCUE (5000, 35). The ally rescue now comes before the TNT fling.
- **Pearl (pearl_spot):**
  - The order is: tether, wall_need, low escape (away from enemies plus toward home), support (beside a fighting teammate more than 35000 away, on the side away from the enemies), then the target.
  - He goes for the target only with a teammate within 40000 of the landing spot and enemies there ≤ that count + 1. With no good spot, S1 throws TNT instead.
- **WallAi for Steve (moves only):** a move to a spot with 2+ visible enemies within 35000 and no teammate within 40000 is redirected to a fighting teammate, 6000 toward home from them.
- **V1:**
  - Attack 94 + 20/lvl (was 108 + 24).
  - Shotgun 22%/pellet (was 28%), second pump 70% (was 100%), so a full double pump at point blank is 6 × 22% × 1.7 ≈ 224% of a basic attack (was 336%).
  - Parry buff 10–25% (was 10–40%).
  - Coin 30 + 45% AD (was 40 + 60%), split shot 50% (was 60%).
  - Railgun 35 + 65% AD (was 40 + 80%).
- **Frieren:**
  - Basic-attack Zoltraak 10 + 30% AP every 140 ticks (was 15 + 40% every 110).
  - Limiter: +25% magic power / +15% attack speed (was +40 / +30).
  - Fern's Zoltraak 40 + 55% AP (was 50 + 70%).
  - Party Zoltraak 24 + 35% AP every 4 s (was 30 + 45% every 3 s).
- **Generation:** V1 and Frieren data via gen.js (r9/new25; the other four folders are byte-identical to new20); blockcraft via the Skill Lab e2e.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round25. Rian needs to run build.bat.

## Oct 2: round 26 (native 0.5.5; tfm2_starwars 0.1.10, tfm2_jojo 0.1.12, tfm2_jjk 0.1.27, tfm2_blockcraft 0.1.6)
- **Request:** "darth vader animation still isnt loaded, ive never seen dio stand uses lunge, the sprite yellow line isnt loadded also with dash strike, dio ult make it so that 1 other teammate is not freezed same as gojo ult and both gets cdr and timestop buff, steve travel slow on the wall and make the wall bigger in size, dio stand on BA range smaller, boost pull strength on blue gojo".
- **Vader:**
  - The choke_seq tag exists in the mod's fanim (18 frames, 1.6 s) and the data asked for it, yet it never showed.
  - Hypothesis (unconfirmed): the game only plays action animations with standard tag names.
  - Fix: skill2's action_name is now 'skill2', and the sprite's skill2 tag carries the choke_seq frames, in sprites-data.js and in the mod's fanim (edited in place on the PC). If it still doesn't show, the next suspect is the game fitting or skipping long action animations.
- **DIO lunge never seen:**
  - Likely cause: in Stand Out the guard is usually armed, and every blocked shot (and the melee counter, and the dash's Pop) replaced the Stand's act, wiping the 1 s wind-up.
  - Now Block, Counter and Pop never replace Windup / Lunge / Grab. A counter during a lunge still stuns and deals a single hit.
- **Dash strike visuals:**
  - The "!" animation is 0.42 s but the wait is 0.6 s, so it's now replayed every 24 ticks.
  - The gold strip (lunge_area sprites) is now also drawn from DIO toward the target during the wait, filling up.
- **DIO Stand Out:** basic attack range bonus 18000 (was 32000; reach 41000). Stand Out switches in at 38000 (was 45000).
- **ZA WARUDO:**
  - The teammate closest to DIO inside the field (picked once, stored as dtg<id>:<guest>) isn't frozen.
  - DIO gets dio_world and the guest gets dio_world_guest for the time stop: +100% skill_cooldown_mult, +20% move and attack speed. The guest also gets −40% attack and cc_immune.
- **Steve:** RIDE_SPEED 2500 (was 6000). Blocks twice the size: STEP 15200, sprite 16×68, ice 20 px, WALL_HALF 8000, ice band 16000.
- **Gojo Blue:** black-hole pull 4800 / 6000 from level 3 (was 3200 / 4000), radius 32000 (was 26000; the art is about 34000).
- **Generation:** starwars / jojo via gen.js (r9/new26; other folders byte-identical), Gojo via gengojo.js (r9/gojo26; the round-25 presets reproduce the PC's file exactly), blockcraft via the Skill Lab e2e.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round26. Rian needs to run build.bat.

## Oct 2: round 27 (native 0.5.6, tfm2_blockcraft 0.1.7): Steve's wall longer, over terrain, the boat rams
- **Request:** "much longer wall, can go through the walls map, knocks when the boat hits an enemy".
- **Length:** WALL_LEN is 600000 (was 300000). At RIDE_SPEED 2500 that's a 4 s ride; the wall then stands for 10 s once built.
- **Terrain:**
  - lay_line no longer clips at terrain; only the map edge (2000 in from the border) cuts it.
  - `sealed` now counts the halves that pass over terrain (sampled every 4000), so a wall that closes the corridor it crosses still scores +10 per half.
  - Steve rides straight over terrain (stv_riding keeps him exempt). If the boat ends on terrain he's stepped back along the wall onto free ground; jump spots already use pull_back.
- **The boat rams:** BOAT_HIT_R 16000; once per enemy per ride. 30 + 4% of his max HP (V1 can parry it), then a ForceMove knock (3500 × 12 ticks = 42000) out to the side the enemy is on; right on the line, to the far side from his team.
- **Unverified:** what the game does with a champion set_pos'd onto terrain cells while riding.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round27. Rian needs to run build.bat.

## Oct 2: round 28 (native 0.5.7, tfm2_blockcraft 0.1.8, tfm2_ultrakill 0.1.14): wall back to 300000, V1 can break it
- **Request:** "make v1 can cancel the wall when parrying the wall, revert back to the 300k range and the nerf but add the goes through walls".
- **Interpretation:** the wall goes back to 300000 long (that's the nerf). It still runs over terrain, and the boat's ram stays. Confirm with Rian if he meant something else by "the nerf".
- **V1:** when the boat rams V1 with his parry armed (try_parry), the whole wall breaks. All `sbw:` buffs are removed, every built block crumbles (wall_rise), the ride ends, and Steve is set down on free ground. V1's parry goes on its cooldown as usual.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round28. Rian needs to run build.bat.

## Oct 2: round 29 (native 0.5.8, tfm2_blockcraft 0.1.9): closing leaks in the boat wall
- **Request:** "sometimes enemies can go through wall". He saw this on native 0.5.5 (log.log 01:21).
- **Likely causes:**
  1. BLINK_PASS was 6000: any move of 6000+ in one tick counted as a teleport and passed. That covered fast dashes (base MoveTo ~5600+) and big knockbacks.
  2. Collision only covered the built line ±3000 at the ends, but since round 26 the end blocks are drawn 7600 past each end, so people walked through the drawn end blocks.
- **Fixes:**
  - BLINK_PASS is 15000: only real teleports and blinks cross. DIO's dash strike, Minato's kunai teleport and Steve's pearl are long jumps and still pass.
  - enforce_walls covers along −WALL_HALF … len + WALL_HALF.
  - crosses() (used by the AI, fog and map awareness) extends the segment by WALL_HALF at both ends.
  - Detour points are 16000 / 26000 past the ends (were 10000 / 20000).
- **Possible visual mismatch, not changed:** the stone sprite stands on the point (stones drawn above it), and unit sprites have their feet about 20 px below their anchor. So the drawn wall may sit visually north of the collision line. If Rian still sees people "in" the stones, shift the stone sprite down to feet level in gen2.py.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round29. Rian needs to run build.bat.

## Oct 2: round 30 (native 0.5.9, tfm2_blockcraft 0.1.10): no wall over terrain, objectives first, no wasted casts
- **Request:** "revert the ult can go through wall, the now is good enough but he should be more focused in objectives until that is necessary, ive seen the obj is ready to take but weasted a wall".
- **Terrain:** lay_line is back to the round-25 version (each half stops at terrain; ends resting on terrain count toward `sealed`). WALL_LEN stays 300000. The ram knock and V1's wall break stay.
- **Objectives first:**
  - While any big neutral monster (2500+ max HP) is alive anywhere, plan_ult only accepts "needed" plans:
    - Lock / Trap (always);
    - Cut with 3+ enemies, a tower dive or 2 low enemies;
    - Split with 4+ enemies involved;
    - Peel.
  - Lock is scored 60+ (was 40+) and Trap 65+ (was 45+), so they beat fights.
- **No wasted casts:**
  - The native rules keep `stv_ult_ok` up (30 ticks, refreshed every 20) while a plan exists.
  - The data ult is now SwitchByBuff stv_ult_ok. Without it, the cast only adds stv_ult_wait (12 ticks, ult_cooldown_mult +4700), meant to cut the cooldown to ~100 ticks.
  - This catches casts set off by other CC on enemies (the ult targets EnemyChampionInCC).
  - **Unconfirmed:** that ult_cooldown_mult works like the skill_cooldown_mult haste trick. If it doesn't, an empty cast costs the full 80 s.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round30. Rian needs to run build.bat.

## Oct 2: round 31 (native 0.5.10, tfm2_blockcraft 0.1.11): the pick, when a teammate's CC makes the AI cast
- **Request:** when the AI presses the ult because of another CC, "watch the macro of where the teammates is, because some of it is actually really useful".
- **New plan, Pick:**
  - `caught(sim, id)`: the enemy has Airborne / Stun / Bind / Taunt / Fear / Charm with cc.tick ≥ 8, so Steve's own 2-tick Bind doesn't count.
  - A visible caught enemy within ULT_REACH, with a teammate (not Steve) within 60000 of it, where his side is there in strength (allies within 70000 ≥ enemies within 50000).
  - The wall goes 25000 from the enemy, square to the way help would come (the visible enemies within 200000, else their base). He jumps off 10000 from the enemy, on the wall side.
  - Score 50 + 10 per teammate on it + 15 if the enemy is at ≤50% HP + 5 per helper + sealed × 10. Needed = true (allowed while an objective is up).
- **Readiness:** stv_ult_ok is now re-evaluated every 2 ticks while any visible enemy is caught (otherwise every 20), so the cast the AI makes at that moment finds the flag up. The 2-tick Bind is only applied on the 20-tick cadence when nobody is caught.
- **Unconfirmed:** whether cc.tick is the remaining or the total duration.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round31. Rian needs to run build.bat.

## Oct 2: round 32 (native 0.5.11, tfm2_blockcraft 0.1.12): faster TNT, the hook bites walls
- **Request:** "the tnt took too long to explode, make the fishing rod to not go through wall so the steve can dash to the wall".
- **TNT:**
  - Lit as it lands: TNT_FUSE 36 (0.6 s; it used to lie unlit until a swing, then 1 s).
  - A swing by Steve or a teammate within TNT_TOUCH_R sets it off in TNT_QUICK (6 ticks).
  - The block and its ring are replayed every 12 ticks (their animations are only 0.2 / 0.48 s).
- **Rod:**
  - In every mode, a hook that reaches terrain or a boat wall (the drawn span, ±WALL_HALF at the ends) bites there, and Steve grapples to it: 6000 short of terrain, or WALL_HALF + 4000 short of a boat wall, pulled back onto free ground, for up to 45 ticks.
  - Nothing is reeled through walls any more. HookMode::Wall is still how the low-HP escape aims.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round32. Rian needs to run build.bat.

## Oct 2: round 33 (native 0.5.12, tfm2_jojo 0.1.13): DIO stops lone time stops, blinks in first
- **Request:** "reduce dio solo ulting or atleast make him TP first where its quite close to the enemy then ult".
- **ts_plan (batch2.rs):**
  - Visible enemies within TS_REACH 170000. For each, a group within 4/5 of TS_R.
  - The spot is 15000 in front of the group (on DIO's side), or where he stands if he's within 25000.
  - Worth it: caught (within TS_R − 6000 of the spot) ≥ 2, or caught ≥ 1 with a teammate within TS_MATE 110000 of the spot or an enemy at ≤40% HP. Never a spot within 60000 of an enemy tower with fewer than 3 caught.
  - Score: caught × 10 + mates × 5 + low 8 − distance / 20000.
- **The gate (same pattern as Steve):**
  - DIO's on_update re-evaluates every 20 ticks while the ult is ready: dio_ts_ok (30 ticks) plus a 2-tick Bind on the key enemy.
  - The data ult targets EnemyChampionInCC and is SwitchByBuff dio_ts_ok. Without it, the cast only adds dio_ts_wait (ult_cooldown_mult +4700), the same unverified ult haste as Steve's.
- **Blink in:** at the field's creation in run_dio, DIO blinks to the ts_plan spot (vanish → appear) when it's more than 12000 away. The field, the Stand and the guest pick are centred on the spot.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round33. Rian needs to run build.bat.

## Oct 2: round 34 (native 0.5.13, tfm2_jojo 0.1.14): DIO's time-stop blink capped at his basic attack range
- **Request:** "170k is too far, makes it his BA range max tp".
- **Change:**
  - TS_BLINK = 23000 + OUT_RANGE_BONUS = 41000 (his Stand Out basic attack reach).
  - The spot is DIO + direction × min(distance − 15000, 41000), pulled back onto free ground.
  - Groups count only within TS_REACH = TS_BLINK + TS_R (about 117000). Whether the field catches them is still judged at the spot (TS_R − 6000).
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round34. Rian needs to run build.bat.

## Oct 2: round 35 (native 0.5.14, tfm2_blockcraft 0.1.13): more walls again; Steve calls his team in
- **Request:** "now theres less wall, maybe keep it before, and maybe for the team that has steve, go engage what is the wall is used for, steve communicate to the team and maybe engage on the situtation if its a winning wituatuin after that wall".
- **More walls:** the round-30 objective hold (`needed` / `obj_up`) is removed. Lock (60+) and Trap (65+) still score highest, so they win when on offer. The stv_ult_ok gate, the Bind trick and the Pick plan stay.
- **The call:**
  - UltPlan.rally = (spot, enemy to follow). Cut: the group centre; Pick: the caught enemy (followed); Split: the fight centre; Lock / Trap: the objective; Peel: none.
  - At the cast, if rally_good, Steve gets the buff "stv_rally:<x>:<y>:<id|-1>:<until>" (until = ride + WALL_LIFE).
  - **rally_good:** his team within RALLY_R 220000 of the spot and above 30% HP, at least 2 of them and at least as many as the visible enemies within 70000 of the spot that no standing wall cuts off.
  - Re-checked every 30 ticks (dropped when no longer winning), with a "warp" ping at the spot.
- **WallAi (moves only):** a teammate (not a Steve) above 30% HP, within RALLY_R of the rally target and more than 20000 from it, whose move destination isn't already within 25000 of it, is sent to the target. Wall detours still apply afterwards. Casts and attacks are never touched.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round35. Rian needs to run build.bat.

## Oct 2: round 36 (native 0.5.15; tfm2_blockcraft 0.1.14, tfm2_cyberpunk 0.1.12): Steve plays TNT with the rod and reads his rod line; David's area gravity
- **Requests:**
  - "the tnt is used but isnt really effective, smallewr range or just he is not using the fishing rod to play".
  - "when davis use the ult with no lock on, make it have a bit bigger range and faster conjure but less stuns".
  - "make steve be able to know that this is a dash to a wall or a pull".
- **TNT:**
  - TNT_R 32000 (was 26000), TNT_OFFSET 8000 (was 12000).
  - Thrown where the target will be: last-tick velocity × (TNT_FLIGHT + TNT_FUSE / 2); Steve.seen holds last tick's enemy positions.
  - **On landing:** an enemy within TNT_R → lit, TNT_FUSE 30. Nobody → a mine for TNT_MINE_LIFE 300, set off (TNT_MINE_FUSE 12) by an enemy within TNT_TRIGGER_R 20000 or by a swing.
- **Rod + TNT combos:**
  - With an enemy on the line and his side not outnumbered within 45000, choose_tool → TNT, dropped 14000 out toward the reeled enemy (throw_tnt).
  - The rod prefers an enemy whose pull path passes within 14000 of his TNT.
  - The rod flings a TNT that would miss (no enemy within TNT_R, one within 70000).
- **Rod line awareness:**
  - line_block() samples the line (terrain + boat walls incl. ends): None = it pulls; Some(p) = it dashes to p.
  - dash_ok(): at least 20000 away; if low, only with no visible enemy within 40000 of p; otherwise foes ≤ mates + 1 there.
  - **cast_rod order:**
    1. Ally with a clear line (pull).
    2. Ally behind a wall → dash, if it brings him closer and dash_ok.
    3. TNT fling (clear line).
    4. Low escape dash.
    5. Pullable enemy (clear line: over-TNT first, then the data's mark, then the nearest).
    6. The data's target behind a wall → engage dash if dash_ok.
    7. Otherwise no hook.
- **David's area cast (no lock-on):**
  - GRAV_R_AREA 40000, GRAV_DELAY_AREA 36, GRAV_TICKS_AREA 90.
  - New sprites crosshair_area / gravity_area (gen2.py crosshair(r) / gravity(r)).
  - danger_zones reads dvz v[3] == 0 as area → GRAV_R_AREA. The locked cast is unchanged (30000 / 60 / 150).
  - The cyberpunk sheet and fanim were committed from /home/claude/vfx (pixel-identical on the PC).
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round36. Rian needs to run build.bat.

## Oct 2: round 37 (native 0.5.16, tfm2_starwars 0.1.11): Vader's force choke is harder to dodge
- **Request:** "vader 2nd skill buff range, hes not doing good lately, people keep dodging the 2nd skill, can you make the triangle follow the target and it has bigger area".
- **Likely causes:** the wedge snapped to 8 directions (45° steps) with a ±0.6 rad cone, so a target near the edge of a snapped wedge was missed; plus 45000 cast range against 55000 length.
- **Changes:**
  - CHOKE_LEN 70000, CHOKE_HALF_ANGLE 0.8, aimed at the exact angle to the target.
  - The aimed target is always caught within CHOKE_LEN + 12000.
  - Wedge art in 16 directions (gen2.py, 70000 / 0.8; draw_wedge picks the nearest), redrawn every 20 ticks through the hold, pointing at the first choked enemy.
  - Data chokeRange 60000 (was 45000); views list choke_v0..15.
- The starwars sheet and fanim were committed from /home/claude/vfx (pixel-identical on the PC). The Vader champion sprite (skill2 = choke_seq frames) is untouched.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round37. Rian needs to run build.bat.

## Oct 2: round 38 (native 0.5.17; tfm2_jojo 0.1.15, tfm2_blockcraft 0.1.15): DIO's empty ults; Steve walls more often
- **Request:** "maybe add a delay on dios ult, sometimes he ult but still misses why, then back to steve wall, remind him to use it frequently, it could alter even the smallest fight, like isolating bot to suppor or wall to gank from mid to jung, or mid wall so we can pass, or just use it as an engage so team can play around the wall".
- **DIO:**
  - Diagnosis: the "misses" were most likely the round-33 empty casts. Any CC on an enemy makes the AI press the ult (EnemyChampionInCC). Without dio_ts_ok the cast did nothing, but the ult pose still played, so it looked like a missed ult.
  - Fix: the data ult is ungated again (every cast is a time stop), with range 110000 (≤ TS_REACH 117000) so the enemy in CC is always within blink + field.
  - At cast, ts_plan(strict) or else ts_plan(relaxed: any spot catching an enemy, no tower rule) picks the spot. +10 score when a stunned enemy is in the field (the time stop chains onto the CC).
  - The Bind trick and dio_ts_ok stay, for good setups.
  - No extra delay was added: the blink already lands him in front of them before the freeze.
- **Steve, more walls:**
  - min_len 90000 (was 120000).
  - Cut needs only a favourable fight (1v1 / 2v2 lane fights and ganks count).
  - Split needs only 35000 between the fight and the helpers (isolate a laner from its support).
  - New Screen plan (score 12+): his team moving together (2+ within 60000), visible enemies within 160000 but none within 35000, so a wall goes between them (pass, rotate, set up). No rally for Screen.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round38. Rian needs to run build.bat.

## Oct 2: round 39 — why data changes never reached the game (custom DB / career copies)
- **Request:** "i dont think the mod updates it, i still see no animation on dio".
- **Finding:** the native DLL is current (log.log: 0.5.17 loaded), but the gameplay JSON of the mod champions comes from copies.
  - custom_db_enabled.flag is on. custom_database.tfm2db (written Oct 1 02:53 UTC) holds a copy of every mod champion it saw then: Minato, Gojo, DIO, David, V1, Vader, Frieren.
  - Every new career / save copies its mod champions from that DB (hashes identical, DIO aside). Champions not in it (Steve) come fresh from the mods.
  - So every data change since Oct 1 morning never reached play for those seven: DIO's per-mode S2 (no lunge, no dash markers), V1 / Frieren nerfs, Vader's range / skill2 tag, David's area ult, Gojo's Blue pull, and so on. Native changes did.
  - The view bindings (view_effects / projectiles / buffs) are NOT in the save copies; they come from the mod files at runtime.
- **Save JSON format:** serde re-serialization of the gameplay struct (mod format minus view_*, plus default fields). An entry is "1HCM" + u32 version + u64 len + JSON, three sheet copies per file.
- **Fix:**
  - `sync_mod_champions.py` (C:\Games\tfm2\Claude outputs\, also $HOME/work/patch.py in the VM) replaces every entry whose id matches a current mod champion with the mod JSON minus view_*, then re-gzips and rewrites the gzip length and CRC in the header.
  - Applied to custom_database.tfm2db: all 7 now match the mods.
  - Also applied to save_20261002_030555.data.bak, but that's an old save. The game was running: Rian had started save_20261002_032514.data at 20:25 UTC from the stale DB, so it still has old copies. Patch it once the game is closed, or start a new career after restarting the game.
- **Unverified:** that the game accepts mod-format JSON (missing default fields) in the save sheets. The mod loader parses the same format, so it should.
- Backups: C:\Games\tfm2\Claude outputs\backup-before-round39 (custom DB + 030555 save).
- **Process rule from now on:** after any data change, run sync_mod_champions.py on custom_database.tfm2db (and on the active save, with the game closed).

## Oct 2: round 40 (native 0.5.18, tfm2_blockcraft 0.1.16): career save synced; Steve's objective wall, escape wall
- **Career save:** Rian closed the game (log: shutdown 20:38 UTC) and named save_20261002_032514.data. sync_mod_champions.py was applied to it and to its .bak; all 8 mod champions now match the mods. The originals are in backup-before-round39.
- **Request:** "also for steve, still remembers the objective wall, he didnt do it, then also add this if you are in danger and being chased and ult is up wiht a low hp friend or not you can use it to run, other way also as an engage is possible too".
- **Objective wall fixes (likely causes):**
  1. Objective detection skipped e.is_minion(). Neutral camp monsters may report as minions; it now skips only minions of a champion team, in plan_ult and wall_need.
  2. ours / theirs at the objective were counted within 40000, too tight for ranged champions; now 70000.
  3. Plans with no enemy his team can see to hold (key None) can't be cast at all (the AI only presses at an enemy in CC). consider() now skips them (unless relaxed), so they no longer block castable plans.
- **Escape plan (score 80+):**
  - Triggers when Steve is chased (an enemy within 50000 and ≤50% HP or outnumbered) or a teammate at ≤40% within 120000 has an enemy within 45000.
  - The wall goes between the runners' centroid and the chasers (within 70000 of them), at 0.6 × the distance (8000–30000) from the runners.
  - He jumps 25000 toward his towers.
  - It outranks fights; Peel stays as the fallback.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round40. Rian needs to run build.bat.

## Oct 2: round 41 (native 0.5.19, tfm2_blockcraft 0.1.17): the wall is always full length
- **Request:** "the wall is always conjured at max range but steve can jump off anytime".
- **lay_line now:**
  - Measures the free run each way (fa, fb, up to WALL_LEN) and lays a wall of min(WALL_LEN, fa + fb).
  - It stays as centred on c as it can, sliding along its line away from terrain or the map edge.
  - Only a passage narrower than WALL_LEN gets a shorter, wall-to-wall wall.
  - sealed = ends that rest on terrain.
  - Unit test: a terrain cell 32000 north of c gives a 300000 wall from y 210000 to 510000, sealed 1.
- The jump-off rules are unchanged (the planned point, a teammate about to die, or at once when low; the empty boat always finishes the full wall). Only the tooltip wording changed.
- No data change, so there's nothing to sync to the custom DB or saves.
- Backup: C:\Games\tfm2\Claude outputs\backup-before-round41. Rian needs to run build.bat.

## Oct 2: Sprite kit for Rian (C:\Games\tfm2\Sprite kit)
- **Request:** "can you make a file that's holding all of the sprites so can try to design one".
- **Built on the PC with kit.py** (stored in the kit folder):
  - Every mod sheet (champions + vfx) and all 78 base champion sheets from bundle.game_data (champion entries with '#').
  - Each sheet comes as <name>.png + <name>.anim.json + <name>_guide.png: frames boxed and labelled "tag i" with durations; 4x for mod champions, 3x base, 2x vfx.
  - "ALL SPRITES overview.png": the first idle frame of all 89 champion sheets.
- **template.py** writes "NEW CHAMPION template": 48×56 frames.
  - Rows: idle 4, run 6, attack 4, skill1 4, skill2 4, ult 4, hit 1, dead 6.
  - Blank PNG + anim.json + a 4x guide (frame centre cross, feet line 20 px below centre, Steve's idle ghost for scale).
- README.txt explains the files, drawing tips (face right, 1 px outline, feet on the line, 950 units per px for effects) and how to get a sprite in (send Claude the PNG, or the sprite editor's Import PNG per frame).
- **Note:** base frames are trimmed to varying sizes (no offsets in the fanim), while the mod and template frames are uniform.

## Oct 2: Omen and the Valorant folder (native 0.6.0, tfm2_valorant 0.1.1)
- **New mod folder** tfm2_valorant ("Valorant"), champion tfm2_valorant_omen. Native module valorant.rs, passive `tfm2_custom_ai:omen`, native effect `tfm2_custom_ai:omen_blind`. Backups in backup-before-round42.
- **Buy Phase** (native, shared passive for the folder):
  - Credits: start 800, kill +200, assist +100, +150 per 30 s alive, death +300 (gun and armor lost).
  - Buys only within 45000 of where he spawned (respawn or recall). Full buy at 3900+ (Vandal; Operator at 6200+), force buy at 2250+ when behind / after 3 min / 3000+ (Judge vs 2+ close fighters, else Spectre), early Sheriff at 1200+, otherwise eco. Upgrades only.
  - Guns are permanent buffs `omn_gun0..5` (icon over the head). The data BA range is 18000 (Judge); the others add range: Classic +12000, Sheriff +20000 (+35% dmg, -30% AS), Spectre +10000 (-15%, +60% AS), Vandal +24000 (+30%, +15%), Operator +62000 (+200%, -75% AS). Judge damage in on_attack: x2.2 at 10000 falling to x0.7 at 20000, plus 35% to enemies within 9000 of the target.
  - Armor = shield (light 400 credits = 12% max HP, heavy 1000 = 25%), refilled only by buying. Second smoke charge 200 credits (`omn_more`).
- **S1 Dark Cover:** dome radius 26000 for 600 ticks, published as `oms:x:y:end` buffs, redrawn every 30 ticks (0.5 s anim, z 3 over units). Max 3; a re-smoke replaces the one on the spot. Real cooldown 1500; `omn_more` hastes the slot for the bought charge.
- **Smoke vision (match hook valorant::fog, every 2 ticks while a smoke is up; steve::wall_fog stands aside then):** inside = hidden unless an enemy is in the same smoke within 15000; a watcher inside sees only within 15000; lines through a smoke or a boat wall are blocked. A cooldown jump (shot or cast) reveals for 30 ticks.
- **S2:** Shadow Step (36-tick channel with self-Bind, teleport up to 50000) or Paranoia (native Linear projectile radius 26000, speed 3200, range 180000, through walls). Paranoia blinds enemies it touches (via the effect) and teammates on its path (computed natively); the AI only throws it on a line with no teammate within 26000+16000.
- **Blind:** 90 ticks. Shadows = spawn_unit with the victim's enemies' base sprite name (mod champions: "bombardier", overridden by shadow_bombardier), hp 1, Bind + BlockAttack + BlockSkill, moved natively along the enemy's smoothed velocity. The blinded get a 6-tick Taunt onto the nearest shadow every 4 ticks, or BlockAttack when none remain.
- **Ult:** 120-tick channel. Omen is banished (re-applied), a shade (bombardier sprite, 25% of his HP) stands at the destination; if it dies the teleport is cancelled. He smokes the destination first when S1 is ready.
- **Gating:** all three slots are EnemyChampionInCC; every 20 ticks, when a slot has a plan, the nearest visible enemy within 190000 gets a 2-tick Bind. `omn_s1_ok` / `omn_s2_ok` / `omn_ult_ok` gate the effects; empty casts haste the slot (1/12 cooldown; ult via ult_cooldown_mult 4700).
- **Input AI (moves only):** enemies check an Omen smoke in their path (within 70000, unchecked by their team, Omen not visible, HP > 40%) by walking into it; Omen holds still in his own smoke with a Spectre / Judge while 1-2 enemies come.
- **Sprites:** original hooded shadow-caster (omen_sprite.py, 36x44 frames) and the shadow projection (every bombardier tag). VFX sheet "valorant" in gen2.py.
- Omen is new, so existing careers don't have him; try him in an exhibition match first.

## Oct 2: Omen ult spots + the Map tab (native 0.6.1, tfm2_valorant 0.1.2)
- **Rian:** Omen shouldn't smoke his ult destination by default (a smoke warns the enemies); he can ult into bushes, and a smoke is the second option.
  - hide_spot: a Rian ult mark within 60000 first, then a bush cell within 40000 with no enemy on it, then one of his smokes within 45000; only when nothing hides it does the smoke plan smoke the destination first (ult_smoke).
  - The bush grid is now read by WallReader too (`walls::bush_at`, `bushes_near`).
- **Map dump:** WallReader writes the 5v5 map document (`doc.get_json("")`) once per game run to `<game>/mods/tfm2_custom_ai/map_dump.json`.
- **Map editor (editor Map tab, mapedit.js; server /api/mapdump, /api/tactics):**
  - Draws walls / bushes / lanes / fountains / camps / towers / nexus from map_dump.json. Before the first match it shows walls only, taken from the bundle's map_setting (cells with zero baked visibility; the format is Vec<Vec<grid 30x30 of u8>> with u64 lengths).
  - Tools: smoke spot, wall line (drag, clamped to 300000), ult spot, champion token (sketch only), select/move (drag wall ends).
  - Each mark has a champion, a trigger (always / serpen / epic / fight / defend / attack), a side (both = drawn for Blue and mirrored through the two nexuses for Red) and a priority 1-5.
  - Saving writes tactics.json (the editor's copy) and tactics.txt (one line per mark per team) into mods/tfm2_custom_ai, with backups in editor/backups/tactics.
- **Native (tactics.rs):** tactics.txt is reloaded at every match creation, so no career sync is needed.
  - Triggers: serpen / epic = that objective is up (max HP 4000-7999 / 8000+) with a champion within 120000; fight = both teams within 70000 of the mark; defend = 2+ visible enemies within 90000; attack = 2+ allies within 90000.
  - Omen: smoke marks come right after his escape / save-a-teammate reads (within 400000). Steve: wall marks become plan candidates with score 50 + 12 × priority, blocking toward the enemy base.
- Team 0 is assumed to be Blue (nexus_pos[0]). The editor has a "flip vertically" view option in case the map is drawn upside down.

## Oct 2, round 52: Paranoia look, V1 parries it (native 0.6.9, tfm2_valorant 0.1.5)
- **Rian:** Paranoia should be a ball followed by spiralling smoke in a cone (the ball is the tip, the base is smoke); V1 can parry the blind. He also confirmed he wants the AI to realise the vision coverage on its own (round 51).
- **VFX:** paranoia is redrawn pointing right with the ball at the image centre (= the projectile position). A 3R+20 cone of noisy haze behind it, three jittered puff strands on a helix (the near side lighter), wisps at the base; 6 frames. Only the valorant sheet was regenerated (/tmp script regen_valorant.py: valorant() + a vfx2.js bundle patch), because the full gen2 run takes over 2 min.
- **V1:** the reflex spots a projectile from a caster with omn_para_fly at INCOMING_R + 34000 and always treats it as risky. Parried inside the 6-tick window, V1 is put on the Paranoia's hit list (blind() skips v1_parry), so that Paranoia can't blind him.
- The Paranoia tooltip says V1 can parry it.

## Oct 2, round 51: Omen and Steve coordinate vision (native 0.6.8)
- **Rian:** in the Omen + Steve plans a smoke can target a choke a wall already blocks; they should know the vision is already handled.
- **Omen (Omen::vision_covered, applied to every weighed smoke candidate):** no smoke at a spot when
  - a standing boat wall passes within SMOKE_R of it;
  - every enemy who could look at it (visible within 250000, else their base) is behind a standing wall (steve::crosses);
  - or a teammate Steve has his ult ready and one of the team's active wall / block plans (team there) passes within SMOKE_R of it, because Steve will wall it.
  So "both" line-up choke smokes now act as a backup: used only when Steve's wall isn't up or coming.
- Escape, save-a-teammate and the ult-destination smokes are not filtered.
- **Steve:** a wall plan whose middle is already inside one of his team's smokes (2+ s left) scores −15. It isn't dropped (the wall still stops movement).

## Oct 2, round 50: Map tab dragging fixes (editor only)
- **Rian:** sometimes a smoke or wall wouldn't move.
- **Causes:**
  - the mirrored copies (drawn on top, not grabbable) covered plans at the same spot, e.g. the pit smokes, which mirror onto themselves;
  - with Smoke here / Wall here picked, clicking a plan placed a new one instead;
  - the walls' hit area was thin.
- **Fixes:**
  - mirrored copies are drawn underneath and are grabbable (dragging one moves the original, mirrored back);
  - grabbing a plan works with any tool (new plans only go on empty ground);
  - walls have a 30-wide invisible grab line;
  - where plans overlap, the selected one wins, and clicking again without moving cycles to the next one underneath (the status line says how many overlap);
  - drags start after 4 px.

## Oct 2, round 49: Map tab = map-altering champions only, per line-up (native 0.6.7)
- **Rian:** show only the champions with map / vision altering skills (Omen smoke, Steve wall; more later). No general strategy: the team responds to the vision and walls by itself. A checkbox for which of them the team has: only Steve, only Omen, or both.
- **Editor (mapedit.js rewritten):**
  - "Team has" checkboxes (CHAMPS registry: omen → Smoke here / Fake smoke; steve → Wall here). Each plan stores comp (the sorted line-up it was made for); the left playbook and the map show only the current line-up's plans.
  - The right panel holds the tools of the champions in the line-up, plus "Copy plans from" a smaller line-up.
  - Smokes have a Fake flag. Goal kinds, playsets and tokens are gone (older plan types in tactics.json are dropped on load).
- **tactics.txt** gains columns `radius who comp` (who = fake for fake smokes; comp=omen+steve).
- **Native:**
  - tactics::MAP_CHAMPS = ["omen", "steve"] (add the next one here);
  - team_comp() from the team's players (alive or not);
  - marks_for keeps a mark only if its comp is empty or equals the team's comp;
  - Omen's fake smokes skip team_there (weight −10).
- **Playbook v2 (24 plans, side both; generator Claude outputs/map/make_playbook2.py):**
  - Omen only: smoke the pits (serpen / epic 3), fakes on the other objective (1), mid crossing in a teamfight (2), mid tower defence (3), and the four chokes (river from mid 4, their jungle's way 3 at (800000, 600000) / (440000, 230000)).
  - Steve only: the four isolate walls.
  - Both: the four walls plus the smokes minus the chokes.
- Old playbook backed up in backup-before-round49.

## Oct 2, round 48: Map tab in three columns (editor only)
- **Left, Playbook:** plans grouped by situation ("when"). Each situation has a checkbox (all on/off), collapse, and ★ to save it as a playset. Each plan has its own checkbox: unticked plans are kept (off: true) but left out of tactics.txt. Clicking a plan opens its settings inline (Duplicate / Delete). There's a search box.
- **Centre:** the map.
- **Right, Shortcuts:**
  - team plan: Smoke here = cover/fight, Wall here = block/isolate (drag), Fake smoke, Ambush, Flank via, Gank here = group/gank/jungle, Group up, Defend here = control/tower, Push here = control/won, Control area, Avoid = behind, Safe spot;
  - one champion's skill: Omen smoke, Steve wall, Omen ult spot.
  - Picking one shows a "Placing" card with editable defaults; Esc / Done stops.
- **Right, Playsets** (several plans per click):
  - Take objective: on the nearest Serpent / Morgard within 160000, won_X 5 + ahead_X 4 + avoid behind_X + cover X;
  - Contest objective: X_fight control + ambush in the nearest bush + fake on the other objective;
  - Gank lane: jungle + mid;
  - Defend tower: control tower + safe spot 80000 toward our nexus;
  - Teamfight flank: flank via the nearest bush + Omen ambush in the next one;
  - plus "My playsets" (saved situations; positions relative to the first plan) stored in tactics.json.playsets.
- Sketch tokens moved to the bottom of the right panel.

## Oct 2, round 47: starter playbook on the real map (native 0.6.6)
- **Geometry:** the map's team symmetry is the reflection across the river diagonal y = x (the perpendicular bisector of the nexuses), not a point reflection. Morgard (288000, 288000) and the Serpent (672000, 672000) map onto themselves; Blue's camps and towers become Red's. The editor's mirror() is fixed (it used n0+n1−p, which swapped Morgard and the Serpent).
- **Blue's side** is y > x (bottom-left). Top lane runs up the left edge (x 80000) and along the top; bot lane runs along the bottom (y 880000) and up the right edge; mid is the x+y≈961000 diagonal, crossing the river at the centre (480000, 480000).
- **New triggers:** behind_serpen / behind_epic (fewer alive + that objective up).
- **isolate** is now side-agnostic: a fight on one side of the line (both teams within 140000), enemies within 200000 on the other side and none of ours there.
- **Block / wall lines from plans:** Steve builds the free stretch around the line's middle (walls::clip from the mid toward each end), so a line drawn across a gap rests on terrain at both ends and never runs over it.
- **Playbook** (33 plans, all side "both"; tactics.json + tactics.txt installed in mods/tfm2_custom_ai; generator: Claude outputs/map/make_playbook.py):
  - **Serpent and Morgard each:**
    - control on serpen_fight / epic_fight (prio 4), won_* (prio 5) and ahead_* (prio 4);
    - avoid on behind_* (prio 4);
    - cover on serpen / epic (prio 3);
    - block "isolate" on the river choke (prio 4; the Serpent one is 88k between wall cells (16,18)/(18,16)) and a long line cutting their jungle off (prio 3: y=600 x 640-940 for the Serpent, x=370 y 165-400 for Morgard);
    - ambush in a bush near the pit (near1, prio 3);
    - flank through a bush (jungle, prio 2);
    - fake smoke on the other objective (prio 1).
  - **Ganks:** jungle to mid (430000, 530000) prio 3, top (80000, 420000) and bot (540000, 880000) prio 2; mid roams bot / top prio 1.
  - **Defence (trigger tower):** base group at (150000, 810000) prio 5; mid tower control prio 4; top / bot towers near2 prio 3. Safe spot (240000, 720000).
  - **Fights:** after a won fight, push their mid tower (prio 2); flank teamfights via the river bush (528000, 560000) near1; Omen ambush in the mid bush (400000, 432000).
- **build.bat** only copies the DLL and mod.mod_info, so the tactics files survive rebuilds.

## Oct 2, round 46: many more situations, roles, new plan types (native 0.6.5)
- **Rian's common situations:**
  - serpent / Morgard fights;
  - enemies or teammates arriving from base, mid or a lane (late);
  - after a winning fight (a forced objective never succeeds);
  - ganks between lanes and the jungle;
  - teamfights, grouping, surprises, flanks;
  - walls: block the mid-to-objective choke, or isolate a fight from reinforcements;
  - smokes: surprise, smoke over big objectives, deceive.
- **New "when" triggers** (tactics.rs `active`):
  - time: early (<8 min), midgame (8-18), late (18+);
  - objectives: serpen_up / epic_up (alive anywhere), serpen_fight / epic_fight (both teams within 120000);
  - fights: teamfight (3+ each here), won / lost (2+ kills in the last 15 s, more than the other side; from the kill log), ahead / behind (alive counts);
  - here: gank (a lone visible enemy here, 2+ of us within 200000), coming (visible enemies up to 150000 outside the area while we're there), isolate (lines: a fight on our side, determined by our towers' centroid, with enemies within 180000 on the other side), tower (our tower here has an enemy within 30000).
- **New kinds:**
  - cover (Omen smokes the area over; team must be there);
  - fake (Omen smokes it, no presence needed, weight −10);
  - flank (assigned teammates go via the spot when a fight is within r+150000 and they aren't in it; "tfl:x:y" for 900 ticks once within 15000-30000, set by the match hook every 10 ticks);
  - safe (≤35% HP within 300000 fall back to it).
- **Who** (11th field), for control / ambush / flank / group: all, near1, near2, jungle, mid, top, bot, support, omen (from ctx.lane() in WallAi). Non-Omen champions assigned to an ambush hide there (hold) until an enemy enters the area.
- **The real map dump arrived** (Rian played a match):
  - camps carry "ty": Rhino, Mushroom, Stump, Bee, Morgard (288000, 288000), Serpen (672000, 672000);
  - towers carry "ty": Top / Mid / Bottom / TwinA / TwinB / Top2 / Mid2 / Bottom2, pos per team;
  - nexus at (96000, 864000) for team 0 and (864000, 96000) for team 1; fountains [lx, ly, rx, ry];
  - lanes {line, path}; bushes are 30x30 grids with group ids 1 / 2.
  - The editor renders it.
- **Bridge gotcha:** committing a file re-staged at the SAME /mnt/user-data/outputs path after an earlier commit wrote the OLD content. Stage under a new path when re-committing.

## Oct 2, round 45: team strategy in the Map tab (native 0.6.4)
- **Rian:** wanted a general strategy instead of editing each skill.
- **Team plan goals** (champ "team" in tactics.txt; areas carry a radius as a 10th field, default 60000). Each champion carries a goal out with its own kit:
  - **block** (line): Steve walls along it; Omen smokes its middle.
  - **control** (area): Steve lays a wall r+20000 from the centre, square to the enemy approach; Omen smokes r + 0.6·SMOKE_R toward the enemies (or their base); teammates within 250000 move into it (free inside).
  - **ambush** (area): Omen steps there when enemies come by (within r+60000) and lands his ult there (spot first in hide_spot); smoked when it's not a bush.
  - **avoid** (area): moves into it are redirected to just outside.
  - **group** (point): teammates within 250000 go there and hold.
- **Team there:** smokes and walls from plans (per-skill marks included) need someone of the team within r+80000 (areas) or 120000 (lines/spots) of the plan.
- **Weighing:**
  - Omen's smoke choice is now scored: plans 20+15×prio (block 35-95, control −5, ambush −10) vs his own reads (objective 50, ambush 45, fight 40). Escape, save-a-teammate and the ult destination still come first.
  - Steve: block/wall lines 50+12×prio, control 45+12×prio (+5 sealed), vs his own plans.
- **Banned champion:** the goal is simply done by whoever can. Moves (avoid/group/control) apply to every champion through the input AI (moves only, as always).
- **Editor:** a Team plan tool group (Block choke drag, Control / Ambush / Avoid / Group click with a size, plus a radius slider) and the Per-skill group (Smoke / Wall / Ult spot).

## Oct 2, round 44: smoke 100000 ACROSS (native 0.6.3, tfm2_valorant 0.1.4)
- **Rian:** he meant the smoke should be as wide as a 100000 wall on the map editor, not a 100000 radius. SMOKE_R = 50000 (the dome VFX, the Map tab and the tooltip match). The round-43 placement offsets scale with R; the S2 changes stay.

## Oct 2, round 43: Omen smoke 100k, S2 used more (native 0.6.2, tfm2_valorant 0.1.3)
- **Rian:** smoke radius 100000; Omen rarely used Paranoia or the step.
- **SMOKE_R = 100000.** Placements adapted:
  - Objective: the approach smoke sits 0.9R from the pit.
  - Ambush: the enemy sits 0.6R inside the far side.
  - Fight: the dome is centred 0.55R past the hardest-hitting enemy, away from his teammates, and only when no teammate is inside.
  - Enemy smoke checks start from R + 50000 away.
  - Ult: lands in the part of a smoke nearest the enemies; hide_spot counts any point within R − 10000 of his smoke as hidden.
  - The dome VFX is 215 px across (R = 105 px).
- **S2 more often:**
  - Paranoia now fires at any enemy within 140000 on a clean line. It flies only to 35000 past the farthest enemy it can hit (60000-180000), so teammates beyond are safe. The ally margin is 8000 (was 16000), and 7 aim angles are tried (±0.4 rad).
  - New step uses: onto an enemy with at most one other near it when he holds a short gun (Classic / Sheriff / Spectre / Judge), behind a low enemy, and into his smoke to lurk when enemies are around it. Steps into a smoke land inside it within reach (into()).
  - Data: skill2 cooltime 420 (was 540), can_use_with_move true.

## Oct 2, round 53: Mod Power (native 0.7.0)
- Rian's point: tuning mod champions only in mod-vs-mod 1v1s is an endless nerf cycle ("elephant vs elephant"), while in real matches they fight base champions. The target is now: mod champions clearly stronger than the base roster, and balance among the nine judged in mixed teams.
- lib.rs `MOD_POWER = (attack 30, AP 30, HP 20, armour 15, MR 15, move speed 5)` percent, applied once per match as a `mod_power` buff to every champion whose name starts with `tfm2_` (first thing in the match hook). Full-HP champions are topped up to the new max.
- Roster guide PDF rebuilt for nine champions (generator: /home/claude/guide/make_guide.py + content.py, sprites in /tmp/claude-0/guide/img). It has a Mod Power page, a 9x9 matrix and a Map tab page.

## Oct 2, round 54: Omen shop, safe steps, longer blind (native 0.7.1, tfm2_valorant 0.1.6)
- Bug: Omen never changed his gun. buy() topped up heavy armor (1000) on every base visit and kept saving for the Vandal (3900) / Operator (6200) thresholds, so the money never got there. Now the gun comes first: the best he can pay for (Sheriff 800, Spectre 1600, Judge 1850 vs close fighters, Vandal 2900, Operator only from 5500 because its DPS is below the Vandal's), then armor and the smoke charge from what is left.
- Shadow Step lands on the safest spot that does the job. `spot_score` counts visible enemies near the spot (−60 under 25000, −30 under 45000, −10 under 70000; the step target is ignored), enemy towers within 85000 (−80), teammates within 45000 (+15), a bush (+25) and his own smoke (+30). Candidates by step type:
  - escape: 32 ring points + smoke entries, which must move him 15000+ farther from the chasers; bonus for getting closer to home;
  - smoke ambush / lurk: points inside the smoke;
  - Map ambush marks: points within the mark;
  - step onto an enemy: a ring at 9000 (Judge) or 18000 around the target, refused below −30.
- Paranoia blind scales with the distance from the throw: 300 ticks (5 s) up to 20000, falling linearly to 180 ticks (3 s) at 180000 (`blind_time`, unit tested). The decoys last as long as the blind. A re-blind keeps the longer timer.
- Tooltips updated (presets.js; only text/champion.i18n changed in the mod).

## Oct 2, round 55: Mod Power per champion; Omen risky plays, trade-off Paranoia, rush call, money (native 0.7.2, tfm2_valorant 0.1.7)
- Rian's 5 mod vs 5 base tests were losing. The game log of that session shows native **0.6.9** loading (DLL built at 18:05, before Mod Power), so Mod Power wasn't in those games. Structural reasons on top:
  - mod assassins grow 20-22 attack per level vs 30 for base assassins (70-76% of a base assassin's attack at level 10); Gojo and Frieren reach 85-88% of base AP;
  - longer ults (55-90 s vs 50 s / 40 s);
  - friendly fire and self-harm in their kits;
  - setup-gated skills;
  - no healer in a 5-mod comp;
  - Omen has no items.
- Mod Power is now per champion in lib.rs (`MOD_POWER: &[(&str, [i32; 6])]`, matched by the end of the id, `mod_power_of`, unit tested). Values (atk/ap/hp/def/mr/ms):
  - Minato 0 (Rian: "he don't need no mod")
  - Gojo 0/15/5/5/5/0
  - DIO 25/0...
  - David 25/0...
  - V1 30/0/5/10/10/0
  - Vader 5/0...
  - Frieren 0/15/5/15/10/0
  - Steve 0
  - Omen 10/0/10/10/10/0
  - Move speed is 0 for all. The aim is about the base class median by level 10; the kits are the edge.
- Omen changes:
  - Risky step: a low enemy he can finish (pct <= 25 or hp <= attack × gun mult × shots) → land behind them, on the side toward their base, at gun range. Refused when he's under 35% HP, into 3+ others within 45000, or under an enemy tower unless sure (target hp × 2 <= burst and Omen >= 60%).
  - Paranoia trade-off: `para_line(.., trade)` allows teammates on the line when enemies hit >= allies + 2 (unit tested: 3v1 yes, 2v1 no). Used when his side is running (enemies within 60000 outnumber his side, or he's at 50% HP or less with someone on him), and on engage when it catches 3+ and more than the clean line.
  - Rush call: 2+ enemies blinded, his side (>30% HP, within 220000) >= enemies still seeing within 70000 → a `stv_rally:x:y:-1:until` buff on Omen (reuses Steve's call; teammates' input AI walks there until the blinds end, Omen himself excluded) plus an ult_mark ping.
  - Shadows can't be killed: hp 100000, armour and MR 1000, and an undying `omn_shadow` buff for the blind's length.
  - Money over his head: buffs omn_cash ("$..00") + omn_crk0-9 (thousands) + omn_crh0-9 (hundreds), drawn by gen2.py valorant() as cash / cash_k* / cash_h* (gold 3x5 digits at y 10-16 of the 40x80 icon). Only the view changed, so no save sync is needed (verified: gameplay JSON md5 unchanged).

## Oct 2, round 56: Omen uses S2 much more (native 0.7.3, tfm2_valorant 0.1.8)
- Rian: Omen sat on S2 off cooldown even when he could blind one player.
- Causes:
  - S2 casts only happen in the AI's "enemy in CC" window. The 2-tick Bind came every 20 ticks only.
  - s2_plan had no plan in plain fights.
  - Paranoia only counted enemies within 140000.
- Fixes:
  - The Bind window comes every 10-tick refresh while S2 has a plan.
  - Paranoia takes any clean line, even onto one enemy, within PARA_RANGE − 15000 (165000).
  - New fallback "flash": `Gun::reach()` (Classic 30000, Sheriff 38000, Spectre 28000, Judge 18000, Vandal 42000, Operator 80000). Candidates are 32 ring points at STEP_R and 0.6×STEP_R. A spot must keep an enemy within reach − 4000 and stay 15000 from every enemy (6000 with the Judge). He steps when the spot scores ≥ current + 20 on spot_score, or ≥ current when no enemy is in his gun's reach now. Only at 35% HP or more.
- Side effect to watch: while S2 has a plan, the nearest enemy gets a 2-tick Bind every 10 ticks until Omen casts.

## Oct 2, round 57: Omen economy and smoke/blind assists (native 0.7.4, tfm2_valorant text only)
- Rian had never seen Omen buy.
- Economy:
  - start 1000;
  - kill 300, assist 150;
  - +200 every 20 s alive (was 150 / 30 s);
  - +50 for every kill his team gets (read from the kill log; kills_seen resets if the log shrinks);
  - death +400.
- On death he keeps his gun and loses only the armor.
- He buys at his spawn, or anywhere out of combat: no visible enemy within 110000, at most every 5 s.
- Smoke / blind assists:
  - Enemies he blinded, or that stand within SMOKE_R + 4000 of his smoke, get a 3 AP Skill damage tag from Omen on contact and every 90 ticks. deal_damage goes through the engine's kill/assist credit, so this should put him on the game's own assist list.
  - Fallback: if a tagged enemy dies within 300 ticks of its last tag and the game hasn't credited him (on_kill / on_assist within 2 ticks), the native rules add the 150 credits themselves.
- Deployed together with round 58 once the device link came back.

## Oct 2, round 58: Flash for every champion (native 0.7.5, tfm2_valorant 0.1.9)
- New module flash.rs, run from the match hook every 4 ticks for every champion, base and mod.
- Blink: FLASH_R 40000. Cooldown FLASH_CD 7200 (120 s), kept in a static map keyed by sim.seed() and champion id, so it survives death. The blink is entity_set_pos (crosses terrain and Steve's wall). Blocked while stunned, while void_trapped, during DIO's time-stop freeze (tsf…) and while riding Steve's boat (stv_riding).
- Uses, in order (the first that applies wins):
  1. run: at 25% HP or less with an enemy within 30000 and the fight lost around them, or at 15% or less. Lands away from the enemies, toward home (nexus constants) and teammates, never under an enemy tower.
  2. evade: a projectile from an enemy champion who cast a skill or ult in the last 90 ticks (a cooldown jump), predicted to reach them within 18 ticks and within 20000 of its line, when they're at 60% HP or less or it's an ult. Flashes sideways. Velocity is matched to the same caster's nearest projectile last check.
  3. help: a teammate at 35% HP or less with an enemy on them, behind terrain (walls::clip), 25000 to FLASH_R + 25000 away, and they're at 50%+ HP.
  4. catch: they're at 40%+ HP, a visible enemy at 20% HP or less (or under two of their hits) moving away, 30000 to 80000 away. Lands 15000 short; not into a crowd, or under a tower unless the target is at 8% or less.
  5. combo: their ult is ready, 3+ enemies within 25000 of each other 30000 to 75000 away, their team can follow, and they're at 60%+. Lands 12000 short of the group.
  6. engage: they're at 70%+ HP and their side outnumbers the enemies there; an enemy 35000 to 70000 away with at most one friend near and no tower; the weakest first.
- VFX: tfm2_valorant_omen_flash, a white and gold burst, registered on Omen's data and played at both ends. If play_view_effect returns false, it is retried with an Omen in the match as the caster. The Valorant mod is needed to see the flash, but not for the mechanic.
- No save sync needed (gameplay JSON unchanged).

## Oct 2, round 59: Flash fixed and saved for big moments; Omen fixes; David travels (native 0.7.6, tfm2_valorant 0.1.10)
- **Flash had no cooldown.** Its state was keyed by sim.seed(), which isn't stable per match, so every check started fresh.
  - New `match_key(sim)` in lib.rs: a hash of the sorted (id, team, name) of every champion in the match.
  - Used by flash.rs and by Omen's smoke REVEAL map, which had the same bug, so the reveal-on-shooting never held either.
  - A second guard: a `flash_cd` buff (7200 ticks) on the champion.
- **Flash is now only for big moments:**
  - run: 20% HP or less with the fight lost, or 10% or less;
  - evade: only ult shots, Hollow Purple (caster has gojo_purple_strain within 240 ticks of 1260), Paranoia (caster has omn_para_fly), or any skill shot at 30% HP or less;
  - catch: target at 15% HP or less and within two hits;
  - help over walls and the ult combo are unchanged;
  - engage was removed.
- **Omen:**
  - Chased (retreat_para: enemies within 60000 outnumber his side, or he's at 50% HP or less with someone within 45000) with S2 ready, he throws Paranoia himself. The game's AI doesn't press skills while it runs. An `omn_s2_lock` (S2_LOCK 420) keeps the data's S2 shut meanwhile.
  - Armor only with spare money: credits beyond the Vandal's 2900 until he owns a Vandal.
  - fx_seen(): the smoke dome, form and fade, para_cast and the new para_trail are played with a tower as the caster (structures are seen by both teams), falling back to Omen.
  - The Paranoia sprite is brighter: violet haze, a glow and a pale rim. A para_trail puff is played every 5 ticks along its path.
- **David:** out of fights (no enemy within 150000), on the move, fuel at 80%+ and cyberpsychosis under 25 → a 1 s Sandevistan burst for travel.
- **Not done yet:** the jungle and farming use of mod damage skills. Every mod damage skill targets EnemyChampion and runs through native code that expects a champion. The base data champions use EnemyWithoutTower for this. Doing it means a per-champion rework.

## Oct 2, rounds 60-61: farming skills, Omen travel step, and the server/live divergence fix (native 0.7.8)
- **Divergence found in log.log:** "[GamePlayDone] server/live simulation diverged ... overriding precomputed result" on both sets of Rian's 0.7.5 session.
  - The game runs a match's precomputed "server" sim and the "live" sim side by side.
  - Native static memory keyed by sim.seed() or by the line-up was shared by both: Steve's wall positions (LAST), Omen's REVEAL, and Flash's STATE. That made the two sims diverge, and it also wiped Flash's cooldown (each sim's reset cleared the other's).
  - Fix: mod-api-stable sim.rs got `StableSim::instance()` (the address of the sim state; Rian's copy of the crate was patched too). `match_key()` now returns it, and all three maps use it.
- **Farming:**
  - presets.js `withFarm()` wraps damage skills in SwitchByBuff `mod_farm` with plain data versions and sets casting_target to EnemyWithoutTower.
  - Native `farm_mode()` (every 10 ticks) gives tfm2_ champions `mod_farm` while no visible enemy champion is within 110000.
  - Farm kits:
    - Minato: S1 a piercing kunai, 40+60% AD; S2 Rasengan, 50+80% AD.
    - Gojo: targeting only (his flags ride on basic attacks).
    - DIO: S1 3 knives, 30+60% AD each; S2 a Stand strike, 60+100% AD.
    - David: S2 6 pistol shots, 8+18% AD each.
    - V1: S1 a coin burst, 30+45% AD within 30000.
    - Vader: S1 the saber line, 50+90% AD.
    - Frieren: S1 a Fern line, 40+55% AP; S2 Stark lands, 45+65% AP within 20000.
    - Steve: S1 TNT, 70+60% AD within 32000.
  - Ults and utility skills are unchanged.
  - Applied on the PC by `Claude outputs/round60/apply_farm.py` + farm_spec.json (idempotent). The text gets a "Farming: …" sentence.
  - Mod versions bumped: blockcraft 0.1.18, custom 0.1.68, cyberpunk 0.1.13, frieren 0.1.14, jjk 0.1.28, jojo 0.1.16, starwars 0.1.12, ultrakill 0.1.15.
  - Synced to custom_database.tfm2db and both saves, save_20261002_183937 and save_20261002_225252 (CRC ok; mod_farm 27 / 30 / 30). Backups in backup-before-round60/saves.
- **Omen travel step:** no visible enemy within 150000 and moving (smoothed velocity ≥ 200 per tick) → he steps 50000 along his way by himself, with the omn_s2_lock.

## Oct 3, round 62: Scribble, the 35-spell toon mage (native 0.7.9, new mod tfm2_toon 0.1.1)
- **Request:** "now do the skills and the sprites". An original character (the Gear 5 look was declined earlier): white messy hair, open black jacket, white shirt, grey trousers, pencil behind the ear, white lightning around him, flies. Sprite: sprites/scribble/scribble2.py + pack.py (40x52 frames). VFX sheet 'scribble' (vfx/scribble_vfx.py, 2048x776, 50 spell effects + 30 dot icons + 7 rank badges).
- **Kit (native scribble.rs, passive tfm2_custom_ai:scribble):** 6 dots max, elements 1 Pencil / 2 Eraser / 3 Paint / 4 Gadget / 5 Page; Invoke casts the spell whose recipe is exactly the dot sequence (order matters); a non-recipe or a spell on cooldown fizzles. 35 spells: 5 tier 1, 8 tier 2, 8 tier 3, 6 tier 4, 5 tier 5, 3 tier 6, each with its own cooldown (table in presets.js SCRIBBLE_BOOK and in the Docs doc "Scribble: 35-spell kit").
  - The data S1/S2/ult never cast (EnemyChampionInCC, range 1, cooltime 5184000, empty effect); they carry the tooltips. The AI only moves and throws darts (BA TargetProjectile 100% AD, range 60000).
  - Invoke = Animation CC "ult" for INVOKE_T ticks (30 Novice .. 12 Archmage), the spell resolves at the end with a fresh evaluation (target gone = wasted, scored 0).
  - Brain every 6 ticks: value per spell (damage + 300 for a kill, 100 per second of hard CC, heal, shield x0.7), x global meta factor ^(0.4 + 0.1 rank), x rank noise, held-target check for slow spells (rank 3+), chaining bonus after his own 40+ tick lockdown (rank 3+). Goal switches need x1.4 (x2.5 for ranks 0-1). Rank 2+ prepares an opener out of fights and holds it.
  - HUD: `scr_d{slot}_{el}` buffs (dot icons orbiting over his head), `scr_rank{r}` (badge top right).
  - Drawn Wall reuses Steve's `sbw:` buff (enforced by steve::enforce_walls, fog, WallAi detours); Scribble draws its own wall_draw / wall_seg art.
  - Draw a Friend: spawn_unit with the fallen ally's base sprite (or the shadow sprite for mod champions), 60% stats, 600 ticks. valorant::BASE_SPRITES is pub(crate) now.
  - Page Flip: every champion off the mid band (|x + y - 960000| > 110000) goes to (960000 - y, 960000 - x); spots in terrain are nudged (free_near) or skipped.
- **Mastery (per athlete):** ranks Novice 0-4 games, Apprentice 5+, Adept 15+, Expert 30+, Master 60+, Grandmaster 100+, Archmage 150+. Known tiers 2/3/3/4/5/5/6; weave speed +0/10/20/35/50/65/80% (base 30 ticks a dot); misfire 16/11/8/5/3/1/0% per dot; notice 25/45/65/85/95/100/100%; hold time 0/60/150/240/300/360/420 ticks.
  - The athlete id comes from the input AI (WallAi::think reads ctx.athlete_id() for a champion ending in "scribble", keyed by (sim seed, player id)). Rank is latched at tick 60 and pinned per (match_id, set_index, athlete) in a session map, so the server and live sims agree.
  - A game counts at tick 1800 (sig = match id + set + a hash of the line-up and positions); it counts for the next match in the same session at once, and in the file memory at the next launch.
- **Global learning:** every cast is scored realized / promised (kills within 4 s add 300) per spell and situation bucket (clump 1/2/3+ x normal / me low / ally low). Files in mods/tfm2_custom_ai: scribble_memory.txt (W / G / M lines), scribble_pending.txt (g / c lines, merged and emptied at the next launch, deduped by sig), scribble_log.txt (one start/end line per game: athlete, rank, casts, misfires, fizzles, most-cast spells). The meta only changes at load, never mid-match.
- Mod Power entry `_scribble` 0/10/5/0/5/0. Stats like a base mage: 80 AD / 40 AP / 900 HP, growth 6 / 20 / 100.
- **Deployed:** native src (scribble.rs new; lib.rs, steve.rs, valorant.rs), editor presets.js / sprites-data.js / vfx2.js, mods/tfm2_toon (via the Skill Lab e2e), tfm2_toon added to config/game/mods.json. Backup: Claude outputs/backup-before-round62. Rian needs to run build.bat (the last DLL he built was 0.7.5; 0.7.8's fixes come with this build).
- Scribble is new: custom_database.tfm2db and existing careers don't have him (sync_mod_champions.py only replaces). Exhibition matches and new careers get him from the mod.

## Oct 3, round 63: Skill Test tab, Scribble memory page, guide for round 62 (editor only)
- **Request:** "finish the rest".
- **Skill Test tab** (editor/skilltest.js; also "Open Skill Test" on the welcome screen). An arena 330 x 210 art px (950 units/px, drawn 3x):
  - Any champion from the mod folders (or the presets). Right-click = move / attack a dummy, Q W E = the three data slots, R F T G Y = extra actives (none except Scribble), S = stop. Level 1-18, stats with Mod Power.
  - A simplified data interpreter: damage (armour / MR), heal, shield, every CC kind, knockback / grab / pull, teleports and dashes, Linear / Target / Parabolic projectiles, Range / Line / Period areas, RangeEffect, buffs (move speed), Delayed, SwitchByBuff, SwitchByLevel3, AddCasted, view effects, projectile and buff visuals from the champion's own bindings. Native passives are not simulated (the side panel says so), except Scribble's.
  - Scribble: R F T G Y (or 1-5) weave, E invoke at the cursor, Q flick, W the last recipe again; rank selector, optional slips (rank misfire / notice chances), cooldowns toggle. The 35 spells mirror scribble.rs numbers. Spell book list with a play button per spell and "Play all 35" (a gallery: dummies placed per spell, damage per spell listed at the end).
  - Dummies: HP / armour / MR, optional walking (to test slow spells) and hitting back, respawn after 2 s, floating damage and CC labels, a log.
- **Scribble memory page** (same tab): ranks per athlete (games in memory + waiting in pending), set games / reset per athlete, add an athlete, reset the meta or everything (backups in editor/backups/scribble), the learned meta as a 35 x 9 table, the last games from scribble_log.txt. Server: GET/POST /api/scribble. Athlete names show when a save is open (if the athlete ids match).
- presets.js exports TFM2_SCRIBBLE_BOOK.
- **Roster guide PDF** rebuilt for round 62 (30 pages; content.py / make_guide.py, round-55 copies kept as *_r55.py): ten champions, Scribble's page + a spell book page, a "New since round 55" page (Omen S2 and economy, Flash, farming, the divergence fix, Scribble, Skill Test), matrix 10 x 10, balance review and open questions updated. Old PDF in backup-before-round63.
- Deployed to C:\Games\tfm2\editor (skilltest.js new; index.html, app.js, server.js, style.css, presets.js). server.js changed: restart the editor. Backup: Claude outputs/backup-before-round63.
- Still a draft, not built: the Gundam mech kit.

## Oct 3, round 64: Scribble memory review; ranked/official weight, win boost, live in-session learning, tier cooldowns (native 0.7.10, tfm2_toon 0.1.2)
- **Rian's first session with 0.7.9** (02:24-03:05 UTC): 114 Scribble games recorded (all background league fixtures plus a few scrims), 97 athletes (nearly all 1-2 games each, so Novice), athlete 11 with 5 real games (Apprentice). 86 casts per game on average (31-209).
- **Why the meta didn't update:** by design the pending file was merged only when the game starts, and this was the first launch (no scribble_memory.txt yet); the editor page read only the memory file.
- **Spells in that data:** Bucket 40% of all casts (healing chip damage between fights), Paper Cut 18%, ?! Bubble 8%. Delivered / promised: Bucket 0.93, Paper Cut 0.81, Wind-up Key 0.75; Banana Peel 0.02 (stepped on 5% of the time), Draw a Door 0.20, Squeaky Horn 0.25, Cutout 0.34, Rubber Chicken 0.38 (mostly the moment had passed by the end of the invoke).
- **Possible divergence:** log.log showed "overriding precomputed result: set=2 ... server_score=13:14, live_score=13:12". The rank pin was keyed by (match id, set index, athlete) and the same game showed up with two set indexes ("6.0.17ae..." / "6.1.17ae..."), so the two simulations could pin different ranks. Now pinned by the sim seed.
- **Changes (native):**
  - Mastery points: official match (has a match id: league, cups, ranked) 1, scrim / exhibition 0.5, win x1.5. Meta weight: official 1, scrim 0.5, win x1.25. Memory format: `G athlete points games wins`, `M spell bucket sum weight` (old lines still read).
  - Win detection: `r sig athlete tick myTowers theirTowers myNexus% theirNexus% scoreDiff` written when it changes (and every 10 s); the last record decides (nexus, then towers, then score). The nexus is the tower entity within 70000 of the nexus position (falls back to 100%).
  - Live learning: each match pins a snapshot (launch memory + every pending line written this launch), keyed by sim seed, shared by the server and live sims. Ranks and the meta now change within a session.
  - Signature without the set index (`<match id>.<hash>`); old lines dedupe by the hash. The merged pending lines are kept in scribble_history.txt.
  - Per-game summary lines `s sig athlete tick rank casts misfires fizzles originKind official spell:count,...` every 30 s.
  - Bucket: full value only in a fight or below 30% HP (x0.4 at 30-60% out of a fight, 0 above). Banana Peel lands 4000 in front of the nearest enemy (was half-way).
  - **Cooldowns per dot count (Rian):** a spell puts every spell with the same number of dots on its cooldown; the other tiers stay free.
- **Editor:** the memory page merges the pending file the same way (points, games, wins, points to next rank, meta with cast weights) and lists recent games (result, casts, slips, fizzles, most cast). Server returns the tail of scribble_history.txt. Set games now sets points. Skill Test uses tier cooldowns.
- Guide PDF rebuilt (round 64 wording). Backup: Claude outputs/backup-before-round64 (includes the scribble files). Rian needs build.bat.
- Real-data check (merge of his pending file): world 109 games, athlete 11 5.0 points (Apprentice).

## Oct 3, round 65: guide PDF: difficulty out of 10, Scribble by rank; game patches touch mod champions
- Difficulty is now 1-10: Minato 9, Gojo 7, DIO 7, David 6, V1 6, Vader 3, Frieren 5, Steve 7, Omen 8, Scribble 10.
- New page "Scribble by mastery rank": per rank the points, spells known, time per dot, a 3-dot and a 6-dot cast (weave + invoke), wrong-dot and notice chances, hold time, misread noise; what each rank adds; matchups by rank (Novice / Adept / Expert / Archmage) against the nine with reasons (nets -9 / -1 / 0 / +7). The matrix reads Scribble at Expert. Spell book gets a "Known from" rank column.
- **Rian confirmed: the game's own balance patches can nerf mod champions.** Patches change the career's copy of the data (stats, data-skill numbers). Native mechanics are not touched (all of Scribble's spells, Mod Power, kunai, walls, smokes). Careful: sync_mod_champions.py replaces every copy (base, current and the per-patch snapshots) with the mod JSON, so running it on a career undoes the game's patches on mod champions. Offered a patch-preserving sync.
- Guide sources: /home/claude/guide (content.py, make_guide.py; round-55 copies *_r55.py). Old PDF in Claude outputs/backup-before-round65.

## Oct 3, round 66: Omen smokes half the pit when enemies are on it (native 0.7.11, tfm2_valorant 0.1.11 text)
- **Request:** "make omen not smoke on the obj when there is an enemy on the objective, makes it harder to see who is actually inside, make it so it smokes half the obj so we can pick off".
- valorant.rs `half_pit()`: the dome's edge runs through the pit centre (offset 0.95 x SMOKE_R), covering the half away from his team (teammates within 200000 of the pit, else Omen). Enemies on the near half stay visible; those on the far half are cut off and see nothing out.
- Used for his own objective read when the enemies hold it (was: the pit centre), and for every other smoke candidate (Map tab pit plans, fight smokes) whose centre is within 0.85 x SMOKE_R of the pit while a visible enemy stands within 40000 of it. Unit test half_pit_edge_through_the_centre.
- Text only in the mod (i18n edited in place on the PC, mod_info 0.1.11); no save sync. Backup: Claude outputs/backup-before-round66. Rian needs build.bat.

## Oct 3, round 67: Omen ganks with his ult; no smoke on a gank target (native 0.7.12, tfm2_valorant 0.1.12 text)
- **Request:** "if ganking a lane and have ult, should use that because you can literally gank bot from top, also dont smoke the enemy if doing a gank, smoke behind them or where his teammates are".
- valorant.rs `gank_target()`: a visible enemy in a small fight (at most 2 enemies within 50000 and 100000) with a teammate within 45000 of them, his side + Omen >= their count; the lowest-HP one.
- **Ult:** a new plan after "a far fight": no enemy within 70000 of Omen, the target 150000+ away, a teammate at 35%+ on them, not under an enemy tower (75000) unless the target is at 30% or less. Lands 18000 behind the target toward their base (cuts the retreat); hide_spot still prefers a bush near it.
- **Smokes in a gank:** every candidate within SMOKE_R + 5000 of the target is dropped; the gank smoke (weight 48) goes SMOKE_R + 10000 behind the target toward their base (target just outside, his ult landing inside), or over the teammates' centroid when that would catch a teammate. The ult-destination smoke follows the same rule.
- Applies to 2v2 lane skirmishes too (no smoke on the enemy there either).
- Tooltips: presets.js + the PC's i18n edited in place (omen_text.py), mod_info 0.1.12. No save sync. Backup: Claude outputs/backup-before-round67. Rian needs build.bat.

## Oct 3, round 68: Scribble prepares for the situation and drops a held spell (native 0.7.13, tfm2_toon 0.1.3 text)
- **Request:** "yes" to a situation-aware prepared opener, plus: "as an archmage I want him to see the situation and see if draw a door is useful, if something is more useful, rather than holding which can be bad, make him able to drop the skill and change".
- scribble.rs `intent_of()`: Escape (him under 40%), Teamfight (3+ enemies within 60000 of each other within 250000, or 4 of his team grouped), Gank (a teammate within 250000 with 1-2 enemies on them and no more than 2 within 100000), else Default. `prep_list()`: Gank = Erase Legs, Mallet, Rubber Arm, Say Cheese, Pie, ?! Bubble, Rubber Chicken, Pencil Poke; Teamfight = PAUSE, Piano, Group Photo, Laugh Track, Ink Flood, Stamp, Say Cheese, Paint Splat; Escape = Hole Network, Redraw, Door, Erase Myself, Bucket; Default = the old list. Adept+ prepare it while no enemy is within 90000; Expert+ re-pick when the intent changes.
- `score()` (a spell's real worth now, no misreads) and `usable()` split out of choose(). Switching: Novice-Adept as before (through their misreads, vs the stale goal score, x2.5 / x2.0 / x1.5); Expert+ compare the held spell's real worth now: drop at once when it has none, or switch when another is better by SWITCH_K (Expert 1.3, Master 1.15, Grandmaster 1.1, Archmage 1.05). Dots that start the new spell are kept.
- Unit test opener_follows_the_situation. Tooltips (presets.js + the PC's i18n via scr_text.py, toon mod_info 0.1.3) and the guide PDF (rank page; also Omen's half-pit and gank changes) updated. Backup: Claude outputs/backup-before-round68. Rian needs build.bat. No save sync.

## Oct 3, round 69: David's Sandevistan v2 / v3 buffed (native 0.7.14, tfm2_cyberpunk 0.1.14 text)
- **Request:** "put a buff on david sandevistan v2 and v3, hes been off".
- They used to trade attack for speed (v2 +20 ms / +20 as / -15% attack; v3 +45 / +40 / -25%), so a high meter only made him faster. batch2.rs now: v2 +25% move, +35% attack speed, +10% attack, 8% lifesteal; v3 +50% move, +60% attack speed, +20% attack, 15% lifesteal, damaged_reduce 15. (On top of Sandevistan's own +40 / +50; the dv_sv2 / dv_sv3 buffs.)
- Tooltips (presets.js + the PC's i18n via dv_text.py, mod_info 0.1.14) and the guide (David's S1, balance review) updated. No data change, no save sync. Backup: Claude outputs/backup-before-round69. Rian needs build.bat.
- Watch: v3 with 15% lifesteal + 15% damage cut against the 7%/s psycho drain; levers v3 lifesteal 15 -> 10 or attack +20 -> +10.

## Oct 3, round 70: Scribble moves off a spell whose tier is on cooldown (native 0.7.15, tfm2_toon 0.1.4 text)
- **Request:** "make him realize if the skill he wants is a 3dot but its cd, change to 2/4 or like second leading important not 3dot skill".
- New spells were already only chosen if usable (tier ready within the weave time). The gap: a goal picked before its tier got locked (he cast another spell with as many dots) was kept; low ranks waited up to 10 s, and a prepared opener could be one on cooldown.
- scribble.rs: `tier_locked()`; in choose(), a locked goal is replaced by the best usable spell after CD_NOTICE[rank] ticks (Novice 90, Apprentice 60, Adept 30, Expert+ 0); woven dots that start the new spell are kept (set_goal). Prepared openers skip locked spells (usable()) and are re-picked when the held one gets locked. Unit test locked_tier_goes_to_the_next_spell.
- Tooltip (presets.js + the PC's i18n via scr_text2.py, toon mod_info 0.1.4) and the guide's spell-book intro updated. Backup: Claude outputs/backup-before-round70. Rian needs build.bat.

## Oct 3, round 71: Frieren buffed, native only (native 0.7.16, tfm2_frieren 0.1.15 text)
- **Request:** "buff frieren, shes been down".
- Kept to native numbers so no save sync (which would also undo the game's own patches): Fern's Zoltraak 50 + 70% AP (was 40 + 55%); Limiter Fern every 3 s (was 4) for 30 + 45% (was 24 + 35%); Stark's landing 55 + 80% (was 45 + 65%), slow 35% for 1.75 s (was 25% / 1.25 s); Limiter Stark: axe 30 + 35% (was 20 + 25%), leaps 35 + 50% (was 25 + 40%), survives 4 hits (was 3); Mod Power AP 15 -> 25, HP 5 -> 10.
- Unchanged (data): BA Zoltraak 10 + 30%, Limiter +25% MP / +15% AS, the farming versions. Next lever if still weak: Limiter +40% MP (data, needs the sync).
- Tooltips: presets.js (starkSlow 35 / starkSlowTicks 105 are text-only options) and the PC's i18n via fr_apply.py (whole-string swap checked against the old text); data JSON identical. Guide updated (Mod Power table, Frieren page, balance review). Backup: Claude outputs/backup-before-round71. Rian needs build.bat.

## Oct 3, round 72: Scribble Top 10, Invoker-speed weaving, any rank can try any spell, rank skins (native 0.7.17, tfm2_toon 0.1.5)
- **Request:** a new rank above Archmage, the Top 10 (of the ~200 athletes, the ten with the most games on him, 300+ each), with an animated numbered badge #10-#1; skins from Grandmaster up, the Top 10's the coolest. Weaving speed redone as clicks per second, like Dota's Invoker, since every athlete here is a pro: the Top 10 at 11-15 CPS, a Novice at 0.4 s a dot (was 0.5). Any rank can use any spell, but a dot past their level is likely to come out wrong (a Novice going for a 6-dot spell: dot 3 80%, dot 4 86%, ...).
- **Ranks (scribble.rs):** 8 ranks; 7 = Top 10. `Memory::top_ten()`: athletes with 300+ mastery points, ordered by points, then games, then wins, then athlete id (a full tie always breaks the same way in both simulations). Read from the match's pinned memory, so positions only move between matches. "300 plays" is read as 300 mastery points, the same unit as the other thresholds (an official match 1, a scrim 0.5, a win x1.5).
- **Weave speed (CPS):** Novice 2.5, Apprentice 3.25, Adept 4, Expert 5, Master 6.5, Grandmaster 8, Archmage 9.5, Top 10 from 11 (#10) to 15 (#1), linear. Kept in thousandths of a tick, so 15 CPS is exactly 4 ticks a dot and fractions carry from dot to dot (never banked while idle). The hands now weave every tick; the brain still thinks every 6 ticks (at 6 ticks a beat it capped weaving at 10 CPS). Invoke 10 ticks for the Top 10.
- **Any spell, overreach slips:** `KNOWN_TIER` is now `COMFORT_TIER` (2/3/3/4/5/5/6/6). A dot within it slips at the rank's old misfire chance; a dot past it at OVERREACH (Novice 80, Apprentice 65, Adept 50, Expert 40, Master 30, Grandmaster 20) + 6 per further dot, max 98. A noticed slip is flicked away as before; an unnoticed one is cast as whatever it spells (or fizzles).
- **Choice:** a spell's value is multiplied by its build chance ^ AWARE (Novice 0.35 ... Grandmaster+ 1), so rookies still go for big spells now and then (and fumble), the best weigh their odds fully. Prepared openers only use recipes within the comfort tier.
- **Badges and skins (VFX sheet 'scribble', generator `Claude outputs/scribble/rank_art.py`, re-runnable):**
  - `scr_top1..10` (z 4): a numbered medallion with a glint sweep; #6 up an orbiting spark; #3 / #2 / #1 orange / blue / gold flames; #1 a rainbow rim and a crown.
  - Skins are two buff visuals each, since a champion's body sheet can't be swapped during a match (the client can draw sprites in its render hooks but can't see where a champion is or what it's playing): `scr_skin<k>_b` behind him (z −1, like Minato's KCM cloak) and `scr_skin<k>_f` over him (z 3, under the dots). Symmetric, since buff visuals don't flip with his facing.
    - 0 Ruby (Grandmaster): a floating gold crown, a ruby sigil under him, ruby wisps rising, ruby sparks.
    - 1 Prism (Archmage): a turning rainbow halo, three pages orbiting him (behind on the far side, in front on the near side), rainbow sparkles.
    - 2 Legend (Top 10): ink wings with gold trim (flapping), a gold halo with a star, a turning gold sigil, gold lightning and embers.
  - The sheet grew to 2048x1261; the editor's vfx2.js bundle is rebuilt from it. The bindings are only view data, so no save sync is needed.
- **Editor:** Skill Test's mastery menu has the 7 ranks plus Top 10 #10-#1 with their CPS; the spell book greys recipes past the comfort tier (the tooltip gives the odds of building it); the arena draws the badge and the skin (z < 0 behind the body). The memory page marks the Top 10 and how far others are from Top 10 eligibility. Tooltips (presets.js + the mod's i18n) describe CPS, comfort and skins.
- Unit tests: overreach_slips, top_ten_needs_300_points_and_is_ordered, rookies_value_big_spells_by_their_odds; ranks checks the CPS and badge / skin names.
- Rian needs build.bat (DLL 0.7.17). No save sync.

## Oct 3, round 73: seeding the pros' Scribble mastery (editor only)
- **Request:** randomize the ~200 pro players' Scribble rank on a normal distribution, Master at most, and Faker a Master.
- **Editor (Skill Test → Scribble memory → "Randomize pro mastery…"):** every athlete in the open save or database gets a rank drawn from a normal distribution over the ranks (mean Adept, sd 1 rank, rounded, clamped to Novice-Master: about 7% Novice, 24% Apprentice, 38% Adept, 24% Expert, 7% Master), then random points inside that rank's band (Novice 0-4.5, Apprentice 5-14.5, Adept 15-29.5, Expert 30-59.5, Master 60-99.5). A player named Faker (any case) is always a Master with 95 points. Games and wins are filled in to match the points (an official game 1, a win 1.5, about half won). Nobody starts above Master, so Grandmaster, Archmage and the Top 10 still have to be earned.
- The confirm box shows the roll (count per rank, whether Faker was found); every click re-rolls. Only those athletes' G lines are replaced; the learned meta and anyone not in the save stay. Their waiting games in scribble_pending.txt are dropped so the numbers land exactly.
- **Server:** POST /api/scribble `{action: 'seed', entries: [{a, points, games, wins}]}`, with the usual backup in editor/backups/scribble.
- The game reads Scribble's memory once per launch: seed with the game closed (or restart it after).

## Still to watch (Scribble)
- Round 72: do the skin layers sit right on him in game (z −1 behind, centred like the badges), and do the wings look OK when he faces left? Is a Top 10 visibly faster (a 6-dot spell in about 0.4 s)? Do Novices try and fumble big spells now and then, without wasting whole fights? Do the badge numbers read at game zoom?
- Does the Animation CC with name "ult" play the invoke pose (and not freeze him oddly)? Do the dot / badge buff icons show and sit right?
- Does WallAi see his athlete id (scribble_log.txt shows a real athlete, not 1000000+)? Do ranks change after 5 games?
- Are fizzles / misfires visible but not crippling at Novice? Does an Archmage feel clearly stronger?
- Draw a Friend's spawn_unit (crash risk, as with Stark in round 3/8). Page Flip landing spots. Drawn Wall blocking.
- Damage share against base mages; he may need numbers tuned once the meta has a few games.

## Still to watch (Omen)
- Rounds 60-61: no more "server/live simulation diverged" lines in log.log? Do mod champions now use skills on camps and waves, and still save them for champions in fights?
- Round 59: is the smoke visible from the enemy side now (tower caster)? Does the Paranoia trail show? Is Flash now about once per 2 minutes, and only in big moments?
- Round 58: does the flash effect show on base champions (effect name lookup by caster or global)? How often do flashes fire per match, and are any dumb (into towers, into crowds)?
- Round 57: do smoke / blind kills show as Omen assists on the scoreboard (the 3 AP tag)? Do the tag damage numbers look noisy? Does he now show gun upgrades during a match?
- Round 55: does the money row clash with the HP bar? Do teammates walk to the rush mark? Do blinded champions keep hitting the undying shadows? Are the risky steps ever suicidal?
- Round 54: does a gun icon change after his first base visit with 800+ credits? Do the steps land somewhere sensible (not deeper into the enemy)? Is a 5 s blind too much?
- Round 53: does Mod Power show (attack and HP up about 30% / 20% on a mod champion in the stat panel)? If not, the buff mults may not be percents.
- Does spawn_unit with a base champion name draw that champion, and does the bombardier override show the shade? Do the shadows stay put under Bind while moved by set_pos?
- Does Taunt make the blinded attack a shadow (and only that)? Does BlockAttack hold when none remain?
- Does the game AI dodge native Paranoia? Does buff `range` really extend the BA (Operator at 80000)? Does the attack-speed penalty apply?
- Do the empty casts' skill_cooldown_mult hastes bring the slot back fast?
- Smoke vision: do champions inside become untargetable from outside, and does the reveal on shooting work?
- Map tab: is map_dump.json written (DLL next to the game exe)? Do towers / camps land where the game shows them (team 0 = Blue)? Does Steve build Rian's wall lines when their trigger is on?

## Still to watch
- Round 35: do Steve's teammates follow the call to the wall's target (and stop when it's no longer winning)? Does it pull laners off lanes too often (RALLY_R 220000)?
- Round 30: does Steve hold the boat while an objective is up and use it there? Does an empty (no-setup) ult cast come back in ~2 s (ult_cooldown_mult haste) or burn the full cooldown?
- Round 27: does Steve ride over terrain cleanly (no snapping or stuck), and does the boat's knock clear enemies off the line?
- Round 26: does Vader's choke now play (skill2 tag)? Does DIO's Stand lunge show up, with the gold strip, and the dash strike with "!" + strip? Is the ZA WARUDO guest free and buffed? Does a 4800 Blue pull overshoot the centre?
- Round 25: do the wall blocks stay for the full 10 s (and crumble at the end)? Does wall_fog hide people behind the wall without breaking anyone's targeting badly? Where does Steve put the straight wall: does it seal lanes and cut retreats? Does the Steve move redirect make him follow his team without dithering? Is the V1 / Frieren damage in a sensible range now?
- Round 24: does Steve wait for a fight / objective before casting the boat (the Bind trick), and does the box form around the enemies with the open side toward his team? Does he jump off midway? Do units walk out through the open side? Do enemies reeled through terrain land on free ground (no one stuck in a wall)? Is a 220000 through-wall hook too strong?
- Round 23: does Steve pick a sensible tool (apple when an ally is low, TNT into clumps, pearl to chase / escape / cross walls)? Does the tool icon flicker too often?
- Round 21 (Steve): does the input AI cause any freeze or planner panic (log.log)? Does the wall block and do units walk around it? Does the rod release early on close targets? Do teammates light the TNT? Does the boat line actually cut off the target?
- Round 19: V1's damage share after the nerf. If coin/shotgun turns still look too strong, check whether the pistol hit is double-dipping.
- Round 18: Frieren's damage share after the hit-once fix; roll the ratio nerfs back if she drops off.
- Round 17: does DIO chain the 4-skill combo (the second cast of a slot switching stance)? Does the haste really cut the slot cooldown when the other mode is ready? Is the red ! visible before the dash?
- Round 16: does DIO heal at all in Stand Out now, even with lifesteal items? Does a negative vamp total behave (it should sum to 0)? Does the lunge strip line up with where the Stand actually dashes?
- Round 15: is the DIO lunge visible now (wind-up glow, golden streak, grab)? Does he walk into melee on towers instead of punching them from range?
- Round 14: does the AI keep Minato at kunai range? Do marks show on enemies? Does the Rasengan flash fire off a kunai hit, and does the flash back out feel good or too slippery?
- Round 13: does V1's S2 show ready while the parry is armed and go on the full 10 s cooldown right after a parry? Do the frequent re-arm casts (every ~0.9 s) disturb his attacks?
- Round 10: does Vader's 96-tick choke_seq play in full, and does the BA aim line point the right way? Do the red/blue fires sit on David's bar? Does the coin visibly fly toward the enemy?
- Round 9: does the melee dodge fire (the cooldown jump may come at the hit rather than the swing start)? Does Stark die on the 3rd hit, and does Frieren really stop casting during Limiter? Does V1 auto-shoot the coin? Does the pack kunai auto-flash look right?
- Round 8: does Stark (ghoul) spawn without crashing and look like Stark? Does Minato pick up kunai and use the pack throw? Does Vader stay rooted for the full choke + 2 slashes?
- Does the AllyOnlySelf self-pattern apply to Gojo (the cast lock, and Infinity's shimmer that should end when its shield breaks)?
- Which way does the ShrinkingBarrier knockback push, and does the barrier follow Gojo?
- Does the Rasengan empowered hit trigger, and do Gojo's mini attacks fire?
- How often does plain Red now fire at far-away CC'd enemies? Its flight range is only 70000, so those shots are wasted.
- Minato: does the 6-tick fear register? Do the 3 stages happen reliably with a 4 s real cooldown?
- DIO: do the Stand poses line up beside him in both facings? Does range +32000 from the mode buff really extend the BA? Do the modes switch sensibly?
- Minato: does the dodge fire on Purple, Fern's beams and archer shots? How often does he run out of kunai?
- Round 4: no crash with Frieren's ult? Does V1 arm and use the parry (yellow diamond) and parry teleports? Do David's claws line up with his hands in game?
- Round 3 (spawn_unit dropped in round 4): does spawn_unit + the stark_body buff visual show Stark and let him walk/attack? Does V1's parry catch Purple / Fern's big beam? Is the ting audible?
- Rian's current career holds a copy of the original Minato. Offered to overwrite it in the save.
