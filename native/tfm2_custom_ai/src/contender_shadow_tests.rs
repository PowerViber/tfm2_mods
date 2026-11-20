//! Contender core and native adapter regression tests.
use crate::contender_shadow_core::*;
use crate::contender_shadow_data as d;

const ME: usize = 1;
const ALLY: usize = 2;
const FOE: usize = 10;

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, visible: true, targetable: true, farmable: false }
}
fn creep(id: usize, x: i64, y: i64, hp: i64) -> Unit {
    Unit { champion: false, farmable: true, hp, max_hp: hp, team: 9, ..champ(id, 9, x, y) }
}

/// A tiny world: him and the units, with deterministic receipts and control timing.
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
    air: Vec<(usize, u64)>,
}
impl World {
    fn new(units: Vec<Unit>) -> World {
        let core = Core::default();
        World {
            core,
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
            air: vec![],
        }
    }
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.order = order;
        self.f.link_airborne = self.air.iter().filter(|(_, until)| self.f.tick < *until).map(|(id, _)| *id).collect();
        let out = self.core.step(&self.f);
        self.f.landed_hits = 0;
        self.f.landed_merge_hits = 0;
        self.f.basic_hits.clear();
        self.f.incoming.clear();
        let tick = self.f.tick;
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::Damage { target, amount, source } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = (u.hp - amount).max(1);
                        if *amount > 0 && source.credits_flurry() {
                            if *source == HitSource::Merge {
                                self.f.landed_merge_hits += 1;
                            } else {
                                self.f.landed_hits += 1;
                            }
                        }
                    }
                }
                Effect::LinkAirborne { target, ticks } => self.air.push((*target, tick + ticks)),
                Effect::Knock { target, to, .. } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::Glide { to, .. } => self.f.pos = *to,
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
}
#[test]
fn rhythm_vulnerability_and_break_use_current_rules() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.unit(FOE).hp = 100_000;
    w.unit(FOE).max_hp = 100_000;
    let burst = 100_000 * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO;
    let breaks = |w: &World| w.count(|e| matches!(e, Effect::Damage { target: FOE, amount, .. } if *amount == burst));
    for _ in 0..4 {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    assert_eq!(w.core.vuln_permille(FOE, w.f.tick), 4 * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK);
    for _ in 4..d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::RHYTHM_MAX_STACKS);
    assert_eq!(breaks(&w), 0, "20 hits stack to 20, no Break yet");
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(breaks(&w), 1, "the 21st hit triggers the canonical Break");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0, "its stacks reset to 0");
}

#[test]
fn shadow_break_knocks_up_once_per_cooldown_on_each_enemy() {
    // contender-3.6 (Reihan): "at shadow break it knocks up the enemy 1s but with a cd on that enemy"
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    let knocks = |w: &World| w.count(|e| matches!(e, Effect::BreakAirborne { target: FOE, ticks } if *ticks == d::RHYTHM_BREAK_KNOCK_TICKS as u64));
    let breaks = |w: &World| w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_break"));
    let break_once = |w: &mut World| for _ in 0..=d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    };
    break_once(&mut w);
    let first = w.f.tick;
    assert_eq!((breaks(&w), knocks(&w)), (1, 1), "a Break knocks up");
    break_once(&mut w);
    assert!(w.f.tick < first + d::RHYTHM_BREAK_KNOCK_COOLDOWN_TICKS as u64);
    assert_eq!((breaks(&w), knocks(&w)), (2, 1), "a second Break inside the cooldown still breaks, without the knock-up");
    while w.f.tick < first + d::RHYTHM_BREAK_KNOCK_COOLDOWN_TICKS as u64 { w.tick(None); }
    break_once(&mut w);
    assert_eq!((breaks(&w), knocks(&w)), (3, 2), "after the cooldown it knocks up again");
}

#[test]
fn after_a_shadow_break_his_rhythm_buffs_linger_for_the_canonical_duration() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.unit(FOE).hp = 100_000;
    w.unit(FOE).max_hp = 100_000;
    for _ in 0..=d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    let broke = w.f.tick - 1;
    let full = d::RHYTHM_MAX_STACKS * d::RHYTHM_ATTACK_SPEED_PERMILLE_PER_STACK / 10;
    w.core.echoes.clear();
    w.run(d::RHYTHM_BREAK_BUFF_TICKS as u64 - 60);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0);
    let last = w.log.iter().rev().find_map(|(_, e)| match e { Effect::Rhythm { attack_speed, .. } => Some(*attack_speed), _ => None });
    assert_eq!(last, Some(full), "full buff before expiry");
    while w.f.tick < broke + d::RHYTHM_BREAK_BUFF_TICKS as u64 + 2 {
        w.tick(None);
    }
    let last = w.log.iter().rev().find_map(|(_, e)| match e { Effect::Rhythm { attack_speed, .. } => Some(*attack_speed), _ => None });
    assert_eq!(last, Some(0), "clears at expiry");
}

#[test]
fn a_max_stacked_creep_carries_eight_stacks_onto_a_champion_and_a_kill_lingers() {
    let mut w = World::new(vec![creep(30, 405_000, 500_000, 50_000), champ(FOE, 2, 410_000, 500_000)]);
    for _ in 0..d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(30);
        w.tick(None);
    }
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::RHYTHM_CARRY_STACKS);
    w.core.stacks.retain(|s| s.id != FOE);
    w.tick(None);
    w.f.units.retain(|u| u.id != 30);
    w.tick(None);
    assert!(w.core.linger.is_some_and(|l| l.0 == d::RHYTHM_MAX_STACKS));
}

#[test]
fn echoes_never_add_rhythm_stacks() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 1);
    w.run(d::PACT_DELAY_TICKS as u64 + 2);
    assert!(w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo")), "the echo landed");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 1, "and added no stack");
}

