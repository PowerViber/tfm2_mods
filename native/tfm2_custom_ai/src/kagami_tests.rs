//! Kagami's core (`kagami-2.0`): three guarding clones, Clone Save, the team Guardian Angel, the swap lock-on, Mirror
//! Step, Last Reflection as the backup save, and her AI's choices; then the bridge on the glue mock host.
use crate::kagami_core::*;
use crate::kagami_data as d;

const ME: usize = 1;
const ALLY: usize = 2;
const TANK: usize = 3;
const FOE: usize = 10;
/// the clone unit ids the test world hands out, by slot
fn clone_id(slot: usize) -> usize {
    99 - slot
}

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: 1000,
        max_hp: 1000,
        attack: 100,
        champion: true,
        visible: true,
        targetable: true,
        ..Unit::default()
    }
}
fn tower(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        champion: false,
        tower: true,
        hp: 3000,
        max_hp: 3000,
        ..champ(id, team, x, y)
    }
}

/// A tiny world: her, her clones and the units, with her effects applied the way the bridge and the engine would.
struct World {
    core: Core,
    f: Frame,
    clones: Vec<CloneView>,
    log: Vec<(u64, Effect)>,
    wards: Vec<(u64, usize)>,
    clone_wards: Vec<(u64, usize)>,
    /// glides finish at once in this world
    instant: bool,
}
impl World {
    fn new(units: Vec<Unit>) -> World {
        let f = Frame {
            tick: 0,
            owner: ME,
            team: 1,
            alive: true,
            pos: Point::new(400_000, 500_000),
            hp: 1000,
            max_hp: 1000,
            attack: 100,
            ult_unlocked: true,
            home: Point::new(96_000, 864_000),
            units,
            profile: [50; 13],
            manual: true,
            ..Frame::default()
        };
        World {
            core: Core::default(),
            f,
            clones: vec![],
            log: vec![],
            wards: vec![],
            clone_wards: vec![],
            instant: true,
        }
    }
    /// one tick with an optional order
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.clones = self.clones.clone();
        self.f.order = order;
        let out = self.core.step(&self.f);
        let tick = self.f.tick;
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::SpawnClone { slot, at, hp } => {
                    self.clones.push(CloneView {
                        slot: *slot,
                        id: clone_id(*slot),
                        pos: *at,
                        hp: *hp,
                        max_hp: *hp,
                    });
                    // the bridge hands the new id to the core at once
                    self.core.clones[*slot].id = Some(clone_id(*slot));
                }
                Effect::DespawnClone { clone } => self.clones.retain(|c| c.id != *clone),
                Effect::CloneMove { clone, to, .. } => {
                    if self.instant {
                        if let Some(c) = self.clones.iter_mut().find(|c| c.id == *clone) {
                            c.pos = *to;
                        }
                    }
                }
                Effect::Swap { a, b } => {
                    let (pa, pb) = (self.pos_of(*a), self.pos_of(*b));
                    self.set_pos(*a, pb);
                    self.set_pos(*b, pa);
                }
                Effect::Seal { target, .. } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.sealed = true;
                        u.targetable = false;
                    }
                }
                Effect::Revive { target, hp, .. } => {
                    if *target == ME {
                        self.f.hp = self.f.hp.max(*hp);
                    } else if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = u.hp.max(*hp);
                        u.sealed = false;
                        u.targetable = true;
                    }
                }
                Effect::Heal { target, amount, .. } => {
                    if *target == ME {
                        self.f.hp = (self.f.hp + amount).min(self.f.max_hp);
                    } else if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = (u.hp + amount).min(u.max_hp);
                    }
                }
                Effect::Ward { target, .. } => self.wards.push((tick, *target)),
                Effect::CloneWard { target, .. } => self.clone_wards.push((tick, *target)),
                _ => {}
            }
        }
        self.f.tick += 1;
        out
    }
    fn run(&mut self, n: u64) {
        for _ in 0..n {
            self.tick(None);
        }
    }
    fn pos_of(&self, id: usize) -> Point {
        if id == ME {
            self.f.pos
        } else if let Some(c) = self.clones.iter().find(|c| c.id == id) {
            c.pos
        } else {
            self.f.units.iter().find(|u| u.id == id).unwrap().pos
        }
    }
    fn set_pos(&mut self, id: usize, p: Point) {
        if id == ME {
            self.f.pos = p;
        } else if let Some(c) = self.clones.iter_mut().find(|c| c.id == id) {
            c.pos = p;
        } else {
            self.f.units.iter_mut().find(|u| u.id == id).unwrap().pos = p;
        }
    }
    fn unit(&mut self, id: usize) -> &mut Unit {
        self.f.units.iter_mut().find(|u| u.id == id).unwrap()
    }
    fn clone_at(&mut self, slot: usize) -> &mut CloneView {
        self.clones.iter_mut().find(|c| c.slot == slot).unwrap()
    }
    fn guards(&self) -> Vec<usize> {
        self.core.clones.iter().map(|s| s.guard).collect()
    }
    /// a world with all three clones out (spawned on the first tick) and held still unless a test moves them
    fn with_clones(units: Vec<Unit>) -> World {
        let mut w = World::new(units);
        w.tick(None);
        assert_eq!(w.clones.len(), 3, "three clones come out at once");
        for s in w.core.clones.iter_mut() {
            s.moved_at = u64::MAX / 2;
        }
        w
    }
    fn seen<F: Fn(&Effect) -> bool>(&self, f: F) -> bool {
        self.log.iter().any(|(_, e)| f(e))
    }
}

