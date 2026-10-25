const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const out=path.join(__dirname,'../docs/raze-results');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.LAB_URL||'http://localhost:7286')+'/editor/raze.html');await page.waitForFunction(()=>window.RazeLab);
 const arena=page.locator('#arena');
 await page.selectOption('#scenario','sticky');await page.evaluate(()=>{RazeLab.pause();RazeLab.setAutomatic(false);});
 const box=await arena.boundingBox();await arena.focus();await page.mouse.move(box.x+200*box.width/960,box.y+260*box.height/560);
 await page.keyboard.press('Shift+Q');await page.waitForFunction(()=>RazeLab.state?.packs[0]?.ready);await page.evaluate(()=>RazeLab.pause());
 const attached=await page.evaluate(()=>RazeLab.state.packs[0]);assert.equal(attached.attachment.kind,'structure');assert.equal(attached.attachment.surface,7);assert.equal(attached.attachment.normal.x,-1);
 await page.locator('#move-support').click();await page.evaluate(()=>RazeLab.step());const moved=await page.evaluate(()=>RazeLab.state.packs[0]);assert.equal(moved.pos.y-attached.pos.y,12000);
 await page.screenshot({path:path.join(out,'raze-sticky-satchel.png'),fullPage:true});
 await page.locator('#remove-support').click();await page.evaluate(()=>RazeLab.step());const dropped=await page.evaluate(()=>RazeLab.state.packs[0]);assert.equal(dropped.attachment.kind,'ground');assert.equal(dropped.until,attached.until);
 const replay=await page.evaluate(()=>RazeLab.verify());assert(replay.identical);
 async function rocket(level,dodge=true){
  await page.selectOption('#scenario','dodge');await page.selectOption('#enemy-profile',String(level));await page.locator('#enemy-dodge').setChecked(dodge);
  await page.evaluate(async()=>{const L=RazeLab;L.pause();L.setAutomatic(false);L.units[1].pos={x:400000,y:342000};await L.step();L.queueAction({kind:'equip_rocket'});await new Promise(r=>setTimeout(r,10));await L.step(43);L.setAim({x:420000,y:300000});L.queueAction({kind:'fire_rocket',aim:{x:420000,y:300000}});await new Promise(r=>setTimeout(r,10));});
  const before=await page.evaluate(()=>JSON.parse(JSON.stringify({pos:RazeLab.units[1].pos,rocket:RazeLab.state.rocket,defenders:RazeLab.state.dodges})));
  assert(before.rocket&&before.rocket.speed===120000);await page.evaluate(()=>RazeLab.step(14));
  const after=await page.evaluate(()=>JSON.parse(JSON.stringify({pos:RazeLab.units[1].pos,rocket:RazeLab.state.rocket,defenders:RazeLab.state.dodges})));
  await page.screenshot({path:path.join(out,`raze-rocket-warning-${level}-${dodge}.png`),fullPage:true});
  await page.evaluate(()=>RazeLab.step(30));const final=await page.evaluate(()=>({rocket:RazeLab.state.rocket,goals:RazeLab.enemyGoals.size,replayFrames:RazeLab.history.length}));assert.equal(final.rocket,null);assert.equal(final.goals,0);
  assert((await page.evaluate(()=>RazeLab.verify())).identical);return{before,after,final};
 }
 const slow=await rocket(20),fast=await rocket(80),off=await rocket(80,false);
 assert.deepStrictEqual(slow.before.pos,slow.after.pos);assert.notDeepStrictEqual(fast.before.pos,fast.after.pos);assert.deepStrictEqual(off.before.pos,off.after.pos);
 assert(fast.after.defenders['2'].ready<slow.after.defenders['2'].ready);
 assert.deepStrictEqual(errors,[]);
 const report={version:3,shiftQAimedSurface:true,attachment:attached.attachment,structureFollow:true,dropPreservesLifetime:true,slow,fast,off,replay,errors,liveAcceptance:'Browser movement and shared-core evidence; native carrier and host acceptance pending.'};
 fs.writeFileSync(path.join(out,'v3-browser.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({shiftQAimedSurface:true,structureFollow:true,dodgeProfiles:true,rocketCleanup:true,replay:true,errors}));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
