const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const out=path.join(__dirname,'../docs/raze-results');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.LAB_URL||'http://localhost:7286')+'/editor/raze.html');await page.waitForFunction(()=>window.RazeLab);
 assert.equal(await page.evaluate(()=>RAZE_RULES.rules_revision),'raze-3.6');
 await page.evaluate(async()=>{RazeLab.pause();RazeLab.setAutomatic(false);RazeLab.units[1].pos={x:313000,y:300000};await RazeLab.step();});
 const basic=await page.evaluate(()=>({hp:RazeLab.units[1].hp,gun:RazeLab.state.gun}));assert.equal(basic.gun,6);assert.equal(basic.hp,1700); // round 144: Bucky 300% through 10,000, 240% at 13,000
 await page.locator('#reset').click();await page.evaluate(async()=>{const L=RazeLab;L.pause();L.setAutomatic(false);L.units[0].attack=65;await L.step();L.queueAction({kind:'grenade',aim:{...L.units[1].pos}});await new Promise(r=>setTimeout(r,10));await L.step(130);});
 const grenade=await page.evaluate(()=>({bySource:RazeLab.state.metrics.damage_by_source,hp:RazeLab.units[1].hp,events:document.querySelector('#events').textContent}));
 assert.equal(grenade.bySource.grenade,266);assert(grenade.events.includes('266 raw'));assert.equal(grenade.hp,1579);
 await page.screenshot({path:path.join(out,'raze-attack-damage.png'),fullPage:true});assert((await page.evaluate(()=>RazeLab.verify())).identical);
 await page.selectOption('#scenario','unsupported');await page.evaluate(async()=>{RazeLab.setAutomatic(true);await RazeLab.step(24);RazeLab.pause();});
 const unsupported=await page.evaluate(()=>RazeLab.state.candidates.filter(c=>['double_launch','rocket_entry'].includes(c.family)).map(c=>({family:c.family,legal:c.legal,entry:c.entry})));
 assert(unsupported.length>0);assert(unsupported.every(c=>!c.legal&&c.entry.ally===null));
 await page.screenshot({path:path.join(out,'raze-entry-review.png'),fullPage:true});
 await page.selectOption('#scenario','lethal-entry');await page.evaluate(async()=>{RazeLab.setAutomatic(true);await RazeLab.step(24);RazeLab.pause();});
 const dangerous=await page.evaluate(()=>RazeLab.state.candidates.filter(c=>c.entry));assert(dangerous.length>0&&dangerous.every(c=>!c.legal));
 // Those checks guide athletes; the same manual launch remains mechanically legal.
 await page.locator('#reset').click();await page.evaluate(async()=>{const L=RazeLab;L.setAutomatic(false);await L.step();L.queueAction({kind:'pack',aim:{x:290000,y:300000}});await new Promise(r=>setTimeout(r,10));await L.step(8);L.queueAction({kind:'detonate',id:0});await new Promise(r=>setTimeout(r,10));});
 assert(await page.evaluate(()=>!!RazeLab.state.flight));assert((await page.evaluate(()=>RazeLab.verify())).identical);assert.deepStrictEqual(errors,[]);
 const report={rulesRevision:'raze-3.6',buckyBasic:basic,grenade,unsupported,dangerousEntryRejected:true,manualLaunchLegal:true,replay:true,errors};fs.writeFileSync(path.join(out,'balance-browser.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({buckyFalloff:true,attack65Damage266:true,minionsNotFollowup:true,lethalEntriesRejected:true,manualLaunchLegal:true,replay:true,errors}));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
