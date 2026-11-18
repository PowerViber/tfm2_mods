const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),out=process.env.CONTENDER_RESULTS||'docs/contender-shadow-review';
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true}),runs=[];try{
 for(let i=0;i<5;i++){const context=await browser.newContext(),page=await context.newPage(),start=performance.now();await page.goto('http://127.0.0.1:7309/editor/contender.html');await page.waitForFunction(()=>window.contenderLab?.me&&!document.getElementById('addTargets').disabled);const cold=performance.now()-start;
 await page.evaluate(()=>contenderLab.addTargets('stock:knight',52,8,true));const actors=await page.evaluate(()=>contenderLab.actors.length);
 const warm=await page.evaluate(()=>{const t=performance.now();for(let i=0;i<300;i++)contenderLab.draw();return (performance.now()-t)/300;});runs.push({cold_local_ready_ms:cold,warm_draw_ms:warm,actors});await context.close();}
 const report={runs,scope:'Five isolated headless Edge local loads; warm draw includes extracted map and 56 actors. Not Proceed/day-advance measurements or actual-game performance.',decoded_bytes:JSON.parse(fs.readFileSync('docs/contender-shadow-review/presentation.json','utf8')).decoded_bytes};fs.writeFileSync(out+'/performance.json',JSON.stringify(report,null,2));console.log(JSON.stringify(report,null,2));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
