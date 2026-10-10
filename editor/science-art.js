/* Animated inspection of the exact runtime artwork. No combat simulation. */
(function () {
  'use strict';
  const $ = s => document.querySelector(s);
  const canvas = $('#artCanvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d'), images = new Map();
  const colors = ['#63bfff', '#ffc96b', '#63e1bd'];
  let manifest, composite, started = performance.now(), paused = false, frozen = 0, completionUntil = 0, pendingFrame = null, transforming = null;
  function loadImage(src) {
    if (!images.has(src)) {
      const im = new Image();
      im.onload = () => render(performance.now());
      im.onerror = () => { $('#artStatus').textContent = 'Artwork unavailable: ' + src; };
      im.src = src; images.set(src, im);
    }
    return images.get(src);
  }
  function runtime(tag, time, x, y, scale) {
    const asset = manifest.assets[tag];
    if (!asset) throw Error('Missing runtime animation: ' + tag);
    const src = '/science-runtime/' + asset.sheet + '.png';
    const im = loadImage(src);
    const total = asset.frames.reduce((n, f) => n + f.duration, 0);
    let at = (time / 1000) % total, selected = asset.frames[0], index = 0;
    for (let i=0;i<asset.frames.length;i++) { const frame=asset.frames[i]; selected=frame; index=i; if(at<frame.duration)break; at-=frame.duration; }
    const f = selected.data;
    if (im.complete && im.naturalWidth) ctx.drawImage(im, f.x, f.y, f.w, f.h, x-f.w*scale/2, y-f.h*scale/2, f.w*scale, f.h*scale);
    return index+1;
  }
  function render(now) {
    if (!manifest) return;
    const time = paused ? frozen : now-started;
    const rank = Number($('#artRank').value), position = Number($('#artPosition').value);
    const elapsed = transforming ? time-transforming.started : 0;
    if (transforming && elapsed >= 400) {
      $('#artSkill').value=String(transforming.to*25);
      $('#artFrom').value=String(transforming.to);
      $('#artTo').value=String(transforming.from);
      transforming=null;
      $('#artTransformStatus').textContent='Chosen form ready; transformation cooldown finished.';
    }
    $('#artTransform').disabled=Boolean(transforming) || $('#artFrom').value===$('#artTo').value;
    const skill = ScienceData.skills[Number($('#artSkill').value)], science = transforming ? transforming.to : skill.science;
    const podium = rank === 7 ? Math.min(position, 4) : 4;
    const tier = [0,0,0,1,1,2,2,3][rank];
    const key = `${science}_${rank}_${podium}`;
    const frames = manifest.outfits[key], f = frames[Math.floor(time/120)%frames.length];
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle='#0b1625'; ctx.fillRect(0,0,canvas.width,canvas.height);
    ctx.strokeStyle='#1e3046'; ctx.lineWidth=1;
    for(let x=20;x<canvas.width;x+=32){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,canvas.height);ctx.stroke();}
    for(let y=20;y<canvas.height;y+=32){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(canvas.width,y);ctx.stroke();}
    if(now<completionUntil)runtime(`complete_t${tier}_p${podium}`,now-(completionUntil-800),161,170,2);
    if(transforming)runtime(`gearback${transforming.to}_r${rank}`+(rank===7?`_p${podium}`:''),time,161,170,2);
    if(composite.complete && composite.naturalWidth) {
      const body=transforming ? manifest.base : f;
      ctx.drawImage(composite,body.x,body.y,body.w,body.h,65,58,body.w*2,body.h*2);
    }
    if(transforming) {
      const frame=runtime(`transform${transforming.from}_${transforming.to}_r${rank}`,elapsed,161,170,2);
      $('#artTransformStatus').textContent=`Frame ${frame} / 8 · feet planted · choosing ${['Einstein','Newton','Marie Curie'][transforming.to]}`;
    }
    runtime(rank===7?'top'+position:'rank'+rank,time,161,170,2);
    const fieldTag=`field${Number($('#artSkill').value)}_t${tier}`,packetTag=`packet_${skill.id}_t${tier}_a0`;
    const worldTag=manifest.assets[fieldTag]?fieldTag:manifest.assets[packetTag]?packetTag:null;
    $('#artMode option[value="world"]').disabled=!worldTag;
    if(!worldTag)$('#artMode').value='cast';
    const tag=$('#artMode').value==='world'?worldTag:'skill_'+skill.id+'_t'+tier;
    const frame=runtime(tag,time,445,169,2);
    ctx.fillStyle='#9db5c9';ctx.font='12px monospace';ctx.fillText(`SPRITE ${frame} / 8`,400,36);
    ctx.fillStyle=colors[science];ctx.font='13px monospace';ctx.fillText(skill.id+' / '+skill.name,316,288);
    $('#artPosition').disabled=rank!==7;
    const stage=['Cosmic seed','Celestial engine','Impossible laboratory','Unified universe'][tier];
    $('#artCaption').textContent=ScienceLab.ranks[rank]+(rank===7?' #'+position:'')+' · '+stage+' · '+['Einstein','Newton','Marie Curie'][science];
    if(!paused && pendingFrame===null)pendingFrame=requestAnimationFrame(now=>{pendingFrame=null;render(now);});
  }
  $('#artPause').onclick=()=>{
    paused=!paused;
    if(paused)frozen=performance.now()-started;else started=performance.now()-frozen;
    $('#artPause').textContent=paused?'Play animation':'Pause animation';render(performance.now());
  };
  $('#artComplete').onclick=()=>{completionUntil=performance.now()+800;render(performance.now());};
  for(const id of ['#artRank','#artPosition','#artSkill','#artMode'])$(id).onchange=()=>render(performance.now());
  $('#artTransform').onclick=()=>{
    if(transforming || $('#artFrom').value===$('#artTo').value)return;
    transforming={from:Number($('#artFrom').value),to:Number($('#artTo').value),started:paused?frozen:performance.now()-started};
    render(performance.now());
  };
  for(const id of ['#artFrom','#artTo'])$(id).onchange=()=>{
    if(!transforming)$('#artSkill').value=String(Number($('#artFrom').value)*25);
    render(performance.now());
  };
  $('#artSkill').onchange=()=>{
    if(!transforming) {
      const science=ScienceData.skills[Number($('#artSkill').value)].science;
      $('#artFrom').value=String(science);$('#artTo').value=String((science+1)%3);
    }
    render(performance.now());
  };
  ScienceData.skills.forEach((s,i)=>{const o=document.createElement('option');o.value=i;o.textContent=s.id+' / '+s.name;$('#artSkill').append(o);});
  Promise.all([fetch('/science-art-preview.json').then(r=>{if(!r.ok)throw Error('Artwork manifest unavailable');return r.json();}),new Promise((resolve,reject)=>{
    composite=new Image();composite.onload=resolve;composite.onerror=()=>reject(Error('Outfit preview unavailable'));composite.src='science-mastery-preview.png';
  })]).then(([m])=>{manifest=m;$('#artStatus').textContent='Original game sprites · Eight-frame cosmic animations · 75 skills · 8 ranks · 10 leaderboard positions';render(performance.now());}).catch(e=>{$('#artStatus').textContent=e.message;});
})();
