// Kratos Lab: the game's exact core (kratos.wasm) on the real 5v5 map (levi-map.js: the wall grid read off the game's
// wall layer, and the map art), with a small, labelled host model around it: glides, simple enemies, a wave, a camp
// and a tower. Every range is drawn at the same world-to-screen scale as the bodies.
(async () => {
  const $ = id => document.getElementById(id);
  const R = window.KRATOS_RULES, MAP = window.TFM2_LEVI_MAP, canvas = $('arena'), ctx = canvas.getContext('2d');
  const wasm = (await WebAssembly.instantiate(await (await fetch('kratos.wasm')).arrayBuffer(), {})).instance.exports;
  const N = 30, CELL = 32000, WORLD = 960000;
  // the game's tower spots (as levilab.js uses them)
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
  let tick, me, units, glides, state, queue, fx, logLines, hits, pendingHits, basicHits, running = true, aim = {x: 480000, y: 480000}, held = new Set(), lastRejection = '';
  const unit = (id, team, p, o = {}) => ({id, team, pos: {...p}, hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, farmable: false,
    slowUntil: 0, stunUntil: 0, airborneUntil: 0, frost: 0, frostUntil: 0, burnUntil: 0, frozenUntil: 0, nextAttack: 0, ...o});
  /** a free cell, a wall cell, a free cell in a row, nearest the map centre: Kratos left, a foe right */
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
    wasm.kratos_reset();
    tick = 0; state = {}; queue = []; glides = []; fx = []; logLines = []; hits = []; pendingHits = []; basicHits = [];
    const c = wallCrossing();
    me = {pos: {...c.from}, hp: 2000, max_hp: 2000, shields: [], immuneUntil: 0, reduce: 0, reduceUntil: 0, stunUntil: 0, blades: false, sightUntil: 0, pose: '', poseUntil: 0};
    const t = TOWERS.slice().sort((a, b) => Math.hypot(a[0] - c.from.x, a[1] - c.from.y) - Math.hypot(b[0] - c.from.x, b[1] - c.from.y))[0];
    units = [
      unit(2, 1, {x: c.from.x + 40000, y: c.from.y + 30000}),
      unit(3, 1, {x: c.from.x + 50000, y: c.from.y - 25000}),
      unit(4, 0, {x: c.from.x - 30000, y: c.from.y + 10000}),
      unit(10, 1, {x: c.from.x - 20000, y: c.from.y - 30000}, {champion: false, farmable: true, hp: 300, max_hp: 300, radius: 2000}),
      unit(11, 1, {x: c.from.x - 23000, y: c.from.y - 34000}, {champion: false, farmable: true, hp: 300, max_hp: 300, radius: 2000}),
      unit(20, 9, {x: c.to.x, y: c.to.y + 2000}, {champion: false, farmable: true, hp: 900, max_hp: 900, radius: 5000}),
      unit(30, 1, {x: t[0], y: t[1]}, {champion: false, tower: true, hp: 3000, max_hp: 3000, radius: 8000}),
    ];
    log(`Reset on the real map: Kratos west of a wall cell, a camp past it; two enemy champions, a wave and the nearest tower (${t}).`);
  }
  function log(s) { logLines.push(`[${tick}] ${s}`); if (logLines.length > 200) logLines.shift(); $('log').textContent = logLines.slice(-60).join('\n'); $('log').scrollTop = 1e9; }
  const find = id => units.find(u => u.id === id);
  function glide(who, to, ticks) { glides = glides.filter(g => g.who !== who); glides.push({who, from: {...who.pos}, to: {...to}, t0: tick, t1: tick + Math.max(1, ticks)}); }
  const enemyCalm = () => !units.some(u => u.team === 1 && u.champion && u.hp > 0 && dist(u.pos, me.pos) < R.ai.calmRange);
  function frame() {
    const profile = [...$('stats').querySelectorAll('input')].map(i => Math.max(0, Math.min(100, +i.value || 0)));
    const out = {version: 1, rules_revision: R.revision, hits_on_me: pendingHits.map(h => [h.from, true]), frame: {
      tick, owner: 1, team: 0, alive: me.hp > 0, pos: rp(me.pos), hp: Math.round(me.hp), max_hp: me.max_hp, attack: 100,
      stunned: tick < me.stunUntil, ult_unlocked: true,
      units: units.filter(u => u.hp > 0).map(u => ({id: u.id, team: u.team, pos: rp(u.pos), hp: Math.round(u.hp), max_hp: u.max_hp, radius: u.radius,
        champion: u.champion, tower: u.tower, visible: true, targetable: true, farmable: u.farmable && enemyCalm(), stunned: tick < u.stunUntil})),
      terrain: WALLS, profile, basic_hits: basicHits.splice(0), manual: !$('auto').checked, order: queue.shift() || null}};
    return out;
  }
  function hurtMe(raw) {
    let dmg = Math.round(raw * (100 - (tick < me.reduceUntil ? me.reduce : 0)) / 100);
    for (const s of me.shields) { if (tick >= s.until) continue; const a = Math.min(s.amount, dmg); s.amount -= a; dmg -= a; }
    me.hp = Math.max(1, me.hp - dmg);
    return dmg;
  }
  function apply(out) {
    (out.dodged || []).forEach((d, i) => {
      const h = pendingHits[i];
      if (d) { log(`Heimdall's Sight: dodged #${h.from}'s hit`); fx.push({name: 'dodge', at: {...me.pos}, until: tick + 15}); }
      else hurtMe(h.amount);
    });
    pendingHits = [];
    for (const e of out.effects || []) {
      const d = e.data, u = d && d.target !== undefined ? find(d.target) : null;
      switch (e.kind) {
        case 'damage': if (u) { u.hp = Math.max(0, u.hp - d.amount); hits.push({id: u.id, amount: d.amount, ally: u.team === 0, tower: u.tower, until: tick + 30});
          if (u.team === 0 || u.tower) log(`!! #${u.id} (${u.tower ? 'tower' : 'ally'}) hit by a skill (should never happen)`); } break;
        case 'slow': if (u) u.slowUntil = tick + d.ticks; break;
        case 'freeze': if (u) { u.stunUntil = tick + d.ticks; u.frozenUntil = tick + d.ticks; log(`#${d.target} frozen`); } break;
        case 'knock_up': if (u) { u.stunUntil = Math.max(u.stunUntil, tick + d.ticks); u.airborneUntil = tick + d.ticks; log(`#${d.target} knocked up ${(d.ticks / 60).toFixed(2)} s`); } break;
        case 'frost': if (u) { u.frost = d.stacks; u.frostUntil = tick + d.ticks; } break;
        case 'burn': if (u) u.burnUntil = tick + d.ticks; break;
        case 'weapon': me.blades = d.blades; log(`Weapon: ${d.blades ? 'Blades of Chaos' : 'Leviathan Axe'}`); break;
        case 'shield': me.shields.push({amount: d.amount, until: tick + d.ticks}); log(`Shield ${d.amount} (${(d.ticks / 60).toFixed(1)} s)`); break;
        case 'immune': me.immuneUntil = tick + d.ticks; log(`CC immune ${(d.ticks / 60).toFixed(2)} s`); break;
        case 'reduce': me.reduce = d.percent; me.reduceUntil = tick + d.ticks; break;
        case 'cleanse': me.stunUntil = 0; log('Cleansed'); break;
        case 'sight': me.sightUntil = tick + d.ticks; break;
        case 'glide': glide(me, d.to, d.ticks); break;
        case 'hop': glides = glides.filter(g => g.who !== me); me.pos = {...d.to}; log('Chain Dash: over the wall'); break;
        case 'pose': me.pose = d.name; me.poseUntil = tick + d.ticks; break;
        case 'fx': if (!/^krt_fenrir_p\d$/.test(d.name)) fx.push({name: d.name, at: {...d.at}, start: tick, until: tick + Math.max(6, d.ticks)}); break;
        case 'log': log(String(d)); break;
      }
    }
  }
  function step() {
    const text = JSON.stringify(frame()), bytes = new TextEncoder().encode(text);
    const ptr = wasm.kratos_input(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    const outPtr = wasm.kratos_step(bytes.length), n = wasm.kratos_output_len();
    const out = JSON.parse(new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, outPtr, n)));
    if (out.error) { log('Lab error: ' + out.error); return; }
    state = out.state;
    if (out.rejection && out.rejection !== lastRejection) log('Refused: ' + out.rejection);
    lastRejection = out.rejection || '';
    apply(out);
    tick++;
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)); g.who.pos = {x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k}; }
    glides = glides.filter(g => tick < g.t1);
    // his walk (blocked by the real walls)
    if (tick >= me.stunUntil && !glides.some(g => g.who === me)) {
      const dir = {x: (held.has('d') - held.has('a')), y: (held.has('s') - held.has('w'))};
      if (dir.x || dir.y) { const l = Math.hypot(dir.x, dir.y), v = R.stat.move_speed; const next = {x: me.pos.x + dir.x / l * v, y: me.pos.y + dir.y / l * v}; if (!wallAt(next)) me.pos = next; }
    }
    // the Lab's enemies: walk in, hit him (Heimdall's Sight decides on the next frame)
    if ($('moving').checked) {
      for (const u of units.filter(u => u.team === 1 && u.champion && u.hp > 0)) {
        if (tick < u.stunUntil) continue;
        const speed = tick < u.slowUntil ? 500 : 800, d = dist(u.pos, me.pos);
        if (!glides.some(g => g.who === u) && d > 20000) { const k = speed / d, next = {x: u.pos.x + (me.pos.x - u.pos.x) * k, y: u.pos.y + (me.pos.y - u.pos.y) * k}; if (!wallAt(next)) u.pos = next; }
        if (d <= 25000 && tick >= u.nextAttack) { u.nextAttack = tick + 60; pendingHits.push({from: u.id, amount: 60}); }
      }
    }
    for (const u of units) if (u.hp <= 0 && (u.team === 9 || !u.champion) && !u.tower) { u.hp = u.max_hp; }
    me.shields = me.shields.filter(s => tick < s.until && s.amount > 0);
    fx = fx.filter(f => tick < f.until);
    hits = hits.filter(h => tick < h.until);
  }
  function order(kind) {
    const o = {kind};
    if (['rampage', 'redirect', 'dash', 'fenrir'].includes(kind)) o.aim = rp(aim);
    queue.push(o);
  }
  function basic() {
    const reach = (me.blades ? R.basic.bladesRange : R.basic.axeRange) + 3000;
    const t = units.filter(u => u.team !== 0 && u.hp > 0 && dist(u.pos, me.pos) <= reach + u.radius).sort((a, b) => dist(a.pos, me.pos) - dist(b.pos, me.pos))[0];
    if (!t) return log(`Nothing within basic reach (${reach - 3000})`);
    t.hp = Math.max(0, t.hp - 100); hits.push({id: t.id, amount: 100, ally: false, until: tick + 30}); basicHits.push(t.id);
  }
  // ---------------------------------------------------------------- drawing (one world-to-screen scale for everything)
  let cam = {x: 480000, y: 480000, s: 900 / WORLD};
  const P = p => ({x: (p.x - cam.x) * cam.s + 450, y: (p.y - cam.y) * cam.s + 450});
  const W = r => r * cam.s;
  function circle(p, r, fill, stroke, width = 1) { const q = P(p); ctx.beginPath(); ctx.arc(q.x, q.y, Math.max(1.5, W(r)), 0, 7); if (fill) { ctx.fillStyle = fill; ctx.fill(); } if (stroke) { ctx.lineWidth = width; ctx.strokeStyle = stroke; ctx.stroke(); ctx.lineWidth = 1; } }
  function line(a, b, stroke, width = 1) { const p = P(a), q = P(b); ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(q.x, q.y); ctx.lineWidth = width; ctx.strokeStyle = stroke; ctx.stroke(); ctx.lineWidth = 1; }
  function toward(a, b, len) { const d = dist(a, b) || 1; return {x: a.x + (b.x - a.x) * len / d, y: a.y + (b.y - a.y) * len / d}; }
  function immuneUsed() {
    const list = state.immune || [], from = Math.max(0, tick - R.immune.windowTicks);
    return list.reduce((s, [a, b]) => s + Math.max(0, Math.min(b, tick) - Math.max(a, from)), 0);
  }
  function draw() {
    if ($('zoom').checked) cam = {x: me.pos.x, y: me.pos.y, s: 900 / 260000}; else cam = {x: 480000, y: 480000, s: 900 / WORLD};
    ctx.fillStyle = '#2a3340'; ctx.fillRect(0, 0, 900, 900);
    const o = P({x: 0, y: 0});
    if ($('art').checked && art.complete) { ctx.globalAlpha = .75; ctx.drawImage(art, o.x, o.y, W(WORLD), W(WORLD)); ctx.globalAlpha = 1; }
    ctx.fillStyle = 'rgba(20,20,26,.55)';
    for (let i = 0; i < N * N; i++) if (WALLS[i]) { const q = P({x: i % N * CELL, y: Math.floor(i / N) * CELL}); ctx.fillRect(q.x, q.y, W(CELL) + .5, W(CELL) + .5); }
    for (const [tx, ty] of TOWERS) { const q = P({x: tx, y: ty}); ctx.fillStyle = '#c9a94a'; ctx.fillRect(q.x - 4, q.y - 4, 8, 8); }
    if ($('ranges').checked) {
      const linked = state.link && state.link[0] === 'Guard' && tick < state.link[1];
      circle(me.pos, R.dash.range, null, linked ? 'rgba(248,113,113,.25)' : 'rgba(248,113,113,.7)');
      circle(me.pos, R.dash.linkedRange, null, linked ? 'rgba(248,113,113,.9)' : 'rgba(248,113,113,.25)');
      circle(me.pos, R.rampage.range, null, 'rgba(147,197,253,.6)');
      line(me.pos, toward(me.pos, aim, R.fenrir.distance), 'rgba(250,204,21,.35)', Math.max(1, W(R.fenrir.width * 2)));
      line(me.pos, toward(me.pos, aim, linked ? R.dash.linkedRange : R.dash.range), 'rgba(248,113,113,.7)', Math.max(1, W(R.dash.width * 2)));
    }
    for (const f of fx) {
      const age = (tick - (f.start || tick)) / Math.max(1, f.until - (f.start || tick));
      if (/cleave|sweep/.test(f.name)) circle(f.at, R.rampage.swingRadius, f.name.includes('cleave') ? 'rgba(125,211,252,.35)' : 'rgba(251,146,60,.35)', '#fff');
      else if (f.name === 'krt_smash') circle(f.at, R.rampage.smashRadius, 'rgba(192,132,252,.3)', '#e9d5ff', 2);
      else if (f.name === 'krt_chain') line(me.pos, f.at, '#f97316', 2);
      else if (f.name === 'krt_guard') circle(me.pos, 7000, 'rgba(226,232,240,.25)', '#e2e8f0', 2);
      else if (f.name === 'krt_sight' || f.name === 'dodge') circle(me.pos, 9000 + 3000 * age, null, '#facc15', 2);
      else if (f.name === 'krt_freeze') circle(f.at, 4000, 'rgba(186,230,253,.6)');
      else if (f.name.startsWith('krt_fenrir')) circle(f.at, R.fenrir.width, null, '#fde68a', 2);
      else circle(f.at, 3000, null, '#fca5a5');
    }
    if (state.fenrir) { const w = state.fenrir; circle(w.pos, R.fenrir.width, 'rgba(203,213,225,.5)', '#f8fafc', 2); line(w.from, w.to, 'rgba(248,250,252,.4)'); }
    for (const u of units.filter(u => u.hp > 0)) {
      const col = u.tower ? '#c9a94a' : u.team === 0 ? '#60a5fa' : u.team === 9 ? '#a3e635' : u.champion ? '#f87171' : '#fb923c';
      circle(u.pos, u.radius, tick < u.airborneUntil ? '#fff' : col, tick < u.frozenUntil ? '#7dd3fc' : null, 3);
      if (tick < u.burnUntil) circle(u.pos, u.radius + 1500, null, '#f97316', 2);
      const q = P(u.pos);
      for (let i = 0; i < (tick < u.frostUntil ? u.frost : 0); i++) { ctx.fillStyle = '#7dd3fc'; ctx.fillRect(q.x - 8 + i * 7, q.y - W(u.radius) - 14, 5, 5); }
      ctx.fillStyle = '#111'; ctx.fillRect(q.x - 14, q.y - W(u.radius) - 8, 28, 4); ctx.fillStyle = '#4ade80'; ctx.fillRect(q.x - 14, q.y - W(u.radius) - 8, 28 * u.hp / u.max_hp, 4);
    }
    // Kratos: body colour = weapon; HP bar with the shield (white) on top
    circle(me.pos, 3500, me.blades ? '#ea580c' : '#38bdf8', tick < me.immuneUntil ? '#facc15' : '#0b0f14', 3);
    const q = P(me.pos), sh = me.shields.reduce((s, x) => s + x.amount, 0);
    ctx.fillStyle = '#111'; ctx.fillRect(q.x - 22, q.y - W(3500) - 12, 44, 5);
    ctx.fillStyle = '#ef4444'; ctx.fillRect(q.x - 22, q.y - W(3500) - 12, 44 * me.hp / me.max_hp, 5);
    ctx.fillStyle = '#fff'; ctx.fillRect(q.x - 22 + 44 * me.hp / me.max_hp, q.y - W(3500) - 12, Math.min(44 * sh / me.max_hp, 44 - 44 * me.hp / me.max_hp + 10), 5);
    for (const h of hits) { const u = find(h.id); if (u) { const p = P(u.pos); ctx.fillStyle = '#fde68a'; ctx.font = '11px monospace'; ctx.fillText(h.amount, p.x + 8, p.y - 10 - (30 - (h.until - tick))); } }
    ctx.fillStyle = 'rgba(15,20,27,.8)'; ctx.fillRect(8, 870, 520, 22); ctx.fillStyle = '#cbd5e1'; ctx.font = '12px system-ui';
    ctx.fillText(`Map: game 5v5 (levi-map.js) · scale 1:${Math.round(1 / cam.s)} · enemies/minions/tower: Lab model`, 14, 885);
  }
  function hud() {
    const cd = i => Math.max(0, ((state.ready || [])[i] || 0) - tick) / 60;
    const used = immuneUsed();
    $('hud').innerHTML = [
      ['Weapon', me.blades ? 'Blades of Chaos' : 'Leviathan Axe'], ['HP', `${Math.round(me.hp)} / ${me.max_hp}`],
      ['Shield (white bar)', me.shields.reduce((s, x) => s + x.amount, 0)], ['Swap shields', `${(state.swap_shields || []).reduce((s, x) => s + x[0], 0)} / ${me.max_hp * R.passive.swapShieldCapPct / 100}`],
      ['CC immunity used', `${(used / 60).toFixed(2)} s of ${(R.immune.capTicks / 60)} s / 6 s`], ['Immune now', tick < me.immuneUntil ? 'yes' : 'no'],
      ['Link', state.link && tick < state.link[1] ? `${state.link[0]} (${((state.link[1] - tick) / 60).toFixed(1)} s)` : '—'], ['Rampage stage', state.rampage ? state.rampage.stage + 1 : '—'],
      ['Sight', cd(0).toFixed(1)], ['Rampage', cd(1).toFixed(1)], ['Guard', cd(2).toFixed(1)], ['Chain Dash', cd(3).toFixed(1)], ['Fenrir', cd(4).toFixed(1)],
      ['Fenrir now', state.fenrir ? (state.fenrir.leg ? 'slammed back' : 'charging in (4: leap)') : '—'],
    ].map(([k, v]) => `<span>${k}</span><b>${v}</b>`).join('');
    $('status').textContent = `tick ${tick} · ${running ? 'running' : 'paused'} · rules ${R.revision}`;
  }
  function frameLoop() { if (running) step(); draw(); hud(); requestAnimationFrame(frameLoop); }
  canvas.addEventListener('mousemove', e => { const r = canvas.getBoundingClientRect(); aim = {x: (e.clientX - r.left - 450) / cam.s + cam.x, y: (e.clientY - r.top - 450) / cam.s + cam.y}; });
  addEventListener('keydown', e => {
    const k = e.key.toLowerCase();
    if ('wasd'.includes(k) && k.length === 1) held.add(k);
    if (k === '1') order('sight'); if (k === '2') order('rampage'); if (k === 'r') order('redirect'); if (k === '3') order('guard'); if (k === 'e') order('dash');
    if (k === '4') order(state.fenrir && !state.fenrir.leg ? 'slam' : 'fenrir'); if (k === 'f') basic();
  });
  addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  $('play').onclick = () => { running = !running; $('play').textContent = running ? 'Pause' : 'Play'; };
  $('step').onclick = () => { step(); draw(); hud(); };
  $('reset').onclick = () => reset();
  $('stunMe').onclick = () => { if (tick < me.immuneUntil) log('Kratos is CC-immune: the stun does nothing'); else { me.stunUntil = tick + 60; log('Kratos stunned 1 s'); } };
  $('wallDemo').onclick = () => { const c = wallCrossing(); me.pos = {...c.from}; find(2).pos = {...c.to}; find(2).hp = 1000; aim = {...c.to}; log('A foe across the wall: Chain Dash (E) hops over it'); };
  $('group').onclick = () => { units.filter(u => u.team === 1 && u.champion).forEach((u, i) => { u.pos = toward(me.pos, aim, 40000 + i * 12000); }); log('Enemies lined up toward the mouse'); };
  reset();
  window.kratosLab = {step, reset, order, basic, wallCrossing, pause() { running = false; }, setAim(p) { aim = {...p}; }, stunMe() { $('stunMe').onclick(); },
    get state() { return state; }, get me() { return me; }, get units() { return units; }, get hits() { return hits; }, get tick() { return tick; }, get walls() { return WALLS; }, immuneUsed, wallAt};
  requestAnimationFrame(frameLoop);
})();
