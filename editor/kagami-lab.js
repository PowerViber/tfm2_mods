/* Shared Rust skills; explicitly simulated browser host. */
(async()=>{
'use strict';
const $=id=>document.getElementById(id),R=KAGAMI_RULES,canvas=$('arena'),ctx=canvas.getContext('2d'),world=new GameWorld(),painter=new KagamiPainter(),clock=new KratosClock();
const labels=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];
for(const [i,label]of labels.entries()){const l=document.createElement('label');l.textContent=label;const input=document.createElement('input');Object.assign(input,{type:'number',min:0,max:100,value:50});input.dataset.stat=i;l.append(input);$('stats').append(l);}
for(const [id,label]of Object.entries(GameScenario.choices))$('scene').add(new Option(label,id));
let ex,wasmBytes,tick=1,me,units=[],scenario,state={},pending=null,aim={x:480000,y:480000},running=false,clone=null,glides=new Map(),fx=[],threads=[],shots=[],held=new Set(),camera={x:480000,y:480000,span:400000},follow=true,recording=false,record=null,imported=null,notes=[],basic=null;
const round=p=>({x:Math.round(p.x),y:Math.round(p.y)}),distance=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y),actor=id=>id===1?me:units.find(u=>u.id===id),P=p=>({x:(p.x-camera.x)*900/camera.span+450,y:(p.y-camera.y)*900/camera.span+450}),W=n=>n*900/camera.span;
const log=s=>{notes.unshift(tick+': '+s);notes.length=Math.min(30,notes.length);$('log').textContent=notes.join('\n');};
function core(frame,exports=ex){const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame}));const p=exports.kagami_input(bytes.length);new Uint8Array(exports.memory.buffer,p,bytes.length).set(bytes);const out=exports.kagami_step(bytes.length);const r=JSON.parse(new TextDecoder().decode(new Uint8Array(exports.memory.buffer,out,exports.kagami_output_len())));if(r.error)throw Error(r.error);return r;}
function reset(){
 ex.kagami_reset();tick=8*60*60;state={};clone=null;glides.clear();fx=[];threads=[];shots=[];held.clear();basic=null;pending=null;recording=false;notes=[];
 scenario=new GameScenario(world,$('scene').value);const setup=scenario.setup();units=setup.units;
 const stats=scenario.stats(R);for(const [key,i]of [['attack',0],['magic_power',1],['hp',2],['defence',3],['magic_resistance',4]])stats[key]=Math.floor(stats[key]*(100+R.modPower[i])/100);me={id:1,team:0,pos:setup.pos,home:{...setup.pos},hp:stats.hp,max_hp:stats.hp,stats,radius:world.data.settings.champion_radius,champion:true,kind:'champion',name:'Kagami',shields:[],attack:{range:R.basic.range,cooltime:R.basic.cooldownTicks,duration:R.basic.duration,start_timing:R.basic.startTiming},nextAttack:0};
 for(const u of [me,...units]){u.baseStats={...u.stats};u.baseAttack={...u.attack};}
 aim={...units.find(u=>u.id===4).pos};camera={x:me.pos.x,y:me.pos.y,span:400000};follow=true;
 const damage=scenario.damage.bind(scenario);
 scenario.damage=(source,target,raw,t)=>{
  if(t<(target.sealUntil||0))return 0;
  if(t<(target.wardUntil||0))raw=Math.min(raw,Math.max(0,Math.floor((target.hp-1)*(100+Math.max(0,target.stats?.defence||0))/100)));
  return damage(source,target,raw,t);
 };
 const visible=scenario.calculateVisible.bind(scenario);
 scenario.calculateVisible=(u,...args)=>tick<(u.sealUntil||0)?false:visible(u,...args);
 $('description').textContent=setup.description;$('revision').textContent=R.revision;clock.reset();draw();
}
function glide(id,to,ticks){const u=actor(id);if(u)glides.set(id,{from:{...u.pos},to:world.legal(to,u.radius),start:tick,ticks:Math.max(1,ticks)});}
function apply(e){
 const u=actor(e.target);
 switch(e.type){
 case 'spawn_clone':if(!clone){clone=scenario.make(999,0,e.at,'champion','champ-priest',world.data.champions.priest,{name:'Reflection',reflection:true,hp:e.hp,max_hp:e.hp,stunUntil:Number.MAX_SAFE_INTEGER});clone.stats={...me.stats,attack:0,move_speed:0};clone.champion=false;clone.baseStats={...clone.stats};clone.baseAttack={...clone.attack};units.push(clone);}break;
 case 'clone_move':if(clone)glide(clone.id,e.to,e.ticks);break;
 case 'clone_shot':if(u){shots.push({from:e.from,to:{...u.pos},until:tick+12});scenario.damage(me,u,e.amount,tick);if(clone){clone.pose='attack';clone.poseUntil=tick+22;}}break;
 case 'heal':if(u){u.hp=Math.min(u.max_hp,u.hp+e.amount);fx.push({name:'heal',at:{...u.pos},until:tick+30,target:u.id});const caster=clone&&distance(e.from,clone.pos)<100?clone:me;caster.pose='heal';caster.poseUntil=tick+18;}break;
 case 'charm':if(u){u.charmUntil=tick+e.ticks;fx.push({name:'charm',at:{...u.pos},until:tick+24,target:u.id});}break;
 case 'swap_lock':threads.push({a:e.a,b:e.b,until:tick+e.ticks});me.pose='step';me.poseUntil=tick+18;break;
 case 'swap':{const a=actor(e.a),b=actor(e.b);if(a&&b){const p={...a.pos};a.pos=world.legal(b.pos,a.radius);b.pos=world.legal(p,b.radius);glides.delete(a.id);glides.delete(b.id);}break;}
 case 'stun':if(u)u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);break;
 case 'knockback':if(u){const d=Math.max(1,distance(me.pos,u.pos)),k=R.angel.knockSpeed*R.angel.knockTicks/d;glide(u.id,world.sweep(u.pos,{x:u.pos.x+(u.pos.x-me.pos.x)*k,y:u.pos.y+(u.pos.y-me.pos.y)*k},u.radius),R.angel.knockTicks);}break;
 case 'shield':if(u)u.shields.push({amount:e.amount,until:tick+e.ticks});break;
 case 'ward':if(u)u.wardUntil=tick+e.ticks;break;
 case 'self_ward':me.wardUntil=tick+e.ticks;break;
 case 'seal':if(u){u.sealUntil=tick+e.ticks;u.stunUntil=tick+e.ticks;u.pending=null;glides.delete(u.id);}break;
 case 'revive':if(u){u.hp=Math.max(u.hp,e.hp);u.sealUntil=0;u.stunUntil=0;if(e.blessed){u.charmUntil=tick+R.ult.reviveTicks;u.shields.push({amount:Math.floor(u.max_hp*R.ult.shieldPctMaxHp/100),until:tick+R.ult.reviveTicks});u.blessedUntil=tick+R.ult.reviveTicks;}}break;
 case 'steer':if($('ai').checked&&tick>=me.attackUntil)me.goal=e.to;break;
 case 'fx':fx.push({name:e.name,at:e.at,until:tick+e.ticks});break;
 case 'log':log(e.text);break;
 }
}
function frame(){
 return {tick,owner:1,team:0,alive:me.hp>0,pos:round(me.pos),hp:Math.round(me.hp),max_hp:me.max_hp,attack:Math.round(me.stats.attack),stunned:tick<(me.stunUntil||0),ult_unlocked:$('ultimate').checked,home:{x:96000,y:864000},
 clone:clone&&clone.hp>0?{id:clone.id,pos:round(clone.pos),hp:Math.round(clone.hp),max_hp:clone.max_hp}:null,
 units:units.filter(u=>u!==clone&&u.hp>0).map(u=>({id:u.id,team:u.team,pos:u.visible===false&&u.team!==0?{x:0,y:0}:round(u.pos),hp:Math.round(u.hp),max_hp:u.max_hp,attack:Math.round(u.stats.attack||0),champion:u.champion,tower:u.tower,visible:u.visible!==false,targetable:tick>=(u.sealUntil||0),sealed:tick<(u.sealUntil||0)})),
 profile:[...$('stats').querySelectorAll('input')].map(e=>Math.max(0,Math.min(100,Math.round(+e.value||0)))),manual:!$('ai').checked,order:pending,terrain:world.walls,hazards:[]};
}
function step(){
 if(!me)return;
 for(const u of [me,...units]){
  const b=u.baseStats||u.stats;u.stats={...b};u.attack={...(u.baseAttack||u.attack)};
  if(tick<(u.charmUntil||0)){u.stats.attack=Math.floor(b.attack*(100+R.charm.damagePct)/100);u.stats.magic_power=Math.floor((b.magic_power||0)*(100+R.charm.damagePct)/100);u.stats.move_speed=Math.floor(b.move_speed*(100+R.charm.moveSpeedPct)/100);u.attack.cooltime=Math.ceil(u.attack.cooltime/(1+R.charm.attackSpeedPct/100));}
  if(tick<(u.blessedUntil||0)){u.stats.defence=Math.floor(b.defence*(100+R.ult.defencePct)/100);}
  u.moving=false;u.shields=u.shields.filter(s=>s.until>tick&&s.amount>0);}
 for(const [id,g]of glides){const u=actor(id);if(!u||u.hp<=0){glides.delete(id);continue;}const k=Math.min(1,(tick-g.start)/g.ticks),p={x:g.from.x+(g.to.x-g.from.x)*k,y:g.from.y+(g.to.y-g.from.y)*k};u.left=p.x<u.pos.x;u.moving=distance(p,u.pos)>.01;u.pos=round(p);if(k===1)glides.delete(id);}
 if(clone&&clone.hp<=0){units=units.filter(u=>u!==clone);clone=null;}
 scenario.refreshVisibility(me,units);
 const cloneMotion=clone?.moving;
 if($('incoming').checked)scenario.advance(tick,me,units,{hitOwner:h=>scenario.damage(actor(h.from)||me,me,h.amount,tick),hit:()=>{}});
 if(clone&&cloneMotion)clone.moving=true;
 if(me.hp>0&&tick>=(me.sealUntil||0)&&tick>=(me.stunUntil||0)){
  if(!$('ai').checked){const dx=Number(held.has('d'))-Number(held.has('a')),dy=Number(held.has('s'))-Number(held.has('w'));if(dx||dy){const n=Math.hypot(dx,dy);me.goal=null;me.moving=world.move(me,{x:me.pos.x+dx/n*10000,y:me.pos.y+dy/n*10000},me.stats.move_speed);}}
  if(me.goal&&!glides.has(1)&&tick>=me.attackUntil)me.moving=world.move(me,me.goal,me.stats.move_speed);
 }
 if(basic&&tick>=basic.at){const u=actor(basic.target);if(u&&u.hp>0&&tick>=(u.sealUntil||0)){scenario.damage(me,u,me.stats.attack,tick);shots.push({from:{...me.pos},to:{...u.pos},until:tick+12});}basic=null;}
 const f=frame(),r=core(f);pending=null;state=r.state;
 
 if(recording&&record.frames.length<3600)record.frames.push({frame:f,result:r});
 for(const e of r.effects)apply(e);
 if(r.rejection)log('Refused: '+r.rejection);
 fx=fx.filter(f=>tick<f.until);threads=threads.filter(f=>tick<f.until);shots=shots.filter(f=>tick<f.until);tick++;draw();
}
function nearest(list,at=aim){return list.filter(u=>u.hp>0).sort((a,b)=>distance(a.pos,at)-distance(b.pos,at)||a.id-b.id)[0];}
function order(kind){
 const ally=nearest([me,...units.filter(u=>u.champion&&u.team===0)]);
 if(kind==='step')pending=ally&&ally.id!==1&&distance(ally.pos,aim)<20000?{kind:'step_ally',aim:ally.id}:{kind:'step_point',aim:round(aim)};
 else pending={kind,aim:ally?.id||1};
 if(!running)step();
}
function attack(){
 if(tick<me.nextAttack||tick<(me.sealUntil||0)||tick<(me.stunUntil||0))return;
 const u=nearest(units.filter(u=>u.team!==0&&u.visible!==false&&tick>=(u.sealUntil||0)));
 if(u&&distance(u.pos,me.pos)<=R.basic.range+u.radius+me.radius&&world.clear(me.pos,u.pos)){
  me.nextAttack=tick+me.attack.cooltime;me.attackUntil=tick+R.basic.duration;me.pose='attack';me.poseUntil=me.attackUntil;me.left=u.pos.x<me.pos.x;basic={target:u.id,at:tick+R.basic.startTiming};
 }else log('Basic needs a visible target in reach');
}
function ring(p,r,color){const a=P(p);ctx.strokeStyle=color;ctx.beginPath();ctx.arc(a.x,a.y,W(r),0,Math.PI*2);ctx.stroke();}
function draw(){
 if(!me)return;if(follow){camera.x=me.pos.x;camera.y=me.pos.y;}ctx.clearRect(0,0,900,900);world.drawMap(ctx,P,W);
 if($('walls').checked){ctx.fillStyle='#f9484860';world.walls.forEach((b,i)=>{if(b){const p=P({x:i%30*32000,y:Math.floor(i/30)*32000});ctx.fillRect(p.x,p.y,W(32000),W(32000));}});}
 ring(me.pos,R.step.range,'#b9a2e155');ring(aim,5000,'#ffeea8');
 for(const u of [...units,me].filter(u=>u.team===0||u.visible!==false).sort((a,b)=>a.pos.y-b.pos.y)){
  if(u===me||u.reflection)painter.body(ctx,u,tick,P,W);else world.drawUnit(ctx,u,tick,P,W);
  const p=P(u.pos),w=Math.max(12,W(u.radius*2));ctx.fillStyle='#111';ctx.fillRect(p.x-w/2,p.y-W(54000),w,4);ctx.fillStyle=u.team===0?'#73d2b8':'#ef7378';ctx.fillRect(p.x-w/2,p.y-W(54000),w*Math.max(0,u.hp/u.max_hp),4);
  if(u.champion||u.reflection){ctx.fillStyle='#eef';ctx.font='11px sans-serif';ctx.textAlign='center';ctx.fillText(u.name,p.x,p.y-W(57000));}
  if(tick<(u.sealUntil||0))painter.fx(ctx,'seal',u.pos,P,W);
  if(tick<(u.charmUntil||0))painter.fx(ctx,'charm',{x:u.pos.x,y:u.pos.y-50000},P,W);
 }
 for(const t of threads){const a=actor(t.a),b=actor(t.b);if(!a||!b)continue;const p=P(a.pos),q=P(b.pos);ctx.strokeStyle='#d5baff';ctx.lineWidth=2;ctx.beginPath();ctx.moveTo(p.x,p.y-W(20000));ctx.lineTo(q.x,q.y-W(20000));ctx.stroke();}
 for(const f of fx)painter.fx(ctx,f.name,actor(f.target)?.pos||f.at,P,W,Math.min(1,(f.until-tick)/8));
 for(const s of shots){const a=P(s.from),b=P(s.to);ctx.strokeStyle='#ede4ff';ctx.beginPath();ctx.moveTo(a.x,a.y-W(20000));ctx.lineTo(b.x,b.y-W(20000));ctx.stroke();}
 world.drawMap(ctx,P,W,true);
 const cd=n=>(Math.max(0,(state[n]||0)-tick)/60).toFixed(1)+'s';
 $('state').textContent='Tick '+tick+' · 60 ticks/s\nHP '+me.hp+'/'+me.max_hp+'\nStep '+cd('step_at')+' · Charm '+cd('charm_at')+' · Mend '+cd('mend_at')+'\nLast Reflection '+cd('ult_at')+'\nReflection '+(clone?clone.hp+'/'+clone.max_hp:'respawn '+cd('clone_back_at'))+'\nSwap '+(state.swap?state.swap.kind+' → '+Math.max(0,state.swap.at-tick)+' ticks':'none')+'\nGlobal Flash: '+(tick<(me.sealUntil||0)?'blocked: sealed':state.swap?.other===1?'blocked: swap participant':'available (native policy; not simulated)')+'\n'+(state.ai?.reason||'Manual commands')+'\nContinuation: '+((state.ai?.continuation||[]).join(' → ')||'none')+'\nCandidates '+(state.ai?.evaluated||0)+'/64 · horizon '+(state.ai?.horizon||0)+(recording?'\nRecording '+record.frames.length+' ticks':'');
}
function eventPoint(e){const r=canvas.getBoundingClientRect();return{x:camera.x+(e.clientX-r.left-r.width/2)*camera.span/r.width,y:camera.y+(e.clientY-r.top-r.height/2)*camera.span/r.height};}
canvas.onpointermove=e=>{if(e.buttons===4){follow=false;camera.x-=e.movementX*camera.span/canvas.clientWidth;camera.y-=e.movementY*camera.span/canvas.clientHeight;}else aim=eventPoint(e);if(!running)draw();};
canvas.onpointerdown=e=>{canvas.focus();aim=eventPoint(e);if(e.button===0)me.goal=world.legal(aim,me.radius);};
canvas.onwheel=e=>{e.preventDefault();camera.span=Math.max(150000,Math.min(1120000,camera.span*Math.exp(e.deltaY*.001)));draw();};
canvas.onkeydown=e=>{if(e.repeat)return;const k=e.key.toLowerCase();if(['w','a','s','d','q','e','c',' '].includes(k)){e.preventDefault();held.add(k);if(k==='q')order('step');if(k==='e')order('charm');if(k==='c')order('mend');if(k===' ')attack();running=true;}};canvas.onkeyup=e=>held.delete(e.key.toLowerCase());canvas.onblur=()=>held.clear();
document.addEventListener('visibilitychange',()=>{held.clear();clock.reset();});
$('play').onclick=()=>{running=!running;clock.reset();};$('step').onclick=()=>{running=false;step();};$('reset').onclick=reset;$('scene').onchange=reset;$('full').onclick=()=>{follow=false;camera={x:480000,y:480000,span:1040000};draw();};$('follow').onclick=()=>{follow=true;camera.span=400000;draw();};
document.querySelectorAll('[data-order]').forEach(b=>b.onclick=()=>order(b.dataset.order));
$('hurtCarry').onclick=()=>{const a=units.find(u=>u.id===4),e=units.find(u=>u.id===2);a.hp=Math.floor(a.max_hp*.25);e.pos=world.legal({x:a.pos.x+15000,y:a.pos.y},e.radius);a.wardUntil=0;draw();};
$('hurtOwner').onclick=()=>{const e=units.find(u=>u.id===2);me.hp=Math.floor(me.max_hp*.4);e.pos=world.legal({x:me.pos.x+12000,y:me.pos.y},e.radius);draw();};
$('killClone').onclick=()=>{if(clone)clone.hp=0;};
$('sealCarry').onclick=()=>{step();const a=units.find(u=>u.id===4);a.hp=1;step();};
$('record').onclick=()=>{reset();recording=true;record={version:1,rules_revision:R.revision,profile_version:2,host:'simulated',frames:[]};};
$('export').onclick=()=>{if(!record)return;const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify(record)],{type:'application/json'}));a.download='kagami-recording.json';a.click();URL.revokeObjectURL(a.href);};
$('import').onchange=async e=>{try{imported=JSON.parse(await e.target.files[0].text());log('Inspectable '+(imported.rules_revision||'unknown rules')+' · '+(imported.frames?.length||0)+' ticks'+(imported.rules_revision!==R.revision?' · replay requires its original rules':''));}catch(e){log(e.message);}};
$('replay').onclick=async()=>{const r=imported||record;if(!r)return;if(r.rules_revision!==R.revision){log('Replay requires original rules '+r.rules_revision);return;}const instance=(await WebAssembly.instantiate(wasmBytes,{})).instance.exports;instance.kagami_reset();let matches=0;for(const f of r.frames){if(JSON.stringify(core(f.frame,instance))!==JSON.stringify(f.result))break;matches++;}log('Core replay agrees '+matches+'/'+r.frames.length);running=false;};

await Promise.all([world.ready,painter.ready]);await world.loadSprites(Object.keys(world.data.sprites));wasmBytes=await(await fetch('kagami.wasm')).arrayBuffer();const result=await WebAssembly.instantiate(wasmBytes,{});ex=result.instance.exports;reset();
window.kagamiLab={world,painter,draw,step,reset,order,attack,frame,core,get tick(){return tick},get me(){return me},get units(){return units},get state(){return state},get clone(){return clone},get aim(){return aim},set aim(p){aim=p},get camera(){return camera},get recording(){return record}};
function render(now){clock.advance(now,running,step);draw();requestAnimationFrame(render);}requestAnimationFrame(render);
})().catch(e=>{document.getElementById('state').textContent=e.stack;console.error(e);});
