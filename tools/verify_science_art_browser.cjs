const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const path=require('node:path');
(async()=>{
 const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[],badResponses=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('response',r=>{if(r.status()>=400)badResponses.push(r.status()+' '+r.url());});
  await page.goto(process.env.SCIENCE_LAB_URL||'http://127.0.0.1:7272/science.html',{waitUntil:'networkidle'});
  await page.waitForFunction(()=>document.querySelector('#artStatus').textContent.startsWith('Original game sprites'));
  assert.equal(await page.locator('#artSkill option').count(),75);
  for(let rank=0;rank<8;rank++){
   await page.selectOption('#artRank',String(rank));
   assert.equal(await page.locator('#artPosition').isDisabled(),rank!==7);
   for(let s=0;s<75;s++){
    await page.selectOption('#artSkill',String(s));
    assert.match(await page.locator('#artCaption').innerText(),new RegExp(['Einstein','Newton','Marie Curie'][Math.floor(s/25)]));
   }
  }
  await page.selectOption('#artRank','7');
  const manifest=await page.evaluate(async()=>await (await fetch('/science-art-preview.json')).json());
  assert.ok(manifest.base,'Morphs must use a body without a stale Student aura');
  for(let s=0;s<3;s++)for(let rank=0;rank<8;rank++)for(const podium of rank===7?[1,2,3,4]:[4]){
   const tag=`gearback${s}_r${rank}`+(rank===7?`_p${podium}`:'');
   assert.equal(manifest.assets[tag].frames.length,8,tag+' aura lost phases');
   const duration=manifest.assets[tag].frames.reduce((n,f)=>n+f.duration,0);
   assert.ok(Math.abs(duration-(rank===7&&podium===1?1.6:.96))<1e-6,tag+' has the wrong aura cadence');
   assert.equal(manifest.assets[`outfit${s}_r${rank}`].frames.length,8,'Body must animate independently of the slower #1 aura');
  }
  for(let p=1;p<=10;p++){
   await page.selectOption('#artPosition',String(p));
   assert.match(await page.locator('#artCaption').innerText(),new RegExp('#'+p+' ·'));
  }
  await page.selectOption('#artPosition','1');
  for(let science=0;science<3;science++){
   await page.selectOption('#artSkill',String(science*25));
   const title=['Cosmic Genesis','The Unprovable Shape','Genesis Vessel'][science];
   assert.match(await page.locator('#artCaption').innerText(),new RegExp(title));
   assert.equal(await page.locator('#artPosition option[value="1"]').innerText(),'#1 — '+title);
   for(let tier=0;tier<4;tier++)for(const podium of tier===3?[1,2,3,4]:[4]){
    assert.equal(manifest.assets[`complete_t${tier}_p${podium}_s${science}`].frames.length,8,'Missing subject-specific completion');
   }
   await page.click('#artComplete');await page.waitForTimeout(160);
  }
  for(const s of [0,1,5,12,13,16,18,19,20,29,34,39,40,45,54,57,58,59,63,65,66,71,72,73,74]){
   await page.selectOption('#artSkill',String(s));await page.selectOption('#artMode','world');
   assert.equal(await page.locator('#artMode').inputValue(),'world');
  }
  await page.selectOption('#artSkill','2');
  assert.equal(await page.locator('#artMode').inputValue(),'cast','Unsupported travel falls back to activation');
  await page.selectOption('#artPosition','1');await page.selectOption('#artSkill','0');
  await page.waitForLoadState('networkidle');
  const pixels=()=>page.locator('#artCanvas').evaluate(c=>c.toDataURL());
  const a=await pixels();await page.waitForTimeout(220);assert.notEqual(await pixels(),a,'Animation is static');
  await page.click('#artPause');const frozen=await pixels();await page.waitForTimeout(220);assert.equal(await pixels(),frozen,'Pause does not freeze the sprites');
  await page.click('#artPause');await page.click('#artComplete');await page.waitForTimeout(160);
  assert.notEqual(await pixels(),frozen,'Completion animation never appeared');
  // Exercise every direct route, the visual cooldown and planted feet through
  // the real atlas viewer, including a paused transition.
  for(let from=0;from<3;from++)for(let to=0;to<3;to++){
   if(from===to)continue;
   await page.selectOption('#artFrom',String(from));await page.selectOption('#artTo',String(to));
   await page.click('#artTransform');
   assert.equal(await page.locator('#artTransform').isDisabled(),true,'A busy transformation can be restarted');
   await page.waitForTimeout(450);
   assert.equal(await page.locator('#artFrom').inputValue(),String(to),'Transformation ended in the wrong form');
   assert.equal(await page.locator('#artSkill').inputValue(),String(to*25));
   assert.match(await page.locator('#artTransformStatus').innerText(),/cooldown finished/);
  }
  await page.selectOption('#artFrom','0');await page.selectOption('#artTo','0');
  assert.equal(await page.locator('#artTransform').isDisabled(),true,'Same-form morph should be unavailable');
  await page.selectOption('#artTo','2');await page.click('#artTransform');
  await page.click('#artPause');const morphFrozen=await pixels();
  await page.waitForTimeout(450);assert.equal(await pixels(),morphFrozen,'Paused transformation advanced');
  await page.click('#artPause');await page.waitForTimeout(450);
  assert.equal(await page.locator('#artFrom').inputValue(),'2');
  for(const [science,name] of ['einstein','newton','curie'].entries()){
   await page.selectOption('#artSkill',String(science*25+24));
   await page.waitForLoadState('networkidle');
   await page.locator('.art-inspector').screenshot({path:path.resolve(__dirname,`../docs/unified-theory-peak-${name}.png`)});
  }
  await page.selectOption('#artSkill','24');
  await page.locator('.art-inspector').screenshot({path:path.resolve(__dirname,'../docs/unified-theory-art-inspector.png')});
  await page.setViewportSize({width:390,height:844});
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth),'Mobile horizontal overflow');
  assert.deepEqual(errors,[]);assert.deepEqual(badResponses,[]);
  console.log('PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, three distinct #1 subject titles, 21 subject completions, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
