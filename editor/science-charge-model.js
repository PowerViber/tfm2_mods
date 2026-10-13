/* Shared integer charge timing and safe native-log parsing. */
(function(root){
'use strict';
class ChargeBank{
 constructor(){this.slots=[null,null,null];}
 advance(tick){this.slots=this.slots.map(s=>typeof s==='number'&&s<=tick?null:s);}
 available(){return this.slots.filter(s=>s===null).length;}
 reserve(tick){this.advance(tick);const i=this.slots.indexOf(null);if(i<0)return null;this.slots[i]='reserved';return i;}
 commit(i,tick){if(this.slots[i]!=='reserved')return false;this.slots[i]=tick+300;return true;}
 cancel(i){if(this.slots[i]==='reserved')this.slots[i]=null;}
 cells(tick){return this.slots.map(s=>s===null?10:s==='reserved'?11:s<=tick?10:Math.min(9,Math.floor(Math.max(0,300-(s-tick))/30)));}
}
function parseDiagnostics(raw){
 const rows=[];for(const line of raw.split(/\r?\n/)){
  const tick=/\btick (\d+)/.exec(line),caster=/\bcaster (\d+)/.exec(line);if(!tick||!caster)continue;
  const selection=/selection reason="([^"]*)" target (\d+)/.exec(line);
  const release=/release (\w+) target (\d+) origin (-?\d+),(-?\d+) aim (-?\d+),(-?\d+)/.exec(line);
  const spent=/charge_spent slot (\d+) recovery (\d+)/.exec(line),recovered=/charge_recovered slot (\d+)/.exec(line);
  const hit=/target (\d+) skill (\w+).*HP (\d+)->(\d+)/.exec(line);
  const text=selection?`Selection: ${selection[1]} · target ${selection[2]}`:release?`Release ${release[1]} · target ${release[2]} · origin (${release[3]}, ${release[4]}) · aim (${release[5]}, ${release[6]})`:spent?`Charge ${Number(spent[1])+1} spent · ready at tick ${spent[2]}`:recovered?`Charge ${Number(recovered[1])+1} recovered`:hit?`Hit ${hit[2]} · target ${hit[1]} · HP ${hit[3]} → ${hit[4]}`:null;
  if(text)rows.push({tick:Number(tick[1]),caster:Number(caster[1]),text});
 }return rows.sort((a,b)=>a.tick-b.tick).slice(-200);
}
const api={ChargeBank,RECHARGE_TICKS:300,parseDiagnostics};
if(typeof module!=='undefined')module.exports=api;else root.ScienceCharges=api;
})(typeof globalThis!=='undefined'?globalThis:this);