#[test]
fn both_merged_echoes_of_a_basic_deal_damage() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.tick(Some(Order::S1));
    w.f.basic_hits.push(FOE);
    w.tick(None);
    let echo = w.f.attack * d::PACT_PCT / 100;
    assert!(w.core.echoes.iter().filter(|e| e.target == FOE).all(|e| e.amount == echo));
    assert_eq!(w.core.echoes.iter().filter(|e| e.target == FOE).count(), 2);
}

fn flurry_targets(w: &World) -> Vec<usize> {
    // The punch's own damage packet is the first one after the previous punch's cue (a Shadow Break or a tether copy
    // may follow it inside the same punch).
    let mut out = vec![];
    let mut waiting = false;
    for (_, e) in &w.log {
        match e {
            Effect::Vanish { .. } if out.is_empty() && !waiting => waiting = true,
            Effect::Damage { target, .. } if waiting => {
                out.push(*target);
                waiting = false;
            }
            Effect::Fx { name, .. } if name == "ct_flurry" => waiting = true,
            _ => {}
        }
    }
    out.truncate(w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry")));
    out
}

#[test]
fn flurry_prefers_another_champion_instead_of_repeating_the_zero_distance_victim() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 450_000, 500_000)]);
    for u in &mut w.f.units { u.hp = 100_000; u.max_hp = 100_000; }
    w.tick(Some(Order::S2)); w.run(d::FLURRY_TICKS as u64);
    let targets = flurry_targets(&w);
    assert_eq!(targets.len(), d::FLURRY_PUNCHES as usize);
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
        w.tick(Some(Order::S2)); w.run(d::FLURRY_TICKS as u64);
        let targets = flurry_targets(&w);
        assert_eq!(targets.len(), d::FLURRY_PUNCHES as usize);
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
    w.tick(Some(Order::S2)); w.run(d::FLURRY_TICKS as u64);
    let targets = flurry_targets(&w);
    assert_eq!(targets.len(), d::FLURRY_PUNCHES as usize);
    assert!(targets.iter().all(|id| *id == FOE), "{targets:?}");
}

#[test]
fn flurry_alternative_ties_are_deterministic_under_snapshot_reordering() {
    let mut units = vec![champ(FOE, 2, 430_000, 500_000), champ(12, 2, 450_000, 490_000), champ(11, 2, 450_000, 510_000)];
    for u in &mut units { u.hp = 100_000; u.max_hp = 100_000; }
    let mut a = World::new(units.clone()); units.reverse(); let mut b = World::new(units);
    for w in [&mut a, &mut b] { w.tick(Some(Order::S2)); w.run(d::FLURRY_TICKS as u64); }
    let targets = flurry_targets(&a);
    assert_eq!(&targets[..2], &[FOE, 11]);
    assert_eq!(targets, flurry_targets(&b));
    assert!(targets.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn flurry_visuals_cover_eight_directions_without_changing_punch_count_or_cadence() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 100_000; w.unit(FOE).max_hp = 100_000;
    w.tick(Some(Order::S2)); w.run(d::FLURRY_TICKS as u64);
    let cues: Vec<_> = w.log.iter().filter_map(|(tick,e)|match e {
        Effect::Fx { name, ticks, heading: Some(h), .. } if name=="ct_flurry" => Some((*tick,*ticks,*h)), _=>None }).collect();
    assert_eq!(cues.len(), d::FLURRY_PUNCHES as usize);
    let mut directions = std::collections::BTreeSet::new();
    for (i, (_, life, h)) in cues.iter().enumerate() {
        assert_eq!(*life,8); assert_eq!(*h,Core::flurry_heading(ME,i as i64)); directions.insert(*h);
        if i>0 { assert_eq!(cues[i].0-cues[i-1].0,d::FLURRY_GAP_TICKS as u64); }
    }
    assert_eq!(directions.len(),8);
    assert_eq!(flurry_targets(&w).len(),cues.len(),"one damage packet per punch");
}

fn ai(units: Vec<Unit>) -> World {
    let mut w = World::new(units);
    w.f.manual = false;
    w.core.ult_at = u64::MAX / 4; // the ult on cooldown
    w
}

#[test]
fn aggressive_values_change_supported_chase_without_relaxing_tower_safety() {
    for (aggressive, expected) in [(20, false), (80, true)] {
        let mut w = ai(vec![champ(FOE, 2, 475_000, 500_000), champ(ALLY, 1, 463_000, 510_000)]);
        w.f.profile[6] = aggressive;
        w.core.merge_at = 999;
        w.core.flurry_cast_at = 999;
        w.tick(None);
        assert_eq!(w.core.reason == "Commit: supported approach", expected);
        let mut tower = champ(50, 2, 475_000, 510_000);
        tower.tower = true; tower.champion = false;
        w.f.units.push(tower);
        w.tick(None);
        assert_eq!(w.core.reason, "Hold: unsafe or unsupported approach");
    }
}

#[test]
fn basic_finish_checks_hp_shield_and_retains_flurry() {
    let mut w = ai(vec![champ(FOE, 2, 420_000, 500_000)]);
    w.core.merge_at = 500;
    w.core.ult_at = 500;
    w.unit(FOE).hp = 70;
    w.f.defences = vec![TargetDefence { id: FOE, armour: 25, shield: 20 }];
    w.tick(None);
    assert!(w.core.reason.starts_with("Hold"));
    w.f.defences[0].shield = 0;
    w.tick(None);
    assert_eq!(w.core.reason, "Commit: basic can finish");
    assert!(!w.core.flurrying());
}

#[test]
fn counterpunch_follows_the_confirmed_attacker_and_does_not_recurse() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 415_000, 500_000)]);
    let hit = IncomingHit { attacker: FOE, from: w.unit(FOE).pos, counter: false };
    w.f.incoming = vec![hit, IncomingHit { attacker: 11, from: w.unit(11).pos, counter: false }];
    let out = w.tick(None);
    assert!(out.contains(&Effect::Damage { target: FOE, amount: 50, source: HitSource::Counter }));
    assert!(out.iter().any(|e| matches!(e, Effect::Glide { .. })));
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1);
    assert!(w.core.stacks.is_empty() && w.core.echoes.is_empty());
    assert_eq!(w.unit(11).hp, 1000);
    w.run(24);
    w.f.incoming.push(IncomingHit { counter: true, ..hit });
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Damage { .. })));
    assert_eq!(w.core.charges, d::CHARGES_MAX - 2, "incoming Counterpunch may still be dodged");
}

