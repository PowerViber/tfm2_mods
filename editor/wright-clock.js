// Rendering callbacks may run at 30-240 Hz; the shared combat core always advances at 60 Hz.
window.WrightClock=class {
  constructor(){this.reset();}
  reset(){this.last=null;this.accumulator=0;}
  advance(now,running,step){
    if(this.last===null){this.last=now;return;}
    const elapsed=Math.min(250,Math.max(0,now-this.last));this.last=now;
    if(!running){this.accumulator=0;return;}
    this.accumulator+=elapsed;
    while(this.accumulator+1e-7>=1000/60){this.accumulator-=1000/60;step();}
  }
};
