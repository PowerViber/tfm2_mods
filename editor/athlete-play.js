/* What each of the athlete's nine profile values does in a match (native athlete_play.rs, round 140).
   The numbers below are data: a native test reads `rules` and checks them against the Rust thresholds.
   An average athlete (50) plays exactly as the engine does. */
(function (root) {
  'use strict';
  const NATIVE_CASTERS = 'Gojo, Omen and DIO';
  const rules = [
    { key: 'control_speed', below: 50, nativeOnly: true, delayAt0: 18 },
    { key: 'skill_hit', notAt: 50, nativeOnly: true },
    { key: 'judgement', above: 55, nativeOnly: true, patienceTicks: 180 },
    { key: 'skill_avoid', below: 50 },
    { key: 'positioning', above: 55, stepEveryTicks: 30 },
    { key: 'ego', below: 50, retreatBase: 15, retreatPerPoint: 0.8, enterRange: 50000, exitRange: 75000, exitHpMargin: 10 },
    { key: 'aggressive', above: 60, below: 40, chaseFactor: 1.2 },
    { key: 'order', from: 65, twoFrom: 85 },
    { key: 'roaming', above: 60, baseReach: 40000, extraReach: 150000, quietRange: 100000 },
  ];
  const byKey = Object.fromEntries(rules.map(r => [r.key, r]));
  const thousands = n => Math.round(n).toLocaleString('en-US');

  /** Reaction delay in ticks before a fresh cast, for kits that cast natively. */
  function castDelay(v) {
    const reaction = 24 - Math.floor(Math.max(0, Math.min(100, v)) * 18 / 100);
    return Math.max(0, reaction - 15) * 2;
  }
  function retreatPct(ego) {
    const r = byKey.ego;
    return ego >= r.below ? 0 : Math.floor(r.retreatBase + (r.below - ego) * r.retreatPerPoint);
  }
  function roamReach(v) {
    const r = byKey.roaming;
    return r.baseReach + r.extraReach * (Math.min(100, v) - r.above) / 40;
  }

  /** One line: what this value does at this level. `active` is false when it plays like the engine. */
  function describe(key, v) {
    const r = byKey[key];
    if (!r) return { active: false, text: '' };
    const native = r.nativeOnly ? ` (${NATIVE_CASTERS}, who cast natively)` : '';
    switch (key) {
      case 'control_speed': {
        const d = castDelay(v);
        return d ? { active: true, text: `Casts wait about ${d} ticks before going out${native}.` } : { active: false, text: `Reacts at full speed${native}.` };
      }
      case 'skill_hit':
        if (v === r.notAt) return { active: false, text: `Aims at where the target is${native}.` };
        return { active: true, text: v > r.notAt ? `Leads moving targets (${v - 50}% of their motion)${native}.` : `Aims behind moving targets${native}.` };
      case 'judgement':
        return v > r.above ? { active: true, text: `Holds an ultimate up to ${Math.round(r.patienceTicks * Math.min(45, v - r.above) / 45 / 60 * 10) / 10} s until it catches two or a low target${native}.` }
          : { active: false, text: `Fires the ultimate when the kit wants to${native}.` };
      case 'skill_avoid':
        return v < r.below ? { active: true, text: `Misses the step out of a telegraphed zone ${100 - (50 + v)}% of the time (rolled once per zone).` }
          : { active: false, text: 'Always steps out of telegraphed zones.' };
      case 'positioning':
        return v > r.above ? { active: true, text: 'Ranged champions step back between their own shots (at most every 0.5 s).' }
          : { active: false, text: 'Stands where the engine puts it.' };
      case 'ego': {
        const p = retreatPct(v);
        return p ? { active: true, text: `Retreats below ${p}% HP with an enemy within ${thousands(r.enterRange)}, until ${thousands(r.exitRange)} away and ${p + r.exitHpMargin}% HP.` }
          : { active: false, text: 'Retreats when the engine decides to.' };
      }
      case 'aggressive':
        if (v > r.above) return { active: true, text: 'Keeps walking into an enemy already in reach.' };
        if (v < r.below) return { active: true, text: `Gives up chases beyond ${r.chaseFactor}× attack range.` };
        return { active: false, text: 'Chases and engages like the engine.' };
      case 'order':
        if (v >= r.twoFrom) return { active: true, text: 'Waits for two allies before attacking into two or more enemies.' };
        if (v >= r.from) return { active: true, text: 'Waits for an ally before attacking into two or more enemies.' };
        return { active: false, text: 'Attacks without waiting for allies.' };
      case 'roaming':
        return v > r.above ? { active: true, text: `Leaves a quiet spot (no enemy within ${thousands(r.quietRange)}) to join an ally's fight up to ${thousands(roamReach(v))} away.` }
          : { active: false, text: 'Stays on its job or lane.' };
    }
    return { active: false, text: '' };
  }

  /** The four game-sense ratings a fresh athlete starts with (Ratings::initial, without the ±5 seeded offset). */
  function initialRatings(p) {
    const v = k => Math.max(0, Math.min(100, Math.round(+p[k] || 0)));
    const [j, roam, order, pos, cs, hit, avoid] = ['judgement', 'roaming', 'order', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid'].map(v);
    return [
      Math.floor((2 * j + roam + order + pos) / 5),
      Math.floor((2 * cs + 2 * hit + avoid + pos) / 6),
      Math.floor((2 * pos + j + roam) / 4),
      Math.floor((2 * j + 2 * order + roam) / 5),
    ];
  }

  const PRESETS = [
    { name: 'Shot-caller', values: { order: 85, judgement: 80, roaming: 60 } },
    { name: 'Duelist', values: { aggressive: 85, ego: 75, order: 35 } },
    { name: 'Lurker', values: { roaming: 85, positioning: 70 } },
    { name: 'Anchor', values: { order: 70, ego: 30, aggressive: 35 } },
    { name: 'Mechanic', values: { control_speed: 85, skill_hit: 85, skill_avoid: 85 } },
    { name: 'Average', values: Object.fromEntries(rules.map(r => [r.key, 50])) },
  ];
  const RATINGS = ['Macro', 'Micro', 'Map Awareness', 'Strategy'];

  const api = { KEYS: rules.map(r => r.key), rules, describe, castDelay, retreatPct, roamReach, initialRatings, PRESETS, RATINGS };
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.TFM2_ATHLETE_PLAY = api;
})(typeof window !== 'undefined' ? window : globalThis);
