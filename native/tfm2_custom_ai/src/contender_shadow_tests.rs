//! Contender 2.0's core: shadow charges and recasts, Shadow Pact's echoes and their cap, Shadow Rhythm's stacks, the
//! projection (relay, swap, S2/S3 from the shadow), Shadow Cyclone, Slam & Uppercut, the Shadow Flurry and his AI.
use crate::contender_shadow_core::*;
use crate::contender_shadow_data as d;

const ME: usize = 1;
const ALLY: usize = 2;
const FOE: usize = 10;
const SHADOW: usize = 99;

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, visible: true, targetable: true, farmable: false }
}
fn creep(id: usize, x: i64, y: i64, hp: i64) -> Unit {
    Unit { champion: false, farmable: true, hp, max_hp: hp, team: 9, ..champ(id, 9, x, y) }
}

/// A tiny world: him, his projected shadow and the units, with his effects applied the way the bridge and engine would.
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
    shadow: Option<ShadowView>,
    /// damage the relay sent to his body
    relayed: i64,
}
impl World {
    fn new(units: Vec<Unit>) -> World {
        World {
            core: Core::default(),
            f: Frame {
                tick: 1,
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
                terrain: vec![false; (CELLS * CELLS) as usize],
                profile: [50; 13],
                manual: true,
                ..Frame::default()
            },
            log: vec![],
            shadow: None,
            relayed: 0,
        }
    }
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.order = order;
        self.f.shadow = self.shadow;
        let out = self.core.step(&self.f);
        self.f.basic_hits.clear();
        let tick = self.f.tick;
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::Damage { target, amount } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = (u.hp - amount).max(1);
                    }
                }
                Effect::Knock { target, to, .. } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::Glide { to, .. } | Effect::Teleport { to } => self.f.pos = *to,
                Effect::SpawnShadow { at, hp, .. } => self.shadow = Some(ShadowView { id: SHADOW, pos: *at, hp: *hp, max_hp: *hp }),
                Effect::MoveShadow { to, .. } => {
                    if let Some(s) = self.shadow.as_mut() {
                        s.pos = *to;
                    }
                }
                Effect::DespawnShadow => self.shadow = None,
                Effect::SelfDamage { amount } => {
                    self.relayed += amount;
                    self.f.hp -= amount;
                }
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
    fn unit(&mut self, id: usize) -> &mut Unit {
        self.f.units.iter_mut().find(|u| u.id == id).unwrap()
    }
    fn seen<F: Fn(&Effect) -> bool>(&self, f: F) -> bool {
        self.log.iter().any(|(_, e)| f(e))
    }
    fn count<F: Fn(&Effect) -> bool>(&self, f: F) -> usize {
        self.log.iter().filter(|(_, e)| f(e)).count()
    }
    fn hits_on(&self, id: usize) -> usize {
        self.count(|e| matches!(e, Effect::Damage { target, .. } if *target == id))
    }
}

// ------------------------------------------------------------------ charges and recasts

#[test]
fn five_charges_recharge_and_basic_attacks_speed_them_up() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    assert_eq!(w.core.charges, 5);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert_eq!(w.core.charges, 4, "the recast spends a charge");
    let due = w.core.charge_at;
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.charge_at, due - d::CHARGES_BASIC_CUT_TICKS as u64, "a basic attack takes 0.5 s off");
    w.run(d::CHARGES_RECHARGE_TICKS as u64);
    assert_eq!(w.core.charges, 5);
}

