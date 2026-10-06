#!/usr/bin/env node
/* TFM2 Database Editor — tiny local server (no dependencies).
 * Serves the editor UI and gives it read/write access to the game's save + database files.
 *
 *   node server.js            -> http://localhost:7272
 *   node server.js --port 8080 --dir "D:\other\folder"
 *   node server.js --game "D:\SteamLibrary\steamapps\common\Teamfight Manager 2"
 */
'use strict';
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFile } = require('child_process');

const args = process.argv.slice(2);
const argVal = (name, def) => { const i = args.indexOf(name); return i >= 0 && args[i + 1] ? args[i + 1] : def; };
const PORT = Number(argVal('--port', 7272));
const HERE = __dirname;
const APPDATA = process.env.APPDATA || path.join(os.homedir(), 'AppData', 'Roaming');
const BACKUP_DIR = path.join(HERE, 'backups');

// Folders the editor may read/write.
const ROOTS = [
  { id: 'game', label: 'Game data (autosaves + active custom DB)', dir: path.join(APPDATA, 'TeamSamoyed', 'TeamfightManager2', 'data') },
  { id: 'exports', label: 'Exported databases', dir: path.resolve(HERE, '..', 'database') },
];
const extra = argVal('--dir', null);
if (extra) ROOTS.push({ id: 'extra', label: 'Extra folder', dir: path.resolve(extra) });

const EXT_OK = /\.(tfm2db|data|bak)$/i;
const gx = require('./extract-gamedata.js');
const GAME_DIR = gx.findGameDir(argVal('--game', null)) || path.resolve(HERE, '..', 'Teamfight Manager 2');
const MODS_DIR = path.join(GAME_DIR, 'mods');
let BUNDLE = null;
function bundleIndex() {
  const p = path.join(GAME_DIR, 'bundle.game_data');
  try {
    const m = fs.statSync(p).mtimeMs;
    if (!BUNDLE || BUNDLE.mtime !== m) BUNDLE = { path: p, mtime: m, index: gx.indexBundle(p), art: null };
    return BUNDLE;
  } catch (e) { return null; }
}
const MOD_ID_RE = /^[a-z0-9_]{2,40}$/;

/* Custom champion sprites inside mod folders (mods/<id>/champions/<name>#sheet.png + #anim.fanim), in the same
   shape as the base art list, keyed by their full asset path (asset/<id>/champions/<name>). */
function modSpriteArt() {
  const out = {};
  let mods = [];
  try { mods = fs.readdirSync(MODS_DIR, { withFileTypes: true }).filter(d => d.isDirectory()).map(d => d.name); } catch (e) { return out; }
  for (const id of mods) {
    const dir = path.join(MODS_DIR, id, 'champions');
    let files = []; try { files = fs.readdirSync(dir); } catch (e) { continue; }
    for (const f of files) {
      if (!f.endsWith('#anim.fanim')) continue;
      const name = f.slice(0, -'#anim.fanim'.length);
      const png = path.join(dir, name + '#sheet.png');
      if (!fs.existsSync(png)) continue;
      try {
        const anims = JSON.parse(fs.readFileSync(path.join(dir, f), 'utf8').replace(/^\uFEFF/, '')).anims || {};
        const head = Buffer.alloc(24); const fd = fs.openSync(png, 'r'); fs.readSync(fd, head, 0, 24, 0); fs.closeSync(fd);
        const idle = anims.idle || anims.run || Object.values(anims)[0];
        out[`asset/${id}/champions/${name}`] = {
          url: `/api/mod-png?id=${encodeURIComponent(id)}&path=${encodeURIComponent('champions/' + name + '#sheet.png')}&v=${Math.round(fs.statSync(png).mtimeMs)}`,
          sw: head.readUInt32BE(16), sh: head.readUInt32BE(20),
          idle: ((idle && idle.frames) || []).slice(0, 12).map(fr => ({ x: fr.data.x, y: fr.data.y, w: fr.data.w, h: fr.data.h, d: Math.round((fr.duration || 0.1) * 1000) })),
          tags: Object.keys(anims).sort(),
        };
      } catch (e) { /* skip broken sprite */ }
    }
  }
  return out;
}

