// Akane Lab: the game's exact core (akane.wasm) on the real 5v5 map (levi-map.js: the wall grid read off the game's wall
// layer, and the map art), with a small, labelled host model around it: glides, simple enemies that see and attack her,
// deaths that drop Soul Orbs, respawns, a wave and a tower. Every range is drawn at the same scale as the bodies.
(async () => {
  const $ = id => document.getElementById(id);
  const R = window.AKANE_RULES, MAP = window.TFM2_LEVI_MAP, canvas = $('arena'), ctx = canvas.getContext('2d');
  const wasm = (await WebAssembly.instantiate(await (await fetch('akane.wasm')).arrayBuffer(), {})).instance.exports;
  const N = 30, CELL = 32000, WORLD = 960000;
  const TOWERS = [[364000, 76000], [780000, 110000], [840000, 170000], [80000, 364000], [596000, 364000], [364000, 596000],
    [880000, 596000], [110000, 780000], [170000, 840000], [596000, 880000]];
  const WALLS = new Array(N * N).fill(false);
  MAP.walls.forEach((row, y) => { for (let x = 0; x < N; x++) WALLS[y * N + x] = row[x] === '#'; });
  const wallAt = p => p.x < 0 || p.y < 0 || p.x >= WORLD || p.y >= WORLD || WALLS[Math.floor(p.y / CELL) * N + Math.floor(p.x / CELL)];
  const art = new Image(); art.src = 'data:image/png;base64,' + MAP.png;
  const names = ['ego', 'judgement', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid', 'aggressive', 'order', 'roaming', 'macro', 'micro', 'awareness', 'strategy'];
  $('stats').innerHTML = names.map(n => `<span>${n}</span><input type="number" min="0" max="100" value="50">`).join('');
  const rp = p => ({x: Math.round(p.x), y: Math.round(p.y)});
  const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  const cellCenter = (cx, cy) => ({x: cx * CELL + CELL / 2, y: cy * CELL + CELL / 2});
  let tick, me, units, glides, state, queue, fx, logLines, hits, basicHits, takedowns, nightUntil, running = true, aim = {x: 480000, y: 480000}, held = new Set(), lastRejection = '', steer = null;
  const unit = (id, team, p, o = {}) => ({id, team, pos: {...p}, hp: 1000, max_hp: 1000, radius: 3000, attack: 90, armor: 25, champion: true, tower: false, farmable: false,
    slowUntil: 0, stunUntil: 0, airborneUntil: 0, vulnerableUntil: 0, bleed: 0, bleedUntil: 0, crimsonUntil: 0, markUntil: 0, nextAttack: 0, deadUntil: 0, home: {...p},
    flash_ready: true, skill_ready: true, can: 4, ...o});
  function wallCrossing() {
    let best = null;
    for (let cy = 2; cy < N - 2; cy++) for (let cx = 1; cx < N - 2; cx++) {
      if (!WALLS[cy * N + cx] && WALLS[cy * N + cx + 1] && !WALLS[cy * N + cx + 2]) {
        const d = Math.hypot(cx - 15, cy - 15);
        if (!best || d < best.d) best = {d, from: cellCenter(cx, cy), to: cellCenter(cx + 2, cy)};
      }
    }
    return best;
  }
  function reset() {
    wasm.akane_reset();
    tick = 0; state = {}; queue = []; glides = []; fx = []; logLines = []; hits = []; basicHits = []; takedowns = []; nightUntil = 0; steer = null;
    const c = wallCrossing();
    me = {pos: {...c.from}, hp: 1600, max_hp: 1600, attack: 180, soulUntil: 0, speedPct: 0, invulnUntil: 0, stunUntil: 0};
    const t = TOWERS.slice().sort((a, b) => Math.hypot(a[0] - c.from.x, a[1] - c.from.y) - Math.hypot(b[0] - c.from.x, b[1] - c.from.y))[0];
    units = [
      unit(2, 1, {x: c.from.x + 45000, y: c.from.y + 25000}),
      unit(3, 1, {x: c.from.x + 70000, y: c.from.y - 30000}, {can: 1 | 8}),
      unit(4, 0, {x: c.from.x - 30000, y: c.from.y + 10000}),
      unit(10, 1, {x: c.from.x - 20000, y: c.from.y - 30000}, {champion: false, farmable: true, hp: 300, max_hp: 300, radius: 2000, attack: 20, armor: 0}),
      unit(11, 1, {x: c.from.x - 24000, y: c.from.y - 34000}, {champion: false, farmable: true, hp: 300, max_hp: 300, radius: 2000, attack: 20, armor: 0}),
      unit(20, 9, {x: c.to.x, y: c.to.y + 2000}, {champion: false, farmable: true, hp: 900, max_hp: 900, radius: 5000, attack: 30, armor: 10}),
      unit(30, 1, {x: t[0], y: t[1]}, {champion: false, tower: true, hp: 3000, max_hp: 3000, radius: 8000, attack: 150}),
    ];
    log(`Reset on the real map: Akane west of a wall cell, a camp past it; two enemy champions, a wave and the nearest tower (${t}).`);
  }
  function log(s) { logLines.push(`[${tick}] ${s}`); if (logLines.length > 200) logLines.shift(); $('log').textContent = logLines.slice(-60).join('\n'); $('log').scrollTop = 1e9; }
  const find = id => units.find(u => u.id === id);
  const alive = u => u.hp > 0 && tick >= u.deadUntil;
  function glide(who, to, ticks) { glides = glides.filter(g => g.who !== who); glides.push({who, from: {...who.pos}, to: {...to}, t0: tick, t1: tick + Math.max(1, ticks)}); }
  const nightOn = () => tick < nightUntil;
  /** Lab vision: can the enemy team see her? Not in Dismiss; in Crimson Night only within the nearsight radius. */
  function seenByEnemy(p) {
    if (tick < me.soulUntil && p === me.pos) return false;
    const eyes = units.filter(u => u.team === 1 && alive(u));
    if (!nightOn()) return true;
    return eyes.some(e => dist(e.pos, p) <= R.night.sight);
  }
  function frame() {
    const profile = [...$('stats').querySelectorAll('input')].map(i => Math.max(0, Math.min(100, +i.value || 0)));
    const calm = !units.some(u => u.team === 1 && u.champion && alive(u) && dist(u.pos, me.pos) < R.ai.calmRange);
    // towers: is one of hers (not her) inside for it to shoot first?
    const tanked = tw => units.some(u => u.team === 0 && alive(u) && dist(u.pos, tw.pos) <= R.ai.towerRange);
    return {version: 1, rules_revision: R.revision, frame: {
      tick, owner: 1, team: 0, alive: me.hp > 0, pos: rp(me.pos), hp: Math.round(me.hp), max_hp: me.max_hp, attack: me.attack, armor: 20,
      stunned: tick < me.stunUntil, ult_unlocked: true, flash_ready: false, home: {x: 96000, y: 864000},
      units: units.filter(alive).map(u => ({id: u.id, team: u.team, pos: rp(u.pos), hp: Math.round(u.hp), max_hp: u.max_hp, shield: 0, radius: u.radius,
        armor: u.armor, attack: u.attack, champion: u.champion, tower: u.tower, visible: true, targetable: true, farmable: u.farmable && calm,
        stunned: tick < u.stunUntil, velocity: rp(u.vel || {x: 0, y: 0}), flash_ready: !!u.flash_ready, skill_ready: !!u.skill_ready, can: u.can || 0,
        cc_immune: false, reduce: 0, parry: false, tanked: u.tower ? tanked(u) : false})),
      terrain: WALLS, bushes: [], profile, basic_hits: basicHits.splice(0), takedowns: takedowns.splice(0), manual: !$('auto').checked, order: queue.shift() || null}};
  }
  function hurt(u, amount) {
    if (!alive(u)) return;
    const amp = tick < u.vulnerableUntil ? 1 + R.edge.vulnerablePct / 100 : 1;
    u.hp = Math.max(0, u.hp - Math.round(amount * amp));
    hits.push({id: u.id, amount: Math.round(amount * amp), ally: u.team === 0, tower: u.tower, until: tick + 30});
    if (u.hp <= 0 && !u.tower) {
      u.deadUntil = tick + (u.champion ? 300 : 120);
      if (u.champion) { takedowns.push([u.id, rp(u.pos)]); log(`#${u.id} slain: a Soul Orb drops`); }
    }
  }
  function apply(out) {
    for (const e of out.effects || []) {
      const d = e.data, u = d && d.target !== undefined ? find(d.target) : null;
      switch (e.kind) {
        case 'damage': if (u) { if (u.team === 0 || u.tower) log(`!! #${u.id} (${u.tower ? 'tower' : 'ally'}) hit by a skill (should never happen)`); hurt(u, d.amount); } break;
        case 'slow': if (u) u.slowUntil = tick + d.ticks; break;
        case 'knock_up': if (u) { u.stunUntil = Math.max(u.stunUntil, tick + d.ticks); u.airborneUntil = tick + d.ticks; } break;
        case 'vulnerable': if (u) { u.vulnerableUntil = tick + d.ticks; log(`#${d.target} vulnerable +${d.percent}%`); } break;
        case 'bleed': if (u) { u.bleed = d.stacks; u.bleedUntil = tick + d.ticks; } break;
        case 'crimson': if (u) u.crimsonUntil = tick + d.ticks; break;
        case 'mark': if (u) u.markUntil = tick + d.ticks; break;
        case 'soul': me.soulUntil = tick + d.ticks; me.speedPct = d.speed_pct; me.invulnUntil = Math.max(me.invulnUntil, tick + d.ticks); log(`Dismiss: invisible, invulnerable, +${d.speed_pct}% for ${(d.ticks / 60).toFixed(1)} s (move only)`); break;
        case 'heal': me.hp = Math.min(me.max_hp, me.hp + d.amount); break;
        case 'invulnerable': me.invulnUntil = Math.max(me.invulnUntil, tick + d.ticks); break;
        case 'teleport': case 'hop': glides = glides.filter(g => g.who !== me); me.pos = {...d.to}; if (e.kind === 'hop') log('Dash: over the wall'); break;
        case 'glide': glide(me, d.to, d.ticks); break;
        case 'night': nightUntil = tick + d.ticks; log('Crimson Night: every enemy is nearsighted'); break;
        case 'steer': steer = {...d.to, until: tick + 6}; break;
        case 'fx': fx.push({name: d.name, at: {...d.at}, start: tick, until: tick + Math.max(6, Math.min(d.ticks, 120))}); break;
        case 'log': log(String(d)); break;
      }
    }
  }
  function step() {
    const text = JSON.stringify(frame()), bytes = new TextEncoder().encode(text);
    const ptr = wasm.akane_input(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    const outPtr = wasm.akane_step(bytes.length), n = wasm.akane_output_len();
    const out = JSON.parse(new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, outPtr, n)));
    if (out.error) { log('Lab error: ' + out.error); return; }
    state = out.state;
    if (out.rejection && out.rejection !== lastRejection) log('Refused: ' + out.rejection);
    lastRejection = out.rejection || '';
    apply(out);
    tick++;
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)); g.who.pos = {x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k}; }
    glides = glides.filter(g => tick < g.t1);
    // her walk: WASD, or the AI's steer
    if (tick >= me.stunUntil && !glides.some(g => g.who === me)) {
      let dir = {x: (held.has('d') - held.has('a')), y: (held.has('s') - held.has('w'))};
      if (!dir.x && !dir.y && steer && tick < steer.until && dist(steer, me.pos) > 1500) dir = {x: steer.x - me.pos.x, y: steer.y - me.pos.y};
      if (dir.x || dir.y) {
        const l = Math.hypot(dir.x, dir.y), v = R.stat.move_speed * (tick < me.soulUntil ? 1 + me.speedPct / 100 : 1);
        const next = {x: me.pos.x + dir.x / l * v, y: me.pos.y + dir.y / l * v};
        if (!wallAt(next)) me.pos = next;
      }
    }
    // the Lab's enemies: walk in when they can see her, hit her
    for (const u of units) {
      if (!alive(u) && u.hp <= 0 && tick >= u.deadUntil && u.deadUntil > 0) { u.hp = u.max_hp; u.pos = {...u.home}; u.deadUntil = 0; }
    }
    if ($('moving').checked) {
      const seen = seenByEnemy(me.pos);
      for (const u of units.filter(u => u.team === 1 && u.champion && alive(u))) {
        const prev = {...u.pos};
        if (tick < u.stunUntil) { u.vel = {x: 0, y: 0}; continue; }
        const speed = tick < u.slowUntil ? 560 : 800, d = dist(u.pos, me.pos);
        if (seen && !glides.some(g => g.who === u) && d > 20000) { const k = speed / d, next = {x: u.pos.x + (me.pos.x - u.pos.x) * k, y: u.pos.y + (me.pos.y - u.pos.y) * k}; if (!wallAt(next)) u.pos = next; }
        u.vel = {x: u.pos.x - prev.x, y: u.pos.y - prev.y};
        if (seen && d <= 25000 && tick >= u.nextAttack && tick >= me.invulnUntil) { u.nextAttack = tick + 60; me.hp = Math.max(1, me.hp - u.attack); }
      }
      const tw = units.find(u => u.tower && alive(u));
      if (tw && seen && dist(tw.pos, me.pos) <= R.ai.towerRange && tick % 60 === 0 && tick >= me.invulnUntil
        && !units.some(u => u.team === 0 && alive(u) && dist(u.pos, tw.pos) <= R.ai.towerRange)) { me.hp = Math.max(1, me.hp - tw.attack); log('The tower hits her'); }
    }
    // bleeds and crimson tick in the core; the Lab only expires the markers
    fx = fx.filter(f => tick < f.until);
    hits = hits.filter(h => tick < h.until);
  }
  function order(kind) {
    if (kind === 'kunai') return queue.push(state.mark && tick < state.mark[1] ? {kind: 'kunai_recast'} : {kind: 'kunai', aim: rp(aim)});
    if (kind === 'night') {
      if (tick < (state.night_until || 0) && !state.night_dash_used) {
        const t = units.filter(u => u.team === 1 && u.champion && alive(u)).sort((a, b) => dist(a.pos, aim) - dist(b.pos, aim))[0];
        return t && queue.push({kind: 'night_dash', aim: t.id});
      }
      return queue.push({kind: 'night'});
    }
    queue.push({kind});
  }
  /** a basic attack on `id`, or on the nearest enemy in reach */
  function basic(id) {
    if (tick < me.soulUntil) return log('Dismiss: she can only move');
    const t = units.filter(u => u.team !== 0 && alive(u) && (id === undefined || u.id === id) && dist(u.pos, me.pos) <= R.basic.range + u.radius + 3000).sort((a, b) => dist(a.pos, me.pos) - dist(b.pos, me.pos))[0];
    if (!t) return log(`Nothing within basic reach (${R.basic.range})`);
    hurt(t, me.attack * 100 / (100 + t.armor)); basicHits.push(t.id);
  }
  // ---------------------------------------------------------------- drawing (one world-to-screen scale for everything)
  let cam = {x: 480000, y: 480000, s: 900 / WORLD};
  const P = p => ({x: (p.x - cam.x) * cam.s + 450, y: (p.y - cam.y) * cam.s + 450});
  const W = r => r * cam.s;
  function circle(p, r, fill, stroke, width = 1) { const q = P(p); ctx.beginPath(); ctx.arc(q.x, q.y, Math.max(1.5, W(r)), 0, 7); if (fill) { ctx.fillStyle = fill; ctx.fill(); } if (stroke) { ctx.lineWidth = width; ctx.strokeStyle = stroke; ctx.stroke(); ctx.lineWidth = 1; } }
  function line(a, b, stroke, width = 1) { const p = P(a), q = P(b); ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(q.x, q.y); ctx.lineWidth = width; ctx.strokeStyle = stroke; ctx.stroke(); ctx.lineWidth = 1; }
  const toward = (a, b, len) => { const d = dist(a, b) || 1; return {x: a.x + (b.x - a.x) * len / d, y: a.y + (b.y - a.y) * len / d}; };
  function draw() {
    if ($('zoom').checked) cam = {x: me.pos.x, y: me.pos.y, s: 900 / 260000}; else cam = {x: 480000, y: 480000, s: 900 / WORLD};
    ctx.fillStyle = '#2a3340'; ctx.fillRect(0, 0, 900, 900);
    const o = P({x: 0, y: 0});
    if ($('art').checked && art.complete) { ctx.globalAlpha = .75; ctx.drawImage(art, o.x, o.y, W(WORLD), W(WORLD)); ctx.globalAlpha = 1; }
    ctx.fillStyle = 'rgba(20,20,26,.55)';
    for (let i = 0; i < N * N; i++) if (WALLS[i]) { const q = P({x: i % N * CELL, y: Math.floor(i / N) * CELL}); ctx.fillRect(q.x, q.y, W(CELL) + .5, W(CELL) + .5); }
    for (const [tx, ty] of TOWERS) { const q = P({x: tx, y: ty}); ctx.fillStyle = '#c9a94a'; ctx.fillRect(q.x - 4, q.y - 4, 8, 8); }
    if (nightOn()) {
      ctx.fillStyle = 'rgba(40,0,10,.45)'; ctx.fillRect(0, 0, 900, 900);
      for (const u of units.filter(u => u.team === 1 && alive(u))) circle(u.pos, R.night.sight, 'rgba(255,255,255,.08)', 'rgba(255,200,200,.5)');
    }
    if ($('ranges').checked) {
      circle(me.pos, R.kunai.range, null, 'rgba(248,113,113,.6)');
      line(me.pos, toward(me.pos, aim, R.kunai.range), 'rgba(248,113,113,.7)', Math.max(1, W(R.kunai.width * 2)));
      circle(me.pos, R.night.dashRange, null, 'rgba(190,24,93,.35)');
      circle(me.pos, R.ai.towerRange, null, 'rgba(201,169,74,.15)');
    }
    for (const orb of state.orbs || []) {
      const armed = tick >= orb.born + R.orb.armTicks;
      circle(orb.pos, R.orb.range, null, armed ? 'rgba(244,114,182,.6)' : 'rgba(244,114,182,.25)');
      circle(orb.pos, 3500, armed ? '#f472b6' : '#9d4b72', '#fff');
      if (orb.since !== null && orb.since !== undefined) { const q = P(orb.pos); ctx.fillStyle = '#fff'; ctx.fillText(`${Math.min(100, Math.round((tick - orb.since) / R.orb.absorbTicks * 100))}%`, q.x + 8, q.y); }
    }
    for (const f of fx) {
      if (f.name === 'akn_burst') circle(f.at, R.edge.burstRadius, 'rgba(220,38,38,.35)', '#fecaca', 2);
      else if (f.name === 'akn_finale') circle(f.at, R.edge.knockRadius, 'rgba(244,63,94,.25)', '#fda4af', 2);
      else if (f.name === 'akn_kunai') line(me.pos, f.at, '#e5e7eb', 2);
      else if (f.name === 'akn_dismiss') circle(f.at, 6000, null, '#f9a8d4', 2);
      else if (f.name === 'akn_dash_slash' || f.name === 'akn_night_slash') circle(f.at, 5000, 'rgba(255,255,255,.3)', '#fff', 2);
    }
    for (const u of units.filter(alive)) {
      const col = u.tower ? '#c9a94a' : u.team === 0 ? '#60a5fa' : u.team === 9 ? '#a3e635' : u.champion ? '#f87171' : '#fb923c';
      const hidden = u.team === 0 && nightOn() && !seenByEnemy(u.pos);
      ctx.globalAlpha = hidden ? .35 : 1;
      circle(u.pos, u.radius, tick < u.airborneUntil ? '#fff' : col, tick < u.vulnerableUntil ? '#fde047' : tick < u.markUntil ? '#e5e7eb' : null, 3);
      ctx.globalAlpha = 1;
      if (tick < u.crimsonUntil) circle(u.pos, u.radius + 1500, null, '#dc2626', 2);
      const q = P(u.pos);
      for (let i = 0; i < (tick < u.bleedUntil ? u.bleed : 0); i++) { ctx.fillStyle = '#ef4444'; ctx.fillRect(q.x - 12 + i * 7, q.y - W(u.radius) - 14, 5, 5); }
      ctx.fillStyle = '#111'; ctx.fillRect(q.x - 14, q.y - W(u.radius) - 8, 28, 4); ctx.fillStyle = '#4ade80'; ctx.fillRect(q.x - 14, q.y - W(u.radius) - 8, 28 * u.hp / u.max_hp, 4);
    }
    // Akane: crimson body, faded while hidden (Dismiss, or the night outside every enemy's sight)
    const hiddenMe = tick < me.soulUntil || (nightOn() && !seenByEnemy(me.pos));
    ctx.globalAlpha = hiddenMe ? .4 : 1;
    circle(me.pos, 3000, '#be123c', tick < me.invulnUntil ? '#fde047' : '#0b0f14', 3);
    ctx.globalAlpha = 1;
    const q = P(me.pos);
    ctx.fillStyle = '#111'; ctx.fillRect(q.x - 22, q.y - W(3000) - 12, 44, 5);
    ctx.fillStyle = '#ef4444'; ctx.fillRect(q.x - 22, q.y - W(3000) - 12, 44 * me.hp / me.max_hp, 5);
    for (let i = 0; i < (state.charges ?? 0); i++) { ctx.fillStyle = '#f9a8d4'; ctx.beginPath(); ctx.arc(q.x - 6 + i * 12, q.y + W(3000) + 9, 4, 0, 7); ctx.fill(); }
    for (const h of hits) { const u = find(h.id); if (u) { const p = P(u.pos); ctx.fillStyle = '#fde68a'; ctx.font = '11px monospace'; ctx.fillText(h.amount, p.x + 8, p.y - 10 - (30 - (h.until - tick))); } }
    ctx.fillStyle = 'rgba(15,10,15,.8)'; ctx.fillRect(8, 870, 560, 22); ctx.fillStyle = '#e5d0d8'; ctx.font = '12px system-ui';
    ctx.fillText(`Map: game 5v5 (levi-map.js) · scale 1:${Math.round(1 / cam.s)} · enemies/minions/tower/deaths: Lab model`, 14, 885);
  }
  function hud() {
    const cd = i => Math.max(0, ((state.ready || [])[i] || 0) - tick) / 60;
    const edge = state.edge ? `${state.edge[0]} / ${R.edge.hits} hits (${((state.edge[1] - tick) / 60).toFixed(1)} s)` : '—';
    const orbs = (state.orbs || []).map(o => `${((o.until - tick) / 60).toFixed(0)} s${o.since != null ? ' absorbing' : ''}`).join(', ') || '—';
    $('hud').innerHTML = [
      ['HP', `${Math.round(me.hp)} / ${me.max_hp}`], ['Soul charges', `${state.charges ?? 2} / ${R.soul.charges}${state.next_charge ? ` (+1 in ${((state.next_charge - tick) / 60).toFixed(1)} s)` : ''}`],
      ['Dismiss', tick < me.soulUntil ? `${((me.soulUntil - tick) / 60).toFixed(2)} s (move only)` : '—'], ['Crimson Edge', edge],
      ['Kunai mark', state.mark && tick < state.mark[1] ? `#${state.mark[0]} (${((state.mark[1] - tick) / 60).toFixed(1)} s)` : '—'], ['Orbs', orbs],
      ['Edge reset ready', (state.edge_reset_ready || 0) > tick ? `in ${((state.edge_reset_ready - tick) / 60).toFixed(0)} s` : 'yes'],
      ['Crimson Night', nightOn() ? `${((nightUntil - tick) / 60).toFixed(1)} s${state.night_dash_used ? ' (dash used)' : ''}` : '—'],
      ['Kunai', cd(0).toFixed(1)], ['Edge', cd(1).toFixed(1)], ['Night', cd(2).toFixed(1)], ['Chain', state.chain ?? 0],
    ].map(([k, v]) => `<span>${k}</span><b>${v}</b>`).join('');
    $('status').textContent = `tick ${tick} · ${running ? 'running' : 'paused'} · rules ${R.revision}`;
  }
  function frameLoop() { if (running) step(); draw(); hud(); requestAnimationFrame(frameLoop); }
  canvas.addEventListener('mousemove', e => { const r = canvas.getBoundingClientRect(); aim = {x: (e.clientX - r.left - 450) / cam.s + cam.x, y: (e.clientY - r.top - 450) / cam.s + cam.y}; });
  addEventListener('keydown', e => {
    const k = e.key.toLowerCase();
    if ('wasd'.includes(k) && k.length === 1) held.add(k);
    if (k === '1') order('soul'); if (k === '2') order('kunai'); if (k === '3') order('edge'); if (k === '4') order('night'); if (k === 'f') basic();
  });
  addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  $('play').onclick = () => { running = !running; $('play').textContent = running ? 'Pause' : 'Play'; };
  $('step').onclick = () => { step(); draw(); hud(); };
  $('reset').onclick = () => reset();
  $('wallDemo').onclick = () => { const c = wallCrossing(); me.pos = {...c.from}; const f = find(2); f.pos = {...c.to}; f.hp = f.max_hp; f.deadUntil = 0; aim = {...c.to}; log('A foe across the wall: Kunai (2) flies over it, 2 again dashes over the wall'); };
  $('lowFoe').onclick = () => { const f = find(2); f.hp = 220; f.flash_ready = false; log('#2 is low and has no Flash'); };
  $('gank').onclick = () => { units.filter(u => u.team === 1 && u.champion).forEach((u, i) => { u.pos = {x: me.pos.x + 30000 + i * 8000, y: me.pos.y + (i ? 12000 : -12000)}; u.hp = u.max_hp; u.deadUntil = 0; }); log('Enemies converge on her'); };
  reset();
  window.akaneLab = {step, reset, order, basic, wallCrossing, pause() { running = false; }, setAim(p) { aim = {...p}; }, hurt,
    get state() { return state; }, get me() { return me; }, get units() { return units; }, get hits() { return hits; }, get tick() { return tick; }, get walls() { return WALLS; }, wallAt, nightOn, seenByEnemy};
  requestAnimationFrame(frameLoop);
})();
