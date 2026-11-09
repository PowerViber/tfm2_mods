"""Exercise real editor-server routes and verify they serve the packaged atlases."""
from pathlib import Path
from playwright.sync_api import sync_playwright
import subprocess,time,urllib.request,json,hashlib,os,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/wright-art-review'
fixture=Path(tempfile.mkdtemp(prefix='wright-routes-',dir=ROOT/'backups'))
server=subprocess.Popen(['node','server.js','--port','7322','--no-open'],cwd=ROOT/'editor',env=dict(os.environ,APPDATA=str(fixture/'appdata')),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
try:
    for _ in range(100):
        try:
            urllib.request.urlopen('http://127.0.0.1:7322/wright.html',timeout=1);break
        except OSError:time.sleep(.1)
    checks=[]
    with sync_playwright() as p:
        browser=p.chromium.launch(headless=True);page=browser.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
        for name in ('wright-review.html','wright.html'):
            page.goto('http://127.0.0.1:7322/'+name);page.wait_for_function('window.wrightArtReview?.painter.loaded||window.wrightLab?.painter.loaded');checks.append(name)
        for name,path in [('wright-body.png','champions/tfm2_custom_wright#sheet.png'),('wright-fx.png','vfx/wright#sheet.png')]:
            response=page.request.get('http://127.0.0.1:7322/'+name)
            assert response.ok and hashlib.sha256(response.body()).digest()==hashlib.sha256((ROOT/'mods/tfm2_custom'/path).read_bytes()).digest();checks.append(name+' packaged hash')
        assert not errors,errors;checks.append('no script errors');browser.close()
    report={'checks':len(checks),'passed':checks,'scope':'Real Node editor routes; isolated AppData; no game installation'}
    (OUT/'routes.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
finally:
    server.terminate();server.wait(timeout=10)
