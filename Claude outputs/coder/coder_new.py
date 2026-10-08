"""Round 107: the 76 new functions (bringing the Coder to 100), each as short game pseudocode in a few languages.

The "code" is flavour drawn over his head, like the first 24: it operates only on game objects (units, enemies,
allies, his program) and is never real software. Each new function is written in its ideal language plus two others
that suit it, so the terminal shows off the 13 languages without ballooning the texture (scripts stay 2-4 lines).

Imported by coder_functions.py, which turns it into coder_code.rs / coder-code.js. Kinds (script / daemon), triggers
and effects live in native/tfm2_custom_ai/src/coder.rs.
"""

# name, tier, ideal language, {lang: [lines]}  (ideal must be one of the langs; lines <= 28 columns)
NEW2 = [
    # ---- cloud / devops (12)
    ('cloud_deploy', 3, 'go', {
        'go': ['func cloudDeploy() {', '  go node.run(ping)', '  bill += 2', '}'],
        'py': ['def cloud_deploy():', '  node = spawn("ping")', '  bill += 2'],
        'ts': ['function cloudDeploy() {', '  node.run(ping);', '  bill += 2;', '}'],
    }),
    ('docker', 2, 'go', {
        'go': ['func docker(a *Unit) {', '  a.box = 90', '  a.hittable = false', '}'],
        'py': ['def docker(a):', '  a.box = 90', '  a.hittable = False'],
        'sh': ['docker() {', '  run --isolate "$1"', '  sleep 1.5', '}'],
    }),
    ('kubernetes', 4, 'go', {
        'go': ['func scale(f Fn) {', '  for i := 0; i < 2; i++ {', '    f.replica()', '  }', '}'],
        'rust': ['fn scale(f: &mut Fn) {', '  for _ in 0..2 {', '    f.replica();', '  }', '}'],
        'py': ['def scale(f):', '  for i in range(2):', '    f.replica()'],
    }),
    ('cron', 2, 'sh', {
        'sh': ['cron() {', '  every 8 run "$last"', '}'],
        'py': ['def cron():', '  every(8, run, last)'],
        'go': ['func cron() {', '  every(8, run, last)', '}'],
        'lua': ['function cron()', '  every(8, run, last)', 'end'],
    }),
    ('load_balancer', 3, 'go', {
        'go': ['func balance() {', '  hurt.dmgTaken = 70', '  rest.dmgTaken = 110', '}'],
        'rust': ['fn balance(t: &mut Team) {', '  t.hurt.taken = 70;', '  t.rest.taken = 110;', '}'],
        'py': ['def balance(team):', '  hurt.taken = 70', '  rest.taken = 110'],
    }),
    ('cdn', 2, 'js', {
        'js': ['function cdn() {', '  for (a of allies)', '    a.speed *= 1.25;', '}'],
        'ts': ['function cdn() {', '  allies.forEach(a =>', '    a.speed *= 1.25);', '}'],
        'py': ['def cdn():', '  for a in allies:', '    a.speed *= 1.25'],
    }),
    ('serverless', 3, 'js', {
        'js': ['function lambda() {', '  near(me, 300)', '   .forEach(hit);', '  bill += 1;', '}'],
        'py': ['def lambda_():', '  for e in near(me, 300):', '    hit(e)', '  bill += 1'],
        'go': ['func lambda() {', '  for _, e := range near(', '      me, 300) { hit(e) }', '  bill++', '}'],
    }),
    ('ci_cd', 3, 'sh', {
        'sh': ['ci_cd() {', '  pipeline.catch = 50', '}'],
        'py': ['def ci_cd():', '  pipeline.catch = 50'],
        'go': ['func ciCd() {', '  pipeline.catch = 50', '}'],
    }),
    ('canary_deploy', 3, 'go', {
        'go': ['func canary() {', '  best.run(target, 50)', '}'],
        'py': ['def canary():', '  best.run(target, 50)'],
        'sh': ['canary() {', '  deploy --weight 50 best', '}'],
    }),
    ('chaos_monkey', 4, 'py', {
        'py': ['def chaos():', '  for i in range(3):', '    pick(program).run(', '      pick(enemies))'],
        'go': ['func chaos() {', '  for i := 0; i < 3; i++ {', '    pick(prog).run(pick(es))', '  }', '}'],
        'js': ['function chaos() {', '  for (let i = 0; i < 3;', '       i++)', '    pick(prog).run(one(es));', '}'],
        'lua': ['function chaos()', '  for i=1,3 do', '    pick(prog):run(one(es))', '  end', 'end'],
    }),
    ('terraform', 3, 'go', {
        'go': ['func terraform(p Vec2) {', '  wall(p, 20)', '  wall.ttl = 180', '}'],
        'rust': ['fn terraform(p: Vec2) {', '  wall(p, 20);', '  wall.ttl = 180;', '}'],
        'py': ['def terraform(p):', '  wall(p, 20)', '  wall.ttl = 180'],
    }),
    ('autoscale', 4, 'go', {
        'go': ['func autoscale() {', '  if outnumbered() {', '    slots += 2', '  }', '}'],
        'rust': ['fn autoscale(me: &mut Me) {', '  if outnumbered() {', '    me.slots += 2;', '  }', '}'],
        'py': ['def autoscale():', '  if outnumbered():', '    me.slots += 2'],
    }),
    # ---- git (8)
    ('git_revert', 3, 'sh', {
        'sh': ['git_revert() {', '  me.hp = hp_at(now - 3)', '}'],
        'py': ['def git_revert():', '  me.hp = hp_at(now - 3)'],
        'js': ['function gitRevert() {', '  me.hp = hpAt(now - 3);', '}'],
    }),
    ('git_blame', 3, 'sh', {
        'sh': ['git_blame() {', '  top = worst_hitter(5)', '  top.taken = 120', '}'],
        'py': ['def git_blame():', '  top = worst_hitter(5)', '  top.taken = 120'],
        'rust': ['fn git_blame(w: &mut Wld) {', '  let t = w.worst(5);', '  w[t].taken = 120;', '}'],
    }),
    ('git_push_force', 3, 'sh', {
        'sh': ['git_push_f() {', '  for e in near(me, 250)', '    knock(e, 260)', '}'],
        'py': ['def git_push_force():', '  for e in near(me, 250):', '    knock(e, 260)'],
        'rust': ['fn git_push_force() {', '  for e in near(me, 250) {', '    knock(e, 260);', '  }', '}'],
    }),
    ('git_stash', 2, 'sh', {
        'sh': ['git_stash() {', '  stash.shield = 160', '}'],
        'py': ['def git_stash():', '  stash.shield = 160'],
        'go': ['func gitStash() {', '  stash.shield = 160', '}'],
    }),
    ('cherry_pick', 3, 'sh', {
        'sh': ['cherry_pick() {', '  me.buff = ally.best()', '}'],
        'py': ['def cherry_pick():', '  me.buff = ally.best()'],
        'js': ['function cherryPick() {', '  me.buff = ally.best();', '}'],
    }),
    ('rebase', 3, 'sh', {
        'sh': ['rebase() {', '  for f in program', '    f.cd = 0', '}'],
        'py': ['def rebase():', '  for f in program:', '    f.cd = 0'],
        'rust': ['fn rebase(p: &mut [Fn]) {', '  for f in p {', '    f.cd = 0;', '  }', '}'],
    }),
    ('merge_conflict', 4, 'py', {
        'py': ['def merge_conflict(a, b):', '  collide(a, b)', '  stun(a, 60)', '  stun(b, 60)'],
        'js': ['function mergeConf(a, b) {', '  collide(a, b);', '  stun(a, 60);', '  stun(b, 60);', '}'],
        'go': ['func mergeConf(a, b *U) {', '  collide(a, b)', '  stun(a, 60)', '  stun(b, 60)', '}'],
    }),
    ('hotfix', 3, 'sh', {
        'sh': ['hotfix() {', '  f = buggiest(program)', '  f.bugs = 0', '  f.run()', '}'],
        'py': ['def hotfix():', '  f = buggiest(program)', '  f.bugs = 0', '  f.run()'],
        'go': ['func hotfix() {', '  f := buggiest(prog)', '  f.bugs = 0', '  f.run()', '}'],
    }),
    # ---- hacking / security (14)
    ('sql_injection', 3, 'sql', {
        'sql': ["inject() {", "  DELETE FROM t.buffs;", "  DELETE FROM t.shield;", "}"],
        'py': ['def sql_injection(t):', '  t.buffs.clear()', '  t.shield = 0'],
        'js': ['function sqlInjection(t) {', '  t.buffs = [];', '  t.shield = 0;', '}'],
    }),
    ('ransomware', 4, 'cpp', {
        'cpp': ['void ransomware(Unit* e) {', '  e->locked = 120;', '}'],
        'py': ['def ransomware(e):', '  e.locked = 120'],
        'rust': ['fn ransomware(e: &mut U) {', '  e.locked = 120;', '}'],
    }),
    ('keylogger', 3, 'cpp', {
        'cpp': ['void keylog(Unit* e) {', '  read(e->hp);', '  e->seen = true;', '}'],
        'py': ['def keylogger(e):', '  read(e.hp)', '  e.seen = True'],
        'rust': ['fn keylogger(e: &mut Unit) {', '  read(e.hp);', '  e.seen = true;', '}'],
    }),
    ('dns_spoof', 3, 'js', {
        'js': ['function dnsSpoof(e) {', '  e.target = tankiest();', '  e.taunt = 90;', '}'],
        'py': ['def dns_spoof(e):', '  e.target = tankiest()', '  e.taunt = 90'],
        'go': ['func dnsSpoof(e *Unit) {', '  e.target = tankiest()', '  e.taunt = 90', '}'],
    }),
    ('honeypot', 3, 'go', {
        'go': ['func honeypot(a *Unit) {', '  a.reflect = 30', '  a.timer = 180', '}'],
        'py': ['def honeypot(a):', '  a.reflect = 30', '  a.timer = 180'],
        'rust': ['fn honeypot(a: &mut Unit) {', '  a.reflect = 30;', '  a.timer = 180;', '}'],
    }),
    ('botnet', 4, 'cpp', {
        'cpp': ['void botnet() {', '  for (int i=0;i<4;++i)', '    spawn(bot);', '}'],
        'rust': ['fn botnet() {', '  for _ in 0..4 {', '    spawn(Bot);', '  }', '}'],
        'py': ['def botnet():', '  for i in range(4):', '    spawn(bot)'],
    }),
    ('buffer_overflow', 4, 'cpp', {
        'cpp': ['void overflow(Unit* t) {', '  char b[8];', '  memset(b, 0, 999);', '  t->hp -= 150 + ap;', '}'],
        'asm': ['overflow:', '  sub rsp, 8', '  mov rcx, 999', '  rep stosb', '  sub [rdi+HP], 150', '  ret'],
        'rust': ['fn overflow(t: &mut Unit) {', '  let mut b = [0u8; 8];', '  smash(&mut b, 999);', '  t.hp -= 150 + ap;', '}'],
    }),
    ('vpn', 2, 'sh', {
        'sh': ['vpn() {', '  me.hidden = 120', '}'],
        'go': ['func vpn() {', '  me.hidden = 120', '}'],
        'py': ['def vpn():', '  me.hidden = 120'],
    }),
    ('fork_bomb', 4, 'sh', {
        'sh': [':(){ :|:& };:', 'fork_bomb() {', '  spray(enemies, 12)', '}'],
        'cpp': ['void fork_bomb() {', '  for (int i=0;i<12;++i)', '    hit(any_enemy(), 8);', '}'],
        'py': ['def fork_bomb():', '  for i in range(12):', '    hit(any_enemy(), 8)'],
    }),
    ('phishing', 3, 'js', {
        'js': ['function phishing(e) {', '  lure(e, myTeam);', '  e.charm = 60;', '}'],
        'py': ['def phishing(e):', '  lure(e, my_team)', '  e.charm = 60'],
        'ts': ['function phishing(e: Unit) {', '  lure(e, myTeam);', '  e.charm = 60;', '}'],
    }),
    ('zero_day', 5, 'rust', {
        'rust': ['fn zero_day(t: &mut Unit) {', '  let d = t.max / 4;', '  t.hp -= d;', '}'],
        'cpp': ['void zero_day(Unit* t) {', '  int d = t->max / 4;', '  t->hp -= d;', '}'],
        'asm': ['zero_day:', '  mov eax, [rdi+MAX]', '  shr eax, 2', '  sub [rdi+HP], eax', '  ret'],
    }),
    ('port_scan', 2, 'sh', {
        'sh': ['port_scan() {', '  reveal(near(me, 300))', '  mark(weakest())', '}'],
        'py': ['def port_scan():', '  reveal(near(me, 300))', '  mark(weakest())'],
        'go': ['func portScan() {', '  reveal(near(me, 300))', '  mark(weakest())', '}'],
    }),
    ('mitm', 3, 'js', {
        'js': ['function mitm(e) {', '  me.hp += e.nextHeal;', '  e.nextHeal = 0;', '}'],
        'py': ['def mitm(e):', '  me.hp += e.next_heal', '  e.next_heal = 0'],
        'go': ['func mitm(e *Unit) {', '  me.hp += e.nextHeal', '  e.nextHeal = 0', '}'],
    }),
    ('brute_force', 3, 'cpp', {
        'cpp': ['void brute(Unit* t) {', '  for (int i=1;i<=10;++i)', '    t->hp -= 5 * i;', '}'],
        'rust': ['fn brute(t: &mut Unit) {', '  for i in 1..=10 {', '    t.hp -= 5 * i;', '  }', '}'],
        'py': ['def brute_force(t):', '  for i in range(1, 11):', '    t.hp -= 5 * i'],
    }),
    # ---- gpu / ai (10)
    ('cuda_kernel', 4, 'cpp', {
        'cpp': ['__global__ void beam(', '    Unit* t) {', '  t->hp -= 120 + ap;', '}'],
        'rust': ['fn cuda_beam(t: &mut Unit) {', '  gpu(|| t.hp -= 120 + ap);', '}'],
        'py': ['def cuda_kernel(t):', '  gpu(lambda:', '    hit(t, 120 + ap))'],
    }),
    ('tensor_core', 4, 'py', {
        'py': ['def tensor_core():', '  ai.pool *= 2', '  ai.lite = False'],
        'cpp': ['void tensor_core() {', '  ai.pool *= 2;', '  ai.lite = false;', '}'],
        'rust': ['fn tensor_core(a: &mut Ai) {', '  ai.pool *= 2;', '  ai.lite = false;', '}'],
    }),
    ('train_model', 4, 'py', {
        'py': ['def train_model():', '  for r in runs:', '    power = min(power', '      + 2, 130)'],
        'rust': ['fn train_model(m: &mut Me) {', '  me.power = (me.power + 2)', '    .min(130);', '}'],
        'go': ['func trainModel() {', '  power = min(power+2, 130)', '}'],
    }),
    ('ray_tracing', 4, 'cpp', {
        'cpp': ['void ray_trace() {', '  reveal(enemies);', '  me->aim = 100;', '}'],
        'rust': ['fn ray_trace(me: &mut Me) {', '  reveal(ENEMIES);', '  me.aim = 100;', '}'],
        'py': ['def ray_tracing():', '  reveal(enemies)', '  me.aim = 100'],
    }),
    ('quantum', 5, 'py', {
        'py': ['def quantum(t):', '  if coin():', '    hit(t, 3 * dmg)'],
        'hs': ['quantum t =', '  when (coin ())', '    (hit t (3 * dmg))'],
        'rust': ['fn quantum(t: &mut Unit) {', '  if coin() {', '    hit(t, 3 * dmg);', '  }', '}'],
    }),
    ('llm_agent', 4, 'py', {
        'py': ['def llm_agent():', '  s = model.write()', '  queue(s)'],
        'ts': ['function llmAgent() {', '  const s = model.write();', '  queue(s);', '}'],
        'go': ['func llmAgent() {', '  s := model.write()', '  queue(s)', '}'],
    }),
    ('deepfake', 3, 'py', {
        'py': ['def deepfake(e):', '  e.target = minion()', '  e.taunt = 120'],
        'js': ['function deepfake(e) {', '  e.target = minion();', '  e.taunt = 120;', '}'],
        'cs': ['void Deepfake(Unit e) {', '  e.Target = Minion();', '  e.Taunt = 120;', '}'],
    }),
    ('diffusion', 3, 'py', {
        'py': ['def diffusion(a):', '  a.regen += 20', '  a.timer = 180'],
        'hs': ['diffusion a =', '  a { regen = 20', '    , timer = 180 }'],
        'rust': ['fn diffusion(a: &mut Unit) {', '  a.regen += 20;', '  a.timer = 180;', '}'],
    }),
    ('overfit', 4, 'py', {
        'py': ['def overfit():', '  last.hp -= 140 + ap'],
        'cpp': ['void overfit() {', '  last->hp -= 140 + ap;', '}'],
        'rust': ['fn overfit(l: &mut Unit) {', '  last.hp -= 140 + ap;', '}'],
    }),
    ('neural_net', 4, 'py', {
        'py': ['def neural_net():', '  me.typo -= 20'],
        'hs': ['neuralNet me =', '  me { typo = typo - 20 }'],
        'cs': ['void NeuralNet() {', '  me.Typo -= 20;', '}'],
    }),
    # ---- data / databases (12)
    ('sql_query', 2, 'sql', {
        'sql': ['SELECT * FROM enemies', 'WHERE hp < max * 0.4;', 'reveal(result);'],
        'py': ['def sql_query():', '  return [e for e in', '    enemies if e.hp', '    < e.max * 0.4]'],
        'cs': ['var low = enemies.Where(', '  e => e.Hp < e.Max*0.4);', 'Reveal(low);'],
    }),
    ('index_scan', 3, 'sql', {
        'sql': ['index_scan() {', '  program.tick = 2;', '}'],
        'py': ['def index_scan():', '  program.tick = 2'],
        'go': ['func indexScan() {', '  program.tick = 2', '}'],
    }),
    ('sharding', 3, 'java', {
        'java': ['void sharding(Unit[] es) {', '  int d = 180 / es.length;', '  for (Unit e : es)', '    e.hp -= d;', '}'],
        'cs': ['void Sharding(Unit[] es) {', '  int d = 180 / es.Length;', '  foreach (var e in es)', '    e.Hp -= d;', '}'],
        'py': ['def sharding(es):', '  d = 180 // len(es)', '  for e in es:', '    e.hp -= d'],
    }),
    ('replication', 3, 'java', {
        'java': ['void replicate(Unit a) {', '  a.shield = last.shield;', '}'],
        'cs': ['void Replicate(Unit a) {', '  a.Shield = last.Shield;', '}'],
        'go': ['func replicate(a *Unit) {', '  a.shield = last.shield', '}'],
    }),
    ('backup', 3, 'sh', {
        'sh': ['backup() {', '  snap = me.hp', '}'],
        'py': ['def backup():', '  snap = me.hp'],
        'sql': ['backup() {', '  INSERT INTO snap', '  VALUES (me.hp);', '}'],
    }),
    ('migrate', 3, 'sql', {
        'sql': ['migrate(ally) {', '  ally.pos = me.pos;', '}'],
        'py': ['def migrate(ally):', '  ally.pos = me.pos'],
        'go': ['func migrate(a *Unit) {', '  a.pos = me.pos', '}'],
    }),
    ('transaction', 4, 'sql', {
        'sql': ['BEGIN;', '  run(next3);', 'COMMIT;'],
        'rust': ['fn transaction(q: &mut Q) {', '  atomic(|| q.run3());', '}'],
        'py': ['def transaction():', '  with atomic():', '    run(next3)'],
    }),
    ('deadlock', 4, 'java', {
        'java': ['void deadlock(Unit a,', '    Unit b) {', '  a.stun = 90;', '  b.stun = 90;', '}'],
        'cs': ['void Deadlock(Unit a,', '    Unit b) {', '  a.Stun = 90;', '  b.Stun = 90;', '}'],
        'py': ['def deadlock(a, b):', '  a.stun = 90', '  b.stun = 90'],
    }),
    ('vacuum', 3, 'sql', {
        'sql': ['VACUUM;', 'pull(near(me, 300), me);'],
        'py': ['def vacuum():', '  for e in near(me, 300):', '    pull(e, me)'],
        'cpp': ['void vacuum() {', '  for (auto* e :', '       near(me, 300))', '    pull(e, me);', '}'],
    }),
    ('map_reduce', 4, 'java', {
        'java': ['void mapReduce() {', '  int d = 90 / seen();', '  for (Unit e : seen)', '    e.hp -= d;', '}'],
        'go': ['func mapReduce() {', '  d := 90 / seen()', '  for _, e := range es {', '    e.hp -= d', '  }', '}'],
        'py': ['def map_reduce():', '  d = 90 // seen()', '  for e in visible:', '    e.hp -= d'],
    }),
    ('blockchain', 4, 'go', {
        'go': ['func onKill() {', '  btc += 50', '  me.shield += 60', '}'],
        'rust': ['fn on_kill(me: &mut Me) {', '  me.btc += 50;', '  me.shield += 60;', '}'],
        'py': ['def blockchain():', '  btc += 50', '  me.shield += 60'],
    }),
    ('bloom_filter', 3, 'cpp', {
        'cpp': ['bool dodge() {', '  return roll() < 15;', '}'],
        'rust': ['fn dodge() -> bool {', '  roll() < 15', '}'],
        'py': ['def bloom_filter():', '  return roll() < 15'],
    }),
    # ---- networking / web (10)
    ('traceroute', 3, 'sh', {
        'sh': ['traceroute() {', '  diver.speed *= 0.6', '  diver.timer = 120', '}'],
        'py': ['def traceroute():', '  diver.speed *= 0.6', '  diver.timer = 120'],
        'go': ['func traceroute() {', '  diver.speed *= 0.6', '  diver.timer = 120', '}'],
    }),
    ('tcp_handshake', 3, 'go', {
        'go': ['func tcp(t *Unit) {', '  hit(t, syn)', '  hit(t, synAck)', '  stun(t, 45)', '}'],
        'rust': ['fn tcp(t: &mut Unit) {', '  hit(t, SYN);', '  hit(t, SYN_ACK);', '  stun(t, 45);', '}'],
        'py': ['def tcp_handshake(t):', '  hit(t, SYN)', '  hit(t, SYN_ACK)', '  stun(t, 45)'],
    }),
    ('udp_flood', 3, 'go', {
        'go': ['func udp() {', '  for i := 0; i < 8; i++ {', '    hit(inCone(), 10)', '  }', '}'],
        'cpp': ['void udp_flood() {', '  for (int i=0;i<8;++i)', '    hit(in_cone(), 10);', '}'],
        'py': ['def udp_flood():', '  for i in range(8):', '    hit(in_cone(), 10)'],
    }),
    ('websocket', 3, 'js', {
        'js': ['function websocket(e) {', '  tether(me, e);', '  every(30, () => hit(e));', '}'],
        'ts': ['function websock(e: Unit) {', '  tether(me, e);', '  every(30, () => hit(e));', '}'],
        'go': ['func websocket(e *Unit) {', '  tether(me, e)', '  every(30, hit, e)', '}'],
    }),
    ('rate_limiter', 3, 'go', {
        'go': ['func rateLimit(e *Unit) {', '  e.atkSpeed *= 0.6', '  e.timer = 180', '}'],
        'ts': ['function rateLim(e: Unit) {', '  e.atkSpeed *= 0.6;', '  e.timer = 180;', '}'],
        'py': ['def rate_limiter(e):', '  e.atk_speed *= 0.6', '  e.timer = 180'],
    }),
    ('oauth', 3, 'ts', {
        'ts': ['function oauth(a: Unit) {', '  me.buff = a.token();', '  me.timer = 180;', '}'],
        'js': ['function oauth(a) {', '  me.buff = a.token();', '  me.timer = 180;', '}'],
        'go': ['func oauth(a *Unit) {', '  me.buff = a.token()', '  me.timer = 180', '}'],
    }),
    ('webhook', 3, 'js', {
        'js': ['function webhook(a) {', '  on(a, "hit", e =>', '    hit(e));', '}'],
        'ts': ['function webhook(a: Unit) {', '  on(a, "hit", hit);', '}'],
        'go': ['func webhook(a *Unit) {', '  on(a, "hit", hit)', '}'],
    }),
    ('captcha', 3, 'js', {
        'js': ['function captcha() {', '  weakest().stun = 60;', '}'],
        'py': ['def captcha():', '  weakest().stun = 60'],
        'cs': ['void Captcha() {', '  Weakest().Stun = 60;', '}'],
    }),
    ('cors', 2, 'js', {
        'js': ['function cors(e) {', '  e.healable = false;', '  e.timer = 180;', '}'],
        'ts': ['function cors(e: Unit) {', '  e.healable = false;', '  e.timer = 180;', '}'],
        'go': ['func cors(e *Unit) {', '  e.healable = false', '  e.timer = 180', '}'],
    }),
    ('api_gateway', 3, 'go', {
        'go': ['func gateway() {', '  for _, a := range near(', '      me, 300) { a.atk++ }', '}'],
        'ts': ['function gateway() {', '  near(me, 300).forEach(a =>', '    a.atk += 10);', '}'],
        'py': ['def api_gateway():', '  for a in near(me, 300):', '    a.atk += 10'],
    }),
    # ---- os / algorithms (10)
    ('sudo', 2, 'sh', {
        'sh': ['sudo() {', '  override(cd)', '  next.power = 150', '}'],
        'py': ['def sudo():', '  override(cd)', '  next.power = 150'],
        'rust': ['fn sudo(me: &mut Me) {', '  me.override_cd();', '  me.next_power = 150;', '}'],
    }),
    ('chmod', 2, 'sh', {
        'sh': ['chmod() {', '  e.buffable = 0', '  e.timer = 180', '}'],
        'py': ['def chmod(e):', '  e.buffable = False', '  e.timer = 180'],
        'go': ['func chmod(e *Unit) {', '  e.buffable = false', '  e.timer = 180', '}'],
    }),
    ('nice', 2, 'sh', {
        'sh': ['nice() {', '  e.speed *= 0.7', '  e.timer = 180', '}'],
        'py': ['def nice(e):', '  e.speed *= 0.7', '  e.timer = 180'],
        'c': ['void nice(Unit* e) {', '  e->speed *= 0.7;', '  e->timer = 180;', '}'],
        'lua': ['function nice(e)', '  e.speed = e.speed*0.7', '  e.timer = 180', 'end'],
    }),
    ('kill_all', 3, 'sh', {
        'sh': ['kill_all() {', '  for s in summons(me)', '    s.hp = 0', '}'],
        'py': ['def kill_all():', '  for s in summons(me):', '    s.hp = 0'],
        'cpp': ['void kill_all() {', '  for (auto* s :', '       summons(me))', '    s->hp = 0;', '}'],
    }),
    ('dijkstra', 4, 'cpp', {
        'cpp': ['void dijkstra(Unit* a) {', '  auto p = path(me, a);', '  dash(me, p);', '}'],
        'rust': ['fn dijkstra(a: &Unit) {', '  let p = path(me, a);', '  dash(me, p);', '}'],
        'py': ['def dijkstra(a):', '  p = path(me, a)', '  dash(me, p)'],
    }),
    ('binary_search', 3, 'cpp', {
        'cpp': ['void bsearch(Unit* t) {', '  if (t->hp < t->max/5)', '    t->hp /= 2;', '}'],
        'rust': ['fn bsearch(t: &mut Unit) {', '  if t.hp < t.max / 5 {', '    t.hp /= 2;', '  }', '}'],
        'py': ['def binary_search(t):', '  if t.hp < t.max // 5:', '    t.hp //= 2'],
    }),
    ('quicksort', 3, 'cpp', {
        'cpp': ['void quicksort() {', '  pull(weakest(), front);', '}'],
        'hs': ['quicksort =', '  pull weakest front'],
        'py': ['def quicksort():', '  pull(weakest(), front)'],
    }),
    ('dynamic_prog', 4, 'cpp', {
        'cpp': ['void dp() {', '  replay(last, 50);', '}'],
        'hs': ['dp = replay last 50'],
        'py': ['def dynamic_prog():', '  replay(last, 50)'],
    }),
    ('regex', 3, 'js', {
        'js': ['function regex(p) {', '  trap(p, 1.5);', '}'],
        'py': ['def regex(p):', '  trap(p, 90)'],
        'rust': ['fn regex(p: Vec2) {', '  trap(p, 90);', '}'],
        'lua': ['function regex(p)', '  trap(p, 90)', 'end'],
    }),
    ('mutex', 3, 'cpp', {
        'cpp': ['void mutex(Unit* a) {', '  if (attackers(a) >= 2)', '    a->taken = 60;', '}'],
        'rust': ['fn mutex(a: &mut Unit) {', '  if attackers(a) >= 2 {', '    a.taken = 60;', '  }', '}'],
        'go': ['func mutex(a *Unit) {', '  if attackers(a) >= 2 {', '    a.taken = 60', '  }', '}'],
    }),
]

# a couple of entries above used 'c' as a stand-in; fold it into cpp so the language set stays the 13
for _n, _t, _id, _L in NEW2:
    if 'c' in _L:
        _L['cpp'] = _L.pop('c')
    assert _id in _L, _n
