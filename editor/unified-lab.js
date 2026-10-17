/* Equation Lab: a thin browser arena around unified_core.rs compiled to WASM (unified.wasm).
   The rules are the game's; this file only moves the scripted bodies and draws. */
(async()=>{'use strict';
const EQ=window.UnifiedEquations,$=s=>document.querySelector(s),canvas=$('#arena'),ctx=canvas.getContext('2d');
const SCI=['#63bfff','#ffc96b','#63e1bd'],SCIENTIST=['Einstein','Newton','Curie'];
const RANKS=['Student','Lab Assistant','Researcher','Scientist','Professor','Fellow','Laureate','Unified Mind'];
const ME=1,VIEW={x:340000,y:280000,w:192000,h:120000};
let api,handle,tick=0,paused=false,units,target=10,aimPoint=null,pending=null,queue=[],snap=null,events=[],fx=[],acc=0,last=null;
const params=new URLSearchParams(location.search);
const sx=x=>(x-VIEW.x)*canvas.width/VIEW.w,sy=y=>(y-VIEW.y)*canvas.height/VIEW.h;
const ux=px=>VIEW.x+px*VIEW.w/canvas.width,uy=py=>VIEW.y+py*VIEW.h/canvas.height;
const log=s=>{events.unshift((tick/60).toFixed(2)+'s  '+s);events.length=Math.min(120,events.length);$('#events').textContent=events.join('\n');};
const unit=(id,team,x,y,hp)=>({id,team,x,y,hp,max:hp,ad:70,ap:120,shield:0,root:0,speedPct:0,speedUntil:0,next:40+id*7});
function setup(){
  for(let r=0;r<8;r++)$('#rank').append(new Option(r+' · '+RANKS[r],r));
  $('#rank').value=params.get('rank')??7;
  EQ.signatures.forEach((g,i)=>$('#sigpick').append(new Option(g.name+'  ('+g.steps.length+')',i)));
  const book=$('#book');
  for(let s of [1,2,0]){
    const box=document.createElement('div');box.className='sci';box.innerHTML=`<h3 style="color:${SCI[s]}">${SCIENTIST[s]}</h3><div class="eqs"></div>`;
    EQ.skills.forEach((k,i)=>{if(k.science!==s)return;const el=document.createElement('div');el.className='eq';el.dataset.skill=i;el.title=k.text+'\nTrait when reserved: '+EQ.traits[i].name+' — '+EQ.traits[i].hit;
      el.innerHTML=`<b style="color:${SCI[s]}">${k.name}</b><code>${k.equation}</code><div class="row"><button data-act="reserve" data-skill="${i}">Reserve</button><button data-act="cast" data-skill="${i}">Cast</button></div><div class="cd"></div>`;box.querySelector('.eqs').append(el);});
    book.append(box);
  }
  book.addEventListener('click',e=>{const b=e.target.closest('button[data-act]');if(!b)return;act(b.dataset.act==='cast'?2:1,Number(b.dataset.skill));});
  for(let i=0;i<5;i++){const p=document.createElement('div');p.className='pip';$('#pips').append(p);}
  canvas.addEventListener('click',e=>{const r=canvas.getBoundingClientRect(),px=(e.clientX-r.left)*canvas.width/r.width,py=(e.clientY-r.top)*canvas.height/r.height;
    if(e.shiftKey){aimPoint=[Math.round(ux(px)),Math.round(uy(py))];log('Aim point set');return;}
    const hit=units.filter(u=>u.hp>0&&u.team===1).find(u=>Math.hypot(sx(u.x)-px,sy(u.y)-py)<18);if(hit){target=hit.id;aimPoint=null;log('Target '+hit.id);}});
  $('#ult').onclick=()=>act(3,0);
  $('#sigplay').onclick=()=>{const g=EQ.signatures[Number($('#sigpick').value)];queue=g.steps.map((k,i)=>k==='ult'?[3,0]:[i+1===g.steps.length?2:1,EQ.skills.findIndex(s=>s.key===k)]);log('Playing '+g.name);};
  $('#pause').onclick=()=>{paused=!paused;$('#pause').textContent=paused?'Resume':'Pause';};
  $('#step').onclick=()=>{paused=true;$('#pause').textContent='Resume';run();draw();};
  $('#reset').onclick=reset;$('#rank').onchange=reset;
}
function aimFor(skill){
  const shape=skill==null?'line':EQ.skills[skill].shape;
  if(shape==='ally')return [0,ME,0];
  const t=units.find(u=>u.id===target&&u.hp>0)||units.find(u=>u.team===1&&u.hp>0);
  if(['zone','lob','planet'].includes(shape))return aimPoint?[1,...aimPoint]:[1,t?t.x:440000,t?t.y:400000];
  return [0,t?t.id:10,0];
}
function act(kind,skill){pending=[kind,skill];}
function reset(){
  if(handle!==undefined)api.unified_drop(handle);
  handle=api.unified_new(1234,ME,Number($('#rank').value));tick=0;events=[];fx=[];queue=[];pending=null;snap=null;
  units=[unit(ME,0,400000,350000,2600),unit(2,0,392000,365000,2600),unit(10,1,470000,342000,2600),unit(11,1,478000,356000,3000),unit(12,1,486000,330000,3400)];
  units[0].ap=140;target=10;aimPoint=null;log('Arena reset · rank '+RANKS[Number($('#rank').value)]);run();draw();
}
function frame(action){
  const [kind,skill]=action||[0,0];const aim=aimFor(kind===3?null:skill);
  const fighters=units.filter(u=>u.hp>0).flatMap(u=>[u.id,u.team,Math.round(u.x),Math.round(u.y),Math.round(u.hp),u.max,u.ad,u.ap,1,1,1,0]);
  return [tick,ME,+$('#auto').checked,+$('#unlock').checked,0,kind,skill,aim[0],aim[1],aim[2],fighters.length/12,...fighters,0];
}
function hurt(u,n){if(!u||u.hp<=0)return;const b=Math.min(u.shield,n);u.shield-=b;u.hp=Math.max(0,u.hp-(n-b));}
function apply(cmd){
  const [k,a,b,c]=cmd;const u=units.find(x=>x.id===a);
  if(k==='damage'){hurt(u,b);fx.push({kind:'num',x:u?.x,y:u?.y,text:String(b),col:c==='True'?'#fff':c==='Phys'?'#ffc96b':'#9fd0ff',until:tick+40});}
  else if(k==='heal'&&u){u.hp=Math.min(u.max,u.hp+b);log('Heal '+a+' +'+b);}
  else if(k==='shield'&&u){u.shield+=b;}
  else if(k==='root'&&u){u.root=Math.max(u.root,tick+b);}
  else if(k==='speed'&&u){u.speedPct=b;u.speedUntil=tick+c;}
  else if(k==='pos'&&u){u.x=b;u.y=c;}
  else if(k==='fx'){const [_,tag,x,y,unitId,ticks]=cmd;fxTag(tag,x,y,unitId,ticks);}
  else if(k==='log'){log(a.replace(/^tick \d+ /,''));}
}
function fxTag(tag,x,y,unitId,ticks){
  let m;
  if((m=tag.match(/^eq(\d+)$/))){const s=EQ.skills[+m[1]];fx.push({kind:'label',line:0,text:s.equation,col:SCI[s.science],until:tick+ticks});}
  else if((m=tag.match(/^sig(\d+)$/))){fx.push({kind:'label',line:2,text:'★ '+EQ.signatures[+m[1]].name,col:'#ffd678',until:tick+ticks});log('SIGNATURE · '+EQ.signatures[+m[1]].name+' — '+EQ.signatures[+m[1]].text);}
  else if((m=tag.match(/^uni(\d+)$/))){const u=EQ.unification[+m[1]];fx.push({kind:'label',line:3,text:u.name,col:'#ffffff',until:tick+ticks});log('UNIFICATION · '+u.name+' — '+u.text);}
  else if((m=tag.match(/^n(\d)_(\d)$/))){const old=fx.find(f=>f.kind==='label'&&f.line===1&&f.until>tick);if(old&&+m[1]>0)old.text+=m[2];else fx.push({kind:'label',line:1,text:'= '+m[2],col:'#ffd678',until:tick+ticks});}
  else if(tag==='insight'){fx.push({kind:'label',line:4,text:'Insight +15%',col:'#ffd678',until:tick+20});}
  else if(tag==='fizzle'){log('Reserve fizzled: charges stay spent');}
  else if((m=tag.match(/^im(\d+)$/))&&unitId<0){fx.push({kind:'ring',x,y,col:SCI[EQ.skills[+m[1]].science],until:tick+18,r0:6});}
  else if(tag==='ultwarn'){fx.push({kind:'ring',x,y,col:'#ffffff',until:tick+ticks,r0:30,shrink:true});log('Theory of Everything: warning');}
  else if(tag==='twin'){fx.push({kind:'ring',x,y,col:SCI[0],until:tick+ticks,r0:10});}
}
function run(){
  tick++;
  let action=null;
  if(pending){action=pending;pending=null;}
  else if(queue.length&&tick%8===0){action=queue.shift();}
  // Scripted bodies.
  const t0=units.filter(u=>u.team===0&&u.hp>0),t1=units.filter(u=>u.team===1&&u.hp>0);
  for(const u of t1){
    if(!t0.length)break;const tgt=t0.reduce((a,b)=>Math.hypot(a.x-u.x,a.y-u.y)<Math.hypot(b.x-u.x,b.y-u.y)?a:b);
    const d=Math.hypot(tgt.x-u.x,tgt.y-u.y),v=700*(100+(tick<u.speedUntil?u.speedPct:0))/100;
    if($('#moving').checked&&tick>=u.root){if(d>17000){u.x+=(tgt.x-u.x)*v/d;u.y+=(tgt.y-u.y)*v/d;}else{const side=(Math.floor(tick/50)+u.id)%2?1:-1;u.x+=-(tgt.y-u.y)*v/d*side;u.y+=(tgt.x-u.x)*v/d*side;}}
    if($('#attacks').checked&&d<=22000&&tick>=u.next){u.next=tick+70;hurt(tgt,28);}
  }
  const a=frame(action),ptr=api.unified_input(a.length);new BigInt64Array(api.memory.buffer,ptr,a.length).set(a.map(v=>BigInt(Math.round(v))));
  const out=api.unified_step(handle,a.length);snap=JSON.parse(new TextDecoder().decode(new Uint8Array(api.memory.buffer,out,api.unified_output_len())));
  if(snap.error)throw Error(snap.error);
  if(action&&action[0]!==0){const ok=snap.commands.some(c=>c[0]==='log');if(!ok&&action[0]!==3)log('Refused: '+(action[0]===1?'reserve ':'cast ')+EQ.skills[action[1]].name+' (charge, cooldown or full reserve)');}
  for(const c of snap.commands)apply(c);
  fx=fx.filter(f=>f.until>tick);
  if(t1.length===0&&tick%60===0){log('All enemies down — reset to continue');}
}
function panel(){
  const ch=snap.charges;$('#pips').querySelectorAll('.pip').forEach((p,i)=>{const sorted=[...ch].sort((a,b)=>a-b),at=sorted[i],ready=at<=tick;
    p.className='pip'+(ready?' ready':'');p.textContent=ready?'':Math.ceil((at-tick)/60)+'s';});
  const q=snap.reserve;$('#queue').innerHTML=q.length?q.map((k,i)=>`<span class="chip2" style="border-color:${SCI[EQ.skills[k].science]}">${i+1}. ${EQ.traits[k].name} <small>(${EQ.skills[k].name})</small></span>`).join(''):'<span class="muted">empty — Cast fires a single equation</span>';
  // The name the next cast would take, and the signatures this reserve can still complete.
  const sel=document.querySelector('.eq:hover');const hover=sel?Number(sel.dataset.skill):null;
  if(q.length&&hover!=null){const counts={};q.forEach(k=>counts[k]=(counts[k]||0)+1);const adj=Object.keys(counts).map(k=>(counts[k]>1?'Doubly ':'')+EQ.traits[k].adjective);$('#next').textContent='Next cast: '+adj.join(' ')+' '+EQ.skills[hover].name+'  (+'+(25*q.length)+'% links)';}
  else $('#next').textContent=q.length?'Hover a Cast button to preview the combo name':'';
  const keys=q.map(k=>EQ.skills[k].key);const open=EQ.signatures.filter(g=>g.steps.length>keys.length&&keys.every((k,i)=>g.steps[i]===k));
  $('#sigs').innerHTML=q.length?(open.length?'Signatures still open: '+open.map(g=>`<b>${g.name}</b> → ${g.steps.slice(keys.length).map(k=>k==='ult'?'Ultimate':EQ.skills.find(s=>s.key===k).name).join(' ▸ ')}`).join('<br>'):'<span class="muted">No signature starts this way</span>'):'';
  document.querySelectorAll('.eq').forEach(el=>{const k=Number(el.dataset.skill),left=snap.cooldown[k]-tick,cd=el.querySelector('.cd');
    if(left>0){cd.style.display='block';cd.style.transform=`scaleX(${left/EQ.cooldowns[EQ.skills[k].cd]})`;}else cd.style.display='none';});
  const s=snap.stats;$('#sCasts').textContent=s.casts;$('#sCombos').textContent=s.combos;$('#sSigs').textContent=s.signatures;$('#sUni').textContent=s.unifications;$('#sIns').textContent=s.insights;$('#sMiss').textContent=s.misplays;$('#sDmg').textContent=s.damage;$('#sEff').textContent=s.measured?Math.round(s.efficiency/s.measured)+'%':'–';
  $('#reason').textContent=(snap.reason||'Ready')+' · '+RANKS[snap.rank]+(snap.ultAt>tick?' · ultimate in '+Math.ceil((snap.ultAt-tick)/60)+'s':'');
}
function draw(){
  ctx.fillStyle='#0b1422';ctx.fillRect(0,0,canvas.width,canvas.height);
  ctx.strokeStyle='#14263a';for(let x=0;x<canvas.width;x+=48){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,canvas.height);ctx.stroke();}
  if(!snap)return;
  for(const l of snap.live){
    const s=l.s===99?6:l.s,col=SCI[EQ.skills[Math.min(s,17)].science];
    if(l.k==='zone'){const r=l.r*canvas.width/VIEW.w;ctx.globalAlpha=l.open?.22:.08;ctx.fillStyle=l.s===99?'#76ffac':col;ctx.beginPath();ctx.arc(sx(l.x),sy(l.y),r,0,7);ctx.fill();ctx.globalAlpha=1;ctx.strokeStyle=col;ctx.stroke();
      ctx.fillStyle=col;ctx.font='11px monospace';ctx.fillText(l.s===99?'puddle':EQ.skills[s].name,sx(l.x)-r,sy(l.y)-r-4);}
    else if(l.k==='ray'){ctx.strokeStyle=col;ctx.lineWidth=3;ctx.beginPath();ctx.moveTo(sx(l.x),sy(l.y));ctx.lineTo(sx(l.x)-l.dx*0.0025,sy(l.y)-l.dy*0.0025);ctx.stroke();ctx.lineWidth=1;}
    else if(l.k==='lob'){ctx.strokeStyle='#e23430';ctx.beginPath();ctx.arc(sx(l.x),sy(l.y),Math.max(3,l.r*canvas.width/VIEW.w*(1-l.t/40)),0,7);ctx.stroke();ctx.fillStyle='#e23430';ctx.fillRect(sx(l.x)-3,sy(l.y)-3-l.t,6,6);}
    else if(l.k==='planet'){ctx.fillStyle='#d6a05a';ctx.beginPath();ctx.arc(sx(l.x),sy(l.y),5,0,7);ctx.fill();ctx.strokeStyle='#554';ctx.beginPath();ctx.arc(sx(l.cx),sy(l.cy),l.r*canvas.width/VIEW.w,0,7);ctx.stroke();}
    else if(l.k==='reaction'){const me=units[0];ctx.strokeStyle=SCI[1];ctx.setLineDash([4,3]);ctx.beginPath();ctx.arc(sx(me.x),sy(me.y),22000*canvas.width/VIEW.w,0,7);ctx.stroke();ctx.setLineDash([]);}
  }
  for(const u of units){
    if(u.hp<=0)continue;const x=sx(u.x),y=sy(u.y);
    ctx.fillStyle=u.id===ME?SCI[snap.persona]:u.team===0?'#7aa2c8':'#e07070';ctx.beginPath();ctx.arc(x,y,u.id===ME?11:9,0,7);ctx.fill();
    if(u.id===target){ctx.strokeStyle='#fff';ctx.beginPath();ctx.arc(x,y,14,0,7);ctx.stroke();}
    ctx.fillStyle='#222';ctx.fillRect(x-16,y-22,32,4);ctx.fillStyle=u.team?'#e86':'#6c8';ctx.fillRect(x-16,y-22,32*u.hp/u.max,4);
    if(u.shield>0){ctx.fillStyle='#ddd';ctx.fillRect(x-16,y-26,Math.min(32,32*u.shield/u.max*4),2);}
    const m=snap.marks[u.id];if(m){ctx.fillStyle='#ffd678';ctx.font='10px monospace';ctx.fillText('◆'.repeat(m),x-10,y+24);}
    if(tick<u.root){ctx.strokeStyle='#e23430';ctx.strokeRect(x-12,y-12,24,24);}
  }
  const me=units[0];
  for(const f of fx){
    if(f.kind==='label'){ctx.font=f.line>=2?'bold 13px monospace':'12px monospace';ctx.fillStyle=f.col;const w=ctx.measureText(f.text).width;ctx.fillText(f.text,sx(me.x)-w/2,sy(me.y)-40-f.line*15);}
    else if(f.kind==='num'&&f.x!=null){ctx.font='11px monospace';ctx.fillStyle=f.col;ctx.fillText(f.text,sx(f.x)+10,sy(f.y)-12-(40-(f.until-tick))/2);}
    else if(f.kind==='ring'){const k=(f.until-tick)/30;ctx.strokeStyle=f.col;ctx.beginPath();ctx.arc(sx(f.x),sy(f.y),f.shrink?f.r0*k+4:f.r0+(1-k)*12,0,7);ctx.stroke();}
  }
  panel();
}
function loop(now){if(last===null)last=now;acc+=Math.min(250,now-last);last=now;while(acc>=1000/60){if(!paused)run();acc-=1000/60;}draw();requestAnimationFrame(loop);}
const wasm=await WebAssembly.instantiate(await (await fetch('unified.wasm')).arrayBuffer(),{});api=wasm.instance.exports;
setup();reset();
if(params.get('auto')==='1')$('#auto').checked=true;
window.UnifiedLab={get snapshot(){return snap;},get units(){return units;},get tick(){return tick;},act,step(){paused=true;run();draw();},reset,queue:steps=>{queue=steps;}};
requestAnimationFrame(loop);
})().catch(e=>{document.querySelector('#reason').textContent='Equation Lab failed to load: '+e.message;console.error(e);});
