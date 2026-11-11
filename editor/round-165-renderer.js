// One supplemental atlas and canonical selectors; presentation only.
window.Round165Painter=class {
 constructor(){this.manifest=window.ROUND165_ART;this.loaded=false;const direct=location.pathname.startsWith('/editor/'),integrated=this.manifest.integrated;
  this.ready=new Promise((ok,no)=>{const im=new Image();im.onload=()=>{this.image=im;this.loaded=true;ok(this);};im.onerror=()=>no(Error('Round 165 artwork unavailable'));im.src=integrated?(direct?'../mods/tfm2_custom/vfx/round_165%23sheet.png':'/round-165-fx.png'):(direct?'../docs/round-165-art-review/supplement.png':'/round-165-review-assets/supplement.png');});
 }
 effect(ctx,tag,age,x,y,scale=1){const a=this.manifest.frames[tag];if(!this.loaded||!a)return false;let t=Math.max(0,Math.min(age,a.frames.reduce((s,f)=>s+f.duration*60,0)-1e-7)),f=a.frames[0];for(const e of a.frames){f=e;if(t<e.duration*60)break;t-=e.duration*60;}const d=f.data;ctx.imageSmoothingEnabled=false;ctx.drawImage(this.image,d.x,d.y,d.w,d.h,x-d.w*scale/2,y-d.h*scale/2,d.w*scale,d.h*scale);return true;}
 waveParts(from,mid){const cfg=this.manifest.wave,dx=(mid.x-from.x)*2,dy=(mid.y-from.y)*2,dist=Math.hypot(dx,dy);if(dist<1)return[];const angle=Math.atan2(dy,dx)*cfg.directions/(Math.PI*2),dir=((Math.sign(angle)*Math.floor(Math.abs(angle)+.5))%cfg.directions+cfg.directions)%cfg.directions;
  const n=Math.min(cfg.max_segments,Math.max(1,Math.ceil(dist/(cfg.lengths.at(-1)*cfg.units_per_pixel)))),span=dist/n/cfg.units_per_pixel,len=[...cfg.lengths].sort((a,b)=>Math.abs(a-span)-Math.abs(b-span)||a-b)[0];
  const round=v=>Math.sign(v)*Math.floor(Math.abs(v)+.5);return Array.from({length:n},(_,i)=>({tag:`tsunami_l${len}_d${dir}`,at:{x:round(from.x+dx*(i+.5)/n),y:round(from.y+dy*(i+.5)/n)}}));
 }
 wave(ctx,from,mid,age,P,scale){for(const p of this.waveParts(from,mid)){const q=P(p.at);this.effect(ctx,p.tag,age,q.x,q.y,scale);}}
 wright(ctx,name,ticks,age,x,y,scale){const tag=this.manifest.wright_aliases[name+':'+ticks];return tag?this.effect(ctx,tag,age,x,y,scale):false;}
};
