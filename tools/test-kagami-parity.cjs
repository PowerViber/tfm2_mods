const fs=require('fs'),cp=require('child_process'),assert=require('assert'),crypto=require('crypto'),vm=require('vm');
const ctx={window:{}};vm.runInNewContext(fs.readFileSync('editor/kagami-data.js','utf8'),ctx);const R=ctx.window.KAGAMI_RULES;
const world=JSON.parse(fs.readFileSync('editor/game-reference/world.json')),terrain=world.doc.walls.flat().map(Boolean);
function fixtures(){
 const result=[];
 for(let i=0;i<36;i++){
  const team=i%2,x=360000+i%3*32000,y=500000+i%3*12000,kind=['heal','rescue','escape','charm','clone-death','ult-seal'][i%6],inputs=[];
  const unit=(id,t,px,hp)=>({id,team:t,pos:{x:px,y},hp,max_hp:1000,attack:150,champion:true,tower:false,visible:true,targetable:true,sealed:false});
  for(let tick=1;tick<=120;tick++){
   let ally=unit(2,team,x+45000,kind==='heal'?300:kind==='rescue'?250:1000),foe=unit(10,1-team,x+55000,1000);
   let clone={id:99,pos:{x:x-25000,y},hp:350,max_hp:350};
   if(kind==='clone-death'&&tick>30&&tick<100)clone=null;
   if(kind==='ult-seal'&&tick>10)ally.hp=50;
   const profile=Array(13).fill(50);profile[i%13]=i<24?20:80;
   const order=tick===20?(kind==='heal'?{kind:'mend',aim:2}:kind==='rescue'?{kind:'step_ally',aim:2}:kind==='charm'?{kind:'charm',aim:2}:null):null;
   inputs.push({version:1,rules_revision:R.revision,frame:{tick,owner:1,team,alive:true,pos:{x,y},hp:kind==='escape'&&tick>10?400:1000,max_hp:1000,attack:100,stunned:i%9===0&&tick>60&&tick<80,ult_unlocked:true,home:{x:96000,y:864000},units:[ally,foe],clone,profile,manual:i%3===0,order,terrain,hazards:i%4===0?[{id:55,at:{x:x+20000,y},radius:14000,until:90}]:[]}});
  }
  result.push({name:(i<24?'development':'holdout')+'-'+i+'-'+kind,inputs});
 }return result;
}
(async()=>{
 const cases=fixtures(),native=cp.spawnSync('native/kagami_lab/target/release/examples/replay.exe',[],{input:cases.map(c=>JSON.stringify(c.inputs)).join('\n')+'\n',encoding:'utf8',maxBuffer:100*1024*1024});assert.equal(native.status,0,native.stderr);
 const expected=native.stdout.trim().split(/\r?\n/).map(JSON.parse),ex=(await WebAssembly.instantiate(fs.readFileSync('editor/kagami.wasm'),{})).instance.exports;
 const run=input=>{const bytes=Buffer.from(JSON.stringify(input));const p=ex.kagami_input(bytes.length);new Uint8Array(ex.memory.buffer,p,bytes.length).set(bytes);const out=ex.kagami_step(bytes.length);return JSON.parse(Buffer.from(ex.memory.buffer,out,ex.kagami_output_len()).toString());};
 const digest=crypto.createHash('sha256');let ticks=0,events=0;
 cases.forEach((c,i)=>{ex.kagami_reset();const actual=c.inputs.map(run);assert(actual.every(r=>!r.error),c.name);assert.deepStrictEqual(actual,expected[i],c.name);ticks+=actual.length;events+=actual.reduce((n,r)=>n+r.effects.length,0);digest.update(JSON.stringify(actual));});
 const benchmarks=[];
 for(let n=0;n<5;n++){ex.kagami_reset();const start=performance.now();for(const input of cases[0].inputs)run(input);benchmarks.push((performance.now()-start)/120*1000);}
 const report={revision:R.revision,development:24,holdout:12,ticks,events,complete_state_and_effects_match:true,trace_sha256:digest.digest('hex'),wasm_transport_us_per_tick:benchmarks,kind:'deterministic snapshot fixtures; not live-game balance results'};
 fs.writeFileSync('docs/kagami-art-review/parity.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
})().catch(e=>{console.error(e);process.exitCode=1;});