// ------------------------------------------------------------------ three clones

#[test]
fn three_clones_come_out_at_once_with_25_percent_hp_each() {
    let mut w = World::new(vec![]);
    let out = w.tick(None);
    let spawned: Vec<(usize, i64)> = out
        .iter()
        .filter_map(|e| if let Effect::SpawnClone { slot, hp, .. } = e { Some((*slot, *hp)) } else { None })
        .collect();
    assert_eq!(spawned, vec![(0, 250), (1, 250), (2, 250)]);
    // none asked for twice while the bridge reports them
    w.clones.clear();
    for s in w.core.clones.iter_mut() {
        s.id = None;
    }
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::SpawnClone { .. })));
}

#[test]
fn every_clone_has_its_own_12_s_timer() {
    let mut w = World::with_clones(vec![]);
    // clone 0 dies, and 4 s later clone 1 dies
    w.clones.retain(|c| c.slot != 0);
    w.tick(None);
    let first = w.f.tick - 1;
    w.run(240 - 1);
    w.clones.retain(|c| c.slot != 1);
    w.tick(None);
    let second = w.f.tick - 1;
    assert_eq!(second - first, 240);
    let mut back = vec![];
    for _ in 0..(d::CLONE_RESPAWN_TICKS as u64 + 10) {
        let t = w.f.tick;
        for e in w.tick(None) {
            if let Effect::SpawnClone { slot, .. } = e {
                back.push((slot, t));
            }
        }
    }
    assert_eq!(back, vec![(0, first + d::CLONE_RESPAWN_TICKS as u64), (1, second + d::CLONE_RESPAWN_TICKS as u64)]);
}

#[test]
fn each_clone_shoots_for_35_percent_and_casts_its_own_mend_and_charm() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 330_000, 500_000), champ(FOE, 2, 310_000, 500_000)]);
    w.unit(ALLY).hp = 300;
    for slot in 0..3 {
        w.clone_at(slot).pos = Point::new(340_000 + slot as i64 * 1000, 500_000);
    }
    let out = w.tick(None);
    let shots: Vec<(usize, i64)> = out
        .iter()
        .filter_map(|e| if let Effect::CloneShot { clone, amount, .. } = e { Some((*clone, *amount)) } else { None })
        .collect();
    assert_eq!(shots.len(), 3, "every clone shoots: {out:?}");
    assert!(shots.iter().all(|s| s.1 == 100 * d::CLONE_DAMAGE_PCT / 100));
    let heals = out.iter().filter(|e| matches!(e, Effect::Heal { target: ALLY, .. })).count();
    assert_eq!(heals, 3, "each clone casts its own Mend");
    assert_eq!(out.iter().filter(|e| matches!(e, Effect::Charm { .. })).count(), 1, "one Charm: the others see it charmed");
    assert!(w.core.clones.iter().all(|s| s.mend_at == 1 + d::CLONE_MEND_COOLDOWN_TICKS as u64));
    assert_eq!(w.core.mend_at, 0, "her own Mend is untouched");
}

#[test]
fn clones_never_shoot_towers() {
    let mut w = World::with_clones(vec![tower(50, 2, 400_000, 520_000)]);
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::CloneShot { .. })));
}

#[test]
fn a_clone_follows_its_champion_across_the_map_with_no_range_limit() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 800_000, 150_000), champ(FOE, 2, 820_000, 150_000)]);
    w.core.clones[1].guard = ALLY;
    w.core.next_assign = u64::MAX; // keep this assignment
    w.core.endangered = vec![ALLY];
    for s in w.core.clones.iter_mut() {
        s.moved_at = 0;
    }
    w.instant = true;
    let start = w.clone_at(1).pos;
    let mut step = None;
    for _ in 0..12 {
        let out = w.tick(None);
        step = out
            .iter()
            .find_map(|e| if let Effect::CloneMove { clone, to, ticks } = e { (*clone == clone_id(1)).then_some((*to, *ticks)) } else { None });
        if step.is_some() {
            break;
        }
    }
    let step = step.expect("it moves");
    let len = start.dist(step.0) as i64;
    assert_eq!(len, d::CLONE_DASH_SPEED * step.1 as i64, "far away it catches up at dash speed");
    w.run(400);
    assert!(w.clone_at(1).pos.within(Point::new(800_000, 150_000), d::CLONE_CATCH_UP_RANGE), "{:?}", w.clone_at(1).pos);
}

