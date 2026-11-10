// Kratos's unchanged WASM core on the installed game's registered map layers
// and exported collision/object coordinates. Stock actors use extracted art and
// settings; their host combat, navigation and vision are explicitly modelled.
(async () => {
  const $ = id => document.getElementById(id);
  const R = window.KRATOS_RULES, canvas = $('arena'), ctx = canvas.getContext('2d');
  const world = await new GameWorld().ready;
  const clock = new KratosClock(), painter = new KratosPainter();
  painter.ready.catch(error => log('Artwork unavailable: ' + error.message));
  const wasm = (await WebAssembly.instantiate(await (await fetch('kratos.wasm')).arrayBuffer(), {})).instance.exports;
  const N = world.geometry.columns, CELL = world.geometry.cell, WORLD = world.geometry.width;
  // Exact object positions from the installed native map export.
  const TOWERS = world.data.doc.towers.flatMap(t=>t.pos), WALLS=world.walls;
  const wallAt = p => world.wallAt(p);
  const names = ['ego', 'judgement', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid', 'aggressive', 'order', 'roaming', 'macro', 'micro', 'awareness', 'strategy'];
  $('stats').innerHTML = names.map(n => `<span>${n}</span><input type="number" min="0" max="100" value="50">`).join('');
  const rp = p => ({x: Math.round(p.x), y: Math.round(p.y)});
  const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  const cellCenter = (cx, cy) => ({x: cx * CELL + CELL / 2, y: cy * CELL + CELL / 2});
  let tick, me, units, glides, state, queue, fx, logLines, hits, pendingHits, basicHits, running = true, aim = {x: 480000, y: 480000}, held = new Set(), lastRejection = '';
  let recording = null, replay = null, replayIndex = 0, lastTrace = null, demo = null;
  let scene=null, fixture=false, selectedTarget=null, imagesReady=Promise.resolve();
  $('scenario').innerHTML=Object.entries(GameScenario.choices).map(([k,v])=>`<option value="${k}">${v}</option>`).join('');
  $('stockChampion').innerHTML=Object.entries(world.data.champions).sort((a,b)=>a[1].name.localeCompare(b[1].name)).map(([id,c])=>`<option value="${id}">${c.name}</option>`).join('');$('stockChampion').value='knight';
  const unit = (id, team, p, o = {}) => ({id, team, pos: {...p}, hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, farmable: false,
    slowUntil: 0, stunUntil: 0, airborneUntil: 0, frost: 0, frostUntil: 0, burnUntil: 0, frozenUntil: 0, nextAttack: 0,kind:o.tower?'tower':o.champion===false?(team===9?'camp':'minion'):'champion',sprite:o.tower?(team?'red_tower':'blue_tower'):o.champion===false?(team===9?'rhino':'minions'):'champ-knight',minion:o.champion===false&&team!==9&&!o.tower?'melee':null,...o});
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
  function reset(options={}) {
    wasm.kratos_reset();
    tick = 0; state = {}; queue = []; glides = []; fx = []; logLines = []; hits = []; pendingHits = []; basicHits = [];
    held.clear(); clock.reset(); replay = null; replayIndex = 0; demo = null; lastRejection = ''; lastTrace = null;pinned=null;
    const c = wallCrossing();
    me = {id:1,team:0,champion:true,radius:world.data.settings.champion_radius,pos: {...c.from}, hp: 2000, max_hp: 2000, shields: [], immuneUntil: 0, reduce: 0, reduceUntil: 0, stunUntil: 0, blades: false, sightUntil: 0, pose: '', poseUntil: 0,
      poseStart: 0, moving: false, moveStart: 0, left: false, attackStart: -100, deadStart: null};
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
    fixture=!!options.fixture;scene=null;selectedTarget=null;world.routes.clear();
    if(fixture){me.radius=3000;$('scenarioNote').textContent='Isolated 100-Attack mechanics fixture; this is not the mid-game scenario snapshot.';}
    else{
      const level=Math.max(1,Math.min(12,+$('level').value||8));scene=new GameScenario(world,options.scenario||$('scenario').value,level);const setup=scene.setup();
      tick=setup.clock;me.pos=setup.pos;units=setup.units;
      me.stats=scene.stats({stat:R.stat,growth:R.growth});
      for(const[key,index]of [['attack',0],['magic_power',1],['hp',2],['defence',3],['magic_resistance',4],['move_speed',5]])me.stats[key]=Math.floor((me.stats[key]||0)*(100+R.modPower[index])/100);
      me.hp=me.max_hp=me.stats.hp;$('role').value=setup.role;$('scenarioNote').textContent=setup.description;
      log('Mid-game snapshot: level '+level+', 8:00, skills ready, no items. Stock actor attacks/movement are a settings-based host model; full stock skill AI is not emulated.');
    }
    aim={x:me.pos.x+70000,y:me.pos.y-50000};
    imagesReady=world.loadSprites(units.map(u=>u.sprite));imagesReady.catch(e=>log(e.message));
    log('Map: verified exported walls, exact tower/camp/lane coordinates, original 1280px layers with 160px border; 1,000 world units per art pixel.');
  }
  function log(s) { logLines.push(`[${tick}] ${s}`); if (logLines.length > 200) logLines.shift(); $('log').textContent = logLines.slice(-60).join('\n'); $('log').scrollTop = 1e9; }
  const find = id => units.find(u => u.id === id);
  function glide(who, to, ticks) { glides = glides.filter(g => g.who !== who); glides.push({who, from: {...who.pos}, to: {...to}, t0: tick, t1: tick + Math.max(1, ticks)}); }
  const visible=u=>replay?u.visible!==false:!scene||scene.visible(u,0,me,units);
  const enemyCalm = () => !units.some(u => u.team === 1 && u.champion && u.hp > 0 && visible(u) && dist(u.pos, me.pos) < R.ai.calmRange);
  function frame() {
    if(scene)scene.refreshVisibility(me,units);
    const calm=enemyCalm();
    const profile = [...$('stats').querySelectorAll('input')].map(i => Math.max(0, Math.min(100, +i.value || 0)));
    const out = {version: 1, rules_revision: R.revision, hits_on_me: pendingHits.map(h => [h.from, h.champion??true]), frame: {
      tick, owner: 1, team: 0, alive: me.hp > 0, pos: rp(me.pos), hp: Math.round(me.hp), max_hp: me.max_hp, attack: me.stats?.attack??100,
      stunned: tick < me.stunUntil, ult_unlocked: true,
      units: units.filter(u => u.hp > 0).map(u => ({id: u.id, team: u.team, pos: visible(u)?rp(u.pos):{x:0,y:0}, hp: visible(u)?Math.round(u.hp):0, max_hp: u.max_hp, radius: u.radius,
        champion: u.champion, tower: u.tower, visible: visible(u), targetable: true, farmable: u.farmable && calm&&(!scene||u.kind==='minion'||u.kind==='objective'||$('role').value==='jungle'), stunned: tick < u.stunUntil})),
      terrain: WALLS, profile, basic_hits: basicHits.splice(0), manual: !$('auto').checked, order: queue.shift() || null}};
    return out;
  }
  function hurtMe(raw) {
    if(scene){const before=me.hp,source={id:0,team:1};scene.damage(source,me,raw,tick);scene.metrics.taken+=before-me.hp;return before-me.hp;}
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
        case 'damage': if (u) { const dealt=scene?scene.damage(me,u,d.amount,tick):Math.min(u.hp,d.amount);if(!scene)u.hp = Math.max(0, u.hp - d.amount);hits.push({id: u.id, amount: dealt, raw:d.amount, ally: u.team === 0, tower: u.tower, until: tick + 30});
          if (u.team === 0 || u.tower) log(`!! #${u.id} (${u.tower ? 'tower' : 'ally'}) hit by a skill (should never happen)`); } break;
        case 'slow': if (u) {u.slowUntil = tick + d.ticks;u.slowPct=d.percent;} break;
        case 'freeze': if (u) { u.stunUntil = tick + d.ticks; u.frozenUntil = tick + d.ticks; log(`#${d.target} frozen`); } break;
        case 'knock_up': if (u) { u.stunUntil = Math.max(u.stunUntil, tick + d.ticks); u.airborneUntil = tick + d.ticks; log(`#${d.target} knocked up ${(d.ticks / 60).toFixed(2)} s`); } break;
        case 'taunt': if(u){u.tauntUntil=tick+d.ticks;u.tauntTarget=me.id;u.pending=null;log(`#${d.target} taunted toward Kratos for ${(d.ticks/60).toFixed(2)} s`);} break;
        case 'frost': if (u) { u.frost = d.stacks; u.frostUntil = tick + d.ticks; } break;
        case 'burn': if (u) u.burnUntil = tick + d.ticks; break;
        case 'weapon': me.blades = d.blades; log(`Weapon: ${d.blades ? 'Blades of Chaos' : 'Leviathan Axe'}`); break;
        case 'shield': me.shields.push({amount: d.amount, until: tick + d.ticks}); log(`Shield ${d.amount} (${(d.ticks / 60).toFixed(1)} s)`); break;
        case 'immune': me.immuneUntil = tick + d.ticks; log(`CC immune ${(d.ticks / 60).toFixed(2)} s`); break;
        case 'reduce': me.reduce = d.percent; me.reduceUntil = tick + d.ticks; break;
        case 'cleanse': me.stunUntil = 0; log('Cleansed'); break;
        case 'sight': me.sightUntil = tick + d.ticks; break;
        case 'haste': me.hastePct = d.percent; me.hasteUntil = tick + d.ticks; log(`Speed +${d.percent}% for ${(d.ticks / 60).toFixed(1)} s`); break;
        case 'glide': glide(me, d.to, d.ticks); break;
        case 'hop': glides = glides.filter(g => g.who !== me); me.pos = scene?world.legal(d.to,me.radius):{...d.to}; log('Chain Dash: over the wall'); break;
        case 'pose': me.pose = d.name; me.poseStart = tick; me.poseUntil = tick + d.ticks; break;
        case 'fx': if (!/^krt_fenrir_p\d$/.test(d.name)) fx.push({name: d.name, at: {...d.at}, from: {...me.pos}, start: tick, until: tick + Math.max(6, d.ticks)}); break;
        case 'log': log(String(d)); break;
      }
    }
  }
  function coreStep(input) {
    const text = JSON.stringify(input), bytes = new TextEncoder().encode(text);
    const ptr = wasm.kratos_input(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    const outPtr = wasm.kratos_step(bytes.length), n = wasm.kratos_output_len();
    return JSON.parse(new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, outPtr, n)));
  }
  const copy = value => structuredClone(value);
  function step() {
    if (replay) {
      const entry = replay.frames[replayIndex++];
      if (!entry) { running = false; log('Replay complete: every shared-core output matched.'); return; }
      const output = coreStep(entry.input);
      if (JSON.stringify(output) !== JSON.stringify(entry.output)) { running = false; log('Replay mismatch at tick ' + entry.input.frame.tick); throw new Error('Kratos recording/core mismatch'); }
      ({tick, me, units, glides, fx, hits} = copy(entry.host)); state = output.state; tick++;
      return;
    }
    if (demo) advanceDemo();
    const input = frame(), host = recording?copy({tick, me, units, glides: [], fx, hits}):null, out = coreStep(input);
    lastTrace = {input, output:out};
    if (recording) {
      const entry={input,output:out,host};recording.bytes=(recording.bytes||0)+JSON.stringify(entry).length;
      if (recording.frames.length < 7200&&recording.bytes<=32*1024*1024) recording.frames.push(entry);
      else { recording = null; log('Recording stopped at its two-minute / 32 MiB limit.'); }
    }
    if (out.error) { log('Lab error: ' + out.error); return; }
    state = out.state;
    if (out.rejection && out.rejection !== lastRejection) log('Refused: ' + out.rejection);
    lastRejection = out.rejection || '';
    apply(out);
    if(me.hp<=0){
      me.deadStart??=tick;me.shields=[];me.pose='dead';me.poseUntil=0;me.pendingBasic=null;glides=[];fx=[];queue=[];pendingHits=[];held.clear();
    }
    const before = {...me.pos};
    tick++;
    for (const g of glides) { const k = Math.min(1, (tick - g.t0) / (g.t1 - g.t0)),to={x: g.from.x + (g.to.x - g.from.x) * k, y: g.from.y + (g.to.y - g.from.y) * k};g.who.pos=scene?world.sweep(g.who.pos,to,g.who.radius):to; }
    glides = glides.filter(g => tick < g.t1);
    // his walk (blocked by the real walls)
    if (me.hp > 0 && tick >= me.stunUntil && (!scene||tick>=me.poseUntil&&tick>=(state.busy_until||0)) && !glides.some(g => g.who === me)) {
      const dir = {x: (held.has('d') - held.has('a')), y: (held.has('s') - held.has('w'))};
      if (dir.x || dir.y) { const l = Math.hypot(dir.x, dir.y), v = (me.stats?.move_speed??R.stat.move_speed) * (tick < (me.hasteUntil || 0) ? 100 + me.hastePct : 100) / 100;const next = {x: me.pos.x + dir.x / l * v, y: me.pos.y + dir.y / l * v};me.pos=world.sweep(me.pos,next,me.radius); }
      else if(scene&&$('auto').checked){const target=units.filter(u=>u.team!==0&&u.hp>0&&u.champion&&visible(u)).sort((a,b)=>dist(me.pos,a.pos)-dist(me.pos,b.pos))[0];if(target&&dist(me.pos,target.pos)>R.basic.axeRange+me.radius+target.radius)world.move(me,target.pos,me.stats.move_speed);}
    }
    const moving = dist(before, me.pos) > 0;
    if (moving && !me.moving) me.moveStart = tick;
    me.moving = moving;
    if (Math.abs(me.pos.x - before.x) > .01) me.left = me.pos.x < before.x;
    // the Lab's enemies: walk in, hit him (Heimdall's Sight decides on the next frame)
    if ($('moving').checked && !scene) {
      for (const u of units.filter(u => u.team === 1 && u.champion && u.hp > 0)) {
        if (tick < u.stunUntil) continue;
        const speed = tick < u.slowUntil ? 500 : 800, d = dist(u.pos, me.pos);
        if (!glides.some(g => g.who === u) && d > 20000) { const k = speed / d, next = {x: u.pos.x + (me.pos.x - u.pos.x) * k, y: u.pos.y + (me.pos.y - u.pos.y) * k}; if (!wallAt(next)) u.pos = next; }
        if (d <= 25000 && tick >= u.nextAttack) { u.nextAttack = tick + 60; pendingHits.push({from: u.id, amount: 60}); }
      }
    }
    if(scene&&$('moving').checked)scene.advance(tick,me,units,{hitOwner:h=>pendingHits.push(h),hit:(u,hp)=>hits.push({id:u.id,amount:hp,until:tick+30})});
    if(me.pendingBasic&&tick>=me.pendingBasic.at){const u=find(me.pendingBasic.target);me.pendingBasic=null;if(u&&u.hp>0){const raw=me.stats?.attack??100,dealt=scene?scene.damage(me,u,raw,tick):Math.min(u.hp,raw);if(!scene)u.hp-=dealt;hits.push({id:u.id,amount:dealt,until:tick+30});basicHits.push(u.id);}}
    if(scene&&$('auto').checked&&tick>=(me.basicReady||0)&&tick>=me.poseUntil)basic();
    if(!scene)for (const u of units) if (u.hp <= 0 && (u.team === 9 || !u.champion) && !u.tower) { u.hp = u.max_hp; }
    me.shields = me.shields.filter(s => tick < s.until && s.amount > 0);
    fx = fx.filter(f => tick < f.until);
    hits = hits.filter(h => tick < h.until);
  }
  function order(kind) {
    const o = {kind};
    if (['rampage', 'redirect', 'dash', 'fenrir','slam_aim'].includes(kind)) o.aim = rp(aim);
    queue.push(o);
    if (Math.abs(aim.x - me.pos.x) > .01) me.left = aim.x < me.pos.x;
  }
  function basic() {
    if(me.hp<=0||tick<me.stunUntil||tick<me.poseUntil||tick<(me.basicReady||0))return;
    const reach = (me.blades ? R.basic.bladesRange : R.basic.axeRange) + me.radius;
    const t = units.filter(u => u.team !== 0 && u.hp > 0 && visible(u)&&world.clear(me.pos,u.pos)&&dist(u.pos, me.pos) <= reach + u.radius).sort((a,b)=>(a.id===selectedTarget?-1:b.id===selectedTarget?1:dist(a.pos,me.pos)-dist(b.pos,me.pos)))[0];
    if (!t) return log(`Nothing within basic reach (${reach - me.radius})`);
    me.basicReady=tick+Math.ceil(R.basic.cooldownTicks/(1+(R.modPower[6]||0)/100));me.pendingBasic={target:t.id,at:tick+R.basic.startTiming};
    me.attackStart = tick; me.left = t.pos.x < me.pos.x;
  }
  function startDemo(kind) {
    reset({fixture:true}); $('auto').checked = false; $('moving').checked = false;
    const c = wallCrossing(); aim = {x: c.from.x + 90000, y: c.from.y};
    if (kind === 'linked-dash') { find(2).pos = {...c.to}; aim = {...c.to}; }
    else units.filter(u => u.team === 1 && u.champion).forEach((u, i) => { u.pos = {x: me.pos.x + 10000 + i * 10000, y: me.pos.y}; });
    demo = {kind, start: tick}; running = true; clock.reset(); log('Preview: ' + kind + ' (shared core, simulated host)');
  }
  function advanceDemo() {
    const age = tick - demo.start, kind = demo.kind;
    if (kind === 'sight' && age === 0) order('sight');
    if (kind === 'sight' && [15, 30].includes(age)) pendingHits.push({from: 2, amount: 100});
    if (kind === 'rampage' && age === 0) order('rampage');
    if (kind === 'linked-dash' && age === 0) order('guard');
    if (kind === 'linked-dash' && age === 16) order('dash');
    if (kind === 'dash-guard' && age === 0) order('dash');
    if (kind === 'dash-guard' && age === 35) order('guard');
    if (kind === 'fenrir' && age === 0) order('fenrir');
    if (kind === 'fenrir' && state.fenrir?.leg===0&&!state.fenrir.slam_at&&dist(state.fenrir.pos,me.pos)<=R.fenrir.width+8000)order('slam_aim');
    if (age >= (kind==='fenrir'?320:220)) { demo = null; running = false; }
  }
  function visualPose() {
    if (me.hp <= 0) return {name: 'dead', start: me.deadStart ?? tick};
    if (tick < me.poseUntil && state.rampage) return {name: ['cleave', 'sweep', 'smash'][state.rampage.stage], start: me.poseStart};
    if (tick < me.poseUntil) return {name: state.last_skill === 'dash' ? 'chain' : me.pose === 'skill2' ? 'guard' : me.pose === 'ult' ? 'ult' : 'sight', start: me.poseStart};
    if (tick - me.attackStart < R.basic.duration) return {name: 'attack', start: me.attackStart};
    return {name: me.moving ? 'movement' : 'idle', start: me.moving ? me.moveStart : 0};
  }
  // ---------------------------------------------------------------- drawing (one world-to-screen scale for everything)
  let cam = {x: 480000, y: 480000, s: 900 / WORLD},pinned=null,pan=null;
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
    ctx.imageSmoothingEnabled=false;
    if ($('zoom').checked) cam = {x: pinned?.x??me.pos.x, y: pinned?.y??me.pos.y, s: (+$('worldZoom').value||2)/world.geometry.units_per_pixel}; else cam = {x: WORLD/2, y: WORLD/2, s: 900 / WORLD};
    ctx.fillStyle = '#2a3340'; ctx.fillRect(0, 0, 900, 900);
    if ($('art').checked) world.drawMap(ctx,P,W);
    if($('colliders').checked){ctx.fillStyle='rgba(255,60,60,.23)';for(let i=0;i<WALLS.length;i++)if(WALLS[i]){const q=P({x:i%N*CELL,y:Math.floor(i/N)*CELL});ctx.fillRect(q.x,q.y,W(CELL),W(CELL));}}
    if ($('ranges').checked) {
      const linked = state.link && state.link[0] === 'Guard' && tick < state.link[1];
      circle(me.pos, R.dash.range, null, linked ? 'rgba(248,113,113,.25)' : 'rgba(248,113,113,.7)');
      circle(me.pos, R.dash.linkedRange, null, linked ? 'rgba(248,113,113,.9)' : 'rgba(248,113,113,.25)');
      circle(me.pos, R.rampage.range, null, 'rgba(147,197,253,.6)');
      line(me.pos, toward(me.pos, aim, R.fenrir.distance), 'rgba(250,204,21,.35)', Math.max(1, W(R.fenrir.width * 2)));
      line(me.pos, toward(me.pos, aim, linked ? R.dash.linkedRange : R.dash.range), 'rgba(248,113,113,.7)', Math.max(1, W(R.dash.width * 2)));
    }
    for (const f of fx) {
      const age = (tick - (f.start ?? tick)) / Math.max(1, f.until - (f.start ?? tick));
      const follow=/^krt_(guard|sight|swap_axe|swap_blades)$/.test(f.name),at=P(follow?me.pos:f.at);
      let tag=f.name;
      if(tag==='krt_chain'&&f.from){
        const h=((Math.round(Math.atan2(f.at.y-f.from.y,f.at.x-f.from.x)/(Math.PI/4))%8)+8)%8;
        tag='krt_chain_'+(dist(f.from,f.at)>R.dash.range+1000?'long':'short')+'_p'+h;
        const mid=P({x:(f.from.x+f.at.x)/2,y:(f.from.y+f.at.y)/2});at.x=mid.x;at.y=mid.y;
      }
      if ($('bodies').checked && painter.loaded && painter.effect(ctx,tag,tick-(f.start??tick),at.x,at.y,W(world.geometry.units_per_pixel))) continue;
      if (/cleave|sweep/.test(f.name)) circle(f.at, R.rampage.swingRadius, f.name.includes('cleave') ? 'rgba(125,211,252,.35)' : 'rgba(251,146,60,.35)', '#fff');
      else if (f.name === 'krt_smash') circle(f.at, R.rampage.smashRadius, 'rgba(192,132,252,.3)', '#e9d5ff', 2);
      else if (f.name === 'krt_chain') line(me.pos, f.at, '#f97316', 2);
      else if (f.name === 'krt_guard') circle(me.pos, 7000, 'rgba(226,232,240,.25)', '#e2e8f0', 2);
      else if (f.name === 'krt_sight' || f.name === 'dodge') circle(me.pos, 9000 + 3000 * age, null, '#facc15', 2);
      else if (f.name === 'krt_freeze') circle(f.at, 4000, 'rgba(186,230,253,.6)');
      else if (f.name.startsWith('krt_fenrir')) circle(f.at, R.fenrir.width, null, '#fde68a', 2);
      else circle(f.at, 3000, null, '#fca5a5');
    }
    if (state.fenrir) { const w = state.fenrir, p = P(w.pos);
      if ($('bodies').checked && painter.loaded) painter.wolf(ctx, w.from, w.to, tick, p.x, p.y, W(world.geometry.units_per_pixel));
      else circle(w.pos, R.fenrir.width, 'rgba(203,213,225,.5)', '#f8fafc', 2);
      line(w.from, w.to, 'rgba(248,250,252,.4)');
      if(!w.leg&&$('ranges').checked){line(w.pos,toward(w.pos,aim,R.fenrir.returnDistance),'rgba(102,218,255,.5)',2);circle(me.pos,R.fenrir.dashMax+R.fenrir.width,null,'rgba(102,218,255,.5)');}}
    let ownerDrawn=false;
    const drawOwner=()=>{const p=P(me.pos),v=visualPose();if($('bodies').checked&&painter.loaded)painter.body(ctx,v.name,tick,p.x,p.y,W(world.geometry.units_per_pixel),me.left,me.blades,v.start);else circle(me.pos,3500,me.blades?'#ea580c':'#38bdf8',null);ownerDrawn=true;};
    for (const u of units.filter(u => u.hp > 0||tick-(u.deadAt??-1000)<60).sort((a,b)=>a.pos.y-b.pos.y)) {
      if(!visible(u)&&!u.tower&&!$('overviewActors').checked)continue;
      if(!ownerDrawn&&u.pos.y>=me.pos.y)drawOwner();
      const col = u.team === 0 ? '#60a5fa' : u.team === 9 ? '#b8a38c' : '#f87171';
      ctx.globalAlpha=visible(u)||u.tower?1:.25;
      if(!world.drawUnit(ctx,u,tick,P,W)){const p=P(u.pos);ctx.fillStyle='#fff';ctx.font='10px system-ui';ctx.fillText('Loading '+(u.name||u.sprite),p.x,p.y);}
      ctx.globalAlpha=1;
      if($('colliders').checked)circle(u.pos,u.radius,null,col);
      if(u.id===selectedTarget)circle(u.pos,u.radius+2000,null,'#fff',2);
      if(tick<u.frozenUntil)circle(u.pos,u.radius+1000,null,'#9fe5ff',2);
      if (tick < u.burnUntil) circle(u.pos, u.radius + 1500, null, '#f97316', 2);
      const q = P(u.pos),spec=world.data.sprites[u.sprite],idle=spec?.anims.idle||Object.values(spec?.anims||{})[0],half=(idle?.frames[0]?.data.h||40)*world.geometry.units_per_pixel/2,top=q.y-W(half)-8;
      for (let i = 0; i < (tick < u.frostUntil ? u.frost : 0); i++) { ctx.fillStyle = '#7dd3fc'; ctx.fillRect(q.x - 8 + i * 7, q.y - W(u.radius) - 14, 5, 5); }
      if(u.hp>0&&(visible(u)||u.team===0)){const width=u.kind==='minion'?18:u.tower?42:34;ctx.fillStyle = '#111'; ctx.fillRect(q.x-width/2,top,width,4);ctx.fillStyle=col;ctx.fillRect(q.x-width/2,top,width*u.hp/u.max_hp,4);
      if($('labels').checked&&(u.champion||u.kind==='objective'||u.id===selectedTarget)){ctx.fillStyle='#edf2eb';ctx.font='10px system-ui';ctx.textAlign='center';ctx.fillText(u.name||u.kind,q.x,top-4);ctx.textAlign='left';}}
    }
    // Kratos: body colour = weapon; HP bar with the shield (white) on top
    if(!ownerDrawn)drawOwner();
    const q = P(me.pos), sh = me.shields.reduce((s, x) => s + x.amount, 0),barY=q.y-W(24000)-12;
    ctx.fillStyle = '#111'; ctx.fillRect(q.x - 22, barY, 44, 5);
    ctx.fillStyle = '#ef4444'; ctx.fillRect(q.x - 22, barY, 44 * me.hp / me.max_hp, 5);
    ctx.fillStyle = '#fff'; ctx.fillRect(q.x - 22 + 44 * me.hp / me.max_hp, barY, Math.min(44 * sh / me.max_hp, 44 - 44 * me.hp / me.max_hp + 10), 5);
    for (const h of hits) { const u = find(h.id); if (u) { const p = P(u.pos); ctx.fillStyle = '#fde68a'; ctx.font = '11px monospace'; ctx.fillText(h.amount, p.x + 8, p.y - 10 - (30 - (h.until - tick))); } }
    if(scene)for(const shot of scene.shots){const u=find(shot.target)||me,from=P(shot.from),to=P(u.pos),k=Math.min(1,(tick-shot.start)/Math.max(1,shot.at-shot.start)),x=from.x+(to.x-from.x)*k,y=from.y+(to.y-from.y)*k;ctx.strokeStyle='#ffdfa4';ctx.lineWidth=2;ctx.beginPath();ctx.moveTo(x-4,y+2);ctx.lineTo(x+4,y-2);ctx.stroke();ctx.lineWidth=1;}
    if($('art').checked)world.drawMap(ctx,P,W,true);
    const edge=P({x:0,y:0});ctx.strokeStyle='#c8dadb';ctx.strokeRect(edge.x,edge.y,W(WORLD),W(WORLD));
    ctx.fillStyle = 'rgba(15,20,27,.8)'; ctx.fillRect(8, 870, 520, 22); ctx.fillStyle = '#cbd5e1'; ctx.font = '12px system-ui';
    ctx.fillText(`Verified 5v5 · 1 art px = 1,000 world · ${fixture?'isolated fixture':'8:00 scenario'} · host model`, 14, 885);
    drawMinimap();
  }
  function drawMinimap(){const mini=$('minimap'),c=mini.getContext('2d'),s=mini.width/WORLD;c.imageSmoothingEnabled=false;c.drawImage(world.ground,160,160,960,960,0,0,mini.width,mini.height);
    for(const u of [me,...units].filter(u=>u.hp>0&&(u.id===1||visible(u)||u.tower))){c.fillStyle=u.id===1?'#fff':u.team===0?'#50b7ff':u.team===9?'#d2b77a':'#ff6b6b';c.fillRect(u.pos.x*s-1,u.pos.y*s-1,u.champion?3:2,u.champion?3:2);}
    c.strokeStyle='#fff';c.strokeRect((cam.x-450/cam.s)*s,(cam.y-450/cam.s)*s,900/cam.s*s,900/cam.s*s);
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
      ['Fenrir now', state.fenrir ? (state.fenrir.leg ? 'redirected toward committed aim' : 'running through (4: aimed bump nearby)') : '—'],
      ['Fenrir gap / catch reach',state.fenrir?`${Math.round(dist(state.fenrir.pos,me.pos))} / ${R.fenrir.dashMax+R.fenrir.width}`:'—'],
      ['Dash → Guard taunt',`${R.guard.tauntRange} range · ${R.guard.tauntTicks/60}s · connected Dash required`],
      ['Presentation', painter.loaded ? (painter.manifest.complete_body ? 'Complete animation set' : 'Approved concepts; animations in progress') : 'Loading'],
      ['Visual pose', visualPose().name], ['Last rejection', lastRejection || '—'],
      ['Snapshot Attack / armour',`${me.stats?.attack??100} / ${me.stats?.defence??0}`],['Champion radius',me.radius],['Map scale',`${world.geometry.units_per_pixel} world / art pixel`],
      ['Selected actor',find(selectedTarget)?.name||'—'],['Selected HP / armour',find(selectedTarget)?`${Math.ceil(find(selectedTarget).hp)} / ${find(selectedTarget).stats?.defence??0}`:'—'],
      ...(scene?[['Raw requested / HP dealt',`${scene.metrics.requested} / ${scene.metrics.hp_damage}`],['Damage taken',scene.metrics.taken],['Champion / wave / objective',`${scene.metrics.champions} / ${scene.metrics.minions} / ${scene.metrics.objectives}`]]:[]),
    ].map(([k, v]) => `<span>${k}</span><b>${v}</b>`).join('');
    $('status').textContent = `${Math.floor(tick/3600)}:${String(Math.floor(tick/60)%60).padStart(2,'0')} · tick ${tick} · ${running ? 'running' : 'paused'} · ${R.revision}`;
    $('play').textContent = running ? 'Pause' : 'Play';
  }
  function frameLoop(now) { clock.advance(now, running && !document.hidden, step); draw(); hud(); requestAnimationFrame(frameLoop); }
  canvas.addEventListener('pointerdown', () => canvas.focus());
  canvas.addEventListener('pointerdown',e=>{if(e.button===1||e.altKey){pan={x:e.clientX,y:e.clientY,from:{x:cam.x,y:cam.y},scale:cam.s};canvas.setPointerCapture(e.pointerId);e.preventDefault();}else if(e.button===0){const r=canvas.getBoundingClientRect(),p={x:((e.clientX-r.left)*900/r.width-450)/cam.s+cam.x,y:((e.clientY-r.top)*900/r.height-450)/cam.s+cam.y};selectedTarget=units.filter(u=>u.hp>0&&visible(u)&&dist(u.pos,p)<u.radius+9000).sort((a,b)=>dist(a.pos,p)-dist(b.pos,p))[0]?.id??null;}});
  canvas.addEventListener('pointermove',e=>{if(pan){const r=canvas.getBoundingClientRect();pinned={x:pan.from.x-(e.clientX-pan.x)*900/r.width/pan.scale,y:pan.from.y-(e.clientY-pan.y)*900/r.height/pan.scale};$('zoom').checked=true;}});
  canvas.addEventListener('pointerup',()=>pan=null);
  canvas.addEventListener('wheel',e=>{e.preventDefault();$('zoom').checked=true;$('worldZoom').value=Math.max(.5,Math.min(4,+$('worldZoom').value*(e.deltaY<0?1.1:1/1.1)));},{passive:false});
  canvas.addEventListener('mousemove', e => { const r = canvas.getBoundingClientRect(); aim = {x: ((e.clientX - r.left) * canvas.width / r.width - 450) / cam.s + cam.x, y: ((e.clientY - r.top) * canvas.height / r.height - 450) / cam.s + cam.y}; });
  addEventListener('keydown', e => {
    if (document.activeElement !== canvas || replay) return;
    const k = e.key.toLowerCase();
    if (!['w','a','s','d','1','2','r','3','e','4','f'].includes(k)) return;
    e.preventDefault(); if (e.repeat) return;
    if ('wasd'.includes(k) && k.length === 1) held.add(k);
    if (k === '1') order('sight'); if (k === '2') order('rampage'); if (k === 'r') order('redirect'); if (k === '3') order('guard'); if (k === 'e') order('dash');
    if (k === '4') order(state.fenrir && !state.fenrir.leg ? 'slam_aim' : 'fenrir'); if (k === 'f') basic();
  });
  addEventListener('keyup', e => held.delete(e.key.toLowerCase()));
  const clearInput = () => { held.clear(); clock.reset(); };
  addEventListener('blur', clearInput); canvas.addEventListener('blur', clearInput);
  document.addEventListener('visibilitychange', clearInput);
  $('play').onclick = () => { running = !running; clock.reset(); $('play').textContent = running ? 'Pause' : 'Play'; };
  $('step').onclick = () => { running = false; clock.reset(); step(); draw(); hud(); };
  $('reset').onclick = () => reset();
  $('loadScenario').onclick=()=>{reset();running=true;clock.reset();pinned=null;};
  $('follow').onclick=()=>{pinned=null;$('zoom').checked=true;};
  $('nativeSize').onclick=()=>{$('worldZoom').value=1;$('zoom').checked=true;};
  $('minimap').onclick=e=>{const r=e.currentTarget.getBoundingClientRect();pinned={x:(e.clientX-r.left)/r.width*WORLD,y:(e.clientY-r.top)/r.height*WORLD};$('zoom').checked=true;};
  $('spawnStock').onclick=()=>{const old=find(selectedTarget)?.champion?find(selectedTarget):find(2),id=$('stockChampion').value,def=world.data.champions[id];if(!old)return;
    const s=scene||new GameScenario(world,'laning',+$('level').value||8),u=s.make(old.id,old.team,old.pos,'champion','champ-'+id,def,{name:def.name,job:old.job});units[units.indexOf(old)]=u;selectedTarget=u.id;imagesReady=world.loadSprites([u.sprite]);log('Replaced selected champion with '+def.name+' using extracted body and stats. Full stock skill AI is not emulated.');};
  $('stunMe').onclick = () => { if (tick < me.immuneUntil) log('Kratos is CC-immune: the stun does nothing'); else { me.stunUntil = tick + 60; log('Kratos stunned 1 s'); } };
  $('wallDemo').onclick = () => { const c = wallCrossing(); me.pos = {...c.from}; find(2).pos = {...c.to}; find(2).hp = 1000; aim = {...c.to}; log('A foe across the wall: Chain Dash (E) hops over it'); };
  $('group').onclick = () => { units.filter(u => u.team === 1 && u.champion).forEach((u, i) => { u.pos = world.legal(toward(me.pos, aim, 40000 + i * 12000),u.radius); }); log('Enemies lined up toward the mouse on legal ground'); };
  $('defeat').onclick = () => {me.hp=0;me.deadStart=tick;log('Owner defeat injected into the Lab host; next core frame is dead.');};
  $('preview').onclick = () => startDemo($('demo').value);
  $('record').onclick = () => { if (recording) { $('record').textContent = 'Start recording'; recording = null; }
    else { reset(); window.kratosRecording = recording = {version: 1, transport: 1, host_revision:'game-world-1',map_sha256:world.data.source.map_sha256,rules_revision: R.revision, frames: []}; $('record').textContent = 'Stop recording'; log('Recording inputs, outputs and host observations (maximum two minutes).'); } };
  $('saveRecording').onclick = () => { const data = window.kratosRecording; if (!data) return log('Start a recording first.');
    const url = URL.createObjectURL(new Blob([JSON.stringify(data)], {type: 'application/json'})), a = document.createElement('a'); a.href = url; a.download = 'kratos-lab-recording.json'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); };
  function loadRecording(data) {
    if (data.version !== 1 || data.transport !== 1 || data.rules_revision !== R.revision || !Array.isArray(data.frames) || !data.frames.length || data.frames.length > 7200 || data.frames.some(e => !e.input || !e.output || !e.host))
      throw new Error('Recording is inspectable as JSON; replay requires the original Kratos rules and transport v1.');
    reset(); recording = null; replay = copy(data);scene=null;fixture=true;running = false;
    log('Replay loaded. Play/step verifies every core output against the saved trace; actor observations come from the saved host snapshot.');
    if(data.host_revision!=='game-world-1'||data.map_sha256!==world.data.source.map_sha256)log('Legacy/different-map host recording: core replay is verified, but its saved layout is not the current verified mid-game scene.');
  }
  $('loadRecording').onchange = async event => { try { loadRecording(JSON.parse(await event.target.files[0].text())); } catch (error) { log(error.message); } };
  reset();
  window.kratosLab = {step, reset, order, basic, wallCrossing, pause() { running = false; clock.reset(); }, setAim(p) { aim = {...p}; }, stunMe() { $('stunMe').onclick(); },
    startDemo, loadRecording, painter, world,clock, visualPose,render:draw,get scene(){return scene;},get imagesReady(){return imagesReady;},get camera(){return cam;},get lastTrace() { return lastTrace; }, get held() { return [...held]; }, get replayIndex() { return replayIndex; },
    get state() { return state; }, get me() { return me; }, get units() { return units; }, get hits() { return hits; }, get tick() { return tick; }, get walls() { return WALLS; }, immuneUsed, wallAt};
  requestAnimationFrame(frameLoop);
})().catch(error=>{document.getElementById('status').textContent='Game reference unavailable: '+error.message;console.error(error);});
