/* Run against an isolated temporary game. Never edits a player's installed mods. */
const {chromium}=require('playwright-core');
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {spawn}=require('node:child_process');
const repo=path.resolve(__dirname,'..'), fixture=fs.mkdtempSync(path.join(os.tmpdir(),'science-editor-'));
const characters=['scribble','levi','isliid','coder','unified_theory'],saved=new Map();
const port=Number(process.env.SCIENCE_EDITOR_PORT||19365),base=`http://127.0.0.1:${port}`;
let server,browser;
(async()=>{
  fs.writeFileSync(path.join(fixture,'bundle.game_data'),'');
  const custom=path.join(fixture,'mods/tfm2_custom'),native=path.join(fixture,'mods/tfm2_custom_ai');
  fs.mkdirSync(custom,{recursive:true});fs.mkdirSync(native,{recursive:true});
  for(const file of ['mod.mod_info','champion','text'])fs.cpSync(path.join(repo,'mods/tfm2_custom',file),path.join(custom,file),{recursive:true});
  fs.copyFileSync(path.join(repo,'mods/tfm2_custom_ai/mod.mod_info'),path.join(native,'mod.mod_info'));
  for(const [i,c] of characters.entries()){
    const value=`W ${i+10}\nG ${100+i} ${i===4?300:15} 12 4\n`;
    fs.writeFileSync(path.join(native,`${c}_memory.txt`),value);saved.set(c,value);
  }
  fs.writeFileSync(path.join(native,'unified_theory_pending.txt'),'g 1.official 104\nr 1.official 104 60 1 0 2 0 0\ng x.fixture 104\nr x.fixture 104 60 1 0 2 0 0\n');
  server=spawn(process.execPath,[path.join(repo,'editor/server.js'),'--no-open','--port',String(port),'--game',fixture],{stdio:'pipe'});
  await new Promise((resolve,reject)=>{let output='';const timer=setTimeout(()=>reject(Error('Editor server did not start: '+output)),10000);server.stdout.on('data',data=>{output+=data;if(output.includes('Database Editor running')){clearTimeout(timer);resolve();}});server.once('error',reject);});
  browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[],badScience=[],requests=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('response',r=>{if(r.status()>=400&&r.url().includes('/science'))badScience.push(r.status()+' '+r.url());});
  page.on('request',r=>{if(r.url().includes('/api/scribble'))requests.push({url:r.url(),method:r.method(),body:r.postData()});});
  await page.goto(base,{waitUntil:'networkidle'});await page.click('#btnScienceLab');
  const frame=page.frameLocator('#stScience iframe');
  await frame.locator('#parity').filter({hasText:'375 NATIVE'}).waitFor();
  await page.waitForFunction(()=>TFM2SkillTest._T.sel?.id==='tfm2_custom_unified_theory');
  assert.equal(await frame.locator('.skill-card').count(),25);
  assert.equal(await page.locator('[data-st="science-art"]').count(),0,'Duplicate viewer tab remains');
  assert.equal(await frame.locator('#artCanvas').isVisible(),false);
  assert.equal(await frame.locator('#arena').isVisible(),true);
  assert.equal(await frame.locator('.il-side #tokens').isVisible(),true);
  const frameWindow=page.frames().find(f=>f.url().includes('embed=1'));
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
  // Real input inside the editor iframe must reach the same practice model.
  const arena=frame.locator('#arena'),arenaBox=await arena.boundingBox();
  await arena.click({button:'right',position:{x:arenaBox.width*.35,y:arenaBox.height*.55}});
  await frameWindow.waitForFunction(()=>ScienceLab.combat.hero.x>100000);
  await arena.press('s');assert.equal(await frameWindow.evaluate(()=>ScienceLab.combat.hero.dest),null);
  await arena.press('Space');await frameWindow.waitForFunction(()=>ScienceLab.combat.plan!==null);
  await arena.press('Escape');assert.equal(await frameWindow.evaluate(()=>ScienceLab.combat.bank.available()),3);
  await frame.locator('#practiceReset').click();
  // The lab shares the actual Coder/Isliid shell, typography and sizing.
  const labStyle=await frameWindow.evaluate(()=>({heading:getComputedStyle(document.querySelector('.il-head h2')).fontSize,width:getComputedStyle(document.querySelector('.il-shell')).maxWidth,font:getComputedStyle(document.body).fontFamily}));
  for(const other of ['code','engraving']){
    await page.click(`[data-st="${other}"]`);
    const otherStyle=await page.locator(other==='code'?'#stCode':'#stEngraving').evaluate(el=>({heading:getComputedStyle(el.querySelector('.il-head h2')).fontSize,width:getComputedStyle(el.querySelector('.il-shell')).maxWidth,font:getComputedStyle(document.body).fontFamily}));
    assert.deepEqual(labStyle,otherStyle,'Lab styling differs from '+other);
  }
  await page.click('[data-st="science"]');
  // Animations and charge testing share a single lab and stop the inactive preview.
  await frame.locator('#tokens input').nth(1).evaluate(el=>{el.value=12;el.dispatchEvent(new Event('input'));});
  assert.equal(await frame.locator('#score').innerText(),'78 / 100');
  await frame.locator('#reset').click();await frame.locator('#add').click();
  await frame.locator('#reserve').click();
  assert.equal(await frame.locator('#sciencePreview').inputValue(),'arena');
  assert.equal(await frame.locator('#arena').isVisible(),true);
  assert.equal(await frame.locator('#artCanvas').isVisible(),false);
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
  await frame.locator('#artRank').selectOption('7');assert.equal(await frame.locator('#artPosition').isDisabled(),false);
  await frame.locator('#artSkill').selectOption('25');assert.match(await frame.locator('#artPosition option[value="1"]').innerText(),/The Unprovable Shape/);
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
  await frame.locator('#artRank').selectOption('0');assert.equal(await frame.locator('#artPosition').isDisabled(),true);
  if(await frame.locator('#cancel').isEnabled())await frame.locator('#cancel').click();
  await frame.locator('#sciencePreview').selectOption('animation');
  await frame.locator('#artStatus').filter({hasText:'Original game sprites'}).waitFor();
  assert.equal(await page.locator('#stScience iframe').count(),1);
  assert.equal(await frame.locator('#artSkill option').count(),75);
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[false,true]);
  for(let skill=0;skill<75;skill++){
    await frame.locator('#artSkill').selectOption(String(skill));
    assert.equal(await frame.locator('#scienceScientist').inputValue(),String(Math.floor(skill/25)));
    assert.match(await frame.locator('#skillId').innerText(),new RegExp('^'+['E','N','C'][Math.floor(skill/25)]));
  }
  await frame.locator('#scienceScientist').selectOption('1');assert.equal(await frame.locator('#artSkill').inputValue(),'25');
  await frame.locator('#artRank').selectOption('7');await frame.locator('#artPosition').selectOption('1');await frame.locator('#artSkill').selectOption('74');
  assert.match(await frame.locator('#artCaption').innerText(),/Unified Mind #1 · Genesis Vessel · Marie Curie/);
  assert.equal(await frame.locator('.plan-row').count(),1,'Notebook was discarded on preview switch');
  assert.match(await frame.locator('#skillId').innerText(),/^C25/);
  for(let recipe=0;recipe<50;recipe++){
    await frame.locator('#combo').selectOption(String(recipe));await frame.locator('#load').click();
    const count=await frame.locator('.plan-row').count();assert.ok(count>=2&&count<=5);
    assert.ok(!(await frame.locator('#planCost').innerText()).includes('NaN'));
  }
  await frame.locator('#compare').click();assert.equal(await frame.locator('#comparison tr').count(),8);
  await frame.locator('.science-transform summary').click();
  await frame.locator('#artFrom').selectOption('2');await frame.locator('#artTo').selectOption('0');await frame.locator('#artTransform').click();
  await page.waitForTimeout(450);assert.equal(await frame.locator('#scienceScientist').inputValue(),'0');
  await frame.locator('#artSkill').selectOption('74');
  await page.click('[data-st="arena"]');
  assert.equal(await page.locator('#stUtScience').inputValue(),'2');assert.equal(await page.locator('#stUtRank').inputValue(),'u1');
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[false,false]);
  // All rank/form and Top 10 appearance changes replace the same three layers.
  for(const rank of [...Array.from({length:7},(_,i)=>String(i)),...Array.from({length:10},(_,i)=>'u'+(i+1))])for(let science=0;science<3;science++){
    await page.selectOption('#stUtRank',rank);await page.selectOption('#stUtScience',String(science));
    const buffs=await page.evaluate(()=>Object.keys(TFM2SkillTest._T.ents.find(e=>e.kind==='hero').buffs));
    assert.equal(buffs.length,3,'Appearance layers stacked: '+buffs);
    const r=rank.startsWith('u')?7:Number(rank),p=rank.startsWith('u')?Number(rank.slice(1)):null;
    assert.deepEqual(buffs.sort(),[`ut_outfit${science}_r${r}`,p?`ut_top${p}`:`ut_rank${r}`,`ut_gearback${science}_r${r}`+(p?`_p${Math.min(p,4)}`:'')].sort());
  }
  assert.equal(await page.locator('[data-science-skill]').count(),25);
  assert.ok(!(await page.locator('#stRight').innerText()).includes('86400'));
  await page.click('[data-st="memory"]');
  await page.locator('#stMemory strong').filter({hasText:'The Unified Theory mastery'}).waitFor();
  assert.equal(await page.locator('[data-games="104"]').inputValue(),'302.3','Pending official/scrim win weight differs from native mastery');
  assert.match(await page.locator('#stMemory').innerText(),/Unified Mind #1/);
  assert.equal(await page.locator('[data-mem="reset-meta"]').count(),0);
  await page.click('[data-mem="seed"]');
  assert.equal(await page.locator('[data-sd="mean"] option').count(),7);
  assert.equal(await page.locator('[data-sd="mean"] option').first().innerText(),'Student');
  assert.equal(await page.locator('[data-sd="top"]').count(),10);
  // Real server mutation goes only to this temporary game's scientist file.
  await page.fill('[data-games="104"]','350');await page.click('[data-mem="set"][data-a="104"]');
  await page.waitForFunction(()=>document.querySelector('[data-games="104"]')?.value==='350.0');
  assert.match(fs.readFileSync(path.join(native,'unified_theory_memory.txt'),'utf8'),/G 104 350.00 12 4/);
  assert.ok(fs.readdirSync(path.join(repo,'editor/backups/unified_theory')).some(n=>n.endsWith('unified_theory_memory.txt')));
  for(const c of characters.slice(0,4)){
    assert.equal(fs.readFileSync(path.join(native,`${c}_memory.txt`),'utf8'),saved.get(c),c+' memory was changed');
    await page.click(`[data-mem="char"][data-c="${c}"]`);
    await page.locator('#stMemory strong').filter({hasText:({scribble:'Scribble',levi:'Levi',isliid:'Isliid',coder:'Coder'})[c]+' mastery'}).waitFor();
  }
  await page.click('[data-mem="char"][data-c="unified_theory"]');
  await page.locator('#stMemory strong').filter({hasText:'The Unified Theory mastery'}).waitFor();
  // Exercise the shared generator with fixture athletes, including a selected #1.
  await page.evaluate(()=>{window.TFM2_APP={athleteName:id=>'Test athlete '+id,athletes:()=>Array.from({length:14},(_,i)=>({id:110+i,name:'Test athlete '+(110+i)}))};});
  await page.click('[data-mem="seed"]');await page.fill('[data-sd="top"][data-i="0"]','110');
  page.once('dialog',dialog=>dialog.accept());await page.click('[data-mem="sd-roll"]');
  await page.waitForFunction(()=>document.querySelector('[data-games="110"]')?.value==='400.0');
  assert.match(await page.locator('#stMemory').innerText(),/Test athlete 110/);
  assert.match(await page.locator('#stMemory').innerText(),/Unified Mind #1/);
  for(const c of characters.slice(0,4))assert.equal(fs.readFileSync(path.join(native,`${c}_memory.txt`),'utf8'),saved.get(c));
  const posts=requests.filter(r=>r.method==='POST');assert.equal(posts.length,2);assert.ok(posts.every(r=>r.url.endsWith('char=unified_theory')));
  // Skill Lab must expose actual native recipes, preserve its passive, and link a selected effect.
  await page.click('.tab[data-tab="skills"]');
  await page.locator('[data-slsel]').filter({hasText:'The Unified Theory'}).click();
  assert.equal(await page.locator('#slScienceViewer').count(),0,'Duplicate viewer button remains');
  assert.equal(await page.locator('[data-passive]').inputValue(),'tfm2_custom_ai:unified_theory');
  assert.equal(await page.locator('[data-science-view]').count(),75);
  assert.equal(await page.locator('[data-slot]').count(),1);
  await page.click('[data-science-view="49"]');
  await frame.locator('#artSkill').filter({visible:true}).waitFor();
  await page.waitForFunction(()=>TFM2SkillTest._T.scienceSkill===49);
  assert.equal(await frame.locator('#artSkill').inputValue(),'49');
  assert.match(await frame.locator('#artCaption').innerText(),/Newton/);
  await frame.locator('#artRank').selectOption('7');await frame.locator('#artPosition').selectOption('1');
  await page.waitForLoadState('networkidle');
  await frameWindow.evaluate(()=>document.querySelectorAll('details').forEach(el=>el.open=false));
  await page.screenshot({path:path.join(repo,'docs/unified-theory-science-lab-ui.png')});
  // User pause survives hide/show; hidden views never keep their RAF loops alive.
  await frame.locator('#artPause').click();
  await page.click('.tab[data-tab="skills"]');
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[false,false]);
  await page.click('.tab[data-tab="test"]');
  assert.equal(await frame.locator('#artPause').innerText(),'Play animation');
  await frame.locator('#artPause').click();
  await frame.locator('#artSkill').selectOption('0');await frame.locator('#preview').click();
  assert.deepEqual(await frameWindow.evaluate(()=>[ScienceLab.animationRunning,ScienceArt.animationRunning]),[true,false]);
  await page.screenshot({path:path.join(repo,'docs/unified-theory-science-lab-study.png')});
  for(const view of ['science','memory']){
    await page.click(`[data-st="${view}"]`);await page.setViewportSize({width:390,height:844});await page.waitForTimeout(80);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'Editor overflows on mobile: '+view);
    if(view!=='memory')assert.ok(await frameWindow.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'Science frame overflows on mobile: '+view);
    await page.setViewportSize({width:1440,height:1100});
  }
  await page.click('[data-st="science"]');await page.setViewportSize({width:390,height:844});
  for(const mode of ['animation','arena']){await frame.locator('#sciencePreview').selectOption(mode);assert.ok(await frameWindow.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));}
  await page.screenshot({path:path.join(os.tmpdir(),'science-lab-unified-mobile.png')});
  assert.deepEqual(errors,[]);assert.deepEqual(badScience,[]);
  console.log('PASS: one Science Lab with no duplicate viewer, shared Coder/Isliid styling, 75 native recipes and animations, all 50 experiments, 375 charge vectors, 51 rank/form appearances without stacking, one persistent iframe, hidden animation suspension, preserved notebook/pause, all five mastery lists, isolated scientist writes/backups and mastery generation with a pinned #1, correct pending win points, desktop/mobile layout.');
})().catch(e=>{console.error(e);process.exitCode=1;}).finally(async()=>{if(browser)await browser.close();if(server)server.kill();if(path.dirname(fixture)!==path.resolve(os.tmpdir())||!path.basename(fixture).startsWith('science-editor-'))throw Error('Unsafe fixture cleanup');fs.rmSync(fixture,{recursive:true,force:true});});
