// Atlas selection only. No combat state, clocks, randomness or movement live here.
window.WrightPainter=class {
  constructor(){
    this.manifest=window.WRIGHT_ART;
    const direct=location.pathname.startsWith('/editor/'),review=this.manifest.review_only;
    const load=src=>new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>resolve(im);im.onerror=()=>reject(new Error('Cannot load '+src));im.src=src;});
    const body=direct?(review?'../docs/wright-art-review/candidate-body.png':'../mods/tfm2_custom/champions/tfm2_custom_wright%23sheet.png'):'wright-body.png';
    const fx=direct?(review?'../docs/wright-art-review/candidate-fx.png':'../mods/tfm2_custom/vfx/wright%23sheet.png'):'wright-fx.png';
    this.ready=Promise.all([load(body),load(fx)]).then(([body,fx])=>{this.bodyImage=body;this.fxImage=fx;this.loaded=true;});
  }
  frame(frames,tick,loop=false){
    const ds=frames.map(f=>Math.round(f.duration*60000)),total=ds.reduce((a,b)=>a+b,0);
    let t=loop?Math.floor(tick*1000)%total:Math.min(Math.floor(tick*1000),total-1);
    for(let i=0;i<ds.length;i++){if(t<ds[i])return i;t-=ds[i];}return ds.length-1;
  }
  blit(ctx,im,d,x,y,s){ctx.imageSmoothingEnabled=false;ctx.drawImage(im,d.x,d.y,d.w,d.h,x-d.w*s/2,y-d.h*s/2,d.w*s,d.h*s);}
  body(ctx,pose,tick,x,y,s=1,left=false,loop=true,index=null){
    const frames=this.manifest.body[pose+(left?'_l':'')]||this.manifest.body.idle;
    const f=index===null?this.frame(frames,tick,loop):Math.min(index,frames.length-1);this.blit(ctx,this.bodyImage,frames[f].data,x,y,s);this.lastBody={pose,left,frame:f};return f;
  }
  effect(ctx,tag,tick,x,y,s=1,loop=false){const spec=this.manifest.effects[tag];if(!spec)return false;const i=this.frame(spec.frames,tick,loop);this.blit(ctx,this.fxImage,spec.frames[i].data,x,y,s);return true;}
};