function listMods() {
  let dirs = [];
  try { dirs = fs.readdirSync(MODS_DIR, { withFileTypes: true }).filter(d => d.isDirectory()).map(d => d.name); } catch (e) { return []; }
  return dirs.map(id => {
    const dir = path.join(MODS_DIR, id);
    const readJson = f => { try { return JSON.parse(fs.readFileSync(path.join(dir, f), 'utf8').replace(/^\uFEFF/, '')); } catch (e) { return null; } };
    const info = readJson('mod.mod_info');
    const champions = [];
    const walk = (d, rel) => {
      let ents = []; try { ents = fs.readdirSync(d, { withFileTypes: true }); } catch (e) { return; }
      for (const e of ents) {
        const r = rel ? rel + '/' + e.name : e.name;
        if (e.isDirectory()) { if (r.split('/').length < 4) walk(path.join(d, e.name), r); }
        else if (/\.data_champion(\.off)?$/i.test(e.name)) {
          // "<id>.data_champion.off" = switched off in the Skill Lab (the game only loads .data_champion)
          const disabled = /\.off$/i.test(e.name);
          try { champions.push({ file: r, disabled, json: JSON.parse(fs.readFileSync(path.join(d, e.name), 'utf8').replace(/^\uFEFF/, '')) }); }
          catch (err) { champions.push({ file: r, disabled, error: String(err.message) }); }
        }
      }
    };
    walk(dir, '');
    return { id, info, text: readJson('text/champion.i18n'), champions, editable: MOD_ID_RE.test(id) };
  }).filter(m => m.info);
}

const MODS_CFG = path.resolve(MODS_DIR, '..', 'config', 'game', 'mods.json');
function enabledMods() {
  try { return JSON.parse(fs.readFileSync(MODS_CFG, 'utf8')).enabled_mods || []; } catch (e) { return []; }
}

function describe(file, stat) {
  const name = path.basename(file);
  let type = 'Database';
  if (/^save_.*\.data$/i.test(name)) type = 'Career save';
  else if (/^custom_database\.tfm2db$/i.test(name)) type = 'Active custom DB';
  else if (/\.bak$/i.test(name)) type = 'Backup';
  return { path: file, name, type, size: stat.size, mtime: stat.mtimeMs };
}

function listFiles() {
  return ROOTS.map(r => {
    let files = [];
    try {
      files = fs.readdirSync(r.dir)
        .filter(n => EXT_OK.test(n))
        .map(n => path.join(r.dir, n))
        .map(f => { try { const s = fs.statSync(f); return s.isFile() ? describe(f, s) : null; } catch (e) { return null; } })
        .filter(Boolean)
        .sort((a, b) => b.mtime - a.mtime);
    } catch (e) { /* folder missing */ }
    return { id: r.id, label: r.label, dir: r.dir, exists: fs.existsSync(r.dir), files };
  });
}

function allowed(p) {
  const abs = path.resolve(p);
  return ROOTS.some(r => {
    const rel = path.relative(r.dir, abs);
    return rel && !rel.startsWith('..') && !path.isAbsolute(rel) && !rel.includes(path.sep);
  }) && /\.(tfm2db|data)$/i.test(abs);
}

function gameRunning() {
  return new Promise(resolve => {
    if (process.platform !== 'win32') return resolve(false);
    execFile('tasklist', ['/FI', 'IMAGENAME eq TeamfightManager2.exe', '/NH'], { windowsHide: true }, (err, out) => {
      resolve(!err && /TeamfightManager2\.exe/i.test(out || ''));
    });
  });
}

function stamp() {
  const d = new Date(), z = n => String(n).padStart(2, '0');
  return `${d.getFullYear()}${z(d.getMonth() + 1)}${z(d.getDate())}-${z(d.getHours())}${z(d.getMinutes())}${z(d.getSeconds())}`;
}

function send(res, code, body, type) {
  res.writeHead(code, { 'Content-Type': type || 'application/json; charset=utf-8', 'Cache-Control': 'no-store' });
  res.end(typeof body === 'string' || Buffer.isBuffer(body) ? body : JSON.stringify(body));
}

