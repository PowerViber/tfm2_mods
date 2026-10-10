/* Reuse the lab's controls inside the same shell as Coder and Isliid. */
(function () {
  'use strict';
  const params=new URLSearchParams(location.search);
  if(params.get('embed')!=='1')return;
  const $=s=>document.querySelector(s),root=document.documentElement,main=$('#scienceLab');
  root.dataset.editorEmbed='true';
  const style=document.createElement('link');style.rel='stylesheet';style.href='science-embed.css';document.head.append(style);
  const shell=document.createElement('div');shell.className='il-shell science-shell';
  shell.innerHTML=`<div class="il-head"><div><h2>Science Lab (The Unified Theory)</h2><p>Watch the original skill sprites, balance token charges, and compose experiments. Charge equations are exact; trajectories and notebook trials are schematic.</p></div></div>
    <div class="il-controls" id="scienceControls"><label>Scientist<select id="scienceScientist"><option value="0">Einstein</option><option value="1">Newton</option><option value="2">Marie Curie</option></select></label></div>
    <div class="il-main"><div class="science-preview"><div id="scienceCanvas"></div><div class="il-actions" id="scienceActions"></div><div class="il-result" id="scienceResult"></div><details class="science-transform"><summary>Transform scientist</summary></details></div><aside class="il-side science-detail"></aside></div>
    <div class="science-tools"><details id="scienceExperiment"><summary>Experiment builder · 50 recipes</summary></details><details id="scienceLibrary"><summary>Every skill · 75 recipes</summary></details></div>
    <details id="scienceComparison"><summary>Mastery comparison · 10 trials per rank</summary></details>`;
  const controls=shell.querySelector('#scienceControls'),actions=shell.querySelector('#scienceActions');
  shell.querySelector('.il-head').append($('#parity'));
  const artControls=[...document.querySelectorAll('.art-controls')];
  controls.append(...artControls[0].children);
  const previewLabel=document.createElement('label');previewLabel.innerHTML='Preview<select id="sciencePreview"><option value="animation">Skill animation</option><option value="study">Trajectory study</option></select>';controls.append(previewLabel);
  shell.querySelector('#scienceCanvas').append($('#artCanvas'),$('#arena'));
  $('#artComplete').textContent='Completion animation';$('#compare').textContent='Compare every rank (10 trials)';
  actions.append($('#artPause'),$('#artComplete'),$('#compare'),$('#cancel'));
  shell.querySelector('#scienceResult').append($('#artCaption'),$('#event'),$('#artStatus'));
  shell.querySelector('.science-transform').append(artControls[1]);
  shell.querySelector('.science-detail').append($('.detail'));
  shell.querySelector('#scienceExperiment').append($('.experiment-layout .panel'));
  shell.querySelector('#scienceLibrary').append($('.library'));
  shell.querySelector('#scienceComparison').append($('.mastery'));
  main.replaceChildren(shell);
  let visible=true,applying=false,lastHeight=0;
  function visibility(){
    const active=visible&&!document.hidden,study=$('#sciencePreview').value==='study';
    ScienceLab.setVisible(active&&study);ScienceArt.setVisible(active&&!study);
  }
  function preview(){
    const study=$('#sciencePreview').value==='study';
    for(const id of ['#arena','#event','#cancel'])$(id).hidden=!study;
    for(const id of ['#artCanvas','#artCaption','#artPause','#artComplete'])$(id).hidden=study;
    $('.science-transform').hidden=study;$('#artMode').parentElement.hidden=study;
    visibility();
  }
  function study(){ $('#sciencePreview').value='study';preview(); }
  function selection(){
    $('#scienceScientist').value=String(ScienceData.skills[ScienceLab.selected].science);
    if(applying)return;
    parent.postMessage({type:'science-editor-selection',rank:Number($('#artRank').value),position:Number($('#artPosition').value),skill:ScienceLab.selected},location.origin);
  }
  function apply(value){
    applying=true;
    if(typeof value.visible==='boolean')visible=value.visible;
    ScienceArt.setSelection({rank:Number(value.rank),position:Number(value.position),skill:Number(value.skill)});
    selection();visibility();applying=false;
  }
  $('#sciencePreview').onchange=preview;
  $('#scienceScientist').onchange=()=>ScienceLab.choose(Number($('#scienceScientist').value)*25);
  $('#preview').addEventListener('click',study);$('#reserve').addEventListener('click',study);
  $('#add').addEventListener('click',()=>{$('#scienceExperiment').open=true;});
  $('#compare').addEventListener('click',()=>{$('#scienceComparison').open=true;});
  window.addEventListener('message',ev=>{
    if(ev.origin===location.origin&&ev.source===parent&&ev.data?.type==='science-editor-state')apply(ev.data);
  });
  document.addEventListener('visibilitychange',visibility);
  document.addEventListener('science-skill-selected',selection);
  for(const id of ['#artRank','#artPosition'])$(id).addEventListener('change',selection);
  new ResizeObserver(()=>{
    const height=Math.ceil(main.getBoundingClientRect().height);
    if(height!==lastHeight){lastHeight=height;parent.postMessage({type:'science-editor-size',height},location.origin);}
  }).observe(main);
  preview();apply({rank:params.get('rank'),position:params.get('position'),skill:params.get('skill')});
})();
