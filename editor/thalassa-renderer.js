// Presentation-only atlas selection shared by the Lab and art inspection page.
// A display callback is not a simulation tick (144 Hz monitors must still run at 60 Hz).
window.ThalassaClock = class {
  constructor() { this.reset(); }
  reset() { this.last = null; this.accumulator = 0; }
  advance(now, running, step) {
    if (this.last === null) { this.last = now; return; }
    const elapsed = Math.min(250, Math.max(0, now - this.last));
    this.last = now;
    if (!running) { this.accumulator = 0; return; }
    this.accumulator += elapsed;
    while (this.accumulator + 1e-7 >= 1000 / 60) {
      this.accumulator -= 1000 / 60; step();
    }
  }
};
window.ThalassaPainter = class {
  constructor() {
    this.manifest=window.THALASSA_ART;
    const direct=location.pathname.startsWith('/editor/');
    const load=src=>new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>resolve(im);im.onerror=reject;im.src=src;});
    this.ready=Promise.all([
      load(direct?'../mods/tfm2_custom/champions/tfm2_custom_thalassa%23sheet.png':'thalassa-body.png'),
      load(direct?'../mods/tfm2_custom/vfx/thalassa%23sheet.png':'thalassa-vfx.png'),
      load(direct?'../mods/tfm2_custom/vfx/thalassa_links%23sheet.png':'thalassa-links.png')
    ]).then(([body,fx,links])=>{this.bodyImage=body;this.fxImage=fx;this.linkImage=links;this.loaded=true;});
  }
  frame(frames,elapsed,loop=true) {
    const lengths=frames.map(f=>Math.round(f.duration*60000)),total=lengths.reduce((a,b)=>a+b,0);
    let t=loop?Math.floor(elapsed*1000)%total:Math.min(Math.floor(elapsed*1000),total-1);
    for(let i=0;i<frames.length;i++){if(t<lengths[i])return i;t-=lengths[i];}
    return frames.length-1;
  }
  blit(ctx,image,data,x,y,scale) {
    if(!image)return;
    ctx.imageSmoothingEnabled=false;
    ctx.drawImage(image,data.x,data.y,data.w,data.h,x-data.w*scale/2,y-data.h*scale/2,data.w*scale,data.h*scale);
  }
  body(ctx,pose,elapsed,x,y,scale=1,left=false,twin=false,loop=true,forceFrame=null) {
    this.lastBody={pose,left,twin};
    const key=pose+(left?'_l':'');
    const frames=this.manifest.body[key]||this.manifest.body.idle;
    const i=forceFrame===null?this.frame(frames,elapsed,loop):Math.min(forceFrame,frames.length-1);
    if(twin){const tag='twin_'+pose+'_'+i+(left?'_l':'');const f=this.manifest.effects[tag];if(f)this.blit(ctx,this.fxImage,f.frames[0].data,x,y,scale);}
    else this.blit(ctx,this.bodyImage,frames[i].data,x,y,scale);
    return i;
  }
  effect(ctx,tag,elapsed,x,y,scale=1,loop=false) {
    const spec=this.manifest.effects[tag];if(!spec)return false;
    const i=this.frame(spec.frames,elapsed,loop);this.blit(ctx,spec.asset?this.linkImage:this.fxImage,spec.frames[i].data,x,y,scale);return true;
  }
  linkSegments(from,to) {
    const cfg=this.manifest.link, dx=to.x-from.x,dy=to.y-from.y, distance=Math.hypot(dx,dy);
    if(distance<1)return [];
    const angle=Math.atan2(dy,dx),turn=angle*cfg.directions/(2*Math.PI),dir=(((Math.sign(turn)*Math.floor(Math.abs(turn)+.5))%cfg.directions)+cfg.directions)%cfg.directions;
    let remaining=distance/cfg.units_per_pixel, consumed=0;const out=[];
    for(let i=0;i<cfg.max_segments&&remaining>0;i++){
      const sizes=cfg.lengths_pixels;
      const length=i===cfg.max_segments-1?sizes.find(v=>v>=remaining)||sizes.at(-1):[...sizes].reverse().find(v=>v<=remaining)||sizes[0];
      const span=Math.min(remaining,length),t=(consumed+span/2)*cfg.units_per_pixel/distance;
      out.push({tag:`tide_link_${length}_${dir}`,at:{x:Math.round(from.x+dx*t),y:Math.round(from.y+dy*t)}});
      remaining-=span;consumed+=span;
    }
    return out;
  }
};
