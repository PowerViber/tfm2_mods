// Exercise the real editor server with an isolated, inert game directory. No real save/config writes.
const fs=require('fs'),path=require('path'),os=require('os'),net=require('net'),assert=require('assert');
const {spawn}=require('child_process');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..');
(async()=>{
 const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'rally-editor-route-'));
 const game=path.join(fixture,'game');fs.mkdirSync(game);const bundle=path.join(game,'bundle.game_data');fs.writeFileSync(bundle,'inert route fixture');fs.utimesSync(bundle,new Date(2000,0,1),new Date(2000,0,1));
 const reference=path.join(root,'editor/gamedata.js'),before=fs.readFileSync(reference);
 const port=await new Promise(resolve=>{const s=net.createServer();s.listen(0,'127.0.0.1',()=>{const p=s.address().port;s.close(()=>resolve(p));});});
 const child=spawn(process.execPath,[path.join(root,'editor/server.js'),'--port',String(port),'--game',game,'--no-open'],{cwd:root,windowsHide:true,env:{...process.env,APPDATA:path.join(fixture,'appdata'),TFM2_GAME:game},stdio:['ignore','pipe','pipe']});
 let log='';child.stdout.on('data',s=>log+=s);child.stderr.on('data',s=>log+=s);
 let browser;
 try{
  await new Promise((resolve,reject)=>{const timeout=setTimeout(()=>reject(Error(log||'Editor did not start')),15000);const poll=setInterval(()=>{if(log.includes('Database Editor running')){clearInterval(poll);clearTimeout(timeout);resolve();}else if(child.exitCode!==null){clearInterval(poll);clearTimeout(timeout);reject(Error(log));}},50);});
  const base=`http://127.0.0.1:${port}`,checks=[];
  for(const route of ['/rally.html','/editor/rally.html','/editor/rally-lab.js','/editor/rally.wasm','/rules/rally.json','/rally-champion.json','/docs/rally-art-review/white_masked_idle.png','/docs/rally-art-review/effect-body_white_p0.png']){
   const r=await fetch(base+route);assert.equal(r.status,200,route);assert((await r.arrayBuffer()).byteLength>0);checks.push(route);
  }
  browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(base+'/editor/rally.html');await page.waitForFunction(()=>window.rallyLab);assert.equal(await page.locator('[data-stat]').count(),13);assert.deepEqual(errors,[]);
  assert(fs.readFileSync(reference).equals(before),'Route checks changed gamedata.js');
  const report={rules:'rally-2.1',checks,realEditorServer:true,sharedWasmLoaded:true,realCareerAccess:false,errors};
  fs.writeFileSync(path.join(root,'docs/rally-results/editor-route.json'),JSON.stringify(report,null,2)+'\n');console.log(report);
 }finally{
  if(browser)await browser.close();child.kill();
  if(!fs.readFileSync(reference).equals(before))fs.writeFileSync(reference,before);
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
