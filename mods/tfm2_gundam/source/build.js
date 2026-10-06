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
    ...action('ult', 960000, 5400, 24, 0, 'AllyChampion',
      all(
        { type: 'Shield', amount: 320, attack_ratio: 0, ap_ratio: 0, tick: 180 },
        targetBuff('gdm_arrival_target', 240),
        selfBuff('gdm_arrival_cast', 240),
        { type: 'ViewEffect', name: `${id}_incoming` },
      )),
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
  ].map(([key, tag, z, is_follow]) => ({
    type: 'Animation', name: `${id}_${key}`, anim: `${asset}/vfx/gundam`, tag, z, is_follow,
  })),
  view_projectiles: [],
  view_buffs: [
    ['gdm_wings', 'wings_light', -1],
    ['gdm_fade', 'wings_fade', -1],
    ['gdm_protect', 'protect', 1],
    ['gdm_zero', 'zero_aura', -2],
    ['gdm_zero_ally', 'ally_aura', -1],
  ].map(([name, tag, z]) => ({ type: 'Animated', name, anim: `${asset}/vfx/gundam`, tag, z })),
};

const text = {
  name: 'Aegis Zero',
  passive: 'Guardian Frame: +8 Defense and Magic Resistance, +10 more while an ally within 60000 is at 40% HP or less. His basic attack is a beam saber; every third one is Palma Fiocina, an open-palm blast for 25% more damage.',
  skill: 'Palma Charge: boosts toward an enemy champion on his thrusters. The first champion struck is carried forward; driving them into terrain stuns them for 1 second. Ends in a palm blast: 55 + 50% Attack damage. 11 second cooldown.',
  skill2: 'Arondight: Challenge: draws the great sword and cuts a 37000-radius circle for 40 + 55% Attack damage, taunting enemies for 1 second. Gains a 4-second shield of 120 + 65 per enemy champion hit. During Zero Protection the Wings of Light flare with it. 12 second cooldown.',
  ult: 'Wings of Light: shields an allied champion for 320 at once and spreads the Wings of Light, then flies across the map to land beside them, leaving afterimages. The landing deals 90 + 70% Attack damage in 45000 and knocks enemies up for 1 second. Zero Protection: +20 Defense and Magic Resistance for himself and +8 for allies within 52000 for 6 seconds, while the wings stay open. Crowd control stops the charge or the flight. 90 second cooldown.',
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
