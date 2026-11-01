(async()=>{'use strict';
const $=s=>document.querySelector(s),canvas=$('#arena'),ctx=canvas.getContext('2d'),R=window.RALLY_RULES.constants,enc=new TextEncoder(),dec=new TextDecoder();
const response=await fetch('rally.wasm?revision='+window.RALLY_RULES.revision);if(!response.ok)throw Error('Rally shared simulation is missing. Build rally_lab for WASM.');const wasm=(await WebAssembly.instantiate(await response.arrayBuffer())).instance.exports;
const art={};for(const form of ['yellow_masked','yellow_unmasked','thunder_masked','thunder_unmasked','white_masked','white_unmasked'])for(const pose of ['idle','run','spike','receive','set']){const im=new Image();im.src='../docs/rally-art-review/'+form+'_'+pose+'.png';try{await im.decode();art[form+'_'+pose]=im}catch{}}
for(const name of ['ball','set','receive','thunder','black','impact','black_impact','support','body_thunder','body_black','trail_thunder','trail_black','stroke_contact','stroke_impact','stroke_black_contact','stroke_black_impact','body_white','white','trail_white','white_impact','white_blink','crater','crater_serve','serve_ring']){const im=new Image();im.src='../docs/rally-art-review/effect-'+name+'.png';try{await im.decode();art[name]=im}catch{}}
for(const name of ['ball','set','receive','thunder','black','impact','black_impact','support','body_thunder','body_black','trail_thunder','trail_black','stroke_contact','stroke_impact','stroke_black_contact','stroke_black_impact','body_white','white','trail_white','white_impact','white_blink','crater','crater_serve','serve_ring'])await Promise.all(Array.from({length:8},async(_,phase)=>{const im=new Image(),key=name+'_p'+phase;im.src='../docs/rally-art-review/effect-'+key+'.png';try{await im.decode();art[key]=im}catch{}}));art.helper=art.yellow_masked_set;
const preview='../docs/rally-results/'+window.RALLY_RULES.revision+'/packed-preview/vfx/';
const impactMeta=(await (await fetch(preview+'rally-impact%23anim.fanim')).json()).anims;
const impactSheet=new Image();impactSheet.src=preview+'rally-impact%23sheet.png';await impactSheet.decode();
function drawPacked(tag,pos,height=2000){const d=impactMeta[tag]?.frames[0]?.data;if(!d)return;const p=screen(pos,height);ctx.drawImage(impactSheet,d.x,d.y,d.w,d.h,p.x-d.w/2,p.y-d.h/2,d.w,d.h);}
const fields=['Ego','Judgement','Positioning','Control Speed','Skill Accuracy','Skill Dodge','Aggressive','Order','Roaming','Macro','Micro','Map Awareness','Strategy'];fields.forEach((name,i)=>{const l=document.createElement('label');l.className='stat';l.innerHTML=`<span>${name}</span><output>50</output><input type="range" min="0" max="100" value="50" data-stat="${i}">`;l.querySelector('input').oninput=e=>l.querySelector('output').textContent=e.target.value;$('#profiles').append(l)});
let session,tick=0,running=false,state={},units=[],held=new Set(),queue=[],aim={x:280000,y:300000},target=2,feedback={},serial=0,history=[],outputs=[],eventLines=[],warnings=new Map(),visuals=[],camera={x:100000,y:220000},last=0,accum=0,loaded=null,replayIndex=0,recordingSetup={};
let lineWarnings=new Map(),demo=new Map(),spikeCaption=null;
function unit(id,team,x,y){return{id,team,pos:{x,y},velocity:{x:0,y:0},hp:id===1?1102:1600,max_hp:id===1?1102:1600,attack:id===1?104:85,radius:10000,champion:true,visible:true,targetable:true,farmable:false,controlled:false,shield:0,shield_until:0,defence:id===1?26:20,buffs:{},next_attack:0,stun_until:0};}
function reset(setup={},firstStep=true){
 if(session!==undefined)wasm.rally_drop(session);session=wasm.rally_new();tick=0;running=false;state={};
 units=[unit(1,0,150000,300000),unit(2,1,360000,300000),unit(3,0,175000,320000)];units[1].hp=units[1].max_hp=10000;
 queue=[];held.clear();feedback={};serial=0;history=[];outputs=[];warnings.clear();lineWarnings.clear();visuals=[];eventLines=[];loaded=null;replayIndex=0;recordingSetup={...setup};demo.clear();spikeCaption=null;
 $('#powerForm').value=setup.form||'base';$('#practicePace').value=setup.pace||0;$('#practiceBank').value=setup.bank_percent||0;
 if(Object.keys(setup).length)applyPractice(setup);$('#play').textContent='Play';if(firstStep)step();running=false;render();
}
function execute(f){const bytes=enc.encode(JSON.stringify(f)),p=wasm.rally_input(bytes.length);if(!p)throw Error('Frame too large');new Uint8Array(wasm.memory.buffer,p,bytes.length).set(bytes);const out=wasm.rally_step(session,bytes.length);const result=JSON.parse(dec.decode(new Uint8Array(wasm.memory.buffer,out,wasm.rally_output_len())));if(result.error)throw Error(result.error);return result;}
function blocks(){return $('#walls').checked?[{min:{x:200000,y:270000},max:{x:207000,y:345000}}]:[];}
function smoke(){return $('#smoke').checked?[{min:{x:168000,y:305000},max:{x:185000,y:335000}}]:[];}
function legal(p,r=10000){return p.x>=r&&p.y>=r&&p.x<=R.MAP-r&&p.y<=R.MAP-r&&!blocks().some(b=>p.x+r>=b.min.x&&p.x-r<=b.max.x&&p.y+r>=b.min.y&&p.y-r<=b.max.y);}
const distance=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y);function toward(a,b,n){const d=distance(a,b);return d?{x:Math.round(a.x+(b.x-a.x)*Math.min(n,d)/d),y:Math.round(a.y+(b.y-a.y)*Math.min(n,d)/d)}:{...a};}
function tierAt(h){return h>=R.WHITE_MIN_HEIGHT?'white_storm':h>=R.BLACK_MIN_HEIGHT?'black_thunder':h>=R.THUNDER_MIN_HEIGHT?'thunder':'normal';}
function permanentTier(){return state.white_storm?'white_storm':state.transformed?'black_thunder':state.thunder_earned?'thunder':'normal';}
function formSpeed(){return({normal:R.SPIKE_SPEED,thunder:R.THUNDER_SPEED,black_thunder:R.BLACK_SPEED,white_storm:R.WHITE_SPEED})[permanentTier()]*R.TICKS;}
function heightText(h,digits=2){const base=10**digits;return (Math.floor(h*base/1000)/base).toFixed(digits);}
function speedText(speed){return (Math.floor(speed*R.TICKS*36/R.HEIGHT_WORLD_PER_M)/10).toFixed(1);}
function unitVisible(u){return u.team===0||!$('#fog').checked;}
function tossTravel(a){const near=R.TOSS_TICKS/2-4;return a<R.SET_ARRIVAL_TICKS?Math.floor(a*near/R.SET_ARRIVAL_TICKS):a<=R.TOSS_TICKS-R.SET_ARRIVAL_TICKS?near+Math.floor((a-R.SET_ARRIVAL_TICKS)*8/(R.TOSS_TICKS-2*R.SET_ARRIVAL_TICKS)):R.TOSS_TICKS/2+4+Math.floor((a-(R.TOSS_TICKS-R.SET_ARRIVAL_TICKS))*near/R.SET_ARRIVAL_TICKS);}
function move(u,p){p={x:Math.round(p.x),y:Math.round(p.y)};if(legal(p,u.radius))u.pos=p;}
function emit(s){eventLines.push(`${(tick/60).toFixed(2)}s ${s}`);if(eventLines.length>60)eventLines.shift();}
function maxBuff(u,field,channel){return Math.max(0,...Object.values(u.buffs).filter(b=>!channel||b.channel===channel).map(b=>b[field]||0));}
function hit(u,raw,basic=false){const reduction=maxBuff(u,'reduce'),afterArmour=Math.floor(raw*100/(100+u.defence)),damage=Math.floor(afterArmour*(100-reduction)/100),absorbed=Math.min(u.shield,damage);u.shield-=absorbed;const hpLoss=Math.min(u.hp,damage-absorbed);u.hp-=hpLoss;if(u.id===1)feedback.receipts=[...(feedback.receipts||[]),{serial:++serial,source:2,delivered:hpLoss+absorbed,reduction,incoming_force:afterArmour,champion:true,verified:true,reflected:false}];emit(`${basic?'Basic':'Spike'}: ${raw} raw → ${hpLoss} HP + ${absorbed} shield`);return hpLoss+absorbed>0;}
function apply(out){const previousPhase=state.phase?.kind;state=out.state;rememberSpike(previousPhase);const delivered=new Map();for(const e of out.effects){switch(e.kind){
case'white_transform':emit('White Storm: stronger role tier replaces Black until death');break;
case'knock_up':{const u=units.find(u=>u.id===e.target);if(u&&!u.cc_immune)u.stun_until=Math.max(u.stun_until,tick+e.ticks);break;}
case'slow':{const u=units.find(u=>u.id===e.target);if(u){u.slow=tick<u.slow_until?Math.max(u.slow,e.percent):e.percent;u.slow_until=Math.max(u.slow_until||0,tick+e.ticks);}break;}
case'transform':emit('Black Thunder: +1.5m Spiker jump, +10pp Setter aura, +15pp Receiver reduction until death');break;
case'move':move(units[0],e.pos);break;
case'shield':units[0].shield+=e.amount;units[0].shield_until=tick+e.ticks;break;
case'buff':{const u=units.find(u=>u.id===e.buff.target);if(u){const key='1:'+e.buff.channel;if(e.remove)delete u.buffs[key];else u.buffs[key]=e.buff;}break;}
case'damage':{const u=units.find(u=>u.id===e.target&&u.hp>0);if(u){const positive=hit(u,e.amount,e.basic);const prev=delivered.get(e.serial);delivered.set(e.serial,{serial:e.serial,target:u.id,delivered:positive||(prev?.delivered??false),champion:(positive&&u.champion)||(prev?.champion??false)});}break;}
case'line_warning':{const previous=lineWarnings.get(e.serial);lineWarnings.set(e.serial,{...e,seen:previous?.seen??tick,react:previous?.react??tick+15});break;}
case'warning':{const previous=warnings.get(e.serial);warnings.set(e.serial,{...e,seen:previous?.seen??tick,react:previous?.react??tick+15});break;}
case'clear_warning':lineWarnings.delete(e.serial);warnings.delete(e.serial);break;
case'visual':visuals.push({...e,start:tick,until:tick+16});break;
case'reject':emit(e.reason);break;
}}feedback.hits=[...(feedback.hits||[]),...delivered.values()];}
function enqueue(kind,options={}){if(options.aim)aim={...options.aim};if(options.bars)$('#burstBars').value=options.bars;queue.push({kind,target:options.target??(kind==='ally_set'?3:target),aim:{...aim}});loaded=null;canvas.focus();running=true;$('#play').textContent='Pause';}
for(const [kind,label]of [['receive','Q Receive'],['self_set','E Self Set'],['ally_set','⇧E Teammate Set'],['approach','Space Run-up'],['takeoff','Space Takeoff'],['spike','F Spike'],['ultimate','R Serve']]){const b=document.createElement('button');b.textContent=label;b.onclick=()=>enqueue(kind);$('#techniques').append(b);}
function makeFrame(){const enemy=units[1];return{version:2,tick,owner:1,seed:1,
 units:units.map(u=>({...u,attack:Math.floor(u.attack*(100+maxBuff(u,'attack','aura')+maxBuff(u,'attack','support'))/100),visible:unitVisible(u)})),
 blocks:blocks(),smoke:smoke(),bounds:{min:{x:0,y:0},max:{x:R.MAP,y:R.MAP}},
 profile:[...document.querySelectorAll('[data-stat]')].map(e=>Number(e.value)),manual:!$('#auto').checked,action:queue.shift()||null,
 aim:{...aim},burst_bars:Number($('#burstBars').value),
 steer:{x:(held.has('d')?1:0)-(held.has('a')?1:0),y:(held.has('s')?1:0)-(held.has('w')?1:0)},
 move_step:Math.floor(1050*(100+speedBonus(units[0]))/100),skill1:true,skill2:true,ultimate:true,
 interrupted:units[0].stun_until>tick,rooted:false,danger:unitVisible(enemy)&&$('#attacking').checked&&enemy.hp>0&&distance(enemy.pos,units[0].pos)<55000?75:0,
 uncertainty:$('#fog').checked?40:0,incoming_verified:true,support_verified:true,...feedback};}
function speedBonus(u){return maxBuff(u,'speed','aura')+maxBuff(u,'speed','support')+Math.min(0,...Object.values(u.buffs).filter(b=>b.channel==='exhaustion').map(b=>b.speed));}
function nearest(a,b,p){const x=b.x-a.x,y=b.y-a.y,n=x*x+y*y,t=n?Math.max(0,Math.min(1,((p.x-a.x)*x+(p.y-a.y)*y)/n)):0;return{x:a.x+x*t,y:a.y+y*t};}
function step(){if(loaded){if(replayIndex>=loaded.length){running=false;return}const f=loaded[replayIndex++];tick=f.tick;units=structuredClone(f.units);const out=execute(f);history.push(f);outputs.push(out);apply(out);return}
 tick++;if(demo.has(tick)){const a=demo.get(tick);if(a.kind){if(a.kind==='takeoff')held.clear();if(a.bars)$('#burstBars').value=a.bars;queue.push({kind:a.kind,target:target,aim:{...aim}});}else if(a.aim)aim={...a.aim};}for(const u of units){for(const [k,b]of Object.entries(u.buffs))if(b.until&&tick>=b.until)delete u.buffs[k];if(tick>=u.shield_until)u.shield=0;}
 if(!$('#auto').checked&&state.phase?.kind==='idle'&&units[0].hp>0&&units[0].stun_until<=tick){const dx=(held.has('d')?1:0)-(held.has('a')?1:0),dy=(held.has('s')?1:0)-(held.has('w')?1:0),n=Math.hypot(dx,dy);if(n)move(units[0],{x:units[0].pos.x+dx/n*1050*(100+speedBonus(units[0]))/100,y:units[0].pos.y+dy/n*1050*(100+speedBonus(units[0]))/100});}
 const enemy=units[1];if(enemy.hp>0&&enemy.stun_until<=tick){let escaped=false;if($('#evade').checked)for(const warning of [...warnings.values(),...[...lineWarnings.values()].filter(l=>!l.harmless).map(l=>({...l,pos:nearest(l.from,l.to,enemy.pos)}))]){if(tick>=warning.react&&tick<warning.until&&Math.floor(distance(enemy.pos,warning.pos))<=warning.radius+enemy.radius){let away={x:enemy.pos.x+(enemy.pos.x-warning.pos.x||1),y:enemy.pos.y+(enemy.pos.y-warning.pos.y||1000)};move(enemy,toward(enemy.pos,away,1000*(tick<enemy.slow_until?1-enemy.slow/100:1)));escaped=true;break;}}if(!escaped&&$('#moving').checked){enemy.velocity={x:0,y:Math.round((tick%240<120?650:-650)*(tick<enemy.slow_until?1-enemy.slow/100:1))};move(enemy,{x:enemy.pos.x,y:enemy.pos.y+enemy.velocity.y});}if($('#attacking').checked&&tick>=enemy.next_attack&&distance(enemy.pos,units[0].pos)<55000){hit(units[0],100,true);enemy.next_attack=tick+60;}}
 const ally=units[2];if($('#allyAttacking').checked&&ally.hp>0&&ally.stun_until<=tick&&enemy.hp>0&&tick>=ally.next_attack&&distance(ally.pos,enemy.pos)<65000&&unitVisible(enemy)){const raw=Math.floor(ally.attack*(100+maxBuff(ally,'attack','aura')+maxBuff(ally,'attack','support'))/100),positive=hit(enemy,raw,true);ally.next_attack=tick+60;feedback.support_receipts=[{serial:++serial,ally:3,champion:true,positive,verified:true}];}
 const f=makeFrame();feedback={};const out=execute(f);history.push(f);outputs.push(out);apply(out);if(history.length>20000){history.shift();outputs.shift();}for(const [k,w]of warnings)if(tick>=w.until)warnings.delete(k);for(const[k,w]of lineWarnings)if(tick>=w.until)lineWarnings.delete(k);visuals=visuals.filter(e=>tick<e.until);}
function basic(){if(tick<units[0].next_attack||units[0].hp<=0)return;const enemy=units.find(u=>u.id===target);if(!enemy||enemy.hp<=0||!unitVisible(enemy))return;const reach=state.ball?.kind==='held'?R.VOLLEY_RANGE:R.PALM_RANGE;if(distance(units[0].pos,enemy.pos)-enemy.radius-units[0].radius>reach){emit('Basic target outside effective range');return;}feedback.basics=[{serial:++serial,target:enemy.id}];units[0].next_attack=tick+Math.ceil(60/(1.1));running=true;}
const scale=1/R.WORLD_UNITS_PER_PIXEL;let zoom=1;function screen(p,h=2000){return{x:(p.x-camera.x)*scale,y:(p.y-camera.y)*scale-(h-2000)*R.HEIGHT_PIXELS_PER_M/1000};}function circle(p,r,color){const q=screen(p);ctx.strokeStyle=color;ctx.lineWidth=2;ctx.beginPath();ctx.arc(q.x,q.y,r*scale,0,Math.PI*2);ctx.stroke();}
function drawArt(name,p,height=2000,size=null,flip=false){const im=art[name+'_p'+(Math.floor(tick/2)%8)]||art[name];if(!im)return;const q=screen(p,height),w=size||im.width,h=size?im.height*size/im.width:im.height;ctx.save();ctx.translate(q.x,q.y);if(flip)ctx.scale(-1,1);ctx.drawImage(im,-w/2,-h/2,w,h);ctx.restore();}
function ballPoint(){if(state.phase?.kind==='impossible')return{p:state.phase.origin,h:state.phase.height};const b=state.ball;if(!b)return null;if(b.kind==='held')return{p:{x:units[0].pos.x+5000,y:units[0].pos.y-3000},h:2000};if(b.kind==='toss'){const a=Math.min(R.TOSS_TICKS,Math.max(0,tick-b.start)),travel=tossTravel(a);return{p:{x:b.origin.x+(b.to.x-b.origin.x)*travel/R.TOSS_TICKS,y:b.origin.y+(b.to.y-b.origin.y)*travel/R.TOSS_TICKS},h:2000+4*(b.peak-2000)*a*(R.TOSS_TICKS-a)/R.TOSS_TICKS**2};}if(b.kind==='flying'){const total=Math.max(1,Math.floor(distance(b.origin,b.to)));return{p:b.pos,h:2000+Math.floor((b.height-2000)*Math.min(total,Math.floor(distance(b.pos,b.to)))/total)}}return b.pos?{p:b.pos,h:2000}:null;}
function render(){ctx.setTransform(1,0,0,1,0,0);ctx.imageSmoothingEnabled=false;ctx.clearRect(0,0,canvas.width,canvas.height);camera={x:units[0].pos.x-canvas.width*.35/zoom/scale,y:units[0].pos.y-Math.max(canvas.height*.67/zoom,(state.contact_height-2000)*R.HEIGHT_PIXELS_PER_M/1000+120)/scale};ctx.setTransform(zoom,0,0,zoom,0,0);ctx.fillStyle=$('#terrainTheme').value==='light'?'#b9c5b1':'#18333c';ctx.fillRect(0,0,canvas.width/zoom,canvas.height/zoom);ctx.strokeStyle='#254a53';ctx.lineWidth=1;for(let x=-camera.x*scale%40;x<canvas.width/zoom;x+=40){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,canvas.height/zoom);ctx.stroke()}for(let y=-camera.y*scale%40;y<canvas.height/zoom;y+=40){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(canvas.width/zoom,y);ctx.stroke()}
 for(const b of blocks()){const p=screen(b.min);ctx.fillStyle='#48575a';ctx.fillRect(p.x,p.y,(b.max.x-b.min.x)*scale,(b.max.y-b.min.y)*scale)}for(const b of smoke()){const p=screen(b.min);ctx.fillStyle='#8f7abb66';ctx.fillRect(p.x,p.y,(b.max.x-b.min.x)*scale,(b.max.y-b.min.y)*scale)}

 for(const crater of state.craters||[]){
  const style=crater.impossible?'impossible':({normal:'normal',thunder:'yellow',black_thunder:'black',white_storm:'white'})[crater.tier];
  const colour=({normal:'#7c9694',yellow:'#d7b94d',black:'#b54752',white:'#c9e7ed',impossible:'#bfa7ed'})[style];
  if($('#presentation').checked){drawPacked('ground_'+style,crater.pos);for(const [x,y] of [[1000,0],[707,707],[0,1000],[-707,707],[-1000,0],[-707,-707],[0,-1000],[707,-707]])drawPacked('chip_'+style,{x:crater.pos.x+Math.trunc(x*crater.radius*3/4000),y:crater.pos.y+Math.trunc(y*crater.radius*3/4000)});}
  circle(crater.pos,crater.radius,colour);
 }
   const charged=state.phase?.kind==='impossible',flying=state.ball?.kind==='flying'&&state.ball.impossible,shot=charged?state.phase.shot:flying?state.ball:null,local=shot&&!shot.serve;
  if(state.serve||charged||flying){const origin=charged?state.phase.origin:flying?state.ball.origin:units[0].pos,range=shot&&!local?null:local?R.IMPOSSIBLE_RANGE:R.SERVE_RANGE;
   ctx.save();ctx.setLineDash([8,6]);ctx.strokeStyle='#9edee477';ctx.lineWidth=1;
   if(range)circle(origin,range,'#9edee477');else{const p=screen({x:0,y:0});ctx.strokeRect(p.x,p.y,R.MAP*scale,R.MAP*scale);}ctx.restore();}
const phase=state.phase?.kind||'idle',form=(state.white_storm?'white_':state.transformed?'thunder_':'yellow_')+(state.awakened?'unmasked':'masked');if(phase==='air'||phase==='impossible'){const h=state.phase.attempted?state.last_spike_height:state.contact_height,t=phase==='impossible'||state.phase.white?'white_storm':tierAt(h);if(t!=='normal')drawArt(t==='white_storm'?'body_white':t==='black_thunder'?'body_black':'body_thunder',units[0].pos,state.contact_height);}for(const u of units){if(!unitVisible(u))continue;const p=screen(u.pos);ctx.fillStyle=u.team?'#d97f73':'#6ed6de';ctx.beginPath();ctx.ellipse(p.x,p.y+20,13,5,0,0,Math.PI*2);ctx.fill();if(u.id===1){let pose=(phase==='air'||phase==='impossible')?'spike':phase==='approach'?(state.serve?'set':'run'):state.receive_start!=null&&tick-state.receive_start<18?'receive':phase==='prepare'?'set':'idle';drawArt(form+'_'+pose,u.pos,state.contact_height||2000,64,aim.x<u.pos.x);}else{ctx.fillStyle=u.team?'#bd6f68':'#e3d9a7';ctx.fillRect(p.x-10,p.y-17,20,35);ctx.fillStyle='#f0d8b5';ctx.fillRect(p.x-8,p.y-28,16,13);}ctx.fillStyle='#07131b';ctx.fillRect(p.x-22,p.y+26,44,5);ctx.fillStyle=u.team?'#f09981':'#70e3cd';ctx.fillRect(p.x-21,p.y+27,42*Math.max(0,u.hp/u.max_hp),3);ctx.fillStyle='#dceff1';ctx.fillText(`#${u.id} ${u.hp}${u.shield?' +'+u.shield:''}`,p.x-24,p.y+44);if(Object.values(u.buffs).some(b=>b.channel==='support_ball'))drawArt('support',{x:u.pos.x,y:u.pos.y-9000});}
 if(state.helper){drawArt(form+'_set',state.helper[0]);}
 const ball=ballPoint();if(ball&&phase!=='impossible'){if(state.ball.kind==='flying'&&(state.ball.thunder||state.ball.impossible)){const p=screen(ball.p,ball.h),q=screen(state.ball.to),a=Math.atan2(q.y-p.y,q.x-p.x);ctx.save();ctx.translate(p.x,p.y);ctx.rotate(a);const trail=art[(state.ball.white||state.ball.impossible?'trail_white':state.ball.black?'trail_black':'trail_thunder')+'_p'+(Math.floor(tick/2)%8)];if(trail)ctx.drawImage(trail,-trail.width+8,-trail.height/2);ctx.restore();}const name=state.ball?.kind==='flying'&&(state.ball.thunder||state.ball.impossible)?(state.ball.white||state.ball.impossible?'white':state.ball.black?'black':'thunder'):'ball';drawArt(name,ball.p,ball.h,name==='ball'?64:64);}for(const e of visuals){if(e.name.startsWith('impossible_'))continue;const map={impossible_charge:'body_white',impossible_release:'body_white',impossible_impact:'white_impact',overpressure_release:'white_blink',overpressure_impact:'white_impact',setter:'helper',toss:'set',setter_ready:'set',serve_ready:'set',serve_impact_ring:'serve_ring',white_release:'body_white',white_blink:'white_blink',black_ready:'body_black',black_release:'body_black',spike_release:'body_thunder',contact:'ball'};const name=map[e.name]||e.name,phase=Math.min(7,Math.floor((tick-e.start)/2)),pos=['impact','black_impact','white_impact'].includes(name)?{x:e.pos.x,y:e.pos.y+R.IMPACT_ANCHOR_PIXELS*R.WORLD_UNITS_PER_PIXEL}:e.pos;drawArt(art[name+'_p'+phase]?name+'_p'+phase:name,pos,e.height||2000);}
 if($('#presentation').checked){drawStorm();drawImpactFrames();}
 for(const w of warnings.values()){circle(w.pos,w.radius,w.radius===R.STORM_RADIUS?'#bdc5ff':'#ff976c');const p=screen(w.pos);ctx.fillStyle='#ffdba7';ctx.fillText(`${Math.max(0,w.until-tick)/60|0}.${Math.floor(Math.max(0,w.until-tick)%60/6)}s`,p.x+5,p.y-5);}
 if(phase==='approach'){const p=screen(units[0].pos),age=(tick-state.phase.start)%R.GAUGE_TICKS;ctx.lineWidth=5;ctx.strokeStyle='#314855';ctx.beginPath();ctx.arc(p.x-30,p.y-5,30,Math.PI*.65,Math.PI*1.45);ctx.stroke();ctx.strokeStyle='#e9d49b';ctx.beginPath();ctx.arc(p.x-30,p.y-5,30,Math.PI*.65+Math.PI*.8*24/36,Math.PI*.65+Math.PI*.8*30/36);ctx.stroke();const a=Math.PI*.65+Math.PI*.8*Math.min(age,36)/36;ctx.fillStyle='#f6faf8';ctx.fillRect(p.x-30+Math.cos(a)*30-3,p.y-5+Math.sin(a)*30-3,6,6);}
  renderV3(phase);}

