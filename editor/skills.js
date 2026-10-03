/* Skill Lab — builds a Teamfight Manager 2 data-champion mod (official mod format, see
 * https://github.com/teamsamoyed/TeamfightManager2Mod) so champion skills can be rebuilt from effect blocks.
 * A .data_champion with the id of an existing champion reworks that champion while the mod is enabled.
 */
(function () {
  'use strict';
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const clone = o => JSON.parse(JSON.stringify(o));
  const humanize = k => String(k).replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());

  // ------------------------------------------------------------------ schema
  const TARGETS = ['Enemy', 'EnemyWithoutTower', 'EnemyChampion', 'EnemyChampionInCC', 'EnemyChampionRecentlyAttacked', 'Ally', 'AllyChampion', 'AllyChampionInCC', 'AllyNotSelf', 'AllyOnlySelf', 'Both', 'BothWithoutTower', 'BothChampion', 'None'];
  const CASTING = ['Targeting', 'Position', 'Direction', 'None'];
  const ATTACK_TYPES = ['BaseAttack', 'Skill', 'Dot', 'DotIgnoreShield', 'Item', 'Well'];
  const CATEGORIES = ['Melee', 'Range', 'Magician', 'Util', 'Assassin'];
  const TAGS = ['AD', 'AP', 'Heal', 'Shield', 'Dot', 'CC', 'Range', 'Melee', 'Tank', 'Magic'];
  const STATS = ['attack', 'magic_power', 'hp', 'defence', 'magic_resistance', 'move_speed', 'hp_regen', 'stack', 'crit_chance'];
  const BUFF_NUM = ['attack', 'attack_mult', 'magic_power', 'magic_power_mult', 'defence', 'defence_mult', 'hp', 'hp_mult', 'hp_regen', 'magic_resistance', 'magic_resistance_mult', 'move_speed_mult', 'attack_speed_mult', 'skill_cooldown_mult', 'ult_cooldown_mult', 'damaged_amplify', 'damaged_reduce', 'dot_amplify', 'base_attack_enemy_max_hp_damage', 'skill_enemy_max_hp_damage', 'self_max_hp_damage', 'base_attack_damaged_reduce', 'skill_damaged_reduce', 'defence_penetration', 'magic_resistance_penetration', 'range', 'heal_reduce', 'toughness', 'crit_chance', 'radius_mult', 'vamp', 'damage_reflect'];
  const BUFF_BOOL = ['cc_immune', 'undying', 'ignore_wall'];
  const PASSIVES = { 'tfm2_custom_ai:v1': [], 'tfm2_custom_ai:steve': [], 'tfm2_custom_ai:omen': [], 'tfm2_custom_ai:vader': [], 'tfm2_custom_ai:david': [], 'tfm2_custom_ai:dio': [], 'tfm2_custom_ai:ult_learned': [], ogre: ['hit_hp'], dancer: ['vamp'], ghost: ['heal', 'add_attack', 'add_attack_speed'], circus_blade: ['charge_count'], gunner: ['move_speed_up', 'move_speed_up_duration'], hunter: ['recast_duration', 'kill_extend_count'], berserker: ['cooltime_reduction', 'max_cooltime_reduction'], poison_dart_hunter: ['add_move_speed', 'range'], swordman: [], vampire: [] };

  // field kinds: int, tick (sim ticks, 60/s), dist, bool, str, target, casted, shape, aet, heal, apply, buff, applied (list of {effect,casting_type}), list (raw effects), one (single effect)
  const F = (kind, def) => ({ kind, def });
  const DMG = { damage: F('int', 50), attack_ratio: F('int', 100), hp_ratio: F('int', 0), target_hp_ratio: F('int', 0), attack_effect_type: F('aet', 'Target') };
  const PROJ = { applied_target: F('target', 'Enemy'), applied_effects: F('applied', []) };
  const E = {
    Attack: { g: 'Damage & healing', d: 'Physical damage (scales with Attack)', f: DMG },
    ApAttack: { g: 'Damage & healing', d: 'Magic damage (scales with Ability Power)', f: { damage: F('int', 80), attack_ratio: F('int', 100), hp_ratio: F('int', 0), can_crit: F('bool', false), attack_effect_type: F('aet', 'Target') } },
    FixedAttack: { g: 'Damage & healing', d: 'True/fixed damage', f: { damage: F('int', 120), attack_ratio: F('int', 0), hp_ratio: F('int', 0), target_hp_ratio: F('int', 0), attack_effect_type: F('aet', 'Target') } },
    Heal: { g: 'Damage & healing', d: 'Restore health', f: { amount: F('int', 80), attack_ratio: F('int', 0), ap_ratio: F('int', 40), heal_type: F('heal', 'Ally') } },
    Shield: { g: 'Damage & healing', d: 'Give a shield', f: { amount: F('int', 120), attack_ratio: F('int', 0), ap_ratio: F('int', 50), tick: F('tick', 300) } },
    Stun: { g: 'Crowd control', d: 'Target can’t act', f: { duration: F('tick', 60) } },
    Airborne: { g: 'Crowd control', d: 'Knock up', f: { duration: F('tick', 45) } },
    Knockback: { g: 'Crowd control', d: 'Push away', f: { speed: F('int', 2000), tick: F('tick', 10) } },
    Grab: { g: 'Crowd control', d: 'Pull target to the caster', f: { speed: F('int', 3500), tick: F('tick', 12) } },
    Pull: { g: 'Crowd control', d: 'Pull toward the effect point', f: { speed: F('int', 2500), tick: F('tick', 15) } },
    Fear: { g: 'Crowd control', d: 'Target runs away', f: { tick: F('tick', 90) } },
    Charm: { g: 'Crowd control', d: 'Target walks to the caster', f: { tick: F('tick', 90) } },
    Bind: { g: 'Crowd control', d: 'Root (can’t move)', f: { duration: F('tick', 90) } },
    Taunt: { g: 'Crowd control', d: 'Target must attack the caster', f: { duration: F('tick', 90) } },
    BlockAttack: { g: 'Crowd control', d: 'Disarm (no basic attacks)', f: { tick: F('tick', 90) } },
    BlockSkill: { g: 'Crowd control', d: 'Silence (no skills)', f: { tick: F('tick', 90) } },
    BlockMoveSkill: { g: 'Crowd control', d: 'No movement skills', f: { tick: F('tick', 90) } },
    Invisible: { g: 'Crowd control', d: 'Make the target invisible', f: { tick: F('tick', 120) } },
    Banish: { g: 'Crowd control', d: 'Remove target from the fight briefly', f: { duration: F('tick', 120), lock_effect_name: F('str', ''), end_effect_name: F('str', '') } },
    Teleport: { g: 'Movement', d: 'Blink the caster to the target/point (after a projectile: where it ended)', f: {} },
    DirTeleport: { g: 'Movement', d: 'Blink a fixed distance in the aimed direction', f: { moved: F('dist', 32000) } },
    Rush: { g: 'Movement', d: 'Dash to the point, hitting things on the way', f: { speed: F('int', 3500), range: F('dist', 50000), move_speed_ratio: F('int', 0), casting_target: F('target', 'Enemy'), penetrate: F('bool', false), applied_effects: F('applied', []) } },
    RushTime: { g: 'Movement', d: 'Dash in a direction for a time', f: { speed: F('int', 3500), tick: F('tick', 30), range: F('dist', 50000), casting_target: F('target', 'Enemy'), penetrate: F('bool', false), applied_effects: F('applied', []) } },
    RushMoveToBack: { g: 'Movement', d: 'Dash behind the target, then hit', f: { speed: F('int', 4500), applied_effects: F('list', []) } },
    MoveTo: { g: 'Movement', d: 'Move to target/point, then run end effects', f: { speed: F('int', 3500), range: F('dist', 50000), end_effects: F('list', []) } },
    MoveToTarget: { g: 'Movement', d: 'Chase a target, then run end effects', f: { speed: F('int', 3500), range: F('dist', 50000), end_effects: F('list', []) } },
    MoveBack: { g: 'Movement', d: 'Hop backwards', f: { speed: F('int', 2500), tick: F('tick', 12) } },
    LinearProjectile: { g: 'Projectiles', d: 'Skillshot in a straight line', f: { penetrate: F('bool', false), speed: F('int', 4200), range: F('dist', 65000), name: F('str', ''), shape: F('shape', { Circle: { radius: 9000 } }), ...PROJ, end_effects: F('list', []) } },
    TargetProjectile: { g: 'Projectiles', d: 'Homing shot at the target', f: { speed: F('int', 4500), name: F('str', ''), y_offset: F('int', 0), ...PROJ } },
    TargetSplashProjectile: { g: 'Projectiles', d: 'Homing shot that jumps to another target', f: { speed: F('int', 4500), name: F('str', ''), range: F('dist', 22000), y_offset: F('int', 0), ...PROJ } },
    AutoTargetProjectile: { g: 'Projectiles', d: 'Auto-aims at a nearby enemy', f: { speed: F('int', 4500), range: F('dist', 60000), name: F('str', ''), y_offset: F('int', 0), ...PROJ } },
    ParabolicProjectile: { g: 'Projectiles', d: 'Lobbed shot that lands on an area', f: { name: F('str', ''), travel_time: F('tick', 45), range: F('dist', 70000), range_effect_name: F('str', ''), shape: F('shape', { Circle: { radius: 24000 } }), ...PROJ, end_effects: F('list', []) } },
    BackToCasterLinearProjectile: { g: 'Projectiles', d: 'Returns to the caster (use inside end effects)', f: { penetrate: F('bool', true), speed: F('int', 4200), range: F('dist', 65000), name: F('str', ''), shape: F('shape', { Circle: { radius: 8000 } }), ...PROJ, end_effects: F('list', []) } },
    TargetProjectileFromProjectile: { g: 'Projectiles', d: 'Fire a new homing shot from a projectile', f: { speed: F('int', 4500), name: F('str', ''), y_offset: F('int', 0), ...PROJ } },
    RangeProjectile: { g: 'Areas', d: 'Delayed area at a point', f: { name: F('str', ''), delay: F('tick', 30), apply: F('tick', 20), shape: F('shape', { Circle: { radius: 26000 } }), ...PROJ } },
    LineRangeProjectile: { g: 'Areas', d: 'Delayed line area', f: { width: F('dist', 10000), length: F('dist', 70000), delay: F('tick', 20), apply: F('tick', 10), name: F('str', ''), ...PROJ } },
    RangePeriodProjectile: { g: 'Areas', d: 'Area that ticks repeatedly', f: { name: F('str', ''), tick: F('tick', 180), period: F('tick', 30), first_delay: F('tick', 0), shape: F('shape', { Circle: { radius: 26000 } }), ...PROJ, end_effects: F('applied', []) } },
    ApplyInProjectile: { g: 'Areas', d: 'Invisible area that triggers after a delay', f: { name: F('str', ''), follow_caster: F('bool', true), tick: F('tick', 45), shape: F('shape', { Circle: { radius: 24000 } }), ...PROJ } },
    RangeEffect: { g: 'Areas', d: 'Instant area around the caster or in front', f: { shape: F('shape', { Circle: { radius: 40000 } }), target: F('target', 'Enemy'), apply_type: F('apply', 'AroundCaster'), effects: F('list', []) } },
    ShrinkingBarrier: { g: 'Areas', d: 'Ring that closes in on a target', f: { name: F('str', ''), start_radius: F('dist', 70000), end_radius: F('dist', 16000), shrink_per_tick: F('int', 800), tick: F('tick', 120), edge_thickness: F('dist', 6000), applied_effects: F('applied', []) } },
    AddBuff: { g: 'Buffs', d: 'Buff/debuff the target', f: { buff_state: F('buff', { name: 'buff', duration: { Time: { tick: 180 } }, move_speed_mult: -30 }) } },
    AddCasterBuff: { g: 'Buffs', d: 'Buff the caster', f: { only_to_enemy: F('bool', false), buff_state: F('buff', { name: 'buff', duration: { Time: { tick: 180 } }, attack_speed_mult: 30 }) } },
    RemoveCasterBuff: { g: 'Buffs', d: 'Remove a named buff from the caster', f: { name: F('str', '') } },
    AddCasted: { g: 'Buffs', d: 'Damage/heal over time (bleed, poison, burn…)', f: { duration: F('tick', 180), period: F('tick', 30), casted_type: F('casted', 'Fire'), effects: F('list', []) } },
    Combine: { g: 'Flow', d: 'Do several things at once', f: { effects: F('list', []) } },
    Delayed: { g: 'Flow', d: 'Do things after a delay', f: { tick: F('tick', 30), effects: F('list', []) } },
    WithSelf: { g: 'Flow', d: 'Apply effects to the caster', f: { effects: F('list', []) } },
    RandomTarget: { g: 'Flow', d: 'Pick a random target nearby', f: { range: F('dist', 65000), casting_target: F('target', 'EnemyChampion'), from_projectile: F('bool', false), effects: F('list', []) } },
    SwitchByBuff: { g: 'Flow', d: 'Different effect while the caster has a buff', f: { buff_name: F('str', ''), effect_none: F('one', null), effect_buff: F('one', null) } },
    SwitchByLevel3: { g: 'Flow', d: 'Upgrade at level 3', f: { effect_start: F('one', null), effect_level3: F('one', null) } },
    ViewEffect: { g: 'Visual & sound', d: 'Play a named visual at the target', f: { name: F('str', '') } },
    CasterViewEffect: { g: 'Visual & sound', d: 'Play a named visual on the caster', f: { name: F('str', '') } },
    CasterAnimation: { g: 'Visual & sound', d: 'Caster animation state', f: { name: F('str', ''), tick: F('tick', 30) } },
    RemoveCasterAnimation: { g: 'Visual & sound', d: 'End a caster animation state', f: { name: F('str', '') } },
    Sfx: { g: 'Visual & sound', d: 'Sound at the caster', f: { name: F('str', '') } },
    TargetSfx: { g: 'Visual & sound', d: 'Sound at the target', f: { name: F('str', '') } },
  };
  const ORDER_GROUPS = ['Damage & healing', 'Crowd control', 'Movement', 'Projectiles', 'Areas', 'Buffs', 'Flow', 'Visual & sound'];
  const TIME_KEYS = /^(duration|tick|delay|apply|period|first_delay|travel_time|cooltime|start_timing|move_speed_up_duration|recast_duration)$/;
  const CHILD_KINDS = new Set(['applied', 'list', 'one']);
  const SLOTS = [['attack', 'Basic attack'], ['skill', 'Ability 1'], ['skill2', 'Ability 2'], ['ult', 'Ultimate']];

  function defaultEffect(type) {
    const def = E[type]; const n = { type };
    if (!def) return n;
    for (const [k, fd] of Object.entries(def.f)) if (fd.def !== null) n[k] = clone(fd.def);
    return n;
  }
  const asRaw = list => (list || []).map(x => (x && x.effect ? x.effect : x));
  const asApplied = list => (list || []).map(x => (x && x.effect ? x : { effect: x, casting_type: 'Targeting' }));
  function convert(old, type) {
    const n = defaultEffect(type); const def = E[type] || { f: {} };
    for (const [k, v] of Object.entries(old)) {
      if (k === 'type' || !def.f[k]) continue;
      const kind = def.f[k].kind;
      if (kind === 'applied') { if (Array.isArray(v)) n[k] = asApplied(clone(v)); }
      else if (kind === 'list') { if (Array.isArray(v)) n[k] = asRaw(clone(v)); }
      else if (typeof v === typeof def.f[k].def || def.f[k].def === null) n[k] = clone(v);
    }
    // keep a CC duration when swapping between CC types that name it differently
    const dur = old.duration != null ? old.duration : old.tick;
    if (dur != null) { if ('duration' in n && old.duration == null) n.duration = dur; if ('tick' in n && old.tick == null && !def.f.speed) n.tick = dur; }
    // keep children when swapping between effects with different child keys
    const kids = old.applied_effects || old.effects || old.end_effects;
    if (kids && kids.length) {
      const target = Object.entries(def.f).find(([, fd]) => fd.kind === 'applied' || fd.kind === 'list');
      if (target && !(n[target[0]] || []).length) {
        const [tk, fd] = target;
        const raw = kids.map(x => (x && x.effect ? x.effect : x));
        n[tk] = fd.kind === 'applied' ? raw.map(e => ({ effect: e, casting_type: 'Targeting' })) : raw;
      }
    }
    return n;
  }

  // ------------------------------------------------------------------ describe (auto descriptions)
  const sec = t => (+(t / 60).toFixed(2)) + 's';
  function describe(e, dmgWord) {
    if (!e || !e.type) return '';
    const kids = (list) => (list || []).map(x => describe(x && x.effect ? x.effect : x)).filter(Boolean).join(', ');
    const hit = e.applied_effects ? kids(e.applied_effects) : '';
    const end = e.end_effects && e.end_effects.length ? kids(e.end_effects) : '';
    switch (e.type) {
      case 'Attack': return `deals ${e.damage || 0} + ${e.attack_ratio || 0}% AD physical damage`;
      case 'ApAttack': return `deals ${e.damage || 0} + ${e.attack_ratio || 0}% AP magic damage`;
      case 'FixedAttack': return `deals ${e.damage || 0} true damage`;
      case 'Heal': return `heals ${e.amount || 0}${e.ap_ratio ? ' + ' + e.ap_ratio + '% AP' : ''}${e.attack_ratio ? ' + ' + e.attack_ratio + '% AD' : ''}`;
      case 'Shield': return `shields for ${e.amount || 0}${e.ap_ratio ? ' + ' + e.ap_ratio + '% AP' : ''} (${sec(e.tick || 300)})`;
      case 'Stun': return `stuns for ${sec(e.duration || 0)}`;
      case 'Airborne': return `knocks up for ${sec(e.duration || 0)}`;
      case 'Bind': return `roots for ${sec(e.duration || 0)}`;
      case 'Taunt': return `taunts for ${sec(e.duration || 0)}`;
      case 'Fear': return `fears for ${sec(e.tick || 0)}`;
      case 'Charm': return `charms for ${sec(e.tick || 0)}`;
      case 'Knockback': return 'knocks back';
      case 'Grab': return 'pulls the target in';
      case 'Pull': return 'pulls enemies in';
      case 'BlockAttack': return `disarms for ${sec(e.tick || 0)}`;
      case 'BlockSkill': return `silences for ${sec(e.tick || 0)}`;
      case 'Invisible': return `turns invisible for ${sec(e.tick || 0)}`;
      case 'Banish': return `banishes for ${sec(e.duration || 0)}`;
      case 'Teleport': return 'teleports there';
      case 'DirTeleport': return 'blinks forward';
      case 'Rush': case 'RushTime': return `dashes${hit ? ', and on hit ' + hit : ''}`;
      case 'RushMoveToBack': return `dashes behind the target${e.applied_effects && e.applied_effects.length ? ' and ' + kids(e.applied_effects) : ''}`;
      case 'MoveTo': case 'MoveToTarget': return `moves in${end ? ', then ' + end : ''}`;
      case 'MoveBack': return 'hops back';
      case 'LinearProjectile': return `fires a ${e.penetrate ? 'piercing ' : ''}projectile that ${hit || 'flies forward'}${end ? '; where it ends, ' + end : ''}`;
      case 'TargetProjectile': case 'TargetProjectileFromProjectile': return `fires a homing shot that ${hit}`;
      case 'TargetSplashProjectile': return `fires a bouncing shot that ${hit}`;
      case 'AutoTargetProjectile': return `fires at a nearby enemy: ${hit}`;
      case 'ParabolicProjectile': return `lobs a projectile that ${hit}${end ? ', then ' + end : ''}`;
      case 'RangeProjectile': case 'LineRangeProjectile': return `after ${sec(e.delay || 0)}, the area ${hit}`;
      case 'RangePeriodProjectile': return `creates a zone for ${sec(e.tick || 0)} that every ${sec(e.period || 1)} ${hit}`;
      case 'ApplyInProjectile': return `after ${sec(e.tick || 0)}, nearby enemies: ${hit}`;
      case 'RangeEffect': return `enemies in the area: ${kids(e.effects)}`;
      case 'ShrinkingBarrier': return `creates a closing ring that ${hit}`;
      case 'AddBuff': return 'applies ' + buffText(e.buff_state);
      case 'AddCasterBuff': return 'gains ' + buffText(e.buff_state);
      case 'AddCasted': return `${(e.casted_type || 'Fire').toLowerCase()}s over ${sec(e.duration || 0)}: ${kids(e.effects)}`;
      case 'Combine': case 'WithSelf': return kids(e.effects);
      case 'Delayed': return `after ${sec(e.tick || 0)}, ${kids(e.effects)}`;
      case 'RandomTarget': return `a random target ${kids(e.effects)}`;
      case 'SwitchByBuff': return `${describe(e.effect_none)} (empowered: ${describe(e.effect_buff)})`;
      case 'SwitchByLevel3': return `${describe(e.effect_start)} (level 3+: ${describe(e.effect_level3)})`;
      default: return '';
    }
  }
  function buffText(b) {
    if (!b) return 'a buff';
    const parts = BUFF_NUM.filter(k => b[k]).map(k => `${b[k] > 0 ? '+' : ''}${b[k]}${/mult|amplify|reduce|penetration|vamp|reflect/.test(k) ? '%' : ''} ${humanize(k).replace(/ Mult$/, '')}`);
    BUFF_BOOL.filter(k => b[k]).forEach(k => parts.push(humanize(k)));
    const d = b.duration && b.duration.Time ? ' for ' + sec(b.duration.Time.tick) : '';
    return (parts.join(', ') || 'a buff') + d;
  }
  const sentence = s => s ? s.charAt(0).toUpperCase() + s.slice(1) + '.' : '';

  // ------------------------------------------------------------------ state
  // Each folder is its own mod in the game's mods folder, so it can be switched on/off in the game's Mod Manager.
  // folder: { id, name, version, diskVersion, champs, removed, dirty, isNew (not on disk yet), open }
  const L = { ctx: null, loaded: false, mods: [], cur: null, sel: null, showJson: false, enabled: [], modsDir: '', gameRunning: false };
  const newFolder = (id, name) => ({ id, name, version: '0.1.0', diskVersion: null, champs: [], removed: [], dirty: false, isNew: true, open: true });
  // the Skill Lab code below works on "the current folder" through these
  for (const [k, f] of [['modId', 'id'], ['modName', 'name'], ['modVersion', 'version'], ['diskVersion', 'diskVersion'], ['champs', 'champs'], ['removed', 'removed']]) {
    Object.defineProperty(L, k, { get() { return L.cur ? L.cur[f] : (f === 'champs' || f === 'removed' ? [] : ''); }, set(v) { if (L.cur) L.cur[f] = v; } });
  }
  Object.defineProperty(L, 'dirty', { get() { return !!(L.cur && L.cur.dirty); }, set(v) { if (L.cur) L.cur.dirty = v; } });
  const anyDirty = () => L.mods.some(m => m.dirty);
  const folderOf = c => L.mods.find(m => m.champs.includes(c));
  const champFile = c => `champion/${c.json.id}.data_champion${c.disabled ? '.off' : ''}`;
  // champ entry: { json, text: {name, skill, skill2, ult}, auto: {skill:true…}, isNew, disabled, file }

  // ------------------------------------------------------------------ templates
  function dmgType(tags) { return (tags || []).includes('AP') && !(tags || []).includes('AD') ? 'ApAttack' : 'Attack'; }
  function nativeTemplate(id) {
    const G = window.TFM2_GAMEDATA; const info = G.info[id];
    const live = L.ctx.liveStats(id) || {};
    const j = { id, category: info.category, tags: info.tags.slice(), stat: clone(live.stat || info.stat), growth: clone(live.growth || info.growth) };
    const D = dmgType(info.tags);
    const cc = a => {
      const out = [];
      const t = (k1, k2) => a[k1] != null ? a[k1] : a[k2];
      if (t('stun', 'stun_duration') != null) out.push({ type: 'Stun', duration: t('stun', 'stun_duration') });
      if (t('airborne', 'airborne_time') != null) out.push({ type: 'Airborne', duration: t('airborne', 'airborne_time') });
      if (t('bind', 'bind_duration') != null) out.push({ type: 'Bind', duration: t('bind', 'bind_duration') });
      if (a.fear_duration != null || a.fear_tick != null) out.push({ type: 'Fear', tick: a.fear_duration || a.fear_tick });
      if (a.taunt_duration != null) out.push({ type: 'Taunt', duration: a.taunt_duration });
      if (a.charm_duration != null) out.push({ type: 'Charm', tick: a.charm_duration });
      if (a.knockback_speed != null) out.push({ type: 'Knockback', speed: a.knockback_speed, tick: a.knockback_tick || 10 });
      if (a.slow_duration != null) out.push({ type: 'AddBuff', buff_state: { name: `${id}_slow`, duration: { Time: { tick: a.slow_duration } }, move_speed_mult: -(a.slow_ratio || a.slow_speed || a.slow || 30) } });
      return out;
    };
    const dmg = a => ({ type: D, damage: a.attack || a.damage || 0, attack_ratio: a.attack_ratio || a.ap_ratio || a.magic_ratio || a.damage_ratio || 0 });
    const act = (slot, a) => {
      const base = { action_name: slot, duration: a.duration || 20, cooltime: a.cooltime || 300, start_timing: a.start_timing || 10, cancelable: !!a.cancelable, range: a.range || a.cast_range || a.attack_range || 40000, casting_type: 'Targeting', casting_target: 'Enemy', attack_type: slot === 'attack' ? 'BaseAttack' : 'Skill' };
      if (slot !== 'attack') base.description = `#asset/base/text/champion?description.${id}.${slot}`;
      if (slot === 'attack') {
        base.can_use_with_move = !!a.can_use_with_move;
        base.effect = a.speed ? { type: 'TargetProjectile', speed: a.speed, name: a.name || `${id}_attack`, y_offset: a.y_offset || 0, applied_target: 'Enemy', applied_effects: [{ effect: { type: 'Attack', damage: a.attack || 0, attack_ratio: a.attack_ratio || 100 }, casting_type: 'Targeting' }] }
          : { type: 'Attack', damage: a.attack || 0, attack_ratio: a.attack_ratio || 100 };
        return base;
      }
      const hits = [];
      if ((a.attack || a.damage || a.attack_ratio || a.ap_ratio || a.magic_ratio)) hits.push(dmg(a));
      hits.push(...cc(a));
      if (a.heal != null || a.heal_ratio != null) { base.casting_target = 'AllyChampion'; return Object.assign(base, { effect: { type: 'Combine', effects: [{ type: 'Heal', amount: a.heal || 0, ap_ratio: a.heal_ratio || 0, heal_type: 'Ally' }, ...(a.shield != null ? [{ type: 'Shield', amount: a.shield, ap_ratio: a.shield_ratio || 0, tick: a.shield_duration || 180 }] : [])] } }); }
      if (a.shield != null || a.shield_amount != null) { base.casting_target = 'AllyChampion'; return Object.assign(base, { effect: { type: 'Shield', amount: a.shield || a.shield_amount || 0, ap_ratio: a.shield_ratio || a.shield_ap_ratio || 0, tick: a.shield_duration || 180 } }); }
      if (!hits.length) { base.casting_type = 'None'; base.casting_target = 'AllyOnlySelf'; return Object.assign(base, { effect: { type: 'AddCasterBuff', buff_state: { name: `${id}_${slot}`, duration: { Time: { tick: a.buff_duration || a.duration || 180 } }, attack_speed_mult: 30 } } }); }
      if (a.speed && (a.range || a.projectile_range)) {
        base.casting_type = 'Direction';
        return Object.assign(base, { effect: { type: 'LinearProjectile', penetrate: false, speed: a.speed, range: a.range || a.projectile_range, name: `${id}_${slot}`, shape: { Circle: { radius: a.projectile_radius || a.attack_range || 9000 } }, applied_target: 'Enemy', applied_effects: hits.map(h => ({ effect: h, casting_type: 'Targeting' })), end_effects: [] } });
      }
      base.casting_type = 'None';
      return Object.assign(base, { effect: { type: 'RangeEffect', shape: { Circle: { radius: a.attack_range || a.explosion_range || a.splash_range || a.range || 35000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: hits } });
    };
    j.attack = act('attack', info.attack || {});
    j.skill = act('skill', info.skill || info.skill1 || {});
    j.skill2 = act('skill2', info.skill2 || {});
    j.ult = act('ult', info.ult || {});
    return j;
  }
  function dataTemplate(id) {
    const live = L.ctx.liveJson(id);
    const G = window.TFM2_GAMEDATA;
    return clone(live || (G.modChampions || []).find(m => m.id === id));
  }
  function blankTemplate(id, name) {
    const j = nativeTemplate('archer'); j.id = id;
    j.sprite = 'asset/base/aseprite_resources/champions/archer'; j.anim_prefix = '';
    for (const [s] of SLOTS) if (j[s].description) j[s].description = `#asset/base/text/champion?description.${id}.${s}`;
    return j;
  }

  // ------------------------------------------------------------------ recipes
  function recipes(id, slot, tags) {
    const D = dmgType(tags); const nm = `${id}_${slot}`;
    const hit = (dmg, ratio) => ({ type: D, damage: dmg, attack_ratio: ratio });
    const ap = (e, ct) => ({ effect: e, casting_type: ct || 'Targeting' });
    return {
      proj_tp: { label: 'Projectile → teleport where it lands', cast: ['Direction', 'Enemy'], range: 65000, effect: { type: 'LinearProjectile', penetrate: false, speed: 4200, range: 65000, name: nm, shape: { Circle: { radius: 9000 } }, applied_target: 'Enemy', applied_effects: [ap(hit(60, 80))], end_effects: [{ type: 'Teleport' }] } },
      proj_stun: { label: 'Skillshot that stuns', cast: ['Direction', 'Enemy'], range: 70000, effect: { type: 'LinearProjectile', penetrate: false, speed: 4500, range: 70000, name: nm, shape: { Circle: { radius: 9000 } }, applied_target: 'Enemy', applied_effects: [ap(hit(70, 90)), ap({ type: 'Stun', duration: 60 })], end_effects: [] } },
      blink_back: { label: 'Blink behind enemy & strike', cast: ['Targeting', 'EnemyChampion'], range: 60000, effect: { type: 'RushMoveToBack', speed: 4500, applied_effects: [hit(80, 120), { type: 'Stun', duration: 30 }] } },
      dash_through: { label: 'Dash through enemies, knock up', cast: ['Direction', 'Enemy'], range: 50000, effect: { type: 'RushTime', speed: 3800, tick: 18, range: 12000, casting_target: 'Enemy', penetrate: true, applied_effects: [ap(hit(60, 80)), ap({ type: 'Airborne', duration: 45 })] } },
      aoe_stun: { label: 'Stun everything around me', cast: ['None', 'Enemy'], range: 35000, effect: { type: 'RangeEffect', shape: { Circle: { radius: 35000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: [hit(90, 80), { type: 'Stun', duration: 60 }] } },
      line_knockup: { label: 'Delayed line knock-up', cast: ['Direction', 'Enemy'], range: 70000, effect: { type: 'LineRangeProjectile', width: 12000, length: 70000, delay: 18, apply: 6, name: nm, applied_target: 'Enemy', applied_effects: [ap(hit(80, 90)), ap({ type: 'Airborne', duration: 50 })] } },
      meteor: { label: 'Meteor: lobbed AoE that stuns', cast: ['Position', 'EnemyChampion'], range: 70000, effect: { type: 'ParabolicProjectile', name: nm, travel_time: 45, range: 70000, range_effect_name: '', shape: { Circle: { radius: 24000 } }, applied_target: 'Enemy', applied_effects: [ap(hit(120, 100)), ap({ type: 'Stun', duration: 40 })], end_effects: [] } },
      poison_zone: { label: 'Poison zone (damage over time)', cast: ['Position', 'EnemyChampion'], range: 70000, effect: { type: 'RangePeriodProjectile', name: nm, tick: 180, period: 30, first_delay: 0, shape: { Circle: { radius: 26000 } }, applied_target: 'EnemyWithoutTower', applied_effects: [ap({ type: 'ApAttack', damage: 20, attack_ratio: 20 }), ap({ type: 'AddBuff', buff_state: { name: `${nm}_slow`, duration: { Time: { tick: 40 } }, move_speed_mult: -25 } })], end_effects: [] } },
      pull_in: { label: 'Pull enemies to me', cast: ['None', 'Enemy'], range: 40000, effect: { type: 'RangeEffect', shape: { Circle: { radius: 40000 } }, target: 'Enemy', apply_type: 'AroundCaster', effects: [{ type: 'Grab', speed: 3500, tick: 12 }, hit(50, 60)] } },
      heal_shield: { label: 'Heal + shield an ally', cast: ['Targeting', 'AllyChampion'], range: 60000, effect: { type: 'Combine', effects: [{ type: 'Heal', amount: 80, attack_ratio: 0, ap_ratio: 40, heal_type: 'Ally' }, { type: 'Shield', amount: 100, attack_ratio: 0, ap_ratio: 40, tick: 180 }] } },
      self_buff: { label: 'Frenzy: attack speed + move speed', cast: ['None', 'AllyOnlySelf'], range: 0, effect: { type: 'AddCasterBuff', only_to_enemy: false, buff_state: { name: nm, duration: { Time: { tick: 240 } }, attack_speed_mult: 40, move_speed_mult: 20 } } },
      stealth: { label: 'Vanish: invisible + speed', cast: ['None', 'AllyOnlySelf'], range: 0, effect: { type: 'Combine', effects: [{ type: 'WithSelf', effects: [{ type: 'Invisible', tick: 120 }] }, { type: 'AddCasterBuff', buff_state: { name: nm, duration: { Time: { tick: 120 } }, move_speed_mult: 30 } }] } },
      bounce: { label: 'Bouncing bolt', cast: ['Targeting', 'EnemyChampion'], range: 60000, effect: { type: 'TargetSplashProjectile', speed: 4500, name: nm, range: 25000, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(70, 70))] } },
      multi_blink: { label: '3 shots at random enemies, blink to each hit', cast: ['None', 'EnemyChampion'], range: 70000, effect: { type: 'Combine', effects: [0, 25, 50].map(d => {
        const rt = { type: 'RandomTarget', range: 70000, casting_target: 'EnemyChampion', from_projectile: false, effects: [{ type: 'TargetProjectile', speed: 5000, name: nm, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(40, 60)), ap({ type: 'Teleport' })] }] };
        return d ? { type: 'Delayed', tick: d, effects: [rt] } : rt; }) } },
      blink_random: { label: 'Shots at 3 enemies, blink to one random hit', cast: ['None', 'EnemyChampion'], range: 70000, effect: { type: 'Combine', effects: [
        { type: 'RandomTarget', range: 70000, casting_target: 'EnemyChampion', from_projectile: false, effects: [{ type: 'TargetProjectile', speed: 5000, name: nm, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(40, 60))] }] },
        { type: 'RandomTarget', range: 70000, casting_target: 'EnemyChampion', from_projectile: false, effects: [{ type: 'TargetProjectile', speed: 5000, name: nm, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(40, 60))] }] },
        { type: 'RandomTarget', range: 70000, casting_target: 'EnemyChampion', from_projectile: false, effects: [{ type: 'TargetProjectile', speed: 5000, name: nm, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(40, 60)), ap({ type: 'Teleport' })] }] }] } },
      burn: { label: 'Burn (damage over time on hit)', cast: ['Targeting', 'EnemyChampion'], range: 60000, effect: { type: 'TargetProjectile', speed: 4500, name: nm, y_offset: 0, applied_target: 'Enemy', applied_effects: [ap(hit(40, 50)), ap({ type: 'AddCasted', duration: 180, period: 30, casted_type: 'Fire', effects: [{ type: 'ApAttack', damage: 15, attack_ratio: 15 }] })] } },
    };
  }

  // ------------------------------------------------------------------ render helpers
  const P = path => esc(JSON.stringify(path));
  const getAt = (o, path) => path.reduce((x, k) => (x == null ? x : x[k]), o);
  function setAt(o, path, v) { const parent = getAt(o, path.slice(0, -1)); parent[path[path.length - 1]] = v; }
  function typeSelect(cur, attr) {
    return `<select ${attr}>${ORDER_GROUPS.map(g => `<optgroup label="${g}">${Object.entries(E).filter(([, d]) => d.g === g).map(([t, d]) => `<option value="${t}" ${t === cur ? 'selected' : ''} title="${esc(d.d)}">${humanize(t).replace(/([a-z])([A-Z])/g, '$1 $2')}</option>`).join('')}</optgroup>`).join('')}${cur && !E[cur] ? `<option selected value="${esc(cur)}">${esc(cur)}</option>` : ''}</select>`;
  }
  const opt = (list, cur) => list.map(x => `<option ${x === cur ? 'selected' : ''}>${x}</option>`).join('');
  function numInput(path, k, v, kind) {
    const hint = kind === 'tick' || TIME_KEYS.test(k) ? `<span class="fx-hint">${sec(+v || 0)}</span>` : '';
    return `<label class="fx-f"><span>${humanize(k)}</span><input type="number" data-path="${P([...path, k])}" data-kind="num" value="${esc(v)}">${hint}</label>`;
  }
  function fieldHTML(node, path, k, kind) {
    const v = node[k];
    switch (kind) {
      case 'int': case 'tick': case 'dist': return numInput(path, k, v == null ? 0 : v, kind);
      case 'bool': return `<label class="fx-f fx-bool"><input type="checkbox" data-path="${P([...path, k])}" data-kind="bool" ${v ? 'checked' : ''}><span>${humanize(k)}</span></label>`;
      case 'str': return `<label class="fx-f"><span>${humanize(k)}</span><input type="text" data-path="${P([...path, k])}" data-kind="str" value="${esc(v || '')}"></label>`;
      case 'target': return `<label class="fx-f"><span>${humanize(k)}</span><select data-path="${P([...path, k])}" data-kind="str">${opt(TARGETS, v)}</select></label>`;
      case 'casted': return `<label class="fx-f"><span>Type</span><select data-path="${P([...path, k])}" data-kind="str">${opt(['Fire', 'Poison', 'Bleed', 'Heal'], v)}</select></label>`;
      case 'shape': {
        const kindName = v && typeof v === 'object' ? Object.keys(v)[0] : 'Circle';
        const body = (v && v[kindName]) || {};
        return `<div class="fx-f fx-shape"><span>Shape</span><select data-path="${P([...path, k])}" data-kind="shape">${opt(['Circle', 'Rect', 'DirDot', 'Line'], kindName)}</select>
          ${Object.entries(body).map(([bk, bv]) => `<label class="fx-mini">${humanize(bk)}<input type="number" data-path="${P([...path, k, kindName, bk])}" data-kind="num" value="${esc(bv)}"></label>`).join('')}</div>`;
      }
      case 'aet': {
        const name = typeof v === 'string' ? v : v && v.EnemyAll ? 'EnemyAll' : 'Target';
        return `<div class="fx-f"><span>Hits</span><select data-path="${P([...path, k])}" data-kind="aet">${opt(['Target', 'EnemyTarget', 'EnemyAll'], name)}</select>
          ${name === 'EnemyAll' ? `<label class="fx-mini">Range<input type="number" data-path="${P([...path, k, 'EnemyAll', 'range'])}" data-kind="num" value="${esc(v.EnemyAll.range)}"></label>` : ''}</div>`;
      }
      case 'heal': {
        const name = typeof v === 'string' ? v : v && v.AllyAll ? 'AllyAll' : 'Any';
        return `<div class="fx-f"><span>Heals</span><select data-path="${P([...path, k])}" data-kind="heal">${opt(['Caster', 'Any', 'Ally', 'AllyAll'], name)}</select>
          ${name === 'AllyAll' ? `<label class="fx-mini">Range<input type="number" data-path="${P([...path, k, 'AllyAll', 'range'])}" data-kind="num" value="${esc(v.AllyAll.range)}"></label>` : ''}</div>`;
      }
      case 'apply': {
        const name = typeof v === 'string' ? v : v && v.Forward ? 'Forward' : 'AroundCaster';
        return `<div class="fx-f"><span>Placed</span><select data-path="${P([...path, k])}" data-kind="apply">${opt(['AroundCaster', 'Forward'], name)}</select>
          ${name === 'Forward' ? `<label class="fx-mini">Offset<input type="number" data-path="${P([...path, k, 'Forward', 'offset'])}" data-kind="num" value="${esc(v.Forward.offset)}"></label>` : ''}</div>`;
      }
      case 'buff': return buffHTML(v || { name: 'buff' }, [...path, k]);
      default: return '';
    }
  }
  function buffHTML(b, path) {
    const dk = b.duration == null ? 'Permanent' : typeof b.duration === 'string' ? b.duration : Object.keys(b.duration)[0];
    const present = BUFF_NUM.filter(k => b[k] != null);
    return `<div class="fx-buff">
      <label class="fx-f"><span>Buff name</span><input type="text" data-path="${P([...path, 'name'])}" data-kind="str" value="${esc(b.name || '')}"></label>
      <div class="fx-f"><span>Lasts</span><select data-path="${P([...path, 'duration'])}" data-kind="bufdur">${opt(['Time', 'Permanent', 'WithShield'], dk)}</select>
        ${dk === 'Time' ? `<label class="fx-mini">Ticks<input type="number" data-path="${P([...path, 'duration', 'Time', 'tick'])}" data-kind="num" value="${esc(b.duration.Time.tick)}"><span class="fx-hint">${sec(b.duration.Time.tick)}</span></label>` : ''}</div>
      ${present.map(k => `<label class="fx-f"><span>${humanize(k)}</span><input type="number" data-path="${P([...path, k])}" data-kind="num" value="${esc(b[k])}"><button class="fx-x" data-del="${P([...path, k])}" title="remove">✕</button></label>`).join('')}
      ${BUFF_BOOL.map(k => `<label class="fx-f fx-bool"><input type="checkbox" data-path="${P([...path, k])}" data-kind="bool" ${b[k] ? 'checked' : ''}><span>${humanize(k)}</span></label>`).join('')}
      <select class="fx-addstat" data-addstat="${P(path)}"><option value="">+ add stat change…</option>${BUFF_NUM.filter(k => !present.includes(k)).map(k => `<option value="${k}">${humanize(k)}</option>`).join('')}</select>
    </div>`;
  }
  function nodeHTML(node, path, depth) {
    if (!node || typeof node !== 'object') return '';
    const def = E[node.type];
    const known = def ? def.f : {};
    const fields = [], kids = [];
    for (const [k, fd] of Object.entries(known)) (CHILD_KINDS.has(fd.kind) ? kids : fields).push([k, fd.kind]);
    // unknown extra fields (e.g. Native effects from the game's own data champions)
    for (const [k, v] of Object.entries(node)) {
      if (k === 'type' || known[k]) continue;
      if (typeof v === 'number') fields.push([k, 'int']); else if (typeof v === 'boolean') fields.push([k, 'bool']); else if (typeof v === 'string') fields.push([k, 'str']);
      else if (Array.isArray(v)) kids.push([k, v.length && v[0] && v[0].effect ? 'applied' : 'list']);
      else if (v && typeof v === 'object' && v.type) kids.push([k, 'one']);
    }
    const sum = sentence(describe(node));
    return `<div class="fx depth${Math.min(depth, 4)}" data-node="${P(path)}">
      <div class="fx-head">${node.type === 'Native' ? `<span class="fx-native" title="Built-in game code">Native · ${esc(node.effect_ref)}</span>` : typeSelect(node.type, `data-type="${P(path)}"`)}
        <span class="fx-sum" title="${esc(sum)}">${esc(sum)}</span>
        <span class="fx-tools"><button data-op="up" title="Move up">↑</button><button data-op="down" title="Move down">↓</button><button data-op="dup" title="Duplicate">⧉</button><button data-op="wrap" title="Wrap in Combine">⊕</button><button data-op="del" title="Remove">✕</button></span></div>
      ${def && def.d ? `<div class="fx-desc">${esc(def.d)}</div>` : ''}
      ${fields.length ? `<div class="fx-fields">${fields.map(([k, kind]) => fieldHTML(node, path, k, kind)).join('')}</div>` : ''}
      ${kids.map(([k, kind]) => childBlock(node, path, k, kind, depth)).join('')}
    </div>`;
  }
  const CHILD_LABEL = { applied_effects: 'On hit', effects: 'Effects', end_effects: 'When it ends', effect_none: 'Normally', effect_buff: 'With buff', effect_start: 'Before level 3', effect_level3: 'At level 3+' };
  function childBlock(node, path, k, kind, depth) {
    const v = node[k];
    let items = '';
    if (kind === 'one') items = v ? nodeHTML(v, [...path, k], depth + 1) : '';
    else items = (v || []).map((x, i) => kind === 'applied'
      ? `<div class="fx-applied"><select class="fx-ct" data-path="${P([...path, k, i, 'casting_type'])}" data-kind="str" title="How the hit picks its target">${opt(CASTING, x.casting_type || 'Targeting')}</select>${nodeHTML(x.effect, [...path, k, i, 'effect'], depth + 1)}</div>`
      : nodeHTML(x, [...path, k, i], depth + 1)).join('');
    const canAdd = kind !== 'one' || !v;
    return `<div class="fx-kids"><div class="fx-kids-h">${esc(CHILD_LABEL[k] || humanize(k))}</div>${items}
      ${canAdd ? `<select class="fx-add" data-add="${P([...path, k])}" data-addkind="${kind}"><option value="">+ add effect…</option>${ORDER_GROUPS.map(g => `<optgroup label="${g}">${Object.entries(E).filter(([, d]) => d.g === g).map(([t]) => `<option value="${t}">${humanize(t).replace(/([a-z])([A-Z])/g, '$1 $2')}</option>`).join('')}</optgroup>`).join('')}</select>` : ''}</div>`;
  }

  // ------------------------------------------------------------------ main render
  function champLabel(c) { const G = window.TFM2_GAMEDATA; return (c.text && c.text.name) || G.names[c.json.id] || humanize(c.json.id); }
  function render() {
    const root = $('#skillLab'); if (!root) return;
    if (!L.loaded) { root.innerHTML = '<p class="muted" style="padding:20px">Loading…</p>'; load(); return; }
    const keep = $('#slMain') ? $('#slMain').scrollTop : 0;
    const G = window.TFM2_GAMEDATA;
    const all = Object.keys(G.info).concat((G.modChampions || []).map(m => m.id));
    root.innerHTML = `
      <div class="sl-side">
        <div class="sl-mod">
          <label class="fx-f"><span>Folder name</span><input id="slModName" value="${esc(L.modName)}"></label>
          ${L.cur && L.cur.isNew ? `<label class="fx-f"><span>Folder id</span><input id="slModId" value="${esc(L.modId)}"></label>` : ''}
          <div class="muted" style="font-size:12px;margin-top:4px">${L.modsDir ? esc(L.modsDir) + '\\' + esc(L.modId) : 'Game mods folder not found — files will be downloaded.'}</div>
          <button class="btn primary" id="slSave" style="width:100%;margin-top:10px">${anyDirty() ? 'Save ' + (L.mods.filter(m => m.dirty).length > 1 ? L.mods.filter(m => m.dirty).length + ' folders' : 'mod') + ' ●' : 'Save mod'}</button>
        </div>
        <div class="sl-list">${L.mods.map(folderHTML).join('')}
          <button class="sl-addfolder" id="slNewFolder" data-drop="__new__">+ New folder <span class="muted" style="font-weight:400;font-size:12px">(or drop a champion here)</span></button></div>
        <div class="sl-new">
          <select id="slPick"><option value="">Rework a champion…</option>${all.filter(id => !L.mods.some(m => m.champs.some(c => c.json.id === id))).map(id => `<option value="${id}">${esc(G.names[id] || humanize(id))}</option>`).join('')}</select>
          <button class="btn small" id="slNewChamp" style="width:100%;margin-top:6px">+ Brand-new champion</button>
          <select id="slPreset" style="width:100%;margin-top:6px"><option value="">+ New champion from a preset…</option>${Object.entries(window.TFM2_PRESETS || {}).map(([k, p]) => `<option value="${k}">${esc(p.label)}</option>`).join('')}</select>
          ${L.champs.length ? '<button class="btn small danger" id="slClear" style="width:100%;margin-top:6px">Start over (remove all)</button>' : ''}
        </div>
      </div>
      <div class="sl-main" id="slMain">${L.sel ? champHTML(L.sel) : introHTML()}</div>`;
    $('#slMain').scrollTop = keep;
    if (window.TFM2_ART && L.sel) TFM2_ART.animate($('#slMain'));
  }
  function folderHTML(m) {
    const inGame = L.enabled.includes(m.id);
    const on = m.champs.filter(c => !c.disabled).length;
    return `<div class="sl-folder${m === L.cur ? ' cur' : ''}" data-drop="${esc(m.id)}">
      <div class="sl-fhead"><button class="sl-ftoggle" data-folder="${esc(m.id)}" title="Open / close">${m.open ? '▾' : '▸'} <span class="sl-fname">${esc(m.name)}</span><span class="sl-fcount">${on}/${m.champs.length}${m.dirty ? ' ●' : ''}</span></button>
        ${m.isNew ? '<span class="sl-fstate new" title="Not saved yet">new</span>' : `<label class="sl-switch" title="${inGame ? 'On in the game\'s Mod Manager — untick to switch the whole folder off' : 'Off in the game — tick to switch it on (game must be closed)'}"><input type="checkbox" data-modon="${esc(m.id)}" ${inGame ? 'checked' : ''}><span>${inGame ? 'in game' : 'off'}</span></label>`}</div>
      ${m.open ? (m.champs.map(c => champRow(m, c)).join('') || '<p class="muted sl-empty">Empty folder — add champions below.</p>') : ''}
    </div>`;
  }
  function champRow(m, c) {
    const i = m.champs.indexOf(c);
    return `<div class="sl-crow${c.disabled ? ' off' : ''}" draggable="true" data-drag="${esc(m.id)}|${i}" title="Drag onto another folder to move it"><button class="champ-item${L.sel === c ? ' on' : ''}" data-slsel="${esc(m.id)}|${i}">${window.TFM2_ART ? TFM2_ART.icon(c.json.sprite || c.json.id, champLabel(c), '#37d5b3') : `<span class="ico" style="background:#37d5b3">${esc(champLabel(c).slice(0, 2))}</span>`}<span><span class="nm">${esc(champLabel(c))}</span><span class="cat">${c.disabled ? 'switched off' : c.isNew ? 'new champion' : 'rework of ' + esc(c.json.id)}</span></span><span></span></button>
      <label class="sl-switch" title="${c.disabled ? 'Off: the game won\'t load this champion' : 'On: the game loads this champion'}"><input type="checkbox" data-champon="${esc(m.id)}|${i}" ${c.disabled ? '' : 'checked'}></label></div>`;
  }
  function introHTML() {
    return `<div class="sl-intro"><h2>Skill Lab</h2>
      <p>Rebuild any champion's skills from building blocks — projectiles, dashes, teleports, stuns, knock-ups, shields, zones, buffs — or create a brand-new champion.
      Everything is saved as an official Teamfight Manager 2 <strong>mod</strong> in the game's <code>mods</code> folder, so your career save is never touched and you can switch it off any time.</p>
      <ol><li>Pick a champion on the left to rework (it starts from an approximation of its current kit and numbers).</li>
      <li>Change blocks: e.g. switch a <em>Knock up</em> to a <em>Stun</em>, or add <em>Teleport</em> under a projectile's “When it ends”.</li>
      <li>Click <strong>Save mod</strong>, then in the game: title screen → <strong>Mods</strong> → enable “${esc(L.modName)}” → restart the game.</li></ol>
      <p class="muted">While the mod is enabled the rework replaces that champion in every save and in exhibition matches; disabling the mod restores the original.</p></div>`;
  }
  function champHTML(c) {
    const j = c.json; const G = window.TFM2_GAMEDATA;
    const rec = recipes(j.id, 'skill', j.tags);
    return `<div class="sl-head"><h2>${esc(champLabel(c))} <span class="tag">${c.isNew ? 'NEW' : 'REWORK'}</span></h2><span class="spacer"></span>
        <button class="chip-btn" id="slJson">${L.showJson ? 'Hide JSON' : 'View JSON'}</button>
        <button class="chip-btn" id="slReplace">Swap effect type…</button>
        <label class="sl-switch big"><input type="checkbox" id="slChampOn" ${c.disabled ? '' : 'checked'}><span>${c.disabled ? 'Off' : 'On'}</span></label>
        <select id="slMove"><option value="">Move to folder…</option>${L.mods.filter(m => m !== folderOf(c)).map(m => `<option value="${esc(m.id)}">${esc(m.name)}</option>`).join('')}<option value="__new__">+ New folder…</option></select>
        <button class="chip-btn" id="slRemove">Remove from folder</button></div>
      ${c.disabled ? '<div class="sl-offnote">Switched off: after saving, the game no longer loads this champion (the file is kept as <code>.data_champion.off</code>). Careers you already started keep their own copy.</div>' : ''}
      ${L.showJson ? `<textarea class="sl-json" id="slJsonText" spellcheck="false">${esc(JSON.stringify(j, null, 2))}</textarea><button class="btn small" id="slJsonApply">Apply JSON</button>` : ''}
      <div class="sl-block"><div class="sec-head"><h4>Identity</h4></div>
        <div class="fx-fields">
          ${c.isNew ? `<label class="fx-f"><span>Name</span><input data-text="name" value="${esc(c.text.name || '')}"></label>
            <div class="fx-f sl-look"><span>Looks like</span>${window.TFM2_ART ? TFM2_ART.hero(j.sprite || j.id, 56) : ''}
              <select data-sprite="1">${ownSprite(c) ? `<option value="__own" selected>★ Own sprite</option>` : '<option value="__own">★ Own sprite (opens the sprite editor)</option>'}${Object.keys(G.info).map(id => `<option value="${id}" ${j.sprite === 'asset/base/aseprite_resources/champions/' + id ? 'selected' : ''}>${esc(G.names[id] || id)}</option>`).join('')}</select>
              <button class="btn small" id="slEditSprite" title="Draw or change this champion's look, frame by frame">✎ Edit sprite…</button></div>` : `<div class="muted" style="font-size:12.5px">Keeps ${esc(G.names[j.id] || j.id)}'s name, look and id — saves, drafts and patches keep working.</div>`}
          <label class="fx-f"><span>Class</span><select data-top="category">${opt(CATEGORIES, j.category)}</select></label>
          <div class="fx-f fx-tags"><span>Tags</span>${TAGS.map(t => `<label class="fx-bool"><input type="checkbox" data-tag="${t}" ${(j.tags || []).includes(t) ? 'checked' : ''}>${t}</label>`).join('')}</div>
          <label class="fx-f"><span>Passive</span><select data-passive="1"><option value="">None</option>${Object.keys(PASSIVES).map(p => `<option ${j.passive && j.passive.passive_ref === p ? 'selected' : ''}>${p}</option>`).join('')}</select></label>
          ${j.passive ? (PASSIVES[j.passive.passive_ref] || []).map(pk => `<label class="fx-f"><span>${humanize(pk)}</span><input type="number" data-path="${P(['passive', 'params', pk])}" data-kind="num" value="${esc((j.passive.params || {})[pk] || 0)}"></label>`).join('') : ''}
        </div></div>
      <div class="sl-block"><div class="sec-head"><h4>Stats</h4><span class="note">level 1 · per level</span></div>
        <div class="sl-stats">${STATS.map(k => `<label class="fx-f"><span>${humanize(k)}</span><input type="number" data-path="${P(['stat', k])}" data-kind="num" value="${esc(j.stat[k])}"><input type="number" data-path="${P(['growth', k])}" data-kind="num" value="${esc(j.growth[k])}" title="per level"></label>`).join('')}</div></div>
      ${SLOTS.map(([slot, title]) => slotHTML(c, slot, title, rec)).join('')}`;
  }
  function slotHTML(c, slot, title, rec) {
    const a = c.json[slot];
    if (!a) return `<div class="sl-block"><div class="sec-head"><h4>${title}</h4><span class="spacer"></span><button class="chip-btn" data-addslot="${slot}">Add</button></div></div>`;
    const recs = recipes(c.json.id, slot, c.json.tags);
    return `<div class="sl-block" data-slot="${slot}">
      <div class="sec-head"><h4>${title}</h4><span class="spacer"></span>
        <select class="fx-recipe" data-recipe="${slot}"><option value="">Quick recipe…</option>${Object.entries(recs).map(([k, r]) => `<option value="${k}">${esc(r.label)}</option>`).join('')}</select></div>
      <div class="fx-fields sl-action">
        ${numInput([slot], 'cooltime', a.cooltime || 0, 'tick')}${numInput([slot], 'duration', a.duration || 0, 'tick')}${numInput([slot], 'start_timing', a.start_timing || 0, 'tick')}
        ${numInput([slot], 'range', a.range || 0, 'dist')}
        ${(() => { const tags = window.TFM2_ART && TFM2_ART.tags(c.json.sprite || c.json.id); return tags ? `<label class="fx-f"><span>Animation</span><select data-path="${P([slot, 'action_name'])}" data-kind="str">${opt(tags.includes(a.action_name) ? tags : [a.action_name].concat(tags), a.action_name)}</select></label>` : ''; })()}
        <label class="fx-f"><span>Aim</span><select data-path="${P([slot, 'casting_type'])}" data-kind="str">${opt(CASTING, a.casting_type || 'Targeting')}</select></label>
        <label class="fx-f"><span>Targets</span><select data-path="${P([slot, 'casting_target'])}" data-kind="str">${opt(TARGETS, a.casting_target || 'Ally')}</select></label>
        <label class="fx-f"><span>Damage kind</span><select data-path="${P([slot, 'attack_type'])}" data-kind="str">${opt(ATTACK_TYPES, a.attack_type || 'BaseAttack')}</select></label>
        <label class="fx-f fx-bool"><input type="checkbox" data-path="${P([slot, 'cancelable'])}" data-kind="bool" ${a.cancelable ? 'checked' : ''}><span>Cancelable</span></label>
      </div>
      ${slot !== 'attack' ? `<label class="fx-f sl-desc"><span>Tooltip</span><textarea data-desc="${slot}" rows="2" placeholder="${esc(((window.TFM2_GAMEDATA.desc[c.json.id] || {})[slot]) || 'Tooltip text')}">${esc(c.text[slot] || '')}</textarea><span class="muted" style="font-size:11px">${c.auto[slot] !== false ? 'auto-written from the blocks below — edit to write your own' : 'custom text · <a href="#" data-autodesc="' + slot + '">use auto text</a>'}</span></label>` : ''}
      <div class="fx-root">${a.effect ? nodeHTML(a.effect, [slot, 'effect'], 0) : `<select class="fx-add" data-add="${P([slot, 'effect'])}" data-addkind="one"><option value="">+ add effect…</option>${Object.keys(E).map(t => `<option value="${t}">${t}</option>`).join('')}</select>`}</div>
    </div>`;
  }

  // ------------------------------------------------------------------ mutations
  function touched(c) {
    for (const [slot] of SLOTS) if (slot !== 'attack' && c.json[slot] && c.auto[slot] !== false) c.text[slot] = sentence(describe(c.json[slot].effect));
    L.dirty = true;
  }
  function parentOf(path) {
    const last = path[path.length - 1];
    if (typeof last === 'number') return { list: getAt(L.sel.json, path.slice(0, -1)), idx: last, applied: false };
    if (last === 'effect' && typeof path[path.length - 2] === 'number') return { list: getAt(L.sel.json, path.slice(0, -2)), idx: path[path.length - 2], applied: true };
    return { single: true };
  }
  function onChange(e) {
    const c = L.sel; const t = e.target;
    if (t.id === 'slModName') { L.modName = t.value.trim() || 'My custom skills'; L.dirty = true; return render(); }
    if (t.id === 'slModId') {
      const v = t.value.trim().toLowerCase().replace(/[^a-z0-9_]/g, '_');
      if (v.length >= 2 && !L.mods.some(m => m !== L.cur && m.id === v)) { L.modId = v; L.dirty = true; } else L.ctx.toast('That folder id is taken or too short', 'err');
      render(); return;
    }
    if (t.dataset.champon) { const [mid, i] = t.dataset.champon.split('|'); const m = L.mods.find(x => x.id === mid); const ch = m && m.champs[+i]; if (ch) setChampOn(m, ch, t.checked); return; }
    if (t.dataset.modon) return setModOn(t.dataset.modon, t.checked);
    if (t.id === 'slChampOn' && c) return setChampOn(folderOf(c), c, t.checked);
    if (t.id === 'slMove' && t.value && c) return moveChamp(c, t.value);
    if (t.id === 'slPick' && t.value) return addRework(t.value);
    if (t.id === 'slPreset' && t.value) return addPreset(t.value);
    if (!c) return;
    if (t.dataset.type) { const path = JSON.parse(t.dataset.type); setAt(c.json, path, convert(getAt(c.json, path), t.value)); touched(c); return render(); }
    if (t.dataset.add) {
      if (!t.value) return;
      const path = JSON.parse(t.dataset.add), kind = t.dataset.addkind, n = defaultEffect(t.value);
      if (!n.name && E[t.value] && 'name' in E[t.value].f) n.name = `${c.json.id}_${path[0]}_${Math.random().toString(36).slice(2, 6)}`;
      if (kind === 'one') setAt(c.json, path, n);
      else { const list = getAt(c.json, path) || []; list.push(kind === 'applied' ? { effect: n, casting_type: 'Targeting' } : n); setAt(c.json, path, list); }
      touched(c); return render();
    }
    if (t.dataset.recipe) {
      if (!t.value) return;
      const r = recipes(c.json.id, t.dataset.recipe, c.json.tags)[t.value]; const a = c.json[t.dataset.recipe];
      a.effect = clone(r.effect); a.casting_type = r.cast[0]; a.casting_target = r.cast[1]; if (r.range) a.range = r.range;
      touched(c); return render();
    }
    if (t.dataset.addstat) { if (!t.value) return; const path = JSON.parse(t.dataset.addstat); getAt(c.json, path)[t.value] = 10; touched(c); return render(); }
    if (t.dataset.top) { c.json[t.dataset.top] = t.value; L.dirty = true; return; }
    if (t.dataset.tag) { const tags = new Set(c.json.tags || []); t.checked ? tags.add(t.dataset.tag) : tags.delete(t.dataset.tag); c.json.tags = TAGS.filter(x => tags.has(x)); L.dirty = true; return; }
    if (t.dataset.text) { c.text[t.dataset.text] = t.value; L.dirty = true; return render(); }
    if (t.dataset.sprite) {
      if (t.value === '__own') return ownSprite(c) ? undefined : editSprite(c);
      c.json.sprite = 'asset/base/aseprite_resources/champions/' + t.value; c.json.anim_prefix = ''; fitTags(c.json); L.dirty = true; return render();
    }
    if (t.dataset.passive != null && t.matches('[data-passive]')) {
      if (!t.value) delete c.json.passive; else { const params = {}; PASSIVES[t.value].forEach(k => (params[k] = 10)); c.json.passive = { passive_ref: t.value, params }; }
      L.dirty = true; return render();
    }
    if (t.dataset.desc) { c.text[t.dataset.desc] = t.value; c.auto[t.dataset.desc] = false; L.dirty = true; return; }
    if (t.dataset.path) {
      const path = JSON.parse(t.dataset.path); const kind = t.dataset.kind;
      let v;
      if (kind === 'num') { v = Number(t.value); if (!isFinite(v)) return; }
      else if (kind === 'bool') v = t.checked;
      else if (kind === 'shape') v = t.value === 'Circle' ? { Circle: { radius: 10000 } } : t.value === 'Rect' ? { Rect: { width: 30000, height: 16000 } } : t.value === 'DirDot' ? { DirDot: { radius: 8000, range: 700 } } : { Line: { width: 8000, from_x: 0, from_y: 0, to_x: 50000, to_y: 0 } };
      else if (kind === 'aet') v = t.value === 'EnemyAll' ? { EnemyAll: { range: 40000 } } : t.value;
      else if (kind === 'heal') v = t.value === 'AllyAll' ? { AllyAll: { range: 40000 } } : t.value;
      else if (kind === 'apply') v = t.value === 'Forward' ? { Forward: { offset: 24000 } } : t.value;
      else if (kind === 'bufdur') v = t.value === 'Time' ? { Time: { tick: 180 } } : t.value;
      else v = t.value;
      setAt(c.json, path, v); touched(c);
      const structural = ['shape', 'aet', 'heal', 'apply', 'bufdur'].includes(kind);
      return structural ? render() : refreshSummaries(c);
    }
  }
  function refreshSummaries(c) {
    // light update: node summaries + auto tooltips, without rebuilding inputs
    document.querySelectorAll('#slMain .fx[data-node]').forEach(el => {
      const n = getAt(c.json, JSON.parse(el.dataset.node)); const s = el.querySelector(':scope > .fx-head .fx-sum');
      if (n && s) { s.textContent = sentence(describe(n)); s.title = s.textContent; }
    });
    document.querySelectorAll('#slMain .fx-hint').forEach(h => { const inp = h.previousElementSibling; if (inp && inp.value !== undefined) h.textContent = sec(+inp.value || 0); });
    for (const [slot] of SLOTS) { const ta = document.querySelector(`#slMain [data-desc="${slot}"]`); if (ta && c.auto[slot] !== false) ta.value = c.text[slot] || ''; }
    const sv = $('#slSave'); if (sv) sv.textContent = 'Save mod ●';
  }
  function onClick(e) {
    const t = e.target; const c = L.sel;
    const sel = t.closest('[data-slsel]');
    if (sel) { const [mid, i] = sel.dataset.slsel.split('|'); const m = L.mods.find(x => x.id === mid); if (m) { L.cur = m; L.sel = m.champs[+i] || null; L.showJson = false; } return render(); }
    const fold = t.closest('[data-folder]');
    if (fold) { const m = L.mods.find(x => x.id === fold.dataset.folder); if (m) { if (L.cur === m) m.open = !m.open; else { m.open = true; L.cur = m; if (!m.champs.includes(L.sel)) L.sel = null; } } return render(); }
    if (t.id === 'slNewFolder') return addFolder();
    if (t.id === 'slSave') return saveMod();
    if (t.id === 'slNewChamp') return addNew();
    if (t.id === 'slClear') {
      if (!confirm(`Remove every champion from the folder "${L.modName}"? Reworked champions go back to normal after you save and restart the game.`)) return;
      L.removed.push(...L.champs.map(x => x.file)); L.champs = []; L.sel = null; L.dirty = true; return render();
    }
    if (!c) return;
    if (t.id === 'slJson') { L.showJson = !L.showJson; return render(); }
    if (t.id === 'slJsonApply') { try { const j = JSON.parse($('#slJsonText').value); if (!j.id) throw new Error('missing "id"'); c.json = j; touched(c); render(); L.ctx.toast('JSON applied', 'ok'); } catch (err) { L.ctx.toast('Invalid JSON: ' + err.message, 'err', 6000); } return; }
    if (t.id === 'slRemove') { if (!confirm(`Remove ${champLabel(c)} from the mod? (The original champion comes back.)`)) return; L.removed.push(c.file); L.champs = L.champs.filter(x => x !== c); L.sel = L.champs[0] || null; L.dirty = true; return render(); }
    if (t.id === 'slReplace') return swapTypes(c);
    if (t.id === 'slEditSprite') return editSprite(c);
    if (t.dataset.addslot) { c.json[t.dataset.addslot] = { action_name: t.dataset.addslot, duration: 30, cooltime: 1800, start_timing: 15, range: 40000, casting_type: 'None', casting_target: 'Enemy', attack_type: 'Skill', description: `#asset/base/text/champion?description.${c.json.id}.${t.dataset.addslot}`, effect: defaultEffect('RangeEffect') }; touched(c); return render(); }
    if (t.dataset.autodesc) { e.preventDefault(); c.auto[t.dataset.autodesc] = true; touched(c); return render(); }
    if (t.dataset.del) { const path = JSON.parse(t.dataset.del); delete getAt(c.json, path.slice(0, -1))[path[path.length - 1]]; touched(c); return render(); }
    const op = t.closest('[data-op]');
    if (op) {
      const path = JSON.parse(op.closest('[data-node]').dataset.node); const node = getAt(c.json, path); const par = parentOf(path);
      const k = op.dataset.op;
      if (k === 'wrap') { setAt(c.json, path, { type: 'Combine', effects: [node] }); }
      else if (par.single) {
        if (k === 'del') { const last = path[path.length - 1]; const holder = getAt(c.json, path.slice(0, -1)); if (last === 'effect') holder.effect = null; else delete holder[last]; }
        else if (k === 'dup') { setAt(c.json, path, { type: 'Combine', effects: [node, clone(node)] }); }
      } else {
        const list = par.list, i = par.idx;
        if (k === 'del') list.splice(i, 1);
        else if (k === 'dup') list.splice(i + 1, 0, clone(list[i]));
        else if (k === 'up' && i > 0) [list[i - 1], list[i]] = [list[i], list[i - 1]];
        else if (k === 'down' && i < list.length - 1) [list[i + 1], list[i]] = [list[i], list[i + 1]];
      }
      touched(c); return render();
    }
  }
  function swapTypes(c) {
    const found = new Set(); (function walk(o) { if (Array.isArray(o)) o.forEach(walk); else if (o && typeof o === 'object') { if (typeof o.type === 'string' && E[o.type]) found.add(o.type); Object.values(o).forEach(walk); } })(c.json);
    const from = prompt(`Swap every effect of this type (${[...found].join(', ')}):`, [...found].find(x => x === 'Airborne') || [...found][0] || '');
    if (!from || !found.has(from)) return;
    const to = prompt(`…into which type? (e.g. Stun, Airborne, Bind, Fear, Charm, Taunt, Knockback, Teleport…)`, 'Stun');
    if (!to || !E[to]) { if (to) L.ctx.toast('Unknown effect type: ' + to, 'err'); return; }
    let n = 0;
    (function walk(o) {
      if (Array.isArray(o)) { o.forEach((x, i) => { if (x && x.type === from) { o[i] = convert(x, to); n++; } walk(o[i]); }); }
      else if (o && typeof o === 'object') for (const k of Object.keys(o)) { const v = o[k]; if (v && typeof v === 'object' && !Array.isArray(v) && v.type === from && k !== 'buff_state') { o[k] = convert(v, to); n++; } walk(o[k]); }
    })(c.json);
    touched(c); render(); L.ctx.toast(`Swapped ${n} × ${from} → ${to}`, 'ok');
  }
  function addRework(id) {
    const G = window.TFM2_GAMEDATA;
    const isData = (G.modChampions || []).some(m => m.id === id);
    const json = isData ? dataTemplate(id) : nativeTemplate(id);
    fitTags(json);
    const c = { json, text: {}, auto: {}, isNew: false, file: `champion/${id}.data_champion` };
    const d = G.desc[id] || {};
    for (const [slot] of SLOTS) if (slot !== 'attack' && json[slot]) {
      if (isData) { c.text[slot] = ''; c.auto[slot] = false; } else c.auto[slot] = true;
      json[slot].description = `#asset/base/text/champion?description.${id}.${slot === 'skill' && isData ? 'skill' : slot}`;
    }
    touched(c);
    L.champs.push(c); L.sel = c; render();
    if (!isData) L.ctx.toast(`${G.names[id] || id}: starting kit rebuilt from its numbers — its original special mechanics are replaced by standard blocks you can now change.`, '', 8000);
  }
  // action_name is the animation the champion plays; it has to exist on the sprite or the game has nothing to show
  const TAG_PREFS = { attack: ['attack'], skill: ['skill1', 'skill', 'skill_pre', 'skill_attack', 'skill_dash'], skill2: ['skill2', 'skill2_attack', 'skill2_dash', 'skill2_pre'], ult: ['ult', 'ult_pre', 'ult_attack', 'ult_dash', 'ult_start'] };
  const isFx = t => /projectile|effect|target|_hit$|^hit$|dead|idle|run/.test(t);
  function fitTags(j) {
    const tags = window.TFM2_ART && TFM2_ART.tags(j.sprite || j.id); if (!tags) return 0;
    let n = 0;
    for (const [slot] of SLOTS) {
      const a = j[slot]; if (!a || tags.includes(a.action_name)) continue;
      const pick = TAG_PREFS[slot].find(t => tags.includes(t)) || tags.find(t => !isFx(t) && t.startsWith(slot === 'skill' ? 'skill' : slot) && (slot !== 'skill' || !t.startsWith('skill2')));
      if (pick) { a.action_name = pick; n++; }
    }
    return n;
  }
  // ------------------------------------------------------------------ own sprites (sprite editor)
  const spriteKey = (modId, id) => `asset/${modId}/champions/${id}`;
  /** Does the champion use a sprite of its own folder (mods/<folder>/champions/<id>)? */
  function ownSprite(c) { const f = folderOf(c); return !!(f && c.json.sprite === spriteKey(f.id, c.json.id)); }
  /** Put a sheet + fanim into the champion's files and point its sprite at them. */
  function setSpriteFiles(c, modId, pngBase64, fanim) {
    c.assets = Object.assign({}, c.assets || {}, {
      [`champions/${c.json.id}#sheet.png`]: { base64: pngBase64 },
      [`champions/${c.json.id}#anim.fanim`]: JSON.stringify(fanim, null, 1),
    });
    c.json.sprite = spriteKey(modId, c.json.id); c.json.anim_prefix = '';
    registerArt(c.json.sprite, pngBase64, fanim);
  }
  function registerArt(key, pngBase64, fanim) {
    if (!window.TFM2_ART) return;
    const anims = fanim.anims || {}; const idle = anims.idle || anims.run || Object.values(anims)[0];
    let sw = 0, sh = 0; for (const a of Object.values(anims)) for (const f of a.frames || []) { sw = Math.max(sw, f.data.x + f.data.w); sh = Math.max(sh, f.data.y + f.data.h); }
    TFM2_ART.register(key, { url: 'data:image/png;base64,' + pngBase64, sw, sh, tags: Object.keys(anims).sort(),
      idle: ((idle && idle.frames) || []).slice(0, 12).map(f => ({ x: f.data.x, y: f.data.y, w: f.data.w, h: f.data.h, d: Math.round((f.duration || 0.1) * 1000) })) });
  }
  const b64text = b64 => new TextDecoder().decode(Uint8Array.from(atob(b64), ch => ch.charCodeAt(0)));
  async function loadSpriteOf(c) {
    const j = c.json; const f = folderOf(c);
    const mk = src => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error('sheet not readable')); i.src = src; });
    const a = c.assets || {};
    const pngRel = `champions/${j.id}#sheet.png`, animRel = `champions/${j.id}#anim.fanim`;
    if (ownSprite(c) && a[pngRel] && a[animRel]) return { img: await mk('data:image/png;base64,' + a[pngRel].base64), fanim: JSON.parse(a[animRel]) };
    const own = /^asset\/([a-z0-9_]+)\/champions\/([A-Za-z0-9_\-]+)$/.exec(j.sprite || '');
    if (own && own[1] !== 'base') {
      const get = async rel => { const r = await fetch(`/api/mod-file?id=${encodeURIComponent(own[1])}&path=${encodeURIComponent(rel)}`); if (!r.ok) throw new Error(rel + ' not found'); return (await r.json()).base64; };
      const png = await get(`champions/${own[2]}#sheet.png`), anim = b64text(await get(`champions/${own[2]}#anim.fanim`));
      return { img: await mk('data:image/png;base64,' + png), fanim: JSON.parse(anim) };
    }
    if (/^asset\/base\/aseprite_resources\/champions\//.test(j.sprite || '')) {
      const r = await fetch('/api/base-anim?key=' + encodeURIComponent(j.sprite)); if (!r.ok) return null;
      return { img: await mk('/api/asset?key=' + encodeURIComponent(j.sprite + '#sheet')), fanim: await r.json() };
    }
    return null;
  }
  function editSprite(c) {
    if (!window.TFM2_SPRITE_EDITOR) return L.ctx.toast('The sprite editor did not load (sprites.js).', 'err');
    const f = folderOf(c); if (!f) return;
    const required = [...new Set(SLOTS.map(([s]) => c.json[s] && c.json[s].action_name).filter(Boolean))];
    const starters = {};
    for (const [k, s] of Object.entries(window.TFM2_SPRITES || {})) starters[k] = s;
    TFM2_SPRITE_EDITOR.open({
      title: `${champLabel(c)} — sprite`, required, starters, fileBase: c.json.id, toast: L.ctx.toast,
      load: () => loadSpriteOf(c),
      onApply: out => {
        setSpriteFiles(c, f.id, out.png, out.fanim); fitTags(c.json); touched(c); f.dirty = true; render();
        L.ctx.toast(`${champLabel(c)} now uses its own sprite. Save the folder, then restart the game.`, 'ok', 8000);
      },
    });
    render();
  }
  function addPreset(key) {
    const p = (window.TFM2_PRESETS || {})[key]; if (!p) return;
    // presets that belong in a folder of their own (DIO → JoJo, …) go there; the folder is created if needed
    if (p.folder && !(L.mods.some(m => m.champs.some(c => c.json.id.endsWith('_' + p.slug))))) {
      let m = L.mods.find(x => x.id === p.folder[0]);
      if (!m) { m = newFolder(p.folder[0], p.folder[1]); L.mods.push(m); }
      m.open = true; L.cur = m;
    }
    const id = `${L.modId}_${p.slug}`;
    const other = L.mods.find(m => m.champs.some(c => c.json.id === id || (c.json.id.endsWith('_' + p.slug) && c.isNew)));
    if (other) { L.cur = other; L.sel = other.champs.find(c => c.json.id === id || c.json.id.endsWith('_' + p.slug)); render(); return L.ctx.toast(`${p.name} is already in the folder "${other.name}"`, ''); }
    const built = p.build(id, { modId: L.modId }); fitTags(built.json);
    const c = { json: built.json, text: built.text, auto: { skill: false, skill2: false, ult: false }, isNew: true, file: `champion/${id}.data_champion` };
    const vfx = built.vfx && window.TFM2_VFX && window.TFM2_VFX[built.vfx];
    if (vfx) c.assets = { [`vfx/${built.vfx}#sheet.png`]: { base64: vfx.png }, [`vfx/${built.vfx}#anim.fanim`]: JSON.stringify(vfx.fanim) };
    const spr = built.sprite && window.TFM2_SPRITES && window.TFM2_SPRITES[built.sprite];
    if (spr) { setSpriteFiles(c, L.modId, spr.png, spr.fanim); }
    else if (built.spriteFallback && /^asset\/(?!base\/)/.test(built.json.sprite || '')) built.json.sprite = built.spriteFallback;   // sprite data missing: keep a base look
    // extra bodies the preset shows as effects (Frieren's Fern and Stark): champions/<name>#sheet.png + #anim.fanim
    for (const [name, key] of Object.entries(built.extraSprites || {})) {
      const x = window.TFM2_SPRITES && window.TFM2_SPRITES[key]; if (!x) continue;
      c.assets = Object.assign({}, c.assets || {}, { [`champions/${name}#sheet.png`]: { base64: x.png }, [`champions/${name}#anim.fanim`]: JSON.stringify(x.fanim) });
    }
    L.removed = L.removed.filter(f => f !== c.file);
    L.dirty = true; L.champs.push(c); L.sel = c; render();
    L.ctx.toast(`${p.name} added. Click Save mod, then restart the game.`, 'ok', 6000);
  }
  function addNew() {
    const name = prompt('Name of the new champion:', 'Storm Knight'); if (!name) return;
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '') || 'champion';
    const id = `${L.modId}_${slug}`;
    if (L.mods.some(m => m.champs.some(c => c.json.id === id))) return L.ctx.toast('That champion already exists', 'err');
    const c = { json: blankTemplate(id, name), text: { name }, auto: {}, isNew: true, file: `champion/${id}.data_champion` }; fitTags(c.json);
    for (const [slot] of SLOTS) if (slot !== 'attack') c.auto[slot] = true;
    touched(c); L.champs.push(c); L.sel = c; render();
  }

  // ------------------------------------------------------------------ folders
  function addFolder() {
    const name = prompt('Name of the new folder (it becomes its own mod, switchable in the game\'s Mod Manager):', 'My champions 2'); if (!name) return;
    let id = 'tfm2_' + (name.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '') || 'folder');
    id = id.slice(0, 36); let n = 2; const base = id;
    while (L.mods.some(m => m.id === id)) id = `${base}_${n++}`;
    const m = newFolder(id, name.trim()); L.mods.push(m); L.cur = m; L.sel = null; render();
    L.ctx.toast(`Folder "${m.name}" created. Add or drag champions into it, then Save.`, 'ok', 6000);
    return m;
  }
  function setChampOn(m, c, on) {
    if (!m || !c) return;
    const before = c.file; c.disabled = !on; c.file = champFile(c);
    if (before && before !== c.file) m.removed.push(before);
    m.dirty = true; render();
    L.ctx.toast(`${champLabel(c)} ${on ? 'switched on' : 'switched off'} — Save, then restart the game.`, '', 5000);
  }
  async function setModOn(id, on) {
    const m = L.mods.find(x => x.id === id);
    if (m && m.dirty && on) L.ctx.toast('Tip: save the folder too, or the game loads the old version.', '', 6000);
    try {
      const res = await fetch('/api/mod-enabled?id=' + encodeURIComponent(id) + '&on=' + (on ? 1 : 0), { method: 'POST' });
      const j = await res.json(); if (!res.ok) throw new Error(j.error || res.statusText);
      L.enabled = j.enabled; render();
      L.ctx.toast(`"${m ? m.name : id}" ${on ? 'switched on' : 'switched off'} in the game. It applies the next time you start the game.`, 'ok', 6000);
    } catch (e) { render(); L.ctx.toast('Could not change the game\'s mod list: ' + e.message, 'err', 8000); }
  }
  async function moveChamp(c, toId) {
    if (toId === '__new__') { const keep = L.sel; const m = addFolder(); L.sel = keep; if (!m) return render(); toId = m.id; }
    const from = folderOf(c), to = L.mods.find(m => m.id === toId);
    if (!from || !to || from === to) return;
    if (to.champs.some(x => x.json.id === c.json.id)) return L.ctx.toast('That folder already has this champion', 'err');
    // custom VFX live inside the mod folder (asset/<folder id>/vfx/<sheet>), so they move along
    let text = JSON.stringify(c.json);
    const sheets = [...new Set([...text.matchAll(new RegExp(`asset/${from.id}/(vfx|champions)/([A-Za-z0-9_\\-]+)`, 'g'))].map(x => x[1] + '/' + x[2]))];
    const assets = Object.assign({}, c.assets || {});
    for (const sh of sheets) for (const rel of [`${sh}#sheet.png`, `${sh}#anim.fanim`]) {
      if (assets[rel]) continue;
      try {
        const r = await fetch(`/api/mod-file?id=${encodeURIComponent(from.id)}&path=${encodeURIComponent(rel)}`);
        if (!r.ok) continue; const j = await r.json();
        assets[rel] = rel.endsWith('.png') ? { base64: j.base64 } : new TextDecoder().decode(Uint8Array.from(atob(j.base64), ch => ch.charCodeAt(0)));
      } catch (e) { /* no server: the VFX stay in the old folder */ }
    }
    text = text.split(`asset/${from.id}/`).join(`asset/${to.id}/`);
    c.json = JSON.parse(text);
    if (Object.keys(assets).length) c.assets = assets;
    if (c.file) from.removed.push(c.file);
    from.champs = from.champs.filter(x => x !== c); from.dirty = true;
    c.file = champFile(c); to.removed = to.removed.filter(f => f !== c.file);
    to.champs.push(c); to.dirty = true; to.open = true; L.cur = to; L.sel = c; render();
    L.ctx.toast(`${champLabel(c)} moved to "${to.name}". Save to apply${L.enabled.includes(to.id) ? '' : ' — and switch that folder on in the game'}.`, 'ok', 8000);
  }

  // ------------------------------------------------------------------ load & save
  async function load() {
    L.loaded = true;
    if (window.TFM2_ART) await TFM2_ART.load();
    const curId = L.cur && L.cur.id;
    L.mods = [];
    if (L.ctx.server()) {
      try {
        const j = await (await fetch('/api/mods', { cache: 'no-store' })).json();
        L.modsDir = j.gameDirExists ? j.dir : ''; L.gameRunning = !!j.gameRunning;
        L.enabled = (j.enabled || []).map(x => (typeof x === 'string' ? x : x.mod_id || x.id || ''));
        const mine = j.mods.filter(m => m.editable && m.info && m.info.author === 'TFM2 Database Editor' && m.info.mod_type !== 'native' && !m.info.contains_code);
        for (const md of mine) {
          const txt = (md.text && md.text.en && md.text.en.description) || {};
          const f = { id: md.id, name: md.info.name || md.id, version: md.info.version || '0.1.0', diskVersion: md.info.version || null, removed: [], dirty: false, isNew: false, open: true };
          f.champs = md.champions.filter(x => x.json).map(x => {
            const t = txt[x.json.id] || {};
            const isNew = !window.TFM2_GAMEDATA.info[x.json.id] && !(window.TFM2_GAMEDATA.modChampions || []).some(m => m.id === x.json.id);
            if (fitTags(x.json)) f.dirty = true;
            return { json: x.json, file: x.file, disabled: !!x.disabled, isNew, text: { name: t.name, skill: t.skill, skill2: t.skill2, ult: t.ult }, auto: { skill: false, skill2: false, ult: false } };
          });
          L.mods.push(f);
        }
      } catch (e) { console.warn(e); }
    } else L.modsDir = '';
    if (!L.mods.length) L.mods.push(newFolder('tfm2_custom', 'My custom skills'));
    L.cur = L.mods.find(m => m.id === curId) || L.mods.find(m => m.id === 'tfm2_custom') || L.mods[0];
    render();
  }
  function bumpVersion(v) { const p = String(v || '0.1.0').split('.').map(n => parseInt(n, 10) || 0); while (p.length < 3) p.push(0); p[2]++; return p.join('.'); }
  function buildFiles() {
    const files = {};
    const today = new Date().toISOString().slice(0, 10);
    L.modVersion = bumpVersion(L.modVersion);
    files['mod.mod_info'] = JSON.stringify({ name: L.modName, author: 'TFM2 Database Editor', version: L.modVersion, description: 'Custom champion skills made with the TFM2 Database Editor Skill Lab: ' + (L.champs.filter(c => !c.disabled).map(champLabel).join(', ') || 'none switched on'), last_updated: today, dependencies: [{ mod_id: 'base', version: '>=0.4.14' }] }, null, 2);
    // asset overrides: the champion text, plus any a preset in this folder needs (e.g. Frieren's Stark replaces the summon sprite)
    const overrides = { 'asset/base/text/champion': { remapping: `asset/${L.modId}/text/champion`, type: 'merge' } };
    for (const p of Object.values(window.TFM2_PRESETS || {})) {
      if (p.overrides && L.champs.some(c => !c.disabled && c.json.id.endsWith('_' + p.slug))) Object.assign(overrides, p.overrides(L.modId));
    }
    files['mod.override_info'] = JSON.stringify(overrides, null, 2);
    const desc = {};
    for (const c of L.champs) {
      const d = {}; if (c.isNew && c.text.name) d.name = c.text.name;
      for (const [slot] of SLOTS) if (slot !== 'attack' && c.json[slot]) {
        const key = (c.json[slot].description || '').split('?description.' + c.json.id + '.')[1] || slot;
        if (c.text[slot]) d[key] = c.text[slot];
      }
      desc[c.json.id] = d;
      const other = `champion/${c.json.id}.data_champion${c.disabled ? '' : '.off'}`;
      if (!L.removed.includes(other)) L.removed.push(other);   // drop the file with the other on/off state
      c.file = champFile(c);
      files[c.file] = JSON.stringify(c.json, null, 2);
      if (c.assets) Object.assign(files, c.assets);   // VFX sheets that come with a preset
    }
    const i18n = { en: { description: desc } };
    files['text/champion.i18n'] = JSON.stringify(i18n, null, 2);
    return files;
  }
  function validate() {
    const problems = [];
    for (const c of L.champs) {
      for (const s of ['attack', 'skill', 'skill2']) if (!c.json[s]) problems.push(`${champLabel(c)}: ${s} is required`);
      const tags = window.TFM2_ART && TFM2_ART.tags(c.json.sprite || c.json.id);
      if (tags) for (const [s] of SLOTS) if (c.json[s] && !tags.includes(c.json[s].action_name)) problems.push(`${champLabel(c)}: ${s} plays animation “${c.json[s].action_name}”, which this sprite doesn't have (it has: ${tags.filter(t => !isFx(t)).join(', ')})`);
      (function walk(o, where) {
        if (Array.isArray(o)) o.forEach((x, i) => walk(x, where));
        else if (o && typeof o === 'object') {
          if ('effect' in o && o.effect === null && !('casting_type' in o && Object.keys(o).length === 2)) { /* empty slot is allowed */ }
          if (o.type && E[o.type]) for (const [k, fd] of Object.entries(E[o.type].f)) if (fd.kind === 'one' && !o[k]) problems.push(`${champLabel(c)}: ${o.type} needs an effect in “${CHILD_LABEL[k] || k}”`);
          // formats the game rejects outright (the whole champion then fails to load)
          if (o.type === 'RangePeriodProjectile' && (o.end_effects || []).some(x => x && !x.effect)) problems.push(`${champLabel(c)}: RangePeriodProjectile “${o.name || ''}”: its “When it ends” entries must be {effect, casting_type} like its hits — the game won't load this champion`);
          if (o.type && o.type !== 'RushMoveToBack' && Array.isArray(o.applied_effects) && o.applied_effects.some(x => x && !x.effect)) problems.push(`${champLabel(c)}: ${o.type}: every “On hit” entry needs an effect — the game won't load this champion`);
          Object.values(o).forEach(v => walk(v, where));
        }
      })(c.json);
    }
    return problems;
  }
  async function saveMod() {
    const todo = L.mods.filter(m => m.dirty && (m.champs.length || m.removed.length));
    if (!todo.length) return L.ctx.toast(L.mods.some(m => m.dirty) ? 'Add a champion first.' : 'Nothing to save.', L.mods.some(m => m.dirty) ? 'err' : '');
    const keepCur = L.cur, keepSel = L.sel; const saved = [];
    for (const m of todo) { L.cur = m; if (await saveOne()) saved.push(m); }
    L.cur = keepCur; L.sel = keepSel; render();
    if (saved.length) {
      const off = saved.filter(m => !L.enabled.includes(m.id));
      L.ctx.toast(`Saved ${saved.map(m => '"' + m.name + '"').join(', ')}. ` + (off.length ? `Switch on ${off.map(m => '"' + m.name + '"').join(', ')} (the tick next to the folder, or the game's Mod Manager), then restart the game.` : 'Restart the game to load the changes.'), 'ok', 10000);
    }
  }
  async function saveOne() {
    const problems = validate();
    if (problems.length && !confirm(`Folder "${L.modName}" — please check:\n\n` + problems.slice(0, 8).join('\n') + '\n\nSave anyway?')) return false;
    // Don't overwrite a mod that was changed on disk after this page loaded it (e.g. updated by Claude or another tab)
    if (L.modsDir) {
      try {
        const j = await (await fetch('/api/mods', { cache: 'no-store' })).json();
        const disk = (j.mods || []).find(m => m.id === L.modId);
        if (disk && disk.info && L.diskVersion && disk.info.version !== L.diskVersion) {
          if (confirm(`"${L.modName}" was changed on disk after you opened the Skill Lab (now version ${disk.info.version}, you loaded ${L.diskVersion}).\n\nOK = load the newer version (your unsaved edits here are dropped)\nCancel = overwrite it with what you see here`)) {
            await reloadFromDisk(); L.ctx.toast('Loaded the newer version of the mod.', 'ok', 6000); return false;
          }
        }
      } catch (e) { /* can't check — save as before */ }
    }
    const files = buildFiles();
    const keep = new Set(Object.keys(files));
    const remove = L.removed.filter(f => f && !keep.has(f));
    if (!L.modsDir) {
      for (const [name, content] of Object.entries(files)) {
        const data = content && content.base64 ? Uint8Array.from(atob(content.base64), ch => ch.charCodeAt(0)) : content;
        const url = URL.createObjectURL(new Blob([data], { type: content && content.base64 ? 'image/png' : 'application/json' }));
        const a = document.createElement('a'); a.href = url; a.download = name.replace(/\//g, '__'); a.click(); setTimeout(() => URL.revokeObjectURL(url), 20000);
      }
      L.ctx.toast(`Downloaded the mod files. Put them in Teamfight Manager 2\\mods\\${L.modId}\\ (the "__" in names are sub-folders).`, 'ok', 10000); return false;
    }
    try {
      const res = await fetch('/api/mod?id=' + encodeURIComponent(L.modId), { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ files, remove }) });
      const j = await res.json(); if (!res.ok) throw new Error(j.error || res.statusText);
      L.dirty = false; L.removed = []; L.diskVersion = L.modVersion; L.cur.isNew = false;
      for (const c of L.champs) delete c.assets;   // written now; they live in the folder
      return true;
    } catch (e) { L.ctx.toast(`Could not save "${L.modName}": ` + e.message, 'err', 8000); return false; }
  }

  async function reloadFromDisk() {
    const selId = L.sel && L.sel.json.id;
    L.sel = null; L.loaded = true;
    await load();
    for (const m of L.mods) { const hit = m.champs.find(c => c.json.id === selId); if (hit) { L.cur = m; L.sel = hit; } }
    render();
  }
  window.TFM2Skills = {
    // called when the Skill Lab tab is opened: pick up changes made on disk since last time (unless there are unsaved edits)
    show() { if (!L.loaded || anyDirty()) return render(); reloadFromDisk(); },
    init(ctx) {
      L.ctx = ctx;
      const root = $('#skillLab');
      root.addEventListener('change', onChange);
      root.addEventListener('click', onClick);
      // drag a champion row onto any folder to move it there
      let drag = null;
      root.addEventListener('dragstart', e => {
        const row = e.target.closest && e.target.closest('[data-drag]'); if (!row) return;
        const [mid, i] = row.dataset.drag.split('|'); const m = L.mods.find(x => x.id === mid);
        drag = m && m.champs[+i]; if (!drag) return;
        e.dataTransfer.effectAllowed = 'move'; e.dataTransfer.setData('text/plain', champLabel(drag));
      });
      const target = e => e.target.closest && e.target.closest('[data-drop]');
      root.addEventListener('dragover', e => {
        const f = drag && target(e); if (!f) return;
        e.preventDefault(); e.dataTransfer.dropEffect = 'move';
        root.querySelectorAll('.drop').forEach(x => x !== f && x.classList.remove('drop')); f.classList.add('drop');
      });
      root.addEventListener('dragleave', e => { const f = target(e); if (f && !f.contains(e.relatedTarget)) f.classList.remove('drop'); });
      root.addEventListener('drop', e => {
        const f = target(e); const c = drag; drag = null;
        root.querySelectorAll('.drop').forEach(x => x.classList.remove('drop'));
        if (!f || !c) return; e.preventDefault();
        if (folderOf(c) && folderOf(c).id === f.dataset.drop) return;
        moveChamp(c, f.dataset.drop);
      });
      root.addEventListener('dragend', () => { drag = null; root.querySelectorAll('.drop').forEach(x => x.classList.remove('drop')); });
      root.addEventListener('input', e => { const t = e.target; if (t.dataset.desc && L.sel) { L.sel.text[t.dataset.desc] = t.value; L.sel.auto[t.dataset.desc] = false; L.dirty = true; } });
    },
    render,
    isDirty: () => anyDirty(),
    _debug: L, _schema: E, _describe: describe,
  };
})();
