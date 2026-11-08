"""Browser verification of packaged artwork, controls and visual-only Lab equivalence."""
from pathlib import Path
from playwright.sync_api import sync_playwright
import json,hashlib,time,subprocess

ROOT=Path(__file__).resolve().parents[1]; OUT=ROOT/'docs/thalassa-art-review'
URL='http://127.0.0.1:7299/editor/'
checks=[]
def check(name,ok):assert ok,name;checks.append(name)
with sync_playwright() as p:
    browser=p.chromium.launch(headless=True)
    page=browser.new_page(viewport={'width':1320,'height':980}); errors=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.goto(URL+'thalassa-review.html');page.wait_for_function('window.thalassaArtReview?.painter.loaded')
    page.wait_for_function('document.querySelectorAll(".fx figure").length===16')
    check('all review images decode',page.evaluate('Array.from(document.images).every(i=>i.complete&&i.naturalWidth>0)'))
    for pose,n in [('idle',4),('run',6),('attack',4),('skill1',4),('skill2',4),('skill3',4),('ult',4),('hit',1),('dead',6)]:
        page.select_option('#animation',pose)
        check(pose+' complete frame controls',int(page.locator('#frame').get_attribute('max'))==n-1)
        for facing in ('right','left'):
            page.select_option('#animationFacing',facing);page.locator('#animate').uncheck();page.locator('#frame').fill(str(n-1))
            check(pose+' '+facing+' atlas binding',page.evaluate('(THALASSA_ART.body[document.querySelector("#animation").value+(document.querySelector("#animationFacing").value==="left"?"_l":"")].length)>0'))
    # Native's generated clock uses integral thousandths of a tick; validate every boundary against it.
    check('native/lab presentation clock boundaries',page.evaluate('''()=>{const P=thalassaArtReview.painter;for(const [pose,ts]of Object.entries(THALASSA_ART.pose_ticks)){const ds=ts.map(t=>Math.round(t*1000)),total=ds.reduce((a,b)=>a+b,0);for(let tick=0;tick<500;tick++){let left=tick*1000%total,index=0;while(index<ds.length-1&&left>=ds[index])left-=ds[index++];if(P.frame(THALASSA_ART.body[pose],tick)!==index)return false;}}return true;}'''))
    page.screenshot(path=str(OUT/'complete-animation-review.png'),full_page=True)
    page.goto(URL+'thalassa.html');page.wait_for_function('window.thalassaLab?.painter.loaded')
    check('Lab loads packaged atlases',page.evaluate('thalassaLab.painter.bodyImage.width===512&&thalassaLab.painter.fxImage.width<=2048'))
    scenario='''enabled=>{const L=thalassaLab;L.pause();L.reset();L.setAim({x:300000,y:480000});document.querySelector('#moving').checked=false;document.querySelector('#auto').checked=false;document.querySelector('#art').checked=enabled;const trace=[];const run=n=>{for(let i=0;i<n;i++){L.step();trace.push(JSON.stringify({state:L.state,me:L.me,units:L.units,clone:L.clone,tick:L.tick}));}};L.order('tide_pool');run(16);run(480);L.units[0].pos={x:345000,y:478000};L.units[1].pos={x:350000,y:490000};L.setAim({x:347000,y:484000});L.order('tide_pool');run(16);L.setAim({x:340000,y:480000});L.order('twin');run(2);L.units[2].stunUntil=L.tick+60;run(20);L.setAim({x:900000,y:150000});L.order('ult');run(300);const far=L.state.pools.find(p=>p.center.x>700000);if(far){L.setAim(far.center);L.order('dive');run(120);}return trace;}'''
    off=page.evaluate(scenario,False)
    page.evaluate('window.thalassaCoreRecording=[];thalassaLab.setRecorder((input,output)=>thalassaCoreRecording.push([input,output]));')
    on=page.evaluate(scenario,True)
    records=page.evaluate('thalassaLab.setRecorder(null);thalassaCoreRecording')
    check('identical complete combat traces with artwork on/off',off==on)
    native=ROOT/'native/thalassa_lab/target/release/parity.exe'
    r=subprocess.run([str(native)],input='\n'.join(row[0] for row in records)+'\n',capture_output=True,text=True,encoding='utf-8',check=True)
    native_rows=[json.loads(line) for line in r.stdout.splitlines() if line.strip()]
    wasm_rows=[json.loads(row[1]) for row in records]
    check('complete native/WASM states and command arrays',native_rows==wasm_rows)
    page.evaluate('document.querySelector("#art").checked=true');page.screenshot(path=str(OUT/'playable-lab.png'))
    page.evaluate('thalassaLab.me.hp=0;thalassaLab.step();')
    page.wait_for_function('thalassaLab.painter.lastBody?.pose==="dead"')
    check('death selects the packaged death pose and clears owned pools',page.evaluate('thalassaLab.state.pools.length===0&&thalassaLab.painter.lastBody.pose==="dead"'))
    # Five sustained rendering samples; browser drawing only, not game-host FPS.
    warm=page.evaluate('''()=>{const c=document.createElement('canvas');c.width=900;c.height=900;const x=c.getContext('2d'),p=thalassaLab.painter,r=[];for(let run=0;run<5;run++){const t=performance.now();for(let i=0;i<1000;i++)p.body(x,'run',i,450,450,3,i%2===0,false);r.push((performance.now()-t)/1000);}return r;}''')
    check('no browser script errors',not errors)
    cold=[]
    for run in range(5):
        context=browser.new_context();cold_page=context.new_page();start=time.perf_counter()
        cold_page.goto(URL+'thalassa.html');cold_page.wait_for_function('window.thalassaLab?.painter.loaded')
        cold.append((time.perf_counter()-start)*1000);context.close()
    browser.close()
report={'checks':len(checks),'passed':checks,'manual_equivalence_frames':len(on),
        'native_wasm_parity_frames':len(native_rows),
        'trace_sha256':hashlib.sha256('\n'.join(on).encode()).hexdigest(),
        'cold_browser_context_load_ms':cold,'warm_browser_draw_ms_per_body':warm,'measurement_scope':'five new-browser-context loads and five sustained canvas samples; not actual-game host performance',
        'errors':errors,'live_game_acceptance':'pending'}
(OUT/'browser-verification.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
