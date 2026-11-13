//! Round 152: a small stateful host for the champion glue (`contender.rs`, `rally.rs`), not just their cores. It
//! records positions it is told to set, the CCs applied (and reads them back, names included), the buffs, and every
//! damage call, so a test sees what the game would: did the punch move him, did the hit land, on whom.
//! Contender's round-148 tests drove the core with hand-built frames and never saw his own Bind read back as
//! "rooted", which is why his dash never ran in the game.
use mod_api_stable::{
    AttackTypeV1, BuffV1, CcKindV1, CcV1, EntityHandleV1 as E, EntityKindV1 as K, SimCtxV1, SimVtableV1, StablePassive,
    StableSim, StatV1, UnitAttackV1,
};
use std::{cell::RefCell, ffi::c_void, mem::size_of};

#[derive(Clone)]
pub(crate) struct Unit {
    pub kind: K,
    pub team: usize,
    pub pos: (u64, u64),
    pub hp: usize,
    pub name: &'static str,
    /// what `entity_is_targetable` answers (unverified in the game for camps and minions)
    pub targetable: bool,
}
#[derive(Default)]
pub(crate) struct Host {
    pub tick: usize,
    pub units: Vec<Unit>,
    pub buffs: Vec<Vec<BuffV1>>,
    /// CCs with the tick they expire
    pub ccs: Vec<Vec<(CcV1, usize)>>,
    /// (attacker, target, amount, attack type code)
    pub damage: Vec<(usize, usize, usize, u32)>,
    pub moves: Vec<(usize, (u64, u64))>,
    /// units spawned through `spawn_unit`: (id, sprite name, stat)
    pub spawned: Vec<(usize, String, StatV1)>,
    /// Presentation receipts: name, owner, exact SDK target, lifetime.
    pub view_effects: Vec<(String,usize,mod_api_stable::InputTargetV1,u64)>,
    pub record_view_effects: bool,
    /// projectiles in flight: (caster, team, x, y)
    pub shots: Vec<(usize, usize, u64, u64)>,
    pub reject_spawns: bool,
    /// (unit, amount, ticks) for every `entity_add_shield`
    pub shields: Vec<(usize, usize, usize)>,
    /// (caster, target, amount) for every `heal`
    pub heals: Vec<(usize, usize, usize)>,
    /// units `entity_clear_cc` cleansed
    pub cleansed: Vec<usize>,
    /// (unit, ticks) for every `entity_set_invisible`
    pub invisible: Vec<(usize, usize)>,
}
impl Host {
    pub fn new(units: Vec<Unit>) -> RefCell<Host> {
        let n = units.len();
        RefCell::new(Host { units, buffs: vec![vec![]; n], ccs: vec![vec![]; n], ..Host::default() })
    }
}
fn h<'a>(p: *const c_void) -> &'a RefCell<Host> {
    unsafe { &*(p as *const RefCell<Host>) }
}
fn id(e: E) -> usize {
    e.id().unwrap_or(usize::MAX)
}
fn unit(p: *const c_void, e: E) -> Option<Unit> {
    h(p).borrow().units.get(id(e)).cloned()
}
unsafe extern "C" fn tick(p: *const c_void) -> usize { h(p).borrow().tick }
unsafe extern "C" fn seed(_: *const c_void) -> u64 { 11 }
unsafe extern "C" fn count(p: *const c_void) -> usize { h(p).borrow().units.len() }
unsafe extern "C" fn at(_: *const c_void, i: usize) -> E { E::from_id(i) }
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool { unit(p, e).is_some() }
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool { unit(p, e).is_some_and(|u| u.hp > 0) }
unsafe extern "C" fn targetable(p: *const c_void, e: E) -> bool { unit(p, e).is_some_and(|u| u.targetable) }
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize { unit(p, e).map_or(usize::MAX, |u| u.team) }
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 { unit(p, e).map_or(u32::MAX, |u| u.kind.code()) }
unsafe extern "C" fn is_champ(p: *const c_void, e: E) -> bool { unit(p, e).is_some_and(|u| u.kind == K::Champion) }
unsafe extern "C" fn is_tower(p: *const c_void, e: E) -> bool { unit(p, e).is_some_and(|u| u.kind == K::Tower) }
unsafe extern "C" fn is_minion(p: *const c_void, e: E) -> bool { unit(p, e).is_some_and(|u| u.kind == K::Minion) }
unsafe extern "C" fn visible(_: *const c_void, _: usize, _: usize) -> bool { true }
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let Some(u) = unit(p, e) else { return false };
    (*x, *y) = u.pos;
    true
}
unsafe extern "C" fn spawn_unit(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _owner: usize,
    team: usize,
    x: u64,
    y: u64,
    _ticks: u64,
    s: *const StatV1,
    _a: *const UnitAttackV1,
) -> usize {
    let mut g = h(p).borrow_mut();
    if g.reject_spawns {return usize::MAX;}
    let id = g.units.len();
    let name = String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into_owned();
    g.units.push(Unit { kind: K::Minion, team, pos: (x, y), hp: (*s).hp, name: "summon", targetable: true });
    g.buffs.push(vec![]);
    g.ccs.push(vec![]);
    g.spawned.push((id, name, *s));
    id
}
unsafe extern "C" fn set_hp(p: *mut c_void, e: E, hp: usize) -> bool {
    let mut g = h(p).borrow_mut();
    let Some(u) = g.units.get_mut(id(e)) else { return false };
    u.hp = hp;
    true
}
unsafe extern "C" fn set_pos(p: *mut c_void, e: E, x: u64, y: u64) -> bool {
    let mut g = h(p).borrow_mut();
    let i = id(e);
    let Some(u) = g.units.get_mut(i) else { return false };
    u.pos = (x, y);
    g.moves.push((i, (x, y)));
    true
}
unsafe extern "C" fn hp(p: *const c_void, e: E, hp: *mut usize, max: *mut usize) -> bool {
    let Some(u) = unit(p, e) else { return false };
    *hp = u.hp;
    // spawned units report their own max HP; everyone else 2000
    *max = h(p).borrow().spawned.iter().find(|s| s.0 == id(e)).map_or(2000, |s| s.2.hp);
    true
}
unsafe extern "C" fn radius(_: *const c_void, _: E) -> usize { 10_000 }
unsafe extern "C" fn stat(p: *const c_void, e: E, out: *mut StatV1) -> bool {
    if unit(p, e).is_none() {
        return false;
    }
    *out = StatV1 { attack: 100, move_speed: 1000, hp: 2000, defence: 20, magic_resistance: 20, ..StatV1::default() };
    true
}
unsafe extern "C" fn name(p: *const c_void, e: E, buf: *mut u8, cap: usize, len: *mut usize) -> bool {
    let Some(u) = unit(p, e) else { return false };
    *len = u.name.len();
    if !buf.is_null() {
        std::ptr::copy_nonoverlapping(u.name.as_ptr(), buf, u.name.len().min(cap));
    }
    true
}
unsafe extern "C" fn buffs(p: *const c_void, e: E) -> usize { h(p).borrow().buffs.get(id(e)).map_or(0, |v| v.len()) }
unsafe extern "C" fn buff(p: *const c_void, e: E, i: usize, out: *mut BuffV1) -> bool {
    match h(p).borrow().buffs.get(id(e)).and_then(|v| v.get(i)) {
        Some(b) => {
            *out = *b;
            true
        }
        None => false,
    }
}
unsafe extern "C" fn add_buff(p: *mut c_void, e: usize, b: *const BuffV1) {
    let b = *b;
    if let Some(list) = h(p).borrow_mut().buffs.get_mut(e) {
        list.retain(|x| x.name() != b.name());
        list.push(b);
    }
}
unsafe extern "C" fn remove_buff(p: *mut c_void, e: E, name: *const u8, len: usize) -> usize {
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap_or("");
    let mut g = h(p).borrow_mut();
    let Some(list) = g.buffs.get_mut(id(e)) else { return 0 };
    let before = list.len();
    list.retain(|x| x.name() != name);
    before - list.len()
}
unsafe extern "C" fn ccs(p: *const c_void, e: E) -> usize {
    let g = h(p).borrow();
    g.ccs.get(id(e)).map_or(0, |v| v.iter().filter(|c| c.1 > g.tick).count())
}
unsafe extern "C" fn cc(p: *const c_void, e: E, i: usize, out: *mut CcV1) -> bool {
    let g = h(p).borrow();
    match g.ccs.get(id(e)).and_then(|v| v.iter().filter(|c| c.1 > g.tick).nth(i)) {
        Some(c) => {
            *out = c.0;
            true
        }
        None => false,
    }
}
unsafe extern "C" fn apply_cc(p: *mut c_void, e: usize, c: *const CcV1) {
    let mut g = h(p).borrow_mut();
    let until = g.tick + (*c).tick as usize;
    if let Some(list) = g.ccs.get_mut(e) {
        list.push((*c, until));
    }
}
unsafe extern "C" fn deal(p: *mut c_void, a: usize, t: usize, ad: usize, ap: usize, kind: u32) {
    let mut g = h(p).borrow_mut();
    g.damage.push((a, t, ad + ap, kind));
    if let Some(u) = g.units.get_mut(t) {
        u.hp = u.hp.saturating_sub(ad + ap).max(1);
    }
}
unsafe extern "C" fn champ_count(p: *const c_void) -> usize {
    h(p).borrow().units.iter().filter(|u| u.kind == K::Champion).count()
}
unsafe extern "C" fn champ_at(p: *const c_void, i: usize) -> usize {
    h(p).borrow().units.iter().enumerate().filter(|u| u.1.kind == K::Champion).nth(i).map_or(usize::MAX, |u| u.0)
}
unsafe extern "C" fn level(_: *const c_void, _: E) -> usize { 9 }
unsafe extern "C" fn add_shield(p: *mut c_void, e: E, amount: usize, ticks: usize) -> bool {
    h(p).borrow_mut().shields.push((id(e), amount, ticks));
    true
}
unsafe extern "C" fn heal(p: *mut c_void, caster: usize, target: usize, amount: usize) {
    let mut g = h(p).borrow_mut();
    g.heals.push((caster, target, amount));
    if let Some(u) = g.units.get_mut(target) {
        u.hp = (u.hp + amount).min(2000);
    }
}
unsafe extern "C" fn set_invisible(p: *mut c_void, e: E, ticks: usize) -> bool {
    h(p).borrow_mut().invisible.push((id(e), ticks));
    true
}
unsafe extern "C" fn clear_cc(p: *mut c_void, e: E) -> usize {
    let mut g = h(p).borrow_mut();
    let i = id(e);
    g.cleansed.push(i);
    let n = g.ccs.get(i).map_or(0, |v| v.len());
    if let Some(v) = g.ccs.get_mut(i) {
        v.clear();
    }
    n
}
unsafe extern "C" fn shield(_: *const c_void, _: E) -> usize { 0 }
unsafe extern "C" fn effect(
    p: *mut c_void, name: *const u8, len: usize, owner: usize, target: *const mod_api_stable::InputTargetV1, _: u64, _: u64, time: u64,
) -> bool {
    if !h(p).borrow().record_view_effects {return true;}
    let name=String::from_utf8_lossy(std::slice::from_raw_parts(name,len)).into_owned();
    let mut host=h(p).borrow_mut();
    if host.view_effects.len()<4096 {host.view_effects.push((name,owner,*target,time));}
    true
}

