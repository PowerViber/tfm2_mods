"""Real-browser regressions for Thalassa's linked Dive and display-rate fixes."""
from pathlib import Path
from playwright.sync_api import sync_playwright
import json, subprocess

ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/thalassa-art-review'
checks=[]
def check(name,ok):
    assert ok,name
    checks.append(name)
with sync_playwright() as p:
    browser=p.chromium.launch(headless=True)
    page=browser.new_page(viewport={'width':1280,'height':1100});errors=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.goto('http://127.0.0.1:7299/editor/thalassa.html?build=thalassa-1.1')
    page.wait_for_function('window.thalassaLab?.painter.loaded')
    rates=page.evaluate('''()=>[30,60,120,144,240].map(rate=>{const clock=new ThalassaClock();let ticks=0;for(let i=0;i<=rate*2;i++)clock.advance(i*1000/rate,true,()=>ticks++);return {rate,ticks};})''')
    for row in rates:check(str(row['rate'])+' Hz runs exactly 60 simulation ticks/sec',row['ticks']==120)
    check('pause, background and single-step clock do not accumulate catch-up',page.evaluate('''()=>{const c=new ThalassaClock();let t=0;c.advance(0,true,()=>t++);c.advance(1000,false,()=>t++);c.advance(1016.6666667,true,()=>t++);const resumed=t;c.reset();c.advance(200000,true,()=>t++);return resumed===1&&t===1;}'''))
    check('catch-up work is bounded after a stalled frame',page.evaluate('''()=>{const c=new ThalassaClock();let t=0;c.advance(0,true,()=>t++);c.advance(60000,true,()=>t++);return t===15;}'''))
    # Exercise host-model positions throughout flight, not just instantaneous core commands.
    dive=page.evaluate('''()=>{const L=thalassaLab;L.pause();L.reset();document.querySelector('#auto').checked=false;document.querySelector('#moving').checked=false;
      const run=n=>{for(let i=0;i<n;i++)L.step();};L.order('tide_pool');run(16);
      L.setAim({x:300000,y:510000});L.order('twin');run(1);
      const from={...L.me.pos};L.setAim({x:450000,y:480000});L.order('dive');run(1);
      const began=L.tick,until=L.state.moving_until,frames=[];
      while(L.tick<until){run(1);frames.push({me:{...L.me.pos},clone:{...L.clone.pos}});}
      return {from,began,until,frames,to:{...L.me.pos},clone:{...L.clone.pos},pools:L.state.pools};}''')
    check('water Dive reaches the approved 75000 range',dive['to']['x']-dive['from']['x']==75000)
    check('water Dive retains its existing distance-based duration',dive['until']-dive['began']==27)
    check('twin and owner arrive at the same committed destination',dive['clone']==dive['to'])
    check('both real carriers move during the dive',len({r['me']['x'] for r in dive['frames']})>10 and len({r['clone']['x'] for r in dive['frames']})>10)
    check('all ordinary pool gameplay radii equal current whirlpool radius',all(p['radius']==35000 for p in dive['pools']))
    # Both adapters select the same bounded approved-pixel segments in every direction.
    points=[[[100000,100000],[100000,100000]],[[100000,100000],[900000,900000]],[[900000,900000],[100000,100000]]]
    points += [[[480000,480000],[480000+dx,480000+dy]] for dx,dy in [(40000,0),(0,40000),(-40000,0),(0,-40000),(200000,-100000),(-100000,-200000)]]
    js=page.evaluate('''points=>points.map(([a,b])=>thalassaLab.painter.linkSegments({x:a[0],y:a[1]},{x:b[0],y:b[1]}).map(p=>[p.tag,p.at.x,p.at.y]))''',points)
    native=subprocess.run([str(ROOT/'native/thalassa_lab/target/release/parity.exe'),'--links'],input='\n'.join(json.dumps(p) for p in points)+'\n',text=True,capture_output=True,check=True)
    check('native/Lab link geometry and headings match',js==[json.loads(s) for s in native.stdout.splitlines()])
    check('link renewal is bounded to six one-tick segments',all(len(row)<=6 for row in js))
    check('link responds to movement at either endpoint',page.evaluate('''()=>{const p=thalassaLab.painter,a={x:300000,y:480000},b={x:330000,y:520000},c={x:390000,y:440000};return JSON.stringify(p.linkSegments(a,b))!==JSON.stringify(p.linkSegments(a,c))&&JSON.stringify(p.linkSegments(a,b))!==JSON.stringify(p.linkSegments(c,b));}'''))
    check('link and pool assets load without browser errors',not errors)
    page.evaluate('''()=>{const L=thalassaLab;L.reset();L.order('tide_pool');for(let i=0;i<16;i++)L.step();L.setAim({x:310000,y:507000});L.order('twin');for(let i=0;i<25;i++)L.step();L.clone.pos={x:335000,y:535000};}''')
    page.wait_for_timeout(60)
    page.screenshot(path=str(OUT/'followup-lab.png'),full_page=True)
    browser.close()
report={'checks':len(checks),'passed':checks,'display_rates':rates,'dive':{k:v for k,v in dive.items() if k!='frames'},'link_parity_cases':len(points),'live_game_acceptance':'pending'}
(OUT/'followup-verification.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print(json.dumps(report))
