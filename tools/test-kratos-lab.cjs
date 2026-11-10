// Kratos Lab in a real browser, on the real 5v5 map: the page loads the shared core (kratos.wasm) and every skill
// behaves as in the game. Run: node tools/test-kratos-lab.cjs   (uses the preinstalled Playwright / Chromium)
const assert = require('assert'), path = require('path'), {spawn} = require('child_process');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..'), port = 7323;
(async () => {
  const server = spawn('node', ['server.js', '--port', String(port), '--no-open'], {cwd: path.join(root, 'editor'), stdio: 'ignore'});
  await new Promise(r => setTimeout(r, 1500));
  const browser = await chromium.launch(), page = await browser.newPage({viewport: {width: 1300, height: 950}}), errors = [];
  page.on('pageerror', e => errors.push(e.message));
  let checks = 0;
  const check = (name, ok) => { assert(ok, name); checks++; console.log('ok -', name); };
  try {
    await page.goto(`http://127.0.0.1:${port}/kratos.html`);
    await page.waitForFunction(() => window.kratosLab);
    const r = await page.evaluate(() => {
      const L = kratosLab, out = {}, run = n => { for (let i = 0; i < n; i++) L.step(); };
      L.pause(); L.reset(); document.querySelector('#moving').checked = false;
      out.realWalls = L.walls.filter(Boolean).length;
      const foe = L.units.find(u => u.id === 2), tower = L.units.find(u => u.tower), towerHp = tower.hp;
      // S1: immunity + a dodge
      L.order('sight'); run(1); out.sightImmune = L.me.immuneUntil > L.tick;
      L.stunMe(); out.stunRefused = L.me.stunUntil === 0;
      run(100);
      // Rampage on a foe next to him: three hits, a knock-up
      foe.pos = {x: L.me.pos.x + 9000, y: L.me.pos.y}; const hp0 = foe.hp;
      L.setAim(foe.pos); L.order('rampage'); run(80);
      const R = KRATOS_RULES.rampage; out.swings = 2 * (R.cleaveBase + R.cleaveAdRatio) + R.smashBase + R.smashAdRatio; out.rampageDamage = hp0 - foe.hp; out.knockedUp = foe.airborneUntil > 0;
      out.immuneCapOk = L.immuneUsed() <= KRATOS_RULES.immune.capTicks;
      // Guard -> Dash across a real wall cell of the map
      run(400);
      const c = L.wallCrossing(); L.me.pos = {...c.from}; foe.pos = {...c.to}; foe.hp = 1000; L.setAim(foe.pos);
      L.order('guard'); run(1); out.guardShield = L.me.shields.length > 0; out.axe = !L.me.blades;
      L.order('dash'); run(40);
      out.crossed = Math.floor(L.me.pos.x / 32000) > Math.floor(c.from.x / 32000) + 1 && !L.wallAt(L.me.pos);
      out.blades = L.me.blades; out.dashHit = foe.hp < 1000;
      // Fenrir: charges in, recast slams it back
      run(600);
      foe.pos = {x: L.me.pos.x + 40000, y: L.me.pos.y}; foe.hp = 1000; foe.airborneUntil = 0; foe.stunUntil = 0;
      L.setAim({x: L.me.pos.x + 100000, y: L.me.pos.y}); L.order('fenrir'); run(2); out.fenrirOut = !!L.state.fenrir;
      run(48); out.fenrirFirstLeg = foe.hp < 1000;
      L.order('slam'); run(60); out.fenrirSlam = foe.airborneUntil > 0;
      out.towerSpared = tower.hp === towerHp; out.allyHits = L.hits.filter(h => h.ally || h.tower).length;
      return out;
    });
    check(`The Lab loads the game's real wall grid (${r.realWalls} wall cells)`, r.realWalls > 20);
    check("Heimdall's Sight makes him CC-immune: an enemy stun does nothing", r.sightImmune && r.stunRefused);
    check(`Leviathan Rampage: three swings on one foe (${r.rampageDamage} damage) and the finisher's knock-up`, r.rampageDamage >= r.swings && r.rampageDamage < r.swings + 60 && r.knockedUp);
    check('CC immunity stays within 3 s of the last 6 s', r.immuneCapOk);
    check('Spartan Guard shields and swaps to the axe', r.guardShield && r.axe);
    check('Guard -> Chain Dash crosses a real map wall and strikes on the far side (Blades in hand)', r.crossed && r.dashHit && r.blades);
    check("Fenrir's Charge: the wolf charges in and hits", r.fenrirOut && r.fenrirFirstLeg);
    check('The recast slams Fenrir back with a knock-up', r.fenrirSlam);
    check('Towers and allies are never hit by his skills', r.towerSpared && r.allyHits === 0);
    check('No page errors: ' + errors.join(' / '), errors.length === 0);
    console.log(`${checks} checks passed`);
  } finally { await browser.close(); server.kill(); }
})().catch(e => { console.error(e); process.exit(1); });
