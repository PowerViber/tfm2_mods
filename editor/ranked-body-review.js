'use strict';
(() => {
  const $ = id => document.getElementById(id);
  const cache = new Map(), timings = [], cold = performance.now();
  let data, character, playing = false, animationTime = 0, previousTime = 0;
  function url(path) { return '../' + path.split('/').map(encodeURIComponent).join('/'); }
  function image(path) {
    if (!cache.has(path)) cache.set(path, new Promise((resolve, reject) => {
      const im = new Image(); im.onload = () => resolve(im); im.onerror = () => reject(new Error('Asset failed: ' + path)); im.src = url(path);
    }));
    return cache.get(path);
  }
  function selected() {
    return character.proposals.find(p => p.look === Number($('look').value));
  }
  function fill(select, entries, keep) {
    select.replaceChildren(...entries.map(([value, text]) => new Option(text, value)));
    if (keep != null && [...select.options].some(o => o.value === String(keep))) select.value = String(keep);
  }
  function refreshControls() {
    const c = character, oldPose = $('pose').value;
    fill($('look'), c.looks.map((n, i) => [i, n]), $('look').value);
    fill($('pose'), ($('source').value === 'package' ? Object.keys(c.body.anims) : ['idle']).map(p => [p, p]), oldPose);
    const frames = $('source').value === 'package' ? c.body.anims[$('pose').value].frames.length : 1;
    $('frame').max = frames - 1; $('frame').value = 0; animationTime = 0;
    $('play').disabled = $('source').value !== 'package';
    $('lab').href = c.lab; $('lab').textContent = 'Open existing ' + c.name + ' Lab';
    $('rawSource').href = url(c.source);
    render();
  }
  let renderSerial = 0;
  async function render() {
    if (!character) return;
    const serial = ++renderSerial, c = character, proposed = $('source').value !== 'package';
    const canvas = $('stage'), ctx = canvas.getContext('2d'), light = $('terrain').value === 'light';
    ctx.imageSmoothingEnabled = false; ctx.fillStyle = light ? '#b9c7b3' : '#14232c'; ctx.fillRect(0, 0, 640, 360);
    ctx.strokeStyle = light ? '#a6b8a0' : '#1c3541'; ctx.lineWidth = 1;
    for (let x = 0; x < 640; x += 24) { ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, 360); ctx.stroke(); }
    for (let y = 0; y < 360; y += 24) { ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(640, y); ctx.stroke(); }
    let im, frame, path, duration = 0, frames = [], pose = $('pose').value;
    if (proposed) {
      const p = selected();
      $('missing').textContent = p ? 'Approved single-pose reference. Full base animations are under Current package; other ranked animations remain gated.' : 'This appearance is reserved, but its concept is not one of the three approval samples.';
      if (!p) { $('facts').textContent = c.name + ' · ' + c.looks[Number($('look').value)] + '\nApproval pending; no substitute shown.'; return; }
      path = p.files[$('facing').value]; im = await image(path); frame = {x: 0, y: 0, w: im.width, h: im.height};
    } else {
      path = c.body.png; im = await image(path); frames = c.body.anims[pose].frames;
      const i = Math.min(Number($('frame').value), frames.length - 1); frame = frames[i].data; duration = frames[i].duration;
      $('missing').textContent = 'Actual base body from the current package; independent rank overlays and VFX are not composited in this raw-frame inspector.';
    }
    if (serial !== renderSerial) return;
    const start = performance.now(), scale = 4, centre = [320, 180];
    ctx.save(); ctx.translate(...centre);
    if (!proposed && $('facing').value === 'left') ctx.scale(-1, 1);
    ctx.drawImage(im, frame.x, frame.y, frame.w, frame.h, -frame.w * scale / 2, -frame.h * scale / 2, frame.w * scale, frame.h * scale);
    ctx.restore();
    if ($('anchors').checked) {
      const fy = c.feet[1] - c.frame[1] / 2;
      ctx.strokeStyle = '#db7188'; ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(306, 180); ctx.lineTo(334, 180); ctx.moveTo(320, 166); ctx.lineTo(320, 194); ctx.stroke();
      ctx.strokeStyle = '#75d7c7'; ctx.beginPath(); ctx.moveTo(220, 180 + fy * scale); ctx.lineTo(420, 180 + fy * scale); ctx.stroke();
    }
    $('frameLabel').textContent = (Number($('frame').value) + 1) + ' / ' + (proposed ? 1 : frames.length);
    $('facts').textContent = [c.name + ' · ' + (proposed ? c.looks[Number($('look').value)] : 'current base package'),
      'Source: ' + path, 'Facing: ' + $('facing').value + ' · pose: ' + pose,
      'Frame: ' + frame.w + '×' + frame.h + ' · duration: ' + (proposed ? 'concept only' : duration.toFixed(3) + ' s'),
      'World origin: frame centre (pink) · foot reference: teal',
      'Frozen source SHA-256:\n' + (proposed ? c.sourceSha256 : c.body.sha256)].join('\n');
    timings.push(performance.now() - start); if (timings.length > 256) timings.shift();
    const sorted = [...timings].sort((a, b) => a - b);
    $('metrics').textContent = 'Review canvas draw only: warm median ' + sorted[Math.floor(sorted.length / 2)].toFixed(2) + ' ms · p95 ' + sorted[Math.floor((sorted.length - 1) * .95)].toFixed(2) + ' ms. This is not a game-host benchmark.';
  }
  function cards() {
    const light = $('terrain').value === 'light', scale = Number($('scale').value), facing = $('facing').value;
    $('cards').replaceChildren(...data.characters.map(c => {
      const section = document.createElement('section'); section.className = 'panel card'; section.dataset.character = c.key;
      const title = document.createElement('h2'); title.textContent = c.name; section.append(title);
      const p = document.createElement('p'); p.className = 'small'; p.textContent = c.looks.length + ' appearances reserved · approved base / middle / highest concepts'; section.append(p);
      const sheet = new Image(); sheet.className = 'sheet'; sheet.src = url('docs/ranked-body-redesign/' + c.key + '-comparison.png'); sheet.alt = c.name + ' current, Knight and proposals at 1× and 3×'; section.append(sheet);
      const strip = document.createElement('div'); strip.className = 'strip' + (light ? ' light' : ''); strip.dataset.facing = facing;
      c.proposals.forEach(p => { const item = document.createElement('div'); item.className = 'portrait'; const im = new Image(); im.src = url(p.files[facing]); im.width = c.frame[0] * scale; im.height = c.frame[1] * scale; im.alt = p.name + ' ' + facing; item.append(im); strip.append(item); });
      section.append(strip); const motion=document.createElement('details'); const summary=document.createElement('summary'); summary.textContent='Complete base animations · both facings'; motion.append(summary); ['right','left'].forEach(face=>{const im=new Image(); im.src=url('docs/ranked-body-redesign/staged/'+c.key+'-'+face+'.gif'); im.alt=c.name+' complete base animation '+face; im.style.imageRendering='pixelated'; im.style.maxWidth='45%'; motion.append(im)}); section.append(motion); const b = document.createElement('button'); b.textContent = 'Inspect ' + c.name; b.onclick = () => { $('character').value = c.key; character = c; refreshControls(); $('inspection').scrollIntoView({behavior: 'smooth'}); }; section.append(b); return section;
    }));
  }
  function tick(time) {
    if (playing && data && $('source').value === 'package') {
      const frames = character.body.anims[$('pose').value].frames, total = frames.reduce((a, f) => a + f.duration, 0);
      animationTime = (animationTime + Math.min((time - previousTime) / 1000, .1)) % total;
      let t = animationTime, i = 0; while (i < frames.length - 1 && t >= frames[i].duration) t -= frames[i++].duration;
      if (Number($('frame').value) !== i) { $('frame').value = i; render(); }
    }
    previousTime = time; requestAnimationFrame(tick);
  }
  async function boot() {
    const response = await fetch('../docs/ranked-body-redesign/review-data.json'); if (!response.ok) throw new Error('Review data unavailable'); data = await response.json();
    fill($('character'), data.characters.map(c => [c.key, c.name])); character = data.characters[0];
    $('character').onchange = () => { character = data.characters.find(c => c.key === $('character').value); refreshControls(); };
    ['source', 'pose'].forEach(id => $(id).onchange = refreshControls);
    $('look').onchange = render; $('frame').oninput = () => { playing = false; $('play').setAttribute('aria-pressed', 'false'); render(); };
    ['terrain', 'facing', 'scale'].forEach(id => $(id).onchange = () => { cards(); render(); }); $('anchors').onchange = render;
    $('play').onclick = () => { playing = !playing; $('play').setAttribute('aria-pressed', String(playing)); };
    $('budgets').replaceChildren(...data.characters.map(c => { const tr = document.createElement('tr'); [c.name, c.looks.length, (c.declaredDecodedBytes / 2 ** 20).toFixed(2) + ' MiB', c.declaredSheets, c.largestSheet + ' px'].forEach(v => { const td = document.createElement('td'); td.textContent = v; tr.append(td); }); return tr; }));
    data.references.forEach(ref => { const p = document.createElement('p'), a = document.createElement('a'); a.href = ref.page; a.textContent = ref.role; p.append(a); $('refs').append(p); });
    cards(); refreshControls(); await render();
    window.rankArtReview = {data, coldMs: performance.now() - cold, render, select(key, source = 'proposal', look = 0, pose = 'idle') { $('character').value = key; character = data.characters.find(c => c.key === key); $('source').value = source; refreshControls(); $('look').value = look; if ([...$('pose').options].some(o => o.value === pose)) $('pose').value = pose; refreshControls(); return render(); }};
    requestAnimationFrame(tick);
  }
  boot().catch(e => { $('error').textContent = e.message; console.error(e); });
})();
