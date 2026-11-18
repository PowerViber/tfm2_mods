/* Lab-only damage accounting. Never sends infinite HP to the shared combat core. */
(function(root){'use strict';
class LabDamageLedger {
 constructor(owner=1){this.owner=owner;this.rows=new Map();this.version=0;}
 reset(id){if(id===undefined)this.rows.clear();else this.rows.delete(id);this.version++;}
 row(id,tick){const r=this.rows.get(id)||{raw:0,hp:0,shield:0,hits:0,first:null,last:null};return {...r,total:r.hp+r.shield,dps:(r.hp+r.shield)/Math.max(1,(tick-(r.first??tick)+1)/60)};}
 attach(scene){const base=scene.damage.bind(scene);scene.damage=(source,target,raw,tick)=>{
   if(target.hp<=0)return base(source,target,raw,tick);
   // A virtual sink lets the host model measure the whole post-mitigation packet.
   // The real target retains finite max HP, so %-HP formulas remain meaningful.
   const unlimited=target.unlimitedHP&&target.hp>0;
   const sink=unlimited?{...target,hp:Number.MAX_SAFE_INTEGER}:target;
   const shieldBefore=(target.shields||[]).filter(s=>tick<s.until).reduce((n,s)=>n+s.amount,0);
   const hp=base(source,sink,raw,tick);
   const shieldAfter=(target.shields||[]).filter(s=>tick<s.until).reduce((n,s)=>n+s.amount,0);
   const shield=Math.max(0,shieldBefore-shieldAfter);
   if(unlimited){target.hp=target.max_hp;delete target.deadAt;}
   if(source.id===this.owner&&raw>0){const r=this.rows.get(target.id)||{raw:0,hp:0,shield:0,hits:0,first:null,last:null};
     r.raw+=raw;r.hp+=hp;r.shield+=shield;r.hits++;r.first??=tick;r.last=tick;this.rows.set(target.id,r);this.version++;}
   return hp;
 };return scene;}
}
root.LabDamageLedger=LabDamageLedger;
if(typeof module!=='undefined')module.exports={LabDamageLedger};
})(typeof window!=='undefined'?window:globalThis);