#[test]
fn a_recast_needs_its_window_and_a_charge_and_comes_once() {
    let mut w = World::new(vec![creep(30, 410_000, 500_000, 5_000)]);
    w.core.charges = 0;
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert_eq!(w.core.rejection, "s3 recast: no charge");
    let mut w = World::new(vec![creep(30, 410_000, 500_000, 5_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    w.run(d::CHARGES_RECAST_WINDOW_TICKS as u64 + 1);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert_eq!(w.core.rejection, "s3", "the window closed: back on cooldown");
    assert_eq!(w.core.charges, 5);
}

#[test]
fn every_hit_on_anything_takes_a_second_off_the_ult() {
    let mut w = World::new(vec![creep(30, 410_000, 500_000, 5_000)]);
    w.core.ult_at = 10_000;
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert_eq!(w.core.ult_at, 10_000 - d::ULT_CUT_PER_HIT_TICKS as u64, "a creep hit counts");
    w.f.basic_hits.push(30);
    w.tick(None);
    assert_eq!(w.core.ult_at, 10_000 - 2 * d::ULT_CUT_PER_HIT_TICKS as u64);
}

// ------------------------------------------------------------------ Shadow Pact

#[test]
fn every_hit_echoes_a_second_later_at_40_percent_with_half_its_crowd_control() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    let slam = pct(d::SLAM_BASE, d::SLAM_AD_RATIO, 100);
    w.run(d::PACT_DELAY_TICKS as u64);
    let echo = w.log.iter().find(|(t, e)| *t > 1 && matches!(e, Effect::Damage { target: FOE, .. }));
    let (at, e) = echo.expect("the echo landed");
    assert_eq!(*at, 1 + d::PACT_DELAY_TICKS as u64);
    let Effect::Damage { amount, .. } = e else { unreachable!() };
    // the slam's stack makes the echo hit a little harder (vulnerability)
    assert!((*amount - slam * d::PACT_PCT / 100).abs() <= slam * d::PACT_PCT / 100 * 2 / 100 + 1, "{amount}");
    assert!(w.seen(|e| matches!(e, Effect::Stun { target: FOE, ticks } if *ticks == d::SLAM_STUN_TICKS as u64 * d::PACT_CC_PCT as u64 / 100)));
}
fn pct(base: i64, ratio: i64, ad: i64) -> i64 {
    base + ratio * ad / 100
}

#[test]
fn recast_hits_echo_twice_and_echoed_crowd_control_is_capped() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    w.run(3 * d::PACT_DELAY_TICKS as u64);
    // slam + its echo, uppercut + its two echoes
    assert_eq!(w.hits_on(FOE), 5);
    let echoed: u64 = w
        .log
        .iter()
        .filter(|(t, _)| *t > 2)
        .filter_map(|(_, e)| match e {
            Effect::Stun { target: FOE, ticks } | Effect::KnockUp { target: FOE, ticks } => Some(*ticks),
            _ => None,
        })
        .sum();
    assert!(echoed <= d::PACT_CC_CAP_TICKS as u64, "echoes add at most 1.5 s of control: {echoed}");
}

// ------------------------------------------------------------------ Shadow Rhythm

#[test]
fn stacks_buff_him_from_the_most_stacked_enemy_and_make_it_take_more() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    for _ in 0..4 {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 4);
    assert_eq!(w.core.vuln_permille(FOE, w.f.tick), 4 * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK);
    assert!(w.seen(|e| matches!(e, Effect::Rhythm { attack_speed, .. } if *attack_speed == 4 * d::RHYTHM_ATTACK_SPEED_PCT_PER_STACK)));
    w.run(d::RHYTHM_STACK_TICKS as u64 + d::PACT_DELAY_TICKS as u64 + 2);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0, "they fade after 4 s");
}

#[test]
fn a_max_stacked_creep_carries_four_stacks_onto_a_champion_and_a_kill_lingers() {
    let mut w = World::new(vec![creep(30, 405_000, 500_000, 50_000), champ(FOE, 2, 410_000, 500_000)]);
    for _ in 0..d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(30);
        w.tick(None);
    }
    assert_eq!(w.core.stacks_on(30, w.f.tick), d::RHYTHM_MAX_STACKS);
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::RHYTHM_CARRY_STACKS);
    // the stacked creep dies: his buffs stay up
    w.core.stacks.retain(|s| s.id != FOE);
    w.tick(None);
    w.f.units.retain(|u| u.id != 30);
    w.tick(None);
    assert!(w.core.linger.is_some_and(|l| l.0 == d::RHYTHM_MAX_STACKS));
}

