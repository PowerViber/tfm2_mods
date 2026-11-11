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
  const clock=new WrightClock();
  const painter=new WrightPainter();let recorder=null,visual;
  let tick, me, units, carriers, glides, state, queue, fx, logLines, attackTarget, hits, guard, taken, running = true, aim = {x: 480000, y: 480000}, held = new Set();
  const unit = (id, team, x, y, champion = true, farmable = false) => ({id, team, pos: {x, y}, hp: 1000, max_hp: 1000, radius: 8000, champion, tower: false, visible: true, targetable: true, farmable, slowUntil: 0, stunUntil: 0});
  function reset() {
    clock.reset();wasm.wright_reset();
    tick = 0; state = {}; queue = []; glides = []; fx = []; logLines = []; carriers = new Map(); attackTarget = null; hits = []; guard = 0; taken = 0;
    visual={pose:'idle',start:0,until:0,left:false,moving:false,shields:new Map(),burns:new Map(),deadSince:null};
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
      attack_target: attackTarget, damage_taken: (() => { const t = taken; taken = 0; return t; })(), manual: !$('auto').checked, order: queue.shift() || null}};
  }
  const calm = () => !units.some(u => u.team === 1 && u.champion && u.hp > 0 && dist(u.pos, me.pos) < 90000);
  function apply(out) {
    for (const e of out.effects || []) {
      const d = e.data;
      switch (e.kind) {
        case 'damage': { const u = find(d.target); if (u) { u.hp = Math.max(0, u.hp - d.amount); hits.push({id: u.id, amount: d.amount, ally: u.team === 0, until: tick + 30}); if (u.team === 0) log(`!! ally #${u.id} hit by Wright (should never happen)`); } break; }
        case 'stun': { const u = find(d.target); if (u) u.stunUntil = tick + d.ticks; log(`#${d.target} stunned`); break; }
        case 'guard': guard = d.on ? d.pct : 0; log(d.on ? `Carrier guard on: it takes ${d.pct}% of his damage` : 'Carrier guard off'); break;
        case 'slow': { const u = find(d.target); if (u) u.slowUntil = tick + d.ticks; break; }
        case 'knock': { const u = find(d.target); if (u) glide(u, d.to, d.ticks); log(`#${d.target} knocked back`); break; }
        case 'heal': { const u = d.target === 1 ? me : find(d.target); if (u) u.hp = Math.min(u.max_hp, u.hp + d.amount); break; }
        case 'shield': visual.shields.set(d.target,tick+d.ticks);log(`#${d.target} shield ${d.amount}`); break;
        case 'burn': visual.burns.set(d.target,tick+d.ticks);log(`#${d.target} burning`); break;
        case 'taunt': log(`#${d.target} taunted onto the drone`); break;
        case 'spawn_plane': carriers.set(d.key, {key: d.key, kind: d.kind, pos: {...d.pos}, hp: me.max_hp * d.hp_pct / 100, max_hp: me.max_hp * d.hp_pct / 100, until: tick + d.ticks}); break;
        case 'plane_move': { const c = carriers.get(d.key); if (c) glide(c, d.to, d.ticks); break; }
        case 'remove_plane': carriers.delete(d.key); break;
        case 'repair_plane': { const c = carriers.get(d.key); if (c) c.hp = Math.min(c.max_hp, c.hp + c.max_hp * d.pct / 100); break; }
        case 'stock': log(`Airframes: ${d.count}`); break;
        case 'pose': visual.pose=d.name;visual.start=tick;visual.until=tick+d.ticks;break;
        case 'fx': {
          if (/_p\d+$/.test(d.name)||d.name==='wrt_carrier') break;
          const target=d.name==='wrt_carrier_shadow'?1:null;
          fx.push({name:d.name,at:{...d.at},start:tick,ticks:d.ticks,target,until:tick+Math.max(1,d.ticks)});break;
        }
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
    if(recorder)recorder(text,JSON.stringify(out));
    state = out.state;
    apply(out);
    tick++;
    // the host: glides, his walk, enemies walking in and shooting his planes, carriers expiring
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)); g.who.pos = {x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k}; }
    glides = glides.filter(g => tick < g.t1);
    const previous={...me.pos};
    const v = 1000, dir = {x: (held.has('d') - held.has('a')), y: (held.has('s') - held.has('w'))};
    if (dir.x || dir.y) { const l = Math.hypot(dir.x, dir.y); me.pos = {x: Math.max(2000, Math.min(958000, me.pos.x + dir.x / l * v)), y: Math.max(2000, Math.min(958000, me.pos.y + dir.y / l * v))}; }
    visual.moving=dist(previous,me.pos)>0;if(previous.x!==me.pos.x)visual.left=me.pos.x<previous.x;
    if(me.hp<=0&&visual.deadSince===null)visual.deadSince=tick;
    if ($('moving').checked) {
      for (const u of units.filter(u => u.team === 1 && u.hp > 0)) {
        const speed = tick < u.slowUntil ? 500 : 650;
        if (tick < u.stunUntil) continue;
        if (!glides.some(g => g.who === u) && dist(u.pos, me.pos) > 45000) { const k = speed / dist(u.pos, me.pos); u.pos = {x: u.pos.x + (me.pos.x - u.pos.x) * k, y: u.pos.y + (me.pos.y - u.pos.y) * k}; }
        if ($('incomingFire').checked && tick % 50 === 0) {
          const c = [...carriers.values()].filter(c => c.hp > 0 && dist(c.pos, u.pos) < 45000).sort((a, b) => dist(a.pos, u.pos) - dist(b.pos, u.pos))[0];
          if (c) c.hp -= 30; else if (dist(me.pos, u.pos) < 45000) hitMe(40);
        }
      }
    }
    for (const [k, c] of carriers) if (c.hp <= 0 || tick >= c.until) carriers.delete(k);
    for (const u of units) if (u.hp <= 0 && u.team === 9) { u.hp = u.max_hp; }
    fx = fx.filter(f => tick < f.until&&me.hp>0);
    hits = hits.filter(h => tick < h.until);
  }
  // a hit on Wright: the carrier's guard takes its share, the rest is reported to the core
  function hitMe(raw) { const dmg = Math.round(raw * (100 - guard) / 100); me.hp = Math.max(1, me.hp - dmg); taken += dmg; return dmg; }
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
    const art=painter.loaded&&$('art').checked,pixel=W(950);
    ctx.imageSmoothingEnabled=false;
    const t = terrain(); ctx.fillStyle = '#4b5563';
    t.forEach((w, i) => { if (w) { const q = P({x: (i % 30) * 32000, y: Math.floor(i / 30) * 32000}); ctx.fillRect(q.x, q.y, W(32000), W(32000)); } });
    for (const f of fx) {
      const point=f.target===1?me.pos:f.target!==null&&f.target!==undefined?(find(f.target)?.pos||f.at):f.at;
      if(art){
        const tag=painter.effectTag(f.name,f.ticks);
        const q=P(point);painter.effect(ctx,tag,tick-f.start,q.x,q.y,pixel);continue;
      }
      if (/^wrt_warn[123]?$/.test(f.name)) circle(point,R.bomber.radius,'rgba(248,113,113,.08)','#f87171');
      else if (f.name==='wrt_carrier_shadow')circle(point,R.carrier.knockRadius,'rgba(0,0,0,.25)','#9ca3af');
      else if (f.name==='wrt_knock_ring')circle(point,R.carrier.knockRadius,null,'#fde68a');
      else if (f.name==='wrt_bomb'||f.name==='wrt_kamikaze')circle(point,f.name==='wrt_bomb'?10000:R.fighter.kamikazeRadius,'rgba(251,146,60,.45)');
      else if (f.name.startsWith('wrt_bomber_wave'))circle(point,R.bomber.radius,'rgba(251,146,60,.25)','#ffb955');
      else if (f.name==='wrt_lock')circle(point,8000,null,'#ff874b');
    }
    if(state.carrier&&tick>=state.carrier.land_at&&tick<state.carrier.until){
      if(art){const q=P(state.carrier.center);painter.effect(ctx,'carrier',0,q.x,q.y,pixel);}
      else circle(state.carrier.center,R.carrier.kitRadius,'rgba(148,163,184,.12)','#94a3b8');
    }
    for (const u of units.filter(u => u.hp > 0)) {
      circle(u.pos, u.radius, u.team === 0 ? '#60a5fa' : u.team === 9 ? '#a3e635' : '#f87171', tick < u.slowUntil ? '#93c5fd' : null);
      const q = P(u.pos); ctx.fillStyle = '#fff'; ctx.fillText(`#${u.id} ${Math.round(u.hp)}`, q.x - 18, q.y - W(u.radius) - 4);
    }
    if(art){const q=P(me.pos),pose=me.hp<=0?'dead':tick<visual.until?visual.pose:visual.moving?'run':'idle';painter.body(ctx,pose,pose==='dead'?tick-visual.deadSince:tick<visual.until?tick-visual.start:tick,q.x,q.y,pixel,visual.left,pose==='run'||pose==='idle');if(me.hp>0)painter.effect(ctx,'airframes'+(state.stock??0),0,q.x,q.y,pixel);}
    else circle(me.pos,8000,'#f59e0b','#fff');
    // the carrier's guard: its pool over him
    if (state.carrier && state.carrier.guarding) { const q = P(me.pos); ctx.fillStyle = '#111827'; ctx.fillRect(q.x - 30, q.y - W(14000) - 10, 60, 5); ctx.fillStyle = '#38bdf8'; ctx.fillRect(q.x - 30, q.y - W(14000) - 10, 60 * Math.max(0, state.carrier.pool / state.carrier.max_pool), 5); }
    // hit markers: who took damage from him (magenta on an ally: should never happen)
    for (const h of hits) { const u = find(h.id); if (!u) continue; const q = P(u.pos), k = (h.until - tick) / 30;
      ctx.strokeStyle = h.ally ? `rgba(255,0,255,${k})` : `rgba(239,68,68,${k})`; ctx.beginPath(); ctx.arc(q.x, q.y, W(u.radius) + 4, 0, 7); ctx.stroke();
      ctx.fillStyle = `rgba(254,202,202,${k})`; ctx.fillText('-' + h.amount, q.x + 10, q.y - 12 - (1 - k) * 18); }
    for (const p of state.planes || []) {
      // a sortie's committed spot (no homing): a small cross where it will strike
      if (p.kind === 'fighter' && p.phase === 'Run' && p.goal) { const g = P(p.goal); ctx.strokeStyle = 'rgba(253,224,71,.85)'; ctx.beginPath(); ctx.moveTo(g.x - 5, g.y - 5); ctx.lineTo(g.x + 5, g.y + 5); ctx.moveTo(g.x + 5, g.y - 5); ctx.lineTo(g.x - 5, g.y + 5); ctx.stroke(); }
      const host = carriers.get(p.key), at = host ? host.pos : p.pos;
      if(tick<p.start)continue;
      if(art){const q=P(at);painter.effect(ctx,painter.planeTag(p,tick-1),0,q.x,q.y,pixel);}
      else plane(at,p.kind,p.heading,p.kind==='bomber'?9000:p.kind==='rocket'?2500:5000);
      // round 160: a jet's ammo pips (3 strikes per trip, then home to rearm)
      if (p.kind === 'fighter' && !p.kamikaze) { const q = P(at); for (let i = 0; i < (p.ammo ?? 0); i++) { ctx.fillStyle = '#fde047'; ctx.fillRect(q.x - 6 + i * 5, q.y - 12, 3, 3); } }
      if (host) { const q = P(at); ctx.fillStyle = '#22c55e'; ctx.fillRect(q.x - 8, q.y + 8, 16 * Math.max(0, host.hp / host.max_hp), 2); }
    }
    if(art)for(const [map,tag] of [[visual.shields,'repair_shield'],[visual.burns,'burn']]){
      for(const [id,until] of map){if(tick>=until){map.delete(id);continue;}const u=id===1?me:find(id);if(u&&u.hp>0){const q=P(u.pos);painter.effect(ctx,tag,tick,q.x,q.y,pixel,true);}}
    }
    $('hud').innerHTML = [`Tick ${tick}`, `Airframes ${state.stock ?? '-'}`, `Fighters ${(state.planes || []).filter(p => p.kind === 'fighter' && !p.from_carrier && !p.kamikaze).length}`,
      `Carrier ${state.carrier ? 'up: skills empowered (no airframe, half cooldown)' : '-'}`, ...['S1', 'S2', 'S3', 'U'].map((n, i) => `${n} ${state.ready ? Math.max(0, ((state.ready[i] - tick) / 60)).toFixed(1) + ' s' : '-'}`), `Wright HP ${Math.round(me.hp)}`].map(s => `<span>${s}</span>`).join('');
    $('status').textContent = state.rejection ? 'Last rejected: ' + state.rejection : running?'Running':'Paused';
  }
  function loop(now) { clock.advance(now,running&&!document.hidden,step); draw(); requestAnimationFrame(loop); }
  window.addEventListener('blur',()=>held.clear());
  canvas.addEventListener('blur',()=>held.clear());
  document.addEventListener('visibilitychange',()=>clock.reset());
  canvas.addEventListener('mousemove', e => { const r = canvas.getBoundingClientRect(); aim = {x: (e.clientX - r.left - 450) / cam.s + cam.x, y: (e.clientY - r.top - 450) / cam.s + cam.y}; });
  canvas.addEventListener('keydown', e => {
    const k = e.key.toLowerCase(); held.add(k);
    if (k === '1') order('fighters'); if (k === '2') order('bomber'); if (k === '3') order('repair'); if (k === '4') order('carrier');
    if (k === 'f') { const t = units.filter(u => u.team !== 0 && u.hp > 0).sort((a, b) => dist(a.pos, me.pos) - dist(b.pos, me.pos))[0]; if (t && dist(t.pos, me.pos) < R.basic.range + 8000) { attackTarget = t.id; t.hp -= R.basic.base + R.basic.apRatio;visual.pose='attack';visual.start=tick;visual.until=tick+22;visual.left=t.pos.x<me.pos.x; } }
  });
  canvas.addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  $('play').onclick = () => { running = !running; $('play').textContent = running ? 'Pause' : 'Play'; };
  $('step').onclick = () => { running=false;clock.reset();$('play').textContent='Play';step();draw(); };
  $('reset').onclick = () => { reset(); canvas.focus(); };
  $('hurtAlly').onclick = () => { const a = find(4); a.hp = 250; log('The far ally drops to 25%'); };
  $('hurtMe').onclick = () => { me.hp = 450; log('Wright drops to 45%'); };
  $('group').onclick = () => { units.filter(u => u.team === 1).forEach((u, i) => u.pos = {x: me.pos.x + 90000 + i * 6000, y: me.pos.y + i * 5000}); log('Enemies grouped'); };
  $('damagePlanes').onclick=()=>{for(const c of carriers.values())if(c.kind==='fighter')c.hp=Math.max(1,Math.floor(c.max_hp*.2));log('Lab fixture: fighter HP reduced to 20%');};
  $('demoRun').onclick=()=>{
    reset();$('auto').checked=false;$('moving').checked=false;$('incomingFire').checked=false;
    for(const u of units.filter(u=>u.team===1)){u.hp=u.max_hp=100000;}
    const demo=$('demo').value;aim={x:me.pos.x+90000,y:me.pos.y};
    if(demo==='escort'){for(const u of units.filter(u=>u.team===1))u.pos={x:800000,y:800000};order('fighters');}
    if(demo==='rockets'||demo==='kamikaze'){units[0].pos={x:me.pos.x+30000,y:me.pos.y};units[1].pos={x:800000,y:800000};order('fighters');}
    if(demo==='kamikaze'){for(let i=0;i<3;i++)step();$('damagePlanes').onclick();}
    if(demo==='bomber'){units[0].pos={x:aim.x-35000,y:aim.y};units[1].pos={x:aim.x,y:aim.y+35000};order('bomber');}
    if(demo==='repair'){me.hp=450;find(4).hp=250;order('repair');}
    if(demo==='drone'){for(let i=0;i<841;i++)step();log('Lab preview fast-forwarded the real 14-second workshop clock');}
    if(demo==='carrier'){$('moving').checked=true;order('carrier');}
    running=true;clock.reset();$('play').textContent='Pause';canvas.focus();
  };
  window.wrightLab = {get state() { return state; }, get tick() { return tick; }, get me() { return me; }, get units() { return units; }, get carriers() { return carriers; }, get hits() { return hits; }, hitMe,
    painter,step,reset,order,setRecorder:fn=>{recorder=fn;},setAim: p => { aim = {...p}; }, pause: () => { running = false;clock.reset();$('play').textContent = 'Play'; }};
  reset(); canvas.focus(); requestAnimationFrame(loop);
})();
