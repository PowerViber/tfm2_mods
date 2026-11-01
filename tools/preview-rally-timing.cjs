if(require('../rules/rally.json').revision==='rally-3.3'){require('./preview-rally-33.cjs');}else if(require('../rules/rally.json').revision==='rally-3.2'){require('./preview-rally-32.cjs');}else{
// Capture actual shared-core Impossible phases; no second gameplay simulation.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),rev=require(path.join(root,'rules/rally.json')).revision;
const output=path.join(root,'docs/rally-results',rev),temp=fs.mkdtempSync(path.join(os.tmpdir(),'rally3-preview-'));
(async()=>{fs.mkdirSync(output,{recursive:true});const browser=await chromium.launch({channel:'msedge',headless:true});
const page=await browser.newPage({viewport:{width:1550,height:1250}}),errors=[],trace=[];page.on('pageerror',e=>errors.push(e.message));
try {
 await page.goto('http://127.0.0.1:7294/editor/rally.html?revision='+rev);await page.waitForFunction(()=>window.rallyLab);
 await page.evaluate(()=>{rallyLab.fixture({pace:100,form:'white_storm',bank_percent:0});rallyLab.setAim({x:900000,y:300000});document.querySelector('#evade').checked=false;document.querySelector('#zoom').value=.5;document.querySelector('#zoom').dispatchEvent(new Event('change'));});
 const schedule={2:'arm_burst',3:'self_set',16:'ultimate',64:'takeoff',88:'spike'};
 let index=0;
 for(let t=2;t<=310;t++){
  const state=await page.evaluate(({t,action})=>{if(action){rallyLab.command(action,{bars:5});rallyLab.pause();}rallyLab.step();
   return {tick:rallyLab.tick,phase:rallyLab.state.phase.kind,height:rallyLab.state.contact_height,locked:!!rallyLab.state.phase.locked,
    ball:rallyLab.state.ball.kind,speed:rallyLab.state.last_speed,power:rallyLab.state.last_power};},{t,action:schedule[t]||null});
  if(t%6===0||[88,148,238].includes(t)){trace.push(state);const frame=path.join(temp,String(index++).padStart(4,'0')+'.png');await page.locator('#arena').screenshot({path:frame});
   if([88,130,148,238].includes(t))fs.copyFileSync(frame,path.join(output,'impossible-phase-'+t+'.png'));}
 }
 assert(trace.some(f=>f.phase==='impossible'&&f.height===18020));assert(trace.some(f=>f.locked));assert(trace.some(f=>f.ball==='flying'));
 const durations=trace.map((f,i)=>i+1<trace.length?Math.max(10,Math.round((trace[i+1].tick-f.tick)*1000/60)):100);
 const script="from pathlib import Path;from PIL import Image;src=Path("+JSON.stringify(temp)+");frames=[Image.open(p).convert('RGB') for p in sorted(src.glob('*.png'))];frames[0].save("+JSON.stringify(path.join(output,'impossible-spike-lab.gif'))+",save_all=True,append_images=frames[1:],duration="+JSON.stringify(durations)+",loop=0,disposal=2)";
 const pack=spawnSync('rtk',['proxy','python','-X','utf8','-c',script],{encoding:'utf8'});assert.equal(pack.status,0,pack.stderr);
 assert.deepEqual(errors,[]);fs.writeFileSync(path.join(output,'animation-timing.json'),JSON.stringify({rules:rev,scope:'Actual Lab/WASM capture; not native renderer or artwork approval',sampleEveryTicks:6,frameDurationsMs:durations,frames:index,trace,errors},null,2)+'\n');
 console.log({rules:rev,frames:index,errors});
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});

}