#[test]
fn a_clone_within_120k_of_its_champion_walks_and_only_races_beyond_it() {
    for (gap, speed) in [(100_000, d::CLONE_MOVE_SPEED), (150_000, d::CLONE_DASH_SPEED)] {
        let mut w = World::with_clones(vec![champ(ALLY, 1, 400_000 + gap, 500_000), champ(FOE, 2, 400_000 + gap + 20_000, 500_000)]);
        w.core.clones[1].guard = ALLY;
        w.core.next_assign = u64::MAX;
        w.core.endangered = vec![ALLY];
        w.clone_at(1).pos = Point::new(400_000, 500_000);
        for s in w.core.clones.iter_mut() {
            s.moved_at = 0;
        }
        let mut step = None;
        for _ in 0..12 {
            let out = w.tick(None);
            step = out
                .iter()
                .find_map(|e| if let Effect::CloneMove { clone, to, ticks } = e { (*clone == clone_id(1)).then_some((*to, *ticks)) } else { None });
            if step.is_some() {
                break;
            }
        }
        let (to, ticks) = step.expect("it moves");
        assert!((Point::new(400_000, 500_000).dist(to).round() as i64 - speed * ticks as i64).abs() <= 1, "gap {gap}");
    }
}

// ------------------------------------------------------------------ assignment

#[test]
fn calm_assignment_is_one_on_her_and_two_on_the_hardest_hitter() {
    let mut carry = champ(ALLY, 1, 450_000, 520_000);
    carry.attack = 220;
    let w = World::with_clones(vec![carry, champ(TANK, 1, 420_000, 560_000)]);
    assert_eq!(w.guards(), vec![ME, ALLY, ALLY]);
}

#[test]
fn clones_go_to_the_champions_in_danger_and_stack_on_heavy_fire() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000), champ(TANK, 1, 420_000, 560_000)]);
    // the ally is dived by three, low
    w.f.units.push(champ(FOE, 2, 605_000, 300_000));
    w.f.units.push(champ(11, 2, 600_000, 305_000));
    w.f.units.push(champ(12, 2, 595_000, 300_000));
    w.unit(ALLY).hp = 300;
    w.tick(None);
    let on_ally = w.guards().iter().filter(|g| **g == ALLY).count();
    assert!(on_ally >= 2, "two clones on the dived ally: {:?}", w.guards());
    // a lone poke on the tank later takes a clone only if it beats the margin; the dive keeps its clones
    w.f.units.push(champ(13, 2, 425_000, 560_000));
    w.run(d::CLONE_ASSIGN_TICKS as u64 + 1);
    assert!(w.guards().iter().filter(|g| **g == ALLY).count() >= 2, "{:?}", w.guards());
}

#[test]
fn assignment_does_not_flicker_between_near_equal_picks() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000), champ(TANK, 1, 300_000, 700_000)]);
    w.f.units.push(champ(FOE, 2, 620_000, 300_000));
    w.f.units.push(champ(11, 2, 320_000, 700_000));
    w.tick(None);
    let before = w.guards();
    // the two fights swap a little weight back and forth
    for i in 0..10 {
        let p = if i % 2 == 0 { 618_000 } else { 622_000 };
        w.unit(FOE).pos = Point::new(p, 300_000);
        w.run(d::CLONE_ASSIGN_TICKS as u64);
    }
    assert_eq!(w.guards(), before);
}

// ------------------------------------------------------------------ Clone Save

#[test]
fn a_clone_save_holds_swaps_stuns_for_2_s_heals_shields_and_spends_the_clone() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000), champ(FOE, 2, 610_000, 300_000)]);
    w.core.next_assign = u64::MAX;
    w.core.clones[2].guard = ALLY;
    w.clone_at(2).pos = Point::new(560_000, 300_000);
    w.tick(None);
    assert!(w.clone_wards.iter().any(|x| x.1 == ALLY), "the guarded ally is warded");
    let danger = w.pos_of(ALLY);
    w.unit(ALLY).hp = 1; // the ward held a lethal hit
    let out = w.tick(None);
    assert!(out.iter().any(|e| matches!(e, Effect::SwapLock { a, b: ALLY, .. } if *a == clone_id(2))));
    for _ in 0..(d::SWAP_LOCK_TICKS - 1) {
        let t = w.f.tick;
        w.tick(None);
        assert!(w.clone_wards.contains(&(t, ALLY)), "held at 1 HP through the lock");
    }
    let t = w.f.tick;
    w.tick(None);
    let at = |f: &dyn Fn(&Effect) -> bool| w.log.iter().any(|(tick, e)| *tick == t && f(e));
    assert!(at(&|e| matches!(e, Effect::Swap { b: ALLY, .. })));
    assert!(at(&|e| matches!(e, Effect::Stun { target: FOE, ticks } if *ticks == 120)), "a 2 s stun at the danger spot");
    assert!(at(&|e| matches!(e, Effect::Revive { target: ALLY, hp: 300, blessed: false })));
    assert!(at(&|e| matches!(e, Effect::Shield { target: ALLY, amount: 200, ticks: 240 })));
    assert!(at(&|e| matches!(e, Effect::DespawnClone { clone } if *clone == clone_id(2))));
    assert_eq!(w.pos_of(ALLY), Point::new(560_000, 300_000));
    let _ = danger;
    assert_eq!(w.core.clones[2].id, None);
    assert_eq!(w.core.clones[2].back_at, t + d::CLONE_RESPAWN_TICKS as u64);
    assert!(!w.seen(|e| matches!(e, Effect::Seal { .. })), "the ult isn't used");
}

