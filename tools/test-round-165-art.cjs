// Supplemental presentation, actual packaged assets, and complete WASM/native traces.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawn,spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),os.homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/round-165-art-review'),tmp=path.join(root,'backups/round-165-browser');
fs.mkdirSync(tmp,{recursive:true});
const server=spawn('node',['server.js','--port','7336','--no-open'],{cwd:path.join(root,'editor'),stdio:'ignore'});
const direct=spawn('python',['-m','http.server','7337','--bind','127.0.0.1','--directory',root],{cwd:root,stdio:'ignore'});
const checks=[],traces={};
function check(label,ok){assert(ok,label);checks.push(label);console.log('ok - '+label);}
async function ready(port){for(let i=0;i<100;i++){try{if((await fetch('http://127.0.0.1:'+port+'/')).ok)return;}catch{}await new Promise(r=>setTimeout(r,100));}throw Error('Server did not start '+port);}
(async()=>{await Promise.all([ready(7336),ready(7337)]);const browser=await chromium.launch(process.platform==='win32'?{channel:'msedge',headless:true}:{headless:true});
try{
 const errors=[],page=await browser.newPage({viewport:{width:1440,height:1100}});page.on('pageerror',e=>errors.push(e.message));
 for(const file of ['round-165-art-review.html','round-165-art.js','round-165-renderer.js','round-165-fx.png','round-165-review-assets/effects-comparison.png','editor/round-165-art-review.html'])
  check('Editor route '+file,(await page.request.get('http://127.0.0.1:7336/'+file)).status()===200);
 await page.goto('http://127.0.0.1:7336/round-165-art-review.html');await page.waitForFunction(()=>window.round165Review?.art.loaded);
 const rv=await page.evaluate(()=>{const L=round165Review,yes=L.effects.some(e=>e.kind==='fx'&&e.data.name==='tls_surge'&&e.data.ticks===18);document.querySelector('#water').checked=false;L.fire();const no=!L.effects.some(e=>e.kind==='fx'&&e.data.name==='tls_surge'&&e.data.ticks===18);document.querySelector('#water').checked=true;L.fire();return{yes,no,map:L.world.geometry.registration.matching,asset:L.art.image.src};});
 check('Actual WASM review triggers a wave only in water',rv.yes&&rv.no);
 check('Review uses the installed 900-cell map and packaged auxiliary atlas',rv.map===900&&rv.asset.endsWith('/round-165-fx.png'));
 await page.locator('#mapCanvas').screenshot({path:path.join(out,'water-map-preview.png')});
 // Every heading and frame draws its packaged rectangle, including left-facing inspection.
 const pixels=await page.evaluate(()=>{const a=round165Review.art,c=document.createElement('canvas');c.width=c.height=512;const ctx=c.getContext('2d');let n=0,filled=0;for(const [tag,spec]of Object.entries(a.manifest.frames)){let age=0;for(const f of spec.frames){ctx.clearRect(0,0,512,512);a.effect(ctx,tag,age,256,256,1);const d=ctx.getImageData(0,0,512,512).data;for(let i=3;i<d.length;i+=4)if(d[i]){filled++;break;}n++;age+=f.duration*60;}}return{n,filled};});
 check('All supplemental frames/headings have visible pixels',pixels.n===pixels.filled&&pixels.n>1000);
 // Actual Lab statuses are drawn from the same active core state as native.
 await page.goto('http://127.0.0.1:7336/akane.html');await page.waitForFunction(()=>window.akaneLab?.roundArt.loaded);
 const ak=await page.evaluate(async()=>{const L=akaneLab;L.pause();L.reset({fixture:true});document.querySelector('#auto').checked=false;document.querySelector('#autoBasics').checked=false;document.querySelector('#moving').checked=false;const e=L.units.find(u=>u.id===2);e.hp=e.max_hp=100000;e.pos={x:L.me.pos.x+12000,y:L.me.pos.y};const tags=[],effect=L.roundArt.effect.bind(L.roundArt);L.roundArt.effect=(...a)=>{tags.push(a[1]);return effect(...a);};
 for(let i=0;i<5;i++){L.acceptedBasic(2);L.step();L.render();}const rhythm=L.me.rhythm;L.order('soul');L.step();L.render();for(let i=0;i<100;i++)L.step();L.render();const alive=[...new Set(tags)];tags.length=0;L.me.hp=0;L.step();L.render();return{rhythm,alive,dead:tags.length};});
 check('Akane renders five Rhythm states, Dismiss and Ambush',ak.rhythm===5&&['rhythm1','rhythm5','dismiss_aura','ambush'].every(s=>ak.alive.includes(s)));
 check('Dead owners have no remaining Akane indicators',ak.dead===0);
 // Full traces compare art on/off; native parity is checked outside the browser.
 const cold=[],perf=[];
 for(const name of ['wright','thalassa']){
  const start=Date.now();await page.goto('http://127.0.0.1:7336/'+name+'.html');await page.waitForFunction(n=>window[n+'Lab']?.roundArt.loaded&&window[n+'Lab']?.painter.loaded,name);cold.push({name,ready_ms:Date.now()-start});
  const r=await page.evaluate(async name=>{
   const L=window[name+'Lab'];L.pause();const control=document.querySelector('#art'),all=[];
   const run=art=>{L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;if(document.querySelector('#incomingFire'))document.querySelector('#incomingFire').checked=false;control.checked=art;const trace=[];L.setRecorder((input,output)=>trace.push({input:JSON.parse(input),output:JSON.parse(output)}));
    for(let i=0;i<360;i++){if(i===0)L.order(name==='wright'?'carrier':'tide_pool');if(i===40)L.order(name==='wright'?'bomber':'twin');if(i===200)L.order(name==='wright'?'fighters':'ult');L.step();}
    L.setRecorder(null);return{trace,state:JSON.stringify({state:L.state,me:L.me,units:L.units})};};
   const a=run(false),b=run(true);if(a.state!==b.state||JSON.stringify(a.trace)!==JSON.stringify(b.trace))throw Error(name+' artwork changed combat');
   const tags=[],old=L.roundArt.effect.bind(L.roundArt);L.roundArt.effect=(...v)=>{tags.push(v[1]);return old(...v);};L.reset();control.checked=true;document.querySelector('#auto').checked=false;L.order(name==='wright'?'carrier':'tide_pool');for(let i=0;i<90;i++)L.step();await new Promise(r=>requestAnimationFrame(r));
   const indicator=tags.includes('carrier_empowered'),planes=L.state.planes?.length||0;const aliases=L.roundArt.manifest.wright_aliases;
   const frames=L.roundArt.manifest.frames,warningTicks=[60,96,132].map((t,i)=>{const tag=aliases['wrt_warn'+(i+1)+':'+t];return frames[tag].frames.reduce((s,f)=>s+f.duration*60,0);});
   return{trace:b.trace,indicator,planes,warningTicks,bindings:frames!==undefined};
  },name);
  traces[name]=r.trace;check(name+': complete combat state/commands equal with artwork on/off',r.bindings);
  if(name==='wright'){check('Wright carrier indicator follows guard; carrier spawns no aircraft',r.indicator&&r.planes===0);check('Bomber warning phases preserve 60/96/132 ticks',r.warningTicks.every((x,i)=>Math.abs(x-[60,96,132][i])<1e-6));}
  // Five matched draw/step runs; host/UI timings, not live-engine benchmarks.
  perf.push(await page.evaluate(name=>{const L=window[name+'Lab'],runs=[];for(let k=0;k<5;k++){L.reset();L.pause();document.querySelector('#auto').checked=false;L.order(name==='wright'?'fighters':'twin');const a=[];for(let i=0;i<240;i++){const t=performance.now();L.step();a.push(performance.now()-t);}a.sort((a,b)=>a-b);runs.push(a[Math.floor(a.length*.95)]);}return{name,step_p95_ms:runs};},name));
  await page.goto('http://127.0.0.1:7337/editor/'+name+'.html');await page.waitForFunction(n=>window[n+'Lab']?.roundArt.loaded,name);check(name+': direct /editor route loads packaged artwork',await page.evaluate(n=>window[n+'Lab'].roundArt.image.naturalWidth>0,name));await page.evaluate(n=>window[n+'Lab'].pause(),name);
 }
 // Exercise held F, cadence, water-only wave and extended owner reach.
 await page.goto('http://127.0.0.1:7336/thalassa.html');await page.waitForFunction(()=>window.thalassaLab?.roundArt.loaded);
 async function basic(water){await page.evaluate(water=>{const L=thalassaLab;L.pause();L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;const e=L.units[0];e.hp=e.max_hp=100000;e.pos={x:L.me.pos.x+(water?80000:50000),y:L.me.pos.y};L.units[1].hp=L.units[1].max_hp=100000;L.units[1].pos={x:e.pos.x+7000,y:e.pos.y};window.basicTrace=[];L.setRecorder((i,o)=>basicTrace.push(JSON.parse(o)));if(water)L.order('tide_pool');for(let t=0;t<20;t++)L.step();},water);await page.focus('#arena');await page.keyboard.down('f');await page.evaluate(()=>{for(let i=0;i<119;i++)thalassaLab.step();});await page.keyboard.up('f');return await page.evaluate(()=>{thalassaLab.setRecorder(null);return{wave:basicTrace.flatMap(x=>x.effects).filter(e=>e.kind==='fx'&&e.data.name==='tls_surge'&&e.data.ticks===18).length,hp:thalassaLab.units[0].hp,secondary:thalassaLab.units[1].hp,water:thalassaLab.state.in_water};});}
 const land=await basic(false),water=await basic(true);
 check('Held F lands one ordinary bolt within the 120-tick cadence on land',land.hp===99920&&land.wave===0);
 check('Water basic reaches 80,000 and emits exactly one tsunami',water.hp===99920&&water.secondary===99940&&water.wave===1&&water.water);
 await page.goto('http://127.0.0.1:7337/editor/round-165-art-review.html');await page.waitForFunction(()=>window.round165Review?.art.loaded);check('Direct combined review route loads packaged assets',await page.evaluate(()=>round165Review.art.loaded));
 check('No browser page errors',errors.length===0);
 // Complete native/WASM traces; a fresh process owns each sequential session.
 const parity={};for(const name of ['wright','thalassa']){
  const build=spawnSync('rtk',['proxy','cargo','+stable-x86_64-pc-windows-gnu','build','--release','--manifest-path','native/'+name+'_lab/Cargo.toml','--bin','parity'],{cwd:root,encoding:'utf8'});if(build.status)throw Error(build.stderr);
  const bin=path.join(root,'native',name+'_lab','target','release','parity'+(process.platform==='win32'?'.exe':''));
  const r=spawnSync(bin,[],{cwd:root,input:traces[name].map(x=>JSON.stringify(x.input)).join('\n')+'\n',encoding:'utf8',maxBuffer:20*1024*1024});if(r.status)throw Error(r.stderr);
  const actual=r.stdout.trim().split(/\r?\n/).map(JSON.parse);assert.deepStrictEqual(actual,traces[name].map(x=>x.output),name+' parity');parity[name]=actual.length;check(name+': complete native/WASM traces match',actual.length===360);
 }
 fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify({checks:checks.length,passed:checks,parity_frames:parity,cold_page_assets_ms:cold,five_run_lab_step:perf,all_supplement_frames:pixels.n,scope:'Browser/WASM + modeled host; actual-game acceptance pending'},null,2));
 console.log(JSON.stringify({checks:checks.length,parity,cold,perf}));
}finally{await browser.close();server.kill();direct.kill();}
})().catch(e=>{console.error(e);process.exit(1);});

