// Exact native/WASM transport traces; host movement/combat remains a fixture model.
const fs=require('fs'),assert=require('assert'),cp=require('child_process'),crypto=require('crypto');
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync('editor/basalt.wasm')),e=instance.exports,R=JSON.parse(fs.readFileSync('rules/basalt.json','utf8'));
 const plans=[[[0,'s1'],[40,'s1']],[[0,'s2']],[[0,'s3']],[[0,'jump'],[3,'s1']],[[0,'jump'],[3,'s2']],[[0,'jump'],[3,'s3']],[[0,'ult']],[[0,'jump'],[3,'ult']],[[0,'s1'],[15,'pull']],[[0,'jump'],[4,'jump'],[10,'s2'],[20,'ult']],[[0,'s1'],[50,'s2'],[90,'s3']],[[0,'s3'],[20,'s1'],[40,'jump'],[43,'s2']]];
 const inputs=[],expected=[],cases=[];
 for(let variant=0;variant<3;variant++)for(let i=0;i<plans.length;i++){
  inputs.push('reset');expected.push({reset:true});e.basalt_reset();const angle=[0,90,37][variant]*Math.PI/180,origin={x:400000,y:400000},at=d=>({x:Math.round(origin.x+Math.cos(angle)*d),y:Math.round(origin.y+Math.sin(angle)*d)}),timeline=new Map(plans[i]);
  const units=[35000,65000,120000].map((d,j)=>({id:10+j,team:1,pos:at(d),hp:2000,max_hp:2000,attack:100,champion:j<2,tower:j===2,visible:true,targetable:true}));
  const terrain=Array(900).fill(false);if(i===11)terrain[12*30+14]=true;let damage=0;
  for(let t=0;t<300;t++){
   const alive=!(i===9&&t>=250),frame={tick:t+1,owner:1,team:0,alive,pos:origin,hp:alive?2000:0,max_hp:2000,attack:100,stunned:i===10&&t===120,ult_unlocked:true,home:{x:96000,y:864000},terrain,other_walls:[],units,profile:Array(13).fill([20,50,80][variant]),manual:true,order:timeline.has(t)?{kind:timeline.get(t),aim:at(i===6||i===7||i===9?400000:90000)}:null};
   const line=JSON.stringify({version:1,rules_revision:R.revision,frame}),bytes=new TextEncoder().encode(line),ptr=e.basalt_input(bytes.length);new Uint8Array(e.memory.buffer,ptr,bytes.length).set(bytes);const p=e.basalt_step(bytes.length),out=JSON.parse(new TextDecoder().decode(new Uint8Array(e.memory.buffer,p,e.basalt_output_len())));assert(!out.error,out.error);damage+=out.effects.filter(x=>x.type==='damage').length;inputs.push(line);expected.push(out);
  }
  cases.push({group:variant<2?'development':'holdout',fixture:i,angle:[0,90,37][variant],ticks:300,damage_packets:damage});
 }
 const exe='native/basalt_lab/target/release/examples/trace.exe',r=cp.spawnSync('rtk',['proxy',exe],{input:inputs.join('\n')+'\n',encoding:'utf8',maxBuffer:100*1024*1024});assert(r.status===0,r.stderr);const actual=r.stdout.trim().split(/\r?\n/).map(s=>JSON.parse(s));assert(actual.length===expected.length);
 for(let i=0;i<actual.length;i++)assert.deepStrictEqual(actual[i],expected[i],`Native/WASM trace index ${i}`);
 const hash=crypto.createHash('sha256').update(JSON.stringify(expected)).digest('hex');const report={rules_revision:R.revision,cases,development_cases:24,holdout_cases:12,compared_ticks:10800,complete_state_and_effects_equal:true,sha256:hash,scope:'Exact native transport versus WASM serialization on fixed input fixtures. Not host physics, live combat, or balance acceptance.'};fs.writeFileSync((process.env.BASALT_RESULTS||'docs/basalt-art-review')+'/parity.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,cases:report.cases.length},null,2));
})().catch(e=>{console.error(e);process.exit(1);});
