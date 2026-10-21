(async()=>{'use strict';
const $=s=>document.querySelector(s),names=['Macro','Micro','Map Awareness','Strategy'],canvas=$('#arena'),ctx=canvas.getContext('2d');
const api=(await WebAssembly.instantiate(await(await fetch('universal.wasm')).arrayBuffer())).instance.exports;
let session=api.universal_new(),actors=[],targets=[],rects=[],recent=0,unseen=0,last=null,history=[],drag=null;
const actor=(id,team,x,y)=>({id,athlete:100+id,team,x,y,hp:1000,max:1000,speed:1000,power:100,range:40000,role:(id-1)%5,controlled:false,ultimate:true,visible:true,ratings:[80,80,80,80],judgement:80,ego:50});
$('#attributes').innerHTML=names.map((n,k)=>'<label>'+n+'<input id="sense'+k+'" type="number" min="0" max="100" value="80"></label>').join('');
$('#focus').innerHTML=names.map((n,k)=>'<option value="'+k+'">'+n+'</option>').join('');
function reset(){
 session=api.universal_new();history=[];recent=unseen=0;rects=[];
 actors=[actor(1,0,700000,650000),actor(2,0,720000,620000),actor(3,0,750000,600000),actor(4,0,800000,600000),actor(5,0,700000,700000),actor(6,1,550000,450000),actor(7,1,580000,430000),actor(8,1,600000,450000),actor(9,1,610000,460000),actor(10,1,620000,470000)];
 targets=[{id:99,kind:2,team:2,x:480000,y:480000,hp:18000,max:20000,loss:0,visible:true,value:35},{id:90,kind:0,team:1,x:660000,y:600000,hp:500,max:500,loss:0,visible:true,value:10},{id:80,kind:5,team:1,x:300000,y:720000,hp:3000,max:5000,loss:0,visible:true,value:25}];
 const f=$('#fixture').value;
 if(['start','finish','stall','lost','conversion'].includes(f))actors.slice(0,2).forEach((a,k)=>{a.x=480000+k*20000;a.y=500000;});
 if(f==='start')actors.slice(5).forEach(a=>a.x=900000);
 if(f==='finish'){targets[0].hp=700;targets[0].loss=1500;actors.slice(5).forEach(a=>a.x=900000);}
 if(f==='stall'){actors[2].x=580000;actors[2].y=550000;actors[3].x=590000;actors[3].y=550000;targets[0].loss=500;}
 if(f==='lost'){targets[0].hp=300;targets[0].loss=4000;actors[1].hp=100;actors.slice(5).forEach((a,k)=>{a.x=510000+k*10000;a.y=480000;});recent=-2;}
 if(f==='wave')targets[0].visible=false;
 if(f==='fog'){actors.slice(5).forEach(a=>a.visible=false);unseen=30;}
 if(f==='blocked')rects=[[530000,380000,570000,680000]];
 if(f==='reinforce'){actors[0].x=480000;actors[0].y=480000;actors[0].hp=350;actors.slice(5).forEach((a,k)=>{a.x=490000+k*10000;a.y=480000;});targets=[{id:1,kind:6,team:0,x:480000,y:480000,hp:350,max:1000,loss:0,visible:true,value:20}];}
 if(f==='reset'){actors.slice(0,5).forEach(a=>a.hp=200);actors.slice(5).forEach(a=>a.visible=false);targets=[{id:70,kind:7,team:0,x:96000,y:864000,hp:1,max:1,loss:0,visible:true,value:0}];}
 if(f==='conversion'){recent=3;actors.slice(5).forEach(a=>a.hp=0);targets[0].visible=false;}
 $('#athlete').innerHTML=actors.map(a=>'<option value="'+a.id+'">Athlete #'+a.athlete+(a.team?' · Red':' · Blue')+'</option>').join('');
 $('#athlete').value=+$('#team').value===0?'1':'6';sync();evaluate();
}
function selected(){return actors.find(a=>a.id===+$('#athlete').value);}
function sync(){const a=selected();names.forEach((_,k)=>$('#sense'+k).value=a.ratings[k]);$('#judgement').value=a.judgement;$('#ego').value=a.ego;$('#role').value=a.role;}
function edit(){const a=selected();a.ratings=names.map((_,k)=>Math.max(0,Math.min(100,+$('#sense'+k).value)));a.judgement=+$('#judgement').value;a.ego=+$('#ego').value;a.role=+$('#role').value;evaluate();}
function frame(op=0){return[1,+$('#team').value,recent,unseen,actors.length,targets.length,rects.length,op,+$('#focus').value,+$('#day').value,+$('#set').value,15,50,50,50,50,0,0,0,0,...actors.flatMap(a=>[a.id,a.athlete,a.team,Math.round(a.x),Math.round(a.y),a.hp,a.max,a.speed,a.power,a.range,a.role,+a.controlled,+a.ultimate,+a.visible,...a.ratings,a.judgement,a.ego]),...targets.flatMap(t=>[t.id,t.kind,t.team,t.x,t.y,t.hp,t.max,t.loss,+t.visible,t.value]),...rects.flat()];}
function invoke(h,a){const p=api.universal_input(a.length);new BigInt64Array(api.memory.buffer,p,a.length).set(a.map(BigInt));const q=api.universal_step(h,a.length);return JSON.parse(new TextDecoder().decode(new Uint8Array(api.memory.buffer,q,api.universal_len())));}
function evaluate(op=0){const a=frame(op),out=invoke(session,a);if(out.error)throw Error(out.error);last=out;history.push({input:a,output:out});$('#plans').textContent=out.plans.map((p,n)=>(n?'Alternative: ':'Selected: ')+p.goal+' · score '+p.score+' · caller #'+p.caller+'\n'+p.reason+'\n'+p.jobs.map(j=>'#'+j.actor+': '+j.job+' → '+j.goal+' #'+j.target).join(' · ')).join('\n\n');$('#development').textContent=names.map((n,k)=>n+': '+out.development.ratings[k]+' ('+out.development.progress[k]+'/100)').join('\n')+'\n'+(out.development.changed?'Award recorded':'No new award — repeated days/sets do not count');draw();return out;}
function draw(){ctx.fillStyle='#142f33';ctx.fillRect(0,0,960,960);ctx.strokeStyle='#28464a';for(let n=0;n<960;n+=32){ctx.beginPath();ctx.moveTo(n,0);ctx.lineTo(n,960);ctx.moveTo(0,n);ctx.lineTo(960,n);ctx.stroke();}
 for(const r of rects){ctx.fillStyle='#6b6f76';ctx.fillRect(r[0]/1000,r[1]/1000,(r[2]-r[0])/1000,(r[3]-r[1])/1000);}
 if(last?.plans[0])for(const j of last.plans[0].jobs){const a=actors.find(a=>a.id===j.actor);ctx.strokeStyle='#b2c7b977';ctx.beginPath();ctx.moveTo(a.x/1000,a.y/1000);ctx.lineTo(j.point[0]/1000,j.point[1]/1000);ctx.stroke();ctx.fillStyle='#dae8d9';ctx.fillText(j.job,j.point[0]/1000,j.point[1]/1000);}
 for(const t of targets){if(!t.visible)continue;ctx.fillStyle=t.kind===0?'#cdbf88':'#ce9d60';ctx.fillRect(t.x/1000-12,t.y/1000-12,24,24);ctx.fillText(t.kind===2?'Morgard':t.kind===3?'Serpen':t.kind===0?'Wave':t.kind===6?'Ally':t.kind===7?'Base':'Tower',t.x/1000-20,t.y/1000-20);}
 for(const a of actors){if(!a.visible||a.hp<=0)continue;ctx.fillStyle=a.team?'#e38396':'#75d9b9';ctx.beginPath();ctx.arc(a.x/1000,a.y/1000,11,0,7);ctx.fill();ctx.fillText('#'+a.id,a.x/1000-7,a.y/1000-20);if(a.id===+$('#athlete').value){ctx.strokeStyle='white';ctx.beginPath();ctx.arc(a.x/1000,a.y/1000,17,0,7);ctx.stroke();}}
}
function coverage(){const q=$('#filter').value.toLowerCase(),rows=window.FARMING_CAPABILITIES.champions.filter(c=>c.id.includes(q));$('#coverage').innerHTML='<table><tr><th>Champion</th><th>Route</th><th>Skill contributions</th><th>Live casts</th></tr>'+rows.map(c=>'<tr><td>'+c.id+'</td><td>'+c.source+'</td><td>'+c.skills.map(s=>s.slot+': '+s.purpose).join('<br>')+'</td><td>Pending</td></tr>').join('')+'</table>';}
function verify(){const h=api.universal_new();for(const row of history)if(JSON.stringify(invoke(h,row.input))!==JSON.stringify(row.output))throw Error('Replay diverged');$('#status').textContent='Verified '+history.length+' inputs';return history.length;}
$('#reset').onclick=reset;$('#fixture').onchange=reset;$('#team').onchange=reset;$('#evaluate').onclick=()=>evaluate();$('#athlete').onchange=sync;names.forEach((_,k)=>$('#sense'+k).onchange=edit);$('#judgement').onchange=$('#ego').onchange=$('#role').onchange=edit;
$('#focus').onchange=()=>evaluate(1);$('#training').onclick=()=>evaluate(2);$('#result').onclick=()=>evaluate(3);$('#verify').onclick=verify;$('#filter').oninput=coverage;
$('#export').onclick=()=>{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify({version:1,frames:history})],{type:'application/json'}));a.download='game-sense-practice.json';a.click();setTimeout(()=>URL.revokeObjectURL(a.href),1000);};
$('#careerRefresh').onclick=async()=>{try{const r=await(await fetch('/api/game-sense')).json();$('#career').textContent=r.profiles?JSON.stringify(r.profiles,null,2):r.message||'No profile export yet. Open the updated mod in game.';}catch(e){$('#career').textContent=e.message;}};
canvas.onpointerdown=e=>{const b=canvas.getBoundingClientRect(),x=(e.clientX-b.left)*960000/b.width,y=(e.clientY-b.top)*960000/b.height;drag=actors.find(a=>a.visible&&Math.hypot(a.x-x,a.y-y)<20000);canvas.setPointerCapture(e.pointerId);};
canvas.onpointermove=e=>{if(!drag)return;const b=canvas.getBoundingClientRect();drag.x=Math.max(0,Math.min(959999,(e.clientX-b.left)*960000/b.width));drag.y=Math.max(0,Math.min(959999,(e.clientY-b.top)*960000/b.height));draw();};
canvas.onpointerup=()=>{drag=null;evaluate();};
window.UniversalLab={reset,evaluate,frame,invoke,verify,get snapshot(){return last},get actors(){return actors},get targets(){return targets},get history(){return history}};reset();coverage();$('#ready').textContent='Shared planner and development rules ready · v1';
})().catch(e=>{document.querySelector('#ready').textContent='Lab failed: '+e.message;console.error(e);});
