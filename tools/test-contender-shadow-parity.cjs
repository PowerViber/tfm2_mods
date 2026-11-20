const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert'),cp=require('child_process'),path=require('path');
const rules=JSON.parse(fs.readFileSync('rules/contender.json','utf8')),revision=rules.revision,out=process.env.CONTENDER_RESULTS||'docs/contender-1vx-review';fs.mkdirSync(out,{recursive:true});
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage();try{
 const base=process.env.CONTENDER_LAB_URL||'http://127.0.0.1:7309';
 await page.goto(base+'/editor/contender.html');await page.waitForFunction(()=>window.contenderLab?.me&&!document.getElementById('addTargets').disabled);await page.uncheck('#fight');
 const scenes=['laning','gank','countergank','camp','morgard','serpen','defend','dive','shadowLink'],results=[];
 for(let i=0;i<36;i++){
  const scene=scenes[i%8],profile=[20,50,80][Math.floor(i/8)%3];await page.selectOption('#scenario',scene);
  const records=await page.evaluate(async({profile,i})=>{const L=contenderLab;document.querySelectorAll('[data-stat]').forEach(x=>x.value=profile);await L.reset();
    const p=L.me.pos,foe=L.units.find(u=>u.team===1&&u.champion&&u.visible);const to=foe?.pos||{x:p.x+40000,y:p.y};
    const flurryEnd=CONTENDER_RULES.flurry.ticks;
    const queue=new Map([[0,{kind:'s2'}],[flurryEnd+1,{kind:'s1'}],[flurryEnd+2,{kind:'ult',aim:to}]]);
    for(let t=0;t<=flurryEnd+CONTENDER_RULES.merge.crossEveryTicks+2;t++)L.step(queue.get(t));
    return L.records;
  },{profile,i});
  const input=records.map(r=>JSON.stringify({version:1,rules_revision:revision,frame:r.frame})).join('\n')+'\n';
  const native=cp.spawnSync(path.resolve('native/contender_shadow_lab/target/release/trace.exe'),[],{input,encoding:'utf8',maxBuffer:32*1024*1024});assert.strictEqual(native.status,0,native.stderr);
  const outputs=native.stdout.trim().split(/\r?\n/).map(s=>JSON.parse(s));assert.equal(outputs.length,records.length);
  for(let j=0;j<records.length;j++){assert.deepStrictEqual(outputs[j].state,records[j].state,`state ${i}:${j}`);assert.deepStrictEqual(outputs[j].effects,records[j].effects,`effects ${i}:${j}`);}
  const flurryPunches=records.reduce((n,row)=>n+row.effects.filter(e=>e.kind==='damage'&&e.source==='flurry').length,0);
  results.push({split:i<24?'development':'holdout',scene,profile,frames:records.length,flurryPunches,complete_state_and_effects_equal:true});
 }
 const totalFrames=results.reduce((s,x)=>s+x.frames,0),flurryPunches=results.reduce((s,x)=>s+x.flurryPunches,0);
 assert.ok(flurryPunches>=rules.flurry.punches,'parity traces must exercise active Flurry');
 fs.writeFileSync(out+'/parity.json',JSON.stringify({rules:revision,development:24,holdout:12,totalFrames,flurryPunches,results,scope:'Native/WASM deterministic core parity in Lab host fixtures'},null,2));
 console.log(`${results.length} scenarios / ${totalFrames} frames / ${flurryPunches} Flurry punches: complete native/WASM state and effects equal`);
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
