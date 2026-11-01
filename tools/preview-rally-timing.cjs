// Capture the actual shared-core Lab. These are observations, not a second animation simulation.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawnSync}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),output=path.join(root,'docs/rally-results',require(path.join(root,'rules/rally.json')).revision),temp=fs.mkdtempSync(path.join(os.tmpdir(),'rally-preview-'));
(async()=>{fs.mkdirSync(output,{recursive:true});const browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage({viewport:{width:1800,height:1200}}),errors=[],reports=[];page.on('pageerror',e=>errors.push(e.message));
try {for(const mode of ['ordinary','black','white']) {
 const black=mode==='black',white=mode==='white',name=white?'white-storm':black?'black-thunder':'ordinary-thunder',folder=path.join(temp,name);fs.mkdirSync(folder);
 await page.goto('http://127.0.0.1:7294/editor/rally.html?revision='+require(path.join(root,'rules/rally.json')).revision);await page.waitForFunction(()=>window.rallyLab);
 let base=await page.evaluate(({black,white})=>{rallyLab.reset();if(black||white)rallyLab.fixture({flow:100,charges:3,...(white?{transformed:true}:{})});document.querySelector('#evade').checked=false;rallyLab.pause();rallyLab.units[1].pos={x:300000,y:300000};const c=document.querySelector('canvas');c.style.maxWidth='none';c.style.width=c.width+'px';return rallyLab.tick},{black,white});
 if(!white){await page.locator('#arena').focus();await page.keyboard.down('d');base=await page.evaluate(()=>{rallyLab.pause();return rallyLab.tick});}
 const self=base+(black?26:1),approach=self+13,takeoff=approach+(white?48:24),spike=takeoff+24;
 const schedule=new Map([[self,'self_set'],[approach,white?'ultimate':'approach'],[takeoff,'takeoff'],[spike,'spike']]);if(black)schedule.set(base+1,'black_thunder');
 let index=0;const trace=[];
 for(let t=base+1;t<=spike+45;t++) {
  const state=await page.evaluate(action=>{if(action){rallyLab.enqueue(action);rallyLab.pause();}rallyLab.step();return{tick:rallyLab.tick,phase:rallyLab.state.phase,ball:rallyLab.state.ball.kind,height:rallyLab.state.contact_height,transformed:rallyLab.state.transformed,whiteStorm:rallyLab.state.white_storm,helper:!!rallyLab.state.helper,enemyHP:rallyLab.units[1].hp}},schedule.get(t)||null);
  trace.push(state);if((t-base)%2===0){const frame=path.join(folder,String(index++).padStart(4,'0')+'.png');await page.locator('#arena').screenshot({path:frame});if(t===spike||t===spike+1)fs.copyFileSync(frame,path.join(output,name+'-contact.png'));}
 }
 const contact=trace.find(f=>f.tick===spike);assert.equal(contact.ball,'flying');assert(contact.phase.hold_ticks>0);
 if(!white)assert(trace.some(f=>f.transformed)===black,'Only Black contact transforms');else assert(trace.some(f=>f.whiteStorm),'White contact grants its tier');
 if(!white)await page.keyboard.up('d');
 reports.push({name,spike,contactHeight:contact.height,holdTicks:contact.phase.hold_ticks,frames:index,trace});
 const script="from pathlib import Path;from PIL import Image;src=Path("+JSON.stringify(folder)+");frames=[Image.open(p).convert('RGB') for p in sorted(src.glob('*.png'))];frames[0].save("+JSON.stringify(path.join(output,name+'-lab.gif'))+",save_all=True,append_images=frames[1:],duration=34,loop=0,disposal=2)";
 const pack=spawnSync('rtk',['proxy','python','-X','utf8','-c',script],{encoding:'utf8'});assert.equal(pack.status,0,pack.stderr);
 }
 assert.deepEqual(errors,[]);fs.copyFileSync(path.join(output,'ordinary-thunder-lab.gif'),path.join(output,'skill-animation.gif'));
 fs.writeFileSync(path.join(output,'animation-timing.json'),JSON.stringify({scope:'Actual Lab/WASM capture; native game rendering pending',reports,errors},null,2)+'\n');
 console.log(reports.map(({name,contactHeight,holdTicks,frames})=>({name,contactHeight,holdTicks,frames})));
} finally {await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});