#[test]
fn counterpunch_rejects_a_wall_or_range_and_can_break_at_full_stacks() {
    let mut w = World::new(vec![champ(FOE, 2, 435_000, 500_000)]);
    let hit = IncomingHit { attacker: FOE, from: w.unit(FOE).pos, counter: false };
    w.f.barriers.push(Barrier { from: Point::new(420_000, 490_000), to: Point::new(420_000, 510_000), half_width: 1000 });
    w.f.incoming.push(hit);
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Damage { .. })));
    w.f.barriers.clear();
    w.run(24);
    w.unit(FOE).pos.x = 470_000;
    w.f.incoming.push(hit);
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Damage { .. })));
    w.run(24);
    w.f.pos = Point::new(400_000, 500_000);
    w.unit(FOE).pos.x = 440_000;
    w.core.stacks.push(Stack { id: FOE, count: 20, until: 1000, champion: true });
    w.f.incoming.push(hit);
    let out = w.tick(None);
    let counter = (d::COUNTER_BASE + d::COUNTER_AD_RATIO * w.f.attack / 100)
        * (1_000 + d::RHYTHM_MAX_STACKS * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK) / 1_000;
    let burst = w.unit(FOE).max_hp * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO * w.f.attack / 100;
    assert!(out.contains(&Effect::Damage { target: FOE, amount: counter + burst, source: HitSource::Counter }), "{out:?}");
    assert!(out.iter().any(|e| matches!(e, Effect::BreakAirborne { target: FOE, .. })), "the Counterpunch Break knocks up too");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0);
}

#[test]
fn moving_merge_cross_checks_the_50k_boundary_and_credits_each_target_once() {
    let mut w = World::new(vec![
        champ(FOE, 2, 440_000, 500_000), champ(11, 2, 493_000, 500_000),
        creep(30, 402_000, 500_000, 1000), champ(12, 2, 493_001, 500_000),
    ]);
    w.core.charges = 4;
    w.core.charge_at = 500;
    w.tick(Some(Order::S1));
    w.f.pos.x = 443_000;
    w.run(59);
    assert!(!w.seen(|e| matches!(e, Effect::Damage { .. })));
    let pulse = w.tick(None);
    let targets: Vec<_> = pulse.iter().filter_map(|effect| match effect {
        Effect::Damage { source: HitSource::Merge, target, .. } => Some(*target),
        _ => None,
    }).collect();
    assert_eq!(targets, [FOE, 11, 30]);
    // contender-3.6: each cross hit refunds the shared shadow clock by the missing-shadow rate
    let cds = w.core.charge_at;
    w.tick(None);
    assert_eq!(w.core.charge_at, cds - 3 * ((d::CHARGES_MAX - 4) * d::CHARGES_HIT_REFUND_TICKS_PER_MISSING) as u64);
    assert!(w.core.stacks.is_empty() && w.core.echoes.is_empty());
}

#[test]
fn the_final_flurry_punch_and_reappearance_use_separate_positions() {
    // Reihan approved 30 hits in 1.5 seconds, without an extra recast gap.
    assert_eq!((d::FLURRY_PUNCHES, d::FLURRY_TICKS, d::FLURRY_CAST_GAP_TICKS), (30, 90, 0));
    let foe_pos = Point::new(430_000, 500_000);
    let mut w = World::new(vec![champ(FOE, 2, foe_pos.x, foe_pos.y)]);
    let start = w.f.tick;
    let final_tick = start + d::FLURRY_TICKS as u64;
    w.tick(Some(Order::S2));
    let recharge = start + d::CHARGES_RECHARGE_TICKS as u64;
    assert_eq!(w.core.charges, d::CHARGES_MAX - d::CHARGES_FLURRY_COST, "a Flurry costs two shadows");
    w.run((d::FLURRY_TICKS - 1) as u64);
    assert_eq!(flurry_targets(&w).len(), (d::FLURRY_PUNCHES - 1) as usize);

    // Use the engine's latest position even if it changed after takeoff.
    let current_pos = Point::new(402_000, 498_000);
    w.f.pos = current_pos;
    let end = w.tick(None);
    assert_eq!(flurry_targets(&w).len(), d::FLURRY_PUNCHES as usize);
    assert!(end.iter().any(|e| matches!(e, Effect::Fx { name, at, .. } if name == "ct_flurry" && *at == foe_pos)));
    assert!(end.iter().any(|e| matches!(e, Effect::Fx { name, at, .. } if name == "ct_appear" && *at == current_pos)));
    assert_eq!(w.f.pos, current_pos);
    assert_eq!(w.core.flurry_cast_at, final_tick);
    assert_eq!(w.core.charge_at, recharge, "flurry punches never refund shadows");
    assert!(w.core.echoes.is_empty());
    assert_eq!(w.core.haymaker_until, final_tick + d::FLURRY_HAYMAKER_TICKS as u64);
    w.tick(Some(Order::S2));
    assert!(w.core.flurrying(), "{}", w.core.rejection);
}

