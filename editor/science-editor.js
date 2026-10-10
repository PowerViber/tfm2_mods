/* One reusable Science Lab frame; the editor owns its visibility and selection. */
(function () {
  'use strict';
  let frame, loaded=false, visible=false, options;
  function send() {
    if(loaded)frame.contentWindow.postMessage({type:'science-editor-state',rank:options.rank,position:options.position,skill:options.skill,visible},location.origin);
  }
  window.TFM2ScienceEditor={
    mount(container,value){
      options=value;
      if(!frame){
        frame=document.createElement('iframe');frame.title='The Unified Theory — Science Lab';
        const params=new URLSearchParams({embed:'1',rank:value.rank,position:value.position,skill:value.skill});
        frame.src='/science.html?'+params;
        frame.onload=()=>{loaded=true;send();};
        container.append(frame);
      }
      send();
    },
    setVisible(value){visible=Boolean(value);if(options)send();}
  };
  window.addEventListener('message',ev=>{
    if(ev.origin!==location.origin||!frame||ev.source!==frame.contentWindow)return;
    const data=ev.data;
    if(data?.type==='science-editor-size'&&Number.isFinite(data.height))frame.style.height=Math.max(240,Math.min(6000,data.height))+'px';
    if(data?.type==='science-editor-selection'&&[data.rank,data.position,data.skill].every(Number.isInteger)&&data.rank>=0&&data.rank<=7&&data.position>=1&&data.position<=10&&data.skill>=0&&data.skill<75){
      Object.assign(options,{rank:data.rank,position:data.position,skill:data.skill});
      options.onSelection?.({...data,science:Math.floor(data.skill/25)});
    }
  });
})();
