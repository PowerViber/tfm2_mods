const fs=require('fs'),path=require('path'),os=require('os'),net=require('net'),assert=require('assert');
const{spawn}=require('child_process');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..');
const out=path.resolve(root,process.env.MINATO_RESULTS_DIR||'docs/original-body-review');fs.mkdirSync(out,{recursive:true});
(async()=>{
 const fixture=fs.readdirSync(path.join(root,'backups')).filter(x=>x.startsWith('original-body-manager-')||x.startsWith('minato-portrait-manager-')).map(x=>path.join(root,'backups',x)).sort((a,b)=>fs.statSync(b).mtimeMs-fs.statSync(a).mtimeMs)[0];assert(fixture);
 fs.utimesSync(path.join(fixture,'bundle.game_data'),new Date(0),new Date(0));
 const port=await new Promise(resolve=>{const p=net.createServer();p.listen(0,'127.0.0.1',()=>{const n=p.address().port;p.close(()=>resolve(n));});});
 const child=spawn(process.execPath,[path.join(root,'editor/server.js'),'--port',String(port),'--no-open','--game',fixture],{cwd:root,windowsHide:true,stdio:['ignore','pipe','pipe']});let browser,log='';
 try{
  await new Promise((resolve,reject)=>{const timeout=setTimeout(()=>reject(Error(log)),10000);child.on('error',reject);child.stderr.on('data',s=>log+=s);child.stdout.on('data',s=>{log+=s;if(log.includes('Editor running at')){clearTimeout(timeout);resolve();}});});
  browser=await chromium.launch({channel:'msedge',headless:true});const page=await browser.newPage({viewport:{width:1600,height:1100}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://127.0.0.1:${port}/`);await page.locator('#btnSkillTest').click();await page.locator('#stChamp').selectOption('tfm2_custom_minato');
  await page.waitForFunction(()=>{const t=TFM2SkillTest._T;return t.json?.id==='tfm2_custom_minato'&&t.sprite?.img&&t.sheets['asset/tfm2_custom/champions/minato_kcm']?.img;});
  const normal=await page.evaluate(()=>{const t=TFM2SkillTest._T;t.running=false;const h=t.ents.find(e=>e.kind==='hero');return {hp:h.hp,atk:h.atk,ap:h.ap,x:h.x,y:h.y,frameCount:t.sprite.anims.idle.length};});
  const arena=page.locator('#stCanvas');const before=await arena.screenshot({path:path.join(out,'skill-test-normal.png')});
  await page.evaluate(()=>{const t=TFM2SkillTest._T;t.ents.find(e=>e.kind==='hero').buffs.minato_kcm=t.tick+900;});await page.waitForTimeout(100);
  const after=await arena.screenshot({path:path.join(out,'skill-test-kcm.png')});assert(!before.equals(after),'KCM must visibly change the body');
  const kcm=await page.evaluate(()=>{const t=TFM2SkillTest._T,h=t.ents.find(e=>e.kind==='hero');return {hp:h.hp,atk:h.atk,ap:h.ap,x:h.x,y:h.y,frameCount:t.sheets['asset/tfm2_custom/champions/minato_kcm'].anims.idle.length};});assert.deepStrictEqual(normal,kcm);
  const art=await(await page.request.get(`http://127.0.0.1:${port}/api/champ-art`)).json();assert(art.champions['asset/tfm2_custom/champions/minato_presenter_base'].url.includes('minato_presenter_base'));
  assert.deepStrictEqual(errors,[]);const report={normalAndKcmAssetsLoaded:true,formChangeVisible:true,inspectionDoesNotMutateCombatFields:true,editorThumbnailUsesActualBaseAsset:true,errors,scope:'Simplified Skill Test presentation inspection; not native host acceptance.'};fs.writeFileSync(path.join(out,'skill-test.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }finally{if(browser)await browser.close();if(child.exitCode===null){child.kill();await new Promise(resolve=>child.once('exit',resolve));}}
})().catch(e=>{console.error(e);process.exitCode=1;});