unsafe extern "C" fn shot_count(p: *const c_void) -> usize { h(p).borrow().shots.len() }
unsafe extern "C" fn shot_at(_: *const c_void, i: usize) -> mod_api_stable::ProjectileHandleV1 { mod_api_stable::ProjectileHandleV1::from_id(i) }
unsafe extern "C" fn shot_valid(p: *const c_void, s: mod_api_stable::ProjectileHandleV1) -> bool {
    s.id().is_some_and(|i| i < h(p).borrow().shots.len())
}
unsafe extern "C" fn shot_info(p: *const c_void, s: mod_api_stable::ProjectileHandleV1, out: *mut mod_api_stable::ProjectileInfoV1) -> bool {
    let Some((caster, team, x, y)) = s.id().and_then(|i| h(p).borrow().shots.get(i).copied()) else { return false };
    (*out).caster_id = caster;
    (*out).team = team;
    (*out).x = x;
    (*out).y = y;
    (*out).is_end = false;
    true
}

pub(crate) fn with_sim<T>(host: &RefCell<Host>, f: impl FnOnce(&mut StableSim) -> T) -> T {
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.tick = Some(tick);
        t.seed = Some(seed);
        t.entity_count = Some(count);
        t.entity_at = Some(at);
        t.entity_is_valid = Some(valid);
        t.entity_is_alive = Some(alive);
        t.entity_is_targetable = Some(targetable);
        t.entity_team = Some(team);
        t.entity_kind = Some(kind);
        t.entity_is_champion = Some(is_champ);
        t.entity_is_tower = Some(is_tower);
        t.entity_is_minion = Some(is_minion);
        t.is_visible = Some(visible);
        t.entity_pos = Some(pos);
        t.entity_set_pos = Some(set_pos);
        t.spawn_unit = Some(spawn_unit);
        t.entity_set_hp = Some(set_hp);
        t.entity_hp = Some(hp);
        t.entity_radius = Some(radius);
        t.entity_stat = Some(stat);
        t.entity_name = Some(name);
        t.entity_buff_count = Some(buffs);
        t.entity_buff_at = Some(buff);
        t.add_buff = Some(add_buff);
        t.entity_remove_buff = Some(remove_buff);
        t.entity_cc_count = Some(ccs);
        t.entity_cc_at = Some(cc);
        t.apply_cc = Some(apply_cc);
        t.deal_damage = Some(deal);
        t.champion_count = Some(champ_count);
        t.champion_id_at = Some(champ_at);
        t.entity_level = Some(level);
        t.entity_shield = Some(shield);
        t.entity_add_shield = Some(add_shield);
        t.heal = Some(heal);
        t.entity_clear_cc = Some(clear_cc);
        t.entity_set_invisible = Some(set_invisible);
        t.play_view_effect = Some(effect);
        t.projectile_count = Some(shot_count);
        t.projectile_at = Some(shot_at);
        t.projectile_is_valid = Some(shot_valid);
        t.projectile_info = Some(shot_info);
        let mut raw =
            SimCtxV1 { size: size_of::<SimCtxV1>(), sim: &t, frame: std::ptr::null(), state: host as *const _ as *mut c_void };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        f(&mut sim)
    }
}

