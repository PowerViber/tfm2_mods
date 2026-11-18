/* Reusable target catalogue and packed-body loader for 1vX Labs. */
window.LabTargets=class {
 constructor(world){this.world=world;this.entries=new Map();this.art=new Map();this.images=new Map();this.pending=new Map();this.ready=this.load();}
 async load(){const w=this.world,s=w.data.settings;
  for(const [id,def]of Object.entries(w.data.champions))this.add({id:'stock:'+id,name:def.name||id,kind:'champion',sprite:'champ-'+id,def});
  for(const[id,name,kind,sprite,rule,extra]of[
   ['melee','Melee minion','minion','minions','melee_minion',{minion:'melee'}],['ranged','Ranged minion','minion','minions','range_minion',{minion:'range'}],
   ['rhino','Rhino','camp','rhino','rhino_jungle',{}],['mushroom','Mushroom','camp','mushroom','mushroom_jungle',{}],['stump','Stump','camp','stump','tree_jungle',{}],['bee','Bee','camp','bee','bee_jungle',{}],
   ['morgard','Morgard','objective','epic','epic_jungle',{}],['serpen','Serpen','objective','serpen','serpen_jungle',{}],['tower','Enemy tower','tower','red_tower','tower',{}],['nexus','Enemy nexus','nexus','red_nexus','nexus',{}]])this.add({id,name,kind,sprite,def:s[rule],extra});
  this.add({id:'dummy',name:'Practice dummy (Knight body)',kind:'dummy',sprite:'champ-knight',def:{stat:{hp:5000,defence:0,magic_resistance:0,attack:0,move_speed:0},growth:{},attack:{attack_ratio:100,range:0,cooltime:60}},extra:{}});
  const response=await fetch('lab-custom-targets.json');if(!response.ok)throw Error('Missing generated Lab target catalogue');const data=await response.json();
  for(const c of data.champions)this.add({...c,id:'custom:'+c.id,def:c});
  this.add({id:'fighter',name:'Wright fighter (configured summon)',kind:'summon',asset:'asset/tfm2_custom/vfx/wright',tags:{idle:['fighter_p0'],run:['fighter_p0'],attack:['fighter_p0']},def:{stat:{hp:300,defence:0,magic_resistance:0,attack:30,move_speed:1000},growth:{},attack:{range:15000,cooltime:30,attack_ratio:100}}});
  this.add({id:'reflection',name:'Reflection (configured summon)',kind:'summon',asset:'asset/tfm2_custom/vfx/kagami',tags:{idle:Array.from({length:4},(_,i)=>'clone_idle_'+i),run:Array.from({length:6},(_,i)=>'clone_run_'+i),attack:Array.from({length:4},(_,i)=>'clone_attack_'+i)},def:{stat:{hp:600,defence:20,magic_resistance:20,attack:40,move_speed:1000},growth:{},attack:{range:45000,cooltime:60,attack_ratio:100}}});
  return this;
 }
 add(e){this.entries.set(e.id,e);}
 async prepare(id){const e=this.entries.get(id);if(!e)throw Error('Unknown target '+id);if(e.sprite){await this.world.loadSprites([e.sprite]);return e;}
  if(this.pending.has(e.asset)){await this.pending.get(e.asset);return e;}
  const p=(async()=>{const base='../'+e.asset.replace('asset/','mods/'),r=await fetch(base+'%23anim.fanim');if(!r.ok)throw Error('Missing target animation '+e.name);const spec=await r.json(),im=new Image();im.src=base+'%23sheet.png';await im.decode();this.art.set(e.asset,spec);this.images.set(e.asset,im);})();this.pending.set(e.asset,p);try{await p;}catch(error){this.pending.delete(e.asset);throw error;}return e;
 }
 make(scene,id,entityId,pos,level=8){const e=this.entries.get(id),old=scene.level;scene.level=level;
  const u=scene.make(entityId,e.kind==='camp'||e.kind==='objective'?9:1,pos,e.kind,e.sprite,e.def,{...e.extra,name:e.name});scene.level=old;
  u.pos=this.world.legal(u.pos,u.radius);u.home={...u.pos};
  for(const[k,pct]of ['attack','magic_power','hp','defence','magic_resistance','move_speed'].map((k,i)=>[k,e.modPower?.[i]||0]))u.stats[k]=Math.floor((u.stats[k]||0)*(100+pct)/100);
  u.hp=u.max_hp=u.stats.hp;u.labType=id;u.level=level;u.labTarget=true;u.champion=e.kind==='champion';u.tower=['tower','nexus'].includes(e.kind);u.farmable=!u.champion&&!u.tower;u.catalog=e;
  u.attack={...u.attack,cooltime:Math.max(1,Math.round((u.attack.cooltime||60)*100/(100+(e.modPower?.[6]||0))))};
  // Configured minions have no lane path; use a stationary route until challenged.
  if(e.kind==='minion'){u.path=[{...u.pos}];u.waypoint=0;}
  return u;
 }
 draw(ctx,u,tick,P,W){const e=u.catalog;if(!e||e.sprite)return this.world.drawUnit(ctx,u.hp<=0?{...u,poseStart:u.deadAt??tick}:u,tick,P,W);
  const spec=this.art.get(e.asset),im=this.images.get(e.asset);if(!spec||!im)return false;
  const pose=u.hp<=0?'dead':tick<u.attackUntil?'attack':u.moving?'run':'idle',prefix=e.prefix||'';
  let a;if(e.tags){const names=e.tags[pose]||e.tags.idle;a={frames:names.flatMap(tag=>spec.anims[tag].frames.map(f=>({...f,duration:pose==='attack'?.1:pose==='run'?.8/names.length:.8/names.length})))};}
  else a=spec.anims[prefix+pose+(u.left?'_l':'')]||spec.anims[prefix+pose]||spec.anims[prefix+'idle']||spec.anims.idle;
  if(!a)return false;const total=a.frames.reduce((n,f)=>n+f.duration*60,0),age=tick-(pose==='attack'?u.attackStart:pose==='dead'?u.deadAt||tick:0);let t=pose==='dead'?Math.min(total-.001,age):age%total,i=0;
  while(i<a.frames.length-1&&t>=a.frames[i].duration*60)t-=a.frames[i++].duration*60;const r=a.frames[i].data,p=P(u.pos),s=W(1000);
  const flip=u.left&&(e.tags||!spec.anims[prefix+pose+'_l']);ctx.save();ctx.translate(p.x,p.y);ctx.scale(flip?-1:1,1);ctx.imageSmoothingEnabled=false;ctx.drawImage(im,r.x,r.y,r.w,r.h,-r.w*s/2,-r.h*s/2,r.w*s,r.h*s);ctx.restore();return true;
 }
};
