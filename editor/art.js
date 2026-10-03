/* Champion art: draws each champion's idle sprite (read by the server from the game's bundle.game_data).
 * Falls back to the two-letter badge when the art isn't available (no server / game folder not found).
 */
(function () {
  'use strict';
  const A = { data: null, loading: null, listeners: [], timer: null };
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const idOf = sprite => String(sprite || '').split('/').pop();

  function load() {
    if (A.loading) return A.loading;
    A.loading = fetch('/api/champ-art', { cache: 'no-store' })
      .then(r => (r.ok ? r.json() : { champions: {} }))
      .then(j => { A.data = j.champions || {}; })
      .catch(() => { A.data = {}; })
      .then(() => { A.listeners.splice(0).forEach(f => { try { f(); } catch (e) { console.warn(e); } }); });
    return A.loading;
  }
  const ready = fn => (A.data ? fn() : (A.listeners.push(fn), load()));
  // base sprites are keyed by champion id, custom mod sprites by their full asset path
  const get = id => (A.data && (A.data[String(id || '')] || A.data[idOf(id)])) || null;
  /* Add or replace a sprite in memory (the sprite editor's result before it is saved). */
  function register(key, art) { if (!A.data) A.data = {}; A.data[key] = art; }

  // style for one frame scaled to fit a box
  function frameStyle(a, f, box) {
    const k = Math.min(box / f.w, box / f.h);
    return `width:${(f.w * k).toFixed(1)}px;height:${(f.h * k).toFixed(1)}px;` +
      `background-image:url('${a.url || '/api/asset?key=' + encodeURIComponent(a.sheet)}');` +
      `background-size:${(a.sw * k).toFixed(1)}px ${(a.sh * k).toFixed(1)}px;` +
      `background-position:${(-f.x * k).toFixed(1)}px ${(-f.y * k).toFixed(1)}px;`;
  }

  /* Small badge for lists. id: champion id or sprite path. */
  function icon(id, label, color, box) {
    box = box || 34;
    const a = get(id); const f = a && a.idle[0];
    if (!f) return `<span class="ico" style="background:${color || '#8b97a8'}">${esc(String(label || '?').slice(0, 2))}</span>`;
    return `<span class="ico art" style="--tint:${color || '#8b97a8'}"><i class="sprite" style="${frameStyle(a, f, box - 4)}"></i></span>`;
  }

  /* Larger animated idle sprite (champion header). Call animate() after inserting it. */
  function hero(id, box) {
    const a = get(id); if (!a || !a.idle.length) return '';
    return `<span class="champ-art" data-art="${esc(A.data && A.data[String(id)] ? id : idOf(id))}" data-box="${box || 72}"><i class="sprite" style="${frameStyle(a, a.idle[0], box || 72)}"></i></span>`;
  }
  function animate(root) {
    clearTimeout(A.timer);
    const el = (root || document).querySelector('.champ-art[data-art]'); if (!el) return;
    const a = get(el.dataset.art); if (!a || a.idle.length < 2) return;
    const box = +el.dataset.box; const i = el.querySelector('.sprite'); let n = 0;
    const step = () => {
      if (!el.isConnected) return;
      n = (n + 1) % a.idle.length; i.setAttribute('style', frameStyle(a, a.idle[n], box));
      A.timer = setTimeout(step, Math.max(60, a.idle[n].d || 100));
    };
    A.timer = setTimeout(step, a.idle[0].d || 100);
  }

  /* Animation tags available on a sprite (null if unknown). */
  const tags = id => { const a = get(id); return a ? a.tags : null; };

  window.TFM2_ART = { load, ready, icon, hero, animate, tags, register, get, has: id => !!get(id) };
})();
