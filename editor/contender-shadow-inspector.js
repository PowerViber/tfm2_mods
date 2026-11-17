(async()=>{try{
const $=id=>document.getElementById(id),world=await new GameWorld().ready,painter=await new ContenderPainter().ready;
await world.loadSprites(['champ-knight','champ-berserker']);
const pose=$('pose');pose.replaceChildren();for(const tag of Object.keys(painter.data.body.anims).filter(n=>!n.endsWith('_l')))pose.add(new Option(tag,tag));
const controls=document.createElement('div');controls.innerHTML='<label>Frame <input id="frame" type="range" min="0" max="5" value="0"></label><label><input id="animate" type="checkbox" checked> Play preview</label><label><input id="light" type="checkbox"> Light terrain</label><p id="frameInfo"></p><label>Effect <select id="effectTag"></select></label><button id="playEffect">Play once</button><canvas id="effectArena" width="1160" height="320"></canvas>';
$('bodyArena').after(controls);for(const tag of Object.keys(painter.data.effects.anims))$('effectTag').add(new Option(tag,tag));
let tick=0,start=performance.now(),effectStart=null;const cv=$('bodyArena'),ctx=cv.getContext('2d'),ec=$('effectArena'),ex=ec.getContext('2d');
function draw(){
 const s=+$('zoom').value,P=p=>({x:580+(p.x-480000)*s,y:235+(p.y-480000)*s}),W=n=>n*s;
 ctx.clearRect(0,0,cv.width,cv.height);world.drawMap(ctx,P,W);if($('light').checked){ctx.fillStyle='#bac5a6';ctx.fillRect(0,0,cv.width,cv.height);}
 const tag=pose.value+($('left').checked?'_l':''),a=painter.data.body.anims[tag],elapsed=$('animate').checked?tick%painter.duration('body',tag):a.frames.slice(0,+$('frame').value).reduce((n,f)=>n+f.duration*60,0);
 $('frame').max=a.frames.length-1;painter.draw(ctx,'body',tag,{x:480000,y:480000},elapsed,P,W,true);
 for(const[sprite,x]of[['champ-knight',480000-110/s],['champ-berserker',480000+110/s]])world.drawUnit(ctx,{sprite,pos:{x,y:480000},hp:100,moving:false},0,P,W);
 $('frameInfo').textContent=`${tag}: ${a.frames.length} frames / ${(painter.duration('body',tag)/60).toFixed(3)} s. Whole-body pixels; foot anchor (40,52).`;
 ex.fillStyle=$('light').checked?'#bac5a6':'#243a42';ex.fillRect(0,0,ec.width,ec.height);
 if(effectStart!==null){const age=tick-effectStart;for(const z of[1,3])painter.draw(ex,'effects',$('effectTag').value,{x:z===1?280:800,y:160},age,p=>p,n=>n*z/1000);}
}
for(const id of['pose','left','zoom','frame','animate','light'])$(id).oninput=draw;
$('playEffect').onclick=()=>{effectStart=tick;};
function loop(now){tick=(now-start)*60/1000;draw();requestAnimationFrame(loop);}requestAnimationFrame(loop);
$('mapStatus').textContent='Extracted map: '+world.geometry.registration.matching+'/900 registered collision cells. Body/effect inspector uses actual packaged assets. Preview playback repeats only when requested; native effects expire.';
window.contenderShadowReview={world,painter,loaded:true,draw};
}catch(e){document.getElementById('error').textContent=e.stack;throw e;}})();
