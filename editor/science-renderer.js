/* Shared renderer for the practice arena and artwork inspector. All actor and
 * skill layers come from transparent runtime atlases, never portrait cards. */
(function(root){
'use strict';
const colors=['#63bfff','#ffc96b','#63e1bd'],tiers=[0,0,0,1,1,2,2,3];
class Renderer{
 constructor(canvas){this.canvas=canvas;this.ctx=canvas.getContext('2d');this.images=new Map();this.pending=new Map();this.assets={};this.missing=new Set();this.ready=false;this.error=null;}
 async image(src){if(this.images.has(src))return this.images.get(src);if(this.pending.has(src))return this.pending.get(src);
  const p=new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>{this.images.set(src,im);resolve(im);};im.onerror=()=>reject(Error('Artwork unavailable: '+src));im.src=src;});this.pending.set(src,p);try{return await p;}finally{this.pending.delete(src);}
 }
 async load(){try{
  const json=async src=>{const r=await fetch(src);if(!r.ok)throw Error('Artwork unavailable: '+src);return r.json();};
  const [manifest,body,crystal,misc]=await Promise.all([json('/science-art-preview.json'),json('/science-body.fanim'),json('/science-runtime/science_crystal_body.fanim'),json('/science-runtime/science_vfx.fanim')]);
  this.assets=manifest.assets;
  for(const [tag,a]of Object.entries(body.anims))this.assets['body_'+tag]={frames:a.frames,src:'/science-body.png'};
  for(const [tag,a]of Object.entries(crystal.anims))this.assets[tag]={frames:a.frames,src:'/science-runtime/science_crystal_body.png'};
  for(const [tag,a]of Object.entries(misc.anims))if(!this.assets[tag])this.assets[tag]={frames:a.frames,src:'/science-runtime/science_vfx.png'};
  await Promise.all(['/science-body.png','/science-runtime/science_crystal_body.png','/science-runtime/science_vfx.png'].map(src=>this.image(src)));this.ready=true;return this;
 }catch(e){this.error=e.message;throw e;}}
 src(asset){return asset.src||'/science-runtime/'+asset.sheet+'.png';}
 tag(name,rank){return this.assets[name]?name:this.assets[name+'_t'+tiers[rank]]?name+'_t'+tiers[rank]:null;}
 async prepareSkills(skills,rank,position=10){if(!this.ready)throw Error('Artwork is still loading');const names=new Set(['body_idle','body_run','body_dead',rank===7?'top'+position:'rank'+rank]);
  for(const s of skills){const skill=ScienceData.skills[s],science=skill.science,tier=tiers[rank];names.add('gearback'+science+'_r'+rank+(rank===7?'_p'+Math.min(position,4):''));names.add('outfit'+science+'_r'+rank);
   for(let p=0;p<3;p++)names.add('pose'+science+'_r'+rank+'_p'+p);
   for(const name of ['skill_'+skill.id+'_t'+tier,'impact_'+skill.id+'_t'+tier,'field'+s+'_t'+tier,'packet_'+skill.id+'_t'+tier+'_a0'])if(this.assets[name])names.add(name);
   names.add('complete_t'+tier+'_p'+(rank===7?Math.min(position,4):4)+'_s'+science);
   names.add('accident'+science);
  }
  names.add('complete_unified_t'+tiers[rank]+'_p'+(rank===7?Math.min(position,4):4));
  for(const k of [26,42,50,69])names.add('mark'+k+'_t'+tiers[rank]);
  await Promise.all([...new Set([...names].map(n=>this.assets[n]).filter(Boolean).map(a=>this.src(a)))].map(src=>this.image(src)));
 }
 blit(name,ticks,x,y,scale=2,flip=false,rank=6,loop=true){const tag=this.tag(name,rank),a=tag&&this.assets[tag];if(!a){this.missing.add(name);return 0;}
  const src=this.src(a),im=this.images.get(src);if(!im){this.image(src).catch(e=>{this.error=e.message;});return 0;}
  const total=a.frames.reduce((n,f)=>n+f.duration*60,0);let at=loop?(ticks%total+total)%total:Math.min(Math.max(0,ticks),total-.0001),f=a.frames[0],index=0;
  for(let i=0;i<a.frames.length;i++){f=a.frames[i];index=i;if(at<f.duration*60)break;at-=f.duration*60;}
  const r=f.data,c=this.ctx;c.save();c.translate(Math.round(x),Math.round(y));if(flip)c.scale(-1,1);c.imageSmoothingEnabled=false;c.drawImage(im,r.x,r.y,r.w,r.h,-r.w*scale/2,-r.h*scale/2,r.w*scale,r.h*scale);c.restore();return index+1;
 }
 background(){const c=this.ctx,cv=this.canvas;c.imageSmoothingEnabled=false;c.fillStyle='#0b1625';c.fillRect(0,0,cv.width,cv.height);c.strokeStyle='#1e3046';c.lineWidth=1;for(let x=0;x<cv.width;x+=40){c.beginPath();c.moveTo(x,0);c.lineTo(x,cv.height);c.stroke();}for(let y=0;y<cv.height;y+=40){c.beginPath();c.moveTo(0,y);c.lineTo(cv.width,y);c.stroke();}}
 actor(s,x,y,scale=2){const m=s.hero,science=s.persona,rank=s.rank,t=s.tick,flip=m.face<0;
  if(m.hp>0)this.blit('gearback'+science+'_r'+rank+(rank===7?'_p'+Math.min(s.position,4):''),t,x,y,scale,flip,rank);
  const pose=s.releasePose&&s.releasePose.science===science&&t-s.releasePose.tick<24?(t-s.releasePose.tick<12?1:2):s.plan&&ScienceData.skills[s.plan.stages[s.plan.index].skill].science===science?0:null;
  this.blit('body_'+(m.hp<=0?'dead':m.dest?'run':'idle'),t,x,y,scale,flip,rank);
  if(m.hp>0){const transform=s.transformation&&s.transformation.until>t?s.transformation:null;this.blit(transform?'transform'+transform.from+'_'+transform.to+'_r'+rank:pose===null?'outfit'+science+'_r'+rank:'pose'+science+'_r'+rank+'_p'+pose,transform?t-transform.born:pose===null?t:t-(s.releasePose?.tick||0),x,y,scale,flip,rank);}
  if(m.hp>0)this.blit(rank===7?'top'+s.position:'rank'+rank,t,x,y,scale,flip,rank);
 }
 arena(s,selected){const c=this.ctx,cv=this.canvas,UPX=ScienceCombat.UPX,z=cv.width/(ScienceCombat.WIDTH/UPX),sx=x=>x/UPX*z,sy=y=>y/UPX*z;
  this.background();const m=s.hero,skill=ScienceData.skills[selected],target=s.dummies.find(d=>d.id===s.locked&&d.hp>0&&d.visible),aim=target||s.aim;
  c.save();c.setLineDash([4,5]);c.strokeStyle=colors[skill.science]+'66';c.beginPath();c.arc(sx(m.x),sy(m.y),skill.range/UPX*z,0,Math.PI*2);c.stroke();c.restore();
  let placement=aim;if(skill.placement==='self'||skill.placement==='anchor')placement=m;else if(skill.placement==='midpoint')placement={x:(m.x+aim.x)/2,y:(m.y+aim.y)/2};else if(skill.placement==='forward'){const [dx,dy]=ScienceCombat.unit(aim.x-m.x,aim.y-m.y),k=selected===57?2:1;placement={x:m.x+dx*k,y:m.y+dy*k};}
  if(!ScienceCombat.PACKET_SKILLS.includes(selected)&&['Field','Protect','Setup','Observe'].includes(skill.role)){c.save();c.setLineDash([3,4]);c.strokeStyle=colors[skill.science]+'88';c.beginPath();c.arc(sx(placement.x),sy(placement.y),([5,12,13,18,20,45,57,58,65,66].includes(selected)?30000:8000)/UPX*z,0,Math.PI*2);c.stroke();c.restore();}
  for(const a of s.anchors)this.blit('field1',s.tick-a.born,sx(a.x),sy(a.y),z,false,s.rank);
  for(const f of s.fields){c.strokeStyle=colors[ScienceData.skills[f.kind].science]+'77';c.beginPath();c.arc(sx(f.x),sy(f.y),(f.kind===13?18000:30000)/UPX*z,0,Math.PI*2);c.stroke();this.blit('field'+f.kind,s.tick-f.born,sx(f.x),sy(f.y),z,false,s.rank);if(f.kind===57)this.blit('crystal_body',s.tick-f.born,sx(f.x),sy(f.y),z,false,s.rank);}
  for(const d of s.dummies){const x=sx(d.x),y=sy(d.y);c.save();c.globalAlpha=d.visible?1:.25;c.translate(Math.round(x),Math.round(y));c.scale(z,z);if(!d.hp)c.rotate(.9);c.fillStyle='#6b4a2b';c.fillRect(-1,-2,2,14);c.fillStyle='#4a3420';c.fillRect(-5,11,10,2);c.fillStyle=d.ccUntil>s.tick?'#dab7fa':'#d8b26a';c.fillRect(-6,-10,12,12);c.fillStyle='#c23b3b';c.beginPath();c.arc(0,-4,3.5,0,Math.PI*2);c.fill();c.fillStyle='#f4f1e6';c.beginPath();c.arc(0,-4,2,0,Math.PI*2);c.fill();c.restore();c.fillStyle='#172433';c.fillRect(x-20,y-37,40,4);c.fillStyle='#e5a968';c.fillRect(x-20,y-37,40*d.hp/d.maxhp,4);c.fillStyle='#9db2c4';c.font='10px monospace';c.fillText('#'+d.id+' '+d.hp,x-22,y+39);
   if(d.id===s.locked){c.strokeStyle='#ffda91';c.lineWidth=2;c.beginPath();c.ellipse(x,y+25,18*z,8*z,0,0,Math.PI*2);c.stroke();}for(const [tag,b]of Object.entries(d.buffs)){c.fillStyle=tag==='acid'?'#7de899':'#a89bea';c.fillText(tag,x-20,y-45-Object.keys(d.buffs).indexOf(tag)*11);}
  }
  // Prepared calculations stay visible: force vector, forecast, sample or isotope label.
  if(s.theory.vector.some(Boolean)&&s.theory.origin)this.blit('mark26',s.tick,sx(s.theory.origin.x),sy(s.theory.origin.y),z,false,s.rank);
  if(s.theory.aim)this.blit('mark42',s.tick,sx(s.theory.aim.x),sy(s.theory.aim.y),z,false,s.rank);
  const marked=(s.theory.sample||s.theory.label)&&s.dummies.find(d=>d.id===s.theory.target&&d.hp>0);if(marked)this.blit(s.theory.label?'mark69':'mark50',s.tick,sx(marked.x),sy(marked.y),z,false,s.rank);
  const podium=s.rank===7?Math.min(s.position,4):4;
  for(const v of s.visuals.filter(v=>v.tag==='complete'||v.tag==='complete_unified'||v.tag.startsWith('skill_'))){const at=v.follow?m:v,tag=v.tag==='complete'?'complete_t'+tiers[s.rank]+'_p'+podium+'_s'+v.science:v.tag==='complete_unified'?'complete_unified_t'+tiers[s.rank]+'_p'+podium:v.tag;this.blit(tag,s.tick-v.born,sx(at.x),sy(at.y),z,m.face<0,s.rank,false);}
  this.actor(s,sx(m.x),sy(m.y),z);
  if(s.theory.vector.some(Boolean)){const [vx,vy]=s.theory.vector;c.strokeStyle='#ffc96b99';c.lineWidth=2;c.beginPath();c.moveTo(sx(m.x),sy(m.y));c.lineTo(sx(m.x+vx*3),sy(m.y+vy*3));c.stroke();}
  for(const q of [...s.packets,...s.deliveries]){if(q.launch||q.launched===false)continue;const v=q.artMotion||[q.dx||0,q.dy||0],heading=(Math.round(Math.atan2(v[1],v[0])*4/Math.PI)+8)%8,tag='packet_'+ScienceData.skills[q.skill].id+'_t'+tiers[s.rank]+'_a'+heading;this.blit(tag,s.tick-q.born,sx(q.x),sy(q.y),z,false,s.rank);}
  for(const q of s.incoming){c.fillStyle='#ff9a85';c.beginPath();c.arc(sx(q.x),sy(q.y),3*z,0,Math.PI*2);c.fill();}
  for(const v of s.visuals.filter(v=>v.tag!=='complete'&&v.tag!=='complete_unified'&&!v.tag.startsWith('skill_'))){const at=v.follow?m:v;this.blit(v.tag,s.tick-v.born,sx(at.x),sy(at.y),z,false,s.rank,false);}
  const shield=m.shields.reduce((n,s)=>n+s.amount,0);if(shield){c.strokeStyle='#94ede3';c.beginPath();c.ellipse(sx(m.x),sy(m.y),18*z,27*z,0,0,Math.PI*2);c.stroke();}
  c.fillStyle='#1c2b3d';c.fillRect(sx(m.x)-22*z,sy(m.y)-50*z,44*z,4*z);c.fillStyle='#83dab3';c.fillRect(sx(m.x)-22*z,sy(m.y)-50*z,44*z*m.hp/m.maxhp,4*z);
  this.blit('orb_'+s.persona+'_'+(s.plan?1:s.packets.length||s.deliveries.length?2:0)+'_'+(s.failureUntil>s.tick?3:0),s.tick,sx(m.x)-22.5*z,sy(m.y)-65*z,z,false,s.rank);
  this.blit('bar_'+s.charges.join('_'),s.tick,sx(m.x)+7.5*z,sy(m.y)-65*z,z,false,s.rank);
  c.strokeStyle=target?'#ffda91':colors[skill.science];c.lineWidth=1;const ax=sx(aim.x),ay=sy(aim.y);c.beginPath();c.moveTo(ax-8,ay);c.lineTo(ax+8,ay);c.moveTo(ax,ay-8);c.lineTo(ax,ay+8);c.stroke();
  c.fillStyle='#c5d6e2';c.font='12px monospace';c.fillText((s.paused?'PAUSED · ':'')+(target?'LOCKED #'+target.id:'CURSOR AIM')+' · '+(s.tick/60).toFixed(2)+'s',12,20);c.fillStyle='#9ab0c2';c.fillText('Material '+s.material+'/12 · Momentum '+s.momentum+'/100 · Shield '+shield,12,cv.height-12);
  const calculations=['cone','proper','blue','elastic','boundary','sample','acid','base','catalyst','label'].filter(k=>s.theory[k]),buffs=Object.keys(m.buffs);if(calculations.length||buffs.length){c.fillStyle=colors[s.persona];c.font='10px monospace';c.fillText([...calculations,...buffs].join(' · '),12,38);}
  if(this.error){c.fillStyle='#ff9d92';c.fillText(this.error,12,40);}
 }
}
root.ScienceRenderer={Renderer,tiers,colors};
})(typeof globalThis!=='undefined'?globalThis:this);
