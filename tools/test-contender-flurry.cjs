const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert'),out=process.env.CONTENDER_RESULTS||'docs/contender-flurry-review';fs.mkdirSync(out,{recursive:true});
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1560,height:1040}}),errors=[];page.on('pageerror',e=>errors.push(e.message));try{
 await page.goto('http://127.0.0.1:7309/editor/contender.html');await page.waitForFunction(()=>window.contenderLab?.me&&document.querySelectorAll('.targetRow').length===3);
 await page.uncheck('#fight');
 const run=async(type,count)=>page.evaluate(async({type,count})=>{const L=contenderLab;L.clearTargets();await L.reset();for(let i=0;i<count;i++)await L.addTargets(i===0?'stock:knight':type,1,8,true);
  L.units.forEach((u,i)=>{u.pos=L.world.legal({x:L.me.pos.x+16000+i*14000,y:L.me.pos.y-12000},u.radius);u.home={...u.pos};});
  L.step({kind:'ult'});for(let i=0;i<305;i++)L.step();
  const damage=L.records.flatMap(r=>r.effects).filter(e=>e.kind==='damage').map(e=>e.target),cues=L.records.flatMap(r=>r.effects).filter(e=>e.kind==='fx'&&e.name==='ct_flurry');
  return {damage,cues,targets:L.units.map(u=>({id:u.id,type:u.labType,total:L.ledger.row(u.id,L.tick).total})),revision:CONTENDER_RULES.revision};
 },{type,count});
 const multiple=await run('stock:berserker',2);assert.equal(multiple.revision,'contender-2.2');assert.equal(multiple.damage.length,36);assert(multiple.damage.every((id,i)=>!i||id!==multiple.damage[i-1]));assert(multiple.targets.every(u=>u.total>0));assert.equal(new Set(multiple.cues.map(c=>c.heading)).size,8);
 const creep=await run('melee',2);assert.equal(creep.damage.length,36);assert(creep.damage.every((id,i)=>!i||id!==creep.damage[i-1]));assert(creep.targets.every(u=>u.total>0));
 const solo=await run('stock:knight',1);assert.equal(solo.damage.length,36);assert.equal(new Set(solo.damage).size,1);
 const assets=await page.evaluate(()=>{const L=contenderLab,canvas=document.createElement('canvas');canvas.width=1024;canvas.height=440;const c=canvas.getContext('2d');c.fillStyle='#1b2f3e';c.fillRect(0,0,1024,440);const hashes=[];
  for(let h=0;h<8;h++){const tag=`flurry_d${h}_t8`,a=L.painter.data.effects.anims[tag];if(!a||L.painter.duration('effects',tag)!==8)throw Error('Unbound heading '+h);const p={x:128+(h%4)*256,y:100+Math.floor(h/4)*220};L.painter.draw(c,'effects',tag,p,4,p=>p,n=>n*.002);c.fillStyle='#eadbff';c.fillText(['E','SE','S','SW','W','NW','N','NE'][h],p.x-8,p.y+88);}
  return {preview:canvas.toDataURL('image/png'),decoded:L.painter.data.decoded_bytes};});fs.writeFileSync(out+'/eight-directions.png',Buffer.from(assets.preview.split(',')[1],'base64'));assert(assets.decoded<32*1024*1024);
 // Fresh page gives a readable multi-target screenshot, with live directional effects.
 await page.evaluate(async()=>{const L=contenderLab;L.clearTargets();await L.reset();await L.addTargets('stock:knight',2,8,true);L.units.forEach((u,i)=>u.pos=L.world.legal({x:L.me.pos.x+16000+i*14000,y:L.me.pos.y-12000},u.radius));L.step({kind:'ult'});for(let i=0;i<125;i++)L.step();L.draw();});await page.screenshot({path:out+'/flurry-lab.png'});
 assert.deepEqual(errors,[]);const report={revision:multiple.revision,multiple,creep,solo,decoded_bytes:assets.decoded,errors,scope:'Shared-core Lab tests: different-target preference, solo fallback and eight directional punches. Not native visual acceptance.'};fs.writeFileSync(out+'/flurry-browser.json',JSON.stringify(report,null,2));console.log(JSON.stringify({revision:report.revision,punches:multiple.damage.length,directions:8,alternates:true,creepSwitch:true,soloFallback:true,decoded:assets.decoded,errors}));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
