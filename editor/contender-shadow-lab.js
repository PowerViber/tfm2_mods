const labLevel=()=>{let e=document.getElementById('labLevel');if(!e){const l=document.createElement('label');l.title='Champion level: Flurry unlocks at 2, Shadow Link at 5';l.innerHTML=' Level <input id="labLevel" type="number" min="1" max="18" value="18" style="width:46px"> ';const host=document.getElementById('ultLearned')?.closest('label')||document.querySelector('.row,.controls,header,h1')||document.body;host.after?host.after(l):document.body.prepend(l);e=document.getElementById('labLevel');}return Math.max(0,Math.min(18,+e.value||0));};
/* Shared Rust skills and AI. Extracted map/assets; explicitly simulated host. */
(async()=>{'use strict';
const $=id=>document.getElementById(id),R=CONTENDER_RULES,canvas=$('arena'),ctx=canvas.getContext('2d');
const ex=(await WebAssembly.instantiate(await(await fetch('contender.wasm')).arrayBuffer())).instance.exports;
const world=await new GameWorld().ready,painter=await new ContenderPainter().ready,clock=new KratosClock();
if(new URLSearchParams(location.search).get('mergePreview')==='1')$('mergePreviewNote').hidden=false;
const targets=await new LabTargets(world).ready;
let scene,tick,me,units,incoming=[],aim,running=false,held=new Set(),glides=new Map(),fx=[],notes=[],hits=[],state={},nextBasic=0,pendingBasic=null,flags={},queued=null,view,zoom=.0025,records=[];
let landedHits=0, landedMergeHits=0, lastCounter='';
const linkWarningImage=new Image();
linkWarningImage.src='../mods/tfm2_custom/vfx/contender_link_warning%23sheet.png';
let ledger=new LabDamageLedger(),panel=null,targetSpecs=null,nextTargetId=1000,selectedTarget=null,resetEpoch=0,isDamageTest=false,loading=false;
const dist=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y),round=p=>({x:Math.round(p.x),y:Math.round(p.y)});
const P=p=>({x:450+(p.x-view.x)*zoom,y:450+(p.y-view.y)*zoom}),W=n=>n*zoom;
const actor=id=>id===1?me:units.find(u=>u.id===id);
const log=s=>{notes.unshift(tick+': '+s);notes.length=Math.min(30,notes.length);$('log').textContent=notes.join('\n');};
const labels=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];
for(const [i,name]of labels.entries())$('athleteStats').insertAdjacentHTML('beforeend',`<label>${name} <input data-stat="${i}" type="number" min="0" max="100" value="50" style="width:50px"></label>`);
$('scenario').add(new Option('1vX — configurable damage test','damage'));
$('scenario').add(new Option('Shadow Link and Counterpunch','shadowLink'));
for(const[k,v]of Object.entries(GameScenario.choices))$('scenario').add(new Option(v,k));
$('fight').checked=false;
function core(frame){const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame})),p=ex.contender_input(bytes.length);new Uint8Array(ex.memory.buffer,p,bytes.length).set(bytes);const out=ex.contender_step(bytes.length),r=JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer,out,ex.contender_output_len())));if(r.error)throw Error(r.error);return r;}
async function reset(){
 const epoch=++resetEpoch;loading=true;$('addTargets').disabled=$('clearTargets').disabled=true;running=false;ex.contender_reset();const fixture=$('scenario').value==='shadowLink',test=$('scenario').value==='damage';isDamageTest=test;scene=new GameScenario(world,test||fixture?'laning':$('scenario').value||'laning',8);const s=scene.setup();ledger=new LabDamageLedger();ledger.attach(scene);tick=s.clock;landedHits=landedMergeHits=0;lastCounter='';flags={};fx=[];hits=[];glides.clear();incoming=[];pendingBasic=null;state={};nextBasic=0;notes=[];records=[];held.clear();queued=null;clock.reset();
 const stats=scene.stats(R);for(const[k,pct]of [['attack',R.modPower[0]],['hp',R.modPower[2]],['defence',R.modPower[3]],['magic_resistance',R.modPower[4]]])stats[k]=Math.floor(stats[k]*(100+pct)/100);
 me={id:1,team:0,pos:s.pos,hp:stats.hp,max_hp:stats.hp,attack:stats.attack,stats,radius:world.data.settings.champion_radius,champion:true,targetable:true,visible:true,attackStart:-100,attackUntil:0,moving:false,shields:[]};
 units=test||fixture?[]:s.units;aim=test||fixture?{x:me.pos.x+30000,y:me.pos.y-20000}:{...units.find(u=>u.team===1&&u.champion).pos};view={...me.pos};$('description').textContent=test?'One controlled Contender against your chosen targets. Add mixed targets on the right; each has its own HP setting and damage counter. Opponent attacks are optional.':s.description;$('revision').textContent=R.revision;
 if(test){targetSpecs??=[{type:'stock:knight',level:8,unlimited:true},{type:'stock:berserker',level:8,unlimited:true},{type:'stock:ice_mage',level:8,unlimited:true}];
  for(let i=0;i<targetSpecs.length;i++){const spec=targetSpecs[i];await targets.prepare(spec.type);if(epoch!==resetEpoch)return state;const pos=spec.pos||{x:aim.x+(i-1)*18000,y:aim.y+(i%2)*18000};const u=targets.make(scene,spec.type,spec.id??nextTargetId++,pos,spec.level);u.unlimitedHP=spec.unlimited;if(spec.maxHP)u.hp=u.max_hp=u.stats.hp=spec.maxHP;if(spec.armour!==undefined)u.stats.defence=spec.armour;u.initialShield=spec.shield||0;u.shields=spec.shield?[{amount:spec.shield,until:Number.MAX_SAFE_INTEGER}]:[];units.push(u);}}
 if(fixture){
  for(const spec of [
   {type:'stock:knight', name:'Tank', hp:3000, dx:34000, dy:-16000},
   {type:'stock:ninja', name:'Assassin', hp:1200, dx:40000, dy:-29000},
   {type:'rhino', name:'Creep', hp:650, dx:17000, dy:-29000},
   {type:'stock:archer', name:'Training ally', hp:1200, dx:5000, dy:-24000, ally:true}
  ]){
   await targets.prepare(spec.type);if(epoch!==resetEpoch)return state;
   const unit=targets.make(scene,spec.type,nextTargetId++,{x:me.pos.x+spec.dx,y:me.pos.y+spec.dy},8);
   unit.name=spec.name;unit.hp=unit.max_hp=unit.stats.hp=spec.hp;
   if(spec.ally){unit.team=0;unit.labTarget=false;}
   units.push(unit);
  }
  $('description').textContent='Press R to group the tank, assassin and creep. Use the incoming-hit control to test which attacker receives the counter.';
 }
 await world.loadSprites(units.filter(u=>u.sprite).map(u=>u.sprite));if(epoch!==resetEpoch)return state;loading=false;$('addTargets').disabled=$('clearTargets').disabled=false;scene.refreshVisibility(me,units);saveTargets();panel?.rebuild();listAttackers();draw();return state;
}
function saveTargets(){if(isDamageTest)targetSpecs=units.map(u=>({type:u.labType,id:u.id,level:u.level,pos:{...u.home},unlimited:!!u.unlimitedHP,maxHP:u.max_hp,armour:u.stats.defence,shield:u.initialShield||0}));}
async function addTargets(type,count=1,level=8,unlimited=false){
 if(loading)throw Error('Wait for the current target setup to finish loading.');
 if(!Number.isInteger(count)||count<1||count>64)throw Error('Choose 1–64 targets per addition.');if(!Number.isInteger(level)||level<1||level>30)throw Error('Choose a level from 1 to 30.');
 const capacity=()=>{if(units.filter(u=>u.labTarget).length+count>64||units.length+count>128)throw Error('Limit: 64 configured targets and 128 total units.');};capacity();const epoch=resetEpoch;await targets.prepare(type);if(epoch!==resetEpoch)return;capacity();
 for(let i=0;i<count;i++){const angle=i*Math.PI*2/Math.max(1,count),radius=count===1?0:Math.min(80000,14000*Math.ceil(Math.sqrt(count))),pos=world.legal({x:aim.x+Math.cos(angle)*radius,y:aim.y+Math.sin(angle)*radius});const u=targets.make(scene,type,nextTargetId++,pos,level);u.unlimitedHP=!!unlimited;units.push(u);}saveTargets();scene.refreshVisibility(me,units);panel?.rebuild();listAttackers();draw();
}
function removeTarget(u){units=units.filter(e=>e.id!==u.id);glides.delete(u.id);ledger.reset(u.id);if(selectedTarget===u.id)selectedTarget=null;saveTargets();panel?.rebuild();listAttackers();draw();}
function clearTargets(){if(loading)return;units=units.filter(u=>u.team===0);ledger.reset();selectedTarget=null;saveTargets();panel?.rebuild();listAttackers();draw();}
function shieldOf(unit){return (unit.shields||[]).filter(shield=>tick<shield.until).reduce((sum,shield)=>sum+shield.amount,0);}
function frame(order){
 const enemiesNear=units.some(unit=>unit.team===1&&unit.champion&&unit.hp>0&&unit.visible&&dist(unit.pos,me.pos)<80000);
 const living=units.filter(unit=>unit.hp>0);
 return {tick,owner:1,team:0,alive:me.hp>0,pos:round(me.pos),hp:Math.max(0,Math.round(me.hp)),max_hp:me.max_hp,
  attack:Math.floor(me.attack*(1+((flags.rhythm?.attack_permille)||0)/1000)),stunned:tick<(me.stunUntil||0),ult_unlocked:$('ultLearned').checked,level:labLevel(),
  home:{x:96000,y:864000},
  units:living.map(unit=>({id:unit.id,team:unit.team,pos:round(unit.pos),hp:Math.round(unit.hp),max_hp:unit.max_hp,radius:unit.radius,
   champion:!!unit.champion,tower:!!unit.tower,visible:unit.visible,targetable:unit.targetable!==false,
   farmable:unit.farmable&&unit.team!==0&&(scene.kind==='camp'||!enemiesNear)})),
  terrain:world.walls,profile:[...document.querySelectorAll('[data-stat]')].map(input=>Math.max(0,Math.min(100,+input.value||0))),
  manual:!$('ai').checked,order:$('ai').checked?null:order,basic_hits:hits.splice(0),
  incoming:incoming.slice(0,16).map(hit=>({attacker:hit.attacker,from:round(hit.pos),counter:!!hit.counter})),
  landed_hits:landedHits,landed_merge_hits:landedMergeHits,
  link_excluded:living.filter(unit=>unit.ccImmune||(!unit.champion&&!['minion','camp'].includes(unit.kind))).map(unit=>unit.id),
  separated:living.filter(unit=>unit.separated).map(unit=>unit.id),
  link_airborne:living.filter(unit=>tick<(unit.linkAirborneUntil||0)).map(unit=>unit.id),
  defences:living.filter(unit=>unit.champion).map(unit=>({id:unit.id,armour:Math.round(unit.stats?.defence||0),shield:Math.round(shieldOf(unit))}))};
}
function glide(id,to,ticks){const u=actor(id);if(u)glides.set(id,{from:{...u.pos},to:{...to},start:tick,ticks:Math.max(1,ticks)});}
function damage(u,raw){
 if(!u||u.hp<=0||raw<=0)return false;
 const shield=shieldOf(u);
 return scene.damage(me,u,raw,tick)>0||shieldOf(u)<shield;
}
function apply(e){const u=actor(e.target);switch(e.kind){
 case 'damage':
  if(damage(u,e.amount)){
   if(e.source==='merge')landedMergeHits++;
   else if(e.source==='echo'||e.source==='counter')landedHits++;
  }
  break;
 case 'hold_forward':flags.holdForward=true;break;
 case 'link_airborne':
  if(u&&!u.ccImmune){u.linkAirborneUntil=tick+e.ticks;u.airborneUntil=tick+e.ticks;u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);}
  break;
 case 'link_stun':
  if(u&&!u.ccImmune)u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);
  break;
 case 'log':
  lastCounter=e.text.replace(/attacker #(\d+)/,(_,id)=>actor(+id)?.name||'attacker #'+id);
  log(lastCounter);
  break;
 case 'knock':glide(e.target,e.to,e.ticks);break;case 'glide':glide(1,e.to,e.ticks);break;
 case 'vanish':flags.vanish=tick+e.ticks;break;
 case 'merge':flags.merge=e.ticks>1?{until:tick+e.ticks,pct:e.move_pct}:null;flags.mergeCast=e.ticks>1?{pose:'projection',start:tick,end:tick+18}:null;break;
 case 'rhythm':flags.rhythm=e;flags.rhythmUntil=tick+e.ticks;break;case 'charges':flags.charges=e.count;break;case 'steer':flags.walk=e.to;flags.walkUntil=tick+6;flags.steered=true;break;
  case 'fx':{if(e.name==='ct_shadow')break;if(e.name==='ct_appear')flags.vanish=tick;const group=e.name==='ct_break'?'rhythm':'effects';if(e.name==='ct_break')flags.breakAt=tick;const tag=e.name==='ct_flurry'&&e.heading!==undefined?`flurry_d${e.heading%8}_t${e.ticks}`:e.name.replace('ct_','')+'_t'+e.ticks+(me.pos.x>e.at.x?'_l':'');if(!painter.data[group].anims[tag])throw Error('Unbound cue '+tag);fx.push({group,tag,at:{...e.at},start:tick,until:tick+e.ticks});break;}
}}
function stepWorld(order){
 scene.refreshVisibility(me,units);if(tick>=flags.rhythmUntil)flags.rhythm=null;
 flags.holdForward=flags.steered=false;
 const f=frame(order),r=core(f);landedHits=landedMergeHits=0;state=r.state;if(r.rejection)log('Rejected: '+r.rejection);if(incoming.length&&state.last_dodge===f.tick){const h=incoming[0];me.hp=Math.min(me.max_hp,me.hp+h.amount);scene.metrics.taken-=h.amount;log('Dodged '+h.amount);}incoming=[];for(const e of r.effects)apply(e);
 if(flags.holdForward&&!flags.steered&&!state.reason.startsWith('Retreat')&&!state.reason.startsWith('Peel'))flags.walk=null;
 records.push({frame:f,effects:r.effects,state:r.state});if(records.length>1200)records.shift();
 for(const[id,g]of glides){const u=actor(id);if(!u){glides.delete(id);continue;}const k=Math.min(1,(tick-g.start+1)/g.ticks),to={x:g.from.x+(g.to.x-g.from.x)*k,y:g.from.y+(g.to.y-g.from.y)*k},prev=u.pos;u.pos=world.sweep(u.pos,to,u.radius);u.left=u.pos.x<prev.x;u.moving=dist(prev,u.pos)>.1;if(k>=1)glides.delete(id);}
 const free=me.hp>0&&tick>=(me.stunUntil||0)&&tick>=(flags.vanish||0);me.moving=false;
 if(free&&!glides.has(1)){let dx=held.has('d')-held.has('a'),dy=held.has('s')-held.has('w'),goal;
  if(dx||dy){const l=Math.hypot(dx,dy);goal={x:me.pos.x+dx/l*16000,y:me.pos.y+dy/l*16000};}else if($('ai').checked&&flags.walk&&tick<flags.walkUntil)goal=flags.walk;
  if(goal&&tick>=me.attackUntil)me.moving=world.move(me,goal,me.stats.move_speed*(1+((flags.rhythm?.move_permille)||0)/1000+(tick<(flags.merge?.until||0)?flags.merge.pct/100:0)));}
 if(free&&pendingBasic&&tick>=pendingBasic.at){
  const target=actor(pendingBasic.target);
  if(target&&target.hp>0){
   const stacks=(state.stacks||[]).find(stack=>stack.id===target.id&&tick<stack.until)?.count||0;
   if(damage(target,Math.floor(f.attack*(1000+stacks*R.rhythm.vulnerabilityPermillePerStack)/1000)))hits.push(target.id);
  }
  pendingBasic=null;
 }
 if(!free)pendingBasic=null;
 if(free&&($('ai').checked||held.has(' '))&&tick>=nextBasic){
  const target=units.filter(unit=>unit.team!==0&&unit.hp>0&&unit.visible
   && world.clear(me.pos,unit.pos)&&dist(unit.pos,me.pos)<=R.basic.range+unit.radius+me.radius)
   .sort((a,b)=>(b.champion-a.champion)||dist(a.pos,aim)-dist(b.pos,aim)||a.id-b.id)[0];
  if(target){
   me.attackStart=tick;me.attackUntil=tick+R.basic.duration;me.left=target.pos.x<me.pos.x;
   pendingBasic={at:tick+R.basic.startTiming,target:target.id};
   nextBasic=tick+Math.round(R.basic.cooldownTicks*100/(110+((flags.rhythm?.attack_speed)||0)));
  }
 }
 if($('fight').checked)scene.advance(tick,me,units,{hit:()=>{},hitOwner:e=>incomingAttack(actor(e.from),e.amount)});
 painter.observeMerge(me,tick,flags.mergeCast);
  if(me.hp<=0){me.deadAt??=tick;fx=[];delete flags.breakAt;}fx=fx.filter(e=>tick<e.until);tick++;
}
function incomingAttack(from,raw){
 if(tick<(flags.vanish||0)||!from||me.hp<=0)return;
 const dealt=scene.damage(from,me,raw,tick);
 scene.metrics.taken+=dealt;
 if(dealt>0){
  me.hitUntil=tick+6;
  if(from.champion&&from.team!==me.team)incoming.push({attacker:from.id,pos:{...from.pos},amount:dealt});
 }
}
function listAttackers(){
 const list=$('counterAttacker'),selected=list.value;
 list.replaceChildren(...units.filter(u=>u.team!==me.team&&u.champion&&u.hp>0).map(u=>new Option(u.name||'#'+u.id,u.id)));
 if([...list.options].some(option=>option.value===selected))list.value=selected;
 $('simulateHit').disabled=list.options.length===0;
}
$('simulateHit').onclick=()=>{
 incomingAttack(actor(+$('counterAttacker').value),90);
 if(!running){next();draw();}
};
function drawRhythmStacks(){
 const stacks=new Map((state.stacks||[]).filter(s=>s.count>0&&tick<s.until).map(s=>[s.id,s.count]));
 const icon=painter.data.effects.anims.uppercut.frames[0].data;
 ctx.save();ctx.font='bold 9px monospace';ctx.textAlign='center';ctx.textBaseline='middle';
 for(const u of units){const count=stacks.get(u.id);if(!count||u.team===me.team||u.hp<=0||!u.visible)continue;
  const p=P(u.pos),y=p.y-W(35000)-21,colour=count===R.rhythm.maxStacks?'#ffe0a0':'#b88aef';
  ctx.fillStyle='#20132f';ctx.fillRect(p.x-11,y-11,22,22);ctx.strokeStyle=colour;ctx.strokeRect(p.x-11,y-11,22,22);
  // Reuse the approved clenched fist, cropping out the uppercut's travel trail.
  ctx.drawImage(painter.images.effects,icon.x+10,icon.y,26,26,p.x-10,y-10,20,20);
  ctx.fillStyle='#20132f';ctx.fillRect(p.x+1,y+2,14,11);ctx.fillStyle=colour;ctx.fillText(String(count),p.x+8,y+7);
 }
 ctx.restore();
}
function drawShadow(){
 const link=state.shadow_link;
 if(link?.phase==='flight'){
  if(!$('showArt').checked)return;
  const facing=link.at.x<link.from.x?'_l':'';
  painter.draw(ctx,'effects','body_run'+Math.floor(tick/4)%6+facing,link.cursor,0,P,W);
 }
 if(link?.phase==='warning'){
  const center=P(link.at);
  if(linkWarningImage.complete&&linkWarningImage.naturalWidth){
   const size=W(linkWarningImage.naturalWidth*1000);
   ctx.drawImage(linkWarningImage,center.x-size/2,center.y-size/2,size,size);
  }else{
   ctx.beginPath();ctx.arc(center.x,center.y,W(R.shadowLink.radius),0,Math.PI*2);
   ctx.fillStyle='#925bff22';ctx.fill();ctx.strokeStyle='#d2adff';ctx.stroke();
  }
  if($('showArt').checked)painter.draw(ctx,'effects','body_idle0',link.at,0,P,W);
  for(const id of link.linked){
   const target=actor(id);if(!target?.visible||target.hp<=0)continue;
   const p=P(target.pos);
   ctx.strokeStyle='#d6b8ff88';ctx.beginPath();ctx.moveTo(center.x,center.y);ctx.lineTo(p.x,p.y);ctx.stroke();
   ctx.strokeStyle='#d6b8ff';ctx.strokeRect(p.x+16,p.y-W(35000),8,6);
  }
 }
}
function shadowReadout(){
 const link=state.shadow_link;
 const who=id=>actor(id)?.name||'#'+id;
 const phase=link?.phase;
 const message=phase==='flight'?'Shadow Link travelling'
  :phase==='warning'?'Pull in '+Math.max(0,(link.until-tick)/60).toFixed(2)+' s · '+(link.linked.map(who).join(', ')||'no links')
  :phase==='pull'?'Pull · '+link.pulled.map(unit=>who(unit.target)).join(', ')
  :phase==='airborne'?'Knock-up · '+link.targets.map(who).join(', ')
  :'Shadow Link idle';
 return [message,lastCounter].filter(Boolean).join('\n');
}
function draw(){if(!me)return;if($('follow').checked)view={...me.pos};ctx.clearRect(0,0,900,900);ctx.imageSmoothingEnabled=false;world.drawMap(ctx,P,W);
  if($('showArt').checked&&me.hp>0&&flags.breakAt!==undefined)painter.draw(ctx,'rhythm','full_rhythm_t480',me.pos,tick-flags.breakAt,P,W);
  for(const e of fx)if($('showArt').checked)painter.draw(ctx,e.group||'effects',e.tag,e.at,tick-e.start,P,W);
 for(const u of units.filter(u=>u.visible||u.team===0)){targets.draw(ctx,u,tick,P,W);if(u.hp>0){const p=P(u.pos);ctx.fillStyle='#251a2f';ctx.fillRect(p.x-18,p.y-W(31000),36,3);ctx.fillStyle=u.team===0?'#85dcbb':'#f18386';ctx.fillRect(p.x-18,p.y-W(31000),36*u.hp/u.max_hp,3);if(u.team!==0){ctx.fillStyle='#eee5fb';ctx.font='11px monospace';ctx.fillText('#'+u.id+(u.unlimitedHP?' ∞':''),p.x-18,p.y-W(35000));}}}
 drawShadow();
 if($('showArt').checked&&tick>=(flags.vanish||0))painter.body(ctx,me,tick,P,W,tick<(flags.merge?.until||0),flags.mergeCast);world.drawMap(ctx,P,W,true);drawRhythmStacks();
 const p=P(me.pos);ctx.fillStyle='#2b183b';ctx.fillRect(p.x-20,p.y-W(34000),40,5);ctx.fillStyle='#9be2b9';ctx.fillRect(p.x-20,p.y-W(34000),40*me.hp/me.max_hp,5);ctx.fillStyle='#c7a0ff';ctx.font='13px monospace';ctx.fillText('◆'.repeat(Math.min(7,state.charges??5))+'◇'.repeat(Math.max(0,R.charges.max-(state.charges??5))),p.x-25,p.y-W(39000));
 if($('ranges').checked){ctx.strokeStyle='#bca1e5';for(const radius of[R.flurry.startRange,R.shadowLink.range,R.merge.crossRadius]){ctx.beginPath();ctx.arc(p.x,p.y,W(radius),0,Math.PI*2);ctx.stroke();}}
 const a=P(aim);ctx.strokeStyle='#fff';ctx.beginPath();ctx.moveTo(a.x-6,a.y);ctx.lineTo(a.x+6,a.y);ctx.moveTo(a.x,a.y-6);ctx.lineTo(a.x,a.y+6);ctx.stroke();
 const seconds=at=>(Math.max(0,(at||0)-tick)/60).toFixed(2);
 const dodges=state.charges??R.charges.max;
 const flurry=state.flurry_charges??R.flurry.maxCharges;
 $('pips').textContent='Dodge '+'◆'.repeat(dodges)+'◇'.repeat(R.charges.max-dodges)+'   Flurry '+'◆'.repeat(flurry)+'◇'.repeat(R.flurry.maxCharges-flurry);
 $('state').textContent=[
  'Tick '+tick+' · HP '+Math.round(me.hp)+'/'+me.max_hp,
  'Merge '+seconds(state.merge_at)+' s · Ultimate '+seconds(state.ult_at)+' s',
  'Flurry '+flurry+'/'+R.flurry.maxCharges+' · next charge '+seconds(state.flurry_charge_at)+' s · cast gap '+seconds(state.flurry_cast_at)+' s',
  'Recharge saved '+((state.flurry_credited_ticks||0)/60).toFixed(2)+' s · punches '+(state.flurry?.done||0)+' · Rhythm '+(state.focus?.[1]||0),
  state.reason||'Manual controls',
  'Requested '+scene.metrics.requested+' · HP '+scene.metrics.hp_damage+' · shields '+scene.metrics.shield_damage,
  'Map '+world.geometry.registration.matching+'/900 cells · 1,000 units/pixel · FX alive '+fx.length
 ].join('\n');
 $('shadowStatus').textContent=shadowReadout();
 panel?.update(tick);
}
const orderFor=k=>({q:{kind:'s1'},e:{kind:'s2'},exchange:{kind:'exchange_dodge'},r:{kind:'ult',aim:round(aim)}})[k];
function next(){if(loading)return;const o=queued;queued=null;stepWorld(o);}
function clearHeld(){held.clear();queued=null;clock.reset();}
canvas.addEventListener('pointerdown',e=>{canvas.focus();const r=canvas.getBoundingClientRect(),x=(e.clientX-r.left)*900/r.width,y=(e.clientY-r.top)*900/r.height;aim={x:Math.max(0,Math.min(960000,view.x+(x-450)/zoom)),y:Math.max(0,Math.min(960000,view.y+(y-450)/zoom))};draw();});
canvas.addEventListener('keydown',e=>{const k=e.key.toLowerCase();if(!['w','a','s','d','q','e','r',' '].includes(k))return;e.preventDefault();held.add(k);if(!e.repeat){const order=orderFor(k==='e'&&e.shiftKey?'exchange':k);if(order)queued=order;}running=true;});
addEventListener('keyup',e=>held.delete(e.key.toLowerCase()));canvas.addEventListener('blur',clearHeld);addEventListener('blur',clearHeld);document.addEventListener('visibilitychange',clearHeld);
canvas.addEventListener('wheel',e=>{e.preventDefault();zoom=Math.max(.0008,Math.min(.006,zoom*Math.exp(-e.deltaY*.001)));draw();},{passive:false});
for(const b of document.querySelectorAll('[data-key]'))b.onclick=()=>{queued=orderFor(b.dataset.key);if(!running){next();draw();}};
for(const[id,dx,dy]of [['panLeft',-80000,0],['panRight',80000,0],['panUp',0,-80000],['panDown',0,80000]])$(id).onclick=()=>{$('follow').checked=false;view={x:view.x+dx,y:view.y+dy};draw();};
for(const id of['ranges','showArt','follow'])$(id).onchange=draw;
$('reset').onclick=()=>reset();$('loadScene').onclick=()=>reset();$('play').onclick=()=>{running=!running;clock.reset();};$('step').onclick=()=>{running=false;next();draw();};$('fullMap').onclick=()=>{$('follow').checked=false;view={x:480000,y:480000};zoom=.00086;draw();};
$('saveRecording').onclick=()=>{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify({version:1,rules_revision:R.revision,records})],{type:'application/json'}));a.download='contender-recording.json';a.click();URL.revokeObjectURL(a.href);};
$('recording').onchange=async e=>{try{const j=JSON.parse(await e.target.files[0].text()),rows=j.records||[];$('recordingStatus').textContent=`Inspection: ${rows.length} frames. `+(j.rules_revision===R.revision?'Current rules; inspection does not advance the live sandbox.':'Replay requires original rules '+j.rules_revision+'.');$('recordedFrame').max=Math.max(0,rows.length-1);$('recordedFrame').value=0;$('recordedFrame').oninput=()=>{$('recordedDetail').textContent=JSON.stringify(rows[+$('recordedFrame').value]||{},null,2);};$('recordedFrame').oninput();}catch(e){$('recordingStatus').textContent='Invalid recording: '+e.message;}};
window.contenderLab={step:o=>{if(o)stepWorld(o);else next();draw();return state;},get state(){return state},get lastCounter(){return lastCounter},get me(){return me},get units(){return units},get actors(){return [me,...units]},get incoming(){return incoming},get tick(){return tick},get scene(){return scene},get effects(){return fx},get records(){return records},get ledger(){return ledger},world,painter,clock,targets,addTargets,removeTarget,clearTargets,pause:()=>{running=false;clock.reset();},order:(kind,to=aim)=>{queued={kind,aim:round(to)}},reset,draw};
if(new URLSearchParams(location.search).get('demo')==='shadow-link')$('scenario').value='shadowLink';
await reset();panel=new LabTargetPanel(targets,{units:()=>units,tick:()=>tick,ledger:()=>ledger,level:()=>scene.level,selected:()=>selectedTarget,add:addTargets,remove:removeTarget,clear:clearTargets,
 changed:u=>{saveTargets();draw();},resetMetrics:()=>{for(const k of Object.keys(scene.metrics))scene.metrics[k]=0;},
 focus:u=>{selectedTarget=u.id;$('follow').checked=false;view={...u.pos};panel.update(tick,true);draw();},move:u=>{u.home=u.pos=world.legal(aim,u.radius);glides.delete(u.id);saveTargets();draw();}});panel.rebuild();
function loop(now){clock.advance(now,running&&!document.hidden,next);draw();panel.update(tick);requestAnimationFrame(loop);}requestAnimationFrame(loop);
})().catch(e=>{document.getElementById('state').textContent=e.stack;throw e;});
