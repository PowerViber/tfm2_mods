const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.LAB_URL||'http://localhost:7286')+'/editor/raze.html');await page.waitForFunction(()=>window.RazeLab);
 await page.evaluate(()=>RazeLab.step());
 const before=await page.evaluate(()=>({credits:RazeLab.state.credits,gun:RazeLab.state.gun,balance:document.querySelector('#shop-credits').textContent,icon:document.querySelector('#gun-icon').toDataURL(),pixels:[...document.querySelector('#cash-icon').getContext('2d').getImageData(0,0,32,24).data].filter((v,i)=>i%4===3&&v>0).length}));
 assert.equal(before.credits,1000);assert.equal(before.balance,'$1,000');assert(before.pixels>0);assert.equal(await page.locator('[data-shop-gun]').count(),6);
 await page.locator('#buy').click();await page.waitForFunction(()=>RazeLab.state.credits<1000);
 const after=await page.evaluate(()=>({credits:RazeLab.state.credits,gun:RazeLab.state.gun,spent:RazeLab.state.metrics.spent,balance:document.querySelector('#shop-credits').textContent,name:document.querySelector('#shop-gun').textContent,icon:document.querySelector('#gun-icon').toDataURL(),selected:[...document.querySelectorAll('[data-shop-gun].selected')].map(e=>+e.dataset.shopGun)}));
 assert.equal(after.credits,1000-after.spent);assert.equal(after.gun,1);assert.equal(after.name,'Sheriff');assert.equal(after.balance,'$'+after.credits);assert.deepStrictEqual(after.selected,[after.gun]);assert.notEqual(after.icon,before.icon);
 await page.getByText('Equipment prices',{exact:true}).click();await page.screenshot({path:path.join(__dirname,'../docs/raze-results/raze-shop.png'),fullPage:true});
 await page.evaluate(async()=>{RazeLab.units[0].pos={x:370000,y:300000};RazeLab.units[1].pos={x:375000,y:300000};await RazeLab.step();});assert(await page.locator('#buy').isDisabled());
 const replay=await page.evaluate(()=>RazeLab.verify());assert(replay.identical);
 const report={rulesRevision:'raze-3.1',omenMoneyPixels:true,omenGunPixels:true,gunIconChangesOnPurchase:true,exactCreditAccounting:true,currentGunHighlight:true,safeShopGate:true,replay,errors};assert.deepStrictEqual(errors,[]);fs.writeFileSync(path.join(__dirname,'../docs/raze-results/shop.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
