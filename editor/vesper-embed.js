/* The Rift Lab inside the Database Editor: compact layout, rank and position from the editor, size reports back. */
(function () {
  'use strict';
  document.addEventListener('DOMContentLoaded', () => {
    const params = new URLSearchParams(location.search);
    if (params.get('embed') !== '1') return;
    const root = document.documentElement, $ = s => document.querySelector(s), main = $('#riftLab');
    root.dataset.editorEmbed = 'true';
    const style = document.createElement('link');
    style.rel = 'stylesheet';
    style.href = 'vesper-embed.css';
    document.head.append(style);
    for (const el of [$('header .back'), $('.hero'), $('footer')]) if (el) el.hidden = true;
    let visible = true, applying = false, lastHeight = 0;
    function visibility() {
      const on = visible && !document.hidden;
      window.VesperLab && window.VesperLab.setVisible(on);
      window.VesperArt && window.VesperArt.setVisible(on);
    }
    function selection() {
      if (applying) return;
      parent.postMessage({ type: 'vesper-editor-selection', rank: Number($('#rank').value), position: Number($('#position').value) || 1, seed: Number($('#seed').value) || 1 }, location.origin);
    }
    function apply(value) {
      applying = true;
      if (typeof value.visible === 'boolean') visible = value.visible;
      const rank = Number(value.rank), position = Number(value.position);
      if (Number.isInteger(rank) && window.VesperLab) {
        if (Number.isInteger(Number(value.seed)) && Number(value.seed) >= 1) $('#seed').value = String(value.seed);
        window.VesperLab.setRank(rank, rank >= 7 ? position : null);
        window.VesperArt && window.VesperArt.setSelection({ rank, position: position || 1 });
      }
      visibility();
      applying = false;
    }
    window.addEventListener('message', ev => {
      if (ev.origin === location.origin && ev.source === parent && ev.data?.type === 'vesper-editor-state') apply(ev.data);
    });
    document.addEventListener('visibilitychange', visibility);
    document.addEventListener('vesper-rank-changed', selection);
    new ResizeObserver(() => {
      const height = Math.ceil(main.getBoundingClientRect().height);
      if (height !== lastHeight) { lastHeight = height; parent.postMessage({ type: 'vesper-editor-size', height }, location.origin); }
    }).observe(main);
    setTimeout(() => apply({ rank: params.get('rank'), position: params.get('position'), seed: params.get('seed') }), 0);
  });
})();
