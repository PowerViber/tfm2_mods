// Complete approved body; proposed effects stay outside the mod until approved.
window.KickerPainter=class{
 constructor(){
  this.loaded=false;
  const editor=location.pathname.startsWith('/editor/');
  const base=editor?'../docs/kicker-art-review/':'kicker-review-assets/';
  const body=editor?'../mods/tfm2_custom/champions/tfm2_custom_kicker%23sheet.png':'kicker-body.png';
  const effects=KICKER_ART.effects_approved?(editor?'../mods/tfm2_custom/vfx/kicker%23sheet.png':'kicker-effects.png'):base+'effects.png';
  const load=src=>new Promise((ok,no)=>{const im=new Image();im.onload=()=>ok(im);im.onerror=()=>no(Error('Kicker art unavailable: '+src));im.src=src+'?revision='+KICKER_ART.revision;});
  this.ready=Promise.all([load(body),load(effects)]).then(([b,f])=>{this.image=b;this.effects=f;this.loaded=true;return this;});
 }
 sample(pose,elapsed=0,left=false,forced=null,effect=false){
  const tags=effect?KICKER_ART.effect_anims:KICKER_ART.body_anims;
  let tag=pose+(left&&!effect?'_l':'');
  const frames=(tags[tag]||tags[effect?'land':'idle']).frames;
  const duration=frames.reduce((n,f)=>n+f.duration*60,0);
  let t=Math.max(0,elapsed);
  const loop=effect||['idle','run','move','perch'].includes(pose);
  t=loop?t%duration:Math.min(t,duration-.0001);
  let index=frames.length-1;
  for(let i=0;i<frames.length;i++){if(t<frames[i].duration*60){index=i;break;}t-=frames[i].duration*60;}
  if(forced!==null)index=Math.max(0,Math.min(frames.length-1,forced));
  return{rect:frames[index].data,index,duration,count:frames.length};
 }
 draw(ctx,pose,x,y,scale=1,left=false,elapsed=0,forced=null){
  if(!this.loaded)return false;
  const {rect:r}=this.sample(pose,elapsed,left,forced);
  ctx.imageSmoothingEnabled=false;
  ctx.drawImage(this.image,r.x,r.y,r.w,r.h,x-r.w*scale/2,y-r.h*scale/2,r.w*scale,r.h*scale);
  return true;
 }
 fx(ctx,tag,x,y,scale=1,elapsed=0){
  if(!this.loaded)return false;
  const {rect:r}=this.sample(tag,elapsed,false,null,true);
  ctx.imageSmoothingEnabled=false;
  ctx.drawImage(this.effects,r.x,r.y,r.w,r.h,x-r.w*scale/2,y-r.h*scale/2,r.w*scale,r.h*scale);
  return true;
 }
};
