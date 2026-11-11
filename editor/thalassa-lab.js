// Thalassa Lab: the game's own core (thalassa.wasm, built from native/thalassa_lab) drives her kit; this page models the
// host: positions, engine glides, the clone unit, simple enemies and an ally.
(async () => {
  const R = window.THALASSA_RULES, REV = R.revision, $ = s => document.querySelector(s);
  const canvas = $('#arena'), ctx = canvas.getContext('2d'), W = canvas.width, MAP = 960000;
  const painter = new ThalassaPainter(), roundArt=new Round165Painter();roundArt.ready.catch(console.warn);
  const clock = new ThalassaClock();
  let recorder=null;
  let S = W / MAP, cam = {x: MAP / 2, y: MAP / 2};
  const wasm = (await WebAssembly.instantiate(await (await fetch('thalassa.wasm')).arrayBuffer(), {})).instance.exports;
  const names = ['ego', 'judgement', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid', 'aggressive', 'order', 'roaming', 'macro', 'micro', 'awareness', 'strategy'];
  $('#stats').innerHTML = names.map(n => `<span>${n}</span><input type="number" min="0" max="100" value="50">`).join('');
  let tick, units, me, state, logLines, queue, glides, clone, echo = null, visual, running = true, aim = {x: 480000, y: 480000}, held = new Set(), basicHits = [], banished = 0, nextBasic=0, bolts=[];
  const myShield = () => { me.shields = me.shields.filter(s => s.until > tick); return me.shields.reduce((a, s) => a + s.amount, 0); };
  const unit = (id, team, x, y, champion = true) => ({id, team, pos: {x, y}, hp: 1000, max_hp: 1000, radius: 8000, champion, tower: false, visible: true, targetable: true, farmable: false, stunned: false, stunUntil: 0, velocity: {x: 0, y: 0}});
  function reset() {
    clock.reset();
    wasm.thalassa_reset();
    tick = 0; state = {}; logLines = []; queue = []; glides = []; clone = null; echo = null; banished = 0;nextBasic=0;bolts=[];
    visual = {pose:'idle',start:0,until:0,left:false,cloneLeft:false,echoPose:'idle',echoStart:0,echoUntil:0,echoLeft:false,moving:false,fx:[],deadSince:null};
    me = {pos: {x: 300000, y: 480000}, hp: 1000, max_hp: 1000, stunUntil: 0, shields: []};
    units = [unit(5, 1, 470000, 470000), unit(6, 1, 480000, 500000), unit(7, 0, 330000, 520000)];
    log('Reset. Thalassa (blue) vs two enemies (red), one ally (green).');
  }
  function log(s) { logLines.push((tick / 60).toFixed(2) + 's ' + s); logLines = logLines.slice(-80); $('#log').textContent = logLines.join('\n'); $('#log').scrollTop = 1e9; }
  const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  const terrain = () => { const t = Array(900).fill(false); if ($('#walls').checked) for (let y = 10; y < 20; y++) t[y * 30 + 18] = true; return t; };
  function call(input) {
    const bytes = new TextEncoder().encode(JSON.stringify(input));
    const at = wasm.thalassa_input(bytes.length); // may grow memory: take the buffer after it
    new Uint8Array(wasm.memory.buffer, at, bytes.length).set(bytes);
    const ptr = wasm.thalassa_step(bytes.length), len = wasm.thalassa_output_len();
    const output=new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, ptr, len));
    if(recorder)recorder(JSON.stringify(input),output);
    return JSON.parse(output);
  }
  const rp = p => ({x: Math.round(p.x), y: Math.round(p.y)});
  function frame() {
    return {version: 1, rules_revision: REV, frame: {
      tick, owner: 1, team: 0, alive: me.hp > 0, pos: rp(me.pos), hp: Math.round(me.hp), max_hp: me.max_hp, shield: myShield(), ap: 100, stunned: tick < me.stunUntil, ult_unlocked: true,
      units: units.map(u => ({id: u.id, team: u.team, pos: rp(u.pos), hp: Math.round(u.hp), max_hp: u.max_hp, radius: u.radius, champion: u.champion, tower: u.tower, visible: u.visible, targetable: u.targetable, farmable: u.farmable, stunned: tick < u.stunUntil, velocity: rp(u.velocity)})), terrain: terrain(),
      profile: [...document.querySelectorAll('.stat input')].map(i => Number(i.value)),
      clone: clone ? {id: 99, pos: rp(clone.pos), hp: clone.hp} : null, echo: echo ? {id: 98, pos: rp(echo.pos), hp: Math.round(echo.hp)} : null, basic_hits: basicHits.splice(0),
      manual: !$('#auto').checked, order: queue.shift() || null}};
  }
  const find = id => id === 1 ? me : units.find(u => u.id === id);
  function glide(who, to, ticks) { glides = glides.filter(g => g.who !== who); glides.push({who, from: {...who.pos}, to: {...to}, t0: tick, t1: tick + Math.max(1, ticks)}); }
  function apply(out) {
    if (out.error) { log('ERROR ' + out.error); running = false; return; }
    state = out.state;
    const waveOrigin={...me.pos};
    const waterDive=out.effects.some(e=>e.kind==='submerge');
    for (const e of out.effects) {
      const d = e.data;
      if(e.kind==='pose'){visual.pose=d.name;visual.start=tick;visual.until=tick+d.ticks;}
      if(e.kind==='fx'&&!d.name.startsWith('tls_twin_p')&&d.name!=='tls_tide_link'&&!d.name.startsWith('tls_trident_p')){
        const tag=d.name.replace(/^tls_/,'');const spec=painter.manifest.effects[tag];
        if(d.name==='tls_surge'&&d.ticks===18)visual.fx.push({tag:'tsunami',from:waveOrigin,at:{...d.at},start:tick,until:tick+18});else if(spec)visual.fx.push({tag,at:{...d.at},start:tick,until:tick+spec.frames.reduce((s,f)=>s+f.duration*60,0)});
      }
      switch (e.kind) {
        case 'damage': { const u = find(d.target); if (u) { u.hp = Math.max(0, u.hp - d.amount); log(`#${d.target} takes ${d.amount}${d.from_clone ? ' (twin)' : ''}`); } break; }
        case 'stun': { const u = find(d.target); if (u) { u.stunUntil = Math.max(u.stunUntil || 0, tick + d.ticks); log(`#${d.target} stunned ${(d.ticks / 60).toFixed(2)} s`); } break; }
        case 'trident_hit': { const u = find(d.target); if (u) { u.stunUntil = Math.max(u.stunUntil || 0, tick + d.ticks); log(`#${d.target} impaled: stunned ${(d.ticks / 60).toFixed(2)} s`); } break; }
        case 'pull': { const u = find(d.target); if (u) glide(u, d.to, d.ticks); break; }
        case 'glide': if(d.to.x!==me.pos.x)visual.left=d.to.x<me.pos.x; glide(me, d.to, d.ticks); break;
        case 'submerge': banished = tick + d.ticks; visual.exitAt=tick+d.ticks; break;
        case 'heal': me.hp = Math.min(me.max_hp, me.hp + d.amount); break;
        case 'shield': if (d.target === 1) me.shields.push({amount: d.amount, until: tick + d.ticks}); if (d.target !== 1 || d.amount > 20) log(`#${d.target} shield ${d.amount}`); break;
        case 'cleanse': { const u = find(d.target); if (u) u.stunUntil = 0; log(`#${d.target} cleansed`); break; }
        case 'cc_immune': log(`#${d.target} crowd-control immune ${(d.ticks / 60).toFixed(2)} s`); break;
        case 'water': log(d.on ? 'Water mode ON' : 'Water mode off'); break;
        case 'spawn_clone': clone = {pos: {...d.pos}, hp: 200, until: tick + d.ticks}; log('Tide Twin appears'); break;
        case 'clone_move': if (clone) {if(d.to.x!==clone.pos.x)visual.cloneLeft=d.to.x<clone.pos.x;if(waterDive){visual.fx.push({tag:'dive_in',at:{...clone.pos},start:tick,until:tick+18});visual.cloneExitAt=tick+d.ticks;}glide(clone, d.to, d.ticks);} break;
        case 'pool_calm': log('A whirlpool calms into a pool'); break;
        case 'spawn_echo': echo = {pos: {...d.pos}, hp: 250, until: tick + d.ticks}; log('The trident leaves a clone of her'); break;
        case 'echo_move': if (echo) { if (d.to.x !== echo.pos.x) visual.echoLeft = d.to.x < echo.pos.x; glide(echo, d.to, d.ticks); } break;
        case 'remove_echo': echo = null; break;
        case 'echo_pose': visual.echoPose = d.name; visual.echoStart = tick; visual.echoUntil = tick + d.ticks; break;
        case 'remove_clone': clone = null; visual.cloneExitAt=0; break;
        case 'swap': { const c = clone; me.pos = {...d.owner_to}; if (c) c.pos = {...d.clone_to}; log('Tide link: swapped with the twin'); break; }
        case 'log': log(String(e.data)); break;
      }
    }
    if (state.rejection) { log(state.rejection); }
  }
  function step() {
    tick++;
    const previous={...me.pos};
    // host model: glides, the player's walk, enemies, the clone expiring
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)); g.who.pos = {x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k}; }
    glides = glides.filter(g => tick < g.t1);
    const moving = glides.some(g => g.who === me);
    let x = Number(held.has('d')) - Number(held.has('a')), y = Number(held.has('s')) - Number(held.has('w'));
    if (!moving && tick >= me.stunUntil && (x || y)) { const m = Math.hypot(x, y), sp = 990 * (state.in_water ? 1.2 : 1); me.pos = {x: me.pos.x + x / m * sp, y: me.pos.y + y / m * sp}; if(x)visual.left=x<0; }
    for (const u of units.filter(u => u.team === 1)) {
      if ($('#moving').checked && tick >= u.stunUntil && !glides.some(g => g.who === u) && u.hp > 0) { const dy = (tick % 240 < 120 ? 1 : -1) * 600; u.pos = {x: u.pos.x, y: u.pos.y + dy}; u.velocity = {x: 0, y: dy}; } else u.velocity = {x: 0, y: 0};
      if (u.hp <= 0) { u.hp = 1000; u.pos = {x: 470000 + (u.id - 5) * 10000, y: 470000 + (u.id - 5) * 30000}; log(`#${u.id} respawns`); }
    }
    if (clone && tick >= clone.until) clone = null;
    if (echo && tick >= echo.until) echo = null;
    if (held.has('f') && tick>=nextBasic && me.hp>0 && tick>=me.stunUntil && tick>=banished && !state.cast && tick>=(state.moving_until||0)) {
      const t=units.filter(u=>u.team===1&&u.hp>0&&u.visible&&u.targetable).sort((a,b)=>dist(me.pos,a.pos)-dist(me.pos,b.pos))[0],range=R.basic.range*(state.in_water?1+R.water.rangePct/100:1);
      if(t&&dist(me.pos,t.pos)<=range+10000+t.radius){nextBasic=tick+R.basic.cooldownTicks;bolts.push({target:t.id,from:{...me.pos},release:tick+12,at:tick+12+Math.max(1,Math.ceil(dist(me.pos,t.pos)/R.basic.boltSpeed))});visual.pose='attack';visual.start=tick;visual.until=tick+22;visual.left=t.pos.x<me.pos.x;}
    }
    for(const b of bolts.filter(b=>tick>=b.at)){const u=find(b.target);if(u&&u.hp>0){u.hp=Math.max(0,u.hp-R.basic.base-Math.floor(R.basic.apRatio*100/100));basicHits.push(u.id);}}
    bolts=bolts.filter(b=>tick<b.at);
    apply(call(frame()));
    visual.moving=me.pos.x!==previous.x||me.pos.y!==previous.y;
    visual.fx=visual.fx.filter(f=>tick<f.until&&me.hp>0);
    if(me.hp<=0&&visual.deadSince===null)visual.deadSince=tick;
    if(visual.exitAt&&tick>=visual.exitAt){visual.exitAt=0;visual.fx.push({tag:'dive_out',at:{...me.pos},start:tick,until:tick+18});}
    if(visual.cloneExitAt&&tick>=visual.cloneExitAt){visual.cloneExitAt=0;if(clone)visual.fx.push({tag:'dive_out',at:{...clone.pos},start:tick,until:tick+18});}
  }
  function order(kind) { queue.push(kind === 'swap' ? {kind} : {kind, aim: {x: Math.round(aim.x), y: Math.round(aim.y)}}); }
  // ---- drawing
  const P = p => ({x: (p.x - cam.x) * S + W / 2, y: (p.y - cam.y) * S + W / 2});
  function circle(p, r, fill, stroke) { const q = P(p); ctx.beginPath(); ctx.arc(q.x, q.y, Math.max(2, r * S), 0, Math.PI * 2); if (fill) { ctx.fillStyle = fill; ctx.fill(); } if (stroke) { ctx.strokeStyle = stroke; ctx.stroke(); } }
  function draw() {
    const zoom = $('#zoom').checked; S = W / (zoom ? 260000 : MAP); cam = zoom ? {...me.pos} : {x: MAP / 2, y: MAP / 2};
    ctx.clearRect(0, 0, W, W);
    ctx.imageSmoothingEnabled=false;
    const art=painter.loaded&&$('#art').checked, pixel=950*S;
    if ($('#walls').checked) { const a = P({x: 18 * 32000, y: 10 * 32000}); ctx.fillStyle = '#3d4a3a'; ctx.fillRect(a.x, a.y, 32000 * S, 10 * 32000 * S); }
    for (const p of state.pools || []) {
      if(art){const q=P(p.center),phase=Math.floor((tick-Math.max(0,p.until-R.water.poolTicks))/8)%8;painter.effect(ctx,(p.whirl?'whirlpool':'pool')+'_p'+phase,0,q.x,q.y,pixel);}
      else circle(p.center, p.radius, p.whirl ? 'rgba(60,140,220,.45)' : 'rgba(80,170,230,.3)', p.whirl ? '#9fd4ff' : '#5fb2d9');
    }
    if (clone&&me.hp>0) {
      if (state.twin && state.twin.linked && !state.twin.swapped) { const a = P(me.pos), b = P(clone.pos); if(art)for(const part of painter.linkSegments(me.pos,clone.pos)){const q=P(part.at);painter.effect(ctx,part.tag,0,q.x,q.y,pixel);}else{ctx.strokeStyle = '#7fe0ff';ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();} }
      if(art){const q=P(clone.pos),moving=glides.some(g=>g.who===clone),pose=tick<visual.until&&['attack','skill1','skill2'].includes(visual.pose)?visual.pose:moving?'run':'idle';painter.body(ctx,pose,tick<visual.until?tick-visual.start:tick,q.x,q.y,pixel,visual.cloneLeft,true,pose==='idle'||pose==='run');}
      else circle(clone.pos, 9000, 'rgba(120,220,255,.5)', '#bff0ff');
    }
    if (echo) {
      // the trident's clone of her, drawn a little translucent so it reads apart from the Tide Twin
      ctx.save(); ctx.globalAlpha = 0.75;
      if(art){const q=P(echo.pos),moving=glides.some(g=>g.who===echo),pose=tick<visual.echoUntil?visual.echoPose:moving?'run':'idle';painter.body(ctx,pose,tick<visual.echoUntil?tick-visual.echoStart:tick,q.x,q.y,pixel,visual.echoLeft,true,pose==='idle'||pose==='run');}
      else circle(echo.pos, 9000, 'rgba(160,200,255,.45)', '#d0e4ff');
      ctx.restore();
    }
    if (state.trident) { if(art){const q=P(state.trident.at),phase=Math.floor(((state.trident.heading_mdeg+22500)%360000+360000)%360000/45000);painter.effect(ctx,'trident_p'+phase,0,q.x,q.y,pixel);}else circle(state.trident.at, 6000, '#ffd36b', '#fff'); }
    for (const u of units) { circle(u.pos, u.radius, u.team ? '#d9735f' : '#6fcf8f', tick < u.stunUntil ? '#ffe36b' : '#111'); const q = P(u.pos); ctx.fillStyle = '#fff'; ctx.fillText(`#${u.id} ${Math.round(u.hp)}`, q.x - 18, q.y - 12); }
    ctx.globalAlpha = tick < banished ? .3 : 1;
    if(art){const q=P(me.pos),pose=me.hp<=0?'dead':tick<me.stunUntil?'hit':tick<visual.until?visual.pose:visual.moving?'run':'idle';const elapsed=pose==='dead'?tick-visual.deadSince:tick<visual.until?tick-visual.start:tick;painter.body(ctx,pose,elapsed,q.x,q.y,pixel,visual.left,false,pose==='idle'||pose==='run');if(state.in_water&&me.hp>0)painter.effect(ctx,'water_aura',tick,q.x,q.y,pixel,true);}
    else circle(me.pos, 9000, '#4fa8ff', tick < me.stunUntil ? '#ffe36b' : '#e8f6ff');ctx.globalAlpha=1;
    if(art)for(const fx of visual.fx){if(fx.tag==='tsunami'){roundArt.wave(ctx,fx.from,fx.at,tick-fx.start,P,pixel);continue;}const q=P(fx.at);painter.effect(ctx,fx.tag,tick-fx.start,q.x,q.y,pixel);}
    const a = P(aim); ctx.strokeStyle = '#ffcf8a'; ctx.beginPath(); ctx.moveTo(a.x - 6, a.y); ctx.lineTo(a.x + 6, a.y); ctx.moveTo(a.x, a.y - 6); ctx.lineTo(a.x, a.y + 6); ctx.stroke();
    const r = state.ready || [0, 0, 0, 0], label = ['Tide Pool', 'Dive', 'Tide Twin', 'Trident'];
    $('#cds').innerHTML = label.map((l, i) => `${l}: ${tick >= r[i] ? '<b>ready</b>' : ((r[i] - tick) / 60).toFixed(1) + ' s'}`).join(' · ') + `<br>In water: <b>${state.in_water ? 'yes' : 'no'}</b> · overheal ${state.overheal || 0} · HP ${Math.round(me.hp)}`;
    $('#status').textContent = `${running ? 'Running' : 'Paused'} · tick ${tick} · rules ${REV}`;
  }
  function loop(now) { clock.advance(now,running&&!document.hidden,step); draw(); requestAnimationFrame(loop); }
  window.addEventListener('blur',()=>held.clear());
  document.addEventListener('visibilitychange',()=>clock.reset());
  canvas.addEventListener('mousemove', e => { const b = canvas.getBoundingClientRect(); const k = W / b.width; aim = {x: ((e.clientX - b.left) * k - W / 2) / S + cam.x, y: ((e.clientY - b.top) * k - W / 2) / S + cam.y}; });
  canvas.addEventListener('keydown', e => {
    const k = e.key.toLowerCase(); if ('wasdf'.includes(k) && k.length === 1) held.add(k);
    if (k === '1') order('tide_pool'); if (k === '2') order('dive'); if (k === '3') order('twin'); if (k === 'x') order('swap');
    if (k === '4') order(state.trident ? 'steer' : 'ult');
    e.preventDefault();
  });
  canvas.addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  $('#play').onclick = () => { running = !running; $('#play').textContent = running ? 'Pause' : 'Play'; canvas.focus(); };
  $('#step').onclick = () => { running = false; $('#play').textContent = 'Play'; step(); };
  $('#reset').onclick = () => { reset(); canvas.focus(); };
  $('#stunAlly').onclick = () => { units[2].stunUntil = tick + 60; log('#7 (ally) stunned'); canvas.focus(); };
  $('#stunMe').onclick = () => { me.stunUntil = tick + 60; log('Thalassa stunned'); canvas.focus(); };
  $('#dashOnMe').onclick = () => { const u = units[0]; u.pos = {x: me.pos.x + 9000, y: me.pos.y}; me.hp = Math.max(1, me.hp - 200); log('#5 dashes onto her (and hits hard)'); canvas.focus(); };
  window.thalassaLab = {roundArt,get state() { return state; }, get tick() { return tick; }, get me() { return me; }, get units() { return units; }, get clone() { return clone; }, get echo() { return echo; }, painter, step, reset, order, setRecorder: fn=>{recorder=fn;}, setAim: p => { aim = {...p}; }, pause: () => { running = false; clock.reset(); $('#play').textContent='Play'; }};
  reset(); canvas.focus(); requestAnimationFrame(loop);
})();
