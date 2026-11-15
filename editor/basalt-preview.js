// Candidate artwork on the extracted battlefield. Skills remain in the existing WASM core.
// Movement, opponents, mitigation and visibility below are a labelled host model.
(async()=>{try{
const world=await new GameWorld().ready,painter=await new BasaltPainter().ready,$=id=>document.getElementById(id);
while(!wasm)await new Promise(r=>setTimeout(r,25));clearInterval(window.basaltLegacyTimer);
const clock=new KratosClock(),held=new Set(),effects=[],rocks=new Map();let scene,walkGoal=null,pose='idle',poseUntil=0,left=false,lastPos=null,movedAt=-100,lastWallKey='',cam={x:480000,y:480000,s:.0018};
const panel=document.createElement('div');panel.innerHTML='<p style="color:#ffc887">Original artwork proposal — approval pending. Real map and stock actors; host movement, attacks, armour and visibility are simulated.</p><label>Situation <select id="scenario"></select></label><button id="loadScene">Load</button><p id="sceneText"></p><label><input id="follow" type="checkbox" checked> Follow / game-size view</label><label><input id="wallGrid" type="checkbox"> Collision grid</label><label><input id="showArt" type="checkbox" checked> Candidate presentation</label><label><input id="hostModel" type="checkbox" checked> Opponents / waves</label><p>Focus arena. Arrow keys / right-click: move. Space: Jump · F: Whip Pull · Q/W/E: S1/S2/S3 · R: ultimate/recast.</p><button id="singleStep">Step once</button><button id="incomingStun">Incoming stun</button><details><summary>13 athlete values</summary><div id="athleteStats"></div></details>';
$('side').insertBefore(panel,$('state'));$('side').style.width='430px';
for(const [value,label]of Object.entries(GameScenario.choices))$('scenario').add(new Option(label,value));$('scenario').value='gank';
const statNames=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];
$('athleteStats').innerHTML=statNames.map(n=>'<label style="display:inline-block;width:205px">'+n+' <input type="number" min="0" max="100" value="50" style="width:48px"></label>').join('');
const gridWall=world.wallAt.bind(world);world.wallAt=(p,r=0)=>{if(gridWall(p,r))return true;return(state.walls||[]).some(w=>Math.abs(p.x-w.a.x)<R.wall.size/2+r&&Math.abs(p.y-w.a.y)<R.wall.size/2+r);};
const baseMove=world.move.bind(world);world.move=(u,to,speed)=>{if(glides.has(u.id))return false;return baseMove(u,to,speed);};
reset=function(){
 wasm.exports.basalt_reset();effects.length=0;rocks.clear();glides.clear();held.clear();logLines=[];state={};walkGoal=null;pose='idle';lastPos=null;lastWallKey='';world.routes.clear();
 scene=new GameScenario(world,$('scenario').value,8);const setup=scene.setup(),stats=scene.stats({stat:R.stat,growth:R.growth});
 tick=setup.clock;me={id:1,team:0,pos:setup.pos,hp:Math.floor(stats.hp*(1+R.modPower[2]/100)),max_hp:Math.floor(stats.hp*(1+R.modPower[2]/100)),attack:Math.floor(stats.attack*(1+R.modPower[0]/100)),stats:{...stats,defence:Math.floor(stats.defence*(1+R.modPower[3]/100)),magic_resistance:Math.floor(stats.magic_resistance*(1+R.modPower[4]/100))},radius:world.data.settings.champion_radius,champion:true,stun:0,air:0,shields:[]};
 units=setup.units.map(u=>({...u,stun:0,up:0,slow:0}));lastPos={...me.pos};movedAt=-100;terrain.splice(0,terrain.length,...world.walls);$('sceneText').textContent=setup.description;scene.refreshVisibility(me,units);
 world.loadSprites(units.map(u=>u.sprite)).catch(fail);pending=null;draw();
};
function fail(e){play=false;log('Reference error: '+e.message);$('sceneText').textContent=e.message;}
function presentOrder(order){if(!order)return;left=order.aim.x<me.pos.x;pose=order.kind==='s1'?(tick<me.air?'air_punch':'raise'):order.kind==='s2'?'whip':order.kind==='s3'?'whip':order.kind==='ult'?(state.recast?'whip':tick<me.air?'air_punch':'charge'):'air_punch';poseUntil=tick+24;}
step=function(){
 if(!scene)return;
 for(const[id,gl]of glides){const u=byId(id);if(!u){glides.delete(id);continue;}const k=Math.min(1,(tick-gl.t0)/gl.n);u.pos={x:gl.a.x+(gl.b.x-gl.a.x)*k,y:gl.a.y+(gl.b.y-gl.a.y)*k};u.left=gl.b.x<gl.a.x;if(k>=1)glides.delete(id);}
 if(!aiOn&&tick>=me.stun&&!state.charging&&!glides.has(1)){
  const dx=Number(held.has('ArrowRight'))-Number(held.has('ArrowLeft')),dy=Number(held.has('ArrowDown'))-Number(held.has('ArrowUp'));let goal=walkGoal;
  if(dx||dy){const n=Math.hypot(dx,dy);goal={x:me.pos.x+dx/n*10000,y:me.pos.y+dy/n*10000};walkGoal=null;}
  if(goal){if(tick<me.air){const d=Math.hypot(goal.x-me.pos.x,goal.y-me.pos.y),k=Math.min(1,1050/d);if(d>0)me.pos={x:me.pos.x+(goal.x-me.pos.x)*k,y:me.pos.y+(goal.y-me.pos.y)*k};}else world.move(me,goal,1050);}
 }
 if($('hostModel').checked){scene.refreshVisibility(me,units);scene.advance(tick,me,units,{hit:()=>{},hitOwner:h=>{if(tick<me.air)return;const source=units.find(u=>u.id===h.from)||{id:h.from};scene.damage(source,me,h.amount,tick);}});}
 const order=pending;pending=null;presentOrder(order);
 const frame={tick,owner:1,team:0,alive:me.hp>0,pos:P(me.pos),hp:Math.round(me.hp),max_hp:me.max_hp,attack:me.attack,stunned:tick<me.stun,ult_unlocked:true,home:{x:96000,y:864000},terrain,other_walls:[],units:units.filter(u=>u.hp>0).map(u=>({id:u.id,team:u.team,pos:u.visible?P(u.pos):{x:0,y:0},hp:Math.round(u.hp),max_hp:u.max_hp,attack:u.stats.attack,champion:u.champion,tower:u.tower,visible:u.visible,targetable:true})),profile:[...document.querySelectorAll('#athleteStats input')].map(i=>Math.max(0,Math.min(100,Number(i.value)||0))),manual:!aiOn,order};
 const bytes=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:R.revision,frame})),ptr=wasm.exports.basalt_input(bytes.length);new Uint8Array(wasm.exports.memory.buffer,ptr,bytes.length).set(bytes);
 const result=wasm.exports.basalt_step(bytes.length),out=JSON.parse(new TextDecoder().decode(new Uint8Array(wasm.exports.memory.buffer,result,wasm.exports.basalt_output_len())));if(out.error)throw new Error(out.error);
 const previous=state;state=out.state;
 for(const e of out.effects){
  const u=byId(e.target);
  if(e.type==='damage'&&u)scene.damage(me,u,e.amount,tick);
  else if(e.type==='knock'&&u){glides.set(u.id,{a:{...u.pos},b:e.to,t0:tick,n:Math.max(1,e.ticks)});u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);}
  else if(e.type==='glide')glides.set(1,{a:{...me.pos},b:e.to,t0:tick,n:Math.max(1,e.ticks)});
  else if(e.type==='air')me.air=Math.max(me.air,tick+e.ticks);
  else if(e.type==='root')glides.delete(1);
  else if(e.type==='stun'&&u){u.stun=u.stunUntil=tick+e.ticks;}
  else if(e.type==='knock_up'&&u){u.up=u.airborneUntil=tick+e.ticks;u.stunUntil=Math.max(u.stunUntil||0,tick+e.ticks);}
  else if(e.type==='slow'&&u){u.slow=u.slowUntil=tick+e.ticks;u.slowPct=e.pct;}
  else if(e.type==='steer'&&aiOn&&!glides.has(1)&&!state.charging){const to=e.to,d=Math.hypot(to.x-me.pos.x,to.y-me.pos.y);if(d>0){const k=Math.min(1,1050*6/d),target=world.sweep(me.pos,{x:me.pos.x+(to.x-me.pos.x)*k,y:me.pos.y+(to.y-me.pos.y)*k},me.radius);glides.set(1,{a:{...me.pos},b:target,t0:tick,n:6});}}
  else if(e.type==='fx'&&effects.length<96)effects.push({name:e.name.replace(/^bs_/,''),at:e.at,start:tick,end:tick+Math.min(e.ticks,60)});
  else if(e.type==='log')log(e.text);
 }
 for(const b of state.boulders||[]){const key=JSON.stringify(b);if(!rocks.has(key))rocks.set(key,{air:b[0],to:b[1],end:b[2],start:tick,from:{...me.pos}});}
 for(const[key,b]of rocks)if(tick>b.end)rocks.delete(key);
 if(previous.charging&&!state.charging){pose=previous.charging[0]?'air_punch':'whip';poseUntil=tick+20;}
 const wallKey=JSON.stringify((state.walls||[]).map(w=>[w.id,w.a]));if(wallKey!==lastWallKey){world.routes.clear();lastWallKey=wallKey;}
 for(let i=effects.length-1;i>=0;i--)if(tick>=effects[i].end)effects.splice(i,1);
 if(lastPos&&Math.hypot(me.pos.x-lastPos.x,me.pos.y-lastPos.y)>100){left=me.pos.x<lastPos.x;movedAt=tick;}lastPos={...me.pos};
};
draw=function(){
 if(!scene)return;const c=cv.getContext('2d');cam=$('follow').checked?{x:me.pos.x,y:me.pos.y,s:.0018}:{x:480000,y:480000,s:Math.min(cv.width,cv.height)/960000};
 const PP=p=>({x:cv.width/2+(p.x-cam.x)*cam.s,y:cv.height/2+(p.y-cam.y)*cam.s}),W=n=>n*cam.s,z=W(1000),art=$('showArt').checked;c.clearRect(0,0,cv.width,cv.height);world.drawMap(c,PP,W);
 if(!aiOn&&tick>=me.air&&!state.charging&&tick>=(state.s3_at||0))painter.clapCone(c,PP(me.pos),PP(mouse),cam.s);
 for(const cl of state.claps||[])painter.clapCone(c,PP(cl.apex),PP({x:cl.apex.x+cl.dir[0]*R.clap.reach,y:cl.apex.y+cl.dir[1]*R.clap.reach}),cam.s,true);
 if($('wallGrid').checked){c.strokeStyle='#f2cf8e88';world.walls.forEach((w,i)=>{if(w){const p=PP({x:i%30*32000,y:Math.floor(i/30)*32000});c.strokeRect(p.x,p.y,W(32000),W(32000));}});}
 for(const w of state.walls||[]){const p=PP(w.a);if(art)painter.fx(c,'square',p,z);c.strokeStyle='#ffbc6e';c.strokeRect(p.x-W(R.wall.size/2),p.y-W(R.wall.size/2),W(R.wall.size),W(R.wall.size));c.fillStyle='#fff3d9';c.fillText('◆'.repeat(w.integrity),p.x-12,p.y+W(25000));}
 for(const u of units){if(!u.visible)continue;world.drawUnit(c,u,tick,PP,W);if(u.hp>0){const p=PP(u.pos);c.fillStyle='#171e24';c.fillRect(p.x-16,p.y-28*z,32,4);c.fillStyle=u.team===0?'#81d6ba':u.team===9?'#e6bb6a':'#ef786b';c.fillRect(p.x-16,p.y-28*z,32*u.hp/u.max_hp,4);}}
 if(art)for(const e of effects)painter.fx(c,e.name,PP(e.at),z,0,Math.min(1,(e.end-tick)/10));
 for(const b of rocks.values()){const aim=PP(b.to),k=Math.max(0,Math.min(1,(tick-b.start)/Math.max(1,b.end-b.start))),p=PP({x:b.from.x+(b.to.x-b.from.x)*k,y:b.from.y+(b.to.y-b.from.y)*k});p.y-=W(70000)*Math.sin(Math.PI*k);c.strokeStyle='#ffbc6e';c.beginPath();c.arc(aim.x,aim.y,W(R.ult.radius),0,Math.PI*2);c.stroke();c.fillStyle='#fff4d9';c.fillText(((b.end-tick)/60).toFixed(1)+'s',aim.x-10,aim.y);if(art)painter.fx(c,'boulder',p,z);}
 for(const m of state.meteors||[]){const p=PP(m[0]),remaining=m[1]-tick;c.strokeStyle='#ed9d64';c.beginPath();c.arc(p.x,p.y,W(R.ult.meteorHitRadius),0,Math.PI*2);c.stroke();if(art&&remaining<30)painter.fx(c,'meteor',{x:p.x-remaining*1.5*z,y:p.y-remaining*3*z},z);}
 const moving=tick<=movedAt+2;
 let body=state.charging?(state.charging[0]?'air_punch':'charge'):tick<poseUntil?pose:tick<me.air?'air_punch':moving?(Math.floor(tick/12)%2?'walk_a':'walk_b'):'idle';
 const p=PP(me.pos),rise=tick<me.air?W(state.charging?35000:20000):0;if(art)painter.pose(c,body,{x:p.x,y:p.y-rise},z,left);
 if(state.charging&&art){const q={x:p.x+(state.charging[0]?55*z:0),y:p.y-rise-70*z};painter.fx(c,'boulder',q,z);c.fillStyle='#ffe8c4';c.fillText('Charge '+((state.charging[2]-tick)/60).toFixed(1)+'s',p.x-35,p.y+40*z);}
 if(state.swing){const target=units.find(u=>u.id===state.swing.target);if(target&&art)painter.whip(c,{x:p.x,y:p.y-rise-8*z},PP(target.pos),z);}
 const ownGlide=glides.get(1);if(ownGlide&&art&&tick<me.air)painter.whip(c,{x:p.x,y:p.y-rise},PP(ownGlide.b),z);
 c.fillStyle='#171e24';c.fillRect(p.x-22,p.y-rise-32*z,44,5);c.fillStyle='#83dcb6';c.fillRect(p.x-22,p.y-rise-32*z,44*me.hp/me.max_hp,5);
 const cd=t=>(Math.max(0,(t||0)-tick)/60).toFixed(1)+'s';$('state').textContent=R.revision+' · review-only artwork\n'+(tick<me.air?'AIRBORNE':'Ground')+' · Walls '+(state.walls||[]).length+'/'+R.wall.max+'\nJump '+state.jumps+' · S1 '+state.punches+' · Pull '+cd(state.pull_at)+'\nS2 '+cd(state.s2_at)+' · S3 '+cd(state.s3_at)+' · Ult '+cd(state.ult_at)+(state.recast?' · RECAST':'')+'\n'+(state.reason||'')+'\nRequested skill damage '+scene.metrics.requested+' · model HP damage '+scene.metrics.hp_damage;
 $('log').textContent=logLines.slice(-60).join('\n');
};
cv.tabIndex=0;cv.addEventListener('pointerdown',()=>cv.focus());cv.addEventListener('contextmenu',e=>{e.preventDefault();if(!aiOn)walkGoal={...mouse};});
cv.addEventListener('mousemove',e=>{const r=cv.getBoundingClientRect(),x=(e.clientX-r.left)*cv.width/r.width,y=(e.clientY-r.top)*cv.height/r.height;mouse={x:Math.round(cam.x+(x-cv.width/2)/cam.s),y:Math.round(cam.y+(y-cv.height/2)/cam.s)};e.stopImmediatePropagation();},true);
addEventListener('keydown',e=>{if(document.activeElement!==cv){if([' ','f','q','w','e','r'].includes(e.key.toLowerCase()))e.stopImmediatePropagation();return;}if(e.repeat){e.stopImmediatePropagation();e.preventDefault();return;}if(e.key.startsWith('Arrow')){held.add(e.key);e.preventDefault();e.stopImmediatePropagation();}},true);
addEventListener('keyup',e=>held.delete(e.key));cv.addEventListener('blur',()=>held.clear());addEventListener('blur',()=>held.clear());
$('loadScene').onclick=reset;$('singleStep').onclick=()=>{play=false;$('play').textContent='Play';step();tick++;draw();};$('incomingStun').onclick=()=>{if(tick>=me.air)me.stun=tick+60;else log('Airborne CC immunity blocks the simulated stun');};
document.addEventListener('visibilitychange',()=>{clock.reset();held.clear();});
reset();await world.loadSprites(units.map(u=>u.sprite));
window.basaltLab={world,painter,pause(){play=false;$('play').textContent='Play';},step(){step();tick++;draw();},reset,order(kind,aim=mouse){pending={kind,aim:P(aim)};},get state(){return state;},get tick(){return tick;},get actors(){return[me,...units];},get scene(){return scene;}};
function loop(now){clock.advance(now,play&&!document.hidden,()=>{step();tick++;});draw();requestAnimationFrame(loop);}requestAnimationFrame(loop);
}catch(e){console.error(e);document.getElementById('log').textContent=e.stack;}})();
