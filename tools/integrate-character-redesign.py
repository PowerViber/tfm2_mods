"""Build the approved body batch from frozen imagegen sources; preserve combat and effect clocks."""
from pathlib import Path
from PIL import Image,ImageOps,ImageFilter
import subprocess,io,json,hashlib,importlib.util,copy,inspect,math,base64,sys
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review';BASE='00fe08dbbf2f09c560121e11a1c972dff8d92ad3'
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
B=module('body_tools',ROOT/'tools/original_bodies.py')
def original(rel):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel],cwd=ROOT)
def old_json(rel):return json.loads(original(rel))
def old_image(rel):return Image.open(io.BytesIO(original(rel))).convert('RGBA')
def crop(im,f):
 r=f['data'];return im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def digest(im):return hashlib.sha256(str(im.size).encode()+im.tobytes()).hexdigest()
def pack(anims):
 """Deduplicate exact frame rectangles and use deterministic MaxRects packing."""
 unique={}
 for a in anims.values():
  for im,t in a:unique[digest(im)]=im
 for order in [lambda p:(-max(p[1].size),-p[1].width*p[1].height,p[0]),lambda p:(-p[1].width*p[1].height,-p[1].height,p[0]),lambda p:(-p[1].height,-p[1].width,p[0])]:
  free=[(0,0,2048,2048)];placed={};failed=False
  for key,im in sorted(unique.items(),key=order):
   choices=[(min(w-im.width,h-im.height),max(w-im.width,h-im.height),y,x,i)for i,(x,y,w,h)in enumerate(free)if im.width<=w and im.height<=h]
   if not choices:failed=True;break
   _,_,y,x,_=min(choices);used=(x,y,im.width,im.height);new=[]
   for fx,fy,fw,fh in free:
    if x>=fx+fw or x+im.width<=fx or y>=fy+fh or y+im.height<=fy:new.append((fx,fy,fw,fh));continue
    if x>fx:new.append((fx,fy,x-fx,fh))
    if x+im.width<fx+fw:new.append((x+im.width,fy,fx+fw-x-im.width,fh))
    if y>fy:new.append((fx,fy,fw,y-fy))
    if y+im.height<fy+fh:new.append((fx,y+im.height,fw,fy+fh-y-im.height))
   free=[]
   for i,r in enumerate(new):
    if r[2]<=0 or r[3]<=0:continue
    if any(i!=j and s[0]<=r[0]and s[1]<=r[1]and s[0]+s[2]>=r[0]+r[2]and s[1]+s[3]>=r[1]+r[3]and(i>j or r!=s)for j,s in enumerate(new)):continue
    free.append(r)
   placed[key]=used
  if not failed:break
 else:raise RuntimeError('Cannot fit lossless frames into a 2048 atlas')
 w=max(x+ww for x,y,ww,h in placed.values());h=max(y+hh for x,y,ww,hh in placed.values());sheet=Image.new('RGBA',(w,h))
 for key,(x,y,ww,hh)in placed.items():sheet.alpha_composite(unique[key],(x,y))
 meta={'anims':{name:{'frames':[{'duration':t,'data':dict(zip(['x','y','w','h'],placed[digest(im)]))}for im,t in frames]}for name,frames in anims.items()}}
 return sheet,meta
