const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
(async () => {
 const browser = await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
 try {
 const page = await browser.newPage({viewport:{width:1440,height:1000}});
 const errors=[]; page.on('pageerror',e=>errors.push(e.message));
 await page.goto(process.env.DB_EDITOR_URL||'http://127.0.0.1:7272/',{waitUntil:'networkidle'});
 await page.click('#btnSkillTest'); await page.click('[data-st="code"]');
 assert.equal(await page.locator('.il-chip').innerText(),'NATIVE PARITY');
 await page.click('[data-cl="funcs"]');
 assert.equal(await page.locator('#clFuncs tr[data-f]').count(),100);
 for (const name of ['kubernetes','regex','zero_day']) {
  const f=await page.evaluate(n=>window.TFM2_CODER.FUNCS.findIndex(f=>f.name===n),name);
  await page.click('#clFuncs tr[data-f="'+f+'"]');
  assert.equal(await page.locator('#clFunc').inputValue(),String(f));
  const options=await page.locator('#clLang option').evaluateAll(o=>o.map(x=>Number(x.value)).filter(x=>x>=0));
  const expected=await page.evaluate(f=>window.TFM2_CODER.LANGS.map((l,i)=>window.TFM2_CODER.FUNCS[f].lens[l]?.length?i:-1).filter(i=>i>=0),f);
  assert.deepEqual(options,expected);
  await page.click('[data-cl="again"]');
 }
 for (const rank of [0,3,6,7]) {
  await page.selectOption('#clRank',String(rank));
  if(rank===7) await page.selectOption('#clRoot','1');
  await page.click('[data-cl="arena"]');
  const arenaText=await page.locator('#clResult').innerText();
  assert.ok(arenaText.includes('30 s arena'));
  assert.ok(!arenaText.includes('[object Object]'));
  assert.match(arenaText,/ran \d+ times/);
 }
 await page.selectOption('#clRuns','10'); await page.click('[data-cl="compare"]');
 await page.waitForFunction(()=>!window.TFM2CoderLab._state.busy,null,{timeout:60000});
 const result=await page.evaluate(()=>window.TFM2CoderLab._state.rows.map(r=>({rank:r.label,shipped:r.shipped,distinct:r.distinct,dps:r.dps})));
 assert.equal(result.length,9); assert.ok(result.every(r=>Object.values(r).every(v=>typeof v!=='number'||Number.isFinite(v))));
 assert.ok(!(await page.locator('#clTable').innerText()).includes('NaN'));
 assert.deepEqual(errors,[]);
 for(let i=1;i<result.length;i++){assert.ok(result[i].shipped>=result[i-1].shipped);assert.ok(result[i].distinct>=result[i-1].distinct);}
 console.log('PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.');
 console.log(JSON.stringify(result));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
