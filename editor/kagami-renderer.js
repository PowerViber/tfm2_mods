/* Actual packaged atlas frames, shared clocks and world anchors. */
window.KagamiPainter=class {
 constructor(){this.ready=Promise.all(['body','effects'].map(async n=>{const im=new Image();im.src='kagami-review-assets/'+n+'.png';await im.decode();this[n+"Image"]=im;}));}
 sample(pose,tick,left=false){
  const A=KAGAMI_ART,tag=pose+(left?'_l':''),an=A.body_anims[tag]||A.body_anims.idle,ts=an.frames.map(f=>Math.round(f.duration*60)),total=ts.reduce((a,b)=>a+b,0);let t=['idle','run'].includes(pose)?tick%total:Math.min(total-1,tick),i=0;
  while(i<ts.length-1&&t>=ts[i])t-=ts[i++];return {index:i,data:an.frames[i].data,duration:total,count:ts.length};
 }
 body(ctx,u,tick,P,W,pose){
  pose=pose||(u.hp<=0?'dead':tick<(u.poseUntil||0)?u.pose:u.moving?'run':'idle');if(pose==='heal')pose='skill2';if(pose==='step')pose='skill1';
  const elapsed=tick<(u.poseUntil||0)?Math.max(0,tick-(u.poseStart||u.poseUntil-18)):tick,sample=this.sample(pose,elapsed,u.left),r=sample.data,p=P(u.pos),s=W(1000);
  ctx.imageSmoothingEnabled=false;
  if(u.reflection){const an=KAGAMI_ART.effects['clone_'+pose+'_'+sample.index+(u.left?'_l':'')]||KAGAMI_ART.effects.clone_idle_0;const r=an.frames[0].data;ctx.drawImage(this.effectsImage,r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);}
  else ctx.drawImage(this.bodyImage,r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);
 }
 fx(ctx,name,at,P,W,alpha=1){const an=KAGAMI_ART.effects[name.replace(/^kg_/,'').replace('sealed','seal')];if(!an)return;const r=an.frames[0].data,p=P(at),s=W(1000);ctx.save();ctx.globalAlpha=alpha;ctx.imageSmoothingEnabled=false;ctx.drawImage(this.effectsImage,r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);ctx.restore();}
};