#[test]
fn a_clone_save_works_for_her_too() {
    let mut w = World::with_clones(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.core.next_assign = u64::MAX;
    w.core.clones[0].guard = ME;
    w.clone_at(0).pos = Point::new(300_000, 500_000);
    // the other two stand far off, so the nearest guarding clone is clone 0
    w.clone_at(1).pos = Point::new(100_000, 100_000);
    w.clone_at(2).pos = Point::new(120_000, 100_000);
    w.tick(None);
    w.f.hp = 1;
    w.tick(None);
    w.run(d::SWAP_LOCK_TICKS as u64);
    assert_eq!(w.f.pos, Point::new(300_000, 500_000));
    assert_eq!(w.f.hp, 300);
    assert!(w.seen(|e| matches!(e, Effect::Shield { target: ME, amount: 200, .. })));
}

#[test]
fn no_guarding_clone_means_no_clone_save() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000), champ(FOE, 2, 610_000, 300_000)]);
    w.core.next_assign = u64::MAX;
    for s in w.core.clones.iter_mut() {
        s.guard = ME;
    }
    w.f.ult_unlocked = false;
    w.tick(None);
    assert!(!w.clone_wards.iter().any(|x| x.1 == ALLY));
    w.unit(ALLY).hp = 50;
    w.tick(None);
    assert!(!w.seen(|e| matches!(e, Effect::SwapLock { b: ALLY, .. })));
}

// ------------------------------------------------------------------ Guardian Angel

#[test]
fn guardian_angel_protects_any_teammate_with_two_charges_and_a_30_s_cooldown_each() {
    let mut w = World::with_clones(vec![
        champ(ALLY, 1, 600_000, 300_000),
        champ(TANK, 1, 300_000, 700_000),
        champ(FOE, 2, 620_000, 300_000),
        champ(11, 2, 310_000, 700_000),
        tower(50, 2, 605_000, 300_000),
    ]);
    w.f.ult_unlocked = false;
    w.unit(ALLY).hp = 600;
    w.unit(TANK).hp = 600;
    w.tick(None);
    // both cross below 50% on the same tick: both get it (two charges)
    w.unit(ALLY).hp = 450;
    w.unit(TANK).hp = 450;
    let out = w.tick(None);
    let knocked: Vec<(usize, usize)> = out.iter().filter_map(|e| if let Effect::Knockback { target, from } = e { Some((*target, *from)) } else { None }).collect();
    assert_eq!(knocked, vec![(FOE, ALLY), (11, TANK)], "away from the champion it protects; never a tower");
    assert!(out.iter().any(|e| matches!(e, Effect::Shield { target: ALLY, amount: 150, .. })));
    assert!(out.iter().any(|e| matches!(e, Effect::Shield { target: TANK, amount: 150, .. })));
    // no charge left: her own drop gets nothing
    w.f.hp = 600;
    w.tick(None);
    w.f.hp = 400;
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Shield { target: ME, .. })));
}

#[test]
fn guardian_angel_refuses_the_same_champion_inside_its_cooldown_even_with_a_charge() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000), champ(TANK, 1, 300_000, 700_000), champ(FOE, 2, 620_000, 300_000)]);
    w.f.ult_unlocked = false;
    let cross = |w: &mut World, id: usize| -> bool {
        w.unit(id).hp = 700;
        w.tick(None);
        w.unit(id).hp = 400;
        w.tick(None).iter().any(|e| matches!(e, Effect::Shield { target, .. } if *target == id))
    };
    assert!(cross(&mut w, ALLY), "A");
    assert!(!cross(&mut w, ALLY), "A again at once: refused, though a charge is left");
    assert!(cross(&mut w, TANK), "B uses the second charge");
    // after 30 s A's own cooldown and a charge are back: A -> B -> A
    w.run(d::ANGEL_PER_CHAMPION_COOLDOWN_TICKS as u64);
    assert!(cross(&mut w, ALLY), "A again after its cooldown");
}

// ------------------------------------------------------------------ Mirror Step

#[test]
fn mirror_step_on_a_teammate_uses_the_safest_clone_at_any_distance() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 800_000, 150_000), champ(FOE, 2, 805_000, 150_000), champ(11, 2, 405_000, 500_000)]);
    w.core.next_assign = u64::MAX;
    w.clone_at(0).pos = Point::new(400_000, 500_000); // in a fight
    w.clone_at(1).pos = Point::new(150_000, 800_000); // safe, far from everyone
    w.clone_at(2).pos = Point::new(790_000, 150_000); // beside the ally
    w.tick(Some(Order::StepAlly(ALLY)));
    assert!(w.core.swaps.iter().any(|s| s.kind == SwapKind::StepAlly && s.clone == clone_id(1) && s.other == ALLY), "{:?}", w.core.swaps);
    w.run(d::SWAP_LOCK_TICKS as u64);
    assert_eq!(w.pos_of(ALLY), Point::new(150_000, 800_000));
    assert!(w.seen(|e| matches!(e, Effect::Stun { target: FOE, ticks: 120 })));
}

