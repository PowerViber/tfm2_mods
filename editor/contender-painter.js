/* Packed frames only; effect expiry never holds the last frame. */
window.ContenderPainter=class {
 constructor(){this.data=CONTENDER_PRESENTATION;this.loaded=false;this.ready=this.load();}
 async load(){this.images={};for(const k of ['body','effects']){const im=new Image();im.src=this.data[k].file.replaceAll('#','%23');await im.decode();this.images[k]=im;}this.loaded=true;return this;}
 duration(kind,tag){return this.data[kind].anims[tag]?.frames.reduce((s,f)=>s+f.duration*60,0)||0;}
 draw(ctx,kind,tag,at,elapsed,P,W,loop=false,opacity=1){
  const a=this.data[kind].anims[tag];if(!a||!this.loaded)return false;
  const total=this.duration(kind,tag);if(!loop&&elapsed>=total)return false;
  let t=loop?elapsed%total:Math.max(0,elapsed),i=0;
  while(i<a.frames.length-1&&t>=a.frames[i].duration*60)t-=a.frames[i++].duration*60;
  const r=a.frames[i].data,p=P(at),s=W(1000);ctx.save();ctx.globalAlpha=opacity;ctx.imageSmoothingEnabled=false;
  ctx.drawImage(this.images[kind],r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);ctx.restore();return true;
 }
 body(ctx,u,tick,P,W){const pose=u.hp<=0?'dead':tick<u.attackUntil?'attack':u.moving?'run':'idle';
  const elapsed=u.hp<=0?Math.min(this.duration('body','dead')-.01,tick-(u.deadAt||tick)):pose==='attack'?tick-u.attackStart:tick;
  return this.draw(ctx,'body',pose+(u.left?'_l':''),u.pos,elapsed,P,W,pose==='idle'||pose==='run');}
};
