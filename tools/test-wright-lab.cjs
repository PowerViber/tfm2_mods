// Wright Lab in a real browser: the page loads the shared core (wright.wasm) and every skill behaves as in the game.
// Run: node tools/test-wright-lab.cjs   (uses the preinstalled Playwright / Chromium)
const assert = require('assert'), path = require('path'), {spawn} = require('child_process');
const {chromium} = require('playwright');
const root = path.resolve(__dirname, '..'), port = 7321;
(async () => {
  const server = spawn('node', ['server.js', '--port', String(port), '--no-open'], {cwd: path.join(root, 'editor'), stdio: 'ignore'});
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
      // jets: patrol, then boost in and strafe a target on their own
      L.units[0].pos = {x: L.me.pos.x + 50000, y: L.me.pos.y}; const hp = L.units[0].hp;
      L.order('fighters'); run(2); out.carriers = L.carriers.size;
      const phases = new Set(); for (let i = 0; i < 300; i++) { L.step(); (L.state.planes || []).filter(p => p.kind === 'fighter').forEach(p => phases.add(p.phase)); }
      out.phases = [...phases]; out.fighterDamage = L.units[0].hp < hp;
      // S2: a ready-up, then 3 waves; the last stuns
      L.units[0].hp = 1000; L.units[1].hp = 1000;
      L.units[1].pos = {x: L.me.pos.x + 80000, y: L.me.pos.y}; L.units[0].pos = {x: L.me.pos.x + 82000, y: L.me.pos.y + 6000};
      // (his jets are still fighting, so watch the stun: none during the ready-up, then the last wave's)
      const hp1 = L.units[1].hp; L.units[1].stunUntil = 0; L.setAim(L.units[1].pos); L.order('bomber'); run(40); out.readyUp = L.units[1].stunUntil === 0;
      run(80); out.waves = hp1 - L.units[1].hp; out.stunned = L.units[1].stunUntil > 0;
      // repair: the far ally gets the global repair plane
      const ally = L.units[2]; ally.hp = 250; run(300); L.order('repair'); run(400); out.allyRepaired = ally.hp > 250;
      // the carrier drops on Wright (knockback, no damage), follows him and takes 80% of his damage
      run(1300);
      const e = L.units[1]; e.hp = e.max_hp; e.stunUntil = 0; e.pos = {x: L.me.pos.x + 10000, y: L.me.pos.y}; const before = {...e.pos};
      L.setAim({x: 900000, y: 900000}); L.order('carrier'); run(62 + 22);
      out.knocked = Math.hypot(e.pos.x - before.x, e.pos.y - before.y) > 40000; out.noDropDamage = e.hp === e.max_hp;
      out.guard = !!(L.state.carrier && L.state.carrier.guarding);
      L.me.pos = {x: L.me.pos.x - 20000, y: L.me.pos.y}; run(3);
      out.follows = L.state.carrier && Math.abs(L.state.carrier.center.x - Math.round(L.me.pos.x)) < 2;
      const meHp = L.me.hp, pool = L.state.carrier.pool;
      for (let i = 0; i < 5; i++) { L.hitMe(100); run(10); }
      out.meLost = meHp - L.me.hp; out.poolLost = pool - (L.state.carrier ? L.state.carrier.pool : 0);
      out.allyHits = L.hits.filter(h => h.ally).length;
      return out;
    });
    check('The workshop stocks airframes up to 3', r.stock === 3);
    check('Fighter Wing spawns two real, targetable carriers', r.carriers >= 2);
    check(`Jets patrol, boost in and break away (${r.phases})`, r.phases.includes('Run') && r.phases.includes('Break'));
    check('Jets attack on their own', r.fighterDamage);
    check('S2 readies up before the waves', r.readyUp);
    check(`S2 drops its waves and the last one stuns (${r.waves} damage)`, r.waves > 0 && r.stunned);
    check('Repair Wing reaches the far ally', r.allyRepaired);
    check('The Sky Carrier drops on Wright: knockback, no damage, guard on', r.knocked && r.noDropDamage && r.guard);
    check('The carrier follows him', r.follows);
    check(`It takes 80% of his damage (500 raw: he lost ${r.meLost} after its chopper heals, its pool ${r.poolLost})`, r.meLost > 0 && r.meLost <= 100 && r.poolLost === 400);
    check('Nothing of his ever hits an ally', r.allyHits === 0);
    check('No page errors: ' + errors.join(' / '), errors.length === 0);
    console.log(`${checks} checks passed`);
  } finally { await browser.close(); server.kill(); }
})().catch(e => { console.error(e); process.exit(1); });
