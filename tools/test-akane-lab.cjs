// Akane Lab in a real browser, on the real 5v5 map: the page loads the shared core (akane.wasm) and every skill
// behaves as in the game. Run: node tools/test-akane-lab.cjs   (uses the preinstalled Playwright / Chromium)
const assert = require('assert'), path = require('path'), {spawn} = require('child_process');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..'), port = 7331;
(async () => {
  const server = spawn('node', ['server.js', '--port', String(port), '--no-open'], {cwd: path.join(root, 'editor'), stdio: 'ignore'});
  await new Promise(r => setTimeout(r, 1500));
  const browser = await chromium.launch(), page = await browser.newPage({viewport: {width: 1300, height: 950}}), errors = [];
  page.on('pageerror', e => errors.push(e.message));
  let checks = 0;
  const check = (name, ok) => { assert(ok, name); checks++; console.log('ok -', name); };
  try {
    await page.goto(`http://127.0.0.1:${port}/akane.html`);
    await page.waitForFunction(() => window.akaneLab);
    const r = await page.evaluate(() => {
      const L = akaneLab, R = AKANE_RULES, out = {}, run = n => { for (let i = 0; i < n; i++) L.step(); };
      L.pause(); L.reset(); document.querySelector('#moving').checked = false;
      out.realWalls = L.walls.filter(Boolean).length;
      const foe = L.units.find(u => u.id === 2), tower = L.units.find(u => u.tower), towerHp = tower.hp;
      // S1: invisible, invulnerable, move only; no dash
      const before = {...L.me.pos};
      L.order('soul'); run(1);
      out.soul = L.me.soulUntil > L.tick && L.me.invulnUntil > L.tick && !L.seenByEnemy(L.me.pos);
      out.noDash = Math.hypot(L.me.pos.x - before.x, L.me.pos.y - before.y) < 2000;
      L.order('edge'); run(1); out.moveOnly = !L.state.edge;
      run(100);
      // Kunai Flip over a real map wall
      const c = L.wallCrossing(); L.me.pos = {...c.from}; foe.pos = {...c.to}; foe.hp = foe.max_hp; L.setAim(foe.pos);
      L.order('kunai'); run(15); out.marked = L.state.mark && L.state.mark[0] === 2;
      L.order('kunai'); run(40);
      out.crossed = Math.floor(L.me.pos.x / 32000) > Math.floor(c.from.x / 32000) + 1 && !L.wallAt(L.me.pos);
      // Crimson Edge: 5 hits
      foe.hp = foe.max_hp; foe.pos = {x: L.me.pos.x + 12000, y: L.me.pos.y};
      L.order('edge'); run(1);
      for (let i = 0; i < 5; i++) { L.basic(2); run(1); }
      out.burstFx = foe.crimsonUntil > L.tick || foe.hp <= 0;
      out.knocked = foe.airborneUntil > 0 || foe.hp <= 0;
      out.vulnerable = foe.vulnerableUntil > 0 || foe.hp <= 0;
      // a kill drops an orb; standing on it 0.75 s absorbs it and resets the kunai
      foe.deadUntil = 0; foe.hp = 30; foe.pos = {x: L.me.pos.x + 10000, y: L.me.pos.y};
      L.basic(2); run(2);
      out.orb = (L.state.orbs || []).length === 1;
      run(R.orb.armTicks + R.orb.absorbTicks + 3);
      out.absorbed = (L.state.orbs || []).length === 0 && (L.state.ready[0] <= L.tick);
      // Crimson Night: her team hidden beyond 20,000 of enemy eyes; one recast dash
      run(400);
      const other = L.units.find(u => u.id === 3); other.pos = {x: L.me.pos.x + 90000, y: L.me.pos.y}; other.hp = other.max_hp;
      L.order('night'); run(1);
      out.hidden = L.nightOn() && !L.seenByEnemy(L.me.pos);
      L.setAim(other.pos); L.order('night'); run(40);
      out.nightDash = L.state.night_dash_used && other.hp < other.max_hp;
      out.towerSpared = tower.hp === towerHp;
      out.allyHits = L.hits.filter(h => h.ally || h.tower).length;
      return out;
    });
    check(`The Lab loads the game's real wall grid (${r.realWalls} wall cells)`, r.realWalls > 20);
    check('Dismiss & Devour: invisible and invulnerable, no dash', r.soul && r.noDash);
    check('During Dismiss she can only move (Crimson Edge refused)', r.moveOnly);
    check('Kunai Flip marks the foe, and the recast dashes over a real map wall', r.marked && r.crossed);
    check('Crimson Edge: vulnerable, the crimson burst and the finale knock-up', r.vulnerable && r.burstFx && r.knocked);
    check('A kill drops a Soul Orb; absorbing it resets Kunai Flip', r.orb && r.absorbed);
    check('Crimson Night hides her far from enemy eyes, and the recast dash lands', r.hidden && r.nightDash);
    check('Towers and allies are never hit by her skills', r.towerSpared && r.allyHits === 0);
    check('No page errors: ' + errors.join(' / '), errors.length === 0);
    console.log(`${checks} checks passed`);
  } finally { await browser.close(); server.kill(); }
})().catch(e => { console.error(e); process.exit(1); });
