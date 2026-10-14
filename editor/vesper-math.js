// Vesper's integer rules for the Rift Lab, ported 1:1 from native/tfm2_custom_ai/src/vesper_math.rs and the
// charge bank / rolling windows of unified_theory_math.rs. Checked against native/.../vesper_vectors.txt.
(function (root) {
  'use strict';
  const D = (typeof module !== 'undefined') ? require('./vesper-data.js') : root.RiftData;
  const C = D.constants;
  const MASK = (1n << 64n) - 1n;
  const div = (a, b) => Math.trunc(a / b);
  const rem = (a, n) => ((a % n) + n) % n;
  const clamp = (v, lo, hi) => Math.min(hi, Math.max(lo, v));

  // xorshift64, exactly like vesper_math::next. `state` is { s: BigInt }.
  function next(state) {
    let x = state.s === 0n ? 1n : state.s;
    x ^= (x << 13n) & MASK;
    x ^= x >> 7n;
    x ^= (x << 17n) & MASK;
    state.s = x;
    return x;
  }
  function roll(state, n) {
    return Number(next(state) % BigInt(Math.max(1, n)));
  }
  function seedState(seed) {
    const s = BigInt.asUintN(64, BigInt(seed));
    return { s: s === 0n ? 1n : s };
  }
  function dist2(ax, ay, bx, by) {
    const dx = ax - bx, dy = ay - by;
    return dx * dx + dy * dy;
  }
  function isqrt(v) {
    if (v <= 0) return 0;
    let x = Math.floor(Math.sqrt(v));
    while (x * x > v) x--;
    while ((x + 1) * (x + 1) <= v) x++;
    return x;
  }
  const dist = (ax, ay, bx, by) => isqrt(dist2(ax, ay, bx, by));
  function keepBp(len) {
    const l = BigInt(Math.max(0, len));
    const s = BigInt(C.link_scale);
    const loss = l * l * BigInt(C.keep_quad * 100) / (s * s);
    let v = BigInt(C.keep_base * 100) - loss;
    if (v < BigInt(C.keep_min * 100)) v = BigInt(C.keep_min * 100);
    if (v > BigInt(C.keep_base * 100)) v = BigInt(C.keep_base * 100);
    return Number(v);
  }
  function chainBp(links) {
    return links.reduce((acc, l) => div(acc * keepBp(l), 10000), 10000);
  }
  function openDelay(distance, delayPct) {
    return Math.max(1, div((C.delay_base + div(Math.max(0, distance), C.delay_div)) * delayPct, 100));
  }
  function twistIndex(a, b) {
    if (a > b) [a, b] = [b, a];
    return 10 * a - div(a * Math.max(0, a - 1), 2) + (b - a);
  }
  class Atlas {
    constructor(seed) {
      this.seed = BigInt.asUintN(64, BigInt(seed));
      const st = { s: this.seed ^ 0x5645535045524154n };
      if (st.s === 0n) st.s = 1n;
      const base = [];
      for (let i = 0; i < 100; i++) base.push(i);
      for (let i = 99; i >= 1; i--) {
        const j = roll(st, i + 1);
        [base[i], base[j]] = [base[j], base[i]];
      }
      this.base = base;
      this._shift = new Map();
    }
    shift(epochs) {
      if (this._shift.has(epochs)) return this._shift.get(epochs);
      const st = { s: this.seed ^ 0x445249465400n };
      if (st.s === 0n) st.s = 1n;
      let sx = 0, sy = 0;
      for (let i = 0; i < epochs; i++) {
        switch (roll(st, 4)) {
          case 0: sy -= 1; break;
          case 1: sx += 1; break;
          case 2: sy += 1; break;
          default: sx -= 1;
        }
      }
      const out = [rem(sx, C.atlas_n), rem(sy, C.atlas_n)];
      this._shift.set(epochs, out);
      return out;
    }
    static epochs(tick) { return div(tick, C.drift_ticks); }
    static cell(x, y) {
      return [clamp(div(x, C.atlas_cell), 0, C.atlas_n - 1), clamp(div(y, C.atlas_cell), 0, C.atlas_n - 1)];
    }
    dimAtEpoch(x, y, epochs) {
      const [cx, cy] = Atlas.cell(x, y);
      const [sx, sy] = this.shift(epochs);
      const bx = rem(cx - sx, C.atlas_n), by = rem(cy - sy, C.atlas_n);
      return this.base[by * C.atlas_n + bx];
    }
    dimAt(x, y, tick) { return this.dimAtEpoch(x, y, Atlas.epochs(tick)); }
    centreOf(dim, tick) {
      const at = Math.max(0, this.base.indexOf(dim));
      const [sx, sy] = this.shift(Atlas.epochs(tick));
      const cx = rem(at % C.atlas_n + sx, C.atlas_n);
      const cy = rem(div(at, C.atlas_n) + sy, C.atlas_n);
      return [cx * C.atlas_cell + div(C.atlas_cell, 2), cy * C.atlas_cell + div(C.atlas_cell, 2)];
    }
  }
  const lerp = (ax, ay, bx, by, pct) => [ax + div((bx - ax) * pct, 100), ay + div((by - ay) * pct, 100)];
  function along(x, y, dx, dy, len) {
    const d = isqrt(dx * dx + dy * dy);
    if (d === 0) return [x, y];
    return [x + div(dx * len, d), y + div(dy * len, d)];
  }
  const clampMap = (x, y) => [clamp(x, 0, C.map), clamp(y, 0, C.map)];
  function segDist(ax, ay, bx, by, x, y) {
    const dx = BigInt(bx - ax), dy = BigInt(by - ay);
    const len = dx * dx + dy * dy;
    if (len === 0n) return dist(ax, ay, x, y);
    let t = BigInt(x - ax) * dx + BigInt(y - ay) * dy;
    if (t < 0n) t = 0n;
    if (t > len) t = len;
    return dist(x, y, ax + Number(dx * t / len), ay + Number(dy * t / len));
  }
  function side(ax, ay, bx, by, x, y) {
    const c = (bx - ax) * (y - ay) - (by - ay) * (x - ax);
    return Math.sign(c);
  }
  // Rolling cap (unified_theory_math::Window).
  class Window {
    constructor() { this.q = []; }
    allow(tick, span, cap, n) {
      while (this.q.length && Math.max(0, tick - this.q[0][0]) >= span) this.q.shift();
      const used = this.q.reduce((a, e) => a + e[1], 0);
      const granted = Math.min(n, Math.max(0, cap - used));
      if (granted > 0) this.q.push([tick, granted]);
      return granted;
    }
    clone() { const w = new Window(); w.q = this.q.map(e => e.slice()); return w; }
  }
  // Independently recovering charges (unified_theory_math::ChargeBank<N>). 'R' ready, or a recovery deadline.
  class ChargeBank {
    constructor(n) { this.slots = Array(n).fill('R'); }
    advance(tick) { this.slots = this.slots.map(s => (s !== 'R' && s <= tick) ? 'R' : s); }
    available() { return this.slots.filter(s => s === 'R').length; }
    spend(tick, n) {
      this.advance(tick);
      if (this.available() < n) return false;
      let left = n;
      this.slots = this.slots.map(s => { if (s === 'R' && left > 0) { left--; return tick + C.recharge; } return s; });
      return true;
    }
    cells(tick) {
      return this.slots.map(s => s === 'R' || s <= tick ? 10 : Math.min(9, div(Math.max(0, C.recharge - (s - tick)), 30)));
    }
    clone() { const c = new ChargeBank(this.slots.length); c.slots = this.slots.slice(); return c; }
  }
  const api = { div, rem, clamp, next, roll, seedState, dist2, isqrt, dist, keepBp, chainBp, openDelay, twistIndex, Atlas, lerp, along, clampMap, segDist, side, Window, ChargeBank };
  if (typeof module !== 'undefined') module.exports = api; else root.VesperMath = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
