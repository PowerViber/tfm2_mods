// kratos-2: real browser checks of the changed shared mechanics and candidate art.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),{spawn}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/kratos-skill-review'),tmp=path.join(root,'backups/kratos-skill-tests');fs.mkdirSync(out,{recursive:true});fs.mkdirSync(tmp,{recursive:true});
const server=spawn('node',['server.js','--port','7333','--no-open'],{cwd:path.join(root,'editor'),stdio:'ignore'});
(async()=>{await new Promise(r=>setTimeout(r,1500));const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[],checks=[];page.on('pageerror',e=>errors.push(e.message));const check=(n,b)=>{assert(b,n);checks.push(n);console.log('ok - '+n);};
try{
 await page.goto('http://127.0.0.1:7333/kratos.html');await page.waitForFunction(()=>window.kratosLab?.painter.loaded);await page.evaluate(async()=>{kratosLab.pause();await kratosLab.imagesReady;document.querySelector('#moving').checked=false;});
 const r=await page.evaluate(()=>{
  const L=kratosLab,R=KRATOS_RULES,trace=[],run=n=>{for(let t=0;t<n;t++){L.step();trace.push(structuredClone(L.lastTrace));}},reset=()=>{L.reset({fixture:true});L.pause();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;L.me.pos={x:440000,y:520000};};
  reset();const foe=L.units.find(u=>u.id===2);foe.pos={x:440000+77000/Math.SQRT2,y:520000-77000/Math.SQRT2};L.setAim(foe.pos);L.order('rampage');run(1);const start={...L.me.pos};run(17);const before=foe.hp;run(1);const after=foe.hp;run(96);const travel=Math.hypot(L.me.pos.x-start.x,L.me.pos.y-start.y),damage=1000-foe.hp;
  reset();L.me.pos={x:480000,y:480000};const target=L.units.find(u=>u.id===2);target.pos={x:480000,y:540000};L.setAim({x:800000,y:480000});L.order('fenrir');run(1);const spawn=L.state.fenrir.pos,far=Math.hypot(spawn.x-480000,spawn.y-480000);let previous=spawn,moving=true;
  for(let i=0;i<180&&L.state.fenrir&&Math.hypot(L.state.fenrir.pos.x-L.me.pos.x,L.state.fenrir.pos.y-L.me.pos.y)>R.fenrir.width;i++){run(1);const p=L.state.fenrir?.pos;if(p){moving&&=p.x!==previous.x||p.y!==previous.y;previous=p;}}
  L.setAim({x:480000,y:800000});L.order('slam_aim');run(5);const turned=L.state.fenrir?.leg===1&&L.state.fenrir.to.y>L.state.fenrir.from.y;run(160);const secondDamage=1000-target.hp,knocked=target.airborneUntil>0;
  L.reset({scenario:'laning'});L.pause();document.querySelector('#moving').checked=false;L.me.pos={x:480000,y:480000};const knight=L.units.find(u=>u.id===2);knight.pos={x:505000,y:480000};L.setAim(knight.pos);L.order('dash');run(24);L.order('guard');run(1);const taunted=knight.tauntUntil>L.tick;document.querySelector('#moving').checked=true;run(1);const forced=knight.pending?.target===1;document.querySelector('#moving').checked=false;
  const M=L.painter.manifest,anims=L.painter.anims.effects,rect=anims.chain_short_p0.frames[0].data,c=document.createElement('canvas');c.width=rect.w;c.height=rect.h;const ctx=c.getContext('2d');ctx.drawImage(L.painter.images.effects,rect.x,rect.y,rect.w,rect.h,0,0,rect.w,rect.h);const a=ctx.getImageData(0,0,c.width,c.height).data;let bands=0,was=false;for(let y=0;y<c.height;y++){let row=false;for(let x=0;x<c.width;x++)row||=a[(y*c.width+x)*4+3]>0;if(row&&!was)bands++;was=row;}
  return{revision:R.revision,travel,damage,impactAt18:before===1000&&after<1000,far,moving,turned,secondDamage,knocked,taunted,forced,bands,canvas:M.fenrir_canvas,heavy:M.pose_ticks.cleave.reduce((a,b)=>a+b,0),decoded:M.decoded_bytes,approved:M.effects_approved&&L.painter.images.effects.src.endsWith('/kratos-effects.png'),trace};
 });
 check('kratos-2 rules and 36-tick heavy swings are loaded',r.revision==='kratos-2'&&r.heavy===36);
 check('Rampage advances over 100,000 units and reaches a distant foe',r.travel>100000&&r.damage>=350);
 check('The first heavy impact waits until tick 18',r.impactAt18);
 check('Fenrir starts at least 250,000 units away on this map approach',r.far>=250000);
 check('Fenrir keeps moving through the approach instead of parking',r.moving);
 check('The aimed recast redirects Fenrir south, perpendicular to his original charge',r.turned);
 check('That return hits and knocks up an off-axis target once',r.secondDamage===200&&r.knocked);
 check('Guard after a connected Dash taunts and the modeled opponent targets Kratos',r.taunted&&r.forced);
 check('Twin Chain Dash has two separate chain bands',r.bands===2);
 check('Fenrir presentation uses the twice-sized 160px canvas',r.canvas[0]===160&&r.canvas[1]===160);
 check('Approved effects load from the game-pack sheet and stay below 32 MiB decoded art',r.approved&&r.decoded<=32*1024*1024);
 // Put the 8:00 session first, so resets to isolated tick-zero fixtures are
 // explicitly visible to the native trace runner's decreasing-tick reset rule.
 fs.writeFileSync(path.join(tmp,'parity.json'),JSON.stringify([...r.trace.filter(t=>t.input.frame.tick>=28800),...r.trace.filter(t=>t.input.frame.tick<28800)]));delete r.trace;
 for(const [name,order,ticks]of[['sight','sight',8],['twin-chain','dash',1],['heavy-cleave','rampage',19],['fenrir','fenrir',100]]){
  await page.evaluate(async({order,ticks})=>{const L=kratosLab;L.reset({fixture:true});L.pause();document.querySelector('#moving').checked=false;L.me.pos={x:480000,y:480000};L.setAim({x:800000,y:480000});L.order(order);for(let i=0;i<ticks;i++)L.step();await L.imagesReady;L.render();},{order,ticks});await page.locator('#arena').screenshot({path:path.join(out,name+'.png')});
 }
 check('No page errors: '+errors.join('; '),errors.length===0);
 fs.writeFileSync(path.join(out,'browser-results.json'),JSON.stringify({checks:checks.length,passed:checks,result:r,scope:'Shared Rust/WASM mechanics and candidate presentation; stock opponent taunt is modeled host behavior, not live-engine acceptance.'},null,2)+'\n');console.log(checks.length+' checks passed');
}finally{await browser.close();server.kill();}})().catch(e=>{console.error(e);server.kill();process.exitCode=1;});
