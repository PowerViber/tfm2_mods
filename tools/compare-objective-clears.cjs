/* Host-free clear-time models using the actual Frieren core; no engine attribution claim. */
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');const root=path.resolve(__dirname,'..');
(async()=>{const a=(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'editor/frieren.wasm')))).instance.exports,rows=[];
for(const [name,base,growth] of [['Serpen',5000,1000],['Morgard',10000,1500]])for(const steps of [0,3])for(const party of [2,5])for(const skills of [false,true]){
 const clears=[];for(const multiplier of [1,2]){let hp=(base+steps*growth)*multiplier;const max=hp,ids=Array.from({length:party},()=>a.frieren_new());let tick;
 for(tick=0;tick<36000&&hp>0;tick++)for(let i=0;i<party&&hp>0;i++){
  const units=Array.from({length:party},(_,j)=>[j+1,0,100000+j*1000,100000,1000,1000,100,50,1,1,1,0,0,0,0]);units.push([99,2,160000,100000,hp,max,0,0,1,1,0,1,0,0,0]);
  const f=[2,tick,i+1,+skills,0,1,0,99,0,999,0,0,0,0,1,1,units.length,...units.flat(),0];const p=a.frieren_input(f.length);new BigInt64Array(a.memory.buffer,p,f.length).set(f.map(BigInt));const o=a.frieren_step(ids[i],f.length);const s=JSON.parse(new TextDecoder().decode(new Uint8Array(a.memory.buffer,o,a.frieren_output_len())));
  for(const c of s.commands)if(c[0]===0&&c[1]===99)hp=Math.max(0,hp-c[2]);
  if(tick%72===0)hp=Math.max(0,hp-65); // unchanged data basic: 20 + 45% of 100 AP
 }assert.ok(hp<=0,'fixture did not clear');clears.push(tick);for(const id of ids)a.frieren_drop(id);}
 rows.push({objective:name,growthSteps:steps,attackers:party,mode:skills?'Frieren core plus data basic':'Data basic only',baseClearSeconds:+(clears[0]/60).toFixed(3),doubledClearSeconds:+(clears[1]/60).toFixed(3)});
}
const result={scope:'Unmitigated host-free fixtures, static objectives, 100 AP per attacker. Not stock-team or real-match clear times; no item, armour, aggro or retreat simulation.',rows,percentageHpNote:'Current percentage-of-maximum-HP equations retain their coefficients. Their raw damage scales with the enlarged objective and does not promise double clear time.'};fs.mkdirSync(path.join(root,'docs/objective-stark-results'),{recursive:true});fs.writeFileSync(path.join(root,'docs/objective-stark-results/objective-clears.json'),JSON.stringify(result,null,2)+'\n');console.log('PASS:',rows.length,'paired objective-clear models; each doubled HP/growth fixture completes.');})().catch(e=>{console.error(e);process.exitCode=1;});
