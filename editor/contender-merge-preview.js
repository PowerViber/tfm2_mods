/* Opt-in concept preview only. Uses whole-body poses, never a detached idle clone. */
window.ContenderMergePreview=class {
 constructor(){this.data=CONTENDER_MERGE_PREVIEW;this.ready=this.load();}
 async load(){this.image=new Image();this.image.src=this.data.file;await this.image.decode();return this;}
 draw(ctx,u,tick,P,W,pose){
  if(u.hp<=0)return false;
  pose??=tick<u.attackUntil?'straight':u.moving?'run':'idle';
  const r=this.data.frames[pose+(u.left?'_l':'')],p=P(u.pos),s=W(1000);if(!r)return false;
  ctx.save();ctx.imageSmoothingEnabled=false;ctx.drawImage(this.image,r.x,r.y,r.w,r.h,p.x-r.w*s/2,p.y-r.h*s/2,r.w*s,r.h*s);ctx.restore();return true;
 }
};
