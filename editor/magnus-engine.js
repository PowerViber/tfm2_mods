/* Version 3 transport. All combat decisions and matching remain in Rust. */
(function(root){'use strict';
class MagnusEngine{
 static async load(url='/magnus.wasm'){const {instance}=await WebAssembly.instantiate(await(await fetch(url)).arrayBuffer(),{});return new MagnusEngine(instance.exports);}
 constructor(api){if(api.magnus_version()!==3)throw Error('Chess Lab requires version 3 rules. Reload the editor.');this.api=api;this.owner=1;this.handle=api.magnus_new(42,0,1,3);}
 reset(seed=42,rank=3,owner=1){this.api.magnus_drop(this.handle);this.owner=owner;this.handle=this.api.magnus_new(seed>>>0,0,owner,rank);}
 read(ptr){return JSON.parse(new TextDecoder().decode(new Uint8Array(this.api.memory.buffer,ptr,this.api.magnus_output_len())));}
 values(frame){const f=frame,a=f.action||{type:9},o=f.owner,fs=f.fighters||[],ws=f.walls||[],ss=f.smokes||[],cs=f.captures||[],hp=Array.isArray(f.pieceHp)?f.pieceHp:Object.entries(f.pieceHp||{}).map(([id,hp])=>[Number(id),hp]),hs=f.hits||[];
 const fighter=(u,owner=false)=>[u.id??(owner?this.owner:0),u.team,u.x,u.y,u.hp,u.maxHp,u.attack,u.ap,u.champion??owner?1:0,u.visible??true?1:0,u.targetable??true?1:0,u.tags||0,u.mateImmuneUntil||0,u.sightSource??owner?1:0];
 const values=[3,f.tick,f.controlled?1:0,f.automatic?1:0,f.rank??3,a.type,a.type===4?a.kind:a.piece||0,a.type===5?a.kind:a.x||0,a.y||0,a.direction??a.heading??0,fs.length,ws.length,ss.length,cs.length,hp.length,hs.length,f.ultUnlocked===false?0:1,f.topPosition||0,...fighter(o,true)];
 for(const u of fs)values.push(...fighter(u));for(const r of ws)values.push(...r);for(const s of ss)values.push(...s);for(const c of cs)values.push(c.piece,c.by??-1,c.combat?1:0);for(const p of hp)values.push(...p);for(const h of hs)values.push(h.command,h.piece,h.target,h.amount);return values;}
 step(frame){return this.raw(this.values(frame));}
 raw(values){const ptr=this.api.magnus_input(values.length);if(!ptr)throw Error('Practice transport could not allocate this frame.');new BigInt64Array(this.api.memory.buffer,ptr,values.length).set(values.map(n=>BigInt(Math.round(n))));return this.read(this.api.magnus_step(this.handle,values.length));}
 fixture(id){this.owner=1;return this.read(this.api.magnus_fixture(this.handle,id));}
 piece(id,x,y,hp){return this.api.magnus_piece(this.handle,id,x,y,hp);}
 add(kind,x,y,direction=0){return this.api.magnus_add(this.handle,kind,x,y,direction);}
 coverage(id){return this.read(this.api.magnus_coverage(this.handle,id));}
 challenge(id){return this.api.magnus_challenge(this.handle,id);}
}
root.MagnusEngine=MagnusEngine;if(typeof module!=='undefined')module.exports=MagnusEngine;
})(typeof window!=='undefined'?window:globalThis);
