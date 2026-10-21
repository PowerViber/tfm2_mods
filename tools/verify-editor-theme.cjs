/* Editor theme check in a real browser: every page and Database Editor view loads without page errors,
   has no horizontal page scroll at phone width, renders in both the light and dark theme, and keeps
   few visible dropdowns (the rest are enhanced into segmented controls, chips and searchable pickers
   that drive the original <select>). `--count` only reports the dropdown counts. */
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),cp=require('node:child_process'),assert=require('node:assert/strict');
const {chromium}=require('playwright');
const root=path.resolve(__dirname,'..'),port=19447,base='http://127.0.0.1:'+port,countOnly=process.argv.includes('--count');
const PAGES=['athlete.html','frieren.html','gojo.html','gojo-v1.html','gojo-v2.html','gojo-v3.html','magnus.html','steve.html','steve-v1.html','unified.html','universal.html','vesper.html'];
// Database Editor views reachable without opening a save: [name, button ids clicked in order].
const VIEWS=[['welcome',[]],['skill-lab',['#btnSkillLab']],['skill-lab-champion',['#btnSkillLab','#slNewChamp']],['skill-test',['#btnSkillTest']],['map',['#btnMapEd']],
 ['equation-lab',['#btnScienceLab']],['rift-lab',['#btnRiftLab']],['chess-lab',['#btnChessLab']],['gojo-lab',['#btnGojoLab']],['game-sense',['#btnUniversalLab']],['stevecraft',['#btnSteveLab']]];
const MAX_VISIBLE=25;
let server,browser;
const count=page=>Promise.all(page.frames().map(f=>f.evaluate(()=>{const all=[...document.querySelectorAll('select')];
 const shown=all.filter(s=>{const r=s.getBoundingClientRect(),c=getComputedStyle(s);return r.width>4&&r.height>4&&c.visibility!=='hidden'&&+c.opacity>.05&&s.getClientRects().length;});
 const kinds={};for(const e of document.querySelectorAll('select[data-ui-enhanced]'))kinds[e.dataset.uiEnhanced]=(kinds[e.dataset.uiEnhanced]||0)+1;
 return {total:all.length,visible:shown.length,enhanced:document.querySelectorAll('[data-ui-enhanced]').length,kinds,ids:shown.map(s=>s.id||s.className||s.name||'?')};}).catch(()=>({total:0,visible:0,enhanced:0,kinds:{},ids:[]}))))
 .then(rs=>rs.reduce((a,r)=>{for(const k in r.kinds)a.kinds[k]=(a.kinds[k]||0)+r.kinds[k];return {...a,total:a.total+r.total,visible:a.visible+r.visible,enhanced:a.enhanced+r.enhanced,ids:a.ids.concat(r.ids)};},{total:0,visible:0,enhanced:0,kinds:{},ids:[]}));
async function open(ctx,url,clicks){const page=await ctx.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('dialog',d=>d.accept(d.defaultValue()||'Test'));
 await page.goto(base+'/'+url,{waitUntil:'load'});await page.waitForTimeout(400);
 for(const sel of clicks){await page.waitForSelector(sel,{state:'attached',timeout:8000});await page.click(sel);await page.waitForTimeout(500);}
 await page.waitForTimeout(300);return {page,errors};}
