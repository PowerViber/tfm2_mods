/* Map tab: where the map-altering champions use their map / vision skills (Omen's smokes, Steve's walls).
 * The rest of the team needs no plan: they react to the vision and the walls by themselves.
 * Plans are made per team line-up of these champions (only Omen, only Steve, both...): the native mod
 * (tfm2_custom_ai 0.6.7+) uses a plan only when the team's line-up matches. It reads them at every match start from
 * mods/tfm2_custom_ai/tactics.txt. The map comes from map_dump.json (written by the native mod at a game run's first
 * match); before that, only the walls are shown (read off the game's baked visibility data).
 * Next map-altering champion: add it to CHAMPS (and its key to MAP_CHAMPS in tactics.rs).
 */
(function () {
  'use strict';
  const W = 960000, K = 1000;                       // world size; SVG units are world / 1000
  const SMOKE_R = 50000, WALL_MAX = 300000;
  const TEAM = ['#4aa3ff', '#ff5a5a'];
  // the map-altering champions and their tools
  const CHAMPS = [
    { key: 'omen', name: 'Omen', tools: [
      { id: 'smoke', kind: 'smoke', label: 'Smoke here', icon: '☁', set: { trigger: 'fight', prio: 3 }, tip: 'Omen smokes this spot (only with his team near it)' },
      { id: 'fake', kind: 'smoke', label: 'Fake smoke', icon: '?', set: { trigger: 'fight', prio: 1, fake: true }, tip: 'Omen smokes it to deceive: nobody needs to be there' }] },
    { key: 'steve', name: 'Steve', tools: [
      { id: 'wall', kind: 'wall', label: 'Wall here', icon: '▬', set: { trigger: 'isolate', prio: 4 }, tip: 'Drag a line: Steve builds this wall (the open stretch around its middle, never over terrain)' }] },
  ];
  const KIND_CHAMP = { smoke: 'omen', wall: 'steve' };
  const KINDS = { smoke: 'Smoke', wall: 'Wall' };
  // "when" a plan is on, grouped for the menus (the native rules: tactics.rs `active`)
  const TRIG_GROUPS = [
    ['Any time', [['always', 'Always'], ['early', 'Early game (first 8 min)'], ['midgame', 'Mid game (8-18 min)'], ['late', 'Late game (18 min+)']]],
    ['Objectives', [['serpen_up', 'Serpent is up'], ['serpen', 'Serpent up, someone at it'], ['serpen_fight', 'Serpent fight (both teams at it)'],
      ['epic_up', 'Morgard is up'], ['epic', 'Morgard up, someone at it'], ['epic_fight', 'Morgard fight (both teams at it)'],
      ['won_serpen', 'Won a fight + Serpent up'], ['won_epic', 'Won a fight + Morgard up'],
      ['ahead_serpen', 'More of us alive + Serpent up'], ['ahead_epic', 'More of us alive + Morgard up'],
      ['behind_serpen', 'Fewer of us alive + Serpent up'], ['behind_epic', 'Fewer of us alive + Morgard up']]],
    ['Fights', [['fight', 'A fight here'], ['teamfight', 'Teamfight here (3+ each)'], ['won', 'Just won a fight (15 s)'], ['lost', 'Just lost a fight (15 s)'],
      ['ahead', 'More of us alive'], ['behind', 'Fewer of us alive']]],
    ['Here', [['defend', 'Defending here (2+ enemies)'], ['attack', 'Attacking here (2+ of us)'], ['gank', 'Gank chance here (a lone enemy, 2+ of us near)'],
      ['coming', 'Enemies coming here (just outside it)'], ['tower', 'Our tower here under attack'],
      ['isolate', 'Isolate a fight (walls: a fight on one side, more enemies coming on the other)']]],
  ];
  const TRIGGERS = TRIG_GROUPS.flatMap(g => g[1]);
  const M = { loaded: false, loading: null, source: 'none', doc: null, marks: [], tool: 'select', toolId: null, sel: null,
    def: { trigger: 'fight', side: 'both', prio: 3, fake: false }, comp: ['omen', 'steve'], flipY: false,
    layers: { walls: true, bushes: true, objects: true, ghosts: true }, dirty: false, drag: null, status: '', collapsed: {}, q: '' };
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const uid = () => Math.random().toString(36).slice(2, 9);
  const round = v => Math.round(v);
  const opt = (list, v) => list.map(([k, l]) => `<option value="${esc(k)}"${String(k) === String(v) ? ' selected' : ''}>${esc(l)}</option>`).join('');
  const optg = (groups, v) => groups.map(([g, list]) => `<optgroup label="${esc(g)}">${opt(list, v)}</optgroup>`).join('');
  const compKey = c => [...c].sort().join('+');
  const compName = c => c.length ? c.map(k => (CHAMPS.find(x => x.key === k) || { name: k }).name).join(' + ') : 'nobody';
  const inComp = k => compKey(k.comp || []) === compKey(M.comp);

  // ---------------------------------------------------------------- map document helpers
  const isPt = v => Array.isArray(v) && v.length >= 2 && typeof v[0] === 'number' && typeof v[1] === 'number';
  function pointsIn(v, out = []) {
    if (isPt(v)) { out.push([v[0], v[1]]); return out; }
    if (Array.isArray(v)) v.forEach(x => pointsIn(x, out));
    else if (v && typeof v === 'object') Object.values(v).forEach(x => pointsIn(x, out));
    return out;
  }
  function grid(name) {
    const g = M.doc && M.doc[name];
    return Array.isArray(g) && g.length >= 30 && Array.isArray(g[0]) ? g : null;
  }
  // rows = y unless that puts towers inside walls (the native WallReader decides the same way)
  function rowsAreY() {
    const g = grid('walls'); if (!g) return true;
    const towers = pointsIn((M.doc || {}).towers || []);
    const at = (x, y, ry) => { const cx = Math.min(29, Math.max(0, Math.floor(x / 32000))), cy = Math.min(29, Math.max(0, Math.floor(y / 32000)));
      return ry ? +g[cy][cx] : +g[cx][cy]; };
    return towers.filter(p => at(p[0], p[1], true)).length <= towers.filter(p => at(p[0], p[1], false)).length;
  }
  function nexus() {
    const n = (M.doc && M.doc.nexus_pos) ? pointsIn(M.doc.nexus_pos) : [];
    return n.length >= 2 ? n.slice(0, 2) : null;
  }
  // the other side's version of a point: the map is mirrored across the line halfway between the two bases (on the
  // 5v5 map that's the river diagonal, so Morgard and the Serpent each stay where they are and Blue's camps become Red's)
  function mirror(p) {
    const n = nexus() || [[96000, 864000], [864000, 96000]];
    const m = [(n[0][0] + n[1][0]) / 2, (n[0][1] + n[1][1]) / 2];
    let ux = n[1][0] - n[0][0], uy = n[1][1] - n[0][1]; const l = Math.hypot(ux, uy) || 1; ux /= l; uy /= l;
    const d = (p[0] - m[0]) * ux + (p[1] - m[1]) * uy;
    return [p[0] - 2 * d * ux, p[1] - 2 * d * uy];
  }
  function label(item) {
    if (!item || typeof item !== 'object') return '';
    for (const k of ['ty', 'kind', 'name', 'monster', 'type', 'camp', 'id']) if (typeof item[k] === 'string') return item[k];
    return '';
  }

  // ---------------------------------------------------------------- server
  async function load() {
    if (M.loading) return M.loading;
    M.loading = (async () => {
      try { const r = await fetch('/api/mapdump', { cache: 'no-store' }); const j = await r.json(); M.source = j.source || 'none'; M.doc = j.doc || null; }
      catch (e) { M.source = 'none'; }
      try {
        const r = await fetch('/api/tactics', { cache: 'no-store' }); const j = await r.json(); const d = j.doc || {};
        // only the map-altering skills live here now (older plan types are dropped on the next save)
        M.marks = (d.marks || []).filter(k => KIND_CHAMP[k.kind]).map(k => Object.assign({ comp: [KIND_CHAMP[k.kind]] }, k));
        M.flipY = !!d.flipY;
        if (Array.isArray(d.comp)) M.comp = d.comp;
      } catch (e) { /* no server: start empty */ }
      M.loaded = true; M.dirty = false; M.loading = null;
    })();
    return M.loading;
  }

  function txt() {
    const lines = ['# tactics.txt — written by the TFM2 editor Map tab; read by tfm2_custom_ai at every match start',
      '# kind champ trigger team priority ax ay bx by radius who comp'];
    for (const k of M.marks.filter(k => !k.off && (k.comp || []).length)) {
      const a = k.a, b = k.b || k.a;
      const sides = k.side === 'both' ? [[0, a, b], [1, mirror(a), mirror(b)]] : [[k.side === 'red' ? 1 : 0, a, b]];
      for (const [team, p, q] of sides)
        lines.push([k.kind, KIND_CHAMP[k.kind], k.trigger || 'always', team, k.prio || 3, round(p[0]), round(p[1]), round(q[0]), round(q[1]), 0,
          k.fake ? 'fake' : 'all', 'comp=' + compKey(k.comp)].join(' ') + (k.note ? '   # ' + String(k.note).replace(/[\r\n#]/g, ' ') : ''));
    }
    return lines.join('\n') + '\n';
  }

  async function save() {
    const body = { doc: { version: 2, flipY: M.flipY, comp: M.comp, marks: M.marks }, txt: txt() };
    try {
      const r = await fetch('/api/tactics', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
      const j = await r.json();
      if (!r.ok) throw new Error(j.error || r.status);
      M.dirty = false; M.status = 'Saved. Used from the next match (native mod 0.6.7 or newer).';
    } catch (e) { M.status = 'Could not save: ' + e.message; }
    render();
  }

  // ---------------------------------------------------------------- drawing
  function svgMap() {
    const parts = [];
    const ry = rowsAreY();
    const cell = (g, cx, cy) => (ry ? g[cy] && g[cy][cx] : g[cx] && g[cx][cy]);
    parts.push(`<rect x="0" y="0" width="960" height="960" fill="#3c4a38"/>`);
    const walls = grid('walls'), bushes = grid('bushes');
    if (M.layers.bushes && bushes) for (let cy = 0; cy < 30; cy++) for (let cx = 0; cx < 30; cx++)
      if (+cell(bushes, cx, cy)) parts.push(`<rect x="${cx * 32}" y="${cy * 32}" width="32" height="32" fill="#2f7a3d" opacity=".8"/>`);
    if (M.layers.walls && walls) for (let cy = 0; cy < 30; cy++) for (let cx = 0; cx < 30; cx++)
      if (+cell(walls, cx, cy)) parts.push(`<rect x="${cx * 32}" y="${cy * 32}" width="32" height="32" fill="#16191e"/>`);
    for (let i = 0; i <= 30; i += 5) parts.push(`<line x1="${i * 32}" y1="0" x2="${i * 32}" y2="960" stroke="#ffffff10"/><line x1="0" y1="${i * 32}" x2="960" y2="${i * 32}" stroke="#ffffff10"/>`);
    const d = M.doc || {};
    if (M.layers.objects) {
      for (const pl of (Array.isArray(d.lanes) ? d.lanes : [])) { const pts = pointsIn(pl); if (pts.length >= 2) parts.push(`<polyline points="${pts.map(p => `${p[0] / K},${p[1] / K}`).join(' ')}" fill="none" stroke="#e8d9a8" stroke-width="3" stroke-dasharray="8 6" opacity=".55"/>`); }
      (Array.isArray(d.fountains) ? d.fountains : []).forEach((f, t) => {
        const v = (f || []).flat ? f.flat(3).filter(x => typeof x === 'number') : [];
        if (v.length >= 4) { const [x1, y1, x2, y2] = v; parts.push(`<rect x="${Math.min(x1, x2) / K}" y="${Math.min(y1, y2) / K}" width="${Math.abs(x2 - x1) / K}" height="${Math.abs(y2 - y1) / K}" fill="${TEAM[t % 2]}22" stroke="${TEAM[t % 2]}" stroke-width="2"/>`); }
      });
      for (const c of (Array.isArray(d.camps) ? d.camps : [])) {
        const lab = label(c); const big = /epic|serpen|morgard/i.test(lab);
        for (const p of pointsIn(c && c.pos).slice(0, 2)) {
          const [x, y] = [p[0] / K, p[1] / K]; const r = big ? 13 : 8;
          parts.push(`<path d="M${x} ${y - r}L${x + r} ${y}L${x} ${y + r}L${x - r} ${y}Z" fill="${big ? '#c77dff' : '#f2b33d'}" stroke="#000" stroke-width="1.5"/>`);
          if (lab) parts.push(`<text x="${x}" y="${y + r + 11}" class="me-lab">${esc(lab)}</text>`);
        }
      }
      for (const t of (Array.isArray(d.towers) ? d.towers : [])) {
        const pos = t && t.pos;
        if (Array.isArray(pos) && pos.length === 2 && isPt(pos[0]) && isPt(pos[1])) pos.forEach((p, team) => parts.push(`<rect x="${p[0] / K - 7}" y="${p[1] / K - 7}" width="14" height="14" fill="${TEAM[team]}" stroke="#000" stroke-width="1.5"/>`));
      }
      const n = nexus();
      if (n) n.forEach((p, t) => parts.push(`<circle cx="${p[0] / K}" cy="${p[1] / K}" r="16" fill="${TEAM[t]}" stroke="#000" stroke-width="2"/><text x="${p[0] / K}" y="${p[1] / K + 4}" class="me-lab big">${t ? 'RED' : 'BLUE'}</text>`));
    }
    // this line-up's plans (and their mirrored copies for "both sides")
    const drawMark = (k, a, b, ghost) => {
      const sel = !ghost && M.sel === k.id;
      const op = k.off ? 0.18 : ghost ? 0.35 : 1, dash = ghost ? 'stroke-dasharray="6 4"' : '';
      const col = ghost ? TEAM[1] : k.side === 'red' ? TEAM[1] : TEAM[0];
      const attr = `data-mark="${k.id}"${ghost ? ' data-ghost="1"' : ''}`;
      if (k.kind === 'smoke') parts.push(`<g ${attr} class="me-hit" opacity="${op}"><circle cx="${a[0] / K}" cy="${a[1] / K}" r="${SMOKE_R / K}" fill="${k.fake ? '#c9a0ff33' : '#2a1840cc'}" stroke="${sel ? '#fff' : col}" stroke-width="${sel ? 3 : 2}" ${k.fake && !ghost ? 'stroke-dasharray="4 3"' : dash}/><text x="${a[0] / K}" y="${a[1] / K + 5}" class="me-lab big">${k.fake ? '?' : ''}${k.prio}</text></g>`);
      if (k.kind === 'wall') parts.push(`<g ${attr} class="me-hit" opacity="${op}"><line x1="${a[0] / K}" y1="${a[1] / K}" x2="${b[0] / K}" y2="${b[1] / K}" stroke="#000" stroke-opacity="0.001" stroke-width="30" stroke-linecap="round"/><line x1="${a[0] / K}" y1="${a[1] / K}" x2="${b[0] / K}" y2="${b[1] / K}" stroke="#000" stroke-width="12" stroke-linecap="square"/><line x1="${a[0] / K}" y1="${a[1] / K}" x2="${b[0] / K}" y2="${b[1] / K}" stroke="${sel ? '#fff' : '#a08a7a'}" stroke-width="8" stroke-linecap="square" ${dash}/><circle cx="${a[0] / K}" cy="${a[1] / K}" r="5" fill="${col}"/><circle cx="${b[0] / K}" cy="${b[1] / K}" r="5" fill="${col}"/><text x="${(a[0] + b[0]) / 2 / K}" y="${(a[1] + b[1]) / 2 / K - 9}" class="me-lab big">${k.prio}</text></g>`);
    };
    const mine = M.marks.filter(inComp);
    if (M.layers.ghosts) for (const k of mine) if (k.side === 'both' && !k.off) drawMark(k, mirror(k.a), mirror(k.b || k.a), true);
    const order = k => (k.id === M.sel ? 3 : 0) + (k.kind === 'wall' ? 1 : 0) + (k.off ? -1 : 0);
    for (const k of [...mine].sort((x, y) => order(x) - order(y))) drawMark(k, k.a, k.b || k.a, false);
    if (M.drag && M.drag.kind === 'newwall') { const { a, b } = M.drag; parts.push(`<line x1="${a[0] / K}" y1="${a[1] / K}" x2="${b[0] / K}" y2="${b[1] / K}" stroke="#fff" stroke-width="6" opacity=".7"/>`); }
    const tf = M.flipY ? 'transform="translate(0 960) scale(1 -1)"' : '';
    return `<svg id="meSvg" viewBox="0 0 960 960" class="me-svg tool-${M.tool}"><g ${tf}>${parts.join('')}</g></svg>`;
  }

  // ---------------------------------------------------------------- panels
  const selected = () => M.marks.find(k => k.id === M.sel) || null;

  function editorHTML(s) {
    return `<div class="me-edit">
        <label>When<select data-f="trigger">${optg(TRIG_GROUPS, s.trigger)}</select></label>
        <div class="me-row2"><label>Side<select data-f="side">${opt([['both', 'Both'], ['blue', 'Blue'], ['red', 'Red']], s.side)}</select></label>
        <label>Priority<select data-f="prio">${opt([['5', '5 (first)'], ['4', '4'], ['3', '3'], ['2', '2'], ['1', '1 (last)']], String(s.prio))}</select></label></div>
        ${s.kind === 'smoke' ? `<label class="me-check"><input type="checkbox" data-f="fake" ${s.fake ? 'checked' : ''}> Fake (to deceive: nobody needs to be there)</label>` : ''}
        <label>Note<input data-f="note" value="${esc(s.note || '')}" placeholder="e.g. blind the river from mid"></label>
        <div class="me-row"><span class="muted small">${s.kind === 'wall' ? `length ${Math.round(Math.hypot(s.b[0] - s.a[0], s.b[1] - s.a[1]))} (max ${WALL_MAX})` : `at ${round(s.a[0])}, ${round(s.a[1])}`}</span><div class="spacer"></div>
          <button class="btn small" data-act="dup">Duplicate</button><button class="btn small danger" data-act="del">Delete</button></div></div>`;
  }

  // left: this line-up's playbook, grouped by situation
  function leftHTML() {
    const q = M.q.trim().toLowerCase();
    const mine = M.marks.filter(inComp);
    const match = k => !q || [KINDS[k.kind], k.note, (TRIGGERS.find(t => t[0] === k.trigger) || [])[1]].some(x => String(x || '').toLowerCase().includes(q));
    const groups = TRIGGERS.map(([t, l]) => [t, l, mine.filter(k => k.trigger === t && match(k)).sort((a, b) => b.prio - a.prio)]).filter(x => x[2].length);
    let html = `<div class="me-card me-book"><div class="me-h">Playbook: ${esc(compName(M.comp))} <span class="muted">${mine.filter(k => !k.off).length} on / ${mine.length}</span></div>
      <input class="me-search" data-q placeholder="Search plans…" value="${esc(M.q)}">`;
    if (!groups.length) html += `<div class="muted small">${mine.length ? 'No match.' : 'No plans for this line-up yet: pick a tool on the right and click the map' + (M.comp.length > 1 ? ', or copy plans from a line-up with fewer of them.' : '.')}</div>`;
    for (const [t, l, ks] of groups) {
      const allOn = ks.every(k => !k.off), col = !!M.collapsed[t];
      html += `<div class="me-sit"><div class="me-sit-h"><input type="checkbox" data-sit-on="${t}" ${allOn ? 'checked' : ''} title="Turn this whole situation on/off">
        <span class="me-sit-t" data-sit-col="${t}">${col ? '▸' : '▾'} ${esc(l)} <span class="muted">(${ks.length})</span></span></div>`;
      if (!col) for (const k of ks) {
        html += `<div class="me-item${M.sel === k.id ? ' on' : ''}${k.off ? ' off' : ''}"><input type="checkbox" data-mark-on="${k.id}" ${k.off ? '' : 'checked'}>
          <span class="me-item-t" data-pick="${k.id}"><span class="me-dot ${k.kind}${k.fake ? ' fake' : ''}"></span><b>${k.prio}</b> ${esc(k.note || (k.fake ? 'Fake smoke' : KINDS[k.kind]))}
          <span class="muted small">${k.fake ? 'Fake smoke' : KINDS[k.kind]} · ${esc(compName([KIND_CHAMP[k.kind]]))}${k.side !== 'both' ? ' · ' + esc(k.side) : ''}</span></span></div>`;
        if (M.sel === k.id) html += editorHTML(k);
      }
      html += '</div>';
    }
    html += '</div>';
    const src = M.source === 'dump' ? 'Map read from your game.'
      : M.source === 'fallback' ? 'Walls only: build the native mod and play one match to get the full map.'
      : 'Map not found (is the editor running next to the game?).';
    html += `<div class="me-card muted small">${src} A plan is used only by a team with exactly this line-up, while its "when" is true (a smoke also needs the team near it, unless it's a fake). The rest of the team needs no plan: they react to the smokes and walls. Priority 5 beats almost everything the champion would do on its own, 1 loses to most. Unticked plans are kept but not used. Changes apply from the next match.</div>`;
    return html;
  }

  // right: the line-up's champions and their tools
  function rightHTML() {
    let html = '';
    const placing = M.tool !== 'select';
    if (placing) {
      const t = CHAMPS.flatMap(c => c.tools).find(x => x.id === M.toolId) || {};
      html += `<div class="me-card me-placing"><div class="me-h">Placing: ${esc(t.label || '')} <span class="muted small">${M.tool === 'wall' ? 'drag on the map' : 'click the map'}</span></div>
        <label>When<select data-def="trigger">${optg(TRIG_GROUPS, M.def.trigger)}</select></label>
        <div class="me-row2"><label>Side<select data-def="side">${opt([['both', 'Both'], ['blue', 'Blue'], ['red', 'Red']], M.def.side)}</select></label>
        <label>Priority<select data-def="prio">${opt([['5', '5'], ['4', '4'], ['3', '3'], ['2', '2'], ['1', '1']], String(M.def.prio))}</select></label></div>
        <button class="btn small" data-tool="select">Done</button></div>`;
    }
    for (const c of CHAMPS.filter(c => M.comp.includes(c.key))) {
      html += `<div class="me-card"><div class="me-h">${esc(c.name)}</div><div class="me-grid">` + c.tools.map(x =>
        `<button class="me-sc${placing && M.toolId === x.id ? ' on' : ''}" data-tool-id="${x.id}" title="${esc(x.tip)}"><span class="me-sc-i">${x.icon}</span>${esc(x.label)}</button>`).join('') + '</div></div>';
    }
    // a bigger line-up can start from the plans of a smaller one
    if (M.comp.length > 1) {
      const subs = M.comp.map(k => [k]);
      html += `<div class="me-card"><div class="me-h">Copy plans</div><div class="muted small">Start from what a team with fewer of them does, then adjust.</div><div class="me-grid">` +
        subs.map(c => `<button class="me-sc" data-copy="${esc(compKey(c))}">from ${esc(compName(c))} (${M.marks.filter(k => compKey(k.comp || []) === compKey(c)).length})</button>`).join('') + '</div></div>';
    }
    if (!M.comp.length) html += `<div class="me-card muted small">Tick at least one champion in "Team has" above.</div>`;
    return html;
  }

  function render() {
    const root = $('#mapEditor'); if (!root) return;
    if (!M.loaded) { root.innerHTML = '<div class="muted" style="padding:20px">Loading the map…</div>'; load().then(render); return; }
    const left = root.querySelector('.me-left'), keepScroll = left ? left.scrollTop : 0;
    root.innerHTML = `
      <div class="me-bar">
        <span class="me-h">Team has:</span>
        ${CHAMPS.map(c => `<label class="me-comp"><input type="checkbox" data-comp="${c.key}" ${M.comp.includes(c.key) ? 'checked' : ''}> ${esc(c.name)}</label>`).join('')}
        <span class="muted small">Each line-up (only ${CHAMPS.map(c => c.name).join(', only ')}, or together) has its own plans.</span>
        <div class="spacer"></div>
        <span class="muted small">${esc(M.status)}</span>
        <button class="btn primary" data-act="save" ${M.dirty ? '' : 'disabled'}>Save plans</button>
      </div>
      <div class="me-3col">
        <div class="me-left">${leftHTML()}</div>
        <div class="me-mapwrap"><div class="me-map" id="meMap">${svgMap()}</div>
          <div class="me-layers">${Object.keys(M.layers).map(l => `<label><input type="checkbox" data-layer="${l}" ${M.layers[l] ? 'checked' : ''}> ${l}</label>`).join('')}
            <label><input type="checkbox" data-flip ${M.flipY ? 'checked' : ''}> flip vertically (view only)</label></div></div>
        <div class="me-right">${rightHTML()}</div>
      </div>`;
    const l2 = root.querySelector('.me-left'); if (l2) l2.scrollTop = keepScroll;
  }

  // ---------------------------------------------------------------- interaction
  function worldAt(ev) {
    const svg = $('#meSvg'); if (!svg) return null;
    const r = svg.getBoundingClientRect();
    let x = (ev.clientX - r.left) / r.width * W, y = (ev.clientY - r.top) / r.height * W;
    if (M.flipY) y = W - y;
    return [Math.max(0, Math.min(W, x)), Math.max(0, Math.min(W, y))];
  }
  const touch = () => { M.dirty = true; M.status = ''; };
  function clampWall(a, b) {
    const d = Math.hypot(b[0] - a[0], b[1] - a[1]);
    return d <= WALL_MAX ? b : [a[0] + (b[0] - a[0]) * WALL_MAX / d, a[1] + (b[1] - a[1]) * WALL_MAX / d];
  }
  const newMark = (kind, a, b) => ({ id: uid(), kind, trigger: M.def.trigger, side: M.def.side, prio: +M.def.prio, fake: !!M.def.fake, comp: [...M.comp].sort(),
    note: (CHAMPS.flatMap(c => c.tools).find(x => x.id === M.toolId) || {}).label || '', a, b: b || a });

  function onDown(ev) {
    if (ev.button !== 0) return;
    const p = worldAt(ev); if (!p) return;
    // every plan under the pointer, top first (real plans before mirrored copies)
    const hits = [];
    for (const el of document.elementsFromPoint(ev.clientX, ev.clientY)) {
      const g = el.closest && el.closest('[data-mark]'); if (!g) continue;
      const h = { id: g.dataset.mark, ghost: !!g.dataset.ghost };
      if (!hits.some(x => x.id === h.id && x.ghost === h.ghost)) hits.push(h);
    }
    hits.sort((a, b) => a.ghost - b.ghost);
    // grabbing a plan always moves it, whatever tool is picked (a new smoke / wall is only placed on empty ground).
    // Where plans overlap, the selected one wins; clicking again without moving picks the next one underneath.
    const pick = hits.find(h => h.id === M.sel) || hits[0];
    if (pick && !(M.tool === 'wall' && ev.shiftKey)) {
      const k = M.marks.find(x => x.id === pick.id);
      if (k) {
        const ghost = pick.ghost, q = ghost ? mirror(p) : p;
        const wasSel = M.sel === k.id;
        M.sel = k.id;
        if (hits.length > 1) M.status = `${hits.length} plans overlap here: click again (without moving) to pick the next one.`;
        const end = k.kind === 'wall' ? (Math.hypot(q[0] - k.a[0], q[1] - k.a[1]) < 18000 ? 'a' : Math.hypot(q[0] - k.b[0], q[1] - k.b[1]) < 18000 ? 'b' : null) : null;
        M.drag = { kind: end ? 'end' : 'move', end, last: q, ghost, start: [ev.clientX, ev.clientY], moved: false, hits, wasSel };
        render(); ev.preventDefault(); return;
      }
    }
    if (M.tool === 'select') { M.sel = null; render(); ev.preventDefault(); return; }
    if (M.tool === 'smoke') { const k = newMark('smoke', p); M.marks.push(k); M.sel = k.id; touch(); render(); }
    else if (M.tool === 'wall') { M.drag = { kind: 'newwall', a: p, b: p }; ev.preventDefault(); }
  }
  function onMove(ev) {
    if (!M.drag) return;
    const p = worldAt(ev); if (!p) return;
    if (M.drag.kind === 'newwall') { M.drag.b = clampWall(M.drag.a, p); render(); return; }
    const s = selected(); if (!s) return;
    if (!M.drag.moved && Math.hypot(ev.clientX - M.drag.start[0], ev.clientY - M.drag.start[1]) < 4) return;   // a click, not a drag (yet)
    M.drag.moved = true;
    if (M.drag.ghost) { const q = mirror(p); p[0] = q[0]; p[1] = q[1]; }
    const dx = p[0] - M.drag.last[0], dy = p[1] - M.drag.last[1]; M.drag.last = p;
    if (M.drag.kind === 'end') { const other = M.drag.end === 'a' ? s.b : s.a; s[M.drag.end] = clampWall(other, p); }
    else { s.a = [s.a[0] + dx, s.a[1] + dy]; s.b = [(s.b || s.a)[0] + dx, (s.b || s.a)[1] + dy]; }
    touch(); render();
  }
  function onUp() {
    if (M.drag && M.drag.kind === 'newwall') {
      const { a, b } = M.drag;
      if (Math.hypot(b[0] - a[0], b[1] - a[1]) >= 30000) { const k = newMark('wall', a, b); M.marks.push(k); M.sel = k.id; touch(); }
    }
    // a click (no drag) on the already selected plan where several overlap: select the next one underneath
    if (M.drag && M.drag.hits && !M.drag.moved && M.drag.wasSel && M.drag.hits.length > 1) {
      const ids = M.drag.hits.map(h => h.id).filter((v, i, a) => a.indexOf(v) === i);
      M.sel = ids[(ids.indexOf(M.sel) + 1) % ids.length];
    }
    if (M.drag) { M.drag = null; render(); }
  }

  function wire() {
    const root = $('#mapEditor'); if (!root || root.dataset.wired) return; root.dataset.wired = '1';
    root.addEventListener('pointerdown', ev => { if (ev.target.closest('#meMap')) onDown(ev); });
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
    root.addEventListener('click', ev => {
      const t = ev.target, tb = t.closest('[data-tool-id]'), cp = t.closest('[data-copy]');
      if (t.dataset.tool) { M.tool = t.dataset.tool; M.toolId = null; render(); }
      else if (tb) {
        const x = CHAMPS.flatMap(c => c.tools).find(v => v.id === tb.dataset.toolId);
        M.tool = x.kind; M.toolId = x.id; Object.assign(M.def, { fake: false }, x.set); render();
      } else if (cp) {
        const from = cp.dataset.copy;
        const src = M.marks.filter(k => compKey(k.comp || []) === from);
        for (const k of src) { const c = JSON.parse(JSON.stringify(k)); c.id = uid(); c.comp = [...M.comp].sort(); M.marks.push(c); }
        touch(); M.status = `Copied ${src.length} plan${src.length === 1 ? '' : 's'}.`; render();
      }
      else if (t.dataset.act === 'save') save();
      else if (t.dataset.act === 'del') { M.marks = M.marks.filter(k => k.id !== M.sel); M.sel = null; touch(); render(); }
      else if (t.dataset.act === 'dup') {
        const s = selected(); if (!s) return;
        const c = JSON.parse(JSON.stringify(s)); c.id = uid(); c.a = [c.a[0] + 20000, c.a[1] + 20000]; c.b = [(c.b || s.a)[0] + 20000, (c.b || s.a)[1] + 20000];
        M.marks.push(c); M.sel = c.id; touch(); render();
      }
      else if (t.closest('[data-sit-col]')) { const k = t.closest('[data-sit-col]').dataset.sitCol; M.collapsed[k] = !M.collapsed[k]; render(); }
      else if (t.closest('[data-pick]')) { const id = t.closest('[data-pick]').dataset.pick; M.sel = M.sel === id ? null : id; M.tool = 'select'; render(); }
    });
    root.addEventListener('change', ev => {
      const t = ev.target;
      if (t.dataset.comp) {
        M.comp = CHAMPS.map(c => c.key).filter(k => k === t.dataset.comp ? t.checked : M.comp.includes(k));
        M.sel = null; if (M.tool !== 'select' && !M.comp.includes(KIND_CHAMP[M.tool])) M.tool = 'select';
        render(); return;
      }
      if (t.dataset.markOn) { const k = M.marks.find(x => x.id === t.dataset.markOn); if (k) { k.off = !t.checked; touch(); render(); } return; }
      if (t.dataset.sitOn) { M.marks.filter(k => inComp(k) && k.trigger === t.dataset.sitOn).forEach(k => { k.off = !t.checked; }); touch(); render(); return; }
      if (t.dataset.def) { M.def[t.dataset.def] = t.dataset.def === 'prio' ? +t.value : t.value; render(); }
      else if (t.dataset.layer) { M.layers[t.dataset.layer] = t.checked; render(); }
      else if (t.hasAttribute('data-flip')) { M.flipY = t.checked; touch(); render(); }
      else if (t.dataset.f) {
        const s = selected(); if (!s) return;
        s[t.dataset.f] = t.dataset.f === 'prio' ? +t.value : t.dataset.f === 'fake' ? t.checked : t.value;
        touch(); render();
      }
    });
    root.addEventListener('input', ev => {
      const t = ev.target; if (!t.hasAttribute('data-q')) return;
      M.q = t.value; const pos = t.selectionStart; render();
      const n = root.querySelector('[data-q]'); if (n) { n.focus(); n.setSelectionRange(pos, pos); }
    });
    document.addEventListener('keydown', ev => {
      if (!$('#tab-map') || $('#tab-map').hidden) return;
      if (ev.key === 'Escape' && M.tool !== 'select') { M.tool = 'select'; M.toolId = null; render(); return; }
      if (/INPUT|SELECT|TEXTAREA/.test((document.activeElement || {}).tagName || '')) return;
      if ((ev.key === 'Delete' || ev.key === 'Backspace') && M.sel) { M.marks = M.marks.filter(k => k.id !== M.sel); M.sel = null; touch(); render(); ev.preventDefault(); }
    });
  }

  window.TFM2Map = {
    show() { wire(); if (!M.dirty) { M.loaded = false; } render(); },
    _debug: M, _txt: txt,
  };
})();
