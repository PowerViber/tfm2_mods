// Shared read-only game reference and collision/routing helpers for playable Labs.
window.GameWorld=class {
  constructor(){this.images=new Map();this.pending=new Map();this.routes=new Map();this.ready=this.load();}
  async load(){
    this.base='game-reference/';const r=await fetch(this.base+'world.json');
    if(!r.ok)throw new Error('Installed game reference unavailable. Run node editor/extract-game-world.js; no estimated Levi fallback is used.');
    this.data=await r.json();this.geometry=this.data.geometry;this.cell=this.geometry.cell;
    this.walls=this.data.doc.walls.flat().map(Boolean);
    this.ground=document.createElement('canvas');this.ground.width=this.ground.height=this.geometry.art_size_pixels;
    const ctx=this.ground.getContext('2d');
    for(const layer of this.data.layers){const im=await this.image(layer.file);if(layer.foreground)this.front=im;else{ctx.drawImage(im,0,0);this.images.delete(layer.file);this.pending.delete(layer.file);}}
    return this;
  }
  image(file){if(this.images.has(file))return Promise.resolve(this.images.get(file));if(this.pending.has(file))return this.pending.get(file);
    const p=new Promise((resolve,reject)=>{const im=new Image();im.onload=()=>{this.images.set(file,im);resolve(im);};im.onerror=()=>reject(new Error('Missing extracted game image: '+file));im.src=this.base+file;});this.pending.set(file,p);return p;}
  loadSprites(ids){return Promise.all([...new Set(ids)].map(id=>{const s=this.data.sprites[id];if(!s)throw new Error('Unknown stock sprite '+id);return this.image(s.file);}));}
  wallAt(p,radius=0){
    const g=this.geometry;if(p.x-radius<0||p.y-radius<0||p.x+radius>=g.width||p.y+radius>=g.height)return true;
    for(let y=Math.floor((p.y-radius)/this.cell);y<=Math.floor((p.y+radius)/this.cell);y++)for(let x=Math.floor((p.x-radius)/this.cell);x<=Math.floor((p.x+radius)/this.cell);x++){
      if(!this.walls[y*30+x])continue;const qx=Math.max(x*this.cell,Math.min((x+1)*this.cell,p.x)),qy=Math.max(y*this.cell,Math.min((y+1)*this.cell,p.y));
      if((p.x-qx)**2+(p.y-qy)**2<=radius**2)return true;
    }return false;
  }
  sweep(a,b,radius=0){const n=Math.max(1,Math.ceil(Math.hypot(b.x-a.x,b.y-a.y)/2000));let last={...a};for(let i=1;i<=n;i++){const q={x:a.x+(b.x-a.x)*i/n,y:a.y+(b.y-a.y)*i/n};if(this.wallAt(q,radius))break;last=q;}return last;}
  clear(a,b,radius=0){const p=this.sweep(a,b,radius);return Math.hypot(p.x-b.x,p.y-b.y)<1;}
  legal(p,radius=10000){
    if(!this.wallAt(p,radius))return {...p};let best=null;
    for(let y=0;y<30;y++)for(let x=0;x<30;x++){const q={x:(x+.5)*this.cell,y:(y+.5)*this.cell},d=(q.x-p.x)**2+(q.y-p.y)**2;if(!this.wallAt(q,radius)&&(!best||d<best.d))best={q,d};}
    if(!best)throw new Error('No legal placement');return best.q;
  }
  path(from,to,radius=10000){
    const goal=this.legal(to,radius),cell=p=>Math.floor(p.y/this.cell)*30+Math.floor(p.x/this.cell),start=cell(from),end=cell(goal),key=start+':'+end+':'+radius;
    if(this.clear(from,goal,radius))return[goal];if(this.routes.has(key))return this.routes.get(key);
    const queue=[start],prev=new Map([[start,null]]);let found=false;
    for(let i=0;i<queue.length&&i<900;i++){const a=queue[i];if(a===end){found=true;break;}const x=a%30,y=Math.floor(a/30);
      for(const[dx,dy]of[[1,0],[0,1],[-1,0],[0,-1],[1,1],[-1,1],[1,-1],[-1,-1]]){const nx=x+dx,ny=y+dy,id=ny*30+nx;
        if(nx<0||nx>=30||ny<0||ny>=30||prev.has(id))continue;const p={x:(x+.5)*this.cell,y:(y+.5)*this.cell},q={x:(nx+.5)*this.cell,y:(ny+.5)*this.cell};
        if(this.wallAt(q,radius)||!this.clear(p,q,radius))continue;prev.set(id,a);queue.push(id);}
    }
    const out=[];if(found){let a=end;while(a!==start){out.push({x:(a%30+.5)*this.cell,y:(Math.floor(a/30)+.5)*this.cell});a=prev.get(a);}out.reverse();out.push(goal);}
    if(this.routes.size>256)this.routes.clear();this.routes.set(key,out);return out;
  }
  move(actor,to,speed){let target=to;if(!this.clear(actor.pos,to,actor.radius)){const route=this.path(actor.pos,to,actor.radius);target=route.find(p=>Math.hypot(p.x-actor.pos.x,p.y-actor.pos.y)>speed)||route.at(-1);if(!target)return false;if(!this.clear(actor.pos,target,actor.radius))target={x:(Math.floor(actor.pos.x/this.cell)+.5)*this.cell,y:(Math.floor(actor.pos.y/this.cell)+.5)*this.cell};}
    const d=Math.hypot(target.x-actor.pos.x,target.y-actor.pos.y);if(!d)return false;const k=Math.min(1,speed/d),next={x:actor.pos.x+(target.x-actor.pos.x)*k,y:actor.pos.y+(target.y-actor.pos.y)*k},p=this.sweep(actor.pos,next,actor.radius);
    actor.left=p.x<actor.pos.x;const moved=Math.hypot(p.x-actor.pos.x,p.y-actor.pos.y)>.01;actor.pos=p;return moved;
  }
  drawMap(ctx,P,W,front=false){const p=P({x:-this.geometry.art_padding_pixels*1000,y:-this.geometry.art_padding_pixels*1000}),size=W(this.geometry.art_size_pixels*1000);ctx.imageSmoothingEnabled=false;ctx.drawImage(front?this.front:this.ground,p.x,p.y,size,size);}
  drawUnit(ctx,u,tick,P,W){
    const spec=this.data.sprites[u.sprite];if(!spec)return false;const image=this.images.get(spec.file);if(!image)return false;
    const pose=u.hp<=0?'dead':tick<u.attackUntil?'attack':u.moving?'run':'idle';let tag=u.minion?`${u.minion}_${u.team?'red':'blue'}_${pose}`:pose;
    if(!u.minion&&pose==='attack'&&spec.anims[u.attack?.action_name])tag=u.attack.action_name;
    let a=spec.anims[tag]||spec.anims.idle||Object.values(spec.anims)[0];const elapsed=Math.max(0,tick-(pose==='attack'?u.attackStart:u.poseStart||0));
    const ds=a.frames.map(f=>Math.max(1,Math.round(f.duration*60000))),total=ds.reduce((x,y)=>x+y,0);let t=pose==='dead'?Math.min(total-1,elapsed*1000):elapsed*1000%total,index=0;
    while(index<ds.length-1&&t>=ds[index])t-=ds[index++];const d=a.frames[index].data,p=P(u.pos),s=W(this.geometry.units_per_pixel);
    ctx.save();ctx.translate(p.x,p.y);ctx.scale(u.left?-1:1,1);ctx.imageSmoothingEnabled=false;ctx.drawImage(image,d.x,d.y,d.w,d.h,-d.w*s/2,-d.h*s/2,d.w*s,d.h*s);ctx.restore();return true;
  }
};
