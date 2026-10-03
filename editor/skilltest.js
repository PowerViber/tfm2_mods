/* Skill Test tab: a small arena to try a champion by hand, with training dummies.
 *
 *   Right-click ground = move · right-click a dummy = attack it · Q / W / E = Ability 1 / Ability 2 / Ultimate
 *   R F T G Y = extra actives (Scribble: weave Pencil / Eraser / Paint / Gadget / Page), 1-5 also weave.
 *   Scribble: E = Invoke, Q = flick the dots away, W = weave + invoke the last recipe again.
 *
 * Champions come from the mod folders on disk (Skill Lab) or from the presets. Their data (the .data_champion
 * effects) is played by a simplified interpreter: damage, healing, shields, crowd control, projectiles, areas,
 * buffs, delays and the view bindings (sprites, effects, projectile and buff visuals). Native passives are not
 * simulated, except Scribble's, whose 35 spells are mirrored here from scribble.rs (same numbers).
 * Also here: the spell gallery (every Scribble spell cast on dummies, one by one) and the Scribble memory page
 * (mastery per athlete and the learned meta, with a reset).
 */
(function () {
  'use strict';
  const TPS = 60, UPX = 950;                       // ticks per second; world units per art pixel
  const AW = 330, AH = 210;                        // arena size in art pixels
  const WW = AW * UPX, WH = AH * UPX;              // ... in world units
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const d2 = (a, b) => (a.x - b.x) ** 2 + (a.y - b.y) ** 2;
  const dist = (a, b) => Math.sqrt(d2(a, b));
  const norm = (dx, dy) => { const l = Math.hypot(dx, dy); return l < 1e-9 ? [1, 0] : [dx / l, dy / l]; };
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const secs = t => (t % 60 ? (t / 60).toFixed(2).replace(/0+$/, '').replace(/\.$/, '') : String(t / 60)) + 's';

  // ------------------------------------------------------------------ Scribble numbers (mirror of scribble.rs)
  // round 72: 8 ranks; 7 = Top 10 (the ten athletes with the most points, 300+ each, ranked #1-#10)
  const RANK_NAMES = ['Novice', 'Apprentice', 'Adept', 'Expert', 'Master', 'Grandmaster', 'Archmage', 'Top 10'];
  const RANK_GAMES = [0, 5, 15, 30, 60, 100, 150];
  const TOP = 7, TOP_POINTS = 300, TOP_SIZE = 10;
  // every rank can try every spell; dots past the comfort tier are overreaches (OVERREACH%, +6 a dot, max 98)
  const COMFORT_TIER = [2, 3, 3, 4, 5, 5, 6, 6];
  const OVERREACH = [80, 65, 50, 40, 30, 20, 0, 0];
  // dots a second (x100); the Top 10 go from 11 (#10) to 15 (#1)
  const CPS100 = [250, 325, 400, 500, 650, 800, 950, 1100];
  const MISFIRE = [16, 11, 8, 5, 3, 1, 0, 0];
  const NOTICE = [25, 45, 65, 85, 95, 100, 100, 100];
  const INVOKE_T = [30, 27, 24, 21, 18, 15, 12, 10];
  const cps100 = (r, pos) => r === TOP ? 1500 - (Math.min(TOP_SIZE, Math.max(1, pos || TOP_SIZE)) - 1) * 400 / 9 : CPS100[r];
  const weaveInterval = (r, pos) => 6000 / cps100(r, pos);   // ticks a dot (fractional)
  const slipPct = (r, dot) => dot <= COMFORT_TIER[r] ? MISFIRE[r] : Math.min(98, OVERREACH[r] + 6 * (dot - COMFORT_TIER[r] - 1));
  const buildChance = (r, len) => { let p = 1; for (let d = 1; d <= len; d++) p *= 1 - slipPct(r, d) / 100; return p; };
  const badgeBuff = (r, pos) => r === TOP ? 'scr_top' + (pos || TOP_SIZE) : 'scr_rank' + r;
  const skinOf = r => ({ 5: 0, 6: 1, 7: 2 })[r];
  // the Top 10 out of the memory: 300+ points, by points, then games, then wins, then athlete id
  const topTen = games => Object.keys(games).filter(a => +a < 1000000 && games[a].points >= TOP_POINTS)
    .sort((x, y) => (games[y].points - games[x].points) || (games[y].games - games[x].games) || (games[y].wins - games[x].wins) || (+x - +y)).slice(0, TOP_SIZE);
  const rankOf = g => { let r = 0; RANK_GAMES.forEach((x, i) => { if (g >= x) r = i; }); return r; };
  const ELEMENTS = ['', 'Pencil', 'Eraser', 'Paint', 'Gadget', 'Page'];
  const EL_COL = ['', '#f2c94c', '#f497b6', '#5aa9ff', '#9aa7b4', '#f4f1e6'];
  // [tag, kind, range] per spell, in SPELLS order; kind: e = enemy target, s = around self, p = point, d = direction
  const SPELL_FX = [
    ['poke', 'e', 60000], ['smudge', 'e', 60000], ['splat', 'e', 60000], ['honk', 's', 30000], ['cut', 'e', 70000],
    ['door', 'd', 35000], ['cutout', 's', 0], ['erase_self', 's', 0], ['bucket', 's', 0], ['peel', 'p', 60000], ['chicken', 'e', 40000],
    ['key', 's', 0], ['bubble', 'e', 60000],
    ['phone', 'e', 45000], ['mallet', 'e', 28000], ['anvil', 'e', 60000], ['glove', 'e', 90000], ['erase_legs', 'e', 60000], ['pie', 'e', 50000],
    ['present', 'e', 55000], ['floor', 'p', 60000],
    ['hole', 'd', 120000], ['panorama', 'e', 70000], ['brawl', 'e', 30000], ['redraw', 's', 0], ['stamp', 's', 35000], ['wall_draw', 'p', 70000],
    ['rewind', 's', 70000], ['piano', 'e', 70000], ['ink_wave', 'e', 120000], ['chase', 's', 0], ['laugh', 's', 90000],
    ['page', 's', 0], ['pause', 's', 0], ['sketch_in', 's', 0],
  ];

  // ------------------------------------------------------------------ state
  const T = {
    wired: false, canvas: null, ctx: null, zoom: 3, champs: [], sel: null, json: null, text: null,
    sheets: {}, sprite: null, ents: [], projs: [], fxs: [], texts: [], later: [], walls: [], log: [], tick: 0, running: true,
    mouse: { x: WW / 2, y: WH / 2 }, keys: {}, opts: { level: 9, rank: 6, topPos: null, slips: false, cooldowns: true, dummyHp: 2500, dummyDef: 30, dummyMr: 30, fightBack: false, strafe: false },
    gallery: null, view: 'arena', mem: null, raf: 0, acc: 0, last: 0,
  };
  let NEXT_ID = 1;

  // ------------------------------------------------------------------ assets
  const img = src => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error('image')); i.src = src; });
  function animsOf(fanim) {
    const out = {};
    for (const [tag, a] of Object.entries((fanim && fanim.anims) || {})) {
      out[tag] = (a.frames || []).map(f => ({ x: f.data.x, y: f.data.y, w: f.data.w, h: f.data.h, d: Math.max(16, (f.duration || 0.1) * 1000) }));
    }
    return out;
  }
  async function modFile(mod, rel) {
    const r = await fetch(`/api/mod-file?id=${encodeURIComponent(mod)}&path=${encodeURIComponent(rel)}`);
    if (!r.ok) throw new Error(rel); return (await r.json()).base64;
  }
  const b64text = b => new TextDecoder().decode(Uint8Array.from(atob(b), c => c.charCodeAt(0)));
  // a VFX sheet named by its anim path asset/<mod>/vfx/<sheet>: the editor's bundle first, else the mod folder
  async function sheet(animPath) {
    if (T.sheets[animPath]) return T.sheets[animPath];
    const m = /^asset\/([a-z0-9_]+)\/(vfx|champions)\/(.+)$/.exec(animPath || '');
    let s = null;
    try {
      const name = m ? m[3] : String(animPath || '').split('/').pop();
      const b = (m && m[2] === 'vfx' && window.TFM2_VFX && window.TFM2_VFX[name]) || null;
      if (b) s = { img: await img('data:image/png;base64,' + b.png), anims: animsOf(b.fanim) };
      else if (m) s = { img: await img('data:image/png;base64,' + await modFile(m[1], `${m[2]}/${m[3]}#sheet.png`)), anims: animsOf(JSON.parse(b64text(await modFile(m[1], `${m[2]}/${m[3]}#anim.fanim`)))) };
      else if (/^asset\/base\//.test(animPath || '')) {
        const r = await fetch('/api/base-anim?key=' + encodeURIComponent(animPath));
        if (r.ok) s = { img: await img('/api/asset?key=' + encodeURIComponent(animPath + '#sheet')), anims: animsOf(await r.json()) };
      }
    } catch (e) { s = null; }
    T.sheets[animPath] = s || { img: null, anims: {} };
    return T.sheets[animPath];
  }
  // the champion's body: the bundled sprite (by the end of its id), its own mod sprite, or a base look
  async function champSprite(json, presetSprite) {
    const key = presetSprite || String(json.id || '').split('_').pop();
    const b = window.TFM2_SPRITES && window.TFM2_SPRITES[key];
    if (b) return { img: await img('data:image/png;base64,' + b.png), anims: animsOf(b.fanim) };
    return sheet(json.sprite);
  }
  const bindings = json => {
    const m = {};
    for (const v of (json.view_effects || [])) m['fx:' + v.name] = v;
    for (const v of (json.view_projectiles || [])) m['pj:' + v.name] = v;
    for (const v of (json.view_buffs || [])) m['bf:' + v.name] = v;
    return m;
  };
  async function preloadViews(json) {
    const paths = new Set([...(json.view_effects || []), ...(json.view_projectiles || []), ...(json.view_buffs || [])].map(v => v.anim).filter(Boolean));
    await Promise.all([...paths].map(sheet));
  }

  // ------------------------------------------------------------------ entities
  function statAt(json, lvl, key) {
    const s = (json.stat || {})[key] || 0, g = (json.growth || {})[key] || 0;
    return s + g * (lvl - 1);
  }
  function makeHero() {
    const j = T.json, L = T.opts.level;
    const mp = modPower(j.id);
    const hp = Math.round(statAt(j, L, 'hp') * (1 + mp[2] / 100));
    return {
      id: NEXT_ID++, kind: 'hero', team: 0, x: WW * 0.28, y: WH * 0.55, r: 9000, hp, maxhp: hp,
      atk: Math.round(statAt(j, L, 'attack') * (1 + mp[0] / 100)), ap: Math.round(statAt(j, L, 'magic_power') * (1 + mp[1] / 100)),
      def: Math.round(statAt(j, L, 'defence') * (1 + mp[3] / 100)), mr: Math.round(statAt(j, L, 'magic_resistance') * (1 + mp[4] / 100)),
      ms: (j.stat || {}).move_speed || 1000, cc: {}, buffs: {}, shields: [], face: 1, anim: null, dest: null, atkTarget: null, cds: {}, act: null,
      hist: [], dots: [], shown: '', name: (T.text && T.text.name) || j.id,
    };
  }
  function makeDummy(x, y, i) {
    return { id: NEXT_ID++, kind: 'dummy', team: 1, x, y, r: 9000, hp: T.opts.dummyHp, maxhp: T.opts.dummyHp, atk: 40, ap: 0, def: T.opts.dummyDef, mr: T.opts.dummyMr,
      ms: 700, cc: {}, buffs: {}, shields: [], face: -1, anim: null, home: { x, y }, phase: i * 1.7, name: 'Dummy ' + (i + 1), dmgTaken: 0, hitFlash: 0, atkCd: 0 };
  }
  function resetArena(layout) {
    NEXT_ID = 1; T.projs = []; T.fxs = []; T.texts = []; T.later = []; T.walls = []; T.tick = 0;
    const hero = makeHero();
    T.ents = [hero];
    const pts = layout || [[0.62, 0.55], [0.72, 0.38], [0.72, 0.72]];
    pts.forEach(([fx, fy], i) => T.ents.push(makeDummy(WW * fx, WH * fy, i)));
    T.scr = scribbleState();
    showRank(hero);
    T.log = []; T.demoRun = false;
  }
  const hero = () => T.ents.find(e => e.kind === 'hero');
  const dummies = () => T.ents.filter(e => e.kind !== 'hero' && e.team === 1 && e.hp > 0);
  const MOD_POWER = { _minato: [0, 0, 0, 0, 0, 0], _gojo: [0, 15, 5, 5, 5, 0], _dio: [25, 0, 0, 0, 0, 0], _david: [25, 0, 0, 0, 0, 0], _v1: [30, 0, 5, 10, 10, 0],
    _vader: [5, 0, 0, 0, 0, 0], _frieren: [0, 15, 5, 15, 10, 0], _steve: [0, 0, 0, 0, 0, 0], _omen: [10, 0, 10, 10, 10, 0], _scribble: [0, 10, 5, 0, 5, 0] };
  function modPower(id) {
    if (!/^tfm2_/.test(id || '')) return [0, 0, 0, 0, 0, 0];
    const k = Object.keys(MOD_POWER).find(s => id.endsWith(s)); return k ? MOD_POWER[k] : [0, 0, 0, 0, 0, 0];
  }
  const has = (e, n) => e.buffs[n] != null && e.buffs[n] > T.tick;
  const ccOn = (e, k) => (e.cc[k] || 0) > T.tick;
  const held = e => ccOn(e, 'Stun') || ccOn(e, 'Airborne') || ccOn(e, 'Banish');
  const rooted = e => held(e) || ccOn(e, 'Bind');
  const speedOf = e => {
    let m = 100;
    for (const [n, until] of Object.entries(e.buffs)) if (until > T.tick && e.bstat && e.bstat[n]) m += e.bstat[n].move_speed_mult || 0;
    return Math.max(50, e.ms * m / 100);
  };

  // ------------------------------------------------------------------ combat
  function say(e, text, col, big) {
    // texts on the same unit in the same moment stack upward instead of on top of each other
    const n = T.texts.filter(t => t.who === e && T.tick - t.t0 < 12).length;
    T.texts.push({ x: e.x, y: e.y - 26000 - n * 9500, text, col, t0: T.tick, big, who: e });
  }
  function logLine(text) { T.log.unshift(`${(T.tick / 60).toFixed(1)}s  ${text}`); if (T.log.length > 80) T.log.pop(); }
  function damage(src, t, amount, type, label) {
    if (!t || t.hp <= 0 || amount <= 0) return 0;
    let d = amount;
    if (type === 'phys') d = amount * 100 / (100 + Math.max(0, t.def));
    else if (type === 'magic') d = amount * 100 / (100 + Math.max(0, t.mr));
    d = Math.round(d);
    let left = d;
    for (const s of t.shields) { if (s.until <= T.tick) continue; const k = Math.min(s.amt, left); s.amt -= k; left -= k; }
    t.shields = t.shields.filter(s => s.amt > 0 && s.until > T.tick);
    t.hp = Math.max(0, t.hp - left);
    t.dmgTaken = (t.dmgTaken || 0) + d; t.hitFlash = T.tick + 6;
    if (T.gallery && T.gallery.cur != null && t.team !== 0) T.gallery.dealt += d;
    say(t, String(d), type === 'phys' ? '#ffb15c' : type === 'magic' ? '#7fc4ff' : '#ffffff', d >= 150);
    logLine(`${label || 'hit'} → ${t.name}: ${d} ${type}${left < d ? ` (${d - left} into shield)` : ''}`);
    if (t.kind === 'dummy' && t.hp <= 0) { say(t, 'KO', '#ff5a5a', true); logLine(`${t.name} down`); t.respawnAt = T.tick + 120; }
    if (t.kind === 'hero') setAnim(t, 'hit', 8);
    return d;
  }
  function heal(t, amount, label) {
    const h = Math.min(Math.round(amount), t.maxhp - t.hp); if (h <= 0) return 0;
    t.hp += h; say(t, '+' + h, '#7dff9a'); logLine(`${label || 'heal'} → ${t.name}: +${h}`); return h;
  }
  function shield(t, amount, ticks, label) { t.shields.push({ amt: Math.round(amount), until: T.tick + ticks }); say(t, `shield ${Math.round(amount)}`, '#e6f1ff'); logLine(`${label || 'shield'} → ${t.name}: ${Math.round(amount)} for ${secs(ticks)}`); }
  const CC_LABEL = { Stun: 'STUN', Airborne: 'UP', Bind: 'ROOT', BlockAttack: 'NO ATTACK', BlockSkill: 'SILENCE', BlockMoveSkill: 'NO DASH', Taunt: 'TAUNT', Fear: 'FEAR', Charm: 'CHARM', Banish: 'BANISH' };
  function cc(t, kind, ticks, label) {
    if (!t || t.hp <= 0 || ticks <= 0) return;
    t.cc[kind] = Math.max(t.cc[kind] || 0, T.tick + ticks);
    say(t, `${CC_LABEL[kind] || kind} ${secs(ticks)}`, '#ffe066');
    logLine(`${label || 'cc'} → ${t.name}: ${kind} ${secs(ticks)}`);
  }
  function slow(t, pct, ticks, name) { addBuff(t, name || 'slow', ticks, { move_speed_mult: -pct }); say(t, `-${pct}% speed`, '#c7a6ff'); }
  function addBuff(t, name, ticks, stats) {
    t.buffs[name] = ticks === Infinity ? Infinity : T.tick + ticks;
    if (stats) { t.bstat = t.bstat || {}; t.bstat[name] = stats; }
  }
  // a push that plays out over ticks (knockback away from a point, or a pull toward one)
  function shove(t, fromX, fromY, speed, ticks, toward) {
    const [ux, uy] = norm(t.x - fromX, t.y - fromY);
    const s = toward ? -1 : 1;
    t.shove = { vx: ux * speed * s, vy: uy * speed * s, until: T.tick + ticks, stopAt: toward ? { x: fromX, y: fromY } : null };
  }
  function blinkTo(e, x, y) { const p = blockedEnd(e.x, e.y, clamp(x, 6000, WW - 6000), clamp(y, 6000, WH - 6000)); e.x = p.x; e.y = p.y; e.dest = null; }

  // ------------------------------------------------------------------ visuals
  function playFx(name, at, opts) {
    const b = T.bind['fx:' + name]; if (!b) return false;
    const follow = b.is_follow && at && at.id;
    T.fxs.push({ anim: b.anim, tag: b.tag, z: b.z || 0, t0: performance.now(), tick0: T.tick, x: at.x, y: at.y, follow: follow ? at : null, loopUntil: opts && opts.loop ? T.tick + opts.loop : 0, flip: opts && opts.flip });
    return true;
  }
  const fxS = (tag, at, opts) => playFx(`${T.json.id}_${tag}`, at, opts);
  function setAnim(e, tag, ticks, loop) { e.anim = { tag, t0: performance.now(), until: T.tick + ticks, loop: !!loop }; }

  // ------------------------------------------------------------------ walls (Scribble's Drawn Wall)
  function crossWall(ax, ay, bx, by) {
    for (const w of T.walls) {
      if (w.until <= T.tick) continue;
      const o = (p, q, r) => Math.sign((q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x));
      const A = { x: ax, y: ay }, B = { x: bx, y: by }, C = { x: w.ax, y: w.ay }, D = { x: w.bx, y: w.by };
      if (o(A, B, C) !== o(A, B, D) && o(C, D, A) !== o(C, D, B)) return true;
    }
    return false;
  }
  function blockedEnd(ax, ay, bx, by) {
    if (!crossWall(ax, ay, bx, by)) return { x: bx, y: by };
    const n = 40; let last = { x: ax, y: ay };
    for (let i = 1; i <= n; i++) { const p = { x: ax + (bx - ax) * i / n, y: ay + (by - ay) * i / n }; if (crossWall(ax, ay, p.x, p.y)) return last; last = p; }
    return last;
  }

  // ------------------------------------------------------------------ the data interpreter
  const isEnemyTarget = t => /^Enemy/.test(t || 'Enemy');
  function pickTargets(filter, caster) {
    const f = filter || 'Enemy';
    if (/OnlySelf/.test(f)) return [caster];
    if (/^Ally/.test(f)) return T.ents.filter(e => e.team === caster.team && e.hp > 0 && !(/NotSelf/.test(f) && e === caster));
    return T.ents.filter(e => e.team !== caster.team && e.hp > 0);
  }
  const radiusOf = shape => (shape && (shape.Circle ? shape.Circle.radius : shape.Rect ? Math.max(shape.Rect.width || 0, shape.Rect.height || 0) / 2 : shape.Sector ? shape.Sector.radius : 10000)) || 10000;
  const raw = list => (list || []).map(x => (x && x.effect ? x.effect : x)).filter(Boolean);
  function runList(list, c) { for (const e of raw(list)) run(e, c); }
  function after(ticks, fn) { T.later.push({ at: T.tick + Math.max(0, ticks | 0), fn }); }
  function run(e, c) {
    if (!e || !e.type) return;
    const me = c.caster, t = c.target;
    const tgt = t || null;
    switch (e.type) {
      case 'Combine': runList(e.effects, c); break;
      case 'Delayed': after(e.tick, () => runList(e.effects, c)); break;
      case 'WithSelf': runList(e.effects, Object.assign({}, c, { target: me })); break;
      case 'SwitchByBuff': run(has(me, e.buff_name) ? e.effect_buff : e.effect_none, c); break;
      case 'SwitchByLevel3': run(T.opts.level >= 3 ? e.effect_level3 : e.effect_start, c); break;
      case 'RandomTarget': { const pool = pickTargets(e.casting_target, me).filter(x => dist(x, me) <= (e.range || 65000)); const p = pool[Math.floor(Math.random() * pool.length)]; if (p) runList(e.effects, Object.assign({}, c, { target: p, x: p.x, y: p.y })); break; }
      case 'Attack': if (tgt) damage(me, tgt, (e.damage || 0) + (e.attack_ratio || 0) * me.atk / 100 + (e.hp_ratio || 0) * me.maxhp / 100 + (e.target_hp_ratio || 0) * tgt.maxhp / 100, 'phys', c.label); break;
      case 'ApAttack': if (tgt) damage(me, tgt, (e.damage || 0) + (e.attack_ratio || 0) * me.ap / 100 + (e.hp_ratio || 0) * me.maxhp / 100, 'magic', c.label); break;
      case 'FixedAttack': if (tgt) damage(me, tgt, (e.damage || 0) + (e.attack_ratio || 0) * me.atk / 100 + (e.hp_ratio || 0) * me.maxhp / 100 + (e.target_hp_ratio || 0) * tgt.maxhp / 100, 'true', c.label); break;
      case 'Heal': { const who = tgt && tgt.team === me.team ? tgt : me; heal(who, (e.amount || 0) + (e.attack_ratio || 0) * me.atk / 100 + (e.ap_ratio || 0) * me.ap / 100, c.label); break; }
      case 'Shield': { const who = tgt && tgt.team === me.team ? tgt : me; shield(who, (e.amount || 0) + (e.attack_ratio || 0) * me.atk / 100 + (e.ap_ratio || 0) * me.ap / 100, e.tick || 300, c.label); break; }
      case 'Stun': case 'Airborne': case 'Bind': case 'Taunt': cc(tgt, e.type, e.duration || 0, c.label); break;
      case 'Fear': case 'Charm': case 'BlockAttack': case 'BlockSkill': case 'BlockMoveSkill': cc(tgt, e.type, e.tick || 0, c.label); break;
      case 'Banish': cc(tgt, 'Banish', e.duration || 0, c.label); break;
      case 'Invisible': if (tgt) { addBuff(tgt, '~invisible', e.tick || 0); say(tgt, 'invisible', '#cfd8e3'); } break;
      case 'Knockback': if (tgt) shove(tgt, me.x, me.y, e.speed || 2000, e.tick || 10); break;
      case 'Grab': if (tgt) shove(tgt, me.x, me.y, e.speed || 3500, e.tick || 30, true); break;
      case 'Pull': if (tgt) shove(tgt, c.x != null ? c.x : me.x, c.y != null ? c.y : me.y, e.speed || 2500, e.tick || 15, true); break;
      case 'Teleport': { const p = tgt || c; if (p && p.x != null) { fxCaster(me); blinkTo(me, p.x, p.y); } break; }
      case 'DirTeleport': { const [ux, uy] = norm((c.x || me.x + 1) - me.x, (c.y || me.y) - me.y); blinkTo(me, me.x + ux * (e.moved || 32000), me.y + uy * (e.moved || 32000)); break; }
      case 'MoveBack': { const [ux, uy] = tgt ? norm(me.x - tgt.x, me.y - tgt.y) : [-me.face, 0]; me.dash = { vx: ux * (e.speed || 2500), vy: uy * (e.speed || 2500), until: T.tick + (e.tick || 12) }; break; }
      case 'Rush': case 'RushTime': case 'MoveTo': case 'MoveToTarget': case 'RushMoveToBack': {
        const goal = (e.type === 'RushMoveToBack' && tgt) ? { x: tgt.x + norm(tgt.x - me.x, tgt.y - me.y)[0] * 12000, y: tgt.y + norm(tgt.x - me.x, tgt.y - me.y)[1] * 12000 } : (tgt || { x: c.x, y: c.y });
        if (goal.x == null) break;
        const sp = e.speed || 3500, len = Math.min(dist(me, goal), e.range || 50000);
        const [ux, uy] = norm(goal.x - me.x, goal.y - me.y);
        const ticks = e.type === 'RushTime' ? (e.tick || 30) : Math.max(1, Math.ceil(len / sp));
        me.dash = { vx: ux * sp, vy: uy * sp, until: T.tick + ticks };
        after(ticks, () => { runList(e.end_effects, Object.assign({}, c, { x: me.x, y: me.y })); if (tgt && (e.applied_effects || []).length) runList(e.applied_effects, c); });
        break;
      }
      case 'LinearProjectile': case 'BackToCasterLinearProjectile': {
        const from = c.fromProj || me;
        const aim = tgt || { x: c.x, y: c.y };
        const [ux, uy] = aim.x != null ? norm(aim.x - from.x, aim.y - from.y) : [me.face, 0];
        T.projs.push({ name: e.name, x: from.x, y: from.y, vx: ux * (e.speed || 4200), vy: uy * (e.speed || 4200), left: e.range || 65000, r: radiusOf(e.shape),
          pen: !!e.penetrate, filter: e.applied_target, applied: e.applied_effects, end: e.end_effects, c, hit: new Set(), back: e.type === 'BackToCasterLinearProjectile' });
        break;
      }
      case 'TargetProjectile': case 'TargetSplashProjectile': case 'AutoTargetProjectile': case 'TargetProjectileFromProjectile': {
        let aim = tgt;
        if (e.type === 'AutoTargetProjectile' || !aim) aim = pickTargets(e.applied_target, me).filter(x => dist(x, me) <= (e.range || 60000)).sort((a, b) => d2(a, me) - d2(b, me))[0];
        if (!aim) break;
        const from = c.fromProj || me;
        T.projs.push({ name: e.name, x: from.x, y: from.y - 3000, speed: e.speed || 4500, homing: aim, applied: e.applied_effects, end: e.end_effects, c, r: 4000, hit: new Set(), filter: e.applied_target });
        break;
      }
      case 'ParabolicProjectile': {
        const aim = tgt || { x: c.x, y: c.y }; if (aim.x == null) break;
        const tt = e.travel_time || 45, r = radiusOf(e.shape);
        const p = { name: e.name, x: me.x, y: me.y, sx: me.x, sy: me.y, tx: aim.x, ty: aim.y, t0: T.tick, tt, arc: true, applied: [], c, hit: new Set() };
        T.projs.push(p);
        after(tt, () => {
          p.dead = true;
          if (e.range_effect_name) playFx(e.range_effect_name, { x: aim.x, y: aim.y });
          for (const x of pickTargets(e.applied_target, me).filter(x => d2(x, aim) <= r * r)) runList(e.applied_effects, Object.assign({}, c, { target: x }));
          runList(e.end_effects, Object.assign({}, c, { x: aim.x, y: aim.y, target: null }));
        });
        break;
      }
      case 'RangeProjectile': case 'LineRangeProjectile': case 'RangePeriodProjectile': case 'ApplyInProjectile': {
        const at = e.type === 'ApplyInProjectile' && e.follow_caster ? me : (tgt || (c.x != null ? { x: c.x, y: c.y } : me));
        const r = e.type === 'LineRangeProjectile' ? Math.max(e.width || 10000, 8000) : radiusOf(e.shape);
        const zone = { x: at.x, y: at.y, r, until: T.tick + (e.tick || e.delay || 30) + 2, line: e.type === 'LineRangeProjectile' ? { len: e.length || 70000, dir: norm((c.x != null ? c.x : at.x + 1) - me.x, (c.y != null ? c.y : at.y) - me.y) } : null };
        T.zones = T.zones || []; T.zones.push(zone);
        if (e.name) playFx(e.name, at);
        const hitNow = () => {
          const pos = e.follow_caster ? me : zone;
          const inside = x => zone.line ? lineHit(me, zone.line.dir, x, zone.line.len, r) : d2(x, pos) <= (r + x.r * 0.5) ** 2;
          for (const x of pickTargets(e.applied_target, me).filter(inside)) runList(e.applied_effects, Object.assign({}, c, { target: x }));
        };
        if (e.type === 'RangePeriodProjectile') {
          const period = Math.max(1, e.period || 30), total = e.tick || 180;
          for (let k = e.first_delay || 0; k < total; k += period) after(k, hitNow);
          after(total, () => runList(e.end_effects, Object.assign({}, c, { x: zone.x, y: zone.y })));
        } else after(e.delay || e.tick || 0, () => { hitNow(); runList(e.end_effects, Object.assign({}, c, { x: zone.x, y: zone.y })); });
        break;
      }
      case 'RangeEffect': {
        const r = radiusOf(e.shape);
        const centre = e.apply_type === 'AroundCaster' || !e.apply_type ? me : (tgt || { x: c.x, y: c.y });
        for (const x of pickTargets(e.target, me).filter(x => d2(x, centre) <= (r + x.r * 0.5) ** 2)) runList(e.effects, Object.assign({}, c, { target: x }));
        break;
      }
      case 'ShrinkingBarrier': { T.zones = T.zones || []; T.zones.push({ x: (tgt || me).x, y: (tgt || me).y, r: e.start_radius || 70000, until: T.tick + (e.tick || 120), ring: true }); break; }
      case 'AddBuff': if (tgt) { const b = e.buff_state || {}; addBuff(tgt, b.name || 'buff', durOf(b), b); if (b.move_speed_mult) say(tgt, `${b.move_speed_mult > 0 ? '+' : ''}${b.move_speed_mult}% speed`, '#c7a6ff'); } break;
      case 'AddCasterBuff': { const b = e.buff_state || {}; addBuff(me, b.name || 'buff', durOf(b), b); break; }
      case 'RemoveCasterBuff': delete me.buffs[e.name]; break;
      case 'AddCasted': if (tgt) { const period = Math.max(1, e.period || 30); for (let k = period; k <= (e.duration || 180); k += period) after(k, () => tgt.hp > 0 && runList(e.effects, Object.assign({}, c, { target: tgt }))); } break;
      case 'ViewEffect': playFx(e.name, tgt || (c.x != null ? { x: c.x, y: c.y } : me)); break;
      case 'CasterViewEffect': playFx(e.name, me); break;
      case 'CasterAnimation': setAnim(me, e.name, e.tick || 30); break;
      case 'RemoveCasterAnimation': if (me.anim && me.anim.tag === e.name) me.anim = null; break;
      case 'Native': say(me, `native: ${String(e.effect_ref || '').split(':').pop()}`, '#9aa7b4'); logLine(`native effect ${e.effect_ref} (not simulated here)`); break;
      default: break;
    }
  }
  function fxCaster(me) { /* a puff at the old spot for blinks with no binding */ }
  const durOf = b => (b && b.duration && b.duration.Time ? b.duration.Time.tick : Infinity);
  function lineHit(from, dir, e, len, halfW) {
    const ex = e.x - from.x, ey = e.y - from.y;
    const along = ex * dir[0] + ey * dir[1], side = Math.abs(ex * dir[1] - ey * dir[0]);
    return along >= -4000 && along <= len && side <= halfW + e.r * 0.5;
  }

  // ------------------------------------------------------------------ slots (the data's 4 actions)
  const SLOT_KEYS = { q: 'skill', w: 'skill2', e: 'ult' };
  function aimTarget(range) {
    const h = hero();
    const pool = dummies().filter(d => dist(d, h) <= range + d.r);
    pool.sort((a, b) => d2(a, T.mouse) - d2(b, T.mouse));
    return pool[0] || null;
  }
  function castSlot(slot, forcedTarget) {
    const h = hero(); const a = T.json[slot]; if (!a || h.hp <= 0) return false;
    if (held(h) || (slot !== 'attack' && ccOn(h, 'BlockSkill')) || (slot === 'attack' && ccOn(h, 'BlockAttack'))) { if (slot !== 'attack') say(h, 'can\'t cast', '#ff9a9a'); return false; }
    if (h.act && !(slot === 'attack' && h.act.slot === 'attack' && h.act.cancelable)) return false;
    if (T.opts.cooldowns && (h.cds[slot] || 0) > T.tick) { if (slot !== 'attack') say(h, `${slot} ${secs(h.cds[slot] - T.tick)}`, '#9aa7b4'); return false; }
    const ct = a.casting_type || 'Targeting';
    let target = null, x = T.mouse.x, y = T.mouse.y;
    if (ct === 'Targeting') {
      if (/^Ally|OnlySelf/.test(a.casting_target || '')) target = h;
      else target = forcedTarget || aimTarget(Math.max(a.range || 30000, 1));
      if (!target) { if (slot !== 'attack') say(h, 'no target in range', '#ff9a9a'); return false; }
      x = target.x; y = target.y;
    }
    if (ct === 'Direction' || ct === 'Position') { const [ux, uy] = norm(x - h.x, y - h.y); const r = Math.min(dist(h, { x, y }), a.range || 60000); x = h.x + ux * r; y = h.y + uy * r; }
    h.face = x >= h.x ? 1 : -1;
    const dur = a.duration || 20, at = Math.min(a.start_timing || 0, dur);
    h.act = { slot, until: T.tick + dur, cancelable: !!a.cancelable };
    setAnim(h, a.action_name || slot, dur);
    h.cds[slot] = T.tick + (a.cooltime || 60) + (slot === 'attack' ? 0 : 0);
    const c = { caster: h, target, x, y, label: slot === 'attack' ? 'basic attack' : (T.text && T.text[slot] ? slot : slot) };
    after(at, () => { if (h.hp > 0) run(a.effect, c); });
    if (!a.can_use_with_move) h.dest = null;
    return true;
  }

  // ------------------------------------------------------------------ Scribble (mirror of the native spells)
  function scribbleState() { return { dots: [], queue: [], nextWeave: 0, invoking: null, ready: new Array(35).fill(0), last: null, misfires: 0, slipNotice: null }; }
  const isScribble = () => T.json && T.json.passive && T.json.passive.passive_ref === 'tfm2_custom_ai:scribble';
  const BOOK = () => window.TFM2_SCRIBBLE_BOOK || [];
  const recipeOf = i => BOOK()[i] ? BOOK()[i][0].split('-').map(Number) : [];
  const recipeIndex = dots => BOOK().findIndex(b => b[0] === dots.join('-'));
  function weave(el) {
    const s = T.scr, h = hero(); if (!h || h.hp <= 0) return;
    if (s.queue.length + (s.invoking ? 0 : s.dots.length) >= 6) { say(h, '6 dots max', '#ff9a9a'); return; }
    s.queue.push(el);
  }
  function scribbleTick() {
    const s = T.scr, h = hero(); if (!h) return;
    const r = T.opts.rank;
    if (s.queue.length && T.tick >= s.nextWeave && !held(h) && !s.invoking) {
      let el = s.queue.shift();
      if (T.opts.slips && Math.random() * 100 < slipPct(r, s.dots.length + 1)) {
        const want = el; el = ((want - 1 + 1 + Math.floor(Math.random() * 4)) % 5) + 1; s.misfires++;
        say(h, `slip! ${ELEMENTS[want]} → ${ELEMENTS[el]}`, '#ff9a9a');
        if (Math.random() * 100 < NOTICE[r]) s.slipNotice = T.tick + 8;
      }
      s.dots.push(el); fxS('weave', h); s.nextWeave = Math.max(s.nextWeave, T.tick - 0.999) + weaveInterval(r, T.opts.topPos);
    }
    if (s.slipNotice && T.tick >= s.slipNotice) { s.slipNotice = null; s.dots = []; s.queue = []; say(h, 'noticed, flicked away', '#9aa7b4'); }
    if (s.invoking && T.tick >= s.invoking.at) { const iv = s.invoking; s.invoking = null; resolveSpell(iv.spell, iv.aim, iv.dots); s.dots = []; }
    // the HUD buffs: scr_d<slot>_<element>
    const key = s.dots.join('');
    if (key !== h.shown) {
      for (const n of Object.keys(h.buffs)) if (n.startsWith('scr_d')) delete h.buffs[n];
      s.dots.forEach((e, k) => { h.buffs[`scr_d${k}_${e}`] = Infinity; });
      h.shown = key;
    }
    showRank(h);
  }
  // the rank badge, and from Grandmaster up the skin's two layers (behind him / over him)
  function showRank(h) {
    const r = T.opts.rank, badge = badgeBuff(r, T.opts.topPos), skin = skinOf(r);
    const want = new Set([badge, ...(skin === undefined ? [] : [`scr_skin${skin}_b`, `scr_skin${skin}_f`])]);
    for (const n of Object.keys(h.buffs)) if (/^scr_(rank|top|skin)/.test(n) && !want.has(n)) delete h.buffs[n];
    for (const n of want) h.buffs[n] = Infinity;
  }
  function invoke(aim) {
    const s = T.scr, h = hero(); if (!h || h.hp <= 0) return;
    const at = aim || { x: T.mouse.x, y: T.mouse.y };
    // pressed while casting or still weaving: it goes off as soon as it can (like a buffered key)
    if (s.invoking) { after(s.invoking.at - T.tick + 1, () => invoke(at)); return; }
    if (s.queue.length) { after(1, () => invoke(at)); return; }
    aim = at;
    if (!s.dots.length) { say(h, 'no dots', '#9aa7b4'); return; }
    if (held(h)) return;
    const t = INVOKE_T[T.opts.rank];
    s.invoking = { at: T.tick + t, spell: recipeIndex(s.dots), aim: aim || { x: T.mouse.x, y: T.mouse.y }, dots: s.dots.slice() };
    h.face = s.invoking.aim.x >= h.x ? 1 : -1; h.dest = null;
    setAnim(h, 'ult', t); h.act = { slot: 'invoke', until: T.tick + t };
    fxS('invoke', h);
  }
  function castRecipe(i, aim) {
    const s = T.scr; s.dots = []; s.queue = recipeOf(i).slice(); s.last = i;
    const waitWeave = () => { if (s.queue.length || T.tick < s.nextWeave - weaveInterval(T.opts.rank, T.opts.topPos) + 1) return after(2, waitWeave); invoke(aim); };
    after(1, waitWeave);
  }
  function resolveSpell(i, aim, dots) {
    const h = hero(), s = T.scr, ap = h.ap;
    const D = (b, r) => b + r * ap / 100;
    if (i < 0) { fxS('fizzle', h); say(h, `${dots.join('-')}: no such recipe`, '#ff9a9a'); logLine(`fizzle: ${dots.join('-')} is no recipe`); return; }
    const [name] = [BOOK()[i][1]];
    if (T.opts.cooldowns && s.ready[i] > T.tick) { fxS('fizzle', h); say(h, `${recipeOf(i).length}-dot spells on cooldown (${secs(s.ready[i] - T.tick)})`, '#ff9a9a'); return; }
    // per dot count (round 64): every spell with as many dots goes on this cooldown
    const tierN = recipeOf(i).length;
    for (let k = 0; k < BOOK().length; k++) if (recipeOf(k).length === tierN) s.ready[k] = Math.max(s.ready[k], T.tick + BOOK()[i][2]);
    if (T.gallery) T.gallery.castAt = T.tick;
    const [tag, kind, range] = SPELL_FX[i];
    const ax = aim.x, ay = aim.y;
    const foes = dummies();
    const near = (p, r) => foes.filter(e => d2(e, p) <= (r + e.r * 0.5) ** 2);
    const target = kind === 'e' ? foes.filter(e => dist(e, h) <= range + e.r).sort((a, b) => d2(a, aim) - d2(b, aim))[0] : null;
    logLine(`${name} (${dots.join('-')})`);
    say(h, name, '#ffffff', true);
    if (kind === 'e' && !target) { fxS('fizzle', h); say(h, 'out of range: wasted', '#ff9a9a'); logLine(`${name}: no enemy within ${range}, wasted`); return; }
    const L = name;
    const dmg = (t, v) => damage(h, t, v, 'magic', L);
    const [dx, dy] = norm(ax - h.x, ay - h.y);
    const inCone = (t, e, r, half) => { if (d2(e, h) > r * r) return false; const [a1, a2] = norm(t.x - h.x, t.y - h.y), [b1, b2] = norm(e.x - h.x, e.y - h.y); return a1 * b1 + a2 * b2 >= Math.cos(half * Math.PI / 180) || d2(e, h) < 6000 ** 2; };
    switch (i) {
      case 0: fxS('poke', target); dmg(target, D(30, 50)); break;
      case 1: fxS('smudge', target); if (target.shields.length) { target.shields = []; say(target, 'shields erased', '#f497b6'); } dmg(target, D(20, 30)); break;
      case 2: fxS('splat', target); for (const e of near(target, 15000)) { dmg(e, D(25, 40)); slow(e, 20, 60, 'scr_slow'); } break;
      case 3: fxS('honk', h); for (const e of near(h, 30000)) { dmg(e, D(15, 30)); cc(e, 'Stun', 12, L); } break;
      case 4: { const tr = Math.ceil(dist(h, target) / 9000) + 1; spawnVisual('page_fly', h, target, 9000); for (let k = 0; k < 4; k++) after(tr + k * 30, () => { if (target.hp > 0) { fxS('cut', target); dmg(target, D(40, 60) / 4); } }); break; }
      case 5: fxS('door', h); blinkTo(h, h.x + dx * 35000, h.y + dy * 35000); fxS('door', h); break;
      case 6: fxS('cutout', h); shield(h, D(80, 60), 180, L); break;
      case 7: fxS('erase_self', h); addBuff(h, '~invisible', 90); say(h, 'invisible 1.5s', '#cfd8e3'); break;
      case 8: fxS('bucket', h); heal(h, D(70, 50), L); break;
      case 9: { const p = { x: ax, y: ay }; fxS('peel', p, { loop: 300 }); const until = T.tick + 300;
        const check = () => { if (T.tick >= until) return; const v = foes.find(e => e.hp > 0 && d2(e, p) <= 10000 ** 2); if (v) { fxS('slip', v); cc(v, 'Airborne', 30, L); shove(v, h.x, h.y, 2000, 14); T.fxs = T.fxs.filter(f => !(f.tag === 'peel' && f.x === p.x && f.y === p.y)); return; } after(2, check); };
        check(); break; }
      case 10: fxS('chicken', target); dmg(target, D(40, 50)); shove(target, h.x, h.y, 3000, 12); break;
      case 11: fxS('key', h); addBuff(h, 'scr_key', 180, { move_speed_mult: 40 }); say(h, '+40% speed', '#c7a6ff'); break;
      case 12: fxS('bubble', target); cc(target, 'BlockAttack', 48, L); break;
      case 13: fxS('phone', h); for (const e of foes.filter(e => inCone(target, e, 45000, 35))) { fxS('photo', e); cc(e, 'Stun', 72, L); } break;
      case 14: fxS('mallet', target); dmg(target, D(60, 80)); cc(target, 'Stun', 60, L); break;
      case 15: { const p = { x: target.x, y: target.y }; fxS('anvil_shadow', p, { loop: 48 }); after(48, () => { fxS('anvil', p); for (const e of near(p, 12000)) { fxS('bonk', e); dmg(e, D(90, 100)); cc(e, 'Stun', 60, L); } if (!near(p, 12000).length) say(p, 'missed', '#9aa7b4'); }); break; }
      case 16: spawnVisual('glove', h, target, 12000); shove(target, h.x, h.y, 4000, Math.ceil(Math.max(0, dist(h, target) - 12000) / 4000), true); say(target, 'GRABBED', '#ffe066'); break;
      case 17: fxS('erase_legs', target); cc(target, 'Bind', 90, L); break;
      case 18: fxS('pie', target); cc(target, 'BlockSkill', 120, L); cc(target, 'BlockAttack', 60, L); break;
      case 19: fxS('present', target); after(120, () => { if (target.hp <= 0) return; fxS('boom', target); for (const e of near(target, 20000)) dmg(e, D(120, 110)); }); break;
      case 20: { const p = { x: ax, y: ay }; fxS('floor', p, { loop: 240 }); for (let k = 0; k < 240; k += 10) after(k, () => { for (const e of T.ents) if (d2(e, p) <= 40000 ** 2) addBuff(e, e.team === h.team ? 'scr_floor_up' : 'scr_floor_dn', 12, { move_speed_mult: e.team === h.team ? 25 : -30 }); }); break; }
      case 21: fxS('hole', h); blinkTo(h, h.x + dx * Math.min(120000, dist(h, aim)), h.y + dy * Math.min(120000, dist(h, aim))); fxS('hole', h); fxS('hole_pop', h); break;
      case 22: fxS('panorama', { x: (h.x + target.x) / 2, y: (h.y + target.y) / 2 }); for (const e of foes.filter(e => inCone(target, e, 70000, 60))) { fxS('photo', e); cc(e, 'Stun', 90, L); } break;
      case 23: { const ts = near(target, 15000); for (const e of ts) { fxS('brawl', e); cc(e, 'Bind', 120, L); } for (let k = 0; k < 4; k++) after(24 + k * 28, () => { for (const e of ts) if (e.hp > 0) { fxS('bonk', e); dmg(e, D(30, 25)); } }); break; }
      case 24: { const then = h.hist.find(p => p.t <= T.tick - 180) || h.hist[0]; fxS('redraw', h); if (then) { h.cc = {}; h.x = then.x; h.y = then.y; } fxS('redraw', h); break; }
      case 25: fxS('stamp', h); for (const e of near(h, 35000)) cc(e, 'Stun', 72, L); shield(h, D(120, 80), 240, L); break;
      case 26: { const [px, py] = [-dy, dx]; const w = { ax: ax + px * 45000, ay: ay + py * 45000, bx: ax - px * 45000, by: ay - py * 45000, until: T.tick + 300 }; T.walls.push(w);
        const n = Math.ceil(90000 / 20000); for (let k = 0; k <= n; k++) { const p = { x: w.ax + (w.bx - w.ax) * k / n, y: w.ay + (w.by - w.ay) * k / n }; fxS('wall_draw', p); after(12, () => fxS('wall_seg', p, { loop: 288 })); } break; }
      case 27: { const then = h.hist.find(p => p.t <= T.tick - 240) || h.hist[0]; fxS('rewind', h); if (then) { fxS('rewind_swirl', h); h.x = then.x; h.y = then.y; if (then.hp > h.hp) heal(h, then.hp - h.hp, L); } break; }
      case 28: { const p = { x: target.x, y: target.y }; fxS('piano_shadow', p, { loop: 60 }); after(60, () => { fxS('piano', p); for (const e of near(p, 30000)) { fxS('bonk', e); dmg(e, D(200, 150)); cc(e, 'Stun', 90, L); } }); break; }
      case 29: { const [ux, uy] = norm(target.x - h.x, target.y - h.y); for (let k = 0; k < 8; k++) { const s2 = 10000 + k * 20000; after(k * 2, () => fxS('ink_wave', { x: h.x + ux * s2, y: h.y + uy * s2 })); }
        for (const e of foes.filter(e => lineHit(h, [ux, uy], e, 160000, 20000))) { dmg(e, D(60, 50)); cc(e, 'BlockAttack', 120, L); slow(e, 40, 120, 'scr_slow'); } break; }
      case 30: { fxS('chase', h); addBuff(h, 'scr_chase', 240, { move_speed_mult: 100 }); const hit = new Set();
        if (T.demoRun) h.dest = { x: WW * 0.9, y: h.y };
        for (let k = 0; k < 240; k += 2) after(k, () => { for (const e of dummies()) if (!hit.has(e.id) && d2(e, h) <= 14000 ** 2 + (e.r * 0.5) ** 2) { hit.add(e.id); fxS('bonk', e); dmg(e, D(40, 40)); cc(e, 'Stun', 18, L); } }); break; }
      case 31: fxS('laugh', { x: h.x, y: h.y - 30000 }); for (const e of near(h, 90000)) { fxS('haha', e); cc(e, 'BlockAttack', 90, L); cc(e, 'BlockSkill', 90, L); } heal(h, D(60, 40), L); break;
      case 32: { // the arena's middle band stays; the top and bottom halves swap
        fxS('page', { x: WW / 2, y: WH / 2 });
        for (const e of T.ents) if (Math.abs(e.y - WH / 2) > 20000) { fxS('page_swish', e); e.y = WH - e.y; e.dest = null; } break; }
      case 33: fxS('pause', { x: h.x, y: h.y - 40000 }); for (const e of foes) { fxS('pause_icon', e); cc(e, 'Stun', 120, L); } break;
      case 34: { const st = { id: NEXT_ID++, kind: 'unit', team: 0, x: h.x, y: h.y, r: 8000, hp: 600, maxhp: 600, atk: Math.round(h.atk * 0.8), ap: 0, def: 20, mr: 20, ms: 900,
        cc: {}, buffs: {}, shields: [], face: 1, anim: null, name: 'Sketch friend', until: T.tick + 600, sprite: 'shadow_bombardier', atkCd: 0 };
        T.ents.push(st); fxS('sketch_in', st); logLine('a sketched friend joins for 10s on the cast spot (in a match: the strongest fallen teammate revived there at 60% HP)'); break; }
      default: break;
    }
  }
  function spawnVisual(tag, from, to, speed) {
    T.projs.push({ name: `${T.json.id}_${tag}`, x: from.x, y: from.y - 3000, speed, homing: to, applied: [], c: { caster: from }, r: 4000, hit: new Set(), visual: true });
  }

  // ------------------------------------------------------------------ the tick
  function step() {
    T.tick++;
    const h = hero();
    // delayed things
    const due = T.later.filter(l => l.at <= T.tick); T.later = T.later.filter(l => l.at > T.tick);
    for (const l of due) { try { l.fn(); } catch (e) { console.warn(e); } }
    if (isScribble()) scribbleTick();
    for (const e of T.ents) {
      if (e.hp <= 0) { if (e.kind === 'dummy' && e.respawnAt && T.tick >= e.respawnAt) { e.hp = e.maxhp; e.cc = {}; e.respawnAt = 0; e.x = e.home.x; e.y = e.home.y; } continue; }
      if (e.act && T.tick >= e.act.until) e.act = null;
      // forced movement first
      if (e.shove && T.tick < e.shove.until) {
        let nx = e.x + e.shove.vx, ny = e.y + e.shove.vy;
        if (e.shove.stopAt && d2({ x: nx, y: ny }, e.shove.stopAt) < 10000 ** 2) { e.shove = null; }
        else { const p = blockedEnd(e.x, e.y, clamp(nx, 4000, WW - 4000), clamp(ny, 4000, WH - 4000)); e.x = p.x; e.y = p.y; }
        continue;
      }
      if (e.dash && T.tick < e.dash.until) { const p = blockedEnd(e.x, e.y, clamp(e.x + e.dash.vx, 4000, WW - 4000), clamp(e.y + e.dash.vy, 4000, WH - 4000)); e.x = p.x; e.y = p.y; continue; }
      if (rooted(e)) continue;
      if (e.kind === 'hero') heroAI(e);
      else if (e.kind === 'dummy') dummyAI(e);
      else unitAI(e);
    }
    if (h) { h.hist.unshift({ t: T.tick, x: h.x, y: h.y, hp: h.hp }); if (h.hist.length > 320) h.hist.pop(); }
    T.ents = T.ents.filter(e => !(e.kind === 'unit' && (T.tick >= e.until || e.hp <= 0)));
    // projectiles
    for (const p of T.projs) {
      if (p.dead) continue;
      if (p.arc) { const k = Math.min(1, (T.tick - p.t0) / p.tt); p.x = p.sx + (p.tx - p.sx) * k; p.y = p.sy + (p.ty - p.sy) * k - Math.sin(k * Math.PI) * 30000; continue; }
      if (p.homing) {
        const t = p.homing; if (t.hp <= 0) { p.dead = true; continue; }
        const [ux, uy] = norm(t.x - p.x, t.y - p.y); p.ang = Math.atan2(uy, ux);
        if (dist(p, t) <= p.speed + 3000) { p.dead = true; if (!p.visual) runList(p.applied, Object.assign({}, p.c, { target: t, x: t.x, y: t.y })); runList(p.end, Object.assign({}, p.c, { x: t.x, y: t.y, target: t })); }
        else { p.x += ux * p.speed; p.y += uy * p.speed; }
        continue;
      }
      const sp = Math.hypot(p.vx, p.vy); p.ang = Math.atan2(p.vy, p.vx);
      p.x += p.vx; p.y += p.vy; p.left -= sp;
      for (const t of pickTargets(p.filter, p.c.caster)) {
        if (p.hit.has(t.id) || d2(t, p) > (p.r + t.r * 0.6) ** 2) continue;
        p.hit.add(t.id); runList(p.applied, Object.assign({}, p.c, { target: t, x: t.x, y: t.y }));
        if (!p.pen) { p.dead = true; runList(p.end, Object.assign({}, p.c, { x: p.x, y: p.y, target: t, fromProj: p })); break; }
      }
      if (!p.dead && (p.left <= 0 || p.x < 0 || p.y < 0 || p.x > WW || p.y > WH)) {
        if (p.back && !p.returning) { p.returning = true; p.homing = p.c.caster; p.speed = sp; p.visual = true; p.applied = []; continue; }
        p.dead = true; runList(p.end, Object.assign({}, p.c, { x: p.x, y: p.y, target: null, fromProj: p }));
      }
    }
    T.projs = T.projs.filter(p => !p.dead);
    T.walls = T.walls.filter(w => w.until > T.tick);
    if (T.zones) T.zones = T.zones.filter(z => z.until > T.tick);
    for (const e of T.ents) for (const n of Object.keys(e.buffs)) if (e.buffs[n] <= T.tick) delete e.buffs[n];
    if (T.gallery) galleryTick();
  }
  function walkTo(e, to, stopAt) {
    const d = dist(e, to); if (d <= (stopAt || 1500)) return true;
    const sp = Math.min(speedOf(e), d - (stopAt || 0));
    const [ux, uy] = norm(to.x - e.x, to.y - e.y);
    const p = blockedEnd(e.x, e.y, e.x + ux * sp, e.y + uy * sp);
    if (p.x === e.x && p.y === e.y) return true;
    e.x = p.x; e.y = p.y; e.face = ux >= 0 ? 1 : -1; e.moving = T.tick; return false;
  }
  function heroAI(h) {
    if (h.act) return;
    const a = T.json.attack;
    if (h.atkTarget && h.atkTarget.hp > 0) {
      const t = h.atkTarget, range = (a && a.range) || 25000;
      if (dist(h, t) > range + t.r * 0.5) walkTo(h, t, range * 0.95);
      else { h.face = t.x >= h.x ? 1 : -1; if ((h.cds.attack || 0) <= T.tick) castSlot('attack', t); }
      return;
    }
    h.atkTarget = null;
    if (h.dest && walkTo(h, h.dest)) h.dest = null;
  }
  function dummyAI(d) {
    const h = hero();
    if (T.opts.fightBack && h && h.hp > 0 && !held(d) && !ccOn(d, 'BlockAttack') && !has(h, '~invisible') && dist(d, h) <= 32000 && T.tick >= d.atkCd) {
      d.atkCd = T.tick + 90; d.face = h.x >= d.x ? 1 : -1; after(10, () => damage(d, h, d.atk, 'phys', d.name));
    }
    if (T.opts.strafe) { const k = Math.sin(T.tick / 50 + d.phase); walkTo(d, { x: d.home.x, y: d.home.y + k * 26000 }, 500); }
  }
  function unitAI(u) {
    const t = dummies().sort((a, b) => d2(a, u) - d2(b, u))[0]; if (!t) return;
    if (dist(u, t) > 22000) walkTo(u, t, 20000);
    else if (T.tick >= u.atkCd) { u.atkCd = T.tick + 70; u.face = t.x >= u.x ? 1 : -1; setAnim(u, 'attack', 20); after(10, () => damage(u, t, u.atk, 'phys', u.name)); }
  }

  // ------------------------------------------------------------------ the gallery (every Scribble spell, one by one)
  // where to put the dummies for each kind of spell (fractions of the arena), and where to aim
  function galleryLayout(i) {
    const [, kind, range] = SPELL_FX[i];
    const r = Math.max(range, 20000) / WW;
    if ([3, 25, 31, 33].includes(i)) return [[0.385, 0.5], [0.35, 0.4], [0.35, 0.6]];
    if (i === 32) return [[0.55, 0.2], [0.7, 0.8], [0.62, 0.5]];
    if (i === 30) return [[0.5, 0.5], [0.62, 0.47], [0.74, 0.53]];
    if (kind === 'e') { const fx = 0.3 + Math.min(r * 0.7, 0.45); return [[fx, 0.5], [fx + 0.04, 0.62], [fx + 0.03, 0.38]]; }
    return [[0.62, 0.5], [0.7, 0.36], [0.7, 0.66]];
  }
  function startGallery(list) {
    T.gallery = { list: list.slice(), cur: null, phase: 'idle', wait: 0, dealt: 0, results: [] };
    galleryNext();
  }
  function galleryNext() {
    const g = T.gallery; if (!g) return;
    if (!g.list.length) { T.galleryDone = g.results; T.gallery = null; renderSide(); return; }
    const i = g.list.shift();
    resetArena(galleryLayout(i));
    const h = hero(); h.x = WW * 0.3; h.y = WH * 0.5; h.face = 1;
    T.scr.ready.fill(0);
    g.cur = i; g.dealt = 0; g.castAt = 0; g.startTick = T.tick;
    const d = dummies()[0];
    const aim = SPELL_FX[i][1] === 'd' ? { x: h.x + 60000, y: h.y } : SPELL_FX[i][1] === 'p' ? { x: d.x - 8000, y: d.y } : { x: d.x, y: d.y };
    T.demoRun = true;
    after(20, () => castRecipe(i, aim));
    renderSide();
  }
  function galleryTick() {
    const g = T.gallery; if (!g || g.cur == null) return;
    const longest = { 9: 300, 15: 90, 19: 150, 20: 240, 23: 150, 26: 300, 28: 100, 30: 240, 33: 140, 34: 300 }[g.cur] || 90;
    if (g.castAt && T.tick >= g.castAt + longest) {
      g.results.push([g.cur, g.dealt]);
      g.cur = null; after(30, galleryNext);
    } else if (!g.castAt && T.tick > g.startTick + 600) { g.cur = null; galleryNext(); }
  }

  // ------------------------------------------------------------------ drawing
  const sx = x => x / UPX * T.zoom, sy = y => y / UPX * T.zoom;
  function frameAt(frames, t0, loop) {
    if (!frames || !frames.length) return null;
    const total = frames.reduce((a, f) => a + f.d, 0);
    let t = performance.now() - t0; if (loop) t %= total; else if (t >= total) return null;
    for (const f of frames) { if (t < f.d) return f; t -= f.d; }
    return frames[frames.length - 1];
  }
  function blit(sh, f, x, y, flip, alpha, ang) {
    if (!sh || !sh.img || !f) return;
    const c = T.ctx, z = T.zoom;
    c.save(); c.globalAlpha = alpha == null ? 1 : alpha; c.translate(Math.round(sx(x)), Math.round(sy(y)));
    if (ang) c.rotate(ang);
    if (flip) c.scale(-1, 1);
    c.drawImage(sh.img, f.x, f.y, f.w, f.h, -f.w * z / 2, -f.h * z / 2, f.w * z, f.h * z);
    c.restore();
  }
  function drawDummy(d) {
    const c = T.ctx, z = T.zoom, x = sx(d.x), y = sy(d.y);
    const hit = d.hitFlash > T.tick, down = d.hp <= 0;
    c.save(); c.translate(Math.round(x), Math.round(y)); c.scale(z, z);
    if (down) c.rotate(Math.PI / 2.4);
    c.fillStyle = '#6b4a2b'; c.fillRect(-1, -2, 2, 14);                 // post
    c.fillStyle = '#4a3420'; c.fillRect(-5, 11, 10, 2);                 // base
    c.fillStyle = hit ? '#fff2c2' : '#d8b26a'; c.fillRect(-6, -10, 12, 12); // straw body
    c.fillStyle = '#b8914c'; c.fillRect(-6, -10, 12, 1); c.fillRect(-6, -1, 12, 1);
    c.fillStyle = '#c23b3b'; c.beginPath(); c.arc(0, -4, 3.5, 0, Math.PI * 2); c.fill();
    c.fillStyle = '#f4f1e6'; c.beginPath(); c.arc(0, -4, 2.2, 0, Math.PI * 2); c.fill();
    c.fillStyle = '#c23b3b'; c.fillRect(-0.7, -4.7, 1.4, 1.4);
    c.fillStyle = hit ? '#fff2c2' : '#e2c27e'; c.beginPath(); c.arc(0, -14, 4, 0, Math.PI * 2); c.fill(); // head
    c.fillStyle = '#2a2a2a'; c.fillRect(-2, -15, 1, 1); c.fillRect(1, -15, 1, 1);
    c.restore();
  }
  function bar(e) {
    const c = T.ctx, w = 16 * T.zoom, x = sx(e.x) - w / 2, y = sy(e.y) - (e.kind === 'dummy' ? 21 : 26) * T.zoom;
    c.fillStyle = 'rgba(0,0,0,.6)'; c.fillRect(x - 1, y - 1, w + 2, 5);
    c.fillStyle = e.team === 0 ? '#5fd17a' : '#e25b5b'; c.fillRect(x, y, w * Math.max(0, e.hp) / e.maxhp, 3);
    const sh = e.shields.filter(s => s.until > T.tick).reduce((a, s) => a + s.amt, 0);
    if (sh > 0) { c.fillStyle = '#e6f1ff'; c.fillRect(x, y + 3, Math.min(w, w * sh / e.maxhp), 1.5); }
    const ccs = Object.keys(e.cc).filter(k => e.cc[k] > T.tick);
    if (ccs.length) { c.fillStyle = '#ffe066'; c.font = `bold ${9}px system-ui`; c.textAlign = 'center'; c.fillText(ccs.map(k => CC_LABEL[k] || k).join(' · '), sx(e.x), y - 4); }
  }
  function draw() {
    const c = T.ctx, cv = T.canvas; if (!c) return;
    c.imageSmoothingEnabled = false;
    c.fillStyle = '#3d5a45'; c.fillRect(0, 0, cv.width, cv.height);
    c.strokeStyle = 'rgba(255,255,255,.05)'; c.lineWidth = 1;
    for (let gx = 0; gx <= WW; gx += 20000) { c.beginPath(); c.moveTo(sx(gx), 0); c.lineTo(sx(gx), cv.height); c.stroke(); }
    for (let gy = 0; gy <= WH; gy += 20000) { c.beginPath(); c.moveTo(0, sy(gy)); c.lineTo(cv.width, sy(gy)); c.stroke(); }
    if (isScribble()) { c.fillStyle = 'rgba(255,255,255,.04)'; c.fillRect(0, sy(WH / 2 - 20000), cv.width, sy(40000)); }
    const h = hero();
    // aim helpers
    if (h && h.hp > 0) {
      const a = T.json.attack; c.strokeStyle = 'rgba(255,255,255,.12)'; c.setLineDash([4, 4]);
      c.beginPath(); c.arc(sx(h.x), sy(h.y), sx((a && a.range) || 25000), 0, Math.PI * 2); c.stroke(); c.setLineDash([]);
      if (h.dest) { c.strokeStyle = 'rgba(120,255,160,.5)'; c.beginPath(); c.arc(sx(h.dest.x), sy(h.dest.y), 5, 0, Math.PI * 2); c.stroke(); }
    }
    for (const z of T.zones || []) { c.strokeStyle = z.ring ? 'rgba(185,243,255,.6)' : 'rgba(255,224,102,.25)'; c.beginPath(); c.arc(sx(z.x), sy(z.y), sx(z.r), 0, Math.PI * 2); c.stroke(); }
    for (const w of T.walls) { c.strokeStyle = 'rgba(255,255,255,.25)'; c.lineWidth = 2; c.beginPath(); c.moveTo(sx(w.ax), sy(w.ay)); c.lineTo(sx(w.bx), sy(w.by)); c.stroke(); c.lineWidth = 1; }
    // ground effects, then units by depth, then the rest
    const fxNow = T.fxs.filter(f => {
      const sh = T.sheets[f.anim]; const fr = sh && sh.anims[f.tag];
      if (!fr) return false;
      const total = fr.reduce((a, x) => a + x.d, 0);
      return f.loopUntil ? T.tick < f.loopUntil : performance.now() - f.t0 < total;
    });
    T.fxs = fxNow;
    const drawFx = f => { const sh = T.sheets[f.anim]; const p = f.follow || f; blit(sh, frameAt(sh.anims[f.tag], f.t0, !!f.loopUntil), p.x, p.y, false); };
    fxNow.filter(f => f.z < 0).forEach(drawFx);
    const order = T.ents.slice().sort((a, b) => a.y - b.y);
    for (const e of order) {
      if (e.kind === 'dummy') { drawDummy(e); if (e.hp > 0) bar(e); continue; }
      const sp = e.kind === 'hero' ? T.sprite : T.sheets['bundle:' + e.sprite];
      let tag = 'idle', loop = true, t0 = 0;
      if (e.hp <= 0) { tag = 'dead'; loop = false; t0 = e.deadAt || (e.deadAt = performance.now()); }
      else if (e.anim && T.tick < e.anim.until && sp && sp.anims[e.anim.tag]) { tag = e.anim.tag; loop = e.anim.loop || true; t0 = e.anim.t0; }
      else if (held(e) && sp && sp.anims.hit) tag = 'hit';
      else if (e.moving && T.tick - e.moving < 3) tag = 'run';
      const frames = sp && (sp.anims[tag] || sp.anims.idle);
      const alpha = has(e, '~invisible') ? 0.35 : 1;
      // buff visuals bound in the data (Scribble's dots, badge and skin, Omen's gun, ...): z < 0 behind the body
      const bufs = Object.keys(e.buffs).map(n => T.bind['bf:' + n]).filter(b => b && T.sheets[b.anim]).sort((x, y) => (x.z || 0) - (y.z || 0));
      const drawBuf = b => { const sh = T.sheets[b.anim]; const fr = sh.anims[b.tag]; if (fr) blit(sh, frameAt(fr, 0, true), e.x, e.y, false); };
      bufs.filter(b => (b.z || 0) < 0).forEach(drawBuf);
      if (frames) blit(sp, frameAt(frames, t0, tag !== 'dead' ? true : false) || frames[frames.length - 1], e.x, e.y, e.face < 0, alpha);
      else { c.fillStyle = '#9ad'; c.beginPath(); c.arc(sx(e.x), sy(e.y), 6 * T.zoom, 0, Math.PI * 2); c.fill(); }
      bufs.filter(b => (b.z || 0) >= 0).forEach(drawBuf);
      if (e.hp > 0) bar(e);
    }
    fxNow.filter(f => f.z >= 0).forEach(drawFx);
    for (const p of T.projs) {
      const b = T.bind['pj:' + p.name]; const sh = b && T.sheets[b.anim];
      if (sh && sh.anims[b.tag]) blit(sh, frameAt(sh.anims[b.tag], 0, true), p.x, p.y, false, 1, p.arc ? 0 : p.ang || 0);
      else { c.fillStyle = '#fff'; c.beginPath(); c.arc(sx(p.x), sy(p.y), 2 * T.zoom, 0, Math.PI * 2); c.fill(); }
    }
    // floating numbers
    T.texts = T.texts.filter(t => T.tick - t.t0 < 70);
    c.textAlign = 'center';
    for (const t of T.texts) {
      const k = (T.tick - t.t0) / 70;
      c.globalAlpha = 1 - k * k; c.font = `bold ${t.big ? 15 : 11}px system-ui`;
      c.lineWidth = 3; c.strokeStyle = 'rgba(0,0,0,.7)'; c.strokeText(t.text, sx(t.x), sy(t.y) - k * 30);
      c.fillStyle = t.col; c.fillText(t.text, sx(t.x), sy(t.y) - k * 30);
    }
    c.globalAlpha = 1;
    // Scribble: the cast bar
    if (h && T.scr && T.scr.invoking) {
      const k = 1 - (T.scr.invoking.at - T.tick) / INVOKE_T[T.opts.rank];
      c.fillStyle = 'rgba(0,0,0,.6)'; c.fillRect(sx(h.x) - 30, sy(h.y) + 18 * T.zoom, 60, 5);
      c.fillStyle = '#ffe066'; c.fillRect(sx(h.x) - 30, sy(h.y) + 18 * T.zoom, 60 * k, 5);
    }
    if (T.gallery && T.gallery.cur != null) {
      const b = BOOK()[T.gallery.cur];
      c.fillStyle = 'rgba(0,0,0,.55)'; c.fillRect(0, 0, cv.width, 30);
      c.fillStyle = '#fff'; c.font = 'bold 15px system-ui'; c.textAlign = 'left';
      c.fillText(`${T.gallery.cur + 1}. ${b[1]}   ${b[0]}   (${secs(b[2])})`, 12, 20);
      c.font = '12px system-ui'; c.fillStyle = '#cfd8e3'; c.fillText(b[3], 12 + c.measureText(`${T.gallery.cur + 1}. ${b[1]}   ${b[0]}   (${secs(b[2])})`).width + 260, 20);
    }
  }

  // ------------------------------------------------------------------ loop
  function frame(now) {
    T.raf = requestAnimationFrame(frame);
    if (!T.canvas || !T.canvas.isConnected || $('#tab-test').hidden) return;
    const dt = Math.min(100, now - (T.last || now)); T.last = now;
    if (T.running && T.json) { T.acc += dt * T.speed; while (T.acc >= 1000 / TPS) { step(); T.acc -= 1000 / TPS; } }
    draw();
    if (T.tick % 15 === 0) renderStatus();
  }

  // ------------------------------------------------------------------ input
  function toWorld(ev) { const r = T.canvas.getBoundingClientRect(); return { x: (ev.clientX - r.left) * (T.canvas.width / r.width) / T.zoom * UPX, y: (ev.clientY - r.top) * (T.canvas.height / r.height) / T.zoom * UPX }; }
  function wireCanvas() {
    const cv = T.canvas;
    cv.addEventListener('contextmenu', ev => ev.preventDefault());
    cv.addEventListener('mousemove', ev => { T.mouse = toWorld(ev); });
    cv.addEventListener('mousedown', ev => {
      const p = toWorld(ev); T.mouse = p; cv.focus();
      const h = hero(); if (!h) return;
      const d = T.ents.filter(e => e.team !== h.team && e.hp > 0 && d2(e, p) <= 14000 ** 2).sort((a, b) => d2(a, p) - d2(b, p))[0];
      if (ev.button === 2) { if (d) { h.atkTarget = d; h.dest = null; } else { h.dest = p; h.atkTarget = null; } }
      else if (ev.button === 0 && d) { h.atkTarget = d; h.dest = null; }
    });
    cv.addEventListener('keydown', ev => {
      const k = ev.key.toLowerCase();
      if (ev.ctrlKey || ev.metaKey || ev.altKey) return;
      const h = hero(); if (!h) return;
      const extra = { r: 1, f: 2, t: 3, g: 4, y: 5, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5 };
      if (isScribble()) {
        if (extra[k]) weave(extra[k]);
        else if (k === 'e') invoke();
        else if (k === 'q') { T.scr.dots = []; T.scr.queue = []; fxS('weave', h); }
        else if (k === 'w') { if (T.scr.last != null) castRecipe(T.scr.last); else say(h, 'nothing cast yet', '#9aa7b4'); }
        else if (k === 's') { h.dest = null; h.atkTarget = null; }
        else return;
        if (k === 'e' && T.scr.dots.length) T.scr.last = recipeIndex(T.scr.dots) >= 0 ? recipeIndex(T.scr.dots) : T.scr.last;
      } else {
        if (SLOT_KEYS[k]) castSlot(SLOT_KEYS[k]);
        else if (k === 's') { h.dest = null; h.atkTarget = null; }
        else if (extra[k]) say(h, 'no extra active', '#9aa7b4');
        else return;
      }
      ev.preventDefault();
    });
  }

  // ------------------------------------------------------------------ UI
  async function loadChamps() {
    const list = [];
    const seen = new Set();
    try {
      const j = await (await fetch('/api/mods', { cache: 'no-store' })).json();
      for (const m of j.mods || []) {
        if (!m.info || m.info.mod_type === 'native' || m.info.contains_code) continue;
        const txt = (m.text && m.text.en && m.text.en.description) || {};
        for (const c of m.champions || []) if (c.json && !seen.has(c.json.id)) { seen.add(c.json.id); list.push({ id: c.json.id, json: c.json, text: txt[c.json.id] || {}, src: m.info.name || m.id }); }
      }
    } catch (e) { /* no server: presets only */ }
    for (const [k, p] of Object.entries(window.TFM2_PRESETS || {})) {
      if (/_(slow|fast)$/.test(k)) continue;
      const modId = (p.folder && p.folder[0]) || 'tfm2_custom';
      const id = `${modId}_${p.slug}`.replace('tfm2_jjk_gojo', 'tfm2_custom_gojo');
      if (seen.has(id)) { const x = list.find(c => c.id === id); if (x) x.preset = p; continue; }
      try { const b = p.build(id, { modId }); seen.add(id); list.push({ id, json: b.json, text: b.text, src: 'preset', preset: p, sprite: b.sprite }); } catch (e) { /* skip */ }
    }
    list.sort((a, b) => (a.id.endsWith('scribble') ? -1 : b.id.endsWith('scribble') ? 1 : String(a.text.name || a.id).localeCompare(b.text.name || b.id)));
    T.champs = list;
  }
  async function pick(id) {
    const c = T.champs.find(x => x.id === id) || T.champs[0]; if (!c) return;
    T.sel = c; T.json = c.json; T.text = c.text || {}; T.bind = bindings(c.json);
    T.sprite = await champSprite(c.json, c.sprite);
    await preloadViews(c.json);
    const sh = window.TFM2_SPRITES && window.TFM2_SPRITES.shadow_bombardier;
    if (sh && !T.sheets['bundle:shadow_bombardier']) T.sheets['bundle:shadow_bombardier'] = { img: await img('data:image/png;base64,' + sh.png), anims: animsOf(sh.fanim) };
    resetArena();
    renderSide();
  }
  function controlsHTML() {
    if (isScribble()) return `<div class="st-keys">
      <div><kbd>Right-click</kbd> move · on a dummy: attack</div>
      <div><kbd>R</kbd><kbd>F</kbd><kbd>T</kbd><kbd>G</kbd><kbd>Y</kbd> (or <kbd>1</kbd>-<kbd>5</kbd>) weave ${[1, 2, 3, 4, 5].map(e => `<b style="color:${EL_COL[e]}">${e} ${ELEMENTS[e]}</b>`).join(' ')}</div>
      <div><kbd>E</kbd> Invoke at the cursor · <kbd>Q</kbd> flick the dots away · <kbd>W</kbd> the last spell again · <kbd>S</kbd> stop</div></div>`;
    return `<div class="st-keys"><div><kbd>Right-click</kbd> move · on a dummy: attack</div>
      <div><kbd>Q</kbd> Ability 1 · <kbd>W</kbd> Ability 2 · <kbd>E</kbd> Ultimate (aimed at the cursor; targeted skills pick the dummy nearest the cursor) · <kbd>S</kbd> stop</div>
      <div><kbd>R</kbd><kbd>F</kbd><kbd>T</kbd><kbd>G</kbd><kbd>Y</kbd> extra actives (none for this champion)</div></div>`;
  }
  function renderSide() {
    const L = $('#stLeft'), R = $('#stRight'); if (!L || !R) return;
    const o = T.opts;
    L.innerHTML = `
      <label class="st-row">Champion<select id="stChamp">${T.champs.map(c => `<option value="${esc(c.id)}"${T.sel && T.sel.id === c.id ? ' selected' : ''}>${esc(c.text.name || c.id)} · ${esc(c.src)}</option>`).join('')}</select></label>
      <label class="st-row">Level<input type="number" id="stLevel" min="1" max="18" value="${o.level}"></label>
      ${isScribble() ? `<label class="st-row">Mastery<select id="stRank">${RANK_NAMES.slice(0, TOP).map((n, i) => `<option value="${i}"${o.rank === i ? ' selected' : ''}>${n} (${RANK_GAMES[i]}+ games, ${cps100(i) / 100} CPS)</option>`).join('')}${[...Array(TOP_SIZE)].map((_, k) => TOP_SIZE - k).map(p => `<option value="t${p}"${o.rank === TOP && o.topPos === p ? ' selected' : ''}>Top 10 #${p} (${(cps100(TOP, p) / 100).toFixed(1)} CPS)</option>`).join('')}</select></label>
      <label class="st-check"><input type="checkbox" id="stSlips"${o.slips ? ' checked' : ''}> Slips (wrong dots, like the AI at this rank)</label>` : ''}
      <label class="st-check"><input type="checkbox" id="stCds"${o.cooldowns ? ' checked' : ''}> Cooldowns</label>
      <h4>Dummies</h4>
      <label class="st-row">HP<input type="number" id="stDHp" step="100" value="${o.dummyHp}"></label>
      <label class="st-row">Armor<input type="number" id="stDDef" value="${o.dummyDef}"></label>
      <label class="st-row">Magic resist<input type="number" id="stDMr" value="${o.dummyMr}"></label>
      <label class="st-check"><input type="checkbox" id="stStrafe"${o.strafe ? ' checked' : ''}> Dummies walk (test slow spells)</label>
      <label class="st-check"><input type="checkbox" id="stFight"${o.fightBack ? ' checked' : ''}> Dummies hit back</label>
      <div class="st-btns"><button class="btn small" data-st="reset">Reset arena</button><button class="btn small" data-st="add">+ Dummy</button><button class="btn small" data-st="pause">${T.running ? 'Pause' : 'Play'}</button>
        <select id="stSpeed">${[0.25, 0.5, 1, 2].map(s => `<option value="${s}"${T.speed === s ? ' selected' : ''}>${s}x</option>`).join('')}</select></div>
      ${T.json && T.json.passive && !isScribble() && T.json.passive.passive_ref ? `<p class="muted st-note">Its passive (${esc(T.json.passive.passive_ref)}) is native: what it adds in a match (markers turned into skills, AI tricks) isn't played here. The data part, animations and visuals are.</p>` : ''}
      <h4>Log</h4><div class="st-log" id="stLog"></div>`;
    if (isScribble()) {
      const r = o.rank, known = COMFORT_TIER[r];
      const rows = BOOK().map((b, i) => {
        const tier = b[0].split('-').length;
        const dots = b[0].split('-').map(e => `<i class="st-dot" style="background:${EL_COL[e]}">${e}</i>`).join('');
        return `<div class="st-spell${tier > known ? ' st-unknown' : ''}" title="${esc(b[3])}${tier > known ? ` (past ${RANK_NAMES[r]}'s comfort: built right ${Math.round(buildChance(r, tier) * 100)}% of the time)` : ''}">
          <button class="btn small" data-cast="${i}" title="Weave and invoke it at the dummy">▶</button>
          <span class="st-sname">${esc(b[1])}</span><span class="st-dots">${dots}</span><span class="st-cd" data-cd="${i}">${secs(b[2])}</span></div>`;
      }).join('');
      R.innerHTML = `<div class="st-rhead"><strong>Spell book</strong>
        <button class="btn small primary" data-st="gallery">${T.gallery ? 'Stop gallery' : '▶ Play all 35'}</button></div>
        <p class="muted st-note">${known >= 6 ? `${RANK_NAMES[r]} builds every recipe reliably.` : `Greyed: past ${RANK_NAMES[r]}'s comfort (${known} dots). Any rank can go for it, but each dot past it slips far more often (hover a spell for the odds).`} ▶ weaves at this rank's speed (${(cps100(r, o.topPos) / 100).toFixed(1)} dots a second, invoke ${INVOKE_T[r]} ticks).</p>
        ${T.gallery ? `<div class="st-gal">Gallery: ${35 - T.gallery.list.length}/35</div>` : ''}
        <div class="st-book">${rows}</div>
        ${galleryResults()}`;
    } else {
      const slots = [['attack', 'Basic attack', 'right-click'], ['skill', 'Ability 1', 'Q'], ['skill2', 'Ability 2', 'W'], ['ult', 'Ultimate', 'E']];
      R.innerHTML = `<div class="st-rhead"><strong>${esc(T.text.name || T.json.id)}</strong></div>` + slots.map(([s, n, k]) => {
        const a = T.json[s] || {};
        return `<div class="st-slot"><div><kbd>${k}</kbd> <b>${n}</b> <span class="muted">${a.action_name ? 'anim ' + esc(a.action_name) : ''} · cd ${secs(a.cooltime || 0)} · range ${a.range || 0}</span></div>
          <div class="muted st-desc">${esc((T.text[s] || '').slice(0, 420))}${(T.text[s] || '').length > 420 ? '…' : ''}</div></div>`;
      }).join('');
    }
    $('#stControls').innerHTML = controlsHTML();
    renderStatus();
  }
  function galleryResults() {
    const g = T.galleryDone; if (!g || !g.length) return '';
    return `<h4>Last gallery: damage per spell</h4><div class="st-res">${g.map(([i, d]) => `<span>${esc(BOOK()[i][1])}: <b>${d}</b></span>`).join('')}</div>`;
  }
  function renderStatus() {
    const s = $('#stStatus'); if (!s || !T.json) return;
    const h = hero(); if (!h) return;
    const ds = T.ents.filter(e => e.kind === 'dummy');
    let txt = `<b>${esc(h.name)}</b> HP ${Math.round(h.hp)}/${h.maxhp} · AD ${h.atk} · AP ${h.ap} · ${(T.tick / 60).toFixed(1)}s`;
    if (isScribble()) {
      const sc = T.scr; const dots = sc.dots.map(e => `<i class="st-dot" style="background:${EL_COL[e]}">${e}</i>`).join('') || '<span class="muted">no dots</span>';
      const m = recipeIndex(sc.dots);
      txt += ` · dots ${dots} ${sc.dots.length ? (m >= 0 ? `→ <b>${esc(BOOK()[m][1])}</b>` : '<span style="color:#ff9a9a">no recipe</span>') : ''}${sc.misfires ? ` · slips ${sc.misfires}` : ''}`;
      document.querySelectorAll('[data-cd]').forEach(el => { const i = +el.dataset.cd; const left = sc.ready[i] - T.tick; el.textContent = left > 0 && T.opts.cooldowns ? secs(left) : secs(BOOK()[i][2]); el.classList.toggle('st-oncd', left > 0 && T.opts.cooldowns); });
    } else {
      txt += ' · ' + ['skill', 'skill2', 'ult'].map((k, i) => `${'QWE'[i]} ${Math.max(0, (h.cds[k] || 0) - T.tick) > 0 ? secs(h.cds[k] - T.tick) : 'ready'}`).join(' · ');
    }
    txt += ' · dummies: ' + ds.map(d => `${d.name} ${Math.round(d.dmgTaken || 0)} dmg`).join(', ');
    s.innerHTML = txt;
    const lg = $('#stLog'); if (lg) lg.textContent = T.log.slice(0, 40).join('\n');
  }
  function onSideClick(ev) {
    const b = ev.target.closest('[data-st],[data-cast],[data-mem]'); if (!b) return;
    if (b.dataset.cast != null) {
      if (T.gallery) T.gallery = null;
      const i = +b.dataset.cast; resetArena(galleryLayout(i)); const h = hero(); h.x = WW * 0.3; h.y = WH * 0.5;
      const d = dummies()[0]; const k = SPELL_FX[i][1];
      T.demoRun = true;
      castRecipe(i, k === 'd' ? { x: h.x + 60000, y: h.y } : k === 'p' ? { x: d.x - 8000, y: d.y } : { x: d.x, y: d.y });
      T.canvas.focus(); return;
    }
    const a = b.dataset.st;
    if (a === 'reset') resetArena();
    if (a === 'add') { const n = T.ents.filter(e => e.kind === 'dummy').length; T.ents.push(makeDummy(WW * (0.5 + Math.random() * 0.4), WH * (0.2 + Math.random() * 0.6), n)); }
    if (a === 'pause') { T.running = !T.running; renderSide(); }
    if (a === 'gallery') {
      if (T.gallery) { T.galleryDone = T.gallery.results; T.gallery = null; }
      else startGallery([...Array(35).keys()]);
      renderSide();
    }
    if (a === 'arena' || a === 'memory') { T.view = a; renderView(); }
    T.canvas && T.canvas.focus();
  }
  function onSideChange(ev) {
    const t = ev.target, o = T.opts;
    if (t.id === 'stChamp') return pick(t.value);
    if (t.id === 'stLevel') { o.level = clamp(+t.value || 1, 1, 18); resetArena(); }
    if (t.id === 'stRank') { if (t.value[0] === 't') { o.rank = TOP; o.topPos = +t.value.slice(1); } else { o.rank = +t.value; o.topPos = null; } renderSide(); }
    if (t.id === 'stSlips') o.slips = t.checked;
    if (t.id === 'stCds') o.cooldowns = t.checked;
    if (t.id === 'stDHp') { o.dummyHp = Math.max(1, +t.value || 1); for (const d of T.ents.filter(e => e.kind === 'dummy')) { d.maxhp = o.dummyHp; d.hp = Math.min(d.hp, d.maxhp); } }
    if (t.id === 'stDDef') { o.dummyDef = +t.value || 0; T.ents.filter(e => e.kind === 'dummy').forEach(d => { d.def = o.dummyDef; }); }
    if (t.id === 'stDMr') { o.dummyMr = +t.value || 0; T.ents.filter(e => e.kind === 'dummy').forEach(d => { d.mr = o.dummyMr; }); }
    if (t.id === 'stStrafe') o.strafe = t.checked;
    if (t.id === 'stFight') o.fightBack = t.checked;
    if (t.id === 'stSpeed') T.speed = +t.value;
    T.canvas && T.canvas.focus();
  }

  // ------------------------------------------------------------------ Scribble memory page
  // The same rules as scribble.rs Memory::merge: an official match (with a match id) counts 1, a scrim / exhibition
  // ("x." signatures) 0.5, a win x1.5 for mastery and x1.25 for the weight of that game's casts in the meta.
  const OFFICIAL_W = 1, SCRIM_W = 0.5, WIN_MASTERY = 1.5, WIN_META = 1.25, META_K = 12;
  const gameKey = sig => String(sig).split('.').pop();
  const isOfficial = sig => !String(sig).startsWith('x.');
  function resultOf(r) {
    const [mt, et, mn, en, sd] = r;
    if (mn !== en) return en < mn; if (mt !== et) return et < mt; if (sd) return sd > 0; return null;
  }
  async function loadMemory() {
    try { const r = await fetch('/api/scribble', { cache: 'no-store' }); T.mem = r.ok ? await r.json() : { error: 'The editor server has no Scribble files yet (start it from the game folder).' }; }
    catch (e) { T.mem = { error: 'Start the editor with its server (Start Editor.bat) to see the memory.' }; }
  }
  function parseMem(text) {
    const m = { world: 0, games: {}, meta: {} };
    for (const line of String(text || '').split(/\r?\n/)) {
      const f = line.trim().split(/\s+/);
      if (f[0] === 'W') m.world = +f[1] || 0;
      if (f[0] === 'G' && f.length === 3) m.games[f[1]] = { points: +f[2] || 0, games: Math.floor(+f[2] || 0), wins: 0 };
      if (f[0] === 'G' && f.length === 5) m.games[f[1]] = { points: +f[2] || 0, games: +f[3] || 0, wins: +f[4] || 0 };
      if (f[0] === 'M' && f.length === 5) m.meta[`${f[1]}:${f[2]}`] = [+f[3] || 0, +f[4] || 0];
    }
    return m;
  }
  // per-game facts from pending lines: results (last structure record), summaries (latest), casts
  function gameFacts(text) {
    const res = {}, sum = {};
    for (const line of String(text || '').split(/\r?\n/)) {
      const f = line.trim().split(/\s+/);
      if (f[0] === 'r' && f.length === 9) { const k = gameKey(f[1]) + ' ' + f[2]; const t = +f[3]; if (!res[k] || t >= res[k].t) res[k] = { t, r: f.slice(4, 9).map(Number) }; }
      if (f[0] === 's' && f.length >= 11) { const k = gameKey(f[1]) + ' ' + f[2]; const t = +f[3]; if (!sum[k] || t >= sum[k].t) sum[k] = { sig: f[1], a: f[2], t, rank: +f[4], casts: +f[5], misfires: +f[6], fizzles: +f[7], kind: +f[8], official: f[9] === '1', top: f[10] }; }
    }
    return { res, sum };
  }
  function merge(m, text) {
    const { res } = gameFacts(text);
    const won = (sig, a) => { const r = res[gameKey(sig) + ' ' + a]; return r ? resultOf(r.r) === true : false; };
    const sg = new Set(), sw = new Set(), sc = new Set(); let casts = 0, games = 0;
    for (const line of String(text || '').split(/\r?\n/)) {
      const f = line.trim().split(/\s+/);
      if (f[0] === 'g' && f.length === 3) {
        const key = gameKey(f[1]);
        if (!sg.has(key + ' ' + f[2])) { sg.add(key + ' ' + f[2]); games++;
          const w = won(f[1], f[2]); const p = m.games[f[2]] || (m.games[f[2]] = { points: 0, games: 0, wins: 0 });
          p.points += (isOfficial(f[1]) ? OFFICIAL_W : SCRIM_W) * (w ? WIN_MASTERY : 1); p.games++; if (w) p.wins++; p.pending = (p.pending || 0) + 1; }
        if (!sw.has(key)) { sw.add(key); m.world++; }
      }
      if (f[0] === 'c' && f.length === 7) {
        const k = gameKey(f[1]) + ' ' + f[2] + ' ' + f[3]; if (sc.has(k)) continue; sc.add(k); casts++;
        const w = (isOfficial(f[1]) ? OFFICIAL_W : SCRIM_W) * (won(f[1], f[2]) ? WIN_META : 1);
        const e = m.meta[`${f[4]}:${f[5]}`] || (m.meta[`${f[4]}:${f[5]}`] = [0, 0]);
        e[0] += Math.max(0, Math.min(3, +f[6] || 0)) * w; e[1] += w;
      }
    }
    return { casts, games };
  }
  const BUCKET = ['1 enemy', '1 enemy, me low', '1 enemy, ally low', '2 enemies', '2, me low', '2, ally low', '3+ enemies', '3+, me low', '3+, ally low'];
  function renderMemory() {
    const box = $('#stMemory'); if (!box) return;
    const M = T.mem;
    if (!M) { box.innerHTML = '<p class="muted">Loading…</p>'; loadMemory().then(renderMemory); return; }
    if (M.error) { box.innerHTML = `<p class="muted">${esc(M.error)}</p>`; return; }
    const mem = parseMem(M.memory), saved = { world: mem.world };
    const waiting = merge(mem, M.pending);
    const names = (window.TFM2_APP && window.TFM2_APP.athleteName) || (() => null);
    const ids = Object.keys(mem.games).filter(a => +a < 1000000).sort((x, y) => mem.games[y].points - mem.games[x].points);
    const top = topTen(mem.games);
    const rows = ids.slice(0, 400).map(a => {
      const p = mem.games[a], pos = top.indexOf(a) + 1, r = pos ? TOP : rankOf(Math.floor(p.points));
      const next = RANK_GAMES[r + 1];
      const label = pos ? `<b style="color:#f2c14e">Top 10 #${pos}</b>` : `<b>${RANK_NAMES[r]}</b>`;
      const hint = pos ? '' : next ? ` <span class="muted">${(next - p.points).toFixed(1)} to ${RANK_NAMES[r + 1]}</span>`
        : p.points < TOP_POINTS ? ` <span class="muted">${(TOP_POINTS - p.points).toFixed(1)} to Top 10 eligibility</span>` : ' <span class="muted">eligible, outside the ten</span>';
      return `<tr><td>${esc(names(+a) || 'athlete ' + a)}</td><td>${p.points.toFixed(1)}</td><td>${p.games}${p.pending ? ` <span class="muted">(${p.pending} this launch)</span>` : ''}</td><td>${p.wins}</td>
        <td>${label}${hint}</td>
        <td><input type="number" min="0" step="0.5" value="${p.points.toFixed(1)}" data-games="${esc(a)}" style="width:70px"> <button class="btn small" data-mem="set" data-a="${esc(a)}">Set</button> <button class="btn small" data-mem="forget" data-a="${esc(a)}">Reset</button></td></tr>`;
    }).join('');
    const meta = BOOK().map((b, i) => {
      let tw = 0;
      const cells = BUCKET.map((_, k) => { const [sum, c] = mem.meta[`${i}:${k}`] || [0, 0]; tw += c; const f = Math.max(0.35, Math.min(1.8, (META_K + sum) / (META_K + c)));
        const col = f > 1.05 ? `rgba(95,209,122,${Math.min(0.8, (f - 1) * 1.2)})` : f < 0.95 ? `rgba(226,91,91,${Math.min(0.8, (1 - f) * 1.4)})` : 'transparent';
        return `<td style="background:${col}" title="${c.toFixed(1)} weighted casts, delivered ${c ? (sum / c).toFixed(2) : '-'} of what it promised">${c ? f.toFixed(2) : '·'}</td>`; }).join('');
      return `<tr><td>${esc(b[1])}</td><td class="muted">${tw ? tw.toFixed(0) : ''}</td>${cells}</tr>`;
    }).join('');
    // per game: the latest summary line of each, with its result
    const all = String(M.history || '') + '\n' + String(M.pending || '');
    const { res, sum } = gameFacts(all);
    const games = Object.entries(sum).sort((x, y) => (y[1].sig > x[1].sig ? 1 : -1)).slice(0, 60).map(([k, g]) => {
      const r = res[k] ? resultOf(res[k].r) : null;
      const top = g.top === '-' ? '' : g.top.split(',').map(x => x.split(':').map(Number)).sort((x, y) => y[1] - x[1]).slice(0, 4).map(([i, n]) => `${esc(BOOK()[i] ? BOOK()[i][1] : i)} ${n}`).join(', ');
      return `<tr><td>${esc(g.sig.split('.')[0] === 'x' ? 'scrim' : 'match ' + g.sig.split('.')[0])}</td><td>${esc(names(+g.a) || 'athlete ' + g.a)}</td><td>${RANK_NAMES[g.rank] || ''}</td>
        <td>${r === true ? '<b style="color:#5fd17a">won</b>' : r === false ? '<span style="color:#ef6a6a">lost</span>' : '?'}</td><td>${(g.t / 60 / 60).toFixed(1)} min</td><td>${g.casts}</td><td>${g.misfires}</td><td>${g.fizzles}</td><td>${top}</td></tr>`;
    }).join('');
    box.innerHTML = `
      <div class="st-rhead"><strong>Scribble memory</strong> <span class="muted">${mem.world} games learned from (${saved.world} saved + ${waiting.games} this launch, ${waiting.casts} casts not merged yet)</span>
        <div class="spacer"></div><button class="btn small" data-mem="seed" title="Give every player in the open save a random mastery rank on a bell curve you set, with players you pin to a rank and a Top 10 you pick">Randomize pro mastery…</button><button class="btn small" data-mem="reload">Reload</button>
        <button class="btn small danger" data-mem="reset-meta">Reset the meta</button><button class="btn small danger" data-mem="reset-all">Reset everything</button></div>
      ${T.seedOpen ? seedPanel() : ''}
      <p class="muted">Shown here: the saved memory plus the games waiting in scribble_pending.txt, counted the way the game will. In the game, a match also uses the games already played in the same launch; the file is merged when the game starts. Official matches count 1, scrims and exhibitions 0.5, a win 1.5x. Resets keep a backup in editor/backups/scribble.${M.gameRunning ? ' <b style="color:#ff9a9a">The game is running: it keeps writing new games.</b>' : ''}</p>
      <h4>Mastery per athlete (${ids.length})</h4>
      ${rows ? `<div class="st-metawrap"><table class="st-table"><tr><th>Athlete</th><th>Points</th><th>Games</th><th>Wins</th><th>Rank</th><th></th></tr>${rows}</table></div>` : '<p class="muted">No athlete has played him yet.</p>'}
      <p class="muted">Ranks (points): ${RANK_NAMES.slice(0, TOP).map((n, i) => `${n} ${RANK_GAMES[i]}+`).join(' · ')} · Top 10: the ten with the most points among those with ${TOP_POINTS}+. Add: <input type="number" id="stNewAth" placeholder="athlete id" style="width:110px"> <button class="btn small" data-mem="add">Add athlete</button></p>
      <h4>Learned meta (delivered / promised, per situation; 1.00 = as promised)</h4>
      <div class="st-metawrap"><table class="st-table st-meta"><tr><th>Spell</th><th>Casts</th>${BUCKET.map(b => `<th>${b}</th>`).join('')}</tr>${meta}</table></div>
      <h4>Recent games</h4>
      ${games ? `<div class="st-metawrap"><table class="st-table"><tr><th>Game</th><th>Athlete</th><th>Rank</th><th>Result</th><th>Seen</th><th>Casts</th><th>Slips</th><th>Fizzles</th><th>Most cast</th></tr>${games}</table></div>` : '<p class="muted">No per-game summaries yet (written by native 0.7.10+).</p>'}`;
  }
  // ------------------------------------------------------------------ seeding the pros' mastery
  // round 73/76 (Rian): every player in the open save gets a random rank on a bell curve (normal over the ranks: a
  // mean rank, a spread in ranks, a highest rank), then random points inside that rank's band. Players can be pinned to
  // a rank, and the Top 10 can be picked by hand (#1 first: 400 points down to 310, so they are the ten with the most).
  const SEED_BANDS = [[0, 4.5], [5, 14.5], [15, 29.5], [30, 59.5], [60, 99.5], [100, 149.5], [150, 299.5]];
  const SEED_DEFAULT = { mean: 2, sd: 1, max: 4, fixed: [], top: Array(TOP_SIZE).fill('') };
  const seedKey = 'tfm2.scribble.seed';
  function seedLoad() {
    if (T.seed) return T.seed;
    let v = null; try { v = JSON.parse(localStorage.getItem(seedKey) || 'null'); } catch (e) { /* none */ }
    T.seed = Object.assign({}, SEED_DEFAULT, v || {});
    T.seed.top = Array.from({ length: TOP_SIZE }, (_, i) => (T.seed.top || [])[i] || '');
    return T.seed;
  }
  function seedSave() { try { localStorage.setItem(seedKey, JSON.stringify(T.seed)); } catch (e) { /* private window */ } }
  function gauss() { let u = 0, v = 0; while (!u) u = Math.random(); while (!v) v = Math.random(); return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v); }
  const seedPlayers = () => (window.TFM2_APP && window.TFM2_APP.athletes && window.TFM2_APP.athletes()) || [];
  const seedLabel = x => `${x.name} · ${x.id}`;
  // a typed pick: "Name · id" from the list, or a bare name / id
  function seedFind(text, all) {
    const t = String(text || '').trim(); if (!t) return null;
    const m = t.match(/·\s*(\d+)\s*$/) || t.match(/^(\d+)$/);
    if (m) return all.find(x => x.id === +m[1]) || null;
    return all.find(x => String(x.name || '').trim().toLowerCase() === t.toLowerCase()) || null;
  }
  // share of players per rank for a mean / spread / highest rank (the normal curve, rounded and clamped)
  function seedShares(mean, sd, max) {
    const cdf = z => 0.5 * (1 + erf(z / Math.SQRT2));
    function erf(x) { const s = Math.sign(x); x = Math.abs(x); const t = 1 / (1 + 0.3275911 * x);
      return s * (1 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * Math.exp(-x * x)); }
    return Array.from({ length: max + 1 }, (_, r) => {
      const lo = r === 0 ? -Infinity : (r - 0.5 - mean) / sd, hi = r === max ? Infinity : (r + 0.5 - mean) / sd;
      return cdf(hi) - cdf(lo);
    });
  }
  function seedPanel() {
    const S = seedLoad(), all = seedPlayers();
    const opts = (sel, n) => RANK_NAMES.slice(0, n).map((x, i) => `<option value="${i}"${sel === i ? ' selected' : ''}>${x}</option>`).join('');
    const shares = seedShares(S.mean, Math.max(0.1, S.sd), S.max);
    const n = all.length;
    const fixed = S.fixed.map((f, i) => `<div style="display:flex;gap:6px;align-items:center;margin:3px 0"><input list="sdNames" data-sd="fwho" data-i="${i}" value="${esc(f.who)}" placeholder="player" style="width:200px">
        <select data-sd="frank" data-i="${i}">${opts(f.rank, TOP)}</select> <button class="btn small" data-mem="sd-del" data-i="${i}">✕</button></div>`).join('');
    const top = S.top.map((w, i) => `<label style="display:flex;gap:6px;align-items:center;margin:3px 0"><span style="width:26px">#${i + 1}</span><input list="sdNames" data-sd="top" data-i="${i}" value="${esc(w)}" placeholder="(random)" style="width:200px"></label>`).join('');
    return `<div class="st-seed" style="border:1px solid var(--line);border-radius:8px;padding:10px;margin:8px 0">
      <strong>Randomize pro mastery</strong> <span class="muted">${n ? `${n} players in the open save` : 'open a save or database first: the players come from it'}</span>
      <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-top:6px">Average rank <select data-sd="mean">${opts(S.mean, TOP)}</select>
        Spread <input type="number" data-sd="sd" min="0.1" max="4" step="0.1" value="${S.sd}" style="width:60px"> ranks
        Highest rank <select data-sd="max">${opts(S.max, TOP)}</select></div>
      <p class="muted" style="margin:4px 0">Expected: ${shares.map((p, r) => `${RANK_NAMES[r]} ${Math.round(p * 100)}%${n ? ` (~${Math.round(p * n)})` : ''}`).join(' · ')}</p>
      <div style="display:flex;gap:24px;flex-wrap:wrap">
        <div><b>Fixed ranks</b> <span class="muted">(these players get this rank, near the top of it)</span>${fixed || '<p class="muted">none</p>'}
          <button class="btn small" data-mem="sd-add">+ Player</button></div>
        <div><b>Top 10</b> <span class="muted">(left empty: the Top 10 is earned in games)</span>${top}</div>
      </div>
      <datalist id="sdNames">${all.map(x => `<option value="${esc(seedLabel(x))}">`).join('')}</datalist>
      <div class="st-btns" style="margin-top:8px"><button class="btn small primary" data-mem="sd-roll">Roll and save</button>
        <button class="btn small" data-mem="sd-close">Close</button></div></div>`;
  }
  // read the panel's fields back into T.seed
  function seedRead() {
    const S = seedLoad(), q = sel => document.querySelectorAll(`#stMemory [data-sd="${sel}"]`);
    const one = sel => q(sel)[0];
    if (one('mean')) S.mean = +one('mean').value;
    if (one('max')) S.max = +one('max').value;
    if (one('sd')) S.sd = Math.min(4, Math.max(0.1, +one('sd').value || 1));
    q('fwho').forEach(el => { const f = S.fixed[+el.dataset.i]; if (f) f.who = el.value; });
    q('frank').forEach(el => { const f = S.fixed[+el.dataset.i]; if (f) f.rank = +el.value; });
    q('top').forEach(el => { S.top[+el.dataset.i] = el.value; });
    seedSave();
    return S;
  }
  function seedPlan() {
    const S = seedRead(), all = seedPlayers();
    if (!all.length) { alert('Open a save or database first (top of the editor): the players and their ids come from it.'); return null; }
    const bad = [], pinned = new Map(), topIds = [];
    S.top.forEach((w, i) => { if (!String(w).trim()) return; const x = seedFind(w, all);
      if (!x) bad.push(`Top 10 #${i + 1}: "${w}" isn't a player in this save`); else if (topIds.includes(x.id)) bad.push(`${x.name} is in the Top 10 twice`); else topIds.push(x.id); });
    S.fixed.forEach(f => { if (!String(f.who).trim()) return; const x = seedFind(f.who, all);
      if (!x) bad.push(`"${f.who}" isn't a player in this save`); else if (!topIds.includes(x.id)) pinned.set(x.id, f.rank); });
    if (bad.length) { alert(bad.join('\n')); return null; }
    const counts = Array(TOP + 1).fill(0);
    const entries = all.map(x => {
      let r, points;
      const t = topIds.indexOf(x.id);
      if (t >= 0) { r = TOP; points = 400 - t * 10; }
      else if (pinned.has(x.id)) { r = pinned.get(x.id); const [lo, hi] = SEED_BANDS[r]; points = Math.round((lo + 0.85 * (hi - lo)) * 2) / 2; }
      else { r = clamp(Math.round(S.mean + S.sd * gauss()), 0, S.max); const [lo, hi] = SEED_BANDS[r]; points = Math.round((lo + Math.random() * (hi - lo)) * 2) / 2; }
      counts[r]++;
      // games and wins that add up to the points (an official game 1, a win 1.5): about half of them won
      const games = Math.round(points / 1.25), wins = Math.round(games / 2);
      return { a: x.id, points, games, wins };
    });
    const dist = RANK_NAMES.map((n, i) => counts[i] ? `${n} ${counts[i]}` : '').filter(Boolean).join(', ');
    const msg = `Give all ${all.length} players a new Scribble mastery?\n\n${dist}\n` +
      (topIds.length ? `Top 10 picked: ${topIds.length}.\n` : '') + (pinned.size ? `Fixed ranks: ${pinned.size}.\n` : '') +
      `\nThis replaces their current mastery (the learned meta stays). A backup goes to editor/backups/scribble. Close the game first: it reads Scribble's memory when it starts.`;
    return confirm(msg) ? { action: 'seed', entries } : null;
  }
  async function memAction(b) {
    const a = b.dataset.mem;
    if (a === 'reload') { T.mem = null; return renderMemory(); }
    let body = null;
    if (a === 'reset-meta') { if (!confirm('Forget every learned spell score? (Athletes keep their games.)')) return; body = { action: 'reset-meta' }; }
    if (a === 'reset-all') { if (!confirm('Reset all of Scribble\'s memory: every athlete back to Novice and the meta forgotten?')) return; body = { action: 'reset-all' }; }
    if (a === 'forget') body = { action: 'set-games', athlete: +b.dataset.a, games: 0 };
    if (a === 'set') { const inp = document.querySelector(`[data-games="${b.dataset.a}"]`); body = { action: 'set-games', athlete: +b.dataset.a, games: Math.max(0, +inp.value || 0) }; }
    if (a === 'add') { const id = +($('#stNewAth').value); if (!(id >= 0)) return; body = { action: 'set-games', athlete: id, games: 0 }; }
    if (a === 'seed') { T.seedOpen = !T.seedOpen; return renderMemory(); }
    if (a === 'sd-close') { seedRead(); T.seedOpen = false; return renderMemory(); }
    if (a === 'sd-add') { seedRead().fixed.push({ who: '', rank: 4 }); seedSave(); return renderMemory(); }
    if (a === 'sd-del') { seedRead().fixed.splice(+b.dataset.i, 1); seedSave(); return renderMemory(); }
    if (a === 'sd-roll') { body = seedPlan(); if (!body) return; T.seedOpen = false; }
    if (!body) return;
    const r = await fetch('/api/scribble', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
    const j = await r.json().catch(() => ({}));
    if (!r.ok) alert(j.error || 'Failed'); T.mem = null; renderMemory();
  }

  // ------------------------------------------------------------------ shell
  function renderView() {
    $('#stArena').hidden = T.view !== 'arena'; $('#stMemory').hidden = T.view !== 'memory';
    document.querySelectorAll('[data-st="arena"],[data-st="memory"]').forEach(b => b.classList.toggle('primary', b.dataset.st === T.view));
    if (T.view === 'memory') renderMemory();
  }
  function wire() {
    if (T.wired) return; T.wired = true;
    const root = $('#skillTest');
    root.innerHTML = `
      <div class="st-top"><button class="btn small primary" data-st="arena">Arena</button><button class="btn small" data-st="memory">Scribble memory</button>
        <span class="muted st-tip">Click the arena first so it gets the keys.</span></div>
      <div id="stArena" class="st-grid">
        <aside class="st-left" id="stLeft"></aside>
        <div class="st-mid"><canvas id="stCanvas" tabindex="0" width="${AW * T.zoom}" height="${AH * T.zoom}"></canvas>
          <div class="st-status" id="stStatus"></div><div id="stControls"></div></div>
        <aside class="st-right" id="stRight"></aside>
      </div>
      <div id="stMemory" class="st-memory" hidden></div>`;
    T.canvas = $('#stCanvas'); T.ctx = T.canvas.getContext('2d'); T.speed = 1;
    wireCanvas();
    root.addEventListener('click', ev => { const b = ev.target.closest('[data-mem]'); if (b) return memAction(b); onSideClick(ev); });
    root.addEventListener('change', ev => {
      const sd = ev.target.dataset && ev.target.dataset.sd;
      if (sd === 'mean' || sd === 'sd' || sd === 'max') { seedRead(); return renderMemory(); }
      if (sd) { seedRead(); return; }
      onSideChange(ev);
    });
    requestAnimationFrame(frame);
  }
  window.TFM2SkillTest = {
    async show() {
      wire(); renderView();
      if (!T.champs.length) { await loadChamps(); await pick(T.champs[0] && T.champs[0].id); }
    },
    _T: T, _resolve: resolveSpell, _castRecipe: castRecipe, _step: step,
  };
})();