// ------------------------------------------------------------------ S1 Shadow Projection

#[test]
fn projection_holds_his_body_and_relays_70_percent_of_the_shadows_damage() {
    let mut w = World::new(vec![champ(FOE, 2, 480_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(480_000, 500_000) }));
    assert!(w.core.projecting());
    assert!(w.seen(|e| matches!(e, Effect::Hold { .. })), "his body is held");
    w.run(20);
    let s = w.shadow.unwrap();
    assert!(s.pos.x > 450_000, "launched toward the target: {:?}", s.pos);
    w.shadow.as_mut().unwrap().hp -= 200;
    w.tick(None);
    assert_eq!(w.relayed, 140);
    // it falls: the rest comes through
    w.shadow = None;
    w.tick(None);
    assert_eq!(w.relayed, 1000 * d::PROJECTION_RELAY_PCT / 100, "a fallen shadow costs 70% of his max HP");
    assert!(!w.core.projecting());
}

#[test]
fn projection_reaches_at_most_100000_and_ends_after_5_s() {
    let mut w = World::new(vec![]);
    w.tick(Some(Order::S1 { aim: Point::new(800_000, 500_000) }));
    w.run(40);
    assert!(w.shadow.unwrap().pos.x <= 400_000 + d::PROJECTION_RANGE + 1);
    w.run(d::PROJECTION_TICKS as u64);
    assert!(!w.core.projecting() && w.shadow.is_none());
}

#[test]
fn the_shadow_boxes_and_its_punches_echo_twice() {
    let mut w = World::new(vec![champ(FOE, 2, 470_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(465_000, 500_000) }));
    w.run(40 + 2 * d::PACT_DELAY_TICKS as u64);
    assert!(w.hits_on(FOE) >= 3, "a shadow punch and its two echoes: {}", w.hits_on(FOE));
    assert_eq!(w.f.pos, Point::new(400_000, 500_000), "his body never moved");
}

#[test]
fn the_recast_swaps_his_body_to_the_shadow_and_its_cross_knocks_back() {
    let mut w = World::new(vec![champ(FOE, 2, 412_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(480_000, 500_000) }));
    w.run(20);
    let at = w.shadow.unwrap().pos;
    w.tick(Some(Order::S1 { aim: at }));
    assert_eq!(w.f.pos, at, "his body swapped to the shadow");
    assert!(w.seen(|e| matches!(e, Effect::Knock { target: FOE, .. })), "the cross from where he stood");
    assert_eq!(w.core.charges, 4);
}

#[test]
fn s2_and_s3_come_from_the_projected_shadow_and_the_uppercut_launches_toward_his_body() {
    let mut w = World::new(vec![champ(FOE, 2, 485_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(480_000, 500_000) }));
    w.run(20);
    w.tick(Some(Order::S3 { aim: Point::new(485_000, 500_000) }));
    assert!(w.core.rejection.is_empty(), "far from his body, in reach of the shadow: {}", w.core.rejection);
    w.tick(Some(Order::S3 { aim: Point::new(485_000, 500_000) }));
    let to = w.log.iter().rev().find_map(|(_, e)| if let Effect::Knock { target: FOE, to, .. } = e { Some(*to) } else { None }).expect("launched");
    assert!(to.x < 485_000 && to.x >= 400_000 + 12_000 - 1, "toward his body, not past it: {to:?}");
    // contender-2.1: S2 from the shadow primes nothing (the projection itself makes the flurry echo)
    let mut w = World::new(vec![champ(FOE, 2, 485_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(480_000, 500_000) }));
    w.run(20);
    w.tick(Some(Order::S2 { aim: Point::new(485_000, 500_000) }));
    assert!(w.core.flurry.is_none());
}

