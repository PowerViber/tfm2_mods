/* Shared Rust skills and AI. Extracted map/assets; explicitly simulated host. */
(async()=>{'use strict';
const $=id=>document.getElementById(id),R=CONTENDER_RULES,canvas=$('arena'),ctx=canvas.getContext('2d');
const ex=(await WebAssembly.instantiate(await(await fetch('contender.wasm')).arrayBuffer())).instance.exports;
const world=await new GameWorld().ready,painter=await new ContenderPainter().ready,clock=new KratosClock();
let scene,tick,me,units,shadow,aim,running=false,held=new Set(),glides=new Map(),fx=[],notes=[],hits=[],state={},nextBasic=0,pendingBasic=null,flags={},queued=null,view,zoom=.0025,records=[];
const dist=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y),round=p=>({x:Math.round(p.x),y:Math.round(p.y)});
const P=p=>({x:450+(p.x-view.x)*zoom,y:450+(p.y-view.y)*zoom}),W=n=>n*zoom;
const actor=id=>id===1?me:id===99?shadow:units.find(u=>u.id===id);
const log=s=>{notes.unshift(tick+': '+s);notes.length=Math.min(30,notes.length);$('log').textContent=notes.join('\n');};
const labels=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];
for(const [i,name]of labels.entries())$('athleteStats').insertAdjacentHTML('beforeend',`<label>${name} <input data-stat="${i}" type="number" min="0" max="100" value="50" style="width:50px"></label>`);
for(const[k,v]of Object.entries(GameScenario.choices))$('scenario').add(new Option(v,k));
function core(frame){const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame})),p=ex.contender_input(bytes.length);new Uint8Array(ex.memory.buffer,p,bytes.length).set(bytes);const out=ex.contender_step(bytes.length),r=JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer,out,ex.contender_output_len())));if(r.error)throw Error(r.error);return r;}
async function reset(){
 ex.contender_reset();scene=new GameScenario(world,$('scenario').value||'laning',8);const s=scene.setup();tick=s.clock;flags={};fx=[];hits=[];glides.clear();shadow=null;pendingBasic=null;state={};nextBasic=0;notes=[];records=[];held.clear();queued=null;clock.reset();
 const stats=scene.stats(R);for(const[k,pct]of [['attack',R.modPower[0]],['hp',R.modPower[2]],['defence',R.modPower[3]],['magic_resistance',R.modPower[4]]])stats[k]=Math.floor(stats[k]*(100+pct)/100);
 me={id:1,team:0,pos:s.pos,hp:stats.hp,max_hp:stats.hp,attack:stats.attack,stats,radius:world.data.settings.champion_radius,champion:true,targetable:true,visible:true,attackStart:-100,attackUntil:0,moving:false,shields:[]};
 units=s.units;aim={...units.find(u=>u.team===1&&u.champion).pos};view={...me.pos};$('description').textContent=s.description;$('revision').textContent=R.revision;
 await world.loadSprites(units.map(u=>u.sprite));scene.refreshVisibility(me,units);draw();return state;
}
function frame(order){return {tick,owner:1,team:0,alive:me.hp>0,pos:round(me.pos),hp:Math.max(0,Math.round(me.hp)),max_hp:me.max_hp,attack:Math.floor(me.attack*(1+((flags.rhythm?.attack_permille)||0)/1000)),stunned:tick<(me.stunUntil||0),ult_unlocked:$('ultLearned').checked,
 home:{x:96000,y:864000},units:units.filter(u=>u.hp>0).map(u=>({id:u.id,team:u.team,pos:round(u.pos),hp:Math.round(u.hp),max_hp:u.max_hp,radius:u.radius,champion:u.champion,tower:u.tower,visible:u.visible,targetable:u.targetable!==false,farmable:u.farmable&&u.team!==0&&(scene.kind==='camp'||!units.some(e=>e.team===1&&e.champion&&e.hp>0&&e.visible&&dist(e.pos,me.pos)<80000))})),
 terrain:world.walls,profile:[...document.querySelectorAll('[data-stat]')].map(i=>Math.max(0,Math.min(100,+i.value||0))),manual:!$('ai').checked,order:$('ai').checked?null:order,basic_hits:hits.splice(0),shadow:shadow&&shadow.hp>0?{id:99,pos:round(shadow.pos),hp:Math.round(shadow.hp),max_hp:shadow.max_hp}:null};}
