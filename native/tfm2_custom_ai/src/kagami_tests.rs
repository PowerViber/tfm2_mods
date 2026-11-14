//! Kagami's core: the clone (Reflection), the swap lock-on, Mirror Step, Guardian Angel, Mirror Swap, Last Reflection
//! and her AI's choices; then the bridge on the glue mock host.
use crate::kagami_core::*;
use crate::kagami_data as d;

const ME: usize = 1;
const CLONE: usize = 99;
const ALLY: usize = 2;
const FOE: usize = 10;

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

/// A tiny world: her, her clone and the units, with her effects applied the way the bridge and the engine would.
struct World {
    core: Core,
    f: Frame,
    clone: Option<CloneView>,
    log: Vec<(u64, Effect)>,
    wards: Vec<(u64, usize)>,
    self_wards: Vec<u64>,
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
            clone: None,
            log: vec![],
            wards: vec![],
            self_wards: vec![],
        }
    }
    /// one tick with an optional order
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.clone = self.clone;
        self.f.order = order;
        let out = self.core.step(&self.f);
        let tick = self.f.tick;
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::SpawnClone { at, hp } => {
                    self.clone = Some(CloneView {
                        id: CLONE,
                        pos: *at,
                        hp: *hp,
                        max_hp: *hp,
                    })
                }
                Effect::CloneMove { to, .. } => {
                    if let Some(c) = self.clone.as_mut() {
                        c.pos = *to;
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
                Effect::SelfWard { .. } => self.self_wards.push(tick),
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
        } else if id == CLONE {
            self.clone.unwrap().pos
        } else {
            self.f.units.iter().find(|u| u.id == id).unwrap().pos
        }
    }
    fn set_pos(&mut self, id: usize, p: Point) {
        if id == ME {
            self.f.pos = p;
        } else if id == CLONE {
            self.clone.as_mut().unwrap().pos = p;
        } else {
            self.f.units.iter_mut().find(|u| u.id == id).unwrap().pos = p;
        }
    }
    fn unit(&mut self, id: usize) -> &mut Unit {
        self.f.units.iter_mut().find(|u| u.id == id).unwrap()
    }
    /// a world with her clone already out (spawned on the first tick), placed at `at`
    fn with_clone(units: Vec<Unit>, at: Point) -> World {
        let mut w = World::new(units);
        w.tick(None);
        assert!(w.clone.is_some(), "the clone comes out at once");
        w.clone.as_mut().unwrap().pos = at;
        w.core.clone_moved_at = u64::MAX / 2; // hold it still unless a test wants it moving
        w
    }
}

// ------------------------------------------------------------------ the clone

#[test]
fn the_clone_comes_out_at_once_with_35_percent_of_her_hp_and_never_casts_step_or_the_ult() {
    let mut w = World::new(vec![]);
    let out = w.tick(None);
    let hp = out
        .iter()
        .find_map(|e| if let Effect::SpawnClone { hp, .. } = e { Some(*hp) } else { None })
        .expect("spawned");
    assert_eq!(hp, 1000 * d::CLONE_HP_PCT / 100);
    // only one, even before the bridge reports it
    w.clone = None;
    w.core.clone = None;
    let again = w.tick(None);
    assert!(
        !again.iter().any(|e| matches!(e, Effect::SpawnClone { .. })),
        "no second clone while the first is on its way"
    );
    // the clone never makes swaps or seals on its own
    let mut w = World::with_clone(vec![champ(ALLY, 1, 420_000, 500_000)], Point::new(380_000, 500_000));
    w.f.ult_unlocked = false; // her own ult would (rightly) seal the dropping ally
    w.unit(ALLY).hp = 50;
    w.run(120);
    assert!(!w.log.iter().any(|(_, e)| matches!(e, Effect::SwapLock { .. } | Effect::Seal { .. })));
}