const STATIC = { '/': 'index.html', '/index.html': 'index.html', '/app.js': 'app.js', '/core.js': 'core.js', '/style.css': 'style.css', '/champions.js': 'champions.js', '/gamedata.js': 'gamedata.js', '/skills.js': 'skills.js', '/presets.js': 'presets.js', '/art.js': 'art.js', '/vfx.js': 'vfx.js', '/vfx2.js': 'vfx2.js', '/sprites.js': 'sprites.js', '/sprites-data.js': 'sprites-data.js', '/mapedit.js': 'mapedit.js', '/levi-map.js': 'levi-map.js', '/levilab.js': 'levilab.js', '/isliidlab.js': 'isliidlab.js', '/isliid-sprite.png': 'isliid-sprite.png', '/isliid-sprite-sheet.png': 'isliid-sprite-sheet.png', '/isliid-trails.png': 'isliid-trails.png', '/isliid-weapons.png': 'isliid-weapons.png', '/isliid-ranks.png': 'isliid-ranks.png', '/isliid-swords8-8.png': 'isliid-swords8-8.png', '/isliid-orbit8-8.png': 'isliid-orbit8-8.png', '/isliid-auras8-8.png': 'isliid-auras8-8.png', '/isliid-aura_fields8-8.png': 'isliid-aura_fields8-8.png', '/isliid-badges8-8.png': 'isliid-badges8-8.png', '/isliid-logos-8.png': 'isliid-logos-8.png', '/isliid-swords_fly8-8.png': 'isliid-swords_fly8-8.png', '/isliid-art-manifest.json': 'isliid-art-manifest.json', '/skilltest.js': 'skilltest.js' };
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.png': 'image/png' };

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://localhost');
  try {
    if (req.method === 'GET' && STATIC[url.pathname]) {
      const f = path.join(HERE, STATIC[url.pathname]);
      return send(res, 200, fs.readFileSync(f), MIME[path.extname(f)]);
    }
    if (url.pathname === '/api/files') {
      return send(res, 200, { roots: listFiles(), gameRunning: await gameRunning(), backupDir: BACKUP_DIR });
    }
    if (url.pathname === '/api/champ-art') {
      const b = bundleIndex();
      if (!b) return send(res, 200, { champions: modSpriteArt() });
      if (!b.art) b.art = gx.champArt(b.path, b.index);
      return send(res, 200, { champions: Object.assign({}, b.art, modSpriteArt()) });
    }
    if (url.pathname === '/api/mod-png' && req.method === 'GET') {
      // a PNG inside a mod folder (custom champion sprites), served as an image
      const id = url.searchParams.get('id') || '', rel = url.searchParams.get('path') || '';
      if (!MOD_ID_RE.test(id) || !/^[a-zA-Z0-9_\-./#]+\.png$/.test(rel) || rel.includes('..')) return send(res, 400, { error: 'Bad path' });
      const f = path.join(MODS_DIR, id, ...rel.split('/'));
      if (!fs.existsSync(f)) return send(res, 404, { error: 'Not found' });
      return send(res, 200, fs.readFileSync(f), 'image/png');
    }
    if (url.pathname === '/api/base-anim' && req.method === 'GET') {
      // the full animation list of a base champion sprite (the sprite editor can start from a base look)
      const b = bundleIndex(); const key = (url.searchParams.get('key') || '') + '#anim';
      const ent = b && b.index.get(key);
      if (!ent || !/^asset\/base\/aseprite_resources\/champions\//.test(key)) return send(res, 404, { error: 'Not found' });
      return send(res, 200, gx.readAsset(b.path, ent).toString('utf8').replace(/^\uFEFF/, ''), 'application/json; charset=utf-8');
    }
    if (url.pathname === '/api/asset') {
      const b = bundleIndex(); const key = url.searchParams.get('key') || '';
      const ent = b && b.index.get(key);
      if (!ent || ent.type !== 'png' || !/^asset\/base\/aseprite_resources\//.test(key)) return send(res, 404, { error: 'Not found' });
      res.writeHead(200, { 'Content-Type': 'image/png', 'Content-Length': ent.size, 'Cache-Control': 'max-age=86400' });
      return res.end(gx.readAsset(b.path, ent));
    }
    if (url.pathname === '/api/mapdump' && req.method === 'GET') {
      // the 5v5 map document the native mod writes at the first match of a game run (walls, bushes, towers, camps...)
      const f = path.join(MODS_DIR, 'tfm2_custom_ai', 'map_dump.json');
      if (fs.existsSync(f)) {
        try { return send(res, 200, { source: 'dump', mtime: fs.statSync(f).mtimeMs, doc: JSON.parse(fs.readFileSync(f, 'utf8').replace(/^\uFEFF/, '')) }); } catch (e) { /* fall through */ }
      }
      // fallback: the wall cells read off the offline-baked visibility in bundle.game_data (cells nobody can see)
      const b = bundleIndex(); const ent = b && b.index.get('asset/base/setting/map_setting');
      if (!ent) return send(res, 200, { source: 'none' });
      const buf = gx.readAsset(b.path, ent); let p = 0;
      const u64 = () => { const v = Number(buf.readBigUInt64LE(p)); p += 8; return v; };
      const walls = [];
      try {
        const A = u64();
        for (let i = 0; i < A; i++) {
          const B = u64(); const row = [];
          for (let j = 0; j < B; j++) {
            const n = u64(); let sum = 0;
            for (let r = 0; r < n; r++) { const m = u64(); for (let k = 0; k < m; k++) sum += buf[p + k]; p += m; }
            row.push(sum === 0 ? 1 : 0);
          }
          walls.push(row);
        }
      } catch (e) { return send(res, 200, { source: 'none' }); }
      return send(res, 200, { source: 'fallback', doc: { walls } });
    }
    if (url.pathname === '/api/tactics' && req.method === 'GET') {
      const f = path.join(MODS_DIR, 'tfm2_custom_ai', 'tactics.json');
      let doc = { marks: [], tokens: [] };
      try { if (fs.existsSync(f)) doc = JSON.parse(fs.readFileSync(f, 'utf8')); } catch (e) { /* keep empty */ }
      return send(res, 200, { doc, dirExists: fs.existsSync(path.join(MODS_DIR, 'tfm2_custom_ai')) });
    }
    if (url.pathname === '/api/tactics' && req.method === 'PUT') {
      // tactics.json is the editor's own copy; tactics.txt is what the native mod reads at every match start
      const dir = path.join(MODS_DIR, 'tfm2_custom_ai');
      if (!fs.existsSync(dir)) return send(res, 404, { error: 'The native mod folder was not found: ' + dir });
      const chunks = []; for await (const c of req) chunks.push(c);
      let body; try { body = JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch (e) { return send(res, 400, { error: 'Bad JSON' }); }
      if (!body || typeof body.txt !== 'string' || typeof body.doc !== 'object') return send(res, 400, { error: 'Missing doc / txt' });
      for (const n of ['tactics.json', 'tactics.txt']) {
        const f = path.join(dir, n);
        if (fs.existsSync(f)) { const bdir = path.join(BACKUP_DIR, 'tactics'); fs.mkdirSync(bdir, { recursive: true }); fs.copyFileSync(f, path.join(bdir, stamp() + '.' + n)); }
      }
      fs.writeFileSync(path.join(dir, 'tactics.json'), JSON.stringify(body.doc, null, 2));
      fs.writeFileSync(path.join(dir, 'tactics.txt'), body.txt);
      console.log(`[tactics saved] ${(body.doc.marks || []).length} marks`);
      return send(res, 200, { ok: true, dir });
    }
    if (url.pathname === '/api/scribble' && req.method === 'GET') {
      const pre = ['scribble', 'levi', 'isliid'].includes(url.searchParams.get('char')) ? url.searchParams.get('char') : 'scribble';
      // Scribble's learning files (written by the native mod): memory, games/casts waiting to be merged, the game log
      const dir = path.join(MODS_DIR, 'tfm2_custom_ai');
      const rd = n => { try { return fs.readFileSync(path.join(dir, n), 'utf8'); } catch (e) { return ''; } };
      // the merged history can grow large: only its tail (per-game summaries for the recent games list)
      let history = '';
      try { const f = path.join(dir, `${pre}_history.txt`); const st = fs.statSync(f); const n = Math.min(st.size, 2 * 1024 * 1024);
        const fd = fs.openSync(f, 'r'); const buf = Buffer.alloc(n); fs.readSync(fd, buf, 0, n, st.size - n); fs.closeSync(fd);
        history = buf.toString('utf8').split(/\r?\n/).filter(l => l.startsWith('s ') || l.startsWith('r ')).join('\n'); } catch (e) { /* none yet */ }
      return send(res, 200, { dir, exists: fs.existsSync(dir), memory: rd(`${pre}_memory.txt`), pending: rd(`${pre}_pending.txt`), history, log: rd(`${pre}_log.txt`), gameRunning: await gameRunning() });
    }
    if (url.pathname === '/api/scribble' && req.method === 'POST') {
      const pre = ['scribble', 'levi', 'isliid'].includes(url.searchParams.get('char')) ? url.searchParams.get('char') : 'scribble';
      // reset-meta | reset-all | set-games {athlete, games} | seed {entries}; the native mod reads the memory at the next game start
      const dir = path.join(MODS_DIR, 'tfm2_custom_ai');
      if (!fs.existsSync(dir)) return send(res, 404, { error: 'The native mod folder was not found: ' + dir });
      const chunks = []; for await (const c of req) chunks.push(c);
      let body; try { body = JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch (e) { return send(res, 400, { error: 'Bad JSON' }); }
      const memF = path.join(dir, `${pre}_memory.txt`), pendF = path.join(dir, `${pre}_pending.txt`);
      const bdir = path.join(BACKUP_DIR, pre); fs.mkdirSync(bdir, { recursive: true });
      const st = stamp();
      for (const f of [memF, pendF]) if (fs.existsSync(f)) fs.copyFileSync(f, path.join(bdir, st + '.' + path.basename(f)));
      const lines = (fs.existsSync(memF) ? fs.readFileSync(memF, 'utf8') : '').split(/\r?\n/).filter(Boolean);
      const pend = (fs.existsSync(pendF) ? fs.readFileSync(pendF, 'utf8') : '').split(/\r?\n/).filter(Boolean);
      let out = lines, outPend = pend;
      if (body.action === 'reset-all') { out = []; outPend = []; }
      else if (body.action === 'reset-meta') { out = lines.filter(l => !l.startsWith('M ')); outPend = pend.filter(l => !l.startsWith('c ')); }
      else if (body.action === 'set-games') {
        // sets the athlete's mastery points (the games / wins counts are kept for reference)
        const a = Number(body.athlete), g = Math.max(0, Math.round((Number(body.games) || 0) * 2) / 2);
        if (!Number.isFinite(a) || a < 0) return send(res, 400, { error: 'Bad athlete id' });
        const old = lines.find(l => l.startsWith(`G ${a} `)); const f = old ? old.split(/\s+/) : [];
        const games = f.length === 5 ? f[3] : f.length === 3 ? f[2] : '0', wins = f.length === 5 ? f[4] : '0';
        out = lines.filter(l => !l.startsWith(`G ${a} `)).concat(g > 0 ? [`G ${a} ${g.toFixed(2)} ${games} ${wins}`] : []);
        outPend = pend.filter(l => !(l.startsWith('g ') && l.split(/\s+/)[2] === String(a)));
      } else if (body.action === 'seed') {
        // round 73: set many athletes' mastery at once ({entries: [{a, points, games, wins}]}); their games waiting in
        // the pending file are dropped so the numbers land exactly (like set-games)
        const list = Array.isArray(body.entries) ? body.entries : [];
        const ok = e => e && Number.isInteger(e.a) && e.a >= 0 && [e.points, e.games, e.wins].every(v => Number.isFinite(v) && v >= 0);
        if (!list.length || list.length > 5000 || !list.every(ok)) return send(res, 400, { error: 'Bad entries' });
        const ids = new Set(list.map(e => String(e.a)));
        out = lines.filter(l => !(l.startsWith('G ') && ids.has(l.split(/\s+/)[1])));
        for (const e of list) {
          const pts = Math.round(e.points * 2) / 2;
          if (pts > 0 || e.games > 0) out.push(`G ${e.a} ${pts.toFixed(2)} ${Math.round(e.games)} ${Math.round(e.wins)}`);
        }
        outPend = pend.filter(l => !(l.startsWith('g ') && ids.has(l.split(/\s+/)[2])));
      } else return send(res, 400, { error: 'Unknown action' });
      if (!out.some(l => l.startsWith('W '))) out.unshift('W 0');
      fs.writeFileSync(memF, out.join('\n') + '\n');
      fs.writeFileSync(pendF, outPend.length ? outPend.join('\n') + '\n' : '');
      console.log(`[${pre}] ${body.action}`);
      return send(res, 200, { ok: true });
    }
    if (url.pathname === '/api/status') return send(res, 200, { gameRunning: await gameRunning() });
    if (url.pathname === '/api/mods' && req.method === 'GET') {
      return send(res, 200, { dir: MODS_DIR, gameDirExists: fs.existsSync(path.dirname(MODS_DIR)), mods: listMods(), enabled: enabledMods(), gameRunning: await gameRunning() });
    }
    if (url.pathname === '/api/mod' && req.method === 'PUT') {
      const id = url.searchParams.get('id') || '';
      if (!MOD_ID_RE.test(id)) return send(res, 400, { error: 'Mod id must be 2-40 characters: lowercase letters, digits, _' });
      if (!fs.existsSync(path.dirname(MODS_DIR))) return send(res, 404, { error: 'Game folder not found: ' + GAME_DIR + ' (start the editor with --game "path\\to\\Teamfight Manager 2")' });
      const chunks = []; for await (const c of req) chunks.push(c);
      let body; try { body = JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch (e) { return send(res, 400, { error: 'Bad JSON' }); }
      const dir = path.join(MODS_DIR, id);
      const safe = rel => typeof rel === 'string' && /^[a-zA-Z0-9_\-./#]+$/.test(rel) && !rel.includes('..') && !path.isAbsolute(rel);
      const files = body.files || {}, remove = body.remove || [];
      for (const rel of Object.keys(files).concat(remove)) if (!safe(rel)) return send(res, 400, { error: 'Bad file path: ' + rel });
      // back up the previous version of the mod folder
      if (fs.existsSync(dir)) {
        const bdir = path.join(BACKUP_DIR, 'mods', id + '.' + stamp());
        fs.mkdirSync(bdir, { recursive: true });
        fs.cpSync(dir, bdir, { recursive: true });
      }
      for (const [rel, content] of Object.entries(files)) {
        const f = path.join(dir, ...rel.split('/'));
        fs.mkdirSync(path.dirname(f), { recursive: true });
        if (content && typeof content.base64 === 'string') fs.writeFileSync(f, Buffer.from(content.base64, 'base64'));   // images
        else fs.writeFileSync(f, typeof content === 'string' ? content : JSON.stringify(content, null, 2));
      }
      for (const rel of remove) { const f = path.join(dir, ...rel.split('/')); if (fs.existsSync(f)) fs.unlinkSync(f); }
      console.log(`[mod saved] ${dir} (${Object.keys(files).length} files)`);
      return send(res, 200, { ok: true, dir, enabled: enabledMods().includes(id) });
    }
    if (url.pathname === '/api/mod-file' && req.method === 'GET') {
      // one file of a mod, as base64 (used when a champion moves to another folder and takes its VFX along)
      const id = url.searchParams.get('id') || '', rel = url.searchParams.get('path') || '';
      if (!MOD_ID_RE.test(id) || !/^[a-zA-Z0-9_\-./#]+$/.test(rel) || rel.includes('..')) return send(res, 400, { error: 'Bad path' });
      const f = path.join(MODS_DIR, id, ...rel.split('/'));
      if (!fs.existsSync(f)) return send(res, 404, { error: 'Not found' });
      return send(res, 200, { base64: fs.readFileSync(f).toString('base64') });
    }
    if (url.pathname === '/api/mod-enabled' && req.method === 'POST') {
      // switch a mod folder on/off in the game's own mod list (config/game/mods.json)
      const id = url.searchParams.get('id') || '', on = url.searchParams.get('on') === '1';
      if (!MOD_ID_RE.test(id) || !fs.existsSync(path.join(MODS_DIR, id, 'mod.mod_info'))) return send(res, 400, { error: 'Unknown mod: ' + id });
      if (await gameRunning()) return send(res, 409, { error: 'Close the game first — it rewrites its mod list when it exits.' });
      let cfg = {}; try { cfg = JSON.parse(fs.readFileSync(MODS_CFG, 'utf8')); } catch (e) { cfg = {}; }
      const list = (cfg.enabled_mods || []).filter(x => x !== id);
      if (on) list.push(id);
      cfg.enabled_mods = list;
      fs.mkdirSync(path.dirname(MODS_CFG), { recursive: true });
      if (fs.existsSync(MODS_CFG)) fs.copyFileSync(MODS_CFG, MODS_CFG + '.bak');
      fs.writeFileSync(MODS_CFG, JSON.stringify(cfg));
      console.log(`[mod ${on ? 'enabled' : 'disabled'}] ${id}`);
      return send(res, 200, { ok: true, enabled: list });
    }
    if (url.pathname === '/api/file' && req.method === 'GET') {
      const p = url.searchParams.get('path');
      if (!p || !allowed(p)) return send(res, 403, { error: 'Path not allowed' });
      const st = fs.statSync(p);
      res.writeHead(200, { 'Content-Type': 'application/octet-stream', 'Content-Length': st.size, 'X-Mtime': String(st.mtimeMs) });
      return fs.createReadStream(p).pipe(res);
    }
    if (url.pathname === '/api/file' && req.method === 'PUT') {
      const p = url.searchParams.get('path');
      if (!p || !allowed(p)) return send(res, 403, { error: 'Path not allowed' });
      const chunks = [];
      for await (const c of req) chunks.push(c);
      const body = Buffer.concat(chunks);
      if (body.length < 25 || body.toString('latin1', 0, 4) !== 'TFM2') return send(res, 400, { error: 'Refusing to write: not a TFM2 file' });
      if (await gameRunning() && url.searchParams.get('force') !== '1')
        return send(res, 409, { error: 'Teamfight Manager 2 is running. Close it first, or it may overwrite your edits.' });
      let backup = null;
      if (fs.existsSync(p)) {
        fs.mkdirSync(BACKUP_DIR, { recursive: true });
        backup = path.join(BACKUP_DIR, `${path.basename(p)}.${stamp()}.bak`);
        fs.copyFileSync(p, backup);
      }
      const tmp = p + '.tfm2edit.tmp';
      fs.writeFileSync(tmp, body);
      fs.renameSync(tmp, p);
      console.log(`[saved] ${p} (${body.length} bytes)${backup ? '  backup -> ' + backup : ''}`);
      return send(res, 200, { ok: true, path: p, bytes: body.length, backup });
    }
    send(res, 404, { error: 'Not found' });
  } catch (e) {
    console.error(e);
    send(res, 500, { error: String(e.message || e) });
  }
});

// Refresh champion reference data when the game has been updated
try {
  const bundle = gx.findBundle(GAME_DIR);
  const out = path.join(HERE, 'gamedata.js');
  if (bundle && (!fs.existsSync(out) || fs.statSync(bundle).mtimeMs > fs.statSync(out).mtimeMs)) {
    console.log('  Reading champion data from the game…');
    console.log(`  ✓ gamedata.js rebuilt (${gx.build(bundle, out)} champions)`);
  }
} catch (e) { console.warn('  (could not refresh champion data: ' + e.message + ')'); }

server.listen(PORT, '127.0.0.1', () => {
  const url = `http://localhost:${PORT}`;
  console.log(`\n  TFM2 Database Editor running at ${url}\n`);
  for (const r of ROOTS) console.log(`  ${fs.existsSync(r.dir) ? '✓' : '✗'} ${r.label}: ${r.dir}`);
  console.log(`  ${fs.existsSync(GAME_DIR) ? '✓' : '✗'} Game (mods go in ${MODS_DIR}): ${GAME_DIR}`);
  console.log(`  Backups: ${BACKUP_DIR}\n  Press Ctrl+C to stop.\n`);
  if (!args.includes('--no-open')) {
    const cmd = process.platform === 'win32' ? 'cmd' : process.platform === 'darwin' ? 'open' : 'xdg-open';
    const a = process.platform === 'win32' ? ['/c', 'start', '', url] : [url];
    execFile(cmd, a, { windowsHide: true }, () => { });
  }
});
server.on('error', e => {
  if (e.code === 'EADDRINUSE') console.error(`Port ${PORT} is busy — is the editor already open? Try http://localhost:${PORT} or use --port.`);
  else console.error(e);
  process.exit(1);
});
