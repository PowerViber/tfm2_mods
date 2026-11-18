/* Packed frames only; effect expiry never holds the last frame. */
window.ContenderPainter=class {
 constructor(){this.data=CONTENDER_PRESENTATION;this.loaded=false;this.ready=this.load();}
 async load(){this.images={};for(const k of ['body','effects','merge'].filter(k=>this.data[k])){const im=new Image();im.src=this.data[k].file.replaceAll('#','%23');await im.decode();this.images[k]=im;}this.loaded=true;return this;}
 duration(kind,tag){return this.data[kind].anims[tag]?.frames.reduce((s,f)=>s+f.duration*60,0)||0;}
 draw(ctx,kind,tag,at,elapsed,P,W,loop=false,opacity=1){
  const a=this.data[kind].anims[tag];if(!a||!this.loaded)return false;
  const total=this.duration(kind,tag);if(!loop&&elapsed>=total)return false;
  let t=loop?elapsed%total:Math.max(0,elapsed),i=0;
  while(i<a.frames.length-1&&t>=a.frames[i].duration*60)t-=a.frames[i++].duration*60;
  const r=a.frames[i].data,p=P(at),s=W(1000);ctx.save();ctx.globalAlpha=opacity;ctx.imageSmoothingEnabled=false;
  ctx.drawImage(this.images[kind],r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);ctx.restore();return true;
 }
 observeMerge(u,tick,cast){
  this.mergeClocks??=new WeakMap();let clock=this.mergeClocks.get(u);if(!clock){clock={pose:'',start:tick};this.mergeClocks.set(u,clock);}
  const pose=tick<(u.stunUntil||0)||tick<(u.hitUntil||0)?'hit':tick<u.attackUntil?'attack':cast&&tick<cast.end?cast.pose:u.moving?'run':'idle';
  if(clock.pose!==pose){clock.pose=pose;clock.start=tick;}
  if(pose==='attack')clock.start=u.attackStart;else if(pose==='projection')clock.start=cast.start;
  return clock;
 }
 body(ctx,u,tick,P,W,merged=false,cast=null){const group=merged&&u.hp>0?'merge':'body',clock=group==='merge'?this.observeMerge(u,tick,cast):null;
  const pose=clock?clock.pose:u.hp<=0?'dead':tick<u.attackUntil?'attack':u.moving?'run':'idle';
  const elapsed=clock?tick-clock.start:u.hp<=0?Math.min(this.duration('body','dead')-.01,tick-(u.deadAt||tick)):pose==='attack'?tick-u.attackStart:tick;
  const at=group==='merge'&&pose==='hit'?Math.min(elapsed,this.duration(group,pose)-.001):elapsed;
  return this.draw(ctx,group,pose+(u.left?'_l':''),u.pos,at,P,W,pose==='idle'||pose==='run');}
};
