(async()=>{
 const painter=new KickerPainter(),world=await new GameWorld().ready,$=id=>document.getElementById(id);
 await Promise.all([painter.ready,world.loadSprites(['champ-knight','champ-monk'])]);
 const base=location.pathname.startsWith('/editor/')?'../docs/kicker-art-review/':'kicker-review-assets/';
 for(const[id,file]of Object.entries({beforeAfter:'before-after-v2.png',stock:'stock-comparison.png',poses:'body-comparison.png',effectsSheet:'effects-comparison.png',idleGif:'idle.gif',runGif:'run.gif',attackGif:'attack.gif',throwGif:'throw.gif'}))$(id).src=base+file;
 $('pose').innerHTML=Object.keys(KICKER_ART.poses).map(p=>'<option>'+p+'</option>').join('');
 const c=$('arena').getContext('2d'),fx=$('fxArena').getContext('2d'),origin=world.legal({x:480000,y:480000},10000),P=p=>({x:560+(p.x-origin.x)*.003,y:200+(p.y-origin.y)*.003}),W=n=>n*.003;
 let start=null;
 function draw(t){
  if(start===null)start=t;
  const elapsed=(t-start)*60/1000,pose=$('pose').value,left=$('left').checked;
  const forced=$('animate').checked?null:Number($('frame').value),sample=painter.sample(pose,elapsed,left,forced);
  $('frame').max=sample.count-1;if(forced===null)$('frame').value=sample.index;
  $('frameInfo').textContent=' '+(sample.index+1)+' / '+sample.count+' · '+(sample.duration/60).toFixed(2)+' s';
  c.fillStyle='#203a47';c.fillRect(0,0,1120,420);world.drawMap(c,P,W);
  const q=P(origin);painter.draw(c,pose,q.x,q.y,W(1000),left,elapsed,forced);
  for(const[i,name]of ['knight','monk'].entries())world.drawUnit(c,{pos:{x:origin.x+(i?50000:-50000),y:origin.y},sprite:'champ-'+name,kind:'champion',hp:1000,max_hp:1000,team:i,radius:10000,left:!i},elapsed,P,W);
  world.drawMap(c,P,W,true);
  const time=Math.floor(elapsed)%240,seq=$('fxSequence').value;
  for(const[col,bg]of [[0,'#203747'],[1,'#bec9b0']]){
   fx.fillStyle=bg;fx.fillRect(col*560,0,560,340);
   fx.fillStyle=col?'#203747':'#fff5dc';fx.font='14px system-ui';fx.fillText(seq+' · actual size and 3×',col*560+12,22);
   for(const[baseX,s]of [[col*560+80,1],[col*560+340,3]]){
    const y=150,ground=y+20*s;
    const show=(tag,x=baseX,yy=y,age=time)=>painter.fx(fx,tag,x,yy,s,age);
    let body='idle';
    if(seq==='landing'){body=time<60?'hop_high':'idle';if(time<24)show('hop',baseX,ground,time);if(time>=60&&time<84)show('drop_in',baseX,ground,time-60);}
    if(seq==='rock'){body=time<12?'throw':'idle';if(time>=6&&time<66)show('rock_e',baseX+(time-6)*s,y-6*s,0);if(time>=66&&time<78)show('rock_hit',baseX+60*s,y-6*s,time-66);}
    if(seq==='kick'){body=time<18?'kick':'idle';if(time>=12&&time<30)show('kick',baseX+22*s,y,time-12);if(time>=30&&time<48)show('wall_slam',baseX+55*s,y,time-30);}
    if(seq==='skyfall'){body=time<60?'dive':'idle';if(time<60)show('skyfall_se',baseX-12*s,y-8*s,0);if(time>=60&&time<84)show('drop_in',baseX,ground,time-60);}
    if(seq==='ult'){body=time<24?'ult':'idle';if(time<150){show('tag',baseX,ground);show('second_wind');}if(time>=150&&time<174)show('return',baseX,y,time-150);}
    if(seq==='status'){body='perch';show('roof',baseX,ground);show('mark',baseX,y-30*s);show('bruised'+Math.floor(time/40),baseX+25*s,y-25*s);}
    painter.draw(fx,body,baseX,y,s,left,time);
   }
  }
  requestAnimationFrame(draw);
 }
 $('pose').onchange=()=>{start=null;$('frame').value=0;};$('frame').oninput=()=>$('animate').checked=false;
 window.kickerReview={painter,world};requestAnimationFrame(draw);
})().catch(console.error);
