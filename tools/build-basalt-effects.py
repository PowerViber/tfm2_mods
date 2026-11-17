"""Pack approved Basalt effects and generate shared native/Lab bindings. Never installs."""
from pathlib import Path
import hashlib,json,math,shutil
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/basalt-art-review'
A=json.loads((ROOT/'rules/basalt-art.json').read_text());R=json.loads((ROOT/'rules/basalt.json').read_text());APP=json.loads((OUT/'approvals.json').read_text());M=json.loads((OUT/'manifest.json').read_text())
N=Image.Resampling.NEAREST
SCALE=A['world_units_per_pixel']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert APP['effects']['approved'] and sha(OUT/APP['effects']['source'])==APP['effects']['sha256']
frozen={'source':sha(OUT/'effects-source-v1.png'),'approved_review':APP['effects']['approved_review']}
fp=OUT/'effects-frozen-sources.json'
if fp.exists():assert json.loads(fp.read_text())==frozen,'Approved source changed'
save(fp,frozen)
source=Image.open(OUT/'effect-proposals.png').convert('RGBA');materials={};anims={};layers={}
for name,size in A['materials'].items():
    alias={'hurl_impact':'slam','clap_impact':'wall_slam'}.get(name,name)
    r=M['effect_frames'][alias];f=source.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    f=f.resize(tuple(size),N);materials[name]=f;layers[name]=-2 if name in A['ground_tags'] else 5
    # Persistent materials are one frame; only accepted one-shot impacts animate.
    if name in ['block','square','boulder','meteor','whip_link']:anims[name]=[(f,1)]
    else:
        seq=[]
        for scale in [.72,.91,1,.84]:
            im=f.resize((max(1,round(f.width*scale)),max(1,round(f.height*scale))),N);tile=Image.new('RGBA',f.size);tile.alpha_composite(im,((f.width-im.width)//2,(f.height-im.height)//2));seq.append((tile,6))
        anims[name]=seq

cone_offsets=[];far=M['clap_geometry']['far']/SCALE;near=M['clap_geometry']['near']/SCALE;half=R['clap']['coneDeg']/2
for h in range(A['cone_headings']):
    radius=math.ceil(far)+4;centre=radius;canvas=Image.new('RGBA',(radius*2,radius*2));d=ImageDraw.Draw(canvas);angle=h*360/A['cone_headings']
    def point(r,a):return (centre+math.cos(math.radians(a))*r,centre+math.sin(math.radians(a))*r)
    outer=[point(far,angle-half+i*2*half/40) for i in range(41)];inner=[point(near,angle+half-i*2*half/40) for i in range(41)]
    d.polygon(outer+inner,fill=(229,107,28,45));d.line([(centre,centre),*outer,(centre,centre)],fill=(255,196,82,220),width=1)
    box=canvas.getbbox();box=(max(0,box[0]-2),max(0,box[1]-2),min(canvas.width,box[2]+2),min(canvas.height,box[3]+2));tile=canvas.crop(box)
    cone_offsets.append([round((tile.width/2-(centre-box[0]))*SCALE),round((tile.height/2-(centre-box[1]))*SCALE)])
    name='clap_cone_'+str(h);anims[name]=[(tile,1)];layers[name]=-3

# Technical tiling of the approved stone-link strip. The complete whip drawings
# stay on the body; these straight strips connect authoritative live endpoints.
strip=materials['whip_link'].resize((48,5),N)
for length in A['whip_lengths']:
    span=max(8,round(length/SCALE));line=Image.new('RGBA',(span,7))
    for x in range(0,span,48):line.alpha_composite(strip,(x,1))
    for h in range(A['whip_headings']):
        rotated=line.rotate(-h*360/A['whip_headings'],N,expand=True);tile=Image.new('RGBA',(rotated.width+4,rotated.height+4));tile.alpha_composite(rotated,(2,2))
        name=f'whip_{length}_{h}';anims[name]=[(tile,1)];layers[name]=6
for name,radius in [('boulder_warning',R['ult']['radius']),('meteor_area_warning',R['ult']['meteorSpread']+R['ult']['meteorHitRadius']),('meteor_warning',R['ult']['meteorHitRadius']),('hurl_warning',R['hurl']['radius'])]:
    side=math.ceil(radius*2/SCALE)+6;im=Image.new('RGBA',(side,side));d=ImageDraw.Draw(im);d.ellipse((3,3,side-4,side-4),outline=(255,185,98,235),width=1);anims[name]=[(im,1)];layers[name]=-3
anims['boulder_impact']=anims['slam'];layers['boulder_impact']=-2

# The host does NOT stretch Animation effects using play_view_effect(time).
# Reuse exactly the same rectangles/pixels, with explicit matching lifetimes.
# Keep the original names as compatibility aliases (one tick for lease art).
lease_tags=[tag for tag,seq in anims.items() if len(seq)==1]
base_ticks={tag:sum(t for _,t in seq) for tag,seq in anims.items()}
for tag,seq in list(anims.items()):
    # Only create durations the presenter uses; directional strips need no
    # 2/3-tick aliases, cones no 6-tick alias. Pixels remain shared either way.
    allowed = ([1,6] if tag.startswith('whip_') else
               A['lease_ticks'] if tag=='square' else
               [1,2,3] if tag.startswith('clap_cone_') or tag in ['boulder','meteor'] else
               A['lease_ticks']) if tag in lease_tags else A['event_ticks']
    for ticks in allowed:
        if ticks==base_ticks[tag]:continue
        # Cumulative integer boundaries preserve all drawings in their order.
        total=base_ticks[tag];elapsed=previous=0;timed=[]
        for image,duration in seq:
            elapsed+=duration;boundary=elapsed*ticks//total
            assert boundary>previous
            timed.append((image,boundary-previous));previous=boundary
        alias=f'{tag}_t{ticks}';anims[alias]=timed;layers[alias]=layers[tag]

# Shelf packing at 2048; de-duplicate aliases. Every local rectangle is validated.
x=y=row=0;unique={};meta={};width=2048
for tag,seq in anims.items():
    frames=[]
    for im,ticks in seq:
        key=(im.size,im.tobytes())
        if key not in unique:
            if x+im.width>width:x=0;y+=row+2;row=0
            assert y+im.height<=2048,(tag,x,y,im.size)
            rect={'x':x,'y':y,'w':im.width,'h':im.height};unique[key]=(im,rect);x+=im.width+2;row=max(row,im.height)
        frames.append({'duration':ticks/60,'data':unique[key][1]})
    meta[tag]={'frames':frames}
height=2**math.ceil(math.log2(y+row+2));sheet=Image.new('RGBA',(width,height))
for im,r in unique.values():sheet.alpha_composite(im,(r['x'],r['y']))
sheet.save(OUT/'effects.png');save(OUT/'effects.fanim',{'anims':meta})
for src,dest in [('effects.png','basalt#sheet.png'),('effects.fanim','basalt#anim.fanim')]:shutil.copyfile(OUT/src,ROOT/'mods/tfm2_custom/vfx'/dest)
lifetimes={tag:sum(t for _,t in seq) for tag,seq in anims.items()}
data={**A,'approved':True,'effects_file':'../mods/tfm2_custom/vfx/basalt%23sheet.png','animations':meta,'layers':layers,'lifetimes':lifetimes,'lease_tags':lease_tags,'expiry':'exclusive','cone_offsets':cone_offsets,'atlas_sha256':sha(OUT/'effects.png'),'source_sha256':frozen['source'],'decoded_bytes':width*height*4,'generation_deterministic':False,'packing_deterministic':True}
assert data['decoded_bytes']+json.loads((OUT/'body-presentation.json').read_text())['decoded_body_bytes']<=32*1024*1024
save(OUT/'effects-presentation.json',data);(ROOT/'editor/basalt-effects.js').write_text('window.BASALT_FX='+json.dumps(data,separators=(',',':'))+';\n',encoding='utf-8')
lines=['//! Generated by tools/build-basalt-effects.py from rules/basalt-art.json.','#![allow(dead_code)]']
for name,key in [('CONE_HEADINGS','cone_headings'),('WHIP_HEADINGS','whip_headings'),('MAX_EMISSIONS','max_lease_emissions'),('MAX_CUES','max_cues'),('DYNAMIC_PERIOD','dynamic_period'),('WHIP_PERIOD','whip_period'),('WALL_PERIOD','standing_wall_period'),('GROUND_LIFT','ground_boulder_lift'),('AIR_LIFT','air_boulder_lift'),('ARC','boulder_arc')]:lines.append(f'pub const {name}: i64 = {A[key]};')
lines.append('pub const WHIP_LENGTHS: [i64; '+str(len(A['whip_lengths']))+'] = '+str(A['whip_lengths'])+';')
lines.append('pub const CONE_OFFSETS: [(i64,i64); '+str(len(cone_offsets))+'] = ['+','.join(f'({x},{y})' for x,y in cone_offsets)+'];')
lines.append('pub const LEASE_TICKS: &[u64] = &'+str(A['lease_ticks'])+';')
lines.append('pub fn lease_ticks(remaining: u64) -> u64 { LEASE_TICKS.iter().rev().copied().find(|t| *t <= remaining).unwrap_or(0) }')
lines.append('pub fn effect_ticks(name: &str) -> Option<u64> { match name.strip_prefix("bs_").unwrap_or(name) {')
for tag,ticks in lifetimes.items():lines.append(f'    "{tag}" => Some({ticks}),')
lines.append('    _ => None,\n} }')
lines.append('pub fn timed_name(name: &str, ticks: u64) -> String { if effect_ticks(name) == Some(ticks) { name.to_owned() } else { format!("{name}_t{ticks}") } }')
(ROOT/'native/tfm2_custom_ai/src/basalt_art_data.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8')
print(json.dumps({'approved':True,'tags':len(meta),'atlas':[width,height],'decoded_bytes':data['decoded_bytes'],'sha256':data['atlas_sha256']}))
