(async function(){
 'use strict';
 const root=document.getElementById('chessLab'),D=window.MagnusData;
 const ranks=['Beginner','Club Player','Tactician','Expert','Master','Grandmaster','Super GM','World Top 10'];
 const names=['Rook A','Rook B','Bishop A','Bishop B','Knight A','Knight B','Queen'];
 const glyphs=['♜','♝','♞','♛'],roles=['♚','♛','♜','♝','♞'];
 const labels=['','Board ready','Check','CHECKMATE','Fork','Discovered attack','Sacrifice','Stalemate','Brilliant Recall','Interrupted','Castling','Pin'];
 const failures=['The position no longer meets the sequence.','The eight-second completion window expired.','The designated king died, left the board, or became hidden.','The required target died, left the board, or became hidden.','A required piece was captured outside its expected sacrifice step.','Cleanup or a capture without an enemy attacker cannot count as a sacrifice.'];
 const esc=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 root.innerHTML='<div class="mg-shell"><header class="mg-header"><div class="mg-title"><h1>Magnus · Chess Lab</h1><p>Seven pieces. One shared clock. Find the combination.</p></div><span class="mg-badge">Pawn-free adaptations</span><button id="new">New practice</button></header><div class="mg-grid"><section class="mg-card"><h2>The board</h2><div class="mg-toolbar"><label>Mastery <select id="rank">'+ranks.map((r,i)=>'<option value="'+i+'"'+(i===3?' selected':'')+'>'+r+'</option>').join('')+'</select></label><label>Seed <input id="seed" type="number" value="42" min="0"></label><label>Forward <select id="heading"><option value="0">North</option><option value="1">East</option><option value="2">South</option><option value="3">West</option></select></label></div><canvas class="mg-canvas" id="board" width="600" height="570" aria-label="Chessboard: select a piece, then click its destination"></canvas><div class="mg-status" id="status" aria-live="polite"></div><div class="mg-toolbar"><button id="run">▶ Play</button><button id="step">+ 0.5 s</button><button id="deploy">Deploy / reset</button><button id="castle">Castle with selected rook</button><button id="dismiss">Dismiss</button></div><div class="mg-roster" id="roster"></div><div class="mg-toolbar"><label><input type="checkbox" id="automatic"> Athlete planner</label><label><input type="checkbox" id="coverage" checked> Coverage</label><label><input type="checkbox" id="motion"> Moving enemies</label><label><input type="checkbox" id="combat"> Enemy attacks pieces</label></div><div class="mg-toolbar"><label>Canvas action <select id="mode"><option value="command">Move selected piece</option><option value="magnus">Move Magnus freely</option><option value="king">Move enemy king</option><option value="queen">Move enemy queen</option><option value="wall">Toggle terrain</option><option value="setup">Place piece (setup)</option></select></label><button id="capture">Enemy captures piece</button><button id="fog">Hide / reveal king</button><button id="interrupt">Stun / silence (0.5 s)</button></div><div class="mg-toolbar"><button id="death">Magnus dies</button><button id="respawn">Respawn</button><label>Terrain <select id="terrain"><option value="empty">Open</option><option value="cross">Cross wall</option><option value="edge">Board-edge exits</option></select></label></div><p class="mg-small">Select a piece below the board, then choose a square. Commands show a warning before moving. Champions move freely. Setup changes affect this practice only.</p></section><aside><section class="mg-card"><h2>Famous-game challenge</h2><div class="mg-toolbar"><select id="history">'+D.games.map((g,i)=>'<option value="'+i+'">'+esc(g.title)+'</option>').join('')+'</select><button id="demo">Load sequence</button></div><div id="challenge"></div><div class="mg-toolbar"><button id="replay">▶ Replay sequence</button><button id="choose">Practise this challenge</button></div><details><summary>How the adaptation works</summary><p>No combat pawns. The lab keeps the historical reference moves separate from the combat sequence. Enemy roles follow visible champions, and each successful selected challenge grants the same five-second speed buff. The Opera accepts any enemy capture of the queen in matches.</p><p>The Marathon uses the rook to protect the knight, with knight coverage of queen escape squares on the opening and closing rook strikes. The queen moves after the knight lands on it.</p></details></section><section class="mg-card" style="margin-top:16px"><h2>Position and clock</h2><div id="metrics"></div><div class="mg-log" id="events"></div></section><p class="mg-footer">Practice uses the game’s compiled Magnus rules. Enemy movement and attacks are practice fixtures. Live engine targeting, item interactions and sprite rendering require a match check.</p></aside></div></div>';
 let engine;
 try{engine=await MagnusEngine.load();}catch(e){root.innerHTML='<div class="mg-error">Chess Lab could not load its rules: '+esc(e.message)+'</div>';return;}
 const $=id=>document.getElementById(id),ctx=$('board').getContext('2d');
 let tick=0,state=null,selected=4,running=false,fixture=null,replaying=false,stunUntil=0,logs=[],images={},manifest={};
 let owner,fighters,walls=[],hp=Array(7).fill(-1),positions={};
 const T=24000,ox=76,oy=498,scale=56/T;
 function world(b,c){let x=(c.x-4)*T,y=c.y*T;[x,y]=[[x,y],[y,-x],[-x,-y],[-y,x]][b.heading%4];return{x:b.x+x,y:b.y+y};}
 function cell(b,p,round=true){let x=p.x-b.x,y=p.y-b.y;[x,y]=[[x,y],[-y,x],[-x,-y],[y,-x]][b.heading%4];return{x:round?Math.floor(x/T+.5)+4:x/T+4,y:round?Math.floor(y/T+.5):y/T};}
 function pixel(p){const b=state?.board||{x:400000,y:400000,heading:Number($('heading').value)},c=cell(b,p,false);return{x:ox+c.x*56,y:oy-c.y*56};}
 function foe(id,c){let p=world({x:400000,y:400000,heading:Number($('heading').value)},c);return{id,team:1,...p,hp:5000,maxHp:5000,attack:45,ap:50,champion:true,visible:true,targetable:true,tags:0};}
 function initialize(){
   engine.reset(Number($('seed').value)||42,Number($('rank').value));tick=0;fixture=null;replaying=false;running=false;stunUntil=0;logs=[];positions={};hp=Array(7).fill(-1);walls=[];
   owner={id:1,team:0,x:400000,y:400000,hp:800,maxHp:800,attack:45,ap:50};
   fighters=[foe(2,{x:4,y:4}),foe(3,{x:6,y:5})];fighters[1].attack=100;
   state=engine.step({tick,owner,fighters,walls,rank:Number($('rank').value),action:{type:0,heading:Number($('heading').value)}});
   render();
 }
 function apply(commands){
   for(const c of commands){
     if(c.type==='spawn'){hp[c.piece]=c.hp;positions[c.piece]={x:c.x,y:c.y};}
     if(c.type==='move')positions[c.piece]={x:c.x,y:c.y};
     if(c.type==='despawn'){hp[c.piece]=-1;delete positions[c.piece];}
     if(c.type==='damage'){const f=fighters.find(f=>f.id===c.target);if(f)f.hp=Math.max(0,f.hp-c.amount);}
     if(c.type==='owner'){owner.x=c.x;owner.y=c.y;}
     if(c.type==='label')logs.unshift((tick/60).toFixed(2)+' s · '+labels[c.code]);
     if(c.type==='stun')logs.unshift((tick/60).toFixed(2)+' s · enemy '+c.target+' stunned '+(c.ticks/60)+' s');
   }
   logs=logs.slice(0,60);
 }
 function advance(action=null,captures=[]){
   tick++;
   if(replaying&&fixture){
     for(const [t,id,x,y] of fixture.relocate)if(t===tick){const f=fighters.find(f=>f.id===id);Object.assign(f,world(state.board,{x,y}));}
     const m=fixture.moves.find(m=>m[0]===tick);if(m)action={type:1,piece:m[1],x:m[2],y:m[3]};
     captures=captures.concat(fixture.captures.filter(c=>c[0]===tick).map(c=>({piece:c[1],by:5,combat:true})));
     if(tick>=fixture.end){running=false;replaying=false;}
   }else{
     if($('motion').checked&&state?.board){for(let i=0;i<fighters.length;i++){const p=world(state.board,{x:3+i*2+Math.sin(tick/100+i)*1.3,y:4+Math.cos(tick/140+i)});if(!blocked(p))Object.assign(fighters[i],{x:Math.round(p.x),y:Math.round(p.y)});}}
     if($('combat').checked&&tick%90===0){
       for(const f of fighters.filter(f=>f.hp>0&&f.visible)){
         const target=state.pieces.filter(p=>p.hp>0).sort((a,b)=>distance(f,positions[a.id]||world(state.board,a))-distance(f,positions[b.id]||world(state.board,b)))[0];
         if(target&&distance(f,positions[target.id]||world(state.board,target))<55000){hp[target.id]=Math.max(0,(hp[target.id]<0?target.hp:hp[target.id])-Math.floor(45*100/115));if(hp[target.id]===0)captures.push({piece:target.id,by:f.id,combat:true});}
       }
     }
   }
   state=engine.step({tick,owner,fighters,walls,captures,pieceHp:hp,controlled:tick<stunUntil,automatic:$('automatic').checked&&!replaying,rank:Number($('rank').value),action});
   if(state.error)throw new Error(state.error);apply(state.commands);render();
 }
 const distance=(a,b)=>Math.hypot(a.x-b.x,a.y-b.y),blocked=p=>walls.some(r=>p.x>=r[0]&&p.y>=r[1]&&p.x<=r[2]&&p.y<=r[3]);
 function submit(a){const before=state.pending;advance(a);if(!state.pending&&a.type!==3&&!before)$('status').textContent='Command unavailable: check movement, terrain, control range or cooldown.';}
 function bits(hex,i){return (BigInt('0x'+hex)&(1n<<BigInt(i)))!==0n;}
 function sprite(tag,at,w,h){
   for(const [name,anims]of Object.entries(manifest)){const frames=anims[tag]?.frames;if(!frames||!images[name])continue;const f=frames[Math.floor(tick/6)%frames.length].data;ctx.imageSmoothingEnabled=false;ctx.drawImage(images[name],f.x,f.y,f.w,f.h,at.x-w/2,at.y-h+12,w,h);return true;}return false;
 }
 function draw(){
   ctx.clearRect(0,0,600,570);ctx.fillStyle='#111a2b';ctx.fillRect(0,0,600,570);
   const b=state.board||{x:400000,y:400000,heading:Number($('heading').value)};
   const warn=BigInt(state.warningLo||0)|(BigInt(state.warningHi||0)<<32n);
   for(let y=0;y<8;y++)for(let x=0;x<8;x++){
     const at={x:ox+x*56,y:oy-y*56},i=y*8+x;
     ctx.fillStyle=(x+y)%2?'#26384e':'#65717a';ctx.fillRect(at.x-27,at.y-27,54,54);
     if(blocked(world(b,{x,y}))){ctx.fillStyle='#172131';ctx.fillRect(at.x-25,at.y-25,50,50);ctx.strokeStyle='#79828b';ctx.beginPath();ctx.moveTo(at.x-15,at.y-15);ctx.lineTo(at.x+15,at.y+15);ctx.moveTo(at.x+15,at.y-15);ctx.lineTo(at.x-15,at.y+15);ctx.stroke();}
     if($('coverage').checked&&state.coverage?.[selected]&&bits(state.coverage[selected],i)){ctx.fillStyle='#72d7bd44';ctx.fillRect(at.x-25,at.y-25,50,50);ctx.fillStyle='#84dac1';ctx.beginPath();ctx.arc(at.x,at.y,4,0,Math.PI*2);ctx.fill();}
     if(warn&(1n<<BigInt(i))){ctx.strokeStyle=state.mateWarning!==null?'#ef7c8c':'#eab45f';ctx.lineWidth=3;ctx.strokeRect(at.x-24,at.y-24,48,48);}
   }
   ctx.font='12px system-ui';ctx.textAlign='center';ctx.fillStyle='#b5c0cc';for(let x=0;x<8;x++)ctx.fillText('abcdefgh'[x],ox+x*56,544);for(let y=0;y<8;y++)ctx.fillText(y+1,37,oy-y*56+5);
   for(const p of state.pieces||[]){
     const at=pixel(positions[p.id]||world(b,p));
     if(p.hp<=0){if(p.readyAt<1000000000){const home=pixel(world(b,{x:p.homeX,y:0}));ctx.fillStyle='#8294a7';ctx.font='10px system-ui';ctx.fillText(names[p.id].split(' ')[0]+' '+Math.max(0,(p.readyAt-tick)/60).toFixed(1)+'s',home.x,home.y+20);}continue;}
     if(p.id===selected){ctx.strokeStyle='#ead3a0';ctx.lineWidth=2;ctx.beginPath();ctx.arc(at.x,at.y,25,0,Math.PI*2);ctx.stroke();}
     if(!sprite('piece_'+['rook','bishop','knight','queen'][p.kind],at,44,44)){ctx.font='38px serif';ctx.fillStyle='#ede3c9';ctx.fillText(glyphs[p.kind],at.x,at.y+12);}
     ctx.fillStyle='#0a1220';ctx.fillRect(at.x-20,at.y+20,40,4);ctx.fillStyle='#85c9ac';ctx.fillRect(at.x-20,at.y+20,40*p.hp/p.maxHp,4);
     ctx.fillStyle='#ead3a0';ctx.font='9px system-ui';ctx.fillText(['R1','R2','B1','B2','N1','N2','Q'][p.id],at.x,at.y-28);
     if(tick<state.glowUntil&&(state.glowMask&(1<<p.id))){ctx.strokeStyle='#efd196';ctx.strokeRect(at.x-23,at.y-27,46,52);}
   }
   const me=pixel(owner);if(!sprite('idle',me,48,56)){ctx.fillStyle='#d7b979';ctx.font='34px serif';ctx.fillText('♔',me.x,me.y+12);}
   ctx.fillStyle='#d4b875';ctx.font='10px system-ui';ctx.fillText('MAGNUS',me.x,me.y+31);
   for(const f of fighters.filter(f=>f.hp>0&&f.visible)){
     const p=pixel(f),r=state.projections.find(r=>r.id===f.id)?.role??4;
     ctx.fillStyle='#9a4454';ctx.beginPath();ctx.arc(p.x,p.y,18,0,Math.PI*2);ctx.fill();ctx.fillStyle='#ffe0dc';ctx.font='29px serif';ctx.fillText(roles[r],p.x,p.y+10);
     ctx.font='10px system-ui';ctx.fillText('enemy '+f.id,p.x,p.y+31);
   }
 }
 function render(){
   draw();$('run').textContent=running?'❚❚ Pause':'▶ Play';
   $('roster').innerHTML=state.pieces.map(p=>'<button class="'+(p.id===selected?'selected':'')+'" data-piece="'+p.id+'">'+glyphs[p.kind]+' '+names[p.id]+'<small>'+(p.hp>0?p.hp+' / '+p.maxHp+' HP':p.readyAt>=1000000000?'Unused in fixture':'Recover '+Math.max(0,(p.readyAt-tick)/60).toFixed(1)+' s')+'</small></button>').join('');
   const p=state.pending;let status=p?(p.type===0?'Deploying board':p.type===2?'Castling':names[p.piece]+' → '+'abcdefgh'[p.x]+(p.y+1))+' · '+(p.committed?'moving':'warning')+' · '+Math.max(0,(p.end-tick)/60).toFixed(2)+' s remaining':'Select a piece, then click a destination.';
   if(tick<state.brilliantUntil)status='Brilliant Recall · '+((state.brilliantUntil-tick)/60).toFixed(1)+' s · '+status;
   if(owner.hp<=0)status='Magnus is down. Pieces have been removed; cooldowns remain.';
   $('status').textContent=status;
   const c=state.challenge,g=D.games[c?.id??Number($('history').value)];
   $('challenge').innerHTML='<span class="mg-badge">ADAPTATION · '+g.year+'</span><h3 class="mg-challenge-title">'+esc(g.title)+'</h3><p class="mg-small">'+esc(g.clue)+'</p><ol class="mg-steps">'+g.steps.map((s,i)=>'<li class="'+(c&&i<c.progress?'done':c&&i===c.progress?'active':'')+'"><span>'+(c&&i<c.progress?'✓':i+1)+'</span>'+esc(s)+'</li>').join('')+'</ol>'+(c?.done?'<p class="mg-success">Sequence completed'+(state.stats.rewards?' · Brilliant Recall earned':' · reward on cooldown')+'</p>':c?.failed?'<p class="mg-failure">Attempt ended: '+esc(failures[c.failure]||failures[0])+'</p>':c?.started!==null&&c?.started!==undefined?'<p class="mg-small">Completion window: '+Math.max(0,(c.started+480-tick)/60).toFixed(1)+' s</p>':'<p class="mg-small">The eight-second window begins with the first required event.</p>')+'<div class="mg-source">'+esc(g.players)+' · '+esc(g.place)+'<br>'+esc(g.event)+'<code>'+esc(g.moves.join(' ')||g.reference)+'</code><br><a href="'+esc(g.source)+'" target="_blank" rel="noreferrer">Read the historical reference ↗</a></div>';
   const cooldown=t=>Math.max(0,(t-tick)/60).toFixed(1)+' s';
   $('metrics').innerHTML='<p class="mg-small">Time '+(tick/60).toFixed(2)+' s · Reset '+cooldown(state.resetAt)+' · Castling '+cooldown(state.castleAt)+'<br>Checkmate '+cooldown(state.mateAt)+' · Recall '+cooldown(state.rewardAt)+'<br>Commands '+state.stats.commands.join(' / ')+' (R / B / N / Q)<br>Raw piece damage '+state.stats.damage+' · Captures '+state.stats.captures+' · Checkmates '+state.stats.mates+'</p>';
   $('events').innerHTML=logs.map(l=>'<div>'+esc(l)+'</div>').join('');
 }
 function loadFixture(){
   running=false;replaying=false;tick=0;logs=[];positions={};hp=Array(7).fill(-1);stunUntil=0;fixture=engine.fixture(Number($('history').value));state=fixture.state;
   owner={id:1,team:0,x:400000,y:400000,hp:800,maxHp:800,attack:45,ap:50};fighters=fixture.fighters;walls=fixture.walls;
   for(const p of state.pieces){if(p.hp>0){hp[p.id]=p.hp;positions[p.id]=world(state.board,p);}}
   const m=fixture.moves.find(m=>m[0]===0);if(m){state=engine.step({tick,owner,fighters,walls,rank:Number($('rank').value),action:{type:1,piece:m[1],x:m[2],y:m[3]}});apply(state.commands);}
   render();
 }
 $('roster').onclick=e=>{const b=e.target.closest('[data-piece]');if(b){selected=Number(b.dataset.piece);render();}};
 $('new').onclick=initialize;$('deploy').onclick=()=>submit({type:0,heading:Number($('heading').value)});
 $('dismiss').onclick=()=>submit({type:3});$('castle').onclick=()=>submit({type:2,piece:selected});
 $('run').onclick=()=>{running=!running;render();};$('step').onclick=()=>{for(let i=0;i<30;i++)advance();};
 $('demo').onclick=loadFixture;$('replay').onclick=()=>{loadFixture();replaying=true;running=true;render();};
 $('choose').onclick=()=>{engine.challenge(Number($('history').value));advance();};
 $('history').onchange=()=>{if(!state.challenge)render();};
 $('capture').onclick=()=>advance(null,[{piece:selected,by:2,combat:true}]);$('fog').onclick=()=>{const f=fighters.find(f=>f.id===(state.projections.find(r=>r.role===0)?.id??2));f.visible=!f.visible;advance();};
 $('interrupt').onclick=()=>{stunUntil=tick+30;advance();};$('death').onclick=()=>{owner.hp=0;advance();};$('respawn').onclick=()=>{owner.hp=800;advance();};
 $('coverage').onchange=render;$('rank').onchange=()=>advance();
 $('terrain').onchange=()=>{
   walls=[];const b=state.board;if(!b)return;
   if($('terrain').value==='cross'){for(const c of [{x:3,y:3},{x:3,y:4},{x:4,y:3},{x:4,y:4}]){const p=world(b,c);walls.push([p.x-11900,p.y-11900,p.x+11900,p.y+11900]);}}
   if($('terrain').value==='edge'){const king=fighters.find(f=>f.id===(state.projections.find(r=>r.role===0)?.id??2));Object.assign(king,world(b,{x:3,y:7}));logs.unshift('Board-edge fixture: squares beyond the board stay safe escape routes.');}
   advance();
 };
 $('board').onclick=e=>{
   const rect=$('board').getBoundingClientRect(),px=(e.clientX-rect.left)*600/rect.width,py=(e.clientY-rect.top)*570/rect.height;
   const c={x:Math.round((px-ox)/56),y:Math.round((oy-py)/56)},mode=$('mode').value,b=state.board||{x:400000,y:400000,heading:Number($('heading').value)},p=world(b,{x:(px-ox)/56,y:(oy-py)/56});
   if(mode==='command'){if(c.x<0||c.x>7||c.y<0||c.y>7)return;submit({type:1,piece:selected,x:c.x,y:c.y});}
   if(mode==='magnus'&&!blocked(p)){owner.x=Math.round(p.x);owner.y=Math.round(p.y);advance();}
   if(mode==='king'||mode==='queen'){const role=mode==='king'?0:1,id=state.projections.find(r=>r.role===role)?.id??(role===0?2:3),f=fighters.find(f=>f.id===id);Object.assign(f,{x:Math.round(p.x),y:Math.round(p.y)});advance();}
   if(mode==='setup'){engine.piece(selected,c.x,c.y,state.pieces[selected].maxHp);hp[selected]=state.pieces[selected].maxHp;positions[selected]=world(b,c);advance();}
   if(mode==='wall'){const at=world(b,c),r=[at.x-11900,at.y-11900,at.x+11900,at.y+11900],i=walls.findIndex(w=>w[0]===r[0]&&w[1]===r[1]);if(i<0)walls.push(r);else walls.splice(i,1);advance();}
 };
 let last=performance.now(),debt=0;
 window.addEventListener('message',e=>{if(e.origin===location.origin&&e.data?.type==='magnus-pause'){running=false;render();}});
 function animate(now){if(running){debt+=Math.min(100,now-last);let count=0;while(debt>=1000/60&&count++<6&&running){advance();debt-=1000/60;}}else debt=0;last=now;requestAnimationFrame(animate);}
 initialize();requestAnimationFrame(animate);
 try{
   manifest=await(await fetch('/magnus-art.json')).json();
   await Promise.all(Object.keys(manifest).map(name=>new Promise(resolve=>{const im=new Image();im.onload=()=>{images[name]=im;resolve();};im.onerror=resolve;im.src='/magnus-runtime/'+name+'.png';})));render();
 }catch(e){logs.push('Artwork preview unavailable: '+e.message);}
 window.MagnusPractice={get state(){return state;},get tick(){return tick;},advance,loadFixture};
})();
