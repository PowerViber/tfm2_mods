// Browser acceptance for artwork, fixed-step playback and unchanged shared-core traces.
const assert=require('assert'),fs=require('fs'),path=require('path'),os=require('os'),crypto=require('crypto'),{spawn}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/kratos-art-review'),tmp=path.join(root,'backups/kratos-art-tests');fs.mkdirSync(tmp,{recursive:true});
const server=spawn('node',['server.js','--port','7327','--no-open'],{cwd:path.join(root,'editor'),stdio:'ignore'});
const direct=spawn('python',['-m','http.server','7328','--bind','127.0.0.1','--directory',root],{cwd:root,stdio:'ignore'});
const hash=x=>crypto.createHash('sha256').update(JSON.stringify(x)).digest('hex');
(async()=>{await new Promise(r=>setTimeout(r,1500));const browser=await chromium.launch({channel:'msedge',headless:true});let count=0;const checks=[];
const check=(name,ok)=>{assert(ok,name);count++;checks.push(name);console.log('ok - '+name);};
try{
 const page=await browser.newPage({viewport:{width:1340,height:1000}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://127.0.0.1:7327/kratos.html');await page.waitForFunction(()=>window.kratosLab?.painter.loaded);await page.evaluate(()=>kratosLab.pause());
 const urls=['kratos-review.html','kratos-clock.js','kratos-renderer.js','kratos.wasm','kratos-body.png','kratos-weapons.png','kratos-effects.png','kratos-review-assets/presentation.json','kratos-review-assets/body-preview.fanim','kratos-review-assets/weapons-preview.fanim','kratos-review-assets/effects-preview.fanim'];
 for(const u of urls)check('Editor route '+u,(await page.request.get('http://127.0.0.1:7327/'+u)).status()===200);
 check('Actual packaged body and weapon assets loaded',await page.evaluate(()=>kratosLab.painter.images.body.src.endsWith('/kratos-body.png')&&kratosLab.painter.images.weapons.src.endsWith('/kratos-weapons.png')));
 check('Arena retains square pixels alongside the tall inspector',await page.locator('#arena').evaluate(e=>Math.abs(e.getBoundingClientRect().width-e.getBoundingClientRect().height)<1));
 const clocks=await page.evaluate(()=>[30,60,120,144,240].map(hz=>{const c=new KratosClock();let n=0;c.advance(0,true,()=>n++);for(let i=1;i<=hz*10;i++)c.advance(i*1000/hz,true,()=>n++);return[hz,n];}));
 check('30/60/120/144/240 Hz all run exactly 600 ticks in ten seconds',clocks.every(([,n])=>n===600));
 const noCatchup=await page.evaluate(()=>{const c=new KratosClock();let n=0;c.advance(0,true,()=>n++);c.advance(10000,false,()=>n++);c.reset();c.advance(20000,true,()=>n++);c.advance(20000+1000/60,true,()=>n++);return n;});
 check('Pause/background reset adds no catch-up ticks',noCatchup===1);
 await page.evaluate(()=>{kratosLab.reset();kratosLab.pause();});await page.locator('#step').click();await page.waitForTimeout(100);
 check('Step advances once and remains paused',await page.evaluate(()=>kratosLab.tick===1));
 await page.locator('#arena').focus();await page.keyboard.down('a');const moved=await page.evaluate(()=>{const L=kratosLab,x=L.me.pos.x;for(let i=0;i<3;i++)L.step();return L.me.pos.x<x;});
 check('Focused WASD supplies movement',moved);await page.locator('details').evaluate(e=>e.open=true);await page.locator('#stats input').first().focus();
 check('Focus loss clears held movement',await page.evaluate(()=>kratosLab.held.length===0));
 await page.keyboard.press('1');check('Typing in an attribute does not cast',await page.evaluate(()=>kratosLab.state.sight_until===0));
 const result=await page.evaluate(()=>{
   const L=kratosLab,run=n=>{for(let i=0;i<n;i++)L.step();};
   const scenario=(seed,art,profile)=>{
    L.pause();L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;document.querySelector('#bodies').checked=art;
    [...document.querySelectorAll('#stats input')].forEach((e,i)=>e.value=profile?.[i]??50);
    const foe=L.units.find(u=>u.id===2);foe.pos={x:L.me.pos.x+9000+seed%3*2000,y:L.me.pos.y+seed%2*3000};
    L.setAim({x:L.me.pos.x+90000,y:L.me.pos.y});const trace=[];
    for(let tick=0;tick<135;tick++){
      if(tick===0)L.order(seed%3===0?'rampage':seed%3===1?'guard':'fenrir');
      if(tick===16&&seed%3===1)L.order('dash');if(tick===49&&seed%3===2)L.order('slam');
      if(tick===80)L.order('sight');L.step();trace.push(L.lastTrace);
    }
    return trace;
   };
   let manualEqual=true,drawEqual=true;const sample=[];
   for(let i=0;i<36;i++){const off=scenario(i,false),on=scenario(i,true);drawEqual&&=JSON.stringify(off)===JSON.stringify(on);if(i<3)sample.push(...on);}
   for(let stat=0;stat<13;stat++)for(let f=0;f<3;f++){
    const low=Array(13).fill(50),high=Array(13).fill(50);low[stat]=20;high[stat]=80;
    const strip=trace=>trace.map(({input,output})=>({input:{...input,frame:{...input.frame,profile:undefined}},output}));
    manualEqual&&=JSON.stringify(strip(scenario(f,true,low)))===JSON.stringify(strip(scenario(f,true,high)));
   }
   document.querySelector('#record').click();L.pause();document.querySelector('#moving').checked=false;L.order('rampage');run(100);document.querySelector('#record').click();
   const rec=JSON.parse(JSON.stringify(window.kratosRecording));L.loadRecording(rec);run(rec.frames.length);const replayOK=L.replayIndex===rec.frames.length;
   let oldRejected=false;try{L.loadRecording({...rec,rules_revision:'kratos-old'});}catch(e){oldRejected=e.message.includes('original');}
   L.reset();L.pause();document.querySelector('#bodies').checked=true;L.startDemo('fenrir');L.pause();run(52);
   return{drawEqual,manualEqual,replayOK,oldRejected,sample,frames:rec.frames.length};
 });
 check('24 development + 12 holdout scenarios have identical complete core traces with artwork off/on',result.drawEqual);
 check('All thirteen manual 20/80 comparisons agree across three fixtures each',result.manualEqual);
 check('Recorded inputs replay with every original core output identical',result.replayOK&&result.frames===100);
 check('Older rules are explicitly rejected for replay',result.oldRejected);
 fs.writeFileSync(path.join(tmp,'parity.json'),JSON.stringify(result.sample));
 await page.screenshot({path:path.join(out,'lab.png')});
 const warm=await page.evaluate(()=>{const p=kratosLab.painter,ctx=document.createElement('canvas').getContext('2d'),samples=[];ctx.canvas.width=900;ctx.canvas.height=900;
  for(let run=0;run<5;run++){const ts=[];for(let t=0;t<300;t++){const start=performance.now();for(let i=0;i<12;i++)p.body(ctx,'run',t,40+i*60,200,1,i%2===0,i%3===0);p.wolf(ctx,{x:0,y:0},{x:100,y:50},t,300,300,1);ts.push(performance.now()-start);}ts.sort((a,b)=>a-b);samples.push({p50:ts[150],p95:ts[285]});}return samples;});
 await page.goto('http://127.0.0.1:7327/kratos-review.html');await page.waitForFunction(()=>window.kratosArtReview?.painter.loaded);
 check('Full pose/effect review loads with the actual packed atlases',await page.locator('#pose option').count()===12&&await page.locator('#effect option').count()>15);
 await page.locator('#pose').selectOption('run');await page.waitForTimeout(200);await page.locator('#poses').screenshot({path:path.join(out,'pose-inspector.png')});
 await page.goto('http://127.0.0.1:7328/editor/kratos.html');await page.waitForFunction(()=>window.kratosLab?.painter.loaded);await page.evaluate(()=>kratosLab.pause());
 check('Direct /editor/ route loads the same packaged artwork',await page.evaluate(()=>kratosLab.painter.images.body.src.includes('/mods/tfm2_custom/champions/')));
 check('No browser errors: '+errors.join('; '),!errors.length);
 const cold=[];for(let i=0;i<5;i++){const c=await browser.newPage();const t=performance.now();await c.goto('http://127.0.0.1:7327/kratos.html');await c.waitForFunction(()=>window.kratosLab?.painter.loaded);cold.push(performance.now()-t);await c.close();}
 const report={checks:count,passed:checks,scenario_counts:{development:24,holdout:12,manual_stat_pairs:39},trace_sha256:hash(result.sample),warm_render_ms:warm,cold_navigation_and_art_ms:cold,
   performance_scope:'Browser rendering only, 12 bodies + Fenrir, five matched runs; not a native/host benchmark or Magnus 128-piece acceptance.',native_observation:'pending'};
 fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify(report,null,2)+'\n');console.log(count+' checks passed');
}finally{await browser.close();server.kill();direct.kill();}})().catch(e=>{console.error(e);server.kill();direct.kill();process.exitCode=1;});