#[test]
fn a_flurry_cast_while_projected_echoes_every_punch_and_starts_at_the_shadow() {
    // the target is far from his body but by the shadow
    let mut w = World::new(vec![champ(FOE, 2, 490_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::S1 { aim: Point::new(480_000, 500_000) }));
    w.run(20);
    w.tick(Some(Order::Ult));
    assert!(w.core.flurry.is_some_and(|fl| fl.echo), "cast while projected: an echoing flurry ({})", w.core.rejection);
    assert!(!w.core.projecting(), "the projection ends as he vanishes");
    w.run(d::ULT_TICKS as u64 + d::PACT_DELAY_TICKS as u64 + 2);
    let punches = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry"));
    let echoes = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo"));
    assert!(echoes >= punches, "every punch echoes: {punches} punches, {echoes} echoes");
    let back = w.log.iter().rev().find_map(|(_, e)| if let Effect::Teleport { to } = e { Some(*to) } else { None }).expect("he reappears");
    assert!(back.within(Point::new(490_000, 500_000), 15_000), "by the last enemy hit, at the shadow's fight: {back:?}");
}

// ------------------------------------------------------------------ S2 Shadow Cyclone

#[test]
fn the_clone_cleaves_four_times_and_the_recast_cleaves_seven_invulnerable_with_a_knockup() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000), creep(30, 455_000, 505_000, 9_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    w.run(d::CYCLONE_HITS as u64 * d::CYCLONE_EVERY_TICKS as u64 + 1);
    assert_eq!(w.count(|e| matches!(e, Effect::Slow { target: FOE, .. })), d::CYCLONE_HITS as usize, "4 cleave hits on the champion");
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: 30, .. })), "creeps too");
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    assert!(w.seen(|e| matches!(e, Effect::Invulnerable { .. })));
    assert!(w.f.pos.within(Point::new(450_000, 500_000), 1_000), "he went there");
    w.run(8 + d::CYCLONE_RECAST_HITS as u64 * d::CYCLONE_RECAST_EVERY_TICKS as u64 + 2);
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::CYCLONE_RECAST_KNOCK_TICKS as u64)), "the last one knocks up");
}

// ------------------------------------------------------------------ S3 Slam & Uppercut

#[test]
fn the_slam_stuns_and_rings_and_never_takes_a_tower() {
    let mut tower = champ(50, 2, 410_000, 500_000);
    tower.champion = false;
    tower.tower = true;
    let mut w = World::new(vec![tower, champ(FOE, 2, 412_000, 500_000), champ(11, 2, 420_000, 505_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert!(w.seen(|e| matches!(e, Effect::Stun { target: FOE, .. })));
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: 11, .. })), "the impact ring");
    assert!(!w.seen(|e| matches!(e, Effect::Damage { target: 50, .. })));
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::SLAM_KNOCK_TICKS as u64)), "the uppercut");
}

// ------------------------------------------------------------------ Ult Shadow Flurry

fn flurry_targets(w: &World) -> Vec<usize> {
    // Ground-cast fixtures have no echoes or unrelated attacks, so each damage
    // packet corresponds to exactly one accepted Flurry punch.
    w.log.iter().filter_map(|(_, e)| match e { Effect::Damage { target, .. } => Some(*target), _ => None }).collect()
}

#[test]
fn flurry_prefers_another_champion_instead_of_repeating_the_zero_distance_victim() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 450_000, 500_000)]);
    for u in &mut w.f.units { u.hp = 100_000; u.max_hp = 100_000; }
    w.tick(Some(Order::Ult)); w.run(d::ULT_TICKS as u64);
    let targets = flurry_targets(&w);
    assert_eq!(targets.len(), d::ULT_PUNCHES as usize);
    assert!(targets.windows(2).all(|pair| pair[0] != pair[1]), "{targets:?}");
    assert!(targets.contains(&FOE) && targets.contains(&11));
}

