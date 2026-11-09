// Wright Lab in a real browser: the page loads the shared core (wright.wasm) and every skill behaves as in the game.
// Run: node tools/test-wright-lab.cjs   (uses the preinstalled Playwright / Chromium)
const assert = require('assert'), path = require('path'), {spawn} = require('child_process');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..'), port = 7321;
(async () => {
  const server = spawn('node', ['server.js', '--port', String(port)], {cwd: path.join(root, 'editor'), stdio: 'ignore'});
  await new Promise(r => setTimeout(r, 1500));
  const browser = await chromium.launch(), page = await browser.newPage({viewport: {width: 1300, height: 950}}), errors = [];
  page.on('pageerror', e => errors.push(e.message));
  let checks = 0;
  const check = (name, ok) => { assert(ok, name); checks++; console.log('ok -', name); };
  try {
    await page.goto(`http://127.0.0.1:${port}/wright.html`);
    await page.waitForFunction(() => window.wrightLab);
    const r = await page.evaluate(() => {
      const L = wrightLab, out = {}, run = n => { for (let i = 0; i < n; i++) L.step(); };
      L.pause(); L.reset(); document.querySelector('#moving').checked = false;
      run(500); out.stock = L.state.stock;
      // fighters: real carriers that fight on their own
      L.units[0].pos = {x: L.me.pos.x + 35000, y: L.me.pos.y}; const hp = L.units[0].hp;
      L.order('fighters'); run(2); out.carriers = L.carriers.size;
      run(240); out.fighterDamage = L.units[0].hp < hp;
      // the carpet bomber over the pair
      L.units[1].pos = {x: L.me.pos.x + 80000, y: L.me.pos.y}; L.units[0].pos = {x: L.me.pos.x + 82000, y: L.me.pos.y + 6000};
      const hp1 = L.units[1].hp; L.setAim(L.units[1].pos); L.order('bomber'); run(420); out.bombed = L.units[1].hp < hp1;
      // repair: the far ally gets the global repair plane
      const ally = L.units[2]; ally.hp = 250; run(300); L.order('repair'); run(400); out.allyRepaired = ally.hp > 250;
      // the carrier drop: no damage, a knockback
      // (let his fighters and burns run out first, so only the drop acts on the target)
      run(1300);
      const e = L.units[1]; e.hp = e.max_hp; e.pos = {x: L.me.pos.x + 100000, y: L.me.pos.y}; const before = {...e.pos}, hpBefore = e.hp;
      L.setAim({x: e.pos.x - 5000, y: e.pos.y}); L.order('carrier'); run(62 + 22);
      out.dropMoved = Math.round(Math.hypot(e.pos.x - before.x, e.pos.y - before.y)); out.dropHp = [hpBefore, e.hp];
      out.knocked = out.dropMoved > 20000; out.noDropDamage = e.hp === hpBefore;
      run(120); out.carrierUp = !!L.state.carrier;
      return out;
    });
    check('The workshop stocks airframes up to 3', r.stock === 3);
    check('Fighter Wing spawns two real, targetable carriers', r.carriers >= 2);
    check('Fighters attack on their own', r.fighterDamage);
    check('The carpet bomber hits the target area', r.bombed);
    check('Repair Wing reaches the far ally', r.allyRepaired);
    check(`The Sky Carrier drop knocks back without damage (moved ${r.dropMoved}, hp ${r.dropHp})`, r.knocked && r.noDropDamage);
    check('The carrier hovers after the drop', r.carrierUp);
    check('No page errors: ' + errors.join(' / '), errors.length === 0);
    console.log(`${checks} checks passed`);
  } finally { await browser.close(); server.kill(); }
})().catch(e => { console.error(e); process.exit(1); });
