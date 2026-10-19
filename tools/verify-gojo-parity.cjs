/* Full native/WASM state and command parity; deterministic scene bookkeeping is not engine damage. */
const fs=require('fs'),path=require('path'),cp=require('child_process'),assert=require('assert');
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/gojo-results');fs.mkdirSync(out,{recursive:true});
function build(args){cp.execFileSync('rtk',['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo',...args,'--manifest-path','native/gojo_lab/Cargo.toml'],{cwd:root,stdio:'inherit'});}
(async()=>{build(['build','--release','--lib','--target','wasm32-unknown-unknown']);build(['build','--release','--bin','gojo-vectors']);const bytes=fs.readFileSync(path.join(root,'native/gojo_lab/target/wasm32-unknown-unknown/release/gojo_lab.wasm'));fs.writeFileSync(path.join(root,'editor/gojo.wasm'),bytes);const api=(await WebAssembly.instantiate(bytes)).instance.exports;
let frames=0;const scenarios=[];
function call(h,a){const p=api.gojo_input(a.length);new BigInt64Array(api.memory.buffer,p,a.length).set(a.map(BigInt));const q=api.gojo_step(h,a.length);return JSON.parse(new TextDecoder().decode(new Uint8Array(api.memory.buffer,q,api.gojo_output_len())));}
for(let seed=1;seed<=36;seed++){
 const h=[api.gojo_new(),api.gojo_new()],rows=[],expected=[];let serial=0,damage=0,flashes=0,allyTicks=0;const unit=(id,team,x,y,champion=true)=>[id,team,x,y,100000,100000,70,81,3,+champion,1,1,0,+!champion,0];const units=[unit(1,0,100000,100000),unit(2,1,120000+(seed%3)*4000,100000),unit(3,0,104000,100000),unit(4,0,112000,100000),unit(5,1,150000,120000)];
 for(let t=0;t<720;t++){
  units[1][3]=100000+Math.round(Math.sin((t+seed*17)/95)*7000);units[1][10]=+(seed%4!==0||t%240<150);units[1][12]=+(seed%7===0&&t%180<40);units[1][9]=+(seed%8!==0);units[1][13]=+!units[1][9];
  let action=0;if(t===0)action=seed%3===0?8:seed%3===1?1:3;if(t===280)action=5;if(t===470)action=6;if(t===590)action=7;
  const basic=t>36&&t%55===37?++serial:0;const walls=seed%5===0?[[7,6],[7,7],[7,8]]:[];
  for(let owner=0;owner<2;owner++){
   const a=[1,t,1,+(owner===1),owner===0?action:0,2,125000,110000,seed%9===0?3:7,+(seed%6===0&&t>=10&&t<30),+(seed%10===0),basic,basic?2:0,+(t%120===0),-1,units.length,...units.flat(),walls.length,...walls.flat()];
   rows.push(owner+' '+a.join(' '));const s=call(h[owner],a);assert(!s.error);expected.push(s);damage+=s.commands.filter(c=>c[0]===0).reduce((n,c)=>n+c[2]+c[3],0);if(owner===0){flashes=s.flashes;allyTicks=+(s.core.match(/ally_seal_ticks: (\d+)/)?.[1]||0);}
  }
 }
 const text=cp.execFileSync(path.join(root,'native/gojo_lab/target/release/gojo-vectors.exe'),{input:rows.join('\n')+'\n',encoding:'utf8',maxBuffer:64*1024*1024});const actual=text.trim().split(/\r?\n/).map(JSON.parse);assert.equal(actual.length,expected.length);actual.forEach((s,i)=>assert.deepStrictEqual(s,expected[i],`seed ${seed} frame ${i}`));frames+=actual.length;scenarios.push({seed,group:seed<=24?'development':'holdout',frames:actual.length,requestedRawDamage:damage,manualFlashes:flashes,alliedSealTicks:allyTicks});h.forEach(api.gojo_drop);
}
const report={frames,completePrivateStateAndCommands:true,independentSessions:true,scenarios,scope:'Host-free core, not observed engine damage or live acceptance'};fs.writeFileSync(path.join(out,'parity.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({frames,scenarios:scenarios.length,passed:true}));})();
