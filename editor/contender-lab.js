/* Contender Lab: the shared Rust core (contender.wasm) on a browser-modelled training floor. */
(async()=>{
'use strict';
const $=id=>document.getElementById(id),R=CONTENDER_RULES,canvas=$('arena'),ctx=canvas.getContext('2d');
const ex=(await WebAssembly.instantiate(await(await fetch('contender.wasm')).arrayBuffer())).instance.exports;
const C={x:480000,y:480000},SPAN=280000;
let tick,me,units,shadow,aim,running=false,held=new Set(),glides=new Map(),fx=[],notes=[],hits=[],state={},nextBasic=0,flags={};
const P=p=>({x:(p.x-C.x)*900/SPAN+450,y:(p.y-C.y)*900/SPAN+450}),W=n=>n*900/SPAN,dist=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y);
const log=s=>{notes.unshift(tick+': '+s);notes.length=Math.min(30,notes.length);$('log').textContent=notes.join('\n');};
const actor=id=>id===1?me:id===99?shadow:units.find(u=>u.id===id);
function core(frame){const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame}));const p=ex.contender_input(bytes.length);new Uint8Array(ex.memory.buffer,p,bytes.length).set(bytes);const out=ex.contender_step(bytes.length);const r=JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer,out,ex.contender_output_len())));if(r.error)throw Error(r.error);return r;}
function unit(id,team,x,y,hp,champion,name,extra={}){return {id,team,pos:{x,y},hp,max_hp:hp,radius:champion?3000:2500,champion,tower:false,visible:true,targetable:true,farmable:!champion,name,cc:0,...extra};}
function reset(){
 ex.contender_reset();tick=1;flags={};fx=[];hits=[];glides.clear();shadow=null;nextBasic=0;notes=[];
 me={id:1,team:0,pos:{x:400000,y:480000},hp:R.stat.hp+R.growth.hp*8,max_hp:R.stat.hp+R.growth.hp*8,attack:Math.floor((R.stat.attack+R.growth.attack*8)*(100+R.modPower[0])/100)};
 units=[unit(10,1,470000,470000,1800,true,'Dummy A'),unit(11,1,480000,492000,1800,true,'Dummy B'),unit(12,1,560000,430000,1800,true,'Dummy C'),
  unit(2,0,390000,500000,1800,true,'Ally'),unit(30,2,420000,580000,2400,false,'Camp'),unit(31,2,430000,588000,900,false,'Camp pup'),
  unit(40,1,520000,560000,420,false,'Minion'),unit(41,1,526000,566000,420,false,'Minion'),unit(42,1,532000,560000,420,false,'Minion')];
 units.find(u=>u.id===2).farmable=false;aim={x:470000,y:470000};$('revision').textContent=R.revision;draw();
}
function frame(order){
 return {tick,owner:1,team:0,alive:me.hp>0,pos:round(me.pos),hp:Math.max(0,Math.round(me.hp)),max_hp:me.max_hp,attack:me.attack,stunned:false,ult_unlocked:true,
  home:{x:96000,y:864000},units:units.filter(u=>u.hp>0).map(u=>({id:u.id,team:u.team,pos:round(u.pos),hp:Math.round(u.hp),max_hp:u.max_hp,radius:u.radius,champion:u.champion,tower:false,visible:true,targetable:true,farmable:u.farmable&&u.team!==0})),
  terrain:[],profile:Array(13).fill(50),manual:!$('ai').checked,order:$('ai').checked?null:order,basic_hits:hits.splice(0),shadow:shadow&&shadow.hp>0?{id:99,pos:round(shadow.pos),hp:Math.round(shadow.hp),max_hp:shadow.max_hp}:null};
}
const round=p=>({x:Math.round(p.x),y:Math.round(p.y)});
function glide(id,to,ticks){const u=actor(id);if(u)glides.set(id,{from:{...u.pos},to,start:tick,ticks:Math.max(1,ticks)});}
function apply(e){
 const u=actor(e.target);
 switch(e.kind){
 case 'damage':if(u){u.hp=Math.max(0,u.hp-e.amount);}break;
 case 'stun':case 'knock_up':if(u){u.cc=Math.max(u.cc,tick+e.ticks);u.ccKind=e.kind;}break;
 case 'slow':if(u)u.slowUntil=tick+e.ticks;break;
 case 'knock':glide(e.target,e.to,e.ticks);break;
 case 'glide':glide(1,e.to,e.ticks);break;
 case 'teleport':me.pos={...e.to};glides.delete(1);break;
 case 'hold':flags.hold=tick+e.ticks;break;
 case 'vanish':flags.vanish=tick+e.ticks;break;
 case 'invulnerable':flags.inv=tick+e.ticks;break;
 case 'spawn_shadow':shadow={id:99,pos:{...e.at},hp:e.hp,max_hp:e.hp};break;
 case 'move_shadow':if(shadow)glide(99,e.to,e.ticks);break;
 case 'despawn_shadow':shadow=null;glides.delete(99);break;
 case 'self_damage':me.hp=Math.max(0,me.hp-e.amount);log('relay: '+e.amount+' to his body');break;
 case 'rhythm':flags.rhythm=e;break;
 case 'charges':flags.charges=e.count;break;
 case 'steer':flags.walk=e.to;break;
 case 'fx':fx.push({name:e.name,at:e.at,until:tick+Math.min(40,e.ticks)});break;
 }
}
function stepWorld(order){
 const r=core(frame(order));state=r.state;
 if(r.rejection)log('rejected: '+r.rejection);if(order&&!r.rejection)log('order: '+order.kind);
 for(const e of r.effects)apply(e);
 for(const [id,g] of glides){const u=actor(id);if(!u){glides.delete(id);continue;}const k=Math.min(1,(tick-g.start+1)/g.ticks);u.pos={x:g.from.x+(g.to.x-g.from.x)*k,y:g.from.y+(g.to.y-g.from.y)*k};if(k>=1)glides.delete(id);}
 const free=!(tick<(flags.hold||0)||tick<(flags.vanish||0));
 const speed=11*(1+((flags.rhythm?.move_permille)||0)/1000)*60;
 if(free&&!glides.has(1)){let dx=0,dy=0;if(held.has('a'))dx--;if(held.has('d'))dx++;if(held.has('w'))dy--;if(held.has('s'))dy++;
  if($('ai').checked&&flags.walk&&dist(flags.walk,me.pos)>20000){const l=dist(flags.walk,me.pos);dx=(flags.walk.x-me.pos.x)/l;dy=(flags.walk.y-me.pos.y)/l;}
  const l=Math.hypot(dx,dy);if(l){me.pos={x:me.pos.x+dx/l*speed/60*100,y:me.pos.y+dy/l*speed/60*100};}}
 // his Jab / Cross (the engine's basic in the game)
 const auto=$('ai').checked||held.has(' ');
 if(free&&auto&&tick>=nextBasic){const t=units.filter(u=>u.team!==0&&u.hp>0&&dist(u.pos,me.pos)<=R.basic.range+u.radius).sort((a,b)=>(b.champion-a.champion)||dist(a.pos,me.pos)-dist(b.pos,me.pos))[0];
  if(t){const stacks=(state.stacks||[]).find(s=>s.id===t.id&&tick<s.until)?.count||0;t.hp=Math.max(0,t.hp-Math.round(me.attack*(1000+stacks*R.rhythm.vulnerabilityPermillePerStack)/1000));hits.push(t.id);nextBasic=tick+Math.round(R.basic.cooldownTicks*100/(100+((flags.rhythm?.attack_speed)||0)));fx.push({name:'jab',at:{...t.pos},until:tick+8});}}
 // dummies hit back (one punch a second at whatever of his is nearest)
 if($('fight').checked&&tick%60===0)for(const u of units.filter(u=>u.team===1&&u.champion&&u.hp>0&&tick>=u.cc)){
  const targets=[me,shadow].filter(x=>x&&x.hp>0);const t=targets.sort((a,b)=>dist(a.pos,u.pos)-dist(b.pos,u.pos))[0];
  if(t&&dist(t.pos,u.pos)<30000){if(t===me&&(tick<(flags.hold||0)||tick<(flags.vanish||0)||tick<(flags.inv||0)))continue;t.hp=Math.max(0,t.hp-60);}}
 for(const u of units)if(u.hp<=0&&!u.dead){u.dead=true;log(u.name+' falls');}
 fx=fx.filter(f=>tick<f.until);tick++;
}
function draw(){
 ctx.clearRect(0,0,900,900);ctx.strokeStyle='#2b3a4d';for(let i=0;i<=14;i++){ctx.beginPath();ctx.moveTo(i*900/14,0);ctx.lineTo(i*900/14,900);ctx.moveTo(0,i*900/14);ctx.lineTo(900,i*900/14);ctx.stroke();}
 for(const f of fx){const p=P(f.at);ctx.strokeStyle=f.name==='ct_cleave'?'#b48cff':f.name==='ct_flurry'?'#ff8fb1':f.name==='ct_echo'?'#7fd1ff':'#ffd27f';ctx.beginPath();ctx.arc(p.x,p.y,f.name==='ct_cleave'||f.name==='ct_shadow'?W(R.cyclone.radius):W(6000),0,7);ctx.stroke();}
 const body=(u,color,label)=>{const p=P(u.pos);ctx.fillStyle=color;ctx.beginPath();ctx.arc(p.x,p.y,Math.max(5,W(u.radius||3000)),0,7);ctx.fill();ctx.fillStyle='#fff';ctx.fillText(label,p.x-18,p.y-14);
  ctx.fillStyle='#400';ctx.fillRect(p.x-18,p.y+10,36,4);ctx.fillStyle='#4f4';ctx.fillRect(p.x-18,p.y+10,36*Math.max(0,u.hp)/u.max_hp,4);
  if(tick<u.cc){ctx.fillStyle='#ffe066';ctx.fillText(u.ccKind==='knock_up'?'↑':'★',p.x+10,p.y-4);}};
 for(const u of units.filter(u=>u.hp>0)){const s=(state.stacks||[]).find(s=>s.id===u.id&&tick<s.until);body(u,u.team===0?'#4aa3ff':u.champion?'#e2574c':'#c08a3e',u.name+(s?' ×'+s.count:''));}
 if(shadow&&shadow.hp>0)body(shadow,'rgba(120,90,200,0.7)','Shadow');
 const hidden=tick<(flags.vanish||0);body(me,hidden?'rgba(255,255,255,0.15)':tick<(flags.hold||0)?'#8a7fb0':'#f2f2f2','Contender');
 const a=P(aim);ctx.strokeStyle='#fff';ctx.beginPath();ctx.moveTo(a.x-6,a.y);ctx.lineTo(a.x+6,a.y);ctx.moveTo(a.x,a.y-6);ctx.lineTo(a.x,a.y+6);ctx.stroke();
 $('pips').textContent='◆'.repeat(state.charges??5)+'◇'.repeat(R.charges.max-(state.charges??5));
 const ult=Math.max(0,((state.ult_at||0)-tick)/60);
 $('state').textContent=`HP ${Math.round(me.hp)}/${me.max_hp}  ult ${ult.toFixed(1)} s\nS1 ${(Math.max(0,(state.cd?.[0]||0)-tick)/60).toFixed(1)} s  S2 ${(Math.max(0,(state.cd?.[1]||0)-tick)/60).toFixed(1)} s  S3 ${(Math.max(0,(state.cd?.[2]||0)-tick)/60).toFixed(1)} s\nprojected: ${!!state.proj}  flurry: ${!!state.flurry}  echoing flurry: ${!!state.flurry?.echo}\nrhythm stacks (focus): ${state.focus?state.focus[1]:0}\nAI: ${state.reason||'-'}`;
}
const orderFor=k=>({q:{kind:'s1',aim:round(aim)},e:{kind:'s2',aim:round(aim)},c:{kind:'s3',aim:round(aim)},r:{kind:'ult'},f:{kind:'end'}})[k];
let queued=null;
function press(k){const o=orderFor(k);if(o)queued=o;}
function loop(){if(running){let o=queued;queued=null;if(!o&&state.proj&&['w','a','s','d'].some(k=>held.has(k))){const d={x:(held.has('d')-held.has('a'))*40000,y:(held.has('s')-held.has('w'))*40000};const sp=state.proj.pos;o={kind:'move',aim:{x:sp.x+d.x,y:sp.y+d.y}};}stepWorld(o);draw();}requestAnimationFrame(loop);}
canvas.addEventListener('click',e=>{const r=canvas.getBoundingClientRect();aim={x:(e.clientX-r.left)*900/r.width,y:(e.clientY-r.top)*900/r.height};aim={x:(aim.x-450)*SPAN/900+C.x,y:(aim.y-450)*SPAN/900+C.y};canvas.focus();draw();});
addEventListener('keydown',e=>{const k=e.key.toLowerCase();if(document.activeElement!==canvas&&k!==' ')return;held.add(k);if(!e.repeat)press(k);if(k===' ')e.preventDefault();});
addEventListener('keyup',e=>held.delete(e.key.toLowerCase()));
for(const b of document.querySelectorAll('[data-key]'))b.onclick=()=>{press(b.dataset.key);if(!running){stepWorld(queued);queued=null;draw();}};
$('reset').onclick=()=>{reset();};$('play').onclick=()=>{running=!running;canvas.focus();};$('step').onclick=()=>{stepWorld(queued);queued=null;draw();};
window.contenderLab={step:o=>{stepWorld(o);draw();return state;},get state(){return state},get me(){return me},get units(){return units},get shadow(){return shadow},reset};
reset();running=true;loop();
})();
