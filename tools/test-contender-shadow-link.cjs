const assert = require('node:assert/strict');
const {chromium} = require(require.resolve('playwright', {
  paths: [process.cwd(), require('os').homedir() + '/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules'],
}));

(async () => {
  const browser = await chromium.launch({channel: 'msedge', headless: true});
  try {
    const page = await browser.newPage({viewport: {width: 1500, height: 1080}});
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    const url = 'http://127.0.0.1:7310/editor/contender.html?demo=shadow-link';
    await page.goto(url);
    await page.waitForFunction(() => window.contenderLab?.units?.length === 4 && !document.getElementById('addTargets').disabled);
    assert.equal(await page.locator('#revision').textContent(), 'contender-3.0');
    assert.equal(await page.locator('#scenario').inputValue(), 'shadowLink');
    await page.check('#shadowAlly');

    // Use the visible C button and the normal step loop.
    await page.locator('[data-key=c]').click();
    const live = await page.evaluate(() => {
      const lab = contenderLab;
      const start = {...lab.me.pos};
      for (let i = 0; i < 22; i++) lab.step();
      return {
        state: lab.state.shadow_link,
        shadow: lab.shadow,
        stayed: lab.me.pos.x === start.x && lab.me.pos.y === start.y,
        enemyNames: lab.units.filter(u => lab.state.shadow_link?.linked?.includes(u.id)).map(u => u.name),
      };
    });
    assert.equal(live.state?.phase, 'active', JSON.stringify(live));
    assert.equal(live.shadow.max_hp, 3000);
    assert.deepEqual(live.enemyNames, ['Tank', 'Assassin', 'Creep']);
    assert.equal(live.stayed, true);
    if (process.env.CONTENDER_LINK_PREVIEW) await page.locator('#arena').screenshot({path: process.env.CONTENDER_LINK_PREVIEW});

    const payout = await page.evaluate(() => {
      const lab = contenderLab;
      for (let i = 0; i < 245; i++) lab.step();
      return {
        payout: lab.lastPayout,
        map: lab.world.geometry.registration.matching,
        shadow: lab.shadow,
        cues: lab.records.flatMap(record => record.effects).filter(effect => effect.kind === 'shadow_end').length,
      };
    });
    assert.equal(payout.map, 900);
    assert.equal(payout.shadow, null);
    assert.equal(payout.cues, 1);
    assert.ok(payout.payout.stored > 0 && payout.payout.stored < 3000, JSON.stringify(payout));
    assert.deepEqual(payout.payout.rows.map(({name,raw})=>[name,raw]), [['Tank',900],['Assassin',360],['Creep',195]]);
    assert.ok(payout.payout.rows.every(row => row.hp > 0 && row.hp <= row.raw));
    assert.match(await page.locator('#shadowStatus').textContent(), /Assassin.*360 raw/);

    await page.evaluate(() => contenderLab.reset());
    await page.uncheck('#shadowAlly');
    await page.evaluate(() => {
      const lab = contenderLab;
      const p = lab.units[0].pos;
      lab.step({kind:'s3',aim:{x:p.x+3000,y:p.y}});
      for(let i=0;i<24;i++) lab.step();
      if (!lab.shadow) throw Error('No stationary test shadow');
      lab.units.find(u=>u.name==='Assassin').pos = {x:lab.shadow.pos.x + 120001,y:lab.shadow.pos.y};
      lab.step();
    });
    const links = await page.evaluate(() => contenderLab.units.filter(u=>contenderLab.state.shadow_link?.linked?.includes(u.id)).map(u=>u.name));
    assert.deepEqual(links, ['Tank','Creep']);

    // An old recording opens in the inspector, with its original rules identified.
    await page.locator('#recording').setInputFiles({
      name:'old.json', mimeType:'application/json',
      buffer:Buffer.from(JSON.stringify({version:1,rules_revision:'contender-2.9',records:[{state:{},effects:[]}]})),
    });
    await page.waitForFunction(() => document.getElementById('recordingStatus').textContent.includes('Replay requires original rules contender-2.9'));
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({registeredCells:payout.map, caps:payout.payout.rows, stored:payout.payout.stored, linkBreak:links, oldRecording:true, errors}, null, 2));
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
