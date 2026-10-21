const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert'),cp=require('child_process');
const{chromium}=require(require.resolve('playwright',{paths:[process.cwd(),path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules')]}));
const root=path.resolve(__dirname,'..'),base=process.env.LAB_URL||'http://localhost:7284';
(async()=>{const b=await chromium.launch({channel:'msedge',headless:true});try{
 const p=await b.newPage();await p.goto(base+'/steve-v1.html');await p.waitForFunction(()=>window.SteveLab);const frames=await p.evaluate(()=>{SteveLab.demo(3);SteveLab.pause();for(let n=0;n<140;n++)SteveLab.step();return SteveLab.recording;});
 await p.goto(base+'/steve.html');await p.waitForFunction(()=>window.SteveLab);const verified=await p.evaluate(frames=>SteveLab.verifyReplay({version:1,frames}),frames);assert.equal(verified,frames.length);
 const r=cp.spawnSync('rtk',['proxy',path.join(root,'native/steve_lab/target/release/steve-vectors.exe')],{input:frames.map(f=>'0 '+f.input.join(' ')).join('\n')+'\n',encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(r.status,0,r.stderr);const rows=r.stdout.trim().split(/\r?\n/).map(JSON.parse);assert.deepStrictEqual(rows,frames.map(f=>f.output));
 const report={version:1,frames:verified,originalWasmMatchesCurrentReader:true,nativeMatchesOriginalWasm:true};fs.writeFileSync(path.join(root,'docs/creative-results/steve-v1-compatibility.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }finally{await b.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
