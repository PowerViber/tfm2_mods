/* The browser runs the compiled native rules. This file only transports frames. */
(function(root){
  'use strict';
  class MagnusEngine {
    static async load(url='/magnus.wasm'){
      const bytes=await (await fetch(url)).arrayBuffer();
      const {instance}=await WebAssembly.instantiate(bytes,{});
      return new MagnusEngine(instance.exports);
    }
    constructor(api){this.api=api;this.handle=api.magnus_new(42,0,1,3);}
    reset(seed=42,rank=3,owner=1){this.api.magnus_drop(this.handle);this.handle=this.api.magnus_new(seed>>>0,0,owner,rank);}
    read(pointer){return JSON.parse(new TextDecoder().decode(new Uint8Array(this.api.memory.buffer,pointer,this.api.magnus_output_len())));}
    step(frame){
      const o=frame.owner,a=frame.action||{type:-1};
      const fighters=frame.fighters||[],walls=frame.walls||[],captures=frame.captures||[];
      const values=[frame.tick,o.hp>0?1:0,o.team,o.x,o.y,o.hp,o.maxHp,o.attack,o.ap,frame.controlled?1:0,frame.automatic?1:0,a.type,a.piece||0,a.x||0,a.y||0,a.heading||0,frame.rank||0,fighters.length,walls.length,captures.length,...(frame.pieceHp||Array(7).fill(-1))];
      for(const f of fighters)values.push(f.id,f.team,f.x,f.y,f.hp,f.maxHp,f.attack,f.ap,f.champion?1:0,f.visible?1:0,f.targetable?1:0,f.tags||0);
      for(const r of walls)values.push(...r);
      for(const c of captures)values.push(c.piece,c.by??-1,c.combat?1:0);
      const ptr=this.api.magnus_input(values.length);
      if(!ptr)throw new Error('Practice frame exceeds the rules buffer.');
      const buffer=new BigInt64Array(this.api.memory.buffer,ptr,values.length);
      buffer.set(values.map(n=>BigInt(Math.round(n))));
      return this.read(this.api.magnus_step(this.handle,values.length));
    }
    fixture(id){return this.read(this.api.magnus_fixture(this.handle,id));}
    piece(id,x,y,hp){return this.api.magnus_piece(this.handle,id,x,y,hp);}
    challenge(id){return this.api.magnus_challenge(this.handle,id);}
  }
  root.MagnusEngine=MagnusEngine;
  if(typeof module!=='undefined')module.exports=MagnusEngine;
})(typeof window!=='undefined'?window:globalThis);
