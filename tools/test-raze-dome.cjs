const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const out=path.join(__dirname,'../docs/raze-results');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1440,height:1080}}),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.LAB_URL||'http://localhost:7288')+'/editor/raze.html');await page.waitForFunction(()=>window.RazeLab);
 assert.equal(await page.evaluate(()=>RAZE_RULES.rules_revision),'raze-3.5');
 async function reset(){await page.locator('#automatic').uncheck();await page.locator('#reset').click();await page.evaluate(()=>RazeLab.pause());}
 async function centre(){await page.evaluate(()=>RazeLab.setAim({...RazeLab.units[0].pos}));await page.locator('#pack').click();await page.evaluate(()=>RazeLab.step(8));await page.locator('#detonate').click();await page.waitForFunction(()=>RazeLab.state?.flight);}
 await reset();await centre();
 const start=await page.evaluate(()=>({pos:{...RazeLab.units[0].pos},flight:RazeLab.state.flight,profile:RazeLab.state.last_launch}));
 assert.equal(start.flight.velocity.x,0);assert.equal(start.flight.velocity.y,0);assert.equal(start.profile.ticks,39);
 await page.evaluate(()=>RazeLab.step(10));assert.deepStrictEqual(await page.evaluate(()=>RazeLab.units[0].pos),start.pos);
 await page.evaluate(()=>RazeLab.step(40));
 const one=await page.evaluate(()=>({flight:RazeLab.state.flight,active:RazeLab.state.air_protection,remaining:RazeLab.state.air_protection_until-RazeLab.history.length,enemies:RazeLab.state.air_protection_enemies}));
 assert.equal(one.flight,null);assert(one.active&&one.enemies===1&&one.remaining>150&&one.remaining<=180);
 const expiry=await page.evaluate(()=>RazeLab.state.air_protection_until);
 await page.evaluate(()=>{for(let i=4;i<8;i++)RazeLab.units.push({...structuredClone(RazeLab.units[1]),id:i,pos:{x:340000+(i-4)*8000,y:280000+(i-4)*12000}});});await page.evaluate(()=>RazeLab.step());
 assert.equal(await page.evaluate(()=>RazeLab.state.air_protection_until),expiry);
 await page.evaluate(()=>RazeLab.step(180));assert.equal(await page.evaluate(()=>RazeLab.state.air_protection),false);
 assert((await page.evaluate(()=>RazeLab.verify())).identical);
 await reset();await page.evaluate(()=>{for(let i=4;i<8;i++)RazeLab.units.push({...structuredClone(RazeLab.units[1]),id:i,pos:{x:340000+(i-4)*8000,y:280000+(i-4)*12000}});});await centre();
 await page.evaluate(()=>RazeLab.step(39));
 const five=await page.evaluate(()=>({active:RazeLab.state.air_protection,remaining:RazeLab.state.air_protection_until-RazeLab.history.length,enemies:RazeLab.state.air_protection_enemies}));
 assert(five.active&&five.enemies===5&&five.remaining>=719&&five.remaining<=720);
 await page.screenshot({path:path.join(out,'raze-landing-defence.png'),fullPage:true});assert((await page.evaluate(()=>RazeLab.verify())).identical);
 await reset();await page.locator('#feet').click();await page.evaluate(()=>RazeLab.step(8));
 const underfoot=await page.evaluate(()=>({owner:{...RazeLab.units[0].pos},pack:RazeLab.state.packs[0].attachment.point}));
 assert.deepStrictEqual(underfoot.pack,underfoot.owner);
 await page.screenshot({path:path.join(out,'raze-dome-guide.png'),fullPage:true});await page.locator('#detonate').click();
 assert.equal(await page.evaluate(()=>RazeLab.state.last_launch.horizontal_speed),0);
 await reset();await page.evaluate(()=>RazeLab.setAim({x:286000,y:300000}));await page.locator('#pack').click();await page.evaluate(()=>RazeLab.step(8));await page.locator('#detonate').click();
 const sweet=await page.evaluate(()=>({profile:RazeLab.state.last_launch,flight:RazeLab.state.flight}));
 assert.equal(sweet.profile.distance,14000);assert.equal(sweet.flight.velocity.x,200000);assert.equal(sweet.profile.ticks,48);
 assert.deepStrictEqual(errors,[]);
 const result={rulesRevision:'raze-3.5',radius:56000,underfoot,centre:start,sweet,one,five,lateEntrantsDoNotRefresh:true,expiry:true,replay:true,errors};
 fs.writeFileSync(path.join(out,'dome-browser.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify({centre:true,sweet:true,landingScaling:true,lateEntrants:true,expiry:true,replay:true,errors}));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});