#[test]
fn flurry_switches_from_a_champion_to_an_eligible_creep_or_summon() {
    for farmable in [false, true] {
        let mut other = creep(30, 450_000, 500_000, 100_000);
        other.farmable = farmable; // a summon need not be a farming target
        let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), other]);
        w.unit(FOE).hp = 100_000; w.unit(FOE).max_hp = 100_000;
        w.tick(Some(Order::Ult)); w.run(d::ULT_TICKS as u64);
        let targets = flurry_targets(&w);
        assert_eq!(targets.len(), d::ULT_PUNCHES as usize);
        assert_eq!(&targets[..4], &[FOE, 30, FOE, 30]);
        assert!(targets.windows(2).all(|pair| pair[0] != pair[1]));
    }
}

#[test]
fn flurry_repeats_only_when_other_candidates_are_ineligible_or_out_of_chain_range() {
    let mut hidden = champ(12, 2, 431_000, 500_000); hidden.visible = false;
    let mut untargetable = champ(13, 2, 432_000, 500_000); untargetable.targetable = false;
    let mut dead = champ(14, 2, 433_000, 500_000); dead.hp = 0;
    let mut tower = champ(15, 2, 434_000, 500_000); tower.tower = true; tower.champion = false;
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(ALLY, 1, 431_000, 500_000), hidden, untargetable, dead, tower, champ(16, 2, 600_000, 500_000)]);
    w.unit(FOE).hp = 100_000; w.unit(FOE).max_hp = 100_000;
    w.tick(Some(Order::Ult)); w.run(d::ULT_TICKS as u64);
    let targets = flurry_targets(&w);
    assert_eq!(targets.len(), d::ULT_PUNCHES as usize);
    assert!(targets.iter().all(|id| *id == FOE), "{targets:?}");
}

#[test]
fn flurry_alternative_ties_are_deterministic_under_snapshot_reordering() {
    let mut units = vec![champ(FOE, 2, 430_000, 500_000), champ(12, 2, 450_000, 490_000), champ(11, 2, 450_000, 510_000)];
    for u in &mut units { u.hp = 100_000; u.max_hp = 100_000; }
    let mut a = World::new(units.clone()); units.reverse(); let mut b = World::new(units);
    for w in [&mut a, &mut b] { w.tick(Some(Order::Ult)); w.run(d::ULT_TICKS as u64); }
    let targets = flurry_targets(&a);
    assert_eq!(&targets[..2], &[FOE, 11]);
    assert_eq!(targets, flurry_targets(&b));
    assert!(targets.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn flurry_visuals_cover_eight_directions_without_changing_punch_count_or_cadence() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 100_000; w.unit(FOE).max_hp = 100_000;
    w.tick(Some(Order::Ult)); w.run(d::ULT_TICKS as u64);
    let cues: Vec<_> = w.log.iter().filter_map(|(tick,e)|match e {
        Effect::Fx { name, ticks, heading: Some(h), .. } if name=="ct_flurry" => Some((*tick,*ticks,*h)), _=>None }).collect();
    assert_eq!(cues.len(), d::ULT_PUNCHES as usize);
    let mut directions = std::collections::BTreeSet::new();
    for (i, (_, life, h)) in cues.iter().enumerate() {
        assert_eq!(*life,8); assert_eq!(*h,Core::flurry_heading(ME,i as i64)); directions.insert(*h);
        if i>0 { assert_eq!(cues[i].0-cues[i-1].0,Core::flurry_gap(i as i64-1)); }
    }
    assert_eq!(directions.len(),8);
    assert_eq!(flurry_targets(&w).len(),cues.len(),"one damage packet per punch");
}

#[test]
fn the_flurry_vanishes_him_chains_36_accelerating_punches_and_brings_him_back_by_the_last() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 450_000, 500_000), creep(30, 470_000, 500_000, 90_000)]);
    for u in w.f.units.iter_mut() {
        u.hp = 90_000;
        u.max_hp = 90_000;
    }
    w.core.charges = 1;
    w.tick(Some(Order::Ult));
    assert!(w.core.flurrying());
    assert_eq!(w.core.charges, 5, "his charges refill");
    w.run(d::ULT_TICKS as u64);
    assert!(!w.core.flurrying());
    let punches = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry"));
    assert_eq!(punches, d::ULT_PUNCHES as usize);
    assert!(Core::flurry_gap(0) > Core::flurry_gap(d::ULT_PUNCHES - 1), "slow at first, fast at the end");
    let ups = w.count(|e| matches!(e, Effect::KnockUp { ticks, .. } if *ticks == d::ULT_KNOCK_TICKS as u64));
    assert!(ups >= 3, "a knock-up every second: {ups}");
    assert!(w.seen(|e| matches!(e, Effect::Teleport { .. })), "he reappears");
    assert!(w.core.haymaker_until > w.f.tick);
}

