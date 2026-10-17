/* Build both targets and compare complete core state and effects for 36 deterministic scenarios. */
const fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),manifest=path.join(root,'native/frieren_lab/Cargo.toml');
function build(args){const p=cp.spawnSync('rtk',['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo',...args,'--manifest-path',manifest],{cwd:root,encoding:'utf8'});assert.equal(p.status,0,p.stdout+p.stderr);}
(async()=>{build(['build','--release','--target','wasm32-unknown-unknown']);build(['build','--release','--bin','vectors']);
const source=path.join(root,'native/frieren_lab/target/wasm32-unknown-unknown/release/frieren_lab.wasm');fs.copyFileSync(source,path.join(root,'editor/frieren.wasm'));
const api=(await WebAssembly.instantiate(fs.readFileSync(source))).instance.exports;
let input=[],expected=[],metrics=[];
for(let seed=1;seed<=36;seed++){
 const id=api.frieren_new(),units=[[1,0,100000,100000,1000,1000,100,50,1,1,1,0,0,0,0],[2,1,160000,100000,1000,1000,50,50,1,1,1,0,0,0,0],[3,0,110000,130000,420,1000,60,50,1,1,1,0,0,0,0],[4,1,180000,110000,1000,1000,50,50,1,1,1,0,0,0,0]];
 let stark=null,lastY=100000,damage=0,shots=0,controls=0,spawned=0;input.push('reset');
 for(let tick=0;tick<720;tick++){
  const t=(tick+seed*7)%250,y=100000+Math.round(Math.sin((tick+seed)/45)*15000);
  units[1][13]=0;units[1][14]=y-lastY;units[1][3]=y;lastY=y;units[1][8]=+(t>12);units[0][12]=+(t>=40&&t<49);units[0][4]=tick>=400&&tick<460?0:1000;
  let action=0,target=2;if(seed%2===0){if(tick===0)action=2;if(tick===60){action=3;target=3;}if(tick===90)action=4;if(tick===250)action=1;}
  const a=[1,tick,1,seed%2,+(tick>50),1,action,target,+!!stark,stark?.[0]||0,stark?.[1]||0,units.length,...units.flat(),0];
  const ptr=api.frieren_input(a.length);new BigInt64Array(api.memory.buffer,ptr,a.length).set(a.map(BigInt));const out=api.frieren_step(id,a.length),state=JSON.parse(new TextDecoder().decode(new Uint8Array(api.memory.buffer,out,api.frieren_output_len())));
  expected.push(state);input.push(a.join(' '));
  for(const cmd of state.commands){if(cmd[0]===0)damage+=cmd[2];if(cmd[0]===7)shots++;if(cmd[0]===3){spawned++;if(seed%5)stark=[100000,100000];}if(cmd[0]===4)stark=null;if(cmd[0]===5&&stark)stark=[cmd[1],cmd[2]];if(cmd[0]===6)controls++;}
  if(tick===350&&seed%3===0)stark=null;
 }
 api.frieren_drop(id);metrics.push({seed,group:seed<=24?'development':'holdout',requestedRawDamage:damage,shots,preparations:controls,spawnRequests:spawned});
}
const native=cp.spawnSync(path.join(root,'native/frieren_lab/target/release/vectors.exe'),[],{input:input.join('\n')+'\n',encoding:'utf8',maxBuffer:40*1024*1024});assert.equal(native.status,0,native.stderr);const actual=native.stdout.trim().split(/\r?\n/).map(JSON.parse);assert.equal(actual.length,expected.length);for(let i=0;i<expected.length;i++)assert.deepEqual(actual[i],expected[i],'complete Frieren parity frame '+i);
fs.mkdirSync(path.join(root,'docs/rework-results'),{recursive:true});fs.writeFileSync(path.join(root,'docs/rework-results/frieren-parity.json'),JSON.stringify({frames:actual.length,scenarios:metrics},null,2)+'\n');console.log(`PASS: ${actual.length} complete native/WASM state and effect frames; 24 development + 12 holdout Frieren scenarios.`);
})().catch(e=>{console.error(e);process.exitCode=1;});
