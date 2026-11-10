/* Read-only extraction of the installed battlefield and stock actors for Labs.
 * Cached game art stays local; it is never added to the distributable mod pack.
 */
'use strict';
const fs=require('fs'),path=require('path'),crypto=require('crypto'),gx=require('./extract-gamedata.js');
const REVISION=2,hash=b=>crypto.createHash('sha256').update(b).digest('hex');
function ensureWorld(gameDir,output=path.join(__dirname,'game-reference')){
  const bundle=path.join(gameDir,'bundle.game_data'),dump=path.join(gameDir,'mods/tfm2_custom_ai/map_dump.json');
  if(!fs.existsSync(bundle)||!fs.existsSync(dump))throw new Error('A verified game-exported map_dump.json and installed bundle.game_data are required. Play a match with the map exporter before generating this reference.');
  const stat=fs.statSync(bundle),dumpBytes=fs.readFileSync(dump),dumpHash=hash(dumpBytes),manifest=path.join(output,'world.json');
  if(fs.existsSync(manifest)){
    let old;try{old=JSON.parse(fs.readFileSync(manifest,'utf8'));}catch{/* Rebuild an incomplete cache. */}
    const files=old?[...old.layers.map(l=>l.file),...Object.values(old.sprites).map(s=>s.file)]:[];
    if(old?.revision===REVISION&&old.source.bundle_mtime_ms===stat.mtimeMs&&old.source.bundle_size===stat.size&&old.source.map_sha256===dumpHash&&files.every(file=>fs.existsSync(path.join(output,file))))return old;
  }
  fs.mkdirSync(output,{recursive:true});const index=gx.indexBundle(bundle),sources={};
  function asset(key){const entry=index.get(key);if(!entry)throw new Error('Game asset missing: '+key);const bytes=gx.readAsset(bundle,entry);sources[key]={sha256:hash(bytes),bytes:bytes.length,offset:entry.off,type:entry.type};return bytes;}
  const json=key=>JSON.parse(asset(key).toString('utf8').replace(/^\uFEFF/,''));
  const settings=json('asset/base/setting/game_setting'),info=json('asset/base/setting/champion_info'),text=json('asset/base/text/champion').en.description;
  const doc=JSON.parse(dumpBytes.toString('utf8').replace(/^\uFEFF/,''));
  if(doc.walls.length!==30||doc.walls.some(r=>r.length!==30))throw new Error('Unexpected wall grid; refusing to guess a transform.');
  const binary=asset('asset/base/setting/map_setting');
  const gridBytes=Buffer.concat([Buffer.from([30,0,0,0,0,0,0,0]),...doc.walls.map(row=>Buffer.concat([Buffer.from([30,0,0,0,0,0,0,0]),Buffer.from(row.map(Number))]))]);
  const wallOffset=binary.indexOf(gridBytes);
  const sprites={},layers=[];
  for(const name of ['background_5v5','wall_shadow_5v5','bush_shadow_5v5','wall_5v5','bush_5v5','wall_5v5_front']){
    const key='asset/base/aseprite_resources/ingame/5v5/'+name,bytes=asset(key),file='map-'+name+'.png';
    if(bytes.readUInt32BE(16)!==1280||bytes.readUInt32BE(20)!==1280)throw new Error('Map layer changed; registration must be recalibrated: '+name);
    fs.writeFileSync(path.join(output,file),bytes);layers.push({name,file,foreground:name==='wall_5v5_front'});
  }
  const registration=JSON.parse(require('child_process').execFileSync('python',[path.join(__dirname,'../tools/check-game-map-registration.py'),dump,path.join(output,'map-wall_5v5.png')],{encoding:'utf8',windowsHide:true}));
  function sprite(id,key){
    const sheet=asset(key+'#sheet'),meta=json(key+'#anim'),file='sprite-'+id+'.png';fs.writeFileSync(path.join(output,file),sheet);
    sprites[id]={file,key,anims:meta.anims,width:sheet.readUInt32BE(16),height:sheet.readUInt32BE(20)};
  }
  const champions={};
  for(const [id,definition]of Object.entries(info)){
    if(!definition?.stat||!index.has('asset/base/aseprite_resources/champions/'+id+'#anim'))continue;
    sprite('champ-'+id,'asset/base/aseprite_resources/champions/'+id);
    champions[id]={name:text[id]?.name||id,...definition};
  }
  sprite('minions','asset/base/aseprite_resources/UI_aseprite/minion');
  for(const id of ['rhino','mushroom','stump','bee','epic','serpen','blue_tower','red_tower','blue_nexus','red_nexus'])sprite(id,'asset/base/aseprite_resources/ingame/'+id);
  const towerPairs=doc.towers.flatMap(t=>t.pos),positionsInBinary=towerPairs.every(([x,y])=>{const b=Buffer.alloc(16);b.writeBigInt64LE(BigInt(x));b.writeBigInt64LE(BigInt(y),8);return binary.indexOf(b)>=0;});
  const world={revision:REVISION,source:{bundle_size:stat.size,bundle_mtime_ms:stat.mtimeMs,map_sha256:dumpHash,map_source:'Native StableMapCustomizer export; collision registration verified against the installed wall artwork',map_binary_sha256:sources['asset/base/setting/map_setting'].sha256,wall_binary_offset:wallOffset,tower_pairs_present_in_binary:positionsInBinary,binary_schema_status:'Map binary recorded for provenance; raw wall/position patterns were not found and full bincode schema is not claimed decoded'},
    geometry:{width:settings.width,height:settings.height,cell:settings.width/30,columns:30,rows:30,units_per_pixel:1000,art_padding_pixels:160,art_size_pixels:1280,registration,registration_scope:'Game-exported coordinates and 900/900 collision-cell artwork samples; not a live-engine camera proof'},
    doc,settings,champions,sprites,layers,sources};
  fs.writeFileSync(manifest,JSON.stringify(world));return world;
}
module.exports={ensureWorld};
if(require.main===module){const game=gx.findGameDir(process.argv[2]);if(!game)throw new Error('Installed game not found');const w=ensureWorld(game);console.log(JSON.stringify({source:w.source,geometry:w.geometry,champions:Object.keys(w.champions).length,sprites:Object.keys(w.sprites).length}));}