(async()=>{
 const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'tfm2-editor-theme-'));fs.writeFileSync(path.join(fixture,'bundle.game_data'),'');fs.mkdirSync(path.join(fixture,'mods'));
 server=cp.spawn(process.execPath,[path.join(root,'editor/server.js'),'--no-open','--port',String(port),'--game',fixture],{stdio:'pipe'});let boot='';server.stdout.on('data',d=>boot+=d);server.stderr.on('data',d=>boot+=d);
 for(let n=0;n<100;n++){try{if((await fetch(base+'/index.html')).ok)break;}catch{}await new Promise(r=>setTimeout(r,100));if(n===99)throw Error(boot);}
 browser=await chromium.launch(process.platform==='win32'?{channel:'msedge',headless:true}:{headless:true});
 const targets=[...VIEWS.map(([name,clicks])=>({name:'index:'+name,url:'index.html',clicks})),...PAGES.map(p=>({name:p,url:p,clicks:[]}))];
 const report={pages:[],totals:{selects:0,visible:0},errors:[]};
 const desk=await browser.newContext({viewport:{width:1280,height:900}});
 for(const t of targets){const {page,errors}=await open(desk,t.url,t.clicks);const c=await count(page);
  report.pages.push({name:t.name,...c});report.totals.selects+=c.total;report.totals.visible+=c.visible;for(const k in c.kinds)report.totals[k]=(report.totals[k]||0)+c.kinds[k];
  if(errors.length)report.errors.push({name:t.name,errors});
  if(!countOnly&&c.enhanced){// the enhanced control still drives the original select
   const ok=await page.evaluate(()=>{const s=[...document.querySelectorAll('select[data-ui-enhanced]')].find(s=>s.options.length>1&&!s.disabled&&s.__ui);
    if(!s)return true;let fired=0;s.addEventListener('change',()=>fired++,{once:true});const ui=s.__ui;const want=[...s.options].find(o=>!o.selected&&!o.disabled);if(!want||!ui)return true;
    ui.choose(want.value);return fired===1&&ui.root.isConnected&&ui.root.querySelector('[aria-checked="true"],.ui-pick-btn')!==null;});
   assert.ok(ok,t.name+': enhanced control drives its select');}
  await page.close();}
 await desk.close();
 if(!countOnly){
  for(const theme of ['light','dark']){const ctx=await browser.newContext({viewport:{width:390,height:844},colorScheme:theme==='light'?'dark':'light'});
   await ctx.addInitScript(th=>{try{localStorage.setItem('tfm2-theme',th);}catch{}},theme);
   for(const t of targets){const {page,errors}=await open(ctx,t.url,t.clicks);
    const m=await page.evaluate(()=>{const de=document.documentElement,bg=getComputedStyle(document.body).backgroundColor.match(/[\d.]+/g).map(Number);
     const lum=(bg[3]===0)?null:(.2126*bg[0]+.7152*bg[1]+.0722*bg[2])/255;
     return {theme:de.dataset.theme,scheme:getComputedStyle(de).colorScheme,overflow:de.scrollWidth-innerWidth,lum,toggle:!!document.querySelector('.ui-theme-toggle')};});
    if(errors.length)report.errors.push({name:t.name+'@390/'+theme,errors});
    assert.equal(m.theme,theme,t.name+' theme attribute');assert.ok(m.scheme.includes(theme),t.name+' color-scheme '+m.scheme);
    assert.ok(m.overflow<=1,`${t.name} at 390 px (${theme}) scrolls sideways by ${m.overflow}px`);assert.ok(m.toggle,t.name+' has a theme toggle');
    if(m.lum!==null)assert.ok(theme==='light'?m.lum>.6:m.lum<.35,`${t.name} ${theme} background luminance ${m.lum.toFixed(2)}`);
    report.pages.find(p=>p.name===t.name)[theme]={overflow:m.overflow,lum:m.lum&&+m.lum.toFixed(3)};await page.close();}
   await ctx.close();}
  console.log(JSON.stringify(report,(k,v)=>k==='ids'&&!v.length?undefined:v,1));
  assert.deepEqual(report.errors,[],'page errors');
  assert.ok(report.totals.visible<MAX_VISIBLE,`${report.totals.visible} visible dropdowns (limit ${MAX_VISIBLE})`);
 }
 if(countOnly)console.log(JSON.stringify(report,(k,v)=>k==='ids'&&!v.length?undefined:v,1));
 else console.log(`PASS: ${targets.length} pages and views, light and dark at 390 px, no page errors; ${report.totals.visible} of ${report.totals.selects} dropdowns left visible (${report.totals.seg||0} segmented, ${report.totals.chips||0} chip groups, ${report.totals.rank||0} rank ladders, ${report.totals.pick||0} searchable pickers).`);
})().catch(e=>{console.error(e);process.exitCode=1;}).finally(async()=>{await browser?.close();server?.kill();});
