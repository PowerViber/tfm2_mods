#!/usr/bin/env node
/* Rebuilds gamedata.js (champion defaults, names, balance-patch settings) from the game's bundle.game_data.
 * Run it after a game update:   node extract-gamedata.js ["path\to\bundle.game_data"]
 * server.js runs it automatically when the bundle is newer than gamedata.js.
 */
'use strict';
const fs = require('fs');
const path = require('path');

const GAME_FOLDER = 'Teamfight Manager 2';
const STEAM_APP_ID = '3009300';

// Steam library folders, from the registry and libraryfolders.vdf
function steamLibraries() {
  const roots = new Set();
  if (process.platform === 'win32') {
    const { execFileSync } = require('child_process');
    for (const [key, val] of [['HKCU\\Software\\Valve\\Steam', 'SteamPath'], ['HKLM\\SOFTWARE\\WOW6432Node\\Valve\\Steam', 'InstallPath'], ['HKLM\\SOFTWARE\\Valve\\Steam', 'InstallPath']]) {
      try {
        const out = execFileSync('reg', ['query', key, '/v', val], { encoding: 'utf8', windowsHide: true, stdio: ['ignore', 'pipe', 'ignore'] });
        const m = out.match(/REG_SZ\s+(.+)/); if (m) roots.add(path.normalize(m[1].trim()));
      } catch (e) { /* not there */ }
    }
    for (const d of ['C:\\Program Files (x86)\\Steam', 'C:\\Program Files\\Steam']) roots.add(d);
  } else {
    const home = require('os').homedir();
    roots.add(path.join(home, '.steam', 'steam')); roots.add(path.join(home, '.local', 'share', 'Steam'));
  }
  const libs = new Set();
  for (const r of roots) {
    if (!fs.existsSync(r)) continue;
    libs.add(r);
    try {
      const vdf = fs.readFileSync(path.join(r, 'steamapps', 'libraryfolders.vdf'), 'utf8');
      for (const m of vdf.matchAll(/"path"\s+"([^"]+)"/g)) libs.add(path.normalize(m[1].replace(/\\\\/g, '\\')));
    } catch (e) { /* no vdf */ }
  }
  return [...libs];
}

// The game's install folder: --game / TFM2_GAME, Steam, or next to the editor
function findGameDir(arg) {
  const cands = [arg, process.env.TFM2_GAME];
  for (const lib of steamLibraries()) {
    let dir = GAME_FOLDER;
    try { const m = fs.readFileSync(path.join(lib, 'steamapps', `appmanifest_${STEAM_APP_ID}.acf`), 'utf8').match(/"installdir"\s+"([^"]+)"/); if (m) dir = m[1]; } catch (e) { /* not in this library */ }
    cands.push(path.join(lib, 'steamapps', 'common', dir));
  }
  cands.push(path.resolve(__dirname, '..', GAME_FOLDER), path.resolve(__dirname, '..'));
  return cands.filter(Boolean).map(p => path.resolve(p)).find(p => fs.existsSync(path.join(p, 'bundle.game_data'))) || null;
}

function findBundle(arg) {
  if (arg && /\.game_data$/i.test(arg) && fs.existsSync(arg)) return arg;
  const dir = findGameDir(arg);
  return dir ? path.join(dir, 'bundle.game_data') : undefined;
}

function asset(buf, name) {
  const key = Buffer.from('asset/base/' + name);
  let i = -1;
  while ((i = buf.indexOf(key, i + 1)) >= 0) {
    const e = i + key.length;
    if (buf[e + 4] !== 0x7b && !(buf[e + 4] === 0xef && buf[e + 7] === 0x7b)) continue; // '{' (optionally after BOM)
    const size = buf.readUInt32LE(e);
    const text = buf.toString('utf8', e + 4, e + 4 + size).replace(/^﻿/, '');
    try { return JSON.parse(text); } catch (err) { /* keep looking */ }
  }
  throw new Error('asset not found: ' + name);
}