#[test]
fn the_clone_follows_her_and_comes_back_15_s_after_dying() {
    let mut w = World::new(vec![]);
    w.tick(None);
    w.clone.as_mut().unwrap().pos = Point::new(400_000, 700_000); // 200,000 away: past the clone range
    w.run(240);
    let c = w.clone.unwrap();
    assert!(c.pos.within(w.f.pos, d::CLONE_RANGE), "it walks back within range: {:?}", c.pos);
    // dead
    w.clone = None;
    w.tick(None);
    let died = w.f.tick - 1;
    let mut back = None;
    for _ in 0..(d::CLONE_RESPAWN_TICKS as u64 + 10) {
        let tick = w.f.tick;
        if w.tick(None).iter().any(|e| matches!(e, Effect::SpawnClone { .. })) {
            back = Some(tick);
            break;
        }
    }
    assert_eq!(back, Some(died + d::CLONE_RESPAWN_TICKS as u64));
}

#[test]
fn the_clone_shoots_for_40_percent_never_at_a_tower_and_champions_first() {
    let mut w = World::with_clone(
        vec![
            tower(50, 2, 380_000, 520_000),
            champ(FOE, 2, 380_000, 540_000),
            Unit {
                champion: false,
                ..champ(30, 2, 375_000, 500_000)
            },
        ],
        Point::new(380_000, 500_000),
    );
    let out = w.tick(None);
    let shots: Vec<(usize, i64)> = out
        .iter()
        .filter_map(|e| {
            if let Effect::CloneShot { target, amount, .. } = e {
                Some((*target, *amount))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(shots, vec![(FOE, 100 * d::CLONE_DAMAGE_PCT / 100)]);
    // the next shot waits for its attack time
    let soon = w.tick(None);
    assert!(!soon.iter().any(|e| matches!(e, Effect::CloneShot { .. })));
    // with only the tower in range: nothing
    let mut w = World::with_clone(vec![tower(50, 2, 380_000, 520_000)], Point::new(380_000, 500_000));
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::CloneShot { .. })));
}

#[test]
fn the_clone_casts_its_own_mend_at_half_and_its_own_charm_each_on_its_own_cooldown() {
    let mut w = World::with_clone(
        vec![champ(ALLY, 1, 330_000, 500_000), champ(FOE, 2, 300_000, 500_000)],
        Point::new(340_000, 500_000),
    );
    w.unit(ALLY).hp = 300;
    let out = w.tick(None);
    let heal = out
        .iter()
        .find_map(|e| {
            if let Effect::Heal { target, amount, from } = e {
                Some((*target, *amount, *from))
            } else {
                None
            }
        })
        .expect("clone mend");
    assert_eq!(heal.0, ALLY);
    assert_eq!(heal.1, (d::MEND_BASE + d::MEND_AD_RATIO * 100 / 100) * d::CLONE_MEND_PCT / 100);
    assert_eq!(heal.2, Point::new(340_000, 500_000), "cast from the clone");
    assert!(
        out.iter().any(|e| matches!(e, Effect::Charm { target: ALLY, .. })),
        "the fighting ally is charmed: {out:?}"
    );
    // her own skills are untouched by the clone's
    assert_eq!(w.core.mend_at, 0);
    assert_eq!(w.core.charm_at, 0);
    assert_eq!(w.core.clone_mend_at, 1 + d::CLONE_MEND_COOLDOWN_TICKS as u64);
    assert_eq!(w.core.clone_charm_at, 1 + d::CLONE_CHARM_COOLDOWN_TICKS as u64);
}

// ------------------------------------------------------------------ swaps

