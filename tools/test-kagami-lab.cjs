const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert'),vm=require('vm');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1500,height:1200}}),errors=[],checks=[];
 page.on('pageerror',e=>errors.push(e.stack));const check=(name,ok)=>{assert(ok,name);checks.push(name);};
 try{
  for(const route of ['kagami.html','editor/kagami.html','kagami-review.html','editor/kagami-review.html','kagami-art.js','kagami.wasm','kagami-review-assets/body.png','kagami-review-assets/effects.png','game-reference/world.json'])check('HTTP '+route,(await page.request.get('http://localhost:7307/'+route)).ok());
  await page.goto('http://localhost:7307/kagami.html');await page.waitForFunction(()=>window.kagamiLab);
  check('13 pinned athlete controls',await page.locator('#stats input').count()===13);
  const map=await page.evaluate(()=>({width:kagamiLab.world.geometry.width,scale:kagamiLab.world.geometry.units_per_pixel,walls:kagamiLab.world.walls.filter(Boolean).length,actors:kagamiLab.units.length,kinds:[...new Set(kagamiLab.units.map(u=>u.kind))],box:document.getElementById('arena').getBoundingClientRect().toJSON()}));
  check('extracted 960000 map / 1000 units per pixel / 90 solid cells',map.width===960000&&map.scale===1000&&map.walls===90);
  check('stock champions, minions, camps, objectives and towers',map.kinds.includes('champion')&&map.kinds.includes('minion')&&map.kinds.includes('camp')&&map.kinds.includes('objective')&&map.kinds.includes('tower'));
  check('square map pixels',Math.abs(map.box.width-map.box.height)<1);
  check('art clocks preserve 22 tick attack and six walking drawings',await page.evaluate(()=>kagamiLab.painter.sample('attack',0).duration===22&&kagamiLab.painter.sample('run',0).count===6));
  await page.uncheck('#incoming');
  await page.evaluate(()=>kagamiLab.step());
  check('real core creates its targetable Reflection',await page.evaluate(()=>kagamiLab.clone?.max_hp===Math.floor(kagamiLab.me.max_hp*.35)));
  const mend=await page.evaluate(()=>{const l=kagamiLab;l.me.hp-=300;l.aim={...l.me.pos};const before=l.me.hp;l.order('mend');return{before,after:l.me.hp,cd:l.state.mend_at-l.tick};});
  check('Mend and native cooldown from shared core',mend.after>mend.before&&mend.cd===599);
  await page.click('#reset');await page.uncheck('#incoming');await page.evaluate(()=>kagamiLab.step());
  const swap=await page.evaluate(()=>{const l=kagamiLab,a=l.units.find(u=>u.id===4),c=l.clone;a.pos=l.world.legal({x:l.me.pos.x+50000,y:l.me.pos.y},a.radius);c.pos=l.world.legal({x:l.me.pos.x-25000,y:l.me.pos.y},c.radius);l.aim={...a.pos};l.order('step');const pending=!!l.state.swap;const locked=l.state.swap?.other===a.id;for(let i=0;i<29;i++)l.step();const start={...c.pos};for(let i=0;i<2;i++)l.step();return{pending,locked,d:Math.hypot(a.pos.x-start.x,a.pos.y-start.y),done:!l.state.swap};});
  check('Q ally rescue swaps the live participants',swap.pending&&swap.locked&&swap.done&&swap.d<1);
  await page.click('#reset');await page.uncheck('#incoming');await page.click('#sealCarry');
  check('automatic global save seals carry',await page.evaluate(()=>kagamiLab.state.seals.some(s=>s.id===4)));
  await page.evaluate(()=>{for(let i=0;i<151;i++)kagamiLab.step();});
  check('seal resolves to full HP and a revive shield',await page.evaluate(()=>{const a=kagamiLab.units.find(u=>u.id===4);return a.hp===a.max_hp&&a.shields.some(s=>s.amount>0);}));
  await page.click('#reset');await page.uncheck('#incoming');
  await page.locator('#arena').focus();const before=await page.evaluate(()=>({...kagamiLab.me.pos}));await page.keyboard.down('d');await page.waitForTimeout(220);await page.keyboard.up('d');await page.click('#play');
  check('real WASD input moves Kagami',await page.evaluate(p=>kagamiLab.me.pos.x>p.x,before));
  const stopped=await page.evaluate(()=>({...kagamiLab.me.pos}));await page.locator('#scene').focus();await page.keyboard.down('w');await page.waitForTimeout(80);await page.keyboard.up('w');await page.evaluate(()=>kagamiLab.step());
  check('keyboard gameplay requires arena focus',await page.evaluate(p=>Math.hypot(kagamiLab.me.pos.x-p.x,kagamiLab.me.pos.y-p.y)<1,stopped));
  await page.click('#record');await page.evaluate(()=>{for(let i=0;i<45;i++)kagamiLab.step();});await page.click('#replay');
  await page.waitForFunction(()=>document.getElementById('log').textContent.includes('Core replay agrees'));
  check('recording replays full shared-core state', (await page.locator('#log').innerText()).includes('45/45'));
  for(const scene of ['laning','gank','countergank','camp','morgard','serpen','defend','dive']){
   await page.selectOption('#scene',scene);await page.uncheck('#incoming');await page.evaluate(()=>{for(let i=0;i<5;i++)kagamiLab.step();});check('scenario '+scene,await page.evaluate(()=>kagamiLab.units.length>=55));
  }
  await page.selectOption('#scene','countergank');await page.check('#ai');await page.check('#incoming');
  await page.evaluate(()=>{for(let i=0;i<240;i++)kagamiLab.step();});check('AI review bounded with explanation',await page.evaluate(()=>kagamiLab.state.ai.evaluated<=64&&!!kagamiLab.state.ai.reason));
  await page.screenshot({path:'docs/kagami-art-review/lab.png'});await page.click('#full');await page.locator('#arena').screenshot({path:'docs/kagami-art-review/full-map.png'});
  const clockCtx={window:{}};vm.runInNewContext(fs.readFileSync('editor/kratos-clock.js','utf8'),clockCtx);
  for(const hz of [30,60,120,144,240]){const c=new clockCtx.window.KratosClock();let n=0;for(let i=0;i<=hz*5;i++)c.advance(i*1000/hz,true,()=>n++);check('60 Hz simulation at '+hz+' Hz display',n===300);c.advance(6000,false,()=>n++);c.advance(6010,false,()=>n++);check('pause at '+hz+' Hz',n===300);}
  check('no browser errors',errors.length===0);
  const report={checks:checks.length,passed:checks,errors,actual_game:'pending'};fs.writeFileSync('docs/kagami-art-review/browser.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
