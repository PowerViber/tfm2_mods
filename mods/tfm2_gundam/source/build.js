// Builds the data side of Aegis Zero into the consolidated mod (mods/tfm2_custom). Combat state which data effects
// can't express lives in native/tfm2_custom_ai/src/gundam.rs. Round 88: Destiny redesign: melee basic attack, the
// Wings of Light as one buff (gdm_wings / gdm_fade), no CasterAnimation chain (it never showed in game), the text
// merged into tfm2_custom's champion.i18n (only this champion's entry is touched).
// Run from the repo root: node mods/tfm2_gundam/source/build.js
const fs = require('fs');
const path = require('path');
const repo = path.resolve(__dirname, '..', '..', '..');
const mod = path.join(repo, 'mods', 'tfm2_custom');
const id = 'tfm2_gundam_aegis_zero';
const asset = 'asset/tfm2_custom';
const time = tick => ({ Time: { tick } });
const buff = (name, tick, extra = {}) => ({ name, duration: time(tick), ...extra });
const selfBuff = (name, tick, extra = {}) => ({ type: 'AddCasterBuff', only_to_enemy: false, buff_state: buff(name, tick, extra) });
const targetBuff = (name, tick) => ({ type: 'AddBuff', buff_state: buff(name, tick) });
const all = (...effects) => ({ type: 'Combine', effects });
const area = (radius, target, ...effects) => ({ type: 'RangeEffect', shape: { Circle: { radius } }, target, apply_type: 'AroundCaster', effects });
const action = (name, range, cooltime, duration, start_timing, target, effect, other = {}) => ({
  action_name: name, cancelable: false, growth_range: 0, can_use_with_move: false,
  duration, cooltime, start_timing, range, casting_type: 'Targeting',
  casting_target: target, attack_type: 'Skill', effect, ...other,
});
const stats = (attack, hp, defence, magic_resistance, move_speed) => ({
  attack, magic_power: 0, hp, defence, magic_resistance, move_speed,
  hp_regen: 0, stack: 0, crit_chance: 0,
});

const data = {
  id, category: 'Util', tags: ['AD', 'Tank', 'Shield', 'CC', 'Melee'],
  sprite: `${asset}/champions/${id}`, anim_prefix: '',
  stat: stats(72, 1180, 42, 34, 960),
  growth: stats(12, 120, 9, 6, 8),
  attack: {
    ...action('attack', 27000, 70, 22, 10, 'Enemy',
      { type: 'Attack', damage: 0, attack_ratio: 100 },
      { cancelable: true, attack_type: 'BaseAttack' }),
  },
  // round 90: S1 Beam Saber Unleash (the cut and the slow are native: gundam.rs slice), S2 Arondight (the dash, the cut,
  // the taunt, the shield and the sword stance are native: gundam.rs Charge -> finisher)
  skill: {
    ...action('skill1', 60000, 540, 18, 4, 'EnemyChampion',
      all(selfBuff('gdm_slice', 12), targetBuff('gdm_slice_target', 12),
        { type: 'CasterViewEffect', name: `${id}_charge_start` })),
    description: `#asset/base/text/champion?description.${id}.skill`,
  },
  skill2: {
    ...action('skill2', 70000, 720, 20, 2, 'EnemyChampion',
      all(selfBuff('gdm_charge', 35), targetBuff('gdm_charge_target', 35),
        { type: 'CasterViewEffect', name: `${id}_charge_start` })),
    description: `#asset/base/text/champion?description.${id}.skill2`,
  },
  ult: {
    // round 90: only when the native defense read (gundam.rs ult_choice: save a hunted ally or join a teamfight) keeps
    // gdm_ult_ok up; any other press only adds gdm_ult_wait, which refunds most of the cooldown (as Steve's ult does)
    ...action('ult', 960000, 5400, 24, 0, 'AllyChampion', {
      type: 'SwitchByBuff', buff_name: 'gdm_ult_ok',
      effect_none: selfBuff('gdm_ult_wait', 12, { ult_cooldown_mult: 4700 }),
      effect_buff: all(
        { type: 'Shield', amount: 320, attack_ratio: 0, ap_ratio: 0, tick: 180 },
        targetBuff('gdm_arrival_target', 240),
        selfBuff('gdm_arrival_cast', 240),
        { type: 'ViewEffect', name: `${id}_incoming` },
      ),
    }),
    description: `#asset/base/text/champion?description.${id}.ult`,
  },
  passive: { passive_ref: 'tfm2_custom_ai:gundam', params: {} },
  view_effects: [
    ['charge_start', 'charge_start', 2, true],
    ['charge_hit', 'charge_hit', 2, false],
    ['wall_hit', 'wall_hit', 2, false],
    ['palm', 'palm', 2, false],
    ['saber', 'saber', 2, false],
    ['sweep', 'sweep', 2, true],
    ['incoming', 'incoming', 1, true],
    ['deploy', 'deploy', -1, true],
    ['retract', 'retract', -1, true],
    ['flare', 'flare', -1, true],
    ['landing', 'landing', -1, false],
    ['after_r', 'after_r', -1, false],
    ['after_l', 'after_l', -1, false],
    // round 90: the ult rises, marks the zone (Gundam mark), dives; S1's cuts; S2's sword
    ['ascend', 'ascend', 3, false],
    ['sky_r', 'sky_r', 4, false],
    ['sky_l', 'sky_l', 4, false],
    ...Array.from({ length: 8 }, (_, k) => [`zone_f${k}`, `zone_f${k}`, -3, false]),
    ['dive', 'dive', 4, false],
    ...Array.from({ length: 16 }, (_, a) => [`slice_cut_a${a}`, `slice_cut_a${a}`, -2, false]),
    ['sword_draw', 'sword_draw', 2, true],
    ['arondight_swing', 'arondight_swing', 2, false],
  ].map(([key, tag, z, is_follow]) => ({
    type: 'Animation', name: `${id}_${key}`, anim: `${asset}/vfx/gundam`, tag, z, is_follow,
  })),
  view_projectiles: [
    { type: 'Animated', name: `${id}_slice_wave`, anim: `${asset}/vfx/gundam`, tag: 'slice_wave', z: 3, repeat: true },
  ],
  view_buffs: [
    ['gdm_wings', 'wings_light', -1],
    ['gdm_fade', 'wings_fade', -1],
    ['gdm_protect', 'protect', 1],
    ['gdm_zero', 'zero_aura', -2],
    ['gdm_zero_ally', 'ally_aura', -1],
    ['gdm_arondight', 'sword_held', 2],
    ['gdm_burn', 'burning', 2],
    ['gdm_sliced', 'sliced', -1],
    ['gdm_slowed', 'slowed', -1],
  ].map(([name, tag, z]) => ({ type: 'Animated', name, anim: `${asset}/vfx/gundam`, tag, z })),
};