#[test]
fn mirror_step_on_a_teammate_locks_on_for_half_a_second_then_trades_their_current_places_and_stuns_there() {
    let clone_at = Point::new(380_000, 500_000);
    let mut w = World::with_clone(
        vec![
            champ(ALLY, 1, 470_000, 500_000),
            champ(FOE, 2, 480_000, 510_000),
            champ(11, 2, 600_000, 500_000),
        ],
        clone_at,
    );
    let out = w.tick(Some(Order::StepAlly(ALLY)));
    assert!(out
        .iter()
        .any(|e| matches!(e, Effect::SwapLock { a: CLONE, b: ALLY, ticks } if *ticks == d::SWAP_LOCK_TICKS as u64)));
    assert!(!out.iter().any(|e| matches!(e, Effect::Swap { .. })), "not at once");
    // nobody is rooted: the ally walks on, past the swap's range, during the lock
    for _ in 0..(d::SWAP_LOCK_TICKS - 1) {
        let p = w.pos_of(ALLY);
        w.set_pos(ALLY, Point::new(p.x + 3_000, p.y));
        assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Swap { .. })));
    }
    let ally_at = Point::new(w.pos_of(ALLY).x + 3_000, 500_000);
    w.set_pos(ALLY, ally_at);
    assert!(!ally_at.within(clone_at, d::STEP_RANGE), "the ally walked out of range: {ally_at:?}");
    let out = w.tick(None);
    assert!(out.iter().any(|e| matches!(e, Effect::Swap { a: CLONE, b: ALLY })), "still swapped: {out:?}");
    assert_eq!(w.pos_of(ALLY), clone_at, "the ally lands at the clone's spot");
    assert_eq!(w.clone.unwrap().pos, ally_at, "the clone lands where the ally stood");
    // the stun is around the clone's new spot only
    let stunned: Vec<usize> = out
        .iter()
        .filter_map(|e| {
            if let Effect::Stun { target, ticks } = e {
                assert_eq!(*ticks, d::SWAP_STUN_TICKS as u64);
                Some(*target)
            } else {
                None
            }
        })
        .collect();
    let foe_close = w.f.units.iter().find(|u| u.id == FOE).unwrap().pos.within(ally_at, d::SWAP_STUN_RADIUS);
    assert_eq!(stunned.contains(&FOE), foe_close);
    assert!(stunned.contains(&11) == Point::new(600_000, 500_000).within(ally_at, d::SWAP_STUN_RADIUS));
    assert!(!stunned.is_empty(), "someone was near the danger spot");
}

#[test]
fn mirror_step_on_a_point_dashes_the_clone_then_swaps_her_and_respects_range_and_cooldown() {
    let mut w = World::with_clone(vec![champ(FOE, 2, 400_000, 420_000)], Point::new(400_000, 480_000));
    let far = Point::new(400_000, 500_000 - d::STEP_RANGE - 1_000);
    w.tick(Some(Order::StepPoint(far)));
    assert_eq!(w.core.rejection, "step", "out of range");
    let p = Point::new(400_000, 410_000);
    let out = w.tick(Some(Order::StepPoint(p)));
    let dash = out
        .iter()
        .find_map(|e| if let Effect::CloneMove { to, ticks } = e { Some((*to, *ticks)) } else { None })
        .expect("the clone dashes");
    assert_eq!(dash.0, p);
    let start = w.pos_of(ME);
    w.run(dash.1 + d::SWAP_LOCK_TICKS as u64);
    assert_eq!(w.pos_of(ME), p, "she lands at the point");
    assert_eq!(w.clone.unwrap().pos, start, "the clone takes her old spot");
    // the stun goes off where the clone lands (her old spot): the foe 80,000 away is out of it
    assert!(!w.log.iter().any(|(_, e)| matches!(e, Effect::Stun { target: FOE, .. })));
    // cooldown
    w.tick(Some(Order::StepPoint(Point::new(400_000, 450_000))));
    assert_eq!(w.core.rejection, "step");
}

#[test]
fn a_swap_is_cancelled_with_half_its_cooldown_back_if_a_side_dies_during_the_lock() {
    let mut w = World::with_clone(vec![champ(ALLY, 1, 470_000, 500_000)], Point::new(380_000, 500_000));
    w.tick(Some(Order::StepAlly(ALLY)));
    let full = w.core.step_at;
    w.run(10);
    w.unit(ALLY).hp = 0;
    w.tick(None);
    assert!(w.core.swap.is_none());
    assert!(!w.log.iter().any(|(_, e)| matches!(e, Effect::Swap { .. })));
    assert_eq!(w.core.step_at, full - d::STEP_COOLDOWN_TICKS as u64 * d::SWAP_REFUND_PCT as u64 / 100);
    // the clone dying cancels too
    let mut w = World::with_clone(vec![champ(ALLY, 1, 470_000, 500_000)], Point::new(380_000, 500_000));
    w.tick(Some(Order::StepAlly(ALLY)));
    w.clone = None;
    w.tick(None);
    assert!(w.core.swap.is_none());
}

// ------------------------------------------------------------------ Guardian Angel and Mirror Swap

