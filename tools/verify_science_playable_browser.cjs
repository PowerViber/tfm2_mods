/* Exercise real input and rendering in both Science Lab surfaces. */
const {chromium}=require('playwright-core'),assert=require('node:assert/strict'),path=require('node:path'),fs=require('node:fs');
const base=process.env.SCIENCE_LAB_URL||'http://127.0.0.1:19366/science.html',root=path.resolve(__dirname,'..');
(async()=>{
 const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'C:/Program Files/Google/Chrome/Application/chrome.exe',headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[],bad=[];page.on('pageerror',e=>errors.push(e.message));page.on('response',r=>{if(r.status()>=400)bad.push(r.status()+' '+r.url());});
  for(const embed of [false,true]){
   await page.goto(base+(embed?'?embed=1':''),{waitUntil:'networkidle'});await page.waitForFunction(()=>window.ScienceLab?.renderer?.ready&&!document.querySelector('#preview').disabled);
   assert.equal(await page.locator('#sciencePreview').inputValue(),'arena');assert.equal(await page.locator('#arena').isVisible(),true);assert.equal(await page.locator('#artCanvas').isVisible(),false);
   assert.deepEqual(await page.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
   const canvas=page.locator('#arena');await canvas.scrollIntoViewIfNeeded();let box=await canvas.boundingBox();
   const initial=await page.evaluate(()=>({x:ScienceLab.combat.hero.x,y:ScienceLab.combat.hero.y}));
   await page.mouse.click(box.x+box.width*.43,box.y+box.height*.64,{button:'right'});await page.waitForFunction(x=>ScienceLab.combat.hero.x>x+12000,initial.x);
   await canvas.press('s');const stopped=await page.evaluate(()=>({...ScienceLab.combat.hero}));assert.equal(stopped.dest,null);await page.waitForTimeout(150);assert.equal(await page.evaluate(()=>ScienceLab.combat.hero.x),stopped.x);
   await page.mouse.click(box.x+box.width*.46,box.y+box.height*.12);assert.equal(await page.evaluate(()=>ScienceLab.combat.locked),null);
   const aim=await page.evaluate(()=>({...ScienceLab.combat.aim}));await canvas.press('Space');await page.mouse.move(box.x+box.width*.60,box.y+box.height*.60);
   await page.waitForFunction(()=>ScienceLab.combat.events.some(e=>e.type==='release'));const release=await page.evaluate(()=>ScienceLab.combat.events.find(e=>e.type==='release'));
   assert.deepEqual(release.aim,aim);assert.equal(release.origin.x,stopped.x);assert.equal(release.target,null);
   // Pausing and single stepping do not advance recovery from wall-clock time.
   await page.click('#practicePause');const tick=await page.evaluate(()=>ScienceLab.combat.tick);await page.waitForTimeout(150);assert.equal(await page.evaluate(()=>ScienceLab.combat.tick),tick);await page.click('#practiceStep');assert.equal(await page.evaluate(()=>ScienceLab.combat.tick),tick+1);
   await page.click('#practiceReset');await page.waitForFunction(()=>!document.querySelector('#preview').disabled);assert.equal(await page.locator('.charge-cell[data-state="ready"]').count(),3);
   await page.check('#practiceMoving');await page.click('#preview');await page.waitForFunction(()=>ScienceLab.combat.events.some(e=>e.type==='release'));
   const locked=await page.evaluate(()=>({release:ScienceLab.combat.events.find(e=>e.type==='release'),dummy:{...ScienceLab.combat.dummies[0]}}));assert.equal(locked.release.target,1);assert.notEqual(locked.release.aim.y,105000);
   await page.click('#practiceReset');await page.selectOption('#artSkill','51');await page.waitForFunction(()=>!document.querySelector('#castReason').textContent.includes('Loading'));
   assert.equal(await page.locator('#preview').isDisabled(),true);assert.match(await page.locator('#castReason').innerText(),/Prepare C01/);await page.click('#preparePrerequisite');await page.waitForFunction(()=>ScienceLab.combat.theory.sample&&!ScienceLab.combat.plan);
   await page.waitForFunction(()=>!document.querySelector('#preview').disabled);await page.click('#preview');await page.waitForFunction(()=>ScienceLab.combat.theory.purity===100);
   assert.equal(await page.evaluate(()=>ScienceLab.combat.events.filter(e=>e.type==='spent').length),2,'Preparation silently granted free resources');
   // Support and fields have distinct outputs, without a generic packet fallback.
   await page.click('#practiceReset');await page.selectOption('#artSkill','67');await page.waitForFunction(()=>!document.querySelector('#preview').disabled);await page.click('#preview');await page.waitForFunction(()=>ScienceLab.combat.hero.shields.length>0);
   assert.equal(await page.evaluate(()=>ScienceLab.combat.packets.length),0);assert.ok(await page.evaluate(()=>ScienceLab.combat.hero.buffs.membrane));
   await page.click('#practiceReset');await page.selectOption('#artRank','6');await page.selectOption('#artSkill','57');await page.waitForFunction(()=>!document.querySelector('#preview').disabled);await page.click('#preview');await page.waitForFunction(()=>ScienceLab.combat.fields.some(f=>f.kind===57));
   await page.selectOption('#artSkill','65');await page.waitForFunction(()=>!document.querySelector('#preview').disabled);await page.click('#preview');await page.waitForFunction(()=>ScienceLab.combat.fields.some(f=>f.kind===65));
   await page.waitForFunction(()=>ScienceLab.combat.events.some(e=>e.type==='hit'&&e.skill===65));await page.click('#practicePause');
   const missing=await page.evaluate(()=>[...ScienceLab.renderer.missing]);assert.deepEqual(missing,[],'Missing runtime sprite binding');
   await page.screenshot({path:path.join(root,'docs/unified-theory-playable-'+(embed?'embedded':'standalone')+'.png'),fullPage:false});
   // Transparency comes from the actual body atlas. Draw it on a clean canvas.
   const transparency=await page.evaluate(()=>{const c=document.createElement('canvas');c.width=48;c.height=64;const r=new ScienceRenderer.Renderer(c);r.assets=ScienceLab.renderer.assets;r.images=ScienceLab.renderer.images;r.blit('body_idle',0,24,32,1);const d=r.ctx.getImageData(0,0,48,64).data;let opaque=0;for(let i=3;i<d.length;i+=4)if(d[i])opaque++;return {opaque,corner:d[3],src:r.assets.body_idle.src};});
   assert.ok(transparency.opaque>100&&transparency.opaque<48*64/2);assert.equal(transparency.corner,0);assert.equal(transparency.src,'/science-body.png');
   await page.selectOption('#sciencePreview','animation');assert.deepEqual(await page.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[false,true]);await page.selectOption('#sciencePreview','arena');assert.deepEqual(await page.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
   // Keyboard controls stay scoped to the focused arena; number fields remain editable.
   await page.fill('#practiceHealth','800');await page.locator('#practiceHealth').press('Tab');assert.equal(await page.evaluate(()=>ScienceLab.combat.hero.hp),800);
   await page.setViewportSize({width:390,height:844});await page.waitForTimeout(80);assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));await page.locator('#arena').scrollIntoViewIfNeeded();box=await canvas.boundingBox();await page.mouse.click(box.x+box.width*.45,box.y+box.height*.45,{button:'right'});
   const dest=await page.evaluate(()=>ScienceLab.combat.hero.dest);assert.ok(Math.abs(dest.x-ScienceWidth()*.45)<2);await page.setViewportSize({width:1440,height:1100});
  }
  // Render all three forms, all ranks, mirrored facing and each body phase using
  // the same renderer as live combat. The resulting contact sheet is visual QA.
  const sheet=await page.evaluate(async()=>{
   const c=document.createElement('canvas');c.width=1152;c.height=2048;const r=new ScienceRenderer.Renderer(c);r.assets=ScienceLab.renderer.assets;r.images=ScienceLab.renderer.images;r.ready=true;r.ctx.fillStyle='#0b1625';r.ctx.fillRect(0,0,c.width,c.height);
   for(let rank=0;rank<8;rank++){await r.prepareSkills([0,25,50],rank,1);for(let science=0;science<3;science++)for(let flip=0;flip<2;flip++)for(let phase=0;phase<4;phase++){
    const col=science*2+flip,row=rank*4+phase,x=col*192+96,y=row*64+38,s=new ScienceCombat.Combat({rank,position:1,science}).snapshot();s.hero.face=flip?-1:1;s.tick=48;if(phase<3){if(phase===0)s.plan={stages:[{skill:science*25}],index:0};else s.releasePose={science,tick:phase===1?48:36};}
    r.actor(s,x,y,1);r.ctx.fillStyle='#9bb5c9';r.ctx.font='8px monospace';r.ctx.fillText(['E','N','C'][science]+' r'+rank+' '+(flip?'L':'R')+' '+['prep','release','recover','idle'][phase],col*192+5,row*64+10);
   }}
   return {url:c.toDataURL('image/png'),missing:[...r.missing]};
  });assert.deepEqual(sheet.missing,[]);fs.writeFileSync(path.join(root,'docs/unified-theory-playable-poses.png'),Buffer.from(sheet.url.split(',')[1],'base64'));
  const logs=await page.evaluate(()=>ScienceLab.renderer.error);assert.equal(logs,null);assert.deepEqual(errors,[]);assert.deepEqual(bad,[]);
  console.log('PASS: standalone/embedded real movement, stop, cursor/locked aim, Space cast, scoped keys, independent charges, pause/step, paid prerequisites, shields, cloud/crystal effects, transparent body, 192 rank/form/facing/pose renders and mobile coordinates.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
function ScienceWidth(){return 330*950;}
