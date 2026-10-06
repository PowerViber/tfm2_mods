// Builds the data side of Aegis Zero. Combat state which cannot be expressed by
// data effects lives in native/tfm2_custom_ai/src/gundam.rs.
const fs = require('fs');
const path = require('path');
const root = path.resolve(__dirname, '..');
const id = 'tfm2_gundam_aegis_zero';
const asset = 'asset/tfm2_gundam';
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
  stat: stats(68, 1180, 42, 34, 950),
  growth: stats(11, 120, 9, 6, 8),
  attack: {
    ...action('attack', 55000, 78, 22, 10, 'Enemy',
      { type: 'Attack', damage: 0, attack_ratio: 100 },
      { cancelable: true, attack_type: 'BaseAttack' }),
  },
  skill: {
    ...action('skill1', 70000, 660, 20, 2, 'EnemyChampion',
      all(selfBuff('gdm_charge', 35), targetBuff('gdm_charge_target', 35),
        { type: 'CasterViewEffect', name: `${id}_charge_start` })),
    description: `#asset/base/text/champion?description.${id}.skill`,
  },
  skill2: {
    ...action('skill2', 37000, 720, 24, 10, 'EnemyChampion',
      all(selfBuff('gdm_challenge', 16),
        { type: 'CasterViewEffect', name: `${id}_sweep` },
        area(37000, 'EnemyWithoutTower',
          { type: 'Attack', damage: 40, attack_ratio: 55 },
          { type: 'Taunt', duration: 60 },
          targetBuff('gdm_challenged', 12)))),
    description: `#asset/base/text/champion?description.${id}.skill2`,
  },
  ult: {
    ...action('ult', 960000, 5400, 60, 0, 'AllyChampion',
      all(
        { type: 'Shield', amount: 320, attack_ratio: 0, ap_ratio: 0, tick: 180 },
        targetBuff('gdm_arrival_target', 240),
        selfBuff('gdm_arrival_cast', 240),
        { type: 'ViewEffect', name: `${id}_incoming` },
        { type: 'CasterViewEffect', name: `${id}_deploy` },
        { type: 'Delayed', tick: 67, effects: [{
          type: 'SwitchByBuff', buff_name: 'gdm_flight',
          effect_buff: { type: 'CasterAnimation', name: 'ult_flight', tick: 90 },
          effect_none: all(),
        }] },
        { type: 'Delayed', tick: 137, effects: [{
          type: 'SwitchByBuff', buff_name: 'gdm_flight',
          effect_buff: { type: 'CasterAnimation', name: 'ult_dive', tick: 20 },
          effect_none: all(),
        }] },
        { type: 'Delayed', tick: 158, effects: [{
          type: 'SwitchByBuff', buff_name: 'gdm_zero',
          effect_buff: { type: 'CasterAnimation', name: 'ult_land', tick: 24 },
          effect_none: all(),
        }] },
      )),
    description: `#asset/base/text/champion?description.${id}.ult`,
  },
  passive: { passive_ref: 'tfm2_custom_ai:gundam', params: {} },
  stack_skill_index: 2,
  view_effects: [
    ['charge_start', 'charge_start', 2, true],
    ['charge_hit', 'charge_hit', 2, false],
    ['wall_hit', 'wall_hit', 2, false],
    ['sweep', 'sweep', 2, true],
    ['saber', 'saber', 2, false],
    ['vulcan', 'vulcan', 2, false],
    ['incoming', 'incoming', -1, true],
    ['deploy', 'deploy', -1, true],
    ['takeoff', 'takeoff', -1, true],
    ['flight', 'flight', 3, true],
    ['landing', 'landing', -1, false],
    ['flare', 'flare', -1, true],
    ['retract', 'retract', -1, true],
  ].map(([key, tag, z, is_follow]) => ({
    type: 'Animation', name: `${id}_${key}`, anim: `${asset}/vfx/gundam`, tag, z, is_follow,
  })),
  view_projectiles: [],
  view_buffs: [
    ['gdm_wing_open', 'wings_open', -1],
    ['gdm_wing_retract', 'wings_retract', -1],
    ['gdm_flight', 'flight', 3],
    ['gdm_protect', 'protect', 1],
    ['gdm_zero', 'zero_aura', -2],
    ['gdm_zero_ally', 'ally_aura', -1],
    ['gdm_fade', 'fade', 1],
  ].map(([name, tag, z]) => ({ type: 'Animated', name, anim: `${asset}/vfx/gundam`, tag, z })),
};

const text = {
  en: { description: { [id]: {
    name: 'Aegis Zero',
    skill: 'Shield Charge: charge toward an enemy champion. The first champion struck is carried forward; driving them into terrain stuns them for 1 second. 55 + 50% Attack damage. 11 second cooldown.',
    skill2: 'Beam Saber: Challenge: sweep a 37000-radius area for 40 + 55% Attack damage and taunt enemies for 1 second. Gain a 4-second shield of 120 + 65 per enemy champion hit. 12 second cooldown.',
    ult: 'Heroic Arrival: shield an allied champion for 320 immediately, unfold four wings, then fly across the map to land beside them. The landing deals 90 + 70% Attack damage and knocks enemies up for 1 second. Zero Protection grants Aegis Zero +20 Defense and Magic Resistance and nearby allies +8 for 6 seconds; wings remain open for its full duration. 90 second cooldown.',
  } } },
};

function write(rel, value) {
  const file = path.join(root, rel);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n');
}
write(`champion/${id}.data_champion`, data);
write('text/champion.i18n', text);
write('mod.mod_info', {
  name: 'Aegis Zero', author: 'TFM2 Mod Suite', version: '0.1.0',
  description: 'Gundam-style tank/support guardian; requires tfm2_custom_ai native rules.',
  last_updated: '2026-10-05',
  dependencies: [{ mod_id: 'base', version: '>=0.4.14' }, { mod_id: 'tfm2_custom_ai', version: '>=0.9.0' }],
});
write('mod.override_info', {
  'asset/base/text/champion': { remapping: `${asset}/text/champion`, type: 'merge' },
});
console.log(`Built ${id}`);
