/* TFM2 champion numbers — locates the champion info sheets inside a database/save payload.
 *
 * The payload stores several copies of the full champion sheet (all 60 champions):
 *   - "base"    : the original numbers the auto-balance patches are measured against
 *   - "current" : the numbers the game uses right now (= latest patch)
 *   - one snapshot per past balance patch (preceded by its version string, e.g. "2028.1.2")
 * Every copy has the same byte layout, so the layout is learned once from a pristine copy
 * (one whose values equal the game's default champion_info) and then applied to all copies.
 */
(function (root) {
  'use strict';
  const STAT = ['attack', 'magic_power', 'hp', 'defence', 'magic_resistance', 'move_speed', 'hp_regen', 'stack', 'crit_chance'];
  const VERSION_RE = /^\d{4}\.\d+\.\d+$/;
  const SKIP_KEYS = new Set(['stat', 'growth', 'category', 'tags', 'id', 'sprite']);

  function u64(p, o) {
    if (o < 0 || o + 8 > p.length) return -1;
    const lo = p[o] | (p[o + 1] << 8) | (p[o + 2] << 16) | (p[o + 3] << 24) >>> 0;
    const hi = p[o + 4] | (p[o + 5] << 8) | (p[o + 6] << 16) | (p[o + 7] << 24);
    if (hi < 0 || hi > 0x1FFFFF) return -1;
    return hi * 4294967296 + (lo >>> 0);
  }
  function encU64s(arr) {
    const b = new Uint8Array(8 * arr.length); const dv = new DataView(b.buffer);
    arr.forEach((v, i) => { dv.setUint32(8 * i, v % 4294967296, true); dv.setUint32(8 * i + 4, Math.floor(v / 4294967296), true); });
    return b;
  }
  const te = new TextEncoder();
  function encStr(s) { const t = te.encode(s); const b = new Uint8Array(8 + t.length); b[0] = t.length & 255; b[1] = t.length >> 8; b.set(t, 8); return b; }

  function indexOf(p, pat, from, to) {
    const end = Math.min(to == null ? p.length : to, p.length) - pat.length;
    const f = pat[0];
    let i = from || 0;
    while (i <= end) {
      i = p.indexOf(f, i);
      if (i < 0 || i > end) return -1;
      let k = 1;
      while (k < pat.length && p[i + k] === pat[k]) k++;
      if (k === pat.length) return i;
      i++;
    }
    return -1;
  }
  const eq = (p, o, pat) => { for (let k = 0; k < pat.length; k++) if (p[o + k] !== pat[k]) return false; return true; };

  // 1) find a copy of the sheet whose stat/growth blocks equal the defaults, champion by champion
  function locatePristine(p, info, names) {
    const pats = names.map(n => ({ s: encU64s(STAT.map(k => info[n].stat[k])), g: encU64s(STAT.map(k => info[n].growth[k])) }));
    let f = -1;
    while ((f = indexOf(p, pats[0].s, f + 1)) >= 0) {
      if (!eq(p, f + 80, pats[0].g)) continue;
      const starts = [f]; let cur = f + 152, ok = true;
      for (let i = 1; i < names.length && ok; i++) {
        let j = cur - 1;
        for (;;) {
          j = indexOf(p, pats[i].s, j + 1, Math.min(f + 120000, cur + 8000));
          if (j < 0) { ok = false; break; }
          if (eq(p, j + 80, pats[i].g)) break;
        }
        if (ok) { starts.push(j); cur = j + 152; }
      }
      if (ok) return starts;
    }
    return null;
  }

  // 2) learn where every numeric field lives (relative to the sheet start)
  function learnLayout(p, info, names, starts) {
    const base = starts[0];
    const champs = [];
    const anchors = []; // strings used to recognise other copies
    names.forEach((n, ci) => {
      const c = info[n], s = starts[ci], e = ci + 1 < names.length ? starts[ci + 1] : s + 8000;
      const fields = [];
      STAT.forEach((k, i) => fields.push({ group: 'stat', key: k, rel: s + 8 * i - base, def: c.stat[k] }));
      STAT.forEach((k, i) => fields.push({ group: 'growth', key: k, rel: s + 80 + 8 * i - base, def: c.growth[k] }));
      let cur = s + 152;
      const walk = (obj, group) => {
        for (const [k, v] of Object.entries(obj)) {
          if (group === '' && SKIP_KEYS.has(k)) continue;
          if (typeof v === 'string') {
            const pat = encStr(v); const j = indexOf(p, pat, cur, e);
            if (j >= 0) { cur = j + pat.length; if (/^[a-z_]{3,}$/.test(v)) anchors.push({ rel: j - base, pat }); }
          } else if (typeof v === 'boolean') {
            if (p[cur] === (v ? 1 : 0)) cur++;
          } else if (typeof v === 'number') {
            if (!Number.isInteger(v) || v < 0) continue;
            if (v === 0) { if (u64(p, cur) === 0) { fields.push({ group: group || 'general', key: k, rel: cur - base, def: 0 }); cur += 8; } continue; }
            let j = cur; while (j < e - 8 && u64(p, j) !== v) j++;
            if (j >= e - 8) continue;
            fields.push({ group: group || 'general', key: k, rel: j - base, def: v }); cur = j + 8;
          } else if (v && typeof v === 'object' && !Array.isArray(v)) {
            walk(v, group ? group + '.' + k : k);
          }
        }
      };
      walk(c, '');
      champs.push({ id: n, fields });
    });
    return { champs, anchors };
  }

  function findCopies(p, anchors) {
    if (!anchors.length) return [];
    const a0 = anchors[0];
    const out = [];
    let i = -1;
    while ((i = indexOf(p, a0.pat, i + 1)) >= 0) {
      const d = i - a0.rel;
      if (anchors.every(a => eq(p, a.rel + d, a.pat))) out.push(d);
    }
    return out;
  }

  function versionBefore(p, start, scanStrings) {
    const strs = scanStrings(p, Math.max(0, start - 9000), start, 16);
    for (let k = strs.length - 1; k >= 0; k--) if (VERSION_RE.test(strs[k].text)) { versionBefore.last = start - strs[k].off; return strs[k].text; }
    return null;
  }

  // Data-driven champions (crossbowman, nightmare, …) are stored as JSON strings after the 60 built-in ones:
  // Vec len u64, then per entry: "1HCM" magic, u32 version, u64 length, JSON bytes
  const MAGIC = [0x31, 0x48, 0x43, 0x4d];
  const td = new TextDecoder();
  function scanModJson(p, from, to) {
    const i = indexOf(p, MAGIC, from, to);
    if (i < 8) return [];
    const count = u64(p, i - 8);
    if (count < 1 || count > 500) return [];
    const out = [];
    let o = i;
    for (let k = 0; k < count; k++) {
      if (!eq(p, o, MAGIC)) break;
      const len = u64(p, o + 8);
      if (len < 2 || o + 16 + len > p.length) break;
      let json;
      try { json = JSON.parse(td.decode(p.subarray(o + 16, o + 16 + len))); } catch (e) { break; }
      out.push({ id: json.id, json, off: o, end: o + 16 + len, head: p.slice(o, o + 8) });
      o += 16 + len;
    }
    return out;
  }

  /** Returns { champions:[{id, fields:[{group,key,def,base,current,offCurrent,offBase}]}], copies, current, base, snapshots } or null */
  function scanChampions(p, info, scanStrings) {
    if (!info) return null;
    const names = Object.keys(info).filter(k => k !== 'mod_champions' && info[k] && info[k].stat && info[k].growth);
    const starts = locatePristine(p, info, names);
    if (!starts) return null;
    const { champs, anchors } = learnLayout(p, info, names, starts);
    const copies = findCopies(p, anchors).sort((a, b) => a - b);
    if (!copies.length) return null;
    const labelled = copies.map(d => ({ start: d, version: versionBefore(p, d, scanStrings), vdist: versionBefore.last }));
    // patch snapshots sit a fixed distance (~5.5 KB) after their version string
    const isSnap = c => c.version && c.vdist > 4000 && c.vdist < 7000;
    const plain = labelled.filter(c => !isSnap(c));
    const snapshots = labelled.filter(isSnap);
    const current = plain.length >= 2 ? plain[1] : plain[0] || labelled[labelled.length - 1];
    const base = plain.length >= 2 ? plain[0] : null;
    // keep only fields whose values look sane in every copy (guards against mis-learned positions)
    for (const c of champs) {
      const refCopies = snapshots.length ? snapshots : labelled;
      c.fields = c.fields.filter(f => refCopies.every(cp => {
        const v = u64(p, cp.start + f.rel);
        if (v < 0) return false;
        return f.def === 0 ? v < 1e7 : v >= f.def * 0.1 && v <= f.def * 20;
      }));
      for (const f of c.fields) {
        f.offCurrent = current.start + f.rel; f.current = u64(p, f.offCurrent);
        if (base) { f.offBase = base.start + f.rel; f.base = u64(p, f.offBase); }
      }
    }
    const lastRel = starts[starts.length - 1] - starts[0];
    const modFor = cp => cp ? scanModJson(p, cp.start + lastRel, cp.start + lastRel + 80000) : [];
    const modCur = modFor(current), modBase = modFor(base);
    const modChampions = modCur.map(m => ({ id: m.id, current: m, base: modBase.find(b => b.id === m.id) || null }));
    return { modChampions, labelled, champions: champs, copies: labelled.length, current, base, snapshots: snapshots.map(s => s.version) };
  }

  const api = { scanChampions, STAT };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  else root.TFM2Champions = api;
})(typeof window !== 'undefined' ? window : globalThis);
