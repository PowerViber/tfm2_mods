// Actual browser controls and complete native/WASM trace comparison.
const assert=require('assert'),fs=require('fs'),path=require('path'),os=require('os'),net=require('net');
const {spawn,spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/grimhald-art-review');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});let server;
 const page=await browser.newPage({viewport:{width:1500,height:1050}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 let checks=0,scenarioCount=0,parityFrames=0;const passed=[];
 const check=(name,condition)=>{assert(condition,name);checks++;passed.push(name)};
 try {
  await page.goto('http://127.0.0.1:7296/editor/grimhald.html');await page.waitForFunction(()=>window.grimhaldLab);
  check('Lab loads approved body and shared WASM',await page.locator('#status').textContent().then(s=>s.includes('Paused')));
  check('13 athlete controls',await page.locator('.stat input').count()===13);
  await page.locator('#arena').focus();await page.keyboard.down('d');
  const walk=await page.evaluate(()=>{grimhaldLab.pause();const frames=[];for(let i=0;i<72;i++){grimhaldLab.step();frames.push(grimhaldLab.animation.index)}return frames});
  await page.keyboard.up('d');check('Live movement renders all six walk drawings',new Set(walk).size===6);
  const gait=await page.evaluate(()=>{grimhaldLab.reset();return grimhaldLab.animation});
  await page.locator('#arena').focus();await page.keyboard.down('d');
  const early=await page.evaluate(()=>{grimhaldLab.pause();const before=grimhaldLab.animation.distance;for(let i=0;i<10;i++)grimhaldLab.step();return {...grimhaldLab.animation,travelled:grimhaldLab.animation.distance-before}});
  await page.keyboard.up('d');
  check('Walking cadence follows travel rather than the old fast timer',early.travelled===10300&&early.index===Math.floor(early.distance/10000)%6);
  const unchanged=await page.evaluate(()=>{const before=JSON.stringify(grimhaldLab.animation);for(let i=0;i<50;i++)grimhaldLab.render();return before===JSON.stringify(grimhaldLab.animation)});
  check('Repeated rendering cannot advance the gait',unchanged);
  const walkRecording=await page.evaluate(()=>({version:1,rules_revision:window.GRIMHALD_RULES.revision,frames:grimhaldLab.frames}));
  const replayGait=await page.evaluate(r=>{grimhaldLab.loadRecording(r);grimhaldLab.pause();for(let i=0;i<r.frames.length;i++)grimhaldLab.step();return grimhaldLab.animation},walkRecording);
  check('Recording reconstructs the same movement phase',replayGait.index===early.index&&replayGait.distance===early.distance&&replayGait.pose===early.pose);
  // Round 3: a unit beside the pillar can always walk away from it (it used to be frozen until the pillar fell)
  const besidePillar=await page.evaluate(()=>{const L=grimhaldLab;L.reset();L.pause();document.querySelector('#auto').checked=false;for(let i=0;i<3;i++)L.step();
   const me=L.units[0];L.setAim({x:me.pos.x+70000,y:me.pos.y});L.command('bulwark');let n=0;while(!L.state.pillar&&n++<200)L.step();
   const c=L.state.pillar.center;L.units[0].pos={x:c.x-14000,y:c.y};return {...L.units[0].pos}});
  await page.locator('#arena').focus();await page.keyboard.down('a');
  const leftPillar=await page.evaluate(()=>{grimhaldLab.pause();for(let i=0;i<20;i++)grimhaldLab.step();return {...grimhaldLab.units[0].pos}});
  await page.keyboard.up('a');check('Walking away from a pillar that rose beside him is never blocked',leftPillar.x<besidePillar.x-10000);
  await page.evaluate(()=>grimhaldLab.reset());
  await page.selectOption('#pose','idle_unarmed');await page.uncheck('#animate');
  const idle=[];for(let i=0;i<4;i++){await page.locator('#artFrame').fill(String(i));await page.evaluate(()=>grimhaldLab.render());idle.push(await page.locator('#inspect').evaluate(c=>c.toDataURL()));}
  check('Matched unarmed idle has four complete drawings',new Set(idle).size===4);
  await page.selectOption('#pose','run_unarmed');await page.uncheck('#animate');
  const unarmed=[];for(let i=0;i<6;i++){await page.locator('#artFrame').fill(String(i));await page.evaluate(()=>grimhaldLab.render());unarmed.push(await page.locator('#inspect').evaluate(c=>c.toDataURL()));}
  check('Unarmed walking inspector has six distinct drawings',new Set(unarmed).size===6);await page.check('#animate');
  await page.uncheck('#animate');
  for(const [name,count] of Object.entries({idle:4,run:6,attack:4,skill1:4,skill2:4,ult:4,hit:1,dead:6})){
   const drawings=[];
   for(const suffix of ['', '_unarmed'])for(const facing of ['right','left']){
    await page.selectOption('#pose',name+suffix);await page.selectOption('#facing',facing);
    for(let i=0;i<count;i++){await page.locator('#artFrame').fill(String(i));await page.evaluate(()=>grimhaldLab.render());drawings.push(await page.locator('#inspect').evaluate(c=>c.toDataURL()));}
   }
   check('Complete '+name+' frames in both weapon states/facings',new Set(drawings.slice(0,count)).size===count&&drawings.every(Boolean));
  }
  await page.selectOption('#pose','idle');await page.selectOption('#facing','right');await page.check('#animate');
  await page.locator('[data-fixture="miss"]').click();
  const away=await page.evaluate(()=>{grimhaldLab.pause();for(let i=0;i<40;i++)grimhaldLab.step();return {kind:grimhaldLab.state.ult.kind,weapon:grimhaldLab.animation.weapon}});
  check('Real flight state removes only the axe layer',away.kind==='flight'&&!away.weapon);
  const returned=await page.evaluate(()=>{for(let i=0;i<170;i++)grimhaldLab.step();return grimhaldLab.animation.weapon});
  check('Axe layer returns after a missed throw completes',returned);
  await page.locator('[data-fixture="cage"]').click();
  const planted=await page.evaluate(()=>{grimhaldLab.pause();for(let i=0;i<180;i++)grimhaldLab.step();return {planted:!!grimhaldLab.state.planted,weapon:grimhaldLab.animation.weapon,tags:grimhaldLab.outputs.flatMap(o=>o.effect_tags||[])}});
  check('Planted axe keeps the body unarmed and uses its ground phase',planted.planted&&!planted.weapon&&planted.tags.includes('grh_axe_planted'));
  const corpse=await page.evaluate(()=>{grimhaldLab.units[0].hp=0;for(let i=0;i<90;i++)grimhaldLab.step();const a={...grimhaldLab.animation};for(let i=0;i<30;i++)grimhaldLab.step();return {a,b:grimhaldLab.animation}});
  check('Death reaches its final frame without restarting or a weapon',corpse.a.index===5&&corpse.b.index===5&&!corpse.b.weapon);
  await page.evaluate(()=>grimhaldLab.reset());
  await page.locator('#arena').focus();await page.keyboard.down('d');await page.waitForTimeout(120);await page.keyboard.up('d');await page.evaluate(()=>grimhaldLab.pause());
  check('Actual WASD moves the owner',await page.evaluate(()=>grimhaldLab.units[0].pos.x>160000));
  await page.keyboard.press('q');await page.waitForTimeout(80);await page.evaluate(()=>grimhaldLab.pause());check('Q activates ghost',await page.evaluate(()=>!!grimhaldLab.state.ghost));
  await page.keyboard.press('q');await page.waitForTimeout(60);await page.evaluate(()=>grimhaldLab.pause());check('Q recast ends ghost',await page.evaluate(()=>!grimhaldLab.state.ghost));
  await page.evaluate(()=>grimhaldLab.reset());await page.locator('#arena').focus();await page.keyboard.press('e');await page.waitForTimeout(1250);await page.evaluate(()=>grimhaldLab.pause());check('E creates persistent pillar',await page.evaluate(()=>!!grimhaldLab.state.pillar&&grimhaldLab.presentation.some(c=>c.tag==='pillar')));
  await page.evaluate(()=>{grimhaldLab.setAim({x:210000,y:325000});grimhaldLab.command('bulwark');grimhaldLab.pause();grimhaldLab.step()});check('E recast relocates without replacing cooldown',await page.evaluate(()=>grimhaldLab.state.pillar.relocated));
  const before=await page.evaluate(()=>grimhaldLab.units[0].pos.x);await page.locator('#arena').focus();await page.keyboard.down('d');await page.waitForTimeout(60);await page.locator('#pose').focus();await page.keyboard.up('d');await page.evaluate(()=>grimhaldLab.pause());const atBlur=await page.evaluate(()=>grimhaldLab.units[0].pos.x);await page.evaluate(()=>{for(let i=0;i<10;i++)grimhaldLab.step()});check('Focus loss clears held movement',await page.evaluate(x=>grimhaldLab.units[0].pos.x===x,atBlur));
  const scenarios=[];
  for(let i=0;i<36;i++){
   const fixture=['combo','cage','miss','cleanse'][i%4],split=i<24?'development':'holdout';
   await page.locator('[data-fixture="'+fixture+'"]').click();
   await page.evaluate(i=>{grimhaldLab.pause();document.querySelectorAll('.stat input').forEach((e,j)=>e.value=String((i*17+j*13)%101));document.querySelector('#walls').checked=i%5===0;document.querySelector('#moving').checked=i%3===0;document.querySelector('#fog').checked=i%7===0;for(let t=0;t<130;t++)grimhaldLab.step()},i);
   const trace=await page.evaluate(()=>({frames:grimhaldLab.frames,outputs:grimhaldLab.outputs}));
   const native=spawnSync(path.join(root,'native/grimhald_lab/target/release/grimhald_lab.exe'),[],{input:trace.frames.map(f=>JSON.stringify(f)).join('\n')+'\n',encoding:'utf8',maxBuffer:100*1024*1024,windowsHide:true});
   assert.equal(native.status,0,native.stderr);const lines=native.stdout.trim().split(/\r?\n/).map(JSON.parse);assert.deepStrictEqual(lines,trace.outputs,'native/WASM complete trace '+i);parityFrames+=lines.length;scenarioCount++;
   scenarios.push({id:i+1,split,fixture,frames:lines.length,parity:true});
  }
  check('24 development + 12 holdout native/WASM scenarios',scenarioCount===36);
  const visualTraces=[];
  for(const enabled of [true,false]){await page.locator('[data-fixture="combo"]').click();await page.evaluate(enabled=>{grimhaldLab.pause();document.querySelector('#presentation').checked=enabled;document.querySelector('#fog').checked=false;document.querySelector('#moving').checked=false;for(let i=0;i<120;i++)grimhaldLab.step()},enabled);visualTraces.push(await page.evaluate(()=>grimhaldLab.outputs));}
  check('Presentation toggle preserves complete combat trace',JSON.stringify(visualTraces[0])===JSON.stringify(visualTraces[1]));await page.check('#presentation');
  const alias=await page.evaluate(()=>grimhaldLab.outputs.flatMap(o=>o.effect_tags||[]).filter(Boolean));check('Shared core presentation selects directional ground fissures',alias.some(n=>n==='grh_fissure_d0'));
  // Exported frames replay through the same core without a second JS skill implementation.
  const recording=await page.evaluate(()=>({version:1,rules_revision:window.GRIMHALD_RULES.revision,frames:grimhaldLab.frames}));const expected=await page.evaluate(()=>grimhaldLab.outputs);
  await page.evaluate(r=>{grimhaldLab.loadRecording(r);grimhaldLab.pause();for(let i=0;i<r.frames.length;i++)grimhaldLab.step()},recording);
  check('Recording reproduces full outputs',JSON.stringify(expected)===JSON.stringify(await page.evaluate(()=>grimhaldLab.outputs)));
  await page.evaluate(()=>{try{grimhaldLab.loadRecording({version:1,rules_revision:'old',frames:[]})}catch(e){document.querySelector('#recordingInfo').dataset.error=e.message}});check('Incompatible recordings explain original-rules requirement',await page.locator('#recordingInfo').getAttribute('data-error').then(s=>s.includes('original')));
  // Manual profile changes must not change any mechanical output.
  const frames=recording.frames.map(f=>({...f,frame:{...f.frame,manual:true}}));
  function nativeWithProfile(value){const result=spawnSync(path.join(root,'native/grimhald_lab/target/release/grimhald_lab.exe'),[],{input:frames.map(f=>JSON.stringify({...f,frame:{...f.frame,profile:Array(13).fill(value)}})).join('\n')+'\n',encoding:'utf8',maxBuffer:100*1024*1024,windowsHide:true});assert.equal(result.status,0);return result.stdout}
  check('Manual mechanics independent of athlete profile',nativeWithProfile(20)===nativeWithProfile(80));
  await page.locator('[data-fixture="cage"]').click();await page.evaluate(()=>{grimhaldLab.pause();document.querySelector('#fog').checked=false;for(let i=0;i<110;i++)grimhaldLab.step()});await page.locator('#arena').screenshot({path:path.join(out,'lab-cage.png')});check('Complete pentagon follows live core cage',await page.evaluate(()=>grimhaldLab.presentation.some(c=>c.tag.startsWith('cage_phase'))));
  for(const facing of ['right','left'])for(const theme of ['light','dark']){await page.selectOption('#facing',facing);await page.selectOption('#theme',theme);await page.selectOption('#pose','skill2');await page.screenshot({path:path.join(out,'lab-'+facing+'-'+theme+'.png')});}
  // The production editor server must serve the same page without touching game data.
  const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'grimhald-editor-'));fs.writeFileSync(path.join(fixture,'bundle.game_data'),'');fs.utimesSync(path.join(fixture,'bundle.game_data'),new Date(0),new Date(0));
  const port=await new Promise(r=>{const s=net.createServer();s.listen(0,'127.0.0.1',()=>{const p=s.address().port;s.close(()=>r(p))})});
  server=spawn(process.execPath,[path.join(root,'editor/server.js'),'--port',String(port),'--no-open','--game',fixture],{cwd:root,windowsHide:true,stdio:['ignore','pipe','pipe']});let log='';
  await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error(log)),10000);server.on('error',reject);server.stdout.on('data',d=>{log+=d;if(log.includes('Editor running at')){clearTimeout(timer);resolve()}});server.stderr.on('data',d=>log+=d)});
  await page.goto('http://127.0.0.1:'+port+'/editor/grimhald.html');await page.waitForFunction(()=>window.grimhaldLab);check('Actual editor server route and assets',await page.locator('#status').textContent().then(s=>s.includes('Paused')));
  check('No browser exceptions',errors.length===0);
  const report={rules_revision:require(path.join(root,'rules/grimhald.json')).revision,art_revision:'body-v6 complete animation and separate weapon',checks,passed,scenarios,parityFrames,errors,liveAcceptance:'pending; Lab host movement, damage mitigation and cage isolation are simulated'};fs.writeFileSync(path.join(out,'lab-results.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({checks,scenarioCount,parityFrames,errors}));
 }finally{await browser.close();if(server)server.kill()}
})().catch(e=>{console.error(e);process.exitCode=1});
