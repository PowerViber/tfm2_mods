/* Actual Lab loading plus independent decoded-pixel oracle for every body frame. */
'use strict';
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),assert=require('node:assert/strict');
const {chromium}=require(require.resolve('playwright',{paths:[path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/ranked-body-redesign'),base='http://127.0.0.1:7298';
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1440,height:1000}}),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 try{
  await page.goto(base+'/ranked-body-review.html');await page.waitForFunction(()=>window.rankArtReview);
  await page.addScriptTag({url:base+'/ranked-body-runtime.js'});await page.evaluate(()=>RankedBodyArt.readyPromise);assert.equal(await page.evaluate(()=>RankedBodyArt.ready),true);
  const oracle=JSON.parse(fs.readFileSync(path.join(out,'frame-pixel-oracle.json')));
  const parity=await page.evaluate(async oracle=>{
   const bad=[];let checked=0;
   for(const[key,poses]of Object.entries(oracle))for(const[pose,frames]of Object.entries(poses))for(let i=0;i<frames.length;i++)for(const facing of ['right','left']){
    const expected=frames[i],canvas=document.createElement('canvas');[canvas.width,canvas.height]=expected.size;const ctx=canvas.getContext('2d');
    if(!RankedBodyArt.draw(ctx,key,pose,0,canvas.width/2,canvas.height/2,1,facing==='left',i))throw Error('Missing '+key+'/'+pose);
    const pixels=ctx.getImageData(0,0,canvas.width,canvas.height).data;
    const sha=[...new Uint8Array(await crypto.subtle.digest('SHA-256',pixels))].map(n=>n.toString(16).padStart(2,'0')).join('');
    if(sha!==expected[facing])bad.push({key,pose,i,facing,sha,expected:expected[facing]});checked++;
   }return{checked,bad};
  },oracle);assert.deepEqual(parity.bad,[]);assert.equal(parity.checked,426);
  const warm=[];
  for(let run=0;run<5;run++)warm.push(await page.evaluate(oracle=>{
   const canvas=document.createElement('canvas');canvas.width=canvas.height=256;const ctx=canvas.getContext('2d'),times=[];
   for(const[key,poses]of Object.entries(oracle))for(const[pose,frames]of Object.entries(poses))for(let i=0;i<frames.length;i++){
    const start=performance.now();RankedBodyArt.draw(ctx,key,pose,0,128,128,3,false,i);times.push(performance.now()-start);
   }times.sort((a,b)=>a-b);return{medianMs:times[Math.floor(times.length/2)],p95Ms:times[Math.floor((times.length-1)*.95)],p99Ms:times[Math.floor((times.length-1)*.99)]};
  },oracle));
  const cold=[];for(let i=0;i<5;i++){const context=await browser.newContext();const p=await context.newPage();await p.goto(base+'/ranked-body-review.html');cold.push(await p.evaluate(async()=>{const s=performance.now();await new Promise((resolve,reject)=>{const t=document.createElement('script');t.src='ranked-body-runtime.js';t.onload=resolve;t.onerror=reject;document.body.append(t)});await RankedBodyArt.readyPromise;if(!RankedBodyArt.ready)throw Error(RankedBodyArt.error);return performance.now()-s}));await context.close();}
  const pages=[];
  for(const name of ['vesper','magnus']){await page.goto(base+'/'+name+'.html');await page.evaluate(()=>RankedBodyArt.readyPromise);assert.equal(await page.evaluate(()=>RankedBodyArt.ready),true);await page.waitForTimeout(400);await page.screenshot({path:path.join(out,name+'-lab.png'),fullPage:false});pages.push(name);}
  await page.goto(base+'/index.html');await page.waitForFunction(()=>window.TFM2CoderLab&&window.TFM2IsliidLab&&window.RankedBodyArt?.ready);
  for(const[name,api]of [['isliid','TFM2IsliidLab'],['coder','TFM2CoderLab']]){
   await page.evaluate(({name,api})=>{const old=document.querySelector('#rankArtTest');if(old)old.remove();const div=document.createElement('div');div.id='rankArtTest';document.body.prepend(div);window[api].mount(div)},{name,api});
   await page.waitForTimeout(400);await page.locator('#rankArtTest').screenshot({path:path.join(out,name+'-lab.png')});pages.push(name);
  }
  assert.deepEqual(errors,[]);
  const report={packedFramePixelParity:parity.checked,labPages:pages,errors,coldBodyLoadMs:cold,warmDrawRuns:warm,scope:'Headless browser presentation only; not game-host timing or native body-frame synchronisation proof',nativeRankMode:'visible-base-fallback',actualGameObserved:false};
  fs.writeFileSync(path.join(out,'lab-browser-results.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
