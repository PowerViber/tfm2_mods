// Review-only body; native remains the Contender placeholder until approval.
window.KickerPainter=class{
 constructor(){this.loaded=false;const base=location.pathname.startsWith('/editor/')?'../docs/kicker-art-review/':'kicker-review-assets/';this.ready=new Promise((ok,no)=>{this.image=new Image();this.image.onload=()=>{this.loaded=true;ok(this);};this.image.onerror=()=>no(Error('Kicker concept unavailable'));this.image.src=base+'concepts.png';});}
 draw(ctx,pose,x,y,scale=1,left=false){if(!this.loaded)return false;const r=KICKER_ART.frames[pose+(left?'_l':'')]||KICKER_ART.frames.idle;ctx.imageSmoothingEnabled=false;ctx.drawImage(this.image,r.x,r.y,r.w,r.h,x-r.w*scale/2,y-r.h*scale/2,r.w*scale,r.h*scale);return true;}
};