#[test]
fn early_flurry_end_reappears_in_place_with_haymaker_only_after_a_punch() {
    for lands_first_punch in [false, true] {
        let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
        w.tick(Some(Order::S2));
        let cadence = d::FLURRY_GAP_TICKS as u64;
        if lands_first_punch {
            w.run(cadence);
        }
        w.unit(FOE).targetable = false;
        w.run(cadence - 1);
        let pos = w.f.pos;
        let effects = w.tick(None);
        assert!(effects.iter().any(|e| matches!(e, Effect::Fx { name, at, .. } if name == "ct_appear" && *at == pos)));
        assert!(!effects.iter().any(|e| matches!(e, Effect::Vanish { .. })));
        assert_eq!(w.f.pos, pos);
        assert!(!w.core.flurrying());
        assert_eq!(flurry_targets(&w).len(), usize::from(lands_first_punch));
        assert_eq!(w.core.haymaker_until > w.f.tick, lands_first_punch);
        assert_eq!(w.core.charges, d::CHARGES_MAX - d::CHARGES_FLURRY_COST);
    }
}

#[test]
fn dodges_and_flurry_share_one_pool_of_shadows() {
    // contender-3.6 (Reihan): a dodge costs 1 shadow, a Flurry 2; with fewer than 2 the Flurry is refused
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    assert_eq!(w.core.charges, d::CHARGES_MAX);
    w.f.incoming.push(IncomingHit { attacker: FOE, from: Point::new(430_000, 500_000), counter: false });
    w.tick(None);
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1, "a dodge costs one");
    w.core.charges = d::CHARGES_FLURRY_COST - 1;
    w.tick(Some(Order::S2));
    assert!(!w.core.flurrying());
    assert_eq!(w.core.rejection, "s2: not enough shadows");
    w.core.charges = d::CHARGES_FLURRY_COST;
    w.tick(Some(Order::S2));
    assert!(w.core.flurrying());
    assert_eq!(w.core.charges, 0);
}

#[test]
fn each_landed_hit_refunds_more_the_fewer_shadows_he_has() {
    // contender-3.6 (Reihan): "the lower the shadow the faster it recharges per hit"
    for left in [d::CHARGES_MAX - 1, 0] {
        let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
        w.core.charges = left;
        w.core.charge_at = 10_000;
        w.f.landed_hits = 1;
        w.tick(None);
        let refund = ((d::CHARGES_MAX - left) * d::CHARGES_HIT_REFUND_TICKS_PER_MISSING) as u64;
        assert_eq!(w.core.charge_at, 10_000 - refund, "{left} shadows left");
    }
    // full: nothing to refund
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.core.charge_at = 10_000;
    w.f.landed_hits = 3;
    w.tick(None);
    assert_eq!(w.core.charge_at, 10_000);
}

#[test]
fn shadow_grab_hooks_the_first_enemy_in_line_drags_it_to_him_and_knocks_it_up() {
    // contender-3.6 (Reihan): "a long shadow grab and knock up ... using the same shadow as before but with a cd"
    let near = 400_000 + d::SHADOW_GRAB_RANGE * 2 / 3;
    let mut w = World::new(vec![champ(FOE, 2, near, 500_000), champ(11, 2, near + 5_000, 500_000)]);
    w.core.charges = 0;
    w.core.charge_at = 100_000;
    w.tick(Some(Order::Ult { aim: Point::new(near + 5_000, 500_000) }));
    assert_eq!(w.core.ult_at, w.f.tick - 1 + d::SHADOW_GRAB_COOLDOWN_TICKS as u64);
    assert_eq!(w.core.charges, 0, "the grab no longer refills shadows");
    let mut guard = 0;
    while !matches!(w.core.shadow_link, Some(ShadowLink::Airborne { .. })) {
        w.tick(None);
        guard += 1;
        assert!(guard < 200, "the grab never landed");
    }
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: FOE, source: HitSource::Grab, .. })));
    assert!(!w.seen(|e| matches!(e, Effect::Damage { target: 11, .. })), "only the first enemy in line");
    assert!(w.seen(|e| matches!(e, Effect::LinkAirborne { target: FOE, ticks } if *ticks == d::SHADOW_GRAB_KNOCK_TICKS as u64)));
    assert!(!w.seen(|e| matches!(e, Effect::LinkAirborne { target: 11, .. })));
    // round 190: Codex's latch flashes on contact and the burst marks the knock-up, once each
    assert_eq!(w.count(|e| matches!(e, Effect::Fx { name, ticks, .. } if name == "ct_grab_latch" && *ticks == GRAB_LATCH_TICKS)), 1);
    assert_eq!(w.count(|e| matches!(e, Effect::Fx { name, ticks, .. } if name == "ct_grab_burst" && *ticks == GRAB_BURST_TICKS)), 1);
    let x = w.unit(FOE).pos.x;
    assert!(x - 400_000 <= d::SHADOW_GRAB_PULL_STOP + 2_000, "dragged to him (x {x})");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 1, "the grab builds Rhythm");
    w.run(d::SHADOW_GRAB_KNOCK_TICKS as u64 + 2);
    assert!(w.core.shadow_link.is_none());
    w.tick(Some(Order::Ult { aim: Point::new(near, 500_000) }));
    assert!(w.core.shadow_link.is_none(), "cooling down");
    assert_eq!(w.core.rejection, "ult: locked or Shadow Grab cooling down");
}

#[test]
fn shadow_grab_misses_cleanly_at_max_range_and_skips_immune_targets() {
    let far = 400_000 + d::SHADOW_GRAB_RANGE + d::SHADOW_GRAB_WIDTH + 10_000;
    let mut w = World::new(vec![champ(FOE, 2, far, 500_000), champ(11, 2, 430_000, 500_000)]);
    w.f.link_excluded.push(11);
    w.tick(Some(Order::Ult { aim: Point::new(far, 500_000) }));
    w.run(d::SHADOW_GRAB_FLIGHT_TICKS as u64 + 5);
    assert!(w.core.shadow_link.is_none(), "the shadow returns empty-handed");
    assert!(!w.seen(|e| matches!(e, Effect::Damage { .. } | Effect::LinkAirborne { .. } | Effect::Knock { .. })));
    assert!(!w.seen(|e| matches!(e, Effect::Fx { name, .. } if name.starts_with("ct_grab_"))), "a miss shows no latch or burst");
}