#[test]
fn mirror_step_on_a_point_uses_her_nearest_clone_and_keeps_two_cooldowns_and_refunds() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000)]);
    w.core.next_assign = u64::MAX;
    w.clone_at(0).pos = Point::new(390_000, 500_000);
    w.clone_at(1).pos = Point::new(200_000, 500_000);
    w.clone_at(2).pos = Point::new(700_000, 300_000);
    let p = Point::new(400_000, 420_000);
    w.tick(Some(Order::StepPoint(p)));
    assert!(w.core.swaps.iter().any(|s| s.kind == SwapKind::StepPoint && s.clone == clone_id(0)));
    // the teammate cooldown is separate
    w.tick(Some(Order::StepAlly(ALLY)));
    assert!(w.core.rejection.is_empty(), "{}", w.core.rejection);
    // her own is on cooldown
    w.run(60);
    w.tick(Some(Order::StepPoint(Point::new(400_000, 450_000))));
    assert_eq!(w.core.rejection, "step");
    // a cancelled teammate swap refunds only its own half
    let mut w = World::with_clones(vec![champ(ALLY, 1, 600_000, 300_000)]);
    w.tick(Some(Order::StepAlly(ALLY)));
    let full = w.core.step_ally_at;
    w.unit(ALLY).hp = 0;
    w.tick(None);
    assert_eq!(w.core.step_ally_at, full - d::STEP_ALLY_COOLDOWN_TICKS as u64 * d::SWAP_REFUND_PCT as u64 / 100);
    assert_eq!(w.core.step_self_at, 0);
}

#[test]
fn the_swap_locks_on_without_rooting_and_trades_current_places() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 470_000, 500_000)]);
    w.core.next_assign = u64::MAX;
    w.clone_at(0).pos = Point::new(380_000, 500_000);
    w.clone_at(1).pos = Point::new(380_000, 520_000);
    w.clone_at(2).pos = Point::new(380_000, 540_000);
    w.tick(Some(Order::StepAlly(ALLY)));
    let s = w.core.swaps[0];
    let clone_start = w.pos_of(s.clone);
    for _ in 0..(d::SWAP_LOCK_TICKS - 1) {
        let p = w.pos_of(ALLY);
        w.set_pos(ALLY, Point::new(p.x + 5_000, p.y));
        assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Swap { .. })));
    }
    let ally_at = w.pos_of(ALLY);
    w.tick(None);
    assert_eq!(w.pos_of(ALLY), clone_start);
    assert_eq!(w.pos_of(s.clone), ally_at);
}

// ------------------------------------------------------------------ Last Reflection

#[test]
fn last_reflection_is_the_backup_for_teammates_with_no_clone_save_and_its_revive_stuns() {
    let mut w = World::with_clones(vec![
        champ(ALLY, 1, 700_000, 200_000),
        champ(TANK, 1, 100_000, 100_000),
        champ(FOE, 2, 700_000, 210_000),
    ]);
    w.core.next_assign = u64::MAX;
    for s in w.core.clones.iter_mut() {
        s.guard = TANK;
    }
    w.tick(None);
    let t = w.f.tick - 1;
    assert!(w.wards.contains(&(t, ALLY)), "no clone on the ally: the ult wards them");
    assert!(!w.wards.contains(&(t, TANK)), "the tank has clone saves: no ult ward");
    assert!(!w.wards.iter().any(|x| x.1 == ME), "never her");
    w.unit(ALLY).hp = 1;
    let out = w.tick(None);
    assert!(out.iter().any(|e| matches!(e, Effect::Seal { target: ALLY, .. })));
    w.run(d::ULT_SEAL_TICKS as u64);
    assert!(w.seen(|e| matches!(e, Effect::Revive { target: ALLY, hp: 1000, blessed: true })));
    assert!(w.seen(|e| matches!(e, Effect::Stun { target: FOE, ticks } if *ticks == d::ULT_REVIVE_STUN_TICKS as u64)), "the revive stuns");
}

#[test]
fn last_reflection_needs_the_ult_learned_and_a_drop() {
    let mut w = World::with_clones(vec![champ(ALLY, 1, 700_000, 200_000)]);
    w.core.next_assign = u64::MAX;
    for s in w.core.clones.iter_mut() {
        s.guard = ME;
    }
    w.f.ult_unlocked = false;
    w.wards.clear();
    w.unit(ALLY).hp = 1;
    w.tick(None);
    w.tick(None);
    assert!(w.wards.is_empty() && !w.seen(|e| matches!(e, Effect::Seal { .. })));
    w.f.ult_unlocked = true;
    w.unit(ALLY).hp = 50;
    w.tick(None);
    w.tick(None);
    assert!(!w.seen(|e| matches!(e, Effect::Seal { .. })));
    w.unit(ALLY).hp = 20;
    w.tick(None);
    assert!(w.seen(|e| matches!(e, Effect::Seal { target: ALLY, .. })));
}

// ------------------------------------------------------------------ AI

