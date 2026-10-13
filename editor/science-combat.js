/* Unified Theory practice model. Native unified_theory.rs is the rules source.
 * No DOM, artwork, wall clock or game persistence is used by this simulation.
 * Cursor aim is a laboratory override; target lock follows native release rules.
 */
(function(root){
'use strict';
const D=typeof module!=='undefined'?require('./unified-theory-data.js'):root.ScienceData;
const Charges=typeof module!=='undefined'?require('./science-charge-model.js'):root.ScienceCharges;
const MathRules=typeof module!=='undefined'?require('./unified-theorylab.js'):root.ScienceLab;
const TPS=60,UPX=950,WIDTH=330*UPX,HEIGHT=210*UPX;
const clamp=(v,a,b)=>Math.max(a,Math.min(b,v)),tr=Math.trunc;
const d2=(a,b)=>(a.x-b.x)**2+(a.y-b.y)**2;
function unit(dx,dy){
 dx=tr(clamp(dx,-2000000,2000000));dy=tr(clamp(dy,-2000000,2000000));
 const n=(BigInt(dx)*BigInt(dx)+BigInt(dy)*BigInt(dy))*100000000n;if(!n)return [0,0];
 let x=n,y=(x+1n)/2n;while(y<x){x=y;y=(x+n/x)/2n;}
 return [Number(BigInt(dx)*100000000n/x),Number(BigInt(dy)*100000000n/x)];
}
function near(a,b,p,r){const dx=b.x-a.x,dy=b.y-a.y,len=dx*dx+dy*dy,t=len?clamp(((p.x-a.x)*dx+(p.y-a.y)*dy)/len,0,1):0;return (p.x-a.x-tr(dx*t))**2+(p.y-a.y-tr(dy*t))**2<=r*r;}
const PACKET_SKILLS=[0,16,39,40,54,59,63,72,73,74];
const PHYSICAL=[16,29,34,39,40],HEAVY=[34,63,71,73,74],MANIPULATORS=[27,37,38,39,46];
const materialCost=s=>s<50?0:[63,71,73,74].includes(s)?2:[54,57,58,59,65,66,72].includes(s)?1:0;
function theory(){return {target:null,until:0,origin:null,vector:[0,0],velocity:[0,0],acceleration:[0,0],aim:null,sync:0,proper:false,cone:false,link:false,curve:0,blue:false,pressure:false,exchange:false,drag:0,scalar:0,orbit:false,elastic:false,root:false,boundary:false,work:0,series:0,impulse:0,sample:false,purity:0,acid:false,base:false,catalyst:false,order:0,label:false,half:0};}
class Window{
 constructor(){this.rows=[];}
 allow(tick,span,cap,n){this.rows=this.rows.filter(r=>tick-r[0]<span);const granted=Math.max(0,Math.min(n,cap-this.rows.reduce((n,r)=>n+r[1],0)));if(granted)this.rows.push([tick,granted]);return granted;}
}
class Combat{
 constructor(options={}){this.reset(options);}
 reset(options={}){
  this.tick=0;this.rank=options.rank??6;this.position=options.position??10;this.persona=options.science??0;this.seed=options.seed??42;this.rng=BigInt(this.seed)||1n;
  this.bank=new Charges.ChargeBank();this.material=8;this.momentum=0;this.theory=theory();this.plan=null;this.nextId=0;
  this.packets=[];this.deliveries=[];this.fields=[];this.anchors=[];this.incoming=[];this.visuals=[];this.events=[];this.sequence=0;this.vfxLeft=6;
  this.budgets=new Map();this.damage=new Map();this.controls=new Map();this.moves=new Map();this.support=new Window();this.observations=new Map();
  this.hero={id:0,x:90000,y:105000,hp:1000,maxhp:1000,ap:400,attack:300,speed:1000,face:1,dest:null,buffs:{},shields:[],dead:false};
  this.dummies=[this.makeDummy(1,180000,105000),this.makeDummy(2,225000,145000),this.makeDummy(3,150000,55000)];
  this.aim={x:180000,y:105000};this.locked=1;this.motion=[0,0];this.releasePose=null;this.transformation=null;this.failureUntil=0;this.paused=false;
  this.options={moving:false,incoming:false,accuracy:false};this.event('reset','Practice ready · right-click to move; Space to cast');
 }
 makeDummy(id,x,y){return {id,x,y,home:{x,y},hp:1000,maxhp:1000,visible:true,buffs:{},shields:[],ccUntil:0,push:null,face:-1};}
 random(n){this.rng=BigInt.asUintN(64,this.rng^(this.rng<<13n));this.rng=BigInt.asUintN(64,this.rng^(this.rng>>7n));this.rng=BigInt.asUintN(64,this.rng^(this.rng<<17n));return Number(this.rng%BigInt(n));}
 event(type,text,extra={}){const e={seq:++this.sequence,tick:this.tick,type,text,...extra};this.events.push(e);if(this.events.length>200)this.events.shift();return e;}
 visual(tag,at,life=24,extra={}){if(this.vfxLeft<=0)return;if(tag.startsWith('impact_')&&this.visuals.some(v=>v.tag===tag&&v.x===at.x&&v.y===at.y&&v.until>this.tick))return;this.vfxLeft--;if(tag.startsWith('skill_'))this.visuals=this.visuals.filter(v=>!v.tag.startsWith('skill_'));else if(['complete','shield','fizzle'].includes(tag))this.visuals=this.visuals.filter(v=>v.tag!==tag);if(this.visuals.length>=32)this.visuals.shift();this.visuals.push({tag,x:at.x,y:at.y,born:this.tick,until:this.tick+life,...extra});}
 target(){return this.dummies.find(d=>d.id===this.locked)||null;}
 setPersona(science){if(science!==this.persona){this.transformation={from:this.persona,to:science,born:this.tick,until:this.tick+24};this.persona=science;}}
 live(id){return this.dummies.find(d=>d.id===id&&d.hp>0&&d.visible)||null;}
 point(p){return {x:tr(clamp(p.x,0,WIDTH)),y:tr(clamp(p.y,0,HEIGHT))};}
 command(name,value){
  switch(name){
   case 'move':if(this.hero.hp>0)this.hero.dest=this.point(value);break;
   case 'stop':this.hero.dest=null;break;
   case 'aim':this.aim=this.point(value);break;
   case 'lock':{
    const id=value??null;if(id!==this.locked){this.locked=id;if(!this.plan&&id!==null&&id!==this.theory.target)this.theory={...theory(),target:id,origin:this.anchors.length?{x:this.anchors.at(-1).x,y:this.anchors.at(-1).y}:null};this.event('target',id===null?'Cursor aim':'Locked dummy '+id);}
    break;
   }
   case 'cast':return this.cast(value);
   case 'cancel':return this.cancel();
   case 'pause':this.paused=Boolean(value);break;
   case 'step':this.step(true);break;
   case 'moving':this.options.moving=Boolean(value);break;
   case 'incoming':this.options.incoming=Boolean(value);break;
   case 'accuracy':this.options.accuracy=Boolean(value);break;
   case 'health':{this.hero.hp=tr(clamp(value,0,this.hero.maxhp));if(!this.hero.hp)this.death();else this.hero.dead=false;break;}
   case 'respawn':this.hero.hp=this.hero.maxhp;this.hero.dead=false;this.event('respawn','Respawned · existing recovery deadlines retained');break;
   case 'dummyHealth':for(const d of this.dummies){d.hp=d.maxhp=tr(clamp(value,1,10000));}break;
   default:throw Error('Unknown ScienceCombat command: '+name);
  }
 }
 valid(s){const t=this.theory,v=t.vector[0]||t.vector[1];switch(s){
  case 7:case 24:return this.anchors.length>0;case 19:return this.anchors.length>=2;case 10:return t.cone;case 11:return Boolean(t.origin);case 17:return this.material>0;case 22:return t.impulse>0;
  case 27:case 28:case 29:case 30:case 32:case 33:return Boolean(v);case 34:return this.momentum>=10;
  case 37:case 38:case 39:case 46:return this.packets.length>0;case 43:return this.momentum>0;case 44:case 45:return Boolean(t.aim);case 47:return this.fields.some(f=>f.kind===57);case 49:return Boolean(t.origin&&v);
  case 51:case 52:case 69:return t.sample;case 56:return t.acid&&t.base;case 62:return this.fields.some(f=>[58,65,66].includes(f.kind));case 63:return t.sample&&t.label;case 70:return t.label;case 74:return t.label&&t.half>0;default:return true;
 }}
 preparation(steps,depth=0){
  if(depth>12)throw Error('Prerequisite cycle');const seen=[];const t=this.theory;const has=s=>seen.includes(s);
  for(const s of steps){const packet=this.packets.length>0||seen.some(s=>[0,16,40,54,59,63,72,73,74].includes(s));let needed=null;
   if([7,24].includes(s)&&!this.anchors.length&&!has(1))needed=1;
   else if(s===19&&this.anchors.length+seen.filter(s=>s===1).length<2)needed=1;
   else if(s===10&&!t.cone&&!has(9)&&!has(48))needed=9;
   else if([11,49].includes(s)&&!t.origin&&!has(1)&&!has(25))needed=25;
   else if(s===22&&!t.impulse&&!has(32)&&!has(49))needed=32;
   else if([27,28,29,30,32,33,49].includes(s)&&!t.vector.some(Boolean)&&!has(26))needed=26;
   else if(s===34&&this.momentum<10&&!has(32))needed=32;
   else if([37,38,39,46].includes(s)&&!packet)needed=0;
   else if(s===43&&!this.momentum&&!has(32))needed=32;
   else if([44,45].includes(s)&&!t.aim&&![42,23,3,48].some(has))needed=42;
   else if(s===47&&!this.fields.some(f=>f.kind===57)&&!has(57))needed=57;
   else if([51,52,63,69].includes(s)&&!t.sample&&!has(50))needed=50;
   else if(s===56&&!t.acid&&!has(54))needed=54;
   else if(s===56&&!t.base&&!has(55))needed=55;
   else if(s===62&&!this.fields.some(f=>[58,65,66].includes(f.kind))&&![58,65,66].some(has))needed=65;
   else if([63,70,74].includes(s)&&!t.label&&!has(69))needed=69;
   else if(s===74&&!t.half&&!has(70))needed=70;
   if(needed!==null)return this.preparation([needed],depth+1)??needed;seen.push(s);
  }return null;
 }
 needsTarget(s){return !PACKET_SKILLS.includes(s)&&![1,2,4,6,7,8,14,15,17,18,19,21,22,24,25,27,30,31,35,37,38,43,46,47,53,55,57,60,61,62,64,67].includes(s);}
 stages(value){return (Array.isArray(value)?value:[value]).map(v=>typeof v==='number'?{skill:v,charge:D.skills[v]?.charge.slice()}:{skill:v.skill,charge:v.charge?.slice()||D.skills[v.skill]?.charge.slice()});}
 availability(value){
  let stages;try{stages=this.stages(value);}catch(_){return {ok:false,reason:'Choose a skill'};}
  if(!stages.length||stages.length>5||stages.some(s=>!D.skills[s.skill]||s.charge.length!==D.skills[s.skill].tokens.length))return {ok:false,reason:'Choose one to five valid stages'};
  if(!this.hero.hp)return {ok:false,reason:'Scientist is dead; respawn to practice'};
  if(this.plan)return {ok:false,reason:'One experiment is already preparing'};
  const target=this.live(this.locked);if(this.locked!==null&&!target)return {ok:false,reason:'Locked target is hidden or dead'};
  if(stages.some(s=>this.needsTarget(s.skill))&&!target)return {ok:false,reason:'This recipe needs a locked target'};
  const at=target||this.aim;
  if(stages.some(s=>d2(this.hero,at)>D.skills[s.skill].range**2))return {ok:false,reason:'Aim is outside this recipe’s legal range'};
  const prerequisite=this.preparation(stages.map(s=>s.skill));if(prerequisite!==null)return {ok:false,prerequisite,reason:'Prepare '+D.skills[prerequisite].id+' · '+D.skills[prerequisite].name+' first (one charge)'};
  if(stages.reduce((n,s)=>n+materialCost(s.skill)+(s.skill===17?1:0),0)>this.material)return {ok:false,reason:'Not enough material'};
  if(stages.some(s=>PACKET_SKILLS.includes(s.skill)||[5,12,13,20,29,33,34,36,45,57,58,65,66,71].includes(s.skill))&&this.packets.length+this.deliveries.length>=8)return {ok:false,reason:'Eight owned outputs are already active'};
  if(!this.bank.available())return {ok:false,reason:'All three charges are reserved or recovering'};
  return {ok:true,reason:target?'Locked dummy '+target.id+' · aim refreshed at release':'Cursor aim · point captured on cast',stages};
 }
 cast(value){const a=this.availability(value);if(!a.ok){this.event('blocked',a.reason);return false;}
  const slot=this.bank.reserve(this.tick),id=++this.nextId,target=this.live(this.locked);
  if(target&&this.theory.target!==target.id){this.theory={...theory(),target:target.id,origin:this.anchors.length?{x:this.anchors.at(-1).x,y:this.anchors.at(-1).y}:null};}
  // Recheck prerequisites after changing the native calculation target.
  const prerequisite=this.preparation(a.stages.map(s=>s.skill));if(prerequisite!==null){this.bank.cancel(slot);this.event('blocked','New target needs '+D.skills[prerequisite].id+' first');return false;}
  this.plan={id,slot,power:this.hero.ap,physical:this.hero.attack,target:target?.id??null,aim:{...this.aim},stages:a.stages,index:0,token:0,ready:this.tick+6,started:this.tick,committed:false,error:false};
  this.budgets.set(id,160+tr(Math.max(this.hero.ap,this.hero.attack)*80/100));
  this.event('reserve','Reserved charge '+(slot+1)+' · '+a.reason,{experiment:id,slot,target:target?.id??null});return true;
 }
 cancel(reason='Preparation canceled'){
  if(!this.plan)return false;const p=this.plan;
  if(!p.committed){this.bank.cancel(p.slot);this.event('cancel',reason+' · reserved charge refunded');}
  else this.event('cancel',reason+' · spent charge continues its five-second recovery');
  this.plan=null;return true;
 }
 death(){if(this.hero.dead)return;this.cancel('Scientist died');this.hero.dead=true;this.hero.dest=null;this.packets=[];this.deliveries=[];this.fields=[];this.anchors=[];this.theory=theory();this.visuals=[];this.momentum=0;this.damage.clear();this.controls.clear();this.moves.clear();this.observations.clear();this.budgets.clear();this.releasePose=null;this.failureUntil=0;this.hero.buffs={};this.hero.shields=[];this.event('death','Scientist died · spent charges retain their deadlines');}
 allow(map,id,span,cap,n){if(!map.has(id))map.set(id,new Window());return map.get(id).allow(this.tick,span,cap,n);}
 emit(target,amount,physical,exp,skill){
  if(!target||!target.hp)return 0;let n=Math.min(amount,this.budgets.get(exp)||0);n=this.allow(this.damage,target.id,120,tr(target.maxhp*35/100),n);if(!n)return 0;
  this.budgets.set(exp,this.budgets.get(exp)-n);const before=target.hp;target.hp=Math.max(0,target.hp-n);
  this.event('hit',D.skills[skill].id+' hit dummy '+target.id+' · '+n+' raw '+(physical?'physical':'magic')+' · HP '+before+' → '+target.hp,{skill,target:target.id,x:target.x,y:target.y,raw:n,experiment:exp});return n;
 }
 shield(n){n=this.support.allow(this.tick,180,tr(this.hero.maxhp/4),n);if(n){this.hero.shields.push({amount:n,until:this.tick+180});this.visual('shield',this.hero,18);this.event('shield','Shield +'+n,{amount:n});}return n;}
 buff(entity,tag,ticks,stats={}){if(!entity?.hp)return;entity.buffs[tag]={until:this.tick+Math.min(ticks,360),...stats};this.event('buff',tag+' · '+(entity.id?'dummy '+entity.id:'scientist'),{target:entity.id,tag});}
 cc(e,ticks){if(!e?.hp)return;const n=this.allow(this.controls,e.id,180,60,ticks);if(n){e.ccUntil=Math.max(e.ccUntil,this.tick+n);this.event('control','Dummy '+e.id+' controlled for '+n+' ticks',{target:e.id,ticks:n});}}
 push(e,pull=false,at=this.hero,ticks=12){if(!e?.hp||!this.allow(this.moves,e.id,120,2,1))return;const n=this.allow(this.controls,e.id,180,60,ticks);if(!n)return;const [dx,dy]=unit(e.x-at.x,e.y-at.y);e.push={dx:tr(dx*(pull?-1000:1000)/10000),dy:tr(dy*(pull?-1000:1000)/10000),until:this.tick+n};this.event('push',(pull?'Pull':'Push')+' dummy '+e.id,{target:e.id,ticks:n});}
 forecast(target,v,a=[0,0],h=12,k=0){return {x:clamp(target.x+clamp(v[0]*h+a[0]*k,-30000,30000),0,960000),y:clamp(target.y+clamp(v[1]*h+a[1]*k,-30000,30000),0,960000)};}
 freshAim(target){const v=this.observations.get(target.id)?.velocity||[0,0],h=clamp(tr(Math.max(Math.abs(target.x-this.hero.x),Math.abs(target.y-this.hero.y))/3600),1,30);return this.forecast(target,v,[0,0],h);}
 release(skill,target,at){this.releasePose={science:D.skills[skill].science,tick:this.tick};this.visual('skill_'+D.skills[skill].id,this.hero,36,{follow:true});this.event('release',D.skills[skill].id+' released · '+(target===null?'cursor':'dummy '+target)+' · aim ('+at.x+', '+at.y+')',{skill,target,origin:{x:this.hero.x,y:this.hero.y},aim:{...at}});}
 impact(skill,at){this.visual('impact_'+D.skills[skill].id,at,24);this.event('impact',D.skills[skill].id+' contact at ('+at.x+', '+at.y+')',{skill,x:at.x,y:at.y});}
 field(s,at,exp,payload=0){const limit=[13,57].includes(s)?2:1;if(this.fields.filter(f=>f.kind===s).length>=limit)this.fields.splice(this.fields.findIndex(f=>f.kind===s),1);
  if([58,65,66].includes(s)&&this.fields.filter(f=>[58,65,66].includes(f.kind)).length>=2)this.fields.splice(this.fields.findIndex(f=>[58,65,66].includes(f.kind)),1);
  if(this.fields.length>=12)return;this.fields.push({kind:s,x:at.x,y:at.y,experiment:exp,payload,born:this.tick,expires:this.tick+360,hp:s===57?120:null});this.event('field',D.skills[s].id+' deployed · '+(s===57?'crystal':'field'),{skill:s,x:at.x,y:at.y});
 }
 packet(s,at,p,n,physical=false){const t=this.theory,aim=t.aim||at,[dx,dy]=unit(aim.x-this.hero.x,aim.y-this.hero.y),split=Math.min(s===72?3:clamp(Math.max(t.scalar,t.series),1,4),8-this.packets.length-this.deliveries.length);
  for(let i=0;i<split;i++)this.packets.push({skill:s,experiment:p.id,x:this.hero.x,y:this.hero.y,dx,dy,payload:tr(n/split)+(i<n%split?1:0),physical,class:physical?2:[54,59,63].includes(s)?3:[71,72,74].includes(s)?1:0,expires:this.tick+180,release:this.tick+Math.max(t.sync,12)+t.order*i,age:0,split,curve:s===40?250:t.curve,orbit:t.orbit,hits:[],launch:{target:p.target,aim:{...p.aim},baseline:[dx,dy],drift:this.launchDrift},born:this.tick});
 }
 deliver(s,p,destination,payload=0,contact=false,placement=D.skills[s].placement){if(this.packets.length+this.deliveries.length>=8)return;this.deliveries.push({skill:s,experiment:p.id,target:p.target,x:this.hero.x,y:this.hero.y,destination:{...destination},placement,contact,payload,release:this.tick+12,expires:this.tick+180,launched:false,born:this.tick});}
 split(parts){const originals=this.packets;this.packets=[];const limit=8-this.deliveries.length,count=Math.min(originals.length,limit);
  originals.slice(0,count).forEach((q,index)=>{const n=Math.min(parts,Math.max(1,limit-this.packets.length-count+index+1));for(let i=0;i<n;i++)this.packets.push({...q,hits:q.hits.slice(),launch:q.launch?{...q.launch,baseline:q.launch.baseline.slice(),aim:{...q.launch.aim}}:null,payload:tr(q.payload/n)+(i<q.payload%n?1:0),curve:q.curve+(i-tr(n/2))*100,split:n});});
 }
 execute(s,p,power=100,instability=0){
  if(!this.valid(s)||materialCost(s)>this.material)return false;
  const t=this.theory,m=this.hero,target=this.live(p.target),at=target||p.aim,[dx,dy]=unit(at.x-m.x,at.y-m.y);
  this.material-=materialCost(s);this.setPersona(D.skills[s].science);t.target=p.target;t.until=this.tick+1200;
  if(p.stages.slice(p.index+1).some(st=>MANIPULATORS.includes(st.skill))&&[0,16,40,54,59,63,72,73,74].includes(s))t.sync=Math.max(t.sync,12);
  const ap=p.power??m.ap,ad=p.physical??m.attack,stat=PHYSICAL.includes(s)?ad:ap;let n=tr((HEAVY.includes(s)?90+tr(stat*45/100):50+tr(stat/4))*power/100);
  if(PHYSICAL.includes(s)){const work=Math.min(t.work,20);t.work-=work;n+=work;}
  this.launchDrift=instability>40?20000:instability>20?5000:0;
  switch(s){
   case 0:this.packet(s,at,p,n);break;
   case 1:if(this.anchors.length>=3)this.anchors.shift();this.anchors.push({x:m.x,y:m.y,expires:this.tick+960,born:this.tick});t.origin={x:m.x,y:m.y};break;
   case 2:t.sync=30;for(const q of this.packets){q.release=this.tick+30;q.orbit=false;}break;
   case 3:{const v=this.observations.get(p.target)?.velocity||[0,0];t.velocity=[v[0]-this.motion[0],v[1]-this.motion[1]];t.aim=this.forecast(at,t.velocity);break;}
   case 4:t.proper=true;break;
   case 5:this.deliver(s,p,at);break;
   case 6:this.buff(m,'contract',180,{radius:-20});break;
   case 7:{const a=this.anchors.at(-1);Object.assign(m,this.point({x:clamp(a.x+dx,m.x-20000,m.x+20000),y:clamp(a.y+dy,m.y-20000,m.y+20000)}));break;}
   case 8:for(const q of this.packets){q.release=this.tick+12;q.orbit=false;}t.sync=12;break;
   case 9:t.cone=true;t.aim=this.forecast(at,t.velocity,[0,0],18);break;
   case 10:t.link=true;break;
   case 11:t.curve=180;break;
   case 12:this.deliver(s,p,at);break;
   case 13:this.deliver(s,p,{x:tr((m.x+at.x)/2),y:tr((m.y+at.y)/2)});break;
   case 14:this.shield(Math.min(120,35+tr(ap/5)+Math.min(2,this.incoming.filter(q=>d2(q,m)<45000**2).length)*15));break;
   case 15:t.blue=true;break;
   case 16:this.packet(s,at,p,n,true);t.pressure=true;break;
   case 17:this.material--;t.exchange=true;this.shield(40+tr(ap/4));break;
   case 18:t.drag=120;this.field(s,m,p.id);break;
   case 19:{const a=this.anchors[0],b=this.anchors.at(-1),ids=this.dummies.filter(e=>e.hp&&e.visible&&near(a,b,e,12000));ids.forEach((e,i)=>{this.emit(e,tr(n/Math.max(1,ids.length))+(i<n%ids.length?1:0),false,p.id,s);this.impact(s,e);});break;}
   case 20:this.deliver(s,p,at);break;
   case 21:if(!t.aim||d2(m,t.aim)<d2(m,at))this.shield(60+tr(ap/3));break;
   case 22:this.deliver(12,p,{x:m.x+dx,y:m.y+dy},0,false,'forward');t.impulse=0;break;
   case 23:t.aim=this.forecast(at,t.velocity,t.acceleration,12,36);break;
   case 24:t.curve=100*Math.min(3,this.anchors.length);t.cone=true;t.sync=6;t.aim={x:at.x,y:at.y};break;
   case 25:t.origin={x:m.x,y:m.y};break;
   case 26:t.vector=m.hp*5<m.maxhp*2?[-dx,-dy]:[dx,dy];break;
   case 27:t.scalar=2;this.split(2);break;
   case 28:t.vector=unit(t.vector[0]+dx,t.vector[1]+dy);break;
   case 29:{if(d2(m,at)>60000**2)return false;const alignment=clamp(tr((t.vector[0]*dx+t.vector[1]*dy)/10000),0,10000);this.deliver(s,p,at,tr(n*alignment/10000),true);break;}
   case 30:t.vector=[-t.vector[1],t.vector[0]];t.curve=-250;break;
   case 31:this.buff(m,'inertia',180,{speed:10});break;
   case 32:t.impulse=Math.min(this.momentum,30);this.momentum=Math.min(100,this.momentum+10);this.buff(m,'force',90,{speed:15});break;
   case 33:this.deliver(s,p,at,0,true);break;
   case 34:{if(d2(m,at)>30000**2)return false;const usable=Math.min(this.momentum,50);this.momentum-=usable;this.deliver(s,p,at,tr(n*usable/50),true);break;}
   case 35:{const v=m.hp*5<m.maxhp*2?[-dx,-dy]:t.vector.some(Boolean)?t.vector:[dx,dy];Object.assign(m,this.point({x:m.x+v[0],y:m.y+v[1]}));this.momentum=Math.max(0,this.momentum-10);break;}
   case 36:this.momentum=Math.max(0,this.momentum-20);this.deliver(s,p,at,0,true);break;
   case 37:t.elastic=true;for(const q of this.packets){q.dx=-q.dx;q.dy=-q.dy;q.hits=[];}break;
   case 38:for(const q of this.packets)q.orbit=true;t.orbit=true;break;
   case 39:for(const q of this.packets){q.orbit=false;q.release=this.tick;const [u,v]=unit(q.x-m.x,q.y-m.y);q.dx=-v;q.dy=u;}t.orbit=false;this.packet(s,at,p,n,true);break;
   case 40:this.packet(s,at,p,n,true);break;
   case 41:if(this.incoming.some(q=>d2(q,m)<=45000**2))this.shield(75+tr(ad/4));break;
   case 42:{const v=this.observations.get(p.target)?.velocity;if(v){t.acceleration=[v[0]-t.velocity[0],v[1]-t.velocity[1]];t.velocity=v.slice();}t.aim=this.forecast(at,t.velocity,[0,0],10);break;}
   case 43:t.work=Math.min(40,t.work+tr(this.momentum/4));this.momentum=Math.max(0,this.momentum-10);break;
   case 44:t.aim={x:tr((t.aim.x+at.x)/2),y:tr((t.aim.y+at.y)/2)};t.curve=tr(t.curve/2);break;
   case 45:this.deliver(s,p,at);break;
   case 46:t.series=3;this.split(3);break;
   case 47:t.boundary=true;break;
   case 48:t.aim=this.forecast(at,t.velocity,t.acceleration,15,60);t.cone=true;break;
   case 49:t.vector=[dx,dy];t.impulse=20;t.work=20;this.buff(m,'principia',180,{speed:10});break;
   case 50:t.sample=true;t.acid=Boolean(target?.buffs.acid);break;
   case 51:if(this.material>0){this.material--;t.purity=100;}break;
   case 52:this.shield(35+tr(Math.min(at.maxhp-at.hp,100)/3));break;
   case 53:this.buff(m,'buffer',180,{skillReduction:10,toughness:15});break;
   case 54:t.acid=true;this.packet(s,at,p,n);break;
   case 55:t.base=true;delete m.buffs.acid;this.shield(30+tr(ap/6));break;
   case 56:t.acid=false;t.base=false;if(target)delete target.buffs.acid;{const healed=Math.min(m.maxhp-m.hp,Math.min(80,25+tr(ap/8)));m.hp+=healed;this.visual('shield',m,24);this.event('heal','Neutralization heals '+healed,{amount:healed});}break;
   case 57:this.deliver(s,p,{x:m.x+dx*2,y:m.y+dy*2});break;
   case 58:this.deliver(s,p,at);break;
   case 59:this.packet(s,at,p,n);break;
   case 60:delete m.buffs.oxidize;this.buff(m,'reduce',180,{mr:10});break;
   case 61:t.catalyst=true;break;
   case 62:this.fields=this.fields.filter(f=>![58,65,66].includes(f.kind));this.material=Math.min(12,this.material+1);break;
   case 63:this.packet(s,at,p,tr(n*(t.acid&&t.base?100:70)/100));break;
   case 64:t.order=20;break;
   case 65:this.deliver(s,p,at,n);break;
   case 66:this.deliver(s,p,at);break;
   case 67:this.shield(80+tr(ap/4));this.buff(m,'membrane',120,{skillReduction:5});break;
   case 68:this.buff(target,'tracer',360);t.aim={x:at.x,y:at.y};break;
   case 69:t.label=true;break;
   case 70:t.half=3;break;
   case 71:if(d2(m,at)<=30000**2)this.deliver(s,p,at,n,true);break;
   case 72:this.packet(s,at,p,n);break;
   case 73:this.packet(s,at,p,n);break;
   case 74:{const parts=Math.min(t.half,8-this.packets.length-this.deliveries.length),scalar=t.scalar,series=t.series;t.scalar=t.series=1;for(let i=0;i<parts;i++){this.packet(s,at,p,tr(n/parts)+(i<n%parts?1:0));this.packets.at(-1).release=this.tick+12+i*30;}t.scalar=scalar;t.series=series;break;}
   default:throw Error('Missing native handler '+s);
  }
  if(!this.packets.some(q=>q.experiment===p.id&&q.skill===s&&q.launch)&&!this.deliveries.some(q=>q.experiment===p.id&&!q.launched))this.release(s,p.target,m);
  return true;
 }
 stepNotebook(){const p=this.plan;if(!p)return;const st=p.stages[p.index],s=st.skill,target=this.live(p.target),at=target||p.aim;this.setPersona(D.skills[s].science);
  if((p.target!==null&&!target)||this.tick-p.started>480){this.cancel('Preparation became invalid');return;}
  if(this.tick<p.ready)return;
  if(p.token<D.skills[s].tokens.length){if(this.options.accuracy&&this.random(10000)<[1200,800,600,400,200,100,30,10][this.rank])p.error=true;p.token++;p.ready=this.tick+(p.token===D.skills[s].tokens.length?12:this.theory.catalyst&&D.skills[s].science===2?8:10);return;}
  const waiting=s===47?this.deliveries.some(q=>q.skill===57):s===62?this.deliveries.some(q=>[58,65,66].includes(q.skill)):false;
  if(!this.valid(s)&&waiting){p.ready=this.tick+1;return;}
  if(!this.valid(s)||d2(this.hero,at)>D.skills[s].range**2){this.cancel('Prerequisite or range became invalid');return;}
  if(!p.committed){this.bank.commit(p.slot,this.tick);p.committed=true;this.event('spent','Charge '+(p.slot+1)+' spent · ready at '+((this.tick+300)/60).toFixed(2)+'s',{slot:p.slot,recovery:this.tick+300});}
  const error=p.error&&this.random(100)>=[15,25,40,55,70,85,96,100][this.rank];let instability=MathRules.instability(D.skills[s].charge,st.charge)??100;
  if(D.skills[s].science===2&&this.theory.purity===100){this.theory.purity=0;instability=Math.max(0,instability-10);}
  const accepted=!error&&instability<=70&&this.execute(s,p,MathRules.strength(D.skills[s].charge,st.charge),instability);
  this.event('commit',D.skills[s].id+(accepted?' committed':' failed')+' · instability '+instability,{skill:s,accepted,experiment:p.id});
  if(!accepted){this.failureUntil=this.tick+24;this.visual('fizzle',this.hero,15);this.cancel('Commitment failed');return;}
  p.index++;if(p.index===p.stages.length){if(p.stages.length>=2)this.visual('complete',this.hero,48,{science:D.skills[s].science,follow:true});this.event('complete','Experiment complete · one shared charge spent',{experiment:p.id});this.plan=null;}
  else{p.token=0;p.error=false;p.ready=this.tick+6;p.started=this.tick;}
 }
 stepDeliveries(){const pending=[];for(const q of this.deliveries){if(q.expires<=this.tick)continue;if(q.release>this.tick){pending.push(q);continue;}
  if(!q.launched){const target=this.live(q.target);if(['enemy','midpoint'].includes(q.placement)&&(!target||d2(this.hero,target)>D.skills[q.skill].range**2)){this.event('discard',D.skills[q.skill].id+' launch discarded · invalid target');continue;}
   const old={x:q.x,y:q.y};q.x=this.hero.x;q.y=this.hero.y;
   if(q.placement==='enemy')q.destination=this.freshAim(target);
   else if(q.placement==='midpoint')q.destination={x:tr((this.hero.x+target.x)/2),y:tr((this.hero.y+target.y)/2)};
   else if(q.placement==='forward'){const at=target||q.destination,[dx,dy]=unit(at.x-this.hero.x,at.y-this.hero.y),scale=q.skill===57?2:1;q.destination={x:this.hero.x+dx*scale,y:this.hero.y+dy*scale};}
   else if(q.placement==='self')q.destination={x:this.hero.x,y:this.hero.y};
   else q.destination={x:q.destination.x+this.hero.x-old.x,y:q.destination.y+this.hero.y-old.y};
   q.launched=true;this.release(q.skill,q.target,q.destination);
  }
  if(q.contact){const e=this.live(q.target);if(e&&d2(this.hero,e)<=D.skills[q.skill].range**2){if([29,34,71].includes(q.skill))this.emit(e,q.payload,PHYSICAL.includes(q.skill),q.experiment,q.skill);else if(q.skill===33){this.push(e);this.buff(this.hero,'recoil',60,{speed:-10});}else if(q.skill===36)this.buff(e,'inelastic',90,{speed:-15});this.impact(q.skill,e);}continue;}
  if(d2(q,q.destination)<=3600**2){if(q.skill===45)this.theory.root=true;this.field(q.skill,q.destination,q.experiment,q.payload);this.impact(q.skill,q.destination);continue;}
  const [dx,dy]=unit(q.destination.x-q.x,q.destination.y-q.y),desired={x:q.x+tr(dx*3600/10000),y:q.y+tr(dy*3600/10000)},clipped=this.point(desired);
  if(d2(desired,clipped)){this.impact(q.skill,clipped);continue;}Object.assign(q,desired);q.dx=dx;q.dy=dy;pending.push(q);
 }this.deliveries=pending;}
 stepPackets(){const t=this.theory,m=this.hero;for(const q of this.packets){if(q.expires<=this.tick||q.release>this.tick)continue;
  if(q.launch){const l=q.launch,target=l.target===null?null:this.live(l.target);if(l.target!==null&&(!target||d2(m,target)>D.skills[q.skill].range**2)){q.expires=this.tick;this.event('discard',D.skills[q.skill].id+' launch discarded · invalid target');continue;}
   const aim=target?this.freshAim(target):l.aim;if(l.target===null&&d2(m,aim)>D.skills[q.skill].range**2){q.expires=this.tick;this.event('discard',D.skills[q.skill].id+' launch discarded · cursor point is out of range');continue;}
   let [u,v]=unit(aim.x-m.x,aim.y-m.y);const dot=tr((l.baseline[0]*q.dx+l.baseline[1]*q.dy)/10000),cross=tr((l.baseline[0]*q.dy-l.baseline[1]*q.dx)/10000);
   [u,v]=unit(tr(u*dot/10000)-tr(v*cross/10000),tr(v*dot/10000)+tr(u*cross/10000));if(!u&&!v)[u,v]=unit(aim.x-m.x,aim.y-m.y);
   [q.dx,q.dy]=unit(u-tr(v*l.drift/100000),v+tr(u*l.drift/100000));q.x=m.x;q.y=m.y;q.launch=null;this.release(q.skill,l.target,aim);
  }
  if(q.orbit){let [x,y]=unit(q.x-m.x,q.y-m.y);if(!x&&!y)x=10000;const old={x:q.x,y:q.y};q.x=m.x+x*2-tr(y/5);q.y=m.y+y*2+tr(x/5);q.artMotion=[q.x-old.x,q.y-old.y];continue;}
  const speed=this.fields.some(f=>f.kind===5&&d2(q,f)<=35000**2)?1800:t.proper?4000:3600,old={x:q.x,y:q.y};
  const turn=q.curve+this.fields.filter(f=>f.kind===18&&d2(q,f)<40000**2).length*120;if(turn)[q.dx,q.dy]=unit(q.dx-tr(q.dy*turn/10000),q.dy+tr(q.dx*turn/10000));
  if(q.class===0)for(const f of this.fields.filter(f=>f.kind===13&&d2(q,f)<18000**2))[q.dx,q.dy]=unit(q.dx+tr((f.x-q.x)/4),q.dy+tr((f.y-q.y)/4));
  const desired={x:q.x+tr(q.dx*speed/10000),y:q.y+tr(q.dy*speed/10000)},clipped=this.point(desired),surface=this.fields.some(f=>f.kind===57&&near(q,desired,f,8000));
  if(d2(desired,clipped)||surface&&t.boundary){if(t.elastic||t.boundary){q.dx=-q.dx;q.dy=-q.dy;}else{q.expires=this.tick;continue;}}
  Object.assign(q,clipped);q.age++;if(!t.blue&&q.age%30===0)q.payload=tr(q.payload*95/100);
  const hits=this.dummies.filter(e=>e.hp&&e.visible&&!q.hits.includes(e.id)&&near(old,q,e,10000)).sort((a,b)=>a.id-b.id),count=q.skill===73?Math.max(1,hits.length):1;
  for(const e of hits.slice(0,count)){const n=tr(q.payload/count);this.emit(e,n,q.physical,q.experiment,q.skill);if(q.skill===54)this.buff(e,'acid',180,{defence:-8});if(q.skill===59)this.buff(e,'oxidize',180,{healReduction:15});this.impact(q.skill,e);q.hits.push(e.id);q.payload=Math.max(0,q.payload-n);if(t.pressure)this.push(e);if(t.link){t.link=false;this.cc(e,12);}if(q.skill!==73){q.expires=this.tick;break;}}
  if(!q.payload)q.expires=this.tick;q.artMotion=[q.dx,q.dy];
 }this.packets=this.packets.filter(q=>q.expires>this.tick).slice(0,8);}
 stepFields(){if(this.tick%12)return;for(const f of this.fields)for(const e of this.dummies.filter(e=>e.hp&&e.visible&&d2(e,f)<=30000**2)){
  switch(f.kind){case 5:this.buff(e,'dilation',24,{speed:-12});break;case 12:case 66:this.push(e,true,f,10);break;case 20:if(d2(e,f)>=24000**2)this.buff(e,'horizon',24,{speed:-15});break;case 45:if(this.theory.root){this.cc(e,18);this.theory.root=false;}break;case 58:if(this.theory.label||this.theory.sample)this.cc(e,12);break;case 65:if(f.payload>0){const n=this.emit(e,Math.max(1,tr(f.payload/10)),false,f.experiment,65);if(n)this.impact(65,e);}break;default:break;}
 }}
 stepIncoming(){if(this.hero.hp&&this.options.incoming&&this.tick%90===0){const e=this.dummies.find(e=>e.hp&&e.visible);if(e){const [dx,dy]=unit(this.hero.x-e.x,this.hero.y-e.y);this.incoming.push({x:e.x,y:e.y,dx,dy,expires:this.tick+120});}}
  this.incoming=this.incoming.filter(q=>{const old={x:q.x,y:q.y};q.x+=tr(q.dx*1800/10000);q.y+=tr(q.dy*1800/10000);if(near(old,q,this.hero,8000)&&this.hero.hp){let damage=20;for(const s of this.hero.shields){const n=Math.min(s.amount,damage);s.amount-=n;damage-=n;}const reduction=Object.values(this.hero.buffs).reduce((n,b)=>n+(b.skillReduction||0),0);damage=tr(damage*(100-clamp(reduction,0,100))/100);this.hero.hp=Math.max(0,this.hero.hp-damage);this.visual('impact_E01',this.hero,24);this.event('incoming','Practice projectile · '+damage+' HP damage');return false;}return q.expires>this.tick;});
 }
 step(force=false){if(this.paused&&!force)return;this.tick++;this.vfxLeft=6;const before=this.bank.cells(this.tick-1);this.bank.advance(this.tick);this.bank.cells(this.tick).forEach((c,i)=>{if(c===10&&before[i]<10)this.event('recovered','Charge '+(i+1)+' recovered',{slot:i});});
  const m=this.hero,old={x:m.x,y:m.y};for(const e of [m,...this.dummies]){for(const [tag,b]of Object.entries(e.buffs))if(b.until<=this.tick)delete e.buffs[tag];e.shields=e.shields.filter(s=>s.until>this.tick&&s.amount>0);}
  if(this.tick%180===0)this.material=Math.min(12,this.material+1);
  if(m.hp&&m.dest){const speed=tr(m.speed*(100+Object.values(m.buffs).reduce((n,b)=>n+(b.speed||0),0))/100),[dx,dy]=unit(m.dest.x-m.x,m.dest.y-m.y);if(d2(m,m.dest)<=speed**2){Object.assign(m,m.dest);m.dest=null;}else Object.assign(m,this.point({x:m.x+tr(dx*speed/10000),y:m.y+tr(dy*speed/10000)}));}
  this.motion=[m.x-old.x,m.y-old.y];if(d2(old,m)>200**2&&d2(old,m)<=3000**2)this.momentum=Math.min(100,this.momentum+1);else if(this.tick%30===0)this.momentum=Math.max(0,this.momentum-1);
  const aim=this.target()||this.aim;if(aim.x!==m.x)m.face=aim.x>=m.x?1:-1;
  for(const e of this.dummies){if(!e.hp)continue;if(e.push&&e.push.until>this.tick)Object.assign(e,this.point({x:e.x+e.push.dx,y:e.y+e.push.dy}));else if(this.options.moving&&e.ccUntil<=this.tick){const phase=this.tick/80+e.id,at={x:tr(e.home.x+Math.sin(phase)*18000),y:tr(e.home.y+Math.cos(phase*.7)*16000)},speed=Math.max(0,tr(700*(100+Object.values(e.buffs).reduce((n,b)=>n+(b.speed||0),0))/100)),[dx,dy]=unit(at.x-e.x,at.y-e.y);if(d2(e,at)<=speed**2)Object.assign(e,at);else Object.assign(e,this.point({x:e.x+tr(dx*speed/10000),y:e.y+tr(dy*speed/10000)}));}}
  if(this.tick%6===0)for(const e of this.dummies.filter(e=>e.hp&&e.visible)){const old=this.observations.get(e.id)?.at||e;this.observations.set(e.id,{at:{x:e.x,y:e.y},velocity:[tr((e.x-old.x)/6),tr((e.y-old.y)/6)]});}
  this.anchors=this.anchors.filter(a=>a.expires>this.tick);this.fields=this.fields.filter(f=>f.expires>this.tick&&(f.hp===null||f.hp>0));this.visuals=this.visuals.filter(v=>v.until>this.tick);if(this.theory.until<this.tick)this.theory=theory();
  if(m.hp){this.stepDeliveries();this.stepPackets();this.stepFields();this.stepNotebook();}this.stepIncoming();if(!m.hp)this.death();
  for(const id of this.budgets.keys())if(this.plan?.id!==id&&!this.packets.some(q=>q.experiment===id)&&!this.deliveries.some(q=>q.experiment===id)&&!this.fields.some(f=>f.experiment===id))this.budgets.delete(id);
 }
 snapshot(){return JSON.parse(JSON.stringify({tick:this.tick,rank:this.rank,position:this.position,persona:this.persona,hero:this.hero,dummies:this.dummies,locked:this.locked,aim:this.aim,charges:this.bank.cells(this.tick),deadlines:this.bank.slots,material:this.material,momentum:this.momentum,theory:this.theory,plan:this.plan,packets:this.packets,deliveries:this.deliveries,fields:this.fields,anchors:this.anchors,incoming:this.incoming,visuals:this.visuals,events:this.events,releasePose:this.releasePose,transformation:this.transformation,failureUntil:this.failureUntil,paused:this.paused}));}
}
const api={Combat,Window,unit,near,materialCost,PACKET_SKILLS,TPS,UPX,WIDTH,HEIGHT,handlerCount:75};
if(typeof module!=='undefined')module.exports=api;else root.ScienceCombat=api;
})(typeof globalThis!=='undefined'?globalThis:this);
