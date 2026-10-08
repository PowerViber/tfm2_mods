"""Round 101: the Coder's functions, the one source of truth for what he types.

Every function is real code in each language (line 0 is its signature; he types it too). The art (coder_code.py)
draws these lines over his head as he types them; this script also writes native/tfm2_custom_ai/src/coder_code.rs
with each line's length and syntax risk, so the native typing time and typo odds come from the very text shown.

Phase 1: 10 functions (tiers 1-3) in Python, C++ and Rust. Phase 2 (round 102): JavaScript and Assembly for all, and
14 more functions (tiers 3-5): 24 functions in 5 languages.

Run from the repo root: python3 "Claude outputs/coder/coder_functions.py"
"""
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
# round 107: 13 languages. Keep this order in step with coder.rs (PY=0, JS=1, ...). A function is written in a subset
# of them (its ideal plus a couple that suit it); the rest are empty in the generated tables.
LANGS = ['py', 'js', 'ts', 'cpp', 'rust', 'go', 'java', 'cs', 'lua', 'hs', 'sh', 'sql', 'asm']

# name, tier, {lang: [lines]}; keep lines within MAX_COLS so the terminal stays readable
MAX_COLS = 28
FUNCS = [
    ('ping', 1, {
        'py': ['def ping(t):', '  t.hp -= 35 + ap // 2'],
        'cpp': ['void ping(Unit* t) {', '  t->hp -= 35 + ap / 2;', '}'],
        'rust': ['fn ping(t: &mut Unit) {', '  t.hp -= 35 + ap / 2;', '}'],
    }),
    ('shield', 1, {
        'py': ['def shield(a):', '  a.shield += 120', '  a.timer = 180'],
        'cpp': ['void shield(Unit* a) {', '  a->shield += 120;', '  a->timer = 180;', '}'],
        'rust': ['fn shield(a: &mut Unit) {', '  a.shield += 120;', '  a.timer = 180;', '}'],
    }),
    ('heal', 1, {
        'py': ['def heal(a):', '  a.hp = min(a.hp + 80,', '             a.max)'],
        'cpp': ['void heal(Unit* a) {', '  a->hp = std::min(', '    a->hp + 80, a->max);', '}'],
        'rust': ['fn heal(a: &mut Unit) {', '  a.hp = (a.hp + 80)', '    .min(a.max);', '}'],
    }),
    ('scan', 1, {
        'py': ['def scan():', '  seen = [e for e in', '    enemies if e.vis]', '  return seen'],
        'cpp': ['auto scan() {', '  std::vector<Unit*> v;', '  for (auto* e : enemies)', '    if (e->vis)',
                '      v.push_back(e);', '  return v;', '}'],
        'rust': ['fn scan(es: &[Unit])', '    -> Vec<&Unit> {', '  es.iter()', '    .filter(|e| e.vis)',
                 '    .collect()', '}'],
    }),
    ('spray', 2, {
        'py': ['def spray():', '  for e in near(me, 250):', '    e.hp -= 25'],
        'cpp': ['void spray() {', '  for (auto* e :', '       near(me, 250))', '    e->hp -= 25;', '}'],
        'rust': ['fn spray(w: &mut World) {', '  for e in w.near_mut(', '      w.me, 250) {', '    e.hp -= 25;', '  }', '}'],
    }),
    ('blink', 2, {
        'py': ['def blink(p):', '  me.pos = away(p, 300)', '  me.cpu -= 10'],
        'cpp': ['void blink(Vec2 p) {', '  me->pos = away(p, 300);', '  me->cpu -= 10;', '}'],
        'rust': ['fn blink(me: &mut Unit,', '         p: Vec2) {', '  me.pos = away(p, 300);', '  me.cpu -= 10;', '}'],
    }),
    ('slow', 2, {
        'py': ['def slow(t):', '  t.speed *= 0.6', '  t.timer = 120'],
        'cpp': ['void slow(Unit* t) {', '  t->speed *= 0.6f;', '  t->timer = 120;', '}'],
        'rust': ['fn slow(t: &mut Unit) {', '  t.speed *= 0.6;', '  t.timer = 120;', '}'],
    }),
    ('cache', 2, {
        'py': ['def cache():', '  me.cpu = min(me.cpu', '    + 40, 100)', '  me.idle(30)'],
        'cpp': ['void cache() {', '  me->cpu = std::min(', '    me->cpu + 40, 100);', '  me->idle(30);', '}'],
        'rust': ['fn cache(me: &mut Unit) {', '  me.cpu = (me.cpu + 40)', '    .min(100);', '  me.idle(30);', '}'],
    }),
    ('chain', 3, {
        'py': ['def chain(t):', '  for i in range(4):', '    t.hp -= 30', '    t = t.nearest()'],
        'cpp': ['void chain(Unit* t) {', '  for (int i = 0; i < 4;', '       ++i) {', '    t->hp -= 30;',
                '    t = t->nearest();', '  }', '}'],
        'rust': ['fn chain(w: &mut World,', '         mut t: Id) {', '  for _ in 0..4 {', '    w[t].hp -= 30;',
                 '    t = w.nearest(t);', '  }', '}'],
    }),
    ('firewall', 3, {
        'py': ['def firewall(a, b):', '  for x in line(a, b):', '    grid[x] = BURN', '  grid.ttl = 240'],
        'cpp': ['void firewall(Vec2 a,', '              Vec2 b) {', '  for (auto x : line(a,b))',
                '    grid[x] = BURN;', '  grid.ttl = 240;', '}'],
        'rust': ['fn firewall(g: &mut Grid,', '    a: Vec2, b: Vec2) {', '  for x in line(a, b) {',
                 '    g[x] = Cell::Burn;', '  }', '  g.ttl = 240;', '}'],
    }),
]

