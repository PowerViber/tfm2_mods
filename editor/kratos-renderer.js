// Candidate presentation only. Never reads/writes the combat core or combat RNG.
window.KratosPainter = class {
  constructor() {
    this.loaded = false;
    this.images = {};
    this.ready = this.load();
  }
  async load() {
    const base = location.pathname.startsWith('/editor/') ? '../docs/kratos-art-review/' : 'kratos-review-assets/';
    const response = await fetch(base + 'presentation.json');
    if (!response.ok) throw new Error('Kratos presentation manifest unavailable');
    this.manifest = await response.json();
    this.anims = {};
    const atlases = this.manifest.atlases && Object.fromEntries(Object.entries(this.manifest.atlases).filter(([key])=>key!=='assembled'));
    const files = atlases ? Object.fromEntries(Object.entries(atlases).map(([key,stem])=>[key,stem+'.png'])) : this.manifest.images;
    if (atlases && this.manifest.body_approved) {
      files.body=location.pathname.startsWith('/editor/')?'../mods/tfm2_custom/champions/tfm2_custom_kratos%23sheet.png':'/kratos-body.png';
      files.weapons=location.pathname.startsWith('/editor/')?'../mods/tfm2_custom/vfx/kratos_weapons%23sheet.png':'/kratos-weapons.png';
    }
    if (atlases && this.manifest.effects_approved) files.effects=location.pathname.startsWith('/editor/')?'../mods/tfm2_custom/vfx/kratos%23sheet.png':'/kratos-effects.png';
    await Promise.all(Object.entries(files).map(([key, file]) => new Promise((resolve, reject) => {
      const image = new Image(); image.onload = () => { this.images[key] = image; resolve(); };
      image.onerror = () => reject(new Error('Missing Kratos image: ' + file)); image.src = file.startsWith('/')||file.startsWith('../mods/')?file:base + file;
    })));
    if (atlases) await Promise.all(Object.entries(atlases).map(async ([key,stem]) => {
      const response=await fetch(base+stem+'.fanim');
      if(!response.ok)throw new Error('Missing Kratos animation metadata: '+stem);
      this.anims[key] = (await response.json()).anims;
      if(!this.anims[key]||!Object.keys(this.anims[key]).length)throw new Error('Invalid Kratos animation metadata: '+stem);
    }));
    this.loaded = true;
  }
  sprite(ctx, name, x, y, scale, left = false, angle = 0) {
    const image = this.images[name]; if (!image) return false;
    const anchor = this.manifest.anchors[name] || [image.width / 2, image.height / 2];
    ctx.save(); ctx.translate(x, y); ctx.rotate(angle); ctx.scale(left ? -scale : scale, scale);
    ctx.imageSmoothingEnabled = false; ctx.drawImage(image, -anchor[0], -anchor[1]); ctx.restore();
    return true;
  }
  body(ctx, pose, tick, x, y, scale, left = false, blades = false, start = 0) {
    if (this.manifest.atlases) {
      pose = {movement:'run'}[pose] || pose;
      const durations=this.manifest.pose_ticks[pose]||this.manifest.pose_ticks.idle;
      const index=this.frame(durations,tick-start,['idle','run'].includes(pose));
      const weapon=this.manifest.weapon_prefix+(blades?'blades':'axe')+'_'+pose+(left?'_l':'_r')+index;
      this.animation(ctx,'weapons',weapon,0,x,y,scale);
      return this.animation(ctx,'body',pose+(left?'_l':''),tick-start,x,y,scale,['idle','run'].includes(pose));
    }
    const tag = pose === 'idle' && blades ? 'blades-idle' : pose === 'idle' ? 'axe-idle' : pose;
    const frames = this.manifest.body[tag] || this.manifest.body[blades ? 'blades-idle' : 'axe-idle'];
    let elapsed = Math.max(0, tick - start), index = 0;
    const total = frames.reduce((n, f) => n + f.ticks, 0);
    elapsed = ['axe-idle', 'blades-idle', 'movement'].includes(tag) ? elapsed % total : Math.min(total - 1, elapsed);
    while (index < frames.length - 1 && elapsed >= frames[index].ticks) elapsed -= frames[index++].ticks;
    return this.sprite(ctx, frames[index].image, x, y, scale, left);
  }
  wolf(ctx, from, to, tick, x, y, scale) {
    const angle = Math.atan2(to.y - from.y, to.x - from.x);
    if (this.manifest.atlases) {
      const h=((Math.round(angle/(Math.PI/4))%8)+8)%8;
      const phase=Math.floor(tick/3)%8, name='fenrir_p'+h+(phase?'_f'+phase:'');
      return this.animation(ctx,'effects',this.anims.effects[name]?name:'fenrir_p'+h,0,x,y,scale,false,[40,40]);
    }
    return this.sprite(ctx, 'fenrir-east', x, y, scale, false, angle);
  }
  animation(ctx, atlas, name, elapsed, x, y, scale, loop = false, anchor = null) {
    const animation=this.anims[atlas]?.[name]; if(!animation)return false;
    const total=animation.frames.reduce((sum,f)=>sum+f.duration*60,0);
    let remaining=Math.max(0,elapsed);remaining=loop?remaining%total:Math.min(total-1e-6,remaining);
    let index=0;while(index<animation.frames.length-1&&remaining>=animation.frames[index].duration*60)remaining-=animation.frames[index++].duration*60;
    const d=animation.frames[index].data, a=anchor||[d.w/2,d.h/2];
    ctx.save();ctx.imageSmoothingEnabled=false;
    ctx.drawImage(this.images[atlas],d.x,d.y,d.w,d.h,x-a[0]*scale,y-a[1]*scale,d.w*scale,d.h*scale);ctx.restore();return true;
  }
  frame(durations,elapsed,loop=false) {
    const total=durations.reduce((a,b)=>a+b,0);let remaining=Math.max(0,Math.floor(elapsed));remaining=loop?remaining%total:Math.min(total-1,remaining);
    for(let i=0;i<durations.length;i++){if(remaining<durations[i])return i;remaining-=durations[i];}return durations.length-1;
  }
  effect(ctx, name, elapsed, x, y, scale) {
    return this.animation(ctx,'effects',name.replace(/^krt_/,''),elapsed,x,y,scale);
  }
};
