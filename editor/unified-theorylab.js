/* Science Lab: exact integer recipe math and mastery comparisons; UI lives in science-lab-ui.js. */
(function(root){
'use strict';
const D=typeof module!=='undefined'?require('./unified-theory-data.js'):root.ScienceData;
const Model=typeof module!=='undefined'?require('./science-charge-model.js'):root.ScienceCharges;
const {ChargeBank,parseDiagnostics}=Model;
const ranks=['Student','Lab Assistant','Researcher','Scientist','Professor','Fellow','Laureate','Unified Mind'];
function instability(w,a){
 if(!w.length||w.length!==a.length||w.some(x=>!Number.isSafeInteger(x)||x<1)||a.some(x=>!Number.isSafeInteger(x)||x<1))return null;
 const W=w.reduce((s,x)=>s+BigInt(x),0n),A=a.reduce((s,x)=>s+BigInt(x),0n);if(A*4n<W*3n)return null;
 let n=0n,d=1n;for(let i=0;i<w.length;i++){let num=BigInt(a[i])*W-BigInt(w[i])*A;num=(num<0n?-num:num)*100n;const den=BigInt(w[i])*A;if(num*d>n*den){n=num;d=den;}}
 const over=A*4n>W*5n?(A*4n-W*5n)*20n:0n,num=n*W+over*d,den=d*W;return Math.min(100,Number((num+den/2n)/den));
}
function strength(w,a){return Math.min(125,Math.floor(a.reduce((s,x)=>s+x,0)*100/w.reduce((s,x)=>s+x,0)));}
function state(w,a){const score=instability(w,a);return score===null?'undercharged':score<=20?'stable':score<=40?'strained':score<=70?'unstable':'critical';}
// Round 125, mirrored from native unified_theory.rs: chain payoff, lab accidents, resonance.
const OFFENSE=new Set([0,16,19,29,34,39,40,54,59,63,65,71,72,73,74]),HEAVY=new Set([34,63,71,73,74]);
const CHAIN_STEP=20,CHAIN_MAX=4,RESONANCE_COOLDOWN=360,ACCIDENT_HP_PCT=4,ACCIDENT_STAGGER=30;
const chainPct=commits=>100+CHAIN_STEP*Math.min(commits,CHAIN_MAX);
const baseOutput=(skill,stat)=>HEAVY.has(skill)?90+Math.floor(stat*45/100):50+Math.floor(stat/4);
const resonanceOutput=(power,physical)=>40+Math.floor(Math.max(power,physical)/5);
const accidentLoss=maxHp=>Math.floor(maxHp*ACCIDENT_HP_PCT/100);
function validateVectors(raw){let count=0;for(const row of raw.trim().split('\n')){if(!row)continue;const [w,a,score,power]=row.split('|');const actual=instability(w.split(',').map(Number),a.split(',').map(Number));if(actual!==(score==='invalid'?null:Number(score))||strength(w.split(',').map(Number),a.split(',').map(Number))!==Number(power))throw Error('Native charge mismatch: '+row);count++;}return count;}
function simulate(rank,seed=1){
 rank=Math.max(0,Math.min(7,rank));let rng=BigInt(seed)||1n;const random=n=>{rng=BigInt.asUintN(64,rng^(rng<<13n));rng=BigInt.asUintN(64,rng^(rng>>7n));rng=BigInt.asUintN(64,rng^(rng<<17n));return Number(rng%BigInt(n));};
 const bank=new ChargeBank(),rows=[];let tick=0,casts=0,failures=0,accidents=0,spent=0;
 for(let k=0;k<75;k++){
  const s=D.skills[k],j=[40,30,22,16,10,6,3,1][rank];let a=s.charge.map(w=>Math.max(1,Math.floor((w*(100+random(j*2+1)-j)+50)/100)));
  if(random(100)<[15,25,40,55,70,85,96,100][rank])a=s.charge.slice();const n=a.reduce((v,x)=>v+x,0)*100;
  let slot=bank.reserve(tick);while(slot===null){bank.advance(++tick);slot=bank.reserve(tick);}
  let error=false;for(const t of s.tokens){if(random(10000)<[1200,800,600,400,200,100,30,10][rank])error=true;for(let i=0;i<10;i++){bank.advance(++tick);}}
  for(let i=0;i<8;i++){bank.advance(++tick);}bank.commit(slot,tick);spent++;
  error=error&&random(100)>=[15,25,40,55,70,85,96,100][rank];const score=instability(s.charge,a),ok=!error&&score!==null&&score<=70;
  const accident=!ok&&(error||(score!==null&&score>70));
  casts+=Number(ok);failures+=Number(!ok);accidents+=Number(accident);rows.push({id:s.id,charge:a,score,ok,accident});
 }
 // Every recipe as one chain: a failure ends it; offense after k successes
 // carries chainPct(k); all three sciences committed together resonate.
 let chains=0,resonances=0,payoff=0,offenses=0;
 for(const combo of D.combos){
  let commits=0,mask=0,failed=false;
  for(const skill of combo.steps){
   const s=D.skills[skill],j=[40,30,22,16,10,6,3,1][rank];let a=s.charge.map(w=>Math.max(1,Math.floor((w*(100+random(j*2+1)-j)+50)/100)));
   if(random(100)<[15,25,40,55,70,85,96,100][rank])a=s.charge.slice();
   let error=false;for(const t of s.tokens)if(random(10000)<[1200,800,600,400,200,100,30,10][rank])error=true;
   error=error&&random(100)>=[15,25,40,55,70,85,96,100][rank];const score=instability(s.charge,a);
   if(error||score===null||score>70){failed=true;accidents+=Number(error||(score!==null&&score>70));break;}
   if(OFFENSE.has(skill)){payoff+=chainPct(commits);offenses++;}
   commits++;mask|=1<<s.science;
  }
  if(!failed){chains++;if(mask===7)resonances++;}
 }
 return {rank,ticks:tick,casts,failures,accidents,spent,rows,chains,resonances,payoff:offenses?payoff/offenses:100};
}
function selfTest(){for(const [a,n]of[[[4,5,3],0],[[5,6,4],7],[[4,8,3],28],[[4,12,3],78],[[8,10,6],60]])if(instability([4,5,3],a)!==n)throw Error('charge vector');if(instability([5,5,4],[5,8,4])!==32)throw Error('gamma');if(D.skills.length!==75||D.combos.length!==50)throw Error('catalogue');const p=new ChargeBank();for(const t of [0,30,60])p.commit(p.reserve(t),t);if(p.reserve(299)!==null||JSON.stringify(p.cells(150))!=='[5,4,3]')throw Error('charge parity');p.advance(300);if(p.available()!==1)throw Error('recovery');p.advance(360);if(p.available()!==3)throw Error('independent recovery');const slot=p.reserve(400);p.cancel(slot);if(p.available()!==3)throw Error('refund');for(let r=0;r<8;r++){const a=simulate(r,42),b=simulate(r,42);if(JSON.stringify(a)!==JSON.stringify(b)||a.casts+a.failures!==75)throw Error('determinism');}
 // Same values the native unit test asserts (chain_percentages_are_bounded).
 if([0,1,2,3,4,5,6].map(chainPct).join()!=='100,120,140,160,180,180,180'||baseOutput(0,400)!==150||baseOutput(73,400)!==270||resonanceOutput(400,300)!==120||accidentLoss(1000)!==40)throw Error('round 125 parity');
 const lo=[0,1,2].map(i=>simulate(0,42+i)),hi=[0,1,2].map(i=>simulate(7,42+i)),sum=(x,k)=>x.reduce((n,v)=>n+v[k],0);
 if(!(sum(hi,'chains')>sum(lo,'chains')&&sum(lo,'accidents')>sum(hi,'accidents')))throw Error('mastery difficulty curve');
 return true;}
const api={data:D,ranks,instability,strength,state,ChargeBank,parseDiagnostics,validateVectors,simulate,selfTest,chainPct,baseOutput,resonanceOutput,accidentLoss,OFFENSE,HEAVY,RESONANCE_COOLDOWN,ACCIDENT_HP_PCT,ACCIDENT_STAGGER};if(typeof module!=='undefined'){module.exports=api;return;}root.ScienceLab=api;
})(typeof globalThis!=='undefined'?globalThis:this);