function rememberSpike(previousPhase){
 if(!state.alive){spikeCaption=null;return;}
 const shot=state.phase?.kind==='impossible'?state.phase.shot:
  state.ball?.kind==='flying'&&!state.ball.basic?state.ball:null;
 if(shot)spikeCaption={height:shot.height,speed:shot.speed,impossible:shot.impossible,until:tick+120};
 else if(previousPhase==='impossible')spikeCaption=null;
}
function drawOverhead(phase){
 if(!state.alive||units[0].hp<=0)return;
 const at=screen(units[0].pos,state.contact_height||2000),x=at.x*zoom,y=at.y*zoom;
 ctx.save();ctx.setTransform(1,0,0,1,0,0);ctx.font='bold 12px monospace';ctx.textBaseline='alphabetic';
 // HUD stays at readable screen size and follows the actual airborne body.
 const total=108,left=Math.round(x-total/2),top=Math.max(56,Math.round(y-56));
 const pace=Math.max(0,Math.min(100,state.pace||0));
 ctx.fillStyle='#07131bef';ctx.fillRect(left-7,top-5,total+14,17);
 ctx.fillStyle='#304553';ctx.fillRect(left,top,total,7);
 ctx.fillStyle='#f7d44b';ctx.fillRect(left,top,Math.round(total*pace/100),7);ctx.fillStyle='#07131b';for(const cut of [1,2])ctx.fillRect(left+total*cut/3-1,top,2,7);
 if(state.armed_bars){ctx.strokeStyle='#fff6ce';ctx.lineWidth=1;ctx.strokeRect(left-.5,top-.5,total*state.armed_bars/5+1,8);}
 const airborne=phase==='air'||phase==='impossible',fresh=spikeCaption&&tick<spikeCaption.until;
 const newJump=phase==='air'&&!state.phase.attempted;
 if(airborne||fresh){
  const shot=fresh&&!newJump?spikeCaption:null,height=shot?shot.height:state.contact_height||2000;
  const text=heightText(height,2)+' m'+(shot?'  |  '+speedText(shot.speed)+' km/h':'');
  const width=Math.ceil(ctx.measureText(text).width)+16,badgeX=Math.max(4,Math.min(canvas.width-width-4,Math.round(x-width/2))),badgeY=Math.max(5,top-49);
  ctx.fillStyle='#07131bf2';ctx.fillRect(badgeX,badgeY,width,21);ctx.fillStyle='#fff4bb';ctx.fillText(text,badgeX+8,badgeY+15);
 }
 ctx.restore();
}

