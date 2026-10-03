/* Skill Lab presets: complete custom champions you can load and then tweak.
 * Each preset returns { json, text } for a .data_champion. Times are in ticks (60 = 1 second).
 */
(function (root) {
  'use strict';

  // ---- small builders
  const hit = (damage, attack_ratio) => ({ type: 'Attack', damage, attack_ratio });
  const buff = (name, tick, extra) => ({ type: 'AddCasterBuff', only_to_enemy: false, buff_state: Object.assign({ name, duration: { Time: { tick } } }, extra || {}) });
  const unbuff = name => ({ type: 'RemoveCasterBuff', name });
  const all = (...effects) => ({ type: 'Combine', effects });
  const nothing = () => ({ type: 'Combine', effects: [] });
  const ifBuff = (buff_name, effect_buff, effect_none) => ({ type: 'SwitchByBuff', buff_name, effect_none: effect_none || nothing(), effect_buff: effect_buff || nothing() });
  const onHit = e => ({ effect: e, casting_type: 'Targeting' });
  const fx = name => ({ type: 'ViewEffect', name });          // visual at the target / landing spot
  const cfx = name => ({ type: 'CasterViewEffect', name });   // visual on the caster
  // Visual bindings for the generated VFX sheet mods/<mod>/vfx/<sheet>#sheet.png + #anim.fanim (see vfx.js)
  function views(modId, sheet, spec) {
    const anim = `asset/${modId}/vfx/${sheet}`;
    return {
      view_effects: (spec.effects || []).map(([name, tag, z, follow]) => ({ type: 'Animation', name, anim, tag, z: z || 0, is_follow: !!follow })),
      view_projectiles: (spec.projectiles || []).map(([name, tag, z]) => ({ type: 'Animated', name, anim, tag, z: z || 0, repeat: true })),
      view_buffs: (spec.buffs || []).map(([name, tag, z]) => ({ type: 'Animated', name, anim, tag, z: z || 0 })),
    };
  }

  /* ================================================================ Minato
   * Cooldowns use the game's real cooldown timer. (A first version faked them with buffs on a 1-second
   * real cooldown; in game the AI kept re-casting, so kunais and Rasengan charges piled up.)
   * Ability 1  Flying Raijin, 3 charges, 4 s real cooldown between casts (no empty casts, so no spam):
   *            1 throw a straight kunai; its seal stays on the ground (up to 12 s)
   *            2 teleport to the kunai → enemies around him are feared 0.1 s
   *            3 teleport back to the kunai again (same spot), no fear; the seal is used up
   *            A charge the AI doesn't use in time lapses and the next cast throws a fresh kunai.
   * Ability 2  Rasengan: charges the next basic attack for 6 s (knockback, or stun after a teleport).
   *            The cooldown counts from the cast and covers the charge window, so the next charge can only
   *            come 8 s after the Rasengan is gone at the earliest (6 + 8 = 14 s).
   * Ult        KCM: 15 s mode; every slot checks "minato_kcm" first and uses an empowered version.
   *            KCM also speeds up his real cooldowns (skill_cooldown_mult).
   * Buffs: hiraishin (12 s after any TP), minato_rasengan (charged), minato_kcm, kcm_burst
   */
  function minato(id, o) {
    o = Object.assign({
      // One real cooldown serves all 3 casts (6 s). Right after the throw a "raijin_haste" buff speeds that cooldown
      // up (skill_cooldown_mult) so the teleport (charge 2) is ready in ~0.75 s; the return (charge 3) and the next
      // throw each wait the normal 6 s. EXPERIMENT: skill_cooldown_mult's exact behaviour isn't documented.
      baRange: 50000,        // kunai basic attack (was a 23000 melee hit)
      baRatio: 70,
      kcmBaRatio: 85,
      baSpeed: 7000,
      markTicks: 240,
      chargeCd: 420,         // 7 s real cooldown (5 s in round 9; the backpack teleports don't need the cast)
      hasteMult: 700,        // +700% cooldown speed after the throw → 6 s / 8 = 0.75 s
      hasteTicks: 12,        // … only while the throw's own cooldown starts
      kunaiLife: 720,        // a kunai stays on the ground up to 12 s (native mod)
      stage2Window: 480,     // the 2nd cast must come within 8 s of the throw
      stage3Window: 660,     // the 3rd within 11 s of the 2nd
      fearTicks: 6,          // 0.1 s fear around him on arrival
      fearRadius: 18000,
      kunaiSpeed: 9000,      // straight kunai: ~18 ticks to full range (was 5000, then 7500)
      kunaiRange: 160000,    // the homing kunai's lifespan (distance) and the throw's cast range (native mod)
      primeTicks: 120,       // a teleport primes a Rasengan stun for 2 s
      rasCharge: 360,        // 6 s to use the charged Rasengan
      rasCdAfter: 480,       // 8 s after the charge is gone
      kcmTicks: 900,         // 15 s
      kcmCd: 3600,           // 60 s cooldown (was 50 s)
      kcmCdSpeed: 50,        // KCM: +50% cooldown speed (skill_cooldown_mult) for cooldowns that start during it
      kcmDelay: 15,          // KCM's buffs go on after the ult's cooldown has started (ult: duration 30, effect at 18)
      modId: 'tfm2_custom',
    }, o || {});
    const kunai = `${id}_kunai`;
    const V = n => `${id}_${n}`;                     // visual names
    const KCM = 'minato_kcm', RAS = 'minato_rasengan'; // buffs with their own visuals

    // --- Flying Raijin (3 casts), thrown and teleported by the native "Gojo & Minato rules" mod (0.3.0+):
    //     1st cast: raijin_throw on Minato + raijin_target on the enemy he aimed at → the mod throws a fan of three
    //       kunai: the middle one chases that enemy (it can curve) until it hits or its lifespan (160000 travelled)
    //       runs out and it drops; the two side kunai fly straight out ~22° either side. All three become teleport spots.
    //       Kurama Mode: 2× range, 1.5× speed.
    //     2nd / 3rd cast: tp_now (+ tp_last) → the mod teleports him to the best kunai: when he's low (≤35% HP) the one
    //       furthest from enemies; otherwise the one nearest the weakest enemy, skipping spots with 3+ enemies around
    //       (and not teleporting at all if every spot is that crowded). The 1st teleport fears enemies around him
    //       (0.1 s); every teleport primes the Rasengan stun (hiraishin). A used kunai is gone.
    //     (Before 0.3.0 this was 180 Delayed data checks per throw and a single straight kunai.)
    // A cooldown's length is fixed the moment it starts, from the current cooldown-speed stat, and that stat also
    // speeds up the ult. So the haste only lives long enough for the THROW's own cooldown to start (12 ticks):
    // it can't leak into a Rasengan or Kurama Mode cast. (It used to last until the teleport, up to 8 s.)
    const haste = buff('raijin_haste', o.hasteTicks, { skill_cooldown_mult: o.hasteMult });
    const throwKunai = all(unbuff('tp_now'), unbuff('tp_last'), unbuff('raijin_3'), buff('raijin_2', o.stage2Window), haste,
      cfx(V('flash')), buff('raijin_throw', 6), { type: 'AddBuff', buff_state: { name: 'raijin_target', duration: { Time: { tick: 6 } } } });
    const flyingRaijin = ifBuff('raijin_3', all(unbuff('raijin_3'), buff('tp_now', 30), buff('tp_last', 30)),
      ifBuff('raijin_2', all(unbuff('raijin_2'), unbuff('raijin_haste'), buff('raijin_3', o.stage3Window), buff('tp_now', 30)), throwKunai));

    // --- Rasengan on the basic attack
    // 50 + 80% AD, +20% AD per dodge stack he holds (minato_flow1..5: up to 50 + 180%, the old full hit)
    const byStacks = (base, ratio, per) => [5, 4, 3, 2, 1].reduceRight((rest, n) => ifBuff(`minato_flow${n}`, hit(base, ratio + per * n), rest), hit(base, ratio));
    const rasenganHit = all(
      unbuff(RAS),
      byStacks(50, 80, 20), fx(V('rasengan_hit')),
      ifBuff('hiraishin', all({ type: 'Stun', duration: 45 }, unbuff('hiraishin')), { type: 'Knockback', speed: 2500, tick: 12 }));
    // Kunai basic attack (Rian, Oct 1: "a speed character using projectiles"): a thrown kunai at range that marks the
    // enemy for 4 s (minato_mark: his next teleport cast goes straight behind them). With the Rasengan charged it also
    // tags them (minato_ras_go): the native rules flash him behind them and slam the Rasengan, then flash him back.
    // Kurama Mode: a piercing kunai. (The old melee hit + rasenganHit are no longer used by the basic attack.)
    const markHit = ratio => [onHit(hit(0, ratio)), onHit({ type: 'AddBuff', buff_state: { name: 'minato_mark', duration: { Time: { tick: o.markTicks } } } })];
    const kunaiShot = extra => ({ type: 'TargetProjectile', speed: o.baSpeed, name: kunai, y_offset: 0, applied_target: 'Enemy', applied_effects: [...markHit(o.baRatio), ...extra] });
    const rasGo = onHit({ type: 'AddBuff', buff_state: { name: 'minato_ras_go', duration: { Time: { tick: 6 } } } });
    const kcmShot = { type: 'LinearProjectile', penetrate: true, speed: o.baSpeed + 2000, range: o.baRange + 15000, name: kunai, shape: { Circle: { radius: 5000 } },
      applied_target: 'Enemy', applied_effects: markHit(o.kcmBaRatio), end_effects: [] };
    const chargeRasengan = buff(RAS, o.rasCharge, { attack_speed_mult: 20 });
    const bigBall = all(cfx(V('big_ball')),
      { type: 'RangeEffect', shape: { Circle: { radius: 20000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: [
        byStacks(60, 90, 15),
        ifBuff('hiraishin', { type: 'Stun', duration: 45 }, { type: 'Knockback', speed: 2500, tick: 12 })] },
      unbuff('hiraishin'));

    // Kurama Mode. Its buffs go on kcmDelay ticks after the effect, once the ult's own cooldown has started, so its
    // cooldown speed can't shorten the ult itself (it did: 60 s became ~40 s; a +5000% "refresh" burst made it ~1 s).
    // The "refresh" of Flying Raijin is a free Kurama throw: a second, separate fan of kunai right away (up to 6 kunai
    // on the ground). (A running cooldown can't be shortened: its length is fixed when it starts.)
    const kcm = all(
      cfx(V('flash')),
      { type: 'Delayed', tick: o.kcmDelay, effects: [all(
        buff(KCM, o.kcmTicks, { move_speed_mult: 20, attack_speed_mult: 15, damaged_reduce: 10, skill_cooldown_mult: o.kcmCdSpeed }),
        buff('kcm_burst', 120, { cc_immune: true }))] },
      // the free Kurama fan is its own set of kunai (native mod): it doesn't touch his Flying Raijin stage, so his
      // next cast still does what it would have (throw or teleport)
      { type: 'Delayed', tick: o.kcmDelay + 1, effects: [buff('raijin_kthrow', 6)] });

    // action_name must be an animation tag of the sprite. The Ninja sprite has: attack, skill_pre, skill2_attack, ult_pre
    // Look: recoloured Ninja body (sprites-data.js 'minato'); its tags are attack, skill_pre, skill2_attack, ult_pre
    const ANIM = { attack: 'attack', skill: 'skill_pre', skill2: 'skill2_attack', ult: 'ult_pre' };
    const act = (slot, a) => Object.assign({ action_name: ANIM[slot], cancelable: false, growth_range: 0, can_use_with_move: false }, a,
      slot === 'attack' ? {} : { description: `#asset/base/text/champion?description.${id}.${slot}` });

    const json = {
      id, category: 'Assassin', tags: ['AD', 'CC', 'Range'],   // ranged kunai basic attack (was Melee)
      // own sprite (sprites-data.js → mods/<folder>/champions/<id>#sheet.png + #anim.fanim); Skill Lab → Edit sprite
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: { attack: 92, magic_power: 0, hp: 950, defence: 25, magic_resistance: 15, move_speed: 1120, hp_regen: 0, stack: 0, crit_chance: 0 },
      growth: { attack: 20, magic_power: 0, hp: 85, defence: 7, magic_resistance: 3, move_speed: 14, hp_regen: 0, stack: 0, crit_chance: 0 },
      attack: act('attack', { duration: 20, cooltime: 50, start_timing: 10, cancelable: true, range: o.baRange, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack',
        effect: ifBuff(KCM, kcmShot, ifBuff(RAS, kunaiShot([rasGo]), kunaiShot([]))) }),
      // cast on the move (no stopping to throw or teleport), and quick: the effect lands 3 ticks in
      skill: act('skill', { duration: 10, cooltime: o.chargeCd, start_timing: 3, can_use_with_move: true, range: o.kunaiRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: flyingRaijin }),
      skill2: act('skill2', { duration: 14, cooltime: o.rasCharge + o.rasCdAfter, start_timing: 6, range: 25000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: ifBuff(KCM, bigBall, chargeRasengan) }),
      ult: act('ult', { duration: 30, cooltime: o.kcmCd, start_timing: 18, range: 60000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: kcm }),
      ...views(o.modId, 'minato', {
        projectiles: [[kunai, 'kunai', 1]],
        effects: [[V('flash'), 'flash', 1], [V('kunai_seal'), 'kunai_seal', -1], [V('kunai_planted'), 'kunai_planted', -1], [V('kunai_marker'), 'kunai_marker', -1], [V('rasengan_hit'), 'rasengan_hit', 1], [V('big_ball'), 'big_ball', 1],
          [V('after_r'), 'after_r', 0], [V('after_l'), 'after_l', 0], [V('slash_big'), 'slash_big', 2]],
        // dodge stacks (native): minato_flow1..5, shown as golden pips over his head
        buffs: [[KCM, 'kcm', -1], [RAS, 'rasengan', 1], ...[1, 2, 3, 4, 5].map(n => [`minato_flow${n}`, `flow_${n}`, 3]),
          ['minato_pack1', 'pack_1', 3], ['minato_pack2', 'pack_2', 3],
          // his Flying Raijin seal on an enemy his kunai hit
          ['minato_mark', 'kunai_marker', 2]],
      }),
    };
    const s = t => +(t / 60).toFixed(1) + 's';
    const text = {
      name: 'Minato',
      skill: `Basic attack: a thrown kunai (range ${o.baRange}, ${o.baRatio}% AD) that marks the enemy with his seal for ${s(o.markTicks)}. Flying Raijin, 3 casts: the teleport is ready about ${s(Math.round(o.chargeCd / (1 + o.hasteMult / 100)))} after the throw; the second teleport and the next throw each take ${s(o.chargeCd)}. 1st: throws three kunai in a wide fan, all flying straight and far. The middle one hits the first enemy champion in its path (40 + 60% AD) and drops there, or where its range runs out; the two side kunai fly their full range. Each kunai stays on the ground for up to ${s(o.kunaiLife)}. 2nd and 3rd: Minato teleports to the best kunai: when he is low, the one furthest from enemies, right away; only to a kunai that's clearly further from the enemies than he is (kunai thrown at them to slow them are no escape); otherwise the one nearest the weakest enemy, if it lands him on an enemy, never into a crowd of three or more. If no kunai lies near an enemy any more, the cast throws a kunai from his backpack instead (the old ones stay); only with the backpack empty does it throw a fresh fan that replaces the old one. A kunai that still lies near an enemy but isn't safe to land on (a crowd, a tower, a danger zone) is kept. A teleport goes straight behind a marked enemy first (using the mark), before any kunai. Hit and run: every attacking teleport leaves a seal where he stood, and ${s(24)} after the strike he flashes back to it if it's still safe. Every teleport slashes the enemies around where he lands: with 3+ dodge stacks a big lightning slash (75 + 115% AD) that spends 2 stacks, otherwise a weak slash (35 + 55% AD). Then, if his Rasengan is charged and he still has a stack, it goes off at once on the nearest enemy (spends 1 stack); with no stack left it waits for his basic attack. Passive, melee dodge: when an enemy champion right next to him starts a basic attack at him, he spends a dodge stack to flash out before it lands (to a kunai nearby, or a step away), with the same afterimage; at most every 3s, and only with a stack to spend. Teleporting to a kunai that hit an enemy fears enemies around him for 0.5s; otherwise only the first teleport fears them, for ${s(o.fearTicks)}. Every teleport primes a Rasengan stun for ${s(o.primeTicks)}. Passive, Flying Raijin dodge: kunai on the ground are dodge charges. When a shot from an enemy champion is about to hit him, he teleports to the nearest kunai out of its way (using it up) and leaves a golden afterimage for 0.5s that takes the hit instead (he can't be hit for those 0.5s). With no kunai down he sidesteps instead, at most every 3s. At most one dodge every 1.5s; not while stunned or sealed, and melee hits can't be dodged. He also dodges out of DIO's red mark and David's gravity just before they strike, and never teleports into an Unlimited Void, DIO's time-stop field or a marked area. In Kurama Mode he spots shots 50% further out, the afterimage lasts 0.75s and he can dodge every 1s. Every dodge adds a stack for 7s (refreshed, max 5): +8% attack, +6% attack speed and +5% move speed each; at 5 stacks also 10% lifesteal. His normal kunai stop at walls. Backpack: walking over one of his kunai picks it up (up to 2, shown as small blades over his head), but only a kunai with no enemy near it, on a safe spot. The pack comes first when he's chasing (an enemy past his reach that runs from him or is at 40% HP or less) and no kunai lies near an enemy, or when none of his kunai is on the ground: he throws one at that enemy, 70% of the range, half the damage (20 + 30% AD) and half the lifespan. If it lands on or next to an enemy he flashes straight to it (no cast needed; teleport slash and Rasengan as usual), unless that spot is a crowd, a deadly tower or a danger zone, or he's fleeing; otherwise it stays on the ground as a teleport spot. At most one pack throw every 2s. Kurama Mode: activating it throws one homing kunai with a long range (320000) that flies through walls and hits the first enemy it reaches (60 + 80% AD). Where it hits, or at the end of its range, it splits into 5 kunai that fly on through walls: 3 ahead (120000) and 2 back the way it came (90000). They're a separate set, so up to 8 kunai can lie on the ground. In Kurama Mode his thrown kunai fly twice as far and 1.5× as fast. (Needs the native "Gojo & Minato rules" mod.)`,
      skill2: `Charges a Rasengan for ${s(o.rasCharge)} (+20% attack speed). Its damage grows with his dodge stacks: 50 + 80% AD, +20% AD per stack (50 + 180% at 5). If he teleports in with a dodge stack to spare, it goes off on arrival (stun 0.75s, spends 1 stack); otherwise his next kunai carries it: when it hits a champion he flashes behind them, slams the Rasengan (stun 0.75s) and flashes back out. Recharges ${s(o.rasCdAfter)} after the charge is gone. In Kurama Mode: instantly blasts a Big Ball Rasengan around him (60 + 90% AD, +15% AD per stack) instead.`,
      ult: `Kurama Chakra Mode for ${s(o.kcmTicks)}: his kunai pierce everything in a line (${o.kcmBaRatio}% AD); a glowing orange chakra cloak; throws a long homing kunai that splits into 5 (3 ahead, 2 back) through walls; his dodge spots shots 50% further out with a 0.75s afterimage. +20% move speed, +15% attack speed, +${o.kcmCdSpeed}% cooldown speed, 10% less damage taken and crowd-control immunity for the first 2s. Rasengan becomes a Big Ball Rasengan. Cooldown ${s(o.kcmCd)}.`,
    };
    return { json, text, vfx: 'minato', sprite: 'minato', spriteFallback: 'asset/base/aseprite_resources/champions/ninja' };
  }

  /* ================================================================ Gojo
   * Passive    Infinity: skill damage can't touch him. Basic attacks chip an invisible shield; when it
   *            breaks, Infinity is down for 15 s, then comes back. (Checked at the start of each of his actions.)
   * Ability 1  Blue: raises the Blue flag. His next basic attack launches the Blue orb at its target instead;
   *            the orb collapses into a black hole that yanks enemies in (stronger from level 3).
   * Ability 2  Red: raises the Red flag. His next basic attack fires the Red blast instead: a piercing shot
   *            that shoves everything in its path.
   * Purple     Both flags up: the next basic attack becomes Hollow Purple: 0.56 s wind-up (Gojo stands still),
   *            then a huge, slow, map-crossing orb, and the recoil knocks Gojo back. Both flags are used up and
   *            both skills recharge at half speed for a while (about 2× cooldown).
   *            The game's AI casts a skill as soon as it's ready and an enemy champion is close, so Purple
   *            happens whenever both are ready together: Blue and Red get cast back to back before his next swing.
   * A flag lasts flagTicks. Real cooldown = flag window + blueCd / redCd, so it effectively counts from
   * the moment the flag is used.
   * Ult        Unlimited Void: Gojo stands still to cast (self-stun), then everyone around him — enemies and,
   *            optionally, his own teammates — is stunned. Slow cast: longer cast, longer stun, big damage
   *            buff + a little speed. Fast cast: short cast, short stun, big speed buff + a little damage.
   *            domain: 'auto' picks slow while Infinity is up, fast when broken.
   * Buffs: gojo_infinity (WithShield), infinity_on (marker), infinity_down, gojo_blue, gojo_red (flags),
   *        gojo_purple_strain (slower cooldowns after Purple), void (domain buff)
   * (Sep 30, v2: replaces the cast-fires-the-skill kit with its Red→Blue teleport combo, orb-hover Purple
   *  and combo locks.)
   */
  function gojo(id, o) {
    o = Object.assign({
      domain: 'auto',          // 'auto' | 'slow' | 'fast'
      // Teammates in the domain: handled by the native mod (Gojo domain rules): the closest one stays free and
      // protected, the rest are stunned. Without that mod, teammates are simply not affected.
      domainHitsAllies: false,
      domainGuard: true,       // while the domain is open Gojo takes no damage from attacks/skills and ignores CC
      domainRadius: 76000,
      domainWarn: 0,           // no warning: the seal starts spreading on the press (was 60 = 1 s)
      growStart: 8000,         // the seal starts this wide around Gojo on the press …
      growStep: 5,             // … and spreads every 5 ticks until it reaches domainRadius when the domain is conjured
      infinityShield: 200,     // damage Infinity absorbs before it breaks (+ infinityAp% AP); was 250 + 60%
      infinityAp: 50,
      infinitySkillBlock: 75,  // % of skill damage blocked while Infinity is up (was 100: skills couldn't touch him)
      infinityAttackBlock: 30, // % of basic-attack damage blocked while Infinity is up
      attackApRatio: 35,       // his basic attacks also deal this % of AP as magic damage (was 60)
      infinityDown: 900,       // 15 s
      flagTicks: 300,          // a flag waits 5 s for his next basic attack, then lapses (was 2.5 s)
      conjureTicks: 36,        // Blue / Red: he stops for 0.6 s to conjure …
      conjureAt: 24,           // … and the flag (or, in the domain, the shot) comes at 0.4 s
      flagRange: 70000,        // the AI raises a flag with an enemy champion this close, then walks in to swing (was 30000)
      blueCd: 450, redCd: 330, // real cooltime = flag window + this = 12.5 s / 10.5 s from the cast (unchanged when the window grew)
      holeTicks: 20,           // the black hole is on screen 0.33 s and pulls ONCE, as it appears
      holePull: 4800,          // one pull, 10 ticks (round 26: 1.5× stronger, was 3200)
      holePullL3: 6000,        // from level 3 (was 4000)
      holeRadius: 32000,       // round 26: the pull reaches out to the art's edge (was 26000)
      blueTravel: 12,          // ticks the Blue orb is in the air
      redSpeed: 8000,
      redPush: { speed: 6000, tick: 9 },
      redBurstRadius: 20000,   // Red explodes where it stops (first enemy hit, or max range)
      redBurst: [40, 50],      // burst damage: flat + % AP (was 60 + 70%; it now lands on the target every time)
      purpleDmg: [200, 160],   // Hollow Purple: flat + % AP (was 260 + 220%)
      redBurstPush: { speed: 3000, tick: 8 },
      purpleWindup: 34,        // 0.56 s
      purpleRecoil: { speed: 3000, tick: 12 },   // Gojo slides back ~36000 as it fires
      purplePush: { speed: 4500, tick: 10 },     // everyone Purple hits is blasted back
      flagAutoFire: 75,        // outside the domain an unused flag fires by itself after 1.25 s (no more running with flags up)
      domainPurpleWait: 45,    // in the domain a lone flag waits 0.75 s for the other skill (→ Purple), then fires alone (the 2nd cast lands 36 ticks after the 1st)
      purpleStrain: 50,        // after Purple both skills recharge 50% slower …
      purpleStrainTicks: 1260, // … for 21 s: exactly 2× on both cooldowns
      purpleRest: 1200,        // no second Purple for 20 s: with both flags up he fires Blue and Red separately instead
      //                          (the domain's cooldown-speed buff cancelled the strain, so Purple got spammed there)
      // domain buff ('void'): Gojo, and through the native mod his one free teammate, once the domain is conjured
      slow: { cast: 150, selfLock: 30, stun: 240, buffTicks: 600, cd: 3600, buff: { magic_power_mult: 30, attack_mult: 30, move_speed_mult: 25, attack_speed_mult: 25, skill_cooldown_mult: 100 } },
      fast: { cast: 45, selfLock: 15, stun: 120, buffTicks: 600, cd: 3600, buff: { move_speed_mult: 40, attack_speed_mult: 40, magic_power_mult: 15, attack_mult: 15, skill_cooldown_mult: 100 } },
      modId: 'tfm2_custom',
    }, o || {});
    const INF = 'gojo_infinity', BLUE = 'gojo_blue', RED = 'gojo_red';   // buffs with their own visuals
    const ap = (damage, attack_ratio) => ({ type: 'ApAttack', damage, attack_ratio });
    // "Apply to Gojo himself". WithSelf turned out to hit the ult's target in game (the enemy got the cast stun),
    // so self-effects go through a tiny area that only accepts the caster.
    const self = (...effects) => ({ type: 'RangeEffect', shape: { Circle: { radius: 1000 } }, target: 'AllyOnlySelf', apply_type: 'AroundCaster', effects });
    const P = n => `${id}_${n}`;

    // --- Infinity upkeep, run first in every action
    const infinity = ifBuff(INF, nothing(),
      ifBuff('infinity_down', nothing(),
        ifBuff('infinity_on',
          all(unbuff('infinity_on'), buff('infinity_down', o.infinityDown)),            // it was up and is gone: it broke
          all(self({ type: 'Shield', amount: o.infinityShield, attack_ratio: 0, ap_ratio: o.infinityAp, tick: 36000 }),
            { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: INF, duration: 'WithShield', skill_damaged_reduce: o.infinitySkillBlock, base_attack_damaged_reduce: o.infinityAttackBlock } },
            { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: 'infinity_on', duration: 'Permanent' } }))));

    // --- what the flagged basic attack fires
    // Blue orb → black hole: ONE violent pull the moment it appears (period longer than its life → a single pulse)
    const blackHole = (pull, dmg) => ({ type: 'RangePeriodProjectile', name: P('blue_hole'), tick: o.holeTicks, period: o.holeTicks + 30, first_delay: 0, shape: { Circle: { radius: o.holeRadius } },
      applied_target: 'EnemyWithoutTower', applied_effects: [onHit({ type: 'Pull', speed: pull, tick: 10 }), onHit(ap(dmg, 80))], end_effects: [] });
    const blueShot = { type: 'ParabolicProjectile', name: P('blue'), travel_time: o.blueTravel, range: 80000, range_effect_name: '', shape: { Circle: { radius: 8000 } },
      applied_target: 'Enemy', applied_effects: [onHit(ap(40, 40))],
      end_effects: [{ type: 'SwitchByLevel3', effect_start: blackHole(o.holePull, 60), effect_level3: blackHole(o.holePullL3, 100) }] };   // collapses the instant it lands
    const redShot = { type: 'LinearProjectile', penetrate: false, speed: o.redSpeed, range: 70000, name: P('red'), shape: { Circle: { radius: 11000 } },
      applied_target: 'EnemyWithoutTower', applied_effects: [onHit(ap(40, 50)), onHit(Object.assign({ type: 'Knockback' }, o.redPush))],
      // stops at the first enemy it hits (or at full range) and explodes there (one pulse, like the black hole)
      end_effects: [fx(P('red_burst')), { type: 'RangePeriodProjectile', name: P('red_burst_zone'), tick: 6, period: 36, first_delay: 0, shape: { Circle: { radius: o.redBurstRadius } },
        applied_target: 'EnemyWithoutTower', applied_effects: [onHit(ap(o.redBurst[0], o.redBurst[1])), onHit(Object.assign({ type: 'Knockback' }, o.redBurstPush))], end_effects: [] }] };
    const purpleShot = { type: 'LinearProjectile', penetrate: true, speed: 4200, range: 400000, name: P('purple'), shape: { Circle: { radius: 26000 } },
      applied_target: 'EnemyWithoutTower', applied_effects: [onHit(ap(o.purpleDmg[0], o.purpleDmg[1])), onHit(Object.assign({ type: 'Knockback' }, o.purplePush)), onHit(fx(P('purple_hit')))], end_effects: [] };
    const purple = all(unbuff(BLUE), unbuff(RED),
      cfx(P('purple_charge')), self({ type: 'Stun', duration: o.purpleWindup - 2 }),            // wind-up: he stands still
      buff('gojo_purple_strain', o.purpleStrainTicks, { skill_cooldown_mult: -o.purpleStrain }), buff('gojo_purple_rest', o.purpleRest),
      { type: 'Delayed', tick: o.purpleWindup, effects: [purpleShot, Object.assign({ type: 'MoveBack' }, o.purpleRecoil)] });

    // Hollow Purple needs the ultimate unlocked (gojo_ult_learned: set by the native "ult_learned" passive the moment
    // the ult is learned, or by his first Unlimited Void as a fallback) and no Purple in the last purpleRest ticks.
    // Otherwise `instead` happens (the single skill).
    const purpleOr = instead => ifBuff('gojo_ult_learned', ifBuff('gojo_purple_rest', instead, purple), instead);

    // --- basic attack: melee normally; with a flag up it fires that skill instead and uses the flag
    const attack = all(infinity,
      // both flags: Hollow Purple if allowed, otherwise just Blue now (Red on the next swing)
      ifBuff(BLUE, ifBuff(RED, purpleOr(all(unbuff(BLUE), blueShot)), all(unbuff(BLUE), blueShot)),
        ifBuff(RED, all(unbuff(RED), redShot), all(hit(0, 100), ap(0, o.attackApRatio)))));

    // --- ult: Unlimited Void
    // The lock starts on the ult press: everyone inside is sealed at once (stun + root + no attacks + no skills
    // + no movement skills, like the Taoist's talismans combined) and stays sealed while the domain is conjured
    // and for its full duration. Gojo himself is only locked for a short moment (selfLock), then acts freely.
    // Void Overload: a stat debuff that goes on together with the seal. CC immunity blocks CC but not stat debuffs,
    // so a CC-immune champion is still crippled; the native "Gojo domain rules" mod also strips the immunity.
    // (A negative `range` is not allowed at all; −90 on attack speed/cooldowns stays clear of zero-speed maths.)
    const overload = tick => ({ type: 'AddBuff', buff_state: { name: P('void_overload'), duration: { Time: { tick } },
      move_speed_mult: -100, attack_speed_mult: -90, attack_mult: -90, magic_power_mult: -90, skill_cooldown_mult: -90, ult_cooldown_mult: -90 } });
    const seal = tick => [{ type: 'Stun', duration: tick }, { type: 'Bind', duration: tick }, { type: 'BlockAttack', tick }, { type: 'BlockSkill', tick }, { type: 'BlockMoveSkill', tick }, overload(tick)];
    const cast = (m, tag) => {
      // Timeline from the press: warn ticks of warning (charge-up only, nobody sealed yet: time to walk out),
      // then the seal spreads with the ring for m.cast ticks, then the finished domain holds for m.stun ticks.
      const W = o.domainWarn;
      const total = W + m.cast + m.stun;
      const R = o.domainRadius;
      // The seal spreads with the ring: from right around the press spot to the edge. Each step is a one-pulse zone
      // spawned at the press (through the self-only area, so it's centred on the spot Gojo pressed it); whoever the
      // growing edge reaches is sealed until the domain closes.
      const radiusAt = t => Math.round(o.growStart + (R - o.growStart) * Math.min(1, t / m.cast));
      const steps = [];
      for (let t = 0; t < m.cast; t += o.growStep) steps.push(t);
      const grow = target => steps.map(t => self({ type: 'RangePeriodProjectile', name: P('void_grow'), tick: W + t + 4, period: W + t + 600, first_delay: W + t,
        shape: { Circle: { radius: radiusAt(t) } }, applied_target: target, applied_effects: seal(total - W - t).map(onHit), end_effects: [] }));
      // once conjured: the full domain re-seals anyone inside every 1/6 s until it closes
      const zone = target => self({ type: 'RangePeriodProjectile', name: P('void_zone'), tick: total, period: 10, first_delay: W + m.cast, shape: { Circle: { radius: R } },
        applied_target: target, applied_effects: seal(16).map(onHit), end_effects: [] });
      const targets = ['EnemyWithoutTower'].concat(o.domainHitsAllies ? ['AllyNotSelf'] : []);
      // for the native mod: gojo_void_active = the domain's whole life; gojo_void_warn = the warning second;
      // gojo_void_conj<cast> = the seal is spreading (its name carries the spread time, so the native edge grows the same way)
      const guard = o.domainGuard ? [buff('gojo_void_guard', total, { base_attack_damaged_reduce: 100, skill_damaged_reduce: 100, cc_immune: true })] : [];
      return all(cfx(P('void_cast_' + tag)),                                             // the warning: charge-up on the spot
        buff('gojo_void_active', total), buff('gojo_void_warn', W + 2), ...guard,   // warn overlaps conj by 2 ticks (native: no gap)
        { type: 'Delayed', tick: W, effects: [buff('gojo_void_conj' + m.cast, m.cast)] },
        ...targets.flatMap(grow), ...targets.map(zone),
        self({ type: 'Stun', duration: m.selfLock }),                                   // Gojo: short lock only
        // Visuals and the "danger zone" live on the press spot: a tiny invisible projectile lands right at his feet,
        // and its end_effects keep that spot (like Minato's kunai):
        //  * a harmless-looking incoming area attack the size of the domain lands there when the seal starts, so the
        //    game's AI can see it coming and may walk out (how well depends on each player's skill-avoid);
        //  * the ring starts spreading when the warning ends, and the finished domain is drawn once conjured.
        // (0.1.17 used a RangePeriodProjectile here; its end_effects need {effect, casting_type} entries, so the game
        //  rejected the whole file.)
        { type: 'LinearProjectile', penetrate: true, speed: 1000, range: 1000, name: P('void_anchor'), shape: { Circle: { radius: 1000 } },
          applied_target: 'EnemyChampion', applied_effects: [], end_effects: [
            ...(W > 0 ? [{ type: 'ParabolicProjectile', name: P('void_warn'), travel_time: W, range: 80000, range_effect_name: '', shape: { Circle: { radius: R } },
              applied_target: 'EnemyWithoutTower', applied_effects: [onHit(ap(10, 0))], end_effects: [] }] : []),
            { type: 'Delayed', tick: Math.max(0, W - 1), effects: [fx(P('void_ring_' + tag))] },
            { type: 'Delayed', tick: W + m.cast - 1, effects: [fx(P('void_' + tag))] }] },
        { type: 'Delayed', tick: W + m.cast, effects: [buff('void', m.buffTicks, m.buff)] });   // fully conjured: Gojo's domain buff
    };
    const ult = o.domain === 'slow' ? cast(o.slow, 'slow') : o.domain === 'fast' ? cast(o.fast, 'fast') : ifBuff(INF, cast(o.slow, 'slow'), cast(o.fast, 'fast'));

    const act = (slot, a) => Object.assign({ action_name: slot === 'skill' ? 'skill1' : slot, cancelable: false, growth_range: 0, can_use_with_move: false }, a,
      slot === 'attack' ? {} : { description: `#asset/base/text/champion?description.${id}.${slot}` });
    // Inside his own domain (everyone near him is sealed) the skills are used on the spot, Purple first:
    //  * the other flag is already up → Hollow Purple right away;
    //  * otherwise the flag goes up and waits domainPurpleWait ticks for the other skill (the AI casts a ready
    //    skill immediately, so with both ready the second cast lands in that window → Purple);
    //    if it's still unused after the wait (no Purple, no basic attack took it), the skill fires by itself.
    const other = { [BLUE]: RED, [RED]: BLUE };
    const shotOf = { [BLUE]: blueShot, [RED]: redShot };
    // Raising a flag: the next basic attack uses it; if no swing has used it after `wait` ticks (he's retreating,
    // chasing, low on HP…) it fires by itself at the cast target, so a flag is never wasted.
    // With the other flag already up (and Purple allowed) the cast becomes Hollow Purple straight away.
    const raise = (flag, wait) => ifBuff(other[flag], purpleOr(shotOf[flag]), all(buff(flag, o.flagTicks),
      { type: 'Delayed', tick: wait, effects: [ifBuff(flag, all(unbuff(flag), shotOf[flag]))] }));
    // Gojo stands still to conjure Blue / Red (can_use_with_move is false): the flag appears at conjureAt, he is
    // busy for conjureTicks in total. (Was 12 / 6 ticks: too short to see him stop.)
    const flagAction = (slot, flag, cd) => act(slot, { duration: o.conjureTicks, cooltime: o.flagTicks + cd, start_timing: o.conjureAt, range: o.flagRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
      effect: all(infinity, ifBuff('gojo_void_active', raise(flag, o.domainPurpleWait), raise(flag, o.flagAutoFire))) });
    const json = {
      id, category: 'Magician', tags: ['AP', 'CC', 'Melee'],
      // own sprite (sprites-data.js); tags: idle, run, attack, skill1, skill2, ult, hit, dead
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: { attack: 70, magic_power: 60, hp: 850, defence: 25, magic_resistance: 25, move_speed: 1080, hp_regen: 0, stack: 0, crit_chance: 0 },
      growth: { attack: 14, magic_power: 14, hp: 95, defence: 6, magic_resistance: 2, move_speed: 10, hp_regen: 0, stack: 0, crit_chance: 0 },
      attack: act('attack', { duration: 20, cooltime: 55, start_timing: 12, cancelable: true, range: 23000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: attack }),
      skill: flagAction('skill', BLUE, o.blueCd),
      skill2: flagAction('skill2', RED, o.redCd),
      ult: act('ult', { duration: 16, cooltime: o.domain === 'slow' ? o.slow.cd : o.domain === 'fast' ? o.fast.cd : Math.round((o.slow.cd + o.fast.cd) / 2), start_timing: 8, range: o.domainRadius - 5000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(infinity, { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: 'gojo_ult_learned', duration: 'Permanent' } }, ult) }),
      passive_ult: { passive_ref: 'tfm2_custom_ai:ult_learned', params: {} },   // native "Gojo domain rules" mod
      ...views(o.modId, 'gojo', {
        projectiles: [[P('blue'), 'blue', 1], [P('red'), 'red', 1], [P('purple'), 'purple', 1], [P('blue_hole'), 'blue_hole', -1]],
        effects: [[P('red_burst'), 'red_burst', 1], [P('purple_charge'), 'purple', 1, true], [P('purple_hit'), 'purple_hit', 1], [P('void_slow'), 'void_slow', -1], [P('void_fast'), 'void_fast', -1], [P('void_ring_slow'), 'void_ring_slow', -1], [P('void_ring_fast'), 'void_ring_fast', -1], [P('void_cast_slow'), 'void_cast_slow', 1, false], [P('void_cast_fast'), 'void_cast_fast', 1, false]],
        buffs: [[INF, 'infinity', 1], [BLUE, 'flag_blue', 1], [RED, 'flag_red', 1]],
      }),
    };
    const s = t => +(t / 60).toFixed(2) + 's';
    const mode = m => `after a ${s(o.domainWarn)} warning the seal spreads for ${s(m.cast)} (Gojo is locked for only ${s(m.selfLock)}), and those it catches stay sealed until ${s(o.domainWarn + m.cast + m.stun)} after the press; once conjured, Gojo and his free teammate get +${m.buff.skill_cooldown_mult}% cooldown speed, ${[m.buff.magic_power_mult && `+${m.buff.magic_power_mult}% damage`, m.buff.move_speed_mult && `+${m.buff.move_speed_mult}% move speed`, m.buff.attack_speed_mult && `+${m.buff.attack_speed_mult}% attack speed`].filter(Boolean).join(', ')} for ${s(m.buffTicks)}`;
    const text = {
      name: 'Gojo',
      skill: `Passive, Infinity: skill damage to Gojo is cut by ${o.infinitySkillBlock}% and basic attacks by ${o.infinityAttackBlock}%; whatever gets through wears down his Infinity (${o.infinityShield} + ${o.infinityAp}% AP); once broken it returns after ${s(o.infinityDown)}. His basic attacks also deal ${o.attackApRatio}% AP as magic damage. Blue: raises the Blue flag for ${s(o.flagTicks)}. His next basic attack launches the Blue orb at its target (40 + 40% AP), which collapses the instant it lands into a black hole that yanks enemies within ${o.holeRadius} hard to its centre (60 + 80% AP, an even stronger pull and 100 + 80% AP from level 3). The flag is used up. Cooldown ${s(o.flagTicks + o.blueCd)} from the cast. With the Red flag also up, that attack becomes Hollow Purple instead.`,
      skill2: `Red: raises the Red flag for ${s(o.flagTicks)}. His next basic attack fires the Red blast: a fast shot that shoves the first enemy it hits (40 + 50% AP) and explodes there (or at the end of its range), blasting everyone around that spot away (${o.redBurst[0]} + ${o.redBurst[1]}% AP). The flag is used up. Cooldown ${s(o.flagTicks + o.redCd)} from the cast. Inside his own domain he uses them on the sealed enemies right away, Purple first: with both ready they combine into Hollow Purple; a lone Blue or Red fires on its own after 0.75 s. If he hasn't swung after ${s(o.flagAutoFire)}, a raised flag fires by itself at its target. Hollow Purple (only once his ultimate is unlocked): with both flags up, his next basic attack winds up for ${s(o.purpleWindup)}, then fires a huge, slow orb that crosses the map (${o.purpleDmg[0]} + ${o.purpleDmg[1]}% AP), blasting back everyone it hits, and the recoil knocks Gojo back. Both flags are used up and Blue and Red recharge ${o.purpleStrain}% slower for ${s(o.purpleStrainTicks)}. After a Purple he can't form another one for ${s(o.purpleRest)}: Blue and Red fire separately in the meantime.`,
      ult: `Unlimited Void: ${o.domainWarn ? `Gojo marks the spot; ${s(o.domainWarn)} later a seal spreads out from it with the ring.` : 'the moment Gojo casts it, a seal spreads out from his spot with the ring.'} Every enemy the ring reaches is sealed on the spot (can't move, attack or use skills) until the domain closes; it reaches the edge as the domain is conjured. The domain stays where it opened. Void Overload also cripples crowd-control-immune champions: they can't move, barely attack or deal damage, and their cooldowns almost freeze.${o.domainGuard ? ' While it is open, nothing from outside can hurt or crowd-control Gojo.' : ''} With the native "Gojo domain rules" mod: his closest teammate inside stays free and protected, every other teammate inside is stunned, crowd-control immunity is ignored, nobody can get in until it closes, and once it's conjured its edge is a wall that those caught inside can't be knocked or dragged through. ${o.domain === 'fast' ? 'Fast cast' : o.domain === 'slow' ? 'Slow cast' : 'Slow cast while Infinity is up, fast cast when it is broken'}. Slow: ${mode(o.slow)}. Fast: ${mode(o.fast)}. Cooldown ${s(json.ult.cooltime)}.`,
    };
    return { json, text, vfx: 'gojo', sprite: 'gojo', spriteFallback: 'asset/base/aseprite_resources/champions/dark_mage' };
  }

  // ---- shared helpers for the batch-2 champions
  const ap = (damage, attack_ratio) => ({ type: 'ApAttack', damage, attack_ratio });
  const markTarget = (name, tick) => ({ type: 'AddBuff', buff_state: { name, duration: { Time: { tick: tick || 6 } } } });
  // "Apply to myself": a tiny area that only accepts the caster (WithSelf hits the action's target in game)
  const selfArea = (...effects) => ({ type: 'RangeEffect', shape: { Circle: { radius: 1000 } }, target: 'AllyOnlySelf', apply_type: 'AroundCaster', effects });
  const delayed = (tick, ...effects) => ({ type: 'Delayed', tick, effects });
  const actFor = (id, anims) => (slot, a) => Object.assign({ action_name: anims[slot], cancelable: false, growth_range: 0, can_use_with_move: false }, a,
    slot === 'attack' ? {} : { description: `#asset/base/text/champion?description.${id}.${slot}` });
  // a harmless incoming area the size of a danger zone: the game's AI sees an attack landing there and may step out
  const dangerZone = (name, travel, range, radius) => ({ type: 'ParabolicProjectile', name, travel_time: travel, range, range_effect_name: '', shape: { Circle: { radius } },
    applied_target: 'EnemyWithoutTower', applied_effects: [], end_effects: [] });
  const secs = t => +(t / 60).toFixed(2) + 's';
  const stats = (attack, magic_power, hp, defence, magic_resistance, move_speed) => ({ attack, magic_power, hp, defence, magic_resistance, move_speed, hp_regen: 0, stack: 0, crit_chance: 0 });
  const bodyAnim = (modId, name, tag) => ({ anim: `asset/${modId}/champions/${name}`, tag });

  /* ================================================================ V1 (ULTRAKILL)
   * BA        Pistol (ranged).
   * Ability 1 Coin toss: flips a coin and shoots it; the shot splits off the coin into every enemy champion near it,
   *           homing. Damage (one basic attack + the skill's own) is shared out: the more enemies, the less each takes.
   * Ability 2 Parry: a short window in which every hit is parried (no damage, no crowd control) and answered
   *           with a counter-shot. It can't stop Unlimited Void (the domain strips crowd-control immunity).
   * Ult       Railgun: the next 3 basic attacks are railgun shots: very long range (less than Hollow Purple),
   *           piercing through everything in the line.
   * Passive   Blood: every champion he hits bleeds a pool on the ground. Standing on a pool heals a fixed amount,
   *           at most once every 8 s. (native "tfm2_custom_ai:v1")
   */
  function v1(id, o) {
    o = Object.assign({
      pistolRange: 30000, coinCd: 420, coinRange: 60000,
      parryCd: 600, parryTicks: 6,   // S2 arms the parry until it's used; 10 s cooldown
      railCd: 3300, railTicks: 900, railRange: 260000, railSpeed: 14000, railDmg: [35, 65], railRadius: 8000,   // round 25: was 40 + 80% AD   // round 20: was 40 s, 70 + 120% AD, radius 10500
      bloodHeal: 70, bloodEvery: 480,   // native constants, shown in the tooltip
      modId: 'tfm2_ultrakill',
    }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    const pistol = { type: 'TargetProjectile', speed: 6000, name: V('bullet'), y_offset: 0, applied_target: 'Enemy', applied_effects: [onHit(hit(0, 100))] };
    // 1.5x as wide as before (7000); can't hit towers
    const rail = { type: 'LinearProjectile', penetrate: true, speed: o.railSpeed, range: o.railRange, name: V('rail'), shape: { Circle: { radius: o.railRadius } },
      applied_target: 'EnemyWithoutTower', applied_effects: [onHit(hit(o.railDmg[0], o.railDmg[1]))], end_effects: [] };
    const railRange = buff('v1_rail_range', o.railTicks, { range: o.railRange - o.pistolRange });
    const basic = ifBuff('v1_rail3', all(unbuff('v1_rail3'), buff('v1_rail2', o.railTicks), rail),
      ifBuff('v1_rail2', all(unbuff('v1_rail2'), buff('v1_rail1', o.railTicks), rail),
        ifBuff('v1_rail1', all(unbuff('v1_rail1'), unbuff('v1_rail_range'), rail), pistol)));
    const json = {
      id, category: 'Assassin', tags: ['AD', 'Range'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(94, 0, 870, 22, 15, 1100), growth: stats(20, 0, 82, 6, 3, 12),   // round 25: was 108 / +24 attack
      attack: act('attack', { duration: 20, cooltime: 50, start_timing: 10, cancelable: true, range: o.pistolRange, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: basic }),
      skill: act('skill', { duration: 14, cooltime: o.coinCd, start_timing: 6, can_use_with_move: true, range: o.coinRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('v1_coin', 8), markTarget('v1_coin_target', 8)) }),
      // S2 arms the parry ("v1_guard", held until it's used): the native reflex opens the 0.1 s window at the right moment.
      // Long cast range so the AI arms it as soon as it's off cooldown.
      // The parry is up only while S2 is READY (Rian, Oct 1). A cast arms it and puts a 5-tick haste on so that cast's own
      // cooldown is ~0.8 s instead of 10 s (cooldowns are fixed when they start, after the action): S2 stays (nearly)
      // ready while the parry is armed. After a parry, v1_parry_cd is on: the next cast does nothing and gets the full
      // 10 s cooldown, which is exactly the parry's lockout. The haste is too short to touch any other cooldown.
      skill2: act('skill2', { duration: 4, cooltime: o.parryCd, start_timing: 1, can_use_with_move: true, range: 200000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: ifBuff('v1_parry_cd', nothing(),
          all(ifBuff('v1_guard', nothing(), all(cfx(V('parry')), { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: 'v1_guard', duration: 'Permanent' } })),
            buff('v1_parry_haste', 5, { skill_cooldown_mult: 1100 }))) }),
      ult: act('ult', { duration: 20, cooltime: o.railCd, start_timing: 10, range: o.railRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(cfx(V('rail_flash')), unbuff('v1_rail2'), unbuff('v1_rail1'), buff('v1_rail3', o.railTicks), railRange) }),
      passive: { passive_ref: 'tfm2_custom_ai:v1', params: {} },
      ...views(o.modId, 'ultrakill', {
        projectiles: [[V('bullet'), 'bullet', 1], [V('rail'), 'rail', 1], [V('coin_ray'), 'coin_ray', 1], [V('pellet'), 'pellet', 1]],
        effects: [[V('coin'), 'coin', 2], [V('parry'), 'parry', 2], [V('rail_flash'), 'parry', 2], [V('blood'), 'blood', -1], [V('heal'), 'heal', 1, true],
          [V('alert_red'), 'alert_red', 3, true], [V('alert_parry'), 'alert_parry', 3, true], [V('parried_on'), 'parried_on', 1, true],
          [V('shotgun_r'), 'shotgun_r', 2], [V('shotgun_l'), 'shotgun_l', 2], [V('muzzle_r'), 'muzzle_r', 3], [V('muzzle_l'), 'muzzle_l', 3]],
        buffs: [['v1_guard', 'guard', 3], ['v1_rail3', 'rail_on', 1], ['v1_rail2', 'rail_on', 1], ['v1_rail1', 'rail_on', 1]],
      }),
    };
    const text = {
      name: 'V1',
      skill: `Coin toss: flips one coin through the air toward the enemy, landing 60% of the way to them (at most 40000 out). His next basic attack while it's up shoots the coin instead, and the shot ricochets into every enemy champion within 60000 of it: that basic attack plus 30 + 45% AD, shared out between them (a lone target takes it all). If no basic attack gets to it first, he shoots it at its peak by himself (${secs(20)} after the toss, at 60% of a basic attack) as long as an enemy is near it, so a coin is never wasted. With the shotgun loaded the shot is a split shot: the whole load goes into the coin and every enemy near it takes 50% of the ricochet instead of a share. Once the coin drops (after ${secs(50)}) it's gone. Basic attacks have short range (${o.pistolRange}). Cooldown ${secs(o.coinCd)}. (Needs the native rules mod.)`,
      skill2: `Parry: the parry is up only while this ability is ready (a small yellow diamond over his head). Using a parry puts the ability on its full cooldown. While it's up he parries by reflex, with a ${secs(o.parryTicks)} window: right before a dangerous shot lands (Purple, Fern's big Zoltraak, anyone hitting as hard as him, or anything while he's low), and the moment an enemy teleports next to him (Minato's Flying Raijin, DIO's The World and MUDA). Parrying Steve's boat when it rams him breaks Steve's whole wall. A parried hit does nothing (no damage, no crowd control). The parry rings out, shows two yellow "!!" and powers him up for 3s: +10% to +25% attack (the harder the blocked hit, the bigger), half of that as attack speed, +20% move speed; then he goes straight for the attacker, and his next basic attack is a double-pump shotgun: two blasts ${secs(12)} apart (the second at 70% strength), each 6 pellets in a cone, each pellet 22% of a basic attack at point blank, falling off with distance (25% of that at the end of its range). He only presses forward when it's worth it: not after an enemy more than 60000 away, not under an enemy tower (unless the enemy is nearly dead and he's healthy), not into three or more enemies without two teammates by him, and not while he's at 35% HP or less. Only shots from champions are parried, never minions or towers. While a parry is ready he plays forward, pressing the nearest enemy champion. When it isn't ready, a red "!" shows he can't parry a dangerous shot. At most one parry every ${secs(o.parryCd)}. It can't parry Unlimited Void or stopped time, and he can't parry while Railgun is active. Cooldown ${secs(o.parryCd)}. Passive, Blood: every champion he hits leaves a pool of blood for 10s (at most one pool every 12s); standing on one heals ${o.bloodHeal} HP (a fixed amount), at most once every ${secs(o.bloodEvery)}.`,
      ult: `Railgun: his next 3 basic attacks (within ${secs(o.railTicks)}) are railgun shots with ${o.railRange} range: they pierce every champion and minion in the line (not towers) for ${o.railDmg[0]} + ${o.railDmg[1]}% AD. Cooldown ${secs(o.railCd)}.`,
    };
    return { json, text, vfx: 'ultrakill', sprite: 'v1', spriteFallback: 'asset/base/aseprite_resources/champions/android' };
  }

  /* ================================================================ Darth Vader (Star Wars)
   * BA        Lightsaber: a melee swing that cuts through every enemy in a line in front of him.
   * Ability 1 Saber throw: the saber flies out and returns to him, hitting both ways. While it's away he can't attack.
   * Ability 2 Force choke: grips the enemies in a wedge in front of him. The stun is shared: the more enemies caught,
   *           the shorter it is for each. While choking he walks slowly and can't attack, but can throw his saber.
   * Ult       Vader's rage: more damage and faster cooldowns. Only possible at 10 stacks.
   * Passive   Every hit he takes adds a stack (max 10). One big hit is only one stack, so burst is the counter.
   *           (native "tfm2_custom_ai:vader")
   */
  function vader(id, o) {
    o = Object.assign({
      throwCd: 720, throwRange: 112000,   // 12 s, -20% range (was 8 s / 140000); the saber flight (straight, slow, return) is native
      chokeCd: 780, chokeRange: 60000,   /* round 37: was 45000 */ chokeTotal: 120,   // 13 s (was 12 s)
      rageCd: 300, rageTicks: 480, rageDmg: 40, rageCdr: 100,
      modId: 'tfm2_starwars',
    }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1_pre', skill2: 'skill2', ult: 'ult' });   // skill2 = the whole choke (pose 1 s, two saber swings; the choke_seq frames, vader_seq.py). Round 26: the custom 'choke_seq' tag never showed in game, so the standard skill2 tag carries it now
    const json = {
      id, category: 'Melee', tags: ['AD', 'Tank', 'CC', 'Melee'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(90, 0, 1100, 35, 25, 1000), growth: stats(20, 0, 110, 9, 4, 10),
      // the swing is aimed first: a red line on the ground over its hit box (native, from the vader_ba marker), and the
      // cut lands 10 ticks later (tick 12, as before)
      attack: act('attack', { duration: 22, cooltime: 70, start_timing: 2, cancelable: true, range: 26000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack',
        effect: all(buff('vader_ba', 4), markTarget('vader_ba_target', 4),
          delayed(10, cfx(V('swing')), { type: 'RangeEffect', shape: { Rect: { width: 32000, height: 12000 } }, target: 'Enemy', apply_type: { Forward: { offset: 16000 } }, effects: [hit(0, 100)] })) }),
      skill: act('skill', { duration: 16, cooltime: o.throwCd, start_timing: 8, range: o.throwRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('vader_throw', 8), markTarget('vader_saber_target', 8)) }),
      // the wedge, damage, lift and slashes are native (a red V shows the area). The skill's own action lasts the whole
      // sequence (96 ticks: choke pose, then two saber swings timed to the native slashes at 68 and 86), so nothing
      // cuts the pose short (the CasterAnimation version never showed)
      skill2: act('skill2', { duration: 96, cooltime: o.chokeCd, start_timing: 8, range: o.chokeRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('vader_choking', 6), markTarget('vader_choke_aim', 6)) }),
      // castable only at an enemy in crowd control: the native passive holds the nearest enemy for 2 ticks once he
      // has 10 stacks, which is what lets the AI press it (Rage itself needs the 10 stacks, checked here)
      ult: act('ult', { duration: 12, cooltime: o.rageCd, start_timing: 4, range: 60000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        effect: ifBuff('vader_full', all(cfx(V('rage_on')), buff('vader_rage_used', 6),
          delayed(15, buff('vader_rage', o.rageTicks, { attack_mult: o.rageDmg, skill_cooldown_mult: o.rageCdr }))), nothing()) }),
      passive: { passive_ref: 'tfm2_custom_ai:vader', params: {} },
      stack_skill_index: 2,
      ...views(o.modId, 'starwars', {
        projectiles: [[V('saber'), 'saber', 1]],
        effects: [[V('swing'), 'swing', 1, true], [V('choke'), 'choke', 2, true], [V('saber_hit'), 'saber_hit', 1], [V('rage_on'), 'saber_hit', 1, true], [V('saber_spin'), 'saber', 2],
          ...Array.from({ length: 16 }, (_, k) => [V(`choke_v${k}`), `choke_v${k}`, -1]),
          ...Array.from({ length: 8 }, (_, k) => [V(`ba_line${k}`), `ba_line${k}`, -1])],
        buffs: [['vader_rage', 'rage', -1]],
      }),
    };
    const text = {
      name: 'Darth Vader',
      skill: `Saber throw (range ${o.throwRange}): the lightsaber spins out (slower than before) in a straight line (50 + 90% AD to every enemy it cuts), then flies back to him (25 + 45% AD on the way back). He can't basic attack while it's away. (Needs the native rules mod.) Cooldown ${secs(o.throwCd)}. Basic attacks cut through every enemy in a line in front of him; a red line on the ground shows where the cut will land just before it does.`,
      skill2: `Force choke: Vader thrusts his hand out and clenches his fist. Every enemy in a big V-shaped wedge (70000 long, about 90° wide) aimed right at his target (shown in red, following the one he chokes) is lifted and choked for 1s (20 + 40% AD); his target can't step out of its edge (caught up to 12000 past its tip). Cast from up to 60000 away. They hang while he holds the pose, standing still. Then he swings his lightsaber twice: 20 + 40% AD, then 30 + 60% AD and a knockback that throws them away. The whole move takes 1.6s. Cooldown ${secs(o.chokeCd)}.`,
      ult: `Passive: every hit Vader takes adds a stack (max 10); one big hit is only one stack. Vader's rage (only at 10 stacks, which it uses up): +${o.rageDmg}% attack and +${o.rageCdr}% cooldown speed for ${secs(o.rageTicks)}. (Needs the native rules mod.)`,
    };
    return { json, text, vfx: 'starwars', sprite: 'vader', spriteFallback: 'asset/base/aseprite_resources/champions/inquisitor' };
  }

  /* ================================================================ Frieren
   * BA        Magic bolt (ranged).
   * Ability 1 Fern appears beside Frieren (she stays put when Frieren walks on) and after 1.5 s fires a long, piercing
   *           Zoltraak at the target, then fades. (native)
   * Ability 2 Stark leaps onto the target: 1.5 s later he lands; the small circle knocks up, the big circle slows.
   * Ult       Limiter release: she stops suppressing her mana: big magic power, cooldown speed, attack speed and range.
   */
  function frieren(id, o) {
    o = Object.assign({
      boltRange: 70000, boltCd: 140, /* round 25: was 110 */ boltSpeed: 12000, fernCd: 600, fernRange: 150000,
      starkCd: 720, starkRange: 60000, starkDelay: 50, starkSmall: 20000, starkBig: 40000, starkAir: 50, starkSlow: 35, starkSlowTicks: 105,
      limitCd: 3600, limitTicks: 480,
      modId: 'tfm2_frieren',
    }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill', skill2: 'skill2', ult: 'ult' });
    // basic attack: a Zoltraak, one thin white line that pierces everything in its path (slower attack rate)
    const bolt = { type: 'LinearProjectile', penetrate: true, speed: o.boltSpeed, range: o.boltRange + 15000, name: V('zoltraak'), shape: { Circle: { radius: 5000 } },
      applied_target: 'Enemy', applied_effects: [onHit(ap(10, 30))], end_effects: [] };   // round 25: was 15 + 40% AP
    // Stark's jump and landing (knock-up + slow) are done by the native rules
    const json = {
      id, category: 'Magician', tags: ['AP', 'CC', 'Range'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(55, 50, 880, 20, 25, 1000), growth: stats(6, 16, 95, 5, 4, 10),
      attack: act('attack', { duration: 22, cooltime: o.boltCd, start_timing: 12, cancelable: true, range: o.boltRange, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: bolt }),
      skill: act('skill', { duration: 16, cooltime: o.fernCd, start_timing: 6, can_use_with_move: true, range: o.fernRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('frieren_fern', 8), markTarget('fern_target', 8)) }),
      skill2: act('skill2', { duration: 16, cooltime: o.starkCd, start_timing: 6, range: o.starkRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        // the jump, landing spot and landing hit are native (the spot is where the target stood when he jumped)
        effect: all(buff('frieren_stark', 8), markTarget('stark_target', 8)) }),
      ult: act('ult', { duration: 24, cooltime: o.limitCd, start_timing: 12, range: 90000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(cfx(V('limit_on')), delayed(15, buff('frieren_limit', o.limitTicks, { magic_power_mult: 25, attack_speed_mult: 15, range: 15000 }),
          buff('frieren_fern_side', o.limitTicks))) }),
      ...views(o.modId, 'frieren', {
        projectiles: [[V('zoltraak'), 'zoltraak', 2], [V('zoltraak_fern'), 'zoltraak_fern', 2], [V('zoltraak_big'), 'zoltraak_big', 2]],
        effects: [[V('stark_land'), 'stark_land', 1], [V('stark_mark'), 'stark_mark', -1, true], [V('stark_up'), 'stark_up', 2], [V('stark_down'), 'stark_down', 2],
          [V('fern_circle'), 'fern_circle', -1], [V('fern_fade'), 'fern_fade', 2], [V('limit_on'), 'limiter', 1, true], [V('fern_s1'), 'fern_s1', 1]],
        buffs: [['frieren_limit', 'limiter', 1], ['frieren_fern_side', 'fern_side', 1]],
      }),
    };
    // Fern and Stark are recoloured base bodies kept in this folder (champions/fern, champions/stark)
    json.view_effects.push({ type: 'Animation', name: V('fern_idle'), ...bodyAnim(o.modId, 'fern', 'idle'), z: 1, is_follow: false },
      { type: 'Animation', name: V('fern_cast'), ...bodyAnim(o.modId, 'fern', 'skill1'), z: 1, is_follow: false },
      { type: 'Animation', name: V('stark_idle'), ...bodyAnim(o.modId, 'stark', 'idle'), z: 1, is_follow: false });
    // Stark on foot in Limiter release: the native rules draw him one pose at a time (idle 4 / run 8 / attack 5 frames,
    // facing right or left). A spawned unit crashed the game, so he isn't a real unit.
    for (const [k, n] of [['i', 4], ['r', 8], ['a', 5]]) for (const side of ['r', 'l']) for (let i = 0; i < n; i++)
      json.view_effects.push({ type: 'Animation', name: V(`stark_${k}${side}${i}`), anim: `asset/${o.modId}/vfx/frieren`, tag: `stark_${k}${side}${i}`, z: 1, is_follow: false });
    const text = {
      name: 'Frieren',
      skill: `Basic attack: Zoltraak, a thin white line that pierces everything in its path (10 + 30% AP), every ${secs(o.boltCd)}. Fern appears beside Frieren and stays there. After 1.5s she fires her own Zoltraak at the target (50 + 70% of Frieren's AP), piercing everything in its line, hitting each enemy once (towers take no damage), then she fades: at a close target a slow, huge beam; at a far one a long, thin beam. Cooldown ${secs(o.fernCd)}. (Needs the native rules mod.)`,
      skill2: `Stark leaps onto the spot where the target stands (a big red circle marks it) and lands there ${secs(o.starkDelay)} later: the inner circle (${o.starkSmall}) knocks enemies up for ${secs(o.starkAir)} (55 + 80% AP), the outer circle (${o.starkBig}) slows them by ${o.starkSlow}% for ${secs(o.starkSlowTicks)}. Cooldown ${secs(o.starkCd)}.`,
      ult: `Limiter release: Frieren stops suppressing her mana for ${secs(o.limitTicks)}: +25% magic power, +15% attack speed and +15000 attack range. Fern joins her at her side for the whole time, firing a huge homing Zoltraak every 3s (30 + 45% AP, towers take no damage). Stark joins as a companion for 12s, like the Druid's bear: enemies can target him, and he goes down after 4 hits (or when his 300 HP + a quarter of Frieren's max HP run out). He fights on his own, swinging his axe every 1.5s (30 + 35% of Frieren's AP), and leaps once onto every enemy that comes within his reach (a new enemy at most every 2s, never the same one twice; 35 + 50% AP and a knock-up where he lands). While Fern or Stark from Limiter release is out, Frieren can't cast her own Fern or Stark (basic attacks only). Cooldown ${secs(o.limitCd)}.`,
    };
    return { json, text, vfx: 'frieren', sprite: 'frieren', spriteFallback: 'asset/base/aseprite_resources/champions/white_mage', extraSprites: { fern: 'fern', stark: 'stark', stark_ghoul: 'stark_ghoul' } };
  }

  /* ================================================================ DIO (JoJo) — Stand Out / Stand In
   * Two modes, switched by the native rules (target in melee reach → Stand In; far → Stand Out; at most every 5 s,
   * also in stopped time). Each slot checks "dio_out" first.
   * Stand Out (no lifesteal, less damage): BA = the Stand's punch thrown at range; S1 arms the Stand's guard (blocks
   *   the next champion shot at 75% damage and no CC, or counters the next melee hit); S2 = Stand lunge (1 s wind-up,
   *   through walls; grab, punches, slow 0.7 s, back to DIO).
   * Stand In (lifesteal): melee BA; S1 knife fan; S2 dash strike (the Stand bursts out, DIO vanishes and reappears
   *   behind the target).
   * Ult (both): 2 s time stop in a black-and-white field the size of Unlimited Void, the Stand at its centre; his
   *   damage is lower meanwhile; he can still switch modes. (native: tfm2_custom_ai:dio + the match hook)
   */
  function dio(id, o) {
    o = Object.assign({ skillCd: 600, skill2Cd: 660, knifeRange: 160000, lungeRange: 80000, dashRange: 90000, outBa: 60, outAs: 45,
      tsCd: 5400, tsTicks: 120, tsRadius: 76000, tsCut: 40, inVamp: 12, modId: 'tfm2_jojo' }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    const OUT = 'dio_out';
    const guard = all(cfx(V('stand_swap')), { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: 'dio_guard', duration: 'Permanent' } });
    const knives = all(ifBuff('dio_timestop', buff('dio_knives4', 6), buff('dio_knives3', 6)), markTarget('dio_target', 6));
    // cuts THIS cast's own cooldown to ~1 s (the slot cooldown is fixed when the action ends; 10 ticks covers it and
    // is too short to touch any other cooldown)
    const haste = buff('dio_haste', 10, { skill_cooldown_mult: 1100 });
    // Stand Out basic attack: the Stand rushes to the target and punches it there (drawn by the native rules). The hit
    // itself is an invisible projectile (no visual bound to 'stand_hit'), slow enough to land about when the fist does.
    const standFist = { type: 'TargetProjectile', speed: 4500, name: V('stand_hit'), y_offset: 0, applied_target: 'Enemy', applied_effects: [onHit(hit(0, o.outBa))] };
    const json = {
      id, category: 'Assassin', tags: ['AD', 'CC', 'Melee'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(100, 0, 980, 28, 20, 1080), growth: stats(22, 0, 92, 7, 4, 12),
      // melee range; Stand Out adds +32000 range through its mode buff
      attack: act('attack', { duration: 20, cooltime: 60, start_timing: 12, cancelable: true, range: 23000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack',
        // Stand In: a melee hit with a golden swipe on the target (his own attack animation no longer carries the blast)
        effect: ifBuff(OUT, standFist, all(hit(0, 100), fx(V('strike')))) }),
      // Each mode keeps its own cooldown (native: dio_r* = ready, dio_c* = used). A cast uses this mode's version; if
      // that's on cooldown it switches stance (dio_sw_*) and uses the other mode's. While the other version is still
      // ready the slot's real cooldown is cut to ~1 s (haste), so the combo flows: S2 dash, S1 knives, S2 (change +
      // lunge), S1 (guard).
      skill: act('skill', { duration: 12, cooltime: o.skillCd, start_timing: 5, can_use_with_move: true, range: o.knifeRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: ifBuff(OUT,
          ifBuff('dio_r1out', all(guard, buff('dio_c1out', 6), ifBuff('dio_r1in', haste)),
            ifBuff('dio_r1in', all(buff('dio_sw_in', 6), knives, buff('dio_c1in', 6)))),
          ifBuff('dio_r1in', all(knives, buff('dio_c1in', 6), ifBuff('dio_r1out', haste)),
            ifBuff('dio_r1out', all(buff('dio_sw_out', 6), guard, buff('dio_c1out', 6))))) }),
      skill2: act('skill2', { duration: 12, cooltime: o.skill2Cd, start_timing: 3, range: o.lungeRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(markTarget('dio_target', 6), ifBuff(OUT,
          ifBuff('dio_r2out', all(buff('dio_lunge', 6), buff('dio_c2out', 6), ifBuff('dio_r2in', haste)),
            ifBuff('dio_r2in', all(buff('dio_sw_in', 6), buff('dio_dash', 6), buff('dio_c2in', 6)))),
          ifBuff('dio_r2in', all(buff('dio_dash', 6), buff('dio_c2in', 6), ifBuff('dio_r2out', haste)),
            ifBuff('dio_r2out', all(buff('dio_sw_out', 6), buff('dio_lunge', 6), buff('dio_c2out', 6)))))) }),
      // round 33: the AI only presses it at an enemy in CC (the native rules hold the key enemy 2 ticks when a time stop
      // is worth it). Round 38: every cast is a real time stop (the empty "no setup" casts looked like misses: the pose
      // with no time stop); range 110000 so the enemy in CC is always within his blink + the field
      ult: act('ult', { duration: 10, cooltime: o.tsCd, start_timing: 2, range: 110000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        effect: all(cfx(V('za_warudo')), buff('dio_timestop', o.tsTicks), buff('dio_ts_new', 6), buff('dio_ts_cut', o.tsTicks, { attack_mult: -o.tsCut })) }),
      passive: { passive_ref: 'tfm2_custom_ai:dio', params: {} },
      ...views(o.modId, 'jojo', {
        projectiles: [[V('knife'), 'knife', 1]],
        effects: [[V('strike'), 'strike', 2], [V('mark'), 'mark', 3, true], [V('muda'), 'muda', 2], [V('za_warudo'), 'za_warudo', 3], [V('knife_hang'), 'knife_hang', 2],
          [V('timestop_field'), 'timestop', -1], [V('stand_world_ult'), 'stand_world_ult', 2], [V('stand_pop'), 'stand_pop', 2],
          [V('stand_block'), 'stand_block', 3], [V('stand_swap'), 'stand_swap', 2],
          // the lunge: gold gathering at the Stand's fist during the wind-up, fading afterimages along the dash
          [V('stand_charge'), 'stand_charge', 2], [V('stand_ghostr'), 'stand_ghostr', -1], [V('stand_ghostl'), 'stand_ghostl', -1],
          // the lunge's ground area charging up during the wind-up (8 directions x 6 fill stages)
          ...Array.from({ length: 48 }, (_, i) => [V(`lunge_area${i % 8}_${Math.floor(i / 8)}`), `lunge_area${i % 8}_${Math.floor(i / 8)}`, -1]), [V('vanish'), 'vanish', 2], [V('appear'), 'appear', 2],
          // the Stand's single poses, drawn one at a time by the native rules (r = facing right, l = left)
          ...[['idle', 4], ['punch', 4], ['bar', 4], ['world', 4], ['lunge', 1], ['grab', 1]].flatMap(([p, n]) =>
            ['r', 'l'].flatMap(side => Array.from({ length: n }, (_, i) => [V(`stand_${p}${side}${i}`), `stand_${p}${side}${i}`, -1])))],
      }),
    };
    const text = {
      name: 'DIO',
      skill: `Two modes, switched by themselves (at most every 5s, even in stopped time): Stand Out when his target is out of reach (38000+), Stand In when it's in melee range. Stand Out: his Stand floats behind him; his basic attacks (range 41000) are the Stand rushing to the target and punching it there, then hovering by it: +${o.outAs}% attack speed, ${o.outBa}% AD per punch, no lifesteal (not even from items). Stand In: melee basic attacks with ${o.inVamp}% lifesteal. Against towers he always switches to Stand In (the Stand doesn't siege from range). Ability 1 — Stand Out: the Stand stands guard until it's needed; the next shot from a champion that would hit him is blocked (he takes 75% of it, no crowd control), or the next melee hit is countered (the Stand appears in front of him, stuns the attacker 0.75s and pummels it, 6 × (6 + 12% AD)). Stand In: knife fan, three knives that fly straight for ${o.knifeRange} (30 + 60% AD each); in stopped time, four knives that hang in the air until time moves again. Each mode has its own cooldown (${secs(o.skillCd)}): if this mode's version is on cooldown, casting it switches stance and uses the other mode's version, so a full combo is S2 dash, S1 knives, S2 (change + lunge), S1 (guard). (Needs the native rules mod.)`,
      skill2: `Stand Out: Stand lunge. The Stand winds up for 1s (gold gathering at its fist, and a gold strip on the ground filling up toward the target shows where it will go), then lunges (through walls), leaving a golden streak; a shot blocked by the guard or a melee counter no longer interrupts it. If it reaches an enemy champion it grabs it by the neck and pummels it for 0.7s (5 × (10 + 14% AD), 40% slow), then floats back to him. Stand In: dash strike. A red "!" stays over the target and a gold strip on the ground fills up from DIO toward it; 0.6s later the Stand bursts out, time skips, and DIO vanishes and reappears behind the target, striking it (60 + 100% AD, stun 0.6s). Each mode has its own cooldown (${secs(o.skill2Cd)}): if this mode's version is on cooldown, casting it switches stance and uses the other mode's version instead.`,
      ult: `ZA WARUDO: he only stops time when it catches enough: 2+ enemies, or one with a teammate close enough to join in or at 40% HP or less (never a lone dive under an enemy tower); and whenever an enemy within reach is caught by crowd control, he chains the time stop onto them. He blinks in first toward the enemies (at most his Stand Out basic attack range, 41000, stopping just short of them), then time stops for ${secs(o.tsTicks)} in a black-and-white field the size of Unlimited Void (radius ${o.tsRadius}), with his Stand at its centre. Every other champion inside is frozen except one: the teammate closest to DIO moves in stopped time with him (immune to crowd control). DIO and that teammate get +100% cooldown speed and +20% move and attack speed for the whole time stop, and both deal ${o.tsCut}% less damage while time is stopped. He can switch modes inside it. Cooldown ${secs(o.tsCd)}. (Needs the native rules mod.)`,
    };
    return { json, text, vfx: 'jojo', sprite: 'dio', spriteFallback: 'asset/base/aseprite_resources/champions/vampire' };
  }

  /* ================================================================ David Martinez (Cyberpunk)
   * Cyberpsychosis (0-100, native): rises with Sandevistan use (+6/s), ability 2 (+10) and the ult (+40); drains 4/s
   *           after 2 s without Sandevistan. 70+: hallucinations: a basic attack can hit a teammate or himself.
   *           100: Sandevistan stays on for good, the bar is stuck, he loses 7% max HP per second until he dies (or is killed), and
   *           his attacks spill onto teammates at reduced damage. The higher the bar, the slower his cooldowns start
   *           (he gets careful). Death resets it.
   * Ability 1 Sandevistan: a fuel bar instead of a real cooldown. While on: big move + attack speed, time slows for
   *           enemies around him. Drains 20/s; refills 8/s (2× the cyberpsychosis drain) when off.
   * Ability 2 Changes with cyberpsychosis: 0-34 pistol barrage (homing, low damage, long range); 35-69 cyber dash
   *           slam + stun + a little gravity pull; 70+ Sandevistan rush grab + explosion that also hurts him (+15).
   * Ult       Gravity projection: a red crosshair on the ground; 1 s later gravity ×100 there for 3 s: everyone
   *           inside (both teams) is pinned and can't act. Only David moves freely. (native)
   */
  function david(id, o) {
    o = Object.assign({ meleeRange: 23000,
      // Sandevistan is pressed in short pulses: each press keeps it on for 1 s and the skill is ready again 0.75 s later,
      // so the AI simply keeps pressing it while he fights and it stays on for as long as there's fuel. A press with
      // an empty tank costs almost nothing (0.75 s), so it comes back on as soon as the fuel does.
      sandeCd: 45, sandeTicks: 60, s2Cd: 480, s2Range: 90000, gravCd: 3600, gravRange: 80000, modId: 'tfm2_cyberpunk' }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    const shot = (d, r) => ({ type: 'TargetProjectile', speed: 6500, name: V('bullet'), y_offset: 0, applied_target: 'Enemy', applied_effects: [onHit(hit(d, r))] });
    const barrage = all(buff('dv_s2', 6), ...[0, 5, 10, 15, 20, 25].map(t => delayed(t, shot(8, 18))));
    // 35-69 cyber dash: he runs in faster than normal (not Sandevistan fast) and slams, with a little gravity well that
    // pulls the enemies around him in (native, dv_minigrav)
    const slam = all(buff('dv_s2', 6), { type: 'MoveToTarget', speed: 12000, range: o.s2Range, end_effects: [
      buff('dv_minigrav', 6),
      { type: 'RangeEffect', shape: { Circle: { radius: 16000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: [hit(50, 95), { type: 'Stun', duration: 60 }] }, cfx(V('slam'))] });
    // 70+ (and in full cyberpsychosis): the grab behind a Sandevistan rush (much faster dash, afterimages);
    // +15 cyberpsychosis instead of +10, and the explosion hurts him as usual
    const rushGrab = all(buff('dv_s2r', 6), buff('dv_rush', 24), { type: 'MoveToTarget', speed: 16000, range: o.s2Range, end_effects: [
      { type: 'Stun', duration: 75 }, hit(40, 70),
      delayed(40, cfx(V('boom')), buff('dv_selfboom', 6),
        { type: 'RangeEffect', shape: { Circle: { radius: 18000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: [hit(60, 90)] })] });
    const json = {
      id, category: 'Assassin', tags: ['AD', 'Melee', 'CC'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(95, 0, 950, 25, 18, 1100), growth: stats(22, 0, 90, 7, 3, 12),
      attack: act('attack', { duration: 20, cooltime: 55, start_timing: 10, cancelable: true, range: o.meleeRange, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: hit(0, 100) }),
      skill: act('skill', { duration: 8, cooltime: o.sandeCd, start_timing: 2, can_use_with_move: true, range: 60000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: ifBuff('dv_nofuel', nothing(), buff('dv_sande', o.sandeTicks, { move_speed_mult: 40, attack_speed_mult: 50 })) }),
      skill2: act('skill2', { duration: 18, cooltime: o.s2Cd, start_timing: 6, range: o.s2Range, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: ifBuff('dv_t2', rushGrab, ifBuff('dv_t1', slam, barrage)) }),
      ult: act('ult', { duration: 20, cooltime: o.gravCd, start_timing: 8, range: o.gravRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('dv_grav', 6), markTarget('dv_grav_target', 6), dangerZone(V('grav_warn'), 60, o.gravRange + 20000, 30000)) }),
      passive: { passive_ref: 'tfm2_custom_ai:david', params: {} },
      ...views(o.modId, 'cyberpunk', {
        projectiles: [[V('bullet'), 'bullet', 1]],
        effects: [[V('slam'), 'slam', 1], [V('grav_mini'), 'grav_mini', 1], [V('boom'), 'boom', 2], [V('crosshair'), 'crosshair', -1], [V('crosshair_lock'), 'crosshair', -1, true], [V('gravity'), 'gravity', -1], [V('crosshair_area'), 'crosshair_area', -1], [V('gravity_area'), 'gravity_area', -1],
          ...['g', 'm', 'c'].flatMap(k => ['r', 'l'].map(d => [V(`after_${k}_${d}`), `after_${k}_${d}`, 0]))],
        // the cyberpsychosis bar (dv_bar0..10) and the current ability-2 icon (dv_t0..3), kept by the native rules
        buffs: [['dv_psycho', 'psycho', 1],
          ...Array.from({ length: 11 }, (_, i) => [`dv_bar${i}`, `bar_${i}`, 3]),
          ['dv_t0', 'ico_pistol', 3], ['dv_t1', 'ico_slam', 3], ['dv_t2', 'ico_rush', 3],
          // Sandevistan v2 / v3: red / blue fire on the bar
          ['dv_sv2', 'fire_red', 3], ['dv_sv3', 'fire_blue', 3],
          // the Sandevistan fuel bar under it (dv_fuel0..10; dv_fueldry while it's recharging after running dry)
          ...Array.from({ length: 11 }, (_, i) => [`dv_fuel${i}`, `fuel_${i}`, 3]), ['dv_fueldry', 'fuel_dry', 3]],
      }),
    };
    const text = {
      name: 'David',
      skill: `Melee. Sandevistan: runs on a fuel bar (the thin bar under his cyberpsychosis bar) instead of a long cooldown: he keeps it on while he fights, as long as there's fuel, and it leaves a trail of coloured afterimages. While on: +40% move speed, +50% attack speed, and time slows for enemies around him (-35% move and attack speed). From 80 cyberpsychosis it runs as Sandevistan v2 (red fire on his bar): another +25% move speed, +35% attack speed, +10% attack and 8% lifesteal. In full cyberpsychosis it's v3 (blue fire): another +50% move speed, +60% attack speed, +20% attack, 15% lifesteal and 15% less damage taken. Sandevistan is too fast for Gojo's Infinity: while it's on, his basic attacks ignore Infinity's 30% cut, and his 3rd hit on Gojo within 3s shatters it (down for its usual 15s). Drains 22 fuel/s; refills 8/s while off; after running dry it's usable again from 15 fuel. When he's low and running from someone, he triggers it himself to escape, if there's fuel and his cyberpsychosis is still below 60. Cyberpsychosis (0-100, the bar above his head; the icon beside it shows what ability 2 does right now): +10/s while Sandevistan is on, +10 per ability 2, +25 or +40 per ult; drains 4/s after 2.5s without Sandevistan. From 70: hallucinations, his basic attacks can hit a teammate or himself. At 100: Sandevistan stays on for good, the bar is stuck, he loses 7% of his max HP per second until he dies, charges the nearest enemy (no retreat, no recall), and his attacks spill onto teammates. The higher the bar, the slower his cooldowns come back. Death resets it. (Needs the native rules mod.)`,
      skill2: `Changes with cyberpsychosis (+10 per use, +15 for the rush grab). 0-34: pistol barrage, 6 homing shots at long range (8 + 18% AD each). 35-69: cyber dash, he runs in faster than normal (not Sandevistan fast) and slams (50 + 95% AD, stun 1s around him); a little gravity well pulls the enemies around him in. 70+ (also in full cyberpsychosis): Sandevistan rush grab, he rushes in at Sandevistan speed leaving afterimages, grabs (stun 1.25s, 40 + 70% AD), then explodes (60 + 90% AD around him, and it hurts David too); +15 cyberpsychosis instead of +10. Cooldown ${secs(o.s2Cd)}.`,
      ult: `Gravity projection, two ways. With other enemies near the target he marks an area (+25 cyberpsychosis): a bigger crosshair (radius 40000), and 0.6s later gravity ×100 smashes down once, pinning every champion inside at that moment, enemies and teammates, for 1.5s. On a lone target worth it (low HP or hits hard) he locks on (+40): the crosshair (radius 30000) follows them, and 1s later the smash pins whoever is inside for 2.5s. Pinned champions can't act (only David moves freely). The smash also shatters Gojo's Infinity (not inside his own open domain). The field doesn't linger: if it catches nobody, it's wasted. +40 cyberpsychosis. Cooldown ${secs(o.gravCd)}.`,
    };
    return { json, text, vfx: 'cyberpunk', sprite: 'david', spriteFallback: 'asset/base/aseprite_resources/champions/hitman' };
  }


  /* ================================================================ Steve (Blockcraft): blocky tank support
   * BA   pickaxe, melee, normal damage (no tower bonus).
   * S1   three tools in turn: pearl (throw, teleport where it lands; drags a hooked enemy along) → TNT (thrown, red
   *      fuse ring, blast + knockback) → golden apple (heal + shield on the hooked ally or the ally lowest on HP).
   * S2   fishing rod: hooks an enemy (pull), an ally (yank to safety), a wall (grapple) or his own TNT (fling it).
   * Ult  boat: rides along an ice trail; a stone wall two blocks high rises behind him and blocks everyone for 5 s.
   * Everything is native ("tfm2_custom_ai:steve" + the match hook + the wall-detour input AI); the data sets markers.
   */
  function steve(id, o) {
    o = Object.assign({ s1Cd: 360, rodCd: 480, rodRange: 154000, rodMin: 70000, boatCd: 4800, modId: 'tfm2_blockcraft' }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    const json = {
      id, category: 'Util', tags: ['Tank', 'CC', 'Shield', 'Melee'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(62, 0, 1150, 40, 35, 1000), growth: stats(11, 0, 120, 10, 5, 10),
      attack: act('attack', { duration: 22, cooltime: 65, start_timing: 12, cancelable: true, range: 23000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: hit(0, 100) }),
      skill: act('skill', { duration: 14, cooltime: o.s1Cd, start_timing: 6, can_use_with_move: true, range: 70000, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('stv_s1', 6), markTarget('stv_target', 6)) }),
      skill2: act('skill2', { duration: 16, cooltime: o.rodCd, start_timing: 6, range: o.rodRange, casting_type: 'Targeting', casting_target: 'EnemyChampion', attack_type: 'Skill',
        effect: all(buff('stv_rod', 6), markTarget('stv_rod_target', 6)) }),
      // The AI only presses it at an enemy in CC: the native rules hold the key enemy for 2 ticks when there's a fight or
      // an objective worth walling in (so the boat isn't wasted on a lone laner).
      ult: act('ult', { duration: 10, cooltime: o.boatCd, start_timing: 4, range: 480000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        // round 30: only when the native rules say a setup is worth it (stv_ult_ok); a cast without it (another CC made
        // the AI press it) does nothing and its cooldown is cut to ~1.7 s (the ult_cooldown_mult haste, as for V1's S2)
        effect: ifBuff('stv_ult_ok', all(cfx(V('splash')), buff('stv_boat', 6), markTarget('stv_ult_target', 6)),
          buff('stv_ult_wait', 12, { ult_cooldown_mult: 4700 })) }),
      passive: { passive_ref: 'tfm2_custom_ai:steve', params: {} },
      ...views(o.modId, 'blockcraft', {
        projectiles: [[V('pearl'), 'pearl', 1], [V('tnt_fly'), 'tnt_fly', 1], [V('apple'), 'apple', 1]],
        effects: [[V('tnt'), 'tnt', 0], [V('tnt_idle'), 'tnt_idle', 0], [V('tnt_ring'), 'tnt_ring', -1], [V('boom'), 'boom', 2], [V('hook'), 'hook', 2], [V('line_dot'), 'line_dot', 1],
          [V('hooked'), 'hooked', 3, true], [V('apple_glow'), 'apple_glow', 2, true], [V('warp'), 'warp', 2], [V('splash'), 'splash', 2],
          [V('boat_r'), 'boat_r', 0], [V('boat_l'), 'boat_l', 0], [V('ice'), 'ice', -1], [V('wall_block'), 'wall_block', 0], [V('wall_rise'), 'wall_rise', 1]],
        buffs: [['stv_tool0', 'tool_0', 3], ['stv_tool1', 'tool_1', 3], ['stv_tool2', 'tool_2', 3],
          // the rod's charge bar
          ...[0, 1, 2, 3, 4, 5].map(k => [`stv_ch${k}`, `charge_${k}`, 3])],
      }),
    };
    const text = {
      name: 'Steve',
      skill: `Three tools; he picks the one the moment needs (the icon over his head shows the one he'd use right now). He's a support first: he never pearls alone into the enemies. The pearl if he has an enemy on his fishing line and his side is outnumbered (otherwise TNT, dropped in the pull's path), a fight behind one of his walls needs him, a teammate is fighting farther away (he lands beside them, on the side away from the enemies), he's low and outnumbered (away from the enemies, toward his team), or the target is far but his team is already there; the golden apple if an ally he just hooked or an ally on low HP with an enemy close by needs saving, or he's low but not outnumbered; otherwise TNT (the target is close or the enemies are bunched up). Pearl: throws it up to 70000 and teleports where it lands; if he has an enemy on his fishing line, he throws it toward his team instead and drags the enemy along. He also walks with his team: a move that would take him alone among two or more enemies becomes a move to a teammate who's fighting. TNT: thrown where the target will be when it lands (he leads moving targets). If an enemy is in its reach as it lands it's lit: a red ring for 0.5s, then it blows up (40 + 6% of his max HP, radius 32000) and knocks enemies away from it. If nobody is, it lies as a mine for up to 5s and goes off (0.2s) when an enemy steps within 20000; a basic attack from Steve or a teammate right next to it sets it off too. He plays it with his rod: with an enemy on his line he drops the TNT where they're being reeled to, so the pull drags them into the blast; he hooks an enemy whose pull would drag them over his TNT; and a TNT that would miss gets yanked by the rod and thrown like a bomb at an enemy, blowing up where it lands. When his side can take the fight it lands behind the target, so the blast throws them toward him and his team; when he's low or outnumbered it lands in front, so the blast throws them away. Golden apple: heals 60 + 8% of his max HP and shields for 10% of his max HP (4s), on the ally he just hooked with his rod, otherwise the ally lowest on HP within 60000 (him included). Cooldown ${secs(o.s1Cd)}. Basic attack: a pickaxe swing (normal damage). (Needs the native rules mod.)`,
      skill2: `Fishing rod: charged. He casts at once to ${o.rodMin} and charges for up to 1s for up to ${o.rodRange}, walking 40% slower while a bar over his head fills; he lets go as soon as the charge reaches what he's aiming at (a little past it), so a close hook flies at once and only a far one charges fully. The hook is big (it catches anything within 16000 of it) but slow, and it doesn't go through walls: if it reaches terrain or a boat wall first, it bites there and Steve dashes to it. Otherwise it catches the first thing it reaches and reels it in fast. He knows before he casts which it will be: a clear line pulls, a wall in the way means a dash to that wall. He only takes the dash when it's a good move (toward a teammate in trouble, away from enemies when he's low, or into a fight his side can take), otherwise he picks a target he can pull, or keeps the rod. A teammate in trouble comes first. An enemy: 20 + 2% of his max HP and reeled in to him (a pearl during the pull drags them on to where he lands). An ally in trouble: yanked to him, and his next golden apple goes to them. A wall (he aims at one when he's low and needs to get away): he dashes to it. His own TNT not yet blown near a fight: yanked and thrown like a bomb at the nearest enemy, blowing up as it lands. Cooldown ${secs(o.rodCd)}.`,
      ult: `Boat (range 480000): one straight wall that can't bend, so where it goes is everything. He waits for a setup worth it (up to 340000 away). It's also his way out: when he's chased (at half HP or less, or outnumbered) or a teammate at 40% or less is, the wall goes up between the chasers and them, close to the chasers, and he jumps off on the safe side; this comes before any fight. He uses it often: even a small fight can turn on it (cutting a laner off from its support, cutting a laner's way home for a gank, a wall his team can pass or rotate behind, or simply an engage his team plays around). Objective plans are worth the most, so he takes one whenever it's on offer. And a pick: when a teammate catches an enemy with crowd control and is on it, he walls it off from whoever would come to help (always worth it). He calls his team in: when going in is a winning situation (as many of his team able to come as enemies there, at least two), a ping marks where the wall is meant to be used (the trapped enemies, the fight, the objective, the caught enemy) and his teammates head there, for as long as the wall stands and it stays a winning situation. Then he hops into a boat and slides slowly along the line on a trail of ice while a dark stone wall two big blocks high and thick, always its full 300000 long (where terrain cuts one side short it slides along its line into the other; only a passage narrower than that gets a wall from side to side, which seals it and which he likes best), rises behind the boat. The boat rams every enemy in its way: 30 + 4% of his max HP and a knock out to the side they're on (right on the line: to the far side from his team), clear of the wall. V1 can parry the boat: that breaks the whole wall. Where he puts it: behind an enemy group his side can take, across its way home (also the way the enemies he can't see would come from); between his team's fight and the enemies on their way to it; between a big neutral monster his team is taking and where the enemies would come from; behind enemies taking one when his team is close enough to punish; or between the teams when his side is losing. Fog of war: he only counts enemies his team can see. He can jump off at any point and the empty boat still builds the wall to its full length; he jumps off (behind the trapped group, on his team's side of the fight, by the monster, or straight to a teammate about to die; when he's low, straight to his team's side). The wall blocks everyone, both teams, for 10s: walking, knockbacks and pulls can't get through, and the players walk around its ends. It also blocks sight like terrain: anyone whose every enemy nearby is on the other side of it is hidden from them. Only teleports and blinks go over it, like a blink over terrain (dashes, knockbacks and pulls are stopped; only his own fishing line pulls through), and champions who have one use it when the wall cuts them off from a teammate in trouble, a tower or an objective under attack that their team can see (Minato's kunai, DIO's dash strike, Steve's pearl). Shots still fly over it. Allies on the ice trail get +25% move speed. He can't be crowd-controlled while riding or jumping. Cooldown ${secs(o.boatCd)}.`,
    };
    return { json, text, vfx: 'blockcraft', sprite: 'steve', spriteFallback: 'asset/base/aseprite_resources/champions/fighter' };
  }


  /* ================================================================ Omen (Valorant): shadow controller
   * Passive  Buy Phase (shared by the Valorant folder): credits, guns and armor instead of items (native).
   * BA       the gun he bought: Classic / Sheriff / Spectre / Judge (shotgun) / Vandal / Operator (native stat buffs).
   * S1       Dark Cover: a shadow dome anywhere on the map, 10 s, 100000 across (radius 50000). 2 charges: cooldown + bought.
   * S2       Shadow Step (teleport, 50000) or Paranoia (a slow shadow through walls that blinds everyone it touches).
   * Ult      From the Shadows: 2 s channel, then a teleport anywhere; killing his shade cancels it.
   * Everything is native ("tfm2_custom_ai:omen" + the match hook's smoke vision + the input AI's smoke checks); the
   * data sets markers. Each slot only does something when the native rules found a use (omn_*_ok), otherwise the
   * cast is empty and its cooldown comes back fast.
   */
  function omen(id, o) {
    o = Object.assign({ smokeCd: 1500, s2Cd: 420, ultCd: 4800, modId: 'tfm2_valorant' }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    // cuts THIS cast's own cooldown to 1/12 (the slot cooldown is fixed when the action ends; 10 ticks covers it)
    const haste = n => buff(n, 10, { skill_cooldown_mult: 1100 });
    const json = {
      id, category: 'Util', tags: ['AD', 'CC', 'Range'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(70, 0, 900, 26, 22, 1020), growth: stats(14, 0, 95, 7, 4, 12),
      // the shotgun's reach; every other gun adds range through its buff (Classic 30000 ... Operator 80000)
      attack: act('attack', { duration: 18, cooltime: 60, start_timing: 8, cancelable: true, range: 18000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: hit(0, 100) }),
      // The AI presses these at an enemy in CC: the native rules hold the nearest enemy for 2 ticks when a slot has a use.
      skill: act('skill', { duration: 12, cooltime: o.smokeCd, start_timing: 5, can_use_with_move: true, range: 480000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        // the bought charge (omn_more) brings the slot straight back for the second smoke
        effect: ifBuff('omn_s1_ok', all(cfx(V('para_cast')), buff('omn_s1', 6), ifBuff('omn_more', haste('omn_haste'))), haste('omn_wait1')) }),
      skill2: act('skill2', { duration: 14, cooltime: o.s2Cd, start_timing: 5, can_use_with_move: true, range: 190000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        effect: ifBuff('omn_s2_ok', buff('omn_s2', 6), haste('omn_wait2')) }),
      ult: act('ult', { duration: 12, cooltime: o.ultCd, start_timing: 4, range: 480000, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill',
        effect: ifBuff('omn_ult_ok', all(cfx(V('ult_cast')), buff('omn_ult', 6)), buff('omn_ult_wait', 12, { ult_cooldown_mult: 4700 })) }),
      passive: { passive_ref: 'tfm2_custom_ai:omen', params: {} },
      ...views(o.modId, 'valorant', {
        projectiles: [[V('paranoia'), 'paranoia', 2], [V('smoke_orb'), 'smoke_orb', 1], [V('tracer'), 'tracer', 1], [V('tracer_op'), 'tracer_op', 1], [V('pellets'), 'pellets', 1]],
        // the dome is drawn over the units in it (z 3): you can't see in
        effects: [[V('smoke_dome'), 'smoke_dome', 3], [V('smoke_fade'), 'smoke_fade', 3], [V('smoke_form'), 'smoke_form', 3], [V('para_cast'), 'para_cast', 2],
          [V('op_flash'), 'op_flash', 2], [V('decoy'), 'decoy', -1, true], [V('step_in'), 'step_in', -1], [V('step_out'), 'step_out', 1], [V('step_pop'), 'step_pop', 2],
          [V('ult_mark'), 'ult_mark', -1], [V('ult_arrive'), 'ult_arrive', 2], [V('ult_cast'), 'ult_cast', 2], [V('buy'), 'buy', 3, true],
          [V('flash'), 'flash', 3], [V('para_trail'), 'para_trail', 2]],
        buffs: [...[0, 1, 2, 3, 4, 5].map(k => [`omn_gun${k}`, `gun_${k}`, 3]), ['omn_arm1', 'arm_1', 3], ['omn_arm2', 'arm_2', 3], ['omn_more', 'smoke_pip', 3], ['omn_cash', 'cash', 3],
          ...[0, 1, 2, 3, 4, 5, 6, 7, 8, 9].flatMap(d => [[`omn_crk${d}`, `cash_k${d}`, 3], [`omn_crh${d}`, `cash_h${d}`, 3]]),
          ['omn_blinded', 'blinded', 3]],
      }),
    };
    const text = {
      name: 'Omen',
      skill: `Dark Cover: a shadow dome 100000 across (radius 50000) for 10s, placed anywhere on the map. Nobody outside sees in or through it; someone inside sees only within 15000 and nothing outside (their teammates' sight still counts). Shooting or casting from inside reveals you for 0.5s. Two charges: one comes back on the cooldown (${secs(o.smokeCd)}), the other is bought (200 credits). He can re-smoke a spot before it fades. Where it goes: the place he's about to ult into, only when no bush or smoke there hides his arrival (a smoke warns the enemies, so it's the second option); on himself when he's low and chased; on a teammate who is; around a big objective (the enemies' way in when his team is taking it, half the pit when the enemies are on it: the far half from his team, so the near half stays in sight to pick them off); over an enemy he can reach with the shotgun or his step (step in, shotgun out); in a fight, over the enemy hitting hardest, kept off his own teammates. In a gank he never smokes the enemy being ganked: behind them (their way home and their help), or over his teammates. Enemies check a smoke in their way before walking past it: Omen could be in any of them. Passive, Buy Phase: credits instead of items (start 1000, kill +300, assist +150, +200 every 20s alive, +50 for every kill his team gets, death +400; a death costs his armor, he keeps his gun). Kills of enemies he blinded or that stood in his smoke count as his assists. He buys at his spawn, or anywhere when no enemy is near, the gun first (armor only with the Vandal's price left over, until he has one): the best one he can pay for (Sheriff from 800, Spectre from 1600, or the Judge shotgun from 1850 against close fighters, Vandal from 2900, the Operator only when rich, from 5500), then armor and the second smoke from what is left. Guns: Classic (free, 30000 range), Sheriff (800: 38000, hard-hitting, slow), Spectre (1600: 28000, fast, light), Judge (1850: 18000, shotgun: up to 2.2x point blank, falling to 0.7x, the spread clips whoever stands by the target), Vandal (2900: 42000, +30% damage, +15% speed), Operator (4700: 80000, +200% damage, very slow). Armor is a shield only buying refills: light 400 (12% max HP), heavy 1000 (25%). The gun shows over his head, and his money under it.`,
      skill2: `Shadow Step or Paranoia, picked by the situation, and used often. Shadow Step: a 0.6s channel (a shadow pool marks the spot), then he teleports up to 50000 (a bit farther than DIO's Stand Out reach), always to the safest spot that does the job: away from enemies he can see and enemy towers, toward his teammates, a bush or one of his smokes. Out when he's low and chased (into one of his smokes if one is on the way), into one of his smokes that has enemies at it (the ambush), within his gun's reach of an enemy with at most one other near it when he holds a close-range gun, on their safest side (not at all if every side is unsafe), or into one of his smokes to lurk when enemies are around it; and with nothing better to do, as a flash to a better firing spot (an enemy still in his gun's reach, somewhere clearly safer: a bush, his smoke, by his team). The risky play: a low enemy he can finish with a few shots, he steps in behind them (between them and their base, cutting the retreat) and shoots; never when he's low himself, into three or more of them, or under an enemy tower unless the kill is certain. Paranoia: a shadow ball trailing a spiral of smoke (Purple's size, slower) that flies through walls and blinds EVERY champion it touches, teammates too: the closer to Omen, the longer, 5s up close down to 3s at its full range; V1 can parry it. He throws it at any enemy within 165000, even a single one, on a line with no teammate on it, and it stops just past the last enemy it can reach, so teammates behind them are safe; the game's AI sees it coming. Chased, he throws it himself at the chasers the moment it's ready. The trade-off: through a teammate or two when it blinds at least two more enemies (3 enemies for 1 teammate), when it catches 3+ enemies on an engage, or when his side is running from a chase (they're running, not shooting). When 2+ enemies are blinded at once and his side can take them, he calls his team to rush them while the blinds last (a mark on the ground). A blinded champion can't see: shadows of the enemies around them appear where those enemies stand and drift on the way they were moving, their attacks can only go at a shadow, and the shadows can't be killed (they last the whole blind); their skills can still hit the real enemies. Cooldown ${secs(o.s2Cd)}.`,
      ult: `From the Shadows: he vanishes for a 2s channel while his shade waits at the destination, anywhere on the map; then he appears there. Killing the shade cancels it. Where: home when he's low and chased with his step down; into a fight his team is in far away (behind the enemies, or into one of his smokes there); a gank anywhere on the map (a teammate in a lane fight his side wins with him there, even bot from top): behind the enemy, cutting their way home; into one of his smokes where enemies are and a teammate is near; behind a lone enemy at 35% HP or less; to a big objective the enemies are taking. He lands in a bush near the spot when there is one (no warning for the enemies), else in one of his smokes there; smoking the spot first is his second option. Cooldown ${secs(o.ultCd)}.`,
    };
    return { json, text, vfx: 'valorant', sprite: 'omen', spriteFallback: 'asset/base/aseprite_resources/champions/shadowmancer', extraSprites: { shadow_bombardier: 'shadow_bombardier' } };
  }

  /* ================================================================ Scribble (round 62, original character)
   * A toon mage with 35 spells. He weaves up to six element dots (1 Pencil, 2 Eraser, 3 Paint, 4 Gadget, 5 Page) and
   * Invoke fires the spell whose recipe is exactly those dots, in that order. Cooldowns are per dot count (round 64): a
   * 5-dot spell puts every 5-dot spell on its cooldown; the other tiers stay free.
   * Everything is native (tfm2_custom_ai:scribble): the data slots below are never cast (the AI only moves and shoots
   * darts), the passive weaves, invokes and casts, and shows the dots and the athlete's mastery badge over him.
   */
  const SCRIBBLE_BOOK = [
    // [recipe, name, cooldown ticks, what it does]
    ['1', 'Pencil Poke', 120, '30 + 50% AP to one enemy'],
    ['2', 'Smudge', 180, 'rubs out the target\'s shields, then 20 + 30% AP'],
    ['3', 'Paint Splat', 180, '25 + 40% AP around the target, slowed 20% for 1s'],
    ['4', 'Squeaky Horn', 240, 'HONK: 15 + 30% AP and a 0.2s stun around him (interrupts)'],
    ['5', 'Paper Cut', 120, 'a flying page: 40 + 60% AP bleeding over 2s'],
    ['1-1', 'Draw a Door', 360, 'steps through a door 35000 away (out of trouble, or after a fleeing kill)'],
    ['1-3', 'Cutout', 420, 'a cardboard cutout shield 80 + 60% AP for 3s on him or a teammate under fire'],
    ['2-1', 'Erase Myself', 480, 'invisible for 1.5s'],
    ['3-3', 'Bucket', 360, 'heals the most hurt teammate near (or himself) 70 + 50% AP'],
    ['3-2', 'Banana Peel', 360, 'a peel on the ground for 5s: the first enemy on it slips (0.5s airborne, sliding)'],
    ['4-3', 'Rubber Chicken', 300, '40 + 50% AP and a knockback'],
    ['4-4', 'Wind-up Key', 480, '+40% move and attack speed for 3s'],
    ['5-4', '?! Bubble', 480, 'a confused bubble: the target can\'t attack for 0.8s'],
    ['4-1-5', 'Say Cheese', 600, 'a phone photo: everyone in the cone in front (45000) is stunned 1.2s'],
    ['4-4-1', 'Mallet', 540, 'a giant mallet up close: 60 + 80% AP, stun 1s'],
    ['4-2-3', 'Anvil', 720, 'a shadow, then an anvil 0.8s later: 90 + 100% AP and a 1s stun to whoever is still under it'],
    ['1-5-2', 'Rubber Arm', 600, 'a stretchy glove grabs an enemy up to 90000 away and pulls them in'],
    ['2-5-1', 'Erase Legs', 660, 'rubs out the target\'s legs: rooted 1.5s'],
    ['4-3-3', 'Pie', 600, 'a pie in the face: no skills for 2s, no attacks for 1s'],
    ['4-1-1', 'Present', 720, 'a gift box stuck to the target, opening 2s later: 120 + 110% AP around them'],
    ['3-1-3', 'Slippery Floor', 840, 'a waxed floor 40000 wide for 4s: enemies on it -30% speed, teammates +25%'],
    ['1-4-2-5', 'Hole Network', 1080, 'draws a hole and pops out 120000 away (home when chased, or into a fight); teammates by him follow 1s later'],
    ['4-1-5-5', 'Group Photo', 960, 'a panorama shot: everyone in a wide cone 70000 deep is stunned 1.5s'],
    ['3-4-3-4', 'Brawl', 1080, 'a cartoon dust cloud: the target and anyone by them rooted 2s, 4 hits of 30 + 25% AP'],
    ['2-2-5-1', 'Redraw', 960, 'erases himself and redraws where he was 3s ago, clearing crowd control'],
    ['5-3-4-1', 'Stamp', 1200, 'a giant rubber stamp: enemies around him stunned 1.2s, teammates there shielded 120 + 80% AP'],
    ['1-2-1-2', 'Drawn Wall', 1200, 'draws a wall 90000 long for 5s (between him and his chasers, or behind a fleeing enemy); nobody gets through'],
    ['5-5-3-1-4', 'Rewind', 2100, 'teammates within 70000 go back to where they were 4s ago with the health they had then (if it was more)'],
    ['4-2-4-1-3', 'Piano', 1800, 'a shadow, then a grand piano 1s later: 200 + 150% AP and a 1.5s stun in a big circle'],
    ['3-5-2-2-1', 'Ink Flood', 1800, 'a wave of ink 160000 long: 60 + 50% AP, no attacks for 2s, slowed 40%'],
    ['1-3-5-4-2', 'Chase Scene', 1680, '+100% move speed for 4s; every enemy he runs into is bonked once (40 + 40% AP, 0.3s stun)'],
    ['2-4-1-3-5', 'Laugh Track', 2100, 'canned laughter: enemies within 90000 can\'t attack or cast for 1.5s, teammates heal 60 + 40% AP'],
    ['5-5-1-2-3-4', 'Page Flip', 5400, 'turns the page: every champion in the top or bottom half swaps sides (top <-> bottom); mid stays'],
    ['4-4-4-1-5-2', 'PAUSE', 4800, 'pauses the replay: every enemy champion is frozen 2s'],
    ['3-1-2-4-5-3', 'Draw a Friend', 5400, 'sketches the strongest fallen teammate back in for 10s at 60% of their stats'],
  ];
  function scribble(id, o) {
    o = Object.assign({ dartRange: 60000, dartCd: 90, modId: 'tfm2_toon' }, o || {});
    const V = n => `${id}_${n}`;
    const act = actFor(id, { attack: 'attack', skill: 'skill1', skill2: 'skill2', ult: 'ult' });
    // the slots exist for the tooltips; they never cast (an enemy in crowd control within 1 unit, once a day)
    const never = slot => act(slot, { duration: 6, cooltime: 5184000, start_timing: 3, range: 1, casting_type: 'Targeting', casting_target: 'EnemyChampionInCC', attack_type: 'Skill', effect: nothing() });
    const dart = { type: 'TargetProjectile', speed: 7000, name: V('dart'), y_offset: 0, applied_target: 'Enemy', applied_effects: [onHit(all(fx(V('dart_hit')), hit(0, 100)))] };
    const fxTags = [
      // [tag, z, follow]
      ['dart_hit', 2], ['weave', 3, true], ['invoke', 3, true], ['fizzle', 3, true],
      ['poke', 2], ['smudge', 2, true], ['splat', 1], ['honk', 2, true], ['cut', 2, true],
      ['door', 1], ['cutout', 2, true], ['erase_self', 2], ['bucket', 2, true], ['peel', -1], ['slip', 2, true], ['chicken', 2], ['key', 3, true], ['bubble', 3, true],
      ['phone', 2, true], ['photo', 3, true], ['mallet', 2], ['anvil_shadow', -1], ['anvil', 3], ['erase_legs', 2, true], ['pie', 3, true], ['present', 3, true], ['boom', 3], ['floor', -1],
      ['hole', -1], ['hole_pop', 2, true], ['panorama', 3], ['brawl', 3, true], ['redraw', 2, true], ['stamp', 3], ['wall_seg', 1], ['wall_draw', 1],
      ['rewind', 3], ['rewind_swirl', 2, true], ['piano_shadow', -1], ['piano', 3], ['ink_wave', 1], ['chase', 1, true], ['bonk', 3, true], ['laugh', 3], ['haha', 3, true],
      ['page', 3], ['page_swish', 2, true], ['pause', 3], ['pause_icon', 3, true], ['sketch_in', 2, true],
    ];
    const book = SCRIBBLE_BOOK.map(([r, n, cd, what]) => `${r} ${n} (${secs(cd)}): ${what}.`).join(' ');
    const json = {
      id, category: 'Magician', tags: ['AP', 'CC', 'Range'],
      sprite: `asset/${o.modId}/champions/${id}`, anim_prefix: '',
      stat: stats(80, 40, 900, 20, 20, 950), growth: stats(6, 20, 100, 7, 3, 9),
      attack: act('attack', { duration: 24, cooltime: o.dartCd, start_timing: 14, cancelable: true, range: o.dartRange, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: 'BaseAttack', effect: dart }),
      skill: never('skill'), skill2: never('skill2'), ult: never('ult'),
      passive: { passive_ref: 'tfm2_custom_ai:scribble', params: {} },
      ...views(o.modId, 'scribble', {
        projectiles: [[V('dart'), 'dart', 2], [V('page_fly'), 'page_fly', 2], [V('glove'), 'glove', 2]],
        effects: fxTags.map(([t, z, f]) => [V(t), t, z, f]),
        buffs: [...[0, 1, 2, 3, 4, 5].flatMap(k => [1, 2, 3, 4, 5].map(e => [`scr_d${k}_${e}`, `dot${k}_${e}`, 4])),
          ...[0, 1, 2, 3, 4, 5, 6].map(r => [`scr_rank${r}`, `rank${r}`, 4]),
          // round 72: Top 10 badges (#1-#10) and the Grandmaster / Archmage / Top 10 skins (a layer behind him, one over him)
          ...[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map(p => [`scr_top${p}`, `top${p}`, 4]),
          ...[0, 1, 2].flatMap(k => [[`scr_skin${k}_b`, `skin${k}_b`, -1], [`scr_skin${k}_f`, `skin${k}_f`, 3]])],
      }),
    };
    const text = {
      name: 'Scribble',
      skill: `Weave: he draws element dots over his head, up to six: 1 Pencil, 2 Eraser, 3 Paint, 4 Gadget, 5 Page. The order matters. Every athlete here is a pro, so even a beginner clicks fast. Mastery (per athlete, from the games they have played on him: an official match counts 1, a scrim or exhibition 0.5, a win 1.5 times that; the badge at his top right) sets their speed in dots a second (CPS) and how long a recipe they build reliably. Any rank can go for any of the 35 spells, but every dot past their comfort is likely to come out wrong (a Novice going for a 6-dot spell: dot 3 80%, dot 4 86%, dot 5 92%, dot 6 98%; each rank above fumbles less). Novice (bronze, 0-4 games): 2.5 CPS, comfortable with 2 dots, often gets a dot wrong and rarely notices, and fires spells at whatever is around; Apprentice (silver, 5+): 3.25 CPS, 3 dots; Adept (gold, 15+): 4 CPS, prepares an opener on the way to a fight and holds it (a catch for a gank, an area spell for a teamfight, an escape when low); Expert (platinum, 30+): 5 CPS, 4 dots, waits for a slow spell's target to be locked down, chains a stun into the big hits, re-picks his opener when the situation changes and drops a held spell that has no use any more (or when another is clearly better), the higher the rank the quicker; Master (diamond, 60+): 6.5 CPS, 5 dots; Grandmaster (ruby, 100+): 8 CPS, almost never slips, wears the Ruby skin (a gold crown, a ruby sigil); Archmage (the rainbow book, 150+): 9.5 CPS, all 6 dots, never slips, wears the Prism skin (a rainbow halo, orbiting pages); Top 10 (the ten athletes with the most mastery on him, 300+ each; a numbered badge, #1 crowned): 11 CPS at #10 up to 15 at #1, the fastest hands in the game, wear the Legend skin (ink wings, a gold halo, gold lightning). Mastery matters more than the athlete's stats. The whole world learns him together: every cast is scored (what it really did against what it promised) and every Scribble player picks spells with what all games so far have taught (each match uses everything played before it; casts from won games and official matches weigh more).`,
      skill2: `Spell book (recipe, cooldown): ${book}`,
      ult: `Invoke: casts the spell the dots spell out (a short cast, faster with mastery), then the dots are used up. A sequence that is no recipe, or one whose tier is still on cooldown, fizzles in a puff of smoke. Cooldowns are per number of dots: a spell puts every spell with the same number of dots on its cooldown (after a 5-dot spell, no other 5-dot spell until it is back), while the other tiers stay free. If the spell he wants has its tier on cooldown he goes for the next best one he can cast, keeping the dots that start it (a Novice notices after 1.5s, an Apprentice 1s, an Adept 0.5s, Expert and up at once).`,
    };
    return { json, text, vfx: 'scribble', sprite: 'scribble', spriteFallback: 'asset/base/aseprite_resources/champions/ice_mage' };
  }

  /* ================================================================ Farming (round 60, Rian)
   * Damage skills also work on jungle monsters and lane minions. Their casting_target becomes EnemyWithoutTower (like
   * the base game's data champions), and the effect is wrapped in SwitchByBuff "mod_farm": the native rules give a
   * mod champion that buff while no enemy champion it can see is within 110000. With it, the cast is a plain
   * data-driven version of the skill that hits whatever it's aimed at; without it, the champion version as before.
   * Ults stay champion-only. Utility skills (parry, choke, rod, smokes, Sandevistan) stay as they are; David's
   * Sandevistan is used for travel by the native rules instead.
   */
  const FARM_BUFF = 'mod_farm';
  const farmAp = (damage, attack_ratio) => ({ type: 'ApAttack', damage, attack_ratio });
  function farmKit(slug, V) {
    const shot = (name, speed, ...effects) => ({ type: 'TargetProjectile', speed, name: V(name), y_offset: 0, applied_target: 'EnemyWithoutTower', applied_effects: effects.map(onHit) });
    const line = (name, speed, range, radius, ...effects) => ({ type: 'LinearProjectile', penetrate: true, speed, range, name: V(name), shape: { Circle: { radius } },
      applied_target: 'EnemyWithoutTower', applied_effects: effects.map(onHit), end_effects: [] });
    // a shot that stops at the first enemy and bursts there (one pulse of an area)
    const bomb = (name, speed, range, radius, boomFx, ...effects) => ({ type: 'LinearProjectile', penetrate: false, speed, range, name: V(name), shape: { Circle: { radius: 8000 } },
      applied_target: 'EnemyWithoutTower', applied_effects: [],
      end_effects: [fx(V(boomFx)), { type: 'RangePeriodProjectile', name: V(boomFx + '_zone'), tick: 6, period: 36, first_delay: 0, shape: { Circle: { radius } },
        applied_target: 'EnemyWithoutTower', applied_effects: effects.map(onHit), end_effects: [] }] });
    switch (slug) {
      case 'minato': return {
        skill: [line('kunai', 9000, 130000, 9000, hit(40, 60)), 'Farming: a straight kunai through the camp or the wave (40 + 60% AD to each).'],
        skill2: [all(fx(V('rasengan_hit')), hit(50, 80)), 'Farming: a Rasengan on the target (50 + 80% AD).'] };
      case 'gojo': return { skill: null, skill2: null };   // the flags already ride on basic attacks, which hit anything
      case 'dio': return {
        skill: [all(shot('knife', 9000, hit(30, 60)), shot('knife', 8000, hit(30, 60)), shot('knife', 7000, hit(30, 60))), 'Farming: three knives at the target (30 + 60% AD each).'],
        skill2: [all(fx(V('strike')), hit(60, 100)), 'Farming: a Stand strike on the target (60 + 100% AD).'] };
      case 'david': return {
        skill2: [all(...[9000, 8500, 8000, 7500, 7000, 6500].map(sp => shot('bullet', sp, hit(8, 18)))), 'Farming: the pistol barrage (6 × (8 + 18% AD)), no cyberpsychosis.'] };
      case 'v1': return {
        skill: [all(fx(V('coin')), bomb('coin_ray', 12000, 60000, 30000, 'coin', hit(30, 45))), 'Farming: a coin shot that ricochets through everything within 30000 of the target (30 + 45% AD each).'] };
      case 'vader': return {
        skill: [line('saber', 5000, 112000, 12000, hit(50, 90)), 'Farming: the saber spins through the camp or the wave (50 + 90% AD).'] };
      case 'frieren': return {
        skill: [line('zoltraak_fern', 14000, 150000, 10000, farmAp(40, 55)), 'Farming: Fern\'s Zoltraak through the camp or the wave (40 + 55% AP).'],
        skill2: [bomb('stark_drop', 6000, 60000, 20000, 'stark_land', farmAp(45, 65)), 'Farming: Stark lands on the target (45 + 65% AP around it).'] };
      case 'steve': return {
        skill: [bomb('tnt_fly', 6000, 70000, 32000, 'boom', hit(70, 60)), 'Farming: TNT on the camp or the wave (70 + 60% AD around it).'] };
      default: return null;
    }
  }
  function withFarm(slug, built) {
    const j = built.json;
    const V = n => `${j.id}_${n}`;
    const kit = farmKit(slug, V);
    if (!kit) return built;
    for (const slot of ['skill', 'skill2']) {
      if (!(slot in kit)) continue;
      j[slot].casting_target = 'EnemyWithoutTower';
      const f = kit[slot];
      if (!f) continue;   // the champion version already works on anything
      j[slot].effect = ifBuff(FARM_BUFF, f[0], j[slot].effect);
      if (built.text && built.text[slot]) built.text[slot] += ' ' + f[1];
    }
    return built;
  }

  const PRESETS = {
    minato: { label: 'Minato (Flying Raijin, Rasengan, Kurama Mode)', name: 'Minato', slug: 'minato', build: minato },
    gojo: { label: 'Gojo (Infinity, Blue, Red, Purple, Unlimited Void: auto cast)', name: 'Gojo', slug: 'gojo', build: (id, o) => gojo(id, Object.assign({ domain: 'auto' }, o)) },
    gojo_slow: { label: 'Gojo (Unlimited Void: always slow cast)', name: 'Gojo', slug: 'gojo', build: (id, o) => gojo(id, Object.assign({ domain: 'slow' }, o)) },
    gojo_fast: { label: 'Gojo (Unlimited Void: always fast cast)', name: 'Gojo', slug: 'gojo', build: (id, o) => gojo(id, Object.assign({ domain: 'fast' }, o)) },
    dio: { label: 'DIO (Stand Out / Stand In, ZA WARUDO)', name: 'DIO', slug: 'dio', folder: ['tfm2_jojo', 'JoJo'], build: dio },
    david: { label: 'David Martinez (Sandevistan, cyberpsychosis, Gravity projection)', name: 'David', slug: 'david', folder: ['tfm2_cyberpunk', 'Cyberpunk'], build: david },
    v1: { label: 'V1 (Coin toss, Parry, Railgun, blood healing)', name: 'V1', slug: 'v1', folder: ['tfm2_ultrakill', 'ULTRAKILL'], build: v1 },
    steve: { label: 'Steve (Pearl / TNT / Apple, Fishing rod, Boat wall)', name: 'Steve', slug: 'steve', folder: ['tfm2_blockcraft', 'Blockcraft'], build: steve },
    vader: { label: 'Darth Vader (Saber throw, Force choke, Rage)', name: 'Darth Vader', slug: 'vader', folder: ['tfm2_starwars', 'Star Wars'], build: vader },
    omen: { label: 'Omen (Buy Phase, Dark Cover, Shadow Step / Paranoia, From the Shadows)', name: 'Omen', slug: 'omen', folder: ['tfm2_valorant', 'Valorant'], build: omen,
      // his shadows (of mod champions) and his shade are mod-spawned units, drawn with the spare base sprite "bombardier"
      // (no base champion uses it): show the shadow figure instead
      overrides: modId => ({
        'asset/base/aseprite_resources/champions/bombardier#sheet': { remapping: `asset/${modId}/champions/shadow_bombardier#sheet`, type: 'override' },
        'asset/base/aseprite_resources/champions/bombardier#anim': { remapping: `asset/${modId}/champions/shadow_bombardier#anim`, type: 'override' } }) },
    scribble: { label: 'Scribble (35 spells: weave dots, Invoke; mastery ranks)', name: 'Scribble', slug: 'scribble', folder: ['tfm2_toon', 'Toon'], build: scribble },
    frieren: { label: 'Frieren (Fern, Stark, Limiter release)', name: 'Frieren', slug: 'frieren', folder: ['tfm2_frieren', 'Frieren'], build: frieren,
      // Stark the companion is a mod-summoned unit, which the game draws with the summon (ghoul) sprite: show Stark instead.
      // (Side effect: the Necromancer's ghouls look like Stark while this mod is on.)
      // (each half of the sprite pair is its own asset: the base keys are ghoul#sheet and ghoul#anim)
      overrides: modId => ({
        'asset/base/aseprite_resources/champions/ghoul#sheet': { remapping: `asset/${modId}/champions/stark_ghoul#sheet`, type: 'override' },
        'asset/base/aseprite_resources/champions/ghoul#anim': { remapping: `asset/${modId}/champions/stark_ghoul#anim`, type: 'override' } }) },
  };
  for (const p of Object.values(PRESETS)) {
    const b = p.build;
    p.build = (id, o) => withFarm(p.slug, b(id, o));
  }
  root.TFM2_PRESETS = PRESETS;
  root.TFM2_SCRIBBLE_BOOK = SCRIBBLE_BOOK;
  if (typeof module !== 'undefined') module.exports = PRESETS;
})(typeof window !== 'undefined' ? window : globalThis);
