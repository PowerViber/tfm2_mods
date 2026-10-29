const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 try{
  const page=await browser.newPage({viewport:{width:1200,height:900},deviceScaleFactor:1});const errors=[];
  page.on('pageerror',e=>errors.push(String(e)));
  const response=await page.goto('http://localhost:7290/editor/original-body-review.html');assert.equal(response.status(),200);
  await page.evaluate(()=>Promise.all(Array.from(document.images,img=>img.decode())));
  const result=await page.evaluate(()=>({title:document.title,sections:document.querySelectorAll('section').length,images:document.images.length,broken:Array.from(document.images).filter(i=>!i.complete||!i.naturalWidth).length,animations:Array.from(document.images).filter(i=>i.src.endsWith('-complete.gif')).length,facings:Array.from(document.images).filter(i=>i.style.transform==='scaleX(-1)').length}));
  assert.equal(result.sections,10);assert.equal(result.broken,0);assert(result.animations>=70);assert.equal(errors.length,0);assert(result.facings>=160);
  await page.screenshot({path:path.join('docs/original-body-review','browser-gallery.png')});
  result.errors=errors;result.scope='Headless browser gallery, image decoding and 1x/3x CSS; not live game acceptance.';
  fs.writeFileSync('docs/original-body-review/browser.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
