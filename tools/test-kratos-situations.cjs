const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),crypto=require('crypto'),{spawn}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/kratos-game-world-review');fs.mkdirSync(out,{recursive:true});
const server=spawn('node',['server.js','--port','7330','--no-open'],{cwd:path.join(root,'editor'),stdio:'ignore'});
(async()=>{await new Promise(r=>setTimeout(r,1500));const browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[],checks=[];page.on('pageerror',e=>errors.push(e.message));
const check=(s,b)=>{assert(b,s);checks.push(s);console.log('ok - '+s);};
try{
 await page.goto('http://127.0.0.1:7330/kratos.html');await page.waitForFunction(()=>window.kratosLab?.painter.loaded);await page.evaluate(async()=>{kratosLab.pause();await kratosLab.imagesReady;});
 const initial=await page.evaluate(()=>{const L=kratosLab;return{scene:L.scene.kind,tick:L.tick,radius:L.me.radius,geometry:L.world.geometry,stock:document.querySelector('#stockChampion').options.length,units:L.units.length,missing:L.units.filter(u=>!L.world.images.has(L.world.data.sprites[u.sprite].file)).length};});
 check('Default is a levelled mid-game lane snapshot, not a coloured-dot fixture',initial.scene==='laning'&&initial.tick>=28800&&initial.radius===10000);
 check('All 60 stock champion choices are available from installed definitions',initial.stock===60);
 check('All scenario actors have their original extracted sprite loaded',initial.missing===0&&initial.units<=64);
 check('Map is 960,000 square with 160px art border and 1,000 world units per pixel',initial.geometry.width===960000&&initial.geometry.art_padding_pixels===160&&initial.geometry.units_per_pixel===1000);
 check('All 900 terrain cells agree with the installed wall pixels',initial.geometry.registration.matching===900);
 const proof=await page.evaluate(()=>{const L=kratosLab,w=L.world,c=L.wallCrossing(),radius=L.me.radius;let p=w.legal(c.from,radius),crossed=false;for(let i=0;i<100;i++){const to=w.sweep(p,{x:p.x+2000,y:p.y},radius);p=to;if(p.x>c.from.x+32000)crossed=true;}
   const path=w.path(c.from,c.to,radius);let a={pos:{...c.from},radius};for(let t=0;t<500;t++)w.move(a,c.to,1000);return{blocked:!crossed,legal:!w.wallAt(p,radius),route:path.length>0,arrived:Math.hypot(a.pos.x-c.to.x,a.pos.y-c.to.y)<6000};});
 check('Ordinary swept walking stops outside a wall including champion radius',proof.blocked&&proof.legal);
 check('Navigation routes around a wall instead of walking through it',proof.route&&proof.arrived);
 const scenes=[];
 for(const kind of ['laning','gank','countergank','camp','morgard','serpen','defend','dive']){
  await page.selectOption('#scenario',kind);await page.click('#loadScenario');await page.evaluate(async()=>{kratosLab.pause();await kratosLab.imagesReady;});
  const state=await page.evaluate(()=>{const L=kratosLab,now=L.tick;let legal=true;for(let i=0;i<240;i++){L.step();legal&&=!L.world.wallAt(L.me.pos,L.me.radius);}
    const src=L.world.data.doc,realTowers=src.towers.flatMap(t=>t.pos).every(p=>L.units.some(u=>u.kind==='tower'&&u.home.x===p[0]&&u.home.y===p[1]));
    const camp=src.camps.every(c=>c.pos.slice(0,['Morgard','Serpen'].includes(c.ty)?1:2).every(p=>L.units.some(u=>u.name===c.ty&&u.home.x===p[0]&&u.home.y===p[1])));
    return{kind:L.scene.kind,elapsed:L.tick-now,legal,realTowers,camp,units:L.units.length,alive:L.me.hp>0,scene:L.scene.description()};});scenes.push(state);
  check(kind+': real structures/camps, bounded actors, legal host movement and 240 shared-core ticks',state.elapsed===240&&state.legal&&state.realTowers&&state.camp&&state.units<=64);
  await page.locator('#arena').screenshot({path:path.join(out,kind+'.png')});
 }
 await page.selectOption('#stockChampion','berserker');await page.click('#spawnStock');await page.evaluate(async()=>await kratosLab.imagesReady);
 check('A stock replacement uses the extracted body and real stat definition',await page.evaluate(()=>{const u=kratosLab.units.find(u=>u.id===2);return u.sprite==='champ-berserker'&&u.stats.attack>0;}));
 await page.click('#nativeSize');check('Native-size inspection uses exactly 1 screen pixel per art pixel in the canvas',await page.evaluate(()=>Math.abs(kratosLab.camera.s-.001)<1e-8));
 await page.uncheck('#zoom');await page.waitForTimeout(80);await page.locator('#arena').screenshot({path:path.join(out,'full-map.png')});
 await page.locator('#minimap').click({position:{x:55,y:55}});await page.waitForTimeout(50);
 check('Minimap inspection pans without moving Kratos',await page.evaluate(()=>kratosLab.camera.x!==kratosLab.me.pos.x));
 const fog=await page.evaluate(()=>{const L=kratosLab;document.querySelector('#scenario').value='laning';L.reset();L.pause();L.units.find(u=>u.id===2).pos={x:880000,y:80000};L.step();const entry=L.lastTrace.input.frame.units.find(u=>u.id===2);return!entry.visible&&entry.pos.x===0&&entry.hp===0;});
 check('Hidden current positions and HP are withheld from the core',fog);
 const unchanged=await page.evaluate(()=>{
  const L=kratosLab,run=(kind,art)=>{L.reset({scenario:kind});L.pause();document.querySelector('#moving').checked=true;document.querySelector('#auto').checked=false;document.querySelector('#bodies').checked=art;
   const trace=[];for(let t=0;t<120;t++){if(t===0)L.order('rampage');if(t===70)L.order('guard');L.step();L.render();trace.push({core:L.lastTrace,me:L.me,units:L.units,shots:L.scene.shots});trace[t]=JSON.stringify(trace[t]);}return trace.join('\n');};
  for(const kind of Object.keys(GameScenario.choices))if(run(kind,false)!==run(kind,true))return false;document.querySelector('#bodies').checked=true;return true;
 });
 check('All eight situations preserve complete core and modeled host observations with artwork off/on',unchanged);
 const manifest=JSON.parse(fs.readFileSync(path.join(root,'editor/game-reference/world.json')));
 const originals=[...manifest.layers.map(l=>[l.file,'asset/base/aseprite_resources/ingame/5v5/'+l.name]),...Object.values(manifest.sprites).map(s=>[s.file,s.key+'#sheet'])];
 check('All 77 local map/sprite PNGs match their extracted source hashes',originals.length===77&&originals.every(([file,key])=>crypto.createHash('sha256').update(fs.readFileSync(path.join(root,'editor/game-reference',file))).digest('hex')===manifest.sources[key].sha256));
 await page.route('**/game-reference/world.json',route=>route.fulfill({status:404,body:'missing reference'}));
 const missing=await page.evaluate(async()=>{try{await new GameWorld().ready;return false;}catch(e){return e.message.includes('no estimated Levi fallback');}});
 check('Missing installed reference fails explicitly instead of using an approximate map',missing);await page.unroute('**/game-reference/world.json');
 const performance=await page.evaluate(()=>{
  const L=kratosLab,pct=(v,q)=>v.sort((a,b)=>a-b)[Math.floor((v.length-1)*q)],runs=[];
  for(let run=0;run<5;run++){
   L.reset({scenario:'laning'});L.pause();document.querySelector('#moving').checked=true;document.querySelector('#auto').checked=true;
   for(let i=0;i<60;i++)L.step();const step=[],draw=[];
   for(let i=0;i<600;i++){let t=performance.now();L.step();step.push(performance.now()-t);t=performance.now();L.render();draw.push(performance.now()-t);}
   runs.push({ticks:600,step_ms:{p50:pct(step,.5),p95:pct(step,.95)},draw_ms:{p50:pct(draw,.5),p95:pct(draw,.95)}});
  }
  document.querySelector('#auto').checked=false;return{runs,scope:'55 original stock actors + Kratos, 60-tick warmup then 600 ticks; settings-based host + unchanged core with AI; draw excludes DOM inspector. Browser timing, not native performance.'};
 });
 const cold=[];for(let i=0;i<5;i++){const p=await browser.newPage();const start=Date.now();await p.goto('http://127.0.0.1:7330/kratos.html');await p.waitForFunction(()=>window.kratosLab?.painter.loaded);await p.evaluate(async()=>{kratosLab.pause();await kratosLab.imagesReady;});cold.push(Date.now()-start);await p.close();}
 performance.cold_page_and_assets_ms=cold;
 check('No page errors: '+errors.join('; '),errors.length===0);
 const report={checks:checks.length,passed:checks,geometry:initial.geometry,scenarios:scenes,performance,source:manifest.source,scope:'Installed original assets and settings; modeled stock movement/attacks/mitigation/vision, not the native stock engine. Kratos mechanics use the shared kratos-2 Rust/WASM core.'};
 fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify(report,null,2)+'\n');console.log(checks.length+' checks passed');
}finally{await browser.close();server.kill();}})().catch(e=>{console.error(e);server.kill();process.exitCode=1;});