function glide(id,to,ticks){const u=actor(id);if(u)glides.set(id,{from:{...u.pos},to:{...to},start:tick,ticks:Math.max(1,ticks)});}
function damage(u,raw){if(u&&u.hp>0)scene.damage(me,u,raw,tick);}
function apply(e){const u=actor(e.target);switch(e.kind){
 case 'damage':damage(u,e.amount);break;
 case 'stun':case 'knock_up':if(u){u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);if(e.kind==='knock_up')u.airborneUntil=tick+e.ticks;}break;
 case 'slow':if(u){u.slowUntil=tick+e.ticks;u.slowPct=e.pct;}break;
 case 'knock':glide(e.target,e.to,e.ticks);break;case 'glide':glide(1,e.to,e.ticks);break;
 case 'teleport':me.pos={...e.to};glides.delete(1);break;
 case 'hold':flags.hold=tick+e.ticks;break;case 'vanish':flags.vanish=tick+e.ticks;break;case 'invulnerable':flags.inv=tick+e.ticks;break;
 case 'spawn_shadow':shadow={id:99,team:0,pos:{...e.at},home:{...e.at},hp:e.hp,max_hp:e.hp,radius:me.radius,stats:{...me.stats,attack:0,move_speed:0},attack:{},champion:false,targetable:true,visible:true,stunUntil:Infinity,shields:[],born:tick};break;
 case 'move_shadow':if(shadow)glide(99,e.to,e.ticks);break;case 'despawn_shadow':shadow=null;glides.delete(99);break;
 case 'self_damage':me.hp=Math.max(0,me.hp-e.amount);if(!me.hp)me.deadAt=tick;log('Shadow relay '+e.amount);break;
 case 'rhythm':flags.rhythm=e;flags.rhythmUntil=tick+e.ticks;break;case 'charges':flags.charges=e.count;break;case 'steer':flags.walk=e.to;break;
 case 'fx':{if(e.name==='ct_shadow')break;const tag=e.name.replace('ct_','')+'_t'+e.ticks+(me.pos.x>e.at.x?'_l':'');if(!painter.data.effects.anims[tag])throw Error('Unbound cue '+tag);fx.push({tag,at:{...e.at},start:tick,until:tick+e.ticks});break;}
}}
function stepWorld(order){
 scene.refreshVisibility(me,units);if(tick>=flags.rhythmUntil)flags.rhythm=null;
 const f=frame(order),r=core(f);state=r.state;if(r.rejection)log('Rejected: '+r.rejection);for(const e of r.effects)apply(e);
 records.push({frame:f,effects:r.effects,state:r.state});if(records.length>1200)records.shift();
 for(const[id,g]of glides){const u=actor(id);if(!u){glides.delete(id);continue;}const k=Math.min(1,(tick-g.start+1)/g.ticks),to={x:g.from.x+(g.to.x-g.from.x)*k,y:g.from.y+(g.to.y-g.from.y)*k},prev=u.pos;u.pos=world.sweep(u.pos,to,u.radius);u.left=u.pos.x<prev.x;u.moving=dist(prev,u.pos)>.1;if(k>=1)glides.delete(id);}
 const free=me.hp>0&&tick>=(me.stunUntil||0)&&tick>=(flags.hold||0)&&tick>=(flags.vanish||0);me.moving=false;
 if(free&&!glides.has(1)){let dx=held.has('d')-held.has('a'),dy=held.has('s')-held.has('w'),goal;
  if(dx||dy){const l=Math.hypot(dx,dy);goal={x:me.pos.x+dx/l*16000,y:me.pos.y+dy/l*16000};}else if($('ai').checked&&flags.walk)goal=flags.walk;
  if(goal&&tick>=me.attackUntil)me.moving=world.move(me,goal,me.stats.move_speed*(1+((flags.rhythm?.move_permille)||0)/1000));}
 if(free&&pendingBasic&&tick>=pendingBasic.at){const t=actor(pendingBasic.target);if(t&&t.hp>0){const stacks=(state.stacks||[]).find(s=>s.id===t.id&&tick<s.until)?.count||0;damage(t,Math.floor(f.attack*(1000+stacks*R.rhythm.vulnerabilityPermillePerStack)/1000));if(!t.tower)hits.push(t.id);}pendingBasic=null;}
 if(!free)pendingBasic=null;
 if(free&&($('ai').checked||held.has(' '))&&tick>=nextBasic){const t=units.filter(u=>u.team!==0&&u.hp>0&&u.visible&&world.clear(me.pos,u.pos)&&dist(u.pos,me.pos)<=R.basic.range+u.radius+me.radius).sort((a,b)=>(b.champion-a.champion)||dist(a.pos,aim)-dist(b.pos,aim)||a.id-b.id)[0];if(t){me.attackStart=tick;me.attackUntil=tick+R.basic.duration;me.left=t.pos.x<me.pos.x;pendingBasic={target:t.id,at:tick+R.basic.startTiming};nextBasic=tick+Math.round(R.basic.cooldownTicks*100/(110+((flags.rhythm?.attack_speed)||0)));}}
 if($('fight').checked)scene.advance(tick,me,shadow?[...units,shadow]:units,{hit:()=>{},hitOwner:e=>{if(tick<(flags.hold||0)||tick<(flags.vanish||0)||tick<(flags.inv||0))return;scene.metrics.taken+=scene.damage(actor(e.from)||me,me,e.amount,tick);}});
 if(me.hp<=0){me.deadAt??=tick;fx=[];shadow=null;}fx=fx.filter(e=>tick<e.until);tick++;
}
function draw(){if(!me)return;if($('follow').checked)view={...me.pos};ctx.clearRect(0,0,900,900);ctx.imageSmoothingEnabled=false;world.drawMap(ctx,P,W);
 for(const e of fx)if($('showArt').checked)painter.draw(ctx,'effects',e.tag,e.at,tick-e.start,P,W);
 if($('showArt').checked)for(const c of state.cleaves||[])if(!c.recast)painter.draw(ctx,'effects','body_punch'+Math.floor(tick/10)%2+(me.pos.x>c.at.x?'_l':''),c.at,0,P,W);
 for(const u of units.filter(u=>u.visible||u.team===0)){world.drawUnit(ctx,u,tick,P,W);if(u.hp>0){const p=P(u.pos);ctx.fillStyle='#251a2f';ctx.fillRect(p.x-18,p.y-W(31000),36,3);ctx.fillStyle=u.team===0?'#85dcbb':'#f18386';ctx.fillRect(p.x-18,p.y-W(31000),36*u.hp/u.max_hp,3);}}
 if(shadow&&shadow.hp>0){const tag='body_'+(glides.has(99)?'run'+Math.floor(tick/8)%6:'idle0')+(shadow.left?'_l':'');if($('showArt').checked)painter.draw(ctx,'effects',tag,shadow.pos,0,P,W);const p=P(shadow.pos);ctx.fillStyle='#ab80ee';ctx.fillRect(p.x-17,p.y-W(31000),34*shadow.hp/shadow.max_hp,4);}
 if($('showArt').checked&&tick>=(flags.vanish||0))painter.body(ctx,me,tick,P,W);world.drawMap(ctx,P,W,true);
 const p=P(me.pos);ctx.fillStyle='#2b183b';ctx.fillRect(p.x-20,p.y-W(34000),40,5);ctx.fillStyle='#9be2b9';ctx.fillRect(p.x-20,p.y-W(34000),40*me.hp/me.max_hp,5);ctx.fillStyle='#c7a0ff';ctx.font='13px monospace';ctx.fillText('◆'.repeat(state.charges??5)+'◇'.repeat(R.charges.max-(state.charges??5)),p.x-25,p.y-W(39000));
 if($('ranges').checked){ctx.strokeStyle='#bca1e5';for(const radius of[R.projection.range,R.cyclone.range,R.slam.grab]){ctx.beginPath();ctx.arc(p.x,p.y,W(radius),0,Math.PI*2);ctx.stroke();}}
 const a=P(aim);ctx.strokeStyle='#fff';ctx.beginPath();ctx.moveTo(a.x-6,a.y);ctx.lineTo(a.x+6,a.y);ctx.moveTo(a.x,a.y-6);ctx.lineTo(a.x,a.y+6);ctx.stroke();
 $('pips').textContent='◆'.repeat(state.charges??5)+'◇'.repeat(R.charges.max-(state.charges??5));$('state').textContent=`Tick ${tick} · HP ${Math.round(me.hp)}/${me.max_hp}\nS1 / S2 / S3: ${(state.cd||[0,0,0]).map(t=>(Math.max(0,t-tick)/60).toFixed(1)).join(' / ')} s\nUltimate: ${(Math.max(0,(state.ult_at||0)-tick)/60).toFixed(1)} s\nProjection ${!!state.proj} · Flurry ${!!state.flurry}\nEchoing flurry ${!!state.flurry?.echo} · Rhythm ${state.focus?.[1]||0}\n${state.reason||'Manual controls bypass athlete decisions'}\nRequested ${scene.metrics.requested} · HP ${scene.metrics.hp_damage} · shields ${scene.metrics.shield_damage}\nMap: ${world.geometry.registration.matching}/900 cells · 1,000 units/pixel\nFX alive ${fx.length}`;
}
const orderFor=k=>({q:{kind:'s1',aim:round(aim)},e:{kind:'s2',aim:round(aim)},c:{kind:'s3',aim:round(aim)},r:{kind:'ult'},f:{kind:'end'}})[k];
function next(){let o=queued;queued=null;if(!o&&state.proj&&['w','a','s','d'].some(k=>held.has(k))){const p=shadow?.pos||state.proj.pos;o={kind:'move',aim:round({x:p.x+(held.has('d')-held.has('a'))*40000,y:p.y+(held.has('s')-held.has('w'))*40000})};}stepWorld(o);}
function clearHeld(){held.clear();queued=null;clock.reset();}
canvas.addEventListener('pointerdown',e=>{canvas.focus();const r=canvas.getBoundingClientRect(),x=(e.clientX-r.left)*900/r.width,y=(e.clientY-r.top)*900/r.height;aim={x:Math.max(0,Math.min(960000,view.x+(x-450)/zoom)),y:Math.max(0,Math.min(960000,view.y+(y-450)/zoom))};draw();});
canvas.addEventListener('keydown',e=>{const k=e.key.toLowerCase();if(!['w','a','s','d','q','e','c','r','f',' '].includes(k))return;e.preventDefault();held.add(k);if(!e.repeat&&orderFor(k))queued=orderFor(k);running=true;});
addEventListener('keyup',e=>held.delete(e.key.toLowerCase()));canvas.addEventListener('blur',clearHeld);addEventListener('blur',clearHeld);document.addEventListener('visibilitychange',clearHeld);
canvas.addEventListener('wheel',e=>{e.preventDefault();zoom=Math.max(.0008,Math.min(.006,zoom*Math.exp(-e.deltaY*.001)));draw();},{passive:false});
for(const b of document.querySelectorAll('[data-key]'))b.onclick=()=>{queued=orderFor(b.dataset.key);if(!running){next();draw();}};
for(const[id,dx,dy]of [['panLeft',-80000,0],['panRight',80000,0],['panUp',0,-80000],['panDown',0,80000]])$(id).onclick=()=>{$('follow').checked=false;view={x:view.x+dx,y:view.y+dy};draw();};
for(const id of['ranges','showArt','follow'])$(id).onchange=draw;
$('reset').onclick=()=>reset();$('loadScene').onclick=()=>reset();$('play').onclick=()=>{running=!running;clock.reset();};$('step').onclick=()=>{running=false;next();draw();};$('fullMap').onclick=()=>{$('follow').checked=false;view={x:480000,y:480000};zoom=.00086;draw();};
$('saveRecording').onclick=()=>{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify({version:1,rules_revision:R.revision,records})],{type:'application/json'}));a.download='contender-recording.json';a.click();URL.revokeObjectURL(a.href);};
$('recording').onchange=async e=>{try{const j=JSON.parse(await e.target.files[0].text()),rows=j.records||[];$('recordingStatus').textContent=`Inspection: ${rows.length} frames. `+(j.rules_revision===R.revision?'Current rules; inspection does not advance the live sandbox.':'Replay requires original rules '+j.rules_revision+'.');$('recordedFrame').max=Math.max(0,rows.length-1);$('recordedFrame').value=0;$('recordedFrame').oninput=()=>{$('recordedDetail').textContent=JSON.stringify(rows[+$('recordedFrame').value]||{},null,2);};$('recordedFrame').oninput();}catch(e){$('recordingStatus').textContent='Invalid recording: '+e.message;}};
window.contenderLab={step:o=>{if(o)stepWorld(o);else next();draw();return state;},get state(){return state},get me(){return me},get units(){return units},get actors(){return [me,...units]},get shadow(){return shadow},get tick(){return tick},get scene(){return scene},get effects(){return fx},get records(){return records},world,painter,clock,pause:()=>{running=false;clock.reset();},order:(kind,to=aim)=>{queued={kind,aim:round(to)}},reset,draw};
await reset();function loop(now){clock.advance(now,running&&!document.hidden,next);draw();requestAnimationFrame(loop);}requestAnimationFrame(loop);
})().catch(e=>{document.getElementById('state').textContent=e.stack;throw e;});
