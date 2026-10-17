/* Equation Lab in a real browser: the game's unified_core.rs (WASM) drives a reserve/cast combo, a named
   signature, the AI at two ranks, and the page stays error-free on desktop and phone widths. */
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),cp=require('node:child_process'),assert=require('node:assert/strict');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'..'),port=19431,base='http://127.0.0.1:'+port;
let server,browser;
(async()=>{
 const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'tfm2-unified-lab-'));fs.writeFileSync(path.join(fixture,'bundle.game_data'),'');fs.mkdirSync(path.join(fixture,'mods'));
 server=cp.spawn(process.execPath,[path.join(root,'editor/server.js'),'--no-open','--port',String(port),'--game',fixture],{stdio:'pipe'});let boot='';server.stdout.on('data',d=>boot+=d);server.stderr.on('data',d=>boot+=d);
 for(let n=0;n<100;n++){try{if((await fetch(base+'/unified.html')).ok)break;}catch{}await new Promise(r=>setTimeout(r,100));if(n===99)throw Error(boot);}
 browser=await chromium.launch(process.platform==='win32'?{channel:'msedge',headless:true}:{headless:true});
 const page=await browser.newPage({viewport:{width:1280,height:1000}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(base+'/unified.html');await page.waitForFunction(()=>window.UnifiedLab?.snapshot);
 await page.click('#pause');
 assert.equal(await page.locator('.eq').count(),18,'eighteen equations');
 assert.equal(await page.locator('#sigpick option').count(),36,'thirty-six signatures');
 const step=async n=>{for(let i=0;i<n;i++)await page.evaluate(()=>UnifiedLab.step());};
 // Prism ▸ Spacetime Well: one combo of two steps.
 await page.click('.eq[data-skill="1"] button[data-act="reserve"]');await step(1);
 assert.deepEqual(await page.evaluate(()=>UnifiedLab.snapshot.reserve),[1]);
 assert.ok((await page.textContent('#sigs')).includes('Focused Lens'),'open signatures listed');
 await page.click('.eq[data-skill="16"] button[data-act="cast"]');await step(1);
 let s=await page.evaluate(()=>UnifiedLab.snapshot.stats);assert.equal(s.combos,1);assert.equal(s.longest,2);
 await step(120);assert.ok((await page.evaluate(()=>UnifiedLab.snapshot.stats.damage))>0,'the lasing well hurts');
 // A signature, stepped by the lab: Criticality (Radium ▸ Chain Reaction ▸ Gamma).
 await page.click('#reset');await page.click('#pause');await page.selectOption('#sigpick','0');await page.click('#sigplay');
 await step(40);s=await page.evaluate(()=>UnifiedLab.snapshot.stats);assert.equal(s.signatures,1,'Criticality fired');
 assert.ok((await page.textContent('#events')).includes('SIGNATURE · Criticality'));
 // The AI at rank 0 and rank 7.
 const ai=async rank=>{await page.selectOption('#rank',String(rank));await page.check('#auto');await step(900);await page.uncheck('#auto');return page.evaluate(()=>UnifiedLab.snapshot.stats);};
 const lo=await ai(0),hi=await ai(7);
 assert.ok(lo.casts>0&&hi.casts>0);assert.ok(hi.longest>=lo.longest,'top ranks combo at least as long');
 fs.mkdirSync(path.join(root,'docs'),{recursive:true});await page.screenshot({path:path.join(root,'docs/unified-equation-lab.png'),fullPage:true});
 await page.setViewportSize({width:390,height:844});await page.waitForTimeout(100);
 assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth+1),'no horizontal overflow at 390 px');
 // Old links land on the new lab.
 await page.goto(base+'/science.html');await page.waitForURL(/unified\.html/);
 assert.deepEqual(errors,[]);
 console.log(`PASS: 18 equations, 36 signatures; Prism ▸ Well combo; Criticality signature; AI rank 0 (${lo.casts} casts, longest ${lo.longest}) and rank 7 (${hi.casts} casts, longest ${hi.longest}, ${hi.signatures} signatures); 390 px without overflow; /science.html redirects.`);
})().catch(e=>{console.error(e);process.exitCode=1;}).finally(async()=>{if(browser)await browser.close();if(server)server.kill();});
