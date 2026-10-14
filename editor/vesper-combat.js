// Vesper's rules for the Rift Lab (the Rift Gauntlets): a 1:1 port of native/tfm2_custom_ai/src/vesper_core.rs (the pure
// engine and its ranked planner) plus the deterministic practice arena of vesper_tests.rs. `node tools/check_vesper_parity.js`
// replays the native simulation vectors decision for decision.
(function (root) {
  'use strict';
  const isNode = typeof module !== 'undefined';
  const D = isNode ? require('./vesper-data.js') : root.RiftData;
  const M = isNode ? require('./vesper-math.js') : root.VesperMath;
  const { div, roll, dist2, dist, keepBp, chainBp, openDelay, twistIndex, lerp, along, clampMap, segDist, side, Window } = M;
  const C = D.constants;
  const P = {};
  D.prims.forEach((p, i) => { P[p.id] = i; });
  const PRIM_SIDE = D.prims.map(p => p.side);
  const PRIM_ONCE = D.prims.map(p => p.once);
  const DIMS = D.dimensions;
  // dimensions are listed family by family
  const FAMILY_START = D.families.map((_, f) => DIMS.findIndex(x => x.family === f));
  const FAMILY_LEN = D.families.map((_, f) => DIMS.filter(x => x.family === f).length);
  const TWISTS = D.twists.map(t => ({ name: t.name, a: t.families[0], b: t.families[1], kind: t.kind, mag: t.mag }));
  const RIFTS = D.rifts;
  const R = {};
  RIFTS.forEach((r, i) => { R[r.id] = i; });
  const TECHS = D.techniques.map(t => ({ ...t, rifts: t.rifts.map(id => R[id]) }));
  const MOVES = D.moves;
  const MV = {};
  MOVES.forEach((m, i) => { MV[m.id] = 28 + i; });
  const MODULES = D.modules;
  const HEX = D.hexcore;
  const NO_EFF = [255, 0, 0, 0];
  const PUNCH = D.dimensions.map(x => ({ main: x.punch.main, rider: x.punch.rider || NO_EFF }));
  const SWIPE = D.dimensions.map(x => ({ main: x.swipe.main, rider: x.swipe.rider || NO_EFF }));
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
  // the look of her pose overlays (native vesper::pose_look): 0-6 ranks, 7 Worldbreaker #4-10, 8 #1, 9 #3, 10 #2
  function poseLook(rank, top) { return rank < 7 ? rank : top === 1 ? 8 : top === 2 ? 10 : top === 3 ? 9 : 7; }
  function artTier(rank) { return [0, 0, 0, 1, 1, 2, 2, 3][Math.min(rank, 7)]; }
  const HABIT_GATE = {
    'led target': 1, 'collapsed a rift before it broke': G.safety, 'shattered while the gauntlet held': G.safety, 'stopped overloading near enemies': G.safety,
    'solved the link count': G.geometry, 'implode on a pile': G.geometry, 'rain on a cluster': G.geometry, 'crossfire split': G.geometry, 'domino through the pile': G.geometry, 'bank shot around a wall': G.geometry, 'swipe through the line': G.geometry,
    'Lifeline set before the fight': G.survival, 'peeled a diver': G.survival, 'saved an ally': G.survival, 'rescued ally': G.survival, 'phased out of a burst': G.survival, 'escaped a dive': G.survival, 'vented overload when dived': G.survival, 'kept a spare orb': G.survival,
    'Escort on the diver': G.team, 'Gate synced with the tank': G.team, 'Fling toward the team': G.team, 'swapped the target into the team': G.team, 'dropped aim for a diver': G.team, 'Awakening timed with the engage': G.team, 'rotated an ally into the fight': G.team,
    'Kidnapped the carry into the team': G.priority, 'Sinkhole at the chokepoint': G.priority, "Seed on the carry's path": G.priority, 'stitched two worlds': G.priority, 'dimension picked for the target': G.priority, "Infused the frontliner's rift": G.priority,
    'spine refreshed': G.spine, 'overload ridden into a Shatter': G.spine, 'overclock stopped in time': G.spine, 'combo families for the twist': G.spine, 'planned a 5-step sequence': G.spine, 'overloaded before the engage': G.spine,
    'planned 2 steps ahead': 3, 'planned 3 steps ahead': 5, 'planned 4 steps ahead': 6,
  };
  const habitGate = h => HABIT_GATE[h] || 0;
  function land(fx, fy, x, y, wall) {
    // no teleport carries anyone farther than teleport_max
    if (dist(fx, fy, x, y) > C.teleport_max) [x, y] = along(fx, fy, x - fx, y - fy, C.teleport_max);
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
  function riftDims(r) {
    const v = [r.dim];
    if (r.dim2 !== null) v.push(r.dim2);
    return v;
  }
  function riftReach(r, x, y) {
    return (r.kind === R.slash || r.kind === R.mirror) ? segDist(r.x, r.y, r.ex, r.ey, x, y) : dist(r.x, r.y, x, y);
  }
  const riftCovers = (r, x, y) => riftReach(r, x, y) <= r.radius;
  // the game's input-AI rule for her enemies (vesper.rs zones_of / steer): [x, y, radius, field] of every visible live
  // rift and armed Kidnap field; never walk into one, and walk straight out of an armed field (the dodge)
  function zonesOf(core, tick) {
    const z = core.rifts.filter(r => riftLive(r) && !riftHidden(r, tick)).map(r => [div(r.x + r.ex, 2), div(r.y + r.ey, 2), r.radius + div(dist(r.x, r.y, r.ex, r.ey), 2), false]);
    for (const k of core.kidnapZones()) z.push([k[0], k[1], k[2], true]);
    return z;
  }
  function steer(zones, me, dest) {
    const outOf = (z, from) => {
      let dx = from[0] - z[0], dy = from[1] - z[1];
      if (dx === 0 && dy === 0) dx = 1;
      return along(z[0], z[1], dx, dy, z[2] + 8000);
    };
    const inside = (z, p) => dist(p[0], p[1], z[0], z[1]) <= z[2];
    const here = zones.find(z => z[3] && inside(z, me));
    if (here) return outOf(here, me);
    const there = zones.find(z => inside(z, dest) && !inside(z, me));
    return there ? outOf(there, dest) : null;
  }
  function cloneRift(r) {
    return { ...r, dwell: new Map(r.dwell), touched: new Set(r.touched), sides: new Map(r.sides), inside: new Set(r.inside), drawn: [...r.drawn] };
  }
  const Spec = {
    at: (kind, x, y) => ({ kind, x, y, ex: x, ey: y, host: null, chain: false, entrance: false, order: null, dim: null }),
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
  function nearestOn(ax, ay, bx, by, x, y) {
    const dx = BigInt(bx - ax), dy = BigInt(by - ay);
    const len = dx * dx + dy * dy;
    if (len === 0n) return [ax, ay];
    let t = BigInt(x - ax) * dx + BigInt(y - ay) * dy;
    if (t < 0n) t = 0n;
    if (t > len) t = len;
    return [ax + Number(dx * t / len), ay + Number(dy * t / len)];
  }
  const isControl = p => [P.BANISH, P.STUN, P.ROOT, P.SILENCE, P.DISARM, P.ENCASE, P.STATICBURST, P.CHILL].includes(p);
  const isDamage = p => [P.AMPLIFY, P.DOT, P.TRUE, P.LENS, P.EXECUTE, P.SPLASH, P.RICOCHET, P.CHAIN, P.ECHO, P.PRISM, P.WILDECHO, P.MARK, P.DRAIN, P.ENTROPY, P.SHATTER, P.STATDOWN, P.HEALCUT, P.MOMENTUM, P.PRESSURE, P.EXPOSE].includes(p);
  const isMove = p => [P.REFLECT, P.PULL, P.KNOCK, P.SHOVE, P.STORM, P.GRAB, P.REWIND, P.TOEND, P.SELFSWAP, P.SCRAMBLE, P.SWAPSPACE, P.TETHER, P.WELL, P.REPEL].includes(p);
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
      this.pure = Array.from({ length: 9 }, (_, i) => ({ open: i < C.orbs_base, over: false, ready_at: 0, family: 255 }));
      this.hex = [0, 0, 0, 0, 0, 0];
      this.modules = M.modulesOf(null);
      this.completion = C.hex_start;
      this.awaken_until = 0; this.corruption = 0; this.left_broken_until = 0; this.right_broken_until = 0;
      this.stress = 0; this.combo = 0; this.combo_at = 0; this.last_family = null;
      this.marks = new Map();
      this.chill = new Map(); this.exposed = new Map();   // Frost stacks [n, last tick]; Radiance marks [until, per mille]
      this.rifts = [];
      this.next_uid = 1;
      this.chain_seq = 0;
      this.packets = [];
      this.later = [];
      this.dots = [];
      this.history = new Map();
      this.velocity = new Map();
      this.damage = new Map(); this.controls = new Map(); this.moves = new Map(); this.support = new Map(); this.heals = new Map();
      this.cooldowns = new Map();
      this.self_move_until = 0; this.next_attack = 0; this.punching = false; this.aiming = false; this.chain_target = null;
      // visual only (pose overlays): the tick each one-shot pose ends
      this.jab_until = 0; this.block_until = 0; this.swipe_until = 0; this.shatter_until = 0;
      this.crossfire_until = 0; this.opunch = 0; this.armed = null; this.plan = []; this.plan_until = 0; this.recent = []; this.last_think = 0;
      this.phased = false; this.risky_until = 0; this.displaced = new Map(); this.hurt = new Map(); this.last_hp = -1; this.slammed = false;
      this.stats = {
        tech_uses: Array(28).fill(0), move_uses: Array(6).fill(0), rifts_opened: 0, rifts_touching: 0, breaks: 0, punches: 0, landed: 0, swipes: 0, swipe_hits: 0,
        eff_sum: 0, damage: 0, orbs: 0, hexorbs: 0, fused: 0, twists: 0, best_combo: 0, shatters: 0, useful_shatters: 0, left_breaks: 0, right_breaks: 0,
        overloads: 0, awakens: 0, good_awakens: 0, takedowns: 0, kidnaps: 0, useful_kidnaps: 0, kidnap_dodges: 0, gates: 0, engaged_gates: 0, lonely_gates: 0, traps: 0,
        trap_catches: 0, risky_moves: 0, self_deaths: 0, implodes: 0, implode_hits: 0, effects: 0, habits: {}, jab_damage: 0,
      };
      this.fx_left = C.fx_per_update; this.fx_tokens = C.fx_tokens; this.kidnap_ready = 0; this.last_tick = null; this.hand_placed = false;
      // per-dimension and per-family ready ticks (shared by both gauntlets); free_casts is the Lab's "ignore cooldowns"
      this.dim_ready = DIMS.map(() => 0); this.family_ready = D.families.map(() => 0); this.free_casts = false;
    }
    random(n) { return roll(this.rng, n); }
    look() { return look(this.rank, this.top); }
    table(name) { return TB[name][Math.min(this.rank, 7)]; }
    gate(g) { return this.rank >= g; }
    fx(out, tag, x, y, unit, ticks) {
      if (this.fx_left > 0 && this.fx_tokens > 0) { this.fx_left--; this.fx_tokens--; this.stats.effects++; out.push({ k: 'Fx', tag, x, y, unit, ticks }); return true; }
      return false;
    }
    // ambient visuals (rift layers, links, packets in flight): only from the tokens above the reserve
    fxLow(out, tag, x, y, unit, ticks) { return this.fx_tokens > C.fx_reserve && this.fx(out, tag, x, y, unit, ticks); }
    habit(h) { if (h && this.rank >= habitGate(h)) this.stats.habits[h] = (this.stats.habits[h] || 0) + 1; }
    rift(uid) { return this.rifts.find(r => r.uid === uid && !r.done) || null; }
    openCount() { return this.rifts.filter(r => !r.done).length; }
    cooldownOk(key, id, tick) { const u = this.cooldowns.get(key + ':' + id); return u === undefined || tick >= u; }
    setCooldown(key, id, until) { this.cooldowns.set(key + ':' + id, until); }
    win(map, id) { if (!map.has(id)) map.set(id, new Window()); return map.get(id); }
    tier4() { return this.look() === 8 ? 4 : artTier(this.rank); }

    // -------------------------------------------------------------- shared limits
    emit(out, f, amount, kind, tick) {
      if (amount <= 0 || f.hp <= 0) return 0;
      const cap = Math.max(1, div(f.max_hp * C.damage_pct, 100));
      const n = this.win(this.damage, f.id).allow(tick, C.damage_window, cap, amount);
      if (n > 0) { out.push({ k: 'Damage', target: f.id, amount: n, kind }); this.stats.damage += n; this.hurt.set(f.id, [tick, f.hp - n, f.max_hp]); }
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
      const want = clampMap(x, y);
      [x, y] = land(f.x, f.y, x, y, wall);
      this.slammed = x !== want[0] || y !== want[1];
      out.push({ k: 'SetPos', target: f.id, x, y });
      this.displaced.set(f.id, tick);
      return true;
    }
    shield(out, f, amount, ticks, tick) {
      const cap = Math.max(1, div(f.max_hp * C.support_pct, 100));
      const n = this.win(this.support, f.id).allow(tick, C.support_window, cap, Math.max(0, amount));
      if (n > 0) out.push({ k: 'Shield', target: f.id, amount: n, ticks });
    }
    // every heal she gives: heal_pct% of the target's max HP per heal_window ticks
    heal(out, t, target, amount, tick) {
      const f = t.get(target);
      if (!f) return;
      const cap = Math.max(1, div(f.max_hp * C.heal_pct, 100));
      const n = this.win(this.heals, target).allow(tick, C.heal_window, cap, Math.max(0, amount));
      if (n > 0) out.push({ k: 'Heal', target, amount: n });
    }
    buff(out, target, name, ticks, mods) {
      if (ticks > 0) out.push({ k: 'Buff', target, name, ticks, mods });
    }

    // -------------------------------------------------------------- effects
    // Momentum: +a% per 10000 units moved in the last momentum_window ticks, at most +b%
    momentum(t, a, b) {
      const back = Math.max(1, div(C.momentum_window, 6));
      const h = this.history.get(t.me.id);
      if (!h || h.length <= back) return 0;
      const [x, y] = h[h.length - 1 - back];
      return Math.min(b, div(dist(x, y, t.me.x, t.me.y) * a, 10000));
    }
    // Radiance: a punch, swipe or implode on an exposed enemy detonates the mark
    detonate(out, f, tick) {
      const e = this.exposed.get(f.id);
      if (!e) return;
      this.exposed.delete(f.id);
      if (tick <= e[0]) { this.emit(out, f, div(f.max_hp * e[1], 1000), 'Magic', tick); this.fx(out, 'hole_l' + this.look(), f.x, f.y, null, 12); }
    }
    // Pressure: a rift that stored charges bursts on every enemy inside when it closes, breaks or implodes
    releasePressure(out, t, r) {
      if (r.charge <= 0) return;
      for (const dim of riftDims(r)) {
        if (DIMS[dim].prim !== P.PRESSURE) continue;
        for (const e of t.enemies().filter(e => riftCovers(r, e.x, e.y))) this.emit(out, e, div(e.max_hp * DIMS[dim].a * r.charge, 1000), 'Magic', t.tick);
        this.fx(out, 'tech_implode_t' + this.tier4(), r.x, r.y, null, 24);
      }
    }
    // Dice: a dimension from any other family, uniformly
    otherFamilyDim(family) {
      let other = this.random(DIMS.length - FAMILY_LEN[family]);
      if (other >= FAMILY_START[family]) other += FAMILY_LEN[family];
      return other;
    }
    applyDim(out, t, dim, f, s, c, src) {
      const d = DIMS[dim];
      if (d.prim === P.DICE) {
        const other = this.otherFamilyDim(d.family);
        if (DIMS[other].prim !== P.DICE) this.applyDim(out, t, other, f, s, c, src);
        return;
      }
      this.applyEff(out, t, dim, [d.prim, d.a, d.b, d.c], f, s, c, src, null);
    }
    applyEff(out, t, dim, e, f, s, c, src, line) {
      const [p, ea, eb, ec] = e;
      if (p === 255) return;
      const sd = PRIM_SIDE[p];
      const enemy = f.team !== t.me.team;
      const me = f.id === t.me.id;
      const ok = sd === 'E' ? enemy : sd === 'A' ? (!enemy && src !== 'Shatter') : sd === 'S' ? (me && src !== 'Shatter') : sd === 'X' ? (src !== 'Shatter' || enemy) : false;
      if (!ok) return;
      const tick = t.tick;
      const sc = v => div(v * s, 100);
      const key = 1000 + p;
      const name = 'vsd_' + DIMS[dim].id;
      const m = Mods();
      switch (p) {
        case P.BANISH:
          if (this.cooldownOk(key, f.id, tick) && this.control(out, f, null, Math.max(12, sc(ea)), tick)) this.setCooldown(key, f.id, tick + 480);
          break;
        case P.STUN: case P.ROOT: case P.SILENCE: case P.DISARM: case P.STATICBURST: {
          const kind = (p === P.STUN || p === P.STATICBURST) ? 'Stun' : p === P.ROOT ? 'Root' : p === P.SILENCE ? 'Silence' : 'Disarm';
          const ticks = p === P.STATICBURST ? this.random(ea + 1) : Math.max(8, sc(ea));
          if (this.cooldownOk(key, f.id, tick) && this.control(out, f, kind, ticks, tick)) this.setCooldown(key, f.id, tick + 240);
          break;
        }
        case P.SLOW: m.move_speed = -sc(ea); m.attack_speed = -sc(eb); this.buff(out, f.id, name, ec, m); break;
        case P.AMPLIFY: m.amplify = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.DOT: case P.DRAIN: case P.ENTROPY: {
          const perMille = p === P.ENTROPY ? this.random(ea + 1) : ea;
          const perPulse = div(f.max_hp * sc(perMille), 2000);
          if (src !== 'Zone' && eb > 0) {
            this.dots = this.dots.filter(x => !(x.target === f.id && x.drain === (p === P.DRAIN)));
            this.dots.push({ target: f.id, per_pulse: perPulse, until: tick + eb, drain: p === P.DRAIN });
          } else {
            const n = this.emit(out, f, perPulse, 'Magic', tick);
            if (p === P.DRAIN && n > 0) this.heal(out, t, t.me.id, n, tick);
          }
          break;
        }
        case P.SHATTER:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'ClearShield', target: f.id }); this.setCooldown(key, f.id, tick + 240); }
          break;
        case P.STATDOWN: case P.STATUP: {
          const v = sc(eb) * (p === P.STATDOWN ? -1 : 1);
          if (ea === 0) m.attack = v; else if (ea === 1) m.defence = v; else if (ea === 2) m.mr = v; else if (ea === 3) m.ap = v;
          else if (p === P.STATDOWN) m.regen = -Math.max(1, div(eb, 10)); else m.crit = v;
          this.buff(out, f.id, name, ec, m);
          break;
        }
        case P.REFLECT: case P.PULL: case P.KNOCK: case P.SHOVE: case P.STORM: case P.GRAB: case P.REWIND: case P.TOEND: {
          if (!this.cooldownOk(key, f.id, tick)) return;
          const cx = c[0], cy = c[1];
          let to;
          if (p === P.REFLECT) to = [2 * cx - f.x, 2 * cy - f.y];
          else if (p === P.PULL) to = lerp(f.x, f.y, cx, cy, Math.min(100, sc(ea)));
          else if (p === P.KNOCK) {
            const [dx, dy] = (f.x !== cx || f.y !== cy) ? [f.x - cx, f.y - cy] : line ? [-(line.by - line.ay), line.bx - line.ax] : [1, 0];
            to = along(f.x, f.y, dx, dy, sc(ea));
          } else if (p === P.SHOVE || p === P.STORM) {
            let vx = 0, vy = 0;
            for (let i = 0; i < (p === P.STORM ? 2 : 1); i++) {
              const [ux, uy] = DIRS[this.random(16)];
              vx += div(ux * sc(ea), 10000); vy += div(uy * sc(ea), 10000);
            }
            to = [f.x + vx, f.y + vy];
          } else if (p === P.GRAB) to = along(f.x, f.y, t.me.x - f.x, t.me.y - f.y, Math.min(sc(ea), dist(f.x, f.y, t.me.x, t.me.y)));
          else if (p === P.TOEND) {
            if (!line) return;
            to = [line.bx, line.by];
          } else {
            const h = this.history.get(f.id);
            if (!h) return;
            const back = Math.max(1, div(div(ea * Math.min(100, Math.max(30, s)), 100), 6));
            if (h.length <= back) return;
            to = h[h.length - 1 - back];
          }
          if (this.displace(out, f, to[0], to[1], tick, t.wall, enemy)) this.setCooldown(key, f.id, tick + (p === P.REWIND ? 480 : 240));
          break;
        }
        case P.WALLSLAM:
          if (this.slammed) this.control(out, f, 'Stun', Math.max(8, sc(ea)), tick);
          break;
        case P.SELFSWAP:
          if (tick >= this.self_move_until && !f.cc_immune && this.displace(out, f, t.me.x, t.me.y, tick, t.wall, true)) {
            this.self_move_until = tick + C.self_move_cd;
            const [x, y] = land(t.me.x, t.me.y, f.x, f.y, t.wall);
            out.push({ k: 'SetPos', target: t.me.id, x, y });
          }
          break;
        case P.BLINK:
          if (tick >= this.self_move_until) {
            this.self_move_until = tick + C.self_move_cd;
            const to = along(t.me.x, t.me.y, c[0] - t.me.x, c[1] - t.me.y, sc(ea));
            const [x, y] = land(t.me.x, t.me.y, to[0], to[1], t.wall);
            out.push({ k: 'SetPos', target: t.me.id, x, y });
          }
          break;
        case P.MARK: this.marks.set(f.id, [tick + eb, sc(ea)]); break;
        case P.ENCASE:
          if (this.control(out, f, 'Root', Math.max(10, sc(ea)), tick)) { m.reduce = eb; this.buff(out, f.id, name, Math.max(10, sc(ea)), m); }
          break;
        case P.RECALLHP: {
          const back = Math.max(1, div(ea, 6));
          const h = this.history.get(f.id);
          if (h && h.length > back) {
            const then = h[h.length - 1 - back][2];
            const lost = Math.min(then - f.hp, div(f.max_hp * eb, 1000));
            if (lost > 0 && this.cooldownOk(key, f.id, tick)) { this.heal(out, t, f.id, sc(lost), tick); this.setCooldown(key, f.id, tick + 600); }
          }
          break;
        }
        case P.CDUP: case P.CDDOWN: m.cooldown = sc(ea) * (p === P.CDUP ? 1 : -1); this.buff(out, f.id, name, eb, m); break;
        case P.HEALCUT: m.heal_reduce = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.SHELTER: m.reduce = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.HEAL:
          if (eb === 0) {
            const k = 3000 + dim;
            if (this.cooldownOk(k, f.id, tick)) { this.heal(out, t, f.id, div(f.max_hp * sc(ea), 1000), tick); this.setCooldown(k, f.id, tick + 600); }
          } else this.heal(out, t, f.id, div(f.max_hp * sc(ea), 2000), tick);
          break;
        case P.SHIELD:
          if (this.cooldownOk(key, f.id, tick)) { this.shield(out, f, div(f.max_hp * sc(ea), 1000), eb, tick); this.setCooldown(key, f.id, tick + 180); }
          break;
        case P.UNDYING: case P.CCIMMUNE:
          if (this.cooldownOk(key, f.id, tick)) {
            if (p === P.UNDYING) m.undying = true; else m.cc_immune = true;
            this.buff(out, f.id, name, Math.max(10, sc(ea)), m);
            this.setCooldown(key, f.id, tick + C.immune_cd);
          }
          break;
        case P.HASTE: m.move_speed = sc(ea); m.attack_speed = sc(eb); this.buff(out, f.id, name, ec, m); break;
        case P.RANGE: m.range = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.GHOST:
          if (this.cooldownOk(key, f.id, tick)) { m.ignore_wall = true; this.buff(out, f.id, name, ea, m); this.setCooldown(key, f.id, tick + C.ghost_cd); }
          break;
        case P.CLEANSE:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'ClearCc', target: f.id }); this.setCooldown(key, f.id, tick + 240); }
          break;
        case P.ALLYINVIS: case P.SELFINVIS:
          if (this.cooldownOk(key, f.id, tick)) { out.push({ k: 'Invisible', target: f.id, ticks: Math.max(10, sc(ea)) }); this.setCooldown(key, f.id, tick + (p === P.ALLYINVIS ? 600 : C.selfinvis_cd)); }
          break;
        case P.BADR: m.ba_reduce = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.REFLECTDMG: m.reflect = sc(ea); this.buff(out, f.id, name, eb, m); break;
        case P.VAMP: m.vamp = sc(ea); this.buff(out, t.me.id, name, eb, m); break;
        case P.TITHE:
          if (enemy && this.cooldownOk(key, f.id, tick)) {
            out.push({ k: 'ClearShield', target: f.id });
            this.setCooldown(key, f.id, tick + 240);
            this.shield(out, t.me, div(t.me.max_hp * C.tithe_shield, 100), 180, tick);
          }
          break;
        case P.WELL: {
          const to = lerp(f.x, f.y, c[0], c[1], Math.min(100, sc(ea)));
          this.displace(out, f, to[0], to[1], tick, t.wall, enemy);
          break;
        }
        case P.REPEL: {
          const [dx, dy] = (f.x !== c[0] || f.y !== c[1]) ? [f.x - c[0], f.y - c[1]] : line ? [-(line.by - line.ay), line.bx - line.ax] : [1, 0];
          const to = along(f.x, f.y, dx, dy, sc(ea));
          this.displace(out, f, to[0], to[1], tick, t.wall, enemy);
          break;
        }
        case P.CHILL: {
          let [n, last] = this.chill.get(f.id) || [0, 0];
          if (tick > last + C.chill_fade) n = 0;
          n++;
          if (n >= Math.max(1, ec)) { this.control(out, f, 'Root', Math.max(8, sc(ea)), tick); n = 0; }   // frozen: the stacks clear
          else { m.move_speed = -(sc(eb) * n); this.buff(out, f.id, name, 40, m); }
          this.chill.set(f.id, [n, tick]);
          break;
        }
        case P.PRESSURE:
          // a zone stores charges instead (stepZones) and bursts when it closes
          if (src !== 'Zone') this.later.push({ kind: 'Burst', at: tick + Math.max(1, eb), x: f.x, y: f.y, pm: sc(ea) });
          break;
        case P.EXPOSE: this.exposed.set(f.id, [tick + eb, sc(ea)]); break;
        case P.COINFLIP:
          if (this.random(2) === 0) { if (enemy) { m.amplify = sc(ea); this.buff(out, f.id, name, 40, m); } }
          else if (!enemy) { m.reduce = sc(ea); this.buff(out, f.id, name, 40, m); }
          break;
        default: break;
      }
    }
    module(out, t, m, x, y) {
      const md = MODULES[m];
      const tick = t.tick;
      const r = this.completion >= HEX.radius_at ? C.module_radius_up : C.module_radius;
      const nearE = t.enemies().filter(e => dist2(e.x, e.y, x, y) <= SQ(r));
      const nearA = t.friends().filter(a => dist2(a.x, a.y, x, y) <= SQ(r));
      const name = 'vsm_' + md.id;
      switch (m) {
        case 0: { const a = minBy(nearA, a => [div(a.hp * 100, Math.max(1, a.max_hp)), a.id]); if (a) this.shield(out, a, div(a.max_hp * md.a, 1000), md.b, tick); break; }
        case 1: for (const a of nearA) this.buff(out, a.id, name, md.b, Mods({ move_speed: md.a })); break;
        case 2: for (const e of nearE) this.buff(out, e.id, name, md.b, Mods({ move_speed: -md.a })); break;
        case 3: for (const e of nearE) { const to = lerp(e.x, e.y, x, y, md.a); this.displace(out, e, to[0], to[1], tick, t.wall, true); } break;
        case 4: for (const a of nearA) this.heal(out, t, a.id, div(a.max_hp * md.a, 1000), tick); break;
        case 5: for (const e of nearE) this.buff(out, e.id, name, md.b, Mods({ amplify: md.a })); break;
        case 6: {
          const v = nearE.slice().sort((a, b) => cmp([dist2(a.x, a.y, x, y), a.id], [dist2(b.x, b.y, x, y), b.id]));
          for (const e of v.slice(0, 2)) this.emit(out, e, md.a + div(t.me.attack * md.b, 100), 'Magic', tick);
          break;
        }
        default: { const e = minBy(nearE, e => [dist2(e.x, e.y, x, y), e.id]); if (e) this.control(out, e, 'Root', md.a, tick); }
      }
      this.fx(out, `mod${m}_t${Math.min(3, this.tier4())}`, x, y, null, 18);
    }

    // -------------------------------------------------------------- gauntlets
    leftBroken(tick) { return tick < this.left_broken_until; }
    rightBroken(tick) { return tick < this.right_broken_until; }
    awakened(tick) { return tick < this.awaken_until; }
    sockets() { return this.pure.filter(s => s.open).length; }
    overSockets() { return this.pure.filter(s => s.open && s.over).length; }
    pureReady(tick) { return this.leftBroken(tick) ? 0 : this.pure.filter(s => s.open && tick >= s.ready_at).length; }
    hexSlots(tick) { return this.rightBroken(tick) ? 0 : M.hexSlots(this.completion); }
    hexReady(tick) { let n = 0; for (let i = 0; i < this.hexSlots(tick); i++) if (tick >= this.hex[i]) n++; return n; }
    orbsReady(tick) { return this.pureReady(tick) + this.hexReady(tick); }
    hexPower(tick) { return this.awakened(tick) ? 100 + div(C.corrupt_power * this.corruption, C.corrupt_per_orb) : Math.min(100, this.completion); }
    hexCd() { return this.completion >= HEX.fast_at ? C.hex_cd_fast : C.hex_cd; }
    take(tick, hex) {
      const takePure = () => {
        if (this.leftBroken(tick)) return null;
        let best = null, bk = null;
        for (let i = 0; i < 9; i++) {
          const s = this.pure[i];
          if (!(s.open && tick >= s.ready_at)) continue;
          const k = [s.over ? 1 : 0, 8 - i];
          if (bk === null || cmp(k, bk) >= 0) { best = i; bk = k; }
        }
        if (best === null) return null;
        const over = this.pure[best].over;
        this.pure[best].ready_at = tick + (over ? C.over_cd : C.orb_cd);
        if (!this.awakened(tick)) this.completion = Math.min(100, this.completion + C.research_orb);
        this.stats.orbs++;
        return { hex: null, over, socket: best };
      };
      const takeHex = () => {
        const n = this.hexSlots(tick);
        for (let i = 0; i < n; i++) {
          if (tick >= this.hex[i]) { this.hex[i] = tick + this.hexCd(); this.stats.hexorbs++; return { hex: i, over: false, socket: 0 }; }
        }
        return null;
      };
      return hex ? (takeHex() || takePure()) : (takePure() || takeHex());
    }
    activate(orb, dim, tick) {
      const fam = DIMS[dim].family;
      const fresh = this.last_family !== null && tick <= this.combo_at + C.combo_window;
      const twist = fresh && this.last_family !== fam ? twistIndex(this.last_family, fam) : null;
      this.combo = fresh ? Math.min(div(C.combo_max, C.combo_step), this.combo + 1) : 0;
      this.stats.best_combo = Math.max(this.stats.best_combo, this.combo);
      this.combo_at = tick;
      this.last_family = fam;
      if (orb.hex === null) this.pure[orb.socket].family = fam;
      return [100 + this.combo * C.combo_step, twist];
    }
    overload(out, t) {
      const tick = t.tick;
      if (this.leftBroken(tick)) return false;
      let i = -1;
      for (let k = C.orbs_base; k < C.orbs_max; k++) if (!this.pure[k].open) { i = k; break; }
      if (i < 0) return false;
      this.pure[i] = { open: true, over: true, ready_at: tick, family: 255 };
      this.stats.overloads++;
      this.addStress(out, t, C.over_stress);
      this.fx(out, `overload_t${Math.min(3, this.tier4())}`, t.me.x, t.me.y, t.me.id, 18);
      return true;
    }
    vent() { for (let i = C.orbs_base; i < 9; i++) this.pure[i] = { open: false, over: false, ready_at: 0, family: 255 }; }
    addStress(out, t, n) {
      this.stress = Math.max(0, this.stress + n);
      if (this.stress >= C.stress_cap && !this.leftBroken(t.tick)) {
        out.push({ k: 'Log', s: `tick ${t.tick} stress ${this.stress} cracked the left gauntlet` });
        this.shatter(out, t, true);
      }
    }
    shatter(out, t, forced) {
      const tick = t.tick;
      this.shatter_until = tick + 30;
      const held = [];
      for (let i = 0; i < 9; i++) if (this.pure[i].open && tick >= this.pure[i].ready_at) held.push(i);
      const over = held.filter(i => this.pure[i].over).length;
      const n = held.length;
      const load = (n - over) * C.shatter_load + over * C.shatter_load_over;
      const amount = div(Math.max(1, n) * (C.shatter_base + div(t.me.attack * C.shatter_ad, 100)) * (100 + over * C.shatter_over_pct), 100);
      const tier = this.tier4();
      let hits = 0;
      const rifts = this.rifts.filter(r => riftLive(r) && !riftHidden(r, tick)).map(cloneRift);
      const seen = new Set();
      for (const r of rifts) {
        const reach = r.radius * 2;
        const inside = t.fighters.filter(f => f.hp > 0 && riftReach(r, f.x, f.y) <= reach);
        for (const f of inside) {
          if (f.team === t.me.team) {
            if (f.id !== t.me.id) {
              const edge = along(r.x, r.y, f.x - r.x, f.y - r.y, reach + 6000);
              const [x, y] = land(f.x, f.y, edge[0], edge[1], t.wall);
              out.push({ k: 'SetPos', target: f.id, x, y });
            }
            continue;
          }
          if (seen.has(f.id)) continue;
          seen.add(f.id);
          hits++;
          this.control(out, f, null, C.shatter_banish, tick);
          this.emit(out, f, amount, 'Phys', tick);
          for (const dim of riftDims(r)) this.applyDim(out, t, dim, f, 100, [r.x, r.y, reach], 'Shatter');
        }
        this.fx(out, 'shatter_t' + tier, r.x, r.y, null, 30);
      }
      for (const i of held) {
        if (this.pure[i].over) this.pure[i] = { open: false, over: false, ready_at: 0, family: 255 };
        else this.pure[i].ready_at = tick + C.orb_cd;
      }
      for (const r of rifts) this.close(out, r.uid, 'break');
      this.stats.shatters++;
      if (hits >= 2) this.stats.useful_shatters++;
      if (!this.awakened(tick)) this.completion = Math.min(100, this.completion + C.research_shatter);
      if (forced) this.stats.move_uses[MV.shatter - 28]++;
      out.push({ k: 'Log', s: `tick ${tick} shatter orbs ${n} overloaded ${over} enemies ${hits} stress ${this.stress}+${load}${forced ? ' (forced)' : ''}` });
      this.stress += load;
      if (forced || this.stress > C.stress_cap) this.leftBreak(out, t);
    }
    leftBreak(out, t) {
      const tick = t.tick;
      this.stats.left_breaks++;
      this.left_broken_until = tick + C.left_break_ticks;
      this.pure = this.pure.map((_, i) => i < C.orbs_base ? { open: true, over: false, ready_at: tick + C.left_break_ticks, family: 255 } : { open: false, over: false, ready_at: 0, family: 255 });
      out.push({ k: 'LoseHp', target: t.me.id, amount: div(t.me.max_hp * C.left_break_hp, 100) });
      out.push({ k: 'Cc', target: t.me.id, kind: 'Stun', ticks: C.break_stagger });
      out.push({ k: 'Log', s: `tick ${tick} LEFT GAUNTLET BROKE (offline ${C.left_break_ticks} ticks)` });
      this.stress = div(C.stress_cap * 2, 5);
      this.packets = this.packets.filter(p => p.module !== null);
      this.fx(out, `gbreak_l_t${Math.min(3, this.tier4())}`, t.me.x, t.me.y, t.me.id, 36);
    }
    awaken(out, t) {
      const tick = t.tick;
      if (this.completion < 100 || this.awakened(tick) || this.rightBroken(tick)) return false;
      this.awaken_until = tick + C.awaken_ticks;
      this.corruption = 0;
      this.hex = [tick, tick, tick, tick, tick, tick];
      this.stats.awakens++;
      const fight = t.enemies().some(e => dist(e.x, e.y, t.me.x, t.me.y) <= 90000) && t.allies().some(a => t.danger(a.x, a.y, 60000) > 0);
      if (fight) this.stats.good_awakens++;
      this.fx(out, 'awaken_t' + this.tier4(), t.me.x, t.me.y, t.me.id, C.awaken_ticks);
      out.push({ k: 'Log', s: `tick ${tick} HEXCORE AWAKENED${fight ? ' in the fight' : ''}` });
      return true;
    }
    overclock(out, t) {
      const tick = t.tick;
      if (!this.awakened(tick) || this.pureReady(tick) === 0) return false;
      this.take(tick, false);
      this.corruption += C.corrupt_per_orb;
      this.fx(out, 'corrupt', t.me.x, t.me.y, t.me.id, 18);
      if (this.corruption >= 100) this.rightBreak(out, t);
      return true;
    }
    rightBreak(out, t) {
      const tick = t.tick;
      this.stats.right_breaks++;
      this.right_broken_until = tick + C.right_break_ticks;
      this.completion = C.hex_start;
      this.awaken_until = tick;
      this.corruption = 0;
      out.push({ k: 'LoseHp', target: t.me.id, amount: div(t.me.max_hp * C.right_break_hp, 100) });
      const burst = C.hexburst_base + div(t.me.attack * C.hexburst_ad, 100);
      for (const e of t.enemies().filter(e => dist(e.x, e.y, t.me.x, t.me.y) <= C.hexburst_radius)) this.emit(out, e, burst, 'Magic', tick);
      out.push({ k: 'Log', s: `tick ${tick} RIGHT GAUNTLET BROKE: Hexcore corrupted` });
      this.fx(out, 'gbreak_r', t.me.x, t.me.y, t.me.id, 36);
    }
    stepGauntlets(out, t) {
      const tick = t.tick;
      const lost = this.last_hp >= 0 ? Math.max(0, this.last_hp - t.me.hp) : 0;
      this.last_hp = t.me.hp;
      const lostPct = div(lost * 100, Math.max(1, t.me.max_hp));
      if (this.awaken_until > 0 && tick === this.awaken_until && !this.rightBroken(tick)) {
        this.completion = C.awaken_after;
        if (this.corruption > 0) this.habit('overclock stopped in time');
        this.corruption = 0;
      }
      if (this.awakened(tick) && lostPct > 0) {
        this.corruption += div(lostPct, C.hit_corrupt_div);
        if (this.corruption >= 100) this.rightBreak(out, t);
      }
      if (this.left_broken_until > 0 && tick === this.left_broken_until) this.fx(out, 'gfix_l', t.me.x, t.me.y, t.me.id, 18);
      const heldOver = this.pure.filter(s => s.open && s.over && tick >= s.ready_at).length;
      if (this.overSockets() > 0 && lostPct > 0) this.addStress(out, t, lostPct * C.hit_stress_pct);
      if (heldOver > 0 && tick % C.over_heat_ticks === 0) this.addStress(out, t, heldOver);
      if (this.overSockets() === 0 && this.openCount() <= 1 && tick % C.stress_decay_ticks === 0) this.stress = Math.max(0, this.stress - 1);
      const gone = [...this.hurt].sort((a, b) => a[0] - b[0]).filter(([id, [at, hp, max]]) => tick <= at + C.takedown_window && hp * 100 <= max * 15 && !t.get(id)).map(([id]) => id);
      for (const id of gone) {
        this.hurt.delete(id);
        this.stats.takedowns++;
        if (!this.awakened(tick)) this.completion = Math.min(100, this.completion + C.research_takedown);
      }
      for (const [id, [at]] of [...this.hurt]) if (!(tick <= at + C.takedown_window)) this.hurt.delete(id);
      for (const [id, [until]] of [...this.marks]) if (!(until > tick)) this.marks.delete(id);
      if (this.last_family !== null && tick > this.combo_at + C.combo_window) this.combo = 0;
    }

    // -------------------------------------------------------------- rifts
    open(out, t, spec, tech, subject, dim, module) {
      const rt = RIFTS[spec.kind];
      const uid = this.next_uid++;
      const d = dist(t.me.x, t.me.y, spec.x, spec.y);
      const delay = ((spec.kind === R.anchor && spec.host !== null) || tech === MV.swipe) ? openDelay(0, 100) : openDelay(d, rt.delay_pct);
      let chain = null;
      if (spec.chain) {
        if (spec.order !== null && spec.order !== undefined) chain = spec.order; else { this.chain_seq++; chain = this.chain_seq; }
      }
      const opens = t.tick + delay;
      this.rifts.push({ uid, kind: spec.kind, tech, dim, dim2: null, module: module === undefined ? null : module, infused: null, x: spec.x, y: spec.y, ex: spec.ex, ey: spec.ey, radius: rt.radius, opens, expires: opens + rt.life, host: spec.host, peer: null, subject: subject === undefined ? null : subject, chain, entrance: spec.entrance, opened: false, fired: false, dwell: new Map(), touched: new Set(), sides: new Map(), inside: new Set(), done: false, charge: 0, drawn: [0, 0, 0, 0] });
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
      this.releasePressure(out, t, r);
      const reach = div(r.radius * 3, 2);
      const amount = C.implode_base + div(t.me.attack * C.implode_ad, 100);
      let hits = 0;
      const targets = t.enemies().filter(e => riftReach(r, e.x, e.y) <= reach);
      for (const e of targets) {
        hits++;
        this.displace(out, e, div(e.x + r.x, 2), div(e.y + r.y, 2), t.tick, t.wall, true);
        this.emit(out, e, amount, 'Phys', t.tick);
        this.detonate(out, e, t.tick);
        for (const dim of riftDims(r)) this.applyDim(out, t, dim, e, 200, [r.x, r.y, r.radius], 'Imbue');
      }
      this.stats.implodes++;
      this.stats.implode_hits += hits;
      this.stress = Math.max(0, this.stress - div(C.stress_tear, 2));
      this.fx(out, 'tech_implode_t' + this.tier4(), r.x, r.y, null, 30);
      this.close(out, uid, 'close');
    }

    // -------------------------------------------------------------- one tick
    step(input) {
      const out = [];
      if (this.last_tick === input.tick) return out;
      this.last_tick = input.tick;
      this.fx_left = C.fx_per_update;
      if (input.tick % C.fx_refill === 0 && this.fx_tokens < C.fx_tokens) this.fx_tokens++;
      const me = input.fighters.find(f => f.id === input.me);
      if (!me) return out;
      const t = new Tick(input.tick, me, input.fighters, input.wall, Math.max(1, input.attack_interval), input.stunned);
      const tick = t.tick;
      this.observe(t);
      for (const [k, until] of [...this.cooldowns]) if (!(until > tick)) this.cooldowns.delete(k);
      this.stepGauntlets(out, t);
      this.stepRifts(out, t);
      this.stepTechniques(out, t);
      this.stepZones(out, t);
      this.stepBreaking(out, t);
      this.stepLater(out, t);
      this.stepDots(out, t);
      this.stepPunch(out, t);
      this.stepPackets(out, t);
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
        if (h.length) {
          const [px, py] = h[h.length - 1];
          // a teleport, not a run: no velocity to lead rifts off the map with (the positions stay, so Rewind can
          // still send them back to before the blink)
          if (dist(f.x, f.y, px, py) > 30000) this.velocity.set(f.id, [0, 0]);
          else this.velocity.set(f.id, [div(f.x - px, 6), div(f.y - py, 6)]);
        }
        h.push([f.x, f.y, f.hp]);
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
          if (r.module !== null) this.module(out, t, r.module, r.x, r.y);
        }
        if (r.opened && r.kind === R.wander) {
          const [nx, ny] = dist(r.x, r.y, r.ex, r.ey) <= C.wander_speed ? [r.ex, r.ey] : along(r.x, r.y, r.ex - r.x, r.ey - r.y, C.wander_speed);
          r.x = nx; r.y = ny;
        }
        if (riftLive(r) && r.infused !== null && (tick + r.uid) % 60 === 0) this.module(out, t, r.infused, r.x, r.y);
        if (tick >= r.expires) {
          const c = cloneRift(r);
          if (c.kind !== R.maw) this.releasePressure(out, t, c);
          if (c.kind === R.maw) this.implode(out, t, c.uid);
          else if (c.tech === T.hook) this.reel(out, t, c);
          else this.close(out, c.uid, 'close');
        }
      }
    }
    reel(out, t, r) {
      const target = r.host !== null ? t.get(r.host) : null;
      if (target) {
        const p = r.peer !== null ? this.rift(r.peer) : null;
        const exit = p ? [p.x, p.y] : [t.me.x, t.me.y];
        const key = 2000 + T.hook;
        if (this.cooldownOk(key, target.id, t.tick) && this.displace(out, target, exit[0], exit[1], t.tick, t.wall, true)) {
          this.control(out, target, null, C.hook_banish, t.tick);
          this.setCooldown(key, target.id, t.tick + TECHS[T.hook].cooldown);
          this.scoreKidnap(t, exit);
          this.fx(out, 'tech_hook_t' + this.tier4(), exit[0], exit[1], null, 24);
        }
      }
      this.close(out, r.uid, 'close');
    }
    // Kidnap's field closes: every enemy still inside is stunned; if the target is one of them, the target drops onto
    // the exit and Vesper lands beside it. A target who walked out leaves her where she blinked.
    resolveKidnap(out, t, target, at, exit, dim, radius) {
      const tick = t.tick;
      const tier = this.tier4();
      const inside = t.enemies().filter(e => dist(e.x, e.y, at[0], at[1]) <= C.kidnap_radius);
      let caught = null;
      if (!t.stunned) for (const e of inside) if (this.control(out, e, 'Stun', C.kidnap_stun, tick) && e.id === target) caught = e;
      const key = 2000 + T.kidnap;
      let dropped = false;
      if (caught && this.cooldownOk(key, caught.id, tick) && this.displace(out, caught, exit[0], exit[1], tick, t.wall, true)) {
        this.setCooldown(key, caught.id, tick + TECHS[T.kidnap].cooldown);
        const beside = along(exit[0], exit[1], exit[0] - at[0], exit[1] - at[1], 15000);   // behind the exit
        const [bx, by] = land(t.me.x, t.me.y, beside[0], beside[1], t.wall);
        out.push({ k: 'SetPos', target: t.me.id, x: bx, y: by });
        this.applyDim(out, t, dim, { ...caught, x: exit[0], y: exit[1] }, 100, [exit[0], exit[1], radius], 'Imbue');
        dropped = true;
      }
      this.stats.kidnaps++;
      if (dropped) {
        // useful when it drops the target among her team: her allies outnumber its own at the landing spot
        const allies = t.allies().filter(a => a.id !== t.me.id && dist2(a.x, a.y, exit[0], exit[1]) <= SQ(45000)).length;
        const foes = t.enemies().filter(e => e.id !== target && dist2(e.x, e.y, exit[0], exit[1]) <= SQ(45000)).length;
        if (allies >= 1 && allies > foes) this.stats.useful_kidnaps++;
        this.fx(out, 'tech_kidnap_t' + tier, at[0], at[1], null, 24);
        this.fx(out, 'tech_kidnap_t' + tier, exit[0], exit[1], null, 24);
      } else {
        this.stats.kidnap_dodges++;
        this.fx(out, 'kidnapmiss_t' + tier, at[0], at[1], null, 18);
      }
    }
    // Kidnap fields armed right now, [x, y, radius, closes at]
    kidnapZones() { return this.later.filter(l => l.kind === 'Kidnap').map(l => [l.x, l.y, C.kidnap_radius, l.at]); }
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
            this.punching = false;
            this.later.push({ kind: 'Unphase', at: tick + 45, x: exit[0], y: exit[1] });
            this.fx(out, 'tech_phase_t' + tier, r.x, r.y, null, 30);
          }
        } else if (tech === T.kidnap && r.entrance && !r.fired) {
          // both ends open: she steps through her tear and lands in front of the target, arming the field
          const exit = this.exitOf(r);
          if (!exit) continue;
          this.fire(uid);
          const v = r.subject !== null ? t.get(r.subject) : null;
          if (!v) continue;
          if (t.stunned || this.phased || dist(t.me.x, t.me.y, v.x, v.y) > C.kidnap_reach) continue;
          const front = along(v.x, v.y, t.me.x - v.x, t.me.y - v.y, C.kidnap_front);
          const [x, y] = land(t.me.x, t.me.y, front[0], front[1], t.wall);
          out.push({ k: 'SetPos', target: t.me.id, x, y });
          this.fx(out, 'tech_kidnap_t' + tier, t.me.x, t.me.y, null, 24);
          this.fx(out, 'kidnapwarn_t' + tier, x, y, null, C.kidnap_arm);
          this.later.push({ kind: 'Kidnap', at: tick + C.kidnap_arm, target: v.id, x, y, exit, dim: r.dim, radius: r.radius });
        } else if (tech === T.sinktrap) {
          const inside = t.enemies().filter(e => riftCovers(r, e.x, e.y));
          const key = 2000 + T.sinktrap;
          for (const e of inside) {
            const rr = this.rift(uid);
            const k = e.id + 1000000;
            const n = (rr.dwell.get(k) || 0) + 1;
            rr.dwell.set(k, n);
            if (tick % C.pulse === 0) this.buff(out, e.id, 'vst_sink', 40, Mods({ move_speed: -30 }));
            if (n >= 60 && this.cooldownOk(key, e.id, tick)) {
              this.setCooldown(key, e.id, tick + TECHS[T.sinktrap].cooldown);
              this.stats.trap_catches++;
              const exit = this.exitOf(r);
              if (exit && this.displace(out, e, exit[0], exit[1], tick, t.wall, true)) this.scoreKidnap(t, exit);
              this.control(out, e, null, C.sink_banish, tick);
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
              for (const dim of riftDims(r)) this.applyDim(out, t, dim, v, 200, [r.x, r.y, r.radius], 'Imbue');
              const exit = this.exitOf(r);
              if (exit) {
                if (this.displace(out, v, exit[0], exit[1], tick, t.wall, true)) this.scoreKidnap(t, exit);
                this.control(out, v, null, C.seed_banish, tick);
              } else this.control(out, v, null, C.seed_banish_alone, tick);
              this.fx(out, 'tech_seedtrap_t' + tier, r.x, r.y, null, 30);
              this.close(out, uid, 'close');
            }
          }
        } else if (tech === T.rain && justOpened) {
          const amount = C.rain_base + div(t.me.attack * C.rain_ad, 100);
          for (let k = 0; k < 6; k++) {
            const [ux, uy] = DIRS[this.random(16)];
            const reach = this.random(r.radius);
            this.later.push({ kind: 'Drop', at: tick + 4 + k * 10, x: r.x + div(ux * reach, 10000), y: r.y + div(uy * reach, 10000), amount });
          }
        } else if (tech === T.volley && justOpened) {
          const amount = C.volley_base + div(t.me.attack * C.volley_ad, 100);
          for (let k = 0; k < 5; k++) {
            const px = r.x + div((r.ex - r.x) * k, 4), py = r.y + div((r.ey - r.y) * k, 4);
            const e = minBy(t.enemies().filter(e => dist(e.x, e.y, px, py) <= C.volley_radius), e => [dist2(e.x, e.y, px, py), e.id]);
            if (e) {
              this.emit(out, e, amount, 'Phys', tick);
              this.fx(out, `packet_l${this.look()}_a${heading8(e.x - px, e.y - py)}`, px, py, null, 8);
            }
          }
        }
        if (r.kind === R.slash || r.kind === R.mirror || (r.tech === T.curtain && r.peer !== null)) this.crossing(out, t, uid);
        if (r.kind === R.maw && justOpened && r.tech !== T.pocket) {
          const inside = t.enemies().filter(e => riftCovers(r, e.x, e.y));
          for (const e of inside) this.control(out, e, null, C.maw_banish, tick);
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
        const dims = riftDims(r);
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
            } else if (pulse && ![P.VENT, P.UNSTABLE, P.SIPHON, P.PRESSURE].includes(p)) this.applyDim(out, t, dim, f, 100, cen, 'Zone');
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
            if (p === P.VENT) this.stress = Math.max(0, this.stress - DIMS[dim].a);
            else if (p === P.PRESSURE) {
              if (inside.some(f => f.team !== t.me.team)) { const rr = this.rift(uid); if (rr) rr.charge = Math.min(Math.max(1, DIMS[dim].c), rr.charge + 1); }
            } else if ((p === P.SWAPSPACE || p === P.SCRAMBLE) && !r.touched.has('swap:' + dim)) {
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
      if (this.awakened(tick)) return;
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
        if (!this.rift(uid)) continue;
        this.stats.breaks++;
        out.push({ k: 'Log', s: `tick ${tick} rift ${uid} broken by an enemy (+${C.stress_break} stress)` });
        const br = this.rift(uid);
        if (br) this.releasePressure(out, t, cloneRift(br));   // a broken Pressure rift still bursts
        this.close(out, uid, 'break');
        this.addStress(out, t, C.stress_break);
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
        else if (l.kind === 'Kidnap') this.resolveKidnap(out, t, l.target, [l.x, l.y], l.exit, l.dim, l.radius);
        else if (l.kind === 'Drop') {
          for (const e of t.enemies().filter(e => dist(e.x, e.y, l.x, l.y) <= C.rain_radius)) this.emit(out, e, l.amount, 'Phys', tick);
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
        } else if (l.kind === 'Burst') {
          for (const e of t.enemies().filter(e => dist(e.x, e.y, l.x, l.y) <= C.pressure_radius)) this.emit(out, e, div(e.max_hp * l.pm, 1000), 'Magic', tick);
          this.fx(out, 'tech_implode_t' + tier, l.x, l.y, null, 18);
        } else if (l.kind === 'Land') {
          for (const e of t.enemies().filter(e => dist(e.x, e.y, l.x, l.y) <= C.skydrop_radius)) this.control(out, e, 'Stun', C.skydrop_stun, tick);
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
          if (d.drain && n > 0) this.heal(out, t, t.me.id, n, tick);
        }
      }
      this.dots = dots.filter(d => d.until > tick && t.get(d.target));
    }


    // -------------------------------------------------------------- rift punches
    chainNodes(tick) {
      return this.rifts.filter(r => riftLive(r) && r.chain !== null && !riftHidden(r, tick)).map(cloneRift).sort((a, b) => a.chain - b.chain);
    }
    punchReserve() { return this.table('reserve') + (this.gate(G.geometry) && this.plan.length ? 1 : 0); }
    stepPunch(out, t) {
      const tick = t.tick;
      const nodes = this.chainNodes(tick);
      const reserve = this.hand_placed ? 0 : this.punchReserve();
      const castable = (this.hand_placed && this.armed !== null) ? this.castOk(this.armed, tick) : this.anyCastable(tick);
      // every dimension resting: she falls back to her plain jab
      const ready = !this.leftBroken(tick) && this.orbsReady(tick) > reserve && castable;
      let through = nodes.length > 0;
      const pick = (from, reach) => {
        const ok = e => dist(e.x, e.y, from[0], from[1]) <= reach;
        let keep = this.chain_target !== null ? t.get(this.chain_target) : null;
        if (keep && !(keep.team !== t.me.team && ok(keep))) keep = null;
        return keep || minBy(t.enemies().filter(ok), e => [dist2(e.x, e.y, from[0], from[1]), e.id]);
      };
      const exit = nodes.length ? nodes[nodes.length - 1] : null;
      let target = exit ? pick([exit.x, exit.y], C.attack_range) : pick([t.me.x, t.me.y], C.punch_reach);
      if (through && target && this.gate(G.team) && t.danger(t.me.x, t.me.y, C.dive_radius) > 0) {
        if (this.aiming) this.habit('dropped aim for a diver');
        through = false;
        target = pick([t.me.x, t.me.y], C.punch_reach);
      }
      if (!target || !ready || this.phased) { this.punching = false; this.aiming = false; return; }
      this.chain_target = target.id;
      this.punching = true;
      this.aiming = through;
      if (tick % 6 === 0) {
        out.push({ k: 'Cc', target: t.me.id, kind: 'Disarm', ticks: 8 });
        if (through) out.push({ k: 'Buff', target: t.me.id, name: 'vs_aim', ticks: 8, mods: Mods({ move_speed: -100 }) });
      }
      if (tick < this.next_attack || this.packets.length >= C.packet_limit) return;
      this.next_attack = tick + t.attack_interval;
      const fuse = this.awakened(tick) && this.pureReady(tick) > 0 && this.hexReady(tick) > 0;
      const hexPref = fuse ? false : this.prefersHex(tick);
      const orb = this.take(tick, hexPref);
      if (!orb) return;
      const dim = (this.hand_placed && this.armed !== null) ? this.armed : this.pickDim(t, 'Punch', target, [target.x, target.y]);
      let [combo, twist] = this.activate(orb, dim, tick);
      let power = orb.hex !== null ? this.hexPower(tick) : 100;
      let module = orb.hex !== null ? this.modules[orb.hex] : null;
      let fused = null;
      if (fuse) {
        const h = this.take(tick, true);
        if (h && h.hex !== null) {
          const d2 = this.pickDim(t, 'Punch', target, [target.x, target.y]);
          const tw = this.activate(h, d2, tick)[1];
          twist = twist !== null ? twist : tw !== null ? tw : twistIndex(DIMS[dim].family, DIMS[d2].family);
          fused = d2;
          power = C.fuse_pct;
          module = this.modules[h.hex];
          this.stats.fused++;
        }
      }
      const path = through ? nodes.map(n => [n.uid, n.x, n.y]) : [];
      path.push([0, target.x, target.y]);
      this.commitCast(fused !== null ? [dim, fused] : [dim], tick);
      const bonus = this.opunch + combo - 100;
      this.opunch = 0;
      this.stats.punches++;
      this.fx(out, 'punch_t' + this.tier4(), t.me.x, t.me.y, t.me.id, 12);
      const packet = (x, y, path, tgt, eff, bonus, ignoreWall, fusedDim, mod, tw, source) => ({ x, y, hop: [x, y], path, target: tgt, eff, bonus, dim, fused: fusedDim, power, module: mod, picked: [], true_pct: 0, ignore_wall: ignoreWall, kaleido: false, twist: tw, source, base: t.me.attack, drawn: 0 });
      this.packets.push(packet(t.me.x, t.me.y, path, target.id, 10000, bonus, false, fused, module, twist, through ? 'chain' : 'punch'));
      if (through && tick < this.crossfire_until && exit) {
        const others = this.rifts.filter(r => riftLive(r) && r.uid !== exit.uid && !riftHidden(r, tick));
        const used = [target.id];
        for (const r of others.slice(0, 4)) {
          if (this.packets.length >= C.packet_limit || used.length >= 3) break;
          const e = minBy(t.enemies().filter(e => !used.includes(e.id) && dist(e.x, e.y, r.x, r.y) <= C.attack_range), e => [dist2(e.x, e.y, r.x, r.y), e.id]);
          if (e) { used.push(e.id); this.packets.push(packet(r.x, r.y, [[0, e.x, e.y]], e.id, C.crossfire_pct * 100, 0, true, null, null, null, 'crossfire')); }
        }
      }
    }
    prefersHex(tick) {
      if (this.hand_placed) return false;
      if (this.rank < G.safety) return this.random(2) === 0;
      return this.hexReady(tick) > 0 && (this.pureReady(tick) <= 2 || this.completion >= HEX.fast_at);
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
          if (uid === 0) {
            if (p.source === 'punch') p.eff = div(p.eff * keepBp(len), 10000);
            hit = true;
            break;
          }
          const r0 = this.rift(uid);
          if (!r0 || !riftLive(r0)) { alive = false; this.fx(out, 'close_l' + this.look(), wx, wy, null, 12); break; }
          const r = cloneRift(r0);
          let kb = keepBp(len);
          if (bank) kb = Math.min(9500, div(kb * 12, 10));
          p.eff = div(p.eff * kb, 10000);
          bank = r.kind === R.mirror;
          if (p.path.length && p.path[0][0] === 0) this.fx(out, 'exit_l' + this.look(), wx, wy, null, 12);   // the fist bursts out of the last rift
          for (const dim of riftDims(r)) {
            const dd = DIMS[dim];
            switch (dd.prim) {
              case P.TRUE: p.true_pct += dd.a; break;
              case P.LENS: p.bonus += dd.a; break;
              case P.MOMENTUM: p.bonus += this.momentum(t, dd.a, dd.b); break;
              case P.ECHO: case P.WILDECHO: case P.PRISM: p.picked.push(dim); break;
              case P.KALEIDO: p.kaleido = true; break;
              case P.UNSTABLE: p.bonus += dd.a - 100; this.addStress(out, t, dd.b); break;
              default: break;
            }
          }
        }
        if (hit) this.hit(out, t, p);
        else if (alive) {
          if (tick >= p.drawn) {
            const dir = p.path.length ? heading8(p.path[0][1] - p.x, p.path[0][2] - p.y) : 0;
            if (this.fxLow(out, `packet_l${this.look()}_a${dir}`, p.x, p.y, null, 4)) p.drawn = tick + 4;
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
      let bonus = p.bonus, truePct = p.true_pct, power = p.power;
      const dims = p.fused !== null ? [p.dim, p.fused] : [p.dim];
      for (const dim of dims) {
        for (const e of [PUNCH[dim].main, PUNCH[dim].rider]) {
          const [pr, ea, eb] = e;
          if (pr === P.LENS) bonus += ea;
          else if (pr === P.MOMENTUM) bonus += this.momentum(t, ea, eb);
          else if (pr === P.TRUE) truePct += ea;
          else if (pr === P.EXECUTE) { if (f.hp * 100 < f.max_hp * eb) bonus += ea; }
          else if (pr === P.UNSTABLE) { power = div(power * ea, 100); this.addStress(out, t, eb); }
        }
      }
      const amount = div(div(div(p.base * p.eff, 10000) * Math.max(10, 100 + bonus), 100) * power, 100);
      const dealt = this.emit(out, f, amount, 'Phys', tick);
      if (truePct > 0) this.emit(out, f, div(amount * truePct, 100), 'True', tick);
      this.stats.eff_sum += p.eff;
      this.stats.landed++;
      const s = div(Math.min(100, Math.max(30, div(p.eff, 100))) * Math.min(C.fuse_pct, power), 100);
      this.detonate(out, f, tick);
      for (const dim of dims) this.deliverPunch(out, t, dim, f, s, amount, dealt, 0);
      const marks = [...this.marks].filter(([id]) => id !== f.id).sort((a, b) => a[0] - b[0]).map(([id, [, pct]]) => [id, pct]);
      if (marks.length) {
        this.marks.clear();
        for (const [id, pct] of marks) {
          const e = t.get(id);
          if (e) { this.emit(out, e, div(amount * pct, 100), 'Phys', tick); this.fx(out, 'hole_l' + this.look(), e.x, e.y, null, 12); }
        }
      }
      for (const dim of p.picked) {
        const pr = DIMS[dim].prim;
        if (pr === P.ECHO) this.later.push({ kind: 'Repeat', at: tick + DIMS[dim].b, target: f.id, amount: div(amount * DIMS[dim].a, 100) });
        else if (pr === P.WILDECHO) {
          const pct = C.wildecho_min + this.random(C.wildecho_span + 1);
          this.later.push({ kind: 'Repeat', at: tick + 30, target: f.id, amount: div(amount * pct, 100) });
        } else {
          const others = t.enemies().filter(e => e.id !== f.id && dist(e.x, e.y, f.x, f.y) <= C.prism_radius).slice(0, DIMS[dim].a - 1);
          for (const e of others) this.emit(out, e, div(amount * DIMS[dim].b, 100), 'Phys', tick);
        }
      }
      if (p.module !== null) this.module(out, t, p.module, f.x, f.y);
      const twist = p.kaleido ? this.random(TWISTS.length) : p.twist;
      if (twist !== null) this.twist(out, t, twist, f, amount);
      this.fx(out, 'hole_l' + this.look(), f.x, f.y, null, 18);
    }
    deliverPunch(out, t, dim, f, s, amount, dealt, depth) {
      const tick = t.tick;
      const me = [t.me.x, t.me.y, 0];
      this.slammed = false;
      const d = PUNCH[dim];
      for (const e of [d.main, d.rider]) {
        const [pr, ea, eb] = e;
        if (pr === 255 || pr === P.LENS || pr === P.TRUE || pr === P.EXECUTE || pr === P.UNSTABLE || pr === P.MOMENTUM) continue;
        if (pr === P.ECHO) this.later.push({ kind: 'Repeat', at: tick + eb, target: f.id, amount: div(amount * ea, 100) });
        else if (pr === P.WILDECHO) {
          const pct = C.wildecho_min + this.random(C.wildecho_span + 1);
          this.later.push({ kind: 'Repeat', at: tick + 30, target: f.id, amount: div(amount * pct, 100) });
        } else if (pr === P.PRISM) {
          const others = t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= C.prism_radius).slice(0, Math.max(1, ea) - 1);
          for (const o of others) this.emit(out, o, div(amount * eb, 100), 'Phys', tick);
        } else if (pr === P.SPLASH) {
          for (const o of t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= eb)) this.emit(out, o, div(amount * ea, 100), 'Phys', tick);
        } else if (pr === P.RICOCHET) {
          const o = minBy(t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= C.ricochet_radius), o => [dist2(o.x, o.y, f.x, f.y), o.id]);
          if (o) { this.emit(out, o, div(amount * ea, 100), 'Phys', tick); this.fx(out, 'hole_l' + this.look(), o.x, o.y, null, 12); }
        } else if (pr === P.CHAIN) {
          const o = minBy(t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= ea), o => [dist2(o.x, o.y, f.x, f.y), o.id]);
          if (o) this.applyEff(out, t, dim, d.main, o, div(s * eb, 100), me, 'Punch', null);
        } else if (pr === P.SIPHON) {
          for (const a of t.friends().filter(a => dist(a.x, a.y, f.x, f.y) <= C.module_radius)) this.heal(out, t, a.id, div(dealt * ea, 100), tick);
        } else if (pr === P.KALEIDO) {
          this.twist(out, t, this.random(TWISTS.length), f, amount);
        } else if (pr === P.VENT) {
          this.stress = Math.max(0, this.stress - ea);
        } else if (pr === P.DICE) {
          const other = this.otherFamilyDim(DIMS[dim].family);
          if (depth === 0 && PUNCH[other].main[0] !== P.DICE) this.deliverPunch(out, t, other, f, s, amount, dealt, 1);
        } else if (pr === P.SCRAMBLE) {
          const o = minBy(t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= C.near_radius), o => [dist2(o.x, o.y, f.x, f.y), o.id]);
          if (o && this.displace(out, f, o.x, o.y, tick, t.wall, true)) this.displace(out, o, f.x, f.y, tick, t.wall, true);
        } else {
          const sd = PRIM_SIDE[pr];
          if (sd === 'E') this.applyEff(out, t, dim, e, f, s, me, 'Punch', null);
          else if (sd === 'S') this.applyEff(out, t, dim, e, t.me, s, [f.x, f.y, 0], 'Punch', null);
          else {
            for (const a of t.friends().filter(a => dist(a.x, a.y, f.x, f.y) <= C.module_radius)) this.applyEff(out, t, dim, e, a, s, me, 'Punch', null);
            if (sd === 'X') this.applyEff(out, t, dim, e, f, s, me, 'Punch', null);
          }
        }
      }
    }
    swipe(out, t, point, dimIn, subject) {
      const tick = t.tick;
      if (this.orbsReady(tick) < MOVES[MV.swipe - 28].orbs || this.openCount() >= C.max_rifts) return false;
      if (!((dimIn !== null && dimIn !== undefined) ? this.castOk(dimIn, tick) : this.anyCastable(tick))) return false;
      const hexPref = this.prefersHex(tick);
      const orb = this.take(tick, hexPref);
      if (!orb) return false;
      this.take(tick, orb.hex !== null);
      const target = subject !== null && subject !== undefined ? t.get(subject) : null;
      const dim = dimIn !== null && dimIn !== undefined ? dimIn : this.pickDim(t, 'Swipe', target, point);
      const [combo, twist] = this.activate(orb, dim, tick);
      this.commitCast([dim], tick);
      let power = orb.hex !== null ? this.hexPower(tick) : 100;
      const module = orb.hex !== null ? this.modules[orb.hex] : null;
      const [dx, dy] = (point[0] === t.me.x && point[1] === t.me.y) ? [1, 0] : [point[0] - t.me.x, point[1] - t.me.y];
      const b = clampMap(...along(t.me.x, t.me.y, dx, dy, C.swipe_len));
      const l = { ax: t.me.x, ay: t.me.y, bx: b[0], by: b[1] };
      const uid = this.open(out, t, Spec.line(R.slash, l.ax, l.ay, l.bx, l.by), MV.swipe, subject === undefined ? null : subject, dim, module);
      const d = SWIPE[dim];
      if (d.main[0] === P.WALL) { const r = this.rift(uid); if (r) r.expires += RIFTS[R.slash].life; }
      this.addStress(out, t, C.stress_tear);
      let bonus = combo - 100, truePct = 0;
      for (const [pr, ea, eb] of [d.main, d.rider]) {
        if (pr === P.LENS) bonus += ea;
        else if (pr === P.MOMENTUM) bonus += this.momentum(t, ea, eb);
        else if (pr === P.TRUE) truePct += ea;
        else if (pr === P.UNSTABLE) { power = div(power * ea, 100); this.addStress(out, t, eb); }
      }
      const amount = div(div((C.swipe_base + div(t.me.attack * C.swipe_ad, 100)) * (100 + bonus), 100) * power, 100);
      const s = Math.min(C.fuse_pct, power);
      const onLine = t.fighters.filter(f => f.hp > 0 && f.id !== t.me.id && segDist(l.ax, l.ay, l.bx, l.by, f.x, f.y) <= C.swipe_width);
      const enemies = onLine.filter(f => f.team !== t.me.team);
      let total = 0;
      const skip = new Set([255, P.LENS, P.TRUE, P.UNSTABLE, P.WALL, P.SWAPSPACE, P.SCRAMBLE, P.VENT, P.KALEIDO, P.DICE, P.SIPHON, P.MOMENTUM]);
      for (const f of onLine) {
        const enemy = f.team !== t.me.team;
        if (enemy) {
          total += this.emit(out, f, amount, 'Phys', tick);
          if (truePct > 0) this.emit(out, f, div(amount * truePct, 100), 'True', tick);
          this.detonate(out, f, tick);
        }
        const c = nearestOn(l.ax, l.ay, l.bx, l.by, f.x, f.y);
        this.slammed = false;
        for (const e of [d.main, d.rider]) {
          const [pr, ea, eb] = e;
          if (skip.has(pr)) continue;
          if (pr === P.SPLASH && enemy) {
            for (const o of t.enemies().filter(o => o.id !== f.id && dist(o.x, o.y, f.x, f.y) <= eb && segDist(l.ax, l.ay, l.bx, l.by, o.x, o.y) > C.swipe_width)) this.emit(out, o, div(amount * ea, 100), 'Phys', tick);
          } else if (pr === P.ECHO && enemy) this.later.push({ kind: 'Repeat', at: tick + eb, target: f.id, amount: div(amount * ea, 100) });
          else this.applyEff(out, t, dim, e, f, s, [c[0], c[1], C.swipe_width], 'Swipe', l);
        }
      }
      for (const e of [d.main, d.rider]) {
        const pr = e[0];
        if (pr === P.SWAPSPACE) {
          const na = minBy(enemies, f => [dist2(f.x, f.y, l.ax, l.ay), f.id]);
          const nb = minBy(enemies, f => [dist2(f.x, f.y, l.bx, l.by), f.id]);
          if (na && nb && na.id !== nb.id && this.displace(out, na, nb.x, nb.y, tick, t.wall, true)) this.displace(out, nb, na.x, na.y, tick, t.wall, true);
        } else if (pr === P.SCRAMBLE) {
          for (const f of enemies) {
            const c = nearestOn(l.ax, l.ay, l.bx, l.by, f.x, f.y);
            this.displace(out, f, l.ax + l.bx - c[0] + (f.x - c[0]), l.ay + l.by - c[1] + (f.y - c[1]), tick, t.wall, true);
          }
        } else if (pr === P.VENT) this.stress = Math.max(0, this.stress - e[1]);
        else if (pr === P.SIPHON) {
          const allies = onLine.filter(f => f.team === t.me.team).map(f => f.id).concat([t.me.id]);
          for (const a of allies) this.heal(out, t, a, div(total * e[1], 100), tick);
        } else if (pr === P.KALEIDO) {
          if (enemies.length) this.twist(out, t, this.random(TWISTS.length), enemies[0], amount);
        } else if (pr === P.DICE) {
          const other = this.otherFamilyDim(DIMS[dim].family);
          if (SWIPE[other].main[0] !== P.DICE) {
            for (const f of onLine) {
              const c = nearestOn(l.ax, l.ay, l.bx, l.by, f.x, f.y);
              this.applyEff(out, t, other, SWIPE[other].main, f, s, [c[0], c[1], C.swipe_width], 'Swipe', l);
            }
          }
        }
      }
      if (twist !== null && enemies.length) this.twist(out, t, twist, enemies[0], amount);
      if (module !== null) this.module(out, t, module, div(l.ax + l.bx, 2), div(l.ay + l.by, 2));
      this.stats.swipes++;
      this.stats.swipe_hits += enemies.length;
      this.fx(out, `swipe_t${this.tier4()}_a${heading8(dx, dy) % 4}`, div(l.ax + l.bx, 2), div(l.ay + l.by, 2), null, 24);
      this.swipe_until = tick + 24;
      return true;
    }
    twist(out, t, idx, f, amount) {
      const tick = t.tick;
      const w = TWISTS[idx];
      this.stats.twists++;
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
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], C.twist_radius)).map(a => a.id)) {
            if (this.cooldownOk(5003, a, tick)) { this.setCooldown(5003, a, tick + 600); out.push({ k: 'Buff', target: a, name: 'vst_sanctuary', ticks: mag, mods: Mods({ cc_immune: true }) }); }
          }
          break;
        case 4:
          if (this.control(out, f, 'Root', mag, tick)) { const to = lerp(f.x, f.y, exit[0], exit[1], 50); this.displace(out, f, to[0], to[1], tick, t.wall, true); }
          break;
        case 5: {
          const e = minBy(t.enemies().filter(e => e.id !== f.id && within(e.x, e.y, f.x, f.y, C.overcharge_radius)), e => [dist2(e.x, e.y, f.x, f.y), e.id]);
          if (e) this.emit(out, e, div(amount * mag, 100), 'Phys', tick);
          break;
        }
        case 6: if (this.displaced.has(f.id) && tick <= this.displaced.get(f.id) + 60) this.emit(out, f, div(f.max_hp * mag, 1000), 'Phys', tick); break;
        case 7: case 12:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], C.twist_radius))) this.heal(out, t, a.id, w.kind === 7 ? div(amount * mag, 100) : div(a.max_hp * mag, 1000), tick);
          break;
        case 8: {
          const far = maxBy(this.rifts.filter(riftLive), r => [dist2(r.x, r.y, exit[0], exit[1]), r.uid]);
          if (far) {
            const r = cloneRift(far);
            const dim = r.dim;
            for (const e of t.enemies().filter(e => riftCovers(r, e.x, e.y))) this.applyDim(out, t, dim, e, 100, [r.x, r.y, r.radius], 'Zone');
          }
          break;
        }
        case 9:
          for (const e of t.enemies().filter(e => e.id !== f.id && within(e.x, e.y, f.x, f.y, C.twist_radius))) { const to = lerp(e.x, e.y, f.x, f.y, Math.min(100, mag)); this.displace(out, e, to[0], to[1], tick, t.wall, true); }
          break;
        case 10:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], C.twist_radius)).map(a => a.id)) out.push({ k: 'Buff', target: a, name: 'vst_escort', ticks: 60, mods: Mods({ move_speed: mag }) });
          break;
        case 11: if (first) this.displace(out, f, first[0], first[1], tick, t.wall, true); break;
        case 13:
          for (const a of t.friends().filter(a => within(a.x, a.y, exit[0], exit[1], C.twist_radius))) this.shield(out, a, div(a.max_hp * mag, 1000), 180, tick);
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
          if (tick < r.drawn[layer]) continue;
          const hidden = riftHidden(r, tick);
          if (hidden && layer > 0) continue;
          const line = r.kind === R.slash || r.kind === R.mirror;
          const tag = layer === 0 ? (hidden ? 'seedhint_l' + l : line ? `rift_${RIFTS[r.kind].id}_l${l}_a${heading8(r.ex - r.x, r.ey - r.y) % 4}` : `rift_${RIFTS[r.kind].id}_l${l}`)
            : layer === 1 ? `core_f${DIMS[r.dim].family}_t${tier}` : 'glyph' + String(r.dim).padStart(2, '0');
          const [x, y] = line ? [div(r.x + r.ex, 2), div(r.y + r.ey, 2)] : [r.x, r.y];
          if (this.fxLow(out, tag, x, y, r.host, 48)) r.drawn[layer] = tick + 48;
        }
      }
      let prev = [t.me.x, t.me.y];
      for (const n of this.chainNodes(tick)) {
        const r = this.rifts.find(q => q.uid === n.uid && !q.done);
        if (r && tick >= r.drawn[3] && this.fxLow(out, `link_l${l}_a${heading8(n.x - prev[0], n.y - prev[1])}`, div(n.x + prev[0], 2), div(n.y + prev[1], 2), null, 48)) r.drawn[3] = tick + 48;
        prev = [n.x, n.y];
      }
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
      return div(e.hp * 100, e.max_hp) - div(e.attack, 6) + (e.id === tank ? 80 : 0) + div(dist(e.x, e.y, t.me.x, t.me.y), 20000);
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
    // An effect's worth to the planner. `ctx` is the fight around the spot, read once per pick, or its bound: the
    // best this effect could be worth anywhere.
    effValue(t, ctx, e, purpose, target, craft) {
      const p = e[0];
      if (p === 255) return 0;
      const sd = PRIM_SIDE[p];
      const foe = purpose === 'Punch' || purpose === 'Swipe' || purpose === 'Enemy';
      if (purpose === 'Route') return sd === 'P' ? 16 : 3;
      if (purpose === 'Ally') {
        if (sd === 'A') return 14 + ((craft >= 2 && [P.HEAL, P.SHIELD, P.SHELTER, P.UNDYING, P.RECALLHP].includes(p) && ctx.ally_low) ? 10 : 0);
        return sd === 'S' ? 6 : sd === 'X' ? 4 : 2;
      }
      if (craft === 0) return ((sd === 'E' || sd === 'P') && foe) ? 12 : sd === 'X' ? 6 : 3;
      let v;
      if (sd === 'A') v = craft >= 2 ? (ctx.allies_near > 0 ? 6 + 4 * ctx.allies_near : -4) : 6;
      else if (sd === 'S') v = 6;
      else v = p === P.CHILL ? 10 : isControl(p) ? 16 : isDamage(p) ? 14 : isMove(p) ? 6 : 8;   // a freeze takes several stacks
      if (craft >= 2) {
        if (!ctx.bound && target) {
          const f = target;
          const low = f.hp * 100 < f.max_hp * 40;
          if (isControl(p) && f.cc_immune) v -= 28;
          if (isControl(p) && f.controlled) v -= 10;
          if ((p === P.EXECUTE || p === P.AMPLIFY) && low) v += 8;
          if (p === P.HEALCUT && f.hp * 2 < f.max_hp) v += 6;
          if ((p === P.GRAB || p === P.PULL) && ctx.ally_by_me) v += 12;
          if (p === P.KNOCK || p === P.REWIND) v += dist(f.x, f.y, t.me.x, t.me.y) <= C.dive_radius ? 12 : -2;
          if (p === P.MOMENTUM && this.momentum(t, e[1], e[2]) === 0) v -= 10;   // worth nothing standing still
        } else if (ctx.bound) {
          // the bound: every bonus this effect could earn against some target
          v += (p === P.EXECUTE || p === P.AMPLIFY) ? 8 : p === P.HEALCUT ? 6 : (p === P.GRAB || p === P.PULL || p === P.KNOCK || p === P.REWIND) ? 12 : 0;
        }
        if (target && p === P.SELFSWAP) v -= 8;
        if ([P.SPLASH, P.CHAIN, P.RICOCHET, P.MARK, P.PRISM].includes(p)) v += 3 * Math.max(0, ctx.enemies_near - 1);
        if (p === P.UNSTABLE) v += this.stress > div(C.stress_cap * 3, 5) ? -25 : 4;
        if (p === P.VENT) v += div(this.stress, 5);
      }
      return v;
    }
    static deliveries(dim, purpose) {
      if (purpose === 'Punch') return [PUNCH[dim].main, PUNCH[dim].rider];
      if (purpose === 'Swipe') return [SWIPE[dim].main, SWIPE[dim].rider];
      const d = DIMS[dim];
      return [[d.prim, d.a, d.b, d.c], NO_EFF];
    }
    dimValue(t, ctx, dim, purpose, target, craft) {
      const [main, rider] = Core.deliveries(dim, purpose);
      let v = this.effValue(t, ctx, main, purpose, target, craft) + div(this.effValue(t, ctx, rider, purpose, target, craft), 2);
      if (craft >= 3 && t.tick <= this.combo_at + C.combo_window && this.last_family !== null) {
        const fam = DIMS[dim].family;
        if (fam !== this.last_family) v += 6 + ([0, 1, 5].includes(TWISTS[twistIndex(this.last_family, fam)].kind) ? 4 : 0);
      }
      return v;
    }
    // The dimensions an orb may open now (all of them until cooldowns exist).
    // whether a dimension may be cast now: its own cooldown and its family's recovery are over
    castOk(dim, tick) { return this.free_casts || (tick >= this.dim_ready[dim] && tick >= this.family_ready[DIMS[dim].family]); }
    availableDims(tick) { return DIMS.map((_, i) => i).filter(d => this.castOk(d, tick)); }
    anyCastable(tick) { return DIMS.some((_, d) => this.castOk(d, tick)); }
    // a cast committed: its dimensions rest and their families recover; one action's rifts and a fused punch lock once
    commitCast(dims, tick) {
      if (this.free_casts) return;
      for (const d of dims) {
        this.dim_ready[d] = Math.max(this.dim_ready[d], tick + DIMS[d].cd);
        const f = DIMS[d].family;
        this.family_ready[f] = Math.max(this.family_ready[f], tick + C.family_recovery);
      }
    }
    dimsFor(a, tick) {
      const needs = d => (d === null || d === undefined) ? this.anyCastable(tick) : this.castOk(d, tick);
      if (a.tech === MV.swipe) return needs(a.dim);
      return a.tech >= 28 || a.rifts.every(r => needs(r.dim));
    }
    // The fight around a spot, read once per pick.
    static pickCtx(t, at) {
      return {
        allies_near: t.allies().filter(a => dist(a.x, a.y, at[0], at[1]) <= 50000).length,
        enemies_near: t.enemies().filter(o => dist(o.x, o.y, at[0], at[1]) <= 30000).length,
        ally_low: t.allies().some(a => a.hp * 2 < a.max_hp),
        ally_by_me: t.allies().some(a => dist(a.x, a.y, t.me.x, t.me.y) <= 50000),
        bound: false,
      };
    }
    // An upper bound on every spot's value: the planner skips a dimension whose bound cannot win.
    static pickBound(t) { return { allies_near: t.allies().length, enemies_near: t.enemies().length, ally_low: true, ally_by_me: true, bound: true }; }
    // A Spark picks any available dimension; better athletes read the fight. Each dimension draws its noise in
    // order; one whose bound plus noise cannot beat the best so far is skipped unscored (same result as a full scan).
    pickDim(t, purpose, target, at) {
      const avail = this.availableDims(t.tick);
      if (avail.length === 0) return 0;
      if (this.rank === 0) return avail[this.random(avail.length)];
      const craft = this.table('orb_craft');
      const noise = this.table('noise_pct');
      const ctx = Core.pickCtx(t, at);
      const bound = Core.pickBound(t);
      let best = null;
      for (const dim of avail) {
        const n = noise > 0 ? this.random(2 * noise + 1) - noise : 0;
        if (best !== null && this.dimValue(t, bound, dim, purpose, target, craft) + n <= best[0]) continue;
        const v = this.dimValue(t, ctx, dim, purpose, target, craft) + n;
        if (best === null || v > best[0]) best = [v, dim];
      }
      const pick = best === null ? 0 : best[1];
      if (purpose === 'Punch' || purpose === 'Swipe') {
        this.habit('dimension picked for the target');
        if (craft >= 3 && this.last_family !== null && this.last_family !== DIMS[pick].family && t.tick <= this.combo_at + C.combo_window) this.habit('combo families for the twist');
      }
      return pick;
    }
    planChain(t, target, kindMid) {
      const guess = this.lead(target, 40);
      const reach = div(C.attack_range * 7, 10);
      const d = dist(t.me.x, t.me.y, guess[0], guess[1]);
      if (d <= C.punch_reach) return null;
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
    // where a pulled enemy should land: just in front of her frontliner, or as far toward it as a drop reaches
    dropSpot(v, from) {
      if (!v.allies.length) return null;
      const front = minBy(v.allies, a => [dist2(a.x, a.y, v.enemy_centre[0], v.enemy_centre[1]), a.id]);
      const spot = along(front.x, front.y, from[0] - front.x, from[1] - front.y, 12000);
      return dist(spot[0], spot[1], from[0], from[1]) > C.teleport_max ? along(from[0], from[1], spot[0] - from[0], spot[1] - from[1], C.teleport_max) : spot;
    }
    randomNear(x, y, r) {
      const [ux, uy] = DIRS[this.random(16)];
      const k = this.random(Math.max(1, r));
      return clampMap(x + div(ux * k, 10000), y + div(uy * k, 10000));
    }
    propose(t, v, tech) {
      if (tech >= 28) return this.proposeMove(t, v, tech);
      const tick = t.tick;
      const me = [t.me.x, t.me.y];
      const f = v.focus;
      const act = (rifts, subject, uses, score, habit) => ({ tech, rifts, subject: subject === undefined ? null : subject, uses, point: null, dim: null, score, habit: habit || '' });
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
        case T.opunch:
          if (this.punching && this.stress >= div(C.stress_cap, 4) && this.opunch === 0) {
            const spend = Math.min(this.stress, C.stress_max_overload);
            out.push(act([], null, [], this.gate(G.safety) ? div(spend, 3) + (this.stress >= div(C.stress_cap * 7, 10) ? 12 : 0) : 14, ''));
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
              const synced = toward && close && dist(ent[0], ent[1], behind[0], behind[1]) <= C.teleport_max;
              const score = synced ? (tech === T.skydrop ? 40 + 6 * cluster : 50) : 8;
              out.push(act([Spec.entrance(R.tear, ent[0], ent[1]), Spec.at(exitKind, behind[0], behind[1])], fr.id, [], score, synced ? 'Gate synced with the tank' : ''));
            }
          } else if (v.allies.length) {
            const a = v.allies[this.random(v.allies.length)];
            out.push(act([Spec.entrance(R.tear, a.x, a.y), Spec.at(exitKind, f.x, f.y)], a.id, [], 16, ''));
          }
          break;
        }
        case T.kidnap: {
          const subject = f;
          const front0 = along(subject.x, subject.y, me[0] - subject.x, me[1] - subject.y, C.kidnap_front);
          // a careful Kidnapper only blinks where at most one other enemy can punish her
          const exposed = this.gate(G.safety) && t.danger(front0[0], front0[1], 50000) > 2;
          const tank = this.gate(G.priority) && subject.id === v.tank;   // the tank is not worth a blink
          if (tick >= this.kidnap_ready && dist(me[0], me[1], subject.x, subject.y) <= C.kidnap_reach && !exposed && !tank && !(this.gate(G.safety) && subject.cc_immune)) {
            const ent = me;
            let exit;
            if (this.gate(G.team) && v.allies.length) {
              // just in front of her frontliner: the target lands in her team and she stays in the fight
              exit = this.dropSpot(v, [subject.x, subject.y]);
            } else if (this.gate(G.safety)) {
              const p = this.safeBack(v, t, 45000);
              exit = dist(p[0], p[1], me[0], me[1]) > 25000 ? p : null;
            } else exit = this.randomNear(subject.x, subject.y, 60000);   // a Spark kidnaps on a whim
            if (exit) {
              const score = this.gate(G.team) ? 44 + div(this.prio(subject, v), 4) : 20;
              out.push(act([Spec.entrance(R.tear, ent[0], ent[1]), Spec.at(R.tear, exit[0], exit[1])], subject.id, [], score, (this.gate(G.priority) && subject.id !== v.tank) ? 'Kidnapped the carry into the team' : ''));
            }
          }
          break;
        }
        case T.sinktrap: {
          const spot = this.gate(G.priority) ? lerp(v.enemy_centre[0], v.enemy_centre[1], v.ally_centre[0], v.ally_centre[1], 35) : this.lead(f, this.delayTo(t, f.x, f.y, R.sinkhole));
          const n = v.enemies.filter(e => dist(e.x, e.y, spot[0], spot[1]) <= 40000).length;
          const exit = this.gate(G.team) ? (this.dropSpot(v, spot) || v.ally_centre) : me;
          out.push(act([Spec.entrance(R.sinkhole, spot[0], spot[1]), Spec.at(R.tear, exit[0], exit[1])], null, [], 9 * n + 10, this.gate(G.priority) ? 'Sinkhole at the chokepoint' : ''));
          break;
        }
        case T.swap:
          if (this.gate(G.team)) {
            const fr = v.front;
            if (fr) {
              const a = this.lead(f, this.delayTo(t, f.x, f.y, R.twin));
              const b = this.lead(fr, this.delayTo(t, fr.x, fr.y, R.twin));
              if (dist(a[0], a[1], b[0], b[1]) <= C.teleport_max) out.push(act([Spec.entrance(R.twin, a[0], a[1]), Spec.at(R.twin, b[0], b[1])], f.id, [], 36 + div(this.prio(f, v), 6), 'swapped the target into the team'));
            }
          } else if (!this.gate(G.safety) || t.danger(f.x, f.y, 35000) < 3) {
            out.push(act([Spec.entrance(R.twin, me[0], me[1]), Spec.at(R.twin, f.x, f.y)], f.id, [], 18, ''));
          }
          break;
        case T.fling: {
          const start = this.lead(f, this.delayTo(t, f.x, f.y, R.wander));
          let end = this.gate(G.team) ? this.dropSpot(v, start) : null;
          if (!end) {
            const [dx, dy] = this.gate(G.team) ? [v.ally_centre[0] - start[0], v.ally_centre[1] - start[1]] : DIRS[this.random(16)];
            const e = along(start[0], start[1], dx, dy, C.teleport_max);
            end = clampMap(e[0], e[1]);
          }
          const n = v.enemies.filter(e => dist(e.x, e.y, start[0], start[1]) <= 30000).length;
          out.push(act([Spec.line(R.wander, start[0], start[1], end[0], end[1])], null, [], 10 * n + 4, this.gate(G.team) ? 'Fling toward the team' : ''));
          break;
        }
        case T.hook:
          if (!(this.gate(G.safety) && f.cc_immune)) {
            const rifts = [{ ...Spec.at(R.anchor, f.x, f.y), host: f.id }];
            if (this.gate(G.team)) { const exit = this.dropSpot(v, [f.x, f.y]); if (exit) rifts.push(Spec.at(R.tear, exit[0], exit[1])); }
            out.push(act(rifts, f.id, [], 22 + div(this.prio(f, v), 6), ''));
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
          if (this.gate(G.team)) { const exit = this.dropSpot(v, spot); if (exit) rifts.push(Spec.at(R.tear, exit[0], exit[1])); }
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
        case T.infuse:
          if (this.hexReady(tick) > 0 && live.length) {
            let pick;
            if (this.gate(G.priority)) {
              const frontliner = v.front ? v.front.id : null;
              pick = maxBy(live.filter(r => r.infused === null), r => [((r.host !== null && r.host === frontliner) ? 1 : 0) * 3 + t.danger(r.x, r.y, r.radius * 2), r.uid]);
            } else pick = live[this.random(live.length)];
            if (pick) {
              const busy = t.danger(pick.x, pick.y, pick.radius * 2) + t.allies().filter(a => riftCovers(pick, a.x, a.y)).length;
              const front = this.gate(G.priority) && pick.host !== null && v.front && pick.host === v.front.id;
              out.push(act([], null, [pick.uid], this.gate(G.priority) ? 8 * busy + (front ? 20 : 0) : 12, front ? "Infused the frontliner's rift" : ''));
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
    proposeMove(t, v, m) {
      const tick = t.tick;
      const me = [t.me.x, t.me.y];
      const act = (point, subject, score, habit) => ({ tech: m, rifts: [], subject: subject === undefined ? null : subject, uses: [], point, dim: null, score, habit: habit || '' });
      const out = [];
      const fighting = v.enemies.some(e => dist(e.x, e.y, me[0], me[1]) <= 90000);
      switch (m) {
        case MV.swipe:
          if (this.orbsReady(tick) >= 2) {
            const on = p => {
              const b = along(me[0], me[1], p[0] - me[0], p[1] - me[1], C.swipe_len);
              return v.enemies.filter(e => segDist(me[0], me[1], b[0], b[1], e.x, e.y) <= C.swipe_width).length;
            };
            let point, n;
            if (this.gate(G.geometry)) {
              let best = [[v.focus.x, v.focus.y], on([v.focus.x, v.focus.y])];
              for (const e of v.enemies) { const k = on([e.x, e.y]); if (k > best[1]) best = [[e.x, e.y], k]; }
              [point, n] = best;
            } else if (this.rank === 0) {
              point = this.randomNear(me[0], me[1], C.swipe_len);
              n = on(point);
            } else { point = [v.focus.x, v.focus.y]; n = on(point); }
            // the spine saves its swipe for lines: one target is worth less than loading an orb
            const score = this.gate(G.geometry) ? (n === 0 ? -20 : (n === 1 && this.gate(G.spine)) ? 12 : 14 * n + 4) : 8 + 8 * n;
            out.push(act(point, v.focus.id, score, n >= 2 ? 'swipe through the line' : ''));
          }
          break;
        case MV.shatter: {
          const held = this.pureReady(tick);
          if (held > 0 && this.openCount() > 0) {
            const over = this.pure.filter(s => s.open && s.over && tick >= s.ready_at).length;
            const load = (held - over) * C.shatter_load + over * C.shatter_load_over;
            const hits = v.enemies.filter(e => this.rifts.some(r => riftLive(r) && !riftHidden(r, tick) && riftReach(r, e.x, e.y) <= r.radius * 2)).length;
            const breaks = this.stress + load > C.stress_cap;
            const score = !this.gate(G.safety) ? 6 + 8 * hits + 3 * over : breaks ? -60 : hits >= 2 ? 12 * hits + 4 * held + (this.gate(G.spine) ? 8 * over : 0) : -10;
            const habit = (!breaks && over > 0 && hits >= 3) ? 'overload ridden into a Shatter' : (!breaks && hits >= 2) ? 'shattered while the gauntlet held' : '';
            out.push(act(null, null, score, habit));
          }
          break;
        }
        case MV.overload:
          if (!this.leftBroken(tick) && this.sockets() < C.orbs_max) {
            const near = this.sockets() >= 7 && t.danger(me[0], me[1], 60000) > 0;
            const score = !this.gate(G.safety) ? 14
              : (near || this.stress + C.over_stress + 10 > this.table('stress_limit')) ? -30
                : (v.enemies.some(e => dist(e.x, e.y, me[0], me[1]) <= 150000) && !v.diver && this.sockets() < 8) ? (this.gate(G.spine) ? 18 : 16) : 4;
            const habit = (this.gate(G.spine) && score > 20) ? 'overloaded before the engage' : near ? 'stopped overloading near enemies' : '';
            out.push(act(null, null, score, habit));
          }
          break;
        case MV.vent:
          if (this.overSockets() > 0) {
            const dived = this.gate(G.survival) && v.diver;
            const score = dived ? 50 : (this.gate(G.safety) && this.stress >= div(C.stress_cap * 3, 4)) ? 30 : 2;
            out.push(act(null, null, score, dived ? 'vented overload when dived' : ''));
          }
          break;
        case MV.awaken:
          if (this.completion >= 100 && !this.awakened(tick) && !this.rightBroken(tick)) {
            const engage = fighting && v.allies.some(a => t.danger(a.x, a.y, 60000) > 0);
            const score = this.gate(G.team) ? (engage ? 48 : -20) : this.gate(G.safety) ? (fighting ? 26 : 4) : 30;
            out.push(act(null, null, score, (this.gate(G.team) && engage) ? 'Awakening timed with the engage' : ''));
          }
          break;
        case MV.overclock:
          if (this.awakened(tick) && this.pureReady(tick) > 0) {
            const next = this.corruption + C.corrupt_per_orb;
            const score = !this.gate(G.safety) ? 20 : next >= 100 ? -60 : this.gate(G.spine) ? ((next <= 45 && fighting) ? 18 : -10) : next < 60 ? 8 : -20;
            out.push(act(null, null, score, ''));
          }
          break;
        default: break;
      }
      return out;
    }
    static cost(a) {
      if (a.tech >= 28) return MOVES[a.tech - 28].orbs;
      return a.rifts.reduce((s, r) => s + RIFTS[r.kind].cost, 0) + TECHS[a.tech].orbs;
    }
    stressOf(a, me) {
      if (a.tech >= 28) return a.tech === MV.swipe ? C.stress_tear : a.tech === MV.overload ? C.over_stress : 0;
      let s = TECHS[a.tech].stress + C.stress_tear * a.rifts.length;
      let prev = me;
      for (const r of a.rifts.filter(r => r.chain)) { if (dist(prev[0], prev[1], r.x, r.y) > C.link_scale) s += C.stress_long; prev = [r.x, r.y]; }
      return s;
    }
    affordable(a, me, tick) {
      const cost = Core.cost(a);
      const reserve = (a.tech >= 28 || TECHS[a.tech].group === 'defense') ? 0 : this.table('reserve');
      if (this.orbsReady(tick) < cost + (cost > 0 ? reserve : 0)) return false;
      if (a.tech === T.infuse && this.hexReady(tick) === 0) return false;
      if (this.openCount() + a.rifts.length + (a.tech === MV.swipe ? 1 : 0) > C.max_rifts) return false;
      if (a.uses.some(u => !this.rift(u)) || !this.dimsFor(a, tick)) return false;
      const buffer = this.gate(G.survival) ? C.stress_break : 0;
      return !this.gate(G.safety) || a.tech === MV.shatter || a.tech === MV.vent || this.stress + this.stressOf(a, me) + buffer <= this.table('stress_limit');
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
      if (this.plan.length && this.plan[0] === T.kidnap && tick < this.kidnap_ready) this.plan.shift();   // skipped, not waited for
      const next = this.plan.length ? this.plan[0] : null;
      if (next !== null && !pool.includes(next)) pool.push(next);
      pool.sort((a, b) => a - b);
      for (let m = 28; m < 34; m++) pool.push(m);
      const noise = this.table('noise_pct');
      const me = [t.me.x, t.me.y];
      let best = null;
      this.recent = this.recent.filter(r => tick < r[1] + C.repeat_window);
      for (const tech of pool) {
        // variety: a technique she just used is less interesting (the gauntlet moves are her mechanics, not skills)
        const repeats = this.recent.filter(r => r[0] === tech).length;
        if (tech < 28 && repeats >= C.repeat_max) continue;
        const last = this.recent.length ? this.recent[this.recent.length - 1][0] : null;
        const fatigue = tech >= 28 ? 0 : C.repeat_penalty * repeats + (last === tech ? C.repeat_last : 0);
        for (const a of this.propose(t, v, tech)) {
          a.score -= Math.min(fatigue, div(Math.max(a.score, 0), 2));
          if (!this.affordable(a, me, tick)) continue;
          if (tech === next) a.score += 30;
          const depth = this.table('combo_depth');
          if (next === null && depth >= 3 && a.score >= 20 && COMBOS.some(c => c[0] === tech)) a.score += 4 * (depth - 1);
          if (noise > 0) a.score += this.random(2 * noise + 1) - noise;
          if (best === null || a.score > best.score) best = a;
        }
      }
      if (!best || best.score < 12) return;
      const tech = best.tech;
      if (tech < 28) this.recent.push([tech, tick]);
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
    static purposeOf(tech, spec) {
      if (spec.chain) return 'Route';
      if ((spec.entrance && [T.gate, T.skydrop, T.rotation, T.rescue, T.step, T.phase].includes(tech)) || tech === T.pocket || tech === T.lifeline) return 'Ally';
      return 'Enemy';
    }
    execute(out, t, a) {
      const tick = t.tick;
      const cost = Core.cost(a);
      if (this.orbsReady(tick) < cost) return;
      if (a.tech >= 28) {
        let done;
        switch (a.tech) {
          case MV.swipe: done = this.swipe(out, t, a.point || [t.me.x + 1000, t.me.y], a.dim, a.subject); break;
          case MV.shatter: if (this.pureReady(tick) === 0) done = false; else { this.shatter(out, t, false); done = true; } break;
          case MV.overload: done = this.overload(out, t); break;
          case MV.vent: { const any = this.overSockets() > 0; this.vent(); done = any; break; }
          case MV.awaken: done = this.awaken(out, t); break;
          default: done = this.overclock(out, t);
        }
        if (done) {
          this.stats.move_uses[a.tech - 28]++;
          this.habit(a.habit);
          out.push({ k: 'Log', s: `tick ${tick} rank ${RANKS[this.rank]} ${MOVES[a.tech - 28].id} score ${a.score} stress ${this.stress} orbs ${this.pureReady(tick)}/${this.hexReady(tick)}` });
        }
        return;
      }
      if ((a.tech === T.infuse && this.hexReady(tick) === 0) || this.openCount() + a.rifts.length > C.max_rifts || !this.dimsFor(a, tick)) return;
      const stress = this.stressOf(a, [t.me.x, t.me.y]);
      this.stats.tech_uses[a.tech]++;
      if (a.tech === T.kidnap) this.kidnap_ready = tick + C.kidnap_cd;
      this.habit(a.habit);
      out.push({ k: 'Log', s: `tick ${tick} rank ${RANKS[this.rank]} ${TECHS[a.tech].name} score ${a.score} rifts ${a.rifts.length} stress ${this.stress + stress}` });
      this.fx(out, `tech_${TECHS[a.tech].id}_t${this.tier4()}`, t.me.x, t.me.y, t.me.id, 24);
      const hexy = this.rank >= G.safety && ['defense', 'team', 'engage'].includes(TECHS[a.tech].group);
      const uids = [];
      const cast = [];
      for (const spec of a.rifts) {
        const n = RIFTS[spec.kind].cost;
        const pref = this.hand_placed ? false : this.rank < G.safety ? this.random(2) === 0 : hexy;
        const orb = this.take(tick, pref);
        if (!orb) break;
        for (let i = 1; i < n; i++) this.take(tick, orb.hex !== null);
        const subj = a.subject !== null ? t.get(a.subject) : null;
        const dim = (spec.dim !== null && spec.dim !== undefined) ? spec.dim : this.pickDim(t, Core.purposeOf(a.tech, spec), (subj && subj.team !== t.me.team) ? subj : null, [spec.x, spec.y]);
        const twist = this.activate(orb, dim, tick)[1];
        cast.push(dim);
        if (twist !== null) {
          const e = minBy(t.enemies().filter(e => dist(e.x, e.y, spec.x, spec.y) <= C.near_radius), e => [dist2(e.x, e.y, spec.x, spec.y), e.id]);
          if (e) this.twist(out, t, twist, e, 20 + div(t.me.attack, 2));
        }
        const module = orb.hex !== null ? this.modules[orb.hex] : null;
        uids.push(this.open(out, t, spec, a.tech, a.subject, dim, module));
      }
      this.commitCast(cast, tick);
      this.addStress(out, t, stress);
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
        case T.crossfire: this.crossfire_until = tick + 180; break;
        case T.opunch: { const spend = Math.min(this.stress, C.stress_max_overload); this.opunch = spend; this.stress -= spend; break; }
        case T.infuse:
          if (a.uses.length) {
            const orb = this.take(tick, true);
            if (orb) {
              const module = orb.hex !== null ? this.modules[orb.hex] : this.modules[0];
              const r = this.rift(a.uses[0]);
              if (r) { r.infused = module; this.fx(out, 'tech_infuse_t' + tier, r.x, r.y, null, 30); }
            }
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
      this.rifts = []; this.packets = []; this.later = []; this.dots = []; this.marks.clear(); this.chill.clear(); this.exposed.clear();
      this.vent();
      this.aiming = false; this.punching = false; this.phased = false; this.chain_target = null; this.plan = []; this.opunch = 0;
      this.awaken_until = Math.min(this.awaken_until, tick);
      this.corruption = 0;
      this.completion = Math.max(C.hex_start, this.completion - C.research_death);
      this.combo = 0; this.last_family = null;
      this.damage.clear(); this.controls.clear(); this.moves.clear();
      this.dim_ready.fill(0); this.family_ready.fill(0);
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
      if (target === 0 && n > 0) this.core.block_until = tick + 18;      // her block pose (visual only)
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
          // Vesper's enemies follow the game's input-AI rule: out of her rifts and Kidnap fields, never into them
          const g = steer(u.f.team !== 0 ? zonesOf(this.core, tick) : [], [u.f.x, u.f.y], [target.f.x, target.f.y]);
          let [nx, ny] = g ? along(u.f.x, u.f.y, g[0] - u.f.x, g[1] - u.f.y, Math.min(speed, dist(u.f.x, u.f.y, g[0], g[1])))
            : along(u.f.x, u.f.y, target.f.x - u.f.x, target.f.y - u.f.y, Math.min(speed, d - u.range + 1));
          [nx, ny] = land(u.f.x, u.f.y, nx, ny, this.wall);
          const me = this.unit(id);
          me.f.x = nx; me.f.y = ny;
        }
      } else if (u.disarm <= tick && tick >= u.next_attack) {
        const interval = div(u.interval * 100, Math.max(20, 100 + m.attack_speed));
        this.unit(id).next_attack = tick + Math.max(10, interval);
        const dmg = div(u.f.attack * Math.max(0, 100 + m.attack), 100);
        this.hurt(target.f.id, dmg, id);
        if (id === 0) { this.core.stats.damage += dmg; this.core.stats.jab_damage += dmg; this.core.jab_until = tick + 24; }
      }
    }
    run(ticks) { for (let i = 0; i < ticks; i++) { this.step(); if (this.wiped() !== null) break; } }
    wiped() {
      for (let team = 0; team < 2; team++) if (!this.units.some(u => u.f.team === team && u.f.hp > 0)) return team;
      return null;
    }
  }
  // Her animated pose overlay, the same choice as native vesper::pose_of.
  function poseOf(core, tick) {
    return tick < core.shatter_until ? 'shatter' : tick < core.swipe_until ? 'swipe' : core.punching ? 'punch'
      : tick < core.jab_until ? 'jab' : tick < core.block_until ? 'block' : 'outfit';
  }
  function duel(seed, rank, top) {
    return new Arena(seed, rank, [new Unit(0, 0, 200000, 480000, 1100, 90, C.normal_range), new Unit(5, 1, 520000, 480000, 2400, 60, 30000)], top);
  }
  // The enemy line-up carries +45% HP and +20% attack: an even fight for a mid-rank Vesper (vesper_tests::teamfight).
  function teamfight(seed, rank, top, foe) {
    const [hpPct, atkPct] = foe || [145, 120];
    const [home, away] = seed % 2 === 1 ? [300000, 600000] : [260000, 700000];
    const units = [new Unit(0, 0, home - 80000, 480000, 1100, 90, C.normal_range)];
    const roles = [[2600, 55, 25000], [1800, 80, 28000], [1200, 95, 60000], [1300, 60, 55000]];
    roles.forEach(([hp, atk, range], i) => units.push(new Unit(1 + i, 0, home - div(range, 3), 400000 + i * 50000, hp, atk, range)));
    const foes = [[2600, 55, 25000], [1800, 80, 28000], [1200, 95, 60000], [1300, 60, 55000], [1100, 90, C.attack_range]];
    foes.forEach(([hp, atk, range], i) => units.push(new Unit(10 + i, 1, away + div(range, 3), 380000 + i * 50000, div(hp * hpPct, 100), div(atk * atkPct, 100), range)));
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
        const s = a.core.stats, c = a.core;
        const hp = a.units.map(u => `${u.f.id}:${u.f.hp}:${u.f.x}:${u.f.y}`).join(',');
        rows.push(`${seed}|${rank}|end tick ${a.tick} stress ${c.stress} hexcore ${c.completion} corruption ${c.corruption} orbs ${c.pureReady(a.tick)}/${c.hexReady(a.tick)} damage ${s.damage} punches ${s.punches} landed ${s.landed} eff ${s.eff_sum} swipes ${s.swipe_hits}/${s.swipes} shatters ${s.useful_shatters}/${s.shatters} breaks L${s.left_breaks} R${s.right_breaks} awakens ${s.good_awakens}/${s.awakens} twists ${s.twists} fused ${s.fused} rifts_broken ${s.breaks} kidnaps ${s.useful_kidnaps}/${s.kidnaps} dodges ${s.kidnap_dodges} gates ${s.engaged_gates}/${s.gates} traps ${s.trap_catches}/${s.traps} implodes ${s.implodes} effects ${s.effects} uses [${s.tech_uses.join(', ')}] moves [${s.move_uses.join(', ')}] units ${hp}`);
      }
    }
    const digest = a => {
      const units = a.units.map(u => `${u.f.id}:${u.f.hp}:${u.f.x}:${u.f.y}`).join(',');
      const s = a.core.stats, c = a.core;
      return `tick ${a.tick} stress ${c.stress} hexcore ${c.completion} corruption ${c.corruption} orbs ${c.pureReady(a.tick)}/${c.hexReady(a.tick)} damage ${s.damage} landed ${s.landed} twists ${s.twists} swipes ${s.swipe_hits}/${s.swipes} units ${units}`;
    };
    for (let dim = 0; dim < DIMS.length; dim++) {
      const a = teamfight(11, 7);
      a.core.hand_placed = true;
      [10, 11, 12, 1].forEach((id, i) => { const u = a.unit(id); u.f.x = 260000 + i * 7000; u.f.y = 480000 + (i % 2) * 5000; });
      a.unit(0).f.x = 220000; a.unit(0).f.y = 480000;
      a.core.armed = dim;
      a.run(150);
      rows.push(`hand|punch|${dim}|${digest(a)}`);
      const b = teamfight(11, 7);
      b.core.hand_placed = true;
      b.frozen = true;
      [10, 11, 12, 1].forEach((id, i) => { const u = b.unit(id); u.f.x = 240000 + i * 9000; u.f.y = 480000 + (i % 2) * 4000; });
      b.unit(0).f.x = 220000; b.unit(0).f.y = 480000;
      b.place({ tech: MV.swipe, rifts: [], subject: 10, uses: [], point: [300000, 480000], dim, score: 0, habit: '' });
      b.run(150);
      rows.push(`hand|swipe|${dim}|${digest(b)}`);
    }
    const g = teamfight(5, 7);
    g.core.hand_placed = true;
    g.frozen = true;
    for (const id of [10, 11]) { const u = g.unit(id); u.f.x = 500000 + id * 1000; u.f.y = 480000; }
    const act = (tech, rifts) => ({ tech, rifts: rifts || [], subject: null, uses: [], point: null, dim: null, score: 0, habit: '' });
    g.place(act(T.chain, [{ ...Spec.node(R.tear, 500000, 480000), dim: 11 }]));
    for (let i = 0; i < 3; i++) g.place(act(MV.overload));
    g.run(60);
    g.place(act(MV.shatter));
    rows.push(`hand|shatter|${digest(g)}`);
    const h = teamfight(5, 7);
    h.core.hand_placed = true;
    h.frozen = true;
    h.unit(10).f.x = 225000; h.unit(10).f.y = 480000;
    h.core.completion = 100;
    h.place(act(MV.overload));
    h.place(act(MV.overload));
    h.place(act(MV.awaken));
    h.run(30);
    for (let i = 0; i < 5; i++) h.place(act(MV.overclock));
    h.run(30);
    rows.push(`hand|overclock|${digest(h)}`);
    return rows.join('\n') + '\n';
  }
  const api = { Core, Arena, Unit, Spec, duel, teamfight, poseOf, poseLook, simVectors, heading8, look, artTier, habitGate, land, riftLive, riftHidden, riftDims, riftReach, nearestOn, P, R, T, MV, DIMS, PUNCH, SWIPE, RIFTS, TECHS, MOVES, MODULES, TWISTS, RANKS, PRIM_SIDE };
  if (isNode) module.exports = api; else root.VesperCombat = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
