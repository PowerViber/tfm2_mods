/* Compare every native state field and emitted command with the shipped WASM. */
'use strict';
const fs=require('fs'),zlib=require('zlib'),path=require('path'),assert=require('assert');
const ROOT=path.resolve(__dirname,'..');
const MagnusEngine=require('../editor/magnus-engine.js');
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(ROOT,'editor/magnus.wasm')),{});
 const engine=new MagnusEngine(instance.exports);let key=null,n=0;
 const file=path.join(ROOT,'editor/magnus-vectors.jsonl.gz');
 for(const line of zlib.gunzipSync(fs.readFileSync(file)).toString().trim().split('\n')){
   const v=JSON.parse(line),k=v.group;
   if(k!==key){if(v.fixture!==null&&v.fixture!==undefined)engine.fixture(v.fixture);else engine.reset(v.seed,v.rank);key=k;}
   const ptr=engine.api.magnus_input(v.values.length);
   new BigInt64Array(engine.api.memory.buffer,ptr,v.values.length).set(v.values.map(BigInt));
   const actual=engine.read(engine.api.magnus_step(engine.handle,v.values.length));
   assert.deepStrictEqual(actual,v.expected,'Native/editor mismatch '+k+' tick '+v.values[1]);n++;
 }
 const a=new MagnusEngine(instance.exports),b=new MagnusEngine(instance.exports);a.reset(11,1);b.reset(22,7,2);
 const frame={tick:0,owner:{hp:800,maxHp:800,team:0,x:400000,y:400000,attack:45,ap:50},rank:1,action:{type:0},fighters:[]};
 a.step(frame);const untouched=b.step({...frame,action:null});assert.equal(untouched.board,null);
 const rook=a.add(0,8,8);a.step({...frame,tick:1,action:null});
 const coverage=a.coverage(rook);assert(coverage.some(([x,y])=>x===9&&y===8));
 b.step({...frame,tick:1,action:null,walls:[[9*24000-100,8*24000-100,9*24000+100,8*24000+100]]});
 assert.deepStrictEqual(a.coverage(rook),coverage,'Another session must not replace this session’s inspection terrain');
 assert(a.step({...frame,tick:2,rank:7,action:null}).debugState.includes('rank: 7'));
 console.log('PASS: '+n+' complete native/editor frames; independent concurrent instances.');
})();