/// An open spot: no wall cell within `clear` of it.
pub(crate) fn open_spot(clear: i64) -> (u64, u64) {
    let blocks = crate::walls::boxing_blocks();
    for y in (120_000..840_000).step_by(16_000) {
        for x in (120_000..840_000).step_by(16_000) {
            if blocks.iter().all(|b| {
                x < b.min.x - clear || x > b.max.x + clear || y < b.min.y - clear || y > b.max.y + clear
            }) {
                return (x as u64, y as u64);
            }
        }
    }
    panic!("no open spot");
}

fn champ(team: usize, pos: (u64, u64), name: &'static str) -> Unit {
    Unit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
}

fn run_contender(host: &RefCell<Host>, c: &mut crate::contender::Contender, from: usize, ticks: usize) {
    for t in from..from + ticks {
        host.borrow_mut().tick = t;
        with_sim(host, |sim| c.on_update(sim, t as u64, 0, 0));
    }
}

#[test]
fn contender_own_locks_never_root_him_and_his_punch_dashes_and_lands() {
    let (x, y) = open_spot(70_000);
    let host = Host::new(vec![
        champ(0, (x, y), "tfm2_custom_contender"),
        champ(1, (x + 33_000, y), "fighter"),
    ]);
    let mut c = crate::contender::Contender::default();
    with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
    run_contender(&host, &mut c, 100, 120);
    let g = host.borrow();
    let skill_hits = g.damage.iter().filter(|d| d.0 == 0 && d.1 == 1 && d.3 == AttackTypeV1::Skill.code()).count();
    let moved = g.moves.iter().any(|m| m.0 == 0 && m.1 .0 > x);
    assert!(moved, "the punch dashes toward the target: moves {:?}", &g.moves[..g.moves.len().min(8)]);
    assert!(skill_hits > 0, "a punch landed on the champion: damage {:?}", g.damage);
    // his own Bind and attack lock carry his names, so the frame never reads them as control
    let own: Vec<_> =
        g.ccs[0].iter().filter(|c| c.0.kind != CcKindV1::Animation.code()).map(|c| c.0.name().to_owned()).collect();
    assert!(own.iter().all(|n| crate::contender::OWN_LOCKS.contains(&n.as_str())), "{own:?}");
    assert!(own.iter().any(|n| n == "ct_motion"));
}

