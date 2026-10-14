/* One reusable Rift Lab frame; the editor owns its visibility, rank and position. */
(function () {
  'use strict';
  let frame, loaded = false, visible = false, options;
  function send() {
    if (loaded) frame.contentWindow.postMessage({ type: 'vesper-editor-state', rank: options.rank, position: options.position, seed: options.seed, visible }, location.origin);
  }
  window.TFM2VesperEditor = {
    mount(container, value) {
      options = value;
      if (!frame) {
        frame = document.createElement('iframe');
        frame.title = 'Vesper — Rift Lab';
        const params = new URLSearchParams({ embed: '1', rank: value.rank, position: value.position, seed: value.seed || 7 });
        frame.src = '/vesper.html?' + params;
        frame.onload = () => { loaded = true; send(); };
        container.append(frame);
      }
      send();
    },
    setVisible(value) { visible = Boolean(value); if (options) send(); },
  };
  window.addEventListener('message', ev => {
    if (ev.origin !== location.origin || !frame || ev.source !== frame.contentWindow) return;
    const data = ev.data;
    if (data?.type === 'vesper-editor-size' && Number.isFinite(data.height)) frame.style.height = Math.max(240, Math.min(9000, data.height)) + 'px';
    if (data?.type === 'vesper-editor-selection' && Number.isInteger(data.rank) && data.rank >= 0 && data.rank <= 7) {
      Object.assign(options, { rank: data.rank, position: data.position, seed: data.seed });
      options.onSelection?.(data);
    }
  });
})();
