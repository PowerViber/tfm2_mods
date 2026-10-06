/* Isliid Engraving Lab: sword staging, parallel drawing and predictive mastery.
 * It previews the native rules but does not run the match simulation or edit a career.
 */
(function () {
  'use strict';

  const W = 820, H = 480, HERO = { x: 94, y: 245 }, STORE = 'tfm2.isliidlab.v1';
  const SWORDS = [
    ['Skylight', '#f6edaa'], ['Terra', '#b79769'], ['Darkbringer', '#a479d1'],
    ['Gale', '#8de8d9'], ['Blood', '#e87283'], ['Rift', '#75a7fa'], ['Emperor', '#ffd166'],
  ];
  // The live swords have different speeds, while mastery changes planning.
  const SWORD_SPEED = [520, 360, 455, 780, 488, 585, 423];
  const LOOK_AHEAD = [0, .5, 1, 1.5, 2, 3, 4, 5];
  const RANKS = [
    { name: 'Bearer', points: 0, decision: 1500, error: 22, candidates: 3 },
    { name: 'Squire', points: 5, decision: 1250, error: 17, candidates: 5 },
    { name: 'Engraver', points: 15, decision: 1000, error: 13, candidates: 8 },
    { name: 'Tactician', points: 30, decision: 800, error: 10, candidates: 12 },
    { name: 'Swordmaster', points: 60, decision: 630, error: 7, candidates: 16 },
    { name: 'Regent', points: 100, decision: 500, error: 5, candidates: 21 },
    { name: 'Sovereign', points: 150, decision: 370, error: 3, candidates: 26 },
    { name: 'Imperial', points: 'Top 10, 300+', decision: 250, error: 1.5, candidates: 30 },
  ];
  const radial = (n, skip = 1) => ({ nodes: Array.from({ length:n },(_,i)=>[Math.cos(-Math.PI/2+i*2*Math.PI/n),Math.sin(-Math.PI/2+i*2*Math.PI/n)]),
    edges: Array.from({ length:n },(_,i)=>[i,(i+skip)%n]) });
  const PATTERNS = {
    line: { name: 'Severing Line', effect: 'Crossing damage; length trades damage for coverage', nodes: [[-1, 0], [1, 0]], edges: [[0, 1]] },
    triangle: { name: 'Execution Seal', effect: 'Physical resistance reduction inside', nodes: [[0, -1], [-0.9, 0.72], [0.9, 0.72]], edges: [[0, 1], [1, 2], [2, 0]] },
    square: { name: 'Imperial Fortress', effect: 'Damage resistance for allies inside', nodes: [[-0.82, -0.82], [0.82, -0.82], [0.82, 0.82], [-0.82, 0.82]], edges: [[0, 1], [1, 2], [2, 3], [3, 0]] },
    pentagon: { name: "Emperor's Blessing", effect: 'Support zone; resource gain becomes cooldown recovery in TFM2', nodes: [[0, -1], [0.95, -0.31], [0.59, 0.81], [-0.59, 0.81], [-0.95, -0.31]], edges: [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0]] },
    domain: { name: "Emperor's Domain", effect: 'Seven-sword territory; strongest commitment', nodes: Array.from({ length: 7 }, (_, i) => [Math.cos(-Math.PI / 2 + i * 2 * Math.PI / 7), Math.sin(-Math.PI / 2 + i * 2 * Math.PI / 7)]), edges: Array.from({ length: 7 }, (_, i) => [i, (i + 1) % 7]) },
    tripwire: { name:'Tripwire', effect:'Precise line snaps swords toward a crossing enemy', nodes:[[-1,0],[1,0]], edges:[[0,1]] },
    funnel: { name:'Funnel', effect:'Pull toward the point', nodes:[[-1,-.8],[0,1],[1,-.8]], edges:[[0,1],[1,2]] },
    expulsion: { name:'Expulsion', effect:'Push away from the point', nodes:[[-1,.8],[0,-1],[1,.8]], edges:[[0,1],[1,2]] },
    corner: { name:'Corner Guard', effect:'Slow at a corner; crossing both roots', nodes:[[-1,-1],[-1,1],[1,1]], edges:[[0,1],[1,2]] },
    road: { name:'Piercing Road', effect:'Attack range along the marked road', nodes:[[-1,0],[0,0],[1,0]], edges:[[0,1],[1,2]] },
    suppression: { name:'Suppression Seal', effect:'Reduce enemy damage inside', nodes:[[0,1],[-.9,-.72],[.9,-.72]], edges:[[0,1],[1,2],[2,0]] },
    ambush: { name:'Ambush Seal', effect:'Gain speed toward enemies entering', nodes:[[-1,-1],[-1,1],[1,1]], edges:[[0,1],[1,2],[2,0]] },
    rectangle: { name:'Marching Ground', effect:'Speed along the long axis', nodes:[[-1,-.5],[1,-.5],[1,.5],[-1,.5]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    diamond: { name:'Judgment Field', effect:'Concentrated damage at center', nodes:[[0,-1],[1,0],[0,1],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    siege: { name:'Siege Ground', effect:'Attack strength toward narrow side', nodes:[[-.5,-1],[.5,-1],[1,1],[-1,1]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    pursuit: { name:'Pursuit Seal', effect:'Slow enemies retreating from tip', nodes:[[0,-1],[.6,0],[0,1],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    rupture: { name:'Rupture', effect:'Burst at crossed lines', nodes:[[-1,-1],[1,1],[1,-1],[-1,1]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    intersection: { name:'Divine Intersection', effect:'Periodic center damage', nodes:[[-1,0],[0,-1],[1,0],[0,1]], edges:[[0,2],[1,3]] },
    execution: { name:'Execution Point', effect:'Precise center burst', nodes:[[-1,-1],[1,1],[1,-1],[-1,1]], edges:[[0,1],[2,3]] },
    roadblock: { name:'Roadblock', effect:'Push approaching enemies sideways', nodes:[[-1,0],[0,-1],[1,0],[0,1]], edges:[[0,2],[1,3]] },
    celestial: { name:'Celestial Judgment', effect:'Repeated hits at star crossings', ...radial(5,2) },
    sanctuary: { name:'Sanctuary', effect:'Heal and shield allies', nodes:[[-1,1],[-1,0],[0,-1],[1,0],[1,1]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    charge: { name:'Imperial Charge', effect:'Speed allies and push enemies forward', nodes:[[-1,-.4],[0,-.4],[0,-1],[1,0],[0,1]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    time: { name:'Time of Judgment', effect:'Growing slow at center', nodes:[[-1,-1],[1,1],[1,-1],[-1,1],[0,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    absolute: { name:'Absolute Territory', effect:'Balanced attack, guard and cooldown buffs', ...radial(6) },
    grand: { name:'Grand Execution', effect:'Converging six-sword damage', ...radial(6,2) },
    maelstrom: { name:'Imperial Maelstrom', effect:'Pull enemies inward', nodes:[[0,-.2],[.35,-.4],[.7,0],[.4,.6],[-.3,.8],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,0]] },
    prison: { name:'Imperial Prison', effect:'Crossing boundary stuns', ...radial(6) },
    heavenfall: { name:'Heavenfall', effect:'Seven-sword convergence burst', ...radial(7,2) },
    authority: { name:"King's Authority", effect:'Strongest team support', nodes:[[-1,0],[-.7,-1],[-.4,0],[0,-1],[.4,0],[.7,-1],[1,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,6],[6,0]] },
  };
  const state = { root: null, canvas: null, ctx: null, sprite: null, weapons: null, badges: null, auras: null, fields: null, flags: null, art: null, trails: null, rank: 3, imperialLevel: 1, pattern: 'triangle', scale: 1, scenario: 'self', active: true, empowerment: 0, marks: [], previewState: 'orbit', auraSide: 'ally', inspectFrame: -1, cancelUntil: 0,
    selected: 0, emperor: false, floatPreview: true, playback: 1, anchors: Array(7).fill(null), slots: [], drag: null, flights: [],
    started: 0, placements: 0, corrections: 0, recalls: 0, result: null, auto: null, compare: null, raf: 0 };

  const $ = s => state.root && state.root.querySelector(s);
  const esc = s => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
  const distance = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  const fmt = ms => (ms / 1000).toFixed(2) + ' s';
  const rand = seed => () => { seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0; return seed / 4294967296; };
  const noise = random => (random() + random() + random() + random() - 2) * 1.732;
  const weaponTier = rank => rank;
  const tierName = RANKS.map(r => r.name);
  const IMPERIAL_LEVELS = 10;
  // Seven numberless designs and ten Imperial subdivisions, eight frames each.
  const badgeFamily = (rank, imperialLevel) => rank < 7 ? rank : 7 + clamp(imperialLevel, 1, IMPERIAL_LEVELS) - 1;
  const frameAt = now => state.inspectFrame < 0 ? Math.floor(now / 100) % 8 : state.inspectFrame;
  function paintArt(c, sheet, family, tag, now, x, y, w, h) {
    const frames = state.art?.[family]?.[tag]?.frames;
    const frame = frames?.[frameAt(now) % frames.length]?.data;
    if (!frame || !sheet?.complete || !sheet.naturalWidth) return false;
    c.imageSmoothingEnabled = false;
    c.drawImage(sheet, frame.x, frame.y, frame.w, frame.h, x, y, w, h);
    return true;
  }

  function paintMark(c, m, now) {
    const dx=m.to.x-m.from.x, dy=m.to.y-m.from.y, length=Math.hypot(dx,dy);
    if (length < 1) return;
    if (!state.trails || !state.trails.complete || !state.trails.naturalWidth) {
      c.strokeStyle=SWORDS[m.sword][1]; c.lineWidth=3; c.beginPath();
      c.moveTo(m.from.x,m.from.y); c.lineTo(m.to.x,m.to.y); c.stroke(); return;
    }
    const angle=Math.round(((Math.atan2(dy,dx)%Math.PI+Math.PI)%Math.PI)*16/Math.PI)%16;
    const phase=Math.floor(now/140)%2, cell=(m.sword*16+angle)*2+phase;
    const count=clamp(Math.ceil(length/15),1,80);
    c.imageSmoothingEnabled=false;
    for (let k=0;k<=count;k++) {
      const t=k/count;
      c.drawImage(state.trails,(cell%16)*32,Math.floor(cell/16)*32,32,32,
        m.from.x+dx*t-16,m.from.y+dy*t-16,32,32);
    }
  }

  function formationCenter(rank = state.rank) {
    const horizon = LOOK_AHEAD[rank], base = { x: W * .56, y: H * .51 };
    // These are visible motion cues in the lab's scenario, not hidden enemy data.
    const velocity = state.scenario === 'objective' ? [0, 0] : state.scenario === 'ally' ? [-14, -9] : [18, 4];
    return { x: base.x + velocity[0] * horizon, y: base.y + velocity[1] * horizon };
  }
  function targets(pattern = state.pattern, scale = state.scale, rank = state.rank) {
    const radius = 104 * scale, { x: cx, y: cy } = formationCenter(rank);
    return PATTERNS[pattern].nodes.map(([x, y]) => ({ x: cx + x * radius, y: cy + y * radius }));
  }
  function legs(pattern = state.pattern, scale = state.scale, rank = state.rank) {
    const nodes = targets(pattern, scale, rank), result = PATTERNS[pattern].edges.map(([a, b]) => ({ from: nodes[a], to: nodes[b] }));
    // Open shapes still need one independently activated sword per leg.
    while (result.length < nodes.length) {
      let longest = 0;
      for (let i = 1; i < result.length; i++) if (distance(result[i].from, result[i].to) > distance(result[longest].from, result[longest].to)) longest = i;
      const { from, to } = result[longest], middle = { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 };
      result.splice(longest, 1, { from, to: middle }, { from: middle, to });
    }
    return result;
  }
  function slotsFor(count, emperor) {
    const a = Array.from({ length: count }, (_, i) => i);
    if (emperor && count < 7) a[count - 1] = 6;
    return a;
  }
  function score(marks, slots, ideal = legs(), radius = 104 * state.scale) {
    const present = slots.map((s, i) => s == null ? null : marks.filter(m => m.sword === s).map(m => ({ m, error: Math.hypot(distance(m.from, ideal[i].from), distance(m.to, ideal[i].to)) / 2 })).sort((a, b) => a.error - b.error)[0]).filter(Boolean);
    const integrity = Math.round(100 * present.length / slots.length);
    if (!present.length) return { precision: 0, integrity, grade: 'Empty', effectiveness: 0, emperor: false };
    const rmse = Math.sqrt(present.reduce((a, p) => a + p.error ** 2, 0) / present.length);
    const precision = Math.round(clamp(100 * (1 - rmse / (1.5 * radius)), 0, 100));
    const valid = present.length === slots.length && precision >= 50;
    const grade = !valid ? 'Invalid' : precision >= 99 ? 'Imperial' : precision >= 95 ? 'Perfect' : precision >= 85 ? 'Refined' : precision >= 70 ? 'Stable' : 'Crude';
    const base = !valid ? 0 : precision >= 99 ? 1.2 : precision >= 95 ? 1.1 : precision >= 85 ? 1 : precision >= 70 ? 0.85 : 0.7;
    const emperor = valid && slots.includes(6), committed = new Set(slots).size;
    const synergy = 1 + 0.25 * Math.max(0, committed - 1);
    const length = ideal.reduce((sum, leg) => sum + distance(leg.from, leg.to), 0);
    const concentration = clamp(1.25 - length / 1200, .55, 1.25);
    return { precision, integrity, grade, effectiveness: Math.round(base * synergy * concentration * 100 / Math.max(1, committed)), emperor, length, committed };
  }
  function simulate(rankIndex, pattern = state.pattern, scale = state.scale, seed = 1, emperor = state.emperor) {
    const rank = RANKS[rankIndex], ideal = legs(pattern, scale, rankIndex), slots = slotsFor(ideal.length, emperor);
    const anchors = Array(7).fill(null), events = [], marks = [], random = rand(seed >>> 0);
    let elapsed = 0, corrections = 0;
    const launch = state.scenario === 'ally' ? { x: W * .72, y: H * .78 } : state.scenario === 'objective' ? { x: W * .83, y: H * .28 } : HERO;
    ideal.forEach((leg, i) => {
      const sword = slots[i];
      const from = leg.from;
      let to = null, best = Infinity;
      for (let option = 0; option < rank.candidates; option++) {
        const candidate = { x: clamp(leg.to.x + noise(random) * rank.error, 15, W - 15), y: clamp(leg.to.y + noise(random) * rank.error, 15, H - 15) };
        const error = distance(candidate, leg.to);
        if (error < best) { best = error; to = candidate; }
      }
      const stageEnd = rank.decision + distance(launch, from) / SWORD_SPEED[sword] * 1000;
      const end = stageEnd + distance(from, to) / SWORD_SPEED[sword] * 1000;
      events.push({ sword, from: launch, to: from, start: rank.decision, end: stageEnd, kind: 'stage' });
      events.push({ sword, from, to, start: stageEnd, end, kind: 'draw' });
      elapsed = Math.max(elapsed, end);
      anchors[sword] = to;
      marks.push({ sword, from, to });
    });
    // Better ranks can recognize a poor engraving and move its worst vertex before manifesting.
    const limit = rankIndex >= 5 ? 2 : rankIndex >= 3 ? 1 : 0;
    for (let k = 0; k < limit && score(marks, slots, ideal, 104 * scale).precision < 95; k++) {
      let worst = 0;
      for (let i = 1; i < slots.length; i++) if (distance(anchors[slots[i]], ideal[i].to) > distance(anchors[slots[worst]], ideal[worst].to)) worst = i;
      const sword = slots[worst], from = ideal[worst].from;
      const to = { x: ideal[worst].to.x + noise(random) * rank.error * 0.25, y: ideal[worst].to.y + noise(random) * rank.error * 0.25 };
      const start = elapsed + rank.decision;
      const staged = start + distance(anchors[sword], from) / SWORD_SPEED[sword] * 1000;
      events.push({ sword, from: anchors[sword], to: from, start, end: staged, kind: 'stage' });
      elapsed = staged + distance(from, to) / SWORD_SPEED[sword] * 1000;
      events.push({ sword, from, to, start: staged, end: elapsed, kind: 'draw' });
      anchors[sword] = to; marks.push({ sword, from, to }); corrections++;
    }
    return { rank: rankIndex, pattern, scale, seed, slots, anchors, events, ms: elapsed, corrections,
      quality: score(marks, slots, ideal, 104 * scale), center: formationCenter(rankIndex) };
  }
  function compare(runs = 50) {
    const rows = RANKS.map((_, rank) => {
      let ms = 0, precision = 0, corrections = 0, valid = 0, perfect = 0;
      for (let i = 0; i < runs; i++) {
        const s = simulate(rank, state.pattern, state.scale, 70217 + i * 73, state.emperor);
        ms += s.ms; precision += s.quality.precision; corrections += s.corrections;
        if (s.quality.effectiveness) valid++; if (s.quality.precision >= 95) perfect++;
      }
      return { rank, ms: ms / runs, precision: precision / runs, corrections: corrections / runs, valid: valid / runs, perfect: perfect / runs };
    });
    state.compare = { runs, rows };
    renderResults();
    return rows;
  }
  function reset() {
    state.anchors = Array(7).fill(null); state.slots = Array(targets().length).fill(null);
    state.auto = null; state.drag = null; state.flights = []; state.started = 0; state.result = null;
    state.marks = []; state.empowerment = 0;
    state.placements = 0; state.corrections = 0; state.recalls = 0;
    renderPalette(); renderResults();
  }
  function evaluate() {
    if (state.auto) return;
    let quality = score(state.marks.filter(m => m.until > performance.now()), state.slots);
    if (!quality.effectiveness && state.marks.length) {
      const last=state.marks[state.marks.length-1], length=distance(last.from,last.to);
      quality={ precision:100, integrity:100, grade:`Solo ${SWORDS[last.sword][0]}`, effectiveness:Math.round(100*clamp(1.25-length/1200,.55,1.25)),
        emperor:last.sword===6, length, committed:1 };
    }
    state.result = { quality, ms: state.started ? performance.now() - state.started : 0,
      placements: state.placements, corrections: state.corrections, recalls: state.recalls };
    renderResults();
  }
  function autoDraw() {
    reset();
    const sim = simulate(state.rank, state.pattern, state.scale, Math.floor(Math.random() * 1e9), state.emperor);
    state.auto = { sim, started: performance.now(), paletteKey: '' };
    state.slots = sim.slots;
    renderResults();
  }
  function renderPalette() {
    const box = $('#ilPalette'); if (!box) return;
    box.innerHTML = SWORDS.map(([name, color], i) => { const status = swordStatus(i);
      return `<button type="button" class="il-sword${state.selected === i ? ' selected' : ''}${status === 'anchor' ? ' planted' : ''}" data-il-sword="${i}" style="--sword:${color}" title="${esc(name)} - ${status}"><span>${i + 1}</span>${esc(name)}<small>${status}</small></button>`; }).join('');
  }
  function swordStatus(i) {
    if (state.anchors[i]) return 'anchor';
    if (state.flights.some(f => f.sword === i)) return 'flying';
    if (state.auto) {
      const elapsed = (performance.now() - state.auto.started) * state.playback;
      let status = 'ready';
      state.auto.sim.events.forEach(e => { if (e.sword === i && elapsed >= e.start) status = elapsed >= e.end ? 'anchor' : 'flying'; });
      return status;
    }
    return 'ready';
  }
  function renderResults() {
    const box = $('#ilResult'); if (!box) return;
    const q = state.result && state.result.quality;
    box.innerHTML = q ? `<b>${q.grade}</b> · precision <b>${q.precision}%</b> · integrity <b>${q.integrity}%</b> · shared effect <b>${q.effectiveness}% per sword</b> · path <b>${Math.round(q.length || 0)} px</b> · ${q.committed || 0} committed${q.emperor ? ' (Emperor sword included)' : ''} · R charges ${state.empowerment}<br>
      ${state.result.ms ? fmt(state.result.ms) : '0 s'} to manifest · ${state.result.placements} placements · ${state.result.corrections} redraws · ${state.result.recalls} recalls` :
      state.auto ? `${RANKS[state.rank].name} is staging and engraving with ${state.slots.length} swords in parallel…` : `Drag any sword to stage or engrave. R empowers the next three completions. Charges: ${state.empowerment}.`;
    const tb = $('#ilTable'); if (!tb) return;
    tb.innerHTML = state.compare ? `<table class="st-table"><tr><th>Mastery</th><th>Points</th><th>Forecast</th><th>Options read</th><th>Mean time</th><th>Precision</th><th>Valid</th><th>Perfect</th></tr>${state.compare.rows.map(r => `<tr${r.rank === state.rank ? ' class="il-active"' : ''}><td>${RANKS[r.rank].name}</td><td>${RANKS[r.rank].points}</td><td>${LOOK_AHEAD[r.rank]}s</td><td>${RANKS[r.rank].candidates}</td><td>${fmt(r.ms)}</td><td>${r.precision.toFixed(0)}%</td><td>${(r.valid * 100).toFixed(0)}%</td><td>${(r.perfect * 100).toFixed(0)}%</td></tr>`).join('')}</table><p class="muted">${state.compare.runs} seeded routes per rank; swords travel at the same type-specific speeds in every rank. The lab simulates their parallel drawing and planning delay.</p>` : '';
  }
  function paintSword(c, p, i, size = 1, now = 0, planted = true, angle = 0, swordState = null) {
    const sh = state.weapons;
    if (planted) {
      c.fillStyle = 'rgba(0,0,0,.32)'; c.beginPath(); c.ellipse(p.x, p.y + 2, 9, 3, 0, 0, Math.PI * 2); c.fill();
    }
    if (sh && sh.complete && sh.naturalWidth) {
      c.imageSmoothingEnabled = false;
      c.save(); c.translate(p.x, p.y); c.rotate(angle);
      paintArt(c, sh, 'swords', `${SWORDS[i][0].toLowerCase()}_rank${state.rank}_${swordState || (planted ? 'planted' : state.previewState)}`,
        now, -16 * size, -(planted ? 58 : 32) * size, 32 * size, 64 * size);
      c.restore();
    } else {
      c.strokeStyle = SWORDS[i][1]; c.lineWidth = 4; c.beginPath();
      c.moveTo(p.x, p.y - 45 * size); c.lineTo(p.x, p.y); c.stroke();
    }
  }
  function paintThrowTrail(c, f, p, progress, now) {
    const dx = f.to.x - f.from.x, dy = f.to.y - f.from.y, len = Math.max(1, Math.hypot(dx, dy));
    const ux = dx / len, uy = dy / len, pulse = 0.65 + Math.sin(now / 70) * 0.2;
    c.save(); c.globalAlpha = (1 - progress) * pulse;
    c.strokeStyle = SWORDS[f.sword][1]; c.lineWidth = 4;
    c.beginPath(); c.moveTo(p.x - ux * 8, p.y - uy * 8); c.lineTo(p.x - ux * 38, p.y - uy * 38); c.stroke();
    c.strokeStyle = '#c7ffff'; c.lineWidth = 1;
    c.beginPath(); c.moveTo(p.x - ux * 12 - uy * 3, p.y - uy * 12 + ux * 3);
    c.lineTo(p.x - ux * 31 - uy * 1, p.y - uy * 31 + ux * 1); c.stroke();
    c.fillStyle = SWORDS[f.sword][1];
    for (let i = 1; i <= 3; i++) { const q = 8 + i * 9; c.globalAlpha = (1 - progress) * (0.55 - i * 0.1); c.fillRect(p.x - ux * q - 1, p.y - uy * q - 1, 3, 3); }
    c.restore();
  }
  function paintRankBadge(c, now, drift, bob) {
    if (!state.badges || !state.badges.complete || !state.badges.naturalWidth) return;
    c.imageSmoothingEnabled = false;
    const tag = state.rank < 7 ? `rank${state.rank}` : `imperial${state.imperialLevel}`;
    // Match the 48x96 game buff sheet's origin beside the 48x56 body sprite.
    paintArt(c, state.badges, 'badges', tag, now, 46 + drift, 133 + bob, 96, 192);
  }
  function paintAura(c, at, sword, side, now) {
    paintArt(c, state.auras, 'auras', `aura_${sword}_rank${state.rank}_${side}`,
      now, at.x - 64, at.y - 91, 128, 128);
  }
  function paintAuraBase(c, at, side, now) {
    paintArt(c, state.auras, 'auras', `aura_base_rank${state.rank}_${side}`,
      now, at.x - 64, at.y - 91, 128, 128);
  }
  function paintSwordField(c, at, sword, now) {
    paintArt(c, state.fields, 'fields', `aura_field_${sword}_rank${state.rank}`,
      now, at.x - 96, at.y - 96, 192, 192);
  }
  function paintFlag(c, now, phase) {
    const center = formationCenter();
    const entries = Object.keys(state.art?.flags || {});
    const index = Object.keys(PATTERNS).indexOf(state.pattern);
    const tag = entries.includes(`flag_pattern_${index}_${phase}`) ? `flag_pattern_${index}_${phase}` : `flag_solo_${state.selected}_${phase}`;
    paintArt(c, state.flags, 'flags', tag, now, center.x - 60, center.y - 76, 120, 48);
  }
  function draw(now) {
    const c = state.ctx; if (!c) return;
    const cv = state.canvas; c.clearRect(0, 0, W, H);
    c.fillStyle = '#1d2635'; c.fillRect(0, 0, W, H);
    c.strokeStyle = 'rgba(235,242,255,.055)'; c.lineWidth = 1;
    for (let x = 0; x < W; x += 40) { c.beginPath(); c.moveTo(x, 0); c.lineTo(x, H); c.stroke(); }
    for (let y = 0; y < H; y += 40) { c.beginPath(); c.moveTo(0, y); c.lineTo(W, y); c.stroke(); }
    const holder = state.scenario === 'ally' ? {x:W*.72,y:H*.78} : HERO;
    const foe = {x:W*.56,y:H*.51};
    if (state.scenario !== 'self') {
      const p=state.scenario==='ally'?{x:W*.72,y:H*.78}:{x:W*.83,y:H*.28};
      c.fillStyle=state.scenario==='ally'?'#79dbaf':'#f5bf72'; c.beginPath(); c.arc(p.x,p.y,12,0,Math.PI*2); c.fill();
      c.font='12px system-ui'; c.fillText(state.scenario==='ally'?'ALLY':'OBJECTIVE',p.x+16,p.y+4);
    }
    const horizon=LOOK_AHEAD[state.rank], forecast=formationCenter(), current={x:W*.56,y:H*.51};
    if (horizon && distance(current,forecast)>0) {
      c.strokeStyle='rgba(255,130,130,.55)'; c.setLineDash([4,4]); c.beginPath();
      c.moveTo(current.x,current.y); c.lineTo(forecast.x,forecast.y); c.stroke(); c.setLineDash([]);
      c.fillStyle='#ffaaaa'; c.font='11px system-ui'; c.fillText(`Visible movement forecast +${horizon}s`,forecast.x+5,forecast.y-10);
    }
    const plannedLegs = legs(), pat = PATTERNS[state.pattern];
    c.setLineDash([7, 6]); c.strokeStyle = 'rgba(122,205,227,.48)'; c.lineWidth = 2;
    plannedLegs.forEach(({from,to}) => { c.beginPath(); c.moveTo(from.x, from.y); c.lineTo(to.x, to.y); c.stroke(); });
    c.setLineDash([]);
    plannedLegs.forEach(({to:p}, i) => { c.strokeStyle = 'rgba(190,235,244,.7)'; c.beginPath(); c.arc(p.x, p.y, 11, 0, Math.PI * 2); c.stroke(); c.fillStyle = 'rgba(190,235,244,.8)'; c.font = '12px system-ui'; c.fillText(String(i + 1), p.x + 15, p.y + 4); });
    const shown = state.anchors.slice(), flightVisual = Array(7).fill(null);
    if (state.auto) {
      const a = state.auto, t = (now - a.started) * state.playback;
      a.sim.events.forEach(e => {
        if (t >= e.end) shown[e.sword] = e.to;
        else if (t >= e.start) {
          const progress = clamp((t - e.start) / (e.end - e.start), 0, 1);
          const p = { x: e.from.x + (e.to.x - e.from.x) * progress, y: e.from.y + (e.to.y - e.from.y) * progress };
          shown[e.sword] = p;
          flightVisual[e.sword] = { f: e, p, progress, angle: Math.atan2(e.to.y - e.from.y, e.to.x - e.from.x) - Math.PI / 2 };
        }
      });
      if (t >= a.sim.ms) {
        state.anchors = a.sim.anchors; state.auto = null;
        state.marks.push(...a.sim.events.filter(e => e.kind === 'draw').map(e => ({ sword: e.sword, from: e.from, to: e.to, until: performance.now() + 30000 })));
        state.result = { quality: a.sim.quality, ms: a.sim.ms, placements: a.sim.slots.length, corrections: a.sim.corrections, recalls: 0 };
        if (state.empowerment) state.empowerment = Math.max(0, state.empowerment - Math.min(3, a.sim.slots.length + 1));
        renderPalette(); renderResults();
      } else {
        const paletteKey = SWORDS.map((_, i) => swordStatus(i)).join('|');
        if (paletteKey !== a.paletteKey) { a.paletteKey = paletteKey; renderPalette(); }
      }
    }
    if (state.flights.length) {
      const landed = [];
      state.flights.forEach(f => {
        const progress = clamp((now - f.started) / f.ms, 0, 1);
        const p = { x: f.from.x + (f.to.x - f.from.x) * progress,
          y: f.from.y + (f.to.y - f.from.y) * progress };
        shown[f.sword] = p;
        flightVisual[f.sword] = { f, p, progress, angle: Math.atan2(f.to.y - f.from.y, f.to.x - f.from.x) - Math.PI / 2 };
        if (progress >= 1) { state.anchors[f.sword] = f.kind === 'return' ? null : f.to; if (f.kind === 'draw') {
          state.marks.push({ sword: f.sword, from: f.from, to: f.to, until: performance.now() + 30000 });
          if (state.empowerment) state.empowerment--;
        } landed.push(f); }
      });
      if (landed.length) { state.flights = state.flights.filter(f => !landed.includes(f)); renderPalette();
        if (landed.some(f=>f.kind==='draw')) evaluate(); }
    }
    state.marks = state.marks.filter(m => m.until > now);
    const marks = state.marks.concat(state.auto ? state.auto.sim.events.filter(e => e.kind === 'draw' && (now-state.auto.started)*state.playback >= e.start).map(e => {
      const t=(now-state.auto.started)*state.playback, progress=clamp((t-e.start)/(e.end-e.start),0,1);
      return { sword:e.sword, from:e.from, to:{x:e.from.x+(e.to.x-e.from.x)*progress,y:e.from.y+(e.to.y-e.from.y)*progress} };
    }) : []);
    marks.forEach(m => paintMark(c,m,now));
    if (state.cancelUntil > now) paintFlag(c,now,'cancelled');
    else if (state.auto) paintFlag(c,now,'drawing');
    else if (state.result) paintFlag(c,now,'complete');
    else if (state.slots.length) paintFlag(c,now,'planned');
    if (state.drag) {
      shown[state.drag.sword] = state.drag.now;
      c.strokeStyle = 'rgba(255,255,255,.42)'; c.lineWidth = 1;
      c.beginPath(); c.moveTo(state.drag.from.x, state.drag.from.y);
      c.lineTo(state.drag.now.x, state.drag.now.y); c.stroke();
    }
    // The lab uses 2x native pixels: 40,000 map units span about 80 canvas px.
    const centers=SWORDS.map((_,i)=>shown[i] || (state.scenario==='ally' && i===state.selected ? holder : HERO));
    shown.forEach((p,i)=>{if(p)paintSwordField(c,p,i,now);});
    const recipients=[{p:HERO,side:'ally'}];
    if(state.scenario==='ally')recipients.push({p:holder,side:'ally'});
    recipients.push({p:foe,side:'enemy'});
    recipients.forEach(({p,side})=>{
      const affecting=centers.map((center,i)=>distance(center,p)<=80?i:-1).filter(i=>i>=0);
      if(!affecting.length)return;
      paintAuraBase(c,p,side,now);
      affecting.forEach(i=>paintAura(c,p,i,side,now));
      c.save();
      c.font='11px system-ui';c.textAlign='center';
      c.fillStyle=side==='ally'?'#a6eac9':'#ffaab2';
      c.fillText(`${affecting.length} aura${affecting.length===1?'':'s'} · ${affecting.length===1?'full':`1/${affecting.length}`} strength`,p.x,p.y+33);
      if(side===state.auraSide){
        c.strokeStyle=side==='ally'?'#a6eac9':'#ffaab2';c.lineWidth=1;
        c.strokeRect(p.x-11,p.y-25,22,27);
      }
      c.restore();
    });
    [{p:foe,side:'enemy'},...(state.scenario==='ally'?[{p:holder,side:'ally'}]:[])].forEach(({p,side})=>{
      c.fillStyle=side==='ally'?'#62d6a4':'#e97a88';
      c.fillRect(p.x-6,p.y-20,12,18);
      c.fillStyle='#18283b';c.fillRect(p.x-3,p.y-16,6,5);
    });
    // Every available sword is a separate layer behind the character. A sword
    // vanishes from the orbit as soon as that exact indexed blade leaves it.
    const orbit = [[25, 191], [50, 136], [91, 112], [136, 136],
      [167, 191], [148, 250], [40, 250]];
    orbit.forEach(([x, y], i) => { if (!shown[i]) paintSword(c,
      { x: x + Math.sin(now / 430 + i) * 2, y: y + Math.sin(now / 310 + i) * 3 },
      i, 1.25, now, false, 0, state.previewState); });
    // The sheet's run row is a hovering glide, with no alternating leg contacts.
    if (state.sprite && state.sprite.complete && state.sprite.naturalWidth) {
      const floating = state.floatPreview;
      const frame = Math.floor(now / (floating ? 90 : 200)) % (floating ? 6 : 4);
      const drift = floating ? Math.sin(now / 720) * 12 : 0;
      const bob = floating ? Math.sin(now / 190) * 3 : Math.sin(now / 390) * 2;
      c.imageSmoothingEnabled = false;
      if (floating) {
        c.strokeStyle = 'rgba(85,231,242,.3)'; c.lineWidth = 2;
        c.beginPath(); c.moveTo(37 + drift, 232 + bob); c.lineTo(18 + drift, 237 + bob); c.stroke();
        c.beginPath(); c.moveTo(43 + drift, 247 + bob); c.lineTo(23 + drift, 251 + bob); c.stroke();
      }
      c.drawImage(state.sprite, frame * 48, floating ? 56 : 0, 48, 56,
        46 + drift, 169 + bob, 96, 112);
      paintRankBadge(c, now, drift, bob);
    }
    else { c.fillStyle = '#f5f2e7'; c.fillRect(HERO.x - 12, HERO.y - 18, 24, 37); }
    flightVisual.forEach(v => { if (v) paintThrowTrail(c, v.f, v.p, v.progress, now); });
    shown.forEach((p, i) => { if (p) paintSword(c, p, i, 1, now, !flightVisual[i], flightVisual[i] ? flightVisual[i].angle : 0,
      flightVisual[i] ? (flightVisual[i].f.kind === 'draw' ? 'drawing' : 'flight') : 'planted'); });
    c.fillStyle = '#f7d784'; c.font = 'bold 14px system-ui'; c.textAlign = 'center'; c.fillText('ISLIID', HERO.x, 310);
    c.fillStyle = '#b6c9d5'; c.font = '12px system-ui'; c.fillText(state.floatPreview ? 'Floating glide' : 'Imperial hover', HERO.x, 337);
    c.textAlign = 'left'; c.fillStyle = 'rgba(246,241,226,.8)'; c.font = '13px system-ui';
    c.fillText(pat.name + '  ·  ' + state.slots.length + ' swords', 20, 28);
    c.fillStyle = 'rgba(246,241,226,.58)'; c.fillText(pat.effect, 20, 48);
    if (state.started && !state.result && !state.auto) { c.fillStyle = '#fff3bc'; c.fillText('Manual time: ' + fmt(now - state.started), W - 165, 28); }
  }
  function loop(now) {
    state.raf = requestAnimationFrame(loop);
    if (!state.root || state.root.hidden || !state.root.isConnected) return;
    draw(now);
  }
  function point(ev) { const r = state.canvas.getBoundingClientRect(); return { x: clamp((ev.clientX - r.left) * W / r.width, 0, W), y: clamp((ev.clientY - r.top) * H / r.height, 0, H) }; }
  function nearest(p) { let best = -1, d = 20; state.anchors.forEach((a, i) => { if (a && distance(a, p) < d) { best = i; d = distance(a, p); } }); return best; }
  function wireCanvas() {
    const cv = state.canvas;
    cv.addEventListener('contextmenu', ev => { ev.preventDefault(); if (state.auto) return; const i = nearest(point(ev)); if (i >= 0) {
      const now=performance.now(), from=state.anchors[i];
      state.marks.forEach(m => { if (m.sword === i) m.until = Math.min(m.until, now + (ev.shiftKey ? 0 : 1000)); });
      if (state.slots.includes(i) && !state.result) state.cancelUntil = now + 1000;
      state.flights.push({ sword:i, from, to:HERO, started:now, ms:distance(from,HERO)/SWORD_SPEED[i]*1000/1.5, kind:'return' });
      state.anchors[i] = null; state.slots = state.slots.map(s => s === i ? null : s); state.recalls++; state.result = null; renderPalette(); renderResults(); } });
    cv.addEventListener('pointerdown', ev => {
      if (ev.button !== 0 || state.auto) return; cv.focus(); const p = point(ev), i = nearest(p);
      const sword = i >= 0 ? i : state.selected;
      if (state.flights.some(f => f.sword === sword)) return;
      const launch=state.scenario==='ally'?{x:W*.72,y:H*.78}:state.scenario==='objective'?{x:W*.83,y:H*.28}:HERO;
      state.drag = { sword, from: state.anchors[sword] || launch, now: p, wasPlanted: !!state.anchors[sword] };
      cv.setPointerCapture(ev.pointerId);
    });
    cv.addEventListener('pointermove', ev => { if (state.drag) state.drag.now = point(ev); });
    cv.addEventListener('pointerup', ev => {
      if (!state.drag || ev.button !== 0) return;
      const d = state.drag, p = point(ev); state.drag = null;
      if (!state.started) state.started = performance.now();
      state.result = null;
      if (d.wasPlanted) state.corrections++;
      else {
        const free = legs().map((leg, i) => ({ leg, i })).filter(x => state.slots[x.i] == null).sort((a, b) => distance(a.leg.to, p) - distance(b.leg.to, p));
        if (free.length) { state.slots[free[0].i] = d.sword; state.placements++; }
      }
      state.anchors[d.sword] = null;
      state.flights.push({ sword:d.sword, from:d.from, to:p, started:performance.now(),
        ms:distance(d.from,p)/SWORD_SPEED[d.sword]*1000, kind:state.active?'draw':'stage' });
      state.selected = d.sword; renderPalette(); renderResults();
    });
    cv.addEventListener('keydown', ev => { if (ev.key.toLowerCase() === 'r') { state.empowerment=3; renderResults(); ev.preventDefault(); } if (ev.key === 'Escape') { reset(); ev.preventDefault(); } });
  }
  function rankFacts() {
    const r = RANKS[state.rank];
    const label = state.rank === 7 ? `Imperial ${state.imperialLevel}` : r.name;
    const points = typeof r.points === 'number' ? `${r.points}+ points` : r.points;
    const badge = state.rank === 7 ? `Imperial badge family · level ${state.imperialLevel}` : `${r.name} badge · unique tier sprite`;
    const effects = [
      '+10,000 range / reveal', '5% guard / 10% slow', '6% physical attack / 8% defence shred',
      '10% movement / 8% attack slow', '6% vamp / 10% heal reduction', '6% radius / 6% slow',
      '6% cooldown / 5% attack reduction'];
    const count = state.scenario === 'ally' ? 1 : Math.max(1, 7-state.anchors.filter(Boolean).length);
    $('#ilRankFacts').innerHTML = `<b>${label}</b><br>${points}<br>Decision ${r.decision} ms per plan<br>Visible-state forecast ${LOOK_AHEAD[state.rank]} s<br>Patterns compared ${r.candidates} / 30<br>Aim spread ${r.error} px<br>Sword speeds ${SWORD_SPEED.join(', ')} px/s at every rank<br>Sword art: ${tierName[weaponTier(state.rank)]}<br>${badge}<br><b>${SWORDS[state.selected][0]} aura:</b> ${effects[state.selected]}<br>${count} overlapping swords: numeric effects divide by ${count}, rounded up; reveal stays local and unscaled.`;
  }
  function renderImperialControl() {
    const wrap = $('#ilImperialWrap');
    if (wrap) wrap.style.display = state.rank === 7 ? '' : 'none';
  }
  function save() { try { localStorage.setItem(STORE, JSON.stringify({ rank: state.rank, imperialLevel: state.imperialLevel, pattern: state.pattern, scale: state.scale, emperor: state.emperor, floatPreview: state.floatPreview, playback: state.playback, scenario: state.scenario, active: state.active, previewState: state.previewState, auraSide: state.auraSide, inspectFrame: state.inspectFrame })); } catch (_) {} }
  function mount(root) {
    if (state.root === root && state.canvas) return;
    state.root = root;
    try { const saved = JSON.parse(localStorage.getItem(STORE) || '{}');
      state.rank = clamp(+saved.rank || 0, 0, 7); state.imperialLevel = clamp(+saved.imperialLevel || 1, 1, IMPERIAL_LEVELS); state.pattern = PATTERNS[saved.pattern] ? saved.pattern : 'triangle';
      state.scale = [0.7, 1, 1.3].includes(+saved.scale) ? +saved.scale : 1; state.emperor = !!saved.emperor;
      state.floatPreview = saved.floatPreview !== false;
      state.playback = [1, 2, 4].includes(+saved.playback) ? +saved.playback : 1;
      state.scenario = ['self','ally','objective'].includes(saved.scenario) ? saved.scenario : 'self';
      state.active = saved.active !== false;
      state.previewState = ['orbit','flight','planted','ready','drawing'].includes(saved.previewState) ? saved.previewState : 'orbit';
      state.auraSide = saved.auraSide === 'enemy' ? 'enemy' : 'ally';
      state.inspectFrame = Number.isInteger(saved.inspectFrame) && saved.inspectFrame >= 0 && saved.inspectFrame < 8 ? saved.inspectFrame : -1;
    } catch (_) {}
    root.innerHTML = `<div class="il-shell">
      <div class="il-head"><div><h2>Imperial Engravings lab</h2><p>Stage swords without marks, or activate each one to engrave. Colored lines are persistent sword marks; all swords can draw in parallel.</p></div><span class="il-chip">EDITOR PROTOTYPE</span></div>
      <div class="il-controls">
        <label>Formation<select id="ilPattern">${Object.entries(PATTERNS).map(([k, p]) => `<option value="${k}"${k === state.pattern ? ' selected' : ''}>${esc(p.name)}</option>`).join('')}</select></label>
        <label>Mastery<select id="ilRank">${RANKS.map((r, i) => `<option value="${i}"${i === state.rank ? ' selected' : ''}>${esc(r.name)} (${r.points}${typeof r.points === 'number' ? '+' : ''})</option>`).join('')}</select></label>
        <label id="ilImperialWrap"${state.rank === 7 ? '' : ' style="display:none"'}>Imperial level<select id="ilImperial">${Array.from({ length: IMPERIAL_LEVELS }, (_, i) => i + 1).map(v => `<option value="${v}"${v === state.imperialLevel ? ' selected' : ''}>Imperial ${v}</option>`).join('')}</select></label>
        <label>Scale<select id="ilScale">${[[0.7, 'Small'], [1, 'Medium'], [1.3, 'Large']].map(([v, n]) => `<option value="${v}"${v === state.scale ? ' selected' : ''}>${n}</option>`).join('')}</select></label>
        <label>Playback<select id="ilPlayback">${[1, 2, 4].map(v => `<option value="${v}"${v === state.playback ? ' selected' : ''}>${v}x</option>`).join('')}</select></label>
        <label>Assignment<select id="ilScenario">${[['self','All with Isliid'],['ally','Supporting an ally'],['objective','Staged at objective']].map(([v,n])=>`<option value="${v}"${state.scenario===v?' selected':''}>${n}</option>`).join('')}</select></label>
        <label>Sword state<select id="ilSwordState">${['orbit','flight','planted','ready','drawing'].map(v=>`<option value="${v}"${state.previewState===v?' selected':''}>${v}</option>`).join('')}</select></label>
        <label>Aura preview<select id="ilAuraSide"><option value="ally"${state.auraSide==='ally'?' selected':''}>Friendly</option><option value="enemy"${state.auraSide==='enemy'?' selected':''}>Enemy</option></select></label>
        <label>Animation frame<select id="ilFrame"><option value="-1"${state.inspectFrame<0?' selected':''}>Play all 8</option>${Array.from({length:8},(_,i)=>`<option value="${i}"${state.inspectFrame===i?' selected':''}>Frame ${i+1}</option>`).join('')}</select></label>
        <label class="il-check"><input type="checkbox" id="ilActive"${state.active?' checked':''}> Activate engraving on drag</label>
        <label class="il-check"><input type="checkbox" id="ilEmperor"${state.emperor ? ' checked' : ''}> Auto includes Emperor sword</label>
        <label class="il-check"><input type="checkbox" id="ilFloat"${state.floatPreview ? ' checked' : ''}> Preview floating glide</label>
      </div>
      <div class="il-main"><div><canvas id="ilCanvas" tabindex="0" width="${W}" height="${H}"></canvas>
        <div class="il-actions"><button class="btn small primary" data-il="auto">Auto draw this rank</button><button class="btn small" data-il="compare">Compare all ranks (50 runs)</button><button class="btn small" data-il="manifest">Empower next 3 (R)</button><button class="btn small" data-il="cancel">Cancel engraving</button><button class="btn small" data-il="reset">Reset (Esc)</button></div>
        <div id="ilResult" class="il-result"></div></div>
        <aside class="il-side"><h3>Seven swords</h3><div id="ilPalette" class="il-palette"></div>
          <p class="muted">Select a sword and drag it to stage or draw. Any sword can leave several colored marks. Auto draw stages and launches swords in parallel. An activated sword cannot be recalled until its stroke finishes. Then right-click to recall at 1.5× speed; Shift + right-click erases its marks instantly. R empowers the next three completions.</p>
          <h3>Mastery model</h3><div id="ilRankFacts"></div>
          <p class="muted">Official game: 1 point; scrim or exhibition: 0.5; win: 1.5×. Generate Isliid mastery in Skill Test → Mastery. This canvas previews drawing; the tactical AI runs in live matches.</p></aside></div>
      <div id="ilTable" class="il-table"></div></div>`;
    state.canvas = $('#ilCanvas'); state.ctx = state.canvas.getContext('2d');
    state.sprite = new Image(); state.sprite.src = '/isliid-sprite-sheet.png';
    state.weapons = new Image(); state.weapons.src = '/isliid-swords8-8.png';
    state.badges = new Image(); state.badges.src = '/isliid-badges8-8.png';
    state.auras = new Image(); state.auras.src = '/isliid-auras8-8.png';
    state.fields = new Image(); state.fields.src = '/isliid-aura_fields8-8.png';
    state.flags = new Image(); state.flags.src = '/isliid-flags-8.png';
    fetch('/isliid-art-manifest.json').then(r => r.json()).then(art => {state.art=art;}).catch(() => {});
    state.trails = new Image(); state.trails.src = '/isliid-trails.png';
    wireCanvas();
    root.addEventListener('click', ev => {
      const s = ev.target.closest('[data-il-sword]'); if (s) { state.selected = +s.dataset.ilSword; renderPalette(); rankFacts(); state.canvas.focus(); return; }
      const b = ev.target.closest('[data-il]'); if (!b) return;
      if (b.dataset.il === 'auto') autoDraw(); if (b.dataset.il === 'compare') compare();
      if (b.dataset.il === 'manifest') { state.empowerment=3; renderResults(); }
      if (b.dataset.il === 'cancel') { state.cancelUntil=performance.now()+1000; state.auto=null; state.flights=state.flights.filter(f=>f.kind!=='draw'); state.marks.forEach(m=>{m.until=Math.min(m.until,performance.now()+1000);}); state.result=null; renderResults(); }
      if (b.dataset.il === 'reset') reset();
    });
    root.addEventListener('change', ev => {
      const t = ev.target;
      if (t.id === 'ilRank') { state.rank = +t.value; renderImperialControl(); }
      if (t.id === 'ilImperial') state.imperialLevel = clamp(+t.value, 1, IMPERIAL_LEVELS);
      if (t.id === 'ilPattern') { state.pattern = t.value; state.compare = null; reset(); }
      if (t.id === 'ilScale') { state.scale = +t.value; state.compare = null; reset(); }
      if (t.id === 'ilPlayback') state.playback = +t.value;
      if (t.id === 'ilScenario') { state.scenario=t.value; state.compare=null; reset(); }
      if (t.id === 'ilSwordState') state.previewState=t.value;
      if (t.id === 'ilAuraSide') state.auraSide=t.value;
      if (t.id === 'ilFrame') state.inspectFrame=+t.value;
      if (t.id === 'ilActive') state.active=t.checked;
      if (t.id === 'ilEmperor') { state.emperor = t.checked; state.compare = null; reset(); }
      if (t.id === 'ilFloat') state.floatPreview = t.checked;
      rankFacts();
      save(); renderResults();
    });
    renderImperialControl(); rankFacts();
    reset();
    if (!state.raf) state.raf = requestAnimationFrame(loop);
  }

  const api = { mount, simulate, compare, score, targets, weaponTier, PATTERNS, RANKS, SWORDS, _state: state };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (typeof window !== 'undefined') window.TFM2IsliidLab = api;
})();