#[test]
fn contender_basics_land_on_minions_camps_and_during_a_cast_even_when_untargetable() {
    let (x, y) = open_spot(70_000);
    let minion = Unit { kind: K::Minion, team: 1, pos: (x + 20_000, y), hp: 500, name: "minion", targetable: false };
    let camp = Unit { kind: K::Jungle, team: 2, pos: (x, y + 20_000), hp: 900, name: "camp", targetable: false };
    let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), minion, camp]);
    let mut c = crate::contender::Contender::default();
    with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
    run_contender(&host, &mut c, 100, 5);
    for (t, target) in [(110usize, 1usize), (140, 2)] {
        host.borrow_mut().tick = t;
        // what `contender_basic` leaves when the game's own basic attack lands
        with_sim(&host, |sim| sim.add_buff(0, &crate::timed(&format!("ct_hit:{t}:{target}"), 2)));
        run_contender(&host, &mut c, t, 3);
    }
    let g = host.borrow();
    for target in [1, 2] {
        assert!(
            g.damage.iter().any(|d| d.0 == 0 && d.1 == target && d.3 == AttackTypeV1::BaseAttack.code()),
            "basic on {target} landed: {:?}",
            g.damage
        );
    }
}

#[test]
fn contender_basic_accepted_mid_cast_still_lands() {
    let (x, y) = open_spot(70_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
    let mut c = crate::contender::Contender::default();
    with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
    // run until he is casting
    let mut t = 100;
    while !c.core.busy() && t < 400 {
        run_contender(&host, &mut c, t, 1);
        t += 1;
    }
    assert!(c.core.busy(), "he started a technique");
    let before = host.borrow().damage.iter().filter(|d| d.3 == AttackTypeV1::BaseAttack.code()).count();
    host.borrow_mut().tick = t;
    with_sim(&host, |sim| sim.add_buff(0, &crate::timed(&format!("ct_hit:{t}:1"), 2)));
    run_contender(&host, &mut c, t, 2);
    let after = host.borrow().damage.iter().filter(|d| d.3 == AttackTypeV1::BaseAttack.code()).count();
    assert_eq!(after, before + 1, "the accepted basic landed during the cast");
}