fn ai(units: Vec<Unit>) -> World {
    let mut w = World::with_clones(units);
    w.f.manual = false;
    w
}

#[test]
fn the_ai_heals_the_lowest_teammate_first() {
    let mut w = ai(vec![champ(ALLY, 1, 430_000, 500_000), champ(TANK, 1, 420_000, 520_000)]);
    w.unit(ALLY).hp = 500;
    w.unit(TANK).hp = 300;
    for s in w.core.clones.iter_mut() {
        s.mend_at = u64::MAX;
    }
    w.run(30);
    assert!(w.seen(|e| matches!(e, Effect::Heal { target: TANK, from, .. } if *from == Point::new(400_000, 500_000))));
}

#[test]
fn the_ai_rescues_a_dived_teammate_through_a_safe_clone() {
    let mut w = ai(vec![champ(ALLY, 1, 480_000, 500_000), champ(FOE, 2, 490_000, 500_000)]);
    w.unit(ALLY).hp = 250;
    w.core.mend_at = u64::MAX;
    w.core.next_assign = u64::MAX;
    for s in w.core.clones.iter_mut() {
        s.guard = ME; // no clone save on the ally: Mirror Step is the rescue
    }
    for slot in 0..3 {
        w.clone_at(slot).pos = Point::new(300_000, 500_000 + slot as i64 * 10_000);
    }
    w.run(40);
    assert!(w.seen(|e| matches!(e, Effect::SwapLock { b: ALLY, .. })), "{:?}", w.core.ai.reason);
}

