/* Native handler parity and deterministic practice scenarios. */
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process');
const {Combat,unit,near}=require('../editor/science-combat.js'),D=require('../editor/unified-theory-data.js');
const root=path.resolve(__dirname,'..'),tracePath=path.join(root,'native/tfm2_custom_ai/target/science-native-handlers.json');
function prepared(){const c=new Combat({rank:7,seed:1});Object.assign(c.hero,{x:100000,y:100000});c.dummies=[1,2,3,4].map(id=>c.makeDummy(id,100000+id*25000,100000));c.aim={x:125000,y:100000};c.material=12;c.momentum=100;
 Object.assign(c.theory,{target:1,until:480,origin:{x:100000,y:100000},vector:[10000,0],aim:{x:125000,y:100000},cone:true,sample:true,label:true,half:3,acid:true,base:true,impulse:20});
 c.anchors=[{x:100000,y:100000,expires:960,born:0},{x:160000,y:100000,expires:960,born:0}];
 c.fields=[{kind:57,x:115000,y:100000,expires:360,experiment:1,payload:0,hp:null,born:0},{kind:65,x:125000,y:100000,expires:360,experiment:1,payload:50,hp:null,born:0}];
 c.packets=[{launch:null,skill:0,experiment:1,x:100000,y:100000,dx:10000,dy:0,payload:50,physical:false,class:0,expires:180,release:0,age:0,split:1,curve:0,orbit:false,hits:[],born:0}];c.budgets.set(1,400);c.nextId=1;return c;
}
function normalize(c,skill,phase){const t=c.theory,point=p=>p?[p.x,p.y]:null;return {skill,phase,material:c.material,momentum:c.momentum,
 state:[point(t.origin),point(t.aim),t.vector,t.velocity,t.acceleration,t.sync,t.proper,t.cone,t.link,t.curve,t.blue,t.pressure,t.exchange,t.drag,t.scalar,t.orbit,t.elastic,t.root,t.boundary,t.work,t.series,t.impulse,t.sample,t.purity,t.acid,t.base,t.catalyst,t.order,t.label,t.half],
 packets:c.packets.map(q=>[q.skill,q.x,q.y,q.dx,q.dy,q.payload,q.release,q.expires,q.physical,q.class,q.curve,q.orbit,q.split]),
 deliveries:c.deliveries.map(q=>[q.skill,q.x,q.y,q.destination.x,q.destination.y,q.payload,q.release,q.expires,q.placement,q.contact]),
 fields:c.fields.map(f=>[f.kind,f.x,f.y,f.payload,f.expires]),anchors:c.anchors.map(a=>[a.x,a.y,a.expires]),
 hits:c.events.filter(e=>e.type==='hit').map(e=>[e.tick,e.target,e.raw]),shields:c.events.filter(e=>e.type==='shield').map(e=>e.amount),buffs:c.events.filter(e=>e.type==='buff').map(e=>e.tag),position:[c.hero.x,c.hero.y]};}
