// Rift Lab parity: the editor's JS port must reproduce the native vectors exactly.
//   node tools/check_vesper_parity.js        (math vectors + simulation vectors)
const fs = require('fs');
const path = require('path');
const root = path.resolve(__dirname, '..');
const M = require(path.join(root, 'editor/vesper-math.js'));
let failures = 0;
function check(label, got, want) {
  if (String(got) !== String(want)) {
    failures++;
    if (failures < 20) console.log(`MISMATCH ${label}\n  js:     ${got}\n  native: ${want}`);
  }
}
const math = fs.readFileSync(path.join(root, 'native/tfm2_custom_ai/src/vesper_vectors.txt'), 'utf8').trim().split(/\r?\n/);
for (const row of math) {
  const f = row.split('|');
  if (f[0] === 'chain') check(row, M.chainBp(f[1].split(',').map(Number)), f[2]);
  else if (f[0] === 'delay') check(row, M.openDelay(+f[1], +f[2]), f[3]);
  else if (f[0] === 'modules') check(row, M.modulesOf(+f[1]).join(','), f[2]);
  else if (f[0] === 'slots') check(row, M.hexSlots(+f[1]), f[2]);
  else if (f[0] === 'rng') {
    const st = M.seedState(BigInt(f[1]));
    check(row, Array.from({ length: 12 }, () => M.roll(st, 1000)).join(','), f[2]);
  }
}
console.log(`math vectors: ${math.length} rows, ${failures} mismatches`);
const simFile = path.join(root, 'native/tfm2_custom_ai/src/vesper_sim_vectors.txt');
if (fs.existsSync(path.join(root, 'editor/vesper-combat.js'))) {
  const V = require(path.join(root, 'editor/vesper-combat.js'));
  const want = fs.readFileSync(simFile, 'utf8').replace(/\r\n/g, '\n');
  const got = V.simVectors();
  const a = got.split('\n'), b = want.split('\n');
  let sim = 0;
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if (a[i] !== b[i]) { sim++; if (sim <= 5) console.log(`SIM MISMATCH line ${i + 1}\n  js:     ${a[i]}\n  native: ${b[i]}`); }
  }
  failures += sim;
  console.log(`simulation vectors: ${b.length - 1} rows, ${sim} mismatches`);
}
process.exit(failures ? 1 : 0);