#[test]
fn guardian_angel_knocks_back_and_shields_when_she_crosses_half_hp_on_a_30_s_cooldown() {
    let mut w = World::with_clone(
        vec![champ(FOE, 2, 420_000, 500_000), champ(11, 2, 600_000, 500_000), tower(50, 2, 410_000, 500_000)],
        Point::new(300_000, 500_000),
    );
    w.f.hp = 600;
    w.tick(None);
    w.f.hp = 450;
    let out = w.tick(None);
    let knocked: Vec<usize> = out
        .iter()
        .filter_map(|e| if let Effect::Knockback { target } = e { Some(*target) } else { None })
        .collect();
    assert_eq!(knocked, vec![FOE], "only the enemy champion within 50,000; never a tower");
    assert!(out.iter().any(|e| matches!(e, Effect::Shield { target: ME, amount, ticks } if *amount == 1000 * d::ANGEL_SHIELD_PCT_MAX_HP / 100 && *ticks == d::ANGEL_SHIELD_TICKS as u64)));
    // back above and down again: cooldown
    w.f.hp = 700;
    w.tick(None);
    w.f.hp = 400;
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Knockback { .. })));
    // staying low doesn't re-trigger after the cooldown either; crossing does
    w.f.tick += d::ANGEL_COOLDOWN_TICKS as u64;
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Knockback { .. })));
    w.f.hp = 700;
    w.tick(None);
    w.f.hp = 400;
    assert!(w.tick(None).iter().any(|e| matches!(e, Effect::Knockback { .. })));
}

#[test]
fn mirror_swap_holds_her_swaps_with_the_clone_seals_her_then_revives_her_at_40_percent() {
    let clone_at = Point::new(300_000, 500_000);
    let mut w = World::with_clone(vec![champ(FOE, 2, 410_000, 500_000)], clone_at);
    w.tick(None);
    assert!(w.self_wards.contains(&(w.f.tick - 1)), "warded while ready with a clone");
    let old = w.f.pos;
    w.f.hp = 1; // held by the ward
    let trigger = w.f.tick;
    let out = w.tick(None);
    assert!(out.iter().any(|e| matches!(e, Effect::SwapLock { a: CLONE, b: ME, .. })));
    // held at 1 HP for the whole lock
    for _ in 0..(d::SWAP_LOCK_TICKS - 1) {
        let t = w.f.tick;
        w.tick(None);
        assert!(w.self_wards.contains(&t));
    }
    let out = w.tick(None);
    assert!(out.iter().any(|e| matches!(e, Effect::Swap { a: CLONE, b: ME })));
    assert_eq!(w.f.pos, clone_at);
    assert_eq!(w.clone.unwrap().pos, old);
    assert!(
        out.iter().any(|e| matches!(e, Effect::Stun { target: FOE, .. })),
        "the stun goes off at her old spot"
    );
    assert!(out
        .iter()
        .any(|e| matches!(e, Effect::Seal { target: ME, ticks } if *ticks == d::SELF_SAVE_SEAL_TICKS as u64)));
    w.run(d::SELF_SAVE_SEAL_TICKS as u64 + 1);
    assert_eq!(w.f.hp, 1000 * d::SELF_SAVE_REVIVE_PCT / 100);
    assert!(w.log.iter().any(|(_, e)| matches!(
        e,
        Effect::Revive {
            target: ME,
            blessed: false,
            ..
        }
    )));
    assert_eq!(w.core.self_save_at, trigger + d::SELF_SAVE_COOLDOWN_TICKS as u64, "90 s from the trigger");
    // on cooldown: no ward
    let t = w.f.tick;
    w.tick(None);
    assert!(!w.self_wards.contains(&t));
}

#[test]
fn mirror_swap_needs_a_live_clone() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.core.clone_back_at = 10_000; // no clone yet
    w.tick(None);
    assert!(w.self_wards.is_empty(), "no ward without a clone");
    w.f.hp = 50;
    let out = w.tick(None);
    assert!(!out.iter().any(|e| matches!(e, Effect::SwapLock { .. })));
}

// ------------------------------------------------------------------ Last Reflection

