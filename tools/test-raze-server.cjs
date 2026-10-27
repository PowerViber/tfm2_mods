const fs=require('fs'),path=require('path'),os=require('os'),net=require('net'),assert=require('assert');
const{spawn}=require('child_process');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..');
(async()=>{
 // An old empty bundle prevents discovery/refresh of the installed game's data.
 const fixture=fs.mkdtempSync(path.join(root,'backups/raze-server-'));
 const bundle=path.join(fixture,'bundle.game_data');fs.writeFileSync(bundle,'');fs.utimesSync(bundle,new Date(0),new Date(0));
 const port=await new Promise(resolve=>{const probe=net.createServer();probe.listen(0,'127.0.0.1',()=>{const port=probe.address().port;probe.close(()=>resolve(port));});});
 const child=spawn(process.execPath,[path.join(root,'editor/server.js'),'--port',String(port),'--no-open','--game',fixture],{cwd:root,windowsHide:true,stdio:['ignore','pipe','pipe']});
 let log='',browser;
 try{
  await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('Editor server startup timeout: '+log)),10000);child.on('error',reject);child.on('exit',code=>reject(Error('Editor exited '+code+': '+log)));child.stderr.on('data',s=>log+=s);child.stdout.on('data',s=>{log+=s;if(log.includes('Editor running at')){clearTimeout(timer);resolve();}});});
  browser=await chromium.launch({channel:'msedge',headless:true});
  const page=await browser.newPage(),errors=[],missing=[];
  page.on('pageerror',e=>errors.push(e.message));page.on('response',r=>{if(r.status()>=400&&!r.url().endsWith('/favicon.ico'))missing.push({url:r.url(),status:r.status()});});
  const base='http://127.0.0.1:'+port;
  await page.goto(base+'/raze.html?build=0.10.72#arena');await page.waitForFunction(()=>window.RazeLab);
  assert.equal(page.url(),base+'/editor/raze.html?build=0.10.72#arena');
  await page.evaluate(()=>RazeLab.step());assert(await page.evaluate(()=>!!RazeLab.state));
  const wasm=await page.request.get(base+'/editor/raze.wasm');assert(wasm.ok());assert.equal(wasm.headers()['content-type'],'application/wasm');assert((await wasm.body()).subarray(0,4).equals(Buffer.from([0,97,115,109])));
  const png=await page.request.get(base+'/mods/tfm2_valorant/vfx/raze%23sheet.png');assert(png.ok());assert.equal(png.headers()['content-type'],'image/png');
  assert.deepStrictEqual(errors,[]);assert.deepStrictEqual(missing,[]);
  // The lightweight repository server is still supported by the same link.
  await page.goto((process.env.LAB_URL||'http://127.0.0.1:7286')+'/raze.html?build=0.10.72#arena');await page.waitForFunction(()=>window.RazeLab);
  assert(page.url().endsWith('/editor/raze.html?build=0.10.72#arena'));
  await page.evaluate(()=>RazeLab.step());assert(await page.evaluate(()=>!!RazeLab.state));
  assert.deepStrictEqual(errors,[]);assert.deepStrictEqual(missing,[]);
  const report={editorServerBoot:true,legacyRedirect:true,queryAndHashPreserved:true,canonicalPageBoot:true,wasmMimeAndBytes:true,checkoutArtwork:true,repositoryServerBoot:true,errors,missing};
  fs.writeFileSync(path.join(root,'docs/raze-results/server.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }finally{
  if(browser)await browser.close();
  if(child.exitCode===null){child.kill();await new Promise(resolve=>child.once('exit',resolve));}
  fs.unlinkSync(bundle);fs.rmdirSync(fixture);
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
