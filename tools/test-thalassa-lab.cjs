// Thalassa Lab in a real browser: the page loads the shared core (thalassa.wasm) and every skill behaves as in the game.
// Run: node tools/test-thalassa-lab.cjs   (uses the preinstalled Playwright / Chromium)
const assert = require('assert'), path = require('path'), {spawn} = require('child_process');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..'), port = 7319;
(async () => {
  const server = spawn('node', ['server.js', '--port', String(port)], {cwd: path.join(root, 'editor'), stdio: 'ignore'});
  await new Promise(r => setTimeout(r, 1500));
  const browser = await chromium.launch(), page = await browser.newPage({viewport: {width: 1300, height: 950}}), errors = [];
  page.on('pageerror', e => errors.push(e.message));
  let checks = 0;
  const check = (name, ok) => { assert(ok, name); checks++; console.log('ok -', name); };
  try {
    await page.goto(`http://127.0.0.1:${port}/thalassa.html`);
    await page.waitForFunction(() => window.thalassaLab);
    const r = await page.evaluate(() => {
      const L = thalassaLab, out = {};
      L.pause(); L.reset(); document.querySelector('#moving').checked = false;
      const run = n => { for (let i = 0; i < n; i++) L.step(); };
      L.order('tide_pool'); run(16);
      out.pool = L.state.pools.length; out.water = L.state.in_water;
      run(480); L.units[0].pos = {x: 345000, y: 478000}; L.units[1].pos = {x: 350000, y: 490000}; L.setAim({x: 347000, y: 484000}); L.order('tide_pool'); run(16);
      out.whirls = L.state.pools.filter(p => p.whirl).length;
      out.bumped = L.units.filter(u => u.team === 1 && u.stunUntil > L.tick).length;
      L.setAim({x: 340000, y: 480000}); L.order('twin'); run(2); out.twin = !!L.clone;
      L.units[2].stunUntil = L.tick + 60; run(20); out.rescued = !L.clone;
      L.setAim({x: 900000, y: 150000}); L.order('ult'); run(300);
      out.trident = L.state.pools.some(p => p.center.x > 700000);
      const far = L.state.pools.find(p => p.center.x > 700000);
      out.wet = L.state.in_water;
      if (far) { L.setAim(far.center); L.order('dive'); run(120); out.farDive = Math.hypot(L.me.pos.x - far.center.x, L.me.pos.y - far.center.y) <= far.radius; out.farX = L.me.pos.x; }
      return out;
    });
    check('Tide Pool on land forms a pool below her and turns water mode on', r.pool === 1 && r.water);
    check('In water Tide Pool surges into whirlpools (target and below her)', r.whirls >= 2);
    check('Two enemy champions caught by the surge bump and are stunned', r.bumped === 2);
    check('Tide Twin spawns', r.twin);
    check('A stunned ally near the twin gets the rescue pool (the twin is used up)', r.rescued);
    check('The trident crosses the map and leaves a pool where it lands', r.trident);
    check(`Dive follows the trident into its far pool (water ${r.wet}, x ${r.farX})`, r.farDive);
    check('No page errors: ' + errors.join(' / '), errors.length === 0);
    console.log(`${checks} checks passed`);
  } finally { await browser.close(); server.kill(); }
})().catch(e => { console.error(e); process.exit(1); });