function build(bundlePath, outPath) {
  const buf = fs.readFileSync(bundlePath);
  const info = asset(buf, 'setting/champion_info');
  const patch = asset(buf, 'setting/patch_setting');
  const text = asset(buf, 'text/champion');
  const modChampions = info.mod_champions || [];
  delete info.mod_champions;
  const en = text.en || {};
  const strip = s => String(s).replace(/<i#[^>]*>/g, '').replace(/<#[0-9a-f]{8}>|<>/g, '');
  const names = {}, desc = {};
  for (const [k, v] of Object.entries(en.description || {})) {
    names[k] = v.name || k;
    desc[k] = {}; for (const [a, x] of Object.entries(v)) if (a !== 'name') desc[k][a] = strip(x);
  }
  const patchActions = {};
  for (const [a, fields] of Object.entries(patch.actions || {})) {
    if (!fields || typeof fields !== 'object' || !Object.keys(fields).length) continue;
    patchActions[a] = {};
    for (const [f, d] of Object.entries(fields)) if (d && typeof d === 'object')
      patchActions[a][f] = { i18n: d.i18n_key, format: d.format, min: d.min_ratio, max: d.max_ratio };
  }
  const patchStat = {};
  for (const [k, d] of Object.entries(patch.stat || {})) patchStat[k] = { i18n: d.i18n_key, growth_i18n: d.growth_i18n_key, min: d.min_ratio, max: d.max_ratio };
  const gd = { info, modChampions, names, desc, category: en.category || {}, statLabels: en.stat || {}, patchKeys: en.patch_key || {}, patchActions, patchStat };
  const js = '/* Champion reference data extracted from your Teamfight Manager 2 install (bundle.game_data). */\n'
    + 'window.TFM2_GAMEDATA = ' + JSON.stringify(gd) + ';\n';
  fs.writeFileSync(outPath, js);
  return Object.keys(info).length;
}

// ---- bundle index (type, key, offset, size of every asset) — used by the server for champion art
function indexBundle(bundlePath) {
  const fd = fs.openSync(bundlePath, 'r');
  const size = fs.fstatSync(fd).size;
  const u32 = pos => { const b = Buffer.alloc(4); fs.readSync(fd, b, 0, 4, pos); return b.readUInt32LE(0); };
  const str = (pos, len) => { const b = Buffer.alloc(len); fs.readSync(fd, b, 0, len, pos); return b.toString('utf8'); };
  const index = new Map();
  try {
    const count = u32(0); let p = 4;
    for (let i = 0; i < count && p < size; i++) {
      const tl = u32(p); const type = str(p + 4, tl); p += 4 + tl;
      const kl = u32(p); const key = str(p + 4, kl); p += 4 + kl;
      const sz = u32(p); p += 4;
      index.set(key, { type, off: p, size: sz });
      p += sz;
    }
  } finally { fs.closeSync(fd); }
  return index;
}
function readAsset(bundlePath, ent) {
  const fd = fs.openSync(bundlePath, 'r');
  try { const b = Buffer.alloc(ent.size); fs.readSync(fd, b, 0, ent.size, ent.off); return b; } finally { fs.closeSync(fd); }
}
// Champion sprites: sheet PNG key, sheet size, idle frames and the animation tags each sprite has
function champArt(bundlePath, index) {
  const out = {};
  const pre = 'asset/base/aseprite_resources/champions/';
  for (const [key, ent] of index) {
    if (!key.startsWith(pre) || !key.endsWith('#anim')) continue;
    const id = key.slice(pre.length, -5);
    const sheetKey = pre + id + '#sheet'; const sheet = index.get(sheetKey);
    if (!sheet) continue;
    try {
      const anims = JSON.parse(readAsset(bundlePath, ent).toString('utf8').replace(/^\uFEFF/, '')).anims || {};
      const head = readAsset(bundlePath, { off: sheet.off, size: 24 });
      const idle = anims.idle || anims.run || Object.values(anims)[0];
      out[id] = {
        sheet: sheetKey, sw: head.readUInt32BE(16), sh: head.readUInt32BE(20),
        idle: ((idle && idle.frames) || []).slice(0, 12).map(f => ({ x: f.data.x, y: f.data.y, w: f.data.w, h: f.data.h, d: Math.round((f.duration || 0.1) * 1000) })),
        tags: Object.keys(anims).sort(),
      };
    } catch (e) { /* skip broken entry */ }
  }
  return out;
}

module.exports = { build, findBundle, findGameDir, indexBundle, readAsset, champArt };

if (require.main === module) {
  const bundle = findBundle(process.argv[2]);
  if (!bundle) { console.error('bundle.game_data not found. Pass its path as an argument.'); process.exit(1); }
  const n = build(bundle, path.join(__dirname, 'gamedata.js'));
  console.log(`gamedata.js rebuilt from ${bundle} (${n} champions)`);
}