function drawStorm(){
 for(const p of state.storm||[]){const active=tick>=p.active_at,q=screen(p.pos),age=tick-(p.active_at-R.STORM_WARNING),seed=p.serial%65521;
  ctx.save();ctx.strokeStyle=active?'#f2f0ff':'#9f9bd477';ctx.lineWidth=active?3:1;
  ctx.beginPath();ctx.arc(q.x,q.y,p.radius*scale,0,Math.PI*2);ctx.stroke();
  if(active)drawPacked('storm_bolt_p'+(Math.floor(age/2)%8),{x:p.pos.x,y:p.pos.y-36000});
  ctx.restore();
 }
}
function drawImpactFrames(){
 for(const cue of state.presentation||[]){const age=tick-cue.start;if(age<0||age>=R.IMPACT_DRAWINGS*R.IMPACT_FRAME_TICKS)continue;
  drawPacked('impact_release_p0',cue.pos,cue.height);
 }
}
function renderV3(phase){
 if(phase==='impossible'){const c=state.phase,age=tick-c.start;drawArt('body_white',c.origin,c.height,208);
 if($('#presentation').checked&&state.charge_presentation){const v=state.charge_presentation;drawPacked('charge_ring_'+v.stage,v.ball,v.height);for(const bolt of v.bolts)drawPacked('storm_bolt_p'+Math.floor(tick/2)%8,bolt,v.height);drawArt('ball',v.ball,v.height);}
 const p=screen(c.origin,c.height);ctx.fillStyle='#e8e7ff';ctx.font='bold 12px monospace';ctx.fillText(c.locked?'SHOT LOCKED':c.shot.serve?'IMPOSSIBLE — GLOBAL AIM':'IMPOSSIBLE — LOCAL AIM',p.x-72,p.y+55);ctx.fillText(Math.max(0,(R.IMPOSSIBLE_CHARGE-age)/60).toFixed(1)+'s',p.x-12,p.y+72);}
 for(const l of lineWarnings.values()){const a=screen(l.from),b=screen(l.to);ctx.save();ctx.setLineDash(l.locked?[8,5]:[4,8]);ctx.strokeStyle=l.locked?'#bde9f6aa':'#bde9f655';ctx.lineWidth=1;ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();ctx.restore();}
 drawOverhead(phase);
 const bank=state.bank_base?Math.min(300,Math.floor(state.caught*100/state.bank_base)):0,reserved=phase==='impossible'?state.phase.shot.caught:0;
 const rows=[['Form',state.white_storm?'White Storm':state.transformed?'Black/red':state.thunder_earned?'Yellow Thunder':'Yellow'],['Role',state.role||'spiker'],
 ['Pace',(state.pace||0)+'/100'],['Armed',state.armed_bars?state.armed_bars+' bars / '+Math.max(0,(state.armed_until-tick)/60).toFixed(1)+'s':'None'],
 ['Exhaustion',Math.max(0,((state.exhaustion_until||0)-tick)/60).toFixed(1)+'s'],['Receive saved',bank+'% / 300% - '+(state.caught||0)+' force'],['Reserved force',reserved],
 ['Spike Power',(state.last_power||100)+'%'],['Ball speed',speedText(state.last_speed||0)+' km/h (game scale)'],
 ['Height',heightText(state.contact_height||2000)+' m'],['Maximum',heightText(state.phase?.max_height||state.phase?.height||2000)+' m'],['Attack tier',spikeCaption?.impossible&&tick<spikeCaption.until?'Impossible':state.last_spike_tier||'normal'],['Ball',state.ball?.kind||'held'],
 ['Impossible',phase==='impossible'?(state.phase.shot.serve?'Global Serve':'Local 400,000')+' / '+(state.phase.locked?'Locked':'Charging'):state.impossible_ready?'10 m+ contact ready':'White + five bars + perfect; non-Serve contact ≥10 m'],['Storm patches',(state.storm||[]).length+' / '+R.STORM_LIMIT],['Blast','Endpoint only · 100% → 25%'],
 ['Cooldown Q / E / Run / R',(state.cds||[]).map(t=>Math.max(0,(t-tick)/60).toFixed(1)).join(' / ')+'s'],['Native force','Unavailable - Lab simulated']];
 $('#readout').innerHTML=rows.map(r=>'<span>'+r[0]+'</span><b>'+r[1]+'</b>').join('');
 $('#paceBars').innerHTML='<span class="pace"><i style="width:'+(state.pace||0)+'%"></i><em></em><em></em></span>';
 drawMinimap();
 $('#status').textContent=(running?'Playing':'Paused')+' - '+(tick/60).toFixed(2)+'s - rally-3.3 - '+phase;
 $('#reason').textContent=state.rejection||state.plans?.[0]?.reason||'Manual practice - no athlete errors';
 $('#plans').replaceChildren(...(state.plans||[]).slice(0,3).map(p=>{const li=document.createElement('li');li.textContent=p.actions.join(' > ')+' - '+p.score+(p.bars?' - '+p.bars+' bars':'');return li;}));
 $('#events').textContent=eventLines.slice(-12).join('\n');
}
const mapCanvas=$('#worldMap'),mapCtx=mapCanvas.getContext('2d');
function drawMinimap(){
 const m=mapCtx,w=mapCanvas.width,scale=w/R.MAP;m.clearRect(0,0,w,w);m.fillStyle='#0f2530';m.fillRect(0,0,w,w);
 m.strokeStyle='#254550';m.lineWidth=1;for(let n=1;n<8;n++){m.beginPath();m.moveTo(n*w/8,0);m.lineTo(n*w/8,w);m.moveTo(0,n*w/8);m.lineTo(w,n*w/8);m.stroke();}
 m.fillStyle='#506167';for(const b of blocks())m.fillRect(b.min.x*scale,b.min.y*scale,(b.max.x-b.min.x)*scale,(b.max.y-b.min.y)*scale);
 for(const warning of warnings.values()){m.strokeStyle=warning.locked?'#ffb47b':'#dab39477';m.beginPath();m.arc(warning.pos.x*scale,warning.pos.y*scale,warning.radius*scale,0,Math.PI*2);m.stroke();}
 for(const line of lineWarnings.values()){m.strokeStyle=line.locked?'#ffb47b':'#dab39477';m.beginPath();m.moveTo(line.from.x*scale,line.from.y*scale);m.lineTo(line.to.x*scale,line.to.y*scale);m.stroke();}
 for(const u of units){if(u.hp<=0||!unitVisible(u))continue;m.fillStyle=u.id===1?'#fff0ad':u.team?'#ef8a78':'#70d9e2';m.beginPath();m.arc(u.pos.x*scale,u.pos.y*scale,u.id===1?4:3,0,Math.PI*2);m.fill();}
 const p=state.phase?.kind==='impossible'&&state.phase.locked?state.phase.aim:aim;
 m.strokeStyle='#ffe275';m.beginPath();m.moveTo(p.x*scale-5,p.y*scale);m.lineTo(p.x*scale+5,p.y*scale);m.moveTo(p.x*scale,p.y*scale-5);m.lineTo(p.x*scale,p.y*scale+5);m.stroke();
}
function mapAim(e){const b=mapCanvas.getBoundingClientRect();aim={x:Math.round(Math.max(0,Math.min(R.MAP,(e.clientX-b.left)/b.width*R.MAP))),y:Math.round(Math.max(0,Math.min(R.MAP,(e.clientY-b.top)/b.height*R.MAP)))};canvas.focus();render();}
mapCanvas.addEventListener('pointerdown',e=>{e.preventDefault();mapAim(e);});
mapCanvas.addEventListener('pointermove',e=>{if(e.buttons===1)mapAim(e);});
canvas.addEventListener('mousemove',e=>{const b=canvas.getBoundingClientRect();aim={x:Math.round((e.clientX-b.left)*canvas.width/b.width/zoom/scale+camera.x),y:Math.round((e.clientY-b.top)*canvas.height/b.height/zoom/scale+camera.y)};aim.x=Math.max(0,Math.min(R.MAP,aim.x));aim.y=Math.max(0,Math.min(R.MAP,aim.y));});
canvas.addEventListener('click',()=>{canvas.focus();basic();});
canvas.addEventListener('keydown',e=>{const k=e.key.toLowerCase();if(['w','a','s','d','q','e','f','r',' ','escape','1','2','3','4','5'].includes(k))e.preventDefault();if(e.repeat)return;
 if('wasd'.includes(k)&&k.length===1){held.add(k);running=true;loaded=null;}else if(/^[135]$/.test(k)){$('#burstBars').value=k;render();}
 else if(k==='q')enqueue('receive');else if(k==='e')enqueue(e.shiftKey?'ally_set':'self_set');else if(k===' ')enqueue(state.phase?.kind==='approach'?'takeoff':'approach');
 else if(k==='f')enqueue('spike');else if(k==='r')enqueue(e.shiftKey?'arm_burst':'ultimate');else if(k==='escape')enqueue('cancel_burst');});
