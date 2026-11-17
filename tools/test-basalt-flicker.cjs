// The canvas honours shipped frame durations, not the requested cue time.
const {chromium}=require(require.resolve('playwright',{paths:[process.cwd(),require('os').homedir()+'/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules']}));
const fs=require('fs'),assert=require('assert');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true}),page=await browser.newPage({viewport:{width:1440,height:1050}}),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 try{
  await page.goto('http://localhost:7308/editor/basalt.html?build=basalt-art-6');
  await page.waitForFunction(()=>window.basaltLab?.painter.loaded);
  await page.evaluate(()=>basaltLab.pause());await page.click('#ai');await page.uncheck('#hostModel');
  const result=await page.evaluate(()=>{
   const L=basaltLab,P=L.painter,canvas=document.createElement('canvas');canvas.width=canvas.height=160;const c=canvas.getContext('2d');
   const check=(ok,msg)=>{if(!ok)throw Error(msg);};
   const hash=()=>{let h=2166136261;for(const b of c.getImageData(0,0,160,160).data)h=Math.imul(h^b,16777619);return h>>>0;};
   // A legacy one-tick sprite must actually disappear. Holding its final frame
   // indefinitely was the Lab behaviour which concealed the native bug.
   c.clearRect(0,0,160,160);check(P.fx(c,'bs_square',{x:80,y:80},1,0,1,0),'legacy first tick');
   c.clearRect(0,0,160,160);check(!P.fx(c,'bs_square',{x:80,y:80},1,0,1,1),'legacy should expire');
   const empty=hash();let steady=null,frames=0,maxEffects=0;
   L.reset();let pos={...L.actors[0].pos};L.order('s1',{x:pos.x+60000,y:pos.y});
   for(let t=0;t<300;t++){
    L.step();for(const cue of [...L.presentation.events,...L.presentation.leases])check(P.fxTicks(cue.name)===cue.ticks,'packed cue mismatch '+cue.name);
    maxEffects=Math.max(maxEffects,L.effects.length);
    if(t<90)continue;
    const walls=L.effects.filter(e=>/^bs_square(?:_t\d+)?$/.test(e.name));
    check(walls.length===1,'gap/duplicate wall at '+t+': '+walls.length);
    const e=walls[0],age=L.tick-1-e.start;
    for(const fraction of[0,.25,.5,.999]){
     c.clearRect(0,0,160,160);check(P.fx(c,e.name,{x:80,y:80},1,0,1,age+fraction),'missing pixels');
     if(steady===null)steady=hash();check(hash()===steady,'brightness pulse');frames++;
    }
   }
   // Each one-shot traverses all four drawings, then becomes fully transparent.
   let oneShots=0;
   for(const name of['bs_clap_impact','bs_break_t18','bs_land_t18','bs_boulder_impact','bs_meteor_impact']){
    const sequence=BASALT_FX.animations[name.replace(/^bs_/,'')].frames;let elapsed=0;
    for(const frame of sequence){c.clearRect(0,0,160,160);check(P.fx(c,name,{x:80,y:80},1,0,1,elapsed),'missing impact phase');elapsed+=frame.duration*60;}
    c.clearRect(0,0,160,160);check(!P.fx(c,name,{x:80,y:80},1,0,1,P.fxTicks(name))&&hash()===empty,'impact last-frame tail');oneShots++;
   }
   L.reset();L.order('ult',{x:800000,y:160000});let moving=0,impact=0;
   for(let t=0;t<240;t++){
    L.step();const rocks=L.effects.filter(e=>/^bs_boulder(?:_t\d+)?$/.test(e.name));
    check(rocks.length<=1,'stale/duplicate boulder');moving+=rocks.length;
    impact+=L.presentation.events.filter(e=>/^bs_boulder_impact/.test(e.name)).length;
   }
   check(moving>30&&impact===1,'boulder flight or single landing missing');
   L.actors[0].hp=0;for(let i=0;i<8;i++)L.step();check(L.effects.length===0&&L.presentation.leases.length===0,'owner-death cleanup');
   return{steady_wall_subframes:frames,one_shot_sequences:oneShots,maximum_effect_instances:maxEffects,boulder_visible_ticks:moving,boulder_landings:impact,expiry:'exclusive',old_one_tick_gap_is_no_longer_hidden:true};
  });
  assert.deepStrictEqual(errors,[]);
  await page.evaluate(()=>{const L=basaltLab;L.reset();const p=L.actors[0].pos;L.order('s3',{x:p.x+90000,y:p.y});for(let i=0;i<14;i++)L.step();});
  await page.locator('#map').screenshot({path:'docs/basalt-flicker/lab.png'});
  const report={...result,errors,scope:'Packaged art and shared WASM in browser; actual game rendering is separate.'};fs.writeFileSync('docs/basalt-flicker/browser-continuity.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1);});
