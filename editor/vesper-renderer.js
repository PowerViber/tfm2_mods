/* Rift Lab renderer: draws a VesperCombat arena with this checkout's runtime sprites (vesper_* sheets, the body and
 * its rank layers) and a vector fallback for anything not yet drawn, so the lab always shows every rule. */
(function (root) {
  'use strict';
  const D = root.RiftData, V = root.VesperCombat, M = root.VesperMath;
  const UNITS_PER_PX = 950; // game scale: one sprite pixel covers 950 world units
  const FAM = D.families.map(f => f.color);
  const ALLY = '#7fd1ff', ENEMY = '#ff8a7a';

  class Renderer {
    constructor(canvas) {
      this.canvas = canvas;
      this.ctx = canvas.getContext('2d');
      this.assets = {};
      this.images = new Map();
      this.ready = false;
      this.error = null;
      this.events = [];
      this.camera = null;
    }
    async load() {
      try {
        const json = async src => { const r = await fetch(src); if (!r.ok) throw Error('missing ' + src); return r.json(); };
        const index = await json('/vesper-runtime/index.json').catch(() => ({ sheets: [] }));
        const sheets = await Promise.all(index.sheets.map(s => json('/vesper-runtime/' + s + '.fanim').then(a => [s, a]).catch(() => null)));
        for (const entry of sheets.filter(Boolean)) {
          const [sheet, anim] = entry;
          for (const [tag, a] of Object.entries(anim.anims)) this.assets[tag] = { frames: a.frames, src: '/vesper-runtime/' + sheet + '.png' };
        }
        const body = await json('/vesper-body.fanim').catch(() => null);
        if (body) for (const [tag, a] of Object.entries(body.anims)) this.assets['body_' + tag] = { frames: a.frames, src: '/vesper-body.png' };
        this.ready = true;
      } catch (e) {
        this.error = e.message;
        this.ready = true;
      }
      return this;
    }
    sprites() { return Object.keys(this.assets).length; }
    image(src) {
      if (this.images.has(src)) return this.images.get(src);
      const im = new Image();
      im.onerror = () => { this.error = 'Artwork unavailable: ' + src; };
      im.src = src;
      this.images.set(src, im);
      return im;
    }
    has(tag) { return Boolean(this.assets[tag]); }
    /** Draws one frame of a runtime animation; false when the tag is not drawn yet. */
    blit(tag, ticks, x, y, scale, loop = true, alpha = 1) {
      const a = this.assets[tag];
      if (!a) return false;
      const im = this.image(a.src);
      if (!im.complete || !im.naturalWidth) return false;
      const total = a.frames.reduce((n, f) => n + f.duration * 60, 0) || 1;
      let at = loop ? ((ticks % total) + total) % total : Math.min(Math.max(0, ticks), total - 0.001);
      let f = a.frames[0];
      for (const fr of a.frames) { f = fr; if (at < fr.duration * 60) break; at -= fr.duration * 60; }
      const r = f.data, c = this.ctx;
      c.save();
      c.globalAlpha = alpha;
      c.imageSmoothingEnabled = false;
      c.drawImage(im, r.x, r.y, r.w, r.h, Math.round(x - r.w * scale / 2), Math.round(y - r.h * scale / 2), r.w * scale, r.h * scale);
      c.restore();
      return true;
    }
    /** Fits the camera around everyone and every rift. */
    fit(arena) {
      const pts = [];
      for (const u of arena.units) if (u.f.hp > 0) pts.push([u.f.x, u.f.y]);
      for (const r of arena.core.rifts) pts.push([r.x, r.y], [r.ex, r.ey]);
      if (!pts.length) pts.push([480000, 480000]);
      let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
      for (const [x, y] of pts) { x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y); }
      const cv = this.canvas, aspect = cv.width / cv.height;
      let w = Math.max(360000, (x1 - x0) * 1.25), h = Math.max(240000, (y1 - y0) * 1.35);
      if (w / h > aspect) h = w / aspect; else w = h * aspect;
      const cx = (x0 + x1) / 2, cy = (y0 + y1) / 2;
      const want = { x: cx - w / 2, y: cy - h / 2, w, h };
      // ease so the view never jumps
      if (!this.camera) this.camera = want;
      else for (const k of ['x', 'y', 'w', 'h']) this.camera[k] += (want[k] - this.camera[k]) * 0.08;
      return this.camera;
    }
    sx(x) { return (x - this.camera.x) / this.camera.w * this.canvas.width; }
    sy(y) { return (y - this.camera.y) / this.camera.h * this.canvas.height; }
    toWorld(px, py) { return [Math.round(this.camera.x + px / this.canvas.width * this.camera.w), Math.round(this.camera.y + py / this.canvas.height * this.camera.h)]; }
    px(units) { return units / this.camera.w * this.canvas.width; }
    spriteScale() { return Math.max(0.5, UNITS_PER_PX / (this.camera.w / this.canvas.width)); }
    /** Fx commands from the core become short-lived drawings. */
    note(cmds, tick) {
      for (const c of cmds) if (c.k === 'Fx') this.events.push({ tag: c.tag, x: c.x, y: c.y, unit: c.unit, born: tick, ticks: Math.max(8, c.ticks) });
      if (this.events.length > 200) this.events.splice(0, this.events.length - 200);
    }
    draw(arena, opts = {}) {
      const c = this.ctx, cv = this.canvas, core = arena.core, tick = arena.tick;
      this.fit(arena);
      const look = core.look(), sc = this.spriteScale();
      c.save();
      c.fillStyle = '#0a1220';
      c.fillRect(0, 0, cv.width, cv.height);
      // chain links, Vesper through every node, with each link's efficiency
      const me = arena.unit(0);
      const nodes = core.chainNodes(tick);
      let prev = [me.f.x, me.f.y];
      for (const n of nodes) {
        const keep = M.keepBp(M.dist(prev[0], prev[1], n.x, n.y));
        c.strokeStyle = look === 8 ? '#fff1b8' : FAM[D.dimensions[n.dim].family];
        c.lineWidth = look >= 6 ? 3 : look >= 2 ? 2 : 1;
        c.setLineDash(look === 0 ? [2, 5] : look === 1 ? [6, 4] : []);
        c.beginPath(); c.moveTo(this.sx(prev[0]), this.sy(prev[1])); c.lineTo(this.sx(n.x), this.sy(n.y)); c.stroke();
        c.setLineDash([]);
        c.fillStyle = '#d7e4ef'; c.font = '10px ui-monospace,monospace';
        c.fillText((keep / 100).toFixed(0) + '%', (this.sx(prev[0]) + this.sx(n.x)) / 2 + 4, (this.sy(prev[1]) + this.sy(n.y)) / 2 - 4);
        prev = [n.x, n.y];
      }
      // rifts
      for (const r of core.rifts) {
        if (r.done) continue;
        const fam = D.dimensions[r.dim].family, col = FAM[fam];
        const line = r.kind === V.R.slash || r.kind === V.R.mirror;
        const [X, Y] = line ? [this.sx((r.x + r.ex) / 2), this.sy((r.y + r.ey) / 2)] : [this.sx(r.x), this.sy(r.y)];
        if (!r.opened) {
          // opening delay: a closing ring and a crack line from Vesper
          const left = Math.max(0, r.opens - tick), k = left / 30;
          c.strokeStyle = col; c.globalAlpha = 0.6;
          c.beginPath(); c.arc(X, Y, this.px(r.radius) * (1 + Math.min(2, k)), 0, Math.PI * 2); c.stroke();
          c.setLineDash([3, 6]); c.beginPath(); c.moveTo(this.sx(me.f.x), this.sy(me.f.y)); c.lineTo(X, Y); c.stroke(); c.setLineDash([]);
          c.globalAlpha = 1;
          c.fillStyle = '#ffd36b'; c.font = '10px ui-monospace,monospace'; c.fillText((left / 60).toFixed(2) + 's', X + 6, Y - 6);
          continue;
        }
        if (V.riftHidden(r, tick)) {
          if (!this.blit('seedhint_l' + look, tick, X, Y, sc, true, 0.6)) { c.strokeStyle = '#9fe8ff55'; c.beginPath(); c.arc(X, Y, this.px(r.radius), 0, Math.PI * 2); c.stroke(); }
          continue;
        }
        const kindId = D.rifts[r.kind].id;
        const shape = line ? `rift_${kindId}_l${look}_a${V.heading8(r.ex - r.x, r.ey - r.y) % 4}` : `rift_${kindId}_l${look}`;
        const drawn = this.blit(shape, tick - r.opens, X, Y, sc);
        this.blit(`core_f${fam}_t${Math.min(3, core.tier4())}`, tick - r.opens, X, Y, sc);
        this.blit('glyph' + String(r.dim).padStart(2, '0'), tick - r.opens, X, Y - this.px(r.radius) - 6, Math.max(0.5, sc * 0.6));
        if (!drawn) {
          c.strokeStyle = col; c.fillStyle = col + '33'; c.lineWidth = 2;
          if (line) { c.beginPath(); c.moveTo(this.sx(r.x), this.sy(r.y)); c.lineTo(this.sx(r.ex), this.sy(r.ey)); c.lineWidth = r.kind === V.R.mirror ? 4 : 3; c.stroke(); }
          else { c.beginPath(); c.ellipse(X, Y, this.px(r.radius), this.px(r.radius) * (r.kind === V.R.sinkhole ? 0.55 : 1), 0, 0, Math.PI * 2); c.fill(); c.stroke(); }
          c.lineWidth = 1;
        }
        c.fillStyle = col; c.font = '10px ui-monospace,monospace';
        c.fillText(D.rifts[r.kind].name + ' · ' + D.dimensions[r.dim].name + (r.dim2 !== null ? ' + ' + D.dimensions[r.dim2].name : ''), X + this.px(r.radius) + 4, Y + 4);
        const dwell = Math.max(0, ...[...r.dwell].filter(([id]) => id < 1000000).map(([, n]) => n));
        if (dwell > 0) { c.fillStyle = '#ff8a7a'; c.fillRect(X - 15, Y + this.px(r.radius) + 4, 30 * dwell / D.constants.break_ticks, 3); }
      }
      // fighters
      for (const u of arena.units) {
        if (u.f.hp <= 0) continue;
        const X = this.sx(u.f.x), Y = this.sy(u.f.y);
        const hidden = u.banish > tick || u.invisible > tick;
        c.globalAlpha = hidden ? 0.35 : 1;
        if (u.f.id === 0) {
          const l = core.look(), rank = core.rank, top = core.top;
          const back = rank >= 7 ? `back_r7_p${Math.min(4, Math.max(1, top || 10))}` : 'back_r' + rank;
          this.blit(back, tick, X, Y, sc);
          const body = this.blit('body_idle', tick, X, Y, sc);
          this.blit((core.punching ? 'punch_r' : 'outfit_r') + l, tick, X, Y, sc);
          if (core.awakened(tick)) this.blit('awaken_t' + core.tier4(), tick, X, Y, sc);
          this.blit(rank >= 7 ? 'top' + (top || 10) : 'rank' + rank, tick, X, Y, sc);
          if (!body) { c.fillStyle = '#c9a6ff'; c.beginPath(); c.arc(X, Y, 9, 0, Math.PI * 2); c.fill(); c.fillStyle = '#0a1220'; c.font = 'bold 11px ui-monospace,monospace'; c.fillText('V', X - 4, Y + 4); }
          if (core.aiming) { c.strokeStyle = '#ffd36b'; c.beginPath(); c.arc(X, Y, 14, 0, Math.PI * 2); c.stroke(); }
          if (core.leftBroken(tick) || core.rightBroken(tick)) { c.fillStyle = '#ff5d5d'; c.font = '9px ui-monospace,monospace'; c.fillText(core.leftBroken(tick) ? 'LEFT BROKEN' : 'HEXCORE BROKEN', X - 28, Y + 28); }
        } else {
          c.fillStyle = u.f.team === 0 ? ALLY : ENEMY;
          c.beginPath(); c.arc(X, Y, u.f.max_hp >= 2400 ? 9 : 7, 0, Math.PI * 2); c.fill();
          if (opts.dragging === u.f.id) { c.strokeStyle = '#fff'; c.stroke(); }
        }
        c.globalAlpha = 1;
        c.fillStyle = '#152234'; c.fillRect(X - 14, Y - 20, 28, 3);
        c.fillStyle = u.f.team === 0 ? '#8be3b5' : '#ff9a85'; c.fillRect(X - 14, Y - 20, 28 * Math.max(0, u.f.hp) / u.f.max_hp, 3);
        if (u.stun > tick || u.root > tick) { c.fillStyle = '#dab7fa'; c.font = '9px ui-monospace,monospace'; c.fillText(u.stun > tick ? 'STUN' : 'ROOT', X - 12, Y + 18); }
      }
      // rift punches in flight: the fist's shockwave in the punched dimension's colour
      for (const p of core.packets) {
        const X = this.sx(p.x), Y = this.sy(p.y);
        const dir = p.path.length ? V.heading8(p.path[0][1] - p.x, p.path[0][2] - p.y) : 0;
        if (!this.blit(`packet_l${look}_a${dir}`, tick, X, Y, sc)) { c.fillStyle = look === 8 ? '#fff1b8' : '#ffe08a'; c.beginPath(); c.arc(X, Y, 3, 0, Math.PI * 2); c.fill(); }
        [p.dim, p.fused].filter(d => d !== null).forEach((d, i) => { c.strokeStyle = FAM[D.dimensions[d].family]; c.beginPath(); c.arc(X, Y, 5 + i * 2, 0, Math.PI * 2); c.stroke(); });
      }
      // effects from the core's commands
      this.events = this.events.filter(e => tick - e.born < e.ticks);
      for (const e of this.events) {
        const u = e.unit !== null && e.unit !== undefined ? arena.unit(e.unit) : null;
        const X = this.sx(u ? u.f.x : e.x), Y = this.sy(u ? u.f.y : e.y);
        if (e.tag.startsWith('rift_') || e.tag.startsWith('core_') || e.tag.startsWith('glyph') || e.tag.startsWith('link_') || e.tag.startsWith('packet_') || e.tag.startsWith('seedhint')) continue;
        if (!this.blit(e.tag, tick - e.born, X, Y, sc, false)) {
          c.globalAlpha = 1 - (tick - e.born) / e.ticks;
          c.strokeStyle = e.tag.startsWith('gbreak') || e.tag.startsWith('shatter') ? '#ff5d5d' : e.tag.startsWith('twist') ? '#ffd36b' : e.tag.startsWith('awaken') || e.tag.startsWith('mod') ? '#7fd1ff' : '#c9a6ff';
          c.beginPath(); c.arc(X, Y, 6 + (tick - e.born) / 2, 0, Math.PI * 2); c.stroke();
          if (!e.tag.startsWith('impact') && !e.tag.startsWith('hole') && !e.tag.startsWith('punch')) { c.fillStyle = c.strokeStyle; c.font = '9px ui-monospace,monospace'; c.fillText(e.tag.replace(/_t\d(_a\d)?$|_l\d$/, ''), X + 8, Y - 8); }
          c.globalAlpha = 1;
        }
      }
      // HUD: the left gauntlet's sockets and stress, the right gauntlet's hexorbs and Hexcore
      c.fillStyle = '#0d1828cc'; c.fillRect(8, 8, 300, 66);
      c.font = '11px ui-monospace,monospace'; c.fillStyle = '#d7e4ef';
      c.fillText('ORBS', 16, 24);
      const lb = core.leftBroken(tick);
      core.pure.forEach((s, i) => {
        if (!s.open) return;
        const ready = !lb && tick >= s.ready_at;
        c.fillStyle = ready ? (s.family !== 255 ? FAM[s.family] : '#c9a6ff') : '#2c3d55';
        c.beginPath(); c.arc(64 + i * 18, 20, 6, 0, Math.PI * 2); c.fill();
        c.strokeStyle = s.over ? '#ff9a3c' : '#526a86'; c.stroke();
      });
      c.fillStyle = '#d7e4ef'; c.fillText('STRESS', 16, 44);
      c.fillStyle = lb ? '#ff5d5d' : core.stress >= 90 ? '#ff5d5d' : core.stress >= 70 ? '#ffb84d' : '#7fd1ff';
      c.fillRect(70, 35, 1.6 * Math.min(100, core.stress), 10); c.strokeStyle = '#526a86'; c.strokeRect(70, 35, 160, 10);
      c.fillStyle = '#d7e4ef'; c.fillText(lb ? 'BROKEN' : String(core.stress), 236, 44);
      c.fillText('HEX ' + (core.rightBroken(tick) ? 'BROKEN' : core.awakened(tick) ? 'AWAKE ' + core.corruption : core.completion + '%'), 16, 64);
      for (let i = 0; i < 6; i++) {
        const open = i < core.hexSlots(tick), ready = open && tick >= core.hex[i];
        c.fillStyle = ready ? '#7fd1ff' : open ? '#2c3d55' : '#151f2e';
        c.fillRect(120 + i * 16, 55, 11, 11);
      }
      c.fillText(`${(tick / 60).toFixed(1)}s · ${core.rank >= 7 && core.top === 1 ? D.top_name : D.ranks[core.rank]}${opts.paused ? ' · PAUSED' : ''}`, cv.width - 230, 20);
      if (this.error) { c.fillStyle = '#ff9d92'; c.fillText(this.error, 12, cv.height - 12); }
      c.restore();
    }
  }
  root.VesperRenderer = { Renderer, FAM };
})(typeof globalThis !== 'undefined' ? globalThis : this);