#[test]
fn the_ai_escapes_through_a_clone_when_dived() {
    let mut w = ai(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.core.mend_at = u64::MAX;
    w.core.angel_charges = vec![u64::MAX; 2];
    w.clone_at(1).pos = Point::new(400_000, 600_000);
    w.f.hp = 900;
    w.tick(None);
    w.f.hp = 400;
    w.run(40);
    assert!(w.seen(|e| matches!(e, Effect::SwapLock { b: ME, .. })));
}

// ------------------------------------------------------------------ the bridge on the glue mock host
mod flash_and_art {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit};
    use crate::kagami::{Kagami, SEALED, SWAP_FLASH_GUARD};
    use crate::kagami_core::{PendingSwap, SwapKind};
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};
    fn host() -> std::cell::RefCell<Host> {
        let (x, y) = open_spot(120000);
        Host::new(vec![
            Unit {
                kind: K::Champion,
                team: 0,
                pos: (x, y),
                hp: 2000,
                name: "tfm2_custom_kagami",
                targetable: true,
            },
            Unit {
                kind: K::Champion,
                team: 0,
                pos: (x + 60000, y),
                hp: 2000,
                name: "ally",
                targetable: true,
            },
        ])
    }
    #[test]
    fn sealed_or_swap_participants_cannot_flash_and_no_cooldown_is_spent() {
        let h = host();
        let mut k = Kagami::default();
        with_sim(&h, |s| k.on_update(s, 0, 0, 0));
        let clone = k.core.clones[0].id.unwrap();
        k.core.swaps = vec![PendingSwap {
            kind: SwapKind::StepAlly,
            other: 1,
            clone,
            started: 0,
            at: 30,
        }];
        with_sim(&h, |s| k.sync_flash_guards(s, 0));
        let tag = format!("{SWAP_FLASH_GUARD}0");
        {
            let g = h.borrow();
            assert!(!g.buffs[0].iter().any(|b| b.name() == tag));
            assert!(g.buffs[1].iter().any(|b| b.name() == tag));
            assert!(g.buffs[clone].iter().any(|b| b.name() == tag));
            assert!(g.ccs[1].iter().all(|c| c.0.kind != CcKindV1::Bind.code()));
        }
        assert!(!with_sim(&h, |s| crate::flash::kit_blink(s, 1, (500000, 500000))));
        assert!(!h.borrow().buffs[1].iter().any(|b| b.name() == "flash_cd"));
        k.core.swaps.clear();
        with_sim(&h, |s| {
            k.sync_flash_guards(s, 0);
            s.add_buff(1, &crate::timed(SEALED, 150));
        });
        h.borrow_mut().tick = 1;
        assert!(!with_sim(&h, |s| crate::flash::kit_blink(s, 1, (500000, 500000))));
        assert!(!h.borrow().buffs[1].iter().any(|b| b.name() == "flash_cd"));
        with_sim(&h, |s| {
            s.entity_remove_buff(1, SEALED);
        });
        h.borrow_mut().tick = 2;
        assert!(with_sim(&h, |s| crate::flash::kit_blink(s, 1, (500000, 500000))));
        assert!(h.borrow().buffs[1].iter().any(|b| b.name() == "flash_cd"));
    }
    #[test]
    fn swap_guard_cleanup_is_owner_scoped_and_runs_on_death_and_respawn() {
        let h = host();
        let mut k = Kagami::default();
        with_sim(&h, |s| k.on_update(s, 0, 0, 0));
        let clone = k.core.clones[0].id.unwrap();
        k.core.swaps = vec![PendingSwap {
            kind: SwapKind::StepPoint,
            other: 0,
            clone,
            started: 0,
            at: 30,
        }];
        with_sim(&h, |s| {
            k.sync_flash_guards(s, 0);
            s.add_buff(0, &crate::timed("kg_swap_pending:77", 30));
        });
        h.borrow_mut().units[0].hp = 0;
        h.borrow_mut().tick = 1;
        with_sim(&h, |s| k.on_update(s, 1, 0, 0));
        let g = h.borrow();
        assert!(!g.buffs[0].iter().any(|b| b.name() == "kg_swap_pending:0"));
        assert!(g.buffs[0].iter().any(|b| b.name() == "kg_swap_pending:77"));
        drop(g);
        with_sim(&h, |s| k.on_spawn(s, 0, 0));
        assert!(!h.borrow().buffs[clone].iter().any(|b| b.name() == "kg_swap_pending:0"));
    }
    #[test]
    fn clone_art_preserves_core_state_hp_targetability_and_motion_without_owner_animation_locks() {
        let a = host();
        let b = host();
        let mut old = Kagami::default();
        let mut new = Kagami::default();
        new.configure("{\"art_v1\":1}");
        for t in 0..120 {
            a.borrow_mut().tick = t;
            b.borrow_mut().tick = t;
            with_sim(&a, |s| old.on_update(s, t as u64, 0, 0));
            with_sim(&b, |s| new.on_update(s, t as u64, 0, 0));
            assert_eq!(serde_json::to_value(&old.core).unwrap(), serde_json::to_value(&new.core).unwrap());
        }
        let g = b.borrow();
        assert!(g.ccs[0].iter().all(|c| c.0.kind != CcKindV1::Animation.code()));
        let clone = new.core.clones[0].id.unwrap();
        assert!(g.units[clone].targetable);
        assert_eq!(
            g.ccs[clone].iter().filter(|c| c.0.kind == CcKindV1::Animation.code()).count(),
            1,
            "one lifetime visual mask; never renewed"
        );
        assert!(g.ccs[clone].iter().all(|c| c.0.kind != CcKindV1::Bind.code()));
        assert_eq!(a.borrow().damage, g.damage);
    }
}

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::kagami::{Kagami, CHARM, CLONE_BODY, REVIVED, SEALED, WARD};
    use crate::kagami_core::{Effect, Point};
    use crate::kagami_data as d;
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit {
            kind: K::Champion,
            team,
            pos,
            hp: 2000,
            name,
            targetable: true,
        }
    }

    #[test]
    fn the_clone_is_a_summoned_body_that_the_frame_finds_and_keeps_out_of_the_units() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kagami"), champ(1, (x + 40_000, y), "foe")]);
        let mut k = Kagami::default();
        with_sim(&host, |sim| k.on_update(sim, 0, 0, 0));
        {
            let g = host.borrow();
            let (id, name, stat) = g.spawned.first().cloned().expect("a clone");
            assert_eq!(name, CLONE_BODY);
            assert_eq!(stat.hp as i64, 2000 * d::CLONE_HP_PCT / 100);
            assert_eq!(stat.move_speed, 1, "it never walks by itself");
            assert!(g.ccs[id].iter().any(|c| c.0.kind == CcKindV1::BlockAttack.code()));
        }
        host.borrow_mut().tick = 1;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        let ids: Vec<usize> = f.clones.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![2, 3, 4], "three clones, by slot");
        assert!(f.units.iter().all(|u| !ids.contains(&u.id)), "no clone is one of the units");
    }

    #[test]
    fn a_swap_trades_the_two_current_places_and_the_stun_never_hits_a_tower() {
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kagami"),
            champ(0, (x + 60_000, y), "ally"),
            HostUnit {
                kind: K::Tower,
                team: 1,
                pos: (x + 70_000, y),
                hp: 3000,
                name: "tower",
                targetable: true,
            },
            champ(1, (x + 65_000, y), "foe"),
        ]);
        let mut k = Kagami::default();
        with_sim(&host, |sim| k.on_update(sim, 0, 0, 0));
        let clone = host.borrow().spawned[0].0;
        host.borrow_mut().units[clone].pos = (x - 20_000, y);
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Swap { a: clone, b: 1 },
                    Effect::Stun { target: 2, ticks: 60 },
                    Effect::Stun { target: 3, ticks: 60 },
                ],
            )
        });
        let g = host.borrow();
        assert_eq!(g.units[1].pos, (x - 20_000, y));
        assert_eq!(g.units[clone].pos, (x + 60_000, y));
        assert!(g.ccs[2].is_empty(), "never a tower");
        assert!(g.ccs[3].iter().any(|c| c.0.kind == CcKindV1::Stun.code()));
    }

    #[test]
    fn the_ward_holds_the_seal_banishes_and_the_blessed_revive_sets_full_hp_with_charm_defence_cc_immunity_and_a_shield() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kagami"), champ(0, (x + 300_000, y), "ally")]);
        let mut k = Kagami::default();
        with_sim(&host, |sim| {
            k.apply(sim, 0, vec![Effect::Ward { target: 1, ticks: 3 }, Effect::Seal { target: 1, ticks: 150 }])
        });
        {
            let g = host.borrow();
            assert!(g.buffs[1].iter().any(|b| b.name() == WARD && b.undying));
            assert!(g.buffs[1].iter().any(|b| b.name() == SEALED));
            assert_eq!(g.banished, vec![(0, 1, 150)]);
        }
        host.borrow_mut().units[1].hp = 1;
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![Effect::Revive {
                    target: 1,
                    hp: 2000,
                    blessed: true,
                }],
            )
        });
        let g = host.borrow();
        assert_eq!(g.units[1].hp, 2000);
        assert!(!g.buffs[1].iter().any(|b| b.name() == SEALED));
        let r = g.buffs[1].iter().find(|b| b.name() == REVIVED).expect("blessed");
        assert!(r.cc_immune);
        assert_eq!(r.defence_mult, d::ULT_DEFENCE_PCT as i32);
        assert_eq!(r.attack_speed_mult, d::CHARM_ATTACK_SPEED_PCT as i32);
        assert_eq!(r.duration_tick, d::ULT_REVIVE_TICKS as usize);
        assert!(g
            .shields
            .contains(&(1, (2000 * d::ULT_SHIELD_PCT_MAX_HP / 100) as usize, d::ULT_REVIVE_TICKS as usize)));
    }

    #[test]
    fn charm_angel_and_the_clone_shot_reach_the_host() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kagami"),
            champ(0, (x + 10_000, y), "ally"),
            champ(1, (x + 20_000, y), "foe"),
        ]);
        let mut k = Kagami::default();
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Charm { target: 1, ticks: 300 },
                    Effect::Knockback { target: 2, from: 1 },
                    Effect::CloneShot {
                        clone: 999,
                        target: 2,
                        amount: 40,
                        from: Point::new(x as i64, y as i64),
                    },
                ],
            )
        });
        let g = host.borrow();
        let c = g.buffs[1].iter().find(|b| b.name() == CHARM).unwrap();
        assert_eq!((c.attack_speed_mult, c.move_speed_mult, c.attack_mult), (25, 15, 10));
        assert_eq!(g.knockbacks, vec![(1, 2, d::ANGEL_KNOCK_SPEED as usize, d::ANGEL_KNOCK_TICKS as usize)]);
        assert!(g.damage.iter().any(|dm| dm.1 == 2 && dm.2 == 40));
    }

    #[test]
    fn a_whole_rescue_runs_through_the_bridge() {
        let (x, y) = open_spot(150_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kagami"),
            champ(0, (x + 80_000, y), "ally"),
            champ(1, (x + 90_000, y), "foe"),
        ]);
        let mut k = Kagami::default();
        with_sim(&host, |sim| k.on_update(sim, 0, 0, 0));
        let clone = host.borrow().spawned[0].0;
        host.borrow_mut().units[1].hp = 400;
        k.core.mend_at = u64::MAX;
        let mut swapped = false;
        for t in 1..60 {
            host.borrow_mut().tick = t;
            with_sim(&host, |sim| k.on_update(sim, 0, 0, 0));
            if host.borrow().units[1].pos.0 < x + 40_000 {
                swapped = true;
                break;
            }
        }
        assert!(swapped, "the ally was pulled out: {:?}", host.borrow().units[1].pos);
        let g = host.borrow();
        assert!(g.units[clone].pos.0 > x + 40_000, "the clone took the ally's spot");
        assert!(g.ccs[2].iter().any(|c| c.0.kind == CcKindV1::Stun.code()), "the foe there is stunned");
    }
}

