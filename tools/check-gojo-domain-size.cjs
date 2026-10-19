/* Verify rendered cosmic pixels fill the gameplay radius; combat outputs must stay identical. */
const fs=require('fs'),path=require('path'),os=require('os'),cp=require('child_process'),assert=require('assert');
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/gojo-results');
const original=cp.execFileSync('rtk',['proxy','git','show','c98540f71fd8334502fb3f2086b844343841522f:editor/gojo-lab.js'],{cwd:root,encoding:'utf8'});
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true}),rows=[];
 for(const [mode,action,steps] of [['full',8,100],['quick',9,16]]){
  const runs=[];
  for(const before of [true,false]){
   const page=await browser.newPage({viewport:{width:1360,height:1000}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
   if(before)await page.route('**/gojo-lab.js',r=>r.fulfill({body:original,contentType:'text/javascript'}));
   await page.goto('http://localhost:7281/gojo.html');await page.waitForFunction(()=>window.GojoLab);
   const result=await page.evaluate(({action,steps})=>{
    GojoLab.pause();GojoLab.reset();GojoLab.submit(action);for(let i=0;i<steps;i++)GojoLab.step();
    const s=GojoLab.snapshot,[x,y,,,,,,radius]=s.domain,c=document.querySelector('#arena'),g=c.getContext('2d'),pixels=g.getImageData(0,0,c.width,c.height).data;
    let dark=0,samples=0;
    for(const fraction of [.65,.8])for(let i=0;i<32;i++){
     const angle=i*Math.PI/16,col=Math.round((x+Math.cos(angle)*radius*fraction)/300),row=Math.round((y+Math.sin(angle)*radius*fraction)/300);
     if(col<0||row<0||col>=c.width||row>=c.height)continue;
     const at=(row*c.width+col)*4,r=pixels[at],g=pixels[at+1],b=pixels[at+2];samples++;
     if(pixels[at+3]>=180&&((r<15&&g<25&&b<40)||(r>g*1.15&&b>r*1.3)))dark++;
    }
    return {core:s.core,radius,dark,samples};
   },{action,steps});
   assert.deepEqual(errors,[]);runs.push(result);
   if(!before)await page.locator('#arena').screenshot({path:path.join(out,`domain-${mode}-sized.png`)});
   await page.close();
  }
  assert.equal(runs[0].core,runs[1].core,'Resizing must not change combat state');
  assert(runs[1].dark>=runs[1].samples*.7,`${mode} cosmic interior does not fill its radius`);
  assert(runs[1].dark>runs[0].dark+20,`${mode} is still drawn at its old fixed pixel size`);
  rows.push({mode,radius:runs[1].radius,beforeCosmicSamples:runs[0].dark,afterCosmicSamples:runs[1].dark,samples:runs[1].samples,combatUnchanged:true});
 }
 await browser.close();fs.writeFileSync(path.join(out,'domain-size.json'),JSON.stringify({scope:'Browser pixels and unchanged core state, not live game acceptance.',rows},null,2)+'\n');console.log(JSON.stringify(rows));
})().catch(e=>{console.error(e);process.exit(1)});