#[test]
fn the_flurry_ends_early_when_nothing_is_left_in_reach_and_echoes_only_when_projected() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::Ult));
    w.run(30);
    w.unit(FOE).pos = Point::new(800_000, 100_000); // it got away
    w.run(30);
    assert!(!w.core.flurrying(), "nothing within 30,000 of the last hit: over");
    assert!(w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry")) < d::ULT_PUNCHES as usize);
    assert!(!w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo")), "not projected: no echoes");
}

// ------------------------------------------------------------------ AI

fn ai(units: Vec<Unit>) -> World {
    let mut w = World::new(units);
    w.f.manual = false;
    w
}

#[test]
fn the_ai_projects_onto_a_far_target_then_fights_from_the_shadow() {
    let mut w = ai(vec![champ(FOE, 2, 470_000, 500_000), champ(ALLY, 1, 410_000, 520_000)]);
    w.f.ult_unlocked = false;
    w.tick(None);
    assert!(w.core.projecting(), "{}", w.core.reason);
    w.run(60);
    assert!(w.hits_on(FOE) >= 2, "{}", w.core.reason);
}

#[test]
fn the_ai_lets_a_losing_projection_go() {
    let mut w = ai(vec![champ(FOE, 2, 470_000, 500_000), champ(11, 2, 475_000, 500_000), champ(12, 2, 480_000, 505_000)]);
    w.f.ult_unlocked = false;
    w.core.proj = Some(Projection { until: 1_000, pos: Point::new(470_000, 500_000), shadow_hp: 300, spawned: true, next_basic: 0 });
    w.shadow = Some(ShadowView { id: SHADOW, pos: Point::new(470_000, 500_000), hp: 300, max_hp: 1000 });
    w.tick(None);
    assert!(!w.core.projecting(), "{}", w.core.reason);
}

#[test]
fn the_ai_never_dives_a_one_versus_four() {
    let mut w = ai(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 445_000, 500_000), champ(12, 2, 440_000, 505_000), champ(13, 2, 445_000, 505_000)]);
    w.run(30);
    assert!(!w.core.projecting() && !w.core.flurrying(), "{}", w.core.reason);
}

#[test]
fn the_ai_farms_a_camp_with_its_skills_when_calm() {
    let mut w = ai(vec![creep(30, 420_000, 500_000, 3_000), creep(31, 425_000, 505_000, 3_000), creep(32, 418_000, 495_000, 600)]);
    w.f.ult_unlocked = false;
    let mut used = vec![];
    for _ in 0..900 {
        w.tick(None);
        if !w.core.reason.is_empty() && !used.contains(&w.core.reason) {
            used.push(w.core.reason.clone());
        }
        for u in w.f.units.iter_mut() {
            u.hp = u.max_hp;
        }
    }
    for k in ["Farm: Shadow Cyclone", "Farm: Slam"] {
        assert!(used.iter().any(|u| u == k), "{k}: {used:?}");
    }
    assert!(used.iter().any(|u| u.contains("recast")), "recasts with full charges: {used:?}");
}