#[test]
fn last_reflection_seals_the_first_teammate_who_drops_under_the_line_then_revives_them_full_and_blessed() {
    let mut w = World::with_clone(
        vec![champ(ALLY, 1, 700_000, 200_000), champ(3, 1, 100_000, 100_000), champ(FOE, 2, 700_000, 210_000)],
        Point::new(380_000, 500_000),
    );
    w.tick(None);
    let t = w.f.tick - 1;
    assert!(
        w.wards.contains(&(t, ALLY)) && w.wards.contains(&(t, 3)),
        "every teammate is warded, anywhere on the map"
    );
    assert!(!w.wards.iter().any(|x| x.1 == ME), "never her");
    w.unit(ALLY).hp = 1; // the ward held a lethal hit
    let out = w.tick(None);
    assert!(out
        .iter()
        .any(|e| matches!(e, Effect::Seal { target: ALLY, ticks } if *ticks == d::ULT_SEAL_TICKS as u64)));
    assert!(w.f.units.iter().find(|u| u.id == ALLY).unwrap().sealed && !w.f.units.iter().find(|u| u.id == ALLY).unwrap().targetable);
    // one ally per use: a second one dropping is not saved, and the ward is gone from the rest
    let t = w.f.tick;
    w.unit(3).hp = 50;
    let out = w.tick(None);
    assert!(!out.iter().any(|e| matches!(e, Effect::Seal { target: 3, .. })));
    assert!(!w.wards.contains(&(t, 3)), "no ward while on cooldown");
    assert!(w.wards.contains(&(t, ALLY)), "the sealed ally stays held");
    w.run(d::ULT_SEAL_TICKS as u64);
    assert!(w.log.iter().any(|(_, e)| matches!(
        e,
        Effect::Revive {
            target: ALLY,
            hp: 1000,
            blessed: true
        }
    )));
    assert_eq!(w.f.units.iter().find(|u| u.id == ALLY).unwrap().hp, 1000);
}

#[test]
fn last_reflection_needs_the_ult_learned_and_a_drop() {
    let mut w = World::with_clone(vec![champ(ALLY, 1, 700_000, 200_000)], Point::new(380_000, 500_000));
    w.f.ult_unlocked = false;
    w.wards.clear();
    w.unit(ALLY).hp = 1;
    w.tick(None);
    w.tick(None);
    assert!(w.wards.is_empty() && !w.log.iter().any(|(_, e)| matches!(e, Effect::Seal { .. })));
    // already low when it comes up: no seal until they actually take another hit
    w.f.ult_unlocked = true;
    w.unit(ALLY).hp = 50;
    w.tick(None);
    w.tick(None);
    assert!(!w.log.iter().any(|(_, e)| matches!(e, Effect::Seal { .. })));
    w.unit(ALLY).hp = 20;
    w.tick(None);
    assert!(w.log.iter().any(|(_, e)| matches!(e, Effect::Seal { target: ALLY, .. })));
}

// ------------------------------------------------------------------ AI

fn ai(units: Vec<Unit>, clone_at: Point) -> World {
    let mut w = World::with_clone(units, clone_at);
    w.f.manual = false;
    w
}

#[test]
fn the_ai_heals_the_lowest_teammate_first() {
    let mut w = ai(
        vec![champ(ALLY, 1, 430_000, 500_000), champ(3, 1, 420_000, 520_000)],
        Point::new(380_000, 500_000),
    );
    w.unit(ALLY).hp = 500;
    w.unit(3).hp = 300;
    w.core.clone_mend_at = u64::MAX;
    w.run(16);
    w.tick(None);
    let out: Vec<_> = w.log.iter().map(|(_, e)| e.clone()).collect();
    assert!(
        out.iter()
            .any(|e| matches!(e, Effect::Heal { target: 3, from, .. } if *from == Point::new(400_000, 500_000))),
        "{out:?}"
    );
}

