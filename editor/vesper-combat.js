// Vesper's rules for the Rift Lab: a 1:1 port of native/tfm2_custom_ai/src/vesper_core.rs (the pure engine and its
// ranked planner) plus the deterministic practice arena of vesper_tests.rs. `node tools/check_vesper_parity.js`
// replays the native simulation vectors decision for decision.
(function (root) {
  'use strict';
  const isNode = typeof module !== 'undefined';
  const D = isNode ? require('./vesper-data.js') : root.RiftData;
  const M = isNode ? require('./vesper-math.js') : root.VesperMath;
  const { div, roll, dist2, dist, keepBp, chainBp, openDelay, twistIndex, Atlas, lerp, along, clampMap, segDist, side, Window, ChargeBank } = M;
  const C = D.constants;
  const P = {};
  D.prims.forEach((p, i) => { P[p.id] = i; });
  const PRIM_SIDE = D.prims.map(p => p.side);
  const PRIM_ONCE = D.prims.map(p => p.once);
  const DIMS = D.dimensions;
  const TWISTS = D.twists.map(t => ({ name: t.name, a: t.families[0], b: t.families[1], kind: t.kind, mag: t.mag }));
  const RIFTS = D.rifts;
  const R = {};
  RIFTS.forEach((r, i) => { R[r.id] = i; });
  const TECHS = D.techniques.map(t => ({ ...t, rifts: t.rifts.map(id => R[id]) }));
  const T = {};
  TECHS.forEach((t, i) => { T[t.id] = i; });
  const COMBOS = D.combos.map(c => c.map(id => T[id]));
  const TB = D.tables;
  const G = D.gates;
  const RANKS = D.ranks;
  const DIRS = [[10000, 0], [9239, 3827], [7071, 7071], [3827, 9239], [0, 10000], [-3827, 9239], [-7071, 7071], [-9239, 3827],
    [-10000, 0], [-9239, -3827], [-7071, -7071], [-3827, -9239], [0, -10000], [3827, -9239], [7071, -7071], [9239, -3827]];
  const D8 = [[10000, 0], [7071, 7071], [0, 10000], [-7071, 7071], [-10000, 0], [-7071, -7071], [0, -10000], [7071, -7071]];
  const SQ = v => v * v;

  // Rust min_by_key keeps the first minimum; max_by_key keeps the last maximum.
  function cmp(a, b) {
    if (Array.isArray(a)) {
      for (let i = 0; i < a.length; i++) { if (a[i] < b[i]) return -1; if (a[i] > b[i]) return 1; }
      return 0;
    }
    return a < b ? -1 : a > b ? 1 : 0;
  }
  function minBy(arr, f) { let best = null, bk; for (const x of arr) { const k = f(x); if (best === null || cmp(k, bk) < 0) { best = x; bk = k; } } return best; }
  function maxBy(arr, f) { let best = null, bk; for (const x of arr) { const k = f(x); if (best === null || cmp(k, bk) >= 0) { best = x; bk = k; } } return best; }
  function heading8(dx, dy) {
    if (dx === 0 && dy === 0) return 0;
    let best = 0, bk = null;
    D8.forEach(([x, y], i) => { const k = x * dx + y * dy; if (bk === null || k >= bk) { bk = k; best = i; } });
    return best;
  }
  function look(rank, top) { return rank >= 7 && top === 1 ? 8 : Math.min(rank, 7); }
  function artTier(rank) { return [0, 0, 0, 1, 1, 2, 2, 3][Math.min(rank, 7)]; }
  const HABIT_GATE = {
    'led target': 1, 'collapsed a rift before it broke': G.safety,
    'solved the link count': G.geometry, 'implode on a pile': G.geometry, 'rain on a cluster': G.geometry, 'crossfire split': G.geometry, 'domino through the pile': G.geometry, 'bank shot around a wall': G.geometry,
    'Lifeline set before the fight': G.survival, 'peeled a diver': G.survival, 'saved an ally': G.survival, 'rescued ally': G.survival, 'phased out of a burst': G.survival, 'escaped a dive': G.survival,
    'Escort on the diver': G.team, 'Gate synced with the tank': G.team, 'Fling toward the team': G.team, 'swapped the target into the team': G.team, 'dropped aim for a diver': G.team, 'borrowed a far dimension': G.team, 'rotated an ally into the fight': G.team,
    'Kidnapped the carry into the team': G.priority, 'Sinkhole at the chokepoint': G.priority, "Seed on the carry's path": G.priority, 'stitched two worlds': G.priority,
    'spine refreshed': G.spine, 'rode strain into Overload': G.spine, 'planned a 5-step sequence': G.spine,
    'planned 2 steps ahead': 3, 'planned 3 steps ahead': 5, 'planned 4 steps ahead': 6,
  };
  const habitGate = h => HABIT_GATE[h] || 0;
  function land(fx, fy, x, y, wall) {
    [x, y] = clampMap(x, y);
    if (!wall(x, y)) return [x, y];
    const steps = Math.max(1, div(dist(fx, fy, x, y), 3000));
    for (let i = 0; i <= steps; i++) {
      const px = x + div((fx - x) * i, steps), py = y + div((fy - y) * i, steps);
      if (!wall(px, py)) return [px, py];
    }
    return [fx, fy];
  }
  const Mods = (o) => Object.assign({ move_speed: 0, attack_speed: 0, amplify: 0, reduce: 0, heal_reduce: 0, attack: 0, defence: 0, mr: 0, ap: 0, regen: 0, crit: 0, range: 0, ba_reduce: 0, reflect: 0, cooldown: 0, vamp: 0, ignore_wall: false, undying: false, cc_immune: false }, o || {});

  // ------------------------------------------------------------------ rifts
  function riftLive(r) { return r.opened && !r.done; }
  function riftHidden(r, tick) { return r.kind === R.seed && tick < r.opens + C.seed_hidden; }
  function riftDims(r, tick) {
    const v = [(r.borrowed && tick < r.borrowed[1]) ? r.borrowed[0] : r.dim];
    if (r.dim2 !== null) v.push(r.dim2);
    return v;
  }
  function riftReach(r, x, y) {
    return (r.kind === R.slash || r.kind === R.mirror) ? segDist(r.x, r.y, r.ex, r.ey, x, y) : dist(r.x, r.y, x, y);
  }
  const riftCovers = (r, x, y) => riftReach(r, x, y) <= r.radius;
  function cloneRift(r) {
    return { ...r, borrowed: r.borrowed ? r.borrowed.slice() : null, dwell: new Map(r.dwell), touched: new Set(r.touched), sides: new Map(r.sides), inside: new Set(r.inside) };
  }
  const Spec = {
    at: (kind, x, y) => ({ kind, x, y, ex: x, ey: y, host: null, chain: false, entrance: false, order: null }),
    line: (kind, x, y, ex, ey) => ({ ...Spec.at(kind, x, y), ex, ey }),
    node: (kind, x, y) => ({ ...Spec.at(kind, x, y), chain: true }),
    entrance: (kind, x, y) => ({ ...Spec.at(kind, x, y), entrance: true }),
  };

  // ------------------------------------------------------------------ tick view
  class Tick {
    constructor(tick, me, fighters, wall, attackInterval, stunned) {
      this.tick = tick; this.me = me; this.fighters = fighters; this.wall = wall; this.attack_interval = attackInterval; this.stunned = stunned;
    }
    get(id) { return this.fighters.find(f => f.id === id && f.hp > 0) || null; }
    enemies() { return this.fighters.filter(f => f.team !== this.me.team && f.hp > 0); }
    allies() { return this.fighters.filter(f => f.team === this.me.team && f.id !== this.me.id && f.hp > 0); }
    friends() { return this.fighters.filter(f => f.team === this.me.team && f.hp > 0); }
    danger(x, y, r) { return this.enemies().filter(e => dist2(e.x, e.y, x, y) <= SQ(r)).length; }
  }
  function centre(v, or) {
    if (!v.length) return or;
    const n = v.length;
    return [div(v.reduce((a, f) => a + f.x, 0), n), div(v.reduce((a, f) => a + f.y, 0), n)];
  }

  class Core {
    constructor(seed, owner, rank, top) {
      this.rank = Math.min(rank, 7);
      this.top = top === undefined ? null : top;
      const s = BigInt.asUintN(64, BigInt(seed)) ^ BigInt.asUintN(64, BigInt(owner) * 0x9e3779b97f4a7c15n) ^ 0x564553504552n;
      this.rng = { s: s === 0n ? 1n : s };
      this.atlas = new Atlas(BigInt(seed));
      this.charges = new ChargeBank(C.charges);
      this.strain = 0;
      this.rifts = [];
      this.next_uid = 1;
      this.chain_seq = 0;
      this.packets = [];
      this.later = [];
      this.dots = [];
      this.history = new Map();
      this.velocity = new Map();
      this.damage = new Map(); this.controls = new Map(); this.moves = new Map(); this.support = new Map();
      this.cooldowns = new Map();
      this.self_move_until = 0; this.next_attack = 0; this.aiming = false; this.chain_target = null;
      this.crossfire_until = 0; this.overload = 0; this.plan = []; this.plan_until = 0; this.last_think = 0;
      this.phased = false; this.risky_until = 0; this.displaced = new Map();
      this.stats = { tech_uses: Array(28).fill(0), rifts_opened: 0, rifts_touching: 0, breaches: 0, breaks: 0, packets: 0, eff_sum: 0, damage: 0, kidnaps: 0, useful_kidnaps: 0, gates: 0, engaged_gates: 0, lonely_gates: 0, traps: 0, trap_catches: 0, risky_moves: 0, self_deaths: 0, implodes: 0, implode_hits: 0, habits: {} };
      this.fx_left = C.fx_per_update; this.last_tick = null; this.hand_placed = false;
    }
    random(n) { return roll(this.rng, n); }
    look() { return look(this.rank, this.top); }
    table(name) { return TB[name][Math.min(this.rank, 7)]; }
    gate(g) { return this.rank >= g; }
    fx(out, tag, x, y, unit, ticks) { if (this.fx_left > 0) { this.fx_left--; out.push({ k: 'Fx', tag, x, y, unit, ticks }); } }
    habit(h) { if (h && this.rank >= habitGate(h)) this.stats.habits[h] = (this.stats.habits[h] || 0) + 1; }
    rift(uid) { return this.rifts.find(r => r.uid === uid && !r.done) || null; }
    openCount() { return this.rifts.filter(r => !r.done).length; }
    cooldownOk(key, id, tick) { const u = this.cooldowns.get(key + ':' + id); return u === undefined || tick >= u; }
    setCooldown(key, id, until) { this.cooldowns.set(key + ':' + id, until); }
    win(map, id) { if (!map.has(id)) map.set(id, new Window()); return map.get(id); }
    tier4() { return this.look() === 8 ? 4 : artTier(this.rank); }
    addStrain(n) { this.strain = Math.max(0, this.strain + n); }

    // -------------------------------------------------------------- shared limits
    emit(out, f, amount, kind, tick) {
      if (amount <= 0 || f.hp <= 0) return 0;
      const cap = Math.max(1, div(f.max_hp * C.damage_pct, 100));
      const n = this.win(this.damage, f.id).allow(tick, C.damage_window, cap, amount);
      if (n > 0) { out.push({ k: 'Damage', target: f.id, amount: n, kind }); this.stats.damage += n; }
      return n;
    }
    control(out, f, kind, ticks, tick) {
      if (f.cc_immune || ticks <= 0) return false;
      const n = this.win(this.controls, f.id).allow(tick, C.control_window, C.control_cap, ticks);
      if (n === 0) return false;
      if (kind) out.push({ k: 'Cc', target: f.id, kind, ticks: n }); else out.push({ k: 'Banish', target: f.id, ticks: n });
      return true;
    }
    displace(out, f, x, y, tick, wall, enemy) {
      if (enemy && f.cc_immune) return false;
      if (this.win(this.moves, f.id).allow(tick, C.move_window, C.move_cap, 1) === 0) return false;
      [x, y] = land(f.x, f.y, x, y, wall);
      out.push({ k: 'SetPos', target: f.id, x, y });
      this.displaced.set(f.id, tick);
      return true;
    }
    shield(out, f, amount, ticks, tick) {
      const cap = Math.max(1, div(f.max_hp * C.support_pct, 100));
      const n = this.win(this.support, f.id).allow(tick, C.support_window, cap, Math.max(0, amount));
      if (n > 0) out.push({ k: 'Shield', target: f.id, amount: n, ticks });
    }
    buff(out, target, dim, ticks, mods) {
      if (ticks > 0) out.push({ k: 'Buff', target, name: 'vsd_' + DIMS[dim].id, ticks, mods });
    }

    // -------------------------------------------------------------- dimensions
    applyDim(out, t, dim, f, s, c, src) {
      const d = DIMS[dim];
      const p = d.prim;
      const sd = PRIM_SIDE[p];
      const enemy = f.team !== t.me.team;
      const me = f.id === t.me.id;
      if (src !== 'Breach') {
        const ok = sd === 'E' ? enemy : sd === 'A' ? !enemy : sd === 'S' ? me : sd === 'X';
        if (!ok) return;
      }
      const tick = t.tick;
      const sc = v => div(v * s, 100);
      const key = 1000 + p;
      const m = Mods();
      switch (p) {
        case P.BANISH:
          if (this.cooldownOk(key, f.id, tick) && this.control(out, f, null, Math.max(20, sc(d.a)), tick)) this.setCooldown(key, f.id, tick + 480);
          break;
        case P.STUN: case P.ROOT: case P.SILENCE: case P.DISARM: case P.STATICBURST: {
          const kind = (p === P.STUN || p === P.STATICBURST) ? 'Stun' : p === P.ROOT ? 'Root' : p === P.SILENCE ? 'Silence' : 'Disarm';
          const ticks = p === P.STATICBURST ? this.random(d.a + 1) : Math.max(10, sc(d.a));
          if (this.cooldownOk(key, f.id, tick) && this.control(out, f, kind, ticks, tick)) this.setCooldown(key, f.id, tick + 240);
          break;
        }
        case P.SLOW: m.move_speed = -sc(d.a); m.attack_speed = -sc(d.b); this.buff(out, f.id, dim, d.c, m); break;
        case P.AMPLIFY: m.amplify = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.DOT: case P.DRAIN: case P.ENTROPY: {
          const perMille = p === P.ENTROPY ? this.random(d.a + 1) : d.a;
          const perPulse = div(f.max_hp * sc(perMille), 2000);
          if (src === 'Imbue' && d.b > 0) {
            this.dots = this.dots.filter(x => !(x.target === f.id && x.drain === (p === P.DRAIN)));
            this.dots.push({ target: f.id, per_pulse: perPulse, until: tick + d.b, drain: p === P.DRAIN });
          } else {
            const n = this.emit(out, f, perPulse, 'Magic', tick);
            if (p === P.DRAIN && n > 0) out.push({ k: 'Heal', target: t.me.id, amount: n });
          }
          break;
        }
        case P.SHATTER:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'ClearShield', target: f.id }); this.setCooldown(key, f.id, tick + 240); }
          break;
        case P.STATDOWN: case P.STATUP: {
          const v = sc(d.b) * (p === P.STATDOWN ? -1 : 1);
          if (d.a === 0) m.attack = v; else if (d.a === 1) m.defence = v; else if (d.a === 2) m.mr = v; else if (d.a === 3) m.ap = v;
          else if (p === P.STATDOWN) m.regen = -Math.max(1, div(d.b, 10)); else m.crit = v;
          this.buff(out, f.id, dim, d.c, m);
          break;
        }
        case P.REFLECT: case P.PULL: case P.KNOCK: case P.SHOVE: case P.STORM: case P.GRAB: case P.REWIND: {
          if (!this.cooldownOk(key, f.id, tick)) return;
          const cx = c[0], cy = c[1];
          let to;
          if (p === P.REFLECT) to = [2 * cx - f.x, 2 * cy - f.y];
          else if (p === P.PULL) to = lerp(f.x, f.y, cx, cy, Math.min(100, sc(d.a)));
          else if (p === P.KNOCK) {
            const [dx, dy] = (f.x === cx && f.y === cy) ? [1, 0] : [f.x - cx, f.y - cy];
            to = along(f.x, f.y, dx, dy, sc(d.a));
          } else if (p === P.SHOVE || p === P.STORM) {
            let vx = 0, vy = 0;
            for (let i = 0; i < (p === P.STORM ? 2 : 1); i++) {
              const [ux, uy] = DIRS[this.random(16)];
              vx += div(ux * sc(d.a), 10000); vy += div(uy * sc(d.a), 10000);
            }
            to = [f.x + vx, f.y + vy];
          } else if (p === P.GRAB) to = along(f.x, f.y, t.me.x - f.x, t.me.y - f.y, Math.min(sc(d.a), dist(f.x, f.y, t.me.x, t.me.y)));
          else {
            const h = this.history.get(f.id);
            if (!h) return;
            const back = Math.max(1, div(div(d.a * Math.min(100, Math.max(30, s)), 100), 6));
            if (h.length <= back) return;
            to = h[h.length - 1 - back];
          }
          const enemyMove = enemy || src === 'Breach';
          if (this.displace(out, f, to[0], to[1], tick, t.wall, enemyMove)) this.setCooldown(key, f.id, tick + (p === P.REWIND ? 480 : 240));
          break;
        }
        case P.CDUP: case P.CDDOWN: m.cooldown = sc(d.a) * (p === P.CDUP ? 1 : -1); this.buff(out, f.id, dim, d.b, m); break;
        case P.HEALCUT: m.heal_reduce = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.SHELTER: m.reduce = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.HEAL:
          if (d.b === 0) {
            const k = 3000 + dim;
            if (this.cooldownOk(k, f.id, tick)) { out.push({ k: 'Heal', target: f.id, amount: div(f.max_hp * sc(d.a), 1000) }); this.setCooldown(k, f.id, tick + 600); }
          } else out.push({ k: 'Heal', target: f.id, amount: div(f.max_hp * sc(d.a), 2000) });
          break;
        case P.SHIELD:
          if (this.cooldownOk(key, f.id, tick)) { this.shield(out, f, div(f.max_hp * sc(d.a), 1000), d.b, tick); this.setCooldown(key, f.id, tick + 180); }
          break;
        case P.UNDYING: case P.CCIMMUNE:
          if (this.cooldownOk(key, f.id, tick)) {
            if (p === P.UNDYING) m.undying = true; else m.cc_immune = true;
            this.buff(out, f.id, dim, Math.max(10, sc(d.a)), m);
            this.setCooldown(key, f.id, tick + 1200);
          }
          break;
        case P.HASTE: m.move_speed = sc(d.a); m.attack_speed = sc(d.b); this.buff(out, f.id, dim, d.c, m); break;
        case P.RANGE: m.range = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.GHOST: m.ignore_wall = true; this.buff(out, f.id, dim, d.a, m); break;
        case P.CLEANSE:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'ClearCc', target: f.id }); this.setCooldown(key, f.id, tick + 240); }
          break;
        case P.ALLYINVIS: case P.SELFINVIS:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'Invisible', target: f.id, ticks: Math.max(10, sc(d.a)) }); this.setCooldown(key, f.id, tick + (p === P.ALLYINVIS ? 600 : 30)); }
          break;
        case P.BADR: m.ba_reduce = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.REFLECTDMG: m.reflect = sc(d.a); this.buff(out, f.id, dim, d.b, m); break;
        case P.VAMP: m.vamp = sc(d.a); this.buff(out, t.me.id, dim, d.b, m); break;
        case P.TITHE:
          if (enemy && this.cooldownOk(key, f.id, tick)) {
            out.push({ k: 'ClearShield', target: f.id });
            this.setCooldown(key, f.id, tick + 240);
            this.shield(out, t.me, div(t.me.max_hp * 5, 100), 180, tick);
          }
          break;
        case P.COINFLIP:
          if (this.random(2) === 0) { if (enemy) { m.amplify = sc(d.a); this.buff(out, f.id, dim, 40, m); } }
          else if (!enemy) { m.reduce = sc(d.a); this.buff(out, f.id, dim, 40, m); }
          break;
        case P.DICE: {
          let other = this.random(90);
          if (div(other, 10) >= d.family) other += 10;
          if (DIMS[other].prim !== P.DICE) this.applyDim(out, t, other, f, s, c, src);
          break;
        }
        default: break;
      }
    }

    // -------------------------------------------------------------- rifts
    open(out, t, spec, tech, subject) {
      const rt = RIFTS[spec.kind];
      const uid = this.next_uid++;
      const d = dist(t.me.x, t.me.y, spec.x, spec.y);
      const delay = (spec.kind === R.anchor && spec.host !== null) ? openDelay(0, 100) : openDelay(d, rt.delay_pct);
      const dim = this.atlas.dimAt(spec.x, spec.y, t.tick + delay);
      let chain = null;
      if (spec.chain) {
        if (spec.order !== null && spec.order !== undefined) chain = spec.order; else { this.chain_seq++; chain = this.chain_seq; }
      }
      const opens = t.tick + delay;
      this.rifts.push({ uid, kind: spec.kind, tech, dim, dim2: null, borrowed: null, x: spec.x, y: spec.y, ex: spec.ex, ey: spec.ey, radius: rt.radius, opens, expires: opens + rt.life, host: spec.host, peer: null, subject: subject === undefined ? null : subject, chain, entrance: spec.entrance, opened: false, fired: false, dwell: new Map(), touched: new Set(), sides: new Map(), inside: new Set(), done: false });
      this.stats.rifts_opened++;
      this.fx(out, 'open_l' + this.look(), spec.x, spec.y, null, delay);
      return uid;
    }
    close(out, uid, why) {
      const l = this.look();
      const r = this.rifts.find(r => r.uid === uid && !r.done);
      if (!r) return;
      r.done = true;
      this.fx(out, why + '_l' + l, r.x, r.y, null, 24);
    }
    implode(out, t, uid) {
      const r0 = this.rift(uid);
      if (!r0) return;
      const r = cloneRift(r0);
      const reach = div(r.radius * 3, 2);
      const amount = C.implode_base + div(t.me.attack * C.implode_ad, 100);
      let hits = 0;
      const targets = t.enemies().filter(e => riftReach(r, e.x, e.y) <= reach);
      for (const e of targets) {
        hits++;
        this.displace(out, e, div(e.x + r.x, 2), div(e.y + r.y, 2), t.tick, t.wall, true);
        this.emit(out, e, amount, 'Phys', t.tick);
        for (const dim of riftDims(r, t.tick)) this.applyDim(out, t, dim, e, 200, [r.x, r.y, r.radius], 'Imbue');
      }
      this.stats.implodes++;
      this.stats.implode_hits += hits;
      this.addStrain(-div(C.strain_tear, 2));
      this.fx(out, 'tech_implode_t' + this.tier4(), r.x, r.y, null, 30);
      this.close(out, uid, 'close');
    }
    breach(out, t) {
      const dim = this.atlas.dimAt(t.me.x, t.me.y, t.tick);
      this.stats.breaches++;
      out.push({ k: 'Log', s: `tick ${t.tick} breach strain ${this.strain} dimension ${DIMS[dim].id} (${DIMS[dim].name})` });
      const hit = t.fighters.filter(f => f.hp > 0 && dist2(f.x, f.y, t.me.x, t.me.y) <= SQ(C.breach_radius));
      for (const f of hit) this.applyDim(out, t, dim, f, 100, [t.me.x, t.me.y, C.breach_radius], 'Breach');
      out.push({ k: 'LoseHp', target: t.me.id, amount: div(t.me.max_hp * C.breach_hp_pct, 100) });
      out.push({ k: 'Cc', target: t.me.id, kind: 'Stun', ticks: C.breach_stagger });
      this.fx(out, 'breach_t' + this.tier4(), t.me.x, t.me.y, t.me.id, 36);
      const uids = this.rifts.filter(r => !r.done).map(r => r.uid);
      for (const uid of uids) this.close(out, uid, 'break');
      this.packets = [];
      this.strain = C.breach_reset;
      this.aiming = false;
    }

    // -------------------------------------------------------------- one tick
    step(input) {
      const out = [];
      if (this.last_tick === input.tick) return out;
      this.last_tick = input.tick;
      this.fx_left = C.fx_per_update;
      const me = input.fighters.find(f => f.id === input.me);
      if (!me) return out;
      const t = new Tick(input.tick, me, input.fighters, input.wall, Math.max(1, input.attack_interval), input.stunned);
      const tick = t.tick;
      this.charges.advance(tick);
      this.observe(t);
      if (this.openCount() <= 1 && tick % C.strain_decay_ticks === 0) this.addStrain(-1);
      for (const [k, until] of [...this.cooldowns]) if (!(until > tick)) this.cooldowns.delete(k);
      this.stepRifts(out, t);
      this.stepTechniques(out, t);
      this.stepZones(out, t);
      this.stepBreaking(out, t);
      this.stepLater(out, t);
      this.stepDots(out, t);
      this.stepChain(out, t);
      this.stepPackets(out, t);
      if (this.strain >= C.breach_at) this.breach(out, t);
      if (!this.hand_placed && !this.phased && !t.stunned && tick >= this.last_think + this.table('think_ticks')) {
        this.last_think = tick;
        this.think(out, t);
      }
      this.render(out, t);
      this.rifts = this.rifts.filter(r => !r.done);
      return out;
    }
    observe(t) {
      if (t.tick % 6 !== 0) return;
      for (const f of t.fighters.filter(f => f.hp > 0)) {
        if (!this.history.has(f.id)) this.history.set(f.id, []);
        const h = this.history.get(f.id);
        if (h.length) { const [px, py] = h[h.length - 1]; this.velocity.set(f.id, [div(f.x - px, 6), div(f.y - py, 6)]); }
        h.push([f.x, f.y]);
        while (h.length > div(C.history, 6) + 1) h.shift();
      }
      const alive = new Set(t.fighters.filter(f => f.hp > 0).map(f => f.id));
      for (const id of [...this.history.keys()]) if (!alive.has(id)) this.history.delete(id);
      for (const id of [...this.velocity.keys()]) if (!alive.has(id)) this.velocity.delete(id);
    }
    forecast(f, ticks) {
      const [vx, vy] = this.velocity.get(f.id) || [0, 0];
      const cl = v => Math.min(60000, Math.max(-60000, v));
      return clampMap(f.x + cl(vx * ticks), f.y + cl(vy * ticks));
    }
    stepRifts(out, t) {
      const tick = t.tick;
      for (let i = 0; i < this.rifts.length; i++) {
        const r = this.rifts[i];
        if (r.done) continue;
        if (r.host !== null) {
          const f = t.get(r.host);
          if (f) { r.x = f.x; r.y = f.y; } else { this.close(out, r.uid, 'close'); continue; }
        }
        if (!r.opened && tick >= r.opens) {
          r.opened = true;
          if (t.enemies().some(e => riftReach(r, e.x, e.y) <= r.radius * 2)) this.stats.rifts_touching++;
        }
        if (r.opened && r.kind === R.wander) {
          const [nx, ny] = dist(r.x, r.y, r.ex, r.ey) <= C.wander_speed ? [r.ex, r.ey] : along(r.x, r.y, r.ex - r.x, r.ey - r.y, C.wander_speed);
          r.x = nx; r.y = ny;
        }
        if (tick >= r.expires) {
          const c = cloneRift(r);
          if (c.kind === R.maw) this.implode(out, t, c.uid);
          else if (c.tech === T.hook) this.reel(out, t, c);
          else this.close(out, c.uid, 'close');
        }
      }
      for (const r of this.rifts) if (r.borrowed && tick >= r.borrowed[1]) r.borrowed = null;
    }
    reel(out, t, r) {
      const target = r.host !== null ? t.get(r.host) : null;
      if (target) {
        const p = r.peer !== null ? this.rift(r.peer) : null;
        const exit = p ? [p.x, p.y] : [t.me.x, t.me.y];
        const key = 2000 + T.hook;
        if (this.cooldownOk(key, target.id, t.tick) && this.displace(out, target, exit[0], exit[1], t.tick, t.wall, true)) {
          this.control(out, target, null, 20, t.tick);
          this.setCooldown(key, target.id, t.tick + TECHS[T.hook].cooldown);
          this.scoreKidnap(t, exit);
          this.fx(out, 'tech_hook_t' + this.tier4(), exit[0], exit[1], null, 24);
        }
      }
      this.close(out, r.uid, 'close');
    }
    scoreKidnap(t, at) {
      this.stats.kidnaps++;
      const allies = t.allies().filter(a => dist2(a.x, a.y, at[0], at[1]) <= SQ(45000)).length;
      if (allies >= 1 && dist(t.me.x, t.me.y, at[0], at[1]) > 25000) this.stats.useful_kidnaps++;
    }
    fire(uid) {
      let peer = null;
      const r = this.rift(uid);
      if (r) { r.fired = true; r.expires = Math.min(r.expires, Math.max(r.opens, 1)); peer = r.peer; }
      const p = peer !== null ? this.rift(peer) : null;
      if (p) p.expires = Math.min(p.expires, Math.max(p.opens, Math.max(0, p.expires - RIFTS[p.kind].life)) + 60);
    }
    exitOf(r) {
      const p = r.peer !== null ? this.rift(r.peer) : null;
      return p && riftLive(p) ? [p.x, p.y] : null;
    }
    stepTechniques(out, t) {
      const tick = t.tick;
      const uids = this.rifts.filter(riftLive).map(r => r.uid);
      for (const uid of uids) {
        const r0 = this.rift(uid);
        if (!r0) continue;
        const r = cloneRift(r0);
        const justOpened = tick === r.opens;
        const tier = this.tier4();
        const tech = r.tech;
        if ((tech === T.gate || tech === T.skydrop || tech === T.rotation) && r.entrance) {
          const exit = this.exitOf(r);
          if (!exit) continue;
          const movers = t.allies().filter(a => riftCovers(r, a.x, a.y));
          for (const a of movers) {
            const key = 2000 + T.gate;
            if (!this.cooldownOk(key, a.id, tick)) continue;
            this.setCooldown(key, a.id, tick + TECHS[T.gate].cooldown);
            const [x, y] = land(a.x, a.y, exit[0], exit[1], t.wall);
            out.push({ k: 'SetPos', target: a.id, x, y });
            this.stats.gates++;
            const idle = t.danger(a.x, a.y, 50000) === 0;
            const foes = t.danger(x, y, 40000);
            const friends = t.friends().filter(f => f.id !== a.id && dist2(f.x, f.y, x, y) <= SQ(70000)).length;
            if (idle && foes >= 1 && friends >= 1) this.stats.engaged_gates++;
            else if (foes >= 3) this.stats.lonely_gates++;
            this.fx(out, 'tech_gate_t' + tier, x, y, null, 24);
            if (tech === T.skydrop) this.later.push({ kind: 'Land', at: tick + 12, x, y, ally: a.id });
          }
        } else if (tech === T.step && r.entrance && !r.fired) {
          const exit = this.exitOf(r);
          if (!exit) continue;
          this.fire(uid);
          if (dist(t.me.x, t.me.y, r.x, r.y) <= C.step_reach && tick >= this.self_move_until) {
            this.self_move_until = tick + C.self_move_cd;
            const [x, y] = land(t.me.x, t.me.y, exit[0], exit[1], t.wall);
            out.push({ k: 'SetPos', target: t.me.id, x, y });
            this.fx(out, 'tech_step_t' + tier, x, y, null, 18);
            if (t.danger(x, y, 35000) >= 3) { this.stats.risky_moves++; this.risky_until = tick + 180; }
          }
        } else if (tech === T.phase && r.entrance && !r.fired) {
          const exit = this.exitOf(r);
          if (!exit) continue;
          this.fire(uid);
          if (dist(t.me.x, t.me.y, r.x, r.y) <= C.step_reach && tick >= this.self_move_until) {
            this.self_move_until = tick + C.self_move_cd;
            out.push({ k: 'Banish', target: t.me.id, ticks: 45 });
            this.phased = true;
            this.aiming = false;
            this.later.push({ kind: 'Unphase', at: tick + 45, x: exit[0], y: exit[1] });
            this.fx(out, 'tech_phase_t' + tier, r.x, r.y, null, 30);
          }
        } else if (tech === T.kidnap && r.entrance && !r.fired) {
          const exit = this.exitOf(r);
          if (!exit) continue;
          this.fire(uid);
          const v = r.subject !== null ? t.get(r.subject) : null;
          if (!v) continue;
          const key = 2000 + T.kidnap;
          if (riftReach(r, v.x, v.y) <= div(r.radius * 13, 10) && this.cooldownOk(key, v.id, tick) && this.displace(out, v, exit[0], exit[1], tick, t.wall, true)) {
            this.setCooldown(key, v.id, tick + TECHS[T.kidnap].cooldown);
            this.control(out, v, null, 30, tick);
            this.scoreKidnap(t, exit);
            this.fx(out, 'tech_kidnap_t' + tier, r.x, r.y, null, 24);
            this.fx(out, 'tech_kidnap_t' + tier, exit[0], exit[1], null, 24);
          }
        } else if (tech === T.sinktrap) {
          const inside = t.enemies().filter(e => riftCovers(r, e.x, e.y));
          const key = 2000 + T.sinktrap;
          for (const e of inside) {
            const rr = this.rift(uid);
            const k = e.id + 1000000;
            const n = (rr.dwell.get(k) || 0) + 1;
            rr.dwell.set(k, n);
            if (tick % C.pulse === 0) this.buff(out, e.id, 0, 40, Mods({ move_speed: -30 }));
            if (n >= 60 && this.cooldownOk(key, e.id, tick)) {
              this.setCooldown(key, e.id, tick + TECHS[T.sinktrap].cooldown);
              this.stats.trap_catches++;
              const exit = this.exitOf(r);
              if (exit && this.displace(out, e, exit[0], exit[1], tick, t.wall, true)) this.scoreKidnap(t, exit);
              this.control(out, e, null, 30, tick);
              this.fx(out, 'tech_sinktrap_t' + tier, e.x, e.y, null, 24);
            }
          }
        } else if (tech === T.swap && r.entrance && !r.fired) {
          const pr = r.peer !== null ? this.rift(r.peer) : null;
          if (!pr || !riftLive(pr)) continue;
          const peer = cloneRift(pr);
          this.fire(uid);
          const near = (x, y, rad) => minBy(t.fighters.filter(f => f.hp > 0 && dist(f.x, f.y, x, y) <= rad), f => [dist2(f.x, f.y, x, y), f.id]);
          const a = near(r.x, r.y, r.radius * 2), b = near(peer.x, peer.y, peer.radius * 2);
          if (!a || !b) continue;
          if (a.id === b.id || (a.team === t.me.team && b.team === t.me.team)) continue;
          const key = 2000 + T.swap;
          if (![a, b].every(f => f.team === t.me.team || (!f.cc_immune && this.cooldownOk(key, f.id, tick)))) continue;
          for (const f of [a, b]) if (f.team !== t.me.team) this.setCooldown(key, f.id, tick + TECHS[T.swap].cooldown);
          const [ax, ay] = land(a.x, a.y, b.x, b.y, t.wall);
          const [bx, by] = land(b.x, b.y, a.x, a.y, t.wall);
          out.push({ k: 'SetPos', target: a.id, x: ax, y: ay });
          out.push({ k: 'SetPos', target: b.id, x: bx, y: by });
          for (const [f, x, y] of [[a, ax, ay], [b, bx, by]]) {
            this.displaced.set(f.id, tick);
            if (f.id === t.me.id && t.danger(x, y, 35000) >= 3) { this.stats.risky_moves++; this.risky_until = tick + 180; }
            if (f.team !== t.me.team) this.scoreKidnap(t, [x, y]);
          }
          this.fx(out, 'tech_swap_t' + tier, r.x, r.y, null, 24);
          this.fx(out, 'tech_swap_t' + tier, peer.x, peer.y, null, 24);
        } else if (tech === T.fling) {
          const key = 2000 + T.fling;
          const caught = t.enemies().filter(e => riftCovers(r, e.x, e.y) && !r.touched.has(e.id));
          for (const e of caught) {
            const rr = this.rift(uid);
            if (rr) rr.touched.add(e.id);
            if (e.cc_immune || !this.cooldownOk(key, e.id, tick)) continue;
            const travel = Math.min(60, Math.max(10, div(dist(r.x, r.y, r.ex, r.ey), C.wander_speed)));
            if (this.control(out, e, null, travel, tick)) {
              this.setCooldown(key, e.id, tick + TECHS[T.fling].cooldown);
              this.later.push({ kind: 'Spit', at: tick + travel, target: e.id, x: r.ex, y: r.ey, ally: false });
              this.fx(out, 'tech_fling_t' + tier, e.x, e.y, null, 24);
            }
          }
        } else if (tech === T.rescue && r.entrance && !r.fired) {
          const exit = this.exitOf(r);
          if (!exit) continue;
          this.fire(uid);
          const a = r.subject !== null ? t.get(r.subject) : null;
          if (a) {
            const key = 2000 + T.rescue;
            if (riftCovers(r, a.x, a.y) && this.cooldownOk(key, a.id, tick)) {
              this.setCooldown(key, a.id, tick + TECHS[T.rescue].cooldown);
              const [x, y] = land(a.x, a.y, exit[0], exit[1], t.wall);
              out.push({ k: 'SetPos', target: a.id, x, y });
              this.fx(out, 'tech_rescue_t' + tier, x, y, null, 24);
              this.habit('rescued ally');
            }
          }
        } else if (tech === T.pocket && justOpened) {
          const a = r.subject !== null ? t.get(r.subject) : null;
          if (a) {
            const key = 2000 + T.pocket;
            if (riftCovers(r, a.x, a.y) && this.cooldownOk(key, a.id, tick)) {
              this.setCooldown(key, a.id, tick + TECHS[T.pocket].cooldown);
              out.push({ k: 'Banish', target: a.id, ticks: 60 });
              this.fx(out, 'tech_pocket_t' + tier, a.x, a.y, null, 30);
            }
          }
        } else if (tech === T.lifeline) {
          if (t.me.hp * 4 < t.me.max_hp && tick >= this.self_move_until) {
            this.self_move_until = tick + C.self_move_cd;
            const [x, y] = land(t.me.x, t.me.y, r.x, r.y, t.wall);
            out.push({ k: 'SetPos', target: t.me.id, x, y });
            this.fx(out, 'tech_lifeline_t' + tier, x, y, null, 24);
            this.close(out, uid, 'close');
            this.aiming = false;
          }
        } else if (tech === T.seedtrap && !riftHidden(r, tick)) {
          const v = minBy(t.enemies().filter(e => riftCovers(r, e.x, e.y)), e => [dist2(e.x, e.y, r.x, r.y), e.id]);
          if (v) {
            const key = 2000 + T.seedtrap;
            if (this.cooldownOk(key, v.id, tick)) {
              this.setCooldown(key, v.id, tick + TECHS[T.seedtrap].cooldown);
              this.stats.trap_catches++;
              for (const dim of riftDims(r, tick)) this.applyDim(out, t, dim, v, 200, [r.x, r.y, r.radius], 'Imbue');
              const exit = this.exitOf(r);
              if (exit) {
                if (this.displace(out, v, exit[0], exit[1], tick, t.wall, true)) this.scoreKidnap(t, exit);
                this.control(out, v, null, 30, tick);
              } else this.control(out, v, null, 45, tick);
              this.fx(out, 'tech_seedtrap_t' + tier, r.x, r.y, null, 30);
              this.close(out, uid, 'close');
            }
          }
        } else if (tech === T.rain && justOpened) {
          const amount = 25 + div(t.me.attack * 25, 100);
          for (let k = 0; k < 6; k++) {
            const [ux, uy] = DIRS[this.random(16)];
            const reach = this.random(r.radius);
            this.later.push({ kind: 'Drop', at: tick + 4 + k * 10, x: r.x + div(ux * reach, 10000), y: r.y + div(uy * reach, 10000), amount });
          }
        } else if (tech === T.volley && justOpened) {
          const amount = 20 + div(t.me.attack * 30, 100);
          for (let k = 0; k < 5; k++) {
            const px = r.x + div((r.ex - r.x) * k, 4), py = r.y + div((r.ey - r.y) * k, 4);
            const e = minBy(t.enemies().filter(e => dist(e.x, e.y, px, py) <= 40000), e => [dist2(e.x, e.y, px, py), e.id]);
            if (e) {
              this.emit(out, e, amount, 'Phys', tick);
              this.fx(out, `packet_l${this.look()}_a${heading8(e.x - px, e.y - py)}`, px, py, null, 8);
            }
          }
        }
        if (r.kind === R.slash || r.kind === R.mirror || (r.tech === T.curtain && r.peer !== null)) this.crossing(out, t, uid);
        if (r.kind === R.maw && justOpened && r.tech !== T.pocket) {
          const inside = t.enemies().filter(e => riftCovers(r, e.x, e.y));
          for (const e of inside) this.control(out, e, null, 45, tick);
          this.fx(out, 'tech_maw_t' + tier, r.x, r.y, null, 30);
        }
      }
    }
    crossing(out, t, uid) {
      const r0 = this.rift(uid);
      if (!r0) return;
      const r = cloneRift(r0);
      let ax, ay, bx, by;
      if (r.tech === T.curtain && r.kind === R.tear) {
        const p = r.peer !== null ? this.rift(r.peer) : null;
        if (!p) return;
        [ax, ay, bx, by] = [r.x, r.y, p.x, p.y];
      } else[ax, ay, bx, by] = [r.x, r.y, r.ex, r.ey];
      const reach = Math.max(r.radius, 9000) * 3;
      const near = t.enemies().filter(e => segDist(ax, ay, bx, by, e.x, e.y) <= reach);
      for (const e of near) {
        const s = side(ax, ay, bx, by, e.x, e.y);
        const rr = this.rift(uid);
        const prev = rr ? rr.sides.get(e.id) : undefined;
        if (rr) rr.sides.set(e.id, s);
        if (prev !== undefined && prev !== 0 && s !== 0 && prev !== s) {
          const nx = -(by - ay), ny = bx - ax;
          const push = r.kind === R.mirror ? 26000 : 14000;
          const dir = prev > 0 ? 1 : -1;
          const to = along(e.x, e.y, nx * dir, ny * dir, push);
          if (this.displace(out, e, to[0], to[1], t.tick, t.wall, true)) {
            const r2 = this.rift(uid);
            if (r2) r2.sides.set(e.id, prev);
            this.fx(out, `tech_${r.kind === R.mirror ? 'reflect' : 'curtain'}_t${this.tier4()}`, e.x, e.y, null, 18);
          }
        }
      }
    }
    stepZones(out, t) {
      const tick = t.tick;
      const uids = this.rifts.filter(r => riftLive(r) && !riftHidden(r, tick)).map(r => r.uid);
      for (const uid of uids) {
        const r0 = this.rift(uid);
        if (!r0) continue;
        const r = cloneRift(r0);
        const dims = riftDims(r, tick);
        const cen = [r.x, r.y, r.radius];
        const pulse = (tick + uid) % C.pulse === 0;
        const inside = t.fighters.filter(f => f.hp > 0 && riftCovers(r, f.x, f.y));
        const insideIds = new Set(inside.map(f => f.id));
        for (const f of inside) {
          const entering = !r.inside.has(f.id);
          for (const dim of dims) {
            const p = DIMS[dim].prim;
            const tkey = f.id ^ (dim << 24);
            if (PRIM_ONCE[p]) {
              if (entering && !r.touched.has(tkey)) {
                const rr = this.rift(uid);
                if (rr) rr.touched.add(tkey);
                if (p === P.FOLD) this.fold(out, t, r, f);
                else if (![P.SWAPSPACE, P.SCRAMBLE, P.WALL, P.TETHER, P.KALEIDO].includes(p) && PRIM_SIDE[p] !== 'P') this.applyDim(out, t, dim, f, 100, cen, 'Zone');
              }
            } else if (pulse && ![P.VENT, P.UNSTABLE, P.SIPHON].includes(p)) this.applyDim(out, t, dim, f, 100, cen, 'Zone');
          }
        }
        for (const dim of dims) {
          if (DIMS[dim].prim !== P.TETHER) continue;
          const left = [...r.inside].sort((a, b) => a - b).filter(id => !insideIds.has(id));
          for (const id of left) {
            const e = t.get(id);
            if (!e || e.team === t.me.team) continue;
            const tkey = id ^ (dim << 24);
            if (!r.touched.has(tkey)) {
              const rr = this.rift(uid);
              if (rr) rr.touched.add(tkey);
              const edge = along(r.x, r.y, e.x - r.x, e.y - r.y, div(r.radius * 8, 10));
              this.displace(out, e, edge[0], edge[1], tick, t.wall, true);
            }
          }
        }
        if (pulse) {
          for (const dim of dims) {
            const p = DIMS[dim].prim;
            if (p === P.VENT) this.addStrain(-DIMS[dim].a);
            else if ((p === P.SWAPSPACE || p === P.SCRAMBLE) && !r.touched.has('swap:' + dim)) {
              const enemiesOnly = p === P.SCRAMBLE;
              const pool = inside.filter(f => !enemiesOnly || f.team !== t.me.team).sort((a, b) => cmp([dist2(a.x, a.y, r.x, r.y), a.id], [dist2(b.x, b.y, r.x, r.y), b.id]));
              if (pool.length >= 2 && pool.slice(0, 2).some(f => f.team !== t.me.team)) {
                const a = pool[0], b = pool[1];
                const rr = this.rift(uid);
                if (rr) rr.touched.add('swap:' + dim);
                if (this.displace(out, a, b.x, b.y, tick, t.wall, a.team !== t.me.team)) this.displace(out, b, a.x, a.y, tick, t.wall, b.team !== t.me.team);
              }
            }
          }
        }
        if (dims.some(d => DIMS[d].prim === P.WALL)) {
          const ax = r.x - r.radius, ay = r.y, bx = r.x + r.radius, by = r.y;
          const near = t.enemies().filter(e => segDist(ax, ay, bx, by, e.x, e.y) <= r.radius);
          for (const e of near) {
            const s = side(ax, ay, bx, by, e.x, e.y);
            const rr = this.rift(uid);
            const k = e.id + 2000000;
            const prev = rr ? rr.sides.get(k) : undefined;
            if (rr) rr.sides.set(k, s);
            if (prev !== undefined && prev !== 0 && s !== 0 && prev !== s) {
              if (this.displace(out, e, e.x, e.y - 12000 * s, tick, t.wall, true)) {
                const r2 = this.rift(uid);
                if (r2) r2.sides.set(k, prev);
              }
            }
          }
        }
        const rr = this.rift(uid);
        if (rr) rr.inside = insideIds;
      }
    }
    fold(out, t, r, f) {
      if (f.team !== t.me.team) return;
      const key = 2000 + T.gate;
      if (!this.cooldownOk(key, f.id, t.tick)) return;
      const o = minBy(this.rifts.filter(o => riftLive(o) && o.uid !== r.uid && !riftHidden(o, t.tick)), o => [dist2(o.x, o.y, r.x, r.y), o.uid]);
      if (o) {
        this.setCooldown(key, f.id, t.tick + 180);
        const [x, y] = land(f.x, f.y, o.x, o.y, t.wall);
        out.push({ k: 'SetPos', target: f.id, x, y });
      }
    }
    stepBreaking(out, t) {
      const tick = t.tick;
      const broken = [];
      for (const r of this.rifts.filter(r => r.opened && !r.done)) {
        if (riftHidden(r, tick) || r.kind === R.maw) continue;
        const here = t.enemies().filter(e => e.id !== r.host && riftReach(r, e.x, e.y) <= C.break_core).map(e => e.id);
        for (const id of [...r.dwell.keys()]) if (!(id >= 1000000 || here.includes(id))) r.dwell.delete(id);
        let snapped = false;
        for (const id of here) {
          const n = (r.dwell.get(id) || 0) + 1;
          r.dwell.set(id, n);
          if (n >= C.break_ticks) snapped = true;
        }
        if (snapped) broken.push(r.uid);
      }
      for (const uid of broken) {
        this.stats.breaks++;
        this.addStrain(C.strain_break);
        out.push({ k: 'Log', s: `tick ${tick} rift ${uid} broken by an enemy (+${C.strain_break} strain)` });
        this.close(out, uid, 'break');
      }
    }
    stepLater(out, t) {
      const tick = t.tick;
      const due = this.later.filter(l => l.at <= tick);
      this.later = this.later.filter(l => !(l.at <= tick));
      const tier = this.tier4();
      for (const l of due) {
        if (l.kind === 'Repeat') {
          const f = t.get(l.target);
          if (f) { this.emit(out, f, l.amount, 'Phys', tick); this.fx(out, 'impact_l' + this.look(), f.x, f.y, null, 12); }
        } else if (l.kind === 'Domino') this.implode(out, t, l.uid);
        else if (l.kind === 'Drop') {
          for (const e of t.enemies().filter(e => dist(e.x, e.y, l.x, l.y) <= 12000)) this.emit(out, e, l.amount, 'Phys', tick);
          this.fx(out, 'tech_rain_t' + tier, l.x, l.y, null, 12);
        } else if (l.kind === 'Spit') {
          const f = t.get(l.target);
          if (f) {
            if (l.ally) { const [x, y] = land(f.x, f.y, l.x, l.y, t.wall); out.push({ k: 'SetPos', target: l.target, x, y }); }
            else if (this.displace(out, f, l.x, l.y, tick, t.wall, true)) this.scoreKidnap(t, [l.x, l.y]);
          }
        } else if (l.kind === 'Unphase') {
          this.phased = false;
          const [x, y] = land(t.me.x, t.me.y, l.x, l.y, t.wall);
          out.push({ k: 'SetPos', target: t.me.id, x, y });
          this.fx(out, 'tech_phase_t' + tier, x, y, null, 24);
        } else if (l.kind === 'Land') {
          for (const e of t.enemies().filter(e => dist(e.x, e.y, l.x, l.y) <= 20000)) this.control(out, e, 'Stun', 24, tick);
          this.fx(out, 'tech_skydrop_t' + tier, l.x, l.y, null, 24);
        }
      }
    }
    stepDots(out, t) {
      const tick = t.tick;
      if (tick % C.pulse !== 0) { this.dots = this.dots.filter(d => d.until > tick); return; }
      const dots = this.dots;
      this.dots = [];
      for (const d of dots) {
        if (d.until <= tick) continue;
        const f = t.get(d.target);
        if (f) {
          const n = this.emit(out, f, d.per_pulse, 'Magic', tick);
          if (d.drain && n > 0) out.push({ k: 'Heal', target: t.me.id, amount: n });
        }
      }
      this.dots = dots.filter(d => d.until > tick && t.get(d.target));
    }

    // -------------------------------------------------------------- chain fire
    chainNodes(tick) {
      return this.rifts.filter(r => riftLive(r) && r.chain !== null && !riftHidden(r, tick)).map(cloneRift).sort((a, b) => a.chain - b.chain);
    }
    stepChain(out, t) {
      const tick = t.tick;
      const nodes = this.chainNodes(tick);
      const exit = nodes.length ? nodes[nodes.length - 1] : null;
      if (!exit) { this.aiming = false; this.chain_target = null; return; }
      const inRange = e => dist(e.x, e.y, exit.x, exit.y) <= C.attack_range;
      let target = this.chain_target !== null ? t.get(this.chain_target) : null;
      if (target && !(target.team !== t.me.team && inRange(target))) target = null;
      if (!target) target = minBy(t.enemies().filter(inRange), e => [dist2(e.x, e.y, exit.x, exit.y), e.id]);
      if (!target) { this.aiming = false; return; }
      this.chain_target = target.id;
      const diver = t.danger(t.me.x, t.me.y, C.dive_radius) > 0;
      if (diver && this.gate(G.team)) {
        if (this.aiming) this.habit('dropped aim for a diver');
        this.aiming = false;
        return;
      }
      this.aiming = true;
      if (tick % 6 === 0) {
        out.push({ k: 'Cc', target: t.me.id, kind: 'Disarm', ticks: 8 });
        out.push({ k: 'Buff', target: t.me.id, name: 'vs_aim', ticks: 8, mods: Mods({ move_speed: -100 }) });
      }
      if (tick < this.next_attack) return;
      this.next_attack = tick + t.attack_interval;
      if (this.packets.length >= C.packet_limit) return;
      const path = nodes.map(n => [n.uid, n.x, n.y]);
      path.push([0, target.x, target.y]);
      const bonus = this.overload;
      this.overload = 0;
      const packet = (x, y, path, tgt, eff, bonus, ignoreWall, split, source) => ({ x, y, hop: [x, y], path, target: tgt, eff, bonus, dims: [], last_families: [], kaleido: false, true_pct: 0, ignore_wall: ignoreWall, split, echo: false, source, base: t.me.attack });
      this.packets.push(packet(t.me.x, t.me.y, path, target.id, 10000, bonus, false, false, 'chain'));
      if (tick < this.crossfire_until) {
        const others = this.rifts.filter(r => riftLive(r) && r.uid !== exit.uid && !riftHidden(r, tick));
        const used = [target.id];
        for (const r of others.slice(0, 4)) {
          if (this.packets.length >= C.packet_limit || used.length >= 3) break;
          const e = minBy(t.enemies().filter(e => !used.includes(e.id) && dist(e.x, e.y, r.x, r.y) <= C.attack_range), e => [dist2(e.x, e.y, r.x, r.y), e.id]);
          if (e) { used.push(e.id); this.packets.push(packet(r.x, r.y, [[0, e.x, e.y]], e.id, C.crossfire_pct * 100, 0, true, true, 'crossfire')); }
        }
      }
    }
    stepPackets(out, t) {
      const tick = t.tick;
      const packets = this.packets;
      this.packets = [];
      const keep = [];
      for (const p of packets) {
        let budget = C.packet_speed, alive = true, hit = false, bank = false;
        while (budget > 0 && alive && !hit) {
          if (!p.path.length) { alive = false; break; }
          let [uid, wx, wy] = p.path[0];
          if (uid === 0) { const f = t.get(p.target); if (f) { wx = f.x; wy = f.y; } }
          const d = dist(p.x, p.y, wx, wy);
          if (d > budget) { [p.x, p.y] = along(p.x, p.y, wx - p.x, wy - p.y, budget); break; }
          budget -= d;
          const len = dist(p.hop[0], p.hop[1], wx, wy);
          p.hop = [wx, wy];
          p.x = wx; p.y = wy;
          p.path.shift();
          if (uid === 0) { hit = true; break; }
          const r0 = this.rift(uid);
          if (!r0 || !riftLive(r0)) { alive = false; this.fx(out, 'close_l' + this.look(), wx, wy, null, 12); break; }
          const r = cloneRift(r0);
          let kb = keepBp(len);
          if (bank) kb = Math.min(9500, div(kb * 12, 10));
          p.eff = div(p.eff * kb, 10000);
          bank = r.kind === R.mirror;
          let unstable = 100;
          for (const dim of riftDims(r, tick)) {
            const dd = DIMS[dim];
            switch (dd.prim) {
              case P.TRUE: p.true_pct += dd.a; break;
              case P.LENS: p.bonus += dd.a; break;
              case P.ECHO: case P.WILDECHO: p.echo = true; break;
              case P.PRISM: p.split = true; break;
              case P.KALEIDO: p.kaleido = true; break;
              case P.UNSTABLE: unstable = dd.a; this.addStrain(dd.b); break;
              default:
                if (PRIM_SIDE[dd.prim] === 'E') p.dims.push(dim);
                else if (PRIM_SIDE[dd.prim] === 'S') this.applyDim(out, t, dim, t.me, 100, [r.x, r.y, r.radius], 'Imbue');
            }
            if (dd.prim === P.ECHO || dd.prim === P.WILDECHO || dd.prim === P.PRISM) p.dims.push(dim);
          }
          if (unstable !== 100) p.bonus += unstable - 100;
          p.last_families.push(DIMS[riftDims(r, tick)[0]].family);
          if (p.last_families.length > 2) p.last_families.shift();
        }
        if (hit) this.hit(out, t, p);
        else if (alive) {
          if ((tick + p.target) % 4 === 0) {
            const dir = p.path.length ? heading8(p.path[0][1] - p.x, p.path[0][2] - p.y) : 0;
            this.fx(out, `packet_l${this.look()}_a${dir}`, p.x, p.y, null, 4);
          }
          keep.push(p);
        }
      }
      this.packets = keep;
    }
    hit(out, t, p) {
      const tick = t.tick;
      const f = t.get(p.target);
      if (!f) return;
      const amount = div(div(p.base * p.eff, 10000) * Math.max(10, 100 + p.bonus), 100);
      const dealt = this.emit(out, f, amount, 'Phys', tick);
      if (p.true_pct > 0) this.emit(out, f, div(amount * p.true_pct, 100), 'True', tick);
      this.stats.packets++;
      this.stats.eff_sum += p.eff;
      const s = Math.min(100, Math.max(30, div(p.eff, 100)));
      for (const dim of p.dims) {
        const pr = DIMS[dim].prim;
        if (pr === P.ECHO) this.later.push({ kind: 'Repeat', at: tick + DIMS[dim].b, target: f.id, amount: div(amount * DIMS[dim].a, 100) });
        else if (pr === P.WILDECHO) {
          const pct = 20 + this.random(61);
          this.later.push({ kind: 'Repeat', at: tick + 30, target: f.id, amount: div(amount * pct, 100) });
        } else if (pr === P.PRISM) {
          const others = t.enemies().filter(e => e.id !== f.id && dist(e.x, e.y, f.x, f.y) <= 30000).slice(0, DIMS[dim].a - 1);
          for (const e of others) this.emit(out, e, div(amount * DIMS[dim].b, 100), 'Phys', tick);
        } else this.applyDim(out, t, dim, f, s, [f.x, f.y, 0], 'Imbue');
      }
      const siphons = [];
      for (const r of this.rifts.filter(r => riftLive(r) && riftCovers(r, f.x, f.y))) {
        for (const d of riftDims(r, tick)) if (DIMS[d].prim === P.SIPHON) siphons.push([r.x, r.y, r.radius, DIMS[d].a]);
      }
      for (const [x, y, rad, pct] of siphons) {
        for (const a of t.friends().filter(a => dist(a.x, a.y, x, y) <= rad)) out.push({ k: 'Heal', target: a.id, amount: div(dealt * pct, 100) });
      }
      if (p.last_families.length === 2) {
        const idx = p.kaleido ? this.random(55) : twistIndex(p.last_families[0], p.last_families[1]);
        this.twist(out, t, idx, f, amount);
      }
      this.fx(out, 'impact_l' + this.look(), f.x, f.y, null, 12);
    }
    twist(out, t, idx, f, amount) {
      const tick = t.tick;
      const w = TWISTS[idx];
      const nodes = this.chainNodes(tick);
      const exit = nodes.length ? [nodes[nodes.length - 1].x, nodes[nodes.length - 1].y] : [f.x, f.y];
      const first = nodes.length ? [nodes[0].x, nodes[0].y] : null;
      const mag = w.mag;
      const within = (x, y, cx, cy, r) => dist2(x, y, cx, cy) <= SQ(r);
      switch (w.kind) {
        case 0: this.control(out, f, 'Root', mag, tick); break;
        case 1: if (f.controlled) this.emit(out, f, div(amount * mag, 100), 'Phys', tick); break;
        case 2: if (this.displaced.has(f.id) && tick <= this.displaced.get(f.id) + 60) this.control(out, f, 'Stun', mag, tick); break;
        case 3:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], 20000)).map(a => a.id)) {
            if (this.cooldownOk(5003, a, tick)) { this.setCooldown(5003, a, tick + 600); out.push({ k: 'Buff', target: a, name: 'vst_sanctuary', ticks: mag, mods: Mods({ cc_immune: true }) }); }
          }
          break;
        case 4:
          if (this.control(out, f, 'Root', mag, tick)) { const to = lerp(f.x, f.y, exit[0], exit[1], 50); this.displace(out, f, to[0], to[1], tick, t.wall, true); }
          break;
        case 5: {
          const e = minBy(t.enemies().filter(e => e.id !== f.id && within(e.x, e.y, f.x, f.y, 30000)), e => [dist2(e.x, e.y, f.x, f.y), e.id]);
          if (e) this.emit(out, e, div(amount * mag, 100), 'Phys', tick);
          break;
        }
        case 6: if (this.displaced.has(f.id) && tick <= this.displaced.get(f.id) + 60) this.emit(out, f, div(f.max_hp * mag, 1000), 'Phys', tick); break;
        case 7: case 12:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], 20000))) out.push({ k: 'Heal', target: a.id, amount: w.kind === 7 ? div(amount * mag, 100) : div(a.max_hp * mag, 1000) });
          break;
        case 8: {
          const far = maxBy(this.rifts.filter(riftLive), r => [dist2(r.x, r.y, exit[0], exit[1]), r.uid]);
          if (far && nodes.length) {
            const r = cloneRift(far);
            const dim = riftDims(nodes[nodes.length - 1], tick)[0];
            for (const e of t.enemies().filter(e => riftCovers(r, e.x, e.y))) this.applyDim(out, t, dim, e, 100, [r.x, r.y, r.radius], 'Zone');
          }
          break;
        }
        case 9:
          for (const e of t.enemies().filter(e => e.id !== f.id && within(e.x, e.y, f.x, f.y, 20000))) { const to = lerp(e.x, e.y, f.x, f.y, Math.min(100, mag)); this.displace(out, e, to[0], to[1], tick, t.wall, true); }
          break;
        case 10:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], 20000)).map(a => a.id)) out.push({ k: 'Buff', target: a, name: 'vst_escort', ticks: 60, mods: Mods({ move_speed: mag }) });
          break;
        case 11: if (first) this.displace(out, f, first[0], first[1], tick, t.wall, true); break;
        case 13:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], 20000))) this.shield(out, a, div(a.max_hp * mag, 1000), 180, tick);
          break;
        default: this.later.push({ kind: 'Repeat', at: tick + 12, target: f.id, amount: div(amount * mag, 100) });
      }
      this.fx(out, 'twist' + w.kind, f.x, f.y, null, 18);
    }
    render(out, t) {
      const tick = t.tick;
      const l = this.look();
      const tier = Math.min(3, this.tier4());
      for (const r of this.rifts.filter(riftLive)) {
        for (let layer = 0; layer < 3; layer++) {
          if ((tick + r.uid * 11 + layer * 16) % 48 !== 0) continue;
          const hidden = riftHidden(r, tick);
          if (hidden && layer > 0) continue;
          const dim = riftDims(r, tick)[0];
          const line = r.kind === R.slash || r.kind === R.mirror;
          const tag = layer === 0 ? (hidden ? 'seedhint_l' + l : line ? `rift_${RIFTS[r.kind].id}_l${l}_a${heading8(r.ex - r.x, r.ey - r.y) % 4}` : `rift_${RIFTS[r.kind].id}_l${l}`)
            : layer === 1 ? `core_f${DIMS[dim].family}_t${tier}` : 'glyph' + String(dim).padStart(2, '0');
          const [x, y] = line ? [div(r.x + r.ex, 2), div(r.y + r.ey, 2)] : [r.x, r.y];
          this.fx(out, tag, x, y, r.host, 48);
        }
      }
      let prev = [t.me.x, t.me.y];
      this.chainNodes(tick).forEach((n, i) => {
        if ((tick + i * 7) % 48 === 0) this.fx(out, `link_l${l}_a${heading8(n.x - prev[0], n.y - prev[1])}`, div(n.x + prev[0], 2), div(n.y + prev[1], 2), null, 48);
        prev = [n.x, n.y];
      });
      if (tick % C.drift_ticks === C.drift_ticks - C.drift_warn) this.fx(out, 'atlas_warn', t.me.x, t.me.y, t.me.id, C.drift_warn);
    }

    // -------------------------------------------------------------- the planner
    view(t, enemies) {
      const allies = t.allies();
      const tankE = maxBy(enemies, e => [e.max_hp, e.id]);
      const tank = tankE ? tankE.id : 0;
      const focus = (this.gate(G.priority) ? minBy(enemies, e => [this.priorityRank(e, tank, t), e.id]) : minBy(enemies, e => [dist2(e.x, e.y, t.me.x, t.me.y), e.id])) || t.me;
      const me = [t.me.x, t.me.y];
      return { ally_centre: centre(allies, me), enemy_centre: centre(enemies, me), diver: t.danger(t.me.x, t.me.y, C.dive_radius) > 0, focus, tank, front: maxBy(allies, a => [a.max_hp, a.id]), allies, enemies };
    }
    priorityRank(e, tank, t) {
      return div(e.hp * 100, Math.max(1, e.max_hp)) - div(e.attack, 6) + (e.id === tank ? 80 : 0) + div(dist(e.x, e.y, t.me.x, t.me.y), 20000);
    }
    prio(e, v) {
      return this.gate(G.priority) ? 30 + div(100 - div(e.hp * 100, Math.max(1, e.max_hp)), 2) + (e.id === v.tank ? -40 : 0) : 20;
    }
    lead(f, delay) {
      if (this.rank === 0) return [f.x, f.y];
      return this.forecast(f, Math.min(delay, this.table('look_ahead')));
    }
    delayTo(t, x, y, kind) { return openDelay(dist(t.me.x, t.me.y, x, y), RIFTS[kind].delay_pct); }
    offPath(t, x, y) {
      if (!this.gate(G.safety)) return true;
      return !t.enemies().some(e => { const [fx, fy] = this.forecast(e, 60); return segDist(e.x, e.y, fx, fy, x, y) <= C.break_core * 2; });
    }
    exitOk(t, x, y) { return !this.gate(G.safety) || t.danger(x, y, 35000) < 3; }
    dimValue(dim) {
      const p = DIMS[dim].prim;
      if ([P.BANISH, P.STUN, P.ROOT, P.SILENCE, P.DISARM].includes(p)) return 18;
      if ([P.SLOW, P.CDUP].includes(p)) return 10;
      if ([P.AMPLIFY, P.DOT, P.TRUE, P.STATDOWN, P.HEALCUT, P.DRAIN, P.SHATTER].includes(p)) return 14;
      if ([P.REFLECT, P.PULL, P.KNOCK, P.SHOVE, P.GRAB, P.REWIND, P.TETHER].includes(p)) return 6;
      if ([P.STATICBURST, P.ENTROPY, P.STORM].includes(p)) return 8;
      if ([P.LENS, P.ECHO, P.PRISM, P.WILDECHO].includes(p)) return 16;
      if (p === P.UNSTABLE) return this.strain > 60 ? -15 : 10;
      if (p === P.VENT) return this.strain > 50 ? 14 : 4;
      if (p === P.SWAPSPACE || p === P.SCRAMBLE) return -6;
      return PRIM_SIDE[p] === 'A' ? 6 : PRIM_SIDE[p] === 'S' ? 8 : 5;
    }
    planChain(t, target, kindMid) {
      const guess = this.lead(target, 40);
      const reach = div(C.attack_range * 7, 10);
      const d = dist(t.me.x, t.me.y, guess[0], guess[1]);
      if (d <= reach + 20000) return null;
      const exit = along(guess[0], guess[1], t.me.x - guess[0], t.me.y - guess[1], reach);
      const span = dist(t.me.x, t.me.y, exit[0], exit[1]);
      const max = this.gate(G.geometry) ? this.table('max_links') : 1;
      let best = [-Infinity, 1, 0];
      for (let n = 1; n <= max; n++) {
        const eff = chainBp(Array(n).fill(div(span, n)));
        const score = eff - n * 400;
        if (score > best[0]) best = [score, n, eff];
      }
      const n = best[1];
      const px = -(exit[1] - t.me.y), py = exit[0] - t.me.x;
      const nodes = [];
      for (let i = 1; i <= n; i++) {
        let p = [t.me.x + div((exit[0] - t.me.x) * i, n), t.me.y + div((exit[1] - t.me.y) * i, n)];
        const routing = this.table('atlas_routing');
        if (routing >= 1) {
          const here = this.dimValue(this.atlas.dimAt(p[0], p[1], t.tick));
          let pick = [here, p];
          for (const off of [-div(C.atlas_cell, 2), div(C.atlas_cell, 2)]) {
            const o = along(0, 0, px, py, off);
            const q = clampMap(p[0] + o[0], p[1] + o[1]);
            const v = this.dimValue(this.atlas.dimAt(q[0], q[1], t.tick));
            const better = routing >= 2 ? v > pick[0] : (here < 0 && v > pick[0]);
            if (better && !t.wall(q[0], q[1])) pick = [v, q];
          }
          p = pick[1];
        }
        if (!this.offPath(t, p[0], p[1])) { const o = along(0, 0, px, py, 24000); p = clampMap(p[0] + o[0], p[1] + o[1]); }
        const kind = (i === 1 && kindMid !== R.tear && n > 1) ? kindMid : R.tear;
        if (kind === R.mirror) {
          const [hx, hy] = along(0, 0, px, py, 20000);
          nodes.push({ ...Spec.line(R.mirror, p[0] - hx, p[1] - hy, p[0] + hx, p[1] + hy), chain: true });
        } else nodes.push(Spec.node(kind, p[0], p[1]));
      }
      const links = [];
      let prev = [t.me.x, t.me.y];
      for (const s of nodes) { links.push(dist(prev[0], prev[1], s.x, s.y)); prev = [s.x, s.y]; }
      return [nodes, chainBp(links), n];
    }
    wallBetween(t, a, b) {
      const steps = Math.max(1, div(dist(a[0], a[1], b[0], b[1]), 16000));
      for (let i = 1; i < steps; i++) if (t.wall(a[0] + div((b[0] - a[0]) * i, steps), a[1] + div((b[1] - a[1]) * i, steps))) return true;
      return false;
    }
    safeBack(v, t, len) {
      let dx = v.ally_centre[0] - v.enemy_centre[0], dy = v.ally_centre[1] - v.enemy_centre[1];
      if (dx === 0 && dy === 0) { dx = t.me.x - v.enemy_centre[0]; dy = t.me.y - v.enemy_centre[1]; }
      const a = along(t.me.x, t.me.y, dx, dy, len);
      return clampMap(a[0], a[1]);
    }
    randomNear(x, y, r) {
      const [ux, uy] = DIRS[this.random(16)];
      const k = this.random(Math.max(1, r));
      return clampMap(x + div(ux * k, 10000), y + div(uy * k, 10000));
    }
    propose(t, v, tech) {
      const tick = t.tick;
      const me = [t.me.x, t.me.y];
      const f = v.focus;
      const act = (rifts, subject, uses, score, habit) => ({ tech, rifts, subject: subject === undefined ? null : subject, uses, score, habit: habit || '' });
      const live = this.rifts.filter(r => riftLive(r) && !riftHidden(r, tick)).map(cloneRift);
      const nodes = this.chainNodes(tick);
      const chainLive = nodes.length > 0;
      const reachOf = r => t.enemies().filter(e => riftReach(r, e.x, e.y) <= div(r.radius * 3, 2)).length;
      const out = [];
      switch (tech) {
        case T.chain:
          if (chainLive) {
            if (this.gate(G.spine)) {
              const n = nodes.find(n => n.expires <= tick + 60);
              if (n) out.push(act([{ ...Spec.node(R.tear, n.x, n.y), order: n.chain }], null, [], 46, 'spine refreshed'));
            }
          } else {
            const pc = this.planChain(t, f, R.tear);
            if (pc) {
              const [rifts, eff, n] = pc;
              out.push(act(rifts, f.id, [], 28 + div(eff, 300), n >= 2 ? 'solved the link count' : this.rank >= 1 ? 'led target' : ''));
            }
          }
          break;
        case T.bank:
          if (!chainLive) {
            const walled = this.wallBetween(t, me, [f.x, f.y]);
            const pc = this.planChain(t, f, R.mirror);
            if (pc) {
              const [rifts, eff, n] = pc;
              if (n === 1) {
                const p = [rifts[0].x, rifts[0].y];
                let [ox, oy] = this.gate(G.safety) ? along(0, 0, -(p[1] - me[1]), p[0] - me[0], 20000) : DIRS[this.random(16)];
                if (!this.gate(G.safety)) { ox *= 2; oy *= 2; }
                rifts[0] = { ...Spec.line(R.mirror, p[0] - ox, p[1] - oy, p[0] + ox, p[1] + oy), chain: true };
              }
              const score = this.gate(G.geometry) ? (walled ? 40 + div(eff, 400) : 0) : 12 + div(eff, 600);
              out.push(act(rifts, f.id, [], score, walled ? 'bank shot around a wall' : ''));
            }
          }
          break;
        case T.crossfire:
          if (chainLive && live.length >= 2 && tick >= this.crossfire_until) {
            const covered = live.filter(r => t.enemies().some(e => dist(e.x, e.y, r.x, r.y) <= C.attack_range)).length;
            out.push(act([], null, [], this.gate(G.geometry) ? 12 * Math.min(3, covered) : 18, covered >= 2 ? 'crossfire split' : ''));
          }
          break;
        case T.rain: {
          let spot = [f.x, f.y], count = 0;
          if (this.gate(G.geometry)) {
            for (const e of v.enemies) {
              const p = this.lead(e, this.delayTo(t, e.x, e.y, R.sky));
              const c = v.enemies.filter(o => dist(o.x, o.y, p[0], p[1]) <= 26000).length;
              if (c > count) { count = c; spot = p; }
            }
          } else count = v.enemies.filter(o => dist(o.x, o.y, spot[0], spot[1]) <= 26000).length;
          out.push(act([Spec.at(R.sky, spot[0], spot[1])], null, [], 14 * count - 4, count >= 2 ? 'rain on a cluster' : ''));
          break;
        }
        case T.volley: {
          const p = this.lead(f, this.delayTo(t, f.x, f.y, R.slash));
          const [dx, dy] = this.gate(G.geometry) ? [-(p[1] - me[1]), p[0] - me[0]] : DIRS[this.random(16)];
          const a = along(p[0], p[1], dx, dy, 30000), b = along(p[0], p[1], -dx, -dy, 30000);
          const n = v.enemies.filter(e => segDist(a[0], a[1], b[0], b[1], e.x, e.y) <= 22000).length;
          out.push(act([Spec.line(R.slash, a[0], a[1], b[0], b[1])], null, [], 10 * n + 2, ''));
          break;
        }
        case T.implode:
          if (live.length) {
            const doomed = maxBy(live.filter(r => [...r.dwell].some(([id, n]) => id < 1000000 && n >= 20)), r => r.uid);
            if (this.gate(G.safety) && doomed) out.push(act([], null, [doomed.uid], 34 + 6 * reachOf(doomed), 'collapsed a rift before it broke'));
            const pick = this.gate(G.geometry) ? maxBy(live, r => [reachOf(r), r.uid]) : live[this.random(live.length)];
            if (pick) {
              const n = reachOf(pick);
              const low = t.enemies().some(e => riftReach(pick, e.x, e.y) <= div(pick.radius * 3, 2) && e.hp * 4 < e.max_hp);
              const score = this.gate(G.geometry) ? ((n >= 2 || low) ? 16 * n + 6 : -20) : 10 + 8 * n;
              out.push(act([], null, [pick.uid], score, n >= 2 ? 'implode on a pile' : ''));
            }
          }
          break;
        case T.domino: {
          const linked = live.filter(r => r.chain !== null || r.kind === R.twin);
          if (linked.length >= 2) {
            const total = linked.reduce((a, r) => a + reachOf(r), 0);
            const score = this.gate(G.geometry) ? (total >= 2 ? 9 * total + 8 : -20) : 12;
            out.push(act([], null, linked.map(r => r.uid).sort((a, b) => a - b), score, total >= 2 ? 'domino through the pile' : ''));
          }
          break;
        }
        case T.overload:
          if (chainLive && this.strain >= 25 && this.overload === 0) {
            const spend = Math.min(this.strain, C.overload_max);
            const score = (this.gate(G.spine) && this.strain >= 80) ? div(spend, 3) + 25 : this.gate(G.safety) ? div(spend, 3) : 14;
            out.push(act([], null, [], score, (this.gate(G.spine) && this.strain >= 80) ? 'rode strain into Overload' : ''));
          }
          break;
        case T.step:
          if (tick >= this.self_move_until) {
            if (this.gate(G.survival) && v.diver) {
              const to = this.safeBack(v, t, 60000);
              out.push(act([Spec.entrance(R.tear, me[0], me[1]), Spec.at(R.tear, to[0], to[1])], null, [], 55, 'escaped a dive'));
            } else {
              const to = along(f.x, f.y, me[0] - f.x, me[1] - f.y, 15000);
              if (this.exitOk(t, to[0], to[1])) out.push(act([Spec.entrance(R.tear, me[0], me[1]), Spec.at(R.tear, to[0], to[1])], null, [], this.gate(G.survival) ? 0 : 16, ''));
            }
          }
          break;
        case T.gate: case T.skydrop: {
          const exitKind = tech === T.skydrop ? R.sky : R.tear;
          if (this.gate(G.team)) {
            const fr = v.front;
            if (fr) {
              const [vx, vy] = this.velocity.get(fr.id) || [0, 0];
              const toward = vx * (v.enemy_centre[0] - fr.x) + vy * (v.enemy_centre[1] - fr.y) > 0;
              const close = dist(fr.x, fr.y, v.enemy_centre[0], v.enemy_centre[1]) < 170000 && t.danger(fr.x, fr.y, 45000) === 0;
              const wait = this.table('ally_wait');
              const ent = this.lead(fr, this.delayTo(t, fr.x, fr.y, R.tear) + wait);
              const dx = v.enemy_centre[0] - v.ally_centre[0], dy = v.enemy_centre[1] - v.ally_centre[1];
              const bh = along(v.enemy_centre[0], v.enemy_centre[1], dx, dy, 18000);
              const behind = clampMap(bh[0], bh[1]);
              const cluster = v.enemies.filter(e => dist(e.x, e.y, behind[0], behind[1]) <= 40000).length;
              const score = (toward && close) ? (tech === T.skydrop ? 40 + 6 * cluster : 50) : 8;
              out.push(act([Spec.entrance(R.tear, ent[0], ent[1]), Spec.at(exitKind, behind[0], behind[1])], fr.id, [], score, (toward && close) ? 'Gate synced with the tank' : ''));
            }
          } else if (v.allies.length) {
            const a = v.allies[this.random(v.allies.length)];
            out.push(act([Spec.entrance(R.tear, a.x, a.y), Spec.at(exitKind, f.x, f.y)], a.id, [], 16, ''));
          }
          break;
        }
        case T.kidnap: {
          const subject = f;
          if (!(this.gate(G.safety) && subject.cc_immune)) {
            const ent = this.lead(subject, this.delayTo(t, subject.x, subject.y, R.tear));
            let exit;
            if (this.gate(G.team) && v.allies.length) {
              const front = minBy(v.allies, a => [dist2(a.x, a.y, v.enemy_centre[0], v.enemy_centre[1]), a.id]);
              exit = along(front.x, front.y, v.ally_centre[0] - front.x, v.ally_centre[1] - front.y, 8000);
            } else if (this.gate(G.safety)) {
              const p = this.safeBack(v, t, 45000);
              exit = dist(p[0], p[1], me[0], me[1]) > 25000 ? p : null;
            } else exit = me;
            if (exit) {
              const score = this.gate(G.team) ? 44 + div(this.prio(subject, v), 2) : 20;
              out.push(act([Spec.entrance(R.tear, ent[0], ent[1]), Spec.at(R.tear, exit[0], exit[1])], subject.id, [], score, (this.gate(G.priority) && subject.id !== v.tank) ? 'Kidnapped the carry into the team' : ''));
            }
          }
          break;
        }
        case T.sinktrap: {
          const spot = this.gate(G.priority) ? lerp(v.enemy_centre[0], v.enemy_centre[1], v.ally_centre[0], v.ally_centre[1], 35) : this.lead(f, this.delayTo(t, f.x, f.y, R.sinkhole));
          const n = v.enemies.filter(e => dist(e.x, e.y, spot[0], spot[1]) <= 40000).length;
          const exit = this.gate(G.team) ? v.ally_centre : me;
          out.push(act([Spec.entrance(R.sinkhole, spot[0], spot[1]), Spec.at(R.tear, exit[0], exit[1])], null, [], 9 * n + 10, this.gate(G.priority) ? 'Sinkhole at the chokepoint' : ''));
          break;
        }
        case T.swap:
          if (this.gate(G.team)) {
            const fr = v.front;
            if (fr) {
              const a = this.lead(f, this.delayTo(t, f.x, f.y, R.twin));
              const b = this.lead(fr, this.delayTo(t, fr.x, fr.y, R.twin));
              out.push(act([Spec.entrance(R.twin, a[0], a[1]), Spec.at(R.twin, b[0], b[1])], f.id, [], 36 + div(this.prio(f, v), 3), 'swapped the target into the team'));
            }
          } else if (!this.gate(G.safety) || t.danger(f.x, f.y, 35000) < 3) {
            out.push(act([Spec.entrance(R.twin, me[0], me[1]), Spec.at(R.twin, f.x, f.y)], f.id, [], 18, ''));
          }
          break;
        case T.fling: {
          const start = this.lead(f, this.delayTo(t, f.x, f.y, R.wander));
          const [dx, dy] = this.gate(G.team) ? [v.ally_centre[0] - start[0], v.ally_centre[1] - start[1]] : DIRS[this.random(16)];
          const e = along(start[0], start[1], dx, dy, 110000);
          const end = clampMap(e[0], e[1]);
          const n = v.enemies.filter(e => dist(e.x, e.y, start[0], start[1]) <= 30000).length;
          out.push(act([Spec.line(R.wander, start[0], start[1], end[0], end[1])], null, [], 10 * n + 4, this.gate(G.team) ? 'Fling toward the team' : ''));
          break;
        }
        case T.hook:
          if (!(this.gate(G.safety) && f.cc_immune)) {
            const rifts = [{ ...Spec.at(R.anchor, f.x, f.y), host: f.id }];
            if (this.gate(G.team)) rifts.push(Spec.at(R.tear, v.ally_centre[0], v.ally_centre[1]));
            out.push(act(rifts, f.id, [], 22 + div(this.prio(f, v), 3), ''));
          }
          break;
        case T.phase:
          if (tick >= this.self_move_until) {
            const hurt = t.me.hp * 2 < t.me.max_hp && t.danger(me[0], me[1], 40000) >= 1;
            const to = this.gate(G.survival) ? this.safeBack(v, t, 50000) : this.randomNear(me[0], me[1], 50000);
            const score = this.gate(G.survival) ? (hurt ? 55 : -10) : 12;
            out.push(act([Spec.entrance(R.tear, me[0], me[1]), Spec.at(R.tear, to[0], to[1])], null, [], score, hurt ? 'phased out of a burst' : ''));
          }
          break;
        case T.curtain: case T.reflect: {
          const kind = tech === T.curtain ? R.slash : R.mirror;
          const e = minBy(v.enemies.filter(e => dist(e.x, e.y, me[0], me[1]) <= 90000), e => [dist2(e.x, e.y, me[0], me[1]), e.id]);
          if (e) {
            const mid = lerp(me[0], me[1], e.x, e.y, 45);
            const half = div(RIFTS[kind].length, 2);
            const [dx, dy] = this.gate(tech === T.reflect ? G.safety : G.survival) ? [-(e.y - me[1]), e.x - me[0]] : DIRS[this.random(16)];
            const a = along(mid[0], mid[1], dx, dy, half), b = along(mid[0], mid[1], -dx, -dy, half);
            const score = this.gate(G.survival) ? (v.diver ? 40 : 0) : 12;
            out.push(act([Spec.line(kind, a[0], a[1], b[0], b[1])], null, [], score, v.diver ? 'peeled a diver' : ''));
          }
          break;
        }
        case T.rescue: case T.pocket: {
          const lim = tech === T.rescue ? 25 : 40;
          const low = minBy(v.allies.filter(a => div(a.hp * 100, Math.max(1, a.max_hp)) < lim), a => [a.hp, a.id]);
          const pick = this.gate(G.survival) ? low : (v.allies.length ? v.allies[this.random(v.allies.length)] : null);
          if (pick) {
            const a = pick;
            const pressed = t.danger(a.x, a.y, 40000) >= (tech === T.rescue ? 1 : 2);
            const ent = this.lead(a, this.delayTo(t, a.x, a.y, R.tear));
            const score = this.gate(G.survival) ? (pressed ? 58 : 10) : 12;
            const rifts = [Spec.entrance(R.tear, ent[0], ent[1])];
            if (tech === T.rescue) { const safe = this.safeBack(v, t, 30000); rifts.push(Spec.at(R.tear, safe[0], safe[1])); }
            out.push(act(rifts, a.id, [], score, pressed ? 'saved an ally' : ''));
          }
          break;
        }
        case T.lifeline:
          if (!this.rifts.some(r => !r.done && r.tech === T.lifeline)) {
            const close = v.enemies.some(e => dist(e.x, e.y, me[0], me[1]) <= 200000);
            const spot = this.gate(G.survival) ? this.safeBack(v, t, 80000) : this.randomNear(me[0], me[1], 80000);
            const score = this.gate(G.survival) ? (close ? 32 : 0) : 10;
            out.push(act([Spec.at(R.anchor, spot[0], spot[1])], t.me.id, [], score, close ? 'Lifeline set before the fight' : ''));
          }
          break;
        case T.escort:
          if (!chainLive) {
            const pick = this.gate(G.team) ? v.front : (v.allies.length ? v.allies[this.random(v.allies.length)] : null);
            if (pick) {
              const diving = dist(pick.x, pick.y, v.enemy_centre[0], v.enemy_centre[1]) < 90000;
              const score = this.gate(G.team) ? (diving ? 42 : 6) : 12;
              out.push(act([{ ...Spec.at(R.anchor, pick.x, pick.y), host: pick.id, chain: true }], pick.id, [], score, diving ? 'Escort on the diver' : ''));
            }
          }
          break;
        case T.rotation: {
          const far = minBy(v.allies.filter(a => dist(a.x, a.y, v.enemy_centre[0], v.enemy_centre[1]) > 220000), a => a.id);
          const fighting = v.allies.filter(a => dist(a.x, a.y, v.enemy_centre[0], v.enemy_centre[1]) <= 120000).length;
          if (far) {
            const score = this.gate(G.team) ? (fighting >= 1 ? 30 : 4) : 8;
            const ent = this.lead(far, this.delayTo(t, far.x, far.y, R.twin));
            const nf = lerp(v.enemy_centre[0], v.enemy_centre[1], me[0], me[1], 50);
            out.push(act([Spec.entrance(R.twin, ent[0], ent[1]), Spec.at(R.twin, nf[0], nf[1])], far.id, [], score, fighting >= 1 ? 'rotated an ally into the fight' : ''));
          }
          break;
        }
        case T.seedtrap: {
          const subject = f;
          const spot = this.rank === 0 ? this.randomNear(subject.x, subject.y, 60000) : this.forecast(subject, this.table('look_ahead') * 2);
          const rifts = [Spec.at(R.seed, spot[0], spot[1])];
          if (this.gate(G.team)) rifts.push(Spec.at(R.tear, v.ally_centre[0], v.ally_centre[1]));
          out.push(act(rifts, subject.id, [], 22 + (this.gate(G.priority) ? 10 : 0), this.gate(G.priority) ? "Seed on the carry's path" : ''));
          break;
        }
        case T.sweep: {
          const c = v.enemy_centre;
          const dx = -(c[1] - me[1]), dy = c[0] - me[0];
          const a = along(c[0], c[1], dx, dy, 50000), b = along(c[0], c[1], -dx, -dy, 50000);
          const n = v.enemies.filter(e => segDist(a[0], a[1], b[0], b[1], e.x, e.y) <= 20000).length;
          out.push(act([Spec.line(R.wander, a[0], a[1], b[0], b[1])], null, [], 8 * n, ''));
          break;
        }
        case T.borrow:
          if (nodes.length) {
            const exit = nodes[nodes.length - 1];
            let cell, val;
            if (this.table('atlas_routing') >= 2) {
              let best = [[0, 0], -Infinity];
              for (let d = 0; d < 100; d++) {
                const c = this.atlas.centreOf(d, tick);
                if (dist(c[0], c[1], me[0], me[1]) <= 320000 && !t.wall(c[0], c[1])) {
                  const vv = this.dimValue(d);
                  if (vv > best[1]) best = [c, vv];
                }
              }
              [cell, val] = best;
            } else {
              cell = this.randomNear(me[0], me[1], 300000);
              val = this.dimValue(this.atlas.dimAt(cell[0], cell[1], tick));
            }
            if (val > -Infinity) {
              const routing2 = this.table('atlas_routing') >= 2;
              out.push(act([Spec.at(R.tear, cell[0], cell[1])], null, [exit.uid], routing2 ? val * 2 : 10, routing2 ? 'borrowed a far dimension' : ''));
            }
          }
          break;
        case T.stitch: {
          const tears = live.filter(r => r.kind === R.tear && r.dim2 === null);
          let pair = null;
          for (let i = 0; i < tears.length; i++) {
            for (let j = i + 1; j < tears.length; j++) {
              const a = tears[i], b = tears[j];
              if (dist(a.x, a.y, b.x, b.y) <= 40000) {
                const fresh = DIMS[a.dim].family !== DIMS[b.dim].family;
                if (pair === null || (fresh && this.gate(G.priority))) pair = [a.uid, b.uid, fresh];
              }
            }
          }
          if (pair) out.push(act([], null, [pair[0], pair[1]], this.gate(G.priority) ? (pair[2] ? 26 : 4) : 12, pair[2] ? 'stitched two worlds' : ''));
          break;
        }
        default: break;
      }
      return out;
    }
    static cost(a) { return a.rifts.reduce((s, r) => s + RIFTS[r.kind].cost, 0); }
    strainOf(a, me) {
      let s = TECHS[a.tech].strain + C.strain_tear * a.rifts.length;
      let prev = me;
      for (const r of a.rifts.filter(r => r.chain)) { if (dist(prev[0], prev[1], r.x, r.y) > C.link_scale) s += C.strain_long; prev = [r.x, r.y]; }
      return s;
    }
    affordable(a, me) {
      const cost = Core.cost(a);
      const reserve = TECHS[a.tech].group === 'defense' ? 0 : this.table('reserve');
      if (this.charges.available() < cost + (cost > 0 ? reserve : 0)) return false;
      if (this.openCount() + a.rifts.length > C.max_rifts) return false;
      if (a.uses.some(u => !this.rift(u))) return false;
      const buffer = this.gate(G.survival) ? C.strain_break : 0;
      return !this.gate(G.safety) || this.strain + this.strainOf(a, me) + buffer <= this.table('strain_cap');
    }
    think(out, t) {
      const tick = t.tick;
      const enemies = t.enemies();
      if (!enemies.length || enemies.every(e => dist(e.x, e.y, t.me.x, t.me.y) > 320000)) return;
      const v = this.view(t, enemies);
      const k = Math.min(28, this.table('tech_options'));
      const order = Array.from({ length: 28 }, (_, i) => i);
      for (let i = 0; i < k; i++) { const j = i + this.random(28 - i); [order[i], order[j]] = [order[j], order[i]]; }
      const pool = order.slice(0, k);
      if (tick >= this.plan_until) this.plan = [];
      const next = this.plan.length ? this.plan[0] : null;
      if (next !== null && !pool.includes(next)) pool.push(next);
      pool.sort((a, b) => a - b);
      const noise = this.table('noise_pct');
      const me = [t.me.x, t.me.y];
      let best = null;
      for (const tech of pool) {
        for (const a of this.propose(t, v, tech)) {
          if (!this.affordable(a, me)) continue;
          if (tech === next) a.score += 30;
          const depth = this.table('combo_depth');
          if (next === null && depth >= 3 && a.score >= 20 && COMBOS.some(c => c[0] === tech)) a.score += 4 * (depth - 1);
          if (noise > 0) a.score += this.random(2 * noise + 1) - noise;
          if (best === null || a.score > best.score) best = a;
        }
      }
      if (!best || best.score < 12) return;
      const tech = best.tech;
      this.execute(out, t, best);
      const depth = this.table('combo_depth');
      if (this.plan.length && this.plan[0] === tech) this.plan.shift();
      else if (depth > 1 && !this.plan.length) {
        for (const combo of COMBOS) {
          const i = combo.indexOf(tech);
          if (i >= 0) {
            const rest = combo.slice(i + 1, i + 1 + (depth - 1));
            if (rest.length) {
              this.plan = rest;
              this.plan_until = tick + 360;
              this.habit(depth === 2 ? 'planned 2 steps ahead' : depth === 3 ? 'planned 3 steps ahead' : depth === 4 ? 'planned 4 steps ahead' : 'planned a 5-step sequence');
              break;
            }
          }
        }
      }
    }
    execute(out, t, a) {
      const tick = t.tick;
      const cost = Core.cost(a);
      if (cost > 0 && !this.charges.spend(tick, cost)) return;
      this.addStrain(this.strainOf(a, [t.me.x, t.me.y]));
      this.stats.tech_uses[a.tech]++;
      this.habit(a.habit);
      out.push({ k: 'Log', s: `tick ${tick} rank ${RANKS[this.rank]} ${TECHS[a.tech].name} score ${a.score} rifts ${a.rifts.length} strain ${this.strain}` });
      this.fx(out, `tech_${TECHS[a.tech].id}_t${this.tier4()}`, t.me.x, t.me.y, t.me.id, 24);
      const uids = a.rifts.map(s => this.open(out, t, s, a.tech, a.subject));
      if (uids.length >= 2 && a.tech !== T.chain && a.tech !== T.bank) {
        const first = uids[0], last = uids[uids.length - 1];
        const r1 = this.rift(first); if (r1) r1.peer = last;
        const r2 = this.rift(last); if (r2) r2.peer = first;
      }
      if (a.tech === T.hook && uids.length === 2) { const r = this.rift(uids[0]); if (r) r.expires = r.opens + 90; }
      if (a.tech === T.seedtrap || a.tech === T.sinktrap) this.stats.traps++;
      if (a.tech === T.lifeline && uids.length) { const r = this.rift(uids[0]); if (r) r.expires = r.opens + 1200; }
      const tier = this.tier4();
      switch (a.tech) {
        case T.implode: if (a.uses.length) this.implode(out, t, a.uses[0]); break;
        case T.domino: a.uses.forEach((u, i) => { if (i === 0) this.implode(out, t, u); else this.later.push({ kind: 'Domino', at: tick + 15 * i, uid: u }); }); break;
        case T.crossfire: this.crossfire_until = tick + 180; this.fx(out, 'tech_crossfire_t' + tier, t.me.x, t.me.y, t.me.id, 30); break;
        case T.overload: {
          const spend = Math.min(this.strain, C.overload_max);
          this.overload = spend;
          this.addStrain(-spend);
          this.fx(out, 'tech_overload_t' + tier, t.me.x, t.me.y, t.me.id, 30);
          break;
        }
        case T.borrow:
          if (a.uses.length && uids.length) {
            const far = this.rift(uids[0]);
            const near = this.rift(a.uses[0]);
            if (far && near) near.borrowed = [far.dim, tick + 120];
          }
          break;
        case T.stitch:
          if (a.uses.length >= 2) {
            const b = this.rift(a.uses[1]);
            const ra = this.rift(a.uses[0]);
            if (b && ra) {
              const bc = cloneRift(b);
              ra.x = div(ra.x + bc.x, 2); ra.y = div(ra.y + bc.y, 2);
              ra.radius = div(ra.radius * 14, 10);
              ra.dim2 = bc.dim;
              ra.expires = Math.max(ra.expires, bc.expires);
              const x = ra.x, y = ra.y;
              this.close(out, a.uses[1], 'close');
              this.fx(out, 'tech_stitch_t' + tier, x, y, null, 30);
            }
          }
          break;
        default: break;
      }
    }
    place(input, a) {
      const out = [];
      const me = input.fighters.find(f => f.id === input.me);
      if (!me) return out;
      const t = new Tick(input.tick, me, input.fighters, input.wall, Math.max(1, input.attack_interval), input.stunned);
      this.fx_left = C.fx_per_update;
      this.execute(out, t, a);
      return out;
    }
    onDead(tick) {
      if (tick <= this.risky_until) this.stats.self_deaths++;
      this.rifts = []; this.packets = []; this.later = []; this.dots = [];
      this.aiming = false; this.phased = false; this.chain_target = null; this.plan = []; this.overload = 0;
      this.damage.clear(); this.controls.clear(); this.moves.clear();
    }
  }

  // ------------------------------------------------------------------ the practice arena (vesper_tests.rs Arena)
  class Unit {
    constructor(id, team, x, y, maxHp, attack, range) {
      this.f = { id, team, x, y, hp: maxHp, max_hp: maxHp, attack, cc_immune: false, controlled: false };
      this.speed = 900; this.range = range; this.interval = 70; this.next_attack = 0;
      this.stun = 0; this.root = 0; this.disarm = 0; this.banish = 0; this.invisible = 0; this.shield = 0;
      this.buffs = new Map(); this.dealt = 0;
    }
    mods(tick) {
      const m = Mods();
      for (const [until, b] of this.buffs.values()) {
        if (until > tick) {
          m.move_speed += b.move_speed; m.attack_speed += b.attack_speed; m.amplify += b.amplify; m.reduce += b.reduce; m.attack += b.attack;
          m.cc_immune = m.cc_immune || b.cc_immune; m.undying = m.undying || b.undying;
        }
      }
      return m;
    }
    clone() { const u = Object.assign(new Unit(0, 0, 0, 0, 1, 0, 0), this); u.f = { ...this.f }; u.buffs = new Map(this.buffs); return u; }
  }
  const NO_WALL = () => false;
  class Arena {
    constructor(seed, rank, units, top) {
      this.tick = 0; this.units = units; this.core = new Core(seed, 0, rank, top); this.wall = NO_WALL;
      this.fx = 0; this.cmds = []; this.keep_cmds = false; this.vesper_alive = true; this.frozen = false;
    }
    unit(id) { return this.units.find(u => u.f.id === id); }
    fighters() {
      const tick = this.tick;
      return this.units.filter(u => u.f.hp > 0 && u.banish <= tick && (u.f.team === 0 || u.invisible <= tick)).map(u => {
        const m = u.mods(tick);
        return { ...u.f, cc_immune: m.cc_immune, controlled: u.stun > tick || u.root > tick || u.disarm > tick };
      });
    }
    input() {
      const v = this.unit(0);
      return { tick: this.tick, me: 0, fighters: this.fighters(), attack_interval: v.interval, stunned: v.stun > this.tick || v.banish > this.tick, wall: this.wall };
    }
    place(a) { const cmds = this.core.place(this.input(), a); this.apply(cmds); return cmds; }
    hurt(target, amount, from) {
      const tick = this.tick;
      const u = this.units.find(u => u.f.id === target && u.f.hp > 0);
      if (!u || u.banish > tick) return;
      const m = u.mods(tick);
      let n = div(amount * Math.max(10, 100 + m.amplify - m.reduce), 100);
      const absorbed = Math.min(n, u.shield);
      u.shield -= absorbed; n -= absorbed;
      u.f.hp -= n;
      if (m.undying && u.f.hp < 1) u.f.hp = 1;
      if (from !== null) { const s = this.units.find(x => x.f.id === from); if (s) s.dealt += n; }
    }
    apply(cmds) {
      const tick = this.tick;
      for (const c of cmds) {
        switch (c.k) {
          case 'Damage': this.hurt(c.target, c.amount, 0); break;
          case 'Heal': { const u = this.units.find(u => u.f.id === c.target && u.f.hp > 0); if (u) u.f.hp = Math.min(u.f.max_hp, u.f.hp + c.amount); break; }
          case 'Shield': this.unit(c.target).shield += c.amount; break;
          case 'Cc': {
            const u = this.unit(c.target), until = tick + c.ticks;
            if (c.kind === 'Stun') u.stun = Math.max(u.stun, until); else if (c.kind === 'Root') u.root = Math.max(u.root, until); else if (c.kind === 'Disarm') u.disarm = Math.max(u.disarm, until);
            break;
          }
          case 'Buff': this.unit(c.target).buffs.set(c.name, [tick + c.ticks, c.mods]); break;
          case 'SetPos': { const u = this.unit(c.target); u.f.x = c.x; u.f.y = c.y; break; }
          case 'Banish': this.unit(c.target).banish = tick + c.ticks; break;
          case 'Invisible': this.unit(c.target).invisible = tick + c.ticks; break;
          case 'ClearCc': { const u = this.unit(c.target); u.stun = 0; u.root = 0; u.disarm = 0; break; }
          case 'ClearShield': this.unit(c.target).shield = 0; break;
          case 'LoseHp': { const u = this.unit(c.target); u.f.hp = Math.max(1, u.f.hp - c.amount); break; }
          case 'Fx': this.fx++; break;
          default: break;
        }
        if (this.keep_cmds) this.cmds.push(c);
      }
    }
    step() {
      const tick = this.tick;
      const vesper = this.unit(0);
      if (vesper.f.hp > 0) {
        this.apply(this.core.step(this.input()));
      } else if (this.vesper_alive) {
        this.vesper_alive = false;
        this.core.onDead(tick);
      }
      if (!this.frozen) for (const id of this.units.filter(u => u.f.hp > 0).map(u => u.f.id)) this.act(id);
      this.tick++;
    }
    act(id) {
      const tick = this.tick;
      const u = this.unit(id).clone();
      if (u.f.hp <= 0 || u.stun > tick || u.banish > tick) return;
      const aiming = id === 0 && this.core.aiming;
      const foes = this.units.filter(o => o.f.team !== u.f.team && o.f.hp > 0 && o.banish <= tick && o.invisible <= tick);
      const target = minBy(foes, o => [dist2(o.f.x, o.f.y, u.f.x, u.f.y), o.f.id]);
      if (!target) return;
      const d = dist(u.f.x, u.f.y, target.f.x, target.f.y);
      const m = u.mods(tick);
      if (d > u.range) {
        if (u.root <= tick && !aiming) {
          const speed = div(u.speed * Math.max(0, 100 + m.move_speed), 100);
          let [nx, ny] = along(u.f.x, u.f.y, target.f.x - u.f.x, target.f.y - u.f.y, Math.min(speed, d - u.range + 1));
          [nx, ny] = land(u.f.x, u.f.y, nx, ny, this.wall);
          const me = this.unit(id);
          me.f.x = nx; me.f.y = ny;
        }
      } else if (u.disarm <= tick && tick >= u.next_attack) {
        const interval = div(u.interval * 100, Math.max(20, 100 + m.attack_speed));
        this.unit(id).next_attack = tick + Math.max(10, interval);
        const dmg = div(u.f.attack * Math.max(0, 100 + m.attack), 100);
        this.hurt(target.f.id, dmg, id);
        if (id === 0) this.core.stats.damage += dmg;
      }
    }
    run(ticks) { for (let i = 0; i < ticks; i++) { this.step(); if (this.wiped() !== null) break; } }
    wiped() {
      for (let team = 0; team < 2; team++) if (!this.units.some(u => u.f.team === team && u.f.hp > 0)) return team;
      return null;
    }
  }
  function duel(seed, rank, top) {
    return new Arena(seed, rank, [new Unit(0, 0, 200000, 480000, 1100, 90, C.attack_range), new Unit(5, 1, 520000, 480000, 2400, 60, 30000)], top);
  }
  function teamfight(seed, rank, top) {
    const [home, away] = seed % 2 === 1 ? [300000, 600000] : [260000, 700000];
    const units = [new Unit(0, 0, home - 80000, 480000, 1100, 90, C.attack_range)];
    const roles = [[2600, 55, 25000], [1800, 80, 28000], [1200, 95, 60000], [1300, 60, 55000]];
    roles.forEach(([hp, atk, range], i) => units.push(new Unit(1 + i, 0, home - div(range, 3), 400000 + i * 50000, hp, atk, range)));
    const foes = [[2600, 55, 25000], [1800, 80, 28000], [1200, 95, 60000], [1300, 60, 55000], [1100, 90, C.attack_range]];
    foes.forEach(([hp, atk, range], i) => units.push(new Unit(10 + i, 1, away + div(range, 3), 380000 + i * 50000, hp, atk, range)));
    const rng = M.seedState(Math.max(1, seed));
    for (const u of units.slice(1)) {
      u.f.y += roll(rng, 40000) - 20000;
      u.f.x += roll(rng, 30000) - 15000;
    }
    return new Arena(seed, rank, units, top);
  }
  // Mirrors vesper_tests::sim_vectors.
  function simVectors() {
    const rows = [];
    for (const seed of [1, 2, 3]) {
      for (const rank of [0, 3, 5, 7]) {
        const a = teamfight(seed, rank);
        a.keep_cmds = true;
        for (let i = 0; i < 900; i++) {
          a.cmds = [];
          a.step();
          for (const c of a.cmds) if (c.k === 'Log') rows.push(`${seed}|${rank}|${c.s}`);
          if (a.wiped() !== null) break;
        }
        const s = a.core.stats;
        const hp = a.units.map(u => `${u.f.id}:${u.f.hp}:${u.f.x}:${u.f.y}`).join(',');
        rows.push(`${seed}|${rank}|end tick ${a.tick} strain ${a.core.strain} damage ${s.damage} packets ${s.packets} eff ${s.eff_sum} breaches ${s.breaches} breaks ${s.breaks} kidnaps ${s.useful_kidnaps}/${s.kidnaps} gates ${s.engaged_gates}/${s.gates} traps ${s.trap_catches}/${s.traps} implodes ${s.implodes} uses [${s.tech_uses.join(', ')}] units ${hp}`);
      }
    }
    return rows.join('\n') + '\n';
  }
  const api = { Core, Arena, Unit, Spec, duel, teamfight, simVectors, heading8, look, artTier, habitGate, land, riftLive, riftHidden, riftDims, riftReach, P, R, T, DIMS, RIFTS, TECHS, TWISTS, RANKS, PRIM_SIDE };
  if (isNode) module.exports = api; else root.VesperCombat = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
