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
    const rules=await page.evaluate(()=>window.CONTENDER_RULES);
    const {flurry,merge,charges,shadowLink}=rules;
    assert.equal(await page.locator('#revision').textContent(),rules.revision);
    assert.equal(await page.locator('[data-key=c]').count(),0);
    assert.equal(await page.evaluate(()=>contenderLab.world.geometry.registration.matching),900);
    await page.locator('[data-key=e]').click();
    const run=await page.evaluate(()=>{
      const lab=contenderLab,start=lab.records.at(-1).frame.tick;
      const launchPos={...lab.me.pos};
      for(let i=0;i<CONTENDER_RULES.flurry.ticks;i++)lab.step();
      const punches=lab.records.flatMap(row=>row.effects
        .filter(effect=>effect.kind==='damage'&&effect.source==='flurry').map(effect=>[row.frame.tick,effect.target]));
      const effects=lab.records.at(-1).effects;
      return {start,punches,gap:lab.state.flurry_cast_at,charges:lab.state.flurry_charges,
        credit:lab.state.flurry_credited_ticks,launchPos,endPos:lab.me.pos,
        appear:effects.find(effect=>effect.kind==='fx'&&effect.name==='ct_appear')?.at,
        teleports:lab.records.flatMap(row=>row.effects).filter(effect=>effect.kind==='teleport').length};
    });
    assert.equal(run.punches.length,flurry.punches,JSON.stringify(run));
    for(const [i,[tick]] of run.punches.entries())assert.equal(tick-run.start,(i+1)*flurry.gapTicks);
    assert.equal(run.punches.at(-1)[0]-run.start,flurry.ticks);
    assert.equal(run.gap-run.punches.at(-1)[0],flurry.castGapTicks);
    assert.equal(run.charges,flurry.maxCharges-1);
    assert.equal(run.credit,0);
    assert.deepEqual(run.endPos,run.launchPos);
    assert.deepEqual(run.appear,run.endPos);
    assert.equal(run.teleports,0);
    const recast=await page.evaluate(()=>{
      const lab=contenderLab;
      lab.step({kind:'s2'});
      return {charges:lab.state.flurry_charges,started:!!lab.state.flurry,rejection:lab.state.rejection};
    });
    assert.equal(recast.charges,flurry.maxCharges-2,recast.rejection);
    assert.equal(recast.started,true);
    await page.evaluate(()=>contenderLab.reset());
    await page.locator('[data-key=e]').click();
    await page.evaluate(()=>{
      const lab=contenderLab;
      for(let i=0;i<CONTENDER_RULES.flurry.ticks;i++)lab.step();
    });
    const cross=await page.evaluate(()=>{
      const lab=contenderLab,from=lab.me.pos,radius=CONTENDER_RULES.merge.crossRadius*0.9;
      const angles=Array.from({length:24},(_,i)=>i*Math.PI/12);
      const at=angles.map(angle=>({x:Math.round(from.x+radius*Math.cos(angle)),y:Math.round(from.y+radius*Math.sin(angle))}))
        .find(pos=>lab.world.clear(from,pos));
      if(!at)throw Error('No clear Shadow Cross path in the fixture');
      const foe=lab.units.find(u=>u.name==='Tank');
      for(const u of lab.units)if(u!==foe)u.targetable=false;
      foe.pos=at;
      const before=lab.state.flurry_charge_at;
      lab.step({kind:'s1'});
      for(let i=0;i<=CONTENDER_RULES.merge.crossEveryTicks;i++)lab.step();
      const hits=lab.records.flatMap(row=>row.effects.filter(e=>e.kind==='damage'&&e.source==='merge'));
      return {targets:hits.map(hit=>hit.target),foe:foe.id,credit:before-lab.state.flurry_charge_at,
        distance:Math.hypot(at.x-from.x,at.y-from.y)};
    });
    assert.deepEqual(cross.targets,[cross.foe]);
    assert.equal(cross.credit,merge.crossRechargeCreditTicks);
    assert.ok(cross.distance>merge.crossRadius*0.85&&cross.distance<=merge.crossRadius);
    await page.locator('#arena').focus();
    await page.keyboard.press('Shift+E');
    await page.waitForFunction(max=>contenderLab.state.flurry_charges===max,flurry.maxCharges);
    assert.equal(await page.evaluate(()=>contenderLab.state.charges),charges.max-1);
    await page.evaluate(()=>contenderLab.pause());
    await page.locator('[data-key=exchange]').click();
    assert.equal(await page.evaluate(()=>contenderLab.state.charges),charges.max-1);
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
    assert.equal(counter.charges,charges.max-1);
    assert.equal(assassin.hp-counter.hp,Math.floor(counter.hit.amount*100/(100+counter.armour)));
    await page.evaluate(()=>contenderLab.reset());
    await page.locator('[data-key=r]').click();
    const warning=await page.evaluate(()=>{
      const lab=contenderLab;
      for(let i=0;i<=CONTENDER_RULES.shadowLink.flightTicks;i++)lab.step();
      return lab.state.shadow_link;
    });
    assert.equal(warning?.phase,'warning');
    assert.equal(warning.linked.length,3);
    assert.equal(await page.request.get(base+'/mods/tfm2_custom/vfx/contender_link_warning%23sheet.png').then(r=>r.status()),200);
    if(process.env.CONTENDER_LINK_PREVIEW)await page.locator('#arena').screenshot({path:process.env.CONTENDER_LINK_PREVIEW});
    const link=await page.evaluate(()=>{
      const lab=contenderLab;
      const link=CONTENDER_RULES.shadowLink;
      for(let i=0;i<=link.warningTicks+link.pullTicks+link.knockTicks+link.stunTicks;i++)lab.step();
      const events=lab.records.flatMap(row=>row.effects.map(e=>({tick:row.frame.tick,...e})));
      return {damage:events.filter(e=>e.kind==='damage'),air:events.filter(e=>e.kind==='link_airborne'),
        stun:events.filter(e=>e.kind==='link_stun')};
    });
    assert.deepEqual(link.damage,[]);
    assert.equal(link.air.length,3,JSON.stringify(link));
    assert.equal(link.stun.length,3);
    for(const air of link.air)assert.equal(link.stun.find(s=>s.target===air.target).tick-air.tick,shadowLink.knockTicks);
    await page.locator('#recording').setInputFiles({name:'old.json',mimeType:'application/json',
      buffer:Buffer.from(JSON.stringify({version:1,rules_revision:'contender-3.0',records:[{state:{},effects:[]}]}))});
    await page.waitForFunction(()=>document.getElementById('recordingStatus').textContent.includes('Replay requires original rules contender-3.0'));
    assert.deepEqual(errors,[]);
    console.log(JSON.stringify({mapCells:900,punches:run.punches.length,castGap:flurry.castGapTicks,inPlace:true,immediateRecast:true,exchange:true,crossCreditTicks:cross.credit,counter:counter.name,linkTargets:link.air.length,
      ccTicks:[shadowLink.knockTicks,shadowLink.stunTicks],olderRecording:true,errors},null,2));
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
