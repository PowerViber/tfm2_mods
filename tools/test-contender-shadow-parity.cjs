const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert'),cp=require('child_process'),path=require('path');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage();try{
 await page.goto('http://127.0.0.1:7309/editor/contender.html');await page.waitForFunction(()=>window.contenderLab?.me);await page.uncheck('#fight');
 const scenes=['laning','gank','countergank','camp','morgard','serpen','defend','dive'],results=[];
 for(let i=0;i<36;i++){
  const scene=scenes[i%8],profile=[20,50,80][Math.floor(i/8)%3];await page.selectOption('#scenario',scene);
  const records=await page.evaluate(async({profile,i})=>{const L=contenderLab;document.querySelectorAll('[data-stat]').forEach(x=>x.value=profile);await L.reset();
    const p=L.me.pos,foe=L.units.find(u=>u.team===1&&u.champion&&u.visible);const to=foe?.pos||{x:p.x+40000,y:p.y};
    for(let t=0;t<150;t++)L.step(t===0?{kind:i%3===0?'s1':i%3===1?'s2':'s3',aim:to}:t===30?{kind:'s2',aim:to}:t===50?{kind:'s3',aim:to}:t===70?{kind:'s1',aim:to}:t===90?{kind:'ult'}:undefined);
    return L.records;
  },{profile,i});
  const input=records.map(r=>JSON.stringify({version:1,rules_revision:'contender-2.0',frame:r.frame})).join('\n')+'\n';
  const native=cp.spawnSync(path.resolve('native/contender_shadow_lab/target/release/trace.exe'),[],{input,encoding:'utf8',maxBuffer:32*1024*1024});assert.strictEqual(native.status,0,native.stderr);
  const outputs=native.stdout.trim().split(/\r?\n/).map(s=>JSON.parse(s));assert.equal(outputs.length,records.length);
  for(let j=0;j<records.length;j++){assert.deepStrictEqual(outputs[j].state,records[j].state,`state ${i}:${j}`);assert.deepStrictEqual(outputs[j].effects,records[j].effects,`effects ${i}:${j}`);}
  results.push({split:i<24?'development':'holdout',scene,profile,frames:records.length,complete_state_and_effects_equal:true});
 }
 fs.writeFileSync('docs/contender-shadow-review/parity.json',JSON.stringify({rules:'contender-2.0',development:24,holdout:12,totalFrames:results.reduce((s,x)=>s+x.frames,0),results,scope:'Native/WASM deterministic core parity in Lab host fixtures; not actual-game acceptance'},null,2));console.log('36 scenarios / 5,400 frames: complete native/WASM state and effects equal');
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
