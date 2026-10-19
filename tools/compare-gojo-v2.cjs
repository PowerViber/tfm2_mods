/* Matched host-free scenes. Raw requested damage is not engine-attributed damage. */
const fs=require('fs'),path=require('path'),assert=require('assert');
const root=path.resolve(__dirname,'..');
async function load(file){return(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'editor',file)))).instance.exports;}
function invoke(a,h,frame){const p=a.gojo_input(frame.length);new BigInt64Array(a.memory.buffer,p,frame.length).set(frame.map(BigInt));const q=a.gojo_step(h,frame.length);return JSON.parse(new TextDecoder().decode(new Uint8Array(a.memory.buffer,q,a.gojo_output_len())));}
function scene(api,version,kind,seed){
 const h=api.gojo_new(),unit=(id,team,x,y,champion=true)=>({id,team,x,y,hp:id===1?935:12000,max:id===1?935:12000,ad:70,ap:id===1?81:60,champion,visible:true,stun:0,hold:0,guard:0,shield:0,published:false,powers:[]});
 const units=[unit(1,0,120000,120000),unit(2,1,138000+seed*600,120000)];
 if(kind==='team'){units.push(unit(3,0,122000,120000),unit(4,0,220000,150000),unit(5,1,154000,132000));}
 if(kind==='farm'){units[1].champion=false;units.push(unit(5,1,154000,125000,false));}
 const m=units[0];let state,attackAt=0,serial=0,requested=0,rawHpLoss=0,starved=0,exhausted=0,fatigue=0,allyLost=0,flashes=0,nukes=0,wasted=0,lastHp=m.hp;
 for(let t=0;t<1800;t++){
  for(const u of units){u.powers=u.powers.filter(p=>p.end>t);const bonus=u.powers.reduce((n,p)=>n+(p.slow?30:15),0);u.ad=Math.floor(70*(100+bonus)/100);u.ap=Math.floor((u.id===1?81:60)*(100+bonus)/100);}
  for(const u of units.filter(u=>u.team===1&&u.hp>0&&u.stun<=t&&u.hold<=t)){const dx=m.x-u.x,dy=m.y-u.y,n=Math.hypot(dx,dy);if(n>17000){u.x+=Math.round(dx/n*350);u.y+=Math.round(dy/n*350);}}
  const target=units.filter(u=>u.team===1&&u.hp>0&&Math.hypot(u.x-m.x,u.y-m.y)<=23000).sort((a,b)=>a.id-b.id)[0];
  const hit=target&&m.hp>0&&m.stun<=t&&!(state?.lockUntil>t)&&t>=attackAt;
  if(hit){serial++;const speed=m.powers.reduce((n,p)=>n+(p.slow?25:40),100);attackAt=t+Math.ceil(5500/speed);}
  const incoming=m.hp<lastHp;lastHp=m.hp;
  const rows=units.map(u=>[u.id,u.team,Math.round(u.x),Math.round(u.y),u.hp,u.max,u.ad,u.ap,3,+u.champion,1,1,(u.guard>t?3:0),+!u.champion,0]);
  const frame=[version,t,1,1,0,2,0,0,7,+(m.stun>t),+(m.hold>t),hit?serial:0,hit?target.id:0,+incoming,m.published?m.shield:-1,units.length,...rows.flat(),0];
  state=invoke(api,h,frame);assert(!state.error);
  for(const c of state.commands){const u=units.find(u=>u.id===c[1]);switch(c[0]){
   case 0:{requested+=c[2]+c[3];if(u&&u.guard<=t){const n=c[2]+c[3],absorbed=Math.min(u.shield,n);u.shield-=absorbed;const loss=Math.min(u.hp,n-absorbed);u.hp-=loss;rawHpLoss+=loss;}break;}
   case 1:if(u){u.x=c[2];u.y=c[3];}break;
   case 2:m.hp=Math.min(m.max,m.hp+c[1]);break;
   case 3:if(c[2]<=0)m.published=false;if(c[1]&&!m.published){m.shield+=c[2];m.published=true;}break;
   case 4:if(u)u.stun=Math.max(u.stun,t+c[2]);break;
   case 5:if(u)u.guard=t+c[2];break;
   case 10:if(u)u.powers.push({end:t+c[2],slow:!!c[3]});break;
   case 11:if(u)u.hold=Math.max(u.hold,t+c[2]);break;
   case 12:m.hp=Math.max(1,m.hp-Math.floor(m.hp/2));nukes++;break;
  }}
  for(const u of units.filter(u=>u.team===1&&u.hp>0&&u.stun<=t&&Math.hypot(u.x-m.x,u.y-m.y)<=23000))if(t%55===0&&m.hp>0&&m.guard<=t){let n=state.infinity?Math.floor(35*.7):35;const absorbed=Math.min(m.shield,n);m.shield-=absorbed;m.hp=Math.max(0,m.hp-n+absorbed);}
  starved+=+(state.ce<(version===2&&state.eyes?600:6000));exhausted+=+(state.eyes===false);fatigue+=+(state.burnout>t);allyLost+=units.filter(u=>u.team===0&&u.id!==1&&u.stun>t).length;flashes=state.flashes;wasted+=+(state.reason==='Preparation lost target'||state.reason==='Interrupted');
 }
 api.gojo_drop(h);return {kind,seed,version,seconds:30,requestedRawDamage:requested,rawFixtureHpLoss:rawHpLoss,remainingHp:m.hp,survived:m.hp>0,flashes,nukes,starvedTicks:starved,eyeExhaustedTicks:exhausted,postDomainFatigueTicks:fatigue,alliedStunTicks:allyLost,wastedPreparationSnapshots:wasted};
}
(async()=>{const a=await load('gojo-v1.wasm'),b=await load('gojo.wasm'),rows=[];for(const kind of ['duel','team','farm'])for(let seed=1;seed<=5;seed++)rows.push({baseline:scene(a,1,kind,seed),candidate:scene(b,2,kind,seed)});const averages=['duel','team','farm'].map(kind=>{const r=rows.filter(r=>r.baseline.kind===kind);const average=side=>Object.fromEntries(['requestedRawDamage','remainingHp','flashes','starvedTicks','alliedStunTicks'].map(k=>[k,r.reduce((n,r)=>n+r[side][k],0)/r.length]));return {kind,baseline:average('baseline'),candidate:average('candidate')};});const report={scope:'Matched deterministic host-free 30s fixtures, five seeds each. Enemy approach, basic clocks and raw shield bookkeeping are explicit fixture behavior. No engine armour, items, attribution, kill credit or live balance claim.',averages,rows};fs.writeFileSync(path.join(root,'docs/gojo-results/comparison.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(averages,null,2));})();
