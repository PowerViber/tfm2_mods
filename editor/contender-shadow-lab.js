const labLevel=()=>{let e=document.getElementById('labLevel');if(!e){const l=document.createElement('label');l.title='Champion level: Shadow Anchor at 2, Smother at 3, Clone Army at 5';l.innerHTML=' Level <input id="labLevel" type="number" min="1" max="18" value="18" style="width:46px"> ';const host=document.getElementById('ultLearned')?.closest('label')||document.querySelector('.row,.controls,header,h1')||document.body;host.after?host.after(l):document.body.prepend(l);e=document.getElementById('labLevel');}return Math.max(0,Math.min(18,+e.value||0));};
// round 190: Codex's approved Shadow Grab art, the same packed sheet the game plays (tools/build-contender-grab-art.py)
// round 192: Codex's approved Smother and Clone Army art (tools/build-contender-40-art.py), drawn the same way
const packedSheet=name=>{const art={sheet:new Image(),anims:null};art.sheet.src=`../mods/tfm2_custom/vfx/${name}%23sheet.png`;fetch(`../mods/tfm2_custom/vfx/${name}%23anim.fanim`).then(r=>r.json()).then(j=>art.anims=j.anims).catch(()=>{});return art;};
const grabArt=packedSheet('contender_grab'),c40Art=packedSheet('contender_40');
function sheetDraw(art,tag,at,elapsed){const a=art.anims?.[tag];if(!a||!art.sheet.complete)return;let t=elapsed/60,f=a.frames[a.frames.length-1];for(const fr of a.frames){if(t<fr.duration){f=fr;break;}t-=fr.duration;}const d=f.data,p=P(at),w=W(d.w*1000),h=W(d.h*1000);ctx.drawImage(art.sheet,d.x,d.y,d.w,d.h,p.x-w/2,p.y-h/2,w,h);}
const grabDraw=(tag,at,elapsed)=>sheetDraw(grabArt,tag,at,elapsed);
function smotherTrail(from,front,shadow){const step=10000,gap=Math.round(dist(from,front));if(gap<step/2)return null;const bucket=Math.max(1,Math.min(Math.ceil(R.smother.range/step),Math.floor((gap+step/2)/step)));return{tag:`trail${shadow?'_s':''}_L${bucket}_d${grabHeading(from,front)}`,at:{x:(from.x+front.x)/2,y:(from.y+front.y)/2}};}
const grabHeading=(a,b)=>{const dx=b.x-a.x,dy=b.y-a.y;return dx||dy?((Math.round(Math.atan2(dy,dx)*180/Math.PI/45)%8)+8)%8:0;};
const toward=(a,b,len)=>{const d=dist(a,b);return d<1e-9?{...a}:{x:a.x+(b.x-a.x)*len/d,y:a.y+(b.y-a.y)*len/d};};
function grabRibbon(me,them){const step=10000,lo=Math.floor(R.shadowGrab.pullStop/step),hi=Math.ceil(R.shadowGrab.range/step),bucket=Math.max(lo,Math.min(hi,Math.floor((Math.round(dist(me,them))+step/2)/step))),start=toward(me,them,10000);return{tag:`ribbon_L${bucket}_d${grabHeading(me,them)}`,at:{x:(start.x+them.x)/2,y:(start.y+them.y)/2-3000}};}
/* Shared Rust skills and AI. Extracted map/assets; explicitly simulated host. */
(async()=>{'use strict';
const $=id=>document.getElementById(id),R=CONTENDER_RULES,canvas=$('arena'),ctx=canvas.getContext('2d');
const ex=(await WebAssembly.instantiate(await(await fetch('contender.wasm')).arrayBuffer())).instance.exports;
const world=await new GameWorld().ready,painter=await new ContenderPainter().ready,clock=new KratosClock();
const targets=await new LabTargets(world).ready;
let scene,tick,me,units,incoming=[],aim,running=false,held=new Set(),glides=new Map(),fx=[],notes=[],hits=[],state={},nextBasic=0,pendingBasic=null,flags={},queued=null,view,zoom=.0025,records=[];
let landedHits=0, lastCounter='';
let ledger=new LabDamageLedger(),panel=null,targetSpecs=null,nextTargetId=1000,selectedTarget=null,resetEpoch=0,isDamageTest=false,loading=false;
const dist=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y),round=p=>({x:Math.round(p.x),y:Math.round(p.y)});
const P=p=>({x:450+(p.x-view.x)*zoom,y:450+(p.y-view.y)*zoom}),W=n=>n*zoom;
const actor=id=>id===1?me:units.find(u=>u.id===id);
const log=s=>{notes.unshift(tick+': '+s);notes.length=Math.min(30,notes.length);$('log').textContent=notes.join('\n');};
const labels=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];
for(const [i,name]of labels.entries())$('athleteStats').insertAdjacentHTML('beforeend',`<label>${name} <input data-stat="${i}" type="number" min="0" max="100" value="50" style="width:50px"></label>`);
$('scenario').add(new Option('1vX — configurable damage test','damage'));
$('scenario').add(new Option('Grab, Anchor, Smother and Counterpunch','shadowLink'));
for(const[k,v]of Object.entries(GameScenario.choices))$('scenario').add(new Option(v,k));
$('fight').checked=false;
function core(frame){const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame})),p=ex.contender_input(bytes.length);new Uint8Array(ex.memory.buffer,p,bytes.length).set(bytes);const out=ex.contender_step(bytes.length),r=JSON.parse(new TextDecoder().decode(new Uint8Array(ex.memory.buffer,out,ex.contender_output_len())));if(r.error)throw Error(r.error);return r;}
async function reset(){
 const epoch=++resetEpoch;loading=true;$('addTargets').disabled=$('clearTargets').disabled=true;running=false;ex.contender_reset();const fixture=$('scenario').value==='shadowLink',test=$('scenario').value==='damage';isDamageTest=test;scene=new GameScenario(world,test||fixture?'laning':$('scenario').value||'laning',8);const s=scene.setup();ledger=new LabDamageLedger();ledger.attach(scene);tick=s.clock;landedHits=0;lastCounter='';flags={};fx=[];hits=[];glides.clear();incoming=[];pendingBasic=null;state={};nextBasic=0;notes=[];records=[];held.clear();queued=null;clock.reset();
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
  $('description').textContent='Q grabs at the aim, E drops (then teleports to) the anchor, F casts Smother, R calls Clone Army. Use the incoming-hit control to test the dodge and counter.';
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
  attack:Math.floor(me.attack),stunned:tick<(me.stunUntil||0),ult_unlocked:$('ultLearned').checked,level:labLevel(),
  home:{x:96000,y:864000},
  units:living.map(unit=>({id:unit.id,team:unit.team,pos:round(unit.pos),hp:Math.round(unit.hp),max_hp:unit.max_hp,radius:unit.radius,
   champion:!!unit.champion,tower:!!unit.tower,visible:unit.visible,targetable:unit.targetable!==false,
   farmable:unit.farmable&&unit.team!==0&&(scene.kind==='camp'||!enemiesNear)})),
  terrain:world.walls,profile:[...document.querySelectorAll('[data-stat]')].map(input=>Math.max(0,Math.min(100,+input.value||0))),
  manual:!$('ai').checked,order:$('ai').checked?null:order,basic_hits:hits.splice(0),
  incoming:incoming.slice(0,16).map(hit=>({attacker:hit.attacker,from:round(hit.pos),counter:!!hit.counter})),
  landed_hits:landedHits,
  link_excluded:living.filter(unit=>unit.ccImmune||(!unit.champion&&!['minion','camp'].includes(unit.kind))).map(unit=>unit.id),
  separated:living.filter(unit=>unit.separated).map(unit=>unit.id)};
}
function glide(id,to,ticks){const u=actor(id);if(u)glides.set(id,{from:{...u.pos},to:{...to},start:tick,ticks:Math.max(1,ticks)});}
function damage(u,raw){
 if(!u||u.hp<=0||raw<=0)return false;
 const shield=shieldOf(u);
 return scene.damage(me,u,raw,tick)>0||shieldOf(u)<shield;
}
function apply(e){const u=actor(e.target);const cc=(ticks,key)=>{if(u&&!u.ccImmune){u[key]=tick+ticks;u.stunUntil=Math.max(u.stunUntil||0,key==='silencedUntil'?0:tick+ticks);}};switch(e.kind){
 case 'damage':if(damage(u,e.amount)&&['counter','grab','smother'].includes(e.source))landedHits++;break;
 case 'airborne':cc(e.ticks,'airborneUntil');break;
 case 'fear':cc(e.ticks,'fearedUntil');break;
 case 'hold':cc(e.ticks,'heldUntil');break;
 case 'silence':if(u)u.silencedUntil=tick+e.ticks;break;
 case 'slow':if(u)u.slowUntil=tick+e.ticks;break;
 case 'smothered':if(u){u.smotheredUntil=tick+e.ticks;u.smothered=e;}break;
 case 'haste':flags.haste={until:tick+e.ticks,as:e.attack_speed_pct,ms:e.move_pct};break;
 case 'shield':me.shields=(me.shields||[]).concat([{amount:e.amount,until:tick+e.ticks}]);break;
 case 'drag':if(u){u.pos={...e.to};}break;
 case 'blink':me.pos={...e.to};glides.delete(1);break;
 case 'glide':glide(1,e.to,e.ticks);break;
 case 'charges':flags.charges=e.count;break;
 case 'log':lastCounter=e.text.replace(/#(\d+)/,(_,id)=>actor(+id)?.name||'#'+id);log(lastCounter);break;
 case 'fx':{if(e.name.startsWith('ct_grab_')){fx.push({art:grabArt,tag:e.name.slice(8),at:{...e.at},start:tick,until:tick+e.ticks});break;}if(e.name.startsWith('ct_40_')){fx.push({art:c40Art,tag:e.name.slice(6),at:{...e.at},start:tick,until:tick+e.ticks});break;}const tag=e.name==='ct_flurry'&&e.heading!==undefined?`flurry_d${e.heading%8}_t${e.ticks}`:e.name.replace('ct_','')+'_t'+e.ticks+(me.pos.x>e.at.x?'_l':'');if(!painter.data.effects.anims[tag])throw Error('Unbound cue '+tag);fx.push({group:'effects',tag,at:{...e.at},start:tick,until:tick+e.ticks});break;}
}}
function stepWorld(order){
 scene.refreshVisibility(me,units);if(flags.haste&&tick>=flags.haste.until)flags.haste=null;
 const f=frame(order),r=core(f);landedHits=0;state=r.state;if(r.rejection)log('Rejected: '+r.rejection);if(incoming.length&&state.last_dodge===f.tick){const h=incoming[0];me.hp=Math.min(me.max_hp,me.hp+h.amount);scene.metrics.taken-=h.amount;log('Dodged '+h.amount);}incoming=[];for(const e of r.effects)apply(e);
 records.push({frame:f,effects:r.effects,state:r.state});if(records.length>1200)records.shift();
 for(const[id,g]of glides){const u=actor(id);if(!u){glides.delete(id);continue;}const k=Math.min(1,(tick-g.start+1)/g.ticks),to={x:g.from.x+(g.to.x-g.from.x)*k,y:g.from.y+(g.to.y-g.from.y)*k},prev=u.pos;u.pos=world.sweep(u.pos,to,u.radius);u.left=u.pos.x<prev.x;u.moving=dist(prev,u.pos)>.1;if(k>=1)glides.delete(id);}
 const free=me.hp>0&&tick>=(me.stunUntil||0);me.moving=false;
 if(free&&!glides.has(1)){let dx=held.has('d')-held.has('a'),dy=held.has('s')-held.has('w'),goal;
  if(dx||dy){const l=Math.hypot(dx,dy);goal={x:me.pos.x+dx/l*16000,y:me.pos.y+dy/l*16000};}
  if(goal&&tick>=me.attackUntil)me.moving=world.move(me,goal,me.stats.move_speed*(1+(flags.haste?.ms||0)/100));}
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
   nextBasic=tick+Math.round(R.basic.cooldownTicks*100/(110+(flags.haste?.as||0)));
  }
 }
 if($('fight').checked)scene.advance(tick,me,units,{hit:()=>{},hitOwner:e=>incomingAttack(actor(e.from),e.amount)});
 painter.observeMerge(me,tick,null);
  if(me.hp<=0){me.deadAt??=tick;fx=[];}fx=fx.filter(e=>tick<e.until);tick++;
}
function incomingAttack(from,raw){
 if(!from||me.hp<=0)return;
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
function casterPos(c){return c?.by==='me'?me.pos:(state.shadows||[]).find(s=>s.id===c?.id)?.pos;}
function drawShadow(){
 if(!$('showArt').checked)return;
 const foe=p=>units.filter(u=>u.team!==0&&u.champion&&u.hp>0).sort((a,b)=>dist(a.pos,p)-dist(b.pos,p))[0];
 for(const s of state.shadows||[]){if(s.anchor){painter.draw(ctx,'effects','body_idle0',s.pos,0,P,W);continue;}
  const f=foe(s.pos),left=f?f.pos.x<s.pos.x:s.pos.x>me.pos.x,age=tick-(s.until-R.cloneArmy.lifeTicks);
  sheetDraw(c40Art,(age<8?'clone_in':'clone')+(left?'_l':''),s.pos,Math.max(0,age));}
 for(const c of state.casts||[]){
  if(c.type==='grab'&&c.grab.phase==='flight'){const g=c.grab,facing=g.at.x<g.from.x?'_l':'';painter.draw(ctx,'effects','body_run'+Math.floor(tick/4)%6+facing,g.cursor,0,P,W);
   let ahead=toward(g.cursor,g.at,14000);if(dist(ahead,g.cursor)<1)ahead=toward(g.cursor,{x:g.cursor.x+Math.sign(g.at.x-g.from.x||1),y:g.cursor.y},14000);
   grabDraw('hand_d'+grabHeading(g.from,g.at),{x:ahead.x,y:ahead.y-3000},0);}
  if(c.type==='grab'&&c.grab.phase==='pull'){const from=casterPos(c.caster),victim=actor(c.grab.target);if(from&&victim){const rb=grabRibbon(from,victim.pos);grabDraw(rb.tag,rb.at,0);}}
  if(c.type==='smother'){const t=smotherTrail(c.from,c.cursor,c.caster?.by!=='me');if(t)sheetDraw(c40Art,t.tag,t.at,0);}
 }
}
function shadowReadout(){
 const who=id=>actor(id)?.name||'#'+id,casts=state.casts||[];
 const lines=casts.map(c=>{const by=c.caster?.by==='me'?'Contender':'shadow '+c.caster?.id;
  if(c.type==='grab')return by+' · grab '+c.grab.phase+(c.grab.target?' · '+who(c.grab.target):'');
  if(c.type==='smother')return by+' · Smother · hit '+c.hit.length;
  return by+' · Shadowborne flurry · '+c.done+' punches';});
 const anchor=(state.shadows||[]).find(s=>s.anchor),clones=(state.shadows||[]).filter(s=>!s.anchor).length;
 lines.unshift('Anchor '+(anchor?'placed':'—')+' · clones '+clones);
 return [...lines,lastCounter].filter(Boolean).join('\n');
}
function draw(){if(!me)return;if($('follow').checked)view={...me.pos};ctx.clearRect(0,0,900,900);ctx.imageSmoothingEnabled=false;world.drawMap(ctx,P,W);
  for(const e of fx)if($('showArt').checked){if(e.art)sheetDraw(e.art,e.tag,e.at,tick-e.start);else painter.draw(ctx,e.group||'effects',e.tag,e.at,tick-e.start,P,W);}
 for(const u of units.filter(u=>u.visible||u.team===0)){targets.draw(ctx,u,tick,P,W);if(u.hp>0){const p=P(u.pos);ctx.fillStyle='#251a2f';ctx.fillRect(p.x-18,p.y-W(31000),36,3);ctx.fillStyle=u.team===0?'#85dcbb':'#f18386';ctx.fillRect(p.x-18,p.y-W(31000),36*u.hp/u.max_hp,3);if(u.team!==0){ctx.fillStyle='#eee5fb';ctx.font='11px monospace';ctx.fillText('#'+u.id+(u.unlimitedHP?' ∞':''),p.x-18,p.y-W(35000));}}}
 drawShadow();
 if($('showArt').checked)painter.body(ctx,me,tick,P,W,false,null);world.drawMap(ctx,P,W,true);drawRhythmStacks();
 const p=P(me.pos);ctx.fillStyle='#2b183b';ctx.fillRect(p.x-20,p.y-W(34000),40,5);ctx.fillStyle='#9be2b9';ctx.fillRect(p.x-20,p.y-W(34000),40*me.hp/me.max_hp,5);ctx.fillStyle='#c7a0ff';ctx.font='13px monospace';ctx.fillText('◆'.repeat(Math.ceil((state.charges??R.charges.max)/R.charges.pipShadows))+'◇'.repeat(Math.max(0,R.charges.max/R.charges.pipShadows-Math.ceil((state.charges??R.charges.max)/R.charges.pipShadows))),p.x-25,p.y-W(39000));
 if($('ranges').checked){ctx.strokeStyle='#bca1e5';for(const radius of[R.shadowGrab.range,R.anchor.range,R.smother.range]){ctx.beginPath();ctx.arc(p.x,p.y,W(radius),0,Math.PI*2);ctx.stroke();}}
 const a=P(aim);ctx.strokeStyle='#fff';ctx.beginPath();ctx.moveTo(a.x-6,a.y);ctx.lineTo(a.x+6,a.y);ctx.moveTo(a.x,a.y-6);ctx.lineTo(a.x,a.y+6);ctx.stroke();
 const seconds=at=>(Math.max(0,(at||0)-tick)/60).toFixed(2);
 const dodges=state.charges??R.charges.max;
 $('pips').textContent='Shadows '+dodges+'/'+R.charges.max+'  '+'◆'.repeat(dodges)+'◇'.repeat(R.charges.max-dodges)+'  (a dodge costs 1)';
 $('state').textContent=[
  'Tick '+tick+' · HP '+Math.round(me.hp)+'/'+me.max_hp,
  'Grab '+seconds(state.grab_at)+' s · Anchor '+seconds(state.anchor_at)+' s · Smother '+seconds(state.smother_at)+' s · Clone Army '+seconds(state.ult_at)+' s',
  'Shadows '+dodges+'/'+R.charges.max+' · next shadow '+seconds(state.charge_at)+' s · shield '+Math.round(shieldOf(me))+' · haste '+(flags.haste?'on':'off'),
  'Rhythm '+Math.max(0,...(state.stacks||[]).map(s=>s.count)),
  state.reason||'Manual controls',
  'Requested '+scene.metrics.requested+' · HP '+scene.metrics.hp_damage+' · shields '+scene.metrics.shield_damage,
  'Map '+world.geometry.registration.matching+'/900 cells · 1,000 units/pixel · FX alive '+fx.length
 ].join('\n');
 $('shadowStatus').textContent=shadowReadout();
 panel?.update(tick);
}
const orderFor=k=>({q:{kind:'s1',aim:round(aim)},e:{kind:'s2',aim:round(aim)},f:{kind:'s3',aim:round(aim)},r:{kind:'ult',aim:round(aim)}})[k];
function next(){if(loading)return;const o=queued;queued=null;stepWorld(o);}
function clearHeld(){held.clear();queued=null;clock.reset();}
canvas.addEventListener('pointerdown',e=>{canvas.focus();const r=canvas.getBoundingClientRect(),x=(e.clientX-r.left)*900/r.width,y=(e.clientY-r.top)*900/r.height;aim={x:Math.max(0,Math.min(960000,view.x+(x-450)/zoom)),y:Math.max(0,Math.min(960000,view.y+(y-450)/zoom))};draw();});
canvas.addEventListener('keydown',e=>{const k=e.key.toLowerCase();if(!['w','a','s','d','q','e','f','r',' '].includes(k))return;e.preventDefault();held.add(k);if(!e.repeat){const order=orderFor(k);if(order)queued=order;}running=true;});
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
