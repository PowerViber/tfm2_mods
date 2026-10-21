/* Shared editor UI: the light/dark theme (remembered per browser) and dropdown enhancement.
   Every <select> stays in the page, so scripts and tests keep reading and setting it; it is visually
   hidden and drawn as a segmented control (few short options), a chip group (a handful), rank chips
   (mastery-rank pickers) or a searchable picker (long lists). Choosing in the control sets the select
   and fires input + change; setting the select from script updates the control.
   Opt out with data-ui="native" on a select or any ancestor. */
(function () {
  'use strict';
  const KEY = 'tfm2-theme', de = document.documentElement;
  const load = () => { try { return localStorage.getItem(KEY); } catch (e) { return null; } };
  const save = v => { try { v ? localStorage.setItem(KEY, v) : localStorage.removeItem(KEY); } catch (e) { /* storage blocked */ } };
  const applyTheme = v => { if (v === 'light' || v === 'dark') de.dataset.theme = v; else delete de.dataset.theme; };
  applyTheme(load());
  addEventListener('storage', e => { if (e.key === KEY) { applyTheme(e.newValue); paintToggle(); } });

  // ------------------------------------------------------------------ theme toggle
  const ICONS = {
    auto: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="8"/><path d="M12 4a8 8 0 0 0 0 16z" fill="currentColor"/></svg>',
    light: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
    dark: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" aria-hidden="true"><path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"/></svg>',
  };
  const NEXT = { auto: 'light', light: 'dark', dark: 'auto' };
  let toggle = null;
  function paintToggle() {
    if (!toggle) return;
    const mode = de.dataset.theme || 'auto';
    toggle.innerHTML = ICONS[mode];
    const label = mode === 'auto' ? 'Theme: follows the system' : 'Theme: ' + mode;
    toggle.title = label + ' (click to change)'; toggle.setAttribute('aria-label', label);
  }
  function mountToggle() {
    if (window.top !== window || document.querySelector('.ui-theme-toggle')) return; // embedded labs follow the host page
    toggle = document.createElement('button');
    toggle.type = 'button'; toggle.className = 'ui-theme-toggle';
    toggle.addEventListener('click', () => { const v = NEXT[de.dataset.theme || 'auto']; applyTheme(v); save(v === 'auto' ? null : v); paintToggle(); });
    const bar = document.querySelector('.topbar');
    if (bar) bar.appendChild(toggle); else { toggle.classList.add('is-floating'); document.body.appendChild(toggle); }
    paintToggle();
  }

  // ------------------------------------------------------------------ select enhancement
  const fire = s => { s.dispatchEvent(new Event('input', { bubbles: true })); s.dispatchEvent(new Event('change', { bubbles: true })); };
  const text = o => (o.label || o.textContent || '').trim();
  function labelOf(s) {
    if (s.getAttribute('aria-label')) return s.getAttribute('aria-label');
    const l = s.closest('label') || (s.id && document.querySelector(`label[for="${CSS.escape(s.id)}"]`));
    if (l) { const t = [...l.childNodes].filter(n => n !== s && !(n.nodeType === 1 && n.matches('select,.ui-seg,.ui-chips,.ui-pick'))).map(n => n.textContent).join(' ').trim(); if (t) return t; }
    const prev = s.previousElementSibling;
    if (prev && /^(SPAN|B|STRONG|SMALL)$/.test(prev.tagName) && prev.textContent.trim().length < 40) return prev.textContent.trim();
    return s.title || s.name || s.id || 'Choose';
  }
  function kindOf(s) {
    const opts = [...s.options], n = opts.length;
    if (n < 2) return null;
    if (s.closest('td, th, .grid') || s.querySelector('optgroup')) return 'pick';
    const len = opts.reduce((a, o) => a + text(o).length, 0), longest = Math.max(...opts.map(o => text(o).length));
    const rank = /rank/i.test(`${s.id} ${s.name} ${s.className}`) || /^(rank|mastery)\b/i.test(labelOf(s));
    if (rank && n <= 10) return 'rank';
    if (n <= 4 && len <= 44 && longest <= 16) return 'seg';
    if (n <= 8 && len <= 120 && longest <= 26) return 'chips';
    return 'pick';
  }

  // radio-style groups: segmented controls, chips, rank chips
  function buildGroup(s, kind) {
    const root = document.createElement('div');
    root.className = kind === 'seg' ? 'ui-seg' : kind === 'rank' ? 'ui-chips ui-rank' : 'ui-chips';
    root.setAttribute('role', 'radiogroup');
    const ui = { root, kind, sig: '' };
    ui.render = () => {
      root.setAttribute('aria-label', labelOf(s));
      root.innerHTML = '';
      [...s.options].forEach((o, i) => {
        const b = document.createElement('button');
        b.type = 'button'; b.setAttribute('role', 'radio'); b.textContent = text(o); b.dataset.value = o.value;
        if (!/^\s*(#?\d|top\b)/i.test(text(o))) b.dataset.step = String(i + 1); // ranks already numbered keep their own label
        if (o.title) b.title = o.title;
        b.disabled = o.disabled;
        root.appendChild(b);
      });
      ui.sync();
    };
    ui.sync = () => {
      const i = s.selectedIndex;
      [...root.children].forEach((b, k) => { const on = k === i; b.setAttribute('aria-checked', on ? 'true' : 'false'); b.tabIndex = on || (i < 0 && k === 0) ? 0 : -1; });
      root.setAttribute('aria-disabled', s.disabled ? 'true' : 'false');
    };
    ui.choose = v => { if (s.value === v) return; s.value = v; ui.sync(); fire(s); };
    ui.focus = () => (root.querySelector('[tabindex="0"]') || root.firstElementChild)?.focus();
    root.addEventListener('click', e => { const b = e.target.closest('button'); if (b && !b.disabled) ui.choose(b.dataset.value); });
    root.addEventListener('keydown', e => {
      const keys = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 };
      if (!(e.key in keys)) return;
      e.preventDefault();
      const bs = [...root.children].filter(b => !b.disabled), at = bs.indexOf(document.activeElement);
      const b = bs[(at + keys[e.key] + bs.length) % bs.length];
      if (b) { ui.choose(b.dataset.value); b.focus(); }
    });
    return ui;
  }

  // searchable picker for long lists
  let openPop = null;
  function closePop(focusBack) {
    if (!openPop) return;
    const p = openPop; openPop = null;
    p.el.remove(); p.ui.btn.setAttribute('aria-expanded', 'false');
    removeEventListener('pointerdown', p.outside, true); removeEventListener('scroll', p.scroll, true); removeEventListener('resize', p.scroll);
    if (focusBack) p.ui.btn.focus();
  }
  function buildPick(s) {
    const root = document.createElement('div');
    root.className = 'ui-pick';
    const btn = document.createElement('button');
    btn.type = 'button'; btn.className = 'ui-pick-btn'; btn.setAttribute('aria-haspopup', 'listbox'); btn.setAttribute('aria-expanded', 'false');
    btn.innerHTML = '<span class="ui-pick-label"></span><span class="ui-pick-chev"></span>';
    root.appendChild(btn);
    if (s.style.width) root.style.width = s.style.width;
    if (s.style.marginTop) root.style.marginTop = s.style.marginTop;
    const ui = { root, btn, kind: 'pick', sig: '' };
    ui.render = () => { btn.setAttribute('aria-label', labelOf(s)); ui.sync(); };
    ui.sync = () => {
      const o = s.options[s.selectedIndex], lab = btn.firstChild;
      lab.textContent = o ? text(o) : '';
      lab.classList.toggle('is-placeholder', !!o && o.value === '' );
      btn.title = o && o.title ? o.title : lab.textContent;
      btn.disabled = s.disabled; root.setAttribute('aria-disabled', s.disabled ? 'true' : 'false');
    };
    ui.choose = v => { if (s.value === v) return; s.value = v; ui.sync(); fire(s); };
    ui.focus = () => btn.focus();
    ui.open = () => {
      closePop();
      const el = document.createElement('div');
      el.className = 'ui-pop'; el.setAttribute('role', 'dialog'); el.setAttribute('aria-label', labelOf(s));
      const many = s.options.length > 8;
      el.innerHTML = (many ? '<input class="ui-pop-q" type="search" placeholder="Search…" aria-label="Search options" autocomplete="off">' : '') + '<div class="ui-pop-list" role="listbox"></div>';
      const list = el.lastChild, q = el.querySelector('.ui-pop-q');
      let rows = [], active = -1;
      const draw = () => {
        const f = q ? q.value.trim().toLowerCase() : '';
        list.innerHTML = ''; rows = [];
        let group = null;
        [...s.options].forEach(o => {
          const t = text(o);
          if (f && !t.toLowerCase().includes(f) && !o.value.toLowerCase().includes(f)) return;
          const g = o.parentElement.tagName === 'OPTGROUP' ? o.parentElement.label : null;
          if (g && g !== group) { const h = document.createElement('div'); h.className = 'ui-grp'; h.textContent = g; list.appendChild(h); }
          group = g;
          const r = document.createElement('div');
          r.className = 'ui-opt'; r.setAttribute('role', 'option'); r.textContent = t; r.dataset.value = o.value;
          r.setAttribute('aria-selected', o.selected ? 'true' : 'false');
          if (o.disabled) r.setAttribute('aria-disabled', 'true');
          if (o.title) r.title = o.title;
          list.appendChild(r); rows.push(r);
        });
        if (!rows.length) list.innerHTML = '<div class="ui-empty">No matches</div>';
        setActive(Math.max(0, rows.findIndex(r => r.getAttribute('aria-selected') === 'true')));
      };
      const setActive = i => {
        if (rows[active]) rows[active].classList.remove('is-active');
        active = Math.min(Math.max(i, 0), rows.length - 1);
        if (rows[active]) { rows[active].classList.add('is-active'); rows[active].scrollIntoView({ block: 'nearest' }); }
      };
      const pick = r => { if (!r || r.getAttribute('aria-disabled') === 'true') return; closePop(true); ui.choose(r.dataset.value); };
      list.addEventListener('click', e => pick(e.target.closest('.ui-opt')));
      list.addEventListener('pointermove', e => { const r = e.target.closest('.ui-opt'); if (r) setActive(rows.indexOf(r)); });
      el.addEventListener('keydown', e => {
        if (e.key === 'ArrowDown') { e.preventDefault(); setActive(active + 1); }
        else if (e.key === 'ArrowUp') { e.preventDefault(); setActive(active - 1); }
        else if (e.key === 'Home' && !q) { e.preventDefault(); setActive(0); }
        else if (e.key === 'End' && !q) { e.preventDefault(); setActive(rows.length - 1); }
        else if (e.key === 'Enter') { e.preventDefault(); pick(rows[active]); }
        else if (e.key === 'Escape') { e.preventDefault(); closePop(true); }
        else if (e.key === 'Tab') closePop(false);
      });
      if (q) q.addEventListener('input', draw);
      document.body.appendChild(el);
      // place below the button (or above when there is no room), inside the viewport
      const r = btn.getBoundingClientRect(), w = Math.min(Math.max(r.width, 240), innerWidth - 16);
      el.style.width = w + 'px';
      el.style.left = Math.min(Math.max(8, r.left), innerWidth - w - 8) + 'px';
      const below = innerHeight - r.bottom - 8, above = r.top - 8;
      if (below < 220 && above > below) { el.style.bottom = (innerHeight - r.top + 4) + 'px'; el.style.maxHeight = Math.min(420, above) + 'px'; }
      else { el.style.top = (r.bottom + 4) + 'px'; el.style.maxHeight = Math.min(420, Math.max(160, below)) + 'px'; }
      draw();
      btn.setAttribute('aria-expanded', 'true');
      const outside = e => { if (!el.contains(e.target) && !btn.contains(e.target)) closePop(false); };
      const scroll = e => { if (!e || !el.contains(e.target)) closePop(false); };
      openPop = { el, ui, outside, scroll };
      addEventListener('pointerdown', outside, true); addEventListener('scroll', scroll, true); addEventListener('resize', scroll);
      (q || list.querySelector('.is-active') || el).focus?.();
      if (!q) { el.tabIndex = -1; el.focus(); }
    };
    btn.addEventListener('click', () => (openPop && openPop.ui === ui ? closePop(true) : ui.open()));
    btn.addEventListener('keydown', e => { if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); ui.open(); } });
    return ui;
  }

  const signature = s => s.options.length + '|' + [...s.options].map(o => o.value + '\u0001' + text(o) + (o.disabled ? '\u0002' : '')).join('\u0003');
  function enhance(s) {
    if (s.multiple || s.size > 1 || s.closest('[data-ui="native"]')) return;
    if (!s.__uiBound) {
      s.__uiBound = true;
      s.addEventListener('change', () => s.__ui && s.__ui.sync());
      s.addEventListener('input', () => s.__ui && s.__ui.sync());
      s.addEventListener('focus', () => s.__ui && s.__ui.focus());
      new MutationObserver(() => schedule(s)).observe(s, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['disabled', 'label', 'value'] });
    }
    const sig = signature(s);
    if (s.__ui && s.__ui.root.isConnected && s.__ui.root.previousElementSibling !== s) s.after(s.__ui.root);
    if (s.__ui && s.__ui.sig === sig && s.__ui.root.isConnected) { s.__ui.sync(); return; }
    const kind = kindOf(s);
    if (s.__ui && s.__ui.kind === kind && s.__ui.root.isConnected) { s.__ui.sig = sig; s.__ui.render(); return; }
    if (s.__ui) { s.__ui.root.remove(); s.__ui = null; }
    if (!kind) { s.classList.remove('ui-native-hidden'); delete s.dataset.uiEnhanced; return; }
    const ui = kind === 'pick' ? buildPick(s) : buildGroup(s, kind);
    ui.sig = sig; ui.select = s; s.__ui = ui;
    ui.root.dataset.uiFor = s.id || '';
    s.after(ui.root);
    s.classList.add('ui-native-hidden'); s.dataset.uiEnhanced = kind; s.tabIndex = -1;
    ui.render();
  }

  // value/selectedIndex/selected set from script: keep the control in step
  const hook = (proto, prop, pick) => {
    const d = Object.getOwnPropertyDescriptor(proto, prop);
    if (!d || !d.set) return;
    Object.defineProperty(proto, prop, { ...d, set(v) { d.set.call(this, v); const s = pick(this); if (s && s.__ui) s.__ui.sync(); } });
  };
  hook(HTMLSelectElement.prototype, 'value', x => x);
  hook(HTMLSelectElement.prototype, 'selectedIndex', x => x);
  hook(HTMLSelectElement.prototype, 'disabled', x => x);
  hook(HTMLOptionElement.prototype, 'selected', o => o.closest('select'));

  const pending = new Set();
  let queued = false;
  function schedule(s) {
    pending.add(s);
    if (queued) return;
    queued = true;
    queueMicrotask(() => { queued = false; const all = [...pending]; pending.clear(); all.forEach(x => x.isConnected && enhance(x)); });
  }
  function scan(node) {
    if (node.nodeType !== 1) return;
    if (node.tagName === 'SELECT') schedule(node); else node.querySelectorAll?.('select').forEach(schedule);
  }

  function start() {
    mountToggle();
    document.querySelectorAll('select').forEach(schedule);
    new MutationObserver(ms => { for (const m of ms) m.addedNodes.forEach(scan); }).observe(document.documentElement, { childList: true, subtree: true });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start); else start();

  window.TFM2UI = { enhance, theme: () => de.dataset.theme || 'auto', setTheme: v => { applyTheme(v); save(v === 'auto' ? null : v); paintToggle(); } };
})();