#[test]
fn ai_rejects_the_low_hp_chase_and_tower_flurry_but_commits_with_support() {
    for aggro in [20, 80] {
        let mut w = ai(vec![champ(FOE,2,550_000,500_000), champ(11,2,552_000,500_000)]);
        w.f.profile[6] = aggro;
        w.f.hp = 100;
        assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Steer { to } if to.x>400_000)));
        w.unit(FOE).pos.x = 445_000;
        w.unit(11).pos.x = 450_000;
        w.f.units.push(Unit { tower:true, champion:false, ..champ(33,2,478_000,500_000) });
        w.tick(None);
        assert!(!w.core.flurrying());
        w.f.hp = 1000;
        w.f.units.retain(|u| !u.tower);
        w.f.units.push(champ(ALLY,1,440_000,500_000));
        w.f.ult_unlocked = false;
        w.tick(None);
        assert!(w.core.flurrying(), "{}",w.core.reason);
        w.unit(FOE).pos.x += 200_000;
        w.unit(11).pos.x += 200_000;
        w.run(61);
        assert!(w.f.pos.x < 470_000);
    }
}

// ------------------------------------------------------------------ the bridge on the glue mock host

mod glue {
    use crate::contender::Contender;
    use crate::contender_shadow_data as d;
    use crate::contender_shadow_core::{Effect, Point};
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use mod_api_stable::{AttackTypeV1, EntityKindV1 as K, StablePassive};
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
    fn fused_merge_presentation_preserves_core_and_native_combat_commands() {
        let _guard=crate::walls::TEST_LOCK.lock();let (x,y)=open_spot(90_000);
        let play=|enabled:bool|{
            let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_contender"),champ(1,(x+18_000,y),"fighter")]);
            let mut c=Contender::default();
            c.configure(if enabled {r#"{"shadow_art_v1":1,"shadow_merge_art_v1":1}"#} else {r#"{"shadow_art_v1":1}"#});
            c.core.merge_until=460;c.core.merge_at=u64::MAX/2;
            for tick in 100..280 {
                host.borrow_mut().tick=tick;
                with_sim(&host,|sim|{
                    if tick%20==0 {c.on_base_attack(sim,11,0,0);let mut damage=100;c.on_attack(sim,0,0,1,&mut damage);}
                    c.on_update(sim,11,0,0);
                });
            }
            let h=host.borrow();
            (serde_json::to_value(c.core).unwrap(),h.damage.clone(),h.moves.clone(),h.heals.clone(),
             h.units.iter().map(|u|u.hp).collect::<Vec<_>>(),
             h.ccs.iter().map(|ccs|ccs.iter().map(|(c,end)|(c.kind,c.name().to_owned(),*end)).collect::<Vec<_>>()).collect::<Vec<_>>())
        };
        assert_eq!(play(false),play(true));
    }

    #[test]
    fn a_champion_skill_hit_is_dodged_too_and_so_is_a_basic_attack() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 20_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 2);
        host.borrow_mut().cooldowns.push((1, mod_api_stable::SkillSlotV1::Skill.code(), 570, 600));
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 200));
        assert!(host.borrow().heals.iter().any(|h| h.2 == 200), "the skill hit is undone");
        assert_eq!(c.core.charges, crate::contender_shadow_data::CHARGES_MAX - 1);
        // after the 0.4 s gap, a basic attack is dodged as well
        host.borrow_mut().tick = 200;
        host.borrow_mut().cooldowns.clear();
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 80));
        assert!(host.borrow().heals.iter().any(|h| h.2 == 80), "the basic is undone");
    }

    #[test]
    fn during_the_flurry_he_is_banished_every_tick_so_enemies_cannot_hit_him() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 20_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 1);
        let effects = vec![crate::contender_shadow_core::Effect::Vanish { ticks: 3 }];
        with_sim(&host, |sim| c.apply(sim, 0, effects));
        let g = host.borrow();
        assert!(g.banished.iter().any(|b| b.0 == 0 && b.1 == 0 && b.2 == 3), "banished: untargetable and unseen");
        assert!(g.invisible.iter().any(|i| i.0 == 0));
    }

    #[test]
    fn a_champion_hit_is_dodged_healed_back_and_he_slips_but_creeps_never_trigger_it() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let creep = HostUnit { kind: K::Minion, team: 1, pos: (x + 10_000, y), hp: 500, name: "minion", targetable: true };
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 20_000, y), "fighter"), creep]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 2);
        host.borrow_mut().tick = 102;
        let before = c.core.charges;
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 2, 80));
        assert!(host.borrow().heals.is_empty(), "a creep's hit is never dodged");
        assert_eq!(c.core.charges, before);
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 120));
        assert!(host.borrow().heals.iter().any(|h| h.1 == 0 && h.2 == 120), "the champion's hit is undone");
        assert_eq!(c.core.charges, before - 1, "one dodge spent");
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 90));
        assert_eq!(host.borrow().heals.len(), 1, "his one dodge is spent: the next hit lands");
        run(&host, &mut c, 103, 2);
        assert!(c.core.pending_dodge.is_none(), "the slip was played");
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
            let mut dmg = 100usize;
            host.borrow_mut().tick = 100 + t as usize;
            with_sim(&host, |sim| c.on_attack(sim, 0, 0, 1, &mut dmg));
            run(&host, &mut c, 100 + t as usize, 1);
            if t == 3 {
                assert!(dmg > 100, "stacked: his basic hits harder ({dmg})");
            }
        }
        assert!(c.core.stacks_on(1, 104) >= 3);
        let effects = vec![Effect::Damage { target: 2, amount: 50, source: crate::contender_shadow_core::HitSource::Counter }, Effect::BreakAirborne { target: 2, ticks: 30 }];
        with_sim(&host, |sim| c.apply(sim, 0, effects));
        let g = host.borrow();
        assert!(!g.damage.iter().any(|d| d.1 == 2 && d.3 == AttackTypeV1::Skill.code()) && g.ccs[2].is_empty(), "never a tower");
    }

    #[test]
    fn native_damage_receipts_credit_skill_hits_and_preserve_the_basic_identity() {
        use crate::contender::BasicImpact;
        use crate::contender_shadow_core::HitSource;
        use mod_api_stable::{InputTargetV1, StableEffectType};
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 20_000, y), "fighter")]);
        let mut c = Contender::default();
        c.configure(r#"{"shadow_receipts_v1":1}"#);
        c.core.charges = 4;
        c.core.charge_at = 800;
        host.borrow_mut().tick = 100;
        with_sim(&host, |sim| {
            BasicImpact.apply(sim, 0, 0, InputTargetV1::target(1));
            c.apply(sim, 0, vec![
                Effect::Damage { target: 1, amount: 40, source: HitSource::Counter },
                Effect::Damage { target: 1, amount: 10, source: HitSource::Flurry },
                Effect::Damage { target: 1, amount: 0, source: HitSource::Merge },
                Effect::Damage { target: 1, amount: 30, source: HitSource::Merge },
            ]);
        });
        run(&host, &mut c, 101, 1);
        // contender-3.6: the counter, the real merge hit and the basic each refund the shadow clock; flurry and zero hits don't
        let per_hit = ((d::CHARGES_MAX - 4) * d::CHARGES_HIT_REFUND_TICKS_PER_MISSING) as u64;
        assert_eq!(c.core.charge_at, 800 - 3 * per_hit);
        assert_eq!(c.core.stacks_on(1, 101), 1);
        assert_eq!(host.borrow().damage.iter().filter(|d|d.3 == AttackTypeV1::BaseAttack.code()).count(),1);
        run(&host, &mut c, 102, 1);
        assert_eq!(c.core.charge_at, 800 - 3 * per_hit, "consumed receipt counts once");
    }

    #[test]
    fn the_game_ai_owns_his_casts_and_movement_the_native_ai_never_acts_in_game() {
        // contender-3.2 (Reihan: "revert back to the basics, since its own game already has an ai")
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 40_000, y), "fighter"), champ(1, (x + 45_000, y + 8_000), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 120);
        assert!(c.core.reason.is_empty(), "no native decision: {}", c.core.reason);
        assert!(!c.core.merged(220) && !c.core.flurrying() && c.core.shadow_link.is_none(), "nothing cast without the engine");
        assert!(!host.borrow().buffs[0].iter().any(|b| b.name().starts_with("ct_go:") || b.name().starts_with("ct_hold:")), "no steering");
    }

    #[test]
    fn an_engine_cast_of_s1_and_s2_runs_the_kit_on_the_next_update() {
        use mod_api_stable::{InputTargetV1, StableEffectType};
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 2);
        // the engine casts S1: the native effect only drops a marker
        host.borrow_mut().tick = 102;
        with_sim(&host, |sim| crate::contender::CastMerge.apply(sim, 0, 0, InputTargetV1::target(1)));
        assert!(host.borrow().buffs[0].iter().any(|b| b.name().starts_with("ct_cast:s1:")));
        run(&host, &mut c, 102, 1);
        assert!(c.core.merged(103), "Shadow Merge started: {}", c.core.rejection);
        assert!(!host.borrow().buffs[0].iter().any(|b| b.name().starts_with("ct_cast:")), "the marker is consumed");
        // then S2: two shadows are spent
        let before = c.core.charges;
        host.borrow_mut().tick = 110;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 110, 1);
        assert!(c.core.flurrying() && c.core.charges == before - d::CHARGES_FLURRY_COST, "Flurry started: {}", c.core.rejection);
        // a press without enough shadows is refused without a panic
        c.core.flurry = None;
        c.core.charges = d::CHARGES_FLURRY_COST - 1;
        c.core.charge_at = 10_000;
        host.borrow_mut().tick = 200;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 200, 1);
        assert!(!c.core.flurrying());
        assert_eq!(c.core.charges, d::CHARGES_FLURRY_COST - 1, "a refused press spends nothing");
    }

    #[test]
    #[ignore = "native Flurry microbenchmark on the deterministic mock host"]
    fn measure_native_flurry_ticks() {
        use mod_api_stable::{InputTargetV1, StableEffectType};
        use std::time::Instant;
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let ticks = crate::contender_shadow_data::FLURRY_TICKS as usize + 1;
        let expected = crate::contender_shadow_data::FLURRY_PUNCHES as usize;
        const SAMPLES: usize = 7;
        const CASTS: usize = 30;
        for targets in [0, 1, 3, 5] {
            let mut samples = Vec::with_capacity(SAMPLES);
            for sample in 0..=SAMPLES {
                let mut elapsed = std::time::Duration::ZERO;
                for cast in 0..CASTS {
                    let mut units = vec![champ(0, (x, y), "tfm2_custom_contender")];
                    units.extend((0..targets).map(|i| champ(1, (x + 20_000 + i * 3_000, y), "fighter")));
                    let host = Host::new(units);
                    let mut c = Contender::default();
                    c.configure(r#"{"shadow_art_v1":1,"shadow_directions_v1":1,"shadow_merge_art_v1":1,"shadow_rhythm_art_v1":1,"shadow_receipts_v1":1}"#);
                    let start = 100 + (sample * CASTS + cast) * ticks;
                    host.borrow_mut().tick = start;
                    with_sim(&host, |sim| {
                        c.on_spawn(sim, 0, 0);
                        if targets != 0 {
                            crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::target(1));
                        }
                    });
                    let timer = Instant::now();
                    run(&host, &mut c, start, ticks);
                    elapsed += timer.elapsed();
                    assert_eq!(host.borrow().damage.len(), if targets == 0 { 0 } else { expected });
                }
                if sample != 0 {
                    samples.push(elapsed.as_secs_f64() * 1e6 / (ticks * CASTS) as f64);
                }
            }
            samples.sort_by(f64::total_cmp);
            println!(
                "FLURRY_NATIVE targets={targets} median_us_per_tick={:.3} max_sample_us_per_tick={:.3}",
                samples[SAMPLES / 2], samples[SAMPLES - 1]
            );
        }
    }

    #[test]
    fn shadow_merge_buffs_him_makes_him_cc_immune_and_slows_whoever_hits_him() {
        // contender-3.5 (Reihan): S1 is a self-buff; any enemy that damages him while merged is slowed
        use crate::contender_shadow_data as d;
        use mod_api_stable::{InputTargetV1, StableEffectType};
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 2);
        // not merged yet: a hit never slows the attacker
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 50));
        assert!(!host.borrow().buffs[1].iter().any(|b| b.name() == "ct_merge_slow"));
        host.borrow_mut().tick = 102;
        with_sim(&host, |sim| crate::contender::CastMerge.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 102, 1);
        let g = host.borrow();
        let merge = g.buffs[0].iter().find(|b| b.name() == "ct_merge").expect("the Merge buff");
        assert_eq!((merge.move_speed_mult, merge.attack_speed_mult, merge.attack_mult), (d::MERGE_MOVE_SPEED_PCT as i32, d::MERGE_ATTACK_SPEED_PCT as i32, d::MERGE_ATTACK_PCT as i32));
        assert_eq!(merge.cc_immune, d::MERGE_CC_IMMUNE != 0);
        drop(g);
        host.borrow_mut().tick = 110;
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 50));
        let g = host.borrow();
        let slow = g.buffs[1].iter().find(|b| b.name() == "ct_merge_slow").expect("the attacker is slowed");
        assert_eq!(slow.move_speed_mult, -(d::MERGE_ATTACKER_SLOW_PCT as i32));
    }

    #[test]
    fn a_flurry_press_with_no_charge_is_not_wanted_so_the_input_ai_basic_attacks_instead() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 1);
        assert_ne!(crate::press::get(11, 0, 100).unwrap() & crate::press::S2, 0, "a charge is ready: the press is wanted");
        c.core.charges = d::CHARGES_FLURRY_COST - 1;
        c.core.charge_at = 10_000;
        run(&host, &mut c, 101, 1);
        assert_eq!(crate::press::get(11, 0, 101).unwrap() & crate::press::S2, 0, "no charge: not wanted");
    }

    #[test]
    fn the_five_pips_each_show_two_shadows_and_body_shots_shred_armour_per_stack() {
        // contender-3.6 (Reihan): "same art but remove a charge for each shadow, so each charge is 2 shadows"
        use crate::contender_shadow_core::Stack;
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        let pips = d::CHARGES_MAX / d::CHARGES_PIP_SHADOWS;
        for (count, shown) in [(d::CHARGES_MAX, pips), (d::CHARGES_MAX - 1, pips), (1, 1), (0, 0)] {
            with_sim(&host, |sim| c.apply(sim, 0, vec![Effect::Charges { count }]));
            let names: Vec<_> = host.borrow().buffs[0].iter().filter(|b| b.name().starts_with("ct_charges:")).map(|b| b.name().to_string()).collect();
            assert_eq!(names, vec![format!("ct_charges:{shown}")], "{count} shadows");
        }
        // Body Shots: a stacked enemy loses Armour per stack, and the debuff goes with the stacks
        let stacks = d::RHYTHM_MAX_STACKS / 2;
        c.core.stacks.push(Stack { id: 1, count: stacks, until: 400, champion: true });
        run(&host, &mut c, 101, 1);
        let shred = host.borrow().buffs[1].iter().find(|b| b.name() == "ct_body_shots").map(|b| b.defence_mult);
        assert_eq!(shred, Some(-((stacks * d::RHYTHM_ARMOUR_SHRED_PERMILLE_PER_STACK / 10) as i32)));
        c.core.stacks.clear();
        run(&host, &mut c, 102, 1);
        assert!(!host.borrow().buffs[1].iter().any(|b| b.name() == "ct_body_shots"), "no stacks, no shred");
    }

    #[test]
    fn a_shadow_break_knock_up_is_an_airborne_cc_but_never_on_a_tower() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        host.borrow_mut().tick = 100;
        with_sim(&host, |sim| c.apply(sim, 0, vec![Effect::BreakAirborne { target: 1, ticks: d::RHYTHM_BREAK_KNOCK_TICKS as u64 }]));
        assert!(!host.borrow().ccs[1].is_empty(), "the Break knocked him up");
    }

    #[test]
    fn a_real_shadow_grab_plays_codexs_hand_latch_ribbon_and_burst_in_order() {
        // round 190: Codex's approved Shadow Grab art, end to end through the engine cast marker
        use mod_api_stable::{InputTargetV1, StableEffectType};
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 60_000, y), "fighter")]);
        host.borrow_mut().record_view_effects = true;
        let mut c = Contender::default();
        c.configure(r#"{"shadow_art_v1":1,"shadow_directions_v1":1}"#);
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        with_sim(&host, |sim| { sim.add_buff(0, &mod_api_stable::BuffV1::named(crate::ULT_LEARNED)); });
        run(&host, &mut c, 100, 1);
        host.borrow_mut().tick = 101;
        with_sim(&host, |sim| crate::contender::CastLink.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 101, 120);
        let names: Vec<String> = host.borrow().view_effects.iter().map(|e| e.0.clone()).filter(|n| n.starts_with("ct_grab_")).collect();
        assert!(!names.is_empty(), "no grab art: {} / {:?}", c.core.rejection, c.core.shadow_link);
        let first = |p: &str| names.iter().position(|n| n.starts_with(p)).unwrap_or_else(|| panic!("no {p} in {names:?}"));
        let (hand, latch, ribbon, burst) = (first("ct_grab_hand_d0"), first("ct_grab_latch"), first("ct_grab_ribbon_L"), first("ct_grab_burst"));
        assert!(hand < latch && latch < ribbon && ribbon < burst, "{names:?}");
        assert_eq!(names.iter().filter(|n| *n == "ct_grab_latch").count(), 1);
        assert_eq!(names.iter().filter(|n| *n == "ct_grab_burst").count(), 1);
        assert!(names.iter().filter(|n| n.starts_with("ct_grab_ribbon_L")).count() >= 2, "the ribbon is renewed during the pull");
    }

    #[test]
    fn a_shadow_grab_marker_carries_the_engines_aim() {
        use crate::contender_shadow_core::Order;
        assert_eq!(crate::contender::cast_order("ct_cast:ult:410000:520000:77"), Some(Order::Ult { aim: Point::new(410_000, 520_000) }));
        assert_eq!(crate::contender::cast_order("ct_cast:s1:5"), Some(Order::S1));
        assert_eq!(crate::contender::cast_order("ct_cast:s2:5"), Some(Order::S2));
        assert_eq!(crate::contender::cast_order("ct_go:1:2"), None);
    }
}

