"""Frozen-source supplemental presentation. Native integration is approval-gated.

Legacy Thalassa frames are packed after symmetric transparent cropping, with
exact RGBA reconstruction checks at their original world centres.
"""
from pathlib import Path
from io import BytesIO
import hashlib,json,math,runpy,subprocess,sys
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/round-165-art-review'
BASE='0d7ec96587ba576dc91d9022be810cedfc3cf4da';ASSET='asset/tfm2_custom/vfx/round_165';N=Image.Resampling.NEAREST
APP=json.loads((OUT/'approval.json').read_text());INTEGRATE='--integrate' in sys.argv
LENGTHS=[8,16,24,32,48,64,80,96,112];DIRECTIONS=32
prep=runpy.run_path(str(ROOT/'tools/prepare-round-165-art.py'));fit=prep['fit']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def blob(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def basejson(p):return json.loads(blob(p))
def unpack(sheet,f):
    r=f['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def pack(anims,width=2048):
    unique={};metadata={}
    for name,frames in anims.items():
        metadata[name]=[]
        for im,duration in frames:
            key=(im.size,im.tobytes());unique.setdefault(key,im);metadata[name].append((key,duration))
    items=sorted(unique.items(),key=lambda kv:(-kv[1].height,-kv[1].width,hashlib.sha256(kv[0][1]).digest()))
    placements={};x=y=row=0
    for key,im in items:
        if x+im.width>width:x=0;y+=row+2;row=0
        assert y+im.height<=2048,('Atlas exceeds 2048',y,im.size)
        placements[key]=dict(x=x,y=y,w=im.width,h=im.height);x+=im.width+2;row=max(row,im.height)
    sheet=Image.new('RGBA',(width,y+row+2))
    for key,im in items:r=placements[key];sheet.paste(im,(r['x'],r['y']))
    return sheet,{'anims':{n:{'frames':[{'duration':d,'data':placements[k]} for k,d in fs]} for n,fs in metadata.items()}}
def compact_legacy():
    prefix='mods/tfm2_custom/vfx/thalassa';sheet=Image.open(BytesIO(blob(prefix+'#sheet.png'))).convert('RGBA');meta=basejson(prefix+'#anim.fanim');anims={};checked=0
    for name,a in meta['anims'].items():
        anims[name]=[]
        for f in a['frames']:
            im=unpack(sheet,f);box=im.getbbox(alpha_only=False)
            if box:
                dx=min(box[0],im.width-box[2]);dy=min(box[1],im.height-box[3]);crop=im.crop((dx,dy,im.width-dx,im.height-dy))
            else:dx=im.width//2-1;dy=im.height//2-1;crop=im.crop((dx,dy,im.width-dx,im.height-dy))
            back=Image.new('RGBA',im.size);back.paste(crop,(dx,dy));assert back.tobytes()==im.tobytes(),('Legacy pixel changed',name)
            anims[name].append((crop,f['duration']));checked+=1
    return pack(anims,1024),checked
def main():
    frozen=json.loads((OUT/'frozen-sources.json').read_text())
    for name,h in frozen.items():assert sha(OUT/name)==h,name
    if INTEGRATE:
        assert APP['effects'].get('approved'), 'New effect approval required before integration'
        assert APP['effects']['source_sha256']==frozen,'Approval does not cover these sources'
    waves,icons=prep['materials']();anims={n:[(im,1)] for n,im in icons.items()};bindings={'akane':[],'wright':[],'thalassa':[]}
    for n in range(1,6):bindings['akane'].append(dict(kind='buff',name='akn_rhythm:'+str(n),tag='rhythm'+str(n),z=8))
    bindings['akane'] += [dict(kind='buff',name='akn_ambush',tag='ambush',z=4),dict(kind='buff',name='akn_soul',tag='dismiss_aura',z=3)]
    bindings['wright'].append(dict(kind='buff',name='wrt_guard',tag='carrier_empowered',z=8))
    # Four readable phases retain the original 18-tick one-shot lifecycle.
    for length in LENGTHS:
        for direction in range(DIRECTIONS):
            seq=[]
            for i,t in zip([0,2,5,7],[4,5,5,4]):
                frame=fit(waves[i],(length,15)).rotate(-direction*360/DIRECTIONS,N,expand=True)
                pad=Image.new('RGBA',(frame.width+2,frame.height+2));pad.paste(frame,(1,1));seq.append((pad,t/60))
            name=f'tsunami_l{length}_d{direction}';anims[name]=seq;bindings['thalassa'].append(dict(kind='effect',name='tls_'+name,tag=name,z=2))
    # Reuse approved Wright pixels at the current radius and exact warning times.
    wm=basejson('docs/wright-art-review/art-manifest.json');wi=Image.open(BytesIO(blob('mods/tfm2_custom/vfx/wright#sheet.png'))).convert('RGBA')
    wr=json.loads((ROOT/'rules/wright.json').read_text());diam=round(wr['bomber']['radius']*2/950)+1;aliases={}
    for index in range(1,4):
        ticks=wr['bomber']['warnTicks']+(index-1)*wr['bomber']['waveEveryTicks'];source=['warn1_t60','warn2_t78','warn3_t96'][index-1];name=f'wright_warn{index}_t{ticks}';seq=[]
        for i,f in enumerate(wm['effects'][source]['frames']):
            im=unpack(wi,f).resize((diam,diam),N);ts=ticks//8+(i<ticks%8);seq.append((im,ts/60))
        anims[name]=seq;aliases[f'wrt_warn{index}:{ticks}']=name;bindings['wright'].append(dict(kind='effect',name='wrt_'+name,tag=name,z=-2))
    for ticks in [14,16,18]:
        name=f'wright_bomb_t{ticks}';seq=[]
        for i,f in enumerate(wm['effects'][f'bomb_t{ticks}']['frames']):seq.append((unpack(wi,f).resize((diam,diam),N),(ticks//8+(i<ticks%8))/60))
        anims[name]=seq;aliases['wrt_bomb:'+str(ticks)]=name;bindings['wright'].append(dict(kind='effect',name='wrt_'+name,tag=name,z=4))
    sheet,meta=pack(anims);sheet.save(OUT/'supplement.png');(OUT/'supplement.fanim').write_text(json.dumps(meta,separators=(',',':'))+'\n')
    (legacy,lmeta),checked=compact_legacy();legacy.save(OUT/'thalassa-compact.png');(OUT/'thalassa-compact.fanim').write_text(json.dumps(lmeta,separators=(',',':'))+'\n')
    legacy_bytes=legacy.width*legacy.height*4
    body=Image.open(BytesIO(blob('mods/tfm2_custom/champions/tfm2_custom_thalassa#sheet.png')));links=Image.open(BytesIO(blob('mods/tfm2_custom/vfx/thalassa_links#sheet.png')))
    budgets={'thalassa':legacy_bytes+body.width*body.height*4+links.width*links.height*4+sheet.width*sheet.height*4,
             'akane':basejson('docs/akane-art-review/art-manifest.json')['decoded_bytes']+sheet.width*sheet.height*4,
             'wright':wm['decoded_bytes']+sheet.width*sheet.height*4}
    assert all(v<=32*1024*1024 for v in budgets.values()),budgets
    manifest={'version':1,'revision':'round-165-art-1','integrated':INTEGRATE,'asset':ASSET,'frames':meta['anims'],'bindings':bindings,'wright_aliases':aliases,'wave':{'lengths':LENGTHS,'directions':DIRECTIONS,'units_per_pixel':950,'duration_ticks':18,'max_segments':16},'source_sha256':frozen,'atlas_size':sheet.size,'atlas_sha256':sha(OUT/'supplement.png'),'decoded_bytes_per_owner':budgets,'legacy_frames_reconstructed_exactly':checked,'baseline':BASE}
    (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (ROOT/'editor/round-165-art.js').write_text('window.ROUND165_ART='+json.dumps(manifest,separators=(',',':'))+';\n')
    if INTEGRATE:
        for image,metadata,stem in [(sheet,meta,'round_165'),(legacy,lmeta,'thalassa')]:
            target=ROOT/'mods/tfm2_custom/vfx';image.save(target/(stem+'#sheet.png'));(target/(stem+'#anim.fanim')).write_text(json.dumps(metadata,separators=(',',':'))+'\n')
        # Frame/world centres and durations are unchanged; only atlas coordinates
        # and symmetric transparent padding are replaced.
        tm=basejson('docs/thalassa-art-review/art-manifest.json')
        for name,a in lmeta['anims'].items():
            if name in tm['effects']:tm['effects'][name]['frames']=a['frames']
        tm['decoded_bytes']=budgets['thalassa'];tm['r165_supplement']=True
        (ROOT/'docs/thalassa-art-review/art-manifest.json').write_text(json.dumps(tm,indent=2)+'\n');(ROOT/'editor/thalassa-art.js').write_text('window.THALASSA_ART='+json.dumps(tm,separators=(',',':'))+';\n')
    # One canonical selector is generated for both adapters. No gameplay writes.
    cfg=manifest['wave'];rust=['//! Generated by tools/build-round-165-art.py. Presentation selection only.','#![allow(dead_code)]',
      'pub fn wave_parts(from:(i64,i64),mid:(i64,i64))->Vec<(String,i64,i64)>{',
      ' let dx=(mid.0-from.0) as f64*2.;let dy=(mid.1-from.1) as f64*2.;let dist=dx.hypot(dy);if dist<1.{return vec![];}',
      f' let dir=((dy.atan2(dx)*{DIRECTIONS}. / std::f64::consts::TAU).round() as i64).rem_euclid({DIRECTIONS});',
      f' let n=((dist/{max(LENGTHS)*950}.).ceil() as usize).clamp(1,16);let span=dist/n as f64/950.;let sizes:&[i64]=&{LENGTHS};',
      ' let len=*sizes.iter().min_by_key(|v|((**v as f64-span).abs()*1000.) as i64).unwrap();',
      ' (0..n).map(|i|{let t=(i as f64+0.5)/n as f64;(format!("tsunami_l{len}_d{dir}"),(from.0 as f64+dx*t).round() as i64,(from.1 as f64+dy*t).round() as i64)}).collect()', '}',
      'pub fn wright_effect(name:&str,ticks:u64)->Option<&\'static str>{match(name,ticks){']
    for key,tag in aliases.items():n,t=key.split(':');rust.append(f' ("{n}",{t})=>Some("wrt_{tag}"),')
    rust+=[' _=>None,}}']
    (ROOT/'native/tfm2_custom_ai/src/round_165_art.rs').write_text('\n'.join(rust)+'\n')
    print(json.dumps({'integrated':INTEGRATE,'aux_atlas':sheet.size,'legacy_compact_atlas':legacy.size,'legacy_frames_equal':checked,'decoded_bytes':budgets}))
if __name__=='__main__':main()