# Phase 2: JavaScript and Assembly for the first ten, then the fourteen new functions in all five languages.
EXTRA = {
    'ping': {
        'js': ['function ping(t) {', '  t.hp -= 35 + ap / 2;', '}'],
        'asm': ['ping:', '  mov eax, [ap]', '  shr eax, 1', '  add eax, 35', '  sub [rdi+HP], eax', '  ret'],
    },
    'shield': {
        'js': ['function shield(a) {', '  a.shield += 120;', '  a.timer = 180;', '}'],
        'asm': ['shield:', '  add dword [rdi+SH], 120', '  mov dword [rdi+TM], 180', '  ret'],
    },
    'heal': {
        'js': ['function heal(a) {', '  a.hp = Math.min(', '    a.hp + 80, a.max);', '}'],
        'asm': ['heal:', '  mov eax, [rdi+HP]', '  add eax, 80', '  cmp eax, [rdi+MAX]',
                '  cmovg eax, [rdi+MAX]', '  mov [rdi+HP], eax', '  ret'],
    },
    'scan': {
        'js': ['const scan = () =>', '  enemies.filter(', '    e => e.vis);'],
        'asm': ['scan:', '  xor ecx, ecx', '.next:', '  test byte [rsi+VIS], 1', '  jz .skip',
                '  mov [rdi+rcx*8], rsi', '  inc ecx', '.skip:', '  add rsi, UNIT', '  cmp rsi, rdx',
                '  jb .next', '  mov eax, ecx', '  ret'],
    },
    'spray': {
        'js': ['function spray() {', '  for (const e of', '       near(me, 250))', '    e.hp -= 25;', '}'],
        'asm': ['spray:', '  call near_250', '.hit:', '  sub dword [rax+HP], 25', '  add rax, UNIT',
                '  dec ecx', '  jnz .hit', '  ret'],
    },
    'blink': {
        'js': ['function blink(p) {', '  me.pos = away(p, 300);', '  me.cpu -= 10;', '}'],
        'asm': ['blink:', '  mov esi, 300', '  call away', '  mov [rbx+POS], rax',
                '  sub dword [rbx+CPU], 10', '  ret'],
    },
    'slow': {
        'js': ['function slow(t) {', '  t.speed *= 0.6;', '  t.timer = 120;', '}'],
        'asm': ['slow:', '  mov eax, [rdi+SPD]', '  imul eax, eax, 6', '  cdq', '  mov ecx, 10',
                '  idiv ecx', '  mov [rdi+SPD], eax', '  mov dword [rdi+TM], 120', '  ret'],
    },
    'cache': {
        'js': ['function cache() {', '  me.cpu = Math.min(', '    me.cpu + 40, 100);', '  me.idle(30);', '}'],
        'asm': ['cache:', '  mov eax, [rbx+CPU]', '  add eax, 40', '  mov ecx, 100', '  cmp eax, ecx',
                '  cmovg eax, ecx', '  mov [rbx+CPU], eax', '  mov edi, 30', '  call idle', '  ret'],
    },
    'chain': {
        'js': ['function chain(t) {', '  for (let i = 0; i < 4;', '       i++) {', '    t.hp -= 30;',
               '    t = t.nearest();', '  }', '}'],
        'asm': ['chain:', '  mov ecx, 4', '.hop:', '  sub dword [rdi+HP], 30', '  push rcx',
                '  call nearest', '  mov rdi, rax', '  pop rcx', '  loop .hop', '  ret'],
    },
    'firewall': {
        'js': ['function firewall(a, b) {', '  for (const x of', '       line(a, b))', '    grid[x] = BURN;',
               '  grid.ttl = 240;', '}'],
        'asm': ['firewall:', '  call line_ab', '.cell:', '  mov byte [rax], BURN', '  inc rax', '  cmp rax, rdx',
                '  jb .cell', '  mov dword [GRID+TTL], 240', '  ret'],
    },
}

