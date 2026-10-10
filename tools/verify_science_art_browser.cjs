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
  for(let p=1;p<=10;p++){
   await page.selectOption('#artPosition',String(p));
   assert.match(await page.locator('#artCaption').innerText(),new RegExp('#'+p+' ·'));
  }
  await page.selectOption('#artPosition','1');await page.selectOption('#artSkill','0');
  await page.waitForLoadState('networkidle');
  const pixels=()=>page.locator('#artCanvas').evaluate(c=>c.toDataURL());
  const a=await pixels();await page.waitForTimeout(220);assert.notEqual(await pixels(),a,'Animation is static');
  await page.click('#artPause');const frozen=await pixels();await page.waitForTimeout(220);assert.equal(await pixels(),frozen,'Pause does not freeze the sprites');
  await page.click('#artPause');await page.click('#artComplete');await page.waitForTimeout(160);
  assert.notEqual(await pixels(),frozen,'Completion animation never appeared');
  await page.locator('.art-inspector').screenshot({path:path.resolve(__dirname,'../docs/unified-theory-art-inspector.png')});
  await page.setViewportSize({width:390,height:844});
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth),'Mobile horizontal overflow');
  assert.deepEqual(errors,[]);assert.deepEqual(badResponses,[]);
  console.log('PASS: 75 actual skill animations across all 8 ranks, 10 Top 10 positions, moving frames, pause/play, completion animation, desktop/mobile layout; no script or asset-loading errors.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
