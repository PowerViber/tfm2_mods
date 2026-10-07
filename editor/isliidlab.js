/* Isliid Engraving Lab: sword staging, parallel drawing and predictive mastery.
 * It previews the native rules but does not run the match simulation or edit a career.
 */
(function () {
  'use strict';

  const W = 820, H = 480, HERO = { x: 94, y: 245 }, STORE = 'tfm2.isliidlab.v1';
  const SWORDS = [
    ['Skylight', '#f6edaa'], ['Terra', '#b79769'], ['Darkbringer', '#a479d1'],
    ['Gale', '#8de8d9'], ['Blood', '#e87283'], ['Rift', '#75a7fa'], ['Emperor', '#ffd166'],
  ];
  // Round 88: the native rules' tables (native/tfm2_custom_ai/src/isliid.rs; tools/verify_isliid.py checks they match).
  const NATIVE = {
    SPEED: [8000, 5500, 7000, 12000, 7500, 9000, 6500],          // map units a tick
    THINK_TICKS: [90, 75, 60, 48, 38, 30, 22, 15],
    LOOK_AHEAD: [0, 30, 60, 90, 120, 180, 240, 300],              // ticks
    PATTERN_BUDGET: [3, 5, 8, 12, 16, 21, 26, 30],
    // round 91: much wider mastery gaps (only Imperial #1 aims perfectly; ally cover climbs from ~10% to ~99%)
    WOBBLE: [13800, 12000, 10500, 8250, 6450, 4500, 3000, 1600],
    IMPERIAL_WOBBLE_STEP: 178,
    NOTICE: [6, 15, 15, 25, 27, 32, 32, 32],
    ESCORTS: [1, 1, 1, 1, 2, 2, 2, 3],
    REASSESS: [45, 55, 65, 80, 95, 110, 130, 150],
    PER_ALLY: 2,                                                   // round 92: no escort range, 2 swords per teammate
    IDLE_RETURN: [240, 210, 180, 150, 120, 100, 80, 60],
    STRIKE_GAP: [90, 84, 78, 72, 66, 60, 54, 48],
    SOLO_QUALITY: [45, 53, 60, 67, 74, 81, 87, 92],
    GRADES: [['Imperial', 99, 120], ['Perfect', 95, 110], ['Refined', 85, 100], ['Stable', 70, 85], ['Crude', 60, 70]],
    THREAT_R: 105000, PLAN_GAP: 180, RETURN: 1.5,
    // round 93: an escort's strike gap grows with its distance from Isliid (to STRIKE_FAR_PCT% from FAR_R on)
    FULL_R: 60000, FAR_R: 200000, STRIKE_FAR_PCT: 250,
    // round 94: every sword leaves at LAUNCH_SPEED and speeds up to TOP_PCT% of SPEED after RAMP_TICKS in the air
    // (returns 1.5x that); basic-attack throws keep their full SPEED
    LAUNCH_SPEED: 1000, RAMP_TICKS: 150, TOP_PCT: 60,
  };
  /** Native sword_speed(): units a tick for sword i in mode ('stage' | 'draw' | 'return' | 'throw') after `air` ticks. */
  const swordSpeed = (i, mode, air) => {
    if (mode === 'throw') return NATIVE.SPEED[i];
    let top = Math.trunc(NATIVE.SPEED[i] * NATIVE.TOP_PCT / 100);
    if (mode === 'return') top = Math.trunc(top * 3 / 2);
    top = Math.max(top, NATIVE.LAUNCH_SPEED);
    return NATIVE.LAUNCH_SPEED + Math.trunc((top - NATIVE.LAUNCH_SPEED) * Math.min(air, NATIVE.RAMP_TICKS) / NATIVE.RAMP_TICKS);
  };
  /** Native flight_ticks(): ticks to fly `units` leaving with `air` ticks in flight (capped at 600). */
  const flightTicks = (i, mode, units, air = 0) => { let left = units, t = 0;
    while (left > 0 && t < 600) { left -= swordSpeed(i, mode, air + t); t++; } return t; };
  const TPS = 60, UPX = 35000 / 104;      // a medium formation (radius 35000 units) is 104 lab px across its radius
  /** Milliseconds for sword i to fly `px` lab pixels from a standstill. */
  const flightMs = (i, mode, px) => flightTicks(i, mode, px * UPX) * 1000 / TPS;
  const LOOK_AHEAD = NATIVE.LOOK_AHEAD.map(t => t / TPS);     // seconds
  const RANKS = ['Bearer', 'Squire', 'Engraver', 'Tactician', 'Swordmaster', 'Regent', 'Sovereign', 'Imperial'].map((name, r) => ({
    name, points: [0, 5, 15, 30, 60, 100, 150, 'Top 10, 300+'][r], decision: Math.round(NATIVE.THINK_TICKS[r] * 1000 / TPS),
    error: NATIVE.WOBBLE[r] / UPX, candidates: NATIVE.PATTERN_BUDGET[r] }));
  /** Unrounded accuracy of a formation: 100 - summed endpoint error x 100 / (radius x legs), clamped (native formation_accuracy). */
  const formationAccuracy = (error, radius, legs) => clamp(100 - error * 100 / (Math.max(1e-9, radius) * Math.max(1, legs)), 0, 100);
  /** [grade, multiplier %] or null under 60 (native grade). */
  const gradeOf = acc => { const g = NATIVE.GRADES.find(g => acc >= g[1]); return g ? [g[0], g[2]] : null; };
  const lvl = imperial => clamp(imperial || 10, 1, 10);
  /** Round 91: the aim error by mastery (native wobble): the rank's, or at Imperial by level (#1 = 0). */
  const wobbleOf = (rank, imperial) => rank >= 7 ? (lvl(imperial) - 1) * NATIVE.IMPERIAL_WOBBLE_STEP : NATIVE.WOBBLE[rank];
  /** A solo stroke's quality (native solo_quality). */
  const soloQuality = (rank, imperial) => rank >= 7 ? 100 - (100 - NATIVE.SOLO_QUALITY[7]) * (lvl(imperial) - 1) / 9 : NATIVE.SOLO_QUALITY[rank];
  /** The notice chance (native notice_pct; integer maths like u64). */
  const noticePct = (rank, imperial) => rank >= 7 ? 99 - Math.floor((99 - NATIVE.NOTICE[7]) * (lvl(imperial) - 1) / 9) : NATIVE.NOTICE[rank];
  /** Whether he notices threatened ally `ally` on this look, exactly as the native notices (u64 maths in BigInt). */
  function notices(seed, tick, ally, pct) {
    const M = (1n << 64n) - 1n;
    const h = ((BigInt(seed) ^ ((BigInt(tick) * 0x9e3779b9n) & M) ^ (BigInt(ally) << 32n)) * 0x2545f4914f6cdd1dn) & M;
    return Number((h >> 33n) % 100n) < pct;
  }
  /** The aim error of planned stroke j of sword i, exactly as the native plan_wobble (u64 maths in BigInt); w from wobbleOf. */
  function planWobble(seed, tick, i, j, w) {
    if (!w) return 0;
    const M = (1n << 64n) - 1n;
    let salt = ((BigInt(seed) ^ BigInt(tick) ^ (BigInt(i) << 24n) ^ BigInt(j)) * 0x9e3779b9n) & M;
    if (salt >= (1n << 63n)) salt -= (1n << 64n);
    const m = BigInt(w * 2 + 1);
    return Number(((salt % m) + m) % m) - w;
  }
  /** The same vectors as the native tests (grades_use_unrounded_accuracy, plan_wobble_is_shared_with_the_lab). */
  function selfTest() {
    let mask = 0n;
    for (let t = 0; t < 100; t++) if (notices(70217, 600 + t, 3, 50)) mask |= 1n << BigInt(t);
    const ok = [Math.abs(formationAccuracy(12345, 55000, 5) - 95.51090909090909) < 1e-9, planWobble(70217, 600, 2, 1, wobbleOf(0)) === -9516,
      mask === 820915055570322631965375425196n, planWobble(1, 1, 0, 0, wobbleOf(7, 1)) === 0,
      gradeOf(98.999)[0] === 'Perfect', gradeOf(99)[0] === 'Imperial', gradeOf(59.999) === null, gradeOf(60)[1] === 70];
    return ok.every(Boolean);
  }
  const radial = (n, skip = 1) => ({ nodes: Array.from({ length:n },(_,i)=>[Math.cos(-Math.PI/2+i*2*Math.PI/n),Math.sin(-Math.PI/2+i*2*Math.PI/n)]),
    edges: Array.from({ length:n },(_,i)=>[i,(i+skip)%n]) });
  const PATTERNS = {
    line: { name: 'Severing Line', effect: 'Crossing damage; length trades damage for coverage', nodes: [[-1, 0], [1, 0]], edges: [[0, 1]] },
    triangle: { name: 'Execution Seal', effect: 'Physical resistance reduction inside', nodes: [[0, -1], [-0.9, 0.72], [0.9, 0.72]], edges: [[0, 1], [1, 2], [2, 0]] },
    square: { name: 'Imperial Fortress', effect: 'Damage resistance for allies inside', nodes: [[-0.82, -0.82], [0.82, -0.82], [0.82, 0.82], [-0.82, 0.82]], edges: [[0, 1], [1, 2], [2, 3], [3, 0]] },
    pentagon: { name: "Emperor's Blessing", effect: 'Support zone; resource gain becomes cooldown recovery in TFM2', nodes: [[0, -1], [0.95, -0.31], [0.59, 0.81], [-0.59, 0.81], [-0.95, -0.31]], edges: [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0]] },
    domain: { name: "Emperor's Domain", effect: 'Seven-sword territory; strongest commitment', nodes: Array.from({ length: 7 }, (_, i) => [Math.cos(-Math.PI / 2 + i * 2 * Math.PI / 7), Math.sin(-Math.PI / 2 + i * 2 * Math.PI / 7)]), edges: Array.from({ length: 7 }, (_, i) => [i, (i + 1) % 7]) },
    tripwire: { name:'Tripwire', effect:'Precise line snaps swords toward a crossing enemy', nodes:[[-1,0],[1,0]], edges:[[0,1]] },
    funnel: { name:'Funnel', effect:'Pull toward the point', nodes:[[-1,-.8],[0,1],[1,-.8]], edges:[[0,1],[1,2]] },
    expulsion: { name:'Expulsion', effect:'Push away from the point', nodes:[[-1,.8],[0,-1],[1,.8]], edges:[[0,1],[1,2]] },
    corner: { name:'Corner Guard', effect:'Slow at a corner; crossing both roots', nodes:[[-1,-1],[-1,1],[1,1]], edges:[[0,1],[1,2]] },
    road: { name:'Piercing Road', effect:'Attack range along the marked road', nodes:[[-1,0],[0,0],[1,0]], edges:[[0,1],[1,2]] },
    suppression: { name:'Suppression Seal', effect:'Reduce enemy damage inside', nodes:[[0,1],[-.9,-.72],[.9,-.72]], edges:[[0,1],[1,2],[2,0]] },
    ambush: { name:'Ambush Seal', effect:'Gain speed toward enemies entering', nodes:[[-1,-1],[-1,1],[1,1]], edges:[[0,1],[1,2],[2,0]] },
    rectangle: { name:'Marching Ground', effect:'Speed along the long axis', nodes:[[-1,-.5],[1,-.5],[1,.5],[-1,.5]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    diamond: { name:'Judgment Field', effect:'Concentrated damage at center', nodes:[[0,-1],[1,0],[0,1],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    siege: { name:'Siege Ground', effect:'Attack strength toward narrow side', nodes:[[-.5,-1],[.5,-1],[1,1],[-1,1]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    pursuit: { name:'Pursuit Seal', effect:'Slow enemies retreating from tip', nodes:[[0,-1],[.6,0],[0,1],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    rupture: { name:'Rupture', effect:'Burst at crossed lines', nodes:[[-1,-1],[1,1],[1,-1],[-1,1]], edges:[[0,1],[1,2],[2,3],[3,0]] },
    intersection: { name:'Divine Intersection', effect:'Periodic center damage', nodes:[[-1,0],[0,-1],[1,0],[0,1]], edges:[[0,2],[1,3]] },
    execution: { name:'Execution Point', effect:'Precise center burst', nodes:[[-1,-1],[1,1],[1,-1],[-1,1]], edges:[[0,1],[2,3]] },
    roadblock: { name:'Roadblock', effect:'Push approaching enemies sideways', nodes:[[-1,0],[0,-1],[1,0],[0,1]], edges:[[0,2],[1,3]] },
    celestial: { name:'Celestial Judgment', effect:'Repeated hits at star crossings', ...radial(5,2) },
    sanctuary: { name:'Sanctuary', effect:'Heal and shield allies', nodes:[[-1,1],[-1,0],[0,-1],[1,0],[1,1]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    charge: { name:'Imperial Charge', effect:'Speed allies and push enemies forward', nodes:[[-1,-.4],[0,-.4],[0,-1],[1,0],[0,1]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    time: { name:'Time of Judgment', effect:'Growing slow at center', nodes:[[-1,-1],[1,1],[1,-1],[-1,1],[0,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,0]] },
    absolute: { name:'Absolute Territory', effect:'Balanced attack, guard and cooldown buffs', ...radial(6) },
    grand: { name:'Grand Execution', effect:'Converging six-sword damage', ...radial(6,2) },
    maelstrom: { name:'Imperial Maelstrom', effect:'Pull enemies inward', nodes:[[0,-.2],[.35,-.4],[.7,0],[.4,.6],[-.3,.8],[-1,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,0]] },
    prison: { name:'Imperial Prison', effect:'Crossing boundary stuns', ...radial(6) },
    heavenfall: { name:'Heavenfall', effect:'Seven-sword convergence burst', ...radial(7,2) },
    authority: { name:"King's Authority", effect:'Strongest team support', nodes:[[-1,0],[-.7,-1],[-.4,0],[0,-1],[.4,0],[.7,-1],[1,0]], edges:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,6],[6,0]] },
  };
  const state = { root: null, canvas: null, ctx: null, sprite: null, weapons: null, fly: null, orbit: null, badges: null, auras: null, fields: null, logos: null, art: null, trails: null, rank: 3, imperialLevel: 1, pattern: 'triangle', scale: 1, scenario: 'self', active: true, empowerment: 0, marks: [], previewState: 'orbit', auraSide: 'ally', inspectFrame: -1, cancelUntil: 0,
    selected: 0, emperor: false, floatPreview: true, playback: 1, anchors: Array(7).fill(null), slots: [], drag: null, flights: [],
    started: 0, placements: 0, corrections: 0, recalls: 0, result: null, auto: null, compare: null, raf: 0 };

  const $ = s => state.root && state.root.querySelector(s);
  const esc = s => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
  const distance = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
  const fmt = ms => (ms / 1000).toFixed(2) + ' s';
  const rand = seed => () => { seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0; return seed / 4294967296; };
  const weaponTier = rank => rank;
  const tierName = RANKS.map(r => r.name);
  const IMPERIAL_LEVELS = 10;
  // Seven numberless designs and ten Imperial subdivisions, eight frames each.
  const badgeFamily = (rank, imperialLevel) => rank < 7 ? rank : 7 + clamp(imperialLevel, 1, IMPERIAL_LEVELS) - 1;
  // round 89: each animation keeps its own frame count and duration (badges 16, grounded swords 12, the rest 8)
  const frameAt = (now, frames) => state.inspectFrame < 0
    ? Math.floor(now / (1000 * (frames?.[0]?.duration || 0.1))) : state.inspectFrame;
  function paintArt(c, sheet, family, tag, now, x, y, w, h) {
    const frames = state.art?.[family]?.[tag]?.frames;
    const frame = frames?.[frameAt(now, frames) % frames.length]?.data;
    if (!frame || !sheet?.complete || !sheet.naturalWidth) return false;
    c.imageSmoothingEnabled = false;
    c.drawImage(sheet, frame.x, frame.y, frame.w, frame.h, x, y, w, h);
    return true;
  }
  /** Round 89: art anchored at its frame centre, at its own pixel size times k (the swords grow with mastery). */
  function paintCentred(c, sheet, family, tag, now, cx, cy, k) {
    const frames = state.art?.[family]?.[tag]?.frames;
    const frame = frames?.[frameAt(now, frames) % frames.length]?.data;
    if (!frame) return false;
    return paintArt(c, sheet, family, tag, now, cx - frame.w * k / 2, cy - frame.h * k / 2, frame.w * k, frame.h * k);
  }

  function paintMark(c, m, now) {
    const dx=m.to.x-m.from.x, dy=m.to.y-m.from.y, length=Math.hypot(dx,dy);
    if (length < 1) return;
    if (!state.trails || !state.trails.complete || !state.trails.naturalWidth) {
      c.strokeStyle=SWORDS[m.sword][1]; c.lineWidth=3; c.beginPath();
      c.moveTo(m.from.x,m.from.y); c.lineTo(m.to.x,m.to.y); c.stroke(); return;
    }
    const angle=Math.round(((Math.atan2(dy,dx)%Math.PI+Math.PI)%Math.PI)*16/Math.PI)%16;
    const phase=Math.floor(now/140)%2, cell=(m.sword*16+angle)*2+phase;
    const count=clamp(Math.ceil(length/15),1,80);
    c.imageSmoothingEnabled=false;
    for (let k=0;k<=count;k++) {
      const t=k/count;
      c.drawImage(state.trails,(cell%16)*32,Math.floor(cell/16)*32,32,32,
        m.from.x+dx*t-16,m.from.y+dy*t-16,32,32);
    }
  }

  function formationCenter(rank = state.rank) {
    const horizon = LOOK_AHEAD[rank], base = { x: W * .56, y: H * .51 };
    // These are visible motion cues in the lab's scenario, not hidden enemy data.
    const velocity = state.scenario === 'objective' ? [0, 0] : state.scenario === 'ally' ? [-14, -9] : [18, 4];
    return { x: base.x + velocity[0] * horizon, y: base.y + velocity[1] * horizon };
  }
  function targets(pattern = state.pattern, scale = state.scale, rank = state.rank) {
    const radius = 104 * scale, { x: cx, y: cy } = formationCenter(rank);
    return PATTERNS[pattern].nodes.map(([x, y]) => ({ x: cx + x * radius, y: cy + y * radius }));
  }
  function legs(pattern = state.pattern, scale = state.scale, rank = state.rank) {
    const nodes = targets(pattern, scale, rank), result = PATTERNS[pattern].edges.map(([a, b]) => ({ from: nodes[a], to: nodes[b] }));
    // Open shapes still need one independently activated sword per leg.
    while (result.length < nodes.length) {
      let longest = 0;
      for (let i = 1; i < result.length; i++) if (distance(result[i].from, result[i].to) > distance(result[longest].from, result[longest].to)) longest = i;
      const { from, to } = result[longest], middle = { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 };
      result.splice(longest, 1, { from, to: middle }, { from: middle, to });
    }
    return result;
  }
  function slotsFor(count, emperor) {
    const a = Array.from({ length: count }, (_, i) => i);
    if (emperor && count < 7) a[count - 1] = 6;
    return a;
  }
  function score(marks, slots, ideal = legs(), radius = 104 * state.scale) {
    // native evaluate_formations: per leg, the closest mark of its sword by summed endpoint distance (either direction)
    const present = slots.map((s, i) => s == null ? null : marks.filter(m => m.sword === s).map(m => ({ m,
      error: Math.min(distance(m.from, ideal[i].from) + distance(m.to, ideal[i].to), distance(m.from, ideal[i].to) + distance(m.to, ideal[i].from)) }))
      .sort((a, b) => a.error - b.error)[0]).filter(Boolean);
    const integrity = Math.round(100 * present.length / slots.length);
    if (!present.length) return { precision: 0, integrity, grade: 'Empty', mult: 0, effectiveness: 0, emperor: false };
    const error = present.reduce((a, p) => a + p.error, 0);
    const precision = formationAccuracy(error, radius, slots.length);
    const g = present.length === slots.length ? gradeOf(precision) : null;
    const committed = new Set(slots).size, emperor = !!g && slots.includes(6);
    const synergy = 1 + 0.25 * Math.max(0, committed - 1);
    const participation = Math.max(25, committed * 100 / slots.length) / 100;
    const length = ideal.reduce((sum, leg) => sum + distance(leg.from, leg.to), 0);
    const concentration = clamp(100000 * 100 / Math.max(50000, radius * UPX * 2), 55, 125) / 100;   // native, by radius
    return { precision, integrity, grade: g ? g[0] : 'Failed', mult: g ? g[1] : 0,
      effectiveness: g ? Math.round(g[1] * synergy * participation * concentration / Math.max(1, committed)) : 0, emperor, length, committed };
  }
  function simulate(rankIndex, pattern = state.pattern, scale = state.scale, seed = 1, emperor = state.emperor, imperial = state.imperialLevel) {
    const rank = RANKS[rankIndex], ideal = legs(pattern, scale, rankIndex), slots = slotsFor(ideal.length, emperor);
    const anchors = Array(7).fill(null), events = [], marks = [];
    let elapsed = 0; const corrections = 0;
    const launch = state.scenario === 'ally' ? { x: W * .72, y: H * .78 } : state.scenario === 'objective' ? { x: W * .83, y: H * .28 } : HERO;
    const tick = 600 + (seed % 600);
    ideal.forEach((leg, i) => {
      const sword = slots[i];
      // native: both endpoints shifted by the same planned error (+e, -e)
      const e = planWobble(seed >>> 0, tick, sword, i, wobbleOf(rankIndex, imperial)) / UPX;
      const from = { x: clamp(leg.from.x + e, 15, W - 15), y: clamp(leg.from.y - e, 15, H - 15) };
      const to = { x: clamp(leg.to.x + e, 15, W - 15), y: clamp(leg.to.y - e, 15, H - 15) };
      const stageEnd = rank.decision + flightMs(sword, 'stage', distance(launch, from));
      const end = stageEnd + flightMs(sword, 'draw', distance(from, to));
      events.push({ sword, from: launch, to: from, start: rank.decision, end: stageEnd, kind: 'stage' });
      events.push({ sword, from, to, start: stageEnd, end, kind: 'draw' });
      elapsed = Math.max(elapsed, end);
      anchors[sword] = to;
      marks.push({ sword, from, to });
    });
    return { rank: rankIndex, pattern, scale, seed, slots, anchors, events, ms: elapsed, corrections,
      quality: score(marks, slots, ideal, 104 * scale), center: formationCenter(rankIndex) };
  }
  /**
   * Round 88: a seeded 30 s skirmish with the native sword control (isliid.rs think / assign_escorts / idle_reclaim /
   * ally_attacks): Isliid, 2 allies and 3 enemies wandering a 400k field, enemies drifting onto the allies. Measures
   *   utilization: sword-ticks with a purpose (escorting a threatened ally, in flight, in a plan, or in hand while an
   *                enemy is within his reach) over 7 x ticks;
   *   idle: mean seconds a sword lies idle on the ground before it's reclaimed;
   *   coverage: of the ticks an ally is threatened, the share with an escort within 40k.
   */
  function simulateSkirmish(rank, seed = 1, ticks = 1800, imperial = state.imperialLevel) {
    const random = rand((seed * 2654435761) >>> 0), N = NATIVE, near = (a, b, r) => Math.hypot(a.x - b.x, a.y - b.y) <= r;
    const walker = (x, y, pull) => ({ x, y, vx: 0, vy: 0, pull, missing: 0 });
    const me = walker(200000, 200000, null), allies = [walker(150000, 160000), walker(250000, 230000)];
    const foes = [walker(80000, 320000), walker(330000, 90000), walker(320000, 330000)];
    const swords = Array.from({ length: 7 }, (_, i) => ({ i, mode: 'orbit', pos: { x: me.x, y: me.y }, goal: null, holder: null,
      idleSince: 0, escortUntil: 0, planUntil: 0 }));
    let used = 0, covered = 0, threatTicks = 0, nextPlan = 0, lastThink = -999, lastThrow = 0;
    const idleRuns = [];
    const step = (w, target, speed) => {
      w.vx = w.vx * 0.9 + (random() - 0.5) * 300 + (target ? Math.sign(target.x - w.x) * 120 : 0);
      w.vy = w.vy * 0.9 + (random() - 0.5) * 300 + (target ? Math.sign(target.y - w.y) * 120 : 0);
      const v = Math.hypot(w.vx, w.vy); if (v > speed) { w.vx *= speed / v; w.vy *= speed / v; }
      w.x = clamp(w.x + w.vx, 0, 400000); w.y = clamp(w.y + w.vy, 0, 400000);
    };
    const threatened = a => { const n = foes.filter(f => near(f, a, N.THREAT_R)).length; return n >= 1 && (a.missing >= 25 || n >= 2) ? n : 0; };
    const holderOf = s => s.holder == null ? me : allies[s.holder];
    const idle = s => s.mode === 'planted' && s.planUntil === 0;
    const flying = s => ['stage', 'return', 'thrown', 'plan'].includes(s.mode);
    const send = (s, mode, goal, holder = null) => { if (idle(s)) idleRuns.push(t - s.idleSince); if (!flying(s)) s.airSince = t; s.mode = mode; s.goal = goal; s.holder = holder; s.idleSince = 0; };
    const free = s => (s.mode === 'orbit' && s.holder == null) || idle(s);
    let t = 0;
    for (t = 0; t < ticks; t++) {
      step(me, { x: (allies[0].x + allies[1].x) / 2, y: (allies[0].y + allies[1].y) / 2 }, 900);
      allies.forEach(a => step(a, null, 1000));
      foes.forEach((f, k) => step(f, allies[k % 2], 1000));
      allies.forEach(a => { const n = foes.filter(f => near(f, a, 60000)).length;
        a.missing = clamp(a.missing + (n ? 0.12 * n : -0.06), 0, 90); });
      // think: escorts first (before the plan gap), leases, then plans
      if (t >= lastThink + N.THINK_TICKS[rank]) {
        lastThink = t;
        // round 92: allies anywhere, but only those he notices on this look
        const pct = noticePct(rank, imperial);
        const noticed = allies.map((a, k) => threatened(a) && notices(seed >>> 0, t, k, pct));
        swords.forEach(s => { if (s.holder != null && s.mode === 'escort') {
          if (t < s.escortUntil) return;
          if (noticed[s.holder]) s.escortUntil = t + N.REASSESS[rank];
          else if (!near(allies[s.holder], me, 40000)) send(s, 'return', null);
        } });
        allies.forEach((a, k) => {
          const n = threatened(a); if (!n || !noticed[k]) return;
          let have = swords.filter(s => s.holder === k && ['escort', 'stage'].includes(s.mode)).length;
          const cap = Math.min(N.ESCORTS[rank], N.PER_ALLY);
          while (have < cap) {
            let pool = swords.filter(free);
            if (pool.filter(s => s.mode === 'orbit').length <= 1 && !(rank >= 5 && a.missing >= 70)) pool = pool.filter(s => s.mode !== 'orbit');
            // round 93: the trip at its slowest (far swords fly slower)
            const best = pool.map(s => [escortScore(s.i, a.missing, n) - flightTicks(s.i, 'stage', Math.trunc(Math.hypot(s.pos.x - a.x, s.pos.y - a.y))) / 4, s])
              .sort((x, y) => y[0] - x[0] || x[1].i - y[1].i)[0];
            if (!best) break;
            send(best[1], 'stage', null, k); best[1].escortUntil = t + N.REASSESS[rank]; have++;
          }
        });
        if (t >= nextPlan && foes.some(f => near(f, me, 200000))) {
          const pool = swords.filter(free).slice(0, 3);
          if (pool.length >= 2) { nextPlan = t + N.PLAN_GAP; const f = foes.find(f => near(f, me, 200000));
            pool.forEach(s => { send(s, 'plan', { x: f.x + (random() - 0.5) * 60000, y: f.y + (random() - 0.5) * 60000 }); s.planUntil = 0; }); }
        }
      }
      // his basic attacks throw a sword at an enemy 23k-65k away (one every 72 ticks)
      const target = foes.find(f => near(f, me, 65000) && !near(f, me, 23000));
      if (target && t >= lastThrow + 72) { const s = swords.find(s => s.mode === 'orbit' && s.holder == null);
        if (s) { send(s, 'thrown', { x: target.x, y: target.y }); lastThrow = t; } }
      // idle reclaim at every rank
      swords.forEach(s => { if (idle(s) && t >= s.idleSince + N.IDLE_RETURN[rank]) {
        const k = allies.findIndex((a, j) => threatened(a) && near(a, s.pos, 100000)
          && notices(seed >>> 0, t, j, noticePct(rank, imperial)));
        if (k >= 0) { send(s, 'stage', null, k); s.escortUntil = t + N.REASSESS[rank]; } else send(s, 'return', null);
      } });
      // movement
      swords.forEach(s => {
        if (s.mode === 'orbit' || s.mode === 'escort') { const h = holderOf(s); s.pos = { x: h.x, y: h.y }; return; }
        if (s.mode === 'planted') { if (s.planUntil && t >= s.planUntil) { s.planUntil = 0; s.idleSince = t; } return; }
        const goal = s.mode === 'stage' || s.mode === 'return' ? holderOf(s) : s.goal;
        // round 94: launches slow, faster the longer it's in the air (basic-attack throws stay fast)
        const sp = swordSpeed(s.i, s.mode === 'thrown' ? 'throw' : s.mode === 'return' ? 'return' : 'stage', t - (s.airSince || 0));
        const d = Math.hypot(goal.x - s.pos.x, goal.y - s.pos.y);
        if (d <= sp) { s.pos = { x: goal.x, y: goal.y };
          if (s.mode === 'stage') s.mode = 'escort';
          else if (s.mode === 'return') { s.mode = 'orbit'; s.holder = null; }
          else if (s.mode === 'plan') { s.mode = 'planted'; s.planUntil = t + 90; }
          else { s.mode = 'planted'; s.idleSince = t; }
        } else s.pos = { x: s.pos.x + (goal.x - s.pos.x) * sp / d, y: s.pos.y + (goal.y - s.pos.y) * sp / d };
      });
      // metrics
      const fighting = foes.some(f => near(f, me, 65000));
      swords.forEach(s => {
        if (['stage', 'return', 'thrown', 'plan'].includes(s.mode) || (s.mode === 'planted' && s.planUntil)) used++;
        else if (s.mode === 'escort' && threatened(allies[s.holder])) used++;
        else if (s.mode === 'orbit' && s.holder == null && fighting) used++;
      });
      allies.forEach((a, k) => { if (!threatened(a)) return; threatTicks++;
        if (swords.some(s => s.holder === k && s.mode === 'escort' && near(s.pos, a, 40000))) covered++; });
    }
    return { utilization: used / (7 * ticks), idle: idleRuns.length ? idleRuns.reduce((a, b) => a + b, 0) / idleRuns.length / TPS : 0,
      coverage: threatTicks ? covered / threatTicks : 1 };
  }
  /** isliid.rs escort_score. */
  function escortScore(i, missing, foes) {
    const m = missing, f = foes;
    return [10 + (m < 25 && f >= 1 ? 15 : 0), 20 + Math.floor(m / 2) + (f >= 2 ? 10 : 0), 15 + (m < 30 ? 15 : 0),
      15 + (m >= 60 ? 25 : 0), 15 + Math.floor(m / 3), 12 + 8 * f, m < 40 ? 18 : 8][i];
  }
  function compare(runs = 50) {
    const rows = RANKS.map((_, rank) => {
      let ms = 0, precision = 0, mult = 0, valid = 0, perfect = 0, util = 0, idleS = 0, cover = 0;
      const mix = Object.fromEntries(NATIVE.GRADES.map(g => [g[0], 0]).concat([['Failed', 0]]));
      for (let i = 0; i < runs; i++) {
        const seed = 70217 + i * 73, s = simulate(rank, state.pattern, state.scale, seed, state.emperor);
        ms += s.ms; precision += s.quality.precision; mult += s.quality.mult;
        mix[s.quality.grade] = (mix[s.quality.grade] || 0) + 1;
        if (s.quality.effectiveness) valid++; if (s.quality.precision >= 95) perfect++;
        const k = simulateSkirmish(rank, seed); util += k.utilization; idleS += k.idle; cover += k.coverage;
      }
      return { rank, ms: ms / runs, precision: precision / runs, mult: mult / runs, mix, valid: valid / runs, perfect: perfect / runs,
        utilization: util / runs, idle: idleS / runs, coverage: cover / runs };
    });
    state.compare = { runs, rows };
    renderResults();
    return rows;
  }
  function reset() {
    state.anchors = Array(7).fill(null); state.slots = Array(targets().length).fill(null);
    state.auto = null; state.drag = null; state.flights = []; state.started = 0; state.result = null;
    state.marks = []; state.empowerment = 0;
    state.placements = 0; state.corrections = 0; state.recalls = 0;
    renderPalette(); renderResults();
  }
  function evaluate() {
    if (state.auto) return;
    let quality = score(state.marks.filter(m => m.until > performance.now()), state.slots);
    if (!quality.effectiveness && state.marks.length) {
      const last=state.marks[state.marks.length-1], length=distance(last.from,last.to);
      const acc=soloQuality(state.rank, state.imperialLevel), g=gradeOf(acc);
      quality={ precision:acc, integrity:100, grade:`Solo ${SWORDS[last.sword][0]} · ${g[0]}`, mult:g[1],
        effectiveness:Math.round(g[1]*clamp(130000*100/Math.max(65000,length*UPX),55,125)/100), emperor:last.sword===6, length, committed:1 };
    }
    state.result = { quality, ms: state.started ? performance.now() - state.started : 0,
      placements: state.placements, corrections: state.corrections, recalls: state.recalls };
    renderResults();
  }
  function autoDraw() {
    reset();
    const sim = simulate(state.rank, state.pattern, state.scale, Math.floor(Math.random() * 1e9), state.emperor);
    state.auto = { sim, started: performance.now(), paletteKey: '' };
    state.slots = sim.slots;
    renderResults();
  }
  function renderPalette() {
    const box = $('#ilPalette'); if (!box) return;
    box.innerHTML = SWORDS.map(([name, color], i) => { const status = swordStatus(i);
      return `<button type="button" class="il-sword${state.selected === i ? ' selected' : ''}${status === 'anchor' ? ' planted' : ''}" data-il-sword="${i}" style="--sword:${color}" title="${esc(name)} - ${status}"><span>${i + 1}</span>${esc(name)}<small>${status}</small></button>`; }).join('');
  }
  function swordStatus(i) {
    if (state.anchors[i]) return 'anchor';
    if (state.flights.some(f => f.sword === i)) return 'flying';
    if (state.auto) {
      const elapsed = (performance.now() - state.auto.started) * state.playback;
      let status = 'ready';
      state.auto.sim.events.forEach(e => { if (e.sword === i && elapsed >= e.start) status = elapsed >= e.end ? 'anchor' : 'flying'; });
      return status;
    }
    return 'ready';
  }
  function renderResults() {
    const box = $('#ilResult'); if (!box) return;
    const q = state.result && state.result.quality;
    box.innerHTML = q ? `<b>${q.grade}</b>${q.mult ? ` ×${(q.mult / 100).toFixed(2)}` : ''} · accuracy <b>${(+q.precision).toFixed(1)}%</b> · integrity <b>${q.integrity}%</b> · shared effect <b>${q.effectiveness}% per sword</b> · path <b>${Math.round(q.length || 0)} px</b> · ${q.committed || 0} committed${q.emperor ? ' (Emperor sword included)' : ''} · R charges ${state.empowerment}<br>
      ${state.result.ms ? fmt(state.result.ms) : '0 s'} to manifest · ${state.result.placements} placements · ${state.result.corrections} redraws · ${state.result.recalls} recalls` :
      state.auto ? `${RANKS[state.rank].name} is staging and engraving with ${state.slots.length} swords in parallel…` : `Drag any sword to stage or engrave. R empowers the next three completions. Charges: ${state.empowerment}.`;
    const tb = $('#ilTable'); if (!tb) return;
    const mixText = mix => NATIVE.GRADES.map(g => g[0]).concat('Failed').filter(k => mix[k]).map(k => `${k[0]}${mix[k]}`).join(' ');
    tb.innerHTML = state.compare ? `<table class="st-table"><tr><th>Mastery</th><th>Points</th><th>Forecast</th><th>Patterns</th><th>Mean time</th><th>Accuracy</th><th>Grades</th><th>Mean ×</th><th>Valid</th><th>Perfect+</th><th>Sword use</th><th>Idle</th><th>Ally cover</th></tr>${state.compare.rows.map(r => `<tr${r.rank === state.rank ? ' class="il-active"' : ''}><td>${RANKS[r.rank].name}</td><td>${RANKS[r.rank].points}</td><td>${LOOK_AHEAD[r.rank]}s</td><td>${RANKS[r.rank].candidates}</td><td>${fmt(r.ms)}</td><td>${r.precision.toFixed(1)}%</td><td>${mixText(r.mix)}</td><td>${(r.mult / 100).toFixed(2)}</td><td>${(r.valid * 100).toFixed(0)}%</td><td>${(r.perfect * 100).toFixed(0)}%</td><td>${(r.utilization * 100).toFixed(0)}%</td><td>${r.idle.toFixed(1)} s</td><td>${(r.coverage * 100).toFixed(0)}%</td></tr>`).join('')}</table><p class="muted">${state.compare.runs} seeds per rank. Accuracy and grades use the native formula and thresholds (Imperial 99, Perfect 95, Refined 85, Stable 70, Crude 60; under 60 the engraving fails) with the native aim error. Sword use, idle time and ally cover come from a 30 s skirmish (2 allies, 3 enemies) run with the native sword control: escorts, leases, idle reclaim, throws and plans. Grades: I Imperial, P Perfect, R Refined, S Stable, C Crude, F Failed.</p>` : '';
  }
  function paintSword(c, p, i, size = 1, now = 0, planted = true, angle = 0, swordState = null) {
    // round 88: flying swords are the game's projectiles (tip right, turned to their heading, wake included);
    // grounded ones stand tip-down at their point
    const name = SWORDS[i][0].toLowerCase(), st = swordState || (planted ? 'planted' : state.previewState);
    if (st === 'flight' || st === 'drawing') {
      c.save(); c.translate(p.x, p.y); c.rotate(angle);
      const ok = paintCentred(c, state.fly, 'fly', `${name}_rank${state.rank}_${st}_f${Math.floor(now / 100) % 4}`, 0, 0, 0, size);
      c.restore(); if (ok) return;
    } else if (paintCentred(c, state.weapons, 'swords', `${name}_rank${state.rank}_${st === 'ready' ? 'ready' : 'planted'}`, now,
      p.x, p.y, size)) return;
    c.strokeStyle = SWORDS[i][1]; c.lineWidth = 4; c.beginPath();
    c.moveTo(p.x, p.y - 45 * size); c.lineTo(p.x, p.y); c.stroke();
  }
  function paintRankBadge(c, now, drift, bob) {
    if (!state.badges || !state.badges.complete || !state.badges.naturalWidth) return;
    c.imageSmoothingEnabled = false;
    const tag = state.rank < 7 ? `rank${state.rank}` : `imperial${state.imperialLevel}`;
    // The 72x96 game buff frame is centred on him like the 48x56 body sprite (round 89: was 48x96).
    paintCentred(c, state.badges, 'badges', tag, now, 94 + drift, 229 + bob, 2);
  }
  function paintAura(c, at, sword, side, now) {
    paintCentred(c, state.auras, 'auras', `aura_${sword}_rank${state.rank}_${side}`, now, at.x, at.y - 27, 2);
  }
  function paintAuraBase(c, at, side, now) {
    paintCentred(c, state.auras, 'auras', `aura_base_rank${state.rank}_${side}`, now, at.x, at.y - 27, 2);
  }
  function paintSwordField(c, at, sword, now) {
    paintCentred(c, state.fields, 'fields', `aura_field_${sword}_rank${state.rank}`, now, at.x, at.y, 2);
  }
  /** The plan's effect logo just above it (native: render_flags, 20000 units above the centre), by native pattern order. */
  function logoTag(phase, pattern = state.pattern) {
    const family = (state.art?.patterns || []).find(p => p.name === PATTERNS[pattern].name)?.family;
    return family ? `logo_${family}_${phase}` : `logo_solo${state.selected}_${phase}`;
  }
  function paintFlag(c, now, phase) {
    const center = formationCenter();
    paintArt(c, state.logos, 'logos', logoTag(phase), now, center.x - 24, center.y - 20000 / UPX - 24, 48, 48);
  }
  const LEGEND = {
    damage: 'Damage', bind: 'Stun', pull: 'Pull enemies inward', push: 'Push enemies out', speed: 'Ally move speed',
    shred: 'Enemy armour down', weaken: 'Enemy attack down', guard: 'Ally damage reduction', attack: 'Ally attack up',
    burst: 'Focused damage at the centre', cooldown: 'Ally cooldowns', heal: 'Heal allies', domain: 'Domain: allies attack, guard, cooldowns; enemies armour down',
  };
  const SOLO_LEGEND = ['Skylight: reveal', 'Terra: slow', 'Darkbringer: armour down', 'Gale: ally speed', 'Blood: damage, leech', 'Rift: pull', 'Emperor: ally attack'];
  /** Round 88: the legend of the 24 x 24 effect logos (13 effect families, 7 solo strokes) in their four phases. */
  function renderLegend() {
    const cv = $('#ilLegend'); if (!cv || !state.art?.logos || !state.logos?.complete || !state.logos.naturalWidth) return;
    const rows = Object.keys(LEGEND).map(f => [f, LEGEND[f], n => `logo_${f}_${n}`])
      .concat(SOLO_LEGEND.map((t, k) => [`solo${k}`, t, n => `logo_solo${k}_${n}`]));
    const c = cv.getContext('2d'); cv.height = rows.length * 30 + 22; c.clearRect(0, 0, cv.width, cv.height);
    c.font = '11px system-ui'; c.fillStyle = '#b6c9d5';
    ['plan', 'draw', 'done', 'fail'].forEach((p, k) => c.fillText(p, 4 + k * 30, 12));
    c.imageSmoothingEnabled = false;
    rows.forEach(([, text, tag], r) => {
      ['planned', 'drawing', 'complete', 'cancelled'].forEach((ph, k) => {
        const f = state.art.logos[tag(ph)]?.frames?.[0]?.data; if (!f) return;
        c.drawImage(state.logos, f.x, f.y, f.w, f.h, 2 + k * 30, 18 + r * 30, 26, 26);
      });
      c.fillStyle = '#e8eef5'; c.fillText(text, 126, 35 + r * 30);
    });
  }
  function draw(now) {
    const c = state.ctx; if (!c) return;
    const cv = state.canvas; c.clearRect(0, 0, W, H);
    c.fillStyle = '#1d2635'; c.fillRect(0, 0, W, H);
    c.strokeStyle = 'rgba(235,242,255,.055)'; c.lineWidth = 1;
    for (let x = 0; x < W; x += 40) { c.beginPath(); c.moveTo(x, 0); c.lineTo(x, H); c.stroke(); }
    for (let y = 0; y < H; y += 40) { c.beginPath(); c.moveTo(0, y); c.lineTo(W, y); c.stroke(); }
    const holder = state.scenario === 'ally' ? {x:W*.72,y:H*.78} : HERO;
    const foe = {x:W*.56,y:H*.51};
    if (state.scenario !== 'self') {
      const p=state.scenario==='ally'?{x:W*.72,y:H*.78}:{x:W*.83,y:H*.28};
      c.fillStyle=state.scenario==='ally'?'#79dbaf':'#f5bf72'; c.beginPath(); c.arc(p.x,p.y,12,0,Math.PI*2); c.fill();
      c.font='12px system-ui'; c.fillText(state.scenario==='ally'?'ALLY':'OBJECTIVE',p.x+16,p.y+4);
    }
    const horizon=LOOK_AHEAD[state.rank], forecast=formationCenter(), current={x:W*.56,y:H*.51};
    if (horizon && distance(current,forecast)>0) {
      c.strokeStyle='rgba(255,130,130,.55)'; c.setLineDash([4,4]); c.beginPath();
      c.moveTo(current.x,current.y); c.lineTo(forecast.x,forecast.y); c.stroke(); c.setLineDash([]);
      c.fillStyle='#ffaaaa'; c.font='11px system-ui'; c.fillText(`Visible movement forecast +${horizon}s`,forecast.x+5,forecast.y-10);
    }
    const plannedLegs = legs(), pat = PATTERNS[state.pattern];
    c.setLineDash([7, 6]); c.strokeStyle = 'rgba(122,205,227,.48)'; c.lineWidth = 2;
    plannedLegs.forEach(({from,to}) => { c.beginPath(); c.moveTo(from.x, from.y); c.lineTo(to.x, to.y); c.stroke(); });
    c.setLineDash([]);
    plannedLegs.forEach(({to:p}, i) => { c.strokeStyle = 'rgba(190,235,244,.7)'; c.beginPath(); c.arc(p.x, p.y, 11, 0, Math.PI * 2); c.stroke(); c.fillStyle = 'rgba(190,235,244,.8)'; c.font = '12px system-ui'; c.fillText(String(i + 1), p.x + 15, p.y + 4); });
    const shown = state.anchors.slice(), flightVisual = Array(7).fill(null);
    if (state.auto) {
      const a = state.auto, t = (now - a.started) * state.playback;
      a.sim.events.forEach(e => {
        if (t >= e.end) shown[e.sword] = e.to;
        else if (t >= e.start) {
          const progress = clamp((t - e.start) / (e.end - e.start), 0, 1);
          const p = { x: e.from.x + (e.to.x - e.from.x) * progress, y: e.from.y + (e.to.y - e.from.y) * progress };
          shown[e.sword] = p;
          flightVisual[e.sword] = { f: e, p, progress, angle: Math.atan2(e.to.y - e.from.y, e.to.x - e.from.x) };
        }
      });
      if (t >= a.sim.ms) {
        state.anchors = a.sim.anchors; state.auto = null;
        state.marks.push(...a.sim.events.filter(e => e.kind === 'draw').map(e => ({ sword: e.sword, from: e.from, to: e.to, until: performance.now() + 30000 })));
        state.result = { quality: a.sim.quality, ms: a.sim.ms, placements: a.sim.slots.length, corrections: a.sim.corrections, recalls: 0 };
        if (state.empowerment) state.empowerment = Math.max(0, state.empowerment - Math.min(3, a.sim.slots.length + 1));
        renderPalette(); renderResults();
      } else {
        const paletteKey = SWORDS.map((_, i) => swordStatus(i)).join('|');
        if (paletteKey !== a.paletteKey) { a.paletteKey = paletteKey; renderPalette(); }
      }
    }
    if (state.flights.length) {
      const landed = [];
      state.flights.forEach(f => {
        const progress = clamp((now - f.started) / f.ms, 0, 1);
        const p = { x: f.from.x + (f.to.x - f.from.x) * progress,
          y: f.from.y + (f.to.y - f.from.y) * progress };
        shown[f.sword] = p;
        flightVisual[f.sword] = { f, p, progress, angle: Math.atan2(f.to.y - f.from.y, f.to.x - f.from.x) };
        if (progress >= 1) { state.anchors[f.sword] = f.kind === 'return' ? null : f.to; if (f.kind === 'draw') {
          state.marks.push({ sword: f.sword, from: f.from, to: f.to, until: performance.now() + 30000 });
          if (state.empowerment) state.empowerment--;
        } landed.push(f); }
      });
      if (landed.length) { state.flights = state.flights.filter(f => !landed.includes(f)); renderPalette();
        if (landed.some(f=>f.kind==='draw')) evaluate(); }
    }
    state.marks = state.marks.filter(m => m.until > now);
    const marks = state.marks.concat(state.auto ? state.auto.sim.events.filter(e => e.kind === 'draw' && (now-state.auto.started)*state.playback >= e.start).map(e => {
      const t=(now-state.auto.started)*state.playback, progress=clamp((t-e.start)/(e.end-e.start),0,1);
      return { sword:e.sword, from:e.from, to:{x:e.from.x+(e.to.x-e.from.x)*progress,y:e.from.y+(e.to.y-e.from.y)*progress} };
    }) : []);
    marks.forEach(m => paintMark(c,m,now));
    if (state.cancelUntil > now) paintFlag(c,now,'cancelled');
    else if (state.auto) paintFlag(c,now,'drawing');
    else if (state.result) paintFlag(c,now,'complete');
    else if (state.slots.length) paintFlag(c,now,'planned');
    if (state.drag) {
      shown[state.drag.sword] = state.drag.now;
      c.strokeStyle = 'rgba(255,255,255,.42)'; c.lineWidth = 1;
      c.beginPath(); c.moveTo(state.drag.from.x, state.drag.from.y);
      c.lineTo(state.drag.now.x, state.drag.now.y); c.stroke();
    }
    // The lab uses 2x native pixels: 40,000 map units span about 80 canvas px.
    const centers=SWORDS.map((_,i)=>shown[i] || (state.scenario==='ally' && i===state.selected ? holder : HERO));
    shown.forEach((p,i)=>{if(p)paintSwordField(c,p,i,now);});
    const recipients=[{p:HERO,side:'ally'}];
    if(state.scenario==='ally')recipients.push({p:holder,side:'ally'});
    recipients.push({p:foe,side:'enemy'});
    recipients.forEach(({p,side})=>{
      const affecting=centers.map((center,i)=>distance(center,p)<=80?i:-1).filter(i=>i>=0);
      if(!affecting.length)return;
      paintAuraBase(c,p,side,now);
      affecting.forEach(i=>paintAura(c,p,i,side,now));
      c.save();
      c.font='11px system-ui';c.textAlign='center';
      c.fillStyle=side==='ally'?'#a6eac9':'#ffaab2';
      c.fillText(`${affecting.length} aura${affecting.length===1?'':'s'} · ${affecting.length===1?'full':`1/${affecting.length}`} strength`,p.x,p.y+33);
      if(side===state.auraSide){
        c.strokeStyle=side==='ally'?'#a6eac9':'#ffaab2';c.lineWidth=1;
        c.strokeRect(p.x-11,p.y-25,22,27);
      }
      c.restore();
    });
    [{p:foe,side:'enemy'},...(state.scenario==='ally'?[{p:holder,side:'ally'}]:[])].forEach(({p,side})=>{
      c.fillStyle=side==='ally'?'#62d6a4':'#e97a88';
      c.fillRect(p.x-6,p.y-20,12,18);
      c.fillStyle='#18283b';c.fillRect(p.x-3,p.y-16,6,5);
    });
    // Every available sword is a separate layer behind the character. A sword
    // vanishes from the orbit as soon as that exact indexed blade leaves it.
    // (round 88: the arsenal ring is drawn over him below, as the game draws the il_ar_* buffs)
    // The sheet's run row is a hovering glide, with no alternating leg contacts.
    if (state.sprite && state.sprite.complete && state.sprite.naturalWidth) {
      const floating = state.floatPreview;
      const frame = Math.floor(now / (floating ? 90 : 200)) % (floating ? 6 : 4);
      const drift = floating ? Math.sin(now / 720) * 12 : 0;
      const bob = floating ? Math.sin(now / 190) * 3 : Math.sin(now / 390) * 2;
      c.imageSmoothingEnabled = false;
      if (floating) {
        c.strokeStyle = 'rgba(85,231,242,.3)'; c.lineWidth = 2;
        c.beginPath(); c.moveTo(37 + drift, 232 + bob); c.lineTo(18 + drift, 237 + bob); c.stroke();
        c.beginPath(); c.moveTo(43 + drift, 247 + bob); c.lineTo(23 + drift, 251 + bob); c.stroke();
      }
      c.drawImage(state.sprite, frame * 48, floating ? 56 : 0, 48, 56,
        46 + drift, 169 + bob, 96, 112);
      paintRankBadge(c, now, drift, bob);
      // the arsenal: each sword still in hand is its own buff on him (selected one glowing); other sword states
      // previewed in a row underneath
      SWORDS.forEach(([name], i) => {
        if (shown[i]) return;
        if (state.previewState === 'orbit') paintCentred(c, state.orbit, 'orbit', `ar_${name.toLowerCase()}_rank${state.rank}${i === state.selected ? '_sel' : ''}`,
          now, 94 + drift, 225 + bob, 2);
        else paintSword(c, { x: 30 + i * 26, y: 450 }, i, 0.5, now, false, 0, state.previewState);
      });
    }
    else { c.fillStyle = '#f5f2e7'; c.fillRect(HERO.x - 12, HERO.y - 18, 24, 37); }
    shown.forEach((p, i) => { if (p) paintSword(c, p, i, 1, now, !flightVisual[i], flightVisual[i] ? flightVisual[i].angle : 0,
      flightVisual[i] ? (flightVisual[i].f.kind === 'draw' ? 'drawing' : 'flight') : 'planted'); });
    c.fillStyle = '#f7d784'; c.font = 'bold 14px system-ui'; c.textAlign = 'center'; c.fillText('ISLIID', HERO.x, 310);
    c.fillStyle = '#b6c9d5'; c.font = '12px system-ui'; c.fillText(state.floatPreview ? 'Floating glide' : 'Imperial hover', HERO.x, 337);
    c.textAlign = 'left'; c.fillStyle = 'rgba(246,241,226,.8)'; c.font = '13px system-ui';
    c.fillText(pat.name + '  ·  ' + state.slots.length + ' swords', 20, 28);
    c.fillStyle = 'rgba(246,241,226,.58)'; c.fillText(pat.effect, 20, 48);
    if (state.started && !state.result && !state.auto) { c.fillStyle = '#fff3bc'; c.fillText('Manual time: ' + fmt(now - state.started), W - 165, 28); }
  }
  function loop(now) {
    state.raf = requestAnimationFrame(loop);
    if (!state.root || state.root.hidden || !state.root.isConnected) return;
    draw(now);
  }
  function point(ev) { const r = state.canvas.getBoundingClientRect(); return { x: clamp((ev.clientX - r.left) * W / r.width, 0, W), y: clamp((ev.clientY - r.top) * H / r.height, 0, H) }; }
  function nearest(p) { let best = -1, d = 20; state.anchors.forEach((a, i) => { if (a && distance(a, p) < d) { best = i; d = distance(a, p); } }); return best; }
  function wireCanvas() {
    const cv = state.canvas;
    cv.addEventListener('contextmenu', ev => { ev.preventDefault(); if (state.auto) return; const i = nearest(point(ev)); if (i >= 0) {
      const now=performance.now(), from=state.anchors[i];
      state.marks.forEach(m => { if (m.sword === i) m.until = Math.min(m.until, now + (ev.shiftKey ? 0 : 1000)); });
      if (state.slots.includes(i) && !state.result) state.cancelUntil = now + 1000;
      state.flights.push({ sword:i, from, to:HERO, started:now, ms:flightMs(i,'return',distance(from,HERO)), kind:'return' });
      state.anchors[i] = null; state.slots = state.slots.map(s => s === i ? null : s); state.recalls++; state.result = null; renderPalette(); renderResults(); } });
    cv.addEventListener('pointerdown', ev => {
      if (ev.button !== 0 || state.auto) return; cv.focus(); const p = point(ev), i = nearest(p);
      const sword = i >= 0 ? i : state.selected;
      if (state.flights.some(f => f.sword === sword)) return;
      const launch=state.scenario==='ally'?{x:W*.72,y:H*.78}:state.scenario==='objective'?{x:W*.83,y:H*.28}:HERO;
      state.drag = { sword, from: state.anchors[sword] || launch, now: p, wasPlanted: !!state.anchors[sword] };
      cv.setPointerCapture(ev.pointerId);
    });
    cv.addEventListener('pointermove', ev => { if (state.drag) state.drag.now = point(ev); });
    cv.addEventListener('pointerup', ev => {
      if (!state.drag || ev.button !== 0) return;
      const d = state.drag, p = point(ev); state.drag = null;
      if (!state.started) state.started = performance.now();
      state.result = null;
      if (d.wasPlanted) state.corrections++;
      else {
        const free = legs().map((leg, i) => ({ leg, i })).filter(x => state.slots[x.i] == null).sort((a, b) => distance(a.leg.to, p) - distance(b.leg.to, p));
        if (free.length) { state.slots[free[0].i] = d.sword; state.placements++; }
      }
      state.anchors[d.sword] = null;
      state.flights.push({ sword:d.sword, from:d.from, to:p, started:performance.now(),
        ms:flightMs(d.sword,state.active?'draw':'stage',distance(d.from,p)), kind:state.active?'draw':'stage' });
      state.selected = d.sword; renderPalette(); renderResults();
    });
    cv.addEventListener('keydown', ev => { if (ev.key.toLowerCase() === 'r') { state.empowerment=3; renderResults(); ev.preventDefault(); } if (ev.key === 'Escape') { reset(); ev.preventDefault(); } });
  }
  function rankFacts() {
    const r = RANKS[state.rank];
    const label = state.rank === 7 ? `Imperial ${state.imperialLevel}` : r.name;
    const points = typeof r.points === 'number' ? `${r.points}+ points` : r.points;
    const badge = state.rank === 7 ? `Imperial badge family · level ${state.imperialLevel}` : `${r.name} badge · unique tier sprite`;
    const effects = [
      '+10,000 range / reveal', '5% guard / 10% slow', '6% physical attack / 8% defence shred',
      '10% movement / 8% attack slow', '6% vamp / 10% heal reduction', '6% radius / 6% slow',
      '6% cooldown / 5% attack reduction'];
    const count = state.scenario === 'ally' ? 1 : Math.max(1, 7-state.anchors.filter(Boolean).length);
    $('#ilRankFacts').innerHTML = `<b>${label}</b><br>${points}<br>Decision ${r.decision} ms per plan<br>Visible-state forecast ${LOOK_AHEAD[state.rank]} s<br>Patterns compared ${r.candidates} / 30<br>Aim error up to ${NATIVE.WOBBLE[state.rank]} units (${r.error.toFixed(1)} px)<br>Swords leave at ${NATIVE.LAUNCH_SPEED} units a tick and speed up to ${NATIVE.TOP_PCT}% of ${NATIVE.SPEED.join(', ')} after ${NATIVE.RAMP_TICKS / TPS} s in the air at every rank (basic-attack throws at full speed; escort strikes ${NATIVE.STRIKE_FAR_PCT / 100}x slower from ${NATIVE.FAR_R} away)<br>Sword art: ${tierName[weaponTier(state.rank)]}<br>${badge}<br><b>${SWORDS[state.selected][0]} aura:</b> ${effects[state.selected]}<br>${count} overlapping swords: numeric effects divide by ${count}, rounded up; reveal stays local and unscaled.`;
  }
  function renderImperialControl() {
    const wrap = $('#ilImperialWrap');
    if (wrap) wrap.style.display = state.rank === 7 ? '' : 'none';
  }
  function save() { try { localStorage.setItem(STORE, JSON.stringify({ rank: state.rank, imperialLevel: state.imperialLevel, pattern: state.pattern, scale: state.scale, emperor: state.emperor, floatPreview: state.floatPreview, playback: state.playback, scenario: state.scenario, active: state.active, previewState: state.previewState, auraSide: state.auraSide, inspectFrame: state.inspectFrame })); } catch (_) {} }
  function mount(root) {
    if (state.root === root && state.canvas) return;
    state.root = root;
    try { const saved = JSON.parse(localStorage.getItem(STORE) || '{}');
      state.rank = clamp(+saved.rank || 0, 0, 7); state.imperialLevel = clamp(+saved.imperialLevel || 1, 1, IMPERIAL_LEVELS); state.pattern = PATTERNS[saved.pattern] ? saved.pattern : 'triangle';
      state.scale = [0.7, 1, 1.3].includes(+saved.scale) ? +saved.scale : 1; state.emperor = !!saved.emperor;
      state.floatPreview = saved.floatPreview !== false;
      state.playback = [1, 2, 4].includes(+saved.playback) ? +saved.playback : 1;
      state.scenario = ['self','ally','objective'].includes(saved.scenario) ? saved.scenario : 'self';
      state.active = saved.active !== false;
      state.previewState = ['orbit','flight','planted','ready','drawing'].includes(saved.previewState) ? saved.previewState : 'orbit';
      state.auraSide = saved.auraSide === 'enemy' ? 'enemy' : 'ally';
      state.inspectFrame = Number.isInteger(saved.inspectFrame) && saved.inspectFrame >= 0 && saved.inspectFrame < 8 ? saved.inspectFrame : -1;
    } catch (_) {}
    root.innerHTML = `<div class="il-shell">
      <div class="il-head"><div><h2>Imperial Engravings lab</h2><p>Stage swords without marks, or activate each one to engrave. Colored lines are persistent sword marks; all swords can draw in parallel.</p></div><span class="il-chip">EDITOR PROTOTYPE</span></div>
      <div class="il-controls">
        <label>Formation<select id="ilPattern">${Object.entries(PATTERNS).map(([k, p]) => `<option value="${k}"${k === state.pattern ? ' selected' : ''}>${esc(p.name)}</option>`).join('')}</select></label>
        <label>Mastery<select id="ilRank">${RANKS.map((r, i) => `<option value="${i}"${i === state.rank ? ' selected' : ''}>${esc(r.name)} (${r.points}${typeof r.points === 'number' ? '+' : ''})</option>`).join('')}</select></label>
        <label id="ilImperialWrap"${state.rank === 7 ? '' : ' style="display:none"'}>Imperial level<select id="ilImperial">${Array.from({ length: IMPERIAL_LEVELS }, (_, i) => i + 1).map(v => `<option value="${v}"${v === state.imperialLevel ? ' selected' : ''}>Imperial ${v}</option>`).join('')}</select></label>
        <label>Scale<select id="ilScale">${[[0.7, 'Small'], [1, 'Medium'], [1.3, 'Large']].map(([v, n]) => `<option value="${v}"${v === state.scale ? ' selected' : ''}>${n}</option>`).join('')}</select></label>
        <label>Playback<select id="ilPlayback">${[1, 2, 4].map(v => `<option value="${v}"${v === state.playback ? ' selected' : ''}>${v}x</option>`).join('')}</select></label>
        <label>Assignment<select id="ilScenario">${[['self','All with Isliid'],['ally','Supporting an ally'],['objective','Staged at objective']].map(([v,n])=>`<option value="${v}"${state.scenario===v?' selected':''}>${n}</option>`).join('')}</select></label>
        <label>Sword state<select id="ilSwordState">${['orbit','flight','planted','ready','drawing'].map(v=>`<option value="${v}"${state.previewState===v?' selected':''}>${v}</option>`).join('')}</select></label>
        <label>Aura preview<select id="ilAuraSide"><option value="ally"${state.auraSide==='ally'?' selected':''}>Friendly</option><option value="enemy"${state.auraSide==='enemy'?' selected':''}>Enemy</option></select></label>
        <label>Animation frame<select id="ilFrame"><option value="-1"${state.inspectFrame<0?' selected':''}>Play all 8</option>${Array.from({length:8},(_,i)=>`<option value="${i}"${state.inspectFrame===i?' selected':''}>Frame ${i+1}</option>`).join('')}</select></label>
        <label class="il-check"><input type="checkbox" id="ilActive"${state.active?' checked':''}> Activate engraving on drag</label>
        <label class="il-check"><input type="checkbox" id="ilEmperor"${state.emperor ? ' checked' : ''}> Auto includes Emperor sword</label>
        <label class="il-check"><input type="checkbox" id="ilFloat"${state.floatPreview ? ' checked' : ''}> Preview floating glide</label>
      </div>
      <div class="il-main"><div><canvas id="ilCanvas" tabindex="0" width="${W}" height="${H}"></canvas>
        <div class="il-actions"><button class="btn small primary" data-il="auto">Auto draw this rank</button><button class="btn small" data-il="compare">Compare all ranks (50 runs)</button><button class="btn small" data-il="manifest">Empower next 3 (R)</button><button class="btn small" data-il="cancel">Cancel engraving</button><button class="btn small" data-il="reset">Reset (Esc)</button></div>
        <div id="ilResult" class="il-result"></div></div>
        <aside class="il-side"><h3>Seven swords</h3><div id="ilPalette" class="il-palette"></div>
          <p class="muted">Select a sword and drag it to stage or draw. Any sword can leave several colored marks. Auto draw stages and launches swords in parallel. An activated sword cannot be recalled until its stroke finishes. Then right-click to recall at 1.5× speed; Shift + right-click erases its marks instantly. R empowers the next three completions.</p>
          <h3>Mastery model</h3><div id="ilRankFacts"></div>
          <h3>Engraving logos</h3><canvas id="ilLegend" class="il-legend" width="330" height="40"></canvas>
          <p class="muted">The game shows a plan's logo just above it: dashed ring = planned, gold ticks = drawing, check = done (graded), red slash = cancelled or failed (under 60% accuracy).</p>
          <p class="muted">Official game: 1 point; scrim or exhibition: 0.5; win: 1.5×. Generate Isliid mastery in Skill Test → Mastery. This canvas previews drawing; the tactical AI runs in live matches.</p></aside></div>
      <div id="ilTable" class="il-table"></div></div>`;
    state.canvas = $('#ilCanvas'); state.ctx = state.canvas.getContext('2d');
    state.sprite = new Image(); state.sprite.src = '/isliid-sprite-sheet.png';
    state.weapons = new Image(); state.weapons.src = '/isliid-swords8-8.png';
    state.badges = new Image(); state.badges.src = '/isliid-badges8-8.png';
    state.auras = new Image(); state.auras.src = '/isliid-auras8-8.png';
    state.fields = new Image(); state.fields.src = '/isliid-aura_fields8-8.png';
    state.fly = new Image(); state.fly.src = '/isliid-swords_fly8-8.png';
    state.orbit = new Image(); state.orbit.src = '/isliid-orbit8-8.png';
    state.logos = new Image(); state.logos.src = '/isliid-logos-8.png';
    state.logos.onload = () => renderLegend();
    fetch('/isliid-art-manifest.json').then(r => r.json()).then(art => { state.art = art; renderLegend(); }).catch(() => {});
    state.trails = new Image(); state.trails.src = '/isliid-trails.png';
    wireCanvas();
    root.addEventListener('click', ev => {
      const s = ev.target.closest('[data-il-sword]'); if (s) { state.selected = +s.dataset.ilSword; renderPalette(); rankFacts(); state.canvas.focus(); return; }
      const b = ev.target.closest('[data-il]'); if (!b) return;
      if (b.dataset.il === 'auto') autoDraw(); if (b.dataset.il === 'compare') compare();
      if (b.dataset.il === 'manifest') { state.empowerment=3; renderResults(); }
      if (b.dataset.il === 'cancel') { state.cancelUntil=performance.now()+1000; state.auto=null; state.flights=state.flights.filter(f=>f.kind!=='draw'); state.marks.forEach(m=>{m.until=Math.min(m.until,performance.now()+1000);}); state.result=null; renderResults(); }
      if (b.dataset.il === 'reset') reset();
    });
    root.addEventListener('change', ev => {
      const t = ev.target;
      if (t.id === 'ilRank') { state.rank = +t.value; renderImperialControl(); }
      if (t.id === 'ilImperial') state.imperialLevel = clamp(+t.value, 1, IMPERIAL_LEVELS);
      if (t.id === 'ilPattern') { state.pattern = t.value; state.compare = null; reset(); }
      if (t.id === 'ilScale') { state.scale = +t.value; state.compare = null; reset(); }
      if (t.id === 'ilPlayback') state.playback = +t.value;
      if (t.id === 'ilScenario') { state.scenario=t.value; state.compare=null; reset(); }
      if (t.id === 'ilSwordState') state.previewState=t.value;
      if (t.id === 'ilAuraSide') state.auraSide=t.value;
      if (t.id === 'ilFrame') state.inspectFrame=+t.value;
      if (t.id === 'ilActive') state.active=t.checked;
      if (t.id === 'ilEmperor') { state.emperor = t.checked; state.compare = null; reset(); }
      if (t.id === 'ilFloat') state.floatPreview = t.checked;
      rankFacts();
      save(); renderResults();
    });
    renderImperialControl(); rankFacts();
    reset();
    if (!state.raf) state.raf = requestAnimationFrame(loop);
  }

  const api = { mount, simulate, simulateSkirmish, compare, score, targets, weaponTier, logoTag, selfTest, formationAccuracy, gradeOf, planWobble,
    wobbleOf, soloQuality, noticePct, notices,
    escortScore, NATIVE, PATTERNS, RANKS, SWORDS, _state: state };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (typeof window !== 'undefined') window.TFM2IsliidLab = api;
})();
