/* Levi's flight lab (Skill Test → Flight lab): his cable AI, mirrored from native/tfm2_custom_ai/src/levi.rs, flown
 * by every mastery rank on the same map and the same route, side by side.
 *
 *   - The map: a built-in course or the game's own (map_dump.json, through the editor server), on the native wall
 *     grid (30 x 30 cells of 32000). Shift+click a cell to add or remove a wall.
 *   - The route: he walks it like the game's AI would (a path around the walls), and his brain decides when a flight
 *     is worth it, exactly like in a match: the way he has walked for the last half second is where he wants to go,
 *     and each press of S1 (every 13 ticks) / S2 asks it whether to fire a cable / use gas. Click to add a waypoint,
 *     right-click removes the last one, Alt+click moves home (where the gas refills).
 *   - Ghost race: one flight per rank with the same seed, drawn as paths; slams are ✕, missed cables ○.
 *   - Focus a rank to see it with his sprite and the in-game effects (cables, trails, forms, HUD); "Follow" zooms in.
 *   - The table averages many flights per rank (different seeds): how long the route takes, cables, misses, slams.
 *
 * Keep the numbers in step with levi.rs (the constants below and the functions marked "mirror").
 */
(function () {
  'use strict';
  const UPX = 950, CELL = 32000, N = 30, W = CELL * N, TPS = 60;
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

  // ------------------------------------------------------------------ mirror of levi.rs
  const RANKS = ['Grounded', 'Tethered', 'Swinger', 'Glider', 'Skyrunner', 'Stormcutter', 'Comet', 'Apex'];
  const RANK_GAMES = ['0+ games', '5+', '15+', '30+', '60+', '100+', '150+', 'Top 10, 300+ points'];
  const RANK_COL = ['#9aa7b4', '#7fd1a8', '#5ec8e8', '#6f8dff', '#b583ff', '#3fc4ff', '#fff1a8', '#ff7ad9'];
  const APEX = 7;
  const BASE_SPEED = [2200, 2500, 2800, 3100, 3400, 3700, 4000, 4300], APEX_TOP_SPEED = 4800;
  const MISAIM = [25, 16, 10, 6, 3, 1, 0, 0];
  const RECOVER = [60, 45, 36, 27, 18, 12, 9, 8];
  const LOOKAHEAD = [0, 6, 10, 14, 18, 24, 30, 30];
  const CABLE_RANGE = 90000, CABLE_MIN = 12000, FLIGHT_T = 90, TURN = 0.15, ANGLE_FLOOR = 0.25, GAIN = [250, 550];
  const SPEED_CEIL = 30000, STEP = 6000, CRASH_SPEED = 3500, LAND_R = 9000, FAST = 4500;
  const GAS_MAX = 100, BOOST_COST = 8, BOOST_ADD = 600, DASH_COST = 10, DASH_CD = 180, DASH_T = 10, DASH_SPEED = 4000, HOME_R = 45000;
  // the lab's stand-ins for the game: his walk speed (data move_speed), how often the AI presses a 12-tick skill,
  // and how close counts as reaching a waypoint
  const WALK = 1050, PRESS_EVERY = 13, WP_R = 30000;
  // what each rank does differently (levi.rs: pick_anchor, on_cable_press, fly, on_gas_press)
  const PLAYS = [
    'Grabs the wall nearest the way he is going; fires whenever the button is up (no timing); gas at random',
    'Times the next cable 6 ticks out; skips walls too close at speed; escapes on cables',
    'Aims for good cable angles; chases on cables; 10-tick timing',
    'Right angles count for more; gas whenever he is slow; 14-tick timing',
    'Plans two cables ahead (a wall to carry on from); 18-tick timing',
    'Reads a slam coming (avoids it, brakes before it); 24-tick timing',
    'The same reads with a faster start; 30-tick timing',
    'The fastest start (#10 4300 to #1 4800); never misaims',
  ];

  const deg = a => a * Math.PI / 180;
  function wrap(a) { a %= 2 * Math.PI; if (a > Math.PI) a -= 2 * Math.PI; if (a < -Math.PI) a += 2 * Math.PI; return a; }
  const angTo = (ax, ay, bx, by) => Math.atan2(by - ay, bx - ax);
  const angleQuality = (a, b) => Math.max(1 - Math.abs(Math.abs(wrap(b - a)) * 180 / Math.PI - 90) / 90, ANGLE_FLOOR);
  function baseSpeed(r, apex) {
    if (r >= APEX) { const p = Math.min(10, Math.max(1, apex || 10)); return APEX_TOP_SPEED - (p - 1) * (APEX_TOP_SPEED - BASE_SPEED[APEX]) / 9; }
    return BASE_SPEED[Math.min(r, APEX)];
  }

  // the wall grid (mirror of lib.rs walls::) and the cable raycast
  function makeWorld(cells, towers) {
    const wallAt = (x, y) => {
      x = Math.trunc(x); y = Math.trunc(y);
      if (x < 0 || y < 0) return true;
      const cx = Math.floor(x / CELL), cy = Math.floor(y / CELL);
      if (cx >= N || cy >= N) return true;
      return cells[cy * N + cx] === 1;
    };
    function raycast(x, y, a) {
      const c = Math.cos(a), s = Math.sin(a);
      let hit = null, t = 3000, last = [Math.trunc(x), Math.trunc(y)];
      while (t <= CABLE_RANGE) {
        const px = Math.trunc(x + c * t), py = Math.trunc(y + s * t);
        if (wallAt(px, py)) { if (t >= CABLE_MIN) hit = [last[0], last[1], t]; break; }
        last = [px, py]; t += 3000;
      }
      const limit = hit ? hit[2] : CABLE_RANGE;
      let best = null;
      for (const [tx, ty] of towers) {
        const dx = tx - x, dy = ty - y, along = dx * c + dy * s, off = Math.abs(dx * s - dy * c);
        if (along >= CABLE_MIN && along < limit && off <= 7000 && (!best || along < best[2])) best = [tx, ty, along];
      }
      const h = best || hit;
      return h ? [h[0], h[1]] : null;
    }
    function clip(x0, y0, x1, y1) {
      const len = Math.trunc(Math.hypot(x1 - x0, y1 - y0)), steps = Math.max(1, Math.trunc(len / 3000));
      let lx = x0, ly = y0;
      for (let i = 1; i <= steps; i++) {
        const x = x0 + (x1 - x0) * i / steps, y = y0 + (y1 - y0) * i / steps;
        if (wallAt(x, y)) return [lx, ly];
        lx = x; ly = y;
      }
      return [x1, y1];
    }
    function pullBack(x0, y0, x1, y1) {
      const len = Math.trunc(Math.hypot(x1 - x0, y1 - y0)), steps = Math.max(1, Math.trunc(len / 3000));
      for (let i = 0; i <= steps; i++) {
        const x = x1 + (x0 - x1) * i / steps, y = y1 + (y0 - y1) * i / steps;
        if (!wallAt(x, y)) return [x, y];
      }
      return [x0, y0];
    }
    return { cells, towers, wallAt, raycast, clip, pullBack };
  }

  // walking: a distance field per waypoint over the free cells (8-neighbour, no corner cutting)
  const NB = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]];
  function field(cells, px, py) {
    const d = new Float32Array(N * N).fill(Infinity);
    const sx = Math.min(N - 1, Math.max(0, Math.floor(px / CELL))), sy = Math.min(N - 1, Math.max(0, Math.floor(py / CELL)));
    d[sy * N + sx] = 0;
    const q = [[sx, sy]];
    for (let i = 0; i < q.length; i++) {
      const [x, y] = q[i], here = d[y * N + x];
      for (const [dx, dy] of NB) {
        const nx = x + dx, ny = y + dy;
        if (nx < 0 || ny < 0 || nx >= N || ny >= N || cells[ny * N + nx]) continue;
        if (dx && dy && (cells[y * N + nx] || cells[ny * N + x])) continue;
        const nd = here + (dx && dy ? 1.414 : 1);
        if (nd < d[ny * N + nx]) { d[ny * N + nx] = nd; q.push([nx, ny]); }
      }
    }
    return d;
  }

  /** One flight session of one rank: Levi walks the route, his brain flies him. cfg: { world, fields, home, route,
   *  rank, apex, seed, ticks, record }. Returns the stats and, with record, every tick's state and the effects. */
  function simulate(cfg) {
    const { world, home, route, rank: r, apex } = cfg;
    let rng = ((cfg.seed + 1) * 2654435761) >>> 0 || 1;
    const next = () => { rng ^= rng << 13; rng >>>= 0; rng ^= rng >>> 17; rng ^= rng << 5; rng >>>= 0; return rng; };
    const roll = pct => next() % 100 < pct;
    const unit = () => (next() % 10000) / 10000;
    const S = { x: home[0], y: home[1], flying: false, heading: 0, speed: 0, cables: [], chain: 0, flightUntil: 0, gliding: false,
      recoverUntil: 0, lastCable: 0, goal: null, gas: GAS_MAX, dashCd: 0, dash: null, stunUntil: 0, track: [], form: 0, lastTrail: null,
      wp: 0, face: 1, moved: false };
    const st = { cables: 0, misses: 0, misaims: 0, slams: 0, stun: 0, top: 0, air: 0, dist: 0, finish: null, boosts: 0, dashes: 0, chainMax: 0 };
    const T = cfg.ticks;
    const rec = cfg.record ? { xs: new Float32Array(T), ys: new Float32Array(T), sp: new Float32Array(T), flags: new Uint8Array(T),
      chain: new Uint8Array(T), gas: new Uint8Array(T), form: new Uint8Array(T), face: new Int8Array(T), wp: new Uint8Array(T),
      fx: [], slams: [], misses: [], len: T } : null;
    const fx = (tag, x, y, t, follow) => { if (rec) rec.fx.push({ t, tag, x, y, follow: !!follow }); };
    const targets = route.concat([home]);   // the route, then back home

    function want() {
      // levi.rs want(): no enemies here, so it's always the way he has been going
      const tr = S.track; if (!tr.length) return null;
      const now = tr[tr.length - 1], then = tr.find(p => p[0] + 30 >= now[0]);
      const dx = now[1] - then[1], dy = now[2] - then[2], walked = Math.hypot(dx, dy);
      if (walked < 4000) return null;
      return { a: Math.atan2(dy, dx), go: walked >= 20000 };
    }
    function pickAnchor(wantA) {   // mirror
      const older = S.cables.length ? (c => angTo(S.x, S.y, c[0], c[1]))(S.cables[S.cables.length - 1]) : null;
      let best = null;
      for (let k = -9; k <= 9; k++) {
        const a = wantA + deg(10 * k);
        const p = world.raycast(S.x, S.y, a); if (!p) continue;
        const dist = Math.max(1, Math.hypot(p[0] - S.x, p[1] - S.y));
        let score = -Math.abs(k) * 10;
        if (r >= 1 && S.flying && dist < S.speed * 8) score -= 40;
        if (r >= 2 && older != null) score += angleQuality(older, a) * (r >= 3 ? 50 : 30);
        if (r >= 4) {
          const mx = S.x + (p[0] - S.x) * 0.7, my = S.y + (p[1] - S.y) * 0.7;
          if ([-0.6, 0, 0.6].some(o => world.raycast(mx, my, wantA + o))) score += 15;
        }
        if (r >= 5 && S.flying && dist < S.speed * 10) score -= 80;
        if (!best || score > best.score) best = { score, a, p };
      }
      return best;
    }
    function endFlight() { S.flying = false; S.gliding = false; S.cables = []; S.chain = 0; S.speed = 0; }
    function connect(p, t) {   // mirror
      const aNew = angTo(S.x, S.y, p[0], p[1]);
      if (!S.flying) {
        S.flying = true; S.heading = aNew; S.speed = baseSpeed(r, apex); S.chain = 1; S.cables = [p];
      } else {
        const a = S.cables[S.cables.length - 1];
        if (a) S.speed += GAIN[0] + GAIN[1] * angleQuality(angTo(S.x, S.y, a[0], a[1]), aNew);
        S.cables.push(p); while (S.cables.length > 2) S.cables.shift();
        S.chain++;
      }
      S.speed = Math.min(S.speed, SPEED_CEIL);
      S.gliding = false; S.flightUntil = t + FLIGHT_T; S.lastCable = t;
      st.cables++; st.chainMax = Math.max(st.chainMax, S.chain);
      fx('hook', p[0], p[1], t);
      if (S.speed >= FAST) {
        if (r === 5) fx('storm_hook', p[0], p[1], t);
        else if (r === 6) fx('starburst', S.x, S.y, t);
        else if (r === APEX) fx('apex_ring', S.x, S.y, t);
      }
    }
    function whiff(a, t) {
      const wx = S.x + Math.cos(a) * CABLE_RANGE * 0.8, wy = S.y + Math.sin(a) * CABLE_RANGE * 0.8;
      fx('whiff', wx, wy, t); st.misses++; if (rec) rec.misses.push({ t, x: wx, y: wy });
      if (S.flying) { S.gliding = true; S.cables = []; S.chain = 0; }
      S.recoverUntil = t + RECOVER[r];
    }
    function crash(t) {
      fx('crash', S.x, S.y, t); st.slams++; if (rec) rec.slams.push({ t, x: S.x, y: S.y, speed: S.speed });
      const stun = Math.trunc(Math.min(120, Math.max(30, 30 + (S.speed - CRASH_SPEED) / 60)));
      S.stunUntil = t + stun; st.stun += stun;
      endFlight();
      S.recoverUntil = t + RECOVER[r];
    }
    function onCablePress(t) {   // mirror
      if (t < S.recoverUntil || S.dash) return;
      const fresh = want();
      if (S.flying) {
        const c = S.cables[S.cables.length - 1];
        const tta = c ? Math.hypot(c[0] - S.x, c[1] - S.y) / Math.max(1, S.speed) : 0;
        const fire = r === 0 ? t >= S.lastCable + 18 : (S.gliding || tta <= LOOKAHEAD[r] || t >= S.lastCable + 40);
        if (!fire) return;
        if (fresh && r >= 2) S.goal = fresh.a;
      } else {
        if (!fresh || !fresh.go) return;
        S.goal = fresh.a;
      }
      if (S.goal == null) return;
      const picked = pickAnchor(S.goal);
      if (picked && roll(MISAIM[r])) {
        st.misaims++;
        const off = deg(10 + 30 * unit()) * (roll(50) ? 1 : -1);
        const q = world.raycast(S.x, S.y, picked.a + off);
        if (q) connect(q, t); else whiff(picked.a + off, t);
      } else if (picked) connect(picked.p, t);
      else whiff(S.goal, t);
    }
    function onGasPress(t) {   // mirror (no enemies: "urge" is whether he wants to go somewhere)
      const w = want();
      if (S.flying && !S.gliding) {
        if (S.gas < BOOST_COST) return;
        const urge = !!(w && w.go);
        const use = r === 0 ? roll(50) : r <= 2 ? urge : (urge || S.speed < baseSpeed(r, apex) * 1.5);
        if (!use) return;
        S.gas -= BOOST_COST; S.speed = Math.min(SPEED_CEIL, S.speed + BOOST_ADD); st.boosts++;
        fx('dash_gas', S.x, S.y, t);
      } else if (!S.flying && t >= S.dashCd && S.gas >= DASH_COST) {
        const go = r === 0 ? false : !!(w && w.go);   // Grounded only dashes with a foe near (none here)
        if (!go) return;
        S.gas -= DASH_COST; S.dashCd = t + DASH_CD; S.dash = { a: w.a, until: t + DASH_T }; st.dashes++;
        fx('dash_gas', S.x, S.y, t);
      }
    }
    function fly(t) {   // mirror
      if (!S.gliding && t >= S.flightUntil) { S.gliding = true; S.cables = []; }
      if (S.gliding) {
        S.speed *= 0.9;
        if (S.speed < 1400) { endFlight(); return; }
      } else if (S.cables.length) {
        const b = S.cables[S.cables.length - 1];
        const tb = angTo(S.x, S.y, b[0], b[1]);
        let target = tb;
        if (S.cables.length === 2) { const ta = angTo(S.x, S.y, S.cables[0][0], S.cables[0][1]); target = Math.atan2(0.7 * Math.sin(tb) + 0.3 * Math.sin(ta), 0.7 * Math.cos(tb) + 0.3 * Math.cos(ta)); }
        const d = wrap(target - S.heading);
        S.heading = wrap(S.heading + Math.max(-TURN, Math.min(TURN, d)));
        const tta = Math.hypot(b[0] - S.x, b[1] - S.y) / Math.max(1, S.speed);
        if (r >= 5 && tta < 6 && S.speed >= CRASH_SPEED && t < Math.max(S.recoverUntil, S.lastCable + 12)) S.speed = Math.max(S.speed * 0.85, CRASH_SPEED - 100);
      }
      const x0 = S.x, y0 = S.y;
      const n = Math.max(1, Math.ceil(S.speed / STEP)), per = S.speed / n;
      const c = Math.cos(S.heading), s = Math.sin(S.heading);
      let x = x0, y = y0, stop = null;
      for (let i = 0; i < n; i++) {
        const nx = x + c * per, ny = y + s * per;
        const b = S.cables[S.cables.length - 1];
        if (b && (nx - b[0]) ** 2 + (ny - b[1]) ** 2 <= LAND_R * LAND_R) { x = nx; y = ny; stop = S.speed >= CRASH_SPEED; break; }
        if (world.wallAt(nx, ny)) { stop = S.speed >= CRASH_SPEED; break; }
        x = nx; y = ny;
      }
      const p = world.pullBack(Math.trunc(x0), Math.trunc(y0), Math.trunc(x), Math.trunc(y));
      S.x = p[0]; S.y = p[1];
      if (stop === true) crash(t); else if (stop === false) endFlight();
    }
    function dashStep(t) {
      const d = S.dash; if (t >= d.until) { S.dash = null; return; }
      const tx = S.x + Math.cos(d.a) * DASH_SPEED, ty = S.y + Math.sin(d.a) * DASH_SPEED;
      const p = world.clip(S.x, S.y, tx, ty);
      if (Math.trunc(p[0]) !== Math.trunc(tx) || Math.trunc(p[1]) !== Math.trunc(ty)) S.dash = null;
      S.x = p[0]; S.y = p[1];
    }
    function walk() {
      const tg = targets[S.wp], f = cfg.fields[S.wp];
      let to = tg;
      const direct = world.clip(S.x, S.y, tg[0], tg[1]);
      if (Math.hypot(direct[0] - tg[0], direct[1] - tg[1]) > 1) {
        const cx = Math.floor(S.x / CELL), cy = Math.floor(S.y / CELL);
        let best = null;
        for (const [dx, dy] of NB) {
          const nx = cx + dx, ny = cy + dy;
          if (nx < 0 || ny < 0 || nx >= N || ny >= N || world.cells[ny * N + nx]) continue;
          if (dx && dy && (world.cells[cy * N + nx] || world.cells[ny * N + cx])) continue;
          if (!best || f[ny * N + nx] < best[0]) best = [f[ny * N + nx], nx, ny];
        }
        if (best) to = [(best[1] + 0.5) * CELL, (best[2] + 0.5) * CELL];
      }
      const dist = Math.hypot(to[0] - S.x, to[1] - S.y); if (dist < 1) return;
      const k = Math.min(1, WALK / dist);
      const p = world.clip(S.x, S.y, S.x + (to[0] - S.x) * k, S.y + (to[1] - S.y) * k);
      if (p[0] === S.x && p[1] === S.y) { const c = [(Math.floor(S.x / CELL) + 0.5) * CELL, (Math.floor(S.y / CELL) + 0.5) * CELL]; const kk = Math.min(1, WALK / Math.max(1, Math.hypot(c[0] - S.x, c[1] - S.y))); S.x += (c[0] - S.x) * kk; S.y += (c[1] - S.y) * kk; }
      else { S.x = p[0]; S.y = p[1]; }
    }
    function tier() {
      if (!S.flying || S.speed < CRASH_SPEED) return 0;
      if (S.speed < FAST) return 1;
      return ({ 5: 2, 6: 3, 7: 4 })[r] || 1;
    }
    function visuals(t) {   // mirror of visuals(): the effects the game is asked to play
      const tr = tier();
      const form = tr >= 2 ? tr : 0;
      if (form !== S.form) { if (form >= 2 && S.form < 2) fx('ignite' + form, S.x, S.y, t, true); S.form = form; }
      if (!S.flying) S.lastTrail = null;
      if (S.flying && t % 2 === 0) {
        for (const [ax, ay] of S.cables) {
          const a = ((angTo(S.x, S.y, ax, ay) * 180 / Math.PI) % 180 + 180) % 180;
          const d = Math.round(a / 11.25) % 16;
          const b = Math.min(6, Math.max(1, Math.round(Math.hypot(ax - S.x, ay - S.y) / 950 / 16)));
          fx(`cable_${d}_${b}`, Math.trunc((S.x + ax) / 2), Math.trunc((S.y + ay) / 2), t);
        }
        const d = Math.round(((S.heading * 180 / Math.PI) % 360 + 360) % 360 / 22.5) % 16;
        const tag = `trail${tr}_${d}`;
        if (S.lastTrail) {
          const [lx, ly] = S.lastTrail, gap = Math.hypot(S.x - lx, S.y - ly) / 950;
          const extra = Math.min(3, Math.max(0, Math.ceil(gap / 18) - 1));
          for (let i = 1; i <= extra; i++) { const k = i / (extra + 1); fx(tag, lx + (S.x - lx) * k, ly + (S.y - ly) * k, t); }
        }
        fx(tag, S.x, S.y, t);
        S.lastTrail = [S.x, S.y];
        if (S.chain >= 8 && t % 4 === 0 && S.track.length >= 7) {
          const p = S.track[S.track.length - 7], side = Math.cos(S.heading) < 0 ? 'l' : 'r';
          fx(tr >= 2 ? `after${tr}_${side}` : `after_${side}`, p[1], p[2], t);
        }
      }
    }

    let t = 0;
    for (; t < T; t++) {
      const x0 = S.x, y0 = S.y;
      const stunned = t < S.stunUntil;
      if (Math.hypot(S.x - home[0], S.y - home[1]) <= HOME_R) S.gas = GAS_MAX;
      S.track.push([t, S.x, S.y]); if (S.track.length > 40) S.track.shift();
      if (!stunned && !cfg.walkOnly && t % PRESS_EVERY === 0) onCablePress(t);
      if (!stunned && !cfg.walkOnly && t % PRESS_EVERY === 6) onGasPress(t);
      if (S.flying) fly(t);
      else if (S.dash) dashStep(t);
      else if (!stunned) walk();
      visuals(t);
      if (S.flying) { st.air++; st.top = Math.max(st.top, S.speed); }
      const mv = Math.hypot(S.x - x0, S.y - y0); st.dist += mv;
      if (mv > 1) S.face = S.x >= x0 ? 1 : -1;
      if (Math.hypot(S.x - targets[S.wp][0], S.y - targets[S.wp][1]) <= WP_R) {
        S.wp++;
        if (S.wp >= targets.length) { st.finish = t; }
      }
      if (rec) {
        rec.xs[t] = S.x; rec.ys[t] = S.y; rec.sp[t] = S.flying ? S.speed : 0;
        rec.flags[t] = (S.flying ? 1 : 0) | (t < S.stunUntil ? 2 : 0) | (mv > 1 ? 4 : 0) | (S.gliding ? 8 : 0) | (S.dash ? 16 : 0);
        rec.chain[t] = S.flying ? Math.min(8, S.chain) : 0; rec.gas[t] = Math.max(0, S.gas); rec.form[t] = S.form; rec.face[t] = S.face; rec.wp[t] = Math.min(S.wp, 255);
      }
      if (st.finish != null) break;
    }
    if (rec) rec.len = Math.min(T, t + 1);
    st.ticks = Math.min(T, t + 1);
    return { stats: st, rec };
  }

  // ------------------------------------------------------------------ maps
  function blank() { const c = new Uint8Array(N * N); for (let i = 0; i < N; i++) { c[i] = 1; c[(N - 1) * N + i] = 1; c[i * N] = 1; c[i * N + N - 1] = 1; } return c; }
  const fill = (c, x0, y0, x1, y1) => { for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) if (x >= 0 && y >= 0 && x < N && y < N) c[y * N + x] = 1; };
  const P = (cx, cy) => [Math.round((cx + 0.5) * CELL), Math.round((cy + 0.5) * CELL)];
  const MAPS = {
    jungle: { label: 'Jungle (lanes and camps)', make() {
      const c = blank();
      // the blue side's blocks (below the diagonal); the red side is the mirror (x <-> y), like the 5v5 map
      const blocks = [[5, 7, 8, 9], [5, 12, 6, 16], [8, 13, 11, 14], [5, 20, 7, 23], [10, 18, 13, 20], [9, 24, 13, 25], [15, 22, 18, 24],
        [19, 25, 22, 25], [15, 18, 16, 19], [11, 16, 12, 16]];
      for (const [x0, y0, x1, y1] of blocks) { fill(c, x0, y0, x1, y1); fill(c, y0, x0, y1, x1); }
      return { cells: c, home: P(2, 27), route: [P(14, 14), P(27, 2)] };
    } },
    pillars: { label: 'Pillar field', make() {
      const c = blank();
      for (let y = 4; y < 27; y += 6) for (let x = 4; x < 27; x += 6) fill(c, x, y, x + 1, y + 1);
      return { cells: c, home: P(2, 27), route: [P(27, 2), P(27, 27)] };
    } },
    corridors: { label: 'Corridors (S-bends)', make() {
      const c = blank();
      fill(c, 1, 23, 23, 23); fill(c, 6, 17, 28, 17); fill(c, 1, 11, 23, 11); fill(c, 6, 5, 28, 5);
      return { cells: c, home: P(2, 27), route: [P(27, 20), P(2, 14), P(27, 8), P(2, 2)] };
    } },
    open: { label: 'Open field (only the edge)', make() {
      return { cells: blank(), home: P(2, 27), route: [P(27, 2)] };
    } },
  };

  // ------------------------------------------------------------------ state
  const L = {
    el: null, mounted: false, canvas: null, ctx: null, mapKey: 'jungle', cells: null, towers: [], home: null, route: [],
    game: null, shown: [true, true, true, true, true, true, true, true], focus: 5, follow: false, apex: 1, seed: 1, limit: 90,
    runs: 20, results: null, recs: null, t: 0, playing: true, speed: 1, last: 0, acc: 0, raf: 0, job: 0, sheets: null, dirty: false,
  };
  const SAVE_KEY = 'tfm2.levilab';
  function save() {
    try { localStorage.setItem(SAVE_KEY, JSON.stringify({ mapKey: L.mapKey, cells: Array.from(L.cells), home: L.home, route: L.route, shown: L.shown,
      focus: L.focus, follow: L.follow, apex: L.apex, seed: L.seed, limit: L.limit, runs: L.runs })); } catch (e) { /* private window */ }
  }
  function load() {
    try {
      const s = JSON.parse(localStorage.getItem(SAVE_KEY) || 'null'); if (!s) return false;
      Object.assign(L, { mapKey: s.mapKey, home: s.home, route: s.route, shown: s.shown, focus: s.focus, follow: s.follow, apex: s.apex, seed: s.seed, limit: s.limit, runs: s.runs });
      if (Array.isArray(s.cells) && s.cells.length === N * N) L.cells = Uint8Array.from(s.cells);
      return !!L.cells;
    } catch (e) { return false; }
  }
  function useMap(key) {
    L.mapKey = key;
    if (key === 'game' && L.game) { L.cells = L.game.cells.slice(); L.towers = L.game.towers; L.home = L.game.home; L.route = L.game.route.slice(); }
    else { const m = (MAPS[key] || MAPS.jungle).make(); L.cells = m.cells; L.towers = []; L.home = m.home; L.route = m.route; }
  }
  async function loadGameMap() {
    try {
      const j = await (await fetch('/api/mapdump', { cache: 'no-store' })).json();
      const g = j.doc && j.doc.walls; if (!Array.isArray(g) || g.length < N || !Array.isArray(g[0])) return;
      const pts = (v, out = []) => { if (Array.isArray(v) && v.length === 2 && typeof v[0] === 'number' && typeof v[1] === 'number') out.push(v); else if (Array.isArray(v)) v.forEach(x => pts(x, out)); else if (v && typeof v === 'object') Object.values(v).forEach(x => pts(x, out)); return out; };
      const towers = pts(j.doc.towers || []).map(p => [p[0], p[1]]);
      // rows = y unless that puts towers inside walls (the native WallReader decides the same way)
      const at = (x, y, ry) => { const cx = Math.min(N - 1, Math.max(0, Math.floor(x / CELL))), cy = Math.min(N - 1, Math.max(0, Math.floor(y / CELL))); return ry ? +g[cy][cx] : +g[cx][cy]; };
      const ry = towers.filter(p => at(p[0], p[1], true)).length <= towers.filter(p => at(p[0], p[1], false)).length;
      const cells = new Uint8Array(N * N);
      for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) cells[y * N + x] = (ry ? +g[y][x] : +g[x][y]) ? 1 : 0;
      const free = p => { // the nearest free cell centre
        let best = null;
        for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) if (!cells[y * N + x]) { const c = P(x, y), d = Math.hypot(c[0] - p[0], c[1] - p[1]); if (!best || d < best[0]) best = [d, c]; }
        return best ? best[1] : p;
      };
      const nx = pts(j.doc.nexus_pos || []);
      const home = free(nx[0] || [96000, 864000]), far = free(nx[1] || [864000, 96000]);
      L.game = { cells, towers: j.source === 'dump' ? towers : [], home, route: [free([480000, 480000]), far], source: j.source };
    } catch (e) { /* no server or no map yet */ }
  }

  // ------------------------------------------------------------------ simulation runs
  function world() { return makeWorld(L.cells, L.towers); }
  function plan() {
    const w = world(), targets = L.route.concat([L.home]);
    return { world: w, home: L.home, route: L.route, fields: targets.map(p => field(L.cells, p[0], p[1])), ticks: L.limit * TPS };
  }
  function resimulate() {
    const job = ++L.job;
    const pl = plan();
    L.recs = RANKS.map((_, r) => simulate(Object.assign({}, pl, { rank: r, apex: L.apex, seed: L.seed, record: true })));
    L.t = 0; L.results = null; L.playing = true;
    L.walk = simulate(Object.assign({}, pl, { rank: 0, seed: 0, record: false, walkOnly: true })).stats;
    renderSide();
    // the averages, a rank at a time so the page stays responsive
    const acc = RANKS.map(() => []);
    let r = 0;
    const stepJob = () => {
      if (job !== L.job) return;
      for (let i = 0; i < L.runs; i++) acc[r].push(simulate(Object.assign({}, pl, { rank: r, apex: L.apex, seed: 1000 + i * 7919, record: false })).stats);
      r++;
      if (r < RANKS.length) { L.results = acc.slice(0, r); renderTable(); setTimeout(stepJob, 0); }
      else { L.results = acc; renderTable(); }
    };
    setTimeout(stepJob, 0);
  }
  const avg = (a, f) => a.length ? a.reduce((s, x) => s + f(x), 0) / a.length : 0;

  // ------------------------------------------------------------------ assets (the bundled sprite and VFX sheets)
  const img = src => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = rej; i.src = src; });
  const anims = fanim => { const o = {}; for (const [k, a] of Object.entries((fanim && fanim.anims) || {})) o[k] = a.frames.map(f => Object.assign({ ticks: Math.max(1, f.duration * TPS) }, f.data)); return o; };
  async function loadSheets() {
    if (L.sheets) return;
    const v = window.TFM2_VFX && window.TFM2_VFX.levi, b = window.TFM2_SPRITES && window.TFM2_SPRITES.levi;
    L.sheets = {};
    try { if (v) L.sheets.vfx = { img: await img('data:image/png;base64,' + v.png), anims: anims(v.fanim) }; } catch (e) { /* none */ }
    try { if (b) L.sheets.body = { img: await img('data:image/png;base64,' + b.png), anims: anims(b.fanim) }; } catch (e) { /* none */ }
  }
  function frameOf(frames, age, loop) {
    if (!frames || !frames.length) return null;
    const total = frames.reduce((a, f) => a + f.ticks, 0);
    if (loop) age = ((age % total) + total) % total; else if (age >= total || age < 0) return null;
    for (const f of frames) { if (age < f.ticks) return f; age -= f.ticks; }
    return frames[frames.length - 1];
  }

  // ------------------------------------------------------------------ drawing
  const view = { scale: 1, ox: 0, oy: 0 };
  const vx = x => (x - view.ox) * view.scale, vy = y => (y - view.oy) * view.scale;
  function blit(sh, f, x, y, flip, alpha) {
    if (!sh || !f) return;
    const c = L.ctx, k = view.scale * UPX;
    c.save(); c.globalAlpha = alpha == null ? 1 : alpha; c.translate(Math.round(vx(x)), Math.round(vy(y)));
    if (flip) c.scale(-1, 1);
    c.drawImage(sh.img, f.x, f.y, f.w, f.h, -f.w * k / 2, -f.h * k / 2, f.w * k, f.h * k);
    c.restore();
  }
  function setView() {
    const cv = L.canvas, rec = L.recs && L.focus >= 0 && L.recs[L.focus] && L.recs[L.focus].rec;
    if (L.follow && rec) {
      view.scale = 3 / UPX;
      const t = Math.min(L.t, rec.len - 1), span = cv.width / view.scale;
      view.ox = Math.min(W - span, Math.max(0, rec.xs[t] - span / 2)); view.oy = Math.min(W - span, Math.max(0, rec.ys[t] - span / 2));
    } else { view.scale = cv.width / W; view.ox = 0; view.oy = 0; }
  }
  function draw() {
    const c = L.ctx, cv = L.canvas; if (!c || !L.cells) return;
    setView();
    c.imageSmoothingEnabled = false;
    c.fillStyle = '#3d5a45'; c.fillRect(0, 0, cv.width, cv.height);
    const cs = CELL * view.scale;
    for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) {
      if (!L.cells[y * N + x]) continue;
      c.fillStyle = '#1a1f24'; c.fillRect(Math.floor(vx(x * CELL)), Math.floor(vy(y * CELL)), Math.ceil(cs), Math.ceil(cs));
    }
    c.strokeStyle = 'rgba(255,255,255,.05)'; c.lineWidth = 1;
    for (let i = 0; i <= N; i++) { c.beginPath(); c.moveTo(vx(i * CELL), vy(0)); c.lineTo(vx(i * CELL), vy(W)); c.stroke(); c.beginPath(); c.moveTo(vx(0), vy(i * CELL)); c.lineTo(vx(W), vy(i * CELL)); c.stroke(); }
    for (const [tx, ty] of L.towers) { c.fillStyle = '#c9a94a'; c.fillRect(vx(tx) - 4, vy(ty) - 4, 8, 8); }
    // home and the route
    c.setLineDash([5, 4]); c.strokeStyle = 'rgba(120,230,255,.7)';
    c.beginPath(); c.arc(vx(L.home[0]), vy(L.home[1]), HOME_R * view.scale, 0, Math.PI * 2); c.stroke();
    c.strokeStyle = 'rgba(255,255,255,.35)'; c.beginPath(); c.moveTo(vx(L.home[0]), vy(L.home[1]));
    for (const p of L.route) c.lineTo(vx(p[0]), vy(p[1]));
    c.lineTo(vx(L.home[0]), vy(L.home[1])); c.stroke(); c.setLineDash([]);
    c.font = 'bold 11px system-ui'; c.textAlign = 'center'; c.textBaseline = 'middle';
    L.route.forEach((p, i) => {
      c.fillStyle = 'rgba(0,0,0,.55)'; c.beginPath(); c.arc(vx(p[0]), vy(p[1]), 9, 0, Math.PI * 2); c.fill();
      c.strokeStyle = '#fff'; c.beginPath(); c.arc(vx(p[0]), vy(p[1]), WP_R * view.scale, 0, Math.PI * 2); c.stroke();
      c.fillStyle = '#fff'; c.fillText(String(i + 1), vx(p[0]), vy(p[1]));
    });
    c.fillStyle = '#7fe3ff'; c.fillText('home', vx(L.home[0]), vy(L.home[1]) - HOME_R * view.scale - 8);
    if (!L.recs) return;
    // every shown rank's path so far (the focused one last, on top)
    const order = RANKS.map((_, r) => r).filter(r => L.shown[r] || r === L.focus).sort((a, b) => (a === L.focus) - (b === L.focus));
    for (const r of order) {
      const rec = L.recs[r].rec, end = Math.min(L.t, rec.len - 1);
      c.strokeStyle = RANK_COL[r]; c.globalAlpha = r === L.focus ? 0.95 : 0.7; c.lineWidth = r === L.focus ? 2.5 : 1.6;
      c.beginPath(); c.moveTo(vx(rec.xs[0]), vy(rec.ys[0]));
      for (let t = 1; t <= end; t += 2) c.lineTo(vx(rec.xs[t]), vy(rec.ys[t]));
      c.lineTo(vx(rec.xs[end]), vy(rec.ys[end])); c.stroke();
      c.lineWidth = 2;
      for (const s of rec.slams) if (s.t <= end) { const x = vx(s.x), y = vy(s.y); c.beginPath(); c.moveTo(x - 5, y - 5); c.lineTo(x + 5, y + 5); c.moveTo(x + 5, y - 5); c.lineTo(x - 5, y + 5); c.stroke(); }
      c.lineWidth = 1.2;
      for (const s of rec.misses) if (s.t <= end) { c.beginPath(); c.arc(vx(s.x), vy(s.y), 4, 0, Math.PI * 2); c.stroke(); }
      c.globalAlpha = 1; c.lineWidth = 1;
    }
    for (const r of order) {
      if (r === L.focus && L.sheets && L.sheets.body) continue;
      const rec = L.recs[r].rec, t = Math.min(L.t, rec.len - 1);
      const x = vx(rec.xs[t]), y = vy(rec.ys[t]);
      c.fillStyle = RANK_COL[r]; c.beginPath(); c.arc(x, y, 7, 0, Math.PI * 2); c.fill();
      c.fillStyle = '#111'; c.font = 'bold 10px system-ui'; c.fillText(String(r + 1), x, y + 0.5);
    }
    if (L.focus >= 0) drawFocus(L.recs[L.focus].rec, L.focus);
  }
  function drawFocus(rec, r) {
    const sh = L.sheets || {}, vfx = sh.vfx, t = Math.min(L.t, rec.len - 1);
    const x = rec.xs[t], y = rec.ys[t], flags = rec.flags[t];
    // effects alive at this tick (each lasts its frames; the longest is under 40 ticks)
    const live = [];
    if (vfx) for (let i = rec.fx.length - 1; i >= 0; i--) {
      const e = rec.fx[i]; if (e.t > t) continue; if (e.t < t - 40) break;
      const f = frameOf(vfx.anims[e.tag], t - e.t, false); if (!f) continue;
      const z = /^trail|^after|^dash_gas/.test(e.tag) ? -1 : 3;
      live.push({ e, f, z });
    }
    const buf = [];
    if (vfx) {
      const form = rec.form[t];
      if (form >= 2) buf.push({ tag: 'form' + form, z: form === 4 ? -1 : 3 });
      if (rec.chain[t]) buf.push({ tag: 'pips' + rec.chain[t], z: 4 });
      buf.push({ tag: 'gas' + Math.trunc((Math.min(GAS_MAX, rec.gas[t]) + 5) / 10), z: 4 });
      buf.push({ tag: r >= APEX ? 'apex' + L.apex : 'rank' + r, z: 4 });
    }
    const drawE = o => { const p = o.e.follow ? [x, y] : [o.e.x, o.e.y]; blit(vfx, o.f, p[0], p[1], false); };
    const drawB = b => blit(vfx, frameOf(vfx.anims[b.tag], t, true), x, y, false);
    live.filter(o => o.z < 0).reverse().forEach(drawE);
    buf.filter(b => b.z < 0).forEach(drawB);
    if (sh.body) {
      const tag = flags & 2 ? 'hit' : flags & 4 ? 'run' : 'idle';
      blit(sh.body, frameOf(sh.body.anims[tag] || sh.body.anims.idle, t, true), x, y, rec.face[t] < 0);
    }
    buf.filter(b => b.z >= 0).forEach(drawB);
    live.filter(o => o.z >= 0).reverse().forEach(drawE);
    if (flags & 2) { L.ctx.fillStyle = '#ffe066'; L.ctx.font = 'bold 11px system-ui'; L.ctx.fillText('STUN', vx(x), vy(y) - 30 * view.scale * UPX); }
  }

  // ------------------------------------------------------------------ UI
  const secs = t => (t / TPS).toFixed(1) + 's';
  function stateOf(rec, t) {
    const f = rec.flags[t];
    if (t >= rec.len - 1 && rec.len < L.limit * TPS) return '<b style="color:#7fd1a8">done</b>';
    return f & 2 ? '<span style="color:#ffe066">stunned</span>' : f & 16 ? 'gas dash' : f & 8 ? 'gliding' : f & 1 ? `flying ${Math.round(rec.sp[t])}` : f & 4 ? 'walking' : 'standing';
  }
  function renderSide() {
    const s = $('#llSide'); if (!s) return;
    const mapOpts = Object.entries(MAPS).map(([k, m]) => `<option value="${k}"${L.mapKey === k ? ' selected' : ''}>${esc(m.label)}</option>`).join('')
      + (L.game ? `<option value="game"${L.mapKey === 'game' ? ' selected' : ''}>Game map (${L.game.source === 'dump' ? 'map_dump.json' : 'walls only'})</option>` : '')
      + (L.mapKey === 'custom' ? '<option value="custom" selected>Custom (edited)</option>' : '');
    s.innerHTML = `
      <label class="st-row">Map<select id="llMap">${mapOpts}</select></label>
      <p class="muted st-note">Click: add a waypoint · right-click: remove the last · Alt+click: move home (gas refills there) · Shift+click: add / remove a wall cell.</p>
      <div class="st-btns"><button class="btn small" data-ll="clear">Clear route</button><button class="btn small" data-ll="reset">Reset map</button></div>
      <h4>Ranks</h4>
      ${RANKS.map((n, r) => `<label class="st-check"><input type="checkbox" data-llr="${r}"${L.shown[r] ? ' checked' : ''}><i class="st-dot" style="background:${RANK_COL[r]}">${r + 1}</i>${n}<span class="muted" style="margin-left:auto">${Math.round(baseSpeed(r, L.apex))}</span></label>`).join('')}
      <label class="st-row">Apex #<select id="llApex">${[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map(p => `<option value="${p}"${L.apex === p ? ' selected' : ''}>#${p} (${Math.round(baseSpeed(APEX, p))})</option>`).join('')}</select></label>
      <h4>Watch</h4>
      <label class="st-row">Focus<select id="llFocus"><option value="-1">none</option>${RANKS.map((n, r) => `<option value="${r}"${L.focus === r ? ' selected' : ''}>${n}</option>`).join('')}</select></label>
      <label class="st-check"><input type="checkbox" id="llFollow"${L.follow ? ' checked' : ''}> Follow (zoom in on the focused rank)</label>
      <label class="st-row">Seed<span style="display:flex;gap:4px"><input type="number" id="llSeed" value="${L.seed}" style="width:70px"><button class="btn small" data-ll="dice" title="Another seed">🎲</button></span></label>
      <label class="st-row">Time limit<select id="llLimit">${[60, 90, 120, 180].map(v => `<option value="${v}"${L.limit === v ? ' selected' : ''}>${v}s</option>`).join('')}</select></label>
      <label class="st-row">Flights per rank<select id="llRuns">${[5, 20, 50, 100].map(v => `<option value="${v}"${L.runs === v ? ' selected' : ''}>${v}</option>`).join('')}</select></label>`;
    renderPlay();
  }
  function renderPlay() {
    const p = $('#llPlay'); if (!p) return;
    const max = L.recs ? Math.max(...L.recs.map(x => x.rec.len)) - 1 : 0;
    p.innerHTML = `<button class="btn small" data-ll="play">${L.playing ? 'Pause' : 'Play'}</button><button class="btn small" data-ll="restart">⟲</button>
      <select id="llSpeed">${[0.25, 0.5, 1, 2, 4].map(v => `<option value="${v}"${L.speed === v ? ' selected' : ''}>${v}x</option>`).join('')}</select>
      <input type="range" id="llScrub" min="0" max="${max}" value="${Math.min(L.t, max)}" style="flex:1"><span id="llTime" class="muted" style="min-width:48px;text-align:right">${secs(L.t)}</span>`;
  }
  function renderLegend() {
    const g = $('#llLegend'); if (!g || !L.recs) return;
    g.innerHTML = RANKS.map((n, r) => {
      const rec = L.recs[r].rec, t = Math.min(L.t, rec.len - 1), st = L.recs[r].stats;
      return `<div class="ll-leg${r === L.focus ? ' ll-on' : ''}" data-llf="${r}"><i class="st-dot" style="background:${RANK_COL[r]}">${r + 1}</i><b>${n}</b>
        <span>${stateOf(rec, t)}</span><span class="muted">waypoint ${Math.min(rec.wp[t] + 1, L.route.length + 1)}/${L.route.length + 1} · chain ${rec.chain[t]} · gas ${rec.gas[t]}</span>
        <span class="muted">${st.finish != null ? 'route in ' + secs(st.finish) : 'not done in ' + L.limit + 's'}</span></div>`;
    }).join('');
  }
  function renderTable() {
    const el = $('#llTable'); if (!el) return;
    const R = L.results || [];
    const rows = RANKS.map((n, r) => {
      const a = R[r];
      const head = `<td><i class="st-dot" style="background:${RANK_COL[r]}">${r + 1}</i>${n}${r === APEX ? ' #' + L.apex : ''}</td><td>${Math.round(baseSpeed(r, L.apex))}</td><td>${MISAIM[r]}%</td><td>${RECOVER[r]}</td><td>${LOOKAHEAD[r] || '-'}</td>`;
      if (!a) return `<tr>${head}<td colspan="8" class="muted">…</td></tr>`;
      const done = a.filter(s => s.finish != null);
      const mins = s => s.ticks / TPS / 60;
      return `<tr>${head}<td><b>${done.length ? secs(avg(done, s => s.finish)) : '-'}</b></td><td>${Math.round(done.length * 100 / a.length)}%</td>
        <td>${(avg(a, s => s.cables / mins(s))).toFixed(0)}</td><td>${avg(a, s => s.misses).toFixed(1)}</td><td>${avg(a, s => s.slams).toFixed(1)}</td>
        <td>${avg(a, s => s.stun / TPS).toFixed(1)}s</td><td>${Math.round(avg(a, s => s.top))}</td><td>${Math.round(avg(a, s => s.air * 100 / s.ticks))}%</td></tr>`;
    }).join('');
    const wk = L.walk ? `<tr class="muted"><td>Walking only (no cables)</td><td>${WALK}</td><td colspan="3"></td><td><b>${L.walk.finish != null ? secs(L.walk.finish) : '-'}</b></td><td colspan="7"></td></tr>` : '';
    el.innerHTML = `<table class="st-table ll-table"><tr><th>Rank</th><th>Start speed</th><th>Misaim</th><th>Recover (ticks)</th><th>Timing (ticks)</th>
      <th>Route time</th><th>Done</th><th>Cables / min</th><th>Missed cables</th><th>Slams</th><th>Stunned</th><th>Top speed</th><th>In the air</th></tr>${rows}${wk}</table>
      <p class="muted st-note">Averages over ${L.runs} flights per rank (seeds differ from the ghost race). Route time counts only the flights that finished within ${L.limit}s. A champion walks ${WALK} a tick.</p>
      <div class="ll-plays">${PLAYS.map((p, r) => `<div><i class="st-dot" style="background:${RANK_COL[r]}">${r + 1}</i><b>${RANKS[r]}</b> <span class="muted">(${RANK_GAMES[r]})</span>: ${esc(p)}</div>`).join('')}</div>`;
  }
  function toWorld(ev) { const r = L.canvas.getBoundingClientRect(); return [(ev.clientX - r.left) * (L.canvas.width / r.width) / view.scale + view.ox, (ev.clientY - r.top) * (L.canvas.height / r.height) / view.scale + view.oy]; }
  function changed(mapEdit) { if (mapEdit && L.mapKey !== 'custom') L.mapKey = 'custom'; save(); resimulate(); }
  function onCanvas(ev) {
    ev.preventDefault();
    const [x, y] = toWorld(ev);
    const cx = Math.floor(x / CELL), cy = Math.floor(y / CELL);
    if (cx < 0 || cy < 0 || cx >= N || cy >= N) return;
    if (ev.shiftKey && ev.button === 0) { L.cells[cy * N + cx] ^= 1; return changed(true); }
    if (L.cells[cy * N + cx]) return;   // waypoints and home go on free ground
    if (ev.button === 2) { if (L.route.length) { L.route.pop(); changed(false); } return; }
    if (ev.altKey) { L.home = [x, y]; return changed(false); }
    L.route.push([x, y]); changed(false);
  }
  function onClick(ev) {
    const f = ev.target.closest('[data-llf]'); if (f) { L.focus = +f.dataset.llf; renderSide(); save(); return; }
    const b = ev.target.closest('[data-ll]'); if (!b) return;
    const a = b.dataset.ll;
    if (a === 'clear') { L.route = []; changed(false); }
    if (a === 'reset') { useMap(L.mapKey === 'custom' ? 'jungle' : L.mapKey); save(); resimulate(); }
    if (a === 'dice') { L.seed = 1 + Math.floor(Math.random() * 99999); save(); resimulate(); }
    if (a === 'play') { if (!L.playing && L.recs && L.t >= Math.max(...L.recs.map(x => x.rec.len)) - 1) L.t = 0; L.playing = !L.playing; renderPlay(); }
    if (a === 'restart') { L.t = 0; L.playing = true; renderPlay(); }
  }
  function onChange(ev) {
    const t = ev.target;
    if (t.dataset.llr != null) { L.shown[+t.dataset.llr] = t.checked; save(); return; }
    if (t.id === 'llMap') { useMap(t.value); save(); resimulate(); }
    if (t.id === 'llApex') { L.apex = +t.value; save(); resimulate(); }
    if (t.id === 'llFocus') { L.focus = +t.value; save(); }
    if (t.id === 'llFollow') { L.follow = t.checked; save(); }
    if (t.id === 'llSeed') { L.seed = Math.max(0, +t.value || 0); save(); resimulate(); }
    if (t.id === 'llLimit') { L.limit = +t.value; save(); resimulate(); }
    if (t.id === 'llRuns') { L.runs = +t.value; save(); resimulate(); }
    if (t.id === 'llSpeed') L.speed = +t.value;
  }
  function frame(now) {
    L.raf = requestAnimationFrame(frame);
    if (!L.el || L.el.hidden || !L.canvas.isConnected) { L.last = now; return; }
    const dt = Math.min(100, now - (L.last || now)); L.last = now;
    if (L.playing && L.recs) {
      const max = Math.max(...L.recs.map(x => x.rec.len)) - 1;
      L.acc += dt * L.speed * TPS / 1000;
      const n = Math.floor(L.acc); L.acc -= n; L.t = Math.min(max, L.t + n);
      if (L.t >= max) { L.playing = false; renderPlay(); }
      const sc = $('#llScrub'); if (sc) sc.value = L.t;
      const tm = $('#llTime'); if (tm) tm.textContent = secs(L.t);
    }
    draw();
    if ((L.frames = (L.frames || 0) + 1) % 6 === 0) renderLegend();
  }
  async function mount(el) {
    L.el = el;
    if (!L.mounted) {
      L.mounted = true;
      el.innerHTML = `<div class="ll-grid">
        <aside class="st-left" id="llSide"></aside>
        <div class="ll-mid"><canvas id="llCanvas" width="720" height="720"></canvas>
          <div class="ll-play" id="llPlay"></div><div id="llLegend" class="ll-legend"></div></div>
        <div class="ll-right"><h4>Every rank on this route</h4><div id="llTable"></div></div></div>`;
      L.canvas = $('#llCanvas'); L.ctx = L.canvas.getContext('2d');
      L.canvas.addEventListener('mousedown', onCanvas);
      L.canvas.addEventListener('contextmenu', ev => ev.preventDefault());
      el.addEventListener('click', onClick);
      el.addEventListener('change', onChange);
      el.addEventListener('input', ev => { if (ev.target.id === 'llScrub') { L.t = +ev.target.value; L.playing = false; const tm = $('#llTime'); if (tm) tm.textContent = secs(L.t); } });
      await Promise.all([loadSheets(), loadGameMap()]);
      if (!load()) useMap('jungle');
      if (L.mapKey === 'game' && !L.game) useMap('jungle');
      resimulate();
      requestAnimationFrame(frame);
    }
  }
  window.TFM2LeviLab = {
    mount,
    /** the Skill Test arena's mastery choice: focus that rank here */
    setRank(r, apex) {
      const again = apex && apex !== L.apex;
      L.focus = r; if (apex) L.apex = apex;
      if (L.mounted) { save(); if (again) resimulate(); else renderSide(); }
    },
    RANKS, RANK_GAMES, PLAYS, baseSpeed, MISAIM, RECOVER, LOOKAHEAD,
    _L: L, _simulate: simulate, _makeWorld: makeWorld, _field: field, _MAPS: MAPS,
  };
})();
