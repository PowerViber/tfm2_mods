const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),cp=require('child_process');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.join(__dirname,'..'),out=path.join(root,'docs/steve-results');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
const page=await browser.newPage({viewport:{width:1500,height:1100}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://localhost:7282/steve.html');await page.waitForFunction(()=>window.SteveLab);
await page.evaluate(()=>SteveLab.pause());const checks=[],recipes=[];
for(let r=0;r<15;r++){
 const x=await page.evaluate(r=>{SteveLab.demo(r);SteveLab.pause();for(let n=0;n<140;n++)SteveLab.step();return {r,s:SteveLab.snapshot,events:SteveLab.recording.flatMap(r=>r.output.commands),frames:SteveLab.recording.length,replay:SteveLab.verifyReplay({version:1,frames:SteveLab.recording})};},r);
 assert(!x.s.error,JSON.stringify(x));assert(x.events.length>0);assert.equal(x.frames,x.replay);const m=x.s.core.match(/casts: \[([^\]]+)\]/);assert(m);assert(+m[1].split(',')[r]>0,JSON.stringify(x));recipes.push({recipe:r,events:x.events.length,replayFrames:x.frames});
}
checks.push('All fifteen recipes accept, resolve, and replay with complete core state');
const frames=await page.evaluate(()=>{
 document.querySelector('#automatic').checked=true;document.querySelector('#moving').checked=true;const all=[];
 for(let n=0;n<36;n++){
  const kind=['combat','rescue','terrain','fog','heavy','curve','interruption'][n%7];SteveLab.reset(kind);
  for(const u of SteveLab.units){if(u.id>0&&u.team===1){u.x=Math.min(900000,u.x+(n*137)%20000);u.y=Math.min(900000,u.y+(n*317)%20000);}}
  for(let t=0;t<360;t++)SteveLab.step();
  all.push({seed:n+1,set:n<24?'development':'holdout',kind,frames:JSON.parse(JSON.stringify(SteveLab.recording)),state:SteveLab.snapshot.core});
 }return all;
});
// 16 reusable native sessions; resetting each scenario is a fresh process batch.
let parity=0;for(const scenario of frames){const input=scenario.frames.map(r=>'0 '+r.input.join(' ')).join('\n')+'\n';const p=cp.spawnSync('rtk',['proxy',path.join(root,'native/steve_lab/target/release/steve-vectors.exe')],{input,encoding:'utf8',maxBuffer:256*1024*1024});assert.equal(p.status,0,p.stderr);const lines=p.stdout.trim().split(/\r?\n/);assert.equal(lines.length,scenario.frames.length);for(let n=0;n<lines.length;n++){assert.deepStrictEqual(JSON.parse(lines[n]),scenario.frames[n].output,`seed ${scenario.seed} frame ${n}`);parity++;}}
checks.push(`${parity} complete native/WASM parity frames across 24 development and 12 holdout scenarios`);
await page.evaluate(()=>{document.querySelector('#automatic').checked=false;document.querySelector('#moving').checked=false;SteveLab.reset('curve');for(let i=0;i<26;i++)SteveLab.step();SteveLab.enqueue([7,0,0,480000,480000,0,4]);for(let i=0;i<160;i++)SteveLab.step();});
await page.screenshot({path:path.join(out,'lab-curved-wall.png'),fullPage:true});const wall=await page.evaluate(()=>SteveLab.snapshot);assert(wall.walls.some(w=>w.path.length===17),wall.reason);checks.push('Herobrine builds the full 16-segment open U wall');
await page.goto('http://localhost:7282/');await page.waitForSelector('#btnSteveLab');await page.click('#btnSteveLab');await page.waitForSelector('#stSteveCraft iframe');checks.push('Skill Test opens the shared SteveCraft Lab');
assert.deepStrictEqual(errors,[]);const report={checks,recipes,parityFrames:parity,scenarios:frames.map(({frames,state,...s})=>({...s,lastState:state})),errors,liveAcceptance:'Pending; browser and ABI fixtures are not a real-match observation.'};fs.writeFileSync(path.join(out,'lab-tests.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({checks,recipes:recipes.length,parityFrames:parity,errors}));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
