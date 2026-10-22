/* Stats changer check in a real browser (Database Editor, player drawer + Bulk edit + Changes).
   No save file is involved: the page is opened with ?test, which exposes the editor state, and two
   synthetic athletes are put in it. The server runs against a fixture game folder whose
   mods/tfm2_custom_ai holds a game-sense export, so reading and applying game sense use the real routes.
   Covers: live descriptions of the nine values, presets, the ratings preview, game-sense sliders,
   focus, the Changes list and its revert, "Apply to career" writing a valid override file,
   Bulk edit for profile values and game sense, and no page errors or sideways scroll at 390 px
   in both themes. Not covered: parsing a real save (core.js has its own checks). */
const fs = require('node:fs'), path = require('node:path'), os = require('node:os'), cp = require('node:child_process'), assert = require('node:assert/strict');
const { chromium } = require('playwright');
const root = path.resolve(__dirname, '..'), port = 19473, base = 'http://127.0.0.1:' + port;
const shots = process.env.SHOTS || fs.mkdtempSync(path.join(os.tmpdir(), 'tfm2-stats-'));
const PROFILE = ['ego', 'judgement', 'positioning', 'control_speed', 'skill_hit', 'skill_avoid', 'aggressive', 'order', 'roaming'];

(async () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'tfm2-stats-game-'));
  fs.writeFileSync(path.join(fixture, 'bundle.game_data'), '');
  const modDir = path.join(fixture, 'mods', 'tfm2_custom_ai'); fs.mkdirSync(modDir, { recursive: true });
  fs.writeFileSync(path.join(modDir, 'game_sense_export_v2.json'), JSON.stringify({ version: 2, career: 'c0ffee', athletes: [
    { id: 11, ratings: [40, 55, 60, 45], progress: [0, 10, 0, 0], focus: 0, existing: null, version: 2 }] }));
  const server = cp.spawn(process.execPath, [path.join(root, 'editor/server.js'), '--no-open', '--port', String(port), '--game', fixture], { stdio: 'pipe' });
  let boot = ''; server.stdout.on('data', d => boot += d); server.stderr.on('data', d => boot += d);
  for (let n = 0; n < 100; n++) { try { if ((await fetch(base + '/index.html')).ok) break; } catch { } await new Promise(r => setTimeout(r, 100)); if (n === 99) throw Error(boot); }
  const browser = await chromium.launch({ headless: true });
  const report = {};
  try {
    for (const [width, theme] of [[1280, 'light'], [390, 'dark']]) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme }), errors = [];
      page.on('pageerror', e => errors.push(e.message)); page.on('dialog', d => d.accept());
      await page.goto(base + '/index.html?test');
      await page.waitForFunction(() => window.TFM2EditorTest && window.TFM2EditorTest.S.server);
      // two synthetic athletes: #11 has a game-sense export entry, #12 does not
      await page.evaluate(PROFILE => {
        const T = window.TFM2EditorTest, S = T.S;
        const make = (key, name, v) => {
          const fields = { name: { type: 'str', off: 0, value: name }, age: { type: 'u64', off: 0, value: 21 } };
          for (const k of ['last_hit', 'skill_avoid', 'skill_hit', 'positioning', 'control_speed', 'concentration', 'mental', 'judgement', 'order', 'roaming', 'aggressive', 'ego', 'top', 'jungle', 'mid', 'bottom', 'support'])
            fields[k] = { type: 'u64', off: 0, value: v };
          return { key, id: key, kind: 'athlete', fields, languages: [[0, 100]], langEdits: null, edits: null, contract: null };
        };
        S.athletes = [make(11, 'Fixture One', 50), make(12, 'Fixture Two', 50)]; S.list = S.athletes.slice();
        document.querySelector('#welcome').hidden = true; document.querySelector('#workspace').hidden = false;
        T.openDrawer(S.athletes[0]);
      }, PROFILE);
      await page.waitForSelector('.ap-presets');
      await page.waitForSelector('[data-gsrow="0"]');
      const desc = k => page.$eval(`.ap-desc[data-desc="${k}"]`, e => ({ text: e.textContent, on: e.classList.contains('on') }));
      // at 50 every value plays like the engine
      for (const k of PROFILE) assert.equal((await desc(k)).on, false, k + ' active at 50');
      // control speed 20 by its slider: the description and the Changes count update live
      await page.$eval('.ap-item [data-row="control_speed"] input[type=range]', el => { el.value = 20; el.dispatchEvent(new Event('input', { bubbles: true })); });
      const cs = await desc('control_speed');
      assert.ok(cs.on && /wait about 12 ticks/.test(cs.text), JSON.stringify(cs));
      assert.equal(await page.$eval('[data-row="control_speed"] input[type=number]', e => e.value), '20', 'the Skills copy of the row follows');
      // Duelist preset: three more edits
      await page.click('[data-appreset="1"]');
      const edits = await page.evaluate(() => Object.keys(window.TFM2EditorTest.S.athletes[0].edits).sort());
      assert.deepEqual(edits, ['aggressive', 'control_speed', 'ego', 'order']);
      assert.ok((await desc('aggressive')).on && (await desc('ego')).on === false && (await desc('order')).on === false);
      const preview = await page.$eval('#apPreview', e => e.textContent);
      assert.match(preview, /Macro/);
      // game sense: Macro 70 and focus Strategy, listed in Changes
      await page.$eval('[data-gsrow="0"] input[type=range]', el => { el.value = 70; el.dispatchEvent(new Event('input', { bubbles: true })); });
      await page.click('[data-gsfocus="3"]');
      await page.click('#dClose');
      await page.evaluate(() => { window.TFM2EditorTest.switchTab('changes'); });
      const changes = await page.$$eval('#changeList .chg', rows => rows.map(r => r.textContent.replace(/\s+/g, ' ').trim()));
      assert.ok(changes.some(c => /Game sense · Fixture One.*Macro.*40.*70/.test(c)), changes.join('\n'));
      assert.ok(changes.some(c => /Training focus/.test(c)), changes.join('\n'));
      assert.equal(changes.length, 6);
      // revert the focus from the Changes tab
      const focusRow = changes.findIndex(c => /Training focus/.test(c));
      await page.click(`#changeList [data-rev="${focusRow}"]`);
      assert.equal(await page.$$eval('#changeList .chg', r => r.length), 5);
      // Bulk edit: all nine profile values set 50 on both, then Micro +10 (only #11 has an export)
      const bulk = async (field, op, value) => {
        await page.evaluate(() => document.querySelector('#btnBulk').click());
        await page.evaluate(([field, op, value]) => {
          document.querySelector('#bulkField').value = field; document.querySelector('#bulkOp').value = op; document.querySelector('#bulkVal').value = value;
          document.querySelector('#bulkDlg').close('apply');
        }, [field, op, value]);
        await page.waitForTimeout(150); // the dialog's close event is dispatched asynchronously
      };
      await bulk('@profile', 'set', 50);
      assert.equal(await page.evaluate(() => Object.keys(window.TFM2EditorTest.S.athletes[0].edits || {}).length), 0, 'profile values back to their originals');
      await bulk('@gs1', 'add', 10);
      const staged = await page.evaluate(() => [...window.TFM2EditorTest.S.gs.edits].map(([id, e]) => [id, e.ratings]));
      assert.deepEqual(staged, [[11, [70, 65, 60, 45]]]);
      // Apply: the server writes the override; the drawer shows it pending
      if (width === 1280) {
        await page.click('#gsApplyAll');
        await page.waitForFunction(() => window.TFM2EditorTest.S.gs.edits.size === 0 && window.TFM2EditorTest.S.gs.pending);
        const file = JSON.parse(fs.readFileSync(path.join(modDir, 'game_sense_overrides_v1.json'), 'utf8'));
        assert.equal(file.version, 1); assert.equal(file.career, 'c0ffee');
        assert.deepEqual(file.athletes, [{ id: 11, ratings: [70, 65, 60, 45], focus: 0 }]);
        await page.evaluate(() => window.TFM2EditorTest.openDrawer(window.TFM2EditorTest.S.athletes[0]));
        assert.match(await page.textContent('.gs-actions'), /Pending/);
        const bad = await fetch(base + '/api/game-sense', { method: 'PUT', body: JSON.stringify({ version: 1, career: 'c0ffee', athletes: [{ id: 1, focus: 7 }] }) });
        assert.equal(bad.status, 400);
      }
      // athlete without an export entry explains why
      await page.evaluate(() => window.TFM2EditorTest.openDrawer(window.TFM2EditorTest.S.athletes[1]));
      assert.match(await page.textContent('.gs-note'), /no game-sense entry yet/);
      await page.evaluate(() => window.TFM2EditorTest.openDrawer(window.TFM2EditorTest.S.athletes[0]));
      await page.$eval('.ap-presets', e => e.scrollIntoView());
      const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth + 1);
      await page.screenshot({ path: path.join(shots, `stats-changer-${theme}-${width}.png`) });
      assert.equal(overflow, false, 'sideways scroll at ' + width);
      assert.deepEqual(errors, []);
      report[theme] = { width, ok: true };
      await page.close();
    }
  } finally { await browser.close(); server.kill(); }
  console.log('PASS: stats changer', JSON.stringify(report), 'screenshots in', shots);
})().catch(e => { console.error(e); process.exitCode = 1; });