#[test]
fn the_ai_rescues_a_dived_teammate_with_mirror_step_leaving_the_clone_in_the_danger() {
    let mut w = ai(
        vec![champ(ALLY, 1, 480_000, 500_000), champ(FOE, 2, 490_000, 500_000)],
        Point::new(380_000, 500_000),
    );
    w.unit(ALLY).hp = 250;
    w.core.mend_at = u64::MAX; // heal on cooldown
    w.run(16);
    w.tick(None);
    assert!(w.core.swap.is_some_and(|s| s.kind == SwapKind::StepAlly && s.other == ALLY));
    w.run(d::SWAP_LOCK_TICKS as u64);
    assert_eq!(w.pos_of(ALLY), Point::new(380_000, 500_000));
    assert!(w.log.iter().any(|(_, e)| matches!(e, Effect::Stun { target: FOE, .. })));
}

#[test]
fn the_ai_never_swaps_a_teammate_into_a_worse_spot() {
    // the clone stands in a bigger fight than the ally's
    let mut w = ai(
        vec![
            champ(ALLY, 1, 480_000, 500_000),
            champ(FOE, 2, 495_000, 500_000),
            champ(11, 2, 385_000, 500_000),
            champ(12, 2, 380_000, 510_000),
        ],
        Point::new(380_000, 500_000),
    );
    w.unit(ALLY).hp = 250;
    w.core.mend_at = u64::MAX;
    w.tick(None);
    assert!(!w.core.swap.is_some_and(|s| s.other == ALLY));
}

#[test]
fn the_ai_escapes_through_the_clone_when_dived() {
    let mut w = ai(vec![champ(FOE, 2, 410_000, 500_000)], Point::new(400_000, 600_000));
    w.core.mend_at = u64::MAX;
    w.core.angel_at = u64::MAX;
    w.f.hp = 900;
    w.tick(None);
    w.f.hp = 400;
    w.run(16);
    w.tick(None);
    assert!(w.core.swap.is_some_and(|s| s.kind == SwapKind::StepPoint), "{:?}", w.core.swap);
    w.run(d::SWAP_LOCK_TICKS as u64 + 2);
    assert_eq!(w.f.pos, Point::new(400_000, 600_000), "she is where the clone was");
}

#[test]
fn the_ai_charms_the_strongest_teammate_who_is_fighting() {
    let mut a = champ(ALLY, 1, 430_000, 500_000);
    a.attack = 200;
    let mut w = ai(
        vec![a, champ(3, 1, 420_000, 520_000), champ(FOE, 2, 470_000, 500_000)],
        Point::new(380_000, 500_000),
    );
    w.core.clone_charm_at = u64::MAX;
    w.run(16);
    w.tick(None);
    let out: Vec<_> = w.log.iter().map(|(_, e)| e.clone()).collect();
    assert!(
        out.iter()
            .any(|e| matches!(e, Effect::Charm { target: ALLY, ticks } if *ticks == d::CHARM_TICKS as u64)),
        "{out:?}"
    );
}

#[test]
fn the_ai_walks_to_the_teammate_she_guards() {
    let mut w = ai(vec![champ(ALLY, 1, 520_000, 500_000)], Point::new(380_000, 500_000));
    w.run(16);
    w.tick(None);
    let out: Vec<_> = w.log.iter().map(|(_, e)| e.clone()).collect();
    let to = out
        .iter()
        .find_map(|e| if let Effect::Steer { to } = e { Some(*to) } else { None })
        .expect("steer");
    assert!(to.within(Point::new(520_000, 500_000), d::AI_GUARD_RANGE));
    assert!(to.x < 520_000, "behind them, on her side");
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
        let clone = k.core.clone.unwrap();
        k.core.swap = Some(PendingSwap {
            kind: SwapKind::StepAlly,
            other: 1,
            clone,
            started: 0,
            at: 30,
        });
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
        k.core.swap = None;
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
        let clone = k.core.clone.unwrap();
        k.core.swap = Some(PendingSwap {
            kind: SwapKind::StepPoint,
            other: 0,
            clone,
            started: 0,
            at: 30,
        });
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
        let clone = new.core.clone.unwrap();
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
        assert_eq!(f.clone.map(|c| c.id), Some(2));
        assert!(f.units.iter().all(|u| u.id != 2), "the clone isn't one of the units");
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
                    Effect::Knockback { target: 2 },
                    Effect::CloneShot {
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
        assert_eq!(g.knockbacks, vec![(0, 2, d::ANGEL_KNOCK_SPEED as usize, d::ANGEL_KNOCK_TICKS as usize)]);
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
