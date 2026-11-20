const assert = require('node:assert/strict');
const { chromium } = require(require.resolve('playwright', {
  paths: [process.cwd(), require('node:os').homedir() + '/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules'],
}));

async function measure(browser, base, targets) {
  const page = await browser.newPage();
  try {
    await page.goto(base + '/editor/contender.html');
    await page.waitForFunction(() => window.contenderLab?.me && !document.getElementById('addTargets').disabled);
    await page.selectOption('#scenario', 'damage');
    return await page.evaluate(async count => {
      const lab = contenderLab;
      const rules = CONTENDER_RULES;
      await lab.reset();
      lab.clearTargets();
      await lab.addTargets('stock:knight', count, 8, true);
      const centre = lab.me.pos;
      const reach = Math.min(rules.basic.range, rules.flurry.startRange, rules.flurry.chainRange);
      const positions = Array.from({ length: 32 }, (_, i) => {
        const angle = i * Math.PI / 16;
        return {
          x: Math.round(centre.x + reach * 0.55 * Math.cos(angle)),
          y: Math.round(centre.y + reach * 0.55 * Math.sin(angle)),
        };
      }).filter(pos => !lab.world.wallAt(pos, lab.me.radius) && lab.world.clear(centre, pos));
      if (positions.length < count) throw Error('No open cluster in the game-map fixture');
      lab.units.forEach((unit, i) => { unit.pos = positions[i]; });

      lab.step({ kind: 's2' });
      for (let i = 0; i < rules.flurry.ticks; i++) lab.step();
      const rhythmStacks = Object.fromEntries(lab.state.stacks.map(stack => [stack.id, stack.count]));
      const finished = lab.records.at(-1);
      const finalPos = { ...lab.me.pos };

      const arena = document.getElementById('arena');
      arena.dispatchEvent(new KeyboardEvent('keydown', { key: ' ' }));
      lab.pause();
      lab.step();
      window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ' }));
      for (let i = 0; i <= rules.basic.startTiming + rules.pact.delayTicks; i++) lab.step();

      const effects = lab.records.flatMap(row => row.effects);
      const packets = effects.filter(effect => effect.kind === 'damage');
      const flurry = packets.filter(effect => effect.source === 'flurry');
      const sum = (items, field = 'amount') => items.reduce((total, item) => total + item[field], 0);
      const basePunch = rules.flurry.base + Math.floor(lab.me.attack * rules.flurry.adRatio / 100);
      const rows = lab.units.map(unit => ({
        id: unit.id, stacksAtEnd: rhythmStacks[unit.id] || 0, unlimited: unit.unlimitedHP,
        ...lab.ledger.row(unit.id, lab.tick),
      }));
      const bonus = packets.find(effect => effect.source === 'bonus');
      const breakTarget = effects.some(effect => effect.kind === 'fx' && effect.name === 'ct_break')
        ? lab.units.find(unit => unit.id === bonus.target) : null;
      const breakRaw = breakTarget
        ? Math.floor(breakTarget.max_hp * rules.rhythm.breakPermilleMaxHp / 1000)
          + Math.floor(lab.me.attack * rules.rhythm.breakAdRatio / 100)
        : 0;
      const flurryEffects = lab.records.filter(row => row.frame.tick <= finished.frame.tick).flatMap(row => row.effects);
      return {
        revision: rules.revision, targets: count, attack: lab.me.attack, expectedPunches: rules.flurry.punches,
        punches: flurry.length, flurryRaw: sum(flurry), flurryRhythmRaw: sum(flurry) - basePunch * flurry.length,
        basicRaw: sum(rows, 'raw') - sum(packets), breakRaw, haymakerRaw: (bonus?.amount ?? 0) - breakRaw,
        echoRaw: sum(packets.filter(effect => effect.source === 'echo')),
        comboRaw: sum(rows, 'raw'), comboHp: sum(rows, 'hp'),
        flurryFx: flurryEffects.filter(effect => effect.kind === 'fx').length,
        flurryCues: flurryEffects.filter(effect => effect.kind === 'fx' && effect.name === 'ct_flurry').length,
        flurryRechargeCredit: finished.state.flurry_credited_ticks,
        moved: centre.x !== finalPos.x || centre.y !== finalPos.y,
        rows,
      };
    }, targets);
  } finally {
    await page.close();
  }
}

(async () => {
  const baseline = process.env.CONTENDER_BASELINE_URL;
  const candidate = process.env.CONTENDER_LAB_URL;
  assert.ok(baseline && candidate, 'Set CONTENDER_BASELINE_URL and CONTENDER_LAB_URL to the two Lab servers');
  const browser = await chromium.launch({ channel: 'msedge', headless: true });
  try {
    const comparisons = [];
    for (const targets of [1, 3, 5]) {
      const before = await measure(browser, baseline, targets);
      const after = await measure(browser, candidate, targets);
      for (const result of [before, after]) {
        assert.equal(result.punches, result.expectedPunches);
        assert.equal(result.flurryCues, result.punches);
        assert.equal(result.flurryRechargeCredit, 0);
        assert.ok(result.rows.every(row => row.unlimited));
        assert.ok(result.haymakerRaw > 0, 'A basic after Flurry should deliver Haymaker');
      }
      assert.equal(after.moved, false);
      assert.equal(after.attack, before.attack);
      comparisons.push({ targets, before, after });
    }
    console.log(JSON.stringify(comparisons, null, 2));
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
