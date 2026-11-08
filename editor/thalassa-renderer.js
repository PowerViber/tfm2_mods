// Presentation-only atlas selection shared by the Lab and art inspection page.
window.ThalassaPainter = class {
  constructor() {
    this.manifest=window.THALASSA_ART;
    const direct=location.pathname.startsWith('/editor/');
    const load=src=>new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>resolve(im);im.onerror=reject;im.src=src;});
    this.ready=Promise.all([
      load(direct?'../mods/tfm2_custom/champions/tfm2_custom_thalassa%23sheet.png':'thalassa-body.png'),
      load(direct?'../mods/tfm2_custom/vfx/thalassa%23sheet.png':'thalassa-vfx.png')
    ]).then(([body,fx])=>{this.bodyImage=body;this.fxImage=fx;this.loaded=true;});
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
    const i=this.frame(spec.frames,elapsed,loop);this.blit(ctx,this.fxImage,spec.frames[i].data,x,y,scale);return true;
  }
};