SPECS={'vader':('tfm2_starwars','tfm2_starwars_vader','vader-source.png',40),'aegis':('tfm2_custom','tfm2_gundam_aegis_zero','aegis-source.png',46),'scribble':('tfm2_toon','tfm2_toon_scribble','scribble-source-floating.png',45)}
def choose(key,p,m,tag,n,count):
 f=n/max(1,count-1)
 if tag=='dead':return ([m[4],m[5],m[6]]if key=='vader'else[m[5],m[6],m[7]]if key=='aegis'else[p[6],m[6],m[7]])[min(2,n*3//count)]
 if tag=='hit':return m[4]if key=='vader'else m[5]if key=='aegis'else p[6]
 if tag=='run' or tag.endswith('dash'):return ([m[0],p[1],m[1],p[1]]if key!='scribble'else[m[1],p[1],m[2],p[1]])[n%4]
 if tag=='attack':return ([m[2],p[2],m[3],p[0]]if key=='vader'else[p[0],p[2],p[3],p[0]]if key=='aegis'else[p[2],m[3],p[4],p[7]])[min(3,n*4//count)]
 if key=='vader':
  if tag in ['choke','choke_seq','skill2']:return p[5]if count==1 or 0<f<1 else p[0]
  if tag in ['ult','ult_area']:return p[6]
  if tag in ['skill1_pre','skill1_end','skill1_effect']:return p[4]
  return [p[0],m[7],m[7],p[0]][n%4]
 if key=='aegis':
  if tag=='skill1':return [m[2],p[4],m[3],p[0]][min(3,n*4//count)]
  if tag=='skill2':return [m[4],p[3],p[3],p[0]][min(3,n*4//count)]
  if tag=='ult':return p[6]
  return p[0]
 if tag=='skill1':return [p[2],p[3],m[4],p[3]][n%4]
 if tag=='skill2':return p[4]
 if tag=='ult':return p[5]if 0<f<1 else p[3]
 return [p[0],m[0],p[7],m[0]][n%4]
def floating_fit(im,reference,scale):
 im=im.resize((max(1,round(im.width*scale)),max(1,round(im.height*scale))),Image.Resampling.NEAREST)
 hair=[x for y in range(max(1,im.height//3))for x in range(im.width)if im.getpixel((x,y))[3]>=128 and min(im.getpixel((x,y))[:3])>180]
 cx=sum(hair)/max(1,len(hair));bounds=reference.getbbox();x=round(B.foot_x(reference)-cx);y=bounds[3]-im.height-6
 px=max(0,-x,x+im.width-reference.width);py=max(0,-y,y+im.height-reference.height);out=Image.new('RGBA',(reference.width+px*2,reference.height+py*2));out.alpha_composite(im,(x+px,y+py));return out
def enlarge(wing,scale=1.3,size=None):
 small=wing.resize((round(wing.width*scale),round(wing.height*scale)),Image.Resampling.NEAREST)
 w,h=size or(188,168);out=Image.new('RGBA',(w,h));out.alpha_composite(small,(round(w/2-72*scale),round(h/2-8-56*scale)));return out
def main():
 decisions=json.loads((OUT/'decisions.json').read_text(encoding='utf8'));assert all(decisions['revisionFeedback'][k]['appearanceApproved']for k in SPECS)
 outputs={};bodies={};report={'base':BASE,'nativeDllSha256':hashlib.sha256(original('mods/tfm2_custom_ai/tfm2_custom_ai.dll')).hexdigest(),'bodies':{},'effectChanges':[]}
 for key,(mod,id,source,height)in SPECS.items():
  p=B.sprite_library(OUT/source,8);m=B.sprite_library(OUT/(key+'-motion-source.png'),8);scale=height/p[0].height;stem=f'mods/{mod}/champions/{id}';old=old_json(stem+'#anim.fanim')['anims'];sheet=old_image(stem+'#sheet.png');anims={}
  for tag,a in old.items():
   frames=[]
   for n,f in enumerate(a['frames']):
    ref=crop(sheet,f)
    if key=='vader'and tag in ['skill1_effect','ult_area']:
     frames.append((ref,f['duration']));continue
    arc=None
    if key=='vader'and tag=='ult_end':
     body_palette=set()
     for t in ['idle','run','attack','skill1_pre','skill1_dash','skill1_end','hit','dead','choke','choke_seq','skill2']:
      for ff in old[t]['frames']:body_palette.update(crop(sheet,ff).get_flattened_data())
     arc=Image.new('RGBA',ref.size);arc.putdata([c if c not in body_palette else(0,0,0,0)for c in ref.get_flattened_data()]);body_ref=Image.new('RGBA',ref.size);body_ref.putdata([c if c in body_palette else(0,0,0,0)for c in ref.get_flattened_data()]);ref=body_ref
    im=choose(key,p,m,tag,n,len(a['frames']));im=floating_fit(im,ref,scale)if key=='scribble'and tag!='dead'else B.fit_pose(im,ref,scale)[0]
    if arc is not None:
     background=Image.new('RGBA',im.size);background.alpha_composite(arc,((im.width-arc.width)//2,(im.height-arc.height)//2));background.alpha_composite(im);im=background
    if key=='vader'and tag in ['ult','ult_dash']:
     im=ImageOps.expand(im,2);halo=Image.new('RGBA',im.size,(115,109,109,255));halo.putalpha(im.getchannel('A').filter(ImageFilter.MaxFilter(3)));halo.alpha_composite(im);im=halo
    frames.append((im,f['duration']))
   anims[tag]=frames
  atlas,meta=pack(anims);outputs[stem]=(atlas,meta);bodies[key]=anims
  report['bodies'][key]={'animations':len(anims),'frames':sum(len(v)for v in anims.values()),'atlas':list(atlas.size),'decodedBytes':atlas.width*atlas.height*4,'timelinesUnchanged':all([t for _,t in anims[k]]==[f['duration']for f in v['frames']]for k,v in old.items()),'hoverIsCosmetic':key=='scribble'}
 # Replace only body-derived Gundam composites and approved enlarged wings.
 stem='mods/tfm2_custom/vfx/gundam';old=old_json(stem+'#anim.fanim')['anims'];sheet=old_image(stem+'#sheet.png');fx={k:[(crop(sheet,f),f['duration'])for f in a['frames']]for k,a in old.items()}
 g=module('gundam_art',ROOT/'mods/tfm2_gundam/source/build_art.py');oldbody=old_image('mods/tfm2_custom/champions/tfm2_gundam_aegis_zero#sheet.png');bm=old_json('mods/tfm2_custom/champions/tfm2_gundam_aegis_zero#anim.fanim')['anims'];legacy_run=crop(oldbody,bm['skill1']['frames'][2]);legacy_ult=crop(oldbody,bm['ult']['frames'][3]);
 for tag,im in [('after_r',g.afterimage(legacy_run,False,0)),('ascend',g.ascend(0,legacy_ult)),('sky_r',g.sky(0,legacy_run)),('dive',g.dive(0,legacy_ult))]:assert im.tobytes()==fx[tag][0][0].tobytes(),('Legacy generator no longer matches',tag)
 for tag in ['wings_light','wings_fade','deploy','retract','flare']:fx[tag]=[(enlarge(im),t)for im,t in fx[tag]];report['effectChanges'].append(tag)
 # Flight figures retain their body scale. Wider transparent frames protect enlarged wing tips.
 def with_wings(body,wing_scale,f=0,alpha=1.):
  im=enlarge(g.wings_of_light(f,scale=wing_scale,alpha=alpha));im.alpha_composite(body,(im.width//2-24,im.height//2-28));return im
 g.with_wings=with_wings
 for name in ['ascend','dive','sky']:
  src=inspect.getsource(getattr(g,name))
  if name in ['ascend','dive']:src=src.replace('W, H = 160, 300','W, H = 208, 300').replace('cx - 72','cx - fig.width / 2').replace('cy - 64','cy - fig.height / 2')
  else:src=src.replace('W, H = 120,','W, H = 160,').replace('.resize((101, 90)', '.resize((132, 118)').replace('cx - 50','cx - 66').replace('SKY_UP - 45','SKY_UP - 59')
  exec(src,g.__dict__)
 def compact_body(pose):
  # Review anchor (40,72) maps to the original (24,49) foot baseline.
  frame=Image.open(OUT/'aegis'/f'pose-{pose}.png').convert('RGBA');return frame.crop((16,23,64,79))
 run=compact_body(1);ult=compact_body(6)
 for tag,fn in [('after_r',lambda n:g.afterimage(run,False,n)),('after_l',lambda n:g.afterimage(run,True,n)),('ascend',lambda n:g.ascend(n,ult)),('sky_r',lambda n:g.sky(n,run)),('sky_l',lambda n:g.sky(n,run,True)),('dive',lambda n:g.dive(n,ult))]:fx[tag]=[(fn(n),t)for n,(_,t)in enumerate(fx[tag])];report['effectChanges'].append(tag)
 # Centre-preserving trim of transparent padding is lossless in world coordinates.
 trimmed=0
 for tag,frames in fx.items():
  new=[]
  for im,t in frames:
   box=im.getbbox()
   if box:
    tx=min(box[0],im.width-box[2]);ty=min(box[1],im.height-box[3]);trimmed+=int(tx>0 or ty>0);im=im.crop((tx,ty,im.width-tx,im.height-ty))
   new.append((im,t))
  fx[tag]=new
 atlas,meta=pack(fx);outputs[stem]=(atlas,meta);report['gundamVfx']={'atlas':list(atlas.size),'decodedBytes':atlas.width*atlas.height*4,'centrePreservingPaddingTrims':trimmed,'unchangedTags':len(fx)-len(report['effectChanges'])}
 for stem,(atlas,meta)in outputs.items():atlas.save(ROOT/(stem+'#sheet.png'));(ROOT/(stem+'#anim.fanim')).write_text(json.dumps(meta,separators=(',',':'))+'\n')
 path=ROOT/'editor/sprites-data.js';text=path.read_text(encoding='utf8');start=text.index('{');data,end=json.JSONDecoder().raw_decode(text[start:])
 for key,(mod,id,source,height)in SPECS.items():
  stem=ROOT/f'mods/{mod}/champions/{id}';entry=data.setdefault(key,{});entry.update(label={'vader':'Darth Vader','aegis':'Aegis Zero / Destiny','scribble':'Scribble / floating Gear 5'}[key],png=base64.b64encode(Path(str(stem)+'#sheet.png').read_bytes()).decode(),fanim=json.loads(Path(str(stem)+'#anim.fanim').read_text()));entry.pop('anim',None)
 path.write_text(text[:start]+json.dumps(data,separators=(',',':'))+text[start+end:],encoding='utf8')
 path=ROOT/'editor/vfx2.js';text=path.read_text(encoding='utf8');start=text.index('{',text.index('{',text.index('window.TFM2_VFX'))+2);data,end=json.JSONDecoder().raw_decode(text[start:]);stem=ROOT/'mods/tfm2_custom/vfx/gundam';data['gundam']={'png':base64.b64encode(Path(str(stem)+'#sheet.png').read_bytes()).decode(),'fanim':json.loads(Path(str(stem)+'#anim.fanim').read_text())};path.write_text(text[:start]+json.dumps(data,separators=(',',':'))+text[start+end:],encoding='utf8')
 (OUT/'integration.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
