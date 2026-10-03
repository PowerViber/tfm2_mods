/* TFM2 database / save core — parses and patches Teamfight Manager 2 .tfm2db and save_*.data files.
 *
 * Container: "TFM2" | kind u8 | timestamp u64 (ms) | gzip length u64 | crc32(gzip) u32 | preview block | gzip(payload)
 * Payload:   Rust bincode (u64 lengths / little endian). Records are located by signature and parsed field-by-field.
 * Works in browsers (window.TFM2Core) and Node (module.exports).
 */
(function (root) {
  'use strict';

  const STAT_FIELDS = ['last_hit', 'skill_avoid', 'skill_hit', 'positioning', 'control_speed', 'concentration',
    'mental', 'judgement', 'order', 'roaming', 'aggressive', 'ego', 'top', 'jungle', 'mid', 'bottom', 'support'];
  const HIDDEN_FIELDS = ['potential', 'stamina_recovery_min', 'stamina_recovery_max', 'stamina_cost_per_set_min',
    'stamina_cost_per_set_max', 'stress_sensitivity', 'condition_baseline', 'condition_amplitude',
    'condition_period', 'condition_phase', 'match_impact_sensitivity'];
  const FACE_FIELDS = ['hair', 'hair_color', 'face_tattoo', 'glasses', 'necklace', 'top_color', 'pants_color', 'boots_color'];

  const LOGO_RE = /^(\d+_\d+|custom:.+)$/;
  const ASSET_RE = /^(#asset\/|asset\/|custom:)/;
  const HOUSE_RE = /^(furniture_|wallpaper_|clean_|plain_|premium_|wide_window|basic_chair)/;
  const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;
  const TIME_RE = /^\d{2}:\d{2}:\d{2}$/;

  const utf8d = new TextDecoder('utf-8', { fatal: true });
  const utf8e = new TextEncoder();

  // ---------- CRC32 ----------
  const CRC_TABLE = (() => {
    const t = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1;
      t[n] = c >>> 0;
    }
    return t;
  })();
  function crc32(u8) {
    let c = 0xFFFFFFFF;
    for (let i = 0; i < u8.length; i++) c = CRC_TABLE[(c ^ u8[i]) & 0xFF] ^ (c >>> 8);
    return (c ^ 0xFFFFFFFF) >>> 0;
  }

  // ---------- container ----------
  function parseContainer(buf) {
    const u8 = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
    if (u8.length < 25 || u8[0] !== 0x54 || u8[1] !== 0x46 || u8[2] !== 0x4D || u8[3] !== 0x32)
      throw new Error('Not a TFM2 file (missing "TFM2" header).');
    const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
    const kind = u8[4];
    const timestamp = Number(dv.getBigUint64(5, true));
    const gzLen = Number(dv.getBigUint64(13, true));
    const crc = dv.getUint32(21, true);
    const gzStart = u8.length - gzLen;
    if (gzLen <= 0 || gzStart < 25) throw new Error('Corrupt TFM2 header (bad gzip length).');
    const gz = u8.subarray(gzStart);
    if (gz[0] !== 0x1f || gz[1] !== 0x8b) throw new Error('Gzip data not found where the header says it should be.');
    return { kind, timestamp, gzLen, crc, header: u8.slice(0, gzStart), gz, crcOk: null };
  }

  function buildContainer(header, gz, timestamp) {
    const out = new Uint8Array(header.length + gz.length);
    out.set(header, 0);
    out.set(gz, header.length);
    const dv = new DataView(out.buffer);
    dv.setBigUint64(5, BigInt(timestamp == null ? Date.now() : timestamp), true);
    dv.setBigUint64(13, BigInt(gz.length), true);
    dv.setUint32(21, crc32(gz), true);
    return out;
  }

  // ---------- reader ----------
  class Reader {
    constructor(p, o) { this.p = p; this.o = o; this.dv = new DataView(p.buffer, p.byteOffset, p.byteLength); }
    need(n) { if (this.o + n > this.p.length) throw FAIL; }
    u8() { this.need(1); return this.p[this.o++]; }
    u64() {
      this.need(8);
      const lo = this.dv.getUint32(this.o, true), hi = this.dv.getUint32(this.o + 4, true);
      this.o += 8;
      if (hi > 0x1FFFFF) return Infinity; // too big to be a sane count/stat
      return hi * 4294967296 + lo;
    }
    f64() { this.need(8); const v = this.dv.getFloat64(this.o, true); this.o += 8; return v; }
    str(max) {
      const n = this.u64();
      if (!(n <= (max || 4096))) throw FAIL;
      this.need(n);
      let s; try { s = utf8d.decode(this.p.subarray(this.o, this.o + n)); } catch (e) { throw FAIL; }
      this.o += n;
      return s;
    }
  }

  const FAIL = { parseFail: true };
  function fail() { throw FAIL; }

  // Parse one athlete starting at `o` (map key). Throws if the bytes don't look like an athlete.
  function parseAthlete(p, o) {
    const r = new Reader(p, o);
    const a = { start: o, fields: {} };
    const F = a.fields;
    const num = (name, type, off, value) => { F[name] = { type, off, value }; return value; };

    a.key = r.u64(); a.version = r.u64(); a.id = r.u64();
    if (a.key !== a.id || a.version > 255 || a.key > 1e7) fail('sig');
    a.nameOff = r.o;
    a.name = r.str(128);
    if (!a.name.length) fail('name');
    for (const ch of a.name) if (ch.charCodeAt(0) < 32) fail('name');
    F.name = { type: 'str', off: a.nameOff, value: a.name };
    a.statVersion = r.u64(); if (a.statVersion > 255) fail('statv');
    for (const f of STAT_FIELDS) { const off = r.o; const v = r.u64(); if (v > 1000) fail('stat'); num(f, 'u64', off, v); }
    // like / dislike champion lists (Vec<u64>)
    a.likeOff = r.o; let n = r.u64(); if (n > 64) fail('like'); a.like = []; for (let i = 0; i < n; i++) { const v = r.u64(); if (v > 1e6) fail('like'); a.like.push(v); }
    a.dislikeOff = r.o; n = r.u64(); if (n > 64) fail('dislike'); a.dislike = []; for (let i = 0; i < n; i++) { const v = r.u64(); if (v > 1e6) fail('dislike'); a.dislike.push(v); }
    // languages: Vec<(id, level)>
    a.langOff = r.o; n = r.u64(); if (n < 1 || n > 32) fail('lang');
    a.languages = [];
    for (let i = 0; i < n; i++) { const id = r.u64(), lv = r.u64(); if (id > 255 || lv > 1000) fail('lang'); a.languages.push([id, lv]); }
    a.langEnd = r.o;
    a.hiddenVersion = r.u64(); if (a.hiddenVersion > 255) fail('hidv');
    for (const f of HIDDEN_FIELDS) { const off = r.o; const v = r.u64(); if (v > 100000) fail('hid'); num(f, 'u64', off, v); }
    a.unknownX = r.u64(); if (a.unknownX > 1e6) fail('x');
    const tag = r.u64();
    if (tag > 1) fail('ctag');
    a.contract = null;
    if (tag === 1) {
      const c = {};
      const tOff = r.o; c.team = r.u64(); if (c.team > 1e7) fail('cteam');
      F.contract_team = { type: 'u64', off: tOff, value: c.team };
      const names = ['start_date', 'start_time', 'end_date', 'end_time'];
      for (let i = 0; i < 4; i++) {
        const off = r.o; const s = r.str(32);
        if (!(i % 2 ? TIME_RE : DATE_RE).test(s)) fail('cdate');
        F['contract_' + names[i]] = { type: 'str', off, value: s };
      }
      let off = r.o; const sal = r.f64(); if (!isFinite(sal) || sal < 0) fail('sal');
      F.contract_salary = { type: 'f64', off, value: sal };
      off = r.o; const fee = r.f64(); if (!isFinite(fee) || fee < 0) fail('fee');
      F.contract_transfer_fee = { type: 'f64', off, value: fee };
      n = r.u64(); if (n > 64) fail('citems');
      c.items = [];
      for (let i = 0; i < n; i++) {
        const x = r.u64(), k = r.u64(), f = r.f64();
        if (x > 1e6 || k > 1000 || !isFinite(f)) fail('citem');
        let e = null; if (k === 2) e = r.u64();
        c.items.push([x, k, f, e]);
      }
      const ot = r.u64(); if (ot > 1000) fail('copt');
      c.opt = [ot, ot ? r.u64() : null];
      a.contract = c;
    }
    a.unknownY = r.u64(); if (a.unknownY > 1e6) fail('y');
    a.faceVersion = r.u64(); if (a.faceVersion > 255) fail('facev');
    for (const f of FACE_FIELDS) { const off = r.o; const v = r.u64(); if (v > 1000) fail('face'); num('face_' + f, 'u64', off, v); }
    a.flagByte = r.u8(); if (a.flagByte > 1) fail('flag');
    const ageOff = r.o; const age = r.u64(); if (age > 150) fail('age');
    num('age', 'u64', ageOff, age);
    a.parsedEnd = r.o;
    return a;
  }

  function dominantVersion(list) {
    const byVer = new Map();
    for (const a of list) byVer.set(a.version, (byVer.get(a.version) || 0) + 1);
    let best = null, bestN = -1;
    for (const [v, c] of byVer) if (c > bestN) { best = v; bestN = c; }
    return list.filter(a => a.version === best);
  }

  function tryTeam(p, o, L) {
    const q = o + 32 + L;
    if (p[q] === 0 || p[q] > 127 || (p[q + 1] | p[q + 2] | p[q + 3] | p[q + 4] | p[q + 5] | p[q + 6] | p[q + 7])) return null;
    try {
      const r = new Reader(p, o);
      const key = r.u64(), version = r.u64(), id = r.u64();
      const nameOff = r.o; const name = r.str(128);
      const logoOff = r.o; const logo = r.str(128);
      if (!LOGO_RE.test(logo)) return null;
      return { start: o, key, version, id, name, logo, afterLogo: r.o, fields: {
        name: { type: 'str', off: nameOff, value: name },
        logo: { type: 'str', off: logoOff, value: logo },
      } };
    } catch (e) { return null; }
  }

  function tryAthlete(p, o, L) {
    const q = o + 32 + L;
    for (let k = 0; k < 6; k++) { const b = q + 8 * k; if (p[b + 2] | p[b + 3] | p[b + 4] | p[b + 5] | p[b + 6] | p[b + 7]) return null; }
    for (let j = o + 32; j < q; j++) if (p[j] < 32) return null;
    try { return parseAthlete(p, o); } catch (e) { return null; }
  }

  // One pass over the payload finding map entries shaped like (key, version, id == key, name, ...)
  async function scanDatabase(p, onProgress) {
    const athletes = [], teamsRaw = [];
    const n = p.length - 256;
    const CHUNK = 1 << 23;
    for (let base = 0; base < n; base += CHUNK) {
      if (onProgress) { onProgress(base / n); await new Promise(r => setTimeout(r, 0)); }
      const lim = Math.min(n, base + CHUNK);
      for (let o = base; o < lim; o++) {
        const L = p[o + 24];
        if (L === 0 || L > 127) continue;
        if (p[o] !== p[o + 16] || p[o + 1] !== p[o + 17] || p[o + 2] !== p[o + 18]) continue;
        if (p[o + 25] | p[o + 26] | p[o + 27] | p[o + 28] | p[o + 29] | p[o + 30] | p[o + 31]) continue;
        if (p[o + 3] | p[o + 4] | p[o + 5] | p[o + 6] | p[o + 7]) continue;
        if (p[o + 9] | p[o + 10] | p[o + 11] | p[o + 12] | p[o + 13] | p[o + 14] | p[o + 15]) continue;
        if (p[o + 19] | p[o + 20] | p[o + 21] | p[o + 22] | p[o + 23]) continue;
        const a = tryAthlete(p, o, L);
        if (a) { athletes.push(a); o = a.parsedEnd - 1; continue; }
        const t = tryTeam(p, o, L);
        if (t) { teamsRaw.push(t); o = t.afterLogo - 1; }
      }
    }
    if (onProgress) onProgress(1);
    const teams = dominantVersion(teamsRaw);
    findTeamStrings(p, teams);
    return { athletes: dominantVersion(athletes), teams };
  }

  // Scan length-prefixed printable strings in [from, to)
  function scanStrings(p, from, to, maxLen, stopRe) {
    const res = [];
    const dv = new DataView(p.buffer, p.byteOffset, p.byteLength);
    for (let i = from; i < to - 8; i++) {
      const L = p[i];
      if (L === 0 || L > (maxLen || 80)) continue;
      if (dv.getUint32(i + 4, true) !== 0 || (p[i + 1] | p[i + 2] | p[i + 3])) continue;
      const s0 = i + 8, s1 = s0 + L;
      if (s1 > to) continue;
      let ok = true;
      for (let j = s0; j < s1; j++) { const c = p[j]; if (c < 32 || c === 127) { ok = false; break; } }
      if (!ok) continue;
      let s; try { s = utf8d.decode(p.subarray(s0, s1)); } catch (e) { continue; }
      res.push({ off: i, text: s });
      i = s1 - 1;
      if (stopRe && stopRe.test(s)) break;
    }
    return res;
  }

  function findTeamStrings(p, teams) {
    // stadium & manager: last strings before the gaming-house block
    for (let i = 0; i < teams.length; i++) {
      const t = teams[i];
      const limit = i + 1 < teams.length ? teams[i + 1].start : Math.min(p.length, t.afterLogo + 4000000);
      const strs = scanStrings(p, t.afterLogo, limit, 80, HOUSE_RE);
      let gi = strs.findIndex(s => HOUSE_RE.test(s.text));
      if (gi < 0) gi = strs.length;
      const cand = strs.slice(0, gi).filter(s => !ASSET_RE.test(s.text) && !DATE_RE.test(s.text) && !TIME_RE.test(s.text));
      const u64at = off => { try { return new Reader(p, off).u64(); } catch (e) { return -1; } };
      // Stadium = Some(Stadium { name, 1, level, capacity, ... })
      const isStadium = s => {
        const e = s.off + 8 + utf8e.encode(s.text).length;
        const lvl = u64at(e + 8), cap = u64at(e + 16);
        return u64at(s.off - 8) === 1 && u64at(e) === 1 && lvl >= 0 && lvl <= 50 && cap >= 100 && cap <= 1e7;
      };
      const st = cand.find(isStadium);
      if (st) t.fields.stadium = { type: 'str', off: st.off, value: st.text };
      // Finances: the last run of three consecutive money-like f64 values before the stadium record
      if (st) {
        const dv = new DataView(p.buffer, p.byteOffset, p.byteLength);
        const norm = off => { const v = dv.getFloat64(off, true); return v >= 1 && v < 1e16; };
        for (let x = st.off - 8; x > Math.max(t.afterLogo, st.off - 3000); x--) {
          if (!norm(x) || !norm(x - 8)) continue;
          let s0 = x - 8;
          if (norm(s0 - 8)) s0 -= 8;
          ['money_1', 'money_2', 'money_3'].forEach((k, i) => { t.fields[k] = { type: 'f64', off: s0 + 8 * i, value: dv.getFloat64(s0 + 8 * i, true) }; });
          break;
        }
      }
      // manager name sits right before the gaming-house block
      const m = cand[cand.length - 1];
      if (m && m !== st) t.fields.manager = { type: 'str', off: m.off, value: m.text };
    }
    return teams;
  }

  // Game date = first two strings of the payload
  function readGameDate(p) {
    try {
      const r = new Reader(p, 0); r.u64();
      const d = r.str(32), t = r.str(32);
      if (DATE_RE.test(d)) return d + ' ' + t;
    } catch (e) { }
    return null;
  }

  // ---------- patching ----------
  // A patch replaces bytes [off, off+len) of the ORIGINAL payload with `bytes`.
  function encodeField(field, value) {
    if (field.type === 'u64') {
      const b = new Uint8Array(8); const dv = new DataView(b.buffer);
      const v = Math.max(0, Math.round(Number(value)));
      dv.setUint32(0, v % 4294967296, true); dv.setUint32(4, Math.floor(v / 4294967296), true);
      return { len: 8, bytes: b };
    }
    if (field.type === 'f64') {
      const b = new Uint8Array(8); new DataView(b.buffer).setFloat64(0, Number(value), true);
      return { len: 8, bytes: b };
    }
    if (field.type === 'str') {
      const oldLen = utf8e.encode(field.value).length;
      const s = utf8e.encode(String(value));
      const b = new Uint8Array(8 + s.length); const dv = new DataView(b.buffer);
      dv.setUint32(0, s.length, true); b.set(s, 8);
      return { len: 8 + oldLen, bytes: b };
    }
    throw new Error('unknown field type ' + field.type);
  }

  function encodeLanguages(langs) {
    const b = new Uint8Array(8 + langs.length * 16); const dv = new DataView(b.buffer);
    dv.setUint32(0, langs.length, true);
    langs.forEach(([id, lv], i) => { dv.setUint32(8 + i * 16, id, true); dv.setUint32(16 + i * 16, lv, true); });
    return b;
  }

  function applyPatches(p, patches) {
    patches = patches.slice().sort((a, b) => a.off - b.off);
    for (let i = 1; i < patches.length; i++)
      if (patches[i].off < patches[i - 1].off + patches[i - 1].len) throw new Error('Overlapping edits at ' + patches[i].off);
    let size = p.length;
    for (const q of patches) size += q.bytes.length - q.len;
    const out = new Uint8Array(size);
    let src = 0, dst = 0;
    for (const q of patches) {
      out.set(p.subarray(src, q.off), dst); dst += q.off - src;
      out.set(q.bytes, dst); dst += q.bytes.length;
      src = q.off + q.len;
    }
    out.set(p.subarray(src), dst);
    return out;
  }

  // Collect patches from records carrying `edits` ({fieldName: newValue}) and optional `langEdits` ([[id, lvl], ...]).
  function collectPatches(records) {
    const patches = [];
    for (const rec of records) {
      if (rec.edits) for (const k in rec.edits) {
        const f = rec.fields[k];
        if (!f) continue;
        const v = rec.edits[k];
        if (f.type === 'str' ? String(v) === f.value : Number(v) === f.value) continue;
        const enc = encodeField(f, v);
        patches.push({ off: f.off, len: enc.len, bytes: enc.bytes });
      }
      if (rec.langEdits) {
        patches.push({ off: rec.langOff, len: rec.langEnd - rec.langOff, bytes: encodeLanguages(rec.langEdits) });
      }
    }
    return patches;
  }

  // ---- champion pool: the career's list of released champions (Vec<String>, release order),
  // followed by the patch version history (Vec<String> like "2026.0.0").
  const u64at = (p, o) => (o + 8 <= p.length ? p[o] + p[o + 1] * 256 + p[o + 2] * 65536 + p[o + 3] * 16777216 + (p[o + 4] + p[o + 5] * 256 + p[o + 6] * 65536 + p[o + 7] * 16777216) * 4294967296 : -1);
  function readStrVec(p, o, maxN, maxLen, test) {
    const n = u64at(p, o); if (n < 0 || n > maxN) return null;
    const out = []; o += 8;
    for (let i = 0; i < n; i++) {
      const l = u64at(p, o); if (l < 1 || l > maxLen || o + 8 + l > p.length) return null;
      let s = ''; for (let j = 0; j < l; j++) { const c = p[o + 8 + j]; if (c < 32 || c > 126) return null; s += String.fromCharCode(c); }
      if (!test(s)) return null;
      out.push(s); o += 8 + l;
    }
    return { list: out, end: o };
  }
  function findChampionPool(p, ids) {
    const idSet = ids instanceof Set ? ids : new Set(ids);
    const VER = /^\d{4}\.\d+\.\d+$/;
    for (const first of ['fighter', 'knight', 'swordman', 'archer']) {
      const needle = new Uint8Array(8 + first.length); needle[0] = first.length;
      for (let i = 0; i < first.length; i++) needle[8 + i] = first.charCodeAt(i);
      let found = null;
      for (let i = p.indexOf(needle[0], 8); i >= 0 && i < p.length; i = p.indexOf(needle[0], i + 1)) {
        let ok = true; for (let j = 1; j < needle.length; j++) if (p[i + j] !== needle[j]) { ok = false; break; }
        if (!ok) continue;
        const pool = readStrVec(p, i - 8, 500, 64, s => idSet.has(s));
        if (!pool || !pool.list.length || new Set(pool.list).size !== pool.list.length) continue;
        // followed by the patch history: Vec<PatchRecord>, each record starting with its version string
        const nPatches = u64at(p, pool.end); if (nPatches < 1 || nPatches > 100000) continue;
        const v = (() => { const l = u64at(p, pool.end + 8); if (l < 5 || l > 32) return null; let s = ''; for (let j = 0; j < l; j++) s += String.fromCharCode(p[pool.end + 16 + j]); return VER.test(s) ? s : null; })();
        if (!v) continue;
        found = { off: i - 8, end: pool.end, ids: pool.list, patches: nPatches, firstPatch: v };
      }
      if (found) return found;
    }
    return null;
  }
  function encodeStrVec(list) {
    const enc = list.map(s => new TextEncoder().encode(s));
    const out = new Uint8Array(8 + enc.reduce((a, b) => a + 8 + b.length, 0));
    const put = (o, n) => { for (let k = 0; k < 8; k++) { out[o + k] = n % 256; n = Math.floor(n / 256); } };
    put(0, list.length); let o = 8;
    for (const b of enc) { put(o, b.length); out.set(b, o + 8); o += 8 + b.length; }
    return out;
  }

  const api = {
    STAT_FIELDS, HIDDEN_FIELDS, FACE_FIELDS, crc32, parseContainer, buildContainer, parseAthlete,
    scanDatabase, scanStrings, readGameDate, encodeField, encodeLanguages, applyPatches, collectPatches, findChampionPool, encodeStrVec,
  };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  else root.TFM2Core = api;
})(typeof window !== 'undefined' ? window : globalThis);
