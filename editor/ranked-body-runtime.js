/* Presentation only: actual packaged base frames from the canonical art manifest.
 * Never changes a combat clock, input, replay or RNG. Rank fallback is explicit.
 */
(function(root){'use strict';
 const script=document.currentScript.src,loaded=new Map(),api={ready:false,manifest:null,error:null};
 const enc=p=>p.split('/').map(encodeURIComponent).join('/');
 api.readyPromise=fetch(new URL('ranked-body-assets.json',script)).then(r=>{if(!r.ok)throw Error('Body manifest unavailable');return r.json()}).then(async data=>{
  api.manifest=data;await Promise.all(Object.entries(data.characters).map(([key,c])=>new Promise((resolve,reject)=>{
   const im=new Image();let fallback=false;im.onload=()=>{loaded.set(key,im);resolve()};im.onerror=()=>{if(fallback)return reject(Error('Body unavailable: '+key));fallback=true;im.src=new URL('../'+enc(c.png),script).href};
   im.src=new URL('rank-art-body-'+key+'.png',script).href;
  })));api.ready=true;return api;
 }).catch(e=>{api.error=e.message;return api});
 api.draw=function(ctx,key,pose,time,x,y,scale,left=false,frameIndex=null){
  if(!api.ready)return false;const c=api.manifest.characters[key],im=loaded.get(key),anim=c?.anims[pose];if(!anim||!im)return false;
  let frame;if(frameIndex!==null)frame=anim.frames[Math.max(0,Math.floor(frameIndex))%anim.frames.length];else{let t=Math.max(0,time)%anim.frames.reduce((s,f)=>s+f.duration,0);for(const f of anim.frames){frame=f;if(t<f.duration)break;t-=f.duration}}
  const r=frame.data;ctx.save();ctx.imageSmoothingEnabled=false;ctx.translate(x,y);if(left)ctx.scale(-1,1);ctx.drawImage(im,r.x,r.y,r.w,r.h,-r.w*scale/2,-r.h*scale/2,r.w*scale,r.h*scale);ctx.restore();return true;
 };
 root.RankedBodyArt=api;
})(window);
