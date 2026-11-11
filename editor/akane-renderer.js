// Presentation only: no host writes, combat clocks, RNG or ability decisions.
window.AkanePainter=class {
  constructor(){this.loaded=false;this.images={};this.anims={};this.ready=this.load();}
  async load(){
    const base=location.pathname.startsWith('/editor/')?'../docs/akane-art-review/':'akane-review-assets/';this.base=base;
    const r=await fetch(base+'presentation.json');if(!r.ok)throw new Error('Akane artwork manifest unavailable');this.manifest=await r.json();
    await Promise.all(Object.entries(this.manifest.atlases).map(async([key,stem])=>{
      let file=base+stem+'.png';
      if(this.manifest.complete_body&&key==='body')file=location.pathname.startsWith('/editor/')?'../mods/tfm2_custom/champions/tfm2_custom_akane%23sheet.png':'/akane-body.png';
      if(this.manifest.effects_approved&&key==='effects')file=location.pathname.startsWith('/editor/')?'../mods/tfm2_custom/vfx/akane%23sheet.png':'/akane-effects.png';
      const im=new Image();await new Promise((ok,no)=>{im.onload=ok;im.onerror=()=>no(new Error('Missing Akane image: '+file));im.src=file;});this.images[key]=im;
      const meta=await fetch(base+stem+'.fanim');if(!meta.ok)throw new Error('Missing Akane frames: '+stem);this.anims[key]=(await meta.json()).anims;
    }));this.loaded=true;return this;
  }
  animation(ctx,atlas,name,age,x,y,scale,loop=false){
    const a=this.anims[atlas]?.[name];if(!a)return false;const ds=a.frames.map(f=>f.duration*60),total=ds.reduce((a,b)=>a+b,0);
    let t=Math.max(0,age),i=0;t=loop?t%total:Math.min(total-1e-7,t);while(i<ds.length-1&&t>=ds[i])t-=ds[i++];const d=a.frames[i].data;
    ctx.imageSmoothingEnabled=false;ctx.drawImage(this.images[atlas],d.x,d.y,d.w,d.h,x-d.w*scale/2,y-d.h*scale/2,d.w*scale,d.h*scale);return true;
  }
  body(ctx,pose,age,x,y,scale,left=false){
    if(!this.loaded)return false;const alias=this.manifest.pose_aliases||{};pose=alias[pose]||pose;
    if(!this.anims.body[pose])pose=pose==='dead'?'idle':pose.includes('dash')?'attack':'idle';
    return this.animation(ctx,'body',pose+(left?'_l':''),age,x,y,scale,['idle','run'].includes(pose));
  }
  effect(ctx,name,age,x,y,scale,loop=false){return this.animation(ctx,'effects',name.replace(/^akn_/,''),age,x,y,scale,loop);}
};