#[test]
fn the_ai_uses_the_flurry_on_a_group() {
    let mut w = ai(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 440_000, 505_000), champ(ALLY, 1, 420_000, 500_000), champ(3, 1, 425_000, 510_000)]);
    w.core.cd = [u64::MAX / 2; 3];
    w.tick(None);
    assert_eq!(w.core.reason, "Shadow Flurry");
    assert!(w.core.flurrying());
}

#[test]
fn the_ai_flurries_from_the_projection_on_a_lone_healthy_target() {
    // one healthy champion: on foot that's no reason to flurry, but projected beside it, it is
    let mut w2 = ai(vec![champ(FOE, 2, 480_000, 500_000)]);
    w2.core.cd = [u64::MAX / 2; 3];
    w2.tick(None);
    assert_ne!(w2.core.reason, "Shadow Flurry", "on foot, a lone healthy target: no flurry");
    let mut w3 = World::new(vec![champ(FOE, 2, 480_000, 500_000)]);
    w3.tick(Some(Order::S1 { aim: Point::new(475_000, 500_000) }));
    w3.run(20);
    w3.f.manual = false;
    w3.core.cd = [u64::MAX / 2; 3];
    w3.tick(None);
    assert_eq!(w3.core.reason, "Shadow Flurry");
    assert!(w3.core.flurry.is_some_and(|fl| fl.echo));
}

// ------------------------------------------------------------------ the bridge on the glue mock host

