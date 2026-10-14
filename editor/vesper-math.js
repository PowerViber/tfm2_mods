// Vesper's integer rules for the Rift Lab, ported 1:1 from native/tfm2_custom_ai/src/vesper_math.rs and the
// rolling windows of unified_theory_math.rs. Checked against native/.../vesper_vectors.txt.
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
    return D.families.length * a - div(a * Math.max(0, a - 1), 2) + (b - a);
  }
  // An athlete's Hexcore tuning: 6 of the 8 tech modules (vesper_math::modules_of).
  function modulesOf(athlete) {
    if (athlete === null || athlete === undefined) return [0, 1, 2, 3, 4, 5];
    let st = BigInt.asUintN(64, BigInt(athlete) * 0x9e3779b97f4a7c15n) ^ 0x484558434f524500n;
    const rng = { s: st === 0n ? 1n : st };
    const all = [0, 1, 2, 3, 4, 5, 6, 7];
    for (let i = 7; i >= 1; i--) { const j = roll(rng, i + 1); [all[i], all[j]] = [all[j], all[i]]; }
    return all.slice(0, 6);
  }
  // Active hexorb slots at a Hexcore completion (vesper_math::hex_slots).
  function hexSlots(completion) {
    let n = 0;
    for (const [at, k] of D.hexcore.slots) if (completion >= at) n = Math.max(n, k);
    return n;
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
  const api = { div, rem, clamp, next, roll, seedState, dist2, isqrt, dist, keepBp, chainBp, openDelay, twistIndex, modulesOf, hexSlots, lerp, along, clampMap, segDist, side, Window };
  if (typeof module !== 'undefined') module.exports = api; else root.VesperMath = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
