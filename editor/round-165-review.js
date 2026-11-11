// Read-only actual-map presentation inspector. Combat receipts come from WASM.
(async()=>{
 const $=id=>document.getElementById(id),art=new Round165Painter(),body=new ThalassaPainter(),world=await new GameWorld().ready;
 const w=(await WebAssembly.instantiate(await(await fetch('thalassa.wasm')).arrayBuffer(),{})).instance.exports;
 await Promise.all([art.ready,body.ready,world.loadSprites(['champ-knight','champ-ninja','minions'])]);
 const canvas=$('mapCanvas'),ctx=canvas.getContext('2d');let origin=world.legal({x:480000,y:480000},10000),enemy,mid=null,begin=-10000,water=true,last=[];
 function call(frame){const b=new TextEncoder().encode(JSON.stringify({version:1,rules_revision:THALASSA_RULES.revision,frame})),p=w.thalassa_input(b.length);new Uint8Array(w.memory.buffer,p,b.length).set(b);const q=w.thalassa_step(b.length),n=w.thalassa_output_len();const r=JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,q,n)));if(r.error)throw Error(r.error);return r;}
 function fire(){
  water=$('water').checked;const reach=THALASSA_RULES.basic.range*(water?1+THALASSA_RULES.water.rangePct/100:1),d=reach*.85/Math.sqrt(2);enemy=world.legal({x:origin.x+d,y:origin.y-d},10000);
  enemy={x:Math.round(enemy.x),y:Math.round(enemy.y)};
  w.thalassa_reset();const f={tick:0,owner:1,team:0,alive:true,pos:origin,hp:1000,max_hp:1000,shield:0,ap:100,stunned:false,ult_unlocked:true,units:[{id:5,team:1,pos:enemy,hp:1000,max_hp:1000,radius:10000,champion:true,tower:false,visible:true,targetable:true,farmable:false,stunned:false,velocity:{x:0,y:0}}],terrain:world.walls,profile:Array(13).fill(50),clone:null,echo:null,basic_hits:[],manual:true,order:water?{kind:'tide_pool',aim:origin}:null};
  for(let t=0;t<20;t++){f.tick=t;call(f);f.order=null;}f.tick=20;f.basic_hits=[5];const out=call(f);last=out.effects;mid=out.effects.find(e=>e.kind==='fx'&&e.data.name==='tls_surge'&&e.data.ticks===18)?.data.at||null;begin=performance.now();
  $('mapStatus').textContent=(water?'Water: 82,500 basic range; tsunami triggered.':'Land: 55,000 basic range; ordinary bolt, no tsunami.')+' Interval stays 2 seconds. Native '+THALASSA_RULES.revision+' output; stock drawings are reference actors.';
 }
 $('fire').onclick=fire;$('water').onchange=fire;
 const base=location.pathname.startsWith('/editor/')?'../docs/round-165-art-review/':'round-165-review-assets/';$('sheet').src=base+'effects-comparison.png';
 const opts=['tsunami_l96_d0','tsunami_l96_d4','tsunami_l96_d8','tsunami_l96_d12','tsunami_l96_d16','rhythm1','rhythm2','rhythm3','rhythm4','rhythm5','ambush','dismiss_aura','carrier_empowered','wright_warn1_t60','wright_warn2_t96','wright_warn3_t132','wright_bomb_t18'];$('effect').innerHTML=opts.map(t=>`<option>${t}</option>`).join('');
 function draw(now){
  const inspect=$('arena').getContext('2d'),tag=$('effect').value,total=art.manifest.frames[tag].frames.reduce((s,f)=>s+f.duration*60,0),age=now*.06%(total+45);inspect.fillStyle='#203747';inspect.fillRect(0,0,560,260);inspect.fillStyle='#bec9b0';inspect.fillRect(560,0,560,260);
  if(age<total||art.manifest.frames[tag].frames.length===1)for(const[x,s]of[[100,1],[350,3],[660,1],[910,3]]){inspect.save();inspect.translate(x,130);if($('facing').value==='Left')inspect.scale(-1,1);art.effect(inspect,tag,age,0,0,s);inspect.restore();}
  const scale=.003,P=p=>({x:560+(p.x-origin.x)*scale,y:380+(p.y-origin.y)*scale}),W=v=>v*scale;ctx.fillStyle='#17313c';ctx.fillRect(0,0,1120,600);world.drawMap(ctx,P,W);const q=P(origin),e=P(enemy),t=(now-begin)*.06;
  const range=THALASSA_RULES.basic.range*(water?1+THALASSA_RULES.water.rangePct/100:1);ctx.strokeStyle=water?'#8fe7df':'#e8cf92';ctx.beginPath();ctx.arc(q.x,q.y,W(range),0,Math.PI*2);ctx.stroke();
  if(water)body.effect(ctx,'pool_p0',0,q.x,q.y,W(950));body.body(ctx,t<22?'attack':'idle',t<22?t:now*.06,q.x,q.y,W(950),false,false,t>=22);
  const u={id:5,pos:enemy,sprite:'champ-knight',hp:1000,max_hp:1000,kind:'champion',champion:true,team:1,radius:10000,moving:false,left:true,attackUntil:0};world.drawUnit(ctx,u,0,P,W);
  if(mid&&t>=0&&t<18)art.wave(ctx,origin,mid,t,P,W(950));world.drawMap(ctx,P,W,true);ctx.fillStyle='#f6efdf';ctx.font='13px system-ui';ctx.fillText('Verified game map · 3× source art · native wave event, modeled stock reference',12,585);
  requestAnimationFrame(draw);
 }
 window.round165Review={art,body,world,fire,get effects(){return last;},get water(){return water;}};fire();requestAnimationFrame(draw);
})().catch(e=>{document.getElementById('status').textContent=e.message;console.error(e);});
