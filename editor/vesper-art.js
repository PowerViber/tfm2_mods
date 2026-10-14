/* Mastery artwork inspector: Vesper's pose at this rank (the scars behind her, body, pose overlay with the rank's reality damage,
 * the animated rank emblem, gauntlet HUD), the nine rift looks
 * (Spark … Worldbreaker, then #1), the chosen family's core and glyphs, link threads, packets and technique
 * animations, the punch hole, swipe, Shatter, Awakening and the eight tech modules, all drawn from the runtime sprites the game uses. */
(function (root) {
  'use strict';
  const D = root.RiftData, VR = root.VesperRenderer;
  const $ = s => document.querySelector(s);
  const LOOK_NAMES = [...D.ranks, D.top_name];
  const tiers = [0, 0, 0, 1, 1, 2, 2, 3];
  let renderer, paused = false, t0 = 0, visible = true, still = null;

  function options(select, rows, value) {
    select.innerHTML = rows.map(([v, t]) => `<option value="${v}">${t}</option>`).join('');
    if (value !== undefined) select.value = String(value);
  }
  function state() {
    const rank = Number($('#artRank').value), pos = Number($('#artPosition').value);
    const top = rank >= 7 ? pos : null;
    const look = rank >= 7 && pos === 1 ? 8 : rank;
    const tier = look === 8 ? 4 : tiers[rank];
    const poseLook = rank < 7 ? rank : pos === 1 ? 8 : pos === 2 ? 10 : pos === 3 ? 9 : 7;
    return { rank, pos, top, look, poseLook, tier, kind: $('#artRift').value, fam: Number($('#artFamily').value), tech: $('#artTech').value, pose: $('#artPose').value, facing: $('#artFacing') ? $('#artFacing').value : 'right' };
  }
  function label(c, text, x, y, color) {
    c.fillStyle = color || '#98aec1';
    c.font = '11px ui-monospace,monospace';
    c.textAlign = 'center';
    c.fillText(text, x, y);
    c.textAlign = 'left';
  }
  function draw(now) {
    if (!visible) return requestAnimationFrame(draw);
    const cv = $('#artCanvas'), c = cv.getContext('2d');
    if (!paused) t0 = now / (1000 / 60);
    const tick = Math.floor(t0), s = state(), missing = [];
    const blit = (tag, x, y, scale) => { if (!renderer.blit(tag, tick, x, y, scale)) missing.push(tag); };
    c.fillStyle = '#0a1220';
    c.fillRect(0, 0, cv.width, cv.height);
    if (!renderer.sprites() && still && still.complete && still.naturalWidth) {
      c.imageSmoothingEnabled = false;
      const k = Math.min(cv.width / still.naturalWidth, cv.height / still.naturalHeight);
      c.drawImage(still, 0, 0, still.naturalWidth * k, still.naturalHeight * k);
      $('#artStatus').textContent = 'Runtime sprites are not generated in this checkout yet; showing the generated preview sheet.';
      return requestAnimationFrame(draw);
    }
    // 1. Vesper at this rank
    const vx = 140, vy = 200, vs = 3;
    const side = s.facing === 'left' ? '_l' : '';
    if (s.rank >= 6) blit((s.rank >= 7 && s.pos === 1 ? 'back_top1' : 'back_wound') + side, vx, vy, vs);
    blit('body_idle', vx, vy, vs);
    blit(s.pose + '_r' + s.poseLook + side, vx, vy, vs);
    blit(s.rank >= 7 ? 'top' + Math.max(1, s.pos) : 'rank' + s.rank, vx, vy, vs);
    for (const tag of ['orbs7_9', 'hex4_6', 'core4', 'stress6']) blit(tag, vx, vy, vs);   // the gauntlet HUD
    if (s.pose === 'punch') blit(`punch_t${s.tier}`, vx, vy, vs);
    if (s.pose === 'shatter') blit(`shatter_t${s.tier}`, vx, vy, vs * .5);
    label(c, (s.rank >= 7 ? (s.pos === 1 ? D.top_name : 'Worldbreaker') + ' · #' + s.pos : LOOK_NAMES[s.look]), vx, 360, '#c9a6ff');
    // 2. the rift at this look, with the family core and its ten glyphs
    const rk = s.kind, line = rk === 'slash' || rk === 'mirror';
    const shape = line ? `rift_${rk}_l${s.look}_a0` : `rift_${rk}_l${s.look}`;
    blit(shape, 400, 130, 2.5);
    blit(`core_f${s.fam}_t${Math.min(3, s.tier)}`, 400, 130, 2.5);
    label(c, D.rifts.find(r => r.id === rk).name + ' · ' + D.families[s.fam].name, 400, 215, D.families[s.fam].color);
    for (let i = 0; i < 10; i++) blit('glyph' + String(s.fam * 10 + i).padStart(2, '0'), 292 + i * 24, 240, 1);
    // every look of this rift, Spark to #1
    for (let l = 0; l < 9; l++) {
      const x = 270 + l * 34, tag = line ? `rift_${rk}_l${l}_a0` : `rift_${rk}_l${l}`;
      if (!renderer.blit(tag, tick, x, 300, 0.7)) missing.push(tag);
      if (l === s.look) { c.strokeStyle = '#ffd36b'; c.strokeRect(x - 16, 284, 32, 32); }
    }
    label(c, 'Spark → Worldbreaker → #1', 405, 340);
    // 3. links, packets and the technique
    blit(`link_l${s.look}_a0`, 720, 70, 1.5);
    label(c, 'link thread', 720, 105);
    for (let a = 0; a < 8; a++) blit(`packet_l${s.look}_a${a}`, 620 + a * 28, 140, 0.8);
    label(c, 'packets (8 headings)', 720, 172);
    blit(`tech_${s.tech}_t${s.tier}`, 720, 250, 2);
    label(c, D.techniques.find(t => t.id === s.tech).name + ' · tier ' + (s.tier === 4 ? '#1' : s.tier), 720, 330, '#ffd36b');
    blit(`shatter_t${s.tier}`, 860, 250, 1.2);
    label(c, 'Shatter', 860, 330);
    // 4. the gauntlet moves: hole, swipe, Awakening and the tech modules
    blit(`hole_l${s.look}`, 560, 230, 1.5);
    label(c, 'punch hole', 560, 275);
    blit(`swipe_t${s.tier}_a1`, 560, 70, 1);
    label(c, 'rift swipe', 560, 125);
    blit(`awaken_t${s.tier}`, 860, 110, 1);
    label(c, 'Awakening', 860, 172);
    for (let m = 0; m < 8; m++) blit(`mod${m}_t${Math.min(3, s.tier)}`, 290 + m * 40, 362, 0.55);
    const shown = 47 - missing.length;
    $('#artStatus').textContent = renderer.sprites()
      ? (missing.length ? `${renderer.sprites()} runtime animations loaded · not drawn yet: ${[...new Set(missing)].slice(0, 6).join(', ')}${missing.length > 6 ? '…' : ''}` : `${renderer.sprites()} runtime animations loaded · every layer of this rank is drawn (${shown}+ layers on screen).`)
      : 'Loading sprites…';
    requestAnimationFrame(draw);
  }
  function init() {
    options($('#artRank'), D.ranks.map((r, i) => [i, i + ' · ' + r]), 7);
    options($('#artPosition'), Array.from({ length: 10 }, (_, i) => [i + 1, '#' + (i + 1) + (i === 0 ? ' — ' + D.top_name : i < 3 ? ' (podium)' : '')]), 1);
    options($('#artRift'), D.rifts.map(r => [r.id, r.name]), 'tear');
    options($('#artFamily'), D.families.map((f, i) => [i, f.name]), 0);
    options($('#artTech'), D.techniques.map(t => [t.id, t.name]), 'kidnap');
    renderer = new VR.Renderer($('#artCanvas'));
    still = new Image();
    still.src = '/vesper-mastery-preview.png';
    renderer.load().then(() => requestAnimationFrame(draw));
    $('#artPause').onclick = () => { paused = !paused; $('#artPause').textContent = paused ? 'Play animation' : 'Pause animation'; };
  }
  root.VesperArt = {
    setVisible(v) { visible = v; },
    setSelection({ rank, position }) {
      if (Number.isInteger(rank)) $('#artRank').value = String(Math.max(0, Math.min(7, rank)));
      if (Number.isInteger(position) && position >= 1) $('#artPosition').value = String(Math.min(10, position));
    },
  };
  document.addEventListener('DOMContentLoaded', init);
})(typeof globalThis !== 'undefined' ? globalThis : this);
