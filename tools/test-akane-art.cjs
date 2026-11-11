// Actual browser, installed map/actors, unchanged core and labelled modeled host.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawn}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/akane-art-review'),tmp=path.join(root,'backups/akane-art-tests');fs.mkdirSync(tmp,{recursive:true});
const server=spawn('node',['server.js','--port','7334','--no-open'],{cwd:path.join(root,'editor'),stdio:'ignore'});
const direct=spawn('python',['-m','http.server','7335','--bind','127.0.0.1','--directory',root],{cwd:root,stdio:'ignore'});
(async()=>{await new Promise(r=>setTimeout(r,1500));const browser=await chromium.launch({channel:'msedge',headless:true});const checks=[];
const check=(s,b)=>{assert(b,s);checks.push(s);console.log('ok - '+s);};
try{
 const page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://localhost:7334/akane.html');await page.waitForFunction(()=>window.akaneLab?.painter.loaded);await page.evaluate(async()=>{akaneLab.pause();await akaneLab.imagesReady;});
 for(const file of ['akane.html','editor/akane.html','akane-review.html','akane-body.png','akane.wasm','akane-renderer.js','akane-review-assets/effects.png','akane-review-assets/body.fanim'])check('Editor route '+file,(await page.request.get('http://localhost:7334/'+file)).status()===200);
 const initial=await page.evaluate(()=>{const L=akaneLab;return{scene:L.scene.kind,tick:L.tick,radius:L.me.radius,g:L.world.geometry,n:document.querySelector('#stockChampion').options.length,missing:L.units.filter(u=>!L.world.images.has(L.world.data.sprites[u.sprite].file)).length};});
 check('Default levelled gank at 8:00 with all original stock actors',initial.scene==='gank'&&initial.tick>=28800&&initial.radius===10000&&initial.missing===0&&initial.n===60);
 check('Verified 960,000 map, 1,000 world/art pixel, all 900 wall cells registered',initial.g.width===960000&&initial.g.units_per_pixel===1000&&initial.g.registration.matching===900);
 check('Canvas retains square pixels',await page.locator('#arena').evaluate(e=>Math.abs(e.getBoundingClientRect().width-e.getBoundingClientRect().height)<1));
 const clocks=await page.evaluate(()=>[30,60,120,144,240].map(hz=>{let n=0,c=new KratosClock();c.advance(0,true,()=>n++);for(let i=1;i<=hz*10;i++)c.advance(i*1000/hz,true,()=>n++);return[hz,n];}));
 check('30/60/120/144/240 Hz each produce exactly 600 ticks in ten seconds',clocks.every(([,n])=>n===600));
 const paused=await page.evaluate(()=>{let n=0,c=new KratosClock();c.advance(0,true,()=>n++);c.advance(10000,false,()=>n++);c.reset();c.advance(20000,true,()=>n++);c.advance(20000+1000/60,true,()=>n++);return n;});check('Pause/background resets have no catch-up ticks',paused===1);
 await page.evaluate(()=>{akaneLab.reset({fixture:true});akaneLab.pause();document.querySelector('#moving').checked=false;});await page.click('#step');check('Step advances exactly once',await page.evaluate(()=>akaneLab.tick===1));
 await page.focus('#arena');await page.keyboard.down('a');const moved=await page.evaluate(()=>{const L=akaneLab,x=L.me.pos.x;for(let i=0;i<3;i++)L.step();return L.me.pos.x<x;});check('Focused WASD moves through terrain validation',moved);
 await page.locator('details:has(#stats)').evaluate(e=>e.open=true);await page.locator('#stats input').first().focus();check('Focus loss clears held inputs',await page.evaluate(()=>akaneLab.held.length===0));await page.keyboard.up('a');
 const scenes=[];for(const kind of ['laning','gank','countergank','camp','morgard','serpen','defend','dive']){
  const r=await page.evaluate(async kind=>{const L=akaneLab;L.reset({scenario:kind});L.pause();document.querySelector('#auto').checked=true;document.querySelector('#moving').checked=true;await L.imagesReady;const start=L.tick;let legal=true;for(let i=0;i<180;i++){L.step();legal&&=!L.world.wallAt(L.me.pos,L.me.radius);}const d=L.world.data.doc;return{kind,n:L.units.length,ticks:L.tick-start,legal,error:L.lastTrace.output.error||null,damage:L.scene.metrics,realTowers:d.towers.flatMap(t=>t.pos).every(p=>L.units.some(u=>u.kind==='tower'&&u.home.x===p[0]&&u.home.y===p[1]))};},kind);scenes.push(r);
  check(kind+': bounded real-map actors and 180 legal shared-core ticks',r.n<=64&&r.ticks===180&&r.legal&&!r.error&&r.realTowers);await page.locator('#arena').screenshot({path:path.join(out,kind+'.png')});
 }
 await page.selectOption('#stockChampion','berserker');await page.click('#spawnStock');await page.evaluate(async()=>await akaneLab.imagesReady);check('Stock replacement loads original sprite/stats',await page.evaluate(()=>akaneLab.units.find(u=>u.id===2).sprite==='champ-berserker'));
 await page.click('#nativeSize');await page.waitForTimeout(70);check('Native-size inspection has one pixel per art pixel',await page.evaluate(()=>Math.abs(akaneLab.camera.s-.001)<1e-9));
 const fog=await page.evaluate(()=>{const L=akaneLab;L.reset({scenario:'laning'});L.pause();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;const u=L.units.find(u=>u.id===2);u.pos={x:880000,y:80000};L.step();const e=L.lastTrace.input.frame.units.find(u=>u.id===2);return!e.visible&&e.pos.x===0&&e.hp===0;});check('Hidden positions/HP withheld from core decisions',fog);
 const result=await page.evaluate(()=>{
  const L=akaneLab,run=(seed,art,profile)=>{
   L.reset({fixture:true});L.pause();document.querySelector('#moving').checked=false;document.querySelector('#auto').checked=false;document.querySelector('#bodies').checked=art;[...document.querySelectorAll('#stats input')].forEach((e,i)=>e.value=profile?.[i]??50);
   L.me.pos={x:440000,y:520000};const foe=L.units.find(u=>u.id===2);foe.pos={x:458000,y:520000};foe.hp=foe.max_hp=5000;L.setAim(foe.pos);const trace=[],all=[];
   for(let t=0;t<90;t++){
    if(t===0)L.order(['edge','kunai','soul'][seed%3]);if(t===32&&seed%3===1)L.order('kunai');if(t===65)L.order('night');if(t===67)L.order('night');
    if(seed%3===0&&[12,24,36,48,60].includes(t))L.acceptedBasic(2);if(t===70&&seed%4===0)L.me.hp=0;
    L.step();L.render();trace.push(structuredClone(L.lastTrace));all.push(JSON.stringify({output:L.lastTrace.output,me:L.me,units:L.units}));
   }return{trace,all:all.join('\n')};
  };
  const parity=[];for(let i=0;i<36;i++){const a=run(i,false),b=run(i,true);if(a.all!==b.all)throw Error('Artwork changed mechanics '+i);parity.push(...b.trace);}
  const stat=[];for(let s=0;s<13;s++)for(let f=0;f<3;f++){let low=Array(13).fill(50),high=[...low];low[s]=20;high[s]=80;const a=run(f,true,low),b=run(f,true,high);if(a.all!==b.all)throw Error('Manual profile affected mechanics '+s+'/'+f);stat.push([s,f]);}
  [...document.querySelectorAll('#stats input')].forEach(e=>e.value=50);document.querySelector('#bodies').checked=true;return{parity,stat};
 });
 check('24 development + 12 holdout art-on/off complete combat traces match',result.parity.length===3240);check('All 13 stats at 20/80 retain manual mechanics across three fixtures',result.stat.length===39);fs.writeFileSync(path.join(tmp,'parity.json'),JSON.stringify(result.parity));
 const cadence=await page.evaluate(()=>{const L=akaneLab;L.reset({fixture:true});L.pause();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;L.me.pos={x:440000,y:520000};const u=L.units.find(u=>u.id===2);u.pos={x:452000,y:520000};u.hp=u.max_hp=5000;const hp=u.hp;L.basic(2);L.step();const early=u.hp===hp;for(let i=0;i<7;i++)L.step();const release=u.hp<hp;const after=u.hp;L.basic(2);for(let i=0;i<5;i++)L.step();return early&&release&&u.hp===after;});check('Normal basics keep release and attack cadence',cadence);
 const flight=await page.evaluate(async()=>{
  const w=(await WebAssembly.instantiate(await(await fetch('akane.wasm')).arrayBuffer(),{})).instance.exports;
  const run=frame=>{const b=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:AKANE_RULES.revision,frame})),p=w.akane_input(b.length);new Uint8Array(w.memory.buffer,p,b.length).set(b);const q=w.akane_step(b.length);return JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,q,w.akane_output_len())));};
  const f={tick:0,owner:1,team:0,alive:true,pos:{x:400000,y:400000},hp:1600,max_hp:1600,attack:180,stunned:false,ult_unlocked:true,profile:Array(13).fill(50),basic_hits:[],takedowns:[],manual:true,order:{kind:'kunai',aim:{x:500000,y:400000}},units:[{id:2,team:1,pos:{x:500000,y:400000},hp:1000,max_hp:1000,radius:3000,champion:true,tower:false,visible:true,targetable:true}]};
  w.akane_reset();const first=run(f);f.order=null;let out;for(let t=1;t<=30;t++){f.tick=t;out=run(f);}const before=out;f.tick=31;const hit=run(f);
  return{first,before,hit};
 });
 check('Kunai launches without instant damage and starts at caster',flight.first.state.kunai_flights.length===1&&!flight.first.effects.some(e=>e.kind==='damage')&&flight.first.state.kunai_flights[0].pos.x===400000);
 check('100,000-distance hit takes 31 ticks and marks once on arrival',flight.before.state.mark===null&&flight.hit.state.mark[0]===2&&flight.hit.effects.filter(e=>e.kind==='damage').length===1&&flight.hit.effects.find(e=>e.kind==='damage').data.amount===138);
 check('Canonical extended ranges and slower movement reach the browser',await page.evaluate(()=>AKANE_RULES.kunai.flip===30000&&AKANE_RULES.kunai.flipTicks===30&&AKANE_RULES.kunai.range===120000&&AKANE_RULES.kunai.dashSpeed===3000&&AKANE_RULES.night.dashRange===240000&&AKANE_RULES.night.dashSpeed===4000));

 await page.click('#record');await page.evaluate(()=>{akaneLab.pause();for(let i=0;i<30;i++){if(i===2)akaneLab.order('soul');akaneLab.step();}});await page.click('#record');const replay=await page.evaluate(()=>{const L=akaneLab,saved=structuredClone(window.akaneRecording),end=JSON.stringify({me:L.me,units:L.units});L.loadRecording(saved);for(let i=0;i<saved.frames.length;i++)L.step();return saved.frames.length===30&&end===JSON.stringify({me:L.me,units:L.units})&&L.replayIndex===30;});check('Recording replays complete core outputs and visible host state',replay);
 const perf=await page.evaluate(()=>{const L=akaneLab,runs=[],p=(v,q)=>v.sort((a,b)=>a-b)[Math.floor((v.length-1)*q)];for(let k=0;k<5;k++){L.reset({scenario:'gank'});L.pause();document.querySelector('#auto').checked=true;document.querySelector('#moving').checked=true;for(let i=0;i<60;i++)L.step();let steps=[],draw=[];for(let i=0;i<300;i++){let t=performance.now();L.step();steps.push(performance.now()-t);t=performance.now();L.render();draw.push(performance.now()-t);}runs.push({ticks:300,step_p95_ms:p(steps,.95),draw_p95_ms:p(draw,.95)});}return runs;});
 const cold=[];for(let i=0;i<3;i++){const p=await browser.newPage(),start=Date.now();await p.goto('http://localhost:7334/akane.html');await p.waitForFunction(()=>window.akaneLab?.painter.loaded);await p.evaluate(async()=>{akaneLab.pause();await akaneLab.imagesReady;});cold.push(Date.now()-start);await p.close();}
 await page.goto('http://localhost:7335/editor/akane.html');await page.waitForFunction(()=>window.akaneLab?.painter.loaded);check('Direct /editor route loads actual assets',await page.evaluate(()=>akaneLab.painter.loaded));await page.evaluate(()=>akaneLab.pause());
 await page.goto('http://localhost:7334/akane-review.html');await page.waitForFunction(()=>window.akaneArtReview?.painter.loaded);check('Review has nine full body poses, both facings and effects',await page.evaluate(()=>akaneArtReview.painter.manifest.poses.length===9&&Object.keys(akaneArtReview.painter.anims.effects).length>=20));
 await page.locator('#poses').screenshot({path:path.join(out,'animated-inspector.png')});check('No browser errors',errors.length===0);
 const report={checks:checks.length,passed:checks,scenarios:scenes,manual_stat_comparisons:result.stat.length,parity_frames:result.parity.length,clocks,cold_page_assets_ms:cold,matched_runs:perf,scope:'Lab + modeled host only; no native-game visual or combat acceptance',live_acceptance:'pending'};fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({checks:checks.length,scenarios:scenes.length,perf,cold}));
}finally{await browser.close();server.kill();direct.kill();}
})().catch(e=>{console.error(e);process.exit(1);});