canvas.addEventListener('keyup',e=>held.delete(e.key.toLowerCase()));canvas.addEventListener('blur',()=>held.clear());window.addEventListener('blur',()=>held.clear());
$('#presentation').onchange=render;$('#terrainTheme').onchange=render;
$('#armBurst').onclick=()=>enqueue('arm_burst');$('#cancelBurst').onclick=()=>enqueue('cancel_burst');
$('#play').onclick=()=>{running=!running;$('#play').textContent=running?'Pause':'Play';};$('#step').onclick=()=>{running=false;step();render();};$('#reset').onclick=()=>reset(recordingSetup);
function preset(){reset({pace:Number($('#practicePace').value),form:$('#powerForm').value,bank_percent:Number($('#practiceBank').value)});}
$('#powerForm').onchange=preset;$('#applyPractice').onclick=preset;
$('#impossibleFixture').onclick=()=>{reset({pace:100,form:'white_storm',bank_percent:Number($('#practiceBank').value)});$('#burstBars').value=5;render();};
function startImpossibleDemo(){
 const direct=$('#demoSetup').value==='spike_only';
 for(const id of ['auto','moving','attacking','allyAttacking','fog','walls','smoke','evade'])$('#'+id).checked=false;
 reset({pace:100,form:'white_storm',bank_percent:Number($('#practiceBank').value)});
 $('#burstBars').value=5;aim={x:900000,y:300000};zoom=.5;$('#zoom').value=zoom;
 demo=direct?new Map([[2,{kind:'arm_burst',bars:5}],[3,{kind:'self_set'}],[16,{kind:'approach'}],[40,{kind:'takeoff'}],[64,{kind:'spike'}],[110,{aim:{x:460000,y:380000}}]]):
 new Map([[2,{kind:'arm_burst',bars:5}],[3,{kind:'self_set'}],[16,{kind:'ultimate'}],[64,{kind:'takeoff'}],[88,{kind:'spike'}],[110,{aim:{x:600000,y:340000}}]]);
 if(direct)held.add('d');
 $('#recordingInfo').textContent='Impossible demo: White + 100 Pace practice preset; simulated Receive bank '+Number($('#practiceBank').value)+'%. '+(direct?'Five-bar Self Set, no Serve: 10.6 m contact and 400,000 range.':'Five-bar Self Set + Serve: global endpoint.')+' 5 s power-building charge, final 1 s lock; warned storm slows and endpoint-only damage.';
 running=true;$('#play').textContent='Pause';
}
$('#impossibleDemo').onclick=startImpossibleDemo;
$('#receiverFixture').onclick=()=>{reset({pace:100,form:'base'});units[1].pos={x:180000,y:300000};$('#attacking').checked=true;render();};
$('#supportFixture').onclick=()=>{reset();units[1].pos={x:220000,y:300000};$('#allyAttacking').checked=true;render();};
$('#deathFixture').onclick=()=>{units[0].hp=units[0].hp>0?0:units[0].max_hp;step();render();};
const zoomSelect=document.createElement('select');zoomSelect.id='zoom';for(const value of [0.5,0.75,1,1.5,2]){const o=document.createElement('option');o.value=value;o.textContent=value+'x view';zoomSelect.append(o);}zoomSelect.value=1;zoomSelect.onchange=()=>{zoom=Number(zoomSelect.value);render();};$('#play').parentElement.append(zoomSelect);
function applyPractice(changes){const bytes=enc.encode(JSON.stringify(changes)),p=wasm.rally_input(bytes.length);new Uint8Array(wasm.memory.buffer,p,bytes.length).set(bytes);if(!wasm.rally_fixture(session,bytes.length))throw Error('Invalid practice setup');}
$('#save').onclick=()=>{const blob=new Blob([JSON.stringify({version:2,rules_revision:window.RALLY_RULES.revision,practice_setup:recordingSetup,frames:history})],{type:'application/json'}),a=document.createElement('a');a.href=URL.createObjectURL(blob);a.download='rally-3-recording.json';a.click();URL.revokeObjectURL(a.href);};
function loadRecording(r){$('#recordingInfo').textContent='Transport '+r.version+' / '+r.rules_revision+' / '+(r.frames?.length||0)+' frames';if(r.version!==2||r.rules_revision!==window.RALLY_RULES.revision||!Array.isArray(r.frames)||r.frames.length>20000)throw Error('Inspectable recording; replay requires its original Rally rules and transport.');reset(r.practice_setup||{},false);loaded=r.frames;replayIndex=0;running=true;}
$('#load').onchange=async e=>{try{loadRecording(JSON.parse(await e.target.files[0].text()));}catch(error){$('#status').textContent=error.message;}};
window.addEventListener('message',e=>{if(e.origin===location.origin&&e.data?.type==='rally-pause'){running=false;held.clear();}});
window.rallyLab={get state(){return state;},get units(){return units;},get tick(){return tick;},get frames(){return history;},get outputs(){return outputs;},
 get warnings(){return [...warnings.values()];},get lineWarnings(){return [...lineWarnings.values()];},loadRecording,reset,execute,isVisible:unitVisible,
 step:()=>{step();render();},pause:()=>{running=false;render();},enqueue,command:enqueue,fixture:reset,render,setAim:p=>{aim={...p};}};
function loop(now){const dt=Math.min(100,now-last);last=now;if(running){accum+=dt;while(accum>=1000/60){step();accum-=1000/60;}}else accum=0;render();requestAnimationFrame(loop)}reset();const params=new URLSearchParams(location.search);if(params.get('route')==='spike')$('#demoSetup').value='spike_only';if(params.get('demo')==='impossible')startImpossibleDemo();requestAnimationFrame(loop);
})().catch(e=>{document.querySelector('#status').textContent=e.message;console.error(e)});
