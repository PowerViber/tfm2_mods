/* TFM2 Database Editor — UI */
(() => {
  'use strict';
  const C = window.TFM2Core;
  const $ = s => document.querySelector(s);
  const $$ = s => [...document.querySelectorAll(s)];
  const esc = s => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

  // ---------------------------------------------------------------- metadata
  const CORE = ['last_hit', 'skill_avoid', 'skill_hit', 'positioning', 'control_speed', 'concentration', 'mental', 'judgement'];
  const PERSONALITY = ['order', 'roaming', 'aggressive', 'ego'];
  const POSITIONS = ['top', 'jungle', 'mid', 'bottom', 'support'];
  const POS_SHORT = { top: 'TOP', jungle: 'JGL', mid: 'MID', bottom: 'BOT', support: 'SUP' };
  const HIDDEN = C.HIDDEN_FIELDS;
  const FACE = C.FACE_FIELDS.map(f => 'face_' + f);
  const LABEL = {
    name: 'Name', age: 'Age',
    last_hit: 'Last hit', skill_avoid: 'Skill dodge', skill_hit: 'Skill accuracy', positioning: 'Positioning',
    control_speed: 'Control speed', concentration: 'Concentration', mental: 'Mental', judgement: 'Judgement',
    order: 'Order', roaming: 'Roaming', aggressive: 'Aggressive', ego: 'Ego',
    top: 'Top', jungle: 'Jungle', mid: 'Mid', bottom: 'Bottom', support: 'Support',
    potential: 'Potential', stamina_recovery_min: 'Stamina recovery (min)', stamina_recovery_max: 'Stamina recovery (max)',
    stamina_cost_per_set_min: 'Stamina cost / set (min)', stamina_cost_per_set_max: 'Stamina cost / set (max)',
    stress_sensitivity: 'Stress sensitivity', condition_baseline: 'Condition baseline', condition_amplitude: 'Condition amplitude',
    condition_period: 'Condition period', condition_phase: 'Condition phase', match_impact_sensitivity: 'Match impact sensitivity',
    contract_team: 'Team', contract_start_date: 'Start date', contract_start_time: 'Start time', contract_end_date: 'End date',
    contract_end_time: 'End time', contract_salary: 'Salary', contract_transfer_fee: 'Transfer fee',
    face_hair: 'Hair style', face_hair_color: 'Hair colour', face_face_tattoo: 'Face tattoo', face_glasses: 'Glasses',
    face_necklace: 'Necklace', face_top_color: 'Top colour', face_pants_color: 'Pants colour', face_boots_color: 'Boots colour',
    logo: 'Logo', stadium: 'Stadium', manager: 'Manager', money_1: 'Balance', money_2: 'Transfer budget?', money_3: 'Salary budget?',
  };
  const SHORT = {
    last_hit: 'LH', skill_avoid: 'Dodge', skill_hit: 'Acc', positioning: 'Pos', control_speed: 'Ctrl', concentration: 'Conc',
    mental: 'Ment', judgement: 'Judg', order: 'Ord', roaming: 'Roam', aggressive: 'Aggr', ego: 'Ego',
    potential: 'Pot', stamina_recovery_min: 'StaRec↓', stamina_recovery_max: 'StaRec↑', stamina_cost_per_set_min: 'StaCost↓',
    stamina_cost_per_set_max: 'StaCost↑', stress_sensitivity: 'Stress', condition_baseline: 'CondBase', condition_amplitude: 'CondAmp',
    condition_period: 'CondPer', condition_phase: 'CondPh', match_impact_sensitivity: 'Impact',
    face_hair: 'Hair', face_hair_color: 'HairCol', face_face_tattoo: 'Tattoo', face_glasses: 'Glasses', face_necklace: 'Necklace',
    face_top_color: 'TopCol', face_pants_color: 'PantsCol', face_boots_color: 'BootsCol',
  };
  const CAP100 = new Set([...CORE, ...PERSONALITY, ...POSITIONS, 'potential']);
  const BULK_FIELDS = [...CORE, ...PERSONALITY, ...POSITIONS, ...HIDDEN, 'age', 'contract_salary', 'contract_transfer_fee'];

  // ---------------------------------------------------------------- state
  const S = {
    server: false, roots: [], src: null, header: null, kind: 0, payload: null, gameDate: null,
    athletes: [], teams: [], teamByKey: new Map(), ranges: {},
    view: 'overview', sort: { key: 'core', dir: -1 }, list: [], sel: null, tab: 'players',
  };

  // ---------------------------------------------------------------- helpers
  const orig = (r, k) => (r.fields[k] ? r.fields[k].value : undefined);
  const val = (r, k) => (r.edits && k in r.edits ? r.edits[k] : orig(r, k));
  const isEdited = (r, k) => !!(r.edits && k in r.edits);
  const langs = r => r.langEdits || r.languages;

  function setVal(r, k, v) {
    const f = r.fields[k];
    if (!f) return;
    if (f.type !== 'str') {
      v = Number(v); if (!isFinite(v)) return;
      if (f.type === 'u64') v = Math.max(0, Math.round(v));
      if (CAP100.has(k)) v = Math.min(100, v);
    }
    r.edits = r.edits || {};
    if ((f.type === 'str' ? String(v) === f.value : v === f.value)) delete r.edits[k];
    else r.edits[k] = v;
    if (k === 'name' && r.kind === 'team') refreshTeamOptions();
    onDirty();
  }

  function mainPos(a) {
    let best = null, bv = -1;
    for (const p of POSITIONS) { const v = val(a, p); if (v > bv) { bv = v; best = p; } }
    return POS_SHORT[best];
  }
  const coreAvg = a => Math.round(CORE.reduce((s, k) => s + val(a, k), 0) / CORE.length);
  const teamName = key => { const t = S.teamByKey.get(key); return t ? val(t, 'name') : key == null ? 'Free agent' : '#' + key; };
  const athleteTeam = a => (a.fields.contract_team ? val(a, 'contract_team') : null);
  const fmtMoney = v => (v == null ? '' : Math.round(v).toLocaleString('en-US'));
  const fmtSize = b => (b > 1048576 ? (b / 1048576).toFixed(1) + ' MB' : Math.round(b / 1024) + ' KB');
  const fmtDate = ms => new Date(ms).toLocaleString();

  function countChanges() {
    let n = 0;
    for (const r of S.athletes.concat(S.teams, S.champs || [])) { if (r.edits) n += Object.keys(r.edits).length; if (r.langEdits) n++; }
    n += (S.poolAdd || []).length;
    return n;
  }

  function toast(msg, kind, ms) {
    const t = $('#toast'); t.textContent = msg; t.className = 'toast ' + (kind || ''); t.hidden = false;
    clearTimeout(toast._t); toast._t = setTimeout(() => (t.hidden = true), ms || 3500);
  }
  function busy(title, frac, detail) {
    if (title === false) { $('#busy').hidden = true; return; }
    $('#busy').hidden = false; $('#busyTitle').textContent = title;
    $('#busyBar').style.width = Math.round((frac || 0) * 100) + '%';
    $('#busyDetail').textContent = detail || '';
  }
  const tick = () => new Promise(r => setTimeout(r, 0));

  async function streamThrough(u8, stream) {
    const s = new Blob([u8]).stream().pipeThrough(stream);
    return new Uint8Array(await new Response(s).arrayBuffer());
  }
  const gunzip = u8 => streamThrough(u8, new DecompressionStream('gzip'));
  const gzip = u8 => streamThrough(u8, new CompressionStream('gzip'));

  // ---------------------------------------------------------------- loading
  function fileTypeOf(name, kind) {
    if (/^save_.*\.data$/i.test(name)) return 'Career save';
    if (/^custom_database/i.test(name)) return 'Active custom DB';
    if (kind === 1) return 'DB export';
    return 'Database';
  }

  async function loadBytes(bytes, src, keepUi) {
    busy('Reading ' + src.name, 0.02, 'Checking header…'); await tick();
    const cont = C.parseContainer(bytes);
    if (C.crc32(cont.gz) !== cont.crc) console.warn('CRC mismatch in source file (continuing).');
    busy('Reading ' + src.name, 0.08, 'Decompressing ' + fmtSize(cont.gz.length) + '…'); await tick();
    const payload = await gunzip(cont.gz);
    busy('Reading ' + src.name, 0.2, 'Scanning ' + fmtSize(payload.length) + ' of game data…');
    const db = await C.scanDatabase(payload, f => {
      $('#busyBar').style.width = Math.round((0.2 + f * 0.78) * 100) + '%';
    });
    if (!db.athletes.length && !db.teams.length) throw new Error('No players or teams found — is this a TFM2 database or save file?');

    let champScan = null;
    if (G && window.TFM2Champions) {
      busy('Reading ' + src.name, 0.98, 'Finding champion numbers…'); await tick();
      try { champScan = window.TFM2Champions.scanChampions(payload, G.info, C.scanStrings); } catch (e) { console.warn('champion scan failed', e); }
    }
    Object.assign(S, { src, header: cont.header, kind: cont.kind, timestamp: cont.timestamp, payload, gameDate: C.readGameDate(payload) });
    const prevChamp = S.champSel && S.champSel.id;
    S.champs = champScan ? buildChampRecords(champScan).concat(buildJsonChampRecords(champScan)) : [];
    S.champSel = S.champs.find(c => c.id === prevChamp) || S.champs[0] || null;
    S.champScan = champScan;
    // champion release list (which champions this career has unlocked so far)
    S.poolAdd = [];
    S.poolAll = G ? Object.keys(G.info).concat(((champScan && champScan.modChampions) || []).map(m => m.id)) : [];
    S.pool = null;
    try { if (S.poolAll.length) S.pool = C.findChampionPool(payload, new Set(S.poolAll)); } catch (e) { console.warn('champion pool scan failed', e); }
    $('#cntChamps').textContent = S.champs.length || '—';
    $('#champMissing').hidden = !!S.champs.length;
    $('#importTip').hidden = cont.kind !== 1;
    S.athletes = db.athletes.map(a => Object.assign(a, { kind: 'athlete', edits: null, langEdits: null }));
    S.teams = db.teams.map(t => Object.assign(t, { kind: 'team', edits: null }));
    S.teamByKey = new Map(S.teams.map(t => [t.key, t]));
    computeRanges();
    busy(false);

    $('#welcome').hidden = true; $('#workspace').hidden = false; $('#fileChip').hidden = false;
    $$('.tab').forEach(t => { t.hidden = false; });
    $('#fileType').textContent = fileTypeOf(src.name, cont.kind);
    $('#fileName').textContent = src.name; $('#fileName').title = src.path || src.name;
    $('#fileMeta').textContent = [S.gameDate && 'in-game ' + S.gameDate.slice(0, 10), fmtSize(bytes.length)].filter(Boolean).join(' · ');
    $('#btnSave').disabled = false; $('#btnSaveAs').disabled = false;
    $('#cntPlayers').textContent = S.athletes.length; $('#cntTeams').textContent = S.teams.length;
    refreshTeamOptions();
    if (!keepUi) { S.sel = null; $('#drawer').hidden = true; }
    else if (S.sel) S.sel = S.athletes.find(a => a.key === S.sel.key) || null;
    onDirty(); applyFilters(); renderTeams();
    if (S.sel) openDrawer(S.sel);
  }

  function computeRanges() {
    const r = {};
    for (const a of S.athletes) for (const k in a.fields) {
      const f = a.fields[k]; if (f.type === 'str') continue;
      const x = r[k] || (r[k] = { min: Infinity, max: -Infinity });
      if (f.value < x.min) x.min = f.value; if (f.value > x.max) x.max = f.value;
    }
    S.ranges = r;
  }
  function sliderMax(k) {
    if ([...CORE, ...PERSONALITY, ...POSITIONS, 'potential'].includes(k)) return 100;
    const x = S.ranges[k]; if (!x) return 100;
    return Math.max(k.startsWith('face_') ? x.max : 100, x.max);
  }

  async function openServerFile(file) {
    try {
      busy('Loading ' + file.name, 0.01, 'Reading from disk…');
      const res = await fetch('/api/file?path=' + encodeURIComponent(file.path));
      if (!res.ok) throw new Error((await res.json()).error || res.statusText);
      const bytes = new Uint8Array(await res.arrayBuffer());
      await loadBytes(bytes, { mode: 'server', path: file.path, name: file.name, dir: file.path.replace(/[\\/][^\\/]*$/, '') });
    } catch (e) { busy(false); toast('Could not open: ' + e.message, 'err', 6000); console.error(e); }
  }

  async function openLocal() {
    try {
      if (window.showOpenFilePicker) {
        const [handle] = await window.showOpenFilePicker({ types: [{ description: 'TFM2 files', accept: { 'application/octet-stream': ['.tfm2db', '.data', '.bak'] } }] });
        const file = await handle.getFile();
        await loadBytes(new Uint8Array(await file.arrayBuffer()), { mode: 'handle', handle, name: file.name });
      } else $('#fileInput').click();
    } catch (e) { if (e.name !== 'AbortError') { busy(false); toast('Could not open: ' + e.message, 'err', 6000); console.error(e); } }
  }
  $('#fileInput').addEventListener('change', async e => {
    const file = e.target.files[0]; if (!file) return;
    try { await loadBytes(new Uint8Array(await file.arrayBuffer()), { mode: 'blob', name: file.name }); }
    catch (err) { busy(false); toast('Could not open: ' + err.message, 'err', 6000); }
    e.target.value = '';
  });

  // ---------------------------------------------------------------- file lists
  function fileListHTML(roots) {
    return roots.map(r => `
      <div class="file-group">
        <h3>${esc(r.label)}</h3>
        <div class="dir">${esc(r.dir)}</div>
        <div class="file-list">${r.files.length ? r.files.map((f, i) => `
          <button type="button" class="file-item" data-root="${esc(r.id)}" data-i="${i}">
            <span class="t ${f.type === 'Backup' ? 'bak' : ''}">${esc(f.type)}</span>
            <span class="n">${esc(f.name)}</span>
            <span class="m">${fmtSize(f.size)}</span>
            <span class="m">${esc(fmtDate(f.mtime))}</span>
          </button>`).join('') : `<div class="empty">${r.exists ? 'No files here yet.' : 'Folder not found.'}</div>`}
        </div>
      </div>`).join('');
  }
  function bindFileList(container, after) {
    container.querySelectorAll('.file-item').forEach(b => b.addEventListener('click', () => {
      const r = S.roots.find(x => x.id === b.dataset.root); const f = r.files[+b.dataset.i];
      if (f.type === 'Backup') return toast('Backups are read-only here — copy it over the original in Explorer if you want to restore it.');
      if (after) after();
      if (countChanges() && !confirm('Discard your unsaved changes?')) return;
      openServerFile(f);
    }));
  }
  async function refreshServer() {
    try {
      const res = await fetch('/api/files', { cache: 'no-store' });
      if (!res.ok) throw 0;
      const j = await res.json();
      S.server = true; S.roots = j.roots;
      $('#gameBanner').hidden = !j.gameRunning;
      return true;
    } catch (e) { S.server = false; return false; }
  }

  async function initWelcome() {
    const ok = location.protocol.startsWith('http') && await refreshServer();
    if (ok) {
      $('#fileGroups').innerHTML = fileListHTML(S.roots);
      bindFileList($('#fileGroups'));
      $('#localHint').textContent = 'Or pick any .tfm2db / .data file from disk.';
      setInterval(async () => {
        try { const j = await (await fetch('/api/status')).json(); $('#gameBanner').hidden = !j.gameRunning; } catch (e) { }
      }, 10000);
    } else {
      $('#fileGroups').innerHTML = `<div class="file-group"><h3>Standalone mode</h3>
        <p class="muted">Run <code>Start Editor.bat</code> to list your saves automatically and save with backups.
        Without it, open a file manually — the game keeps saves in <code>%APPDATA%\\TeamSamoyed\\TeamfightManager2\\data</code>
        (paste that into the file picker's address bar).</p></div>`;
      $('#localHint').textContent = window.showOpenFilePicker ? 'Edits can be saved straight back to the file.' : 'Edits will be downloaded as a new file.';
    }
  }

  $('#btnLocal').addEventListener('click', openLocal);
  $('#btnOpenSave').addEventListener('click', async () => {
    if (!S.server) return toast('Open the save from %APPDATA%\\TeamSamoyed\\TeamfightManager2\\data with "Open…".');
    await refreshServer();
    const saves = S.roots.flatMap(r => r.files).filter(f => f.type === 'Career save').sort((x, y) => y.mtime - x.mtime);
    if (!saves.length) return toast('No career save found.', 'err');
    if (countChanges() && !confirm('Discard your unsaved changes?')) return;
    openServerFile(saves[0]);
  });
  $('#btnOpen').addEventListener('click', async () => {
    if (!S.server) { if (countChanges() && !confirm('Discard your unsaved changes?')) return; return openLocal(); }
    await refreshServer();
    $('#openGroups').innerHTML = fileListHTML(S.roots);
    bindFileList($('#openGroups'), () => $('#openDlg').close());
    $('#openDlg').showModal();
  });
  $('#dlgLocal').addEventListener('click', () => {
    $('#openDlg').close();
    if (countChanges() && !confirm('Discard your unsaved changes?')) return;
    openLocal();
  });

  // ---------------------------------------------------------------- dirty state
  function onDirty() {
    const n = countChanges();
    $('#dirtyBadge').hidden = !n; $('#dirtyBadge').textContent = n + ' unsaved change' + (n === 1 ? '' : 's');
    $('#cntChanges').textContent = n || '';
    if (S.tab === 'changes') renderChanges();
  }
  window.addEventListener('beforeunload', e => { if (countChanges() || (window.TFM2Skills && window.TFM2Skills.isDirty())) { e.preventDefault(); e.returnValue = ''; } });

  // ---------------------------------------------------------------- players grid
  const VIEWS = {
    overview: [
      { k: 'name', label: 'Player', type: 'name' }, { k: 'team', label: 'Team', type: 'team' },
      { k: 'pos', label: 'Pos', type: 'pos' }, { k: 'age', label: 'Age', type: 'num', edit: true },
      { k: 'core', label: 'Core avg', type: 'bar' }, { k: 'potential', label: 'Potential', type: 'num', edit: true },
      { k: 'contract_salary', label: 'Salary', type: 'money' }, { k: 'contract_end_date', label: 'Contract ends', type: 'text' },
      { k: 'langs', label: 'Languages', type: 'langs' },
    ],
    skills: [{ k: 'name', label: 'Player', type: 'name' }, { k: 'pos', label: 'Pos', type: 'pos' }, { k: 'core', label: 'Avg', type: 'num' },
      ...CORE.concat(PERSONALITY).map(k => ({ k, label: SHORT[k], title: LABEL[k], type: 'num', edit: true }))],
    positions: [{ k: 'name', label: 'Player', type: 'name' }, { k: 'team', label: 'Team', type: 'team' }, { k: 'pos', label: 'Main', type: 'pos' },
      ...POSITIONS.map(k => ({ k, label: LABEL[k], type: 'num', edit: true }))],
    hidden: [{ k: 'name', label: 'Player', type: 'name' }, { k: 'pos', label: 'Pos', type: 'pos' },
      ...HIDDEN.map(k => ({ k, label: SHORT[k], title: LABEL[k], type: 'num', edit: true }))],
    contract: [{ k: 'name', label: 'Player', type: 'name' }, { k: 'team', label: 'Team', type: 'team' },
      { k: 'contract_start_date', label: 'Start', type: 'date', edit: true }, { k: 'contract_end_date', label: 'End', type: 'date', edit: true },
      { k: 'contract_salary', label: 'Salary', type: 'money', edit: true }, { k: 'contract_transfer_fee', label: 'Transfer fee', type: 'money', edit: true }],
    looks: [{ k: 'name', label: 'Player', type: 'name' }, { k: 'age', label: 'Age', type: 'num', edit: true },
      ...FACE.map(k => ({ k, label: SHORT[k], title: LABEL[k], type: 'num', edit: true }))],
  };

  function sortValue(a, k) {
    switch (k) {
      case 'name': return val(a, 'name').toLowerCase();
      case 'team': return teamName(athleteTeam(a)).toLowerCase();
      case 'pos': return mainPos(a);
      case 'core': return coreAvg(a);
      case 'langs': return langs(a).length;
      default: { const v = val(a, k); return v == null ? -Infinity : v; }
    }
  }

  function applyFilters() {
    const q = $('#q').value.trim().toLowerCase();
    const ft = $('#fTeam').value, fp = $('#fPos').value;
    let list = S.athletes.filter(a => {
      if (q && !val(a, 'name').toLowerCase().includes(q)) return false;
      if (ft !== '') { const t = athleteTeam(a); if (ft === 'fa' ? t != null : String(t) !== ft) return false; }
      if (fp && mainPos(a) !== fp) return false;
      return true;
    });
    const { key, dir } = S.sort;
    list.sort((x, y) => { const a = sortValue(x, key), b = sortValue(y, key); return (a < b ? -1 : a > b ? 1 : 0) * dir || x.key - y.key; });
    S.list = list;
    $('#shownCount').textContent = list.length === S.athletes.length ? `${list.length} players` : `${list.length} of ${S.athletes.length} players`;
    renderHead(); $('#gridWrap').scrollTop = 0; renderRows();
  }

  function renderHead() {
    const cols = VIEWS[S.view];
    $('#gridHead').innerHTML = '<tr>' + cols.map(c => {
      const num = ['num', 'bar', 'money'].includes(c.type);
      const sorted = S.sort.key === c.k;
      return `<th data-k="${c.k}" class="${num ? 'num' : ''} ${sorted ? 'sorted' : ''}" title="${esc(c.title || c.label)}">${esc(c.label)}${sorted ? `<span class="arrow">${S.sort.dir > 0 ? '▲' : '▼'}</span>` : ''}</th>`;
    }).join('') + '</tr>';
  }
  $('#gridHead').addEventListener('click', e => {
    const th = e.target.closest('th'); if (!th) return;
    const k = th.dataset.k;
    S.sort = { key: k, dir: S.sort.key === k ? -S.sort.dir : (['name', 'team', 'pos'].includes(k) ? 1 : -1) };
    applyFilters();
  });

  function cellHTML(a, c) {
    const ed = c.k in (a.edits || {}) || (c.k === 'langs' && a.langEdits) || (c.k === 'name' && isEdited(a, 'name'));
    const cls = ed ? ' edited' : '';
    switch (c.type) {
      case 'name': return `<td class="name${cls}" data-open="${a.key}" title="${esc(val(a, 'name'))}">${esc(val(a, 'name'))}</td>`;
      case 'team': return `<td class="team-cell${isEdited(a, 'contract_team') ? ' edited' : ''}">${esc(teamName(athleteTeam(a)))}</td>`;
      case 'pos': { const p = mainPos(a); return `<td><span class="pos ${p}">${p}</span></td>`; }
      case 'bar': { const v = coreAvg(a); return `<td class="num"><div class="bar-cell">${v}<span class="mini"><i style="width:${v}%"></i></span></div></td>`; }
      case 'langs': return `<td class="${cls}">${langs(a).map(([id, lv]) => `L${id}${lv < 100 ? ' <span class="muted">' + lv + '</span>' : ''}`).join(', ')}</td>`;
      case 'num': case 'money': case 'date': case 'text': {
        const f = a.fields[c.k];
        if (!f && c.k !== 'core') return '<td class="muted">—</td>';
        const v = c.k === 'core' ? coreAvg(a) : val(a, c.k);
        if (!c.edit) return `<td class="${c.type === 'text' ? '' : 'num'}${cls}">${c.type === 'money' ? fmtMoney(v) : esc(v)}</td>`;
        const shown = c.type === 'money' ? Math.round(v) : v;
        const extra = c.type === 'money' ? ' money' : c.type === 'date' ? ' wide' : '';
        return `<td class="${c.type === 'date' ? '' : 'num'}${cls}"><input class="cell${extra}" data-a="${a.key}" data-f="${c.k}" value="${esc(shown)}" ${c.type === 'date' ? 'maxlength="10"' : 'inputmode="decimal"'}></td>`;
      }
    }
    return '<td></td>';
  }

  const ROW_H = 35, BUF = 12;
  function renderRows() {
    const wrap = $('#gridWrap');
    const cols = VIEWS[S.view];
    const n = S.list.length;
    const first = Math.max(0, Math.floor(wrap.scrollTop / ROW_H) - BUF);
    const last = Math.min(n, first + Math.ceil(wrap.clientHeight / ROW_H) + BUF * 2);
    let html = `<tr class="spacer-row"><td colspan="${cols.length}" style="height:${first * ROW_H}px"></td></tr>`;
    for (let i = first; i < last; i++) {
      const a = S.list[i];
      html += `<tr style="height:${ROW_H}px">` + cols.map(c => cellHTML(a, c)).join('') + '</tr>';
    }
    html += `<tr class="spacer-row"><td colspan="${cols.length}" style="height:${(n - last) * ROW_H}px"></td></tr>`;
    // keep focus on an input across re-render
    const act = document.activeElement;
    const focusKey = act && act.classList.contains('cell') && act.dataset.a != null ? act.dataset.a + '|' + act.dataset.f : null;
    $('#gridBody').innerHTML = html;
    if (focusKey) { const [ak, fk] = focusKey.split('|'); const el = $(`#gridBody input[data-a="${ak}"][data-f="${fk}"]`); if (el) { el.focus(); } }
  }
  let rafPending = false;
  $('#gridWrap').addEventListener('scroll', () => {
    if (rafPending) return; rafPending = true;
    requestAnimationFrame(() => { rafPending = false; if (!document.activeElement || !document.activeElement.classList.contains('cell')) renderRows(); else renderRows(); });
  });
  window.addEventListener('resize', () => S.athletes.length && renderRows());

  const athleteByKey = k => S.athletes.find(a => a.key === +k);
  $('#gridBody').addEventListener('change', e => {
    const inp = e.target.closest('input.cell'); if (!inp) return;
    const a = athleteByKey(inp.dataset.a), k = inp.dataset.f, f = a.fields[k];
    let v = inp.value.trim();
    if (f.type === 'str') {
      if (k.endsWith('_date') && !/^\d{4}-\d{2}-\d{2}$/.test(v)) { toast('Dates must look like 2028-12-31', 'err'); inp.value = val(a, k); return; }
    } else {
      v = Number(v.replace(/[, _]/g, ''));
      if (!isFinite(v) || v < 0) { toast('Please enter a number ≥ 0', 'err'); inp.value = val(a, k); return; }
    }
    setVal(a, k, v);
    renderRows();
    if (S.sel === a) openDrawer(a);
  });
  $('#gridBody').addEventListener('keydown', e => {
    const inp = e.target.closest('input.cell'); if (!inp) return;
    if (e.key === 'Enter' || e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const f = inp.dataset.f, i = S.list.findIndex(x => x.key === +inp.dataset.a);
      const nxt = S.list[i + (e.key === 'ArrowUp' ? -1 : 1)];
      inp.dispatchEvent(new Event('change', { bubbles: true }));
      if (!nxt) return;
      let el = $(`#gridBody input[data-a="${nxt.key}"][data-f="${f}"]`);
      if (!el) { $('#gridWrap').scrollTop += e.key === 'ArrowUp' ? -ROW_H : ROW_H; renderRows(); el = $(`#gridBody input[data-a="${nxt.key}"][data-f="${f}"]`); }
      if (el) { el.focus(); el.select(); el.scrollIntoView({ block: 'nearest' }); }
    }
  });
  $('#gridBody').addEventListener('click', e => {
    const td = e.target.closest('[data-open]'); if (!td) return;
    openDrawer(athleteByKey(td.dataset.open));
  });

  $('#q').addEventListener('input', applyFilters);
  $('#fTeam').addEventListener('change', applyFilters);
  $('#fPos').addEventListener('change', applyFilters);
  $('#viewSeg').addEventListener('click', e => {
    const b = e.target.closest('button'); if (!b) return;
    $$('#viewSeg button').forEach(x => x.classList.toggle('on', x === b));
    S.view = b.dataset.view;
    if (!VIEWS[S.view].some(c => c.k === S.sort.key)) S.sort = { key: 'name', dir: 1 };
    renderHead(); renderRows();
  });

  function refreshTeamOptions() {
    const cur = $('#fTeam').value;
    const opts = S.teams.slice().sort((a, b) => val(a, 'name').localeCompare(val(b, 'name')))
      .map(t => `<option value="${t.key}">${esc(val(t, 'name'))}</option>`).join('');
    $('#fTeam').innerHTML = `<option value="">All teams</option><option value="fa">Free agents</option>${opts}`;
    $('#fTeam').value = cur;
  }

  // ---------------------------------------------------------------- drawer
  const tierColor = (v, max) => {
    const r = max ? v / max : 0;
    return r >= 0.9 ? '#37d5b3' : r >= 0.75 ? '#7bd389' : r >= 0.6 ? '#e6cf5c' : r >= 0.4 ? '#f5a45b' : '#ef6a6a';
  };
  function statRow(a, k, opts = {}) {
    const f = a.fields[k]; if (!f) return '';
    const v = val(a, k), o = f.value, ed = isEdited(a, k);
    const max = Math.max(opts.max != null ? opts.max : sliderMax(k), v, o);
    const d = v - o;
    return `<div class="stat-row${ed ? ' edited' : ''}" data-row="${k}" data-max="${max}">
      <div class="stat-top">
        <span class="stat-name">${esc(LABEL[k] || k)}<small class="was" ${ed ? '' : 'hidden'}>was ${o}</small></span>
        <span class="delta ${d > 0 ? 'up' : 'down'}" ${d ? '' : 'hidden'}>${d > 0 ? '+' : ''}${d}</span>
        <span class="stepper"><button type="button" data-step="-1" title="−1 (Shift: −5)">−</button><input type="number" min="0" data-f="${k}" value="${v}"><button type="button" data-step="1" title="+1 (Shift: +5)">+</button></span>
      </div>
      <div class="track" style="--f:${v / max};--of:${o / max};--c:${opts.neutral ? 'var(--accent)' : tierColor(v, max)}">
        <input type="range" min="0" max="${max}" step="1" data-f="${k}" value="${v}" aria-label="${esc(LABEL[k] || k)}">
        ${ed ? `<span class="orig-mark" title="original ${o}"></span>` : ''}
      </div>
    </div>`;
  }
  function textRow(a, k, opts = {}) {
    const f = a.fields[k]; if (!f) return '';
    const v = val(a, k); const ed = isEdited(a, k) ? ' edited' : '';
    const shown = f.type === 'f64' ? fmtMoney(v) : v;
    return `<div class="text-row${ed}"><label>${esc(LABEL[k] || k)}</label><input type="text" data-f="${k}" ${f.type === 'f64' ? 'inputmode="decimal"' : ''} value="${esc(shown)}" ${opts.maxlength ? `maxlength="${opts.maxlength}"` : ''}></div>`;
  }
  const secHead = (title, note, actions) => `<div class="sec-head"><h4>${title}</h4>${note ? `<span class="note">${note}</span>` : ''}<span class="spacer"></span>${actions || ''}</div>`;
  const groupBtns = g => `<button class="chip-btn" data-gmax="${g}">Max all</button><button class="chip-btn" data-greset="${g}">Reset</button>`;
  const GROUPS = { core: CORE, style: PERSONALITY, pos: POSITIONS, hidden: HIDDEN, face: FACE };

  function openDrawer(a) {
    S.sel = a;
    const d = $('#drawer'); d.hidden = false;
    $('#dName').value = val(a, 'name');
    $('#dName').classList.toggle('edited', isEdited(a, 'name'));
    $('#dSub').innerHTML = `${esc(teamName(athleteTeam(a)))} · <span class="pos ${mainPos(a)}">${mainPos(a)}</span> · age ${val(a, 'age')} · core avg <strong>${coreAvg(a)}</strong> · id ${a.key}`;
    const keepScroll = S._drawerKey === a.key ? $('#dBody').scrollTop : 0; S._drawerKey = a.key;
    const L = langs(a);
    const teamOpts = S.teams.map(t => `<option value="${t.key}" ${t.key === val(a, 'contract_team') ? 'selected' : ''}>${esc(val(t, 'name'))}</option>`).join('');
    $('#dBody').innerHTML = `
      ${secHead('Skills', '', groupBtns('core'))}<div class="stat-grid">${CORE.map(k => statRow(a, k)).join('')}</div>
      ${secHead('Play style', 'tendencies, not strength', '')}<div class="stat-grid">${PERSONALITY.map(k => statRow(a, k, { neutral: true })).join('')}</div>
      ${secHead('Position proficiency', '', groupBtns('pos'))}<div class="stat-grid">${POSITIONS.map(k => statRow(a, k)).join('')}</div>
      ${secHead('Hidden attributes', 'not shown in game', `<button class="chip-btn" data-greset="hidden">Reset</button>`)}<div class="stat-grid">${HIDDEN.map(k => statRow(a, k, { neutral: k !== 'potential' })).join('')}</div>
      ${secHead('General', '', '')}<div class="stat-grid">${statRow(a, 'age', { max: 45, neutral: true })}</div>
      ${secHead('Languages', 'language id · fluency 0–100', '')}
        <div id="langRows">${L.map(([id, lv], i) => `<div class="lang-row">
          <input type="number" min="0" max="255" data-lang="${i}" data-part="0" value="${id}" title="Language id">
          <input type="number" min="0" max="100" data-lang="${i}" data-part="1" value="${lv}" title="Fluency">
          <button class="btn small" data-langdel="${i}" ${L.length < 2 ? 'disabled' : ''}>Remove</button></div>`).join('')}</div>
        <button class="btn small" id="langAdd">Add language</button>
        ${a.langEdits ? '<button class="btn small" id="langReset">Undo language changes</button>' : ''}
      ${secHead('Contract', a.contract ? '' : 'free agent — no contract to edit', '')}
        ${a.contract ? `
        <div class="text-row${isEdited(a, 'contract_team') ? ' edited' : ''}"><label>Team</label><select data-f="contract_team">${teamOpts}</select></div>
        <p class="muted" style="font-size:12.5px;margin:2px 0 8px">Changing the team only rewrites the contract; the team's lineup may not follow. Use the game for real transfers.</p>
        ${textRow(a, 'contract_start_date', { maxlength: 10 })}${textRow(a, 'contract_end_date', { maxlength: 10 })}
        ${textRow(a, 'contract_salary')}${textRow(a, 'contract_transfer_fee')}` : ''}
      ${secHead('Appearance', '', `<button class="chip-btn" data-greset="face">Reset</button>`)}<div class="stat-grid">${FACE.map(k => statRow(a, k, { neutral: true })).join('')}</div>
      <div class="sec" style="margin-top:22px"><button class="btn danger small" id="dRevert" ${a.edits && Object.keys(a.edits).length || a.langEdits ? '' : 'disabled'}>Revert this player</button></div>`;
    $('#dBody').scrollTop = keepScroll;
  }

  // live-update one stat row without rebuilding the drawer (keeps slider drag smooth)
  function paintRow(a, k) {
    const row = $(`#dBody .stat-row[data-row="${k}"]`); if (!row) return;
    const f = a.fields[k], v = val(a, k), o = f.value, max = Math.max(+row.dataset.max, v, o), d = v - o;
    row.classList.toggle('edited', isEdited(a, k));
    const tr = row.querySelector('.track'), rg = tr.querySelector('input');
    if (+rg.max < max) { rg.max = max; row.dataset.max = max; }
    tr.style.setProperty('--f', v / max); tr.style.setProperty('--of', o / max);
    if (!tr.style.getPropertyValue('--c').includes('accent')) tr.style.setProperty('--c', tierColor(v, max));
    if (document.activeElement !== rg) rg.value = v;
    const num = row.querySelector('.stepper input'); if (document.activeElement !== num) num.value = v;
    const dl = row.querySelector('.delta'); dl.hidden = !d; dl.className = 'delta ' + (d > 0 ? 'up' : 'down'); dl.textContent = (d > 0 ? '+' : '') + d;
    let mk = tr.querySelector('.orig-mark');
    if (isEdited(a, k) && !mk) { mk = document.createElement('span'); mk.className = 'orig-mark'; tr.appendChild(mk); }
    if (mk) { mk.hidden = !isEdited(a, k); mk.title = 'original ' + o; }
    const was = row.querySelector('.was'); if (was) { was.hidden = !isEdited(a, k); was.textContent = 'was ' + o; }
    $('#dSub').innerHTML = `${esc(teamName(athleteTeam(a)))} · <span class="pos ${mainPos(a)}">${mainPos(a)}</span> · age ${val(a, 'age')} · core avg <strong>${coreAvg(a)}</strong> · id ${a.key}`;
  }
  const clampFor = (k, v) => Math.max(0, CAP100.has(k) ? Math.min(100, v) : v);

  $('#dClose').addEventListener('click', () => { $('#drawer').hidden = true; S.sel = null; });
  document.addEventListener('keydown', e => { if (e.key === 'Escape' && !$('#drawer').hidden && !document.querySelector('dialog[open]')) { $('#drawer').hidden = true; S.sel = null; } });
  $('#dName').addEventListener('change', e => {
    const v = e.target.value.trim(); if (!v) { e.target.value = val(S.sel, 'name'); return toast('Name cannot be empty', 'err'); }
    setVal(S.sel, 'name', v); e.target.classList.toggle('edited', isEdited(S.sel, 'name')); renderRows();
  });
  let rowsTimer = 0;
  const renderRowsSoon = () => { clearTimeout(rowsTimer); rowsTimer = setTimeout(renderRows, 120); };
  $('#dBody').addEventListener('input', e => {
    const a = S.sel; const el = e.target;
    if (el.dataset.f && el.type === 'range') { setVal(a, el.dataset.f, +el.value); paintRow(a, el.dataset.f); renderRowsSoon(); }
  });
  $('#dBody').addEventListener('change', e => {
    const a = S.sel; const el = e.target;
    if (el.dataset.lang != null) {
      const L = langs(a).map(x => x.slice());
      const v = Math.max(0, Math.min(el.dataset.part === '0' ? 255 : 100, Math.round(+el.value || 0)));
      if (el.dataset.part === '0' && L.some((x, i) => i !== +el.dataset.lang && x[0] === v)) { toast('This player already has language ' + v, 'err'); el.value = L[+el.dataset.lang][0]; return; }
      L[+el.dataset.lang][+el.dataset.part] = v;
      setLangs(a, L); return;
    }
    const k = el.dataset.f; if (!k || el.type === 'range') return;
    const f = a.fields[k];
    if (el.tagName === 'SELECT') { setVal(a, k, +el.value); openDrawer(a); renderRows(); return; }
    if (f.type === 'str') {
      if (/_date$/.test(k) && !/^\d{4}-\d{2}-\d{2}$/.test(el.value)) { toast('Dates must look like 2028-12-31', 'err'); el.value = val(a, k); return; }
      setVal(a, k, el.value);
    } else if (f.type === 'f64') {
      const v = Number(el.value.replace(/[, _]/g, ''));
      if (!isFinite(v) || v < 0) { toast('Please enter a number ≥ 0', 'err'); el.value = fmtMoney(val(a, k)); return; }
      setVal(a, k, v); el.value = fmtMoney(v);
    } else {
      const v = Math.round(+el.value); if (!isFinite(v)) { el.value = val(a, k); return; }
      setVal(a, k, clampFor(k, v)); paintRow(a, k);
    }
    const tr = el.closest('.text-row'); if (tr) tr.classList.toggle('edited', isEdited(a, k));
    renderRows();
  });
  $('#dBody').addEventListener('click', e => {
    const a = S.sel; const t = e.target;
    const step = t.closest('[data-step]');
    if (step) {
      const k = step.closest('.stat-row').dataset.row;
      setVal(a, k, clampFor(k, val(a, k) + (+step.dataset.step) * (e.shiftKey ? 5 : 1))); paintRow(a, k); renderRowsSoon(); return;
    }
    if (t.dataset.gmax) { for (const k of GROUPS[t.dataset.gmax]) setVal(a, k, 100); openDrawer(a); renderRows(); return; }
    if (t.dataset.greset) { for (const k of GROUPS[t.dataset.greset]) if (a.edits) delete a.edits[k]; onDirty(); openDrawer(a); renderRows(); return; }
    if (t.id === 'langAdd') { const L = langs(a).map(x => x.slice()); let id = 0; while (L.some(x => x[0] === id)) id++; L.push([id, 50]); setLangs(a, L); }
    else if (t.dataset.langdel != null) { const L = langs(a).map(x => x.slice()); L.splice(+t.dataset.langdel, 1); setLangs(a, L); }
    else if (t.id === 'langReset') { a.langEdits = null; onDirty(); openDrawer(a); renderRows(); }
    else if (t.id === 'dRevert') { a.edits = null; a.langEdits = null; onDirty(); openDrawer(a); renderRows(); }
  });
  function setLangs(a, L) {
    a.langEdits = JSON.stringify(L) === JSON.stringify(a.languages) ? null : L;
    onDirty(); openDrawer(a); renderRows();
  }

  // ---------------------------------------------------------------- bulk
  $('#btnBulk').addEventListener('click', () => {
    $('#bulkTarget').textContent = `Applies to the ${S.list.length} player${S.list.length === 1 ? '' : 's'} currently shown (use search / filters to narrow it down).`;
    $('#bulkField').innerHTML = [['Skills', CORE], ['Play style', PERSONALITY], ['Positions', POSITIONS], ['Hidden', HIDDEN], ['Other', ['age', 'contract_salary', 'contract_transfer_fee']]]
      .map(([g, ks]) => `<optgroup label="${g}">${ks.map(k => `<option value="${k}">${esc(LABEL[k])}</option>`).join('')}</optgroup>`).join('')
      + '<optgroup label="Groups"><option value="@core">All 8 skills</option><option value="@pos">All 5 positions</option></optgroup>';
    $('#bulkDlg').showModal();
  });
  $('#bulkDlg').addEventListener('close', () => {
    if ($('#bulkDlg').returnValue !== 'apply') return;
    const fk = $('#bulkField').value, op = $('#bulkOp').value, x = Number($('#bulkVal').value);
    if (!isFinite(x)) return toast('Enter a number', 'err');
    const keys = fk === '@core' ? CORE : fk === '@pos' ? POSITIONS : [fk];
    let n = 0;
    for (const a of S.list) for (const k of keys) {
      if (!a.fields[k]) continue;
      const cur = val(a, k); let v = cur;
      if (op === 'set') v = x; else if (op === 'add') v = cur + x; else if (op === 'mul') v = cur * x;
      else if (op === 'min') v = Math.max(cur, x); else if (op === 'max') v = Math.min(cur, x);
      v = Math.max(0, v);
      if (v !== cur) { setVal(a, k, v); n++; }
    }
    renderRows(); if (S.sel) openDrawer(S.sel);
    toast(`Updated ${n} value${n === 1 ? '' : 's'}.`, 'ok');
  });

  // ---------------------------------------------------------------- teams
  function renderTeams() {
    const q = ($('#tq').value || '').trim().toLowerCase();
    const roster = new Map();
    for (const a of S.athletes) { const t = athleteTeam(a); if (t == null) continue; const r = roster.get(t) || roster.set(t, []).get(t); r.push(a); }
    $('#teamHead').innerHTML = `<tr><th>#</th><th>Name</th><th>Logo</th><th>Stadium</th><th>Manager</th>
      <th class="num" title="money_1">Balance</th><th class="num" title="money_2">Transfer budget?</th><th class="num" title="money_3">Salary budget?</th><th class="num">Players</th><th class="num">Core avg</th><th></th></tr>`;
    const inp = (t, k, cls) => t.fields[k]
      ? `<td class="${isEdited(t, k) ? 'edited' : ''}"><input class="cell ${cls}" data-t="${t.key}" data-f="${k}" value="${esc(t.fields[k].type === 'f64' ? fmtMoney(val(t, k)) : val(t, k))}">${t.fields[k].type === 'f64' ? `<span class="money-hint">${(val(t, k) / 1e8).toLocaleString('en-US', { maximumFractionDigits: 1 })} 억</span>` : ''}</td>`
      : '<td class="muted">—</td>';
    $('#teamBody').innerHTML = S.teams.filter(t => !q || val(t, 'name').toLowerCase().includes(q)).map(t => {
      const r = roster.get(t.key) || [];
      const avg = r.length ? Math.round(r.reduce((s, a) => s + coreAvg(a), 0) / r.length) : '—';
      return `<tr><td class="muted">${t.key}</td>${inp(t, 'name', 'wide')}${inp(t, 'logo', '')}${inp(t, 'stadium', 'wide')}${inp(t, 'manager', 'wide')}
        ${inp(t, 'money_1', 'money')}${inp(t, 'money_2', 'money')}${inp(t, 'money_3', 'money')}
        <td class="num">${r.length}</td><td class="num">${avg}</td>
        <td><button class="linkish" data-roster="${t.key}">Players →</button></td></tr>`;
    }).join('');
  }
  $('#tq').addEventListener('input', renderTeams);
  $('#teamBody').addEventListener('change', e => {
    const inp = e.target.closest('input.cell'); if (!inp) return;
    const t = S.teamByKey.get(+inp.dataset.t), k = inp.dataset.f, f = t.fields[k];
    if (f.type === 'f64') {
      const v = Number(inp.value.replace(/[, _]/g, ''));
      if (!isFinite(v) || v < 0) { toast('Please enter a number ≥ 0', 'err'); inp.value = fmtMoney(val(t, k)); return; }
      setVal(t, k, v); inp.value = fmtMoney(v);
      const h = inp.parentElement.querySelector('.money-hint'); if (h) h.textContent = (v / 1e8).toLocaleString('en-US', { maximumFractionDigits: 1 }) + ' 억';
    } else {
      const v = inp.value.trim();
      if (!v) { toast('Cannot be empty', 'err'); inp.value = val(t, k); return; }
      if (k === 'logo' && !/^(\d+_\d+|custom:.+)$/.test(v)) { toast('Logo must look like 3_0 or custom:custom_team_logo/12', 'err'); inp.value = val(t, k); return; }
      setVal(t, k, v);
    }
    inp.closest('td').classList.toggle('edited', isEdited(t, k));
  });
  $('#teamBody').addEventListener('click', e => {
    const b = e.target.closest('[data-roster]'); if (!b) return;
    $('#fTeam').value = b.dataset.roster; $('#q').value = ''; $('#fPos').value = '';
    switchTab('players'); applyFilters();
  });

  // ---------------------------------------------------------------- changes
  function allChanges() {
    const out = [];
    for (const r of S.athletes.concat(S.teams)) {
      const who = (r.kind === 'team' ? 'Team · ' : '') + orig(r, 'name');
      if (r.edits) for (const k in r.edits) {
        const f = r.fields[k];
        const show = v => (f.type === 'f64' ? fmtMoney(v) : k === 'contract_team' ? teamName(v) : v);
        out.push({ r, k, who, what: LABEL[k] || k, old: show(f.value), nu: show(r.edits[k]) });
      }
      if (r.langEdits) out.push({ r, k: '@lang', who, what: 'Languages', old: r.languages.map(x => x.join(':')).join(', '), nu: r.langEdits.map(x => x.join(':')).join(', ') });
    }
    for (const id of S.poolAdd || []) out.push({ r: null, k: '@pool:' + id, who: 'Champion release', what: champName(id), old: 'not released', nu: 'released' });
    for (const x of champChanges()) out.push({ r: x.c, k: x.fk, who: 'Champion · ' + x.c.name, what: (GROUP_LABEL[x.f.group.split('.')[0]] || x.f.group) + ' · ' + x.f.label, old: x.f.value, nu: x.v });
    return out;
  }
  function renderChanges() {
    const ch = allChanges();
    $('#chgTitle').textContent = ch.length ? `${ch.length} unsaved change${ch.length === 1 ? '' : 's'}` : 'No changes yet';
    $('#btnRevertAll').disabled = !ch.length;
    $('#changeList').innerHTML = ch.map((c, i) => `<div class="chg"><span class="who">${esc(c.who)}</span><span class="what">${esc(c.what)}</span>
      <span><span class="old">${esc(c.old)}</span><span class="new">${esc(c.nu)}</span></span>
      <button class="btn small" data-rev="${i}">Revert</button></div>`).join('') || '<p class="muted">Edit players or teams and your changes will be listed here before you save.</p>';
    renderChanges.list = ch;
  }
  $('#changeList').addEventListener('click', e => {
    const b = e.target.closest('[data-rev]'); if (!b) return;
    const c = renderChanges.list[+b.dataset.rev];
    if (c.k.startsWith('@pool:')) { S.poolAdd = S.poolAdd.filter(x => x !== c.k.slice(6)); if (S.champView === 'releases') renderChampDetail(); }
    else if (c.k === '@lang') c.r.langEdits = null; else delete c.r.edits[c.k];
    onDirty(); renderChanges(); renderRows(); renderTeams(); refreshTeamOptions(); if (S.sel) openDrawer(S.sel);
  });
  $('#btnRevertAll').addEventListener('click', () => {
    if (!confirm('Revert every unsaved change?')) return;
    for (const r of S.athletes.concat(S.teams, S.champs)) { r.edits = null; if (r.kind === 'athlete') r.langEdits = null; }
    S.poolAdd = [];
    onDirty(); renderChanges(); renderRows(); renderTeams(); refreshTeamOptions(); if (S.sel) openDrawer(S.sel);
  });

  // ---------------------------------------------------------------- champions
  const G = window.TFM2_GAMEDATA || null;
  const CAT_COLORS = { Melee: '#f5a45b', Range: '#ff8fb1', Magician: '#a98bff', Magic: '#a98bff', Util: '#7bd389', Assassin: '#ef6a6a' };
  const GROUP_LABEL = { stat: 'Base stats', growth: 'Growth per level', attack: 'Basic attack', skill: 'Ability 1', skill1: 'Ability 1', skill2: 'Ability 2', ult: 'Ultimate', general: 'Passive / other' };
  const STAT_LABEL = { attack: 'Attack Damage', magic_power: 'Ability Power', hp: 'Health', defence: 'Armor', magic_resistance: 'Magic Resist', move_speed: 'Movement Speed', hp_regen: 'Health Regen', stack: 'Stacks', crit_chance: 'Crit Chance' };
  const POWER_KEYS = /^(attack|attack_ratio|ap_ratio|magic_ratio|damage|damage_ratio|heal|heal_ratio|heal_amount|shield|shield_ratio|shield_amount|[a-z_]*_damage)$/;
  const TIME_KEY = /(cooltime|duration|_tick$|^tick$|timing|delay|period|interval|^stun$|airborne|_time$)/;
  const REVERSE_KEY = /(cooltime|start_timing|^duration$|cast_time)/;
  S.champs = []; S.champSel = null; S.champMirror = true; S.champView = 'edit';

  const humanize = k => k.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()).replace(/\bAp\b/, 'AP').replace(/\bHp\b/, 'HP');
  function i18n(key) {
    if (!G || !key) return null;
    const [ns, k] = key.split('.');
    if (ns === 'patch_key') return G.patchKeys[k] || null;
    if (ns === 'stat') return G.statLabels[k] || null;
    return null;
  }
  function actionPatch(id, group) {
    if (!G) return {};
    const a = group.split('.')[0];
    const name = a === 'attack' ? [id + '_attack', 'target_attack'] : [id + '_' + a];
    for (const n of name) if (G.patchActions[n]) return G.patchActions[n];
    return {};
  }
  function champFieldLabel(c, f) {
    if (f.group === 'stat' || f.group === 'growth') return STAT_LABEL[f.key] || humanize(f.key);
    const pk = actionPatch(c.id, f.group)[f.key];
    const t = pk && i18n(pk.i18n);
    if (f.key === 'attack') return 'Base Damage';
    if (f.key === 'attack_ratio') return 'Attack Damage Ratio %';
    return t || humanize(f.key);
  }
  function buildChampRecords(ch) {
    return ch.champions.map(c => {
      const fields = {}, order = [];
      for (const f of c.fields) {
        let fk = f.group + '.' + f.key, n = 2; while (fields[fk]) fk = f.group + '.' + f.key + '#' + n++;
        fields[fk] = { type: 'u64', off: f.offCurrent, value: f.current, baseOff: f.offBase, baseValue: f.base, def: f.def, group: f.group, key: f.key };
        order.push(fk);
      }
      const info = G.info[c.id] || {};
      const rec = { kind: 'champ', id: c.id, name: (G.names[c.id] || humanize(c.id)), cat: info.category || '', tags: info.tags || [], fields, order, edits: null };
      for (const fk of order) fields[fk].label = champFieldLabel(rec, fields[fk]);
      return rec;
    });
  }
  // data-driven champions (crossbowman, nightmare, alchemist, sand_mage) live as JSON inside the save
  function jsonLeaves(obj, path, ctx, out) {
    for (const [k, v] of Object.entries(obj)) {
      if (path.length === 0 && ['id', 'sprite', 'category', 'tags', 'anim_prefix'].includes(k)) continue;
      const t = v && typeof v === 'object' && !Array.isArray(v) && typeof v.type === 'string' ? (v.type === 'Native' ? v.effect_ref : v.type) : ctx;
      if (typeof v === 'number') out.push({ path: [...path, k], key: k, value: v, ctx });
      else if (v && typeof v === 'object') jsonLeaves(v, [...path, k], t, out);
    }
    return out;
  }
  const getPath = (o, path) => path.reduce((x, k) => (x == null ? x : x[k]), o);
  const setPath = (o, path, v) => { const last = path[path.length - 1]; const parent = getPath(o, path.slice(0, -1)); if (parent) parent[last] = v; };
  function buildJsonChampRecords(ch) {
    return (ch.modChampions || []).map(m => {
      const def = (G.modChampions || []).find(x => x.id === m.id) || null;
      const fields = {}, order = [];
      for (const leaf of jsonLeaves(m.current.json, [], null, [])) {
        const fk = leaf.path.join('.');
        const baseV = m.base ? getPath(m.base.json, leaf.path) : undefined;
        const defV = def ? getPath(def, leaf.path) : undefined;
        fields[fk] = { type: 'json', path: leaf.path, value: leaf.value, baseValue: typeof baseV === 'number' ? baseV : null,
          def: typeof defV === 'number' ? defV : leaf.value, group: leaf.path[0], key: leaf.key,
          label: (leaf.path[0] === 'stat' || leaf.path[0] === 'growth') ? (STAT_LABEL[leaf.key] || humanize(leaf.key)) : humanize(leaf.key) + (leaf.ctx ? ' · ' + leaf.ctx : '') };
        order.push(fk);
      }
      return { kind: 'champjson', id: m.id, name: (G.names[m.id] || humanize(m.id.replace(/^[a-z0-9]+_custom_/, ''))) , cat: m.current.json.category || '', tags: (m.current.json.tags || []).concat('DATA'), fields, order, edits: null, entry: m };
    });
  }
  function encodeModEntry(head, json) {
    const txt = new TextEncoder().encode(JSON.stringify(json));
    const b = new Uint8Array(16 + txt.length); b.set(head.subarray(0, 8), 0);
    new DataView(b.buffer).setUint32(8, txt.length, true); b.set(txt, 16);
    return b;
  }
  function jsonChampPatches() {
    const out = [];
    for (const c of S.champs) if (c.kind === 'champjson' && c.edits && Object.keys(c.edits).length) {
      const apply = (entry) => { const j = JSON.parse(JSON.stringify(entry.json)); for (const fk in c.edits) setPath(j, c.fields[fk].path, c.edits[fk]); return j; };
      const cur = c.entry.current;
      out.push({ off: cur.off, len: cur.end - cur.off, bytes: encodeModEntry(cur.head, apply(cur)) });
      if (S.champMirror && c.entry.base) { const b = c.entry.base; out.push({ off: b.off, len: b.end - b.off, bytes: encodeModEntry(b.head, apply(b)) }); }
    }
    return out;
  }
  const champEdited = c => !!(c.edits && Object.keys(c.edits).length);
  const niceStep = v => { const x = Math.max(1, Math.abs(v)); const p = Math.pow(10, Math.floor(Math.log10(x)) - 1); return Math.max(1, Math.round(p)); };

  function renderChampList() {
    const q = ($('#cq').value || '').trim().toLowerCase(), cat = $('#cCat').value;
    const list = S.champs.filter(c => (!q || c.name.toLowerCase().includes(q)) && (!cat || c.cat === cat));
    $('#champList').innerHTML = list.map(c => `<button class="champ-item${S.champSel === c ? ' on' : ''}" data-champ="${c.id}">
      ${window.TFM2_ART ? TFM2_ART.icon(c.id, c.name, CAT_COLORS[c.cat]) : `<span class="ico" style="background:${CAT_COLORS[c.cat] || '#8b97a8'}">${esc(c.name.slice(0, 2))}</span>`}
      <span><span class="nm">${esc(c.name)}</span><span class="cat">${esc(c.cat)}${c.tags.length ? ' · ' + esc(c.tags.join(', ')) : ''}</span></span>
      ${champEdited(c) ? '<span class="dot" title="edited"></span>' : '<span></span>'}</button>`).join('') || '<p class="muted" style="padding:14px">No champions match.</p>';
  }

  function numRow(c, fk) {
    const f = c.fields[fk]; const v = val(c, fk), o = f.value, ed = isEdited(c, fk);
    const pct = o ? Math.round((v - o) / o * 100) : (v ? 100 : 0);
    const rev = REVERSE_KEY.test(f.key);
    const good = rev ? v < o : v > o;
    const secs = TIME_KEY.test(f.key) && !/ratio/.test(f.key) ? ` = ${(v / 60).toFixed(2).replace(/\.?0+$/, '')}s` : '';
    const ref = f.def || o || 1, pos = Math.max(0, Math.min(1, v / (ref * 2)));
    return `<div class="num-row${ed ? ' edited' : ''}" data-crow="${fk}">
      <div class="stat-top"><span class="stat-name">${esc(f.label)}<small>${esc(f.key)}${secs}</small></span>
        <span class="delta ${good ? 'up' : 'down'}" ${v === o ? 'hidden' : ''}>${pct > 0 ? '+' : ''}${pct}%</span>
        <span class="stepper"><button type="button" data-cstep="-1">−</button><input type="number" data-cf="${fk}" value="${v}"><button type="button" data-cstep="1">+</button></span></div>
      <div class="ratio-bar"><i style="left:${Math.min(pos, 0.5) * 100}%;width:${Math.abs(pos - 0.5) * 100}%;background:${v === (f.def || o) ? 'transparent' : 'var(--accent)'}"></i><b></b></div>
      <div class="sub"><span>game default ${f.def}</span><span>${ed ? 'was ' + o : f.baseValue != null && f.baseValue !== o ? 'base ' + f.baseValue : ''}</span></div>
    </div>`;
  }

  function renderChampDetail() {
    const c = S.champSel;
    if (S.champView === 'notes') return renderPatchNotes();
    if (S.champView === 'releases') return renderReleases();
    if (!c) { $('#champDetail').innerHTML = '<p class="muted">Pick a champion on the left.</p>'; return; }
    const groups = [];
    for (const fk of c.order) { const g = c.fields[fk].group; if (!groups.includes(g)) groups.push(g); }
    const desc = (G.desc[c.id] || {});
    const keep = S._champKey === c.id ? $('#champDetail').scrollTop : 0; S._champKey = c.id;
    $('#champDetail').innerHTML = `
      <div class="champ-hero">
        ${window.TFM2_ART ? TFM2_ART.hero(c.id, 72) : ''}
        <div><h2>${esc(c.name)}</h2><div style="margin-top:6px"><span class="tag" style="color:${CAT_COLORS[c.cat] || ''}">${esc(c.cat)}</span>${c.tags.map(t => `<span class="tag">${esc(t)}</span>`).join('')}</div></div>
        <span class="spacer"></span>
        <button class="chip-btn" data-cscale="0.9">Nerf −10%</button>
        <button class="chip-btn" data-cscale="1.1">Buff +10%</button>
        <button class="chip-btn" data-cdefault="1">Game defaults</button>
        <button class="chip-btn" data-crevert="1" ${champEdited(c) ? '' : 'disabled'}>Revert</button>
      </div>
      <p class="ability-desc" style="margin-top:10px">Buff/Nerf scales damage, ratios, healing, shields and the base/per-level combat stats.</p>
      ${groups.map(g => {
        const top = g.split('.')[0];
        const d = desc[top === 'skill1' ? 'skill' : top];
        return `${secHead(esc(GROUP_LABEL[top] || humanize(top)) + (g.includes('.') ? ' · ' + esc(humanize(g.split('.').slice(1).join(' '))) : ''), '', '')}
          ${d ? `<p class="ability-desc">${esc(d)}</p>` : ''}
          <div class="num-grid">${c.order.filter(fk => c.fields[fk].group === g).map(fk => numRow(c, fk)).join('')}</div>`;
      }).join('')}`;
    $('#champDetail').scrollTop = keep;
    if (window.TFM2_ART) TFM2_ART.animate($('#champDetail'));
  }

  function champChanges() {
    const out = [];
    for (const c of S.champs) if (c.edits) for (const fk in c.edits) {
      const f = c.fields[fk], v = c.edits[fk];
      const rev = REVERSE_KEY.test(f.key);
      out.push({ c, fk, f, v, buff: rev ? v < f.value : v > f.value });
    }
    return out;
  }
  function renderPatchNotes() {
    const ch = champChanges();
    const by = new Map(); for (const x of ch) { if (!by.has(x.c)) by.set(x.c, []); by.get(x.c).push(x); }
    const adds = S.poolAdd || [];
    $('#champDetail').innerHTML = `<h2 style="margin:0 0 6px">Patch preview</h2>
      <p class="ability-desc">These champion changes are applied when you save${S.champMirror ? ' (to the current numbers and the base numbers, so future auto-patches balance around them)' : ' (to the current numbers only)'}.</p>
      ${adds.length ? `<div class="sec-head"><h4>New champions released</h4><span class="note">${adds.length}</span></div><div class="rel-grid">${adds.map(id => relTile(id, 'pending', false)).join('')}</div>` : ''}
      ${ch.length ? [...by.entries()].map(([c, xs]) => `<div class="sec-head"><h4>${esc(c.name)}</h4><span class="note">${xs.filter(x => x.buff).length} buffs · ${xs.filter(x => !x.buff).length} nerfs</span></div>
        ${xs.map(x => `<div class="patch-note"><span>${esc(GROUP_LABEL[x.f.group.split('.')[0]] || x.f.group)}</span><span>${esc(x.f.label)}: ${x.f.value} → <strong>${x.v}</strong></span><span class="${x.buff ? 'buff' : 'nerf'}">${x.buff ? 'BUFF' : 'NERF'}</span></div>`).join('')}`).join('')
        : adds.length ? '' : '<p class="muted">No champion changes yet.</p>'}`;
  }

  // ---------------------------------------------------------------- champion releases
  function champJson(id) { const m = S.champScan && (S.champScan.modChampions || []).find(x => x.id === id); return m && m.current && m.current.json; }
  function champName(id) {
    const rec = S.champs.find(c => c.id === id);
    return (G && G.names[id]) || (rec && rec.name !== humanize(id) && rec.name) || humanize(id.replace(/^[a-z0-9]+_custom_/, '').replace(/^tfm2_/, ''));
  }
  function champCat(id) { const j = champJson(id); return (G && G.info[id] && G.info[id].category) || (j && j.category) || ''; }
  function relTile(id, state, clickable, order) {
    const j = champJson(id); const cat = champCat(id);
    const kind = G && G.info[id] ? '' : (G && (G.modChampions || []).some(m => m.id === id)) ? 'data' : 'mod';
    const icon = window.TFM2_ART ? TFM2_ART.icon((j && j.sprite) || id, champName(id), CAT_COLORS[cat]) : '';
    return `<${clickable ? 'button' : 'div'} class="rel-tile ${state}"${clickable ? ` data-rel="${esc(id)}"` : ''}>${icon}
      <span><span class="nm">${esc(champName(id))}</span><span class="cat">${esc(cat === 'Magician' ? 'Mage' : cat === 'Util' ? 'Support' : cat === 'Range' ? 'Ranged' : cat)}${kind === 'mod' ? ' · <b class="rel-kind">your mod</b>' : ''}</span></span>
      ${order ? `<span class="rel-no">#${order}</span>` : state === 'pending' ? '<span class="rel-check">✓</span>' : clickable ? '<span class="rel-check off"></span>' : ''}</${clickable ? 'button' : 'div'}>`;
  }
  function renderReleases() {
    const P = S.pool; const el = $('#champDetail');
    if (!P) { el.innerHTML = '<h2 style="margin:0 0 6px">Champion releases</h2><p class="muted">This file has no champion release list (or its layout wasn\'t recognised).</p>'; return; }
    const released = P.ids, adds = S.poolAdd;
    const waiting = S.poolAll.filter(id => !released.includes(id));
    const total = released.length + waiting.length;
    el.innerHTML = `<h2 style="margin:0 0 6px">Champion releases</h2>
      <p class="ability-desc">${S.kind === 1 ? 'This database' : 'This career'} has <strong>${released.length} of ${total}</strong> champions released${P.patches > 1 ? ` after ${P.patches} patches` : ' (no balance patch yet)'}.
        The game releases new champions a few at a time in its automatic patches. Tick the ones you want released now: they're added when you save${S.kind === 1 ? ', and a new career started from this database begins with them' : ''}.</p>
      <div class="rel-bar"><strong>Not released yet · ${waiting.length}</strong><span class="spacer"></span>
        <button class="chip-btn" data-relpick="3">Random 3 (like a patch)</button>
        <button class="chip-btn" data-relpick="all">Select all</button>
        <button class="chip-btn" data-relpick="none" ${adds.length ? '' : 'disabled'}>Clear</button>
        <span class="rel-count">${adds.length ? `${adds.length} selected — released on save` : 'none selected'}</span></div>
      <div class="rel-grid">${waiting.map(id => relTile(id, adds.includes(id) ? 'pending' : 'waiting', true)).join('') || '<p class="muted">Every champion is already released.</p>'}</div>
      <div class="rel-bar"><strong>Released · ${released.length}</strong><span class="note">in release order</span></div>
      <div class="rel-grid">${released.map((id, i) => relTile(id, 'released', false, i + 1)).join('')}</div>`;
  }
  $('#champDetail').addEventListener('click', e => {
    if (S.champView !== 'releases') return;
    const t = e.target.closest('[data-rel],[data-relpick]'); if (!t) return;
    const waiting = S.poolAll.filter(id => !S.pool.ids.includes(id));
    if (t.dataset.rel) { const id = t.dataset.rel; S.poolAdd = S.poolAdd.includes(id) ? S.poolAdd.filter(x => x !== id) : S.poolAdd.concat(id); }
    else if (t.dataset.relpick === 'all') S.poolAdd = waiting.slice();
    else if (t.dataset.relpick === 'none') S.poolAdd = [];
    else { const left = waiting.filter(id => !S.poolAdd.includes(id)); for (let i = 0; i < 3 && left.length; i++) S.poolAdd.push(left.splice(Math.floor(Math.random() * left.length), 1)[0]); }
    const keep = $('#champDetail').scrollTop; renderReleases(); $('#champDetail').scrollTop = keep; onDirty();
  });

  function setChamp(c, fk, v) { const f = c.fields[fk]; const lo = f.value < 0 || f.def < 0 ? -1e9 : 0; setVal(c, fk, Math.max(lo, f.type === 'json' && !Number.isInteger(f.value) ? +(+v).toFixed(3) : Math.round(v))); }
  function afterChampEdit(c, fk) {
    if (fk) {
      const row = $(`#champDetail [data-crow="${CSS.escape(fk)}"]`);
      if (row) row.outerHTML = numRow(c, fk);
    } else renderChampDetail();
    renderChampList();
  }

  $('#champList').addEventListener('click', e => {
    const b = e.target.closest('[data-champ]'); if (!b) return;
    S.champSel = S.champs.find(c => c.id === b.dataset.champ); S.champView = 'edit';
    $('#btnNotes').classList.remove('on'); $('#btnReleases').classList.remove('on');
    renderChampList(); renderChampDetail();
  });
  $('#cq').addEventListener('input', renderChampList);
  $('#cCat').addEventListener('change', renderChampList);
  $('#cMirror').addEventListener('change', e => { S.champMirror = e.target.checked; if (S.champView === 'notes') renderPatchNotes(); });
  function setChampView(v) {
    S.champView = S.champView === v ? 'edit' : v;
    $('#btnNotes').classList.toggle('on', S.champView === 'notes');
    $('#btnReleases').classList.toggle('on', S.champView === 'releases');
    renderChampDetail();
  }
  $('#btnNotes').addEventListener('click', () => setChampView('notes'));
  $('#btnReleases').addEventListener('click', () => setChampView('releases'));
  $('#champDetail').addEventListener('change', e => {
    const el = e.target.closest('[data-cf]'); if (!el) return;
    const c = S.champSel, fk = el.dataset.cf, v = Number(el.value);
    if (!isFinite(v)) { toast('Please enter a number', 'err'); el.value = val(c, fk); return; }
    setChamp(c, fk, v); afterChampEdit(c, fk);
  });
  $('#champDetail').addEventListener('click', e => {
    const c = S.champSel; if (!c) return;
    const st = e.target.closest('[data-cstep]');
    if (st) { const fk = st.closest('[data-crow]').dataset.crow; const f = c.fields[fk]; setChamp(c, fk, val(c, fk) + (+st.dataset.cstep) * niceStep(f.def || f.value) * (e.shiftKey ? 10 : 1)); afterChampEdit(c, fk); return; }
    const t = e.target;
    if (t.dataset.cscale) {
      const m = +t.dataset.cscale;
      for (const fk of c.order) { const f = c.fields[fk];
        const power = (f.group === 'stat' || f.group === 'growth') ? ['attack', 'magic_power', 'hp', 'defence', 'magic_resistance'].includes(f.key) : POWER_KEYS.test(f.key);
        if (power && val(c, fk) > 0) setChamp(c, fk, val(c, fk) * m); }
      afterChampEdit(c);
    } else if (t.dataset.cdefault) { for (const fk of c.order) setChamp(c, fk, c.fields[fk].def); afterChampEdit(c); }
    else if (t.dataset.crevert) { c.edits = null; onDirty(); afterChampEdit(c); }
  });

  // ---------------------------------------------------------------- tabs
  function switchTab(name) {
    S.tab = name;
    $$('.tab').forEach(t => t.classList.toggle('active', t.dataset.tab === name));
    ['players', 'teams', 'champions', 'skills', 'map', 'test', 'changes'].forEach(n => ($('#tab-' + n).hidden = n !== name));
    if (name === 'skills') (window.TFM2Skills.show || window.TFM2Skills.render)();
    if (name === 'map' && window.TFM2Map) window.TFM2Map.show();
    if (name === 'test' && window.TFM2SkillTest) window.TFM2SkillTest.show();
    if (name === 'champions') { renderChampList(); renderChampDetail(); }
    if (name === 'teams') renderTeams();
    if (name === 'changes') renderChanges();
    if (name === 'players') renderRows();
  }
  $$('.tab').forEach(t => t.addEventListener('click', () => switchTab(t.dataset.tab)));

  // ---------------------------------------------------------------- saving
  async function buildOutput() {
    busy('Saving', 0.1, 'Applying changes…'); await tick();
    const patches = C.collectPatches(S.athletes.concat(S.teams, S.champs.filter(c => c.kind === 'champ'))).concat(jsonChampPatches());
    if (S.champMirror) for (const c of S.champs) if (c.kind === 'champ' && c.edits) for (const fk in c.edits) {
      const f = c.fields[fk]; if (f.baseOff == null || c.edits[fk] === f.baseValue) continue;
      patches.push({ off: f.baseOff, len: 8, bytes: C.encodeField({ type: 'u64' }, c.edits[fk]).bytes });
    }
    if (S.pool && S.poolAdd && S.poolAdd.length) patches.push({ off: S.pool.off, len: S.pool.end - S.pool.off, bytes: C.encodeStrVec(S.pool.ids.concat(S.poolAdd)) });
    const payload = C.applyPatches(S.payload, patches);
    busy('Saving', 0.35, 'Compressing ' + fmtSize(payload.length) + '…'); await tick();
    const gz = await gzip(payload);
    busy('Saving', 0.8, 'Writing file…'); await tick();
    // exported DBs get a fresh timestamp (like the community repack tools); saves keep theirs
    return C.buildContainer(S.header, gz, S.kind === 1 ? null : S.timestamp);
  }

  function download(bytes, name) {
    const url = URL.createObjectURL(new Blob([bytes], { type: 'application/octet-stream' }));
    const a = document.createElement('a'); a.href = url; a.download = name; document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 30000);
  }

  async function saveTo(target) {
    // target: {mode:'server', path, name} | {mode:'handle', handle, name} | {mode:'download', name}
    try {
      const out = await buildOutput();
      if (target.mode === 'server') {
        let res = await fetch('/api/file?path=' + encodeURIComponent(target.path), { method: 'PUT', body: out });
        if (res.status === 409) {
          busy(false);
          if (!confirm((await res.json()).error + '\n\nSave anyway?')) return;
          busy('Saving', 0.9, 'Writing file…');
          res = await fetch('/api/file?force=1&path=' + encodeURIComponent(target.path), { method: 'PUT', body: out });
        }
        const j = await res.json();
        if (!res.ok) throw new Error(j.error || res.statusText);
        await loadBytes(out, { mode: 'server', path: target.path, name: target.name, dir: target.path.replace(/[\\/][^\\/]*$/, '') }, true);
        toast(`Saved ${target.name}` + (j.backup ? ' — backup of the previous version kept in the editor\'s backups folder.' : '.'), 'ok', 6000);
      } else if (target.mode === 'handle') {
        const w = await target.handle.createWritable(); await w.write(out); await w.close();
        await loadBytes(out, { mode: 'handle', handle: target.handle, name: target.name }, true);
        toast('Saved ' + target.name, 'ok');
      } else {
        download(out, target.name);
        await loadBytes(out, { mode: 'blob', name: target.name }, true);
        toast('Downloaded ' + target.name + ' — move it where the game expects it.', 'ok', 6000);
      }
      if (S.kind === 1) setTimeout(() => toast('This is an exported database: import it from the game\'s database menu to use it.', '', 7000), 6200);
    } catch (e) { busy(false); toast('Save failed: ' + e.message, 'err', 8000); console.error(e); }
  }

  $('#btnSave').addEventListener('click', async () => {
    if (!S.src) return;
    if (!countChanges() && !confirm('No changes to save. Write the file anyway?')) return;
    if (S.src.mode === 'server') return saveTo(S.src);
    if (S.src.mode === 'handle') return saveTo(S.src);
    return saveTo({ mode: 'download', name: S.src.name });
  });
  $('#btnSaveAs').addEventListener('click', () => {
    const n = S.src.name.replace(/(\.[^.]+)$/, '_edited$1');
    $('#saveName').value = n;
    $('#saveWhere').textContent = S.src.mode === 'server' ? 'Saved next to the original, in ' + S.src.dir : 'Your browser will download the file.';
    $('#saveDlg').showModal();
  });
  $('#saveDlg').addEventListener('close', () => {
    const rv = $('#saveDlg').returnValue; if (rv !== 'save' && rv !== 'download') return;
    let name = $('#saveName').value.trim();
    if (!name) return;
    if (!/\.(tfm2db|data)$/i.test(name)) name += /\.data$/i.test(S.src.name) ? '.data' : '.tfm2db';
    if (/[\\/:*?"<>|]/.test(name)) return toast('File name contains characters Windows does not allow.', 'err');
    if (rv === 'save' && S.src.mode === 'server') {
      const sep = S.src.dir.includes('\\') ? '\\' : '/';
      return saveTo({ mode: 'server', path: S.src.dir + sep + name, name });
    }
    saveTo({ mode: 'download', name });
  });

  // ---------------------------------------------------------------- skill lab
  // champion sprites arrive from the server a moment after start-up
  if (window.TFM2_ART) TFM2_ART.ready(() => { if (S.champs && S.champs.length) { renderChampList(); renderChampDetail(); } });
  window.TFM2Skills.init({
    toast, server: () => S.server,
    liveStats: id => {
      const c = (S.champs || []).find(x => x.id === id && x.kind === 'champ'); if (!c) return null;
      const out = { stat: {}, growth: {} };
      for (const fk of c.order) { const f = c.fields[fk]; if (f.group === 'stat' || f.group === 'growth') out[f.group][f.key] = val(c, fk); }
      return Object.keys(out.stat).length === 9 && Object.keys(out.growth).length === 9 ? out : null;
    },
    liveJson: id => { const c = (S.champs || []).find(x => x.id === id && x.kind === 'champjson'); return c ? c.entry.current.json : null; },
  });
  $('#btnSkillLab').addEventListener('click', () => {
    $('#welcome').hidden = true; $('#workspace').hidden = false;
    if (!S.payload) $$('.tab').forEach(t => { t.hidden = !['skills', 'map', 'test'].includes(t.dataset.tab); });
    switchTab('skills');
  });
  $('#btnSkillTest').addEventListener('click', () => {
    $('#welcome').hidden = true; $('#workspace').hidden = false;
    if (!S.payload) $$('.tab').forEach(t => { t.hidden = !['skills', 'map', 'test'].includes(t.dataset.tab); });
    switchTab('test');
  });
  // athlete names for the Skill Test's Scribble memory page (when a save or database is open)
  window.TFM2_APP = { athleteName: id => { const a = (S.athletes || []).find(x => x.id === id); return a ? a.name : null; },
    // every athlete in the open save or database (id + name), for the Scribble mastery seeding
    athletes: () => (S.athletes || []).map(a => ({ id: a.id, name: a.name })) };
  $('#btnMapEd').addEventListener('click', () => {
    $('#welcome').hidden = true; $('#workspace').hidden = false;
    if (!S.payload) $$('.tab').forEach(t => { t.hidden = !['skills', 'map', 'test'].includes(t.dataset.tab); });
    switchTab('map');
  });

  // ---------------------------------------------------------------- boot
  if (!('DecompressionStream' in window)) {
    document.body.innerHTML = '<p style="padding:40px">This browser is too old for the editor. Please use a current Chrome, Edge or Firefox.</p>';
    return;
  }
  initWelcome();
})();
