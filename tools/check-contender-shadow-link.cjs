/* Matched fixed-snapshot scenarios and native/WASM parity for the 3.0 rules. */
const assert = require('node:assert/strict');
const {spawnSync} = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const baseline = '5e56bfc67c4919d145b0c8b640be6b11d1974813';
const pos = {x:400000,y:500000};
const aim = {x:418000,y:500000};
const home = {x:376000,y:500000};

async function wasm(buffer, revision) {
  const {instance:{exports:api}} = await WebAssembly.instantiate(buffer);
  return {
    revision,
    reset: () => api.contender_reset(),
    step: frame => {
      const bytes = Buffer.from(JSON.stringify({version:1,rules_revision:revision,frame}));
      const ptr = api.contender_input(bytes.length);
      new Uint8Array(api.memory.buffer,ptr,bytes.length).set(bytes);
      const out = api.contender_step(bytes.length);
      const value = JSON.parse(Buffer.from(api.memory.buffer,out,api.contender_output_len()).toString());
      assert.ok(!value.error, value.error);
      return value;
    },
  };
}

function unit(id, x, y=pos.y, team=1, hp=2000, tower=false) {
  return {id, team, pos:{x,y}, hp, max_hp:2000, radius:3000, champion:!tower, tower, visible:true, targetable:true};
}

function baseFrame(dt, units) {
  return {
    tick:1000+dt, owner:1, team:0, alive:true, pos, home, hp:2000, max_hp:2000, attack:161,
    level:9, ult_unlocked:true, units, profile:Array(13).fill(50), manual:true, order:null, basic_hits:[],
    shadow_link_enabled:true,
  };
}

function compareFight(engine, n, profile) {
  engine.reset();
  const units = Array.from({length:n}, (_,i)=>unit(i+2,aim.x+i*1500,aim.y+i*1000));
  const results = [], frames = [];
  let receipt=null, cast=null, stored=0;
  for(let dt=0;dt<265;dt++){
    const frame=baseFrame(dt,units);
    frame.profile[6]=profile;
    if(dt===0)frame.order={kind:'s3',aim};
    if(dt===1)frame.order={kind:'s2',aim};
    if(receipt){
      frame.shadow_observation=receipt;
      receipt=null;
    }else if(cast!==null){
      // Same simulated pressure timeline. In 3.0 the ally aims at the carrier.
      // Ally damage is not included in Contender's direct damage totals.
      stored=Math.min(1500,stored+((dt%36===0)?250:0));
      frame.shadow_observation={kind:'health',cast,hp:2000-stored};
    }
    const result=engine.step(frame);
    results.push(result);
    frames.push(frame);
    const landed=result.effects.find(effect=>effect.kind==='shadow_land');
    if(landed?.candidates.length){
      cast=landed.cast;
      receipt={kind:'landed',cast,targets:landed.candidates.map(target=>target.id)};
    }
    if(result.effects.some(effect=>effect.kind==='shadow_end')){
      cast=null;
    }
  }
  const effects=results.flatMap(result=>result.effects);
  const packets=effects.filter(effect=>['damage','shadow_payout'].includes(effect.kind));
  const payout=packets.filter(effect=>effect.kind==='shadow_payout');
  const directDamage=packets.filter(effect=>effect.kind==='damage').reduce((sum,effect)=>sum+effect.amount,0);
  const visuals=effects.filter(effect=>effect.kind==='fx' || effect.kind==='shadow_land' || effect.kind==='shadow_end').length;
  return {
    frames, results,
    summary:{targets:n, profile, directDamage, bonusDamage:payout.reduce((sum,effect)=>sum+effect.amount,0),
      packets:packets.length, visuals, payoutPackets:payout.length,
      carriers:effects.filter(effect=>effect.kind==='shadow_land').length,
      peakVisualsInTick:Math.max(...results.map(result=>result.effects.filter(effect=>effect.kind==='fx' || effect.kind.startsWith('shadow_')).length))},
  };
}

function compareAi(engine, profile) {
  const run=(frames)=>{engine.reset();return frames.map(frame=>engine.step(frame));};
  const far=baseFrame(0,[unit(2,550000),unit(3,551000)]);
  far.manual=false;far.hp=200;far.profile[6]=profile;
  const first=run([far])[0];
  const init=baseFrame(0,[unit(2,450000)]);init.order={kind:'s2',aim:{x:449000,y:500000}};
  const underTower=baseFrame(1,[unit(2,450000),unit(3,452000),unit(4,455000),unit(5,460000,509000,1,2000,true)]);
  underTower.manual=false;underTower.hp=200;underTower.profile[6]=profile;
  const second=run([init,underTower])[1];
  const supported=baseFrame(0,[unit(2,440000),unit(3,444000),unit(4,428000,500000,0)]);
  supported.shadow_link_enabled=false;supported.manual=false;supported.profile[6]=profile;
  const third=run([supported])[0];
  return {profile,lowHpApproach:first.effects.some(effect=>effect.kind==='steer'&&effect.to.x>pos.x),
    towerGlide:second.effects.some(effect=>effect.kind==='glide'),supported:third.reason};
}

(async()=>{
  const oldBytes=spawnSync('rtk',['proxy','git','show',baseline+':editor/contender.wasm'],{maxBuffer:4*1024*1024});
  assert.equal(oldBytes.status,0,oldBytes.stderr?.toString());
  const old=await wasm(oldBytes.stdout,'contender-2.9');
  const current=await wasm(fs.readFileSync('editor/contender.wasm'),'contender-3.0');
  const trace=path.resolve('native/contender_shadow_lab/target/release/trace.exe');
  const matches=[], ai=[];
  for(const n of [1,3,5]){
    for(const profile of [20,80]){
      const now=compareFight(current,n,profile);
      const before=compareFight(old,n,profile).summary;
      assert.equal(now.summary.carriers,1);
      assert.equal(now.summary.payoutPackets,n);
      assert.ok(now.summary.visuals<before.visuals || n===1);
      const input=now.frames.map(frame=>JSON.stringify({version:1,rules_revision:current.revision,frame})).join('\n')+'\n';
      const native=spawnSync(trace,[],{input,encoding:'utf8',maxBuffer:16*1024*1024});
      assert.equal(native.status,0,native.stderr);
      assert.deepEqual(native.stdout.trim().split(/\r?\n/).map(JSON.parse),now.results);
      matches.push({before,after:now.summary});
    }
  }
  for(const profile of [20,80]){
    const before=compareAi(old,profile),after=compareAi(current,profile);
    assert.equal(before.lowHpApproach,true);
    assert.equal(before.towerGlide,true);
    assert.equal(after.lowHpApproach,false);
    assert.equal(after.towerGlide,false);
    assert.match(after.supported,/supported Shadow Flurry/);
    ai.push({before,after});
  }
  console.log(JSON.stringify({baseline,parityFrames:6*265,matches,ai,scope:'Fixed-position shared-core measurements and transport parity. No actual-game frame-time measurements.'},null,2));
})().catch(error=>{console.error(error);process.exitCode=1;});
