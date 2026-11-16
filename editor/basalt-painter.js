// Review-only projection of generated pixels. No combat decisions or mechanics.
window.BasaltPainter=class {
  constructor(){this.meta=window.BASALT_ART;this.bodyMeta=window.BASALT_BODY;this.fxMeta=window.BASALT_FX;this.loaded=false;this.ready=Promise.all([this.image(this.bodyMeta?.body_file||this.meta.body_file),this.image(this.fxMeta?.effects_file||this.meta.effects_file)]).then(([body,fx])=>{this.body=body;this.effects=fx;this.loaded=true;return this;});}
  image(src){return new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>resolve(im);im.onerror=()=>reject(new Error('Missing Basalt proposal: '+src));im.src=src;});}
  bodyFrame(name,left=false,elapsed=0,loop=true){
    if(!this.bodyMeta)return this.meta.body_frames[name+(left?'_l':'')]||this.meta.body_frames.idle;
    const seq=(this.bodyMeta.body_anims[name+(left?'_l':'')]||this.bodyMeta.body_anims.idle).frames;
    const length=seq.reduce((n,f)=>n+f.duration*60,0);let time=loop?Math.max(0,elapsed)%length:Math.min(Math.max(0,elapsed),length-1e-6);
    for(const f of seq){time-=f.duration*60;if(time<0)return f.data;}return seq.at(-1).data;
  }
  pose(ctx,name,p,scale=1,left=false,elapsed=0,loop=true){if(!this.loaded)return;const r=this.bodyFrame(name,left,elapsed,loop);ctx.imageSmoothingEnabled=false;ctx.drawImage(this.body,r.x,r.y,r.w,r.h,p.x-r.w*scale/2,p.y-r.h*scale/2,r.w*scale,r.h*scale);}
  fx(ctx,name,p,scale=1,angle=0,alpha=1,elapsed=0){if(!this.loaded)return;let r=this.meta.effect_frames[name];if(this.fxMeta){const seq=this.fxMeta.animations[name.replace(/^bs_/,'')]?.frames;if(!seq)return;let time=Math.max(0,elapsed);r=seq.at(-1).data;for(const f of seq){time-=f.duration*60;if(time<0){r=f.data;break;}}}if(!r)return;ctx.save();ctx.imageSmoothingEnabled=false;ctx.globalAlpha=alpha;ctx.translate(p.x,p.y);ctx.rotate(angle);ctx.drawImage(this.effects,r.x,r.y,r.w,r.h,-r.w*scale/2,-r.h*scale/2,r.w*scale,r.h*scale);ctx.restore();}
  clapContains(origin,aim,p){const spec=this.meta.clap_geometry,dx=p.x-origin.x,dy=p.y-origin.y,r=Math.floor(Math.hypot(dx,dy)),ax=aim.x-origin.x,ay=aim.y-origin.y,n=Math.hypot(ax,ay)||1;if(r<spec.near||r>spec.far)return false;return(dx*ax+dy*ay)/(Math.hypot(dx,dy)*n)>=Math.cos(spec.cone_deg*Math.PI/360)-1e-12;}
  clapCone(ctx,origin,aim,worldScale,active=false){const spec=this.meta.clap_geometry,a=Math.atan2(aim.y-origin.y,aim.x-origin.x),half=spec.cone_deg*Math.PI/360,far=spec.far*worldScale,near=spec.near*worldScale;
    ctx.save();ctx.translate(origin.x,origin.y);ctx.fillStyle=active?'rgba(245,73,39,.32)':'rgba(241,125,42,.18)';ctx.strokeStyle=active?'#ffe26b':'#f8bf57';ctx.lineWidth=active?2.5:1.5;
    // Only shade the existing hit region; the outline's V begins at the champion.
    ctx.beginPath();ctx.arc(0,0,far,a-half,a+half);ctx.arc(0,0,near,a+half,a-half,true);ctx.closePath();ctx.fill();
    ctx.beginPath();ctx.moveTo(0,0);ctx.arc(0,0,far,a-half,a+half);ctx.closePath();ctx.stroke();
    ctx.strokeStyle='rgba(255,226,131,.55)';ctx.setLineDash([4,5]);ctx.beginPath();ctx.moveTo(0,0);ctx.lineTo(Math.cos(a)*far,Math.sin(a)*far);ctx.stroke();ctx.restore();
  }
  whip(ctx,a,b,scale=1){const dx=b.x-a.x,dy=b.y-a.y,d=Math.hypot(dx,dy),count=Math.max(1,Math.min(40,Math.ceil(d/(8*scale))));ctx.save();ctx.lineCap='round';ctx.strokeStyle='#d78027';ctx.lineWidth=Math.max(1,2*scale);ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();for(let i=0;i<=count;i++){const k=i/count,x=a.x+dx*k,y=a.y+dy*k;ctx.fillStyle='#171a24';ctx.fillRect(Math.round(x-3*scale),Math.round(y-2*scale),6*scale,4*scale);ctx.fillStyle='#77757b';ctx.fillRect(Math.round(x-2*scale),Math.round(y-scale),4*scale,2*scale);}ctx.restore();}
};