NEW = [
    ('ddos', 3, {
        'py': ['def ddos(t):', '  for i in range(20):', '    send(t, packet)', '  t.atk_speed *= 0.7'],
        'js': ['function ddos(t) {', '  for (let i = 0; i < 20;', '       i++) send(t, pkt);', '  t.atkSpeed *= 0.7;', '}'],
        'cpp': ['void ddos(Unit* t) {', '  for (int i = 0; i < 20;', '       ++i) send(t, pkt);',
                '  t->atk_speed *= 0.7f;', '}'],
        'rust': ['fn ddos(t: &mut Unit) {', '  for _ in 0..20 {', '    send(t, PACKET);', '  }',
                 '  t.atk_speed *= 0.7;', '}'],
        'asm': ['ddos:', '  mov ecx, 20', '.flood:', '  push rcx', '  call send', '  pop rcx', '  loop .flood',
                '  mov eax, [rdi+ASPD]', '  imul eax, eax, 7', '  cdq', '  mov ecx, 10', '  idiv ecx',
                '  mov [rdi+ASPD], eax', '  ret'],
    }),
    ('cleanse', 3, {
        'py': ['def cleanse(a):', '  for d in a.debuffs:', '    a.remove(d)', '  a.shield += 40'],
        'js': ['function cleanse(a) {', '  a.debuffs.length = 0;', '  a.shield += 40;', '}'],
        'cpp': ['void cleanse(Unit* a) {', '  a->debuffs.clear();', '  a->shield += 40;', '}'],
        'rust': ['fn cleanse(a: &mut Unit) {', '  a.debuffs.clear();', '  a.shield += 40;', '}'],
        'asm': ['cleanse:', '  mov dword [rdi+DEBUFS], 0', '  add dword [rdi+SH], 40', '  ret'],
    }),
    ('boost', 3, {
        'py': ['def boost():', '  for a in team:', '    a.atk_speed *= 1.3', '  me.cpu -= 30'],
        'js': ['function boost() {', '  for (const a of team)', '    a.atkSpeed *= 1.3;', '  me.cpu -= 30;', '}'],
        'cpp': ['void boost() {', '  for (auto* a : team)', '    a->atk_speed *= 1.3f;', '  me->cpu -= 30;', '}'],
        'rust': ['fn boost(w: &mut World) {', '  for a in w.team_mut() {', '    a.atk_speed *= 1.3;', '  }',
                 '  w.me_mut().cpu -= 30;', '}'],
        'asm': ['boost:', '  mov ecx, 5', '.ally:', '  mov eax, [rsi+ASPD]', '  imul eax, eax, 13', '  cdq',
                '  mov r8d, 10', '  idiv r8d', '  mov [rsi+ASPD], eax', '  add rsi, UNIT', '  loop .ally',
                '  sub dword [rbx+CPU], 30', '  ret'],
    }),
    ('fork', 4, {
        'py': ['def fork():', '  pid = os.fork()', '  if pid == 0:', '    drone.run(ping, 6)', '  else:',
               '    wait(pid)'],
        'js': ['function fork() {', '  const w = new Worker(', '    "drone.js");', '  w.postMessage(6);', '}'],
        'cpp': ['void fork_drone() {', '  pid_t pid = fork();', '  if (pid == 0) {', '    drone_run(ping, 6);',
                '    _exit(0);', '  }', '  waitpid(pid, 0, 0);', '}'],
        'rust': ['fn fork(w: &mut World) {', '  let d = thread::spawn(', '    move || drone(6));',
                 '  w.drones.push(d);', '}'],
        'asm': ['fork:', '  mov eax, 57', '  syscall', '  test eax, eax', '  jnz .parent', '  mov edi, 6',
                '  call drone_run', '  mov eax, 60', '  syscall', '.parent:', '  ret'],
    }),
    ('swap', 4, {
        'py': ['def swap(a, b):', '  p = a.pos', '  a.pos = b.pos', '  b.pos = p', '  log("swapped")'],
        'js': ['function swap(a, b) {', '  [a.pos, b.pos] =', '    [b.pos, a.pos];', '  log("swapped");', '}'],
        'cpp': ['void swap(Unit* a,', '          Unit* b) {', '  std::swap(a->pos,', '            b->pos);', '  log("swapped");', '}'],
        'rust': ['fn swap(a: &mut Unit,', '        b: &mut Unit) {', '  std::mem::swap(', '    &mut a.pos,',
                 '    &mut b.pos);', '}'],
        'asm': ['swap:', '  mov rax, [rdi+POS]', '  mov rdx, [rsi+POS]', '  mov [rdi+POS], rdx',
                '  mov [rsi+POS], rax', '  ret'],
    }),
    ('sort', 4, {
        'py': ['def sort():', '  q = sorted(enemies,', '    key=lambda e: e.hp)', '  for i, e in enumerate(q):',
               '    e.pos = row(i)', '  q[0].mark()'],
        'js': ['function sort() {', '  const q = [...enemies]', '    .sort((a, b) =>', '      a.hp - b.hp);',
               '  q.forEach((e, i) =>', '    e.pos = row(i));', '  q[0].mark();', '}'],
        'cpp': ['void sort_foes() {', '  auto q = enemies;', '  std::sort(q.begin(),', '    q.end(), by_hp);',
                '  for (int i = 0; i <', '       q.size(); ++i)', '    q[i]->pos = row(i);', '  q[0]->mark();', '}'],
        'rust': ['fn sort(w: &mut World) {', '  let mut q = w.foes();', '  q.sort_by_key(|e| e.hp);',
                 '  for (i, e) in q.iter()', '      .enumerate() {', '    w[*e].pos = row(i);', '  }',
                 '  w[q[0]].mark();', '}'],
        'asm': ['sort:', '  mov rsi, rdi', '  mov ecx, 5', '  call qsort_hp', '  xor edx, edx', '.place:',
                '  call row', '  mov [rdi+POS], rax', '  add rdi, UNIT', '  inc edx', '  cmp edx, 5',
                '  jb .place', '  call mark_first', '  ret'],
    }),
    ('encrypt', 4, {
        'py': ['def encrypt(a):', '  a.key = rand_key()', '  a.dmg_taken *= 0.5', '  a.timer = 180',
               '  return a.key'],
        'js': ['function encrypt(a) {', '  a.key = crypto', '    .randomUUID();', '  a.dmgTaken *= 0.5;',
               '  a.timer = 180;', '}'],
        'cpp': ['Key encrypt(Unit* a) {', '  a->key = rand_key();', '  a->dmg_taken *= 0.5f;',
                '  a->timer = 180;', '  return a->key;', '}'],
        'rust': ['fn encrypt(a: &mut Unit)', '    -> Key {', '  a.key = Key::random();', '  a.dmg_taken *= 0.5;',
                 '  a.timer = 180;', '  a.key', '}'],
        'asm': ['encrypt:', '  rdrand rax', '  mov [rdi+KEY], rax', '  shr dword [rdi+DMG], 1',
                '  mov dword [rdi+TM], 180', '  ret'],
    }),
    ('ddos_all', 4, {
        'py': ['def ddos_all():', '  for e in enemies:', '    for i in range(8):', '      send(e, packet)',
               '    e.atk_speed *= 0.8'],
        'js': ['function ddosAll() {', '  for (const e of enemies)', '  {', '    for (let i = 0; i < 8;',
               '         i++) send(e, pkt);', '    e.atkSpeed *= 0.8;', '  }', '}'],
        'cpp': ['void ddos_all() {', '  for (auto* e : enemies) {', '    for (int i = 0; i < 8;',
                '         ++i) send(e, pkt);', '    e->atk_speed *= 0.8f;', '  }', '}'],
        'rust': ['fn ddos_all(w: &mut World)', '{', '  for e in w.foes_mut() {', '    for _ in 0..8 {',
                 '      send(e, PACKET);', '    }', '    e.atk_speed *= 0.8;', '  }', '}'],
        'asm': ['ddos_all:', '  mov r12d, 5', '.foe:', '  mov ecx, 8', '.pkt:', '  push rcx', '  call send',
                '  pop rcx', '  loop .pkt', '  add rdi, UNIT', '  dec r12d', '  jnz .foe', '  ret'],
    }),
    ('kill9', 5, {
        'py': ['def kill9(t):', '  if t.hp < t.max * 0.15:', '    os.kill(t.pid, 9)', '  else:', '    t.hp -= 60',
               '  log(t.pid)'],
        'js': ['function kill9(t) {', '  if (t.hp < t.max * .15)', '    process.kill(', '      t.pid, "SIGKILL");',
               '  else t.hp -= 60;', '  log(t.pid);', '}'],
        'cpp': ['void kill9(Unit* t) {', '  if (t->hp < t->max *', '      0.15f)', '    kill(t->pid, SIGKILL);',
                '  else t->hp -= 60;', '  log(t->pid);', '}'],
        'rust': ['fn kill9(t: &mut Unit) {', '  if t.hp * 100 < t.max * 15', '  {', '    t.kill(Signal::Kill);',
                 '  } else {', '    t.hp -= 60;', '  }', '  log(t.pid);', '}'],
        'asm': ['kill9:', '  mov eax, [rdi+MAX]', '  imul eax, eax, 15', '  cdq', '  mov ecx, 100', '  idiv ecx',
                '  cmp [rdi+HP], eax', '  jge .hurt', '  mov esi, 9', '  mov eax, 62', '  syscall', '  ret',
                '.hurt:', '  sub dword [rdi+HP], 60', '  ret'],
    }),
    ('rollback', 5, {
        'py': ['def rollback(t):', '  snap = t.history[-180]', '  t.pos = snap.pos', '  if t.ally:',
               '    t.hp = max(t.hp,', '               snap.hp)', '  log("reverted")'],
        'js': ['function rollback(t) {', '  const s = t.history', '    .at(-180);', '  t.pos = s.pos;',
               '  if (t.ally) t.hp =', '    Math.max(t.hp, s.hp);', '}'],
        'cpp': ['void rollback(Unit* t) {', '  auto& s = t->history[', '    t->history.size()', '    - 180];',
                '  t->pos = s.pos;', '  if (t->ally) t->hp =', '    std::max(t->hp, s.hp);', '}'],
        'rust': ['fn rollback(t: &mut Unit) {', '  let s = t.history', '    [t.history.len()', '     - 180];',
                 '  t.pos = s.pos;', '  if t.ally {', '    t.hp = t.hp.max(s.hp);', '  }', '}'],
        'asm': ['rollback:', '  mov rax, [rdi+HIST]', '  mov rcx, [rdi+HLEN]', '  sub rcx, 180',
                '  imul rcx, rcx, SNAP', '  add rax, rcx', '  mov rdx, [rax+S_POS]', '  mov [rdi+POS], rdx',
                '  test byte [rdi+ALLY], 1', '  jz .done', '  mov edx, [rax+S_HP]', '  cmp edx, [rdi+HP]',
                '  jle .done', '  mov [rdi+HP], edx', '.done:', '  ret'],
    }),
    ('recurse', 5, {
        'py': ['def recurse(n):', '  if n == 0:', '    return', '  ping(nearest())', '  recurse(n - 1)'],
        'js': ['function recurse(n) {', '  if (n === 0) return;', '  ping(nearest());', '  recurse(n - 1);', '}'],
        'cpp': ['void recurse(int n) {', '  if (n == 0) return;', '  ping(nearest());', '  recurse(n - 1);', '}'],
        'rust': ['fn recurse(w: &mut World,', '           n: u32) {', '  if n == 0 { return; }',
                 '  ping(w.nearest());', '  recurse(w, n - 1);', '}'],
        'asm': ['recurse:', '  test edi, edi', '  jz .base', '  push rdi', '  call nearest', '  mov rdi, rax',
                '  call ping', '  pop rdi', '  dec edi', '  call recurse', '.base:', '  ret'],
    }),
    ('inject', 5, {
        'py': ['def inject(t):', '  payload = compile(', '    me.program[0])', '  t.exec(payload)', '  t.stun(60)',
               '  t.hp -= 40', '  log("pwned")'],
        'js': ['function inject(t) {', '  const p = new Function(', '    me.program[0]);', '  p.call(t);',
               '  t.stun(60);', '  t.hp -= 40;', '}'],
        'cpp': ['void inject(Unit* t) {', '  auto p = compile(', '    me->program[0]);', '  t->exec(p);',
                '  t->stun(60);', '  t->hp -= 40;', '}'],
        'rust': ['fn inject(t: &mut Unit,', '  me: &Unit) {', '  let p = compile(', '    &me.program[0]);',
                 '  t.exec(&p);', '  t.stun(60);', '  t.hp -= 40;', '}'],
        'asm': ['inject:', '  mov rsi, [rbx+PROG]', '  call compile', '  mov [rdi+RIP], rax',
                '  mov dword [rdi+STUN], 60', '  sub dword [rdi+HP], 40', '  ret'],
    }),
    ('gc', 5, {
        'py': ['def gc():', '  for e in enemies:', '    if e.hp < e.max * .3:', '      del e.shield', '      e.hp -= 50',
               '  collect()'],
        'js': ['function gc() {', '  for (const e of enemies)', '    if (e.hp < e.max * .3)', '    {',
               '      e.shield = 0;', '      e.hp -= 50;', '    }', '}'],
        'cpp': ['void gc() {', '  for (auto* e : enemies)', '    if (e->hp <', '        e->max * 0.3f) {',
                '      e->shield = 0;', '      e->hp -= 50;', '    }', '}'],
        'rust': ['fn gc(w: &mut World) {', '  for e in w.foes_mut()', '    .filter(|e| e.hp * 10', '      < e.max * 3) {',
                 '    e.shield = 0;', '    e.hp -= 50;', '  }', '}'],
        'asm': ['gc:', '  mov ecx, 5', '.foe:', '  mov eax, [rdi+MAX]', '  imul eax, eax, 3', '  cdq',
                '  mov r8d, 10', '  idiv r8d', '  cmp [rdi+HP], eax', '  jge .next', '  mov dword [rdi+SH], 0',
                '  sub dword [rdi+HP], 50', '.next:', '  add rdi, UNIT', '  loop .foe', '  ret'],
    }),
    ('deploy', 5, {
        'py': ['def deploy():', '  for f in me.program:', '    f.power *= 1.5', '    f.cost *= 0.5', '  me.cpu = 100',
               '  push("prod")'],
        'js': ['function deploy() {', '  for (const f of', '       me.program) {', '    f.power *= 1.5;',
               '    f.cost *= 0.5;', '  }', '  push("prod");', '}'],
        'cpp': ['void deploy() {', '  for (auto& f :', '       me->program) {', '    f.power *= 1.5f;',
                '    f.cost *= 0.5f;', '  }', '  push("prod");', '}'],
        'rust': ['fn deploy(me: &mut Unit) {', '  for f in me.program', '    .iter_mut() {', '    f.power *= 1.5;',
                 '    f.cost *= 0.5;', '  }', '  push("prod");', '}'],
        'asm': ['deploy:', '  mov rsi, [rbx+PROG]', '  mov ecx, 5', '.fn:', '  mov eax, [rsi+PWR]', '  imul eax, eax, 3',
                '  shr eax, 1', '  mov [rsi+PWR], eax', '  shr dword [rsi+COST], 1', '  add rsi, FN', '  loop .fn',
                '  mov dword [rbx+CPU], 100', '  ret'],
    }),
]

