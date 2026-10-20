/* Current combat, demonstrations and archived replay readers; host acceptance is separate. */
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),cp=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const out=path.join(__dirname,'../docs/gojo-results');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});
 try {
  const page=await browser.newPage({viewport:{width:1360,height:1040}}),errors=[],checks=[],demos=[];
  page.on('pageerror',e=>errors.push(e.message));await page.goto('http://localhost:7283/gojo.html');await page.waitForFunction(()=>window.GojoLab);
  assert.equal(await page.isChecked('#automatic'),true);assert.equal(await page.inputValue('#fixture'),'combat');
  const combat=await page.evaluate(()=>{GojoLab.pause();GojoLab.reset();for(let i=0;i<1800;i++)GojoLab.step();return {s:GojoLab.snapshot,casts:GojoLab.snapshot.casts,flashes:GojoLab.snapshot.flashes,remainingHp:GojoLab.units[0].hp,groups:GojoLab.snapshot.casts.filter(n=>n>0).length};});
  assert.equal(combat.s.version,4);assert(combat.groups>=4,JSON.stringify(combat));checks.push('Default combat moves and uses at least four technique groups without a showcase');
  let demonstrationParityFrames=0;
  for(const kind of ['nuke','maxred','feint','purple','rct','step','full','quick','blue','red']) {
   const result=await page.evaluate(kind=>{
    GojoLab.playDemo(kind);GojoLab.pause();let orb=null,arrivedAt=null,fusion=false;const hits=[];let selfCost=null;
    for(let i=0;i<650;i++){
     GojoLab.step();const s=GojoLab.snapshot;
     if(!orb&&s.orb){orb=s.orb;arrivedAt=s.tick;}fusion||=!!s.fusion;
     for(const c of s.commands){if(c[0]===0)hits.push({tick:s.tick,target:c[1],damage:c[2]+c[3],basic:c[4]});if(c[0]===12){const row=GojoLab.recording.at(-1);selfCost={before:row.input[20],after:row.scene[0].hp};}}
    }
    return {kind,casts:GojoLab.snapshot.casts,flashes:GojoLab.snapshot.flashes,orb,arrivedAt,fusion,selfCost,hits,hp:GojoLab.units[0].hp,events:GojoLab.recording.filter(r=>r.output.commands.length).length};
   },kind);
   if(kind==='nuke'){assert.equal(result.flashes,1);assert(result.orb);assert.equal(result.orb[2]-result.arrivedAt,180);assert(result.fusion);assert(result.selfCost);assert.equal(result.selfCost.after,Math.ceil(result.selfCost.before/2));}
   if(kind==='maxred'){assert.equal(result.flashes,1);assert.equal(result.casts[3],1);}
   if(kind==='feint'){const hits=result.hits.filter(h=>h.target===2);assert.equal(hits.length,2);assert.equal(hits[1].tick-hits[0].tick,48);assert.equal(hits[0].damage,hits[1].damage);}
   if(kind==='rct')assert(result.hp>400);
   if(kind==='step')assert.equal(result.casts[6],1);
   if(kind==='purple')assert(result.hits.length>0);
   if(kind==='blue'||kind==='red')assert(result.hits.length>0);
   if(kind==='full'||kind==='quick')assert.equal(result.casts[7],1);
   const frames=await page.evaluate(()=>GojoLab.recording.map(r=>({input:r.input,output:r.output})));
   const native=cp.execFileSync(path.join(__dirname,'../native/gojo_lab/target/release/gojo-vectors.exe'),{input:frames.map(r=>'0 '+r.input.join(' ')).join('\n')+'\n',encoding:'utf8',maxBuffer:64*1024*1024}).trim().split(/\r?\n/).map(JSON.parse);
   assert.equal(native.length,frames.length);native.forEach((s,i)=>assert.deepStrictEqual(s,frames[i].output,kind+' parity frame '+i));demonstrationParityFrames+=frames.length;
   demos.push(result);checks.push('Legal demonstration with full native/WASM parity: '+kind);
  }
  const before=await page.evaluate(()=>GojoLab.snapshot.core);await page.click('#replay');assert.equal(await page.evaluate(()=>GojoLab.snapshot.core),before);assert((await page.textContent('#events')).includes('Replay verified'));checks.push('Complete v4 replay');
  await page.evaluate(()=>{GojoLab.playDemo('nuke');GojoLab.pause();for(let i=0;i<380;i++)GojoLab.step();});
  await page.locator('#arena').screenshot({path:path.join(out,'v4-curved-blue.png')});
  await page.evaluate(()=>{for(let i=0;i<185;i++)GojoLab.step();});await page.locator('#arena').screenshot({path:path.join(out,'v4-nuke-warning.png')});
  for(const v of [3,2,1]){await page.goto('http://localhost:7283/gojo-v'+v+'.html');await page.waitForFunction(()=>window.GojoLab);assert.equal(await page.evaluate(()=>GojoLab.snapshot.version),v);}
  checks.push('Frozen v1, v2 and v3 readers load their original rules');assert.deepEqual(errors,[]);
  fs.writeFileSync(path.join(out,'v4-browser.json'),JSON.stringify({checks,demonstrationParityFrames,combat:{casts:combat.casts,flashes:combat.flashes,remainingHp:combat.remainingHp},demos,errors,liveGame:false},null,2)+'\n');
  console.log(JSON.stringify({passed:checks.length,combat:combat.casts,demos:demos.map(r=>({kind:r.kind,casts:r.casts,flashes:r.flashes}))}));
 }finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