mod glue {
    use crate::contender::Contender;
    use crate::contender_shadow_core::{Effect, Point};
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use mod_api_stable::{AttackTypeV1, CcKindV1, EntityKindV1 as K, StablePassive};
    use std::cell::RefCell;

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }
    fn run(host: &RefCell<Host>, c: &mut Contender, from: usize, n: usize) {
        for t in from..from + n {
            host.borrow_mut().tick = t;
            with_sim(host, |sim| c.on_update(sim, 11, 0, 0));
        }
    }

    #[test]
    fn directional_flurry_cues_bind_all_eight_headings_without_damage_or_control() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x,y)=open_spot(90_000);
        for directions in [false,true] {
            let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_contender")]);
            host.borrow_mut().tick=100;host.borrow_mut().record_view_effects=true;
            let mut c=Contender::default();
            c.configure(if directions {r#"{"shadow_art_v1":1,"shadow_directions_v1":1}"#} else {r#"{"shadow_art_v1":1}"#});
            let cues=(0..8).map(|h|Effect::Fx{name:"ct_flurry".into(),at:Point::new(x as i64,y as i64),ticks:8,heading:Some(h)}).collect();
            with_sim(&host,|sim|c.apply(sim,0,cues));
            let h=host.borrow();assert_eq!(h.view_effects.len(),8);
            for (i,(name,_,_,life)) in h.view_effects.iter().enumerate() {
                assert_eq!(*life,8);
                assert_eq!(name,&if directions {format!("ct_flurry_d{i}_t8")} else {"ct_flurry_t8".to_owned()});
            }
            assert!(h.damage.is_empty()&&h.moves.is_empty()&&h.ccs[0].is_empty());
        }
    }

    #[test]
    fn shadow_art_preserves_core_damage_movement_and_control_on_the_host() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let simulate = |art: bool| {
            let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 70_000, y), "fighter")]);
            host.borrow_mut().record_view_effects = true;
            let mut c = Contender::default();
            c.configure(if art { r#"{"shadow_art_v1":1,"shadow_directions_v1":1}"# } else { "{}" });
            with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
            let mut trace = Vec::new();
            for tick in 100..400 {
                run(&host, &mut c, tick, 1);
                let h = host.borrow();
                trace.push(serde_json::json!({"core":c.core,"damage":h.damage,"moves":h.moves,
                    "units":h.units.iter().map(|u|(u.pos,u.hp,u.targetable)).collect::<Vec<_>>(),
                    "cc":h.ccs.iter().map(|cs|cs.iter().map(|(c,t)|(c.name().to_owned(),c.kind,*t)).collect::<Vec<_>>()).collect::<Vec<_>>(),
                    "buffs":h.buffs.iter().map(|bs|bs.iter().map(|b|b.name().to_owned()).collect::<Vec<_>>()).collect::<Vec<_>>() }));
            }
            if art {
                let h = host.borrow();
                assert!(h.view_effects.iter().any(|(n,_,target,t)|n.starts_with("ct_body_") && target.target_id>1 && *t==10),
                    "Projection leases follow the actual targetable carrier for exactly ten ticks");
                assert!(!h.view_effects.iter().any(|(n,_,_,_)| n=="ct_shadow"), "No ambiguous long clone animation");
            }
            trace
        };
        assert_eq!(simulate(false), simulate(true));
    }

    #[test]
    fn the_projection_spawns_a_shadow_carrier_holds_his_body_and_relays_its_damage() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 70_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 30);
        let carrier = c.shadow.expect("the AI projected onto the far target");
        {
            let g = host.borrow();
            assert!(g.spawned.iter().any(|s| s.0 == carrier), "a carrier unit stands for the shadow");
            assert!(g.ccs[0].iter().any(|cc| cc.0.kind == CcKindV1::Bind.code() && cc.0.name() == "ct_hold"), "his body is held");
            assert_eq!(g.units[0].pos, (x, y), "his body never moved");
        }
        // the carrier takes 400: 280 reaches him, raw
        let before = host.borrow().damage.iter().filter(|d| d.1 == 0).map(|d| d.2).sum::<usize>();
        host.borrow_mut().units[carrier].hp -= 400;
        run(&host, &mut c, 130, 1);
        let after = host.borrow().damage.iter().filter(|d| d.1 == 0).map(|d| d.2).sum::<usize>();
        assert_eq!(after - before, 280);
    }

    #[test]
    fn his_basics_report_to_the_core_with_the_vulnerability_and_skills_never_hit_towers() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let tower = HostUnit { kind: K::Tower, team: 1, pos: (x + 15_000, y), hp: 3000, name: "tower", targetable: true };
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 18_000, y), "fighter"), tower]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        for t in 0..4u64 {
            c.core.stacks.retain(|_| true);
            let mut dmg = 100usize;
            host.borrow_mut().tick = 100 + t as usize;
            with_sim(&host, |sim| c.on_attack(sim, 0, 0, 1, &mut dmg));
            run(&host, &mut c, 100 + t as usize, 1);
            if t == 3 {
                assert!(dmg > 100, "stacked: his basic hits harder ({dmg})");
            }
        }
        assert!(c.core.stacks_on(1, 104) >= 3);
        let effects = vec![
            crate::contender_shadow_core::Effect::Damage { target: 2, amount: 50 },
            crate::contender_shadow_core::Effect::Stun { target: 2, ticks: 30 },
        ];
        with_sim(&host, |sim| c.apply(sim, 0, effects));
        let g = host.borrow();
        assert!(!g.damage.iter().any(|d| d.1 == 2 && d.3 == AttackTypeV1::Skill.code()) && g.ccs[2].is_empty(), "never a tower");
    }

    #[test]
    fn a_tick_on_the_host_runs_clean_and_the_shadow_goes_when_he_dies() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 70_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 30);
        let carrier = c.shadow.expect("projected");
        host.borrow_mut().units[0].hp = 0;
        run(&host, &mut c, 130, 1);
        assert!(c.shadow.is_none() && host.borrow().units[carrier].hp == 0, "the shadow is gone with him");
    }
}