function step(c,n){for(let i=0;i<n;i++)c.step();}
if(!process.env.SCIENCE_SKIP_NATIVE){
 fs.mkdirSync(path.dirname(tracePath),{recursive:true});
 // Windows maintainers run through rtk; elsewhere call cargo directly.
 const hasRtk=cp.spawnSync('rtk',['--version'],{encoding:'utf8'}).status===0;
 const native=hasRtk?cp.spawnSync('rtk',['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','test','--release','science_lab_native_handler_traces','--','--nocapture'],{cwd:path.join(root,'native/tfm2_custom_ai'),env:{...process.env,SCIENCE_NATIVE_TRACE_PATH:tracePath},encoding:'utf8',maxBuffer:4*1024*1024}):cp.spawnSync('cargo',['test','--offline','--release','science_lab_native_handler_traces','--','--nocapture'],{cwd:path.join(root,'native/tfm2_custom_ai'),env:{...process.env,SCIENCE_NATIVE_TRACE_PATH:tracePath},encoding:'utf8',maxBuffer:4*1024*1024});assert.equal(native.status,0,native.stdout+'\n'+native.stderr);
 const expected=JSON.parse(fs.readFileSync(tracePath,'utf8')),actual=[];
 for(let skill=0;skill<75;skill++){const c=prepared(),p={id:1,target:1,aim:{x:125000,y:100000},stages:[],index:0};assert.ok(c.execute(skill,p),D.skills[skill].id);actual.push(normalize(c,skill,'commit'));for(let tick=1;tick<=24;tick++){c.tick=tick;c.stepDeliveries();c.stepPackets();c.stepFields();}actual.push(normalize(c,skill,'world24'));}
 assert.equal(expected.length,150);for(let i=0;i<actual.length;i++)assert.deepEqual(JSON.parse(JSON.stringify(actual[i])),expected[i],D.skills[actual[i].skill].id+' '+actual[i].phase+' native parity');
 console.log('PASS: 150 native/JavaScript handler and world traces match for all 75 skills.');
}
// Independent deadlines, reservations, cancellation and failure.
{
 const c=new Combat();assert.ok(c.cast(0));assert.equal(c.bank.available(),2);c.cancel();assert.equal(c.bank.available(),3);assert.ok(c.cast(0));step(c,38);const deadline=c.bank.slots[0];assert.equal(deadline,338);c.cancel();assert.equal(c.bank.slots[0],deadline);
 assert.ok(c.cast({skill:0,charge:[1,20,1]}));step(c,38);assert.ok(c.events.some(e=>e.type==='commit'&&!e.accepted));assert.equal(c.bank.slots.filter(s=>typeof s==='number').length,2);
 assert.ok(c.cast(0));step(c,38);assert.equal(c.bank.available(),0);assert.equal(c.cast(0),false);step(c,224);assert.equal(c.bank.available(),1);assert.equal(c.bank.slots[1],376);step(c,76);assert.equal(c.bank.available(),3);
}
{
 const c=new Combat();c.cast(0);step(c,38);const deadlines=c.bank.slots.slice();c.command('health',0);c.command('respawn');assert.deepEqual(c.bank.slots,deadlines);c.command('health',0);step(c,300);assert.equal(c.bank.available(),3);c.command('respawn');assert.equal(c.bank.available(),3);
 c.cast(0);c.command('health',0);assert.equal(c.bank.available(),3,'Death must refund a reservation');
}
// Fresh origin and target prediction, fixed cursor aim, launch loss, contact art.
{
 const c=new Combat();c.cast(0);step(c,38);c.hero.x+=10000;c.dummies[0].y+=18000;c.command('aim',{x:1,y:1});step(c,12);const release=c.events.find(e=>e.type==='release');assert.equal(release.origin.x,100000);assert.ok(release.aim.y>=123000);assert.notEqual(release.aim.x,1);assert.ok(c.packets[0].dy>0);
 const q=c.packets[0],direction=[q.dx,q.dy];c.dummies[0].y-=70000;step(c,1);assert.deepEqual([q.dx,q.dy],direction,'Launched packets must not home');
 const f=new Combat();f.command('lock',null);f.command('aim',{x:140000,y:70000});f.cast(0);step(f,38);f.hero.y=115000;f.command('aim',{x:200000,y:170000});step(f,12);const r=f.events.find(e=>e.type==='release');assert.deepEqual(r.aim,{x:140000,y:70000});assert.equal(r.origin.y,115000);
 const lost=new Combat();lost.cast(0);step(lost,38);lost.dummies[0].visible=false;step(lost,12);assert.equal(lost.packets.length,0);assert.ok(lost.events.some(e=>e.type==='discard'));assert.equal(lost.bank.slots[0],338);
 const hit=new Combat();hit.cast(0);step(hit,90);const impact=hit.events.find(e=>e.type==='impact');assert.equal(impact.x,hit.dummies[0].x);assert.equal(impact.y,hit.dummies[0].y);
 const edge=new Combat();edge.cast(0);step(edge,38);edge.dummies[0].x=edge.hero.x+119000;step(edge,12);assert.ok(edge.events.some(e=>e.type==='release'),'A valid target must still launch when bounded prediction extends beyond the targeting range');
}
// Instruments refresh once, and preserve deliberate placement.
{
 const c=new Combat();c.cast(12);step(c,38);c.hero.x+=10000;c.dummies[0].y+=10000;step(c,12);assert.equal(c.deliveries[0].x>100000,true);assert.ok(c.deliveries[0].destination.y>=115000);const aim={...c.deliveries[0].destination};c.dummies[0].y+=20000;step(c,1);assert.deepEqual(c.deliveries[0].destination,aim);
 const mid=new Combat();mid.cast(13);step(mid,38);mid.hero.x=100000;mid.dummies[0].x=190000;step(mid,12);assert.equal(mid.deliveries[0].destination.x,145000);
 const forward=new Combat();forward.command('lock',null);forward.command('aim',{x:150000,y:105000});forward.cast(57);step(forward,38);forward.hero.x=100000;step(forward,12);assert.deepEqual(forward.deliveries[0].destination,{x:120000,y:105000});
}
// Manipulation, conservation, defensive and chemical outcomes.
{
 const c=prepared(),p={id:1,target:1,aim:c.aim,stages:[],index:0};const old=c.packets.reduce((n,q)=>n+q.payload,0);c.execute(27,p);assert.equal(c.packets.reduce((n,q)=>n+q.payload,0),old);c.execute(37,p);assert.ok(c.packets.every(q=>q.dx<0));c.execute(38,p);assert.ok(c.packets.every(q=>q.orbit));c.execute(39,p);assert.ok(c.packets.every(q=>!q.orbit));
 const shield=new Combat();shield.cast(67);step(shield,38);assert.equal(shield.hero.shields[0].amount,180);assert.ok(shield.hero.buffs.membrane);assert.equal(shield.packets.length,0);
 const chemistry=prepared();chemistry.hero.hp=800;chemistry.execute(56,{...p,aim:chemistry.aim});assert.equal(chemistry.hero.hp,875);assert.equal(chemistry.theory.acid,false);assert.equal(chemistry.theory.base,false);
 const cloud=new Combat();cloud.cast(65);step(cloud,100);assert.ok(cloud.fields.some(f=>f.kind===65));assert.ok(cloud.events.some(e=>e.type==='hit'&&e.skill===65));
 const control=new Combat();control.cast(5);step(control,110);assert.ok(control.dummies[0].buffs.dilation);
}
// Every authored recipe is accepted with legal pre-existing prerequisites. No
// runtime auto-grants are used: the prepared host is an isolated test fixture.
for(let index=0;index<50;index++){
 const c=prepared();c.hero.x=100000;c.dummies.forEach((d,i)=>{d.hp=d.maxhp=10000;d.x=125000+i*18000;d.home={x:d.x,y:d.y};});c.packets[0].orbit=true;
 c.theory.orbit=true;c.material=12;
 assert.ok(c.cast(D.combos[index].steps),D.combos[index].name+' blocked');step(c,400);
 assert.ok(c.events.some(e=>e.type==='complete'),D.combos[index].name+' failed: '+c.events.map(e=>e.text).join(' | '));
 assert.equal(c.events.filter(e=>e.type==='spent').length,1,D.combos[index].name+' duplicate charge');assert.ok(c.fields.length<=12);assert.ok(c.packets.length+c.deliveries.length<=8);
}
// Prerequisite casts really consume a separate charge, pause freezes time,
// movement creates momentum, support/control budgets remain bounded.
{
 const c=new Combat();assert.equal(c.availability(51).prerequisite,50);c.cast(50);step(c,38);assert.ok(c.cast(51));step(c,38);assert.equal(c.events.filter(e=>e.type==='spent').length,2);
 c.command('pause',true);const tick=c.tick;step(c,30);assert.equal(c.tick,tick);c.command('step');assert.equal(c.tick,tick+1);
 c.command('pause',false);c.command('move',{x:120000,y:105000});step(c,20);assert.equal(c.hero.x,110000);assert.ok(c.momentum>0);c.command('stop');step(c,10);assert.equal(c.hero.x,110000);
 c.cc(c.dummies[0],100);c.cc(c.dummies[0],100);assert.equal(c.dummies[0].ccUntil-c.tick,60);c.shield(300);c.shield(300);assert.ok(c.hero.shields.reduce((n,s)=>n+s.amount,0)<=250);
 const replay=()=>{const c=new Combat({rank:0,seed:57});c.options.accuracy=true;c.cast(0);step(c,200);return c.snapshot();};assert.deepEqual(replay(),replay());
}
{
 const c=new Combat();c.cast(0);c.hero.ap=10;step(c,38);assert.equal(c.packets[0].payload,150,'Experiment must use the stats frozen at reservation');
}
// Ten simultaneous scientist models with 32 visible enemies each. The same
// output, transient-start and history bounds apply throughout a crowded run.
{
 const casters=Array.from({length:10},(_,i)=>{const c=prepared();c.dummies=Array.from({length:32},(_,id)=>c.makeDummy(id+1,120000+(id%8)*12000,65000+Math.floor(id/8)*25000));c.dummies.forEach(d=>d.hp=d.maxhp=10000);c.options.moving=true;c.options.incoming=true;c.fields[1].x=170000;c.fields[1].y=100000;return c;});
 for(let tick=0;tick<720;tick++)for(const c of casters){if(tick%60===0&&c.availability(0).ok)c.cast(0);c.step();assert.ok(c.packets.length+c.deliveries.length<=8);assert.ok(c.fields.length<=12);assert.ok(c.visuals.length<=32);assert.ok(c.vfxLeft>=0&&c.vfxLeft<=6);assert.ok(c.events.length<=200);assert.ok(c.visuals.filter(v=>v.tag.startsWith('skill_')).length<=1);}
}
// Round 125: chain payoff, lab accidents and Unified resonance mirror native.
{
 const {chainPct,resonanceOutput}=require('../editor/science-combat.js');
 assert.deepEqual([0,1,2,3,4,5,6].map(chainPct),[100,120,140,160,180,180,180]);assert.equal(resonanceOutput(400,300),120);
 // Same payloads as native later_offense_in_a_successful_chain_carries_the_payoff.
 for(const [prior,expected] of [[0,150],[1,180],[2,210]]){const c=prepared();c.packets=[];const p={id:7,target:1,aim:{x:125000,y:100000},stages:[],index:prior,power:400,physical:300};assert.ok(c.execute(0,p,100,0));assert.equal(c.packets.find(q=>q.skill===0).payload,expected,'chain '+prior);}
 // A critical stage is a non-lethal lab accident with a stagger; the charge stays spent.
 {const c=new Combat();assert.ok(c.cast({skill:0,charge:[4,12,3]}));step(c,38);assert.equal(c.accidents,1);assert.equal(c.hero.hp,960);assert.ok(c.hero.ccUntil>c.tick);assert.ok(c.events.some(e=>e.type==='accident'));c.command('move',{x:200000,y:105000});const x=c.hero.x;step(c,5);assert.equal(c.hero.x,x,'staggered scientist cannot walk');}
 {const c=new Combat();c.hero.hp=1;assert.ok(c.cast({skill:0,charge:[4,12,3]}));step(c,38);assert.equal(c.hero.hp,1,'accidents are never lethal');}
 // Einstein + Newton + Curie in one chain release one strike, then a 6 s cooldown.
 {const c=prepared();c.bank=new (require('../editor/science-charge-model.js').ChargeBank)();c.locked=1;c.aim={x:125000,y:100000};
  const run=()=>{assert.ok(c.cast([3,42,50]),c.events.at(-1)?.text);for(let i=0;i<200&&c.plan;i++)c.step();};
  run();assert.equal(c.resonances,1);assert.ok(c.events.some(e=>e.type==='resonance'&&e.amount===120));assert.ok(c.visuals.some(v=>v.tag==='complete_unified')||c.events.some(e=>e.type==='complete'));
  run();assert.equal(c.resonances,1,'cooldown');step(c,400);run();assert.equal(c.resonances,2);}
 console.log('PASS: round 125 chain payoff (150/180/210), non-lethal lab accident with stagger, Unified resonance with cooldown match native.');
}
assert.deepEqual(unit(1,0),[10000,0]);assert.ok(near({x:0,y:0},{x:100,y:0},{x:50,y:10},10));
console.log('PASS: all 50 combinations, charge/death invariants, cursor/locked release aim, fields, chemistry, manipulation, limits, frozen stats, movement, pause, deterministic mastery errors and ten crowded scientist models.');