for _name, _tier, _langs in FUNCS:
    _langs.update(EXTRA[_name])
FUNCS += NEW

# round 107: the ideal language of each of the first 24 (they are written in all of py/js/cpp/rust/asm)
IDEAL = {
    'ping': 'py', 'shield': 'py', 'heal': 'py', 'scan': 'js', 'spray': 'py', 'blink': 'py', 'slow': 'py', 'cache': 'js',
    'chain': 'cpp', 'firewall': 'rust', 'ddos': 'cpp', 'cleanse': 'py', 'boost': 'py', 'fork': 'rust', 'swap': 'rust',
    'sort': 'py', 'encrypt': 'rust', 'ddos_all': 'cpp', 'kill9': 'rust', 'rollback': 'rust', 'recurse': 'cpp',
    'inject': 'rust', 'gc': 'cpp', 'deploy': 'rust',
}
# round 107: script (fires on compile) vs daemon (lives in his program). S for the one-shots.
KIND = {
    'ping': 'D', 'shield': 'D', 'heal': 'D', 'scan': 'D', 'spray': 'D', 'blink': 'S', 'slow': 'D', 'cache': 'D',
    'chain': 'D', 'firewall': 'D', 'ddos': 'D', 'cleanse': 'S', 'boost': 'D', 'fork': 'D', 'swap': 'S', 'sort': 'S',
    'encrypt': 'D', 'ddos_all': 'D', 'kill9': 'S', 'rollback': 'S', 'recurse': 'D', 'inject': 'S', 'gc': 'S',
    'deploy': 'S',
}
NEW_KIND = {
    'cloud_deploy': 'D', 'docker': 'S', 'kubernetes': 'D', 'cron': 'D', 'load_balancer': 'D', 'cdn': 'S',
    'serverless': 'S', 'ci_cd': 'D', 'canary_deploy': 'S', 'chaos_monkey': 'S', 'terraform': 'S', 'autoscale': 'D',
    'git_revert': 'S', 'git_blame': 'S', 'git_push_force': 'S', 'git_stash': 'S', 'cherry_pick': 'S', 'rebase': 'S',
    'merge_conflict': 'S', 'hotfix': 'S', 'sql_injection': 'S', 'ransomware': 'S', 'keylogger': 'D', 'dns_spoof': 'S',
    'honeypot': 'S', 'botnet': 'D', 'buffer_overflow': 'S', 'vpn': 'S', 'fork_bomb': 'S', 'phishing': 'S',
    'zero_day': 'S', 'port_scan': 'S', 'mitm': 'S', 'brute_force': 'S', 'cuda_kernel': 'S', 'tensor_core': 'D',
    'train_model': 'D', 'ray_tracing': 'S', 'quantum': 'S', 'llm_agent': 'D', 'deepfake': 'S', 'diffusion': 'S',
    'overfit': 'S', 'neural_net': 'D', 'sql_query': 'S', 'index_scan': 'D', 'sharding': 'S', 'replication': 'D',
    'backup': 'D', 'migrate': 'S', 'transaction': 'S', 'deadlock': 'S', 'vacuum': 'S', 'map_reduce': 'S',
    'blockchain': 'D', 'bloom_filter': 'D', 'traceroute': 'S', 'tcp_handshake': 'S', 'udp_flood': 'S', 'websocket': 'D',
    'rate_limiter': 'S', 'oauth': 'S', 'webhook': 'D', 'captcha': 'S', 'cors': 'S', 'api_gateway': 'D', 'sudo': 'S',
    'chmod': 'S', 'nice': 'S', 'kill_all': 'S', 'dijkstra': 'S', 'binary_search': 'S', 'quicksort': 'S',
    'dynamic_prog': 'D', 'regex': 'S', 'mutex': 'D',
}
KIND.update(NEW_KIND)
from coder_new import NEW2  # noqa: E402
for _name, _tier, _ideal, _langs in NEW2:
    FUNCS.append((_name, _tier, _langs))
    IDEAL[_name] = _ideal

