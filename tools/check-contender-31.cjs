/* Matched stock-size target groups and complete Windows Rust/WASM output comparison. */
const assert = require('node:assert/strict');
const {spawnSync} = require('node:child_process');
const {readFileSync} = require('node:fs');
const {performance} = require('node:perf_hooks');
const path = require('node:path');
const baseline = 'ddbaa450aebca0a767a099ecc7176677144e7d37';

async function wasm(bytes, revision) {
  const {instance:{exports:api}} = await WebAssembly.instantiate(bytes);
  return {revision, reset:api.contender_reset, step(frame) {
    const input = Buffer.from(JSON.stringify({version:1,rules_revision:revision,frame}));
    const ptr = api.contender_input(input.length);
    new Uint8Array(api.memory.buffer,ptr,input.length).set(input);
    const out = api.contender_step(input.length);
    const value = JSON.parse(Buffer.from(api.memory.buffer,out,api.contender_output_len()).toString());
    assert.ok(!value.error,value.error);
    return value;
  }};
}
const unit=(id,x,y,team=1)=>({id,team,pos:{x,y},hp:2000,max_hp:2000,radius:3000,
  champion:true,tower:false,visible:true,targetable:true});
function frame(tick,units,aggressive) {
  const profile=Array(13).fill(50); profile[6]=aggressive;
  return {tick,owner:1,team:0,pos:{x:400000,y:500000},home:{x:375000,y:500000},
    alive:true,hp:2000,max_hp:2000,attack:161,units,profile,manual:true,order:null,level:8,ult_unlocked:true};
}
function sample(engine, count, profile) {
  engine.reset();
  const units = Array.from({length:count},(_,i)=>unit(10+i,430000+1500*i,500000+750*i));
  const old=engine.revision==='contender-3.0';
  let credited=0;
  const frames=[], states=[];
  const start=performance.now();
  let peak=0;
  for(let tick=1;tick<=330;tick++){
    const f=frame(tick,units,profile);
    if(tick===1)f.order={kind:old?'ult':'s2'};
    if(tick===32 && !old)f.incoming=[{attacker:10,from:units[0].pos}];
    if(tick===72 && !old)f.incoming=[{attacker:10,from:units[0].pos}];
    if(tick===82)f.basic_hits=[10];
    if(tick===85 && !old)f.order={kind:'exchange_dodge'};
    if(tick===120 && !old)f.order={kind:'s1'};
    f.landed_hits=credited;
    const begin=performance.now();
    const result=engine.step(f);
    peak=Math.max(peak,performance.now()-begin);
    credited=result.effects.filter(e=>e.kind==='damage'&&['counter','echo','merge'].includes(e.source)&&e.amount>0).length;
    frames.push(f); states.push(result);
  }
  const effects=states.flatMap(s=>s.effects);
  const punches=effects.filter(e=>e.kind==='fx'&&e.name==='ct_flurry').length;
  const raw=effects.filter(e=>e.kind==='damage').reduce((sum,e)=>sum+e.amount,0);
  return {frames,states,metrics:{count,profile,punches,raw,
    visuals:effects.filter(e=>e.kind==='fx').length,
    peakEffects:Math.max(...states.map(s=>s.effects.length)),
    wasmTransportMs:Math.round((performance.now()-start)*100)/100,
    worstWasmTransportMs:Math.round(peak*100)/100}};
}
function linkTrace(engine) {
  engine.reset();
  const units=[unit(10,440000,500000),unit(11,459000,500000),unit(12,430000,505000)];
  units[2]={...units[2],team:9,champion:false,farmable:true};
  const air=new Map(),frames=[],states=[];
  for(let tick=1;tick<=120;tick++){
    const f=frame(tick,structuredClone(units),50);
    f.link_airborne=[...air.entries()].filter(([,until])=>tick<until).map(([id])=>id);
    if(tick===1)f.order={kind:'ult',aim:{x:435000,y:500000}};
    if(tick===30)units[1].pos.x=535000;
    const result=engine.step(f);
    for(const effect of result.effects){
      if(effect.kind==='knock')units.find(u=>u.id===effect.target).pos=effect.to;
      if(effect.kind==='link_airborne')air.set(effect.target,tick+effect.ticks);
    }
    frames.push(f);states.push(result);
  }
  const effects=states.flatMap(s=>s.effects);
  assert.deepEqual(effects.filter(e=>e.kind==='link_stun').map(e=>e.target),[10,12]);
  assert.ok(!effects.some(e=>e.kind==='damage'));
  return {frames,states};
}
function parity(trace,current,frames,results){
  const output=spawnSync(trace,[],{
    input:frames.map(frame=>JSON.stringify({version:1,rules_revision:current.revision,frame})).join('\n')+'\n',
    encoding:'utf8',maxBuffer:48*1024*1024,
  });
  assert.equal(output.status,0,output.stderr);
  const got=output.stdout.trim().split(/\r?\n/).map(JSON.parse);
  assert.equal(got.length,results.length);
  for(let i=0;i<got.length;i++)assert.deepEqual(got[i],results[i], 'frame '+frames[i].tick);
}
(async()=>{
  const blob=spawnSync('rtk',['proxy','git','show',baseline+':editor/contender.wasm'],{maxBuffer:4*1024*1024});
  assert.equal(blob.status,0,blob.stderr?.toString());
  const old=await wasm(blob.stdout,'contender-3.0');
  const revision=JSON.parse(readFileSync('rules/contender.json','utf8')).revision;
  const current=await wasm(readFileSync('editor/contender.wasm'),revision);
  const trace=path.resolve('native/contender_shadow_lab/target/release/'+(process.platform==='win32'?'trace.exe':'trace'));
  const matched=[];
  let parityFrames=0;
  for(const count of [1,3,5])for(const profile of [20,80]){
    const before=sample(old,count,profile), after=sample(current,count,profile);
    parity(trace,current,after.frames,after.states);
    assert.equal(before.metrics.punches,72);
    assert.equal(after.metrics.punches,10);
    assert.ok(after.metrics.visuals<before.metrics.visuals);
    parityFrames+=after.frames.length;
    matched.push({before:before.metrics,after:after.metrics});
  }
  const link=linkTrace(current);
  parity(trace,current,link.frames,link.states);
  parityFrames+=link.frames.length;
  console.log(JSON.stringify({baseline,parityFrames,matched,scope:'Fixed-snapshot synthetic hit receipts; timings include WASM and JSON, not the game renderer.'},null,2));
})().catch(e=>{console.error(e);process.exitCode=1;});
