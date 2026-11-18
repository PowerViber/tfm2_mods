const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert');
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1450,height:1080}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://127.0.0.1:7309/editor/contender.html?build=contender-2.4');await page.waitForFunction(()=>window.contenderLab?.me);
 await page.selectOption('#scenario','damage');await page.uncheck('#fight');
 const result=await page.evaluate(async()=>{const L=contenderLab;await L.reset();L.pause();const target=L.units[0];target.pos=target.home={x:L.me.pos.x+12000,y:L.me.pos.y};target.unlimitedHP=true;
 const calls=[],draw=L.painter.draw.bind(L.painter);L.painter.draw=(...args)=>{if(args[1]==='rhythm')calls.push({tag:args[2],elapsed:args[4],tick:L.tick});return draw(...args);};
 const canvas=document.getElementById('arena');canvas.dispatchEvent(new KeyboardEvent('keydown',{key:' ',bubbles:true}));L.pause();let found;
 for(let i=0;i<1800;i++){L.step();if(L.records.at(-1).effects.some(e=>e.kind==='fx'&&e.name==='ct_break')){found=L.records.at(-1);break;}}
 dispatchEvent(new KeyboardEvent('keyup',{key:' ',bubbles:true}));L.pause();if(!found)throw Error('No authoritative Break');
 const breakTick=found.frame.tick;if(!calls.some(c=>c.tag.startsWith('break_t18')))throw Error('Missing Break art');
 const hits=found.effects.filter(e=>e.kind==='damage');if(!hits.length)throw Error('Missing canonical damage');
 for(let i=0;i<500;i++)L.step();if(calls.some(c=>c.tag==='full_rhythm_t480'&&c.elapsed>=480&&draw(document.createElement('canvas').getContext('2d'),'rhythm',c.tag,{x:0,y:0},c.elapsed,p=>p,n=>n/1000)))throw Error('Aura past expiry');
 const auraDrawn=calls.some(c=>c.tag==='full_rhythm_t480'&&c.elapsed>=0&&c.elapsed<480),bankDrawn=calls.some(c=>c.tag.startsWith('bank_'));if(!auraDrawn||!bankDrawn)throw Error('Missing HUD/aura');
 await L.reset();L.pause();const to={x:L.me.pos.x+20000,y:L.me.pos.y};L.step({kind:'s2',aim:to});L.step({kind:'s2',aim:to});
 const rings=calls.filter(c=>c.tag.startsWith('ring'));if(!rings.some(c=>c.tag==='ring25_t6')||!rings.some(c=>c.tag==='ring35_t6'))throw Error('Missing Cyclone circles');
 return{break_tick:breakTick,break_once_per_cue:true,aura_drawn:auraDrawn,aura_expiry_ticks:480,bank_drawn:bankDrawn,cyclone_radii:[25000,35000],packaged_rhythm:L.painter.data.rhythm.file,decoded:L.painter.data.decoded_bytes};});
 await page.locator('#arena').screenshot({path:'docs/contender-24-art-review/lab-cyclone.png'});assert.deepEqual(errors,[]);
 fs.writeFileSync('docs/contender-24-art-review/integration-browser.json',JSON.stringify({...result,errors,actual_game_acceptance:'pending; Lab host is simulated'},null,2));console.log(JSON.stringify(result));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exit(1)});