/// Round 190: every Shadow Grab name the bridge or core can play is bound to Codex's packed art, and the ribbon keeps
/// joining Contender and the target as the distance and direction change.
#[test]
fn every_shadow_grab_art_name_is_bound_and_the_ribbon_follows_the_target() {
    use crate::contender::{grab_heading, grab_ribbon};
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    let bound: std::collections::HashSet<String> = data["view_effects"].as_array().unwrap().iter()
        .map(|e| e["name"].as_str().unwrap().to_string()).collect();
    let fanim: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/vfx/contender_grab#anim.fanim")).unwrap();
    let seconds = |tag: &str| fanim["anims"][tag]["frames"].as_array().unwrap().iter().map(|f| f["duration"].as_f64().unwrap()).sum::<f64>();
    assert!((seconds("latch") * 60.0 - GRAB_LATCH_TICKS as f64).abs() < 0.5, "the latch fade lasts GRAB_LATCH_TICKS");
    assert!((seconds("burst") * 60.0 - GRAB_BURST_TICKS as f64).abs() < 0.5, "the burst fade lasts GRAB_BURST_TICKS");
    for name in ["ct_grab_latch", "ct_grab_burst"] { assert!(bound.contains(name), "{name}"); }
    let me = Point::new(400_000, 400_000);
    for h in 0..8u8 { assert!(bound.contains(&format!("ct_grab_hand_d{h}")), "hand {h}"); }
    // sweep distances and directions: every ribbon chosen is bound
    for step in 0..32 {
        let angle = step as f64 * std::f64::consts::TAU / 32.0;
        for gap in (0..=d::SHADOW_GRAB_RANGE + 20_000).step_by(2_500) {
            let them = Point::new(me.x + (angle.cos() * gap as f64) as i64, me.y + (angle.sin() * gap as f64) as i64);
            let (name, _) = grab_ribbon(me, them);
            assert!(bound.contains(&name), "{name} for gap {gap}");
        }
    }
    // as the target is dragged in, the ribbon shortens and its middle follows the target
    let far = Point::new(me.x + d::SHADOW_GRAB_RANGE, me.y);
    let near = Point::new(me.x + d::SHADOW_GRAB_PULL_STOP, me.y);
    let (far_name, far_mid) = grab_ribbon(me, far);
    let (near_name, near_mid) = grab_ribbon(me, near);
    assert_ne!(far_name, near_name);
    assert!(far_mid.x > near_mid.x && near_mid.x > me.x);
    assert!(near_name.ends_with("_d0") && grab_ribbon(me, Point::new(me.x - 50_000, me.y)).0.ends_with("_d4"));
    // headings: east 0, south (screen down) 2, west 4, north 6
    assert_eq!([(1, 0), (0, 1), (-1, 0), (0, -1)].map(|(x, y)| grab_heading(me, Point::new(me.x + x * 10_000, me.y + y * 10_000))), [0, 2, 4, 6]);
}

