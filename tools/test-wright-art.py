"""Browser art/control checks, complete native/WASM traces and five rendering samples."""
from pathlib import Path
from playwright.sync_api import sync_playwright
from PIL import Image,ImageDraw,ImageFont
import hashlib,json,subprocess,time,tempfile,io
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/wright-art-review';URL='http://127.0.0.1:7300/editor/'
checks=[]
def check(name,ok):
    assert ok,name
    checks.append(name)
with sync_playwright() as p:
    browser=p.chromium.launch(headless=True);page=browser.new_page(viewport={'width':1320,'height':1100});errors=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.goto(URL+'wright-review.html');page.wait_for_function('window.wrightArtReview?.painter.loaded')
    check('source review images decode',page.evaluate("Array.from(document.images).filter(i=>!i.src.endsWith('/bomber-line.gif')).every(i=>i.complete&&i.naturalWidth>0)"))
    for pose in ('idle','run','attack','skill1','skill2','skill3','ult','hit','dead'):
        page.select_option('#pose',pose);check(pose+' eight-frame inspection',page.locator('#frame').get_attribute('max')=='7')
        for facing in ('left','right'):
            page.select_option('#facing',facing);page.wait_for_function('pose=>wrightArtReview.painter.lastBody?.pose===pose',arg=pose)
            check(pose+' '+facing+' binding',page.evaluate('wrightArtReview.painter.lastBody.pose')==pose)
    page.screenshot(path=str(OUT/'complete-animation-review.png'),full_page=True)
    page.select_option('#plane','fighter');page.select_option('#flightPhase','Run');page.locator('#heading').fill('3')
    check('review exposes dogfight phase inspection',page.locator('#flightPhase').input_value()=='Run')
    page.goto(URL+'wright.html');page.wait_for_function('window.wrightLab?.painter.loaded')
    check('actual packaged atlases load',page.evaluate('wrightLab.painter.bodyImage.width===512&&wrightLab.painter.fxImage.width===1920&&!wrightLab.painter.manifest.review_only'))
    check('thirteen athlete controls retained',page.locator('#stats input').count()==13)
    check('canonical 45k and 100k areas loaded',page.evaluate('WRIGHT_ART.geometry.bomber_radius===WRIGHT_RULES.bomber.radius&&WRIGHT_ART.geometry.carrier_knock_radius===WRIGHT_RULES.carrier.knockRadius'))
    check('normal and carrier-ready warnings select complete lifetimes',page.evaluate("wrightLab.painter.effectTag('wrt_warn',114)==='warn_t114'&&wrightLab.painter.effectTag('wrt_warn',72)==='warn_t72'"))
    check('jet phase selection retains stable birth clock',page.evaluate("wrightLab.painter.planeTag({kind:'fighter',phase:'Run',born:100,heading:3},103)==='fighter_run0_p3'&&wrightLab.painter.planeTag({kind:'fighter',phase:'Run',born:100,heading:3},104)==='fighter_run1_p3'&&wrightLab.painter.planeTag({kind:'fighter',phase:'Break',born:100,heading:3},108)==='fighter_break0_p3'"))
    rates=page.evaluate('''()=>[30,60,120,144,240].map(rate=>{const c=new WrightClock();let ticks=0;for(let i=0;i<=rate*2;i++)c.advance(i*1000/rate,true,()=>ticks++);return {rate,ticks};})''')
    for row in rates:check(str(row['rate'])+' Hz gives 60 ticks/sec',row['ticks']==120)
    check('pause resets accumulator and stalls have bounded catch-up',page.evaluate('''()=>{let ticks=0;const c=new WrightClock();c.advance(0,true,()=>ticks++);c.advance(60000,true,()=>ticks++);if(ticks!==15)return false;c.advance(61000,false,()=>ticks++);c.advance(61016.666667,true,()=>ticks++);return ticks===16;}'''))
    scenario='''({art,rating=50,bomber=true})=>{
      const L=wrightLab;L.pause();L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;document.querySelector('#incomingFire').checked=false;document.querySelector('#art').checked=art;
      document.querySelectorAll('#stats input').forEach(i=>i.value=rating);L.setAim({x:390000,y:480000});for(const u of L.units.filter(u=>u.team===1))u.hp=u.max_hp=100000;
      const trace=[],run=n=>{for(let i=0;i<n;i++){L.step();trace.push(JSON.stringify({state:L.state,me:L.me,units:L.units,carriers:[...L.carriers],tick:L.tick}));}};
      run(500);L.order('fighters');run(250);L.units[0].pos={x:355000,y:480000};L.units[1].pos={x:390000,y:515000};if(bomber)L.order('bomber');run(180);L.units[2].hp=200;L.order('repair');run(180);L.order('carrier');run(540);run(360);return trace;
    }'''
    off=page.evaluate(scenario,{'art':False})
    page.evaluate('window.wrightRecording=[];wrightLab.setRecorder((input,output)=>wrightRecording.push([input,output]));')
    on=page.evaluate(scenario,{'art':True});records=page.evaluate('wrightLab.setRecorder(null);wrightRecording')
    check('complete combat/host trace identical with art on/off',off==on)
    native=subprocess.run([str(ROOT/'native/wright_lab/target/release/parity.exe')],input='\n'.join(r[0] for r in records)+'\n',capture_output=True,text=True,encoding='utf8',check=True)
    native_rows=[json.loads(s) for s in native.stdout.splitlines() if s.strip()];wasm_rows=[json.loads(r[1]) for r in records]
    check('complete native/WASM states and command arrays',native_rows==wasm_rows)
    # The requested bomber geometry intentionally changes combat. Compare the
    # unaffected kit separately; do not claim wright-2 bomber outcomes still match.
    baseline='1f96d4805670a16f18c018b4001bb66fff715749'
    page.evaluate('window.unaffectedRecording=[];wrightLab.setRecorder((input,output)=>unaffectedRecording.push([input,output]));')
    page.evaluate(scenario,{'art':True,'bomber':False});unaffected=page.evaluate('wrightLab.setRecorder(null);unaffectedRecording')
    baseline_bytes=subprocess.check_output(['rtk','proxy','git','show',baseline+':editor/wright.wasm'],cwd=ROOT)
    cache=ROOT/'backups';cache.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=cache,suffix='.wasm',delete=False) as temp:
        temp.write(baseline_bytes);baseline_file=Path(temp.name)
    try:
        baseline_rows=page.evaluate('''async({url,records})=>{const w=(await WebAssembly.instantiate(await(await fetch(url)).arrayBuffer(),{})).instance.exports;w.wright_reset();return records.map(([input])=>{const old=JSON.parse(input);old.rules_revision='wright-2';const b=new TextEncoder().encode(JSON.stringify(old)),p=w.wright_input(b.length);new Uint8Array(w.memory.buffer,p,b.length).set(b);const o=w.wright_step(b.length);return JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,o,w.wright_output_len())));});}''',{'url':'../backups/'+baseline_file.name,'records':unaffected})
    finally:baseline_file.unlink()
    def combat(row):
        row=json.loads(json.dumps(row));row['effects']=[e for e in row.get('effects',[]) if e['kind']!='fx']
        assert row['rules_revision'] in ('wright-2','wright-3')
        row.pop('rules_revision')
        for plane in row['state'].get('planes',[]):
            assert plane.get('strikes') is None
            plane.pop('strikes',None)
        return row
    check('non-bomber baseline complete combat states and nonvisual commands unchanged',[combat(r) for r in baseline_rows]==[combat(json.loads(r[1])) for r in unaffected])
    check('native/WASM jet phase names agree with Lab renderer',page.evaluate('''records=>records.every(([input,output])=>{const row=JSON.parse(output),tick=JSON.parse(input).frame.tick;return(row.state.planes||[]).filter(p=>p.kind==='fighter'&&p.born<tick&&p.start<=tick).every(p=>row.effects.some(e=>e.kind==='fx'&&e.data.name==='wrt_'+wrightLab.painter.planeTag(p,tick)&&e.data.at.x===p.pos.x&&e.data.at.y===p.pos.y));})''',records))
    low=page.evaluate(scenario,{'art':True,'rating':20});high=page.evaluate(scenario,{'art':True,'rating':80})
    check('manual mechanics independent of thirteen-stat profile',low==high)
    line=page.evaluate('''()=>{
      const L=wrightLab;L.pause();L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;document.querySelector('#incomingFire').checked=false;document.querySelector('#zoom').checked=true;
      const template={...L.units[0]},out=[];L.units.length=0;
      for(let i=0;i<3;i++)L.units.push({...template,id:2+i,pos:{x:330000+i*40000,y:524000},hp:1000,max_hp:1000});
      L.setRecorder((input,output)=>out.push(JSON.parse(output)));L.setAim({x:370000,y:480000});L.order('bomber');L.step();L.setRecorder(null);
      window.bomberLineFirst=out;return {centres:L.state.planes.find(p=>p.kind==='bomber').strikes,warnings:out[0].effects.filter(e=>e.kind==='fx'&&/^wrt_warn[123]$/.test(e.data.name)).map(e=>e.data)};
    }''')
    check('three fixed advancing centres',line['centres']==[{'x':330000,'y':480000},{'x':370000,'y':480000},{'x':410000,'y':480000}])
    check('three individually timed ground warnings',[w['ticks'] for w in line['warnings']]==[60,78,96])
    page.wait_for_timeout(40);page.screenshot(path=str(OUT/'bomber-line.png'),full_page=True)
    frames=[Image.open(io.BytesIO(page.locator('#arena').screenshot())).convert('RGB')]
    for _ in range(20):
        page.evaluate('for(let i=0;i<6;i++)wrightLab.step()');page.wait_for_timeout(20)
        frames.append(Image.open(io.BytesIO(page.locator('#arena').screenshot())).convert('RGB'))
    frames[0].save(OUT/'bomber-line.gif',save_all=True,append_images=frames[1:],duration=100,loop=0)
    strip=Image.new('RGB',(1200,350),'#162b36');draw=ImageDraw.Draw(strip);font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',16)
    for i,index in enumerate((11,14,17)):
        im=frames[index].crop((350,240,900,660));im.thumbnail((390,300));strip.paste(im,(i*400,40))
        draw.text((i*400+12,8),'Strike '+str(i+1)+(' / stun' if i==2 else ' / slow'),font=font,fill='#ffbe71')
    strip.save(OUT/'bomber-sequence.png')
    review=browser.new_page();review.goto(URL+'wright-review.html');review.wait_for_function('window.wrightArtReview?.painter.loaded')
    check('all review images including actual-core bomber animation decode',review.evaluate('Array.from(document.images).every(i=>i.complete&&i.naturalWidth>0)'));review.close()
    line_hits=page.evaluate('wrightLab.units.map(u=>({hp:u.hp,slow:u.slowUntil,stun:u.stunUntil}))')
    check('each isolated footprint delivers one large blast',[u['hp'] for u in line_hits]==[915,915,915])
    check('only third footprint receives the final stun',line_hits[0]['stun']==0 and line_hits[1]['stun']==0 and line_hits[2]['stun']>120)
    # Real keyboard movement, focus loss and pause/step on the actual arena.
    page.evaluate('wrightLab.pause();wrightLab.reset();document.querySelector("#auto").checked=false;document.querySelector("#moving").checked=false;')
    page.locator('#arena').focus();page.keyboard.down('d');page.evaluate('for(let i=0;i<10;i++)wrightLab.step()');page.keyboard.up('d')
    check('WASD moves actor',page.evaluate('wrightLab.me.pos.x===310000'))
    page.keyboard.down('d');page.locator('#play').focus();page.evaluate('const x=wrightLab.me.pos.x;wrightLab.step();window.stoppedOnBlur=wrightLab.me.pos.x===x;');page.keyboard.up('d')
    check('arena focus loss clears held movement',page.evaluate('stoppedOnBlur'))
    before=page.evaluate('wrightLab.tick');page.click('#step');check('step pauses and advances once',page.evaluate('wrightLab.tick')==before+1 and page.locator('#play').inner_text()=='Play')
    # Exercise the actual demo controls. No free resources or modified native stats.
    for demo in ('escort','rockets','kamikaze','bomber','repair','drone','carrier'):
        page.select_option('#demo',demo);page.click('#demoRun');page.evaluate('wrightLab.pause();wrightLab.step();');check(demo+' preview starts',page.evaluate('!document.querySelector("#log").textContent.includes("Lab error")'))
    page.select_option('#demo','rockets');page.click('#demoRun');page.evaluate('wrightLab.pause();for(let i=0;i<4;i++)wrightLab.step()');page.click('#damagePlanes')
    check('low-HP fixture changes actual Lab carrier HP only',page.evaluate('[...wrightLab.carriers.values()].filter(c=>c.kind==="fighter").every(c=>c.hp===Math.floor(c.max_hp*.2))'))
    page.select_option('#demo','carrier');page.click('#demoRun');page.evaluate('wrightLab.pause();for(let i=0;i<95;i++)wrightLab.step()');page.wait_for_timeout(30)
    page.screenshot(path=str(OUT/'playable-lab.png'),full_page=True)
    page.evaluate('wrightLab.me.hp=0;wrightLab.step()');page.wait_for_function('wrightLab.painter.lastBody?.pose==="dead"')
    check('death clears fleet and chooses body death pose',page.evaluate('wrightLab.state.planes.length===0&&wrightLab.state.carrier===null'))
    cold=[]
    for _ in range(5):
        context=browser.new_context();tab=context.new_page();start=time.perf_counter();tab.goto(URL+'wright.html');tab.wait_for_function('window.wrightLab?.painter.loaded');cold.append((time.perf_counter()-start)*1000);context.close()
    warm=page.evaluate('''()=>{const c=document.createElement('canvas');c.width=c.height=900;const x=c.getContext('2d'),p=wrightLab.painter,out=[];for(let run=0;run<5;run++){const t=performance.now();for(let i=0;i<1000;i++){p.body(x,'run',i,450,450,2);for(let plane=0;plane<16;plane++){const kind=['fighter','bomber','chopper','repair','drone','rocket'][plane%6];p.effect(x,p.planeTag({kind,heading:i%8,phase:plane%2?'Break':'Run',born:0},i),0,plane*45,300,2);}p.effect(x,'carrier',0,450,600,2);p.effect(x,'warn_t114',i%114,500,300,2);p.effect(x,'bomber_wave3',i%14,500,300,2);p.effect(x,'knock_ring',i%24,450,600,2);}out.push((performance.now()-t)/1000);}return out;}''')
    check('no browser errors',not errors);browser.close()
report={'checks':len(checks),'passed':checks,'complete_trace_frames':len(on),'native_wasm_parity_frames':len(native_rows),'combat_baseline':baseline,'baseline_non_bomber_frames':len(baseline_rows),'trace_sha256':hashlib.sha256('\n'.join(on).encode()).hexdigest(),'display_rates':rates,'cold_browser_load_ms':cold,'sustained_draw_ms_per_16_plane_scene':warm,'performance_scope':'Five cold browser contexts and five sustained canvas scenes; not native-host FPS','live_game_acceptance':'pending','errors':errors}
(OUT/'browser-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
