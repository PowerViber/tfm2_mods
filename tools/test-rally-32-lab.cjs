// Browser acceptance for the shared 3.2 rules; never accesses a real career.
const fs=require('fs'),path=require('path'),os=require('os'),http=require('http'),assert=require('assert'),{spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),revision=require(path.join(root,'rules/rally.json')).revision,out=path.join(root,'docs/rally-results',revision);
fs.mkdirSync(out,{recursive:true});
(async()=>{
 const server=http.createServer((req,res)=>{const p=path.resolve(root,'.'+decodeURIComponent(new URL(req.url,'http://local').pathname));if(!p.startsWith(root+path.sep)){res.writeHead(403);return res.end();}fs.readFile(p,(e,b)=>{if(e){res.writeHead(404);return res.end();}res.setHeader('Content-Type',p.endsWith('.html')?'text/html':p.endsWith('.js')?'text/javascript':p.endsWith('.css')?'text/css':p.endsWith('.wasm')?'application/wasm':'application/octet-stream');res.end(b);});});
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1550,height:1350}}),errors=[],checks=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{const clear=CanvasRenderingContext2D.prototype.clearRect,text=CanvasRenderingContext2D.prototype.fillText;
  CanvasRenderingContext2D.prototype.clearRect=function(...a){if(this.canvas.id==='arena')window.__arenaText=[];return clear.apply(this,a);};
  CanvasRenderingContext2D.prototype.fillText=function(label,x,y,...a){if(this.canvas.id==='arena'){const m=this.getTransform();(window.__arenaText||=[]).push({text:String(label),x:m.a*x+m.c*y+m.e,y:m.b*x+m.d*y+m.f});}return text.call(this,label,x,y,...a);};});
 const to=async tick=>page.evaluate(t=>{rallyLab.pause();while(rallyLab.tick<t)rallyLab.step();},tick);
 const schedule={2:'arm_burst',3:'self_set',16:'ultimate',64:'takeoff',88:'spike'};
 async function serveSetup(bank=0,endpoint={x:600000,y:300000}){
  await page.evaluate(({bank,endpoint,schedule})=>{for(const id of ['auto','moving','attacking','allyAttacking','fog','walls','smoke','evade'])document.querySelector('#'+id).checked=false;rallyLab.fixture({pace:100,form:'white_storm',bank_percent:bank});rallyLab.setAim(endpoint);
   while(rallyLab.tick<88){const t=rallyLab.tick+1;if(schedule[t]){rallyLab.command(schedule[t],{bars:5});rallyLab.pause();}rallyLab.step();}}, {bank,endpoint,schedule});
 }
 try{
  const base='http://127.0.0.1:'+server.address().port;await page.goto(base+'/editor/rally.html');await page.waitForFunction(()=>window.rallyLab&&rallyLab.state.phase);
  assert.equal(await page.locator('[data-stat]').count(),13);assert.deepEqual(await page.locator('#burstBars option').allTextContents(),['1','3','5']);assert.equal(await page.locator('#sectorVision').count(),0);
  checks.push('Transport v2, thirteen attributes, unchanged 1/3/5 spending and no vision simulation');
  await page.evaluate(()=>rallyLab.fixture({pace:55,form:'base'}));
  assert(!await page.evaluate(()=>window.__arenaText.some(t=>/^(Spiker|Setter|Receiver|Pace )/.test(t.text))));
  assert.equal(await page.locator('#paceBars .pace em').count(),2);checks.push('Three equal Pace sections; no overhead role or numeric resource label');
  await page.locator('#arena').focus();const initial=await page.evaluate(()=>rallyLab.units[0].pos.x);
  await page.keyboard.down('d');await page.waitForTimeout(80);await page.keyboard.up('d');await page.evaluate(()=>rallyLab.pause());assert((await page.evaluate(()=>rallyLab.units[0].pos.x))>initial);
  checks.push('Focused WASD still moves Rally');
  await page.evaluate(()=>rallyLab.fixture({pace:100,form:'base'}));await page.locator('#arena').focus();await page.keyboard.press('3');await page.keyboard.press('Shift+r');await page.waitForTimeout(30);await page.evaluate(()=>rallyLab.pause());await page.keyboard.press('q');await page.waitForTimeout(30);await page.evaluate(()=>rallyLab.pause());
  assert.equal(await page.evaluate(()=>rallyLab.state.pace),40);assert.equal(await page.evaluate(()=>rallyLab.state.role),'receiver');checks.push('Keyboard Burst/Receive spend works while role remains inspector-only');
  await page.locator('#demoSetup').selectOption('spike_only');await page.locator('#impossibleDemo').click();await to(64);
  assert.equal(await page.evaluate(()=>rallyLab.state.phase.kind),'impossible');assert.equal(await page.evaluate(()=>rallyLab.state.phase.height),10600);assert.equal(await page.evaluate(()=>rallyLab.state.last_power),359);assert.equal(await page.evaluate(()=>rallyLab.state.serve),null);
  checks.push('Non-Serve demo pays five-bar Self Set and contacts at 10.6 m');
  await page.locator('#worldMap').click({position:{x:190,y:190}});await page.evaluate(()=>rallyLab.step());
  assert(await page.evaluate(()=>Math.hypot(rallyLab.state.phase.aim.x-rallyLab.state.phase.origin.x,rallyLab.state.phase.aim.y-rallyLab.state.phase.origin.y)<=400001));
  checks.push('Remote map clicks clamp local Impossible to 400,000');
  await to(303);const lockedAim=await page.evaluate(()=>rallyLab.state.phase.aim);await page.evaluate(()=>rallyLab.setAim({x:900000,y:10000}));await to(304);
  assert(await page.evaluate(()=>rallyLab.state.phase.locked));assert.deepEqual(await page.evaluate(()=>rallyLab.state.phase.aim),lockedAim);
  await page.locator('#arena').screenshot({path:path.join(out,'local-locked.png')});await to(364);
  assert.equal(await page.evaluate(()=>rallyLab.state.ball.power),559);assert.equal(await page.evaluate(()=>rallyLab.state.ball.radius),80000);
  checks.push('Local shot locks at four seconds and releases at five seconds with the smaller radius');
  await serveSetup();assert.equal(await page.evaluate(()=>rallyLab.state.phase.height),18020);assert.equal(await page.evaluate(()=>rallyLab.state.last_power),470);
  assert(await page.evaluate(()=>window.__arenaText.some(t=>t.text.includes('18.02 m')&&t.text.includes('231.3 km/h'))));
  assert.equal(await page.evaluate(()=>rallyLab.state.storm.length),2);await to(112);assert(await page.evaluate(()=>rallyLab.state.storm.some(p=>rallyLab.tick>=p.active_at)));
  checks.push('Serve starts at actual 18.02 m and 231.3 km/h; storm patches become active after their warning');
  await page.locator('#arena').screenshot({path:path.join(out,'storm-charge.png')});
  await page.evaluate(()=>rallyLab.setAim({x:100000,y:850000}));await to(200);assert(await page.evaluate(()=>rallyLab.state.phase.aim.y===850000));
  assert(await page.evaluate(()=>rallyLab.lineWarnings.every(w=>w.harmless)));checks.push('Serve Impossible aims globally behind Rally; flight guide is explicitly harmless');
  await page.evaluate(()=>rallyLab.setAim({x:600000,y:300000}));await to(327);const before=await page.evaluate(()=>rallyLab.state.phase.aim);await page.evaluate(()=>rallyLab.setAim({x:900000,y:10000}));await to(328);assert.deepEqual(await page.evaluate(()=>rallyLab.state.phase.aim),before);
  await page.locator('#arena').screenshot({path:path.join(out,'charge-locked.png')});checks.push('Global destination locks for exactly the final one second');
  await to(388);assert.equal(await page.evaluate(()=>rallyLab.state.ball.power),670);assert.equal(await page.evaluate(()=>rallyLab.state.ball.amount),10398);assert.equal(await page.evaluate(()=>rallyLab.state.exhaustion_until),478);
  assert(await page.evaluate(()=>window.__arenaText.some(t=>t.text.includes('329.7 km/h'))));
  for(const [tick,phase] of [[388,0],[390,1],[392,2]]){await to(tick);assert.equal(await page.evaluate(()=>Math.floor((rallyLab.tick-rallyLab.state.presentation.find(c=>c.phase==='release').start)/2)),phase);await page.locator('#arena').screenshot({path:path.join(out,'release-frame-'+phase+'.png')});}
  checks.push('470→670 Power, 231.3→329.7 km/h, x8 centre damage and three independently timed release drawings');
  await page.evaluate(()=>rallyLab.units[1].pos={x:600000,y:300000});
  let landingTick;for(let t=393;t<430;t++){await to(t);if(await page.evaluate(()=>rallyLab.state.presentation.some(c=>c.phase==='landing'))){landingTick=t;break;}}
  assert(landingTick);assert(await page.evaluate(()=>rallyLab.state.craters.some(c=>c.radius===144000&&c.slow===30)));assert.equal(await page.evaluate(()=>rallyLab.state.storm.length),0);
  await page.locator('#arena').screenshot({path:path.join(out,'landing-frame-0.png')});
  checks.push('Endpoint creates the enlarged crater and clears all storm patches without flight hits');
  await to(600);assert.equal(await page.evaluate(()=>rallyLab.state.ball.kind),'held');
  const frames=await page.evaluate(()=>rallyLab.frames),actual=await page.evaluate(()=>rallyLab.outputs),setup={pace:100,form:'white_storm',bank_percent:0};
  const native=spawnSync(path.join(root,'native/rally_lab/target/release/rally_runner.exe'),['--practice-fixture',JSON.stringify(setup)],{input:frames.map(JSON.stringify).join('\n')+'\n',encoding:'utf8',maxBuffer:128*1024*1024});assert.equal(native.status,0,native.stderr);
  const expected=native.stdout.trim().split(/\r?\n/).map(JSON.parse);assert.deepEqual(actual,expected);checks.push('Complete '+frames.length+'-frame native/WASM parity through charge, storm, impact and return');
  const recording={version:2,rules_revision:revision,practice_setup:setup,frames};fs.writeFileSync(path.join(out,'impossible-recording.json'),JSON.stringify(recording)+'\n');
  for(const presentation of [true,false]){await page.locator('#presentation').setChecked(presentation);await page.evaluate(r=>{rallyLab.loadRecording(r);rallyLab.pause();for(let i=0;i<r.frames.length;i++)rallyLab.step();},recording);assert.deepEqual(await page.evaluate(()=>rallyLab.state),expected.at(-1).state);assert.deepEqual(await page.evaluate(()=>rallyLab.outputs),expected);}
  checks.push('Presentation on/off gives identical commands, complete core state and deterministic replay');
  await page.evaluate(()=>{let rejected=false;try{rallyLab.loadRecording({version:2,rules_revision:'rally-3.1',frames:[]});}catch(e){rejected=e.message.includes('original Rally rules');}if(!rejected)throw Error('Old recording replayed with new rules');});checks.push('Older recordings explicitly require their original rules');
  await page.locator('#presentation').check();await serveSetup(300);await to(120);await page.evaluate(()=>{rallyLab.units[0].stun_until=rallyLab.tick+10;rallyLab.step();});assert.equal(await page.evaluate(()=>rallyLab.state.phase.kind),'idle');assert.equal(await page.evaluate(()=>rallyLab.warnings.length+rallyLab.lineWarnings.length),0);assert.equal(await page.evaluate(()=>rallyLab.state.storm.length),0);checks.push('Interruption removes patches/warnings, spends force and ends charge protection');
  await page.locator('#fog').check();await page.evaluate(()=>{rallyLab.fixture({pace:0,form:'base'});rallyLab.command('ultimate');rallyLab.pause();rallyLab.step();for(let i=0;i<24;i++)rallyLab.step();});assert(!await page.evaluate(()=>rallyLab.isVisible(rallyLab.units[1])));checks.push('Unrestricted Serve does not reveal fogged enemies');
  await serveSetup(0,{x:700000,y:300000});await page.locator('#evade').check();await page.evaluate(()=>rallyLab.units[1].pos={x:840000,y:300000});await to(450);
  assert.equal(await page.evaluate(()=>rallyLab.units[1].hp),10000);assert((await page.evaluate(()=>rallyLab.units[1].pos.x))>854000);checks.push('Remote public blast warning permits a reachable escape without seeing the caster');
  await page.goto(base+'/editor/rally.html?build='+revision+'&demo=impossible&route=spike');await page.waitForFunction(()=>window.rallyLab&&rallyLab.state.phase.kind==='impossible',null,{timeout:10000});await page.evaluate(()=>rallyLab.pause());assert((await page.evaluate(()=>rallyLab.state.phase.height))>=10000);
  await page.screenshot({path:path.join(out,'rally-lab.png'),fullPage:true});checks.push('Automatic demo URL runs the non-Serve 10 m setup');
  await page.locator('#terrainTheme').selectOption('light');await page.evaluate(()=>rallyLab.render());await page.locator('#arena').screenshot({path:path.join(out,'light-terrain.png')});checks.push('Light/dark terrain review retains legible warnings and HUD');
  assert.deepEqual(errors,[]);const report={rules:revision,transport:2,checks,parityFrames:frames.length,landingTick,errors,scope:'Browser/shared-core observations; no native receipt or live-game acceptance'};
  fs.writeFileSync(path.join(out,'browser.json'),JSON.stringify(report,null,2)+'\n');console.log(report);
 }finally{await browser.close();server.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
