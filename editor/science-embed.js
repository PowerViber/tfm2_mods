/* Same-origin bridge. Standalone Science Lab keeps its normal layout. */
(function () {
  'use strict';
  const params=new URLSearchParams(location.search);
  if(params.get('embed')!=='1')return;
  const root=document.documentElement, main=document.getElementById('scienceLab');
  root.dataset.editorEmbed='true';
  const title=document.createElement('strong');title.className='editor-title';main.querySelector('header').prepend(title);
  let view='test',visible=true,applying=false,lastHeight=0;
  function visibility(){
    ScienceLab.setVisible(visible&&!document.hidden&&view==='test');
    ScienceArt.setVisible(visible&&!document.hidden&&view==='viewer');
  }
  function selection(){
    if(applying)return;
    parent.postMessage({type:'science-editor-selection',rank:Number(document.getElementById('artRank').value),position:Number(document.getElementById('artPosition').value),skill:ScienceLab.selected},location.origin);
  }
  function apply(value){
    applying=true;
    view=value.view==='viewer'?'viewer':'test';root.dataset.editorView=view;
    title.textContent=view==='viewer'?'The Unified Theory · Skill viewer':'The Unified Theory · Science Lab';
    if(typeof value.visible==='boolean')visible=value.visible;
    ScienceArt.setSelection({rank:Number(value.rank),position:Number(value.position),skill:Number(value.skill)});
    visibility();applying=false;
  }
  window.addEventListener('message',ev=>{
    if(ev.origin===location.origin&&ev.source===parent&&ev.data?.type==='science-editor-state')apply(ev.data);
  });
  document.addEventListener('visibilitychange',visibility);
  document.addEventListener('science-skill-selected',selection);
  for(const id of ['artRank','artPosition'])document.getElementById(id).addEventListener('change',selection);
  new ResizeObserver(()=>{
    const height=Math.ceil(main.getBoundingClientRect().height);
    if(height!==lastHeight){lastHeight=height;parent.postMessage({type:'science-editor-size',height},location.origin);}
  }).observe(main);
  apply({view:params.get('view'),rank:params.get('rank'),position:params.get('position'),skill:params.get('skill')});
})();
