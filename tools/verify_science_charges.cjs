const fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const {ChargeBank,parseDiagnostics}=require('../editor/science-charge-model.js');
const binary=path.join(root,'native/tfm2_custom_ai/target/science-charge-parity.exe');
const build=cp.spawnSync('rustup',['run','stable-x86_64-pc-windows-gnu','rustc',path.join(__dirname,'science_charge_parity.rs'),'-o',binary],{encoding:'utf8'});
assert.equal(build.status,0,build.stderr);
const native=cp.spawnSync(binary,[],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);
const bank=new ChargeBank(),rows=[];let held=null;
for(let tick=0;tick<2400;tick++){
 bank.advance(tick);
 if(tick%17===0){const slot=bank.reserve(tick);if(slot!==null){if(tick%51===0)bank.cancel(slot);else assert.ok(bank.commit(slot,tick));}}
 if(tick%47===0)held=bank.reserve(tick);
 if(tick%47===1&&held!==null){bank.cancel(held);held=null;}
 rows.push(`${tick}|${bank.available()}|${bank.cells(tick).join(',')}`);
}
assert.equal(native.stdout.replace(/\r\n/g,'\n').trim(),rows.join('\n'));
const log='seed 1 tick 0 caster 7 selection reason="direct attack" target 8\nseed 1 tick 38 caster 7 charge_spent slot 0 recovery 338\nseed 1 tick 50 caster 7 release E01 target 8 origin 100000,100000 aim 125000,100000\nseed 1 tick 57 caster 7 target 8 skill E01 experiment 1 raw 100 type magic HP 1000->900 budget 0\nseed 1 tick 338 caster 7 charge_recovered slot 0';
assert.equal(parseDiagnostics(log).length,5);assert.match(parseDiagnostics(log)[3].text,/HP 1000 → 900/);
console.log('PASS: 2400 Rust/JavaScript charge states match exactly across reservation, cancellation, exhaustion and independent recovery; five native diagnostic event types parse safely.');
