// Presentation refresh is independent of the fixed 60 Hz combat clock.
window.KratosClock = class {
  constructor() { this.reset(); }
  reset() { this.last = null; this.accumulator = 0; }
  advance(now, running, step) {
    if (this.last === null) { this.last = now; return; }
    const elapsed = Math.min(250, Math.max(0, now - this.last));
    this.last = now;
    if (!running) { this.accumulator = 0; return; }
    this.accumulator += elapsed;
    while (this.accumulator + 1e-7 >= 1000 / 60) {
      this.accumulator -= 1000 / 60;
      step();
    }
  }
};
