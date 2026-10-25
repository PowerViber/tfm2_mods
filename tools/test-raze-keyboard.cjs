const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1400,height:1000}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.LAB_URL||'http://localhost:7286')+'/editor/raze.html');await page.waitForFunction(()=>window.RazeLab);
 const arena=page.locator('#arena');
 async function reset(){await page.locator('#automatic').uncheck();await page.locator('#reset').click();await arena.focus();}
 async function pause(){await page.evaluate(()=>RazeLab.pause());}
 await reset();const start=await page.evaluate(()=>({...RazeLab.units[0].pos}));
 await page.keyboard.down('w');await page.waitForFunction(y=>RazeLab.units[0].pos.y<y-3000,start.y);await page.keyboard.up('w');await pause();
 const north=await page.evaluate(()=>({...RazeLab.units[0].pos}));assert(north.y<start.y&&north.x===start.x);
 await reset();await page.keyboard.down('w');await page.keyboard.down('d');await page.waitForFunction(()=>RazeLab.history.length>=6);await page.keyboard.up('w');await page.keyboard.up('d');await pause();
 const diagonal=await page.evaluate(()=>RazeLab.history.map(f=>f.input.units[0].pos));let diagonalSeen=false;
 for(let n=1;n<diagonal.length;n++){const dx=diagonal[n].x-diagonal[n-1].x,dy=diagonal[n].y-diagonal[n-1].y;assert(Math.hypot(dx,dy)<=1021);if(dx>0&&dy<0)diagonalSeen=true;}assert(diagonalSeen);
 await reset();await page.keyboard.down('w');await page.waitForFunction(()=>RazeLab.units[0].pos.y<297000);await page.keyboard.up('w');await pause();
 // A rapid second Q is queued during preparation rather than attempting a second throw.
 await page.keyboard.press('q');await page.keyboard.press('q');await page.waitForFunction(()=>RazeLab.state?.air_protection);
 await page.keyboard.down('d');await page.waitForFunction(()=>RazeLab.state?.flight?.velocity.x>5000);await page.keyboard.up('d');await pause();
 const launch=await page.evaluate(()=>({state:RazeLab.state,replayFrames:RazeLab.history.length}));
 assert.equal(launch.state.stock[0],1);assert(launch.state.air_protection);assert.equal(launch.state.metrics.launches,1);
 await page.keyboard.press('e');await page.waitForFunction(()=>RazeLab.state?.cast?.action.kind==='grenade');await pause();
 assert(await page.locator('#stats').innerText().then(s=>s.includes('15%')));
 await page.screenshot({path:path.join(__dirname,'../docs/raze-results/raze-air-protection.png'),fullPage:true});
 await page.locator('#root').click();await page.evaluate(()=>RazeLab.step(2));assert.equal(await page.evaluate(()=>RazeLab.state.air_protection),false);
 await reset();await page.keyboard.press('x');await page.waitForFunction(()=>RazeLab.state?.armed_until>RazeLab.history.length);await pause();
 const box=await arena.boundingBox();await page.mouse.move(box.x+box.width*.42,box.y+box.height*.55);await page.mouse.click(box.x+box.width*.42,box.y+box.height*.55);
 await page.waitForFunction(()=>RazeLab.state?.metrics.rocket_shots===1);await pause();
 const rocket=await page.evaluate(()=>({grounded:RazeLab.state.flight?.grounded,guard:RazeLab.state.air_protection,remaining:RazeLab.state.cooldowns[2]-RazeLab.history.length }));
 // round 143: the ultimate's cooldown starts when the rocket fires
 assert(rocket.grounded&&!rocket.guard&&rocket.remaining>=JSON.parse(fs.readFileSync(path.join(__dirname,'../rules/raze.json'))).constants.ROCKET_COOLDOWN-5,JSON.stringify(rocket));
 await reset();await page.keyboard.down('d');await page.waitForFunction(()=>RazeLab.units[0].pos.x>304000);await page.locator('#seed').focus();await page.keyboard.up('d');
 const blur=await page.evaluate(async()=>{const before={...RazeLab.units[0].pos};RazeLab.pause();await RazeLab.step(8);return{before,after:RazeLab.units[0].pos};});assert.deepStrictEqual(blur.before,blur.after);
 const file=path.join(os.tmpdir(),'raze-legacy-recording-test.json');fs.writeFileSync(file,JSON.stringify({version:1,frames:[{input:{version:1},signature:'old'}]}));
 await page.locator('#import').setInputFiles(file);await page.waitForFunction(()=>document.querySelector('#status').textContent.includes('incompatible'));
 await page.getByText('Imported recording inspection',{exact:true}).click();assert((await page.locator('#recording-inspect').innerText()).includes('old'));fs.unlinkSync(file);
 const replay=await page.evaluate(()=>RazeLab.verify());assert(replay.identical);assert.deepStrictEqual(errors,[]);
 const report={version:3,wasdGroundMovement:true,diagonalNormalized:true,qQueuedDetonation:true,wasdAirSteering:true,grenadeKeyboard:true,rocketEquipAndMouseFire:true,groundRecoilNoProtection:true,blurClearsMovement:true,legacyInspection:true,replay,errors};
 fs.writeFileSync(path.join(__dirname,'../docs/raze-results/keyboard.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
