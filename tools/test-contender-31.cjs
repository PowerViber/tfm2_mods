const assert = require('node:assert/strict');
const {chromium} = require(require.resolve('playwright', {
  paths: [process.cwd(), require('os').homedir() + '/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules'],
}));
const base = process.env.CONTENDER_LAB_URL || 'http://127.0.0.1:7311';

(async () => {
  const browser = await chromium.launch({channel:'msedge',headless:true});
  try {
    const page = await browser.newPage({viewport:{width:1500,height:1080}});
    const errors=[];
    page.on('pageerror', e=>errors.push(e.message));
    await page.goto(base+'/editor/contender.html?demo=shadow-link');
    await page.waitForFunction(()=>window.contenderLab?.units?.length===4 && !document.getElementById('addTargets').disabled);
    assert.equal(await page.locator('#revision').textContent(),'contender-3.1.2');
    assert.equal(await page.locator('[data-key=c]').count(),0);
    assert.equal(await page.evaluate(()=>contenderLab.world.geometry.registration.matching),900);
    await page.locator('[data-key=e]').click();
    const run=await page.evaluate(()=>{
      const lab=contenderLab,start=lab.tick-1;
      for(let i=0;i<60;i++)lab.step();
      const punches=lab.records.flatMap(row=>row.effects
        .filter(effect=>effect.kind==='damage'&&effect.source==='flurry').map(effect=>[row.frame.tick,effect.target]));
      return {start,punches,gap:lab.state.flurry_cast_at,charges:lab.state.flurry_charges};
    });
    assert.equal(run.punches.length,10,JSON.stringify(run));
    assert.equal(run.punches[9][0]-run.start,60);
    assert.equal(run.gap-run.punches[9][0],45);
    assert.equal(run.charges,2);
    const cross=await page.evaluate(()=>{
      const lab=contenderLab,from=lab.me.pos;
      const angles=Array.from({length:24},(_,i)=>i*Math.PI/12);
      const at=angles.map(angle=>({x:Math.round(from.x+45000*Math.cos(angle)),y:Math.round(from.y+45000*Math.sin(angle))}))
        .find(pos=>lab.world.clear(from,pos));
      if(!at)throw Error('No clear Shadow Cross path in the fixture');
      const foe=lab.units.find(u=>u.name==='Tank');
      for(const u of lab.units)if(u!==foe)u.targetable=false;
      foe.pos=at;
      const before=lab.state.flurry_charge_at;
      lab.step({kind:'s1'});
      for(let i=0;i<61;i++)lab.step();
      const hits=lab.records.flatMap(row=>row.effects.filter(e=>e.kind==='damage'&&e.source==='merge'));
      return {targets:hits.map(hit=>hit.target),foe:foe.id,credit:before-lab.state.flurry_charge_at,
        distance:Math.hypot(at.x-from.x,at.y-from.y)};
    });
    assert.deepEqual(cross.targets,[cross.foe]);
    assert.equal(cross.credit,45);
    assert.ok(cross.distance>30000&&cross.distance<50000);
    await page.locator('#arena').focus();
    await page.keyboard.press('Shift+E');
    await page.waitForFunction(()=>contenderLab.state.flurry_charges===3);
    assert.equal(await page.evaluate(()=>contenderLab.state.charges),4);
    assert.equal(await page.evaluate(()=>contenderLab.state.flurry_cast_at),run.gap);
    await page.evaluate(()=>contenderLab.pause());
    await page.locator('[data-key=exchange]').click();
    assert.equal(await page.evaluate(()=>contenderLab.state.charges),4);
    assert.match(await page.locator('#log').textContent(),/Flurry charges are full/);
    await page.evaluate(()=>contenderLab.reset());
    const assassin=await page.evaluate(()=>{
      const lab=contenderLab,u=lab.units.find(u=>u.name==='Assassin');
      u.pos={x:lab.me.pos.x+15000,y:lab.me.pos.y};
      return {id:u.id,hp:u.hp};
    });
    await page.selectOption('#counterAttacker',String(assassin.id));
    await page.click('#simulateHit');
    const counter=await page.evaluate(()=>{
      const lab=contenderLab;
      return {hit:lab.records.at(-1).effects.find(e=>e.source==='counter'),name:lab.lastCounter,charges:lab.state.charges,
        hp:lab.units.find(u=>u.name==='Assassin').hp, armour:lab.units.find(u=>u.name==='Assassin').stats.defence};
    });
    assert.equal(counter.hit.target,assassin.id);
    assert.match(counter.name,/Assassin/);
    assert.equal(counter.charges,4);
    assert.equal(assassin.hp-counter.hp,Math.floor(counter.hit.amount*100/(100+counter.armour)));
    await page.evaluate(()=>contenderLab.reset());
    await page.locator('[data-key=r]').click();
    const warning=await page.evaluate(()=>{
      const lab=contenderLab;
      for(let i=0;i<19;i++)lab.step();
      return lab.state.shadow_link;
    });
    assert.equal(warning?.phase,'warning');
    assert.equal(warning.linked.length,3);
    assert.equal(await page.request.get(base+'/mods/tfm2_custom/vfx/contender_link_warning%23sheet.png').then(r=>r.status()),200);
    if(process.env.CONTENDER_LINK_PREVIEW)await page.locator('#arena').screenshot({path:process.env.CONTENDER_LINK_PREVIEW});
    const link=await page.evaluate(()=>{
      const lab=contenderLab;
      for(let i=0;i<110;i++)lab.step();
      const events=lab.records.flatMap(row=>row.effects.map(e=>({tick:row.frame.tick,...e})));
      return {damage:events.filter(e=>e.kind==='damage'),air:events.filter(e=>e.kind==='link_airborne'),
        stun:events.filter(e=>e.kind==='link_stun')};
    });
    assert.deepEqual(link.damage,[]);
    assert.equal(link.air.length,3,JSON.stringify(link));
    assert.equal(link.stun.length,3);
    for(const air of link.air)assert.equal(link.stun.find(s=>s.target===air.target).tick-air.tick,30);
    await page.locator('#recording').setInputFiles({name:'old.json',mimeType:'application/json',
      buffer:Buffer.from(JSON.stringify({version:1,rules_revision:'contender-3.0',records:[{state:{},effects:[]}]}))});
    await page.waitForFunction(()=>document.getElementById('recordingStatus').textContent.includes('Replay requires original rules contender-3.0'));
    assert.deepEqual(errors,[]);
    console.log(JSON.stringify({mapCells:900,punches:run.punches.length,castGap:45,exchange:true,crossCreditTicks:cross.credit,counter:counter.name,linkTargets:link.air.length,
      ccTicks:[30,24],olderRecording:true,errors},null,2));
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
