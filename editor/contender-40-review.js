(async () => {
  'use strict';
  const $ = id => document.getElementById(id);
  const rules = CONTENDER_RULES;
  const smother = rules.smother;
  const assets = '../docs/contender-40-art-preview/';
  async function request(url) {
    const response = await fetch(url);
    if (!response.ok) throw Error('Could not load ' + url);
    return response;
  }
  async function loadImage(file) {
    const image = new Image();
    image.src = assets + file;
    await image.decode();
    return image;
  }
  if (rules.revision !== 'contender-4.0') throw Error('This preview requires Contender 4.0.');
  const painter = new ContenderPainter();
  const [art, wasm] = await Promise.all([
    request(assets + 'sources.json').then(response => response.json()),
    request('contender.wasm').then(response => response.arrayBuffer()),
    painter.ready,
  ]);
  const [smotherImage, cloneImage, module] = await Promise.all([
    loadImage(art.smother.file), loadImage(art.clone.file), WebAssembly.instantiate(wasm),
  ]);
  const core = module.instance.exports;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const origin = {x: 480000, y: 480000};
  function at(dx, dy) {
    return {x: origin.x + Math.round(dx), y: origin.y + Math.round(dy)};
  }
  function step(input) {
    const data = encoder.encode(JSON.stringify({version: 1, rules_revision: rules.revision, frame: input}));
    const ptr = core.contender_input(data.length);
    new Uint8Array(core.memory.buffer, ptr, data.length).set(data);
    const out = core.contender_step(data.length);
    const result = JSON.parse(decoder.decode(new Uint8Array(core.memory.buffer, out, core.contender_output_len())));
    if (result.error || result.rejection) throw Error(result.error || result.rejection);
    return result;
  }
  function simulate(scene, facing, diagonal) {
    const army = scene === 'army' || scene === 'anchor';
    const anchoring = scene === 'anchor';
    const theta = diagonal ? Math.PI / 8 : 0;
    const sight = {x: facing * Math.cos(theta), y: Math.sin(theta)};
    const aim = at(sight.x * smother.range, sight.y * smother.range);
    const firstContact = army ? smother.range * 1.65 : smother.range * .74;
    const cross = scene === 'miss' ? smother.width * 2.1 : 0;
    const target = {id: 2, team: 1, champion: true, tower: false, visible: true, targetable: true,
      farmable: false, radius: 3000, hp: 40000, max_hp: 40000,
      pos: at(sight.x * firstContact - sight.y * cross, sight.y * firstContact + sight.x * cross)};
    const anchorTick = 1;
    const summons = new Map();
    const armyTick = anchoring ? 18 : 1;
    if (anchoring) summons.set(anchorTick, {kind:'s2', aim:at(-facing * 36000, 60000)});
    if (army) summons.set(armyTick, {kind:'ult', aim});
    const castTick = army ? armyTick + 30 : 1;
    summons.set(castTick, {kind:'s3', aim});
    const end = anchoring ? anchorTick + rules.anchor.lifeTicks + 12
      : army ? armyTick + rules.cloneArmy.lifeTicks + 12 : castTick + smother.flightTicks + rules.shadowborne.ticks + 12;
    core.contender_reset();
    const frames = [];
    let owner = origin;
    let impacts = [];
    let lines = [];
    let landed = 0;
    const blank = {tick:0, owner, target:{...target}, shadows:[], lines:[], impacts:[], flurries:0, landed};
    frames.push(blank);
    for (let tick = 1; tick <= end; tick++) {
      const result = step({
        tick, owner:1, team:0, alive:true, pos:owner, hp:1000, max_hp:1000, attack:100, level:18,
        ult_unlocked:true, home:origin, units:[target], profile:Array(13).fill(50),
        manual:true, order:summons.get(tick) ?? null, landed_hits:landed,
      });
      landed = 0;
      impacts = impacts.filter(cue => tick < cue.until);
      for (const e of result.effects) {
        if (e.kind === 'fx' && e.name === 'ct_echo') impacts.push({at:e.at, tick, until:tick + e.ticks});
        if (e.kind === 'blink') owner = e.to;
        if (e.kind === 'damage' && e.target === target.id) {
          target.hp = Math.max(1, target.hp - e.amount);
          if (e.source === 'smother' || e.source === 'grab' || e.source === 'counter') landed++;
        }
      }
      const active = result.state.casts.filter(cast => cast.type === 'smother');
      const finishing = lines.filter(cast => cast.landing === tick).map(cast => ({...cast, cursor:cast.to}));
      lines = active;
      frames.push({tick, owner, target:{...target}, shadows:result.state.shadows, lines:[...lines, ...finishing],
        impacts:[...impacts], flurries:result.state.casts.filter(cast => cast.type === 'flurry').length});
    }
    return frames;
  }
  const selects = ['scene', 'facing', 'angle', 'speed'].map($);
  const stages = [1, 3].map((scale,i) => ({scale, ctx:$(i ? 'large' : 'small').getContext('2d')}));
  const width = $('small').width;
  const height = $('small').height;
  const unitsToPixels = value => value / 1000;
  const millisecondsPerSecond = 1000;
  let frames = [];
  let current = 0;
  let playing = false;
  let started = performance.now();
  let startFrame = 0;
  function drawSprite(ctx, image, crop, x, y, w, h) {
    ctx.drawImage(image, ...crop, Math.round(x), Math.round(y), Math.round(w), Math.round(h));
  }
  function drawTrail(ctx, line, screen) {
    const a = screen(line.from);
    const b = screen(line.cursor);
    const length = Math.hypot(b.x - a.x, b.y - a.y);
    if (length < 1) return;
    const breadth = unitsToPixels(smother.width * 2);
    ctx.save();
    ctx.translate(a.x, a.y);
    ctx.rotate(Math.atan2(b.y - a.y, b.x - a.x));
    ctx.globalAlpha = line.caster.by === 'me' ? 1 : .70;
    drawSprite(ctx, smotherImage, art.smother.trail, 0, -breadth / 2, length, breadth);
    ctx.restore();
  }
  function cloneBody(ctx, shadow, facing, screen, tick) {
    const {x, y} = screen(shadow.pos);
    if (shadow.anchor) {
      ctx.fillStyle = '#965cc8';
      ctx.fillRect(x - 9, y + 19, 19, 3);
      painter.draw(ctx, 'effects', 'body_idle0' + (facing < 0 ? '_l' : ''), shadow.pos, 0, screen, unitsToPixels);
      return;
    }
    const [w, h] = art.clone.frame;
    const birth = shadow.until - rules.cloneArmy.lifeTicks;
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(facing, 1);
    ctx.globalAlpha = art.clone.alpha * Math.min(1, .55 + (tick - birth) / 8);
    drawSprite(ctx, cloneImage, art.clone.crop, -w / 2, 20 - h, w, h);
    ctx.restore();
  }
  function draw(frame) {
    const facing = Number($('facing').value);
    const left = facing < 0;
    const screen = p => ({x:Math.round((left ? 172 : 72) + unitsToPixels(p.x - origin.x)),
      y:Math.round(58 + unitsToPixels(p.y - origin.y))});
    const clones = frame.shadows.filter(shadow => !shadow.anchor).length;
    const anchor = frame.shadows.length > clones;
    for (const {ctx, scale} of stages) {
      const light = scale === 3;
      ctx.setTransform(scale, 0, 0, scale, 0, 0);
      ctx.imageSmoothingEnabled = false;
      ctx.fillStyle = light ? '#b8c3a6' : '#263b45';
      ctx.fillRect(0, 0, width, height);
      ctx.strokeStyle = light ? '#adba9d' : '#304852';
      ctx.beginPath();
      for (let i = 24; i < width; i += 24) { ctx.moveTo(i, 0);ctx.lineTo(i, height); }
      for (let i = 24; i < height; i += 24) { ctx.moveTo(0, i);ctx.lineTo(width, i); }
      ctx.stroke();
      for (const c of frame.lines) drawTrail(ctx, c, screen);
      for (const shadow of frame.shadows) cloneBody(ctx, shadow, facing, screen, frame.tick);
      const target = screen(frame.target.pos);
      painter.draw(ctx, 'body', left ? 'idle' : 'idle_l', frame.target.pos, 0, screen, unitsToPixels, true, .7);
      ctx.fillStyle = '#e49a90';
      ctx.fillRect(target.x - 9, target.y - 26, 18, 2);
      painter.draw(ctx, 'body', left ? 'idle_l' : 'idle', frame.owner, 0, screen, unitsToPixels, true);
      const [w, h] = art.smother.contactFrame;
      const shown = new Set();
      for (const impact of [...frame.impacts].reverse()) {
        const point = screen(impact.at);
        const key = point.x + ':' + point.y;
        if (shown.has(key)) continue;
        shown.add(key);
        ctx.globalAlpha = 1 - .8 * (frame.tick - impact.tick) / (impact.until - impact.tick);
        drawSprite(ctx, smotherImage, art.smother.contact, point.x - w / 2, point.y - h / 2, w, h);
        ctx.globalAlpha = 1;
      }
    }
    $('timeline').value = frame.tick;
    $('time').value = (frame.tick / 60).toFixed(2) + 's';
    const detail = [clones + (clones === 1 ? ' clone' : ' clones')];
    if (anchor) detail.push('Anchor active');
    if (frame.lines.length) detail.push(frame.lines.length + ' Smother ' + (frame.lines.length === 1 ? 'lane' : 'lanes'));
    if (frame.impacts.length) detail.push('Contact');
    if (frame.flurries) detail.push('Shadowborne');
    $('status').textContent = detail.join(' · ');
  }
  function pause() {
    playing = false;
    $('pause').textContent = 'Play';
  }
  function play() {
    playing = true;
    startFrame = current === frames.length - 1 ? 0 : current;
    started = performance.now();
    $('pause').textContent = 'Pause';
  }
  function replay() {
    current = 0;
    draw(frames[0]);
    play();
  }
  function select() {
    frames = simulate($('scene').value, Number($('facing').value), $('angle').value === 'diagonal');
    $('timeline').max = frames.length - 1;
    replay();
  }
  for (const input of selects.slice(0, 3)) input.onchange = select;
  $('scene').addEventListener('change', () => {$('speed').value = ['army','anchor'].includes($('scene').value) ? '1' : '.25';});
  $('speed').onchange = () => { if (playing) play(); };
  $('replay').onclick = replay;
  $('pause').onclick = () => playing ? pause() : play();
  $('timeline').oninput = () => {
    pause();
    current = Number($('timeline').value);
    draw(frames[current]);
  };
  select();
  for (const el of [...selects, $('timeline'), $('replay'), $('pause')]) el.disabled = false;
  function animate(now) {
    if (playing) {
      const progress = Math.max(0, Math.floor((now - started) * Number($('speed').value) * 60 / millisecondsPerSecond));
      const next = Math.min(frames.length - 1, (startFrame + progress) % (frames.length + 30));
      if (next !== current) {
        current = next;
        draw(frames[current]);
      }
    }
    requestAnimationFrame(animate);
  }
  requestAnimationFrame(animate);
})().catch(error => {
  document.getElementById('error').hidden = false;
  document.getElementById('error').textContent = error.message;
  throw error;
});
