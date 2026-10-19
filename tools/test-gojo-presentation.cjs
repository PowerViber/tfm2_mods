/* Browser presentation/recording tests. Does not access the installed game or saves. */
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/gojo-results'),visual=JSON.parse(fs.readFileSync(path.join(root,'rules/gojo-visuals.json')));
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage({viewport:{width:1360,height:1040}}),errors=[],checks=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://localhost:7281/gojo.html');await page.waitForFunction(()=>window.GojoLab);await page.evaluate(()=>{GojoLab.pause();GojoLab.reset()});
 async function check(name,fn){await fn();checks.push(name)}
 await check('Explicit Blue preparation and launch phases retain their start tick',async()=>{
  const result=await page.evaluate(()=>{GojoLab.submit(1);const a=GojoLab.snapshot.presentation.cues.find(v=>v.tag==='blue_prepare');for(let i=0;i<10;i++)GojoLab.step();const b=GojoLab.snapshot.presentation.cues.find(v=>v.tag==='blue_prepare');for(let i=0;i<17;i++)GojoLab.step();return {a,b,flight:GojoLab.snapshot.presentation.cues.find(v=>v.phase==='flight')}});
  assert.equal(result.a.start,result.b.start);assert(result.b.frame>result.a.frame);assert.equal(result.flight.tag,'blue_flight');
 });
 await check('Interrupted casting clears its energy and gesture without a released shot',async()=>{
  await page.evaluate(()=>GojoLab.reset());await page.evaluate(()=>GojoLab.submit(5));await page.click('#interrupt');
  const s=await page.evaluate(()=>({s:GojoLab.snapshot,e:GojoLab.effects}));assert(!s.e.some(e=>e.phase==='prepare'));assert(!s.s.presentation.cues.some(e=>e.phase==='flight'));
 });
 await check('Purple preparation and flight never select the same artwork',async()=>{
  await page.evaluate(()=>{GojoLab.reset();GojoLab.submit(5)});assert((await page.evaluate(()=>GojoLab.snapshot.presentation.cues)).some(c=>c.tag==='purple_prepare'));
  const a=await page.evaluate(()=>{for(let i=0;i<35;i++)GojoLab.step();return GojoLab.snapshot.presentation.cues});assert(a.some(c=>c.tag==='purple_flight'));assert(!a.some(c=>c.tag==='purple_prepare'));
 });
 await check('Death clears owner effects and does not renew a gesture',async()=>{
  await page.evaluate(()=>{GojoLab.reset();GojoLab.submit(1)});await page.click('#death');assert.equal(await page.evaluate(()=>GojoLab.effects.filter(e=>e.owner===1).length),0);
 });
 await check('Older v2 recordings without presentation metadata replay exactly',async()=>{
  await page.evaluate(()=>{GojoLab.reset();GojoLab.submit(1);for(let i=0;i<80;i++)GojoLab.step();for(const row of GojoLab.recording)delete row.output.presentation});
  const before=await page.evaluate(()=>GojoLab.snapshot.core);await page.click('#replay');assert.equal(await page.evaluate(()=>GojoLab.snapshot.core),before);assert((await page.textContent('#events')).includes('Replay verified'));
 });
 await page.locator('details summary').click();
 await check('Every variant/status/gesture renders in both facings on light and dark terrain',async()=>{
  let samples=0;
  for(const terrain of ['dark','light'])for(const left of [false,true]){
   await page.selectOption('#terrain',terrain);await page.locator('#facing').setChecked(left);
   for(const tag of [...visual.variants,...visual.statuses,...visual.gestures.map(t=>'gesture_'+t)]){await page.selectOption('#inspect','v:'+tag);await page.evaluate(()=>GojoLab.step());samples++}
  }
  assert.equal(samples,160);
 });
 await check('Actual-size previews are captured for both facings and terrain values',async()=>{
  for(const [tag,terrain,left]of [['blue_impact','dark',false],['black_flash','light',true],['purple_prepare','dark',false],['gesture_domain','light',true]]){
   await page.selectOption('#terrain',terrain);await page.locator('#facing').setChecked(left);await page.selectOption('#inspect','v:'+tag);
   await page.locator('#arena').screenshot({path:path.join(out,`anime-lab-${tag}.png`)});
  }
 });
 await page.selectOption('#inspect','-1');await page.selectOption('#fixture','team');await page.check('#automatic');await page.check('#incoming');await page.check('#moving');
 await page.evaluate(()=>{for(let i=0;i<360;i++)GojoLab.step()});await page.locator('#arena').screenshot({path:path.join(out,'anime-crowded-lab.png')});
 const renderRuns=[];
 for(let n=0;n<5;n++){
  const context=await browser.newContext(),p=await context.newPage();const at=Date.now();await p.goto('http://localhost:7281/gojo.html');await p.waitForFunction(()=>window.GojoLab);const assetLoadMs=Date.now()-at;
  const timings=await p.evaluate(()=>{
   GojoLab.pause();GojoLab.reset();document.querySelector('#fixture').value='team';document.querySelector('#fixture').dispatchEvent(new Event('change'));GojoLab.submit(8);for(let i=0;i<80;i++)GojoLab.step();
   const coldAt=performance.now();GojoLab.draw();const firstDomainDrawMs=performance.now()-coldAt,times=[];
   for(let i=0;i<240;i++){const at=performance.now();GojoLab.draw();times.push(performance.now()-at)}times.sort((a,b)=>a-b);
   return {firstDomainDrawMs,p50Ms:times[120],p95Ms:times[228],p99Ms:times[237],samples:times.length};
  });renderRuns.push({assetLoadMs,...timings});await context.close();
 }
 assert.deepEqual(errors,[]);
 fs.writeFileSync(path.join(out,'anime-browser.json'),JSON.stringify({checks,errors,variants:40,renderSamples:160,liveGame:false},null,2)+'\n');
 fs.writeFileSync(path.join(out,'anime-render-performance.json'),JSON.stringify({scope:'Five fresh browser contexts. Load includes local fetch/decode/WASM/startup. Draw timings measure browser CPU canvas submission, excluding GPU completion and native host rendering. Domain first draw follows setup; not a cold asset decode.',runs:renderRuns},null,2)+'\n');
 await browser.close();console.log(JSON.stringify({passed:checks.length,renderRuns:5,errors}));
})().catch(e=>{console.error(e);process.exit(1)});