SYMBOLS = set('()[]{}<>:;,.&|*=+-/%!?\'"_#@^~')


def typed(line: str) -> str:
    """What he actually types: the editor indents for him."""
    return line.strip()


def risk(line: str) -> int:
    """Syntax risk of a line, x10: every character counts 10, a symbol 25 (brackets, ->, ::, &mut are where typos
    bite)."""
    t = typed(line)
    return sum(25 if ch in SYMBOLS else 10 for ch in t if ch != ' ') + 5 * t.count(' ')


def check():
    seen = set()
    for name, tier, langs in FUNCS:
        assert name not in seen, name
        seen.add(name)
        assert set(langs) <= set(LANGS), (name, set(langs) - set(LANGS))
        assert langs, name
        assert IDEAL[name] in langs, (name, IDEAL[name])
        for lang, lines in langs.items():
            assert lines, (name, lang)
            for ln in lines:
                assert len(ln) <= MAX_COLS, (name, lang, ln, len(ln))
                assert all(32 <= ord(c) < 127 for c in ln), (name, ln)


def write_rust():
    """native/tfm2_custom_ai/src/coder_code.rs: per function its tier and, per language, each line's typed length
    and risk."""
    out = ['//! Generated by Claude outputs/coder/coder_functions.py: do not edit. The Coder\'s functions as the art',
           '//! draws them: per language, each line\'s typed length (characters) and syntax risk (x10). A function not',
           '//! written in a language has an empty slice there (round 107).', '',
           f'pub const LANGS: [&str; {len(LANGS)}] = [{", ".join(repr(l).replace(chr(39), chr(34)) for l in LANGS)}];', '',
           '/// (name, tier, per language: [(typed length, risk)] by line)',
           f'pub const FUNCS: [(&str, usize, [&[(usize, usize)]; {len(LANGS)}]); {len(FUNCS)}] = [']
    for name, tier, langs in FUNCS:
        per = []
        for lang in LANGS:
            lines = langs.get(lang, [])
            per.append('&[' + ', '.join(f'({len(typed(ln))}, {risk(ln)})' for ln in lines) + ']')
        out.append(f'    ("{name}", {tier}, [{", ".join(per)}]),')
    out.append('];')
    out.append('')
    out.append('/// The ideal language index of each function (round 107).')
    out.append(f'pub const IDEAL: [usize; {len(FUNCS)}] = [{", ".join(str(LANGS.index(IDEAL[n])) for n, _, _ in FUNCS)}];')
    out.append(f'pub const NF: usize = {len(FUNCS)};')
    out.append(f'pub const KIND: [bool; {len(FUNCS)}] = [{", ".join("true" if KIND[n] == "S" else "false" for n, _, _ in FUNCS)}];  // true: script')
    out.append('')
    out.append('/// Index of each new function (round 107); the first 24 keep their own constants in coder.rs.')
    for i, (n, _, _) in enumerate(FUNCS):
        if i >= 24:
            out.append(f'#[allow(dead_code)] pub const F_{n.upper()}: usize = {i};')
    path = os.path.join(ROOT, 'native', 'tfm2_custom_ai', 'src', 'coder_code.rs')
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write('\n'.join(out) + '\n')
    return path


