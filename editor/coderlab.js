/* The Coder's Code lab (round 103): watch any rank write any function, run a 30 s arena per rank, compare them all.
 *
 * The writing is a bit-exact mirror of native/tfm2_custom_ai/src/coder.rs (the tables, the xorshift Rng, the Typing
 * machine: typing with a typo roll per character, the review, the fixes, the compile, Rust's borrow checker, the AI
 * copilots' knobs); tools/verify_coder.py checks the tables match and that the lab reproduces the native runs in
 * native/tfm2_custom_ai/src/coder_vectors.txt. The arena is a reduced model of the native brain (his choice of what to
 * write, the program, the rig, the AI ult, the bugs at run time) on a one-dimensional lane: an ally pressed by one
 * enemy, a second enemy diving him at 10 s. Its numbers compare the ranks; they are not a match.
 */
(function () {
  'use strict';

  const CODE = (typeof window !== 'undefined' && window.TFM2_CODER) ||
    (typeof require !== 'undefined' ? require('./coder-code.js') : null);

  // ---------------------------------------------------------------- mirror of coder.rs: the tables
  const NATIVE = {
    RANK_NAMES: ['Script Kiddie', 'Intern', 'Junior', 'Developer', 'Senior', 'Staff', 'Architect', 'Root'],
    ROOT: 7,
    CPS100: [450, 750, 1050, 1350, 1800, 2250, 2850, 3300], CPS100_TOP: 4500,
    TYPO: [420, 310, 220, 150, 95, 55, 28, 14], TYPO_TOP: 0,
    NOTICE: [15, 30, 45, 60, 75, 87, 94, 97], NOTICE_TOP: 100,
    AWARE: [20, 35, 50, 62, 75, 85, 93, 96], AWARE_TOP: 100,
    CLOCK: [60, 40, 30, 24, 20, 15, 12, 11], CLOCK_TOP: 10,
    READ: [30, 22, 16, 11, 7, 4, 2, 1], READ_TOP: 0,
    IQ: [10, 25, 40, 55, 70, 85, 93, 97], IQ_TOP: 100,
    OC_OFF: [103, 101, 98, 95, 92, 90, 89, 89], OC_OFF_TOP: 88,
    OC_LAG: [60, 45, 30, 20, 12, 8, 4, 2], OC_LAG_TOP: 0,
    PROMPT: [200, 180, 160, 140, 120, 100, 80, 70], PROMPT_TOP: 50,
    // Lang {typo, syntax, compile, power, heat, ram, catch, bugs[7]}: py, js, ts, cpp, rust, go, java, cs, lua, hs, sh, sql, asm
    LANG: [
      { typo: 80, syntax: 50, compile: 0, power: 80, heat: 150, ram: 150, catch: 0, bugs: [20, 25, 10, 35, 10, 0, 0] },
      { typo: 90, syntax: 40, compile: 0, power: 90, heat: 150, ram: 120, catch: 0, bugs: [15, 15, 10, 30, 30, 0, 0] },
      { typo: 95, syntax: 45, compile: 40, power: 95, heat: 160, ram: 120, catch: 50, bugs: [15, 20, 10, 20, 20, 0, 0] },
      { typo: 120, syntax: 60, compile: 60, power: 130, heat: 400, ram: 80, catch: 0, bugs: [15, 20, 10, 0, 10, 25, 20] },
      { typo: 110, syntax: 70, compile: 150, power: 120, heat: 250, ram: 70, catch: 80, bugs: [25, 35, 20, 0, 20, 0, 0] },
      { typo: 85, syntax: 45, compile: 30, power: 105, heat: 200, ram: 90, catch: 30, bugs: [20, 20, 15, 10, 10, 10, 0] },
      { typo: 100, syntax: 55, compile: 120, power: 110, heat: 250, ram: 160, catch: 40, bugs: [15, 20, 10, 30, 10, 0, 10] },
      { typo: 100, syntax: 55, compile: 80, power: 110, heat: 250, ram: 120, catch: 40, bugs: [18, 22, 12, 15, 12, 8, 5] },
      { typo: 70, syntax: 40, compile: 0, power: 75, heat: 120, ram: 60, catch: 0, bugs: [20, 25, 10, 30, 15, 0, 0] },
      { typo: 150, syntax: 60, compile: 260, power: 120, heat: 250, ram: 90, catch: 95, bugs: [20, 20, 20, 10, 20, 0, 0] },
      { typo: 95, syntax: 45, compile: 0, power: 95, heat: 180, ram: 70, catch: 0, bugs: [35, 20, 15, 10, 15, 0, 0] },
      { typo: 90, syntax: 50, compile: 0, power: 95, heat: 150, ram: 100, catch: 30, bugs: [20, 25, 10, 15, 10, 0, 0] },
      { typo: 160, syntax: 30, compile: 0, power: 160, heat: 600, ram: 40, catch: 0, bugs: [15, 15, 30, 0, 10, 30, 0] },
    ],
    // (cooldown ticks, CPU load % x100, RAM MB while it lasts, base value, how long it lasts); 100 functions
    SPEC: [
      [45, 600, 0, 40, 0], [360, 1500, 2000, 30, 180], [300, 1500, 0, 25, 0], [600, 1000, 1000, 15, 180], [180, 1800, 0, 30, 0], [600, 1200, 0, 20, 0],
      [300, 1000, 1500, 25, 120], [480, 0, 0, 10, 0], [300, 2200, 0, 45, 0], [600, 2500, 4000, 40, 240], [300, 2000, 0, 35, 0], [480, 1500, 0, 25, 0],
      [900, 3000, 3000, 35, 180], [900, 2500, 4000, 45, 360], [900, 2000, 0, 30, 0], [1200, 3000, 0, 40, 0], [600, 1500, 1500, 30, 180], [1200, 3500, 0, 45, 0],
      [600, 2500, 0, 50, 0], [1200, 3000, 0, 45, 0], [600, 2000, 0, 40, 0], [900, 2500, 0, 45, 0], [900, 2500, 0, 40, 0], [1800, 0, 2000, 40, 480],
      [300, 1500, 2000, 25, 360], [600, 2000, 0, 30, 0], [900, 2500, 3000, 25, 240], [600, 1000, 1000, 15, 480], [600, 1500, 1500, 25, 180], [480, 1500, 0, 25, 0],
      [360, 1800, 0, 30, 0], [900, 1000, 1000, 20, 480], [480, 2000, 0, 30, 0], [600, 2500, 0, 35, 0], [600, 2000, 0, 30, 0], [900, 1000, 2000, 20, 999],
      [600, 1000, 0, 25, 0], [480, 1500, 0, 30, 0], [480, 2000, 0, 30, 0], [480, 1000, 0, 25, 0], [600, 1000, 0, 25, 0], [900, 500, 0, 25, 0],
      [900, 2500, 0, 40, 0], [600, 1500, 0, 30, 0], [480, 2000, 0, 35, 0], [600, 2000, 0, 35, 0], [600, 1000, 1000, 15, 240], [480, 1500, 0, 30, 0],
      [480, 1500, 0, 25, 0], [900, 2500, 4000, 40, 360], [600, 2500, 0, 50, 0], [600, 1200, 0, 20, 0], [900, 3000, 0, 40, 0], [600, 1800, 0, 30, 0],
      [600, 2500, 0, 50, 0], [480, 1200, 0, 20, 0], [480, 1500, 0, 30, 0], [480, 2200, 0, 35, 0], [480, 3000, 0, 55, 0], [900, 1000, 2000, 25, 480],
      [600, 1000, 1500, 20, 999], [600, 2000, 0, 25, 0], [480, 2000, 0, 40, 0], [900, 1000, 2000, 20, 480], [600, 1800, 0, 30, 0], [480, 1500, 1000, 25, 180],
      [480, 2500, 0, 45, 0], [900, 1000, 1500, 15, 480], [480, 1000, 0, 20, 0], [900, 1000, 1500, 15, 480], [600, 2500, 0, 40, 0], [600, 1500, 2000, 25, 180],
      [900, 500, 1500, 20, 999], [600, 1500, 0, 30, 0], [900, 2000, 0, 35, 0], [900, 2500, 0, 40, 0], [600, 2000, 0, 30, 0], [900, 3000, 0, 45, 0],
      [900, 500, 1500, 20, 999], [900, 500, 1000, 15, 480], [480, 1500, 0, 25, 0], [480, 2200, 0, 35, 0], [300, 2000, 0, 30, 0], [600, 1500, 1500, 25, 360],
      [480, 1500, 0, 25, 0], [600, 1000, 0, 25, 0], [900, 1000, 1500, 20, 480], [480, 1800, 0, 30, 0], [480, 1500, 0, 25, 0], [600, 1000, 1500, 20, 240],
      [600, 1000, 0, 25, 0], [480, 1500, 0, 25, 0], [480, 1500, 0, 25, 0], [600, 2000, 0, 30, 0], [600, 1500, 0, 25, 0], [600, 2000, 0, 45, 0],
      [480, 2000, 0, 30, 0], [900, 1000, 1500, 20, 360], [600, 1800, 0, 30, 0], [900, 1000, 1500, 20, 240]
    ],
    IDEAL: [0, 0, 0, 1, 0, 0, 0, 1, 3, 4, 3, 0, 0, 4, 4, 0, 4, 3, 4, 4, 3, 4, 3, 4, 5, 5, 5, 10, 5, 1, 1, 10, 5, 0, 5, 5, 10, 10, 10, 10, 10, 10, 0, 10, 11, 3, 3, 1, 5, 3, 3, 10, 10, 1, 4, 10, 1, 3, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 11, 11, 6, 6, 10, 11, 11, 6, 11, 6, 5, 3, 10, 5, 5, 1, 5, 2, 1, 1, 1, 5, 10, 10, 10, 10, 3, 3, 3, 3, 1, 3],
    KIND: [false, false, false, false, false, true, false, false, false, false, false, true, false, false, true, true, false, false, true, true, false, true, true, true, false, true, false, false, false, true, true, false, true, true, true, false, true, true, true, true, true, true, true, true, true, true, false, true, true, false, true, true, true, true, true, true, true, true, true, false, false, true, true, false, true, true, true, false, true, false, true, false, false, true, true, true, true, true, false, false, true, true, true, false, true, true, false, true, true, false, true, true, true, true, true, true, true, false, true, false],  // true: script
    // round 107: daemon program slots by rank (Script Kiddie..Architect, then Root, Zero-Day): see slots()
    SLOTS_RANK: [4, 5, 5, 6, 7, 8, 9, 10],
    // round 108: six parts (ram, disk, ssd, cool, cpu, gpu), each tier 0..4 (3 a workstation, 4 a data center)
    MAX_TIER: 4, NPARTS: 6,
    PRICE: [[60, 160, 450, 1100], [40, 110, 300, 800], [50, 130, 350, 900], [50, 150, 400, 1000], [80, 220, 600, 1500], [90, 260, 700, 1800]],
    RAM_MB: [32000, 64000, 128000, 512000, 2000000], STORAGE: [16, 32, 64, 128, 256], RELOAD: [48, 24, 6, 3, 0],
    COMPILE_PCT: [100, 80, 60, 50, 40], COOLING: [8, 12, 17, 24, 34], GHZ: [300, 360, 420, 500, 560],
    CPU_LOAD: [100, 100, 100, 50, 35], GPU_POWER: [25, 70, 100, 120, 150], GPU_HEAT: [300, 120, 150, 180, 250],
    PART_W: [[5, 10, 20, 60, 200], [5, 10, 15, 40, 80], [5, 5, 8, 15, 30], [0, 5, 20, 60, 150], [65, 90, 125, 280, 560], [0, 200, 450, 300, 700]],
    BREAKER_W: 1800, UPKEEP: 25, BILL_RESUME: 2000, PUMP_FAIL: 1, PUMP_TICKS: 300, OUTAGE_TICKS: 180, START_CREDIT: 25000,
    FX_BUDGET: 12, FX_REFILL: 5,
    // round 108: each new function (index f - 24): [trigger, range, class, a, b] (coder.rs NB)
    NB: [
      [21, 70000, 0, 35, 50],
      [9, 0, 7, 90, 0],
      [8, 0, 6, 0, 0],
      [12, 0, 6, 0, 0],
      [3, 0, 5, 30, 180],
      [7, 0, 5, 25, 180],
      [1, 80000, 1, 25, 30],
      [8, 0, 6, 0, 0],
      [0, 80000, 6, 50, 0],
      [1, 80000, 6, 60, 0],
      [6, 50000, 7, 180, 0],
      [8, 0, 6, 2, 0],
      [16, 0, 2, 0, 0],
      [0, 90000, 4, 20, 300],
      [1, 50000, 4, 3000, 14],
      [22, 0, 3, 160, 0],
      [23, 0, 5, 0, 0],
      [20, 0, 6, 0, 0],
      [1, 60000, 4, 60, 0],
      [19, 0, 6, 0, 0],
      [25, 60000, 4, 120, 0],
      [0, 60000, 4, 120, 0],
      [0, 70000, 7, 240, 0],
      [0, 60000, 4, 90, 0],
      [3, 0, 5, 30, 180],
      [5, 0, 0, 3, 0],
      [0, 55000, 0, 150, 70],
      [4, 0, 7, 120, 0],
      [1, 80000, 1, 12, 8],
      [0, 70000, 4, 60, 0],
      [0, 70000, 0, 25, 0],
      [1, 80000, 4, 80000, 0],
      [0, 70000, 2, 60, 180],
      [0, 60000, 0, 10, 5],
      [0, 80000, 0, 120, 80],
      [8, 0, 6, 0, 0],
      [8, 0, 6, 15, 0],
      [1, 120000, 4, 180, 0],
      [0, 70000, 0, 40, 0],
      [13, 0, 6, 20, 0],
      [0, 70000, 4, 120, 0],
      [2, 0, 2, 25, 5],
      [18, 80000, 0, 140, 60],
      [8, 0, 6, 80, 0],
      [1, 120000, 4, 40, 0],
      [8, 0, 6, 0, 0],
      [1, 90000, 1, 180, 0],
      [14, 0, 3, 0, 0],
      [15, 0, 2, 30, 0],
      [9, 0, 7, 0, 0],
      [24, 0, 6, 3, 0],
      [1, 80000, 4, 90, 0],
      [1, 80000, 4, 2400, 20],
      [1, 120000, 1, 90, 0],
      [8, 0, 6, 0, 0],
      [8, 0, 6, 15, 0],
      [6, 70000, 4, 40, 120],
      [0, 55000, 0, 30, 25],
      [1, 60000, 1, 8, 10],
      [0, 70000, 0, 20, 240],
      [0, 55000, 4, 40, 180],
      [23, 0, 5, 180, 0],
      [10, 0, 0, 20, 20],
      [1, 70000, 4, 60, 0],
      [0, 60000, 4, 180, 0],
      [7, 0, 5, 10, 180],
      [20, 0, 6, 150, 0],
      [25, 60000, 4, 180, 0],
      [0, 60000, 4, 30, 180],
      [5, 80000, 1, 20, 20],
      [9, 0, 7, 0, 0],
      [17, 70000, 0, 20, 0],
      [1, 60000, 4, 3000, 0],
      [13, 0, 6, 50, 0],
      [0, 70000, 4, 90, 180],
      [11, 0, 5, 40, 120]
    ],
    GPU_FX: ['cuda_kernel', 'ray_tracing', 'quantum', 'deepfake', 'diffusion', 'overfit', 'map_reduce'],
    META: ['kubernetes', 'cron', 'canary_deploy', 'chaos_monkey', 'llm_agent', 'hotfix', 'dynamic_prog'],
    // MODELS[provider][flagship, lite]: {cps100, syntax, logic, think, per_prompt}
    MODELS: [
      [{ cps100: 4000, syntax: 100, logic: 300, think: 0, per_prompt: 1 }, { cps100: 7000, syntax: 300, logic: 600, think: 0, per_prompt: 1 }],
      [{ cps100: 4500, syntax: 300, logic: 100, think: 150, per_prompt: 1 }, { cps100: 7000, syntax: 400, logic: 900, think: 0, per_prompt: 1 }],
      [{ cps100: 5000, syntax: 400, logic: 300, think: 0, per_prompt: 3 }, { cps100: 7000, syntax: 800, logic: 600, think: 0, per_prompt: 1 }],
    ],
    POOL: [15000, 24000, 36000], REFILL: [50, 90, 130], AI_TICKS: 720, AI_COOLDOWN: 2400, BSOD_SHY: 600, HOT_SKIP: 9500,
    HI_RANK: 6,
  };
  /** What each function does (the 76 new ones: coder.rs NB's comments). */
  const DESC = ["a packet hits the nearest enemy", "a shield on an ally under pressure", "heals the most-hurt ally", "reads every enemy's health exactly for 3 s", "hex glyphs blast every enemy around him", "escapes a diver (blinks away)", "an enemy slowed", "idles to clear his CPU", "a zap jumping between up to 4 enemies", "a burning wall between him and an enemy", "20 quick hits on one enemy", "frees a stunned ally and shields them", "allies near him attack faster", "2 drones that ping the nearest enemy", "a diver on an ally is thrown back", "lines enemies up by health, marks the weakest", "an ally takes half damage for 3 s", "8 hits on every enemy near him", "executes an enemy under 15%", "an ally's health or an enemy's position back to 3 s ago", "8 hits that seek the nearest enemy", "an enemy stunned 1 s and hit", "hits and unshields enemies under 30%", "his program runs at 150% and half the load for 8 s", "a cloud node pings (no local CPU; 0.5 BTC a run)", "an endangered ally is containerised (untargetable 1.5 s)", "every daemon run gets a 50% replica", "re-runs his last script (every 8 s)", "the ally under pressure takes -30% for 3 s", "allies near him +25% move speed for 3 s", "a hit on every enemy in range (no local CPU; 0.3 BTC each)", "half the logic bugs of every later compile are caught", "his strongest daemon test-run at 50% on one enemy", "three random effects from his program, random targets, 60%", "a server-block wall between him and the diver", "+2 program slots while enemies outnumber his team near him", "his health back to 3 s ago", "the most dangerous enemy near: marked +20% damage taken for 5 s", "knocks back every enemy near him", "a shield stored, popped by itself when he drops under 30%", "copies an ally's best buff onto himself", "resets his daemons' cooldowns", "two enemies collide: both stunned 1 s", "patches his program's worst bug, then runs that function at once", "DROP TABLE: an enemy's buffs and shields are gone", "an enemy's skills locked 2 s", "one enemy's health read exactly and marked for 4 s", "an enemy taunted onto his tankiest ally for 1.5 s", "an ally reflects 30% of damage for 3 s", "3-5 mini drones (by rank)", "a huge hit; 40% he segfaults himself", "invisible 2 s", "12 hits over every enemy; his CPU maxed", "an enemy pulled toward his team, charmed 1 s", "true damage, 25% of max health", "every enemy near read exactly; the weakest marked", "steals an enemy's healing for 3 s (he heals instead)", "10 hits, each harder", "a massive beam on one target", "AI pools refill x2; lite models write like flagships", "+2% power a run this fight, up to +30%", "every enemy near read exactly and marked 3 s", "50/50: a triple hit or nothing", "a mini AI writes one script for him (every 12 s)", "an enemy fears its own shadow 2 s (attacks nothing)", "heal over time on the most-hurt ally, \"denoising\"", "huge damage to the last enemy he hit", "his typos -20% while it's in his program", "SELECT: every enemy under 40% read exactly and marked", "his program checks its triggers twice as often", "one big hit split over up to 3 enemies", "his last shield copied onto a second ally", "30% of his health restored when he'd drop under 15% (once a minute)", "an endangered ally teleported to his side", "his next 3 scripts run as one combo (+30% each)", "two enemies frozen 1.5 s", "nearby enemies pulled together onto him", "every visible enemy in 120000 hit, split", "kills and assists mint double Bitcoin and a shield", "15% less damage taken while it's in his program", "the diver slowed 40% for 2 s", "SYN, SYN-ACK, ACK: 3 hits, the 3rd stuns", "a fast spray with no aim", "a tether pinging one enemy every 0.5 s for 4 s", "an enemy's attack speed -40% for 3 s", "borrows an ally's buff for 3 s", "when an ally is hit, pings the attacker", "the enemy with the lowest attack stunned 1 s", "an enemy can't be healed for 3 s", "allies near him +10% attack for 3 s", "one daemon's cooldown overridden; his next run +50%", "an enemy's buffs stripped, and kept off for 3 s", "an enemy slowed 30% for 3 s", "every enemy process near him killed: shields gone and a hit", "dashes him along the shortest path to an endangered ally", "an enemy under 20% halved", "the weakest enemy pulled to the front (onto him)", "his last effect again at 50% (every 6 s)", "a trap where the enemy stands: the first in is rooted 1.5 s", "an ally hit by two or more takes -40% for 2 s"];
  const N = NATIVE, ROOT = N.ROOT, NF = CODE.FUNCS.length;
  const fIndex = name => CODE.FUNCS.findIndex(fn => fn.name === name);
  const GPU_FX = N.GPU_FX.map(fIndex), META = N.META.map(fIndex);
  const isScript = f => !!CODE.FUNCS[f].script;
  const passive = f => f >= 24 && N.NB[f - 24][0] === 8;
  const klass = f => f >= 24 ? N.NB[f - 24][2] : 0;
  const CLASS_NAMES = ['hit', 'area', 'heal', 'shield', 'crowd control', 'ally buff', 'passive / meta', 'utility'];
  /** mirror of start_tiers() */
  const startTiers = rank => rank <= 2 ? [0, 0, 0, 0, 0, 0] : rank === 3 ? [1, 1, 1, 1, 1, 1] : rank === 4 ? [2, 1, 2, 1, 1, 1]
    : rank === 5 ? [2, 2, 2, 2, 2, 2] : rank === 6 ? [3, 3, 3, 3, 3, 3] : [4, 4, 4, 4, 4, 4];
  const [PY, JS, TS, CPP, RUST, GO, JAVA, CS, LUA, HS, SH, SQL, ASM] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
  /** mirror of slots(): daemon program slots by rank. */
  const slots = (rank, root) => rank < ROOT ? N.SLOTS_RANK[rank] : (root === 1 ? 12 : 11);
  /** mirror of avail(): a function is written in a subset of the 13 languages. */
  const avail = (f, lang) => lines(f, lang).length > 0;
  const F = { PING: 0, SHIELD: 1, HEAL: 2, SCAN: 3, SPRAY: 4, BLINK: 5, SLOW: 6, CACHE: 7, CHAIN: 8, FIREWALL: 9, DDOS: 10,
    CLEANSE: 11, BOOST: 12, FORK: 13, SWAP: 14, SORT: 15, ENCRYPT: 16, DDOS_ALL: 17, KILL9: 18, ROLLBACK: 19, RECURSE: 20,
    INJECT: 21, GC: 22, DEPLOY: 23 };
  const RAM = 0, DISK = 1, SSD = 2, COOL = 3, CPU = 4, GPU = 5;
  const CLAUDE = 0, GPT = 1, GEMINI = 2;
  const PROVIDERS = ['Claude', 'ChatGPT', 'Gemini'], MODEL_NAMES = [['Claude Max 20x', 'Claude Haiku'], ['ChatGPT Pro', 'GPT mini'], ['Gemini AI Ultra', 'Gemini Flash']];
  const BUGS = ['wrong target', 'off-by-one', 'infinite loop', 'null reference', 'sign flip', 'segfault', 'leak'];
  const [WRONG, OFF, LOOP, NULLREF, FLIP, SEGV, LEAK] = [0, 1, 2, 3, 4, 5, 6];
  const LANG_NAMES = ['Python', 'JavaScript', 'TypeScript', 'C++', 'Rust', 'Go', 'Java', 'C#', 'Lua', 'Haskell', 'Bash', 'SQL', 'Assembly'];
  const TPS = 60;
  const fl = Math.floor;

  /** mirror of tv(): a table's value at rank; Root interpolates from #10 (the table's last entry) to #1 (top). */
  function tv(table, top, rank, root) {
    if (rank < ROOT) return table[rank];
    const p = Math.min(10, Math.max(1, root == null ? 10 : root));
    const a = table[ROOT], b = top;
    return a + Math.trunc((b - a) * (10 - p) / 9);
  }
  const t = (name, rank, root) => tv(N[name], N[name + '_TOP'], rank, root);
  /** mirror of ideal() */
  const ideal = (f, rank) => (rank >= 6 && avail(f, ASM) && [F.CHAIN, F.DDOS, F.RECURSE].includes(f)) ? ASM : N.IDEAL[f];
  /** mirror of lines() / chars() / tier(): from coder_code.rs (here coder-code.js, generated from the same source) */
  const lines = (f, lang) => CODE.FUNCS[f].lens[CODE.LANGS[lang]] || [];
  const chars = (f, lang) => lines(f, lang).reduce((s, l) => s + l[0], 0);
  const tier = f => CODE.FUNCS[f].tier;
  /** mirror of compile_ticks() */
  const compileTicks = (lang, rank) => { const c = N.LANG[lang].compile; return lang === RUST ? fl(c * (100 + 15 * (ROOT - Math.min(rank, ROOT))) / 100) : c; };
  /** Round 106: mirror of theme_of() / fit_of(): each rank's editor theme and outfit (coder_code.py THEMES). */
  const themeOf = (rank, root) => rank === 0 ? 0 : rank <= 2 ? 1 : rank <= 4 ? 2 : rank === 5 ? 3 : rank === 6 ? 4 : (root === 1 ? 6 : 5);
  const fitOf = (rank, root) => rank < ROOT ? rank : (root === 1 ? 8 : 7);
  // window bg, tab strip, frame, status bar, accent (coder_code.py THEMES)
  const THEME_COL = [
    { bg: '#060a08', strip: '#0a120d', frame: '#3cff8c', status: '#0c2214', accent: '#5aff96', name: 'green terminal' },
    { bg: '#1e1208', strip: '#2a1a0b', frame: '#ffaa3c', status: '#46280a', accent: '#ffc464', name: 'amber CRT' },
    { bg: '#1a1e2a', strip: '#141721', frame: '#465678', status: '#245cc4', accent: '#ebf2ff', name: 'dark editor' },
    { bg: '#1c1c1e', strip: '#141416', frame: '#d6ac4a', status: '#3e3216', accent: '#ffd878', name: 'charcoal and gold' },
    { bg: '#12326c', strip: '#0e2858', frame: '#e4f4ff', status: '#0a2250', accent: '#8cecff', name: 'blueprint' },
    { bg: '#080406', strip: '#120609', frame: '#ff384c', status: '#400a14', accent: '#ff606c', name: 'red on black' },
    { bg: '#060608', strip: '#0e0d0a', frame: '#ffd87a', status: '#1e190a', accent: '#ffde80', name: 'black and gold' },
  ];
  // the shared code palette (coder_code.py NAME / KEY / NUM / STR / SYM)
  const PAL = { name: 'rgb(226,232,245)', key: 'rgb(150,205,255)', num: 'rgb(255,178,132)', str: 'rgb(242,216,130)', sym: 'rgb(152,162,180)' };
  /** mirror of rig_tier(): Senior 1 .. Architect 3, Root 4, Zero-Day 5 */
  const rigTier = (rank, root) => rank === 4 ? 1 : rank === 5 ? 2 : rank === 6 ? 3 : rank >= ROOT ? (root === 1 ? 5 : 4) : 0;
  /** mirror of needed() */
  function needed(tiers, ooms, overwrites, reloads, hotSecs, bsods) {
    const score = [ooms * 3, overwrites * 2, reloads, bsods * 4 + fl(hotSecs / 10), 1, 1];
    let best = null;
    for (let p = 0; p < N.NPARTS; p++) {   // max_by_key((score, is CPU)): the last of equal keys
      if (tiers[p] >= N.MAX_TIER) continue;
      const k = [score[p], p === CPU ? 1 : 0];
      if (best == null || k[0] > best[1][0] || (k[0] === best[1][0] && k[1] >= best[1][1])) best = [p, k];
    }
    return best ? best[0] : null;
  }

  /** mirror of Rng: xorshift64 */
  const M64 = (1n << 64n) - 1n;
  class Rng {
    constructor(seed) { this.s = BigInt.asUintN(64, BigInt(seed)); }
    next() {
      let x = this.s === 0n ? 1n : this.s;
      x ^= (x << 13n) & M64; x ^= x >> 7n; x ^= (x << 17n) & M64;
      this.s = x;
      return x;
    }
    chance(per, of) { return this.next() % BigInt(of) < BigInt(per); }
    below(n) { return Number(this.next() % BigInt(Math.max(1, n))); }
  }

  /** The test knobs (coder.rs tests::knobs): his rank alone, no rig. */
  function knobs(rank, root) {
    const cps100 = t('CPS100', rank, root);
    return { cps100, typo: t('TYPO', rank, root), notice: t('NOTICE', rank, root), rank, syntax: null, langMult: true, read100: cps100, compilePct: 100 };
  }
  /** mirror of ai_knobs() */
  function aiKnobs(p, lite, f, lang, prompt, notice, rank, read100) {
    const m = N.MODELS[p][lite ? 1 : 0];
    let syntax = m.syntax;
    if (p === CLAUDE && lite && tier(f) >= 4) syntax *= 3;
    if (p === GEMINI && !lite && chars(f, lang) > 80) syntax *= 2;
    const n = Math.max(1, lines(f, lang).length);
    const avg = Math.max(1, fl(chars(f, lang) / n));
    const perChar = fl(fl((syntax + m.logic) * prompt / 100) / avg);
    return { cps100: m.cps100, typo: perChar, notice, rank, syntax: fl(syntax * 100 / Math.max(1, syntax + m.logic)), langMult: false, read100, compilePct: 100 };
  }

  // ---------------------------------------------------------------- mirror of coder.rs: the Typing machine
  /** A typo: {line, syntax (bool), bug (index), col (where on the line: the lab's own, for the terminal)}. */
  class Typing {
    constructor(f, lang, tick, ai, think) {
      this.f = f; this.lang = lang;
      this.phase = think > 0 ? { k: 'think', left: think } : { k: 'type', line: 0, done: 0 };
      this.typos = []; this.caught = 0; this.started = tick; this.ai = ai || null;
      this.shownCaught = [];     // lab only: the typos the review just caught (drawn amber while he fixes them)
    }
    static reload(f, lang, bugs, tick, ticks) {
      const ty = new Typing(f, lang, tick, null, 0);
      ty.phase = { k: 'load', left: ticks };
      ty.typos = bugs.map(b => ({ line: 0, syntax: false, bug: b, col: -1 }));
      return ty;
    }
    cursor() {
      const p = this.phase;
      if (p.k === 'type') return [p.line, fl(p.done / 100)];
      if (p.k === 'fix') return p.queue.length ? [p.queue[0], fl(p.done / 100)] : null;
      return null;
    }
    /** mirror of type_line(): a tick's worth of the line, a typo roll per character crossed; true when done. */
    typeLine(line, ph, k, rng) {
      const [len, risk] = lines(this.f, this.lang)[line];
      const before = fl(ph.done / 100);
      ph.done += fl(k.cps100 * 100 / 6000);
      const after = Math.min(fl(ph.done / 100), len);
      const L = N.LANG[this.lang];
      const langK = k.langMult ? L.typo : 100;
      const perChar = fl(fl(k.typo * langK / 100) * risk / (10 * Math.max(1, len)));
      const syntax = k.syntax == null ? L.syntax : k.syntax;
      for (let c = before; c < after; c++) {
        if (rng.chance(perChar, 10000)) {
          if (rng.chance(syntax, 100)) this.typos.push({ line, syntax: true, bug: -1, col: c });
          else {
            const w = L.bugs;
            let r = rng.below(w.reduce((a, b) => a + b, 0));
            let pick = OFF;
            for (let i = 0; i < w.length; i++) { if (r < w[i]) { pick = i; break; } r -= w[i]; }
            this.typos.push({ line, syntax: false, bug: pick, col: c });
          }
        }
      }
      return after >= len;
    }
    /** mirror of tick(): {k: 'none'} | {k: 'failed', line} | {k: 'compiled', bugs: [...]} */
    tick(k, rng) {
      const nLines = lines(this.f, this.lang).length;
      const p = this.phase;
      let out = { k: 'none' };
      const compile = fl(compileTicks(this.lang, k.rank) * k.compilePct / 100);
      switch (p.k) {
        case 'think':
          this.phase = p.left > 0 ? { k: 'think', left: p.left - 1 } : { k: 'type', line: 0, done: 0 };
          break;
        case 'type':
          if (this.typeLine(p.line, p, k, rng)) {
            this.phase = p.line + 1 < nLines ? { k: 'type', line: p.line + 1, done: 0 }
              : { k: 'review', left: fl(chars(this.f, this.lang) * 6000 / Math.max(1, k.read100 * 3)), total: fl(chars(this.f, this.lang) * 6000 / Math.max(1, k.read100 * 3)) };
          }
          break;
        case 'review':
          if (p.left > 0) { this.phase = { k: 'review', left: p.left - 1, total: p.total }; break; }
          {
            const queue = [], kept = [];
            this.shownCaught = [];
            for (const ty of this.typos) {
              if (rng.chance(k.notice, 100)) {
                this.caught++;
                this.shownCaught.push(ty);
                if (!queue.includes(ty.line)) queue.push(ty.line);
              } else kept.push(ty);
            }
            this.typos = kept.filter(ty => !queue.includes(ty.line));
            this.phase = queue.length ? { k: 'fix', queue, done: 0 } : { k: 'compile', left: compile, total: compile };
          }
          break;
        case 'fix':
          if (this.typeLine(p.queue[0], p, k, rng)) {
            const queue = p.queue.slice(1);
            this.phase = queue.length ? { k: 'fix', queue, done: 0 } : { k: 'compile', left: compile, total: compile };
          }
          break;
        case 'compile':
          if (p.left > 0) { this.phase = { k: 'compile', left: p.left - 1, total: p.total }; break; }
          {
            const c = N.LANG[this.lang].catch;
            let failed = null;
            const syn = this.typos.find(ty => ty.syntax);
            if (syn) failed = syn.line;
            if (failed == null && c > 0) {
              for (const ty of this.typos) { if (!ty.syntax && rng.chance(c, 100)) { failed = ty.line; break; } }
            }
            if (failed != null) {
              this.caught += this.typos.filter(ty => ty.line === failed).length;
              this.typos = this.typos.filter(ty => ty.line !== failed);
              out = { k: 'failed', line: failed };
              this.phase = { k: 'fix', queue: [failed], done: 0 };
            } else {
              out = { k: 'compiled', bugs: this.logicBugs() };
              this.phase = { k: 'compile', left: 0, total: p.total };
            }
          }
          break;
        case 'load':
          if (p.left > 0) { this.phase = { k: 'load', left: p.left - 1 }; break; }
          out = { k: 'compiled', bugs: this.logicBugs() };
          this.phase = { k: 'load', left: 0 };
          break;
      }
      return out;
    }
    logicBugs() { return this.typos.filter(ty => !ty.syntax).map(ty => ty.bug); }
  }

  /** mirror of tests::run(): writes f in lang start to finish: [ticks, logic bugs shipped, syntax errors hit]. */
  function run(k, f, lang, seed, think) {
    const rng = new Rng(seed);
    const ty = new Typing(f, lang, 0, think > 0 ? [0, false] : null, think);
    let fails = 0;
    for (let tick = 1; tick < 400000; tick++) {
      const ev = ty.tick(k, rng);
      if (ev.k === 'compiled') return [tick, ev.bugs.length, fails];
      if (ev.k === 'failed') fails++;
    }
    throw new Error('never compiled');
  }
  const write = (rank, root, f, lang, seed) => run(knobs(rank, root), f, lang, seed, 0);
  /** mirror of tests::mean_k(): over n seeds (0x1234 + i x 7919): [seconds, bugs, syntax errors] */
  function meanK(k, f, lang, think, n) {
    const s = [0, 0, 0];
    for (let i = 0; i < n; i++) { const r = run(k, f, lang, 0x1234 + i * 7919, think); s[0] += r[0]; s[1] += r[1]; s[2] += r[2]; }
    return [s[0] / n / 60, s[1] / n, s[2] / n];
  }

  /** The lines coder_vectors.txt holds (the native test lab_vectors writes the same). */
  const VEC_FUNCS = [F.PING, F.CHAIN, F.FIREWALL, F.ROLLBACK], VEC_SEEDS = [1, 99, 0x1234];
  const VEC_RANKS = [[0, null], [1, null], [2, null], [3, null], [4, null], [5, null], [6, null], [7, null], [7, 1]];
  function vectors() {
    const out = [];
    for (const [r, p] of VEC_RANKS) for (const f of VEC_FUNCS) for (let l = 0; l < CODE.LANGS.length; l++) {
      if (!avail(f, l)) continue;
      for (const s of VEC_SEEDS) out.push(`write ${r} ${p == null ? '-' : p} ${f} ${l} ${s} ${write(r, p, f, l, s).join(' ')}`);
    }
    for (let p = 0; p < 3; p++) for (const lite of [false, true]) for (const s of VEC_SEEDS) {
      const k = aiKnobs(p, lite, F.CHAIN, CPP, N.PROMPT[3], N.NOTICE[3], 3, N.CPS100[3]);
      out.push(`ai ${p} ${lite ? 1 : 0} ${F.CHAIN} ${CPP} ${s} ${run(k, F.CHAIN, CPP, s, N.MODELS[p][lite ? 1 : 0].think).join(' ')}`);
    }
    return out;
  }
  /** A few of the vectors, checked in the browser (verify_coder.py checks them all). */
  function selfTest() {
    const ok = [];
    ok.push(tv(N.CPS100, N.CPS100_TOP, 3, null) === 1350 && tv(N.CPS100, N.CPS100_TOP, ROOT, 10) === 3300 && tv(N.CPS100, N.CPS100_TOP, ROOT, 1) === 4500);
    ok.push(tv(N.PROMPT, N.PROMPT_TOP, ROOT, 1) === 50);
    ok.push(JSON.stringify(write(2, null, F.FIREWALL, RUST, 99)) === JSON.stringify(write(2, null, F.FIREWALL, RUST, 99)));
    ok.push(rigTier(4, null) === 1 && rigTier(ROOT, 1) === 5 && rigTier(ROOT, 3) === 4);
    return ok.every(Boolean);
  }

  // ---------------------------------------------------------------- the arena: a reduced model of the native brain
  /** 30 s on a lane: the Coder at 0, his ally at 30000 pressed by enemy 1, enemy 2 diving the Coder at 10 s. */
  function simulateSkirmish(rank, root, seed, opts = {}) {
    const TICKS = opts.ticks || 1800;
    const rng = new Rng(BigInt(seed) * 0x9E3779B97F4A7C15n ^ 0xC0DEn);
    const T_ = name => t(name, rank, root);
    const AP = 120;
    const U = [
      { id: 0, team: 0, name: 'Coder', hp: 1500, max: 1500, x: 0, home: 0, dead: 0 },
      { id: 1, team: 0, name: 'Ally', hp: 2000, max: 2000, x: 30000, home: 30000, dead: 0, stun: 0 },
      { id: 2, team: 1, name: 'Bruiser', hp: 1800, max: 1800, x: 48000, home: 48000, dead: 0 },
      { id: 3, team: 1, name: 'Diver', hp: 1600, max: 1600, x: 110000, home: 110000, dead: 0 },
    ];
    for (const u of U) Object.assign(u, { shield: 0, shieldUntil: 0, encrypt: 0, slow: 0, slowFlip: false, ddos: 0, ddosFlip: false,
      boost: 0, boostFlip: false, stunUntil: 0, marked: 0, hist: [] });
    const me = U[0];
    const S = { shipped: 0, clean: 0, bugs: 0, caught: 0, syntax: 0, writeTicks: 0, frozen: 0, damage: 0, saves: 0, kills: 0,
      deaths: 0, allyDeaths: 0, bsods: 0, ooms: 0, runs: 0, prompts: [0, 0, 0], lite: 0, btc: 0, written: [] };
    const st = { typing: null, program: [], storage: [], cooldown: new Array(NF).fill(0), nextCheck: 0, nextChoice: 0, nextDebug: 600,
      debugUntil: 0, heat: 4000, load: 0, oc: false, ocOffAt: null, frozenUntil: 0, procs: [], leak: 0, walls: [], hits: [], drones: [],
      scanUntil: 0, deployUntil: 0, tiers: startTiers(rank), mining: false, lastHp: me.hp,
      // round 108: scripts and daemons, the no-spam memory, what the meta functions repeat
      lastRun: new Array(NF).fill(0), lastWrite: [], pending: null, lastScript: null, lastEffect: null, combo: 0,
      ai: { pools: N.POOL.slice(), provider: 0, lite: false, until: 0, next: 1200, hold: 0, queue: [] } };
    const timeline = new Uint8Array(TICKS);   // 0 idle, 1 typing, 2 AI, 3 review/compile, 4 frozen, 5 dead, 6 blue screen
    const runs = [];
    const alive = u => u.dead === 0;
    const dist = (a, b) => Math.abs(a.x - b.x);
    const ghz = () => { const base = N.GHZ[st.tiers[CPU]] + (st.oc ? 90 : 0); const h = fl(st.heat / 100);
      return h <= 85 ? base : fl(base * (100 - Math.min(70, fl((h - 85) * 100 / 30))) / 100); };
    const myKnobs = () => {
      let cps = fl(T_('CPS100') * ghz() / 300);
      if (st.oc) cps = fl(cps * 120 / 100);
      if (st.mining) cps = fl(cps * 80 / 100);
      let typo = T_('TYPO');
      if (st.oc) typo = fl(typo * 115 / 100);
      if (st.heat >= 8500) typo = fl(typo * 125 / 100);
      return { cps100: cps, typo, notice: T_('NOTICE'), rank, syntax: null, langMult: true, read100: cps, compilePct: N.COMPILE_PCT[st.tiers[SSD]] };
    };
    const aiOn = tick => tick < st.ai.until;
    const knobsFor = (ty, tick) => {
      const mine = myKnobs();
      if (ty.ai && aiOn(tick)) { const k = aiKnobs(ty.ai[0], ty.ai[1], ty.f, ty.lang, T_('PROMPT'), mine.notice, rank, mine.cps100); k.compilePct = mine.compilePct; return k; }
      return mine;
    };
    const ramUsed = () => st.procs.reduce((s, p) => s + p.mb, 0) + st.leak;
    const ramCap = () => N.RAM_MB[st.tiers[RAM]];
    const freeze = (tick, ticks) => { st.frozenUntil = Math.max(st.frozenUntil, tick + ticks); S.frozen += ticks; };
    const readHp = (u, tick) => {
      const real = fl(u.hp * 100 / u.max);
      if (tick < st.scanUntil || (aiOn(tick) && st.ai.provider === GEMINI && !st.ai.lite)) return real;
      const n = T_('READ');
      if (n === 0) return real;
      const h = Number(((BigInt(u.id) ^ BigInt(fl(tick / 60)) ^ BigInt(seed)) * 0x9E3779B97F4A7C15n & M64) >> 40n);
      return real + (h % (2 * n + 1)) - n;
    };
    const foes = () => U.filter(u => u.team === 1 && alive(u));
    const value = f => {
      const nFoes = foes().filter(u => dist(u, me) <= 90000).length;
      const hurt = U.filter(u => u.team === 0 && alive(u) && u.hp * 100 < u.max * 70).length;
      const mates = U.filter(u => u.team === 0 && u.id !== 0 && alive(u) && dist(u, me) <= 60000).length;
      const pressedN = U.filter(u => u.team === 0 && alive(u) && u.hp * 100 >= u.max * 70 && foes().some(e => dist(e, u) <= 30000)).length;
      const base = N.SPEC[f][3];
      if ([F.SHIELD, F.HEAL, F.ENCRYPT, F.CLEANSE, F.ROLLBACK, F.SWAP].includes(f)) return base + 12 * Math.min(2, hurt) + fl(6 * Math.min(2, pressedN) * T_('IQ') / 100);
      if ([F.CHAIN, F.SPRAY, F.FIREWALL, F.DDOS_ALL, F.SORT, F.GC].includes(f)) return base + 8 * Math.min(3, nFoes);
      if (f === F.BOOST) return base + 6 * Math.min(3, mates);
      if (f === F.BLINK) return base + (me.hp * 2 < me.max ? 20 : 0);
      if (f === F.SCAN) return base + (T_('READ') > 5 ? 15 : 0);
      if (f === F.CACHE) return base + (st.load > 5000 ? 15 : 0);
      if (f === F.DEPLOY) return base + 5 * st.program.length;
      const k = klass(f);
      if (f >= 24 && (k === 2 || k === 3)) return base + 12 * Math.min(2, hurt) + fl(6 * Math.min(2, pressedN) * T_('IQ') / 100);
      if (f >= 24 && k === 1) return base + 8 * Math.min(3, nFoes);
      if (f >= 24 && k === 5) return base + 6 * Math.min(3, mates);
      if (passive(f)) return base + 2 * st.program.length;
      return base;
    };
    const anyLang = f => { const ls = CODE.LANGS.map((_, l) => l).filter(l => avail(f, l)); return ls.length ? ls[rng.below(ls.length)] : N.IDEAL[f]; };
    const fightingNow = () => foes().some(u => dist(u, me) <= 70000);
    // mirror of believed() / candidates()
    const believed = (f, lang, aware) => {
      const k = myKnobs();
      const n = chars(f, lang);
      const risk = lines(f, lang).reduce((s, l) => s + l[1], 0);
      const secs = fl(n * 100 / Math.max(1, k.cps100)) + fl(compileTicks(lang, k.rank) / 60);
      const L = N.LANG[lang];
      const typos100 = fl(fl(fl(fl(k.typo * L.typo / 100) * risk / 10) / 100));
      const slip100 = fl(fl(fl(typos100 * (100 - k.notice) / 100) * (100 - fl(L.catch * (100 - L.syntax) / 100)) / 100));
      const clean = Math.max(0, 100 - Math.min(100, slip100));
      return [fl(secs * (40 + fl(60 * aware / 100)) / 100), fl((clean * aware + 100 * (100 - aware)) / 100)];
    };
    let now = 0;
    const candidates = sharp => {
      const iq = sharp ? 100 : T_('IQ'), aware = sharp ? 100 : T_('AWARE');
      const weakest = st.program.length ? Math.min(...st.program.map(c => value(c.f))) : 0;
      const repeat = 90 - 5 * Math.min(rank, ROOT);
      const out = [];
      for (let f = 0; f < NF; f++) {
        if (st.program.some(c => c.f === f)) continue;
        if (st.typing && st.typing.f === f) continue;
        if (isScript(f) && (now < st.cooldown[f] || (st.pending && st.pending.f === f))) continue;
        const lang = rng.chance(iq, 100) ? ideal(f, rank) : anyLang(f);
        const v = value(f);
        if (!isScript(f) && st.program.length >= slots(rank, root) && v * 10 < weakest * 13) continue;
        const insurance = [F.SHIELD, F.HEAL, F.ENCRYPT, F.CLEANSE, F.ROLLBACK, F.SWAP].includes(f) && U.some(u => u.team === 0 && alive(u) && foes().some(e => dist(e, u) <= 30000));
        const ready = insurance || (passive(f) && fightingNow()) || trigger(f, now, false, false) !== undefined;
        const va = ready ? v : fl(v * (100 - fl(iq / 2)) / 100);
        const [secs, clean] = believed(f, lang, aware);
        const recent = st.lastWrite.filter(w => w[0] === f).length;
        let vr = 115;
        if (recent) { vr = 100; for (let i = 0; i < Math.min(4, recent); i++) vr = fl(vr * repeat / 100); }
        out.push([fl(fl(fl(va * clean * vr / 100) * 100) / (100 + secs * 8)), f, lang]);
      }
      out.sort((a, b) => b[0] - a[0]);
      return out;
    };
    const flagshipOk = p => st.ai.pools[p] * 100 >= N.POOL[p] * 60;
    const ship = (f, lang, bugs, reloaded, tick, ty) => {
      S.shipped++; if (!bugs.length) S.clean++; S.bugs += bugs.length;
      S.caught += ty.caught; S.writeTicks += tick - ty.started;
      S.written.push({ f, lang, bugs: bugs.slice(), ai: ty.ai, tick });
      st.lastWrite.push([f, tick]);
      st.lastWrite = st.lastWrite.filter(w => w[1] + 2700 > tick);
      const save = reloaded || rng.chance(T_('IQ'), 100);
      const c = { f, lang, bugs, saved: save };
      if (save && !reloaded) {
        st.storage = st.storage.filter(s => s.f !== f);
        if (st.storage.length >= N.STORAGE[st.tiers[DISK]]) st.storage.shift();
        st.storage.push(c);
      }
      // round 107: a script runs at once (no slot) or waits 2 s for its trigger; a daemon joins the program
      if (isScript(f)) { if (!runOne(c, tick)) st.pending = { ...c, until: tick + 120 }; return; }
      if (st.program.length >= slots(rank, root)) { let wi = 0; st.program.forEach((c, i) => { if (value(c.f) < value(st.program[wi].f)) wi = i; }); st.program.splice(wi, 1); }
      st.program.push(c);
      st.program.sort((a, b) => value(b.f) - value(a.f));
    };
    const onDead = () => {
      st.typing = null; st.procs = []; st.walls = []; st.hits = []; st.drones = [];
      st.heat = 4000; st.oc = false; st.ocOffAt = null; st.mining = false;
    };
    const bsod = tick => {
      S.bsods++; st.heat = 7000; st.oc = false; st.ocOffAt = null; st.mining = false; st.typing = null;
      st.program = st.program.filter(c => c.saved); st.procs = []; st.leak = 0;
      freeze(tick, 150); st.bsodUntil = tick + 150;
    };
    const oom = tick => {
      S.ooms++;
      const p = st.procs.pop();
      if (p) {
        if (p.f === F.SHIELD) U[p.target].shield = 0;
        if (p.f === F.ENCRYPT) U[p.target].encrypt = 0;
        if (p.f === F.SLOW) U[p.target].slow = 0;
        if (p.f === F.FIREWALL) st.walls.pop();
        if (p.f === F.FORK) st.drones.pop();
        if (p.f === F.SCAN) st.scanUntil = 0;
        if (p.f === F.DEPLOY) st.deployUntil = 0;
      }
      if (ramUsed() > ramCap()) st.leak = 0;
      freeze(tick, 30);
    };
    // damage and healing
    const damage = (u, n, tick) => {
      if (!alive(u) || n <= 0) return 0;
      let d = n;
      if (u.team === 1 && tick < u.marked) d = fl(d * 120 / 100);
      if (tick < u.encryptUntil) d = u.encryptFlip ? fl(d * 130 / 100) : fl(d / 2);
      if (u.encryptUntil > tick && !u.encryptFlip) S.saves += n - d;
      if (tick < u.shieldUntil && u.shield > 0) { const a = Math.min(u.shield, d); u.shield -= a; d -= a; if (u.team === 0) S.saves += a; }
      u.hp -= d;
      if (u.hp <= 0) {
        u.hp = 0; u.dead = u.team === 1 ? 180 : 300;
        if (u.team === 1) S.kills++; else if (u.id === 0) { S.deaths++; onDead(); } else S.allyDeaths++;
      }
      return d;
    };
    const heal = (u, n) => { if (!alive(u)) return 0; const h = Math.min(n, u.max - u.hp); u.hp += h; return h; };
    const hit = (u, n, flip, tick) => { if (flip) heal(u, n); else S.damage += damage(u, n, tick); };
    const near = (r, k, wrong) => { const v = foes().filter(u => dist(u, me) <= fl(r * k / 100)).sort((a, b) => dist(a, me) - dist(b, me)); if (wrong) v.reverse(); return v; };
    const mates = k => U.filter(u => u.team === 0 && alive(u) && dist(u, me) <= fl(60000 * k / 100));
    const pressed = u => foes().some(e => dist(e, u) <= 60000);
    const minBy = (arr, key) => arr.reduce((b, u) => (b == null || key(u) < key(b) ? u : b), null);
    const trigger = (f, tick, off, wrong) => {
      const k = off ? 80 : 100, shift = off ? 20 : 0;
      switch (f) {
        case F.PING: case F.DDOS: case F.RECURSE: return (near(70000, k, wrong)[0] || {}).id;
        case F.SHIELD: return (minBy(mates(k).filter(c => readHp(c, tick) < 70 + shift && pressed(c)), c => readHp(c, tick)) || {}).id;
        case F.HEAL: return (minBy(mates(k).filter(c => readHp(c, tick) < 55 + shift), c => readHp(c, tick)) || {}).id;
        case F.SCAN: return tick >= st.scanUntil && near(90000, k, wrong).length ? 0 : undefined;
        case F.SPRAY: return near(25000, k, wrong).length >= 2 ? 0 : undefined;
        case F.BLINK: return readHp(me, tick) < 40 + shift ? (near(30000, k, wrong)[0] || {}).id : undefined;
        case F.SLOW: return (near(45000, k, wrong)[0] || {}).id;
        case F.FIREWALL: case F.INJECT: return (near(55000, k, wrong)[0] || {}).id;
        case F.CACHE: return st.load > 6000 && !near(60000, k, wrong).length ? 0 : undefined;
        case F.CHAIN: { const v = near(50000, k, wrong); return v.length >= 2 ? v[0].id : undefined; }
        case F.CLEANSE: return (mates(k).find(c => tick < c.stunUntil) || {}).id;
        case F.BOOST: return mates(k).length >= 2 && near(70000, k, wrong).length ? 0 : undefined;
        case F.FORK: return near(80000, k, wrong).length ? 0 : undefined;
        case F.SWAP: return (mates(k).filter(c => c.id !== 0 && readHp(c, tick) < 40 + shift).find(c => foes().some(e => dist(e, c) <= 20000)) || {}).id;
        case F.SORT: return near(60000, k, wrong).length >= 3 ? 0 : undefined;
        case F.ENCRYPT: return (minBy(mates(k).filter(c => readHp(c, tick) < 50 + shift && pressed(c)), c => readHp(c, tick)) || {}).id;
        case F.DDOS_ALL: return near(80000, k, wrong).length >= 2 ? 0 : undefined;
        case F.KILL9: return (near(70000, k, wrong).find(c => readHp(c, tick) < 15 + shift) || {}).id;
        case F.ROLLBACK: {
          const burst = mates(k).find(c => c.hist.length && c.hist[0].hp > c.hp + fl(c.max / 3));
          const diver = near(70000, k, wrong).find(c => c.hist.length && Math.abs(c.hist[0].x - c.x) > 40000);
          return (burst || diver || {}).id;
        }
        case F.GC: return near(70000, k, wrong).some(c => readHp(c, tick) < 30 + shift) ? 0 : undefined;
        case F.DEPLOY: return st.program.length >= 4 && near(80000, k, wrong).length ? 0 : undefined;
      }
      return f >= 24 ? triggerNew(f, tick, k, shift, wrong) : undefined;
    };
    /** round 108: the new functions' triggers on the lane (coder.rs trigger_new, reduced: no buffs to copy here) */
    const triggerNew = (f, tick, k, shift, wrong) => {
      const [tr, r0, , a] = N.NB[f - 24];
      const r = fl(r0 * k / 100);
      const fighting = fightingNow();
      const on = (c, rr) => foes().filter(e => dist(e, c) <= rr).length;
      const all = mates(k).filter(c => c.id !== 0);
      const ago = (u, t) => u.hist.length ? u.hist[Math.max(0, u.hist.length - 1 - fl(t / 6))] : null;
      switch (tr) {
        case 0: case 21: case 25: return (near(r, 100, wrong)[0] || {}).id;
        case 1: return near(r, 100, false).length >= 2 ? 0 : undefined;
        case 2: return (minBy(all.filter(c => readHp(c, tick) < 55 + shift), c => readHp(c, tick)) || {}).id;
        case 3: return (minBy(all.filter(c => readHp(c, tick) >= 55 && on(c, 30000) >= 1), c => readHp(c, tick)) || {}).id;
        case 4: return readHp(me, tick) < 40 + shift && fighting ? 0 : undefined;
        case 5: return fighting ? 0 : undefined;
        case 6: return (near(r, 100, wrong).find(c => c.hist.length && Math.abs(c.hist[0].x - c.x) > 40000) || {}).id;
        case 7: return all.length && foes().some(u => dist(u, me) <= 120000) ? 0 : undefined;
        case 9: return (minBy(all.filter(c => readHp(c, tick) < 40 + shift && on(c, 30000) >= 1), c => readHp(c, tick)) || {}).id;
        case 10: { const hurt = [me, ...all].find(c => { const h = ago(c, 60); return h && h.hp > c.hp + fl(c.max * 8 / 100); });
          return hurt ? (minBy(foes().filter(e => dist(e, hurt) <= 40000), e => dist(e, hurt)) || {}).id : undefined; }
        case 11: return ([me, ...all].find(c => on(c, 30000) >= 2) || {}).id;
        case 12: return fighting && st.lastScript ? 0 : undefined;
        case 13: return fighting && (f === fIndex('llm_agent') || st.lastEffect) ? 0 : undefined;
        case 15: return readHp(me, tick) < 15 + shift ? 0 : undefined;
        case 16: { const h = me.hist[0]; return h && h.hp > me.hp + fl(me.max / 5) ? 0 : undefined; }
        case 17: return (near(r, 100, wrong).find(c => readHp(c, tick) < a + shift) || {}).id;
        case 19: return fighting && st.program.some(c => c.bugs.length) ? 0 : undefined;
        case 20: return fighting && st.program.some(c => !passive(c.f) && tick < st.cooldown[c.f]) ? 0 : undefined;
        case 22: case 24: return fighting && !st.combo ? 0 : undefined;
      }
      return undefined;   // 8 passive; 14, 18, 23: not modelled on the lane
    };
    const execute = (c, target, tick, mb) => {
      if (c.bugs.includes(LOOP) && rng.chance(60, 100)) { freeze(tick, c.f === F.RECURSE ? 150 : 90); return 'loop'; }
      if (c.bugs.includes(SEGV) && rng.chance(50, 100)) { freeze(tick, 60); return 'segfault'; }
      if (c.bugs.includes(NULLREF) && rng.chance(50, 100)) return 'null';
      const flip = c.bugs.includes(FLIP), wrong = c.bugs.includes(WRONG);
      const deploy = tick < st.deployUntil ? 150 : 100;
      const power = fl(fl(N.LANG[c.lang].power * ghz() / 300) * deploy / 100);
      const amt = (base, ratio) => fl((base + fl(AP * ratio / 100)) * power / 100);
      const T = U[target];
      if (mb > 0) st.procs.push({ until: tick + N.SPEC[c.f][4], mb, f: c.f, target });
      switch (c.f) {
        case F.PING: hit(T, amt(35, 50), flip, tick); break;
        case F.SHIELD: case F.HEAL: case F.ENCRYPT: {
          const tt = wrong ? (minBy(foes(), u => dist(u, me)) || T) : T;
          if (c.f === F.SHIELD) { tt.shield = amt(120, 50); tt.shieldUntil = tick + 180; }
          else if (c.f === F.ENCRYPT) { tt.encryptUntil = tick + 180; tt.encryptFlip = flip; }
          else if (flip) { tt.hp = Math.max(1, tt.hp - amt(40, 20)); }
          else { const h = heal(tt, amt(80, 40)); if (tt.team === 0) S.saves += h; }
          break;
        }
        case F.SCAN: st.scanUntil = tick + 180; break;
        case F.SPRAY: for (const e of near(25000, 100, false)) hit(e, amt(25, 30), flip, tick); break;
        case F.BLINK: me.x += wrong ? 30000 * Math.sign(T.x - me.x || 1) : -30000 * Math.sign(T.x - me.x || 1); break;
        case F.SLOW: T.slow = tick + 120; T.slowFlip = flip; break;
        case F.CACHE: st.load = Math.max(0, st.load - 4000); freeze(tick, 30); break;
        case F.CHAIN: {
          let cur = T; const hits = [];
          for (let i = 0; i < 4 && cur; i++) {
            hit(cur, amt(30, 35), flip, tick); hits.push(cur.id);
            cur = minBy(foes().filter(u => !hits.includes(u.id) && dist(u, cur) <= 35000), u => dist(u, cur));
          }
          break;
        }
        case F.FIREWALL: st.walls.push({ x: me.x + fl((T.x - me.x) * 6 / 10), until: tick + 240, next: tick, dmg: amt(15, 15), flip }); break;
        case F.DDOS: case F.DDOS_ALL: {
          const ts = c.f === F.DDOS ? [T] : foes().filter(u => dist(u, me) <= 80000);
          const [n, gap, dmg] = c.f === F.DDOS ? [20, 3, amt(4, 6)] : [8, 3, amt(3, 4)];
          for (const u of ts) { for (let i = 0; i < n; i++) st.hits.push({ at: tick + i * gap, target: u.id, dmg, flip, nearest: false }); u.ddos = tick + 150; u.ddosFlip = flip; u.ddosPct = c.f === F.DDOS ? 30 : 20; }
          break;
        }
        case F.CLEANSE: T.stunUntil = 0; T.shield = amt(40, 20); T.shieldUntil = tick + 120; break;
        case F.BOOST: for (const u of U.filter(u => (u.team === 0) !== wrong && alive(u) && dist(u, me) <= 60000)) { u.boost = tick + 180; u.boostFlip = flip; } break;
        case F.FORK: { const n = ramUsed() + 2 * mb <= ramCap() ? 2 : 1;
          for (let i = 0; i < n; i++) { st.drones.push([tick + N.SPEC[F.FORK][4], tick + 15 * i]); if (i === 1) st.procs.push({ until: tick + N.SPEC[F.FORK][4], mb, f: F.FORK, target }); }
          break; }
        case F.SWAP: {   // the diver on the ally is sent back where it came from (a lane can't trade places)
          const d = minBy(foes(), u => dist(u, T));
          if (!d) return 'none';
          if (wrong) me.x = T.x; else d.x += 40000 * Math.sign(d.x - T.x || 1);
          break;
        }
        case F.SORT: break;
        case F.KILL9: if (T.hp * 100 < T.max * 15 && !flip) S.damage += damage(T, T.hp + T.max, tick); else hit(T, amt(60, 40), flip, tick); break;
        case F.ROLLBACK: {
          const s = T.hist[0];
          if (!s) return 'none';
          if (T.team === 0 && !flip) { const h = heal(T, Math.max(0, s.hp - T.hp)); S.saves += h; } else T.x = s.x;
          break;
        }
        case F.RECURSE: for (let i = 0; i < 8; i++) st.hits.push({ at: tick + 6 * i, target, dmg: amt(12, 15), flip, nearest: true }); break;
        case F.INJECT: T.stunUntil = tick + (flip ? 1 : 60); hit(T, amt(40, 40), flip, tick); break;
        case F.GC: for (const e of near(70000, 100, false).filter(u => u.hp * 100 < u.max * 30)) { if (!flip) e.shield = 0; hit(e, amt(50, 40), flip, tick); } break;
        case F.DEPLOY: st.deployUntil = tick + N.SPEC[F.DEPLOY][4]; st.load = 0; break;
        default: return executeNew(c, T, tick, flip, amt);
      }
      return 'ok';
    };
    /** round 108: the new functions by class (coder.rs execute_new, reduced to what the lane can show) */
    const executeNew = (c, T, tick, flip, amt) => {
      const [, r, k, a, b] = N.NB[c.f - 24];
      const meta = META.includes(c.f);
      if (meta) {
        const p = st.lastEffect;
        if (!p || META.includes(p.f)) return 'none';
        const t2 = trigger(p.f, tick, false, false);
        if (t2 === undefined) return 'none';
        execute(p, t2, tick, 0);
        return 'ok';
      }
      switch (k) {
        case 0: hit(T, amt(Math.max(30, Math.min(150, a)), 50), flip, tick); break;
        case 1: for (const e of near(Math.max(60000, r), 100, false)) hit(e, amt(25, 30), flip, tick); break;
        case 2: { const u = T.team === 0 ? T : me; const h = heal(u, amt(60, 30)); S.saves += h; break; }
        case 3: { const u = T.team === 0 ? T : me; u.shield = amt(100, 40); u.shieldUntil = tick + 180; break; }
        case 4: if (T.team === 1) { T.stunUntil = tick + (flip ? 1 : 60); T.marked = tick + 180; } else for (const e of near(80000, 100, false).slice(0, 2)) e.stunUntil = tick + 60; break;
        case 5: for (const u of U.filter(u => u.team === 0 && alive(u) && dist(u, me) <= 60000)) { u.boost = tick + 180; u.boostFlip = flip; } break;
        case 7: { const u = T.team === 0 ? T : me; u.shield = Math.max(u.shield, 400); u.shieldUntil = tick + 90; break; }
        default: return 'none';
      }
      return 'ok';
    };
    /** mirror of run_one(): try to run a compiled function now */
    const runOne = (c, tick) => {
      if (tick < st.cooldown[c.f]) return false;
      const target = trigger(c.f, tick, c.bugs.includes(OFF), c.bugs.includes(WRONG));
      if (target === undefined) return false;
      const cost = cloudCpu(c.f) ? 0 : fl(fl(N.SPEC[c.f][1] * (c.lang === GO ? 70 : 100) / 100 / (tick < st.deployUntil ? 2 : 1)) * N.CPU_LOAD[st.tiers[CPU]] / 100);
      if (st.load + cost > 10000) return false;
      const mb = fl(N.SPEC[c.f][2] * N.LANG[c.lang].ram / 100);
      st.load += cost;
      st.heat += N.LANG[c.lang].heat * (st.oc ? 2 : 1) + (GPU_FX.includes(c.f) ? N.GPU_HEAT[st.tiers[GPU]] : 0);
      st.cooldown[c.f] = tick + fl(N.SPEC[c.f][0] * 80 / 100);
      st.lastRun[c.f] = tick;
      S.runs++;
      const res = execute(c, target, tick, mb);
      runs.push({ tick, f: c.f, res });
      if (res === 'ok' && !META.includes(c.f)) { if (isScript(c.f)) st.lastScript = c; st.lastEffect = c; }
      return true;
    };
    const cloudCpu = f => f === fIndex('cloud_deploy') || f === fIndex('serverless');
    const runProgram = tick => {
      if (st.pending) { if (tick >= st.pending.until || runOne(st.pending, tick)) st.pending = null; }
      if (tick < st.nextCheck) return;
      st.nextCheck = tick + Math.max(4, fl(T_('CLOCK') * 300 / Math.max(1, ghz())));
      // round 107: the daemon that ran least recently (value breaks ties among those last run within the same 2 s)
      let best = null;
      st.program.forEach((c, i) => {
        if (passive(c.f) || tick < st.cooldown[c.f]) return;
        if (trigger(c.f, tick, c.bugs.includes(OFF), c.bugs.includes(WRONG)) === undefined) return;
        const mb = fl(N.SPEC[c.f][2] * N.LANG[c.lang].ram / 100);
        if (mb > 0 && ramUsed() + mb > ramCap() && rng.chance(T_('IQ'), 100)) return;
        const heat = N.LANG[c.lang].heat * (st.oc ? 2 : 1);
        if (st.heat + heat >= N.HOT_SKIP && rng.chance(T_('IQ'), 100)) return;
        const key = [fl(st.lastRun[c.f] / 120), -value(c.f), i];
        if (!best || key[0] < best[0] || (key[0] === best[0] && (key[1] < best[1] || (key[1] === best[1] && key[2] < best[2])))) best = key;
      });
      if (best) runOne(st.program[best[2]], tick);
    };
    const activateAi = tick => {
      if (aiOn(tick) || tick < st.ai.next || tick % 30 !== 0) return;
      if (!foes().some(u => dist(u, me) <= 120000)) return;
      let p;
      const smart = rng.chance(T_('IQ'), 100);
      if (smart) {
        const c = candidates(false)[0];
        const nextTier = c ? tier(c[1]) : 1;
        const want = st.program.length <= 2 ? GEMINI : nextTier >= 4 ? CLAUDE : GPT;
        p = flagshipOk(want) ? want : ([0, 1, 2].find(flagshipOk) ?? want);
      } else p = rng.below(3);
      if (smart && st.typing && !st.typing.ai && st.typing.phase.k !== 'load') st.typing = null;   // hands it to the AI
      Object.assign(st.ai, { provider: p, until: tick + N.AI_TICKS, next: tick + N.AI_COOLDOWN, hold: 0, queue: [] });
      st.nextChoice = tick;
    };
    const promptAi = tick => {
      const p = st.ai.provider;
      if (!flagshipOk(p) && rng.chance(T_('IQ'), 100)) {
        const q = [0, 1, 2].filter(q => q !== p && flagshipOk(q)).sort((a, b) => fl(st.ai.pools[a] * 100 / N.POOL[a]) - fl(st.ai.pools[b] * 100 / N.POOL[b])).pop();
        if (q != null) { st.ai.provider = q; st.ai.hold = tick + 60; return; }
      }
      const lite = !flagshipOk(p);
      const picks = candidates(p === GPT && !lite);
      const n = N.MODELS[p][lite ? 1 : 0].per_prompt;
      const chosen = picks.slice(0, n).map(c => [c[1], c[2]]);
      if (!chosen.length) return;
      const cost = chosen.reduce((s, [f, l]) => s + chars(f, l) * 50, 0);
      if (st.ai.pools[p] < cost) { st.ai.hold = tick + 120; return; }
      st.ai.pools[p] -= cost; st.ai.lite = lite; S.prompts[p]++; if (lite) S.lite++;
      st.typing = new Typing(chosen[0][0], chosen[0][1], tick, [p, lite], N.MODELS[p][lite ? 1 : 0].think);
      st.ai.queue = chosen.slice(1);
    };
    const choose = tick => {
      if (st.typing || tick < st.nextChoice || tick < st.debugUntil) return;
      st.nextChoice = tick + 30;
      if (aiOn(tick)) {
        const q = st.ai.queue.shift();
        if (q && !st.program.some(c => c.f === q[0])) { st.typing = new Typing(q[0], q[1], tick, [st.ai.provider, st.ai.lite], 0); return; }
        if (tick < st.ai.hold) return;
        promptAi(tick);
        return;
      }
      const c = candidates(false)[0];
      if (!c) return;
      const saved = st.storage.find(s => s.f === c[1]);
      st.typing = saved ? Typing.reload(c[1], saved.lang, saved.bugs.slice(), tick, isScript(c[1]) ? 0 : N.RELOAD[st.tiers[SSD]]) : new Typing(c[1], c[2], tick, null, 0);
    };
    const stepTyping = tick => {
      const ty = st.typing;
      if (!ty) return;
      const k = knobsFor(ty, tick);
      const reloaded = ty.phase.k === 'load';
      const ev = ty.tick(k, rng);
      if (ty.cursor() && !ty.ai) st.heat += st.oc ? 4 : 2;
      if (ev.k === 'failed') S.syntax++;
      if (ev.k === 'compiled') { st.typing = null; st.nextChoice = tick; ship(ty.f, ty.lang, ev.bugs, reloaded, tick, ty); }
    };

    for (let tick = 0; tick < TICKS; tick++) {
      now = tick;
      // the world: respawns, the enemies close in and hit, the ally hits back
      for (const u of U) {
        if (u.dead > 0) { u.dead--; if (u.dead === 0) { u.hp = u.max; u.x = u.home; if (u.id === 0) st.lastHp = u.hp; } continue; }
        if (tick % 6 === 0) { u.hist.push({ x: u.x, hp: u.hp }); if (u.hist.length > 31) u.hist.shift(); }
      }
      const ally = U[1], b = U[2], d = U[3];
      const move = (u, goal) => { const sp = tick < u.slow ? (u.slowFlip ? 312 : 150) : 250; if (Math.abs(goal - u.x) > sp) u.x += Math.sign(goal - u.x) * sp; else u.x = goal; };
      const prevX = { 2: b.x, 3: d.x };
      if (alive(b)) move(b, ally.x + 18000);
      if (alive(d) && tick >= 600) move(d, me.x + 20000);
      const atk = (u, target, dmg, stunEvery) => {
        if (!alive(u) || !alive(target) || tick < u.stunUntil || dist(u, target) > 25000) return;
        const per = u.ddos > tick ? (u.ddosFlip ? 100 + u.ddosPct : 100 - u.ddosPct) : 100;
        if (tick % 30 === 0) damage(target, fl(dmg * per / 100), tick);
        if (stunEvery && tick % stunEvery === 0 && tick > 0) target.stunUntil = tick + 60;
      };
      atk(b, ally, 45, 480);
      if (tick >= 600) atk(d, me, 60, 0);
      if (alive(ally) && tick >= ally.stunUntil && alive(b) && dist(ally, b) <= 25000 && tick % 30 === 0) {
        damage(b, ally.boost > tick ? (ally.boostFlip ? 28 : 52) : 40, tick);
      }
      if (!alive(me)) { timeline[tick] = 5; continue; }
      // his rig
      if (st.heat > 4000) st.heat -= N.COOLING[st.tiers[COOL]];
      if (st.oc) st.heat += 14;
      if (st.mining) st.heat += 4;
      st.load = Math.max(0, st.load - 50 + (st.mining ? 20 : 0));
      if (tick % 60 === 0) { if (st.tiers[RAM] < 3) st.leak += 400 * st.program.filter(c => c.bugs.includes(LEAK)).length; S.btc += 75 + (st.mining ? 300 : 0); }
      st.procs = st.procs.filter(p => p.until > tick);
      if (ramUsed() > ramCap()) oom(tick);
      if (st.heat >= 10000) bsod(tick);
      if (tick % 60 === 0) for (let p = 0; p < 3; p++) st.ai.pools[p] = Math.min(N.POOL[p], st.ai.pools[p] + N.REFILL[p]);
      if (!aiOn(tick)) st.ai.queue = [];
      // hits landing later, the drones, the walls
      for (const h of st.hits.filter(h => h.at <= tick)) {
        const u = h.nearest ? near(70000, 100, false)[0] : (alive(U[h.target]) ? U[h.target] : null);
        if (u) hit(u, h.dmg, h.flip, tick);
      }
      st.hits = st.hits.filter(h => h.at > tick);
      st.drones = st.drones.filter(dr => dr[0] > tick);
      for (const dr of st.drones) {
        if (tick < dr[1]) continue;
        dr[1] = tick + 30;
        const u = near(70000, 100, false)[0];
        if (u) S.damage += damage(u, fl((15 + fl(AP / 5)) * ghz() / 300), tick);
      }
      st.walls = st.walls.filter(w => w.until > tick);
      for (const w of st.walls) {
        for (const e of foes()) {
          const crossed = (prevX[e.id] - w.x) * (e.x - w.x) < 0;
          if (crossed || (tick >= w.next && Math.abs(e.x - w.x) <= 6000)) hit(e, w.dmg, w.flip, tick);
        }
        if (tick >= w.next) w.next = tick + 20;
      }
      const frozen = tick < st.frozenUntil;
      timeline[tick] = tick < (st.bsodUntil || 0) ? 6 : frozen ? 4 : st.typing ? (st.typing.ai ? 2 : ['review', 'compile', 'load'].includes(st.typing.phase.k) ? 3 : 1) : 0;
      if (frozen) continue;
      const fighting = foes().some(u => dist(u, me) <= 70000);
      if (tick % 6 === 0) {   // overclock
        const off = T_('OC_OFF') * 100;
        if (st.oc) {
          if (st.heat >= off && st.ocOffAt == null) st.ocOffAt = tick + T_('OC_LAG');
          if ((st.ocOffAt != null && tick >= st.ocOffAt) || !fighting) { st.oc = false; st.ocOffAt = null; }
        } else if (fighting && st.heat < off - 1500 && tick >= (st.bsodUntil || 0) + N.BSOD_SHY) st.oc = true;
      }
      if (tick % 60 === 0) st.mining = rng.chance(T_('IQ'), 100) ? (!fighting && st.heat < 7000 && !st.oc) : true;
      if (me.hp + fl(me.max / 30) < st.lastHp && rng.chance(25, 100)) {
        if (st.typing && !st.typing.ai && st.typing.phase.k === 'type') st.typing.phase.done = 0;
      }
      st.lastHp = me.hp;
      if (tick >= st.nextDebug && !st.typing && st.program.length) {
        st.nextDebug = tick + 600; st.debugUntil = tick + 60;
        const notice = Math.min(100, T_('NOTICE') + 10);
        for (const c of st.program) { const kept = []; for (const bb of c.bugs) { if (rng.chance(notice, 100)) S.caught++; else kept.push(bb); } c.bugs = kept; }
      }
      activateAi(tick);
      choose(tick);
      stepTyping(tick);
      runProgram(tick);
    }
    return Object.assign(S, { timeline, runs, program: st.program.map(c => ({ f: c.f, lang: c.lang, bugs: c.bugs.slice() })), seconds: TICKS / TPS });
  }

  const LADDER = [0, 1, 2, 3, 4, 5, 6].map(r => [r, null]).concat([[7, 10], [7, 1]]);
  const rankLabel = (r, p) => r < ROOT ? N.RANK_NAMES[r] : p === 1 ? 'Root #1 Zero-Day' : `Root #${p || 10}`;

  /** Every rank, `runs` arenas each (seeds 70217 + i x 73), plus the exact writing numbers of chain() in C++. */
  function compareRow(r, p, runs) {
    const a = { shipped: 0, clean: 0, bugs: 0, writeTicks: 0, frozen: 0, damage: 0, saves: 0, kills: 0, deaths: 0, bsods: 0, runs: 0 };
    let first = null, distinct = 0;
    for (let i = 0; i < runs; i++) {
      const s = simulateSkirmish(r, p, 70217 + i * 73);
      if (!first) first = s;
      for (const k of Object.keys(a)) a[k] += s[k];
      distinct += new Set(s.written.map(w => w.f)).size;   // round 108: how many different functions (no spam)
    }
    const m = meanK(knobs(r, p), F.CHAIN, CPP, 0, runs);
    return { rank: r, root: p, label: rankLabel(r, p), shipped: a.shipped / runs, clean: a.shipped ? a.clean * 100 / a.shipped : 0,
      typing: a.shipped ? a.writeTicks / a.shipped / TPS : 0, freeze: a.frozen / runs / TPS, dps: a.damage / runs / 30,
      saves: a.saves / runs, bugsPerFn: a.shipped ? a.bugs / a.shipped : 0, kills: a.kills / runs, deaths: a.deaths / runs,
      bsods: a.bsods / runs, distinct: distinct / runs, chain: m, timeline: first.timeline, sample: first };
  }
  function compare(runs = 50) { return LADDER.map(([r, p]) => compareRow(r, p, runs)); }

  // ---------------------------------------------------------------- the page
  const W = 820, H = 470, STORE = 'tfm2.coderlab.v1';
  const state = { root: null, rank: 3, rootLv: 10, f: F.CHAIN, lang: -1, seed: 1, speed: 1, ty: null, rng: null, k: null, tick: 0,
    result: null, resultAge: 0, history: [], art: {}, rows: null, arena: null, busy: false, frame: 0, auto: true };
  const $ = s => state.root.querySelector(s);
  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const curRoot = () => state.rank >= ROOT ? state.rootLv : null;
  const curLang = () => state.lang < 0 ? ideal(state.f, state.rank) : state.lang;

  function save() { try { localStorage.setItem(STORE, JSON.stringify({ rank: state.rank, rootLv: state.rootLv, f: state.f, lang: state.lang, speed: state.speed })); } catch (_) {} }
  function load() { try { const s = JSON.parse(localStorage.getItem(STORE) || '{}');
    if (Number.isInteger(s.rank) && s.rank >= 0 && s.rank <= ROOT) state.rank = s.rank;
    if (Number.isInteger(s.rootLv) && s.rootLv >= 1 && s.rootLv <= 10) state.rootLv = s.rootLv;
    if (Number.isInteger(s.f) && s.f >= 0 && s.f < NF) state.f = s.f;
    if (Number.isInteger(s.lang) && s.lang >= -1 && s.lang < CODE.LANGS.length) state.lang = s.lang;
    if ([1, 4, 16].includes(s.speed)) state.speed = s.speed; } catch (_) {} }

  function restart(nextSeed) {
    if (nextSeed) state.seed++;
    state.ty = new Typing(state.f, curLang(), 0, null, 0);
    state.rng = new Rng(state.seed);
    state.k = knobs(state.rank, curRoot());
    state.tick = 0; state.result = null; state.resultAge = 0; state.fails = 0; state.flash = null;
    renderFacts();
  }
  function step() {
    if (!state.ty || state.result) return;
    const ev = state.ty.tick(state.k, state.rng);
    state.tick++;
    if (ev.k === 'failed') { state.fails++; state.flash = { text: state.ty.lang === RUST ? 'error[E0502]: borrow' : 'SyntaxError!', col: '#ff5a5a', age: 0 }; }
    if (ev.k === 'compiled') {
      state.result = { bugs: ev.bugs, ticks: state.tick };
      state.history.unshift({ seed: state.seed, rank: rankLabel(state.rank, curRoot()), f: CODE.FUNCS[state.f].name, lang: LANG_NAMES[state.ty.lang],
        secs: state.tick / TPS, bugs: ev.bugs.map(b => BUGS[b]), fails: state.fails, caught: state.ty.caught });
      state.history.length = Math.min(state.history.length, 8);
      renderFacts();
    }
  }

  // ---- drawing: the pixel font, his body, his rig and crest
  const GLYPH = {};
  function glyphCanvas(ch) {
    if (GLYPH[ch]) return GLYPH[ch];
    const rows = CODE.font.glyphs[ch] || CODE.font.glyphs['?'];
    GLYPH[ch] = rows;
    return rows;
  }
  const KEYWORDS = new Set(CODE.keywords);
  const rgb = c => `rgb(${c[0]},${c[1]},${c[2]})`;
  function colours(line) {
    const out = new Array(line.length).fill(PAL.sym);
    const re = /[A-Za-z_][A-Za-z_0-9]*|\d+(\.\d+)?f?|"[^"]*"/g;
    let m;
    while ((m = re.exec(line))) {
      const tok = m[0];
      const c = KEYWORDS.has(tok) ? PAL.key : /\d/.test(tok[0]) ? PAL.num : tok[0] === '"' ? PAL.str : PAL.name;
      for (let i = m.index; i < m.index + tok.length; i++) out[i] = c;
    }
    return out;
  }
  function text(ctx, s, x, y, cols, sc) {
    for (let i = 0; i < s.length; i++) {
      const rows = glyphCanvas(s[i]);
      ctx.fillStyle = Array.isArray(cols) ? cols[i] : cols;
      for (let yy = 0; yy < rows.length; yy++) for (let xx = 0; xx < rows[yy].length; xx++) {
        if (rows[yy][xx] === '#') ctx.fillRect(x + (i * 5 + xx) * sc, y + yy * sc, sc, sc);
      }
    }
  }
  function loadArt() {
    // the repo's copy (server.js STATIC), else the game's mods folder
    const REL = { body: 'champions/tfm2_custom_coder', vfx: 'vfx/coder_vfx', rank: 'vfx/coder_rank' };
    const sheet = (name) => {
      const img = new Image();
      const o = { img, anims: null };
      img.onerror = () => { img.onerror = null; img.src = `/api/mod-png?id=tfm2_custom&path=${encodeURIComponent(REL[name] + '#sheet.png')}`; };
      img.src = `/coder-${name}.png`;
      fetch(`/coder-${name}.fanim`).then(r => { if (!r.ok) throw new Error(); return r.json(); })
        .catch(() => fetch(`/api/mod-file?id=tfm2_custom&path=${encodeURIComponent(REL[name] + '#anim.fanim')}`).then(r => r.json())
          .then(j => JSON.parse(new TextDecoder().decode(Uint8Array.from(atob(j.base64), c => c.charCodeAt(0))))))
        .then(j => { o.anims = j.anims; }).catch(() => {});
      return o;
    };
    state.art = { body: sheet('body'), vfx: sheet('vfx'), rank: sheet('rank') };
  }
  function blit(ctx, sh, tag, frame, cx, cy, sc) {
    if (!sh || !sh.anims || !sh.img.complete || !sh.img.naturalWidth) return false;
    const a = sh.anims[tag];
    if (!a) return false;
    const fr = a.frames[frame % a.frames.length].data;
    ctx.drawImage(sh.img, fr.x, fr.y, fr.w, fr.h, Math.round(cx - fr.w * sc / 2), Math.round(cy - fr.h * sc / 2), fr.w * sc, fr.h * sc);
    return true;
  }
  function drawHero(ctx, cx, cy, sc, rank, root, f8) {
    const k = rigTier(rank, root);
    if (k) blit(ctx, state.art.rank, `rig${k}`, f8, cx, cy, sc);
    if (!blit(ctx, state.art.body, 'idle', f8, cx, cy, sc)) {
      ctx.fillStyle = '#4a3a7a'; ctx.fillRect(cx - 8 * sc, cy - 16 * sc, 16 * sc, 34 * sc);
    }
    blit(ctx, state.art.rank, `fit${fitOf(rank, root)}`, f8 % 4, cx, cy, sc);   // round 106: his rank's outfit
    blit(ctx, state.art.rank, `kb${fitOf(rank, root)}`, f8, cx, cy, sc);         // and keyboard
    if (k) blit(ctx, state.art.rank, `rigf${k}`, f8, cx, cy, sc);
    blit(ctx, state.art.vfx, rank < ROOT ? `rank${rank}` : `root${root || 10}`, f8, cx, cy, sc);
  }

  function draw() {
    const ctx = state.ctx, ty = state.ty;
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = '#16201b'; ctx.fillRect(0, 0, W, H);
    ctx.fillStyle = '#1d2a23';
    for (let x = 0; x < W; x += 24) ctx.fillRect(x, 0, 1, H);
    const f8 = Math.floor(state.frame / 6) % 8;
    drawHero(ctx, 165, 245, 3, state.rank, curRoot(), f8);
    ctx.fillStyle = '#cfe'; ctx.font = '12px monospace'; ctx.textAlign = 'center';
    ctx.fillText(rankLabel(state.rank, curRoot()), 165, 440);
    ctx.textAlign = 'left';
    if (!ty) return;
    // the terminal
    const code = CODE.FUNCS[ty.f].code[CODE.LANGS[ty.lang]];
    const sc = 2, x0 = 320, y0 = 40, lh = 20;
    const pw = (3 + CODE.MAX_COLS + 2) * 5 * sc + 16, ph = code.length * lh + 50;
    // round 106: the window in his rank's theme
    const TH = THEME_COL[themeOf(state.rank, curRoot())];
    ctx.fillStyle = TH.bg; ctx.fillRect(x0, y0, pw, ph);
    ctx.fillStyle = TH.strip; ctx.fillRect(x0, y0, pw, 18);
    ctx.fillStyle = TH.status; ctx.fillRect(x0, y0 + ph - 22, pw, 22);
    ctx.strokeStyle = TH.frame; ctx.lineWidth = 2; ctx.strokeRect(x0 + 1, y0 + 1, pw - 2, ph - 2);
    ['#ff5f56', '#ffbd2e', '#27c93f'].forEach((c, i) => { ctx.fillStyle = c; ctx.fillRect(x0 + 8 + i * 10, y0 + 7, 6, 6); });
    const title = `${CODE.FUNCS[ty.f].name}.${CODE.LANGS[ty.lang]}`;
    text(ctx, title, x0 + 50, y0 + 5, TH.accent, 1);
    const cur = ty.cursor();
    const p = ty.phase;
    const typedOf = l => {
      if (p.k === 'think') return 0;
      if (p.k === 'type') return l < p.line ? Infinity : l === p.line ? Math.floor(p.done / 100) : 0;
      return Infinity;
    };
    for (let l = 0; l < code.length; l++) {
      const raw = code[l], ind = raw.length - raw.trimStart().length, tl = raw.trim();
      const y = y0 + 24 + l * lh;
      const fixing = p.k === 'fix' && p.queue[0] === l;
      if (fixing) { ctx.fillStyle = 'rgba(255,200,90,0.12)'; ctx.fillRect(x0 + 4, y - 2, pw - 8, lh - 2); }
      if (p.k === 'review') { const at = Math.floor((1 - p.left / Math.max(1, p.total)) * code.length); if (l === at) { ctx.fillStyle = 'rgba(110,200,255,0.12)'; ctx.fillRect(x0 + 4, y - 2, pw - 8, lh - 2); } }
      text(ctx, String(l + 1).padStart(2), x0 + 6, y, 'rgb(104,114,132)', sc);
      const n = fixing ? Math.floor(p.done / 100) : Math.min(typedOf(l), tl.length);
      if (n <= 0) continue;
      const shown = raw.slice(0, ind + Math.min(n, tl.length));
      const cols = colours(raw).slice(0, shown.length);
      const live = ty.typos.filter(q => q.line === l);
      const wrong = shown.split('');
      for (const q of live) if (q.col >= 0 && q.col < n) { wrong[ind + q.col] = String.fromCharCode(33 + ((tl.charCodeAt(q.col) + 7) % 90)); cols[ind + q.col] = q.syntax ? '#ff5a5a' : '#ff9a3c'; }
      for (const q of ty.shownCaught) if (q.line === l && fixing && q.col >= n) { /* being retyped */ }
      text(ctx, wrong.join(''), x0 + 6 + 3 * 5 * sc, y, cols, sc);
      if (cur && cur[0] === l && state.frame % 30 < 18) { ctx.fillStyle = 'rgb(240,244,255)'; ctx.fillRect(x0 + 6 + (3 + shown.length) * 5 * sc + 2, y, 2 * sc, 7 * sc); }
    }
    // the status line
    const sy = y0 + ph - 18;
    const status = state.result
      ? (state.result.bugs.length ? [`[!] compiled with ${state.result.bugs.length} bug${state.result.bugs.length > 1 ? 's' : ''}: ${state.result.bugs.map(b => BUGS[b]).join(', ')}`, '#ff9a3c'] : ['[ok] compiled clean', TH.accent])
      : p.k === 'think' ? ['Thinking...', '#ffaa6e'] : p.k === 'review' ? ['reviewing...', '#6ec8ff']
        : p.k === 'fix' ? [`fixing line ${p.queue[0] + 1}`, '#ffc85a'] : p.k === 'compile' ? [ty.lang === RUST ? 'rustc: compiling...' : ty.lang === CPP ? 'g++ -O2: compiling...' : 'running...', '#ffc85a']
          : ['', '#fff'];
    text(ctx, status[0].slice(0, 60), x0 + 6, sy, status[1], 1);
    if (p.k === 'compile' && p.total) { ctx.fillStyle = '#ffc85a'; ctx.fillRect(x0 + 6, sy + 11, (pw - 12) * (1 - p.left / p.total), 2); }
    if (state.flash && state.flash.age < 50) { text(ctx, state.flash.text, x0 + pw - 6 - state.flash.text.length * 10, sy - 2, state.flash.col, 2); }
    // live counts
    const made = ty.typos.length + ty.caught;
    const lines2 = [
      `t = ${(state.tick / TPS).toFixed(1)} s   ${(state.k.cps100 / 100).toFixed(0)} chars/s   ${LANG_NAMES[ty.lang]}`,
      `typos so far ${made}   caught ${ty.caught}   in the code ${ty.typos.length}   compile errors ${state.fails}`,
    ];
    ctx.fillStyle = '#cfe'; ctx.font = '12px monospace';
    lines2.forEach((s, i) => ctx.fillText(s, x0, y0 + ph + 20 + i * 16));
  }

  function loop() {
    if (!state.root || !state.root.isConnected) { state.raf = null; return; }
    state.raf = requestAnimationFrame(loop);
    if (state.root.offsetParent === null) return;
    state.frame++;
    if (state.flash) state.flash.age++;
    if (!state.result) {
      if (state.speed >= 16) { for (let i = 0; i < 400000 && !state.result; i++) step(); } else for (let i = 0; i < state.speed; i++) step();
    } else if (state.auto && ++state.resultAge > 150) restart(true);
    draw();
  }

  function renderFacts() {
    const r = state.rank, p = curRoot(), k = knobs(r, p);
    const lang = curLang();
    const facts = [
      ['Typing', `${(k.cps100 / 100).toFixed(1)} chars/s`],
      ['Typos', `${(k.typo / 100).toFixed(2)}% a plain character`],
      ['Review eye', `${k.notice}% of typos caught`],
      ['Knows his limits', `${t('AWARE', r, p)}%`],
      ['Judgement (IQ)', `${t('IQ', r, p)}%`],
      ['Program check', `every ${t('CLOCK', r, p)} ticks`],
      ['Prompts', `x${(t('PROMPT', r, p) / 100).toFixed(2)} AI errors`],
      ['Theme', THEME_COL[themeOf(r, p)].name],
      ['Rig', ['plain hoodie', 'one monitor', 'two monitors', 'four monitors, circuit floor', 'monitor wall, code rain', 'Zero-Day: rainbow rain, crown'][rigTier(r, p)]],
      ['Effects', r >= N.HI_RANK ? 'top-rank (_hi)' : 'normal'],
      ['This function', `${chars(state.f, lang)} chars in ${LANG_NAMES[lang]}`],
      ['Kind', CODE.FUNCS[state.f].script ? 'script (fires on compile)' : 'daemon (lives in his program)'],
      ['Cooldown', `${(N.SPEC[state.f][0] * 80 / 100 / 60).toFixed(1)} s`],
      ['Written in', LANG_NAMES.filter((_, i) => avail(state.f, i)).join(', ')],
    ];
    $('#clFacts').innerHTML = `<table class="st-table">${facts.map(([a, b]) => `<tr><td>${a}</td><td>${esc(b)}</td></tr>`).join('')}</table>`;
    $('#clHistory').innerHTML = state.history.length ? `<table class="st-table"><tr><th>seed</th><th>rank</th><th>wrote</th><th>s</th><th>shipped bugs</th></tr>${state.history.map(h =>
      `<tr><td>${h.seed}</td><td>${esc(h.rank)}</td><td>${esc(h.f)} (${esc(h.lang)})</td><td>${h.secs.toFixed(1)}</td><td>${h.bugs.length ? esc(h.bugs.join(', ')) : 'clean'}</td></tr>`).join('')}</table>` : '';
  }

  const TL_COL = ['#26332c', '#6eb9ff', '#b48cff', '#6ec8ff', '#ff5a5a', '#000000', '#2a6cff'];
  function renderTable() {
    const rows = state.rows;
    if (!rows) { $('#clTable').innerHTML = ''; return; }
    const head = '<tr><th>Rank</th><th>Functions shipped</th><th>Different functions</th><th>Clean %</th><th>Typing s / fn</th><th>Frozen s</th><th>Function DPS</th><th>Saves (HP)</th><th>Bugs / fn</th><th>Kills</th><th>Deaths</th><th>Blue screens</th><th>chain() C++ (exact): s, bugs</th></tr>';
    $('#clTable').innerHTML = `<h3>${rows.length < LADDER.length ? `Comparing... ${rows.length}/${LADDER.length}` : `Every rank, ${state.runs} arenas of 30 s each`}</h3>
      <table class="st-table ll-table">${head}${rows.map(r => `<tr><td>${esc(r.label)}</td><td>${r.shipped.toFixed(1)}</td><td>${r.distinct.toFixed(1)}</td><td>${r.clean.toFixed(0)}</td><td>${r.typing.toFixed(1)}</td>
      <td>${r.freeze.toFixed(1)}</td><td>${r.dps.toFixed(0)}</td><td>${r.saves.toFixed(0)}</td><td>${r.bugsPerFn.toFixed(2)}</td><td>${r.kills.toFixed(2)}</td><td>${r.deaths.toFixed(2)}</td>
      <td>${r.bsods.toFixed(2)}</td><td>${r.chain[0].toFixed(1)} s, ${r.chain[1].toFixed(2)}</td></tr>`).join('')}</table>
      <canvas id="clTimeline" width="${W}" height="${rows.length * 18 + 30}"></canvas>
      <p class="muted">Timeline (first seed): <span style="color:${TL_COL[1]}">typing</span> · <span style="color:${TL_COL[2]}">the AI writing</span> · <span style="color:${TL_COL[3]}">review / compile / reload</span> · <span style="color:${TL_COL[4]}">frozen (loop, segfault, OOM)</span> · <span style="color:${TL_COL[6]}">blue screen</span> · dead; ticks under the bar: a function ran (red: a bug struck).
      The arena is a reduced model of his brain on a lane (the shop, the data center's risks and the details of the 76 newer functions are left out: those run by their class); the chain() column is the exact native writing.</p>`;
    const c = $('#clTimeline'), ctx = c.getContext('2d');
    ctx.fillStyle = '#121a16'; ctx.fillRect(0, 0, c.width, c.height);
    const x0 = 150, w = W - x0 - 10;
    rows.forEach((r, i) => {
      const y = 10 + i * 18;
      ctx.fillStyle = '#cfe'; ctx.font = '11px monospace'; ctx.fillText(r.label, 6, y + 10);
      const tl = r.timeline;
      for (let x = 0; x < w; x++) { const v = tl[Math.floor(x / w * tl.length)]; ctx.fillStyle = TL_COL[v]; ctx.fillRect(x0 + x, y, 1, 11); }
      for (const run of r.sample.runs) { ctx.fillStyle = run.res === 'ok' ? '#e8f0ea' : '#ff5a5a'; ctx.fillRect(x0 + Math.floor(run.tick / tl.length * w), y + 12, 1, 4); }
    });
  }
  function runCompare() {
    if (state.busy) return;
    state.busy = true; state.rows = []; state.runs = +$('#clRuns').value || 50;
    let i = 0;
    const next = () => {
      if (i >= LADDER.length || !state.root.isConnected) { state.busy = false; renderTable(); return; }
      const [r, p] = LADDER[i++];
      state.rows.push(compareRow(r, p, state.runs));
      renderTable();
      setTimeout(next, 0);
    };
    renderTable();
    setTimeout(next, 0);
  }
  /** Round 108 ("show me the func"): every function, what it is and what it does; a row picks it to write above. */
  function renderFunctions() {
    const box = $('#clFuncs');
    if (!state.showFuncs) { box.innerHTML = ''; return; }
    const head = '<tr><th>#</th><th>Function</th><th>Kind</th><th>Class</th><th>Tier</th><th>Cooldown</th><th>Ideal</th><th>Written in</th><th>What it does</th></tr>';
    const rows = CODE.FUNCS.map((fn, f) => `<tr data-f="${f}"${f === state.f ? ' class="sel"' : ''} style="cursor:pointer"><td>${f + 1}</td><td><b>${esc(fn.name)}()</b></td>
      <td>${passive(f) ? 'daemon (passive)' : fn.script ? 'script' : 'daemon'}</td><td>${f < 24 ? 'original' : CLASS_NAMES[klass(f)]}${GPU_FX.includes(f) ? ', GPU' : ''}</td>
      <td>${fn.tier}</td><td>${(N.SPEC[f][0] * 80 / 100 / TPS).toFixed(1)} s</td><td>${LANG_NAMES[N.IDEAL[f]]}</td>
      <td>${LANG_NAMES.filter((_, l) => avail(f, l)).join(', ')}</td><td>${esc(DESC[f])}</td></tr>`).join('');
    box.innerHTML = `<h3>Every function (${NF}): ${CODE.FUNCS.filter(fn => fn.script).length} scripts, ${CODE.FUNCS.filter(fn => !fn.script).length} daemons</h3>
      <table class="st-table ll-table">${head}${rows}</table>`;
    box.querySelector('table').onclick = e => {
      const tr = e.target.closest('tr[data-f]');
      if (!tr) return;
      state.f = +tr.dataset.f;
      if (state.lang >= 0 && !avail(state.f, state.lang)) state.lang = -1;
      $('#clFunc').value = String(state.f);
      state.sync();
      renderFunctions();
    };
  }
  function langOptions() {
    return `<option value="-1"${state.lang < 0 ? ' selected' : ''}>his pick</option>${LANG_NAMES.map((n, i) => avail(state.f, i) ? `<option value="${i}"${i === state.lang ? ' selected' : ''}>${n}</option>` : '').join('')}`;
  }
  function runArena() {
    const s = simulateSkirmish(state.rank, curRoot(), 70217 + state.seed * 73);
    const lines3 = s.written.map(w => `${(w.tick / TPS).toFixed(1)} s  ${CODE.FUNCS[w.f].name}.${CODE.LANGS[w.lang]}${w.ai ? ' (' + MODEL_NAMES[w.ai[0]][w.ai[1] ? 1 : 0] + ')' : ''}${w.bugs.length ? '  bugs: ' + w.bugs.map(b => BUGS[b]).join(', ') : ''}`);
    $('#clResult').innerHTML = `<b>${esc(rankLabel(state.rank, curRoot()))}, 30 s arena (seed ${state.seed})</b>: shipped ${s.shipped} (${s.clean} clean), ran ${s.runs} times,
      ${(s.damage / 30).toFixed(0)} function DPS, saved ${s.saves} HP, frozen ${(s.frozen / TPS).toFixed(1)} s, ${s.bsods} blue screens, ${s.kills} kills, ${s.deaths} deaths.
      AI prompts: ${s.prompts.map((n, i) => `${PROVIDERS[i]} ${n}`).join(', ')} (${s.lite} lite).<br><pre class="cl-pre">${esc(lines3.join('\n')) || 'nothing shipped'}</pre>
      In his program at the end: ${s.program.map(c => esc(CODE.FUNCS[c.f].name + '.' + CODE.LANGS[c.lang] + (c.bugs.length ? '*' : ''))).join(' ') || 'nothing'}`;
  }

  function mount(root) {
    if (state.root === root && state.canvas) { if (!state.raf) loop(); return; }
    state.root = root;
    load();
    const rankOpts = N.RANK_NAMES.map((n, i) => `<option value="${i}"${i === state.rank ? ' selected' : ''}>${n}${i === ROOT ? ' (Top 10)' : ''}</option>`).join('');
    root.innerHTML = `<div class="il-shell">
      <div class="il-head"><div><h2>Code lab (Coder)</h2><p>Watch any rank write any function: the real code, typed at his speed, with his typos (red: syntax errors, orange: logic bugs), his review, his fixes and the compiler. The writing is the native code's own, run for run.</p></div><span class="il-chip">${selfTest() ? 'NATIVE PARITY' : 'PARITY CHECK FAILED'}</span></div>
      <div class="il-controls">
        <label>Rank<select id="clRank">${rankOpts}</select></label>
        <label id="clRootWrap"${state.rank === ROOT ? '' : ' style="display:none"'}>Root place<select id="clRoot">${Array.from({ length: 10 }, (_, i) => i + 1).map(v => `<option value="${v}"${v === state.rootLv ? ' selected' : ''}>#${v}${v === 1 ? ' Zero-Day' : ''}</option>`).join('')}</select></label>
        <label>Function<select id="clFunc">${CODE.FUNCS.map((fn, i) => `<option value="${i}"${i === state.f ? ' selected' : ''}>${fn.name}() ${fn.script ? 'script' : 'daemon'}, tier ${fn.tier}</option>`).join('')}</select></label>
        <label>Language<select id="clLang">${langOptions()}</select></label>
        <label>Playback<select id="clSpeed">${[[1, '1x'], [4, '4x'], [16, 'instant']].map(([v, n]) => `<option value="${v}"${v === state.speed ? ' selected' : ''}>${n}</option>`).join('')}</select></label>
        <label>Compare runs<select id="clRuns">${[10, 25, 50, 100].map(v => `<option value="${v}"${v === 50 ? ' selected' : ''}>${v}</option>`).join('')}</select></label>
        <label class="il-check"><input type="checkbox" id="clAuto"${state.auto ? ' checked' : ''}> Next seed when done</label>
      </div>
      <div class="il-main"><div><canvas id="clCanvas" width="${W}" height="${H}"></canvas>
        <div class="il-actions"><button class="btn small primary" data-cl="again">Write it again (next seed)</button><button class="btn small" data-cl="arena">30 s arena at this rank</button><button class="btn small" data-cl="compare">Compare every rank</button><button class="btn small" data-cl="funcs">Every function</button></div>
        <div id="clResult" class="il-result">The arena runs his whole brain for 30 s: what he chooses to write, his program, his rig, the AI ult and the bugs when they strike.</div></div>
        <aside class="il-side"><h3>This rank</h3><div id="clFacts"></div><h3>Last runs</h3><div id="clHistory"></div>
          <p class="muted">Every rank can write every function; the rank decides how fast, how clean, and how well he judges what he can pull off. The rig layers and the top-rank effects are the game's own art.</p></aside></div>
      <div id="clFuncs" class="il-table"></div>
      <div id="clTable" class="il-table"></div></div>`;
    state.canvas = $('#clCanvas'); state.ctx = state.canvas.getContext('2d');
    loadArt();
    const sync = () => { $('#clRootWrap').style.display = state.rank === ROOT ? '' : 'none'; $('#clLang').innerHTML = langOptions(); save(); restart(false); };
    state.sync = sync;
    $('#clRank').onchange = e => { state.rank = +e.target.value; sync(); };
    $('#clRoot').onchange = e => { state.rootLv = +e.target.value; sync(); };
    $('#clFunc').onchange = e => { state.f = +e.target.value; if (state.lang >= 0 && !avail(state.f, state.lang)) state.lang = -1; sync(); };
    $('#clLang').onchange = e => { state.lang = +e.target.value; sync(); };
    $('#clSpeed').onchange = e => { state.speed = +e.target.value; save(); };
    $('#clAuto').onchange = e => { state.auto = e.target.checked; };
    root.querySelector('.il-actions').onclick = e => {
      const a = e.target.dataset && e.target.dataset.cl;
      if (a === 'again') restart(true);
      if (a === 'arena') runArena();
      if (a === 'compare') runCompare();
      if (a === 'funcs') { state.showFuncs = !state.showFuncs; renderFunctions(); }
    };
    restart(false);
    if (!state.raf) loop();
  }
  function setRank(r, p) { state.rank = r; if (p) state.rootLv = p; if (state.canvas) { $('#clRank').value = String(r); restart(false); } }

  const api = { mount, setRank, NATIVE, Rng, Typing, knobs, aiKnobs, tv, ideal, chars, compileTicks, rigTier, needed, run, write, meanK,
    vectors, selfTest, themeOf, fitOf, simulateSkirmish, compare, compareRow, LADDER, rankLabel, DESC, startTiers, _state: state };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (typeof window !== 'undefined') window.TFM2CoderLab = api;
})();
