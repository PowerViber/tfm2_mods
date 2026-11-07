// Exercise the real review controls and load the packaged PNG/FANIM data.
'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os');
const {chromium}=require(require.resolve('playwright',{paths:[path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),base=process.env.RANK_ART_REVIEW_URL||'http://127.0.0.1:7297';
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage({viewport:{width:1440,height:1000}});
 const errors=[],failed=[];page.on('pageerror',e=>errors.push(e.message));page.on('response',r=>{if(r.status()>=400)failed.push([r.status(),r.url()])});
 let proposals=0,poses=0,frames=0;
 try{
  await page.goto(base+'/editor/ranked-body-review.html');await page.waitForFunction(()=>window.rankArtReview);
  const data=await page.evaluate(()=>rankArtReview.data);assert.equal(data.characters.length,4);assert.equal(data.characters.reduce((a,c)=>a+c.looks.length,0),40);
  for(const c of data.characters){
   for(const p of c.proposals){for(const facing of ['right','left']){for(const terrain of ['dark','light']){
    await page.selectOption('#facing',facing);await page.selectOption('#terrain',terrain);
    await page.evaluate(async({key,look})=>{await rankArtReview.select(key,'proposal',look)}, {key:c.key,look:p.look});
    await page.waitForFunction(()=>document.querySelector('#facts').textContent.includes('concept only'));
    assert.equal(await page.locator('#missing').textContent(),'Single-pose appearance concept. Full animation is not yet authored.');proposals++;
   }}}
   for(const facing of ['right','left']){await page.selectOption('#facing',facing);for(const pose of Object.keys(c.body.anims)){
    await page.evaluate(async({key,pose})=>{await rankArtReview.select(key,'package',0,pose)},{key:c.key,pose});
    for(let i=0;i<c.body.anims[pose].frames.length;i++){
     await page.locator('#frame').fill(String(i));await page.locator('#frame').dispatchEvent('input');
     await page.evaluate(()=>rankArtReview.render());assert.match(await page.locator('#facts').textContent(),/current base package/);frames++;
    }poses++;
   }}
  }
  // The missing ranks must not silently show a substitute body.
  await page.evaluate(()=>rankArtReview.select('vesper','proposal',1));assert.match(await page.locator('#facts').textContent(),/no substitute shown/);
  await page.evaluate(()=>rankArtReview.select('magnus','package',0,'idle'));
  await page.click('#play');await page.waitForTimeout(150);assert.equal(await page.locator('#play').getAttribute('aria-pressed'),'true');await page.click('#play');
  await page.selectOption('#scale','1');await page.selectOption('#facing','left');await page.selectOption('#terrain','light');
  await page.locator('#cards img').evaluateAll(imgs=>Promise.all(imgs.map(im=>im.decode())));
  assert.equal(await page.locator('#cards .portrait img').count(),12);
  assert.equal(await page.locator('#cards .strip.light[data-facing=left]').count(),4);
  await page.screenshot({path:path.join(root,'docs/ranked-body-redesign/review-light-left.png'),fullPage:true});
  await page.selectOption('#scale','3');await page.selectOption('#facing','right');await page.selectOption('#terrain','dark');
  await page.evaluate(()=>rankArtReview.select('vesper','proposal',0));await page.locator('#cards img').evaluateAll(imgs=>Promise.all(imgs.map(im=>im.decode())));
  await page.screenshot({path:path.join(root,'docs/ranked-body-redesign/review-dark-right.png'),fullPage:true});
  assert.deepEqual(errors,[]);assert.deepEqual(failed,[]);
  const report={proposalsChecked:proposals,packagedPosesChecked:poses,packagedFramesChecked:frames,appearanceCount:40,consoleErrors:errors,failedRequests:failed,coldReviewMs:await page.evaluate(()=>rankArtReview.coldMs),observedInGame:false};
  fs.writeFileSync(path.join(root,'docs/ranked-body-redesign/browser-results.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
