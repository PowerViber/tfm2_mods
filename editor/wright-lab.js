// Wright Lab: the game's exact core (wright.wasm) with a small host model around it:
// positions, engine glides, the plane carrier units, simple enemies, an ally and a camp.
(async () => {
  const $ = id => document.getElementById(id);
  const R = window.WRIGHT_RULES, canvas = $('arena'), ctx = canvas.getContext('2d');
  const wasm = (await WebAssembly.instantiate(await (await fetch('wright.wasm')).arrayBuffer(), {})).instance.exports;
  const names = ['ego', 'judgement', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid', 'aggressive', 'order', 'roaming', 'macro', 'micro', 'awareness', 'strategy'];
  $('stats').innerHTML = names.map(n => `<span>${n}</span><input type="number" min="0" max="100" value="50">`).join('');
  const rp = p => ({x: Math.round(p.x), y: Math.round(p.y)});
  const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  let tick, me, units, carriers, glides, state, queue, fx, logLines, attackTarget, running = true, aim = {x: 480000, y: 480000}, held = new Set();
  const unit = (id, team, x, y, champion = true, farmable = false) => ({id, team, pos: {x, y}, hp: 1000, max_hp: 1000, radius: 8000, champion, tower: false, visible: true, targetable: true, farmable, slowUntil: 0});
  function reset() {
    wasm.wright_reset();
    tick = 0; state = {}; queue = []; glides = []; fx = []; logLines = []; carriers = new Map(); attackTarget = null;
    me = {pos: {x: 300000, y: 480000}, hp: 1000, max_hp: 1000};
    units = [unit(2, 1, 400000, 470000), unit(3, 1, 410000, 500000), unit(4, 0, 700000, 760000), unit(20, 9, 330000, 400000, false, true)];
    log('Reset: two enemies east, a far ally to the south-east, a camp to the north');
  }
  function log(s) { logLines.push(`[${tick}] ${s}`); if (logLines.length > 200) logLines.shift(); $('log').textContent = logLines.slice(-60).join('\n'); $('log').scrollTop = 1e9; }
  const find = id => units.find(u => u.id === id);
  function glide(who, to, ticks) { glides = glides.filter(g => g.who !== who); glides.push({who, from: {...who.pos}, to: {...to}, t0: tick, t1: tick + Math.max(1, ticks)}); }
  function terrain() {
    if (!$('walls').checked) return [];
    const t = new Array(900).fill(false);
    for (let y = 8; y < 22; y++) t[y * 30 + 15] = true;
    return t;
  }
  function frame() {
    const profile = [...$('stats').querySelectorAll('input')].map(i => Math.max(0, Math.min(100, +i.value || 0)));
    return {version: 1, rules_revision: R.revision, frame: {
      tick, owner: 1, team: 0, alive: me.hp > 0, pos: rp(me.pos), hp: Math.round(me.hp), max_hp: me.max_hp, ap: 100, stunned: false, ult_unlocked: true,
      units: units.filter(u => u.hp > 0).map(u => ({id: u.id, team: u.team, pos: rp(u.pos), hp: Math.round(u.hp), max_hp: u.max_hp, radius: u.radius,
        champion: u.champion, tower: false, visible: true, targetable: true, farmable: u.farmable && calm()})),
      terrain: terrain(), profile,
      planes: [...carriers.values()].filter(c => c.hp > 0 && tick < c.until).map(c => ({key: c.key, pos: rp(c.pos), hp: Math.round(c.hp), max_hp: c.max_hp})),
      attack_target: attackTarget, manual: !$('auto').checked, order: queue.shift() || null}};
  }
  const calm = () => !units.some(u => u.team === 1 && u.champion && u.hp > 0 && dist(u.pos, me.pos) < 90000);
  function apply(out) {
    for (const e of out.effects || []) {
      const d = e.data;
      switch (e.kind) {
        case 'damage': { const u = find(d.target); if (u) u.hp = Math.max(0, u.hp - d.amount); break; }
        case 'slow': { const u = find(d.target); if (u) u.slowUntil = tick + d.ticks; break; }
        case 'knock': { const u = find(d.target); if (u) glide(u, d.to, d.ticks); log(`#${d.target} knocked back`); break; }
        case 'heal': { const u = d.target === 1 ? me : find(d.target); if (u) u.hp = Math.min(u.max_hp, u.hp + d.amount); break; }
        case 'shield': log(`#${d.target} shield ${d.amount}`); break;
        case 'burn': log(`#${d.target} burning`); break;
        case 'taunt': log(`#${d.target} taunted onto the drone`); break;
        case 'spawn_plane': carriers.set(d.key, {key: d.key, kind: d.kind, pos: {...d.pos}, hp: me.max_hp * d.hp_pct / 100, max_hp: me.max_hp * d.hp_pct / 100, until: tick + d.ticks}); break;
        case 'plane_move': { const c = carriers.get(d.key); if (c) glide(c, d.to, d.ticks); break; }
        case 'remove_plane': carriers.delete(d.key); break;
        case 'repair_plane': { const c = carriers.get(d.key); if (c) c.hp = Math.min(c.max_hp, c.hp + c.max_hp * d.pct / 100); break; }
        case 'stock': log(`Airframes: ${d.count}`); break;
        case 'fx': fx.push({name: d.name, at: {...d.at}, until: tick + Math.max(1, d.ticks)}); break;
        case 'log': log(String(d)); break;
      }
    }
  }
  function step() {
    const text = JSON.stringify(frame()), bytes = new TextEncoder().encode(text);
    const ptr = wasm.wright_input(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    const outPtr = wasm.wright_step(bytes.length), n = wasm.wright_output_len();
    const out = JSON.parse(new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, outPtr, n)));
    if (out.error) { log('Lab error: ' + out.error); return; }
    state = out.state;
    apply(out);
    tick++;
    // the host: glides, his walk, enemies walking in and shooting his planes, carriers expiring
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)); g.who.pos = {x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k}; }
    glides = glides.filter(g => tick < g.t1);
    const v = 1000, dir = {x: (held.has('d') - held.has('a')), y: (held.has('s') - held.has('w'))};
    if (dir.x || dir.y) { const l = Math.hypot(dir.x, dir.y); me.pos = {x: Math.max(2000, Math.min(958000, me.pos.x + dir.x / l * v)), y: Math.max(2000, Math.min(958000, me.pos.y + dir.y / l * v))}; }
    if ($('moving').checked) {
      for (const u of units.filter(u => u.team === 1 && u.hp > 0)) {
        const speed = tick < u.slowUntil ? 500 : 650;
        if (!glides.some(g => g.who === u) && dist(u.pos, me.pos) > 45000) { const k = speed / dist(u.pos, me.pos); u.pos = {x: u.pos.x + (me.pos.x - u.pos.x) * k, y: u.pos.y + (me.pos.y - u.pos.y) * k}; }
        if (tick % 50 === 0) {
          const c = [...carriers.values()].filter(c => c.hp > 0 && dist(c.pos, u.pos) < 45000).sort((a, b) => dist(a.pos, u.pos) - dist(b.pos, u.pos))[0];
          if (c) c.hp -= 30; else if (dist(me.pos, u.pos) < 45000) me.hp = Math.max(1, me.hp - 20);
        }
      }
    }
    for (const [k, c] of carriers) if (c.hp <= 0 || tick >= c.until) carriers.delete(k);
    for (const u of units) if (u.hp <= 0 && u.team === 9) { u.hp = u.max_hp; }
    fx = fx.filter(f => tick <= f.until);
  }
  function order(kind) { queue.push({kind, aim: rp(aim)}); }
  // ---------------------------------------------------------------- drawing
  let cam = {x: 480000, y: 480000, s: 900 / 960000};
  const P = p => ({x: (p.x - cam.x) * cam.s + 450, y: (p.y - cam.y) * cam.s + 450});
  const W = r => r * cam.s;
  function circle(p, r, fill, stroke) { const q = P(p); ctx.beginPath(); ctx.arc(q.x, q.y, Math.max(1.5, W(r)), 0, 7); if (fill) { ctx.fillStyle = fill; ctx.fill(); } if (stroke) { ctx.strokeStyle = stroke; ctx.stroke(); } }
  const COLORS = {fighter: '#e5e7eb', drone: '#fbbf24', bomber: '#9ca3af', chopper: '#34d399', repair: '#6ee7b7', rocket: '#f87171'};
  function plane(p, kind, heading, size) {
    const q = P(p), a = heading * Math.PI / 4, s = Math.max(4, W(size));
    ctx.save(); ctx.translate(q.x, q.y); ctx.rotate(a); ctx.beginPath(); ctx.moveTo(s, 0); ctx.lineTo(-s * .7, s * .6); ctx.lineTo(-s * .4, 0); ctx.lineTo(-s * .7, -s * .6); ctx.closePath();
    ctx.fillStyle = COLORS[kind] || '#fff'; ctx.fill(); ctx.restore();
  }
  function draw() {
    if ($('zoom').checked) cam = {x: me.pos.x, y: me.pos.y, s: 900 / 320000}; else cam = {x: 480000, y: 480000, s: 900 / 960000};
    ctx.clearRect(0, 0, 900, 900);
    const t = terrain(); ctx.fillStyle = '#4b5563';
    t.forEach((w, i) => { if (w) { const q = P({x: (i % 30) * 32000, y: Math.floor(i / 30) * 32000}); ctx.fillRect(q.x, q.y, W(32000), W(32000)); } });
    for (const f of fx) {
      if (f.name === 'wrt_warn') circle(f.at, R.bomber.orbitRadius, 'rgba(248,113,113,.08)', '#f87171');
      else if (f.name === 'wrt_carrier_shadow') circle(f.at, R.carrier.knockRadius, 'rgba(0,0,0,.25)', '#111827');
      else if (f.name === 'wrt_knock_ring') circle(f.at, R.carrier.knockRadius, null, '#fde68a');
      else if (f.name === 'wrt_bomb' || f.name === 'wrt_kamikaze') circle(f.at, f.name === 'wrt_bomb' ? R.bomber.bombRadius : R.fighter.kamikazeRadius, 'rgba(251,146,60,.45)');
      else if (f.name === 'wrt_lock') { const q = P(f.at); ctx.strokeStyle = '#ef4444'; ctx.strokeRect(q.x - 9, q.y - 9, 18, 18); }
      else if (f.name === 'wrt_repair_kit') circle(f.at, 5000, 'rgba(52,211,153,.6)');
      else if (f.name === 'wrt_mg') circle(f.at, 2500, '#fde047');
      else if (f.name === 'wrt_carrier') { const q = P(f.at); ctx.fillStyle = 'rgba(148,163,184,.85)'; ctx.fillRect(q.x - W(30000), q.y - W(9000), W(60000), W(18000)); ctx.strokeStyle = '#e2e8f0'; ctx.strokeRect(q.x - W(30000), q.y - W(9000), W(60000), W(18000)); }
    }
    for (const u of units.filter(u => u.hp > 0)) {
      circle(u.pos, u.radius, u.team === 0 ? '#60a5fa' : u.team === 9 ? '#a3e635' : '#f87171', tick < u.slowUntil ? '#93c5fd' : null);
      const q = P(u.pos); ctx.fillStyle = '#fff'; ctx.fillText(`#${u.id} ${Math.round(u.hp)}`, q.x - 18, q.y - W(u.radius) - 4);
    }
    circle(me.pos, 8000, '#f59e0b', '#fff');
    for (const p of state.planes || []) {
      const host = carriers.get(p.key), at = host ? host.pos : p.pos;
      plane(at, p.kind, p.heading, p.kind === 'bomber' ? 9000 : p.kind === 'rocket' ? 2500 : 5000);
      if (host) { const q = P(at); ctx.fillStyle = '#22c55e'; ctx.fillRect(q.x - 8, q.y + 8, 16 * Math.max(0, host.hp / host.max_hp), 2); }
    }
    $('hud').innerHTML = [`Tick ${tick}`, `Airframes ${state.stock ?? '-'}`, `Fighters ${(state.planes || []).filter(p => p.kind === 'fighter' && !p.from_carrier && !p.kamikaze).length}`,
      `Carrier ${state.carrier ? 'up' : '-'}`, ...['S1', 'S2', 'S3', 'U'].map((n, i) => `${n} ${state.ready ? Math.max(0, ((state.ready[i] - tick) / 60)).toFixed(1) + ' s' : '-'}`), `Wright HP ${Math.round(me.hp)}`].map(s => `<span>${s}</span>`).join('');
    $('status').textContent = state.rejection ? 'Last rejected: ' + state.rejection : 'Running';
  }
  function loop() { if (running) step(); draw(); requestAnimationFrame(loop); }
  canvas.addEventListener('mousemove', e => { const r = canvas.getBoundingClientRect(); aim = {x: (e.clientX - r.left - 450) / cam.s + cam.x, y: (e.clientY - r.top - 450) / cam.s + cam.y}; });
  canvas.addEventListener('keydown', e => {
    const k = e.key.toLowerCase(); held.add(k);
    if (k === '1') order('fighters'); if (k === '2') order('bomber'); if (k === '3') order('repair'); if (k === '4') order('carrier');
    if (k === 'f') { const t = units.filter(u => u.team !== 0 && u.hp > 0).sort((a, b) => dist(a.pos, me.pos) - dist(b.pos, me.pos))[0]; if (t && dist(t.pos, me.pos) < R.basic.range + 8000) { attackTarget = t.id; t.hp -= R.basic.base + R.basic.apRatio; } }
  });
  canvas.addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  $('play').onclick = () => { running = !running; $('play').textContent = running ? 'Pause' : 'Play'; };
  $('step').onclick = () => { step(); draw(); };
  $('reset').onclick = () => { reset(); canvas.focus(); };
  $('hurtAlly').onclick = () => { const a = find(4); a.hp = 250; log('The far ally drops to 25%'); };
  $('hurtMe').onclick = () => { me.hp = 450; log('Wright drops to 45%'); };
  $('group').onclick = () => { units.filter(u => u.team === 1).forEach((u, i) => u.pos = {x: me.pos.x + 90000 + i * 6000, y: me.pos.y + i * 5000}); log('Enemies grouped'); };
  window.wrightLab = {get state() { return state; }, get tick() { return tick; }, get me() { return me; }, get units() { return units; }, get carriers() { return carriers; },
    step, reset, order, setAim: p => { aim = {...p}; }, pause: () => { running = false; $('play').textContent = 'Play'; }};
  reset(); canvas.focus(); requestAnimationFrame(loop);
})();
