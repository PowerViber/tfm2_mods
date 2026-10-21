/* Catalogue and integration checks. These do not claim stock casts were executed. */
const fs=require('fs'),assert=require('assert'),path=require('path'),cp=require('child_process');
const root=path.resolve(__dirname,'..'),p=path.join(root,'rules/farming-capabilities.json'),before=fs.readFileSync(p,'utf8');
cp.execFileSync('rtk',['proxy','node','tools/generate-farming-capabilities.cjs'],{cwd:root});assert.equal(fs.readFileSync(p,'utf8'),before);
const d=JSON.parse(before);assert.equal(d.champions.length,77);assert.equal(d.champions.filter(c=>c.source==='stock').length,60);
const rows=[];for(const c of d.champions){assert.equal(c.skills.length,3);for(const s of c.skills){assert(s.resourceRule&&s.reserve&&s.execution);assert(s.verification.eligible&&s.verification.selected&&s.verification.executed&&s.verification.delivered);
 for(let team=0;team<2;team++)for(let role=0;role<5;role++)for(const target of ['lane_minion','summon','ordinary_camp','morgard','serpen']){
  const compatible=s.targets.includes(target),roleAllowed=target!=='ordinary_camp'||role===1;rows.push({champion:c.id,skill:s.slot,team,role,target,metadataCompatible:compatible,roleAllowed,executionObserved:false});
 }
 if(!s.purpose.includes('offense'))assert.equal(s.targets.length,0);
}}
global.window={};require('../editor/native-tactics-presets.js');require('../editor/steve-preset.js');const presets=require('../editor/presets.js');
for(const slug of ['frieren','dio','steve']){const p=presets[slug].build('test_id',{modId:'test'}).json;assert(p.passive.passive_ref.endsWith(slug==='dio'?'dio_v2':slug==='steve'?'steve_craft':'frieren'));if(slug==='frieren')assert(JSON.stringify(p.attack).includes('frieren_basic_damage'));if(slug==='dio')assert(JSON.stringify(p.skill).includes('dio_farm_knife'));}
const out=path.join(root,'docs/creative-results');fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'farming-catalogue.json'),JSON.stringify({champions:77,skills:231,metadataRoleTeamTargetCombinations:rows.length,stockRoute:'engine owned; no input injection',stockExecutionObserved:0,nativeExecution:'See shared-core/lab/ABI fixtures; live attribution is pending',verified:['deterministic canonical generation','complete roster metadata','resource-respecting native presets','role/target catalogue combinations'],limitation:'Compatibility, selection, cast acceptance and delivered contribution are separate. This test verifies catalogue/presets, not successful in-game casts.'},null,2)+'\n');console.log(JSON.stringify({champions:77,skills:231,metadataCombinations:rows.length,liveCastsClaimed:0}));
