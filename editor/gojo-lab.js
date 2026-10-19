/* Scene-only adapter. Decisions and combat commands come from gojo_core.rs in WASM. */
(async()=>{'use strict';
const $=s=>document.querySelector(s),canvas=$('#arena'),ctx=canvas.getContext('2d');
let api,handles=[],units=[],tick=0,serial=0,queued=0,basic=0,selected=2,aim=[130000,100000],paused=false,controlUntil=0,snapshot,states=new Map(),effects=[],warnings=[],events=[],recording=[],last=null,acc=0,replaying=false;
const art={};const names=['Blue','Maximum Blue','Red','Maximum Red','Purple','RCT','Spatial Step','Domain'];
const log=s=>{events.unshift((tick/60).toFixed(2)+'s  '+s);events.length=Math.min(events.length,80);$('#events').textContent=events.join('\n')};
const unit=(id,team,x,y,champion=true)=>({id,team,x,y,hp:id===1?935:2200,max:id===1?935:2200,ad:70,ap:id===1?81:60,level:3,champion,visible:true,targetable:true,immune:false,farmable:!champion,tower:false,stunned:0,shield:0,hit:false});
function frame(owner,action,hit,target=selected){
 const me=units.find(u=>u.id===owner),walls=$('#wall').checked?Array.from({length:10},(_,n)=>[9,n+3]):[];
 const barriers=[...states.entries()].filter(([id,s])=>id!==owner&&s.domain&&s.domain[5]!==owner&&tick>=s.domain[2]+s.domain[3]&&tick<s.domain[4]).map(([,s])=>[s.domain[0],s.domain[1],76000]);
 const haste=Object.values(me.powers||{}).filter(p=>p.end>tick&&p.owner!==owner).length*100;
 return[1,tick,owner,+$('#automatic').checked,action,selected,...aim,$('#unlock').checked?7:3,+(tick<controlUntil&&owner===1||me.stunned>tick),+(me.held>tick),hit?serial:0,hit?target:0,+me.hit,me.infinityPublished?Math.round(me.shield):-1,units.length,...units.flatMap(u=>[u.id,u.team,Math.round(u.x),Math.round(u.y),Math.round(u.hp),u.max,u.ad,u.ap,u.level,+u.champion,+u.visible,+u.targetable,(+(u.immune||u.guardUntil>tick))+(u.ccImmune||u.guardUntil>tick?2:0),+u.farmable,+u.tower]),walls.length,...walls.flat(),haste,0,barriers.length,...barriers.flat()];
}
function invoke(handle,a){const p=api.gojo_input(a.length);new BigInt64Array(api.memory.buffer,p,a.length).set(a.map(BigInt));const q=api.gojo_step(handle,a.length);const s=JSON.parse(new TextDecoder().decode(new Uint8Array(api.memory.buffer,q,api.gojo_output_len())));if(s.error)throw Error(s.error);return s;}
function hit(u,n,owner){
 if(!u||u.hp<=0||u.immune||u.guardUntil>tick)return;
 if(owner!==undefined&&units.find(v=>v.id===owner)?.champion&&u.champion)u.hit=true;
 const absorbed=Math.min(u.shield,n);u.shield-=absorbed;u.hp=Math.max(0,u.hp-(n-absorbed));
}
function apply(s,owner){
 for(const c of s.commands){const u=units.find(u=>u.id===c[1]);switch(c[0]){
 case 0:{let n=c[2]+c[3];if(states.get(c[1])?.infinity)n=Math.floor(n*(c[4]?.7:.25));hit(u,n,owner);log((c[4]?'Melee':'Technique')+' → '+c[1]+' · '+n+' raw before shields');break;}
 case 1:if(u){u.x=c[2];u.y=c[3];}break;
 case 2:{const me=units.find(u=>u.id===owner);me.hp=Math.min(me.max,me.hp+c[1]);break;}
 case 3:{const me=units.find(u=>u.id===owner);if(c[2]<=0)me.infinityPublished=false;if(c[1]&&!me.infinityPublished){me.shield+=c[2];me.infinityPublished=true;}break;}
 case 4:if(u)u.stunned=Math.max(u.stunned,tick+c[2]);break;
 case 5:if(u)u.guardUntil=tick+c[2];break;
 case 6:effects.push({kind:c[1],x:c[2],y:c[3],start:tick,end:tick+c[4],flight:c[1]===12||c[1]===13||c[1]===7&&c[4]<=2,origin:s.lockUntil});break;
 case 7:warnings.push({x:c[1],y:c[2],r:c[3],end:tick+c[4]});break;
 case 9:for(const v of units)if(v.guardOwner===owner)v.guardUntil=0;log('Domain closes · burnout');break;
 case 10:if(u){u.powers||={};u.powers[owner]={owner,end:tick+c[2],slow:!!c[3]};}break;
 case 11:if(u)u.held=Math.max(u.held||0,tick+c[2]);break;
 }if(c[0]===5&&u)u.guardOwner=owner;}
}
function refreshStats(){for(const u of units){const powers=Object.values(u.powers||{}).filter(p=>p.end>tick);const bonus=powers.reduce((n,p)=>n+(p.slow?30:15),0);u.ad=Math.floor((u.baseAd||70)*(100+bonus)/100);u.ap=Math.floor((u.baseAp||(u.id===1?81:60))*(100+bonus)/100);}}
function run(){
 if(replaying)return;tick++;refreshStats();
 for(const u of units){u.visible=u.id!==selected||!$('#hidden').checked;if(u.team===1&&$('#moving').checked&&u.hp>0&&u.stunned<=tick&&!(u.held>tick))u.y=100000+Math.round(Math.sin(tick/75+u.id)*18000);}
 for(const [owner,h]of handles){
  const me=units.find(u=>u.id===owner),old=states.get(owner);let target=selected,attack=owner===1&&basic;
  const speed=Object.values(me.powers||{}).filter(p=>p.end>tick).reduce((n,p)=>n+(p.slow?25:40),100);
  if($('#automatic').checked&&me.hp>0&&me.stunned<=tick&&!(old?.lockUntil>tick)&&tick>=(me.attackAt||0)){
   const foe=units.filter(u=>u.team!==me.team&&u.hp>0&&u.visible&&u.targetable&&Math.hypot(u.x-me.x,u.y-me.y)<=23000).sort((a,b)=>(a.id===old?.combo?.[0]?-1:0)-(b.id===old?.combo?.[0]?-1:0)||Math.hypot(a.x-me.x,a.y-me.y)-Math.hypot(b.x-me.x,b.y-me.y)||a.id-b.id)[0];
   if(foe){target=foe.id;attack=++serial;me.attackAt=tick+Math.ceil(5500/speed);}
  }
  const a=frame(owner,owner===1?queued:0,attack,target),s=invoke(h,a);states.set(owner,s);apply(s,owner);if(s.flashes>(old?.flashes||0))log('BLACK FLASH · 20% damage for eight seconds');if(owner===1)snapshot=s;me.hit=false;
  if(recording.length<20000)recording.push({owner,input:a,output:s,scene:JSON.parse(JSON.stringify(units)),effects:JSON.parse(JSON.stringify(effects)),warnings:JSON.parse(JSON.stringify(warnings)),serial,controlUntil});
 }
 queued=0;basic=0;
 if($('#incoming').checked&&tick%55===0){for(const u of units.filter(u=>!handles.some(h=>h[0]===u.id)&&u.hp>0&&u.stunned<=tick)){const v=units.filter(v=>v.team!==u.team&&v.hp>0&&v.visible).sort((a,b)=>Math.hypot(a.x-u.x,a.y-u.y)-Math.hypot(b.x-u.x,b.y-u.y)||a.id-b.id)[0];if(v&&Math.hypot(v.x-u.x,v.y-u.y)<=60000)hit(v,states.get(v.id)?.infinity?24:35,u.id);}}
 effects=effects.filter(e=>e.end>tick).slice(-48);warnings=warnings.filter(e=>e.end>tick);if(recording.length){const row=recording.at(-1);row.scene=JSON.parse(JSON.stringify(units));row.effects=JSON.parse(JSON.stringify(effects));row.warnings=JSON.parse(JSON.stringify(warnings));}update();draw();
}
function update(){if(!snapshot)return;$('#ce').textContent=(snapshot.ce/1000).toFixed(1)+' / 100';$('#strain').textContent=(snapshot.strain/1000).toFixed(1)+' / 100';$('#cebar').value=snapshot.ce/1000;$('#strainbar').value=snapshot.strain/1000;$('#flash').textContent=snapshot.amplifiedUntil>tick?'20% · '+((snapshot.amplifiedUntil-tick)/60).toFixed(1)+'s':snapshot.flashes+' earned';$('#guest').textContent=snapshot.domain?(snapshot.domain[5]<0?'No ally selected':'Ally '+snapshot.domain[5]):'No active Domain';$('#reason').textContent=snapshot.reason+(snapshot.burnout>tick?' · burnout '+((snapshot.burnout-tick)/60).toFixed(1)+'s':'');$('#cooldowns').textContent=names.map((n,i)=>n+': '+(snapshot.cooldowns[i]>tick?((snapshot.cooldowns[i]-tick)/60).toFixed(1)+'s':'ready')).join(' · ');$('#combo').textContent=(snapshot.combo?'Combo: enemy '+snapshot.combo[0]+' · '+snapshot.combo[2]+'/2 punches · '+((snapshot.combo[1]-tick)/60).toFixed(1)+'s. ':'')+'Selected target '+selected+' · Space: melee · Q/W: Blue/Red · Shift Q/W: maximum · E: Domain · R: heal · F: step · P: Purple';$('#state').textContent=JSON.stringify(snapshot,null,2);}
function sprite(sheet,tag,x,y,time=0,flip=false){const a=art[sheet];if(!a)return;const frames=a.meta.anims[tag]?.frames;if(!frames)return;const total=frames.reduce((n,f)=>n+f.duration,0);let at=((time/60)%total+total)%total;const f=(frames.find(f=>{at-=f.duration;return at<0})||frames.at(-1)).data;ctx.save();ctx.translate(x,y);if(flip)ctx.scale(-1,1);ctx.imageSmoothingEnabled=false;ctx.drawImage(a.image,f.x,f.y,f.w,f.h,-f.w/2,-f.h/2,f.w,f.h);ctx.restore();}
function draw(){ctx.clearRect(0,0,1080,600);const scale=1/300;ctx.strokeStyle='#233847';for(let x=0;x<1080;x+=40){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,600);ctx.stroke()}for(let y=0;y<600;y+=40){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(1080,y);ctx.stroke()}
if($('#wall').checked){ctx.fillStyle='#606e75';ctx.fillRect(144000*scale,48000*scale,16000*scale,160000*scale)}
for(const w of warnings){ctx.strokeStyle='#f2b263';ctx.setLineDash([7,5]);ctx.beginPath();ctx.arc(w.x*scale,w.y*scale,w.r*scale,0,Math.PI*2);ctx.stroke();ctx.setLineDash([])}
for(const s of states.values())if(s.domain){const[x,y,start,expand,end,guest]=s.domain,r=8000+68000*Math.min(1,(tick-start)/expand);ctx.fillStyle='#6450a71c';ctx.strokeStyle='#bcadff';ctx.beginPath();ctx.arc(x*scale,y*scale,r*scale,0,Math.PI*2);ctx.fill();ctx.stroke();if(guest>=0){const u=units.find(u=>u.id===guest);if(u)sprite('fx','fx11',u.x*scale,u.y*scale-30,tick)}}
for(const u of units){const x=u.x*scale,y=u.y*scale;ctx.globalAlpha=u.visible?1:.25;ctx.fillStyle=u.hp<=0?'#495461':u.team?'#cf7f82':'#7bb8c7';ctx.beginPath();ctx.arc(x,y,11,0,Math.PI*2);ctx.fill();if(u.id===1||u.id===9)sprite('body',states.get(u.id)?.lockUntil>tick?'skill1':'idle',x,y-10,tick,$('#facing').checked);ctx.strokeStyle=u.id===selected?'#f6d48a':'transparent';ctx.lineWidth=2;ctx.stroke();ctx.fillStyle='#d5e9ed';ctx.font='13px system-ui';ctx.fillText(u.id===1?'Gojo':u.id===9?'Gojo 2':u.team?'Enemy '+u.id:'Ally '+u.id,x-24,y-38);ctx.fillStyle='#394855';ctx.fillRect(x-22,y+20,44,4);ctx.fillStyle='#9ec99d';ctx.fillRect(x-22,y+20,44*u.hp/u.max,4);if(u.stunned>tick){ctx.fillStyle='#ffcc73';ctx.fillText('STUN '+((u.stunned-tick)/60).toFixed(1)+'s',x-28,y+40)}if(u.shield>0){ctx.strokeStyle='#99efff';ctx.beginPath();ctx.arc(x,y,20,0,Math.PI*2);ctx.stroke()}ctx.globalAlpha=1;}
for(const e of effects){if(e.flight){const name=e.kind===13?'blue':e.kind===12?'red':'purple',count=name==='purple'?6:4,frame=Math.floor(Math.max(0,tick-e.origin)*5/18)%count;sprite('fx','flight_'+name+frame,e.x*scale,e.y*scale,0);}else sprite('fx','fx'+e.kind,e.x*scale,e.y*scale,tick-e.start);}
ctx.strokeStyle='#79d6ec';ctx.beginPath();ctx.moveTo(aim[0]*scale-6,aim[1]*scale);ctx.lineTo(aim[0]*scale+6,aim[1]*scale);ctx.moveTo(aim[0]*scale,aim[1]*scale-6);ctx.lineTo(aim[0]*scale,aim[1]*scale+6);ctx.stroke();
const inspect=+$('#inspect').value;if(inspect>=0){ctx.fillStyle='#0b1422ee';ctx.fillRect(0,0,1080,600);sprite('body','skill1',470,300,tick,$('#facing').checked);sprite('fx','fx'+inspect,570,280,tick);ctx.fillStyle='white';ctx.fillText('Actual-size pose and effect · '+$('#inspect').selectedOptions[0].textContent,350,390);}}
function reset(){for(const[,h]of handles)api.gojo_drop(h);handles=[[1,api.gojo_new()]];states.clear();tick=0;serial=0;queued=basic=0;controlUntil=0;effects=[];warnings=[];recording=[];events=[];selected=2;units=[unit(1,0,100000,100000),unit(2,1,120000,100000)];const f=$('#fixture').value;if(f==='team'){units.push(unit(3,0,104000,100000),unit(4,0,108000,100000),unit(5,1,140000,120000))}if(f==='farm'){units[1].champion=false;units[1].farmable=true}if(f==='two'){units.push(unit(9,1,170000,120000),unit(4,0,110000,130000));handles.push([9,api.gojo_new()])}run();log('Fixture reset');}
function submit(code){queued=code;log(names[code-1]+' requested');if(paused)run()}
document.querySelectorAll('[data-action]').forEach(b=>b.onclick=()=>submit(+b.dataset.action));$('#punch').onclick=()=>{basic=++serial;if(paused)run()};$('#pause').onclick=()=>{paused=!paused;$('#pause').textContent=paused?'Resume':'Pause'};$('#step').onclick=()=>{paused=true;$('#pause').textContent='Resume';run()};$('#second').onclick=()=>{paused=true;$('#pause').textContent='Resume';for(let i=0;i<60;i++)run()};$('#interrupt').onclick=()=>{controlUntil=tick+45;run()};$('#death').onclick=()=>{units[0].hp=units[0].hp>0?0:units[0].max;if(!units[0].hp){units[0].shield=0;units[0].infinityPublished=false;}run()};$('#reset').onclick=reset;$('#fixture').onchange=reset;
canvas.onclick=e=>{const r=canvas.getBoundingClientRect(),x=(e.clientX-r.left)*1080/r.width*300,y=(e.clientY-r.top)*600/r.height*300;const u=units.find(u=>u.team===1&&Math.hypot(u.x-x,u.y-y)<12000);if(u)selected=u.id;else aim=[Math.round(x),Math.round(y)];draw()};
$('#inspect').onchange=draw;$('#facing').onchange=draw;window.addEventListener('keydown',e=>{if(/INPUT|SELECT|TEXTAREA/.test(e.target.tagName))return;const map={q:e.shiftKey?2:1,w:e.shiftKey?4:3,e:8,r:6,f:7,p:5};if(e.key===' '){$('#punch').click();e.preventDefault()}else if(map[e.key.toLowerCase()]){submit(map[e.key.toLowerCase()]);e.preventDefault()}});
$('#export').onclick=()=>{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify({version:1,frames:recording})],{type:'application/json'}));a.download='gojo-replay.json';a.click();URL.revokeObjectURL(a.href)};
$('#replay').onclick=()=>{
 paused=true;$('#pause').textContent='Resume';if(!recording.length)return;replaying=true;const replay=new Map();
 try{for(const row of recording){if(!replay.has(row.owner))replay.set(row.owner,api.gojo_new());const s=invoke(replay.get(row.owner),row.input);if(JSON.stringify(s)!==JSON.stringify(row.output))throw Error('Replay diverged at tick '+row.input[1]);states.set(row.owner,s);}
  for(const[,h]of handles)api.gojo_drop(h);handles=[...replay.entries()];const row=recording.at(-1);units=JSON.parse(JSON.stringify(row.scene));effects=row.effects;warnings=row.warnings;tick=row.input[1];serial=row.serial;controlUntil=row.controlUntil;snapshot=states.get(1);log('Replay verified '+recording.length+' complete inputs; scene and controls restored');
 }catch(e){for(const h of replay.values())api.gojo_drop(h);log(e.message);}finally{replaying=false;update();draw();}
};
window.addEventListener('message',e=>{if(e.origin===location.origin&&e.data?.type==='gojo-pause'){paused=true;$('#pause').textContent='Resume'}});
try{for(const[name,prefix]of[['body','gojo-body'],['fx','gojo-fx']]){const image=new Image();image.src='/'+prefix+'.png';await image.decode();art[name]={image,meta:await(await fetch('/'+prefix+'.fanim')).json()}}api=(await WebAssembly.instantiate(await(await fetch('/gojo.wasm')).arrayBuffer())).instance.exports;reset();window.GojoLab={get snapshot(){return snapshot},get units(){return units},get recording(){return recording},get tick(){return tick},step:run,submit,reset,punch(){basic=++serial;run()},pause(){paused=true;$('#pause').textContent='Resume'}};requestAnimationFrame(function animate(t){if(last!==null&&!paused&&!document.hidden){acc+=Math.min(.1,(t-last)/1000);while(acc>=1/60){run();acc-=1/60}}last=t;requestAnimationFrame(animate)})}catch(e){$('#reason').textContent='Could not load Gojo rules: '+e.message;console.error(e)}
})();
