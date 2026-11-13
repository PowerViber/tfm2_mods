const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert'),cp=require('child_process');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1500,height:1100}}),errors=[],checks=[];
 page.on('pageerror',e=>errors.push(e.message));
 const check=(name,passed)=>{assert(passed,name);checks.push(name);};
 try{
  for(const route of ['kicker-review.html','editor/kicker-review.html','kicker.html','editor/kicker.html','kicker-body.png','kicker-body.fanim','kicker-review-assets/body-comparison.png','kicker-review-assets/effects.png']){
   check('HTTP '+route,(await page.request.get('http://localhost:7306/'+route)).ok());
  }
  await page.goto('http://localhost:7306/kicker-review.html?build=kicker-art-3');
  await page.waitForFunction(()=>window.kickerReview?.painter.loaded);
  const review=await page.evaluate(()=>{
   const P=kickerReview.painter,A=KICKER_ART;
   return{poses:Object.keys(A.poses).length,runFrames:P.sample('run').count,runTicks:P.sample('run').duration,
    allFacings:Object.keys(A.poses).every(p=>A.body_anims[p+'_l']),frameBounds:Object.values(A.body_anims).every(a=>a.frames.every(f=>f.data.x+f.data.w<=P.image.width&&f.data.y+f.data.h<=P.image.height)),
    native:true,effectsIntegrated:A.effects_approved,bodyIntegrated:A.body_approved};
  });
  check('14 full-body poses / both facings',review.poses===14&&review.allFacings);
  check('8 jogging drawings, 48 ticks',review.runFrames===8&&review.runTicks===48);
  check('all body rectangles in atlas',review.frameBounds);
  check('body approved; effects still gated',review.bodyIntegrated&&!review.effectsIntegrated);
  await page.selectOption('#pose','run');await page.locator('#arena').screenshot({path:'docs/kicker-art-review/animated-map.png'});
  await page.selectOption('#fxSequence','ult');await page.locator('#fxArena').screenshot({path:'docs/kicker-art-review/effects-loop.png'});
  await page.goto('http://localhost:7306/kicker.html?build=kicker-art-3');await page.waitForFunction(()=>window.kickerLab?.painter.loaded);
  const scenarios=await page.evaluate(()=>{
   const L=kickerLab,out=[];L.pause();
   for(const name of Object.keys(GameScenario.choices)){
    document.getElementById('scenario').value=name;L.reset();L.capture(true);
    L.order('rock');for(let i=0;i<120;i++)L.step();
    out.push({name,tick:L.tick,frames:L.trace.length,error:L.trace.some(t=>t.output.error),actors:L.units.length,missing:L.units.filter(u=>!u.sprite).length});
   }
   return out;
  });
  check('eight actual-map situations execute',scenarios.length===8&&scenarios.every(s=>s.frames===121&&!s.error&&s.missing===0));
  const parity=await page.evaluate(()=>{
   const L=kickerLab;document.getElementById('hostModel').checked=false;
   const run=art=>{
    document.getElementById('artEnabled').checked=art;L.reset();L.units.splice(0);
    L.me.pos={x:480000,y:480000};L.capture(true);
    const T=L.scene.make(17,1,{x:500000,y:480000},'champion','champ-knight',L.world.data.champions.knight);
    L.units.push(T);L.setAim(T.pos);L.order('rock');
    for(let i=0;i<60;i++)L.step();L.order('kick');
    for(let i=0;i<90;i++)L.step();L.order('ult');
    for(let i=0;i<60;i++)L.step();
    return{trace:L.trace,me:structuredClone(L.me),units:structuredClone(L.units)};
   };
   const enabled=run(true),disabled=run(false);
   return{trace:enabled.trace,equal:JSON.stringify(enabled)===JSON.stringify(disabled)};
  });
  check('art on/off leaves complete inputs, outputs and model state identical',parity.equal);
  fs.mkdirSync('backups/kicker-art-tests',{recursive:true});fs.writeFileSync('backups/kicker-art-tests/parity.json',JSON.stringify(parity.trace));
  const native=cp.execFileSync('native/kicker_lab/target/release/examples/trace.exe',[],{input:parity.trace.map(t=>JSON.stringify(t.input)).join('\n')+'\n',encoding:'utf8',maxBuffer:30*1024*1024}).trim().split(/\r?\n/).map(JSON.parse);
  check('native/WASM complete output parity',native.length===parity.trace.length&&native.every((x,i)=>JSON.stringify(x)===JSON.stringify(parity.trace[i].output)));
  const mount=await page.evaluate(()=>{
   const L=kickerLab;L.reset();L.units.splice(0);document.getElementById('hostModel').checked=false;
   let p=null;
   for(let i=31;i<869&&!p;i++)if(L.walls[i]&&!L.walls[i+1]&&i%30<28){
    const q={x:(i%30+1.5)*32000,y:(Math.floor(i/30)+.5)*32000};
    if(!L.world.wallAt(q,L.me.radius))p=q;
   }
   L.me.pos=p;L.order('mount');for(let i=0;i<35;i++)L.step();
   const roof=L.state.roof;
   L.setAim({x:p.x+30000,y:p.y});L.order('rock');const fan=L.state.rocks.length;
   return{roof,fan,p,error:document.getElementById('log').textContent.includes('Invalid Kicker')};
  });
  check('mount at a real nearby wall',mount.roof);
  check('roof throws three actual core rocks',mount.fan===3&&!mount.error);
  await page.evaluate(()=>{const L=kickerLab;L.reset();L.units.splice(0);document.getElementById('hostModel').checked=false;L.me.pos=L.world.legal({x:480000,y:480000},L.me.radius);document.getElementById('artEnabled').checked=true;});
  const before=await page.evaluate(()=>({...kickerLab.me.pos}));await page.locator('#map').focus();await page.keyboard.down('d');await page.waitForTimeout(180);await page.keyboard.up('d');await page.evaluate(()=>kickerLab.pause());
  const after=await page.evaluate(()=>({...kickerLab.me.pos}));
  check('actual WASD input moves on legal map',Math.hypot(after.x-before.x,after.y-before.y)>1000);
  await page.keyboard.down('a');await page.locator('#scenario').focus();const lost=await page.evaluate(()=>({...kickerLab.me.pos}));await page.waitForTimeout(100);await page.keyboard.up('a');await page.evaluate(()=>kickerLab.pause());const end=await page.evaluate(()=>({...kickerLab.me.pos}));
  check('focus loss clears held input',end.x===lost.x&&end.y===lost.y);
  await page.locator('#map').screenshot({path:'docs/kicker-art-review/lab-complete.png'});
  check('no browser exceptions',errors.length===0);
  const report={checks:checks.length,passed:checks,review,scenarios,native_wasm_frames:native.length,art_on_off_equal:parity.equal,mount,errors,live_game:'pending by user request',native_special_poses:'disabled; no Animation CC enabled',effects:'review-only; approval pending'};
  fs.writeFileSync('docs/kicker-art-review/browser-complete.json',JSON.stringify(report,null,2));console.log(JSON.stringify(report,null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});

