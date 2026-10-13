/* Science Lab: exact integer charge math, with a deliberately schematic combat preview. */
(function(root){
'use strict';
const D=typeof module!=='undefined'?require('./unified-theory-data.js'):root.ScienceData;
const ranks=['Student','Lab Assistant','Researcher','Scientist','Professor','Fellow','Laureate','Unified Mind'];
function instability(w,a){
 if(!w.length||w.length!==a.length||w.some(x=>!Number.isSafeInteger(x)||x<1)||a.some(x=>!Number.isSafeInteger(x)||x<1))return null;
 const W=w.reduce((s,x)=>s+BigInt(x),0n),A=a.reduce((s,x)=>s+BigInt(x),0n);if(A*4n<W*3n)return null;
 let n=0n,d=1n;for(let i=0;i<w.length;i++){let num=BigInt(a[i])*W-BigInt(w[i])*A;num=(num<0n?-num:num)*100n;const den=BigInt(w[i])*A;if(num*d>n*den){n=num;d=den;}}
 const over=A*4n>W*5n?(A*4n-W*5n)*20n:0n,num=n*W+over*d,den=d*W;return Math.min(100,Number((num+den/2n)/den));
}
function strength(w,a){return Math.min(125,Math.floor(a.reduce((s,x)=>s+x,0)*100/w.reduce((s,x)=>s+x,0)));}
function state(w,a){const score=instability(w,a);return score===null?'undercharged':score<=20?'stable':score<=40?'strained':score<=70?'unstable':'critical';}
class Pool{constructor(){this.free=10000;this.reserved=0;this.remainder=0;}reserve(n){if(n>this.free||n<0)return false;this.free-=n;this.reserved+=n;return true;}commit(n){if(n>this.reserved||n<0)return false;this.reserved-=n;return true;}refund(n){n=Math.min(n,this.reserved);this.reserved-=n;this.free+=n;}regen(combat){this.remainder+=combat?700:1200;const n=Math.floor(this.remainder/60);this.remainder%=60;this.free=Math.min(this.free+n,10000-this.reserved);}}
// Round 125, mirrored from native unified_theory.rs: chain payoff, lab accidents, resonance.
const OFFENSE=new Set([0,16,19,29,34,39,40,54,59,63,65,71,72,73,74]),HEAVY=new Set([34,63,71,73,74]);
const CHAIN_STEP=30,CHAIN_MAX=4,RESONANCE_COOLDOWN=360,ACCIDENT_HP_PCT=4,ACCIDENT_STAGGER=30;
const chainPct=commits=>100+CHAIN_STEP*Math.min(commits,CHAIN_MAX);
const baseOutput=(skill,stat)=>HEAVY.has(skill)?75+Math.floor(stat*40/100):40+Math.floor(stat*22/100);
const resonanceOutput=(power,physical)=>70+Math.floor(Math.max(power,physical)/2);
const accidentLoss=maxHp=>Math.floor(maxHp*ACCIDENT_HP_PCT/100);
function validateVectors(raw){let count=0;for(const row of raw.trim().split('\n')){if(!row)continue;const [w,a,score,power]=row.split('|');const actual=instability(w.split(',').map(Number),a.split(',').map(Number));if(actual!==(score==='invalid'?null:Number(score))||strength(w.split(',').map(Number),a.split(',').map(Number))!==Number(power))throw Error('Native charge mismatch: '+row);count++;}return count;}
function simulate(rank,seed=1){
 rank=Math.max(0,Math.min(7,rank));let rng=BigInt(seed)||1n;const random=n=>{rng=BigInt.asUintN(64,rng^(rng<<13n));rng=BigInt.asUintN(64,rng^(rng>>7n));rng=BigInt.asUintN(64,rng^(rng<<17n));return Number(rng%BigInt(n));};
 const pool=new Pool(),rows=[];let tick=0,casts=0,failures=0,accidents=0,spent=0;
 for(let k=0;k<75;k++){
  const s=D.skills[k],j=[40,30,22,16,10,6,3,1][rank];let a=s.charge.map(w=>Math.max(1,Math.floor((w*(100+random(j*2+1)-j)+50)/100)));
  if(random(100)<[15,25,40,55,70,85,96,100][rank])a=s.charge.slice();const n=a.reduce((v,x)=>v+x,0)*100;
  while(!pool.reserve(n)){pool.regen(true);tick++;}
  let error=false;for(const t of s.tokens){if(random(10000)<[1200,800,600,400,200,100,30,10][rank])error=true;for(let i=0;i<10;i++){pool.regen(true);tick++;}}
  for(let i=0;i<8;i++){pool.regen(true);tick++;}pool.commit(n);spent+=n;
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
function selfTest(){for(const [a,n]of[[[4,5,3],0],[[5,6,4],7],[[4,8,3],28],[[4,12,3],78],[[8,10,6],60]])if(instability([4,5,3],a)!==n)throw Error('charge vector');if(instability([5,5,4],[5,8,4])!==32)throw Error('gamma');if(D.skills.length!==75||D.combos.length!==50)throw Error('catalogue');const p=new Pool();p.reserve(9000);for(let i=0;i<600;i++)p.regen(false);if(p.free+p.reserved!==10000)throw Error('reservation inflation');p.commit(1000);p.refund(8000);if(p.free!==9000)throw Error('refund');for(let r=0;r<8;r++){const a=simulate(r,42),b=simulate(r,42);if(JSON.stringify(a)!==JSON.stringify(b)||a.casts+a.failures!==75)throw Error('determinism');}
 // Same values the native unit test asserts (chain_percentages_are_bounded).
 if([0,1,2,3,4,5,6].map(chainPct).join()!=='100,130,160,190,220,220,220'||baseOutput(0,400)!==128||baseOutput(73,400)!==235||resonanceOutput(400,300)!==270||accidentLoss(1000)!==40)throw Error('round 125 parity');
 const q=new Pool();q.free=0;for(let i=0;i<60;i++)q.regen(true);if(q.free!==700)throw Error('combat regeneration');
 const lo=[0,1,2].map(i=>simulate(0,42+i)),hi=[0,1,2].map(i=>simulate(7,42+i)),sum=(x,k)=>x.reduce((n,v)=>n+v[k],0);
 if(!(sum(hi,'chains')>sum(lo,'chains')&&sum(lo,'accidents')>sum(hi,'accidents')))throw Error('mastery difficulty curve');
 return true;}
const api={data:D,ranks,instability,strength,state,Pool,validateVectors,simulate,selfTest,chainPct,baseOutput,resonanceOutput,accidentLoss,OFFENSE,HEAVY,RESONANCE_COOLDOWN,ACCIDENT_STAGGER};if(typeof module!=='undefined'){module.exports=api;return;}root.ScienceLab=api;
if(!document.getElementById('scienceLab'))return;
const $=s=>document.querySelector(s), el=(tag,text,cls)=>{const n=document.createElement(tag);if(text!==undefined)n.textContent=text;if(cls)n.className=cls;return n;};
let selected=0,science=0,charges=D.skills[0].charge.slice(),plan=[],run=null,frame=0,animationFrame=null,visible=true;
function catalogue(){const q=$('#search').value.toLowerCase(),role=$('#role').value;$('#catalogue').replaceChildren();let count=0;D.skills.forEach((s,i)=>{if(s.science!==science||(role&&s.role!==role)||!`${s.id} ${s.name} ${s.tokens.join(' ')}`.toLowerCase().includes(q))return;const b=el('button',undefined,'skill-card'+(i===selected?' selected':''));b.append(el('span',s.id,'skill-id'),el('strong',s.name),el('span',s.role+' · '+s.cost+' CU · '+s.cooldown/60+'s','skill-meta'));b.onclick=()=>choose(i);$('#catalogue').append(b);count++;});$('#count').textContent=count+' / 25 skills';}
function choose(i){selected=i;const s=D.skills[i];science=s.science;charges=s.charge.slice();document.documentElement.dataset.science=science;document.querySelectorAll('.scientist').forEach((b,j)=>b.classList.toggle('active',j===science));$('#skillId').textContent=s.id+' / '+['RELATIVITY','MECHANICS','CHEMISTRY'][science];$('#skillName').textContent=s.name;$('#skillEffect').textContent=s.effect;$('#tokens').replaceChildren();s.tokens.forEach((t,j)=>{const label=el('label',undefined,'token');label.append(el('span',t));const inp=el('input');inp.type='range';inp.min='1';inp.max='20';inp.value=charges[j];inp.setAttribute('aria-label',t+' charge');const output=el('output',charges[j]+' CU');label.append(inp,output,el('small','Preferred '+s.charge[j]+' CU'));inp.oninput=()=>{charges[j]=Number(inp.value);output.textContent=inp.value+' CU';update();};$('#tokens').append(label);});$('#recipe').textContent=s.tokens.join(' → ');catalogue();update();document.dispatchEvent(new CustomEvent('science-skill-selected',{detail:{skill:i}}));}
function update(){const s=D.skills[selected],score=instability(s.charge,charges),status=state(s.charge,charges),total=charges.reduce((a,x)=>a+x,0);$('#stability').dataset.state=status;$('#status').textContent=status.toUpperCase();$('#score').textContent=(score===null?'—':score)+' / 100';$('#charge').textContent=total+' CU';$('#strength').textContent=strength(s.charge,charges)+'%';$('#stabilityBar').value=score??100;$('#activation').textContent=status==='critical'?'Commit fails; dependent stages cancel.':status==='undercharged'?'At least 75% of preferred charge is required.':status==='unstable'?'Large trajectory drift; original damage limits still apply.':status==='strained'?'Small predictable drift.': 'Recipe is ready. Strength remains capped at 125%.';}
function renderPlan(){const list=$('#plan');list.replaceChildren();let total=0;plan.forEach((p,i)=>{const s=D.skills[p.skill],n=p.charge.reduce((a,x)=>a+x,0);total+=n;const row=el('div',undefined,'plan-row');row.append(el('span',String(i+1),'step-num'),el('strong',s.id+' '+s.name),el('span',n+' CU · '+state(s.charge,p.charge),'plan-meta'));const b=el('button','×','remove');b.setAttribute('aria-label','Remove '+s.name);b.onclick=()=>{plan.splice(i,1);renderPlan();};row.append(b);list.append(row);});if(!plan.length)list.append(el('p','Add a skill or load an experiment recipe.','muted'));$('#planCost').textContent=total+' / 100 CU';$('#reserve').disabled=!plan.length||total>100;$('#planHint').textContent=total>100?'Reservation exceeds capacity. Reduce allocations or split the experiment.':'Preparation such as crystals, anchors and isotope labels may need earlier casts. Their costs are additional.';}
function start(){const stages=plan.length?plan.map(p=>({...p,charge:p.charge.slice()})):[{skill:selected,charge:charges.slice()}];const total=stages.reduce((n,p)=>n+p.charge.reduce((a,x)=>a+x,0),0);if(total>100){$('#event').textContent='More than 100 CU: reservation refused.';return;}run={stages,index:0,token:0,tick:0,packets:[],spent:0,reserved:total,events:[],failed:false,commits:0,mask:0,flash:null};$('#event').textContent='Charge reserved. Notebook preparing…';}
function compare(){const body=$('#comparison');body.replaceChildren();for(let r=0;r<8;r++){const samples=Array.from({length:10},(_,i)=>simulate(r,(Number($('#seed').value)||1)+i));const avg=k=>samples.reduce((n,s)=>n+s[k],0)/10;const row=el('tr');[ranks[r],avg('casts').toFixed(1),avg('failures').toFixed(1),avg('accidents').toFixed(1),avg('chains').toFixed(1)+' / 50','×'+(avg('payoff')/100).toFixed(2),avg('resonances').toFixed(1),(avg('ticks')/60).toFixed(1)+'s'].forEach(x=>row.append(el('td',x)));body.append(row);}}
const canvas=$('#arena'),ctx=canvas.getContext('2d'),portrait=new Image();portrait.src='science-portraits.png';
function draw(){
 animationFrame=null;if(!visible)return;
 frame++;ctx.fillStyle='#0c1726';ctx.fillRect(0,0,canvas.width,canvas.height);ctx.strokeStyle='#1e3249';ctx.lineWidth=1;for(let x=20;x<canvas.width;x+=30){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,canvas.height);ctx.stroke();}for(let y=20;y<canvas.height;y+=30){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(canvas.width,y);ctx.stroke();}
 ctx.strokeStyle=['#63bfff','#ffc96b','#63e1bd'][science];ctx.beginPath();ctx.ellipse(160,190,80,40,0,0,Math.PI*2);ctx.stroke();ctx.fillStyle='#e7eef3';ctx.font='12px monospace';ctx.fillText('SCHEMATIC TRAJECTORY',20,26);ctx.fillStyle='#658299';ctx.fillText('Combat preview • charge math is exact',20,44);
 if(portrait.complete){ctx.imageSmoothingEnabled=false;ctx.drawImage(portrait,science*320+64,42,192,256,130,100,60,80);}
 for(let i=0;i<3;i++){const x=400+i*60,y=120+i*45;ctx.fillStyle='#39465b';ctx.fillRect(x-10,y-15,20,30);ctx.strokeStyle='#ff8e9e';ctx.strokeRect(x-10,y-15,20,30);}
 if(run){run.tick++;const p=run.stages[run.index];if(p&&!run.failed){const s=D.skills[p.skill],duration=6+(s.tokens.length-1)*10+12;const token=Math.min(s.tokens.length,Math.floor(Math.max(0,run.tick-6)/10));ctx.fillStyle=['#63bfff','#ffc96b','#63e1bd'][s.science];ctx.fillText(s.id+' '+s.tokens.slice(0,token).join(' → '),20,canvas.height-32);if(run.tick>=duration){const status=state(s.charge,p.charge);const n=p.charge.reduce((a,x)=>a+x,0);run.spent+=n;run.reserved-=n;const failed=status==='critical'||status==='undercharged';const chain=OFFENSE.has(p.skill)&&run.commits?' · Chain ×'+(chainPct(run.commits)/100).toFixed(1):'';run.events.push(s.id+' '+status+chain);if(failed){run.failed=true;run.reserved=0;if(status==='critical'){run.flash={kind:'accident',science:s.science,until:frame+40};$('#event').textContent='LAB ACCIDENT: '+['chalk explodes and his watch cracks','the apple lands on his head','the vial boils over'][s.science]+' · '+ACCIDENT_HP_PCT+'% max HP and a '+(ACCIDENT_STAGGER/60).toFixed(1)+' s stagger. Later reservations refunded.';}else $('#event').textContent='Cast failed; later reservations refunded.';}else{run.commits++;run.mask|=1<<s.science;const parts=p.skill===72?3:p.skill===74?3:1;if(['Attack','Heavy','Field'].includes(s.role)){for(let i=0;i<parts;i++)run.packets.push({x:170,y:165,age:0,curve:p.skill===40||p.skill===11?1:0,delay:i*20,color:['#63bfff','#ffc96b','#63e1bd'][s.science],width:2+run.commits});}run.index++;run.tick=0;if(run.index>=run.stages.length&&run.mask===7){run.flash={kind:'unified',until:frame+60};run.events.push('UNIFIED RESONANCE');}$('#event').textContent=run.events.join(' · ');}}}
  run.packets.forEach(q=>{q.age++;if(q.age<q.delay)return;q.x+=3;q.y+=q.curve*Math.sin(q.age/18)*1.3;ctx.strokeStyle=q.color;ctx.lineWidth=q.width||3;ctx.beginPath();ctx.moveTo(q.x-16,q.y);ctx.lineTo(q.x,q.y);ctx.stroke();});run.packets=run.packets.filter(q=>q.x<canvas.width&&q.age<200);ctx.fillStyle='#9cbbce';ctx.fillText('Spent '+run.spent+' CU / Reserved '+run.reserved+' CU',20,canvas.height-12);
  if(run.flash&&frame<run.flash.until){const k=(run.flash.until-frame)/60;if(run.flash.kind==='unified'){['#63bfff','#ffc96b','#63e1bd'].forEach((c,i)=>{ctx.strokeStyle=c;ctx.lineWidth=3;ctx.beginPath();ctx.moveTo(130+i*30,140+i*10);ctx.lineTo(470,140);ctx.stroke();ctx.beginPath();ctx.arc(470,140,20+(1-k)*50+i*6,0,Math.PI*2);ctx.stroke();});ctx.fillStyle='#ffffff';ctx.font='bold 16px monospace';ctx.fillText('UNIFIED RESONANCE',380,90);ctx.font='12px monospace';}else{ctx.fillStyle='rgba(255,120,110,'+Math.min(1,k*1.5)+')';ctx.font='bold 16px monospace';ctx.fillText('LAB ACCIDENT',110,92);ctx.font='12px monospace';ctx.strokeStyle='#ff8e9e';ctx.beginPath();ctx.arc(160,130,18+(1-k)*20,0,Math.PI*2);ctx.stroke();}}
 }
 animationFrame=requestAnimationFrame(draw);
}
document.querySelectorAll('.scientist').forEach((b,i)=>b.onclick=()=>choose(i*25));$('#search').oninput=catalogue;$('#role').onchange=catalogue;$('#reset').onclick=()=>choose(selected);$('#preview').onclick=start;$('#reserve').onclick=start;$('#cancel').onclick=()=>{if(run){const n=run.reserved;run.reserved=0;run.failed=true;$('#event').textContent=n+' CU refunded; committed stages remain spent.';}};$('#add').onclick=()=>{if(plan.length<5){plan.push({skill:selected,charge:charges.slice()});renderPlan();}};$('#clear').onclick=()=>{plan=[];renderPlan();};$('#compare').onclick=compare;
D.combos.forEach((c,i)=>{const o=el('option',String(i+1).padStart(2,'0')+' / '+c.name);o.value=i;$('#combo').append(o);});$('#load').onclick=()=>{plan=D.combos[Number($('#combo').value)].steps.map(skill=>({skill,charge:D.skills[skill].charge.slice()}));renderPlan();};
$('#formulaToggle').onclick=()=>{$('#formula').hidden=!$('#formula').hidden;};
try{selfTest();fetch('/science-vectors.txt').then(r=>{if(!r.ok)throw Error('Native vectors unavailable');return r.text();}).then(raw=>{$('#parity').textContent=validateVectors(raw)+' NATIVE CHARGE VECTORS VERIFIED';}).catch(e=>{$('#parity').textContent='CHARGE SELF-TEST PASSED / '+e.message;});}catch(e){$('#parity').textContent='CHECK FAILED: '+e.message;}
api.choose=i=>{if(Number.isInteger(i)&&i>=0&&i<D.skills.length&&i!==selected)choose(i);};
Object.defineProperty(api,'selected',{get:()=>selected});
Object.defineProperty(api,'animationRunning',{get:()=>visible&&animationFrame!==null});
api.setVisible=value=>{visible=Boolean(value);if(!visible&&animationFrame!==null){cancelAnimationFrame(animationFrame);animationFrame=null;}else if(visible&&animationFrame===null)draw();};
if(!new URLSearchParams(location.search).has('embed'))document.addEventListener('visibilitychange',()=>api.setVisible(!document.hidden));
choose(0);renderPlan();compare();draw();
})(typeof globalThis!=='undefined'?globalThis:this);
