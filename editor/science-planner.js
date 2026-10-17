/* Bounded athlete choice model. Manual commands never call this planner. */
(function(root){'use strict';
function install(A,D){const offense=new Set(A.OFFENSE),physical=new Set([16,29,34,39,40]),heavy=new Set([34,63,71,73,74]),dist=(a,b)=>(a.x-b.x)**2+(a.y-b.y)**2;
 A.Combat.prototype.suggest=function(){const m=this.hero,rank=this.rank,foes=this.dummies.filter(e=>e.hp&&e.visible),target=foes.filter(e=>dist(m,e)<=100000**2).sort((a,b)=>dist(m,a)-dist(m,b)||a.id-b.id)[0];if(!target||!this.bank.available())return null;
 if(this.theory.target!==target.id)this.command('lock',target.id);
 const comfort=[1,1,2,2,3,4,5,5][rank],limit=[8,16,24,40,64,96,128,128][rank],near=foes.some(e=>dist(m,e)<45000**2);let best=null,branches=0;
 for(let n=0;n<125;n++){const wanted=n<75?[n]:D.combos[n-75].steps,prep=this.preparation(wanted),steps=prep===null?wanted:[prep];if(wanted.some(s=>dist(m,target)>D.skills[s].range**2)||steps.reduce((n,s)=>n+A.materialCost(s),0)>this.material||!this.valid(steps[0]))continue;
  const s=steps[0];if(prep!==null&&this.deliveries.some(q=>q.skill===s))continue;
  if(prep===null&&[5,12,13,18,20,45,57,58,65,66].includes(s)&&this.fields.some(f=>f.kind===s))continue;
  if(prep===null&&[6,14,17,21,31,41,52,53,55,56,60,67,68].includes(s)&&m.hp*2>=m.maxhp)continue;
  if(s===41&&!this.incoming.some(q=>dist(m,q)<=45000**2))continue;
  if(wanted.includes(19)&&!(this.anchors.length>=2&&A.near(this.anchors[0],this.anchors.at(-1),target,12000)))continue;
  if(wanted.includes(29)&&this.theory.vector.some(Boolean)){const[u,v]=A.unit(target.x-m.x,target.y-m.y);if(this.theory.vector[0]*u+this.theory.vector[1]*v<=0)continue;}
  if(s===21&&this.theory.aim&&dist(m,this.theory.aim)>=dist(m,target))continue;
  if(steps.some(s=>offense.has(s)||D.skills[s].purpose==='control')&&this.packets.length+this.deliveries.length>=8)continue;
  const attack=wanted.some(s=>offense.has(s)),defend=wanted.some(s=>D.skills[s].purpose==='defend'||D.skills[s].role==='Protect'),control=wanted.some(s=>D.skills[s].purpose==='control'),urgent=defend&&m.hp*2<m.maxhp&&foes.length&&prep===null&&D.skills[s].purpose==='defend';if(!urgent&&!attack&&!control)continue;
  const mask=wanted.reduce((v,s)=>v|1<<D.skills[s].science,0),preparation=wanted.reduce((v,s)=>v+18+D.skills[s].tokens.length*10,0)+(prep===null?0:45),output=wanted.reduce((v,s,i)=>v+(offense.has(s)?Math.trunc((heavy.has(s)?90+Math.trunc((physical.has(s)?m.attack:m.ap)*45/100):50+Math.trunc((physical.has(s)?m.attack:m.ap)/4))*A.chainPct(i)/100):0),0);
  const materials=wanted.reduce((v,s)=>v+A.materialCost(s),0),expectedMaterial=this.material+Math.trunc((this.tick%180+preparation)/180);if(materials>expectedMaterial)continue;
  let utility=100+Math.trunc(output*100/Math.max(preparation,this.bank.available()>=2&&near?60:100,Math.trunc(materials*180*(8-Math.min(this.material,8))/8)))+(control?30:0)+(wanted.length<=comfort?25:-(wanted.length-comfort)*(8-rank)*4);if(this.plannerGoal===n)utility+=150;
  if(branches<limit){branches++;const velocity=this.observations.get(target.id)?.velocity||[0,0];utility-=Math.trunc(Math.min(1500,Math.abs(velocity[0])+Math.abs(velocity[1]))*preparation*rank/60000);if(near)utility-=Math.trunc(preparation*rank/12);utility+=([1,2,4].filter(x=>mask&x).length-1)*rank*4;}
  const preferred=this.style<3?!!(mask&(1<<this.style)):this.style===3?defend:this.style===4?wanted.length<=2:mask===7&&wanted.length>=3,signature=n>=75&&this.signatures.includes(n-75),preference=(preferred?(this.style<3?Math.trunc(60/[1,2,4].filter(x=>mask&x).length):60):0)+(signature?40:0);utility+=Math.trunc(Math.max(0,utility)*preference/1000);
  const score=(urgent?3:1)*10000+Math.max(-9000,Math.min(9000,utility));if(!best||score>best.score||score===best.score&&this.random(2)===0)best={score,steps:steps.slice(),goal:prep===null?null:n,reason:urgent?'urgent defense':prep===null?'delivery value after preparation and enemy reply':'funded prerequisite'};
 }
 if(best){best.branches=branches;best.target=target.id;best.overcharge=rank>=2&&m.hp*100>m.maxhp*75&&!near&&best.steps.length>=3&&this.bank.available()===3;}return best;
 };
 const step=A.Combat.prototype.step;A.Combat.prototype.step=function(force=false){step.call(this,force);if(this.paused&&!force||!this.options.athleteAI||this.tick%6||this.plan||this.hero.hp<=0||this.hero.ccUntil>this.tick)return;const p=this.suggest();if(!p)return;this.plannerGoal=p.goal;this.lastReason=p.reason+' · '+p.branches+' reviewed branches';const stages=p.steps.map(s=>{let charge=D.skills[s].charge.slice();if(this.options.accuracy){const jitter=[40,30,22,16,10,6,3,1][this.rank];charge=charge.map(w=>Math.max(1,Math.trunc((w*(100+this.random(2*jitter+1)-jitter)+50)/100)));if(this.random(100)<[15,25,40,55,70,85,96,100][this.rank])charge=D.skills[s].charge.slice();}return {skill:s,charge};});if(this.cast(stages)&&p.overcharge)this.beginOvercharge();};
}
if(typeof module!=='undefined')module.exports=install;else install(root.ScienceCombat,root.ScienceData);
})(globalThis);
