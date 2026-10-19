"""Restore the retained cosmic Domain. Nearest-neighbour scale, no body changes."""
from pathlib import Path
from PIL import Image,ImageDraw
import importlib.util,json,math
import gojo_anime_art as anime
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'mods/tfm2_jjk/vfx';DOC=ROOT/'docs/gojo-results'
spec=importlib.util.spec_from_file_location('pack',ROOT/'tools/generate-gojo-art.py');pack=importlib.util.module_from_spec(spec);spec.loader.exec_module(pack)
def generate():
    old=Image.open(OUT/'gojo#sheet.png').convert('RGBA');meta=json.loads((OUT/'gojo#anim.fanim').read_text(encoding='utf-8'));frames=[]
    timelines=[];domain_sizes={}
    rules=json.loads((ROOT/'rules/gojo-v2.json').read_text(encoding='utf-8'))
    for mode,radius,source in [('full',rules['domain_radius'],'slow'),('quick',rules['quick_radius'],'fast')]:
        # The original 166px artwork represents the original 76,000-unit radius.
        width=round(166*radius/76000)
        domain_sizes[mode]={'worldRadius':radius,'nativeFramePixels':width}
        for part,tag in [('field','void_'+source),('ring','void_ring_'+source),('cast','void_cast_'+source)]:
            durations=[round(f['duration']*60000) for f in meta['anims'][tag]['frames']]
            timelines.append(f'        ("{mode}","{part}") => &{durations!r},')
            for f in meta['anims'][tag]['frames']:
                r=f['data'];im=old.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
                if part!='cast':im=im.resize((width,width),Image.Resampling.NEAREST)
                im=anime.polish_domain(im,part,len([x for x in frames if x[0]==f'void_{mode}_{part}']))
                frames.append((f'void_{mode}_{part}',im,f['duration']))
                frames.append((f'void_{mode}_{part}{len([x for x in frames if x[0]==f"void_{mode}_{part}"])-1}',im,f['duration']))
    frames.extend(anime.frames());groups={}
    for kind in [14,15,16,17]:
        group=[]
        for n in range(8):
            size=48 if kind in [14,17] else 212;im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=size//2
            if kind==14:
                d.ellipse((c-8,c-8,c+8,c+8),fill=(5,35,110,230),outline=(150,240,255,255),width=2)
                d.ellipse((c-3,c-3,c+3,c+3),fill=(225,255,255,255))
                for q in range(3):
                    a=q*math.tau/3+n*.15;x=c+round(math.cos(a)*17);y=c+round(math.sin(a)*17);d.arc((5,5,42,42),q*120+n*8,q*120+n*8+50,fill=(55,145,255,210),width=2)
            elif kind==17:
                d.ellipse((10,18,38,30),outline=(225,165,70,230),width=2);d.line((12,13,36,35),fill=(255,100,80,255),width=3)
            else:
                radius=96 if kind==15 else 12+n*12
                d.ellipse((c-radius,c-radius,c+radius,c+radius),outline=(165,90,255,220),width=2 if kind==15 else 5)
                if kind==15:
                    d.text((c-22,c-6),'FUSION',fill=(245,230,255,255))
                    for q in range(12):
                        a=q*math.tau/12+n*.03;x=c+round(math.cos(a)*radius);y=c+round(math.sin(a)*radius);d.line((c+(x-c)*.82,c+(y-c)*.82,x,y),fill=(235,150,255,255),width=2)
                else:
                    for q in range(8):
                        a=q*math.tau/8+n*.1;x=c+round(math.cos(a)*radius);y=c+round(math.sin(a)*radius);d.line((c,c,x,y),fill=(140,65,230,180),width=3)
                    d.ellipse((c-8,c-8,c+8,c+8),fill=(250,230,255,255))
            # Compatibility aliases now resolve to the upgraded presentation.
            im=anime.draw_effect({14:'orb',15:'fusion',16:'nuke',17:'guest'}[kind],n) if kind!=17 else anime.status('exhausted',n)
            group.append(im);frames.append((f'fx{kind}',im,.05))
        groups[kind]=group
    # A readable six-second timer uses the same orb artwork and compact digits.
    for seconds in range(1,7):
        timer=[]
        for n,im in enumerate(groups[14]):
            im=im.copy();ImageDraw.Draw(im).text((im.width//2-2,im.height//2+19),str(seconds),fill=(190,240,255,255));frames.extend([(f'orb{seconds}',im,.05),(f'orb{seconds}{n}',im,.05)])
            timer.append(im)
        frames.extend(anime.windows(f'orb{seconds}',timer,24,True))
    source=Image.new('RGBA',(2048,16384));anims={};x=y=row=0
    for tag,im,duration in frames:
        if x+im.width>2048:y+=row+1;x=row=0
        source.paste(im,(x,y));anims.setdefault(tag,{'frames':[]})['frames'].append({'duration':duration,'data':{'x':x,'y':y,'w':im.width,'h':im.height}});x+=im.width+1;row=max(row,im.height)
    atlas,packed=pack.save_packed(source.crop((0,0,2048,y+row)),{'anims':anims},OUT/'gojo_v2')
    rust='''// Generated from retained pre-PR40 animation metadata. Units: 1/1000 tick.
pub fn frame(mode:&str,part:&str,elapsed:usize,expand:usize)->usize {
    let durations:&[usize]=match (mode,part) {
'''+ '\n'.join(timelines)+'''
        _=>return 0,
    };
    let total=durations.iter().sum::<usize>();
    // The warning already expands the ring. Do not replay the small opening sphere
    // inside the open field: quick Void would close before it reached full size.
    let mut time=if part=="field" {
        let opening=durations[..4].iter().sum::<usize>();
        let last=durations.iter().rposition(|d|*d>=12000).unwrap_or(durations.len()-1);
        let hold=durations[4..=last].iter().sum::<usize>();
        opening+elapsed.saturating_sub(expand)*1000%hold
    }
        else {elapsed.min(expand)*total/expand.max(1)};
    for (n,d) in durations.iter().enumerate() {if time<*d {return n;} time-=d;}
    durations.len()-1
}
#[cfg(test)]
mod tests {
    #[test] fn open_domains_use_only_full_size_frames() {
        for (mode,expand,last) in [("full",60,18),("quick",12,10)] {
            assert_eq!(super::frame(mode,"field",expand,expand),4);
            for elapsed in expand..expand+240 {
                assert!((4..=last).contains(&super::frame(mode,"field",elapsed,expand)));
            }
        }
    }
}
'''
    (ROOT/'native/tfm2_custom_ai/src/gojo_v2_art.rs').write_text(rust,encoding='utf-8')
    presentation=anime.metadata()
    anchor_rust='pub fn gesture_offset(tag:&str,elapsed:usize)->[i64;2] {\nlet frames:&[(usize,i64,i64)]=match tag {\n'
    for pose,tags in [('skill1','"gesture_blue"|"gesture_purple"'),('skill2','"gesture_red"'),('ult','"gesture_domain"')]:
        values=[(v['ticks1000'],*v['offset']) for v in presentation['poseFrames'][pose]];anchor_rust+=f'{tags}=>&{values!r},\n'
    anchor_rust+='_=>return [0,0]}; let mut t=elapsed*1000%frames.iter().map(|v|v.0).sum::<usize>(); for (d,x,y) in frames {if t<*d {return [*x,*y];} t-=d;} [0,0]\n}\n'
    (ROOT/'native/tfm2_custom_ai/src/gojo_visual_data.rs').write_text('// Generated from rules/gojo-visuals.json and unchanged body frame metadata.\n'+f'pub const VERSION:usize={anime.RULES["version"]};\npub const FRAMES:usize={anime.RULES["frames"]};\npub const FRAME_TICKS:usize={anime.RULES["frameTicks"]};\npub const RENEW_TICKS:usize={anime.RULES["limits"]["renewTicks"]};\npub const TAIL_TICKS:usize={anime.RULES["limits"]["tailTicks"]};\npub const PIXEL_UNITS:i64={anime.RULES["worldUnitsPerPixel"]};\n'+anchor_rust,encoding='utf-8')
    (ROOT/'editor/gojo-visuals.json').write_text(json.dumps(presentation,separators=(',',':'))+'\n',encoding='utf-8')
    paths=[ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#sheet.png',OUT/'gojo#sheet.png',OUT/'gojo_rework#sheet.png',OUT/'gojo_v2#sheet.png'];total=sum(Image.open(p).width*Image.open(p).height*4 for p in paths);assert total<=32*1024*1024,total
    contact=Image.new('RGBA',(840,440),(12,22,38,255));d=ImageDraw.Draw(contact)
    for i,tag in enumerate(['void_full_field','void_quick_field','fx14','fx15','fx16','fx17']):
        f=packed['anims'][tag]['frames'][len(packed['anims'][tag]['frames'])//2]['data'];im=atlas.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']));x=(i%3)*280+(280-im.width)//2;y=(i//3)*220+8;contact.alpha_composite(im,(x,y));d.text(((i%3)*280+24,(i//3)*220+205),tag,fill='white')
    contact.save(DOC/'v2-contact-sheet.png');animation=[]
    for n in range(8):
        im=Image.new('RGBA',(636,244),(12,22,38,255));d=ImageDraw.Draw(im)
        for i,k in enumerate([14,15,16]):im.alpha_composite(groups[k][n],(i*212+(212-groups[k][n].width)//2,0));d.text((i*212+15,222),['Maximum Blue orb','Fusion warning','Hollow Nuke'][i],fill='white')
        animation.append(im.convert('RGB'))
    animation[0].save(DOC/'v2-skills.gif',save_all=True,append_images=animation[1:],duration=75,loop=0)
    # Primary review at actual size and 3x, with the unchanged sprite beside each effect.
    bm=json.loads((ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#anim.fanim').read_text());bs=Image.open(ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#sheet.png').convert('RGBA')
    q=bm['anims']['idle']['frames'][0]['data'];body=bs.crop((int(q['x']),int(q['y']),int(q['x']+q['w']),int(q['y']+q['h'])))
    tags=['six_eyes','blue_impact','red_impact','purple_prepare','purple_flight','black_flash','void_full_field','void_quick_field']
    def crop(tag,n):
        f=packed['anims'][tag]['frames'][n%len(packed['anims'][tag]['frames'])]['data'];return atlas.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']))
    animation=[]
    for n in range(16):
        sheet=Image.new('RGBA',(1000,570),(15,24,38,255));d=ImageDraw.Draw(sheet)
        for i,tag in enumerate(tags):
            x=i%4*250;y=i//4*285;d.rectangle((x,y,x+249,y+270),fill=(15,24,38,255));im=crop(tag,(n%8)+4 if tag.startswith('void_') else n%8)
            sheet.alpha_composite(body,(x+15,y+110));sheet.alpha_composite(im,(x+135-im.width//2,y+125-im.height//2));d.text((x+8,y+247),tag,fill='white')
            if i<6:
                detail=im.resize((im.width*2,im.height*2),Image.Resampling.NEAREST);sheet.alpha_composite(detail,(x+130-detail.width//2,y+130-detail.height//2))
        animation.append(sheet.convert('RGB'))
    animation[3].save(DOC/'anime-primary-contact.png');animation[0].save(DOC/'anime-primary.gif',save_all=True,append_images=animation[1:],duration=100,loop=0)
    contact=Image.new('RGBA',(8*150,7*150),(17,27,43,255));d=ImageDraw.Draw(contact)
    review=anime.RULES['variants']+['gesture_'+t for t in anime.RULES['gestures']]+anime.RULES['statuses']
    for i,tag in enumerate(review):
        x=i%8*150;y=i//8*150;im=crop(tag,4)
        if im.width>140:im=im.resize((130,130),Image.Resampling.NEAREST)
        contact.alpha_composite(im,(x+75-im.width//2,y+67-im.height//2));d.text((x+3,y+132),tag,fill='white')
    contact.save(DOC/'anime-all-skills.png')
    report={'decodedBytesIncludingV1Compatibility':total,'v2Atlas':atlas.size,'cosmicSource':'Retained pre-PR40 frames, nearest-neighbour radius scaling, interior-only star polish','domainSizes':domain_sizes,'bodyPixelsChanged':False,'presentationVersion':anime.RULES['version'],'variants':len(review)};(DOC/'v2-art.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
if __name__=='__main__':generate()