// ------------------------------------------------------------------ kagami-2.1: a roamer who can jungle

fn camp_monster(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, farmable: true, hp: 2_000, max_hp: 2_000, ..champ(id, 9, x, y) }
}

#[test]
fn she_jungles_a_camp_alone_charm_mirror_step_mend_and_clone_shots() {
    // a far camp: Mirror Step carries her onto it
    let mut w = ai(vec![camp_monster(30, 470_000, 500_000)]);
    w.f.hp = 500; // hurt: Mend on herself between camps
    let mut steps = false;
    for _ in 0..600 {
        w.tick(None);
        if w.core.ai.reason.starts_with("Jungle: Mirror Step") {
            steps = true;
        }
        for u in w.f.units.iter_mut() {
            u.hp = u.max_hp;
        }
    }
    assert!(steps, "Mirror Step onto the far camp");
    assert!(w.seen(|e| matches!(e, Effect::Charm { target: ME, .. })), "Kindred Charm on herself at the camp");
    assert!(w.seen(|e| matches!(e, Effect::Heal { target: ME, .. })), "Moonlit Mend on herself");
    assert!(w.seen(|e| matches!(e, Effect::CloneShot { target: 30, .. })), "her clones shoot the camp");
}

#[test]
fn she_never_jungles_with_an_enemy_champion_near() {
    let mut w = ai(vec![camp_monster(30, 420_000, 500_000), champ(FOE, 2, 440_000, 520_000)]);
    w.run(200);
    assert!(!w.seen(|e| matches!(e, Effect::Charm { target: ME, .. })), "{}", w.core.ai.reason);
}
