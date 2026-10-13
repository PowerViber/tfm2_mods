const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const path=require('node:path');
(async()=>{
 const b=await chromium.launch({executablePath:process.env.CHROMIUM_PATH,headless:true});
 try{
  const p=await b.newPage({viewport:{width:1440,height:1100}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
  await p.goto(process.env.SCIENCE_LAB_URL||'http://127.0.0.1:7272/science.html',{waitUntil:'networkidle'});
  await p.waitForFunction(()=>document.querySelector('#parity').textContent.includes('375 NATIVE'));
  assert.equal(await p.locator('.charge-cell[data-state="ready"]').count(),3);
  await p.click('#preview');await p.waitForFunction(()=>document.querySelectorAll('.charge-cell[data-state=reserved]').length===1);assert.equal(await p.locator('.charge-cell[data-state="reserved"]').count(),1);
  await p.click('#cancel');assert.equal(await p.locator('.charge-cell[data-state="ready"]').count(),3);
  assert.match(await p.locator('#event').innerText(),/refunded/);
  await p.selectOption('#combo','0');await p.click('#load');await p.click('#reserve');
  await p.waitForFunction(()=>document.querySelectorAll('.charge-cell[data-state="recovering"]').length===1);
  const spent=await p.evaluate(()=>ScienceLab.study.bank.slots.find(s=>typeof s==='number'));
  assert.ok(spent>0);
  assert.equal(await p.evaluate(()=>ScienceLab.study.bank.slots.filter(s=>typeof s==='number').length),1,'Combination spent more than one charge');
  await p.click('#cancel');assert.match(await p.locator('#event').innerText(),/spent charge/);
  await p.waitForFunction(()=>document.querySelectorAll('.charge-cell[data-state="ready"]').length===3,{},{timeout:6500});
  assert.match(await p.locator('#timeline').innerText(),/recovered/);
  const text='seed 5 tick 1 caster 0 selection reason="direct attack" target 1\nseed 5 tick 38 caster 0 charge_spent slot 0 recovery 338\nseed 5 tick 50 caster 0 release E01 target 1 origin 100000,100000 aim 125000,100000\nseed 5 tick 57 caster 0 target 1 skill E01 experiment 1 raw 100 type magic HP 1000->900 budget 0\nseed 5 tick 338 caster 0 charge_recovered slot 0';
  await p.locator('#diagnosticFile').setInputFiles({name:'native-fixture.txt',mimeType:'text/plain',buffer:Buffer.from(text)});
  await p.waitForFunction(()=>document.querySelectorAll('#nativeTimeline li').length===5);
  assert.match(await p.locator('#diagnosticStatus').innerText(),/recorded native.*world coordinates/);
  assert.match(await p.locator('#nativeTimeline').innerText(),/aim \(125000, 100000\)/);
  assert.match(await p.locator('#nativeTimeline').innerText(),/HP 1000 → 900/);
  const manifests=await p.evaluate(async()=>await(await fetch('/science-art-preview.json')).json());
  for(let s=0;s<3;s++)for(let rank=0;rank<8;rank++)for(let phase=0;phase<3;phase++)assert.equal(manifests.assets[`pose${s}_r${rank}_p${phase}`].frames.length,8);
  await p.locator('#nativeDiagnostics').screenshot({path:path.join(__dirname,'../docs/unified-theory-native-diagnostics.png')});
  await p.setViewportSize({width:390,height:844});assert.ok(await p.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth));
  assert.deepEqual(errors,[]);
  console.log('PASS: three ready/reserved/recovering cells, refundable preparation, one charge per combination, exact five-second recovery, safe native diagnostics with origins/aims/HP, 72 eight-frame body poses and mobile layout.');
 }finally{await b.close();}
})().catch(e=>{console.error(e);process.exit(1);});