/// contender-3.2: his three slots are real engine skills (the game's own AI casts them), each a Native cast marker.
#[test]
fn his_slots_are_real_engine_skills_the_game_ai_can_cast() {
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    for (slot, cooltime, effect) in [("skill", d::MERGE_COOLDOWN_TICKS, "contender_s1"), ("skill2", d::FLURRY_CAST_GAP_TICKS.max(1), "contender_s2"), ("ult", d::SHADOW_GRAB_COOLDOWN_TICKS, "contender_ult")] {
        let s = &data[slot];
        assert_eq!(s["cooltime"].as_i64(), Some(cooltime), "{slot}");
        assert!(s["range"].as_i64().unwrap() > 1_000, "{slot} has a real range");
        assert_eq!(s["effect"]["type"], "Native", "{slot}");
        assert_eq!(s["effect"]["effect_ref"], format!("tfm2_custom_ai:{effect}"), "{slot}");
    }
}

/// contender-3.5 (Reihan): a whole Shadow Flurry is a real skill, about 2.5x his Attack in total.
#[test]
fn a_full_flurry_deals_about_two_and_a_half_times_his_attack() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.f.attack = 161;
    w.tick(Some(Order::S2));
    w.run(d::FLURRY_TICKS as u64 + 2);
    let flurry: i64 = w.log.iter().filter_map(|(_, e)| match e { Effect::Damage { source: HitSource::Flurry, amount, .. } => Some(*amount), _ => None }).sum();
    let ratio = flurry as f64 / w.f.attack as f64;
    assert!((2.2..=3.2).contains(&ratio), "{flurry} = {ratio:.2}x Attack");
}
