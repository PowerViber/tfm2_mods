(async()=>{'use strict';
const $=id=>document.getElementById(id),painter=await new ContenderPainter().ready,c=$('preview').getContext('2d');
for(const tag of Object.keys(painter.data.body.anims).filter(t=>!t.endsWith('_l')))$('pose').add(new Option(tag,tag));
let elapsed=0,last=null,playing=true;
function draw(){
 const tag=$('pose').value+($('left').checked?'_l':''),frames=painter.data.body.anims[tag].frames,total=painter.duration('body',tag);
 $('frame').max=frames.length-1;
 let t=elapsed%total,i=0;while(i<frames.length-1&&t>=frames[i].duration*60)t-=frames[i++].duration*60;
 if(playing)$('frame').value=i;else{i=+$('frame').value;elapsed=frames.slice(0,i).reduce((n,f)=>n+f.duration*60,0);}
 const at=playing?elapsed%total:elapsed;
 c.clearRect(0,0,1000,580);c.font='14px sans-serif';
 for(let row=0;row<2;row++){
  c.fillStyle=row?'#bac5a6':'#29454a';c.fillRect(0,row*290,1000,290);c.fillStyle=row?'#152b35':'#f4e9d4';
  for(let form=0;form<2;form++){
   c.fillText(form?'Merged':'Normal',form*500+25,row*290+25);
   for(const scale of [1,3]){const p={x:form*500+(scale===1?120:330),y:row*290+155};painter.draw(c,form?'merge':'body',tag,p,at,p=>p,n=>n*scale/1000);}
  }
 }
 $('timing').textContent=`Frame ${i+1}/${frames.length} · ${total.toFixed(1)} ticks / ${(total/60).toFixed(2)} s`;
}
$('pose').onchange=()=>{elapsed=0;$('frame').value=0;draw();};$('left').onchange=draw;
$('play').onclick=()=>{playing=!playing;last=null;$('play').textContent=playing?'Pause':'Play';draw();};$('frame').oninput=()=>{playing=false;$('play').textContent='Play';draw();};
function loop(now){if(last!==null&&playing)elapsed+=Math.min(100,now-last)*.06;last=now;draw();requestAnimationFrame(loop);}requestAnimationFrame(loop);
window.contenderMergeReview={painter,draw,get playing(){return playing;}};
})().catch(e=>{document.body.append('Could not load Merge assets: '+e.message);throw e;});
