const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),cp=require('child_process');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),out=path.join(root,'docs/creative-results');fs.mkdirSync(out,{recursive:true});
(async()=>{const browser=await chromium.launch({channel:'msedge',headless:true});try{
 const page=await browser.newPage({viewport:{width:1560,height:1100}}),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto((process.env.LAB_URL||'http://localhost:7284')+'/universal.html');await page.waitForFunction(()=>window.UniversalLab);
 const scenarios=await page.evaluate(()=>{const result=[],fixtures=[...document.querySelectorAll('#fixture option')].map(o=>o.value);
  for(let n=0;n<36;n++)for(let team=0;team<2;team++){
   document.querySelector('#team').value=team;document.querySelector('#fixture').value=fixtures[n%fixtures.length];UniversalLab.reset();
   for(const a of UniversalLab.actors){a.x=Math.max(1000,Math.min(959000,a.x+(n*137+a.id*67)%20000));a.y=Math.max(1000,Math.min(959000,a.y+(n*317)%15000));}
   for(let step=0;step<20;step++){for(const a of UniversalLab.actors){a.x=Math.min(959000,a.x+step*3);}UniversalLab.evaluate();}
   result.push({seed:n+1,team,set:n<24?'development':'holdout',fixture:fixtures[n%fixtures.length],frames:JSON.parse(JSON.stringify(UniversalLab.history)),selected:UniversalLab.snapshot.plans[0]});
  }return result;});
 let parity=0;for(const scenario of scenarios){const input=scenario.frames.map(f=>'0 '+f.input.join(' ')).join('\n')+'\n';const p=cp.spawnSync('rtk',['proxy',path.join(root,'native/universal_lab/target/release/universal-vectors.exe')],{input,encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(p.status,0,p.stderr);const lines=p.stdout.trim().split(/\r?\n/);assert.equal(lines.length,scenario.frames.length);lines.forEach((line,n)=>{assert.deepStrictEqual(JSON.parse(line),scenario.frames[n].output);parity++;});}
 const differences=await page.evaluate(()=>{
  const results=[];const fixtures=[...document.querySelectorAll('#fixture option')].map(o=>o.value);
  for(let k=0;k<4;k++){let found=null;
   for(const fixture of fixtures){for(let valueBias=0;valueBias<50;valueBias++){
    document.querySelector('#fixture').value=fixture;document.querySelector('#team').value=0;UniversalLab.reset();for(const a of UniversalLab.actors)if(a.team===0)a.ratings=[50,50,50,50];
    const objective=UniversalLab.targets.find(t=>t.kind===2||t.kind===3);if(objective)objective.value+=valueBias*2;
    const actor=UniversalLab.actors.find(a=>a.team===0&&(k!==1||a.id===3));actor.ratings[k]=0;const low=UniversalLab.evaluate();actor.ratings[k]=100;const high=UniversalLab.evaluate();
    const decision=o=>JSON.stringify(o.plans.slice(0,1).map(p=>({caller:p.caller,goal:p.goal,target:p.target,jobs:p.jobs})));
    if(decision(low)!==decision(high)){found={attribute:['Macro','Micro','Map Awareness','Strategy'][k],fixture,valueBias,low:low.plans,high:high.plans};break;}
   }if(found)break;}results.push(found);
  }return results;});assert(differences.every(Boolean),JSON.stringify(differences));
 const learning=await page.evaluate(()=>{const frame=UniversalLab.frame(),h=999999;const fresh=UniversalLab.invoke(h,frame); // bad sessions must reject, never alias another athlete
  const before=UniversalLab.evaluate();document.querySelector('#day').value=20400101;const a=UniversalLab.evaluate(2),repeat=UniversalLab.evaluate(2);document.querySelector('#set').value=555;const set=UniversalLab.evaluate(3),duplicate=UniversalLab.evaluate(3);return{fresh,before:before.development,a:a.development,repeat:repeat.development,set:set.development,duplicate:duplicate.development,replayed:UniversalLab.verify()};});
 assert(learning.fresh.error);assert(learning.a.changed);assert(!learning.repeat.changed);assert(learning.set.changed);assert(!learning.duplicate.changed);
 const response=await(await page.request.get((process.env.LAB_URL||'http://localhost:7284')+'/api/game-sense')).json();assert(response.profiles||response.message);
 await page.screenshot({path:path.join(out,'universal-lab.png'),fullPage:true});assert.deepStrictEqual(errors,[]);
 const report={scenarios:scenarios.map(({frames,...s})=>s),parityFrames:parity,independentAttributeDecisions:differences,learning,errors,stockCastsObserved:false,liveAcceptance:'Pending game observations; strategy and route fixtures cannot establish stock cast delivery.'};fs.writeFileSync(path.join(out,'universal-lab.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({scenarios:scenarios.length,parityFrames:parity,attributes:differences.map(d=>d.attribute),trainingIdempotence:true,resultIdempotence:true,errors}));
 }finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
