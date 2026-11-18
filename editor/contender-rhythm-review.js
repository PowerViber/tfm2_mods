(async()=>{'use strict';
const $=id=>document.getElementById(id),data=CONTENDER_RHYTHM_ART,painter=await new ContenderPainter().ready,image=new Image();image.src=data.file.replaceAll('#','%23');await image.decode();
let tick=0,last=null,playing=true,breakAt=0,auraAt=0;
function art(c,tag,p,t,scale){const a=data.anims[tag];if(!a)return;const total=a.frames.reduce((n,f)=>n+f.duration*60,0);if(t<0||t>=total)return;let i=0;while(i<a.frames.length-1&&t>=a.frames[i].duration*60)t-=a.frames[i++].duration*60;const r=a.frames[i].data;c.drawImage(image,r.x,r.y,r.w,r.h,p.x-r.w*scale/2,p.y-r.h*scale/2,r.w*scale,r.h*scale);}
function body(c,p,scale){painter.draw(c,$('merged').checked?'merge':'body','idle'+($('left').checked?'_l':''),p,tick,p=>p,n=>n*scale/1000,true);}
function draw(){
 const c=$('effects').getContext('2d');c.imageSmoothingEnabled=false;c.clearRect(0,0,1040,540);
 for(let row=0;row<2;row++)for(let col=0;col<2;col++){const scale=col?3:1,p={x:col*520+(col?170:220),y:row*270+130};c.fillStyle=row?'#bac5a6':'#29454a';c.fillRect(col*520,row*270,520,270);art(c,'full_rhythm_t480',p,tick-auraAt,scale);body(c,p,scale);art(c,'break_t18',{x:p.x+75*scale,y:p.y},tick-breakAt,scale);}
 $('phase').textContent=`Aura ${Math.max(0,(480-(tick-auraAt))/60).toFixed(1)} s remaining · Break ${tick-breakAt<18?'playing':'finished'} · ${playing?'playing':'paused'}`;
 const h=$('hud').getContext('2d'),stacks=+$('stacks').value,bank=+$('bank').value;h.imageSmoothingEnabled=false;h.clearRect(0,0,1040,460);
 for(let row=0;row<2;row++)for(let col=0;col<2;col++){const scale=col?3:1,p={x:col*520+260,y:row*230+135};h.fillStyle=row?'#bac5a6':'#29454a';h.fillRect(col*520,row*230,520,230);body(h,p,scale);art(h,`enemy_${stacks}`,p,0,scale);art(h,`bank_${bank}`,p,0,scale);}
 $('values').textContent=`Stacks ${stacks}/20 · bank ${bank}/40${bank===40?' · ready':''}`;
}
$('break').onclick=()=>{breakAt=tick;draw();};$('aura').onclick=()=>{auraAt=tick;draw();};$('pause').onclick=()=>{playing=!playing;last=null;$('pause').textContent=playing?'Pause':'Play';};for(const id of['merged','left','stacks','bank'])$(id).oninput=draw;
function loop(now){if(last!==null&&playing)tick+=Math.min(100,now-last)*.06;last=now;draw();requestAnimationFrame(loop);}requestAnimationFrame(loop);
window.contenderRhythmReview={painter,data,draw,art,get tick(){return tick;},seek:t=>{tick=t;playing=false;draw();},restart:()=>{tick=breakAt=auraAt=0;draw();}};
})().catch(e=>{document.body.append('Could not load the review: '+e.message);throw e;});
