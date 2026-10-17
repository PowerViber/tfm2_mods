/* Skill Test's Unified pane: embeds the Equation Lab (unified.html, the game's core in WASM). */
(function(root){'use strict';
let frame=null,host=null;
root.TFM2ScienceEditor={
  mount(el,opts={}){
    if(!el)return;
    const src='/unified.html?embed=1&rank='+(opts.rank??7);
    if(!frame||host!==el||frame.dataset.src!==src){
      el.innerHTML='';frame=document.createElement('iframe');frame.dataset.src=src;frame.src=src;
      frame.title='Unified Equation Lab';frame.style.cssText='width:100%;height:1150px;border:0;background:#0b1422';
      el.append(frame);host=el;
    }
  },
  setVisible(v){if(host)host.hidden=!v;}
};
})(globalThis);