def write_js():
    """Round 103: editor/coder-code.js for the Code lab: the code itself, each line's typed length and risk (the same
    numbers as coder_code.rs), and the pixel font with its syntax colours."""
    import json
    import coder_font as F
    funcs = [{'name': name, 'tier': tier, 'ideal': LANGS.index(IDEAL[name]), 'script': KIND[name] == 'S', 'code': {l: langs[l] for l in langs},
              'lens': {l: [[len(typed(ln)), risk(ln)] for ln in langs[l]] for l in langs}} for name, tier, langs in FUNCS]
    data = {'LANGS': LANGS, 'MAX_COLS': MAX_COLS, 'FUNCS': funcs,
            'font': {'cw': F.CW, 'ch': F.CH, 'glyphs': F.glyphs()},
            'keywords': sorted(F.KEYWORDS),
            'colours': {'green': F.GREEN, 'cyan': F.CYAN, 'orange': F.ORANGE, 'yellow': F.YELLOW, 'grey': F.GREY}}
    body = json.dumps(data, separators=(',', ':'))
    path = os.path.join(ROOT, 'editor', 'coder-code.js')
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write('// Generated by Claude outputs/coder/coder_functions.py: do not edit. The Coder\'s 24 functions in 5 '
                 'languages, for the Code lab.\n')
        fh.write('(function(d){ if (typeof module !== "undefined" && module.exports) module.exports = d; '
                 'if (typeof window !== "undefined") window.TFM2_CODER = d; })(' + body + ');\n')
    return path


if __name__ == '__main__':
    check()
    print('wrote', write_rust())
    print('wrote', write_js())
    print(len(FUNCS), 'functions,', len(LANGS), 'languages')
