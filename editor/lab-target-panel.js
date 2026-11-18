/* Stable right-side rows; updating counters never replaces focused controls. */
window.LabTargetPanel=class {
 constructor(catalog,api){this.catalog=catalog;this.api=api;this.rows=new Map();this.lastTick=-Infinity;this.$=id=>document.getElementById(id);this.populate();
  this.$('targetSearch').oninput=()=>this.populate();
  this.$('addTargets').onclick=async()=>{const b=this.$('addTargets');b.disabled=true;this.$('targetError').textContent='';try{await api.add(this.$('targetType').value,+this.$('targetCount').value,+this.$('targetLevel').value,this.$('spawnUnlimited').checked);}catch(e){this.$('targetError').textContent=e.message;}finally{b.disabled=false;}};
  this.$('clearTargets').onclick=()=>api.clear();this.$('resetDamage').onclick=()=>{api.ledger().reset();api.resetMetrics();this.update(api.tick(),true);};
 }
 populate(){const previous=this.$('targetType').value,find=this.$('targetSearch').value.toLowerCase(),select=this.$('targetType');select.replaceChildren();const groups=new Map();
  for(const e of this.catalog.entries.values()){if(!e.name.toLowerCase().includes(find)&&!e.id.includes(find))continue;const label=e.id.startsWith('stock:')?'Stock champions':e.id.startsWith('custom:')?'Custom champions':e.kind==='summon'?'Summon templates':'Minions, camps and structures';if(!groups.has(label)){const g=document.createElement('optgroup');g.label=label;groups.set(label,g);select.append(g);}groups.get(label).append(new Option(e.name,e.id));}
  if([...select.options].some(o=>o.value===previous))select.value=previous;else if(!find)select.value='stock:knight';
 }
 rebuild(){const list=this.$('targetList');list.replaceChildren();this.rows.clear();
  for(const u of this.api.units().filter(u=>u.team!==0)){
   const row=document.createElement('section');row.className='targetRow';row.dataset.targetId=u.id;const title=document.createElement('h3');title.textContent=`#${u.id} ${u.name}`;row.append(title);
   const kind=document.createElement('small');kind.textContent=`${u.kind} · level ${u.level||this.api.level()}`;row.append(kind);const total=document.createElement('div');total.className='damageTotal';row.append(total);const details=document.createElement('small');details.className='damageBreakdown';row.append(details);const status=document.createElement('small');row.append(status);
   const unlimited=document.createElement('input');unlimited.type='checkbox';unlimited.checked=!!u.unlimitedHP;unlimited.setAttribute('aria-label',`Unlimited HP for target ${u.id}`);const label=document.createElement('label');label.append(unlimited,' Unlimited HP');row.append(label);
   unlimited.onchange=()=>{u.unlimitedHP=unlimited.checked;if(u.unlimitedHP){u.hp=u.max_hp;delete u.deadAt;}this.api.changed(u);this.update(this.api.tick(),true);};
   const settings=document.createElement('details'),summary=document.createElement('summary');summary.textContent='Target setup / placement';settings.append(summary);row.append(settings);
   const number=(text,value,max,change)=>{const label=document.createElement('label'),input=document.createElement('input');input.type='number';input.value=value;input.min=text==='Max HP'?1:0;input.max=max;input.setAttribute('aria-label',`${text} for target ${u.id}`);label.append(text+' ',input);settings.append(label);input.onchange=()=>{const v=Math.max(+input.min,Math.min(max,Math.round(+input.value||+input.min)));input.value=v;change(v);this.api.changed(u);this.update(this.api.tick(),true);};};
   number('Max HP',u.max_hp,100000000,v=>{u.max_hp=u.stats.hp=v;u.hp=u.unlimitedHP?v:Math.min(v,u.hp);});number('Armour',u.stats.defence||0,10000,v=>u.stats.defence=v);number('Shield',u.initialShield||0,100000000,v=>{u.initialShield=v;u.shields=v?[{amount:v,until:Number.MAX_SAFE_INTEGER}]:[];});
   const buttons=document.createElement('div');for(const[text,fn]of [['Focus',()=>this.api.focus(u)],['Move to aim',()=>this.api.move(u)],['Refill HP',()=>{u.hp=u.max_hp;delete u.deadAt;this.api.changed(u);}],['Reset',()=>{this.api.ledger().reset(u.id);this.update(this.api.tick(),true);}],['Remove',()=>this.api.remove(u)]]){const b=document.createElement('button');b.textContent=text;b.onclick=fn;if(['Reset','Remove'].includes(text))buttons.append(b);else settings.append(b);}row.append(buttons);list.append(row);this.rows.set(u.id,{row,total,details,status});
  }this.update(this.api.tick(),true);
 }
 update(tick,force=false){const ledger=this.api.ledger();if(!force&&tick<this.lastTick+6&&this.lastVersion===ledger.version)return;this.lastTick=tick;this.lastVersion=ledger.version;const units=this.api.units().filter(u=>u.team!==0);let sum=0,raw=0;
  for(const u of units){const row=this.rows.get(u.id);if(!row)continue;const r=ledger.row(u.id,tick);sum+=r.total;raw+=r.raw;row.total.textContent=`${Math.floor(r.total).toLocaleString()} damage · ${r.dps.toFixed(1)} DPS`;row.details.textContent=`Raw ${Math.floor(r.raw)} · HP ${Math.floor(r.hp)} · shield ${Math.floor(r.shield)} · ${r.hits} hits`;row.status.textContent=`${u.unlimitedHP?'∞ HP · max '+u.max_hp:'HP '+Math.max(0,Math.ceil(u.hp))+'/'+u.max_hp}${u.hp<=0?' · defeated':''}`;row.row.classList.toggle('selected',u.id===this.api.selected());}
  this.$('targetSummary').textContent=`1v${units.length} · ${units.filter(u=>u.hp>0).length} alive · ${Math.floor(sum).toLocaleString()} damage (${Math.floor(raw).toLocaleString()} raw)`;
 }
};