const text = {
  name: 'Aegis Zero',
  passive: 'Guardian Frame: +8 Defense and Magic Resistance, +10 more while an ally within 60000 is at 40% HP or less. His basic attack is a beam saber; every third one is Palma Fiocina, an open-palm blast for 25% more damage.',
  skill: 'Beam Saber Unleash: unleashes the beam saber toward an enemy champion, cutting a 65000-long strip of ground. Every enemy in the cut takes 45 + 55% Attack damage and is slowed by 30% for 1.5 seconds. 9 second cooldown.',
  skill2: 'Arondight: boosts toward an enemy champion on his thrusters, carrying the first champion struck (driving them into terrain stuns them for 1 second). At the end he draws Arondight and cuts a 37000-radius circle for 40 + 55% Attack damage, taunting enemies for 1 second, and gains a 4-second shield of 120 + 65 per enemy champion hit. For 5 seconds his basic attacks swing the great sword: +15% damage, they burn the target (3 burns of 10 + 10% Attack over 1.5 seconds, refreshed by each hit) and pull it a little toward him. During Zero Protection the Wings of Light flare with it. 12 second cooldown.',
  ult: 'Wings of Light: used to save an ally under attack at low HP or to join a teamfight. Shields the ally for 320 at once, spreads the Wings of Light and rises into the sky, out of reach. His landing zone is marked on the ground beside the ally with the Gundam mark: after 1.3 seconds he dives onto it. Enemies in the inner circle (76000, the size of Unlimited Void and ZA WARUDO) take 90 + 70% Attack damage and are knocked up for 1 second; those in the outer ring (114000) are slowed by 35% for 1.5 seconds. Zero Protection: +20 Defense and Magic Resistance for himself and +8 for allies within 52000 for 6 seconds, while the wings stay open. 90 second cooldown.',
};

const champFile = path.join(mod, 'champion', `${id}.data_champion`);
fs.writeFileSync(champFile, JSON.stringify(data, null, 2) + '\n');
const i18nFile = path.join(mod, 'text', 'champion.i18n');
const raw = fs.readFileSync(i18nFile, 'utf8');
const bom = raw.startsWith('﻿') ? '﻿' : '';
const i18n = JSON.parse(raw.replace(/^﻿/, ''));
i18n.en.description[id] = text;
fs.writeFileSync(i18nFile, bom + JSON.stringify(i18n, null, 2) + (raw.endsWith('\n') ? '\n' : ''));
console.log(`Built ${id} into mods/tfm2_custom`);
