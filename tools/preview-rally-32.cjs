const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),output=path.join(root,'docs/rally-results/rally-3.2'),temp=fs.mkdtempSync(path.join(os.tmpdir(),'rally32-preview-'));
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1550,height:1250}}),trace=[],errors=[];page.on('pageerror',e=>errors.push(e.message));
 try{
  await page.goto('http://127.0.0.1:7294/editor/rally.html?build=rally-3.2');await page.waitForFunction(()=>window.rallyLab);
  await page.evaluate(()=>{rallyLab.fixture({pace:100,form:'white_storm',bank_percent:0});rallyLab.setAim({x:600000,y:300000});document.querySelector('#evade').checked=false;document.querySelector('#zoom').value=.75;document.querySelector('#zoom').dispatchEvent(new Event('change'));});
  const schedule={2:'arm_burst',3:'self_set',16:'ultimate',64:'takeoff',88:'spike'};let index=0;
  for(let t=2;t<=510;t++){
   const state=await page.evaluate(action=>{if(action){rallyLab.command(action,{bars:5});rallyLab.pause();}rallyLab.step();const s=rallyLab.state;return{tick:rallyLab.tick,phase:s.phase.kind,height:s.contact_height,locked:!!s.phase.locked,ball:s.ball.kind,power:s.last_power,speed:s.last_speed,patches:s.storm.length,cues:s.presentation};},schedule[t]||null);
   const cueFrame=state.cues.some(c=>(t-c.start)%2===0);
   if(t%12===0||[88,328,388].includes(t)||cueFrame){trace.push(state);const file=path.join(temp,String(index++).padStart(4,'0')+'.png');await page.locator('#arena').screenshot({path:file});
    if([88,240,328,388].includes(t))fs.copyFileSync(file,path.join(output,'impossible-phase-'+t+'.png'));
   }
  }
  assert(trace.some(s=>s.cues.some(c=>c.phase==='landing')));assert.deepEqual(errors,[]);
  const durations=trace.map((s,i)=>i+1<trace.length?Math.max(10,Math.round((trace[i+1].tick-s.tick)*1000/60)):200);
  const code="from pathlib import Path;from PIL import Image;src=Path("+JSON.stringify(temp)+");frames=[Image.open(p).convert('RGB') for p in sorted(src.glob('*.png'))];frames[0].save("+JSON.stringify(path.join(output,'impossible-spike-lab.gif'))+",save_all=True,append_images=frames[1:],duration="+JSON.stringify(durations)+",loop=0,disposal=2)";
  const p=spawnSync('rtk',['proxy','python','-X','utf8','-c',code],{encoding:'utf8'});assert.equal(p.status,0,p.stderr);
  fs.writeFileSync(path.join(output,'animation-timing.json'),JSON.stringify({rules:'rally-3.2',scope:'Actual WASM Lab capture; not native renderer or art approval',trace,frameDurationsMs:durations,frames:index,errors},null,2)+'\n');console.log({frames:index,errors,output:path.join(output,'impossible-spike-lab.gif')});
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
