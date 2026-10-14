/* Rift Lab: the practice arena (hand placement or the ranked planner), the rank duel, the atlas, the bonus grid,
 * the dimension and technique browsers, the mastery comparison and native diagnostics. All rules come from
 * VesperCombat (a 1:1 port of the native engine). */
(function (root) {
  'use strict';
  const D = root.RiftData, V = root.VesperCombat, M = root.VesperMath, VR = root.VesperRenderer;
  const $ = s => document.querySelector(s);
  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const FAM = VR.FAM;
  const GROUPS = [['offense', 'Offense'], ['engage', 'Engage'], ['defense', 'Defense'], ['team', 'Team & atlas']];
  // the rank from which the planner uses a technique well (its habit gate)
  const WELL = { chain: 3, crossfire: 3, bank: 3, rain: 3, volley: 3, implode: 3, domino: 3, overload: 7, step: 4, gate: 5, skydrop: 5, kidnap: 6, sinktrap: 6, swap: 5, fling: 5, hook: 5, phase: 4, curtain: 4, reflect: 2, rescue: 4, pocket: 4, lifeline: 4, escort: 5, rotation: 5, seedtrap: 6, sweep: 3, borrow: 5, stitch: 6 };
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
    overload: { steps: [], text: 'Spends up to 60 strain on the next chain shot.' },
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
    borrow: { steps: ['spot'], text: 'Needs a chain. Click a far cell: the chain exit borrows its dimension for 2 s.' },
    stitch: { steps: ['rift', 'rift'], text: 'Click two Tears within 40000 of each other.' },
  };

  const S = { arena: null, renderer: null, paused: false, tech: 'chain', clicks: [], drag: null, log: [], habits: {}, compareRunning: false };

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
    S.renderer.showAtlas = $('#atlasShow').checked;
    S.renderer.draw(S.arena, { paused: S.paused, dragging: S.drag && S.drag.id });
    drawPlacement();
    if (now - (S.lastPanel || 0) > 200) { S.lastPanel = now; renderReadouts(); }
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
    if (nodes.length) {
      lines.push(`total efficiency ${(M.chainBp(links) / 100).toFixed(1)}%`);
      if (nodes.length >= 2) {
        const f1 = D.dimensions[nodes[nodes.length - 2].dim].family, f2 = D.dimensions[nodes[nodes.length - 1].dim].family;
        lines.push(`exit-pair bonus: ${D.twists[M.twistIndex(f1, f2)].name}`);
      }
      lines.push(core.aiming ? 'aiming: every basic attack rides the chain' : 'not aiming (no target in range of the exit)');
    } else lines.push('no live chain: she attacks normally');
    const pending = core.rifts.filter(r => !r.opened);
    if (pending.length) lines.push(`${pending.length} rift(s) still opening`);
    $('#chainReadout').textContent = lines.join('\n');
    const s = core.stats;
    $('#counters').textContent = `strain ${core.strain} · charges ${core.charges.available()}/5\nrifts opened ${s.rifts_opened} · broken ${s.breaks} · breaches ${s.breaches}\npackets ${s.packets} · avg eff ${s.packets ? (s.eff_sum / s.packets / 100).toFixed(1) : '0'}%\ndamage ${s.damage}\nkidnaps ${s.useful_kidnaps}/${s.kidnaps} useful · gates ${s.engaged_gates}/${s.gates} engaged\ntraps ${s.trap_catches} catches / ${s.traps} set · implodes ${s.implodes}`;
    renderEvents();
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
    const cost = D.rifts.find(x => x.id === t.rifts[0]).cost;
    $('#techInfo').innerHTML = `<strong>${esc(t.name)}</strong> · ${esc(t.group)} · ${esc(RECIPES[id].text)}<br>${esc(t.text)}. Strain +${t.strain}${t.cooldown ? ` · per-target cooldown ${t.cooldown / 60}s` : ''} · first rift costs ${cost} charge${cost > 1 ? 's' : ''} · the planner uses it well from rank ${WELL[id]} (${esc(D.ranks[WELL[id]])}).`;
    $('#hint').textContent = RECIPES[id].text;
    if (!RECIPES[id].steps.length) finish();
  }
  function nearest(list, x, y, max) {
    let best = null, bd = max * max;
    for (const o of list) { const d = M.dist2(o.x, o.y, x, y); if (d <= bd) { bd = d; best = o; } }
    return best;
  }
  function onArenaClick(x, y) {
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
      case 'borrow': { const nodes = a.core.chainNodes(a.tick); if (!nodes.length) { $('#hint').textContent = 'Borrow needs a live chain.'; S.clicks = []; return; } uses = [nodes[nodes.length - 1].uid]; rifts = [Sp.at(R.tear, k[0].x, k[0].y)]; break; }
      case 'stitch': uses = [k[0].uid, k[1].uid]; break;
      default: break;
    }
    const action = { tech: techIndex(T), rifts, subject, uses, score: 0, habit: '' };
    const cost = rifts.reduce((s, r) => s + D.rifts[r.kind].cost, 0);
    if (a.core.charges.available() < cost) { $('#hint').textContent = `Not enough charges: needs ${cost}, ${a.core.charges.available()} ready.`; S.clicks = []; return; }
    if (a.core.openCount() + rifts.length > D.constants.max_rifts) { $('#hint').textContent = 'At most 5 rifts can be open.'; S.clicks = []; return; }
    if ((T === 'crossfire' || T === 'overload') && !a.core.chainNodes(a.tick).length) { $('#hint').textContent = 'Needs a live chain.'; S.clicks = []; return; }
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
    return `${D.ranks[a.core.rank]} · ${(a.tick / 60).toFixed(1)}s${won !== null ? won === 1 ? ' · WON' : ' · LOST' : ''}\ndamage ${s.damage} · eff ${s.packets ? (s.eff_sum / s.packets / 100).toFixed(0) : 0}% · breaches ${s.breaches} · broken ${s.breaks}\nkidnaps ${s.useful_kidnaps}/${s.kidnaps} · gates ${s.engaged_gates}/${s.gates} · traps ${s.trap_catches}/${s.traps}`;
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

  // ------------------------------------------------------------------ atlas, bonuses, dimensions, codex
  function renderAtlas() {
    const atlas = new M.Atlas(BigInt(Math.max(1, Number($('#seed').value) || 1)));
    const epoch = Number($('#drift').value);
    $('#driftLabel').textContent = `Seed ${$('#seed').value} after ${epoch} drift${epoch === 1 ? '' : 's'} (${epoch * 30} s): shift ${atlas.shift(epoch).join(', ')}. Each of the 100 dimensions sits in exactly one cell.`;
    let html = '';
    for (let cy = 0; cy < 10; cy++) for (let cx = 0; cx < 10; cx++) {
      const dim = atlas.dimAtEpoch(cx * 96000 + 1, cy * 96000 + 1, epoch), d = D.dimensions[dim], p = D.prims[d.prim];
      html += `<div class="cell" style="background:${FAM[d.family]}22;border-color:${FAM[d.family]}55" title="${esc(d.name)} (${esc(D.families[d.family].name)}): ${esc(p.text)} · a=${d.a} b=${d.b} c=${d.c}"><b>${d.id}</b>${esc(d.name)}</div>`;
    }
    $('#atlas').innerHTML = html;
  }
  function renderTwists() {
    const fam = D.families;
    let html = '<table><thead><tr><th></th>' + fam.map(f => `<th style="color:${f.color}">${esc(f.name)}</th>`).join('') + '</tr></thead><tbody>';
    for (let a = 0; a < 10; a++) {
      html += `<tr><th style="color:${fam[a].color}">${esc(fam[a].name)}</th>`;
      for (let b = 0; b < 10; b++) {
        const t = D.twists[M.twistIndex(a, b)], k = D.twist_kinds[t.kind];
        html += `<td title="${esc(t.name)}: ${esc(k.text)} (strength ${t.mag})">${esc(k.name)}</td>`;
      }
      html += '</tr>';
    }
    $('#twists').innerHTML = html + '</tbody></table>';
  }
  function renderDimensions() {
    const q = $('#dimSearch').value.trim().toLowerCase(), fam = $('#dimFamily').value;
    $('#dimensions').innerHTML = D.dimensions.filter(d => (fam === '' || String(d.family) === fam) && (!q || (d.name + ' ' + d.flavour + ' ' + D.prims[d.prim].text + ' ' + D.prims[d.prim].id).toLowerCase().includes(q))).map(d => {
      const p = D.prims[d.prim];
      return `<div class="dim" style="border-left-color:${FAM[d.family]}"><strong>${d.id} ${esc(d.name)}</strong> · <span style="color:${FAM[d.family]}">${esc(D.families[d.family].name)}</span><br>${esc(d.flavour)}<small>${esc(p.id)} (${p.side === 'E' ? 'enemies' : p.side === 'A' ? 'allies' : p.side === 'S' ? 'Vesper' : p.side === 'P' ? 'shots through it' : 'special'}): ${esc(p.text)} · a=${d.a} b=${d.b} c=${d.c}</small></div>`;
    }).join('') || '<p class="muted">No dimension matches.</p>';
  }
  function renderCodex() {
    let html = '<h3>Rift types</h3><table><thead><tr><th>Rift</th><th>Charges</th><th>Life</th><th>Size</th><th>What the shape does</th></tr></thead><tbody>';
    for (const r of D.rifts) html += `<tr><td>${esc(r.name)}</td><td>${r.cost}</td><td>${(r.life / 60).toFixed(1)}s</td><td>r${r.radius}${r.length ? ' · ' + r.length + ' long' : ''}</td><td>${esc(r.text)}</td></tr>`;
    html += '</tbody></table><h3 style="margin-top:16px">Techniques</h3><table><thead><tr><th>Technique</th><th>Group</th><th>Rifts</th><th>Strain</th><th>Cooldown</th><th>Used well from</th><th>What happens</th></tr></thead><tbody>';
    for (const t of D.techniques) html += `<tr><td><button class="codex-try" data-try="${t.id}">${esc(t.name)}</button></td><td>${esc(t.group)}</td><td>${esc(t.rifts.map(r => D.rifts.find(x => x.id === r).name).join(', '))}</td><td>+${t.strain}</td><td>${t.cooldown ? t.cooldown / 60 + 's' : '—'}</td><td>${WELL[t.id]} · ${esc(D.ranks[WELL[t.id]])}</td><td>${esc(t.text)}</td></tr>`;
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
      const r = { damage: 0, eff: 0, breaches: 0, breaks: 0, k: [0, 0], g: [0, 0], tr: [0, 0], wins: 0, len: 0 };
      for (const seed of seeds) {
        const a = V.teamfight(seed, rank);
        a.run(3600);
        const s = a.core.stats;
        r.damage += s.damage; r.eff += s.packets ? s.eff_sum / s.packets : 0; r.breaches += s.breaches; r.breaks += s.breaks;
        r.k[0] += s.useful_kidnaps; r.k[1] += s.kidnaps; r.g[0] += s.engaged_gates; r.g[1] += s.gates; r.tr[0] += s.trap_catches; r.tr[1] += s.traps;
        if (a.wiped() === 1) r.wins++;
        r.len += a.tick;
        await new Promise(res => setTimeout(res, 0));
      }
      const tr = document.createElement('tr');
      tr.innerHTML = `<td>${rank} · ${esc(D.ranks[rank])}</td><td>${r.damage}</td><td>${(r.eff / seeds.length / 100).toFixed(1)}%</td><td>${r.breaches}</td><td>${r.breaks}</td><td>${r.k[0]}/${r.k[1]}</td><td>${r.g[0]}/${r.g[1]}</td><td>${r.tr[0]}/${r.tr[1]}</td><td>${r.wins}/${seeds.length}</td><td>${(r.len / seeds.length / 60).toFixed(1)}s</td>`;
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
      const ok = M.chainBp([90000, 90000]) === 5347 && M.chainBp([60000, 60000, 60000]) === 5614 && new M.Atlas(42n).base.length === 100 && M.openDelay(960000, 100) === 86;
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
    S.renderer = new VR.Renderer($('#arena'));
    duel.r1 = new VR.Renderer($('#duelLeft'));
    duel.r2 = new VR.Renderer($('#duelRight'));
    renderPalette(); renderAtlas(); renderTwists(); renderDimensions(); renderCodex(); parity();
    newArena();
    selectTech('chain');
    wirePointer();
    $('#techniques').addEventListener('click', ev => { const b = ev.target.closest('[data-tech]'); if (b) selectTech(b.dataset.tech); });
    $('#codex').addEventListener('click', ev => { const b = ev.target.closest('[data-try]'); if (b) { selectTech(b.dataset.try); $('#arena').scrollIntoView({ behavior: 'smooth', block: 'center' }); } });
    $('#play').onclick = () => { S.paused = !S.paused; $('#play').textContent = S.paused ? 'Play' : 'Pause'; };
    $('#stepTick').onclick = () => { S.paused = true; $('#play').textContent = 'Play'; for (let i = 0; i < 6; i++) tickOnce(); };
    $('#reset').onclick = newArena;
    for (const id of ['#rank', '#position', '#layout', '#seed']) $(id).addEventListener('change', () => { newArena(); renderAtlas(); document.dispatchEvent(new CustomEvent('vesper-rank-changed')); });
    $('#planner').onchange = () => { S.arena.core.hand_placed = !$('#planner').checked; };
    $('#freeze').onchange = () => { S.arena.frozen = $('#freeze').checked; };
    $('#drift').oninput = renderAtlas;
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
      newArena(); renderAtlas();
    },
    selectTech,
  };
  document.addEventListener('DOMContentLoaded', init);
})(typeof globalThis !== 'undefined' ? globalThis : this);
