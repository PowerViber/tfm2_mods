// Actual installed map and stock actors. WASM owns Kicker's decisions; host behaviour is a labelled model.
(async()=>{
 const world=await new GameWorld().ready,painter=new KickerPainter(),clock=new KratosClock();await painter.ready;
 while(!wasm?.instance)await new Promise(r=>setTimeout(r,50));
 const oldReset=reset,coreStep=step,oldApply=apply;
 let cam={x:480000,y:480000,s:.002},lastState=null,scene,trace=[],capture=false,fx=[],pose=null,poseAt=0,held=new Set(),walked=false,walkStart=0,lastPos=null;
 let basicReady=0,basicImpact=null,basicEnd=0,invulnerableUntil=0,undyingUntil=0,basicHits=[],facing=false,deadAt=null,stunUntil=0;
 const top=document.createElement('div');
 top.innerHTML='<p style="color:#a6d5d2">Installed map, wall cells and stock actors. Kicker uses the shared core; movement, mitigation, visibility and opponent actions are simulated, with no stock skill AI.</p><label>Scenario <select id="scenario"></select></label><button id="scenarioLoad">Load mid-game situation</button><label><input id="follow" type="checkbox" checked> Follow / 2x</label><label><input id="grid" type="checkbox"> Wall grid</label><label><input id="artEnabled" type="checkbox" checked> Artwork</label><label><input id="hostModel" type="checkbox" checked> Simulated opponents / waves</label><button id="singleStep">Step once</button><p>Focus arena: WASD move, Space basic, Q mount/hop, E rock, C kick/Skyfall, Shift+C Flash kick, F marked dash, R Second Wind.</p><details><summary>13 athlete values</summary><div id="athleteStats"></div></details>';
 $('side').insertBefore(top,$('side').children[1]);
 const names=['ego','judgement','positioning','control_speed','skill_hit','skill_avoid','aggressive','order','roaming','macro','micro','awareness','strategy'];
 $('athleteStats').innerHTML=names.map(n=>'<label>'+n+' <input min="0" max="100" type="number" value="50" style="width:50px"></label>').join('');
 $('scenario').innerHTML=Object.entries(GameScenario.choices).map(([k,v])=>'<option value="'+k+'">'+v+'</option>').join('');$('scenario').value='gank';
 reset=function(){
  oldReset();world.walls=world.data.doc.walls.flat().map(Boolean);world.routes.clear();walls=[...world.walls];
  scene=new GameScenario(world,$('scenario').value,8);const s=scene.setup();
  me.pos=s.pos;const stats=scene.stats({stat:R.stat,growth:R.growth}),mp=R.modPower;
  me.max_hp=me.hp=Math.floor(stats.hp*(1+(mp[2]||0)/100));me.attack=Math.floor(stats.attack*(1+(mp[0]||0)/100));
  Object.assign(me,{id:1,team:0,kind:'champion',champion:true,radius:world.data.settings.champion_radius,stats:{...stats,defence:Math.floor(stats.defence*(1+(mp[3]||0)/100))},shields:[]});
  units=s.units;tick=s.clock;lastState=null;trace=[];fx=[];pose=null;basicReady=0;basicImpact=null;basicEnd=0;basicHits=[];invulnerableUntil=0;undyingUntil=0;held.clear();deadAt=null;lastPos={...me.pos};
  stunUntil=0;
  world.loadSprites(units.map(u=>u.sprite)).catch(console.error);aim={x:me.pos.x+50000,y:me.pos.y};scene.refreshVisibility(me,units);
 };
 window.kickerHost={stunned:()=>tick<stunUntil,takeBasicHits:()=>basicHits.splice(0),record:(frame,res)=>{if(capture&&trace.length<3000)trace.push(structuredClone({input:{version:1,rules_revision:R.revision,frame},output:res}));}};
 apply=function(e){
  const d=e.data;
  if(e.kind==='fx')fx.push({name:d.name.replace(/^kk_/,''),at:{...d.at},start:tick,end:tick+d.ticks});
  if(e.kind==='pose'){pose=d.name;poseAt=tick;}
  if(e.kind==='shield')me.shields.push({amount:d.amount,until:tick+d.ticks});
  if(e.kind==='invulnerable')invulnerableUntil=Math.max(invulnerableUntil,tick+d.ticks);
  if(e.kind==='undying')undyingUntil=tick+d.ticks;
  if(e.kind==='undying_end')undyingUntil=0;
  if(e.kind==='knock_up'||e.kind==='stun'){const u=units.find(u=>u.id===d.target);if(u)u.stunUntil=Math.max(u.stunUntil||0,tick+d.ticks);}
  if(e.kind==='damage'){
   const u=units.find(u=>u.id===d.target);if(u){const hp=scene.damage(me,u,d.amount,tick);log('hit '+d.target+': '+d.amount+' requested / '+hp+' HP (model)');}
   return;
  }
  oldApply(e);
 };
 function basic(){
  if(me.hp<=0||tick<basicReady||lastState?.moving||lastState?.kick)return;
  const target=near(aim,units.filter(u=>u.team!==0&&u.hp>0&&u.visible!==false));
  if(!target||Math.hypot(target.pos.x-me.pos.x,target.pos.y-me.pos.y)>R.basic.range+me.radius+target.radius||!world.clear(me.pos,target.pos))return log('basic: no legal target in reach');
  basicReady=tick+R.basic.cooldownTicks;basicEnd=tick+R.basic.duration;basicImpact={target:target.id,at:tick+R.basic.startTiming};facing=target.pos.x<me.pos.x;pose='attack';poseAt=tick;
 }
 step=function(){
  const was={...me.pos};walked=false;
  const dx=(held.has('d')?1:0)-(held.has('a')?1:0),dy=(held.has('s')?1:0)-(held.has('w')?1:0),length=Math.hypot(dx,dy);
  if(length&&me.hp>0&&tick>=stunUntil&&!lastState?.moving&&!lastState?.kick&&tick>=basicEnd){
   const speed=me.stats.move_speed*(lastState?.roof?1+R.roof.speedPct/100:1);
   const to={x:me.pos.x+dx/length*speed,y:me.pos.y+dy/length*speed};
   if(lastState?.roof)me.pos={x:Math.round(Math.max(me.radius,Math.min(MAP-me.radius,to.x))),y:Math.round(Math.max(me.radius,Math.min(MAP-me.radius,to.y)))};
   else world.move(me,to,speed);
  }
  walked=was.x!==me.pos.x||was.y!==me.pos.y;
  if(walked){if(lastPos?.x===was.x&&lastPos?.y===was.y&&pose!=='run')walkStart=tick;facing=me.pos.x<was.x;pose='run';}
  if(basicImpact&&tick>=basicImpact.at){
   const u=units.find(u=>u.id===basicImpact.target&&u.hp>0);
   if(u){const raw=Math.floor(me.attack*(lastState&&tick<lastState.bonus_until?1+R.dropIn.bonusPct/100:1));scene.damage(me,u,raw,tick);basicHits.push(u.id);}
   basicImpact=null;
  }
  if($('hostModel').checked){
   scene.refreshVisibility(me,units);
   for(const g of glides){const u=units.find(u=>u.id===g.who);if(u)u.stunUntil=Math.max(u.stunUntil||0,g.start+g.ticks);}
   scene.advance(tick,me,units,{hit:()=>{},hitOwner:h=>{
    if(me.hp<=0||tick<invulnerableUntil)return;
    const raw=Math.floor(h.amount*(lastState?.roof?1-R.roof.reducePct/100:1));
    const source=units.find(u=>u.id===h.from)||{id:h.from};
    scene.damage(source,me,raw,tick);if(tick<undyingUntil&&me.hp<=0)me.hp=1;
   }});
  }
  if(me.hp<=0&&deadAt===null)deadAt=tick;
  coreStep();lastPos={...me.pos};fx=fx.filter(f=>tick<f.end);
 };
 const P=p=>({x:(p.x-cam.x)*cam.s+360,y:(p.y-cam.y)*cam.s+360}),W=n=>n*cam.s;
 draw=function(state){
  if(state)lastState=state;
  const c=$('map').getContext('2d'),st=lastState||{},art=$('artEnabled').checked;
  cam=$('follow').checked?{...me.pos,s:.002}:{x:480000,y:480000,s:720/960000};
  c.clearRect(0,0,720,720);world.drawMap(c,P,W);
  if($('grid').checked){c.fillStyle='#fb705133';walls.forEach((b,i)=>{if(b){const q=P({x:i%30*32000,y:Math.floor(i/30)*32000});c.fillRect(q.x,q.y,W(32000),W(32000));}});}
  const drawFx=(tag,at,elapsed=0)=>{const q=P(at);if(art)painter.fx(c,tag,q.x,q.y,W(1000),elapsed);};
  for(const f of fx){if(['rock','tag','skyfall'].includes(f.name))continue;drawFx(f.name,f.at,tick-f.start);}
  for(const u of units){
   if(u.hp<=0||u.visible===false)continue;
   world.drawUnit(c,u,tick,P,W);
   if(st.marks?.some(m=>m[0]===u.id))drawFx('mark_attached',u.pos);
   const bruise=st.bruised?.find(b=>b[0]===u.id);
   if(bruise)drawFx('bruised'+bruise[2]+'_attached',u.pos);
  }
  const q=P(me.pos),moving=st.moving;
  if(st.roof){c.save();c.strokeStyle='#9de44899';c.setLineDash([6,5]);c.beginPath();c.arc(q.x,q.y,W(R.hop.wallRange),0,Math.PI*2);c.stroke();c.restore();}
  let tag=me.hp<=0?'dead':st.roof&&!moving?'perch':moving?.kind==='skyfall'?'dive':moving?(pose?.startsWith('hop_')?pose:'hop_high'):tick<basicEnd?'attack':st.kick?'kick':pose==='throw'&&tick<poseAt+12?'throw':walked?'run':'idle';
  const age=me.hp<=0?tick-deadAt:moving?tick-poseAt:tag==='idle'||tag==='perch'?tick:tag==='run'?tick-walkStart:tick-poseAt;
  if(art)painter.draw(c,tag,q.x,q.y,W(1000),facing,age);
  else{c.strokeStyle='#fff';c.strokeRect(q.x-W(me.radius),q.y-W(me.radius),W(me.radius*2),W(me.radius*2));}
  for(const rock of st.rocks||[]){
   const heading=direction(rock.pos,rock.end);drawFx('rock_'+heading,rock.pos);
  }
  if(st.ult){drawFx('tag',st.ult.anchor);drawFx('second_wind',me.pos);}
  for(const flight of st.flights||[]){const u=units.find(u=>u.id===flight.target);if(u)drawFx('kick_trail_'+direction(flight.from,flight.to),u.pos);}
  if(moving?.kind==='skyfall')drawFx('skyfall_'+direction(me.pos,moving.land),me.pos);
  world.drawMap(c,P,W,true);
  const a=P(aim);c.strokeStyle='#efd58c';c.strokeRect(a.x-4,a.y-4,8,8);
  const bar=P(me.pos);c.fillStyle='#171d24';c.fillRect(bar.x-24,bar.y-54,48,5);c.fillStyle='#8dcf64';c.fillRect(bar.x-24,bar.y-54,48*Math.max(0,me.hp/me.max_hp),5);
  if(state)$('state').textContent='HP '+me.hp+'/'+me.max_hp+' · '+(st.roof?'wall':'ground')+' · hops '+st.hops+' / dashes '+st.dashes+' / kicks '+st.kicks+' · '+st.last_skill;
 };
 function direction(a,b){const n=(Math.round(Math.atan2(b.y-a.y,b.x-a.x)/(Math.PI/4))+8)%8;return KICKER_ART.headings[n];}
 function point(ev){const b=$('map').getBoundingClientRect();return{x:Math.round(((ev.clientX-b.left)*720/b.width-360)/cam.s+cam.x),y:Math.round(((ev.clientY-b.top)*720/b.height-360)/cam.s+cam.y)};}
 $('map').tabIndex=0;
 $('map').onpointerdown=()=>$('map').focus();
 $('map').onmousemove=ev=>{if($('tool').value==='aim')aim=point(ev);};
 $('map').onclick=ev=>{
  const p=point(ev),tool=$('tool').value;
  if(tool==='aim')aim=p;
  else if(tool==='wall'){const i=Math.floor(p.y/CELL)*N+Math.floor(p.x/CELL);if(i>=0&&i<900){walls[i]=!walls[i];world.walls[i]=walls[i];}}
  else if(tool==='me')me.pos=world.legal(p,me.radius);
  else if(units.length<64){
   const kind=tool==='tower'?'tower':tool==='minion'?'minion':'champion',team=tool==='ally'?0:1,key=kind==='tower'?'red_tower':kind==='minion'?'minions':team?'champ-knight':'champ-archer',def=kind==='tower'?world.data.settings.tower:kind==='minion'?world.data.settings.melee_minion:world.data.champions[team?'knight':'archer'];
   units.push(scene.make(nextId++ +1000,team,p,kind,key,def,{minion:'melee'}));world.loadSprites(units.map(u=>u.sprite));
  }
  draw(null);
 };
 $('map').onkeydown=e=>{
  const k=e.key.toLowerCase();if(!['w','a','s','d',' ','q','e','c','f','r'].includes(k))return;e.preventDefault();
  if(e.repeat)return;held.add(k);running=true;
  if(k===' ')basic();if(k==='q')order(e.shiftKey?'drop_kick':lastState?.roof?'hop':'mount');if(k==='e')order('rock');if(k==='c')order(e.shiftKey?'flash_kick':lastState?.roof?'skyfall':'kick');if(k==='f')order('dash');if(k==='r')order('ult');
 };
 $('map').onkeyup=e=>held.delete(e.key.toLowerCase());
 const clear=()=>{held.clear();walked=false;clock.reset();};window.addEventListener('blur',clear);$('map').addEventListener('blur',clear);document.addEventListener('visibilitychange',clear);
 $('scenarioLoad').onclick=()=>{reset();draw(null);};$('singleStep').onclick=()=>{running=false;step();};
 reset();await world.loadSprites(units.map(u=>u.sprite));
 function loop(t){clock.advance(t,running&&!document.hidden,step);draw(null);requestAnimationFrame(loop);}requestAnimationFrame(loop);
 const stun=()=>{stunUntil=tick+60;if(!running)step();};
 const stunButton=document.createElement('button');stunButton.textContent='Incoming stun (1 second)';stunButton.onclick=stun;top.appendChild(stunButton);
 const wallHint=document.createElement('p');wallHint.textContent='Wall-to-wall range: '+R.hop.wallRange.toLocaleString()+' units. Shift+Q: deliberate Drop-in. Walking and terrain-cell changes preserve the wall state.';top.appendChild(wallHint);
 window.kickerLab={step,reset,order,basic,stun,pause:()=>{running=false;clock.reset();},world,painter,setAim:p=>aim={...p},capture:on=>{capture=on;trace=[];},get state(){return lastState;},get me(){return me;},get units(){return units;},get tick(){return tick;},get walls(){return walls;},get trace(){return trace;},get scene(){return scene;}};
})().catch(console.error);
