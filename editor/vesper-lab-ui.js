/* Rift Lab: the practice arena (hand placement or the ranked planner) with the two gauntlets, the rank duel, the
 * bonus grid, the dimension browser (rift, punch and swipe of all 100), the codex, the mastery comparison and native
 * diagnostics. All rules come from VesperCombat (a 1:1 port of the native engine). */
(function (root) {
  'use strict';
  const D = root.RiftData, V = root.VesperCombat, M = root.VesperMath, VR = root.VesperRenderer;
  const $ = s => document.querySelector(s);
  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const FAM = VR.FAM;
  const GROUPS = [['offense', 'Offense'], ['engage', 'Engage'], ['defense', 'Defense'], ['team', 'Team']];
  // the rank from which the planner uses a technique well (its habit gate)
  const WELL = { chain: 3, crossfire: 3, bank: 3, rain: 3, volley: 3, implode: 3, domino: 3, opunch: 2, step: 4, gate: 5, skydrop: 5, kidnap: 6, sinktrap: 6, swap: 5, fling: 5, hook: 5, phase: 4, curtain: 4, reflect: 2, rescue: 4, pocket: 4, lifeline: 4, escort: 5, rotation: 5, seedtrap: 6, sweep: 3, infuse: 6, stitch: 6 };
  const MOVE_WELL = { swipe: 3, shatter: 2, overload: 7, vent: 4, awaken: 5, overclock: 7 };
  const rankName = (rank, top) => rank >= 7 && top === 1 ? D.top_name : D.ranks[rank];

  // ------------------------------------------------------------------ placement recipes
  // Each technique: the clicks it needs. `pick` steps select a fighter (enemy / ally) or a live rift.
  const RECIPES = {
    chain: { steps: 'nodes', text: 'Click up to 5 spots for the chain (Vesper → … → exit), then Enter or double-click.' },
    crossfire: { steps: [], text: 'Needs a live chain and 2+ rifts: splits her next shots for 3 s.' },
    bank: { steps: ['spot'], text: 'Click where the Mirror Pane goes; it joins the chain facing Vesper.' },
    rain: { steps: ['spot'], text: 'Click where the Sky Rift opens overhead.' },
    volley: { steps: ['spot'], text: 'Click the Slash centre; it lies across Vesper’s line of fire.' },
    implode: { steps: ['rift'], text: 'Click a live rift to cave it in.' },
    domino: { steps: [], text: 'Caves in every chain rift and Twin in turn, 15 ticks apart.' },
    opunch: { steps: [], text: 'Spends up to 60 stress on the next rift punch (+1% per point).' },
    step: { steps: ['spot'], text: 'Click the exit: Vesper steps in a rift at her feet and out there.' },
    gate: { steps: ['spot', 'spot'], text: 'Click the entrance (on an ally’s path), then the exit.' },
    skydrop: { steps: ['spot', 'spot'], text: 'Click the entrance, then the Sky Rift exit over the enemies.' },
    kidnap: { steps: ['enemy', 'spot'], text: 'Click an enemy, then where to spit them out.' },
    sinktrap: { steps: ['spot', 'spot'], text: 'Click the Sinkhole, then its exit.' },
    swap: { steps: ['spot', 'spot'], text: 'Click the two Twin Rifts: the fighters nearest each trade places.' },
    fling: { steps: ['spot', 'spot'], text: 'Click where the Wandering Rift starts, then where it ends.' },
    hook: { steps: ['enemy', 'spot'], text: 'Click an enemy to anchor, then the exit they are reeled to.' },
    phase: { steps: ['spot'], text: 'Click the exit: Vesper hides for 45 ticks, then leaves there.' },
    curtain: { steps: ['spot', 'spot'], text: 'Click both ends of the Slash curtain.' },
    reflect: { steps: ['spot', 'spot'], text: 'Click both ends of the Mirror Pane.' },
    rescue: { steps: ['ally', 'spot'], text: 'Click an ally, then the safe exit.' },
    pocket: { steps: ['ally'], text: 'Click an ally to hide inside a rift for 60 ticks.' },
    lifeline: { steps: ['spot'], text: 'Click a safe spot: below 25% HP she is pulled there.' },
    escort: { steps: ['ally'], text: 'Click an ally: the chain’s exit rides on them.' },
    rotation: { steps: ['spot', 'spot'], text: 'Click the entrance near a far ally, then the exit near the fight.' },
    seedtrap: { steps: ['spot'], text: 'Click where the hidden Seed goes.' },
    sweep: { steps: ['spot', 'spot'], text: 'Click the start and end of the sweep.' },
    infuse: { steps: ['rift'], text: 'Click a live rift: a hexorb\u2019s tech module pulses from it for the rest of its life.' },
    stitch: { steps: ['rift', 'rift'], text: 'Click two Tears within 40000 of each other.' },
  };

  const S = { arena: null, renderer: null, paused: false, tech: 'chain', clicks: [], drag: null, log: [], habits: {}, compareRunning: false, swipe: false };

  function options(select, rows, value) {
    select.innerHTML = rows.map(([v, t]) => `<option value="${esc(v)}">${esc(t)}</option>`).join('');
    if (value !== undefined) select.value = String(value);
  }
  function rankRows() { return D.ranks.map((r, i) => [i, `${i} · ${r}${i === 7 ? ' (Top 10)' : ''}`]); }
  function topValue() { const v = Number($('#position').value); return Number($('#rank').value) >= 7 && v ? v : null; }

  function newArena() {
    const seed = Math.max(1, Number($('#seed').value) || 1), rank = Number($('#rank').value);
    const a = $('#layout').value === 'duel' ? V.duel(seed, rank, topValue()) : V.teamfight(seed, rank, topValue());
    a.core.hand_placed = !$('#planner').checked;
    a.frozen = $('#freeze').checked;
    a.core.armed = Number($('#punchDim').value);
    a.core.free_casts = $('#freeCasts').checked;
    a.core.modules = currentModules();
    S.arena = a;
    S.clicks = [];
    S.log = [];
    S.renderer.events = [];
    S.renderer.camera = null;
    renderEvents();
  }

  // ------------------------------------------------------------------ the arena loop
  function tickOnce() {
    const a = S.arena;
    a.keep_cmds = true;
    a.cmds = [];
    a.step();
    S.renderer.note(a.cmds, a.tick);
    for (const c of a.cmds) if (c.k === 'Log') S.log.unshift(c.s);
    if (S.log.length > 80) S.log.length = 80;
  }
  let last = 0, acc = 0;
  function frame(now) {
    const dt = Math.min(100, now - (last || now));
    last = now;
    if (!S.paused && S.visible !== false) {
      acc += dt;
      while (acc >= 1000 / 60) { tickOnce(); acc -= 1000 / 60; }
    }
    S.renderer.draw(S.arena, { paused: S.paused, dragging: S.drag && S.drag.id });
    drawPlacement();
    if (now - (S.lastPanel || 0) > 200) { S.lastPanel = now; renderReadouts(); refreshDimOptions(); }
    requestAnimationFrame(frame);
  }
  function drawPlacement() {
    const c = S.renderer.ctx, r = S.renderer;
    c.save();
    c.strokeStyle = '#ffd36b';
    c.fillStyle = '#ffd36b';
    let prev = null;
    for (const k of S.clicks) {
      const X = r.sx(k.x), Y = r.sy(k.y);
      c.beginPath(); c.arc(X, Y, 5, 0, Math.PI * 2); c.fill();
      if (prev) { c.setLineDash([4, 4]); c.beginPath(); c.moveTo(r.sx(prev.x), r.sy(prev.y)); c.lineTo(X, Y); c.stroke(); c.setLineDash([]); }
      prev = k;
    }
    c.restore();
  }
  function renderReadouts() {
    const a = S.arena, core = a.core, me = a.unit(0), tick = a.tick;
    const nodes = core.chainNodes(tick);
    let prev = [me.f.x, me.f.y];
    const lines = [];
    const links = [];
    for (const n of nodes) {
      const len = M.dist(prev[0], prev[1], n.x, n.y);
      links.push(len);
      lines.push(`link ${links.length}: ${len} units → keeps ${(M.keepBp(len) / 100).toFixed(1)}% · ${D.dimensions[n.dim].name}`);
      prev = [n.x, n.y];
    }
    if (nodes.length) lines.push(`through the chain: ${(M.chainBp(links) / 100).toFixed(1)}%`);
    else lines.push(`no chain: she punches straight at anyone within ${D.constants.punch_reach}`);
    lines.push(core.punching ? (core.aiming ? 'punching through the chain (feet planted)' : 'punching: every basic attack breaks a hole') : core.leftBroken(tick) ? 'left gauntlet broken: normal attacks only' : 'normal attacks (no orb ready or nobody in reach)');
    if (core.combo > 0) lines.push(`orb combo x${core.combo} (+${core.combo * D.constants.combo_step}%) · last family ${D.families[core.last_family].name}`);
    const pending = core.rifts.filter(r => !r.opened);
    if (pending.length) lines.push(`${pending.length} rift(s) still opening`);
    $('#chainReadout').textContent = lines.join('\n');
    const s = core.stats;
    $('#counters').textContent = `stress ${core.stress} · orbs ${core.pureReady(tick)}/${core.sockets()} pure · ${core.hexReady(tick)}/${core.hexSlots(tick)} hex\nHexcore ${core.completion}%${core.awakened(tick) ? ` · AWAKENED ${((core.awaken_until - tick) / 60).toFixed(1)}s · corruption ${core.corruption}` : ''}\npunches ${s.punches} (${s.fused} fused) · avg ${s.landed ? (s.eff_sum / s.landed / 100).toFixed(1) : '0'}% · swipes ${s.swipes} hit ${s.swipe_hits}\nshatters ${s.useful_shatters}/${s.shatters} · breaks L${s.left_breaks} R${s.right_breaks} · twists ${s.twists}\ndamage ${s.damage} · rifts ${s.rifts_opened} opened, ${s.breaks} broken\nkidnaps ${s.useful_kidnaps}/${s.kidnaps} · gates ${s.engaged_gates}/${s.gates} · traps ${s.trap_catches}/${s.traps}`;
    renderGauntlets();
    renderEvents();
  }
  const famColor = f => f === 255 || f === null ? '#526a86' : FAM[f];
  function renderGauntlets() {
    const core = S.arena.core, tick = S.arena.tick;
    const broken = core.leftBroken(tick);
    $('#sockets').innerHTML = core.pure.map((s, i) => {
      if (!s.open) return i < D.constants.orbs_max ? '<span class="sock empty" title="empty socket"></span>' : '';
      const ready = !broken && tick >= s.ready_at;
      const cls = 'sock' + (ready ? ' ready' : '') + (s.over ? ' over' : '');
      const left = Math.max(0, s.ready_at - tick);
      return `<span class="${cls}" style="color:${famColor(s.family)}" title="${s.over ? 'overloaded ' : ''}orb ${i + 1}: ${ready ? 'ready' : broken ? 'broken' : 'recovering ' + (left / 60).toFixed(1) + 's'}">${ready ? '<i></i>' : ''}</span>`;
    }).join('');
    const cap = D.constants.stress_cap;
    $('#stressBar').style.width = Math.min(100, core.stress * 100 / cap) + '%';
    $('#stressBar').style.background = core.stress * 10 >= cap * 9 ? '#ff5d5d' : core.stress * 10 >= cap * 7 ? '#ffb84d' : '#7fd1ff';
    $('#stressText').textContent = broken ? `BROKEN ${((core.left_broken_until - tick) / 60).toFixed(1)}s` : `${core.stress}/${cap}`;
    const awake = core.awakened(tick);
    $('#hexBar').style.width = ((core.completion - 70) / 30 * 100) + '%';
    $('#hexBar').style.background = core.rightBroken(tick) ? '#ff5d5d' : awake ? (core.corruption >= 60 ? '#d24a9a' : '#ffffff') : '#7fd1ff';
    $('#hexText').textContent = core.rightBroken(tick) ? `BROKEN ${((core.right_broken_until - tick) / 60).toFixed(1)}s` : awake ? `AWAKE · corruption ${core.corruption}` : core.completion + '%';
    const slots = core.hexSlots(tick);
    $('#hexSlots').innerHTML = Array.from({ length: 6 }, (_, i) => {
      if (i >= slots) return '<span class="sock hexo empty" title="locked slot (Hexcore research)"></span>';
      const ready = tick >= core.hex[i];
      return `<span class="sock hexo${ready ? ' ready' : ''}" style="color:#7fd1ff" title="hexorb ${i + 1}: ${D.modules[core.modules[i]].name}${ready ? '' : ' · recovering'}">${ready ? '<i></i>' : ''}</span>`;
    }).join('');
    for (const b of document.querySelectorAll('[data-move]')) b.classList.toggle('active', S.swipe && b.dataset.move === 'swipe');
  }
  function currentModules() {
    const sel = [...document.querySelectorAll('#modules select')].map(x => Number(x.value));
    return sel.length === 6 ? sel : [0, 1, 2, 3, 4, 5];
  }
  const showEff = e => !e || e[0] === 255 ? '' : `${D.prims[e[0]].id}${['a', 'b', 'c'].map((k, i) => e[i + 1] ? ` ${k}=${e[i + 1]}` : '').join('')}`;
  function dimLine(d) {
    const z = [d.prim, d.a, d.b, d.c];
    const dl = x => [showEff(x.main), showEff(x.rider)].filter(Boolean).join(' + ');
    return `<span class="deliv"><b>rift</b> ${esc(showEff(z))}</span><span class="deliv"><b>punch</b> ${esc(dl(d.punch))}</span><span class="deliv"><b>swipe</b> ${esc(dl(d.swipe))}</span>`;
  }
  // a dimension's cooldown and, in the live arena, how long it (or its family) still rests
  const secs = t => (t / 60).toFixed(t % 60 ? 1 : 0) + 's';
  function resting(i) {
    const a = S.arena;
    if (!a || a.core.free_casts) return 0;
    const c = a.core, d = D.dimensions[i];
    return Math.max(0, c.dim_ready[i] - a.tick, c.family_ready[d.family] - a.tick);
  }
  function restLine(i) {
    const d = D.dimensions[i], left = resting(i);
    const own = S.arena ? Math.max(0, S.arena.core.dim_ready[i] - S.arena.tick) : 0;
    return `cooldown ${secs(d.cd)}${d.spectacular ? ' (signature)' : ''}` + (left ? ` · <b class="resting">${own ? 'resting' : 'family resting'} ${secs(left)}</b>` : ' · ready');
  }
  function renderDimInfo() {
    const ri = Number($('#riftDim').value), pi = Number($('#punchDim').value);
    const r = D.dimensions[ri], p = D.dimensions[pi];
    $('#dimInfo').innerHTML = `${esc(r.name)} (rift): ${esc(D.prims[r.prim].text)} · ${restLine(ri)}<br>${esc(p.name)} (punch): ${esc(showEff(p.punch.main))}${p.punch.rider ? ' + ' + esc(showEff(p.punch.rider)) : ''} · ${restLine(pi)}`;
  }
  // resting dimensions are greyed in both selects (refreshed with the readouts)
  function refreshDimOptions() {
    for (const sel of [$('#riftDim'), $('#punchDim')]) {
      for (const o of sel.options) {
        const left = resting(Number(o.value));
        o.classList.toggle('resting', left > 0);
        const base = o.dataset.base || (o.dataset.base = o.textContent);
        o.textContent = left ? `${base} · ${secs(left)}` : base;
      }
    }
    renderDimInfo();
  }
  function move(id) {
    const a = S.arena;
    if (id === 'swipe') { S.swipe = !S.swipe; $('#hint').textContent = S.swipe ? 'Click where she swipes: the tear runs from her toward that point.' : 'Swipe cancelled.'; renderGauntlets(); return; }
    const cmds = a.place({ tech: V.MV[id], rifts: [], subject: null, uses: [], point: null, dim: null, score: 0, habit: '' });
    S.renderer.note(cmds, a.tick);
    for (const c of cmds) if (c.k === 'Log') S.log.unshift(c.s);
    if (!cmds.some(c => c.k === 'Log')) $('#hint').textContent = `${D.moves.find(m => m.id === id).name} is not possible right now.`;
    renderReadouts();
  }
  function renderEvents() {
    $('#events').innerHTML = S.log.slice(0, 40).map(l => `<li>${esc(l)}</li>`).join('');
  }

  // ------------------------------------------------------------------ hand placement
  function techIndex(id) { return D.techniques.findIndex(t => t.id === id); }
  function allowedTypes(id) { return D.techniques[techIndex(id)].rifts; }
  function riftKind(id, fallback) {
    const chosen = $('#riftType').value;
    return allowedTypes(id).includes(chosen) ? V.R[chosen] : V.R[fallback];
  }
  function selectTech(id) {
    S.tech = id;
    S.clicks = [];
    for (const b of document.querySelectorAll('[data-tech]')) b.classList.toggle('active', b.dataset.tech === id);
    const t = D.techniques[techIndex(id)];
    options($('#riftType'), t.rifts.map(r => [r, D.rifts.find(x => x.id === r).name]));
    const cost = D.rifts.find(x => x.id === t.rifts[0]).cost + t.orbs;
    $('#techInfo').innerHTML = `<strong>${esc(t.name)}</strong> · ${esc(t.group)} · ${esc(RECIPES[id].text)}<br>${esc(t.text)}. Stress +${t.stress}${t.cooldown ? ` · per-target cooldown ${t.cooldown / 60}s` : ''} · costs ${cost} orb${cost > 1 ? 's' : ''} (pure or hex) · the planner uses it well from rank ${WELL[id]} (${esc(D.ranks[WELL[id]])}).`;
    $('#hint').textContent = RECIPES[id].text;
    if (!RECIPES[id].steps.length) finish();
  }
  function nearest(list, x, y, max) {
    let best = null, bd = max * max;
    for (const o of list) { const d = M.dist2(o.x, o.y, x, y); if (d <= bd) { bd = d; best = o; } }
    return best;
  }
  function onArenaClick(x, y) {
    if (S.swipe) {
      S.swipe = false;
      const a = S.arena;
      const cmds = a.place({ tech: V.MV.swipe, rifts: [], subject: null, uses: [], point: [x, y], dim: Number($('#punchDim').value), score: 0, habit: '' });
      S.renderer.note(cmds, a.tick);
      for (const c of cmds) if (c.k === 'Log') S.log.unshift(c.s);
      const sd = Number($('#punchDim').value);
      if (!a.core.castOk(sd, a.tick)) $('#hint').textContent = `${D.dimensions[sd].name} is resting ${secs(resting(sd))}: pick another dimension or tick Ignore cooldowns.`;
      else if (!a.core.stats.swipes) $('#hint').textContent = 'Swipe needs 2 orbs and a free rift slot.';
      renderReadouts();
      return;
    }
    const a = S.arena, recipe = RECIPES[S.tech];
    const steps = recipe.steps === 'nodes' ? null : recipe.steps;
    const need = steps ? steps[S.clicks.length] : 'spot';
    const fighters = a.fighters();
    const pickR = S.renderer.camera.w / S.renderer.canvas.width * 18;
    if (need === 'enemy' || need === 'ally') {
      const f = nearest(fighters.filter(f => need === 'enemy' ? f.team !== 0 : (f.team === 0 && f.id !== 0)), x, y, pickR);
      if (!f) { $('#hint').textContent = `Click ${need === 'enemy' ? 'an enemy' : 'an ally'}.`; return; }
      S.clicks.push({ x: f.x, y: f.y, id: f.id });
    } else if (need === 'rift') {
      const r = nearest(a.core.rifts.filter(r => V.riftLive(r) && (S.tech !== 'stitch' || r.kind === V.R.tear)), x, y, pickR * 2);
      if (!r) { $('#hint').textContent = 'Click a live rift.'; return; }
      S.clicks.push({ x: r.x, y: r.y, uid: r.uid });
    } else S.clicks.push({ x, y });
    if (steps && S.clicks.length >= steps.length) finish();
    else if (!steps && S.clicks.length >= 5) finish();
  }
  function finish() {
    const a = S.arena, me = a.unit(0).f, k = S.clicks, T = S.tech, R = V.R, Sp = V.Spec;
    const perp = (x, y, half) => { const [dx, dy] = M.along(0, 0, -(y - me.y), x - me.x, half); return [x - dx, y - dy, x + dx, y + dy]; };
    let rifts = [], subject = null, uses = [];
    switch (T) {
      case 'chain': rifts = k.map(c => Sp.node(riftKind(T, 'tear'), c.x, c.y)); break;
      case 'bank': { const [ax, ay, bx, by] = perp(k[0].x, k[0].y, 20000); rifts = [{ ...Sp.line(R.mirror, ax, ay, bx, by), chain: true }]; break; }
      case 'rain': rifts = [Sp.at(R.sky, k[0].x, k[0].y)]; break;
      case 'volley': { const [ax, ay, bx, by] = perp(k[0].x, k[0].y, 30000); rifts = [Sp.line(R.slash, ax, ay, bx, by)]; break; }
      case 'implode': uses = [k[0].uid]; break;
      case 'domino': uses = a.core.rifts.filter(r => V.riftLive(r) && (r.chain !== null || r.kind === R.twin)).map(r => r.uid).sort((x, y) => x - y); break;
      case 'step': case 'phase': rifts = [Sp.entrance(riftKind(T, 'tear'), me.x, me.y), Sp.at(riftKind(T, 'tear'), k[0].x, k[0].y)]; break;
      case 'gate': case 'rotation': rifts = [Sp.entrance(riftKind(T, 'tear'), k[0].x, k[0].y), Sp.at(riftKind(T, 'tear'), k[1].x, k[1].y)]; break;
      case 'skydrop': rifts = [Sp.entrance(R.tear, k[0].x, k[0].y), Sp.at(R.sky, k[1].x, k[1].y)]; break;
      case 'kidnap': subject = k[0].id; rifts = [Sp.entrance(riftKind(T, 'tear'), k[0].x, k[0].y), Sp.at(riftKind(T, 'tear'), k[1].x, k[1].y)]; break;
      case 'sinktrap': rifts = [Sp.entrance(R.sinkhole, k[0].x, k[0].y), Sp.at(R.tear, k[1].x, k[1].y)]; break;
      case 'swap': rifts = [Sp.entrance(R.twin, k[0].x, k[0].y), Sp.at(R.twin, k[1].x, k[1].y)]; break;
      case 'fling': case 'sweep': rifts = [Sp.line(R.wander, k[0].x, k[0].y, k[1].x, k[1].y)]; break;
      case 'hook': subject = k[0].id; rifts = [{ ...Sp.at(R.anchor, k[0].x, k[0].y), host: k[0].id }, Sp.at(R.tear, k[1].x, k[1].y)]; break;
      case 'curtain': rifts = [Sp.line(R.slash, k[0].x, k[0].y, k[1].x, k[1].y)]; break;
      case 'reflect': rifts = [Sp.line(R.mirror, k[0].x, k[0].y, k[1].x, k[1].y)]; break;
      case 'rescue': subject = k[0].id; rifts = [Sp.entrance(riftKind(T, 'tear'), k[0].x, k[0].y), Sp.at(riftKind(T, 'tear'), k[1].x, k[1].y)]; break;
      case 'pocket': subject = k[0].id; rifts = [Sp.entrance(riftKind(T, 'tear'), k[0].x, k[0].y)]; break;
      case 'lifeline': subject = 0; rifts = [Sp.at(R.anchor, k[0].x, k[0].y)]; break;
      case 'escort': subject = k[0].id; rifts = [{ ...Sp.at(R.anchor, k[0].x, k[0].y), host: k[0].id, chain: true }]; break;
      case 'seedtrap': rifts = [Sp.at(R.seed, k[0].x, k[0].y)]; break;
      case 'infuse': uses = [k[0].uid]; break;
      case 'stitch': uses = [k[0].uid, k[1].uid]; break;
      default: break;
    }
    const dim = Number($('#riftDim').value);
    rifts = rifts.map(r => ({ ...r, dim }));
    const action = { tech: techIndex(T), rifts, subject, uses, point: null, dim: null, score: 0, habit: '' };
    const cost = rifts.reduce((s, r) => s + D.rifts[r.kind].cost, 0) + D.techniques[techIndex(T)].orbs;
    const ready = a.core.orbsReady(a.tick);
    if (ready < cost) { $('#hint').textContent = `Not enough orbs: needs ${cost}, ${ready} ready.`; S.clicks = []; return; }
    if (T === 'infuse' && !a.core.hexReady(a.tick)) { $('#hint').textContent = 'Infuse needs a ready hexorb.'; S.clicks = []; return; }
    if (a.core.openCount() + rifts.length > D.constants.max_rifts) { $('#hint').textContent = 'At most 5 rifts can be open.'; S.clicks = []; return; }
    if (T === 'crossfire' && !a.core.chainNodes(a.tick).length) { $('#hint').textContent = 'Needs a live chain.'; S.clicks = []; return; }
    if (rifts.length && !a.core.castOk(dim, a.tick)) { $('#hint').textContent = `${D.dimensions[dim].name} is resting ${secs(resting(dim))} (its own cooldown or its family's 2 s): pick another dimension or tick Ignore cooldowns.`; S.clicks = []; return; }
    const cmds = a.place(action);
    S.renderer.note(cmds, a.tick);
    for (const c of cmds) if (c.k === 'Log') S.log.unshift(c.s);
    S.clicks = [];
    $('#hint').textContent = RECIPES[T].text;
    renderReadouts();
  }

  // ------------------------------------------------------------------ pointer: placement and dragging
  function worldAt(ev) {
    const cv = S.renderer.canvas, r = cv.getBoundingClientRect();
    return S.renderer.toWorld((ev.clientX - r.left) * cv.width / r.width, (ev.clientY - r.top) * cv.height / r.height);
  }
  function wirePointer() {
    const cv = $('#arena');
    cv.addEventListener('pointerdown', ev => {
      if (ev.button !== 0) return;
      const [x, y] = worldAt(ev);
      const pickR = S.renderer.camera.w / cv.width * 14;
      const unit = nearest(S.arena.units.filter(u => u.f.hp > 0).map(u => ({ x: u.f.x, y: u.f.y, id: u.f.id })), x, y, pickR);
      S.down = { x, y, unit, px: ev.clientX, py: ev.clientY };
      cv.setPointerCapture(ev.pointerId);
    });
    cv.addEventListener('pointermove', ev => {
      if (S.down && S.down.unit && !S.drag && Math.hypot(ev.clientX - S.down.px, ev.clientY - S.down.py) > 6) S.drag = S.down.unit;
      if (!S.drag) return;
      const [x, y] = worldAt(ev);
      const u = S.arena.unit(S.drag.id);
      u.f.x = Math.max(0, Math.min(D.constants.map, x));
      u.f.y = Math.max(0, Math.min(D.constants.map, y));
    });
    cv.addEventListener('pointerup', () => {
      if (S.drag) { S.drag = null; S.down = null; return; }
      if (S.down) { onArenaClick(S.down.x, S.down.y); S.down = null; }
    });
    cv.addEventListener('dblclick', () => { if (RECIPES[S.tech].steps === 'nodes' && S.clicks.length) finish(); });
    cv.addEventListener('contextmenu', ev => ev.preventDefault());
    document.addEventListener('keydown', ev => {
      if (ev.key === 'Enter' && RECIPES[S.tech].steps === 'nodes' && S.clicks.length) finish();
      if (ev.key === 'Escape') { S.clicks = []; $('#hint').textContent = 'Placement cancelled.'; }
    });
  }

  // ------------------------------------------------------------------ rank duel
  const duel = { left: null, right: null, r1: null, r2: null, running: false, seen: [{}, {}] };
  function runDuel() {
    const seed = Math.max(1, Number($('#duelSeed').value) || 1);
    duel.left = V.teamfight(seed, Number($('#duelA').value));
    duel.right = V.teamfight(seed, Number($('#duelB').value));
    duel.seen = [{}, {}];
    $('#duelLeftHabits').innerHTML = '';
    $('#duelRightHabits').innerHTML = '';
    duel.r1.camera = null; duel.r2.camera = null; duel.r1.events = []; duel.r2.events = [];
    duel.running = true;
  }
  function duelStats(a) {
    const s = a.core.stats;
    const won = a.wiped();
    return `${D.ranks[a.core.rank]} · ${(a.tick / 60).toFixed(1)}s${won !== null ? won === 1 ? ' · WON' : ' · LOST' : ''}\ndamage ${s.damage} · punches ${s.punches} · swipe hits ${s.swipe_hits}\nshatters ${s.useful_shatters}/${s.shatters} · gauntlet breaks L${s.left_breaks} R${s.right_breaks} · broken rifts ${s.breaks}\nawakenings ${s.good_awakens}/${s.awakens} · kidnaps ${s.useful_kidnaps}/${s.kidnaps}`;
  }
  function duelFrame() {
    if (duel.running && S.visible !== false) {
      for (const [i, a, r] of [[0, duel.left, duel.r1], [1, duel.right, duel.r2]]) {
        for (let k = 0; k < 2 && a.wiped() === null && a.tick < 3600; k++) {
          a.keep_cmds = true; a.cmds = []; a.step(); r.note(a.cmds, a.tick);
          for (const [h, n] of Object.entries(a.core.stats.habits)) {
            if ((duel.seen[i][h] || 0) < n) {
              duel.seen[i][h] = n;
              const li = document.createElement('li');
              li.textContent = `${(a.tick / 60).toFixed(1)}s  ${h}`;
              (i === 0 ? $('#duelLeftHabits') : $('#duelRightHabits')).prepend(li);
            }
          }
        }
      }
      if ((duel.left.wiped() !== null || duel.left.tick >= 3600) && (duel.right.wiped() !== null || duel.right.tick >= 3600)) duel.running = false;
    }
    if (duel.left) {
      duel.r1.draw(duel.left, {}); duel.r2.draw(duel.right, {});
      $('#duelLeftStats').textContent = duelStats(duel.left);
      $('#duelRightStats').textContent = duelStats(duel.right);
    }
    requestAnimationFrame(duelFrame);
  }

  // ------------------------------------------------------------------ bonuses, dimensions, codex
  function renderTwists() {
    const fam = D.families;
    let html = '<table><thead><tr><th></th>' + fam.map(f => `<th style="color:${f.color}">${esc(f.name)}</th>`).join('') + '</tr></thead><tbody>';
    for (let a = 0; a < fam.length; a++) {
      html += `<tr><th style="color:${fam[a].color}">${esc(fam[a].name)}</th>`;
      for (let b = 0; b < fam.length; b++) {
        const t = D.twists[M.twistIndex(a, b)], k = D.twist_kinds[t.kind];
        html += `<td title="${esc(t.name)}: ${esc(k.text)} (strength ${t.mag})">${esc(k.name)}</td>`;
      }
      html += '</tr>';
    }
    $('#twists').innerHTML = html + '</tbody></table>';
  }
  function renderDimensions() {
    const q = $('#dimSearch').value.trim().toLowerCase(), fam = $('#dimFamily').value;
    const text = d => [d.name, d.flavour, D.prims[d.prim].text, D.prims[d.prim].id, showEff(d.punch.main), showEff(d.punch.rider), showEff(d.swipe.main), showEff(d.swipe.rider)].join(' ').toLowerCase();
    $('#dimensions').innerHTML = D.dimensions.filter(d => (fam === '' || String(d.family) === fam) && (!q || text(d).includes(q))).map(d =>
      `<div class="dim" style="border-left-color:${FAM[d.family]}"><strong>${d.id} ${esc(d.name)}</strong> · <span style="color:${FAM[d.family]}">${esc(D.families[d.family].name)}</span> · ${secs(d.cd)}${d.spectacular ? ' signature' : ''}<br>${esc(d.flavour)}${dimLine(d)}</div>`).join('') || '<p class="muted">No dimension matches.</p>';
  }
  function renderCodex() {
    let html = '<h3>Gauntlet moves</h3><table><thead><tr><th>Move</th><th>Orbs</th><th>Played well from</th><th>What happens</th></tr></thead><tbody>';
    for (const m of D.moves) html += `<tr><td>${esc(m.name)}</td><td>${m.orbs}</td><td>${MOVE_WELL[m.id]} · ${esc(D.ranks[MOVE_WELL[m.id]])}</td><td>${esc(m.text)}</td></tr>`;
    html += '</tbody></table><h3 style="margin-top:16px">Hexcore tech modules</h3><table><thead><tr><th>Module</th><th>Effect</th></tr></thead><tbody>';
    for (const m of D.modules) html += `<tr><td>${esc(m.name)}</td><td>${esc(m.text.replace(/\ba\b/, m.a).replace(/\bb\b/, m.b))}</td></tr>`;
    html += '</tbody></table><p class="muted">Hexcore research: +1% per pure orb, +3% per takedown, +4% per Shatter, −10% on death. Slots: ' + D.hexcore.slots.map(([a, n]) => `${a}% → ${n}`).join(', ') + '. At 100% she can Awaken.</p>';
    html += '<h3 style="margin-top:16px">Rift types</h3><table><thead><tr><th>Rift</th><th>Orbs</th><th>Life</th><th>Size</th><th>What the shape does</th></tr></thead><tbody>';
    for (const r of D.rifts) html += `<tr><td>${esc(r.name)}</td><td>${r.cost}</td><td>${(r.life / 60).toFixed(1)}s</td><td>r${r.radius}${r.length ? ' · ' + r.length + ' long' : ''}</td><td>${esc(r.text)}</td></tr>`;
    html += '</tbody></table><h3 style="margin-top:16px">Techniques</h3><table><thead><tr><th>Technique</th><th>Group</th><th>Rifts</th><th>Stress</th><th>Cooldown</th><th>Used well from</th><th>What happens</th></tr></thead><tbody>';
    for (const t of D.techniques) html += `<tr><td><button class="codex-try" data-try="${t.id}">${esc(t.name)}</button></td><td>${esc(t.group)}</td><td>${esc(t.rifts.map(r => D.rifts.find(x => x.id === r).name).join(', '))}</td><td>+${t.stress}</td><td>${t.cooldown ? t.cooldown / 60 + 's' : '—'}</td><td>${WELL[t.id]} · ${esc(D.ranks[WELL[t.id]])}</td><td>${esc(t.text)}</td></tr>`;
    $('#codex').innerHTML = html + '</tbody></table>';
  }
  function renderPalette() {
    $('#techniques').innerHTML = GROUPS.map(([g, title]) => `<div class="tech-group"><h4>${title}</h4><div>${D.techniques.filter(t => t.group === g).map(t => `<button data-tech="${t.id}" title="${esc(t.text)}">${esc(t.name)}</button>`).join('')}</div></div>`).join('');
  }

  // ------------------------------------------------------------------ mastery comparison
  async function compare() {
    if (S.compareRunning) return;
    S.compareRunning = true;
    $('#compare').disabled = true;
    const seeds = Array.from({ length: Math.max(1, Math.min(12, Number($('#cmpSeeds').value) || 4)) }, (_, i) => i + 1);
    const body = $('#comparison');
    body.innerHTML = '';
    for (let rank = 0; rank < 8; rank++) {
      const r = { damage: 0, punches: 0, sw: [0, 0], sh: [0, 0], lb: 0, rb: 0, aw: [0, 0], breaks: 0, k: [0, 0], wins: 0, len: 0 };
      for (const seed of seeds) {
        const a = V.teamfight(seed, rank);
        a.run(3600);
        const s = a.core.stats;
        r.damage += s.damage; r.punches += s.punches; r.sw[0] += s.swipe_hits; r.sw[1] += s.swipes; r.sh[0] += s.useful_shatters; r.sh[1] += s.shatters;
        r.lb += s.left_breaks; r.rb += s.right_breaks; r.aw[0] += s.good_awakens; r.aw[1] += s.awakens; r.breaks += s.breaks;
        r.k[0] += s.useful_kidnaps; r.k[1] += s.kidnaps;
        if (a.wiped() === 1) r.wins++;
        r.len += a.tick;
        await new Promise(res => setTimeout(res, 0));
      }
      const tr = document.createElement('tr');
      tr.innerHTML = `<td>${rank} · ${esc(D.ranks[rank])}</td><td>${r.damage}</td><td>${r.punches}</td><td>${r.sw[0]} in ${r.sw[1]}</td><td>${r.sh[0]}/${r.sh[1]}</td><td>L${r.lb} R${r.rb}</td><td>${r.aw[0]}/${r.aw[1]}</td><td>${r.breaks}</td><td>${r.k[0]}/${r.k[1]}</td><td>${r.wins}/${seeds.length}</td><td>${(r.len / seeds.length / 60).toFixed(1)}s</td>`;
      body.append(tr);
    }
    S.compareRunning = false;
    $('#compare').disabled = false;
  }

  // ------------------------------------------------------------------ native diagnostics
  function diagnostics(file) {
    file.text().then(text => {
      const lines = text.split(/\r?\n/).filter(Boolean).slice(-400);
      $('#nativeTimeline').innerHTML = lines.map(l => `<li>${esc(l)}</li>`).join('');
    });
  }

  // ------------------------------------------------------------------ parity self-check
  function parity() {
    const chip = $('#parity');
    try {
      const ok = M.chainBp([90000, 90000]) === 5347 && M.chainBp([60000, 60000, 60000]) === 5614 && M.hexSlots(85) === 5 && M.modulesOf(null).join() === '0,1,2,3,4,5' && M.openDelay(960000, 100) === 86;
      chip.textContent = ok ? 'RIFT EQUATIONS MATCH NATIVE' : 'RIFT EQUATIONS DIFFER FROM NATIVE';
      chip.className = 'chip ' + (ok ? 'ok' : 'bad');
    } catch (e) { chip.textContent = 'PARITY CHECK FAILED'; chip.className = 'chip bad'; }
  }

  function init() {
    options($('#rank'), rankRows(), 7);
    options($('#position'), [[0, '—'], ...Array.from({ length: 10 }, (_, i) => [i + 1, '#' + (i + 1) + (i === 0 ? ' — ' + D.top_name : '')])], 0);
    options($('#duelA'), rankRows(), 0);
    options($('#duelB'), rankRows(), 7);
    options($('#dimFamily'), [['', 'Every family'], ...D.families.map((f, i) => [i, f.name])], '');
    // the dimension selects, grouped by family
    const grouped = select => {
      select.innerHTML = D.families.map((f, k) => `<optgroup label="${esc(f.name)}">` + D.dimensions.map((d, i) => [d, i]).filter(([d]) => d.family === k)
        .map(([d, i]) => `<option value="${i}">${esc(d.id)} ${esc(d.name)}</option>`).join('') + '</optgroup>').join('');
    };
    grouped($('#riftDim')); $('#riftDim').value = '11';
    grouped($('#punchDim')); $('#punchDim').value = '10';
    $('#modules').innerHTML = Array.from({ length: 6 }, (_, i) => `<label>Slot ${i + 1}<select data-slot="${i}">${D.modules.map((m, k) => `<option value="${k}"${k === i ? ' selected' : ''}>${esc(m.name)}</option>`).join('')}</select></label>`).join('');
    S.renderer = new VR.Renderer($('#arena'));
    duel.r1 = new VR.Renderer($('#duelLeft'));
    duel.r2 = new VR.Renderer($('#duelRight'));
    renderPalette(); renderTwists(); renderDimensions(); renderCodex(); renderDimInfo(); parity();
    newArena();
    selectTech('chain');
    wirePointer();
    $('#techniques').addEventListener('click', ev => { const b = ev.target.closest('[data-tech]'); if (b) selectTech(b.dataset.tech); });
    $('#codex').addEventListener('click', ev => { const b = ev.target.closest('[data-try]'); if (b) { selectTech(b.dataset.try); $('#arena').scrollIntoView({ behavior: 'smooth', block: 'center' }); } });
    $('#play').onclick = () => { S.paused = !S.paused; $('#play').textContent = S.paused ? 'Play' : 'Pause'; };
    $('#stepTick').onclick = () => { S.paused = true; $('#play').textContent = 'Play'; for (let i = 0; i < 6; i++) tickOnce(); };
    $('#reset').onclick = newArena;
    for (const id of ['#rank', '#position', '#layout', '#seed']) $(id).addEventListener('change', () => { newArena(); document.dispatchEvent(new CustomEvent('vesper-rank-changed')); });
    $('#gauntlets').addEventListener('click', ev => { const b = ev.target.closest('[data-move]'); if (b) move(b.dataset.move); });
    $('#riftDim').onchange = renderDimInfo;
    $('#punchDim').onchange = () => { S.arena.core.armed = Number($('#punchDim').value); renderDimInfo(); };
    $('#modules').onchange = () => { S.arena.core.modules = currentModules(); };
    $('#hexSet').oninput = () => { const c = S.arena.core; if (!c.awakened(S.arena.tick)) c.completion = Number($('#hexSet').value); renderGauntlets(); };
    $('#planner').onchange = () => {
      S.arena.core.hand_placed = !$('#planner').checked;
      // a planning Vesper needs a moving fight: frozen units never walk into reach
      if ($('#planner').checked && $('#freeze').checked) { $('#freeze').checked = false; S.arena.frozen = false; }
    };
    $('#freeze').onchange = () => { S.arena.frozen = $('#freeze').checked; };
    $('#freeCasts').onchange = () => { S.arena.core.free_casts = $('#freeCasts').checked; refreshDimOptions(); };
    $('#dimSearch').oninput = renderDimensions;
    $('#dimFamily').onchange = renderDimensions;
    $('#duelRun').onclick = runDuel;
    $('#compare').onclick = compare;
    $('#diagnosticFile').onchange = ev => ev.target.files[0] && diagnostics(ev.target.files[0]);
    Promise.all([S.renderer.load(), duel.r1.load(), duel.r2.load()]).then(() => {
      document.dispatchEvent(new CustomEvent('vesper-sprites', { detail: S.renderer.sprites() }));
    });
    runDuel();
    requestAnimationFrame(frame);
    requestAnimationFrame(duelFrame);
  }
  root.VesperLab = {
    state: S,
    setVisible(v) { S.visible = v; },
    setRank(rank, top) {
      $('#rank').value = String(Math.max(0, Math.min(7, rank)));
      $('#position').value = String(rank >= 7 && top ? top : 0);
      newArena();
    },
    selectTech,
  };
  document.addEventListener('DOMContentLoaded', init);
})(typeof globalThis !== 'undefined' ? globalThis : this);
