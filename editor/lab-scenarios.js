// Repeatable, settings-based host scenarios. This is not the stock engine AI.
window.GameScenario=class {
  static choices={laning:'Mid-lane wave / trading',gank:'Gank from the river',countergank:'Being ganked / countergank',camp:'Assigned jungle camp',morgard:'Morgard contest',serpen:'Serpen contest',defend:'Defend a tower',dive:'Dive with an allied wave'};
  constructor(world,kind,level=8){this.world=world;this.kind=kind;this.level=level;this.time=8*60*60;this.shots=[];this.metrics={requested:0,hp_damage:0,shield_damage:0,taken:0,kills:0,champions:0,minions:0,camps:0,objectives:0};this.startHP=0;}
  stats(def,level=this.level){const out={...def.stat};for(const[k,v]of Object.entries(def.growth||{}))out[k]=(out[k]||0)+v*(level-1);return out;}
  make(id,team,pos,kind,key,def,extra={}){
    const stats=this.stats(def),champion=kind==='champion',radius=champion?this.world.data.settings.champion_radius:kind==='tower'?12000:kind==='objective'?16000:kind==='minion'?4000:9000;
    const p=['tower','nexus','camp','objective'].includes(kind)?{...pos}:this.world.legal(pos,radius);return{id,team,pos:p,home:{...p},kind,sprite:key,name:extra.name||key.replace('champ-',''),champion,tower:kind==='tower'||kind==='nexus',farmable:!champion&&!['tower','nexus'].includes(kind),radius,
      hp:stats.hp,max_hp:stats.hp,stats,attack:def.attack||{},slowUntil:0,slowPct:0,stunUntil:0,airborneUntil:0,frost:0,frostUntil:0,burnUntil:0,frozenUntil:0,nextAttack:0,attackStart:-100,attackUntil:0,poseStart:0,left:team===1,moving:false,visible:true,shields:[],...extra};
  }
  setup(){
    const w=this.world,d=w.data.doc,s=w.data.settings,champ=w.data.champions;
    this.anchor=this.kind==='morgard'?this.camp('Morgard'):this.kind==='serpen'?this.camp('Serpen'):this.kind==='camp'?this.camp('Rhino'):{x:480000,y:480000};
    if(['defend','dive'].includes(this.kind)){const p=d.towers.find(t=>t.ty==='Mid').pos[this.kind==='dive'?1:0];this.anchor={x:p[0],y:p[1]};}
    const a=this.anchor,me=w.legal(this.kind==='gank'?{x:a.x+60000,y:a.y+25000}:{x:a.x-40000,y:a.y+40000},s.champion_radius);
    const units=[];
    for(const[id,team,key,p]of [[2,1,'knight',{x:a.x+25000,y:a.y-18000}],[3,1,'ninja',{x:a.x+110000,y:a.y+35000}],[4,0,'archer',{x:a.x-65000,y:a.y+50000}],
      [5,0,'priest',{x:650000,y:870000}],[6,0,'ninja',{x:a.x-120000,y:a.y-30000}],[7,0,'berserker',{x:80000,y:500000}],
      [8,1,'archer',{x:880000,y:520000}],[9,1,'priest',{x:870000,y:480000}],[12,1,'berserker',{x:500000,y:80000}]]){
      units.push(this.make(id,team,p,'champion','champ-'+key,champ[key],{name:champ[key].name,job:id===3?'enemy-jungle':id===6?'allied-jungle':'lane'}));
    }
    if(this.kind==='gank'){units.find(u=>u.id===2).home=w.legal({x:a.x+15000,y:a.y-18000},10000);units.find(u=>u.id===3).home={x:800000,y:160000};}
    if(this.kind==='countergank')units.find(u=>u.id===3).home=w.legal({x:a.x+65000,y:a.y+40000},10000);
    if(['camp'].includes(this.kind)){for(const u of units.filter(u=>u.team===1&&u.champion))u.home=w.legal({x:800000,y:160000},u.radius);}
    for(const u of units)u.pos={...u.home};
    if(['morgard','serpen'].includes(this.kind))for(const u of units.filter(u=>[3,4].includes(u.id)))u.job='objective';
    let id=100;
    for(const t of d.towers)for(let team=0;team<2;team++){const p=t.pos[team];units.push(this.make(id++,team,{x:p[0],y:p[1]},'tower',team?'red_tower':'blue_tower',s.tower,{name:(team?'Red ':'Blue ')+t.ty,level:1}));}
    for(let team=0;team<2;team++){const p=d.nexus_pos[team];units.push(this.make(id++,team,{x:p[0],y:p[1]},'nexus',team?'red_nexus':'blue_nexus',s.nexus,{name:(team?'Red':'Blue')+' Nexus'}));}
    const kinds={Rhino:['rhino','rhino_jungle'],Mushroom:['mushroom','mushroom_jungle'],Stump:['stump','tree_jungle'],Bee:['bee','bee_jungle'],Morgard:['epic','epic_jungle'],Serpen:['serpen','serpen_jungle']};
    for(const camp of d.camps){const[key,rule]=kinds[camp.ty],major=['Morgard','Serpen'].includes(camp.ty);for(const p of (major?camp.pos.slice(0,1):camp.pos)){
      const u=this.make(id++,9,{x:p[0],y:p[1]},major?'objective':'camp',key,s[rule],{name:camp.ty});
      // Camp/objective growth depends on host spawning. Keep extracted base HP
      // and disclose it rather than inventing an authoritative scaling rule.
      Object.assign(u.stats,s[rule].stat);u.hp=u.max_hp=u.stats.hp;
      if(major&&this.kind===camp.ty.toLowerCase())u.hp=Math.floor(u.max_hp*.6);
      units.push(u);
    }}
    for(const lane of d.lanes)for(let team=0;team<2;team++)for(let i=0;i<3;i++){
      const path=lane.path.map(p=>({x:p[0],y:p[1]})),middle=path[Math.floor(path.length/2)],direction=team?-1:1;
      let p={x:middle.x-direction*(28000+i*13000),y:middle.y+direction*(28000+i*13000)};
      if(lane.line==='Top')p={x:middle.x+direction*(28000+i*13000),y:middle.y};
      if(lane.line==='Bottom')p={x:middle.x,y:middle.y-direction*(28000+i*13000)};
      if(['defend','dive'].includes(this.kind)&&lane.line==='Mid')p={x:a.x+(team?1:-1)*(25000+i*12000),y:a.y-(team?1:-1)*(25000+i*12000)};
      const ranged=i===2,def=s[ranged?'range_minion':'melee_minion'];
      const u=this.make(id++,team,p,'minion','minions',def,{name:lane.line+' '+(ranged?'ranged':'melee')+' minion',minion:ranged?'range':'melee',lane:lane.line,path:team?[...path].reverse():path,waypoint:Math.floor(path.length/2)+1});
      const wave=s.minion_wave_setting,growth=Math.max(0,Math.floor((Math.min(this.time,wave.growth_end_tick)-wave.growth_start_tick)/wave.growth_tick)+1);
      u.stats=this.stats(def,growth+1);u.growthSteps=growth;u.hp=u.max_hp=u.stats.hp;units.push(u);
    }
    this.startHP=0;
    return{pos:me,units,description:this.description(),clock:this.time,role:this.kind==='camp'?'jungle':'mid'};
  }
  camp(name){const p=this.world.data.doc.camps.find(c=>c.ty===name).pos[0];return{x:p[0],y:p[1]};}
  description(){return{
    laning:'Trade against Knight around an arriving wave. Clear before rotating; tower and jungle routes retain real positions.',
    gank:'Approach the mid-laner from river with an allied laner. Enemy jungle remains away; judge follow-up and your exit.',
    countergank:'Knight holds the lane while Ninja approaches from the side after two seconds. Peel, disengage or turn with your ally.',
    camp:'Assigned Rhino clear with the rest of the battlefield alive. Skill farming is enabled for the jungle role.',
    morgard:'Two teams contest the real Morgard pit, with the objective at 60% base HP. Compare damage, zoning and peel.',
    serpen:'Contest the real Serpen pit at 60% base HP. Watch the approaches and decide whether to finish or turn.',
    defend:'An enemy wave approaches your real mid tower. Use the wave, protect your laner and punish an overstep.',
    dive:'Enter an enemy tower with an allied wave. The model prioritises minions unless you damage a protected champion.'
  }[this.kind];}
  visible(u,team,me,units){if(this.vision?.[team]?.has(u.id))return this.vision[team].get(u.id);return this.calculateVisible(u,team,me,units);}
  calculateVisible(u,team,me,units){if(u.team===team)return true;const allies=[...(team===0&&me.hp>0?[me]:[]),...units.filter(a=>a.team===team&&a.champion&&a.hp>0)];return allies.some(a=>Math.hypot(a.pos.x-u.pos.x,a.pos.y-u.pos.y)<=this.world.data.settings.visible_distance&&this.world.clear(a.pos,u.pos));}
  refreshVisibility(me,units){this.vision={};for(const team of[0,1])this.vision[team]=new Map([me,...units].map(u=>[u.id,this.calculateVisible(u,team,me,units)]));}
  damage(source,target,raw,tick){
    if(target.hp<=0)return 0;const armour=Math.max(0,target.stats?.defence||0);let amount=Math.floor(raw*100/(100+armour));
    if(tick<target.reduceUntil)amount=Math.floor(amount*(100-target.reduce)/100);
    let shield=0;for(const s of target.shields||[]){if(tick>=s.until)continue;const a=Math.min(s.amount,amount);s.amount-=a;amount-=a;shield+=a;}
    const delivered=Math.min(target.hp,amount);target.hp-=delivered;if(target.hp===0)target.deadAt??=tick;
    if(source.id===1){this.metrics.requested+=raw;this.metrics.hp_damage+=delivered;this.metrics.shield_damage+=shield;
      const k=target.champion?'champions':target.kind==='minion'?'minions':target.kind==='objective'?'objectives':'camps';this.metrics[k]+=delivered;
      if(target.hp===0){this.metrics.kills++;target.deadAt=tick;}
    }
    if(target.champion){source.provokedTeam=target.team;source.provokedUntil=tick+180;}
    return delivered;
  }
  advance(tick,me,units,callbacks){
    const w=this.world,living=[me,...units].filter(u=>u.hp>0);
    for(const u of units){
      u.visible=this.visible(u,0,me,units);u.moving=false;if(tick<u.stunUntil)u.pending=null;if(u.hp<=0||tick<u.stunUntil||u.kind==='nexus')continue;
      const targets=living.filter(t=>u.team===9?t.team!==9&&Math.hypot(t.pos.x-u.home.x,t.pos.y-u.home.y)<65000:t.id!==u.id&&t.team!==u.team&&(t.team!==9||u.job==='objective'&&t.kind==='objective'&&t.name.toLowerCase()===this.kind)&&this.visible(t,u.team,me,units));
      const reach=u.attack.range||20000;
      let target=targets.filter(t=>Math.hypot(t.pos.x-u.pos.x,t.pos.y-u.pos.y)<=(u.tower?reach+u.radius+t.radius:u.champion?150000:65000)&&w.clear(u.pos,t.pos)).sort((a,b)=>{
        const priority=t=>u.tower?(t.provokedTeam===u.team&&tick<t.provokedUntil?-2:t.kind==='minion'?-1:0):u.job==='objective'&&t.kind==='objective'?-2:t.kind==='minion'&&!u.champion?-1:0;
        return priority(a)-priority(b)||Math.hypot(a.pos.x-u.pos.x,a.pos.y-u.pos.y)-Math.hypot(b.pos.x-u.pos.x,b.pos.y-u.pos.y)||a.id-b.id;
      })[0];
      if(u.job==='enemy-jungle'&&!['countergank','morgard','serpen'].includes(this.kind))target=null;
      if(u.job==='enemy-jungle'&&this.kind==='countergank'&&tick-this.time<120)target=null;
      if(u.pending&&tick>=u.pending.release){
        const p=u.pending;u.pending=null;const t=living.find(t=>t.id===p.target);
        if(t){const travel=u.attack.speed?Math.max(1,Math.ceil(Math.hypot(t.pos.x-u.pos.x,t.pos.y-u.pos.y)/u.attack.speed)):0;
          this.shots.push({from:{...u.pos},source:u.id,target:t.id,raw:u.stats.attack*(u.attack.attack_ratio||100)/100,start:tick,at:tick+travel});}
      }
      if(target&&Math.hypot(target.pos.x-u.pos.x,target.pos.y-u.pos.y)<=reach+u.radius+target.radius&&tick>=u.nextAttack){
        u.attackStart=tick;u.attackUntil=tick+(u.attack.duration||24);u.nextAttack=tick+(u.attack.cooltime||60);u.left=target.pos.x<u.pos.x;u.pending={target:target.id,release:tick+(u.attack.start_timing||12)};
      }else if(!u.tower&&tick>=u.attackUntil){
        let goal=target?.pos;
        if(!goal&&u.kind==='minion'){while(u.waypoint<u.path.length-1&&Math.hypot(u.pos.x-u.path[u.waypoint].x,u.pos.y-u.path[u.waypoint].y)<8000)u.waypoint++;goal=u.path[u.waypoint];}
        if(!goal&&u.champion)goal=u.home;if(!goal&&u.team===9)goal=u.home;
        const speed=(u.stats.move_speed||0)*(tick<u.slowUntil?1-(u.slowPct||20)/100:1);
        if(goal&&Math.hypot(u.pos.x-goal.x,u.pos.y-goal.y)>Math.max(1000,target?reach+u.radius+target.radius-2000:1000))u.moving=w.move(u,goal,speed);
      }
    }
    for(const shot of this.shots.filter(s=>tick>=s.at)){
      const source=[me,...units].find(u=>u.id===shot.source),target=living.find(u=>u.id===shot.target);if(!source||!target)continue;
      if(target.id===1)callbacks.hitOwner({from:source.id,champion:source.champion,amount:Math.floor(shot.raw)});
      else{const hp=this.damage(source,target,Math.floor(shot.raw),tick);callbacks.hit(target,hp);}
    }
    this.shots=this.shots.filter(s=>tick<s.at);
  }
};
