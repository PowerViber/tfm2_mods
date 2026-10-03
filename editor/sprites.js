/* Sprite editor: a small pixel-art animation editor for champion sprites (Skill Lab → "Edit sprite").
 *
 * The game draws a champion from a sheet PNG plus a .fanim (named animations, each a list of pixel rectangles
 * with a duration). The centre of every frame is the champion's position on the map, and the feet stand about
 * 11 px below it. The editor keeps every frame on one uniform canvas centred on that point, and packs a fresh
 * sheet (one row per animation) when you apply.
 *
 *   TFM2_SPRITE_EDITOR.open({ title, load: async () => ({ img, fanim }) | null, required: [tags],
 *                             starters: { key: { label, png(base64), fanim } }, onApply: ({ png, fanim, w, h }) => {} })
 */
(function () {
  'use strict';
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const ALWAYS = ['idle', 'run', 'hit', 'dead'];
  const SUGGEST = ['idle', 'run', 'attack', 'skill1', 'skill2', 'ult', 'hit', 'dead', 'skill', 'skill_pre', 'skill2_attack', 'ult_pre', 'ult_loop'];
  const FEET = 11; // px below the anchor where the feet stand in base sprites

  const S = {
    doc: null,            // { w, h, anims: [{ tag, frames: [{ img: ImageData, dur }] }] }
    ai: 0, fi: 0,
    tool: 'pencil', size: 1, color: '#000000', alpha: 255,
    zoom: 8, grid: true, onion: true, guides: true,
    undo: [], redo: [],
    playing: true, pscale: 2, ptimer: null, pframe: 0,
    opts: null, dirty: false, root: null,
  };

  // ------------------------------------------------------------------ image helpers
  const blank = (w, h) => new ImageData(w, h);
  const cloneImg = im => new ImageData(new Uint8ClampedArray(im.data), im.width, im.height);
  function hexToRgb(h) { h = h.replace('#', ''); return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)]; }
  const rgbToHex = (r, g, b) => '#' + [r, g, b].map(v => v.toString(16).padStart(2, '0')).join('');
  function toCanvas(im) { const c = document.createElement('canvas'); c.width = im.width; c.height = im.height; c.getContext('2d').putImageData(im, 0, 0); return c; }
  function loadImage(src) { return new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error('image failed to load')); i.src = src; }); }
  const cur = () => S.doc.anims[S.ai];
  const frame = () => cur() && cur().frames[S.fi];

  /** Build a doc from a sheet image + fanim: every frame is centred on a uniform canvas (its centre is the anchor). */
  function docFromSheet(img, fanim) {
    const anims = (fanim && fanim.anims) || {};
    let w = 0, h = 0;
    for (const a of Object.values(anims)) for (const f of a.frames || []) { w = Math.max(w, f.data.w); h = Math.max(h, f.data.h); }
    w = Math.max(16, Math.ceil(w / 2) * 2); h = Math.max(16, Math.ceil(h / 2) * 2);
    const src = document.createElement('canvas'); src.width = img.width; src.height = img.height;
    const sx = src.getContext('2d'); sx.drawImage(img, 0, 0);
    const doc = { w, h, anims: [] };
    for (const [tag, a] of Object.entries(anims)) {
      const frames = (a.frames || []).map(f => {
        const d = f.data; const c = document.createElement('canvas'); c.width = w; c.height = h;
        const x = c.getContext('2d');
        // centre stays the centre: offset by half the size difference (frames have odd or even sizes)
        x.drawImage(src, d.x, d.y, d.w, d.h, Math.round((w - d.w) / 2), Math.round((h - d.h) / 2), d.w, d.h);
        return { img: x.getImageData(0, 0, w, h), dur: Math.round((f.duration || 0.1) * 1000) };
      });
      if (frames.length) doc.anims.push({ tag, frames });
    }
    // keep the common tags first, in a familiar order
    doc.anims.sort((p, q) => (SUGGEST.indexOf(p.tag) + 1 || 99) - (SUGGEST.indexOf(q.tag) + 1 || 99));
    return doc;
  }
  function blankDoc(tags) {
    const w = 64, h = 64;
    return { w, h, anims: [...new Set(ALWAYS.concat(tags || []))].map(tag => ({ tag, frames: [{ img: blank(w, h), dur: 120 }] })) };
  }
  /** Pack the doc into one sheet (a row per animation) + fanim. */
  function pack() {
    const { w, h, anims } = S.doc;
    const cols = Math.max(1, ...anims.map(a => a.frames.length));
    const c = document.createElement('canvas'); c.width = cols * w; c.height = Math.max(1, anims.length) * h;
    const x = c.getContext('2d');
    const fanim = { anims: {} };
    anims.forEach((a, r) => {
      fanim.anims[a.tag] = { frames: a.frames.map((f, i) => {
        x.putImageData(f.img, i * w, r * h);
        return { duration: +(f.dur / 1000).toFixed(3), data: { x: i * w, y: r * h, w, h } };
      }) };
    });
    const url = c.toDataURL('image/png');
    return { png: url.split(',')[1], url, fanim, w: c.width, h: c.height, fw: w, fh: h };
  }

  // ------------------------------------------------------------------ undo
  function snapshot() { return { w: S.doc.w, h: S.doc.h, anims: S.doc.anims.map(a => ({ tag: a.tag, frames: a.frames.map(f => ({ img: f.img, dur: f.dur })) })), ai: S.ai, fi: S.fi }; }
  function pushUndo() { S.undo.push(snapshot()); if (S.undo.length > 200) S.undo.shift(); S.redo = []; S.dirty = true; }
  function restore(snap) { S.doc = { w: snap.w, h: snap.h, anims: snap.anims }; S.ai = Math.min(snap.ai, S.doc.anims.length - 1); S.fi = Math.min(snap.fi, cur() ? cur().frames.length - 1 : 0); }
  function undo() { if (!S.undo.length) return; S.redo.push(snapshot()); restore(S.undo.pop()); renderAll(); }
  function redo() { if (!S.redo.length) return; S.undo.push(snapshot()); restore(S.redo.pop()); renderAll(); }
  /** Before changing the current frame's pixels: remember it and give the frame its own copy. */
  function editFrame() { pushUndo(); const f = frame(); f.img = cloneImg(f.img); return f.img; }

  // ------------------------------------------------------------------ pixel ops
  function setPx(im, x, y, rgba) {
    if (x < 0 || y < 0 || x >= im.width || y >= im.height) return;
    const i = (y * im.width + x) * 4; im.data[i] = rgba[0]; im.data[i + 1] = rgba[1]; im.data[i + 2] = rgba[2]; im.data[i + 3] = rgba[3];
  }
  function getPx(im, x, y) { if (x < 0 || y < 0 || x >= im.width || y >= im.height) return null; const i = (y * im.width + x) * 4; return [im.data[i], im.data[i + 1], im.data[i + 2], im.data[i + 3]]; }
  function brush(im, x, y, rgba) {
    const n = S.size, o = Math.floor((n - 1) / 2);
    for (let dy = 0; dy < n; dy++) for (let dx = 0; dx < n; dx++) {
      const px = x - o + dx, py = y - o + dy;
      setPx(im, px, py, rgba);
      if (S.mirror) setPx(im, im.width - 1 - px, py, rgba);
    }
  }
  function line(im, x0, y0, x1, y1, rgba) {
    const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1; let err = dx + dy;
    for (;;) { brush(im, x0, y0, rgba); if (x0 === x1 && y0 === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x0 += sx; } if (e2 <= dx) { err += dx; y0 += sy; } }
  }
  function fill(im, x, y, rgba) {
    const start = getPx(im, x, y); if (!start) return;
    if (start.every((v, i) => v === rgba[i])) return;
    const same = (px, py) => { const p = getPx(im, px, py); return p && p[0] === start[0] && p[1] === start[1] && p[2] === start[2] && p[3] === start[3]; };
    const stack = [[x, y]];
    while (stack.length) { const [px, py] = stack.pop(); if (!same(px, py)) continue; setPx(im, px, py, rgba); stack.push([px + 1, py], [px - 1, py], [px, py + 1], [px, py - 1]); }
  }
  function shift(src, dx, dy) {
    const out = blank(src.width, src.height);
    for (let y = 0; y < src.height; y++) for (let x = 0; x < src.width; x++) { const p = getPx(src, x, y); if (p[3]) setPx(out, x + dx, y + dy, p); }
    return out;
  }
  function flipH(src) { const out = blank(src.width, src.height); for (let y = 0; y < src.height; y++) for (let x = 0; x < src.width; x++) setPx(out, src.width - 1 - x, y, getPx(src, x, y)); return out; }
  function outline(src) {
    const out = cloneImg(src); const col = [0, 0, 0, 255];
    for (let y = 0; y < src.height; y++) for (let x = 0; x < src.width; x++) {
      if (getPx(src, x, y)[3]) continue;
      if ([[1, 0], [-1, 0], [0, 1], [0, -1]].some(([dx, dy]) => { const p = getPx(src, x + dx, y + dy); return p && p[3] > 0 && !(p[0] === 0 && p[1] === 0 && p[2] === 0); })) setPx(out, x, y, col);
    }
    return out;
  }
  function recolor(from, to) {
    // every frame of every animation
    pushUndo();
    let n = 0;
    for (const a of S.doc.anims) for (const f of a.frames) {
      const im = cloneImg(f.img); const d = im.data; let hit = false;
      for (let i = 0; i < d.length; i += 4) if (d[i + 3] && d[i] === from[0] && d[i + 1] === from[1] && d[i + 2] === from[2]) { d[i] = to[0]; d[i + 1] = to[1]; d[i + 2] = to[2]; n++; hit = true; }
      if (hit) f.img = im;
    }
    return n;
  }
  function palette() {
    const count = new Map();
    for (const a of S.doc.anims) for (const f of a.frames) { const d = f.img.data; for (let i = 0; i < d.length; i += 4) if (d[i + 3] > 200) { const k = rgbToHex(d[i], d[i + 1], d[i + 2]); count.set(k, (count.get(k) || 0) + 1); } }
    return [...count.entries()].sort((p, q) => q[1] - p[1]).slice(0, 48).map(e => e[0]);
  }
  function resizeCanvas(w, h) {
    pushUndo();
    for (const a of S.doc.anims) for (const f of a.frames) {
      const c = document.createElement('canvas'); c.width = w; c.height = h;
      c.getContext('2d').drawImage(toCanvas(f.img), Math.round((w - S.doc.w) / 2), Math.round((h - S.doc.h) / 2));
      f.img = c.getContext('2d').getImageData(0, 0, w, h);
    }
    S.doc.w = w; S.doc.h = h;
  }

  // ------------------------------------------------------------------ rendering
  function html() {
    return `<div class="spe-top">
        <strong class="spe-title">${esc(S.opts.title || 'Sprite editor')}</strong>
        <select id="speStart"><option value="">Start from…</option>${Object.entries(S.opts.starters || {}).map(([k, s]) => `<option value="${esc(k)}">${esc(s.label)}</option>`).join('')}<option value="__blank">Blank sprite</option></select>
        <span class="spacer"></span>
        <button class="chip-btn" id="speUndo" title="Ctrl+Z">↶ Undo</button><button class="chip-btn" id="speRedo" title="Ctrl+Y">↷ Redo</button>
        <button class="chip-btn" id="speExport">Download sheet</button>
        <button class="btn small" id="speCancel">Close</button>
        <button class="btn small primary" id="speApply">Use this sprite</button>
      </div>
      <div class="spe-body">
        <div class="spe-anims"><div class="spe-h">Animations</div><div id="speAnimList"></div>
          <div class="spe-addanim"><input id="speNewTag" list="speTagList" placeholder="new animation tag"><datalist id="speTagList">${SUGGEST.map(t => `<option value="${t}">`).join('')}</datalist><button class="chip-btn" id="speAddAnim">+ Add</button></div>
          <div class="spe-h" style="margin-top:14px">Canvas</div>
          <div class="spe-row"><label>W <input type="number" id="speW" min="16" max="256" step="2"></label><label>H <input type="number" id="speH" min="16" max="256" step="2"></label><button class="chip-btn" id="speResize">Resize</button></div>
          <p class="spe-note">The centre of the canvas is the champion's spot on the map; feet go on the dashed line (${FEET} px below). Base champions are about 34 px tall.</p>
          <div id="speMissing"></div>
        </div>
        <div class="spe-center">
          <div class="spe-stage" id="speStage"><canvas id="speCanvas"></canvas></div>
          <div class="spe-frames"><div id="speFrameList"></div>
            <div class="spe-row"><button class="chip-btn" id="speFrAdd" title="Blank frame after this one">+ Blank</button><button class="chip-btn" id="speFrDup" title="Copy this frame">⧉ Duplicate</button>
              <button class="chip-btn" id="speFrLeft">◀ Move</button><button class="chip-btn" id="speFrRight">Move ▶</button><button class="chip-btn danger" id="speFrDel">Delete frame</button>
              <label class="spe-dur">Duration <input type="number" id="speDur" min="10" max="2000" step="10"> ms</label><button class="chip-btn" id="speDurAll" title="Use this duration for every frame of the animation">All frames</button>
              <label class="chip-btn" title="Draw a PNG into this frame (centred)">Import PNG<input type="file" id="speImport" accept="image/png,image/gif,image/webp" hidden></label></div></div>
        </div>
        <div class="spe-tools">
          <div class="spe-h">Tools</div>
          <div class="spe-toolgrid">${[['pencil', '✎', 'Pencil (B) — Shift+click draws a line'], ['eraser', '⌫', 'Eraser (E)'], ['fill', '◉', 'Fill (G)'], ['picker', '⊙', 'Colour picker (I) — Alt+click with any tool'], ['move', '✥', 'Move the whole frame (M), or use the arrow keys']].map(([k, ic, t]) => `<button class="spe-tool" data-tool="${k}" title="${t}">${ic}<span>${k}</span></button>`).join('')}</div>
          <div class="spe-row"><label>Size <select id="speSize"><option>1</option><option>2</option><option>3</option><option>4</option></select></label><label class="spe-chk"><input type="checkbox" id="speMirror"> Mirror</label></div>
          <div class="spe-row"><input type="color" id="speColor"><span class="spe-hex" id="speHex"></span></div>
          <div class="spe-h">Sprite colours <span class="spe-note" style="font-weight:400">click = use · right-click = replace everywhere with the current colour</span></div>
          <div class="spe-pal" id="spePal"></div>
          <div class="spe-h">Frame</div>
          <div class="spe-row wrap"><button class="chip-btn" id="speFlip">⇋ Flip</button><button class="chip-btn" id="speOutline" title="Black 1-px outline around the drawing">Outline</button><button class="chip-btn" id="speClear">Clear</button><button class="chip-btn" id="speFlipAnim" title="Flip every frame of this animation">⇋ Flip animation</button></div>
          <div class="spe-h">View</div>
          <div class="spe-row wrap"><label class="spe-chk"><input type="checkbox" id="speGrid"> Grid</label><label class="spe-chk"><input type="checkbox" id="speOnion"> Onion skin</label><label class="spe-chk"><input type="checkbox" id="speGuides"> Guides</label>
            <label>Zoom <input type="range" id="speZoom" min="2" max="20"></label></div>
          <div class="spe-h">Preview <button class="chip-btn" id="spePlay"></button> <select id="speScale"><option value="1">1×</option><option value="2">2×</option><option value="3">3×</option><option value="4">4×</option></select></div>
          <div class="spe-preview"><canvas id="spePrev"></canvas></div>
        </div>
      </div>`;
  }
  function renderAll() { renderAnims(); renderFrames(); renderCanvas(); renderSide(); restartPreview(); }
  function renderAnims() {
    const req = new Set(ALWAYS.concat(S.opts.required || []));
    $('#speAnimList').innerHTML = S.doc.anims.map((a, i) => `<div class="spe-anim${i === S.ai ? ' on' : ''}" data-ai="${i}">
        <span class="spe-tag">${esc(a.tag)}</span><span class="spe-cnt">${a.frames.length}</span>
        ${req.has(a.tag) ? '<span class="spe-req" title="The champion uses this animation">●</span>' : ''}
        <button class="spe-x" data-renanim="${i}" title="Rename">✎</button><button class="spe-x" data-delanim="${i}" title="Delete animation">✕</button></div>`).join('');
    const have = new Set(S.doc.anims.map(a => a.tag));
    const miss = [...req].filter(t => !have.has(t));
    $('#speMissing').innerHTML = miss.length ? `<div class="spe-warn">Missing: ${miss.map(t => `<button class="chip-btn" data-addtag="${esc(t)}">+ ${esc(t)}</button>`).join(' ')}</div>` : '';
    $('#speW').value = S.doc.w; $('#speH').value = S.doc.h;
  }
  function renderFrames() {
    const a = cur(); const box = 52;
    $('#speFrameList').innerHTML = a ? a.frames.map((f, i) => `<button class="spe-fr${i === S.fi ? ' on' : ''}" data-fi="${i}" title="Frame ${i + 1} · ${f.dur} ms"><canvas width="${box}" height="${box}" data-thumb="${i}"></canvas><span>${i + 1}</span></button>`).join('') : '';
    if (a) a.frames.forEach((f, i) => {
      const c = document.querySelector(`[data-thumb="${i}"]`); const x = c.getContext('2d'); x.imageSmoothingEnabled = false;
      const k = Math.min(box / S.doc.w, box / S.doc.h); x.drawImage(toCanvas(f.img), (box - S.doc.w * k) / 2, (box - S.doc.h * k) / 2, S.doc.w * k, S.doc.h * k);
    });
    const f = frame(); $('#speDur').value = f ? f.dur : '';
  }
  function renderSide() {
    document.querySelectorAll('.spe-tool').forEach(b => b.classList.toggle('on', b.dataset.tool === S.tool));
    $('#speSize').value = S.size; $('#speColor').value = S.color; $('#speHex').textContent = S.color;
    $('#speGrid').checked = S.grid; $('#speOnion').checked = S.onion; $('#speGuides').checked = S.guides; $('#speZoom').value = S.zoom;
    $('#speMirror').checked = !!S.mirror; $('#spePlay').textContent = S.playing ? '❚❚' : '▶'; $('#speScale').value = S.pscale;
    $('#spePal').innerHTML = palette().map(h => `<button class="spe-sw${h === S.color ? ' on' : ''}" data-sw="${h}" style="background:${h}" title="${h}"></button>`).join('');
  }
  function renderCanvas() {
    const cv = $('#speCanvas'); if (!cv || !S.doc) return;
    const { w, h } = S.doc, z = S.zoom;
    cv.width = w * z; cv.height = h * z;
    const x = cv.getContext('2d'); x.imageSmoothingEnabled = false;
    // checkerboard
    for (let yy = 0; yy < h; yy += 2) for (let xx = 0; xx < w; xx += 2) { x.fillStyle = ((xx + yy) / 2) % 2 ? '#2a2f38' : '#323843'; x.fillRect(xx * z, yy * z, 2 * z, 2 * z); }
    const a = cur(); const f = frame(); if (!f) return;
    if (S.onion && a.frames.length > 1) {
      const prev = a.frames[(S.fi - 1 + a.frames.length) % a.frames.length];
      x.globalAlpha = 0.28; x.drawImage(toCanvas(prev.img), 0, 0, w * z, h * z); x.globalAlpha = 1;
    }
    x.drawImage(toCanvas(f.img), 0, 0, w * z, h * z);
    if (S.grid && z >= 6) {
      x.strokeStyle = 'rgba(255,255,255,.07)'; x.lineWidth = 1; x.beginPath();
      for (let i = 0; i <= w; i++) { x.moveTo(i * z + .5, 0); x.lineTo(i * z + .5, h * z); }
      for (let i = 0; i <= h; i++) { x.moveTo(0, i * z + .5); x.lineTo(w * z, i * z + .5); }
      x.stroke();
    }
    if (S.guides) {
      const cx = (w / 2) * z, cy = (h / 2) * z;
      x.strokeStyle = 'rgba(255,90,90,.75)'; x.lineWidth = 1; x.beginPath();
      x.moveTo(cx - 3 * z, cy + .5); x.lineTo(cx + 3 * z, cy + .5); x.moveTo(cx + .5, cy - 3 * z); x.lineTo(cx + .5, cy + 3 * z); x.stroke();
      x.setLineDash([4, 4]); x.strokeStyle = 'rgba(120,220,255,.6)'; x.beginPath();
      x.moveTo(0, (h / 2 + FEET + 1) * z + .5); x.lineTo(w * z, (h / 2 + FEET + 1) * z + .5); x.stroke(); x.setLineDash([]);
    }
  }
  function restartPreview() {
    clearTimeout(S.ptimer); S.pframe = 0; drawPreview();
  }
  function drawPreview() {
    const cv = $('#spePrev'); if (!cv || !S.doc) return;
    const a = cur(); if (!a) return;
    const k = S.pscale; cv.width = S.doc.w * k; cv.height = S.doc.h * k;
    const x = cv.getContext('2d'); x.imageSmoothingEnabled = false;
    x.fillStyle = '#3b4a3a'; x.fillRect(0, 0, cv.width, cv.height); // grass-ish, like the map
    const i = S.playing ? S.pframe % a.frames.length : S.fi;
    x.drawImage(toCanvas(a.frames[i].img), 0, 0, cv.width, cv.height);
    if (S.playing) S.ptimer = setTimeout(() => { S.pframe++; drawPreview(); }, Math.max(16, a.frames[i].dur));
  }

  // ------------------------------------------------------------------ canvas input
  let stroke = null;
  function cellAt(e) {
    const cv = $('#speCanvas'); const r = cv.getBoundingClientRect();
    return [Math.floor((e.clientX - r.left) / (r.width / S.doc.w)), Math.floor((e.clientY - r.top) / (r.height / S.doc.h))];
  }
  function rgbaNow(erase) { return erase ? [0, 0, 0, 0] : hexToRgb(S.color).concat([255]); }
  function pick(x, y) {
    const p = getPx(frame().img, x, y); if (!p) return;
    if (p[3] === 0) { S.tool = 'eraser'; } else { S.color = rgbToHex(p[0], p[1], p[2]); if (S.tool === 'picker' || S.tool === 'eraser') S.tool = 'pencil'; }
    renderSide();
  }
  function onDown(e) {
    if (!frame() || e.button > 0) return;
    e.preventDefault();
    const [x, y] = cellAt(e);
    if (e.altKey || S.tool === 'picker') return pick(x, y);
    if (S.tool === 'fill') { const im = editFrame(); fill(im, x, y, rgbaNow(false)); return afterEdit(); }
    if (S.tool === 'move') { pushUndo(); stroke = { move: true, x0: x, y0: y, base: frame().img }; return; }
    const im = editFrame(); const col = rgbaNow(S.tool === 'eraser');
    if (e.shiftKey && S.last) line(im, S.last[0], S.last[1], x, y, col); else brush(im, x, y, col);
    stroke = { x, y, col }; S.last = [x, y]; renderCanvas();
  }
  function onMove(e) {
    if (!stroke) return;
    const [x, y] = cellAt(e);
    if (stroke.move) { frame().img = shift(stroke.base, x - stroke.x0, y - stroke.y0); return renderCanvas(); }
    if (x === stroke.x && y === stroke.y) return;
    line(frame().img, stroke.x, stroke.y, x, y, stroke.col); stroke.x = x; stroke.y = y; S.last = [x, y]; renderCanvas();
  }
  function onUp() { if (!stroke) return; stroke = null; afterEdit(); }
  function afterEdit() { S.dirty = true; renderCanvas(); renderFrames(); renderSide(); }

  // ------------------------------------------------------------------ actions
  function addAnim(tag) {
    tag = (tag || '').trim().replace(/[^A-Za-z0-9_]/g, '_');
    if (!tag) return;
    if (S.doc.anims.some(a => a.tag === tag)) { S.ai = S.doc.anims.findIndex(a => a.tag === tag); S.fi = 0; return renderAll(); }
    pushUndo();
    // start from the idle pose so there's something to adjust
    const idle = S.doc.anims.find(a => a.tag === 'idle');
    S.doc.anims.push({ tag, frames: [{ img: idle ? idle.frames[0].img : blank(S.doc.w, S.doc.h), dur: 100 }] });
    S.ai = S.doc.anims.length - 1; S.fi = 0; renderAll();
  }
  async function importInto(file) {
    const url = URL.createObjectURL(file);
    try {
      const img = await loadImage(url);
      const im = editFrame(); const c = toCanvas(im); const x = c.getContext('2d'); x.imageSmoothingEnabled = false;
      const k = Math.min(1, S.doc.w / img.width, S.doc.h / img.height);
      const w = Math.round(img.width * k), h = Math.round(img.height * k);
      x.drawImage(img, Math.round((S.doc.w - w) / 2), Math.round((S.doc.h - h) / 2), w, h);
      frame().img = x.getImageData(0, 0, S.doc.w, S.doc.h); afterEdit();
      if (k < 1) toast(`The image was bigger than the canvas, so it was scaled down to ${w}×${h}. Resize the canvas first to keep it pixel-exact.`, '');
    } catch (e) { toast('Could not read that image: ' + e.message, 'err'); } finally { URL.revokeObjectURL(url); }
  }
  async function start(key) {
    if (!key) return;
    if (S.dirty && !confirm('Replace the sprite you are editing? (Undo can bring it back.)')) return;
    if (S.doc) pushUndo();
    if (key === '__blank') S.doc = blankDoc(S.opts.required);
    else {
      const s = S.opts.starters[key];
      const img = await loadImage('data:image/png;base64,' + s.png);
      S.doc = docFromSheet(img, s.fanim);
    }
    S.ai = 0; S.fi = 0; S.dirty = true; renderAll();
  }
  function toast(msg, kind) { if (S.opts.toast) S.opts.toast(msg, kind, 7000); else console.log(msg); }
  function close(force) {
    if (!force && S.dirty && !confirm('Close the sprite editor and discard the changes you have not applied?')) return;
    clearTimeout(S.ptimer); document.removeEventListener('keydown', onKey, true); window.removeEventListener('mouseup', onUp);
    S.root.remove(); S.root = null; S.doc = null;
  }
  function onKey(e) {
    if (!S.root) return;
    if (e.target && /INPUT|SELECT|TEXTAREA/.test(e.target.tagName) && e.target.type !== 'checkbox' && e.target.type !== 'range') return;
    const k = e.key.toLowerCase();
    if ((e.ctrlKey || e.metaKey) && k === 'z') { e.preventDefault(); return e.shiftKey ? redo() : undo(); }
    if ((e.ctrlKey || e.metaKey) && k === 'y') { e.preventDefault(); return redo(); }
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const tools = { b: 'pencil', e: 'eraser', g: 'fill', i: 'picker', m: 'move' };
    if (tools[k]) { S.tool = tools[k]; return renderSide(); }
    if (k === '[' || k === ',') { S.fi = (S.fi - 1 + cur().frames.length) % cur().frames.length; return renderAll(); }
    if (k === ']' || k === '.') { S.fi = (S.fi + 1) % cur().frames.length; return renderAll(); }
    const arrows = { arrowleft: [-1, 0], arrowright: [1, 0], arrowup: [0, -1], arrowdown: [0, 1] };
    if (arrows[k] && frame()) { e.preventDefault(); pushUndo(); frame().img = shift(frame().img, ...arrows[k]); return afterEdit(); }
    if (k === 'escape') close();
  }
  function onClick(e) {
    const t = e.target.closest('button, [data-ai]') || e.target; const a = cur();
    if (t.dataset.renanim) { const i = +t.dataset.renanim; const n = prompt('New tag name:', S.doc.anims[i].tag); if (n && n.trim()) { pushUndo(); S.doc.anims[i].tag = n.trim().replace(/[^A-Za-z0-9_]/g, '_'); renderAll(); } return; }
    if (t.dataset.delanim) { const i = +t.dataset.delanim; if (!confirm(`Delete the animation "${S.doc.anims[i].tag}"?`)) return; pushUndo(); S.doc.anims.splice(i, 1); S.ai = Math.max(0, Math.min(S.ai, S.doc.anims.length - 1)); S.fi = 0; return renderAll(); }
    if (t.dataset.ai != null) { S.ai = +t.dataset.ai; S.fi = 0; return renderAll(); }
    if (t.dataset.fi != null) { S.fi = +t.dataset.fi; renderFrames(); renderCanvas(); if (!S.playing) drawPreview(); return; }
    if (t.dataset.addtag) return addAnim(t.dataset.addtag);
    if (t.dataset.tool) { S.tool = t.dataset.tool; return renderSide(); }
    if (t.dataset.sw) { S.color = t.dataset.sw; if (S.tool === 'eraser' || S.tool === 'picker') S.tool = 'pencil'; return renderSide(); }
    switch (t.id) {
      case 'speUndo': return undo();
      case 'speRedo': return redo();
      case 'speCancel': return close();
      case 'speApply': {
        const have = new Set(S.doc.anims.map(x => x.tag)); const miss = ALWAYS.concat(S.opts.required || []).filter(x => !have.has(x));
        if (miss.length && !confirm(`This sprite has no ${miss.join(', ')} animation. The champion would show nothing for those moments. Use it anyway?`)) return;
        const out = pack(); S.opts.onApply(out); S.dirty = false; return close(true);
      }
      case 'speExport': { const out = pack(); const aEl = document.createElement('a'); aEl.href = out.url; aEl.download = (S.opts.fileBase || 'sprite') + '#sheet.png'; aEl.click();
        const b = new Blob([JSON.stringify(out.fanim, null, 1)], { type: 'application/json' }); const a2 = document.createElement('a'); a2.href = URL.createObjectURL(b); a2.download = (S.opts.fileBase || 'sprite') + '#anim.fanim'; a2.click(); return; }
      case 'speAddAnim': { addAnim($('#speNewTag').value); $('#speNewTag').value = ''; return; }
      case 'speResize': { const w = Math.max(16, Math.min(256, +$('#speW').value | 0)), h = Math.max(16, Math.min(256, +$('#speH').value | 0)); if (w === S.doc.w && h === S.doc.h) return; resizeCanvas(w, h); return renderAll(); }
      case 'speFrAdd': pushUndo(); a.frames.splice(S.fi + 1, 0, { img: blank(S.doc.w, S.doc.h), dur: frame().dur }); S.fi++; return renderAll();
      case 'speFrDup': pushUndo(); a.frames.splice(S.fi + 1, 0, { img: frame().img, dur: frame().dur }); S.fi++; return renderAll();
      case 'speFrDel': if (a.frames.length < 2) return toast('An animation needs at least one frame — delete the animation instead.', ''); pushUndo(); a.frames.splice(S.fi, 1); S.fi = Math.min(S.fi, a.frames.length - 1); return renderAll();
      case 'speFrLeft': if (S.fi > 0) { pushUndo(); [a.frames[S.fi - 1], a.frames[S.fi]] = [a.frames[S.fi], a.frames[S.fi - 1]]; S.fi--; renderAll(); } return;
      case 'speFrRight': if (S.fi < a.frames.length - 1) { pushUndo(); [a.frames[S.fi + 1], a.frames[S.fi]] = [a.frames[S.fi], a.frames[S.fi + 1]]; S.fi++; renderAll(); } return;
      case 'speDurAll': { const d = +$('#speDur').value; if (!(d > 0)) return; pushUndo(); a.frames.forEach(f => (f.dur = d)); return renderAll(); }
      case 'speFlip': pushUndo(); frame().img = flipH(frame().img); return afterEdit();
      case 'speFlipAnim': pushUndo(); a.frames.forEach(f => (f.img = flipH(f.img))); return renderAll();
      case 'speOutline': pushUndo(); frame().img = outline(frame().img); return afterEdit();
      case 'speClear': pushUndo(); frame().img = blank(S.doc.w, S.doc.h); return afterEdit();
      case 'spePlay': S.playing = !S.playing; renderSide(); return restartPreview();
    }
  }
  function onChange(e) {
    const t = e.target;
    switch (t.id) {
      case 'speStart': { const v = t.value; t.value = ''; return start(v); }
      case 'speSize': S.size = +t.value; return;
      case 'speMirror': S.mirror = t.checked; return;
      case 'speColor': S.color = t.value; if (S.tool === 'eraser' || S.tool === 'picker') S.tool = 'pencil'; return renderSide();
      case 'speGrid': S.grid = t.checked; return renderCanvas();
      case 'speOnion': S.onion = t.checked; return renderCanvas();
      case 'speGuides': S.guides = t.checked; return renderCanvas();
      case 'speScale': S.pscale = +t.value; return restartPreview();
      case 'speDur': { const d = +t.value; if (d > 0 && frame()) { pushUndo(); frame().dur = d; renderFrames(); } return; }
      case 'speImport': { const f = t.files && t.files[0]; t.value = ''; if (f) importInto(f); return; }
    }
  }
  function fitZoom() {
    const st = $('#speStage'); if (!st || !S.doc) return;
    const z = Math.floor(Math.min((st.clientWidth - 16) / S.doc.w, (st.clientHeight - 16) / S.doc.h));
    S.zoom = Math.max(2, Math.min(20, z || 8));
  }

  async function open(opts) {
    if (S.root) close(true);
    S.opts = opts; S.undo = []; S.redo = []; S.dirty = false; S.ai = 0; S.fi = 0; S.tool = 'pencil'; S.last = null;
    const root = document.createElement('div'); root.className = 'spe'; root.innerHTML = '<p class="muted" style="padding:30px">Loading sprite…</p>';
    document.body.appendChild(root); S.root = root;
    let loaded = null;
    try { loaded = opts.load ? await opts.load() : null; } catch (e) { toast('Could not load the current sprite: ' + e.message, 'err'); }
    if (!S.root) return;
    S.doc = loaded ? docFromSheet(loaded.img, loaded.fanim) : blankDoc(opts.required);
    root.innerHTML = html();
    const cv = $('#speCanvas');
    cv.addEventListener('mousedown', onDown); cv.addEventListener('mousemove', onMove); window.addEventListener('mouseup', onUp);
    cv.addEventListener('contextmenu', e => e.preventDefault());
    root.addEventListener('click', onClick); root.addEventListener('change', onChange);
    root.addEventListener('input', e => { if (e.target.id === 'speZoom') { S.zoom = +e.target.value; renderCanvas(); } });
    root.addEventListener('contextmenu', e => {
      const sw = e.target.closest('[data-sw]'); if (!sw) return; e.preventDefault();
      if (sw.dataset.sw === S.color) return toast('Pick a different colour first (colour box or another swatch), then right-click the swatch you want to replace.', '');
      if (!confirm(`Replace ${sw.dataset.sw} with ${S.color} in every frame?`)) return;
      const n = recolor(hexToRgb(sw.dataset.sw), hexToRgb(S.color)); renderAll(); toast(`Recoloured ${n} pixels.`, 'ok');
    });
    document.addEventListener('keydown', onKey, true);
    // start on the first animation the champion uses
    const want = (opts.required || [])[0];
    const i = S.doc.anims.findIndex(a => a.tag === 'idle'); S.ai = i >= 0 ? i : 0;
    fitZoom(); renderAll();
    if (!loaded && opts.starters && Object.keys(opts.starters).length) toast('Pick a starting sprite from “Start from…”, or draw from scratch.', '');
    return want;
  }

  window.TFM2_SPRITE_EDITOR = { open, _state: S, _pack: pack };
})();
