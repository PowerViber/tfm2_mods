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
                Effect::Glide { to, .. } | Effect::Teleport { to } => self.f.pos = *to,
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
    w.core.ult_at = 0;
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
    assert_eq!(w.core.charges, 4);
    assert!(w.core.stacks.is_empty() && w.core.echoes.is_empty());
    assert_eq!(w.unit(11).hp, 1000);
    w.run(24);
    w.f.incoming.push(IncomingHit { counter: true, ..hit });
    assert!(!w.tick(None).iter().any(|e| matches!(e, Effect::Damage { .. })));
    assert_eq!(w.core.charges, 3, "incoming Counterpunch may still be dodged");
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
    assert!(out.contains(&Effect::Damage { target: FOE, amount: 70, source: HitSource::Counter }));
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0);
}

#[test]
fn moving_merge_cross_checks_the_50k_boundary_and_credits_each_target_once() {
    let mut w = World::new(vec![
        champ(FOE, 2, 440_000, 500_000), champ(11, 2, 493_000, 500_000),
        creep(30, 402_000, 500_000, 1000), champ(12, 2, 493_001, 500_000),
    ]);
    w.core.flurry_charges = 2;
    w.core.flurry_charge_at = 500;
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
    let cds = w.core.flurry_charge_at;
    w.tick(None);
    assert_eq!(w.core.flurry_charge_at, cds - 135);
    assert!(w.core.stacks.is_empty() && w.core.echoes.is_empty());
}

#[test]
fn tenth_punch_at_sixty_ticks_and_cast_gap_is_exactly_forty_five() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S2));
    assert_eq!(w.core.flurry_charges, 2);
    assert_eq!(w.core.flurry_charge_at, 961);
    w.run(59);
    assert_eq!(w.count(|e| matches!(e, Effect::Damage { source: HitSource::Flurry, .. })), 9);
    let end = w.tick(None);
    assert!(end.iter().any(|e| matches!(e, Effect::Damage { source: HitSource::Flurry, .. })));
    assert!(end.iter().any(|e| matches!(e, Effect::Teleport { .. })));
    assert_eq!(w.core.flurry_cast_at, 106);
    assert_eq!(w.core.flurry_credited_ticks, 0);
    assert!(w.core.echoes.is_empty());
    w.run(43);
    w.tick(Some(Order::S2));
    assert!(!w.core.flurrying());
    w.tick(Some(Order::S2));
    assert!(w.core.flurrying());
}

#[test]
fn trading_a_dodge_grants_one_flurry_charge_without_triggering_a_dodge() {
    let mut w = World::new(vec![]);
    w.core.flurry_charges = 1;
    w.core.flurry_charge_at = 500;
    w.core.flurry_cast_at = 75;
    w.core.last_dodge = 1;
    let effects = w.tick(Some(Order::ExchangeDodge));
    assert_eq!((w.core.charges, w.core.flurry_charges), (4, 2));
    assert_eq!((w.core.charge_at, w.core.flurry_charge_at), (901, 500));
    assert_eq!((w.core.last_dodge, w.core.flurry_cast_at), (1, 75));
    assert!(!w.core.flurrying() && w.core.pending_dodge.is_none());
    assert!(!effects.iter().any(|effect| matches!(effect, Effect::Glide { .. } | Effect::Damage { .. })));

    w.tick(Some(Order::ExchangeDodge));
    assert_eq!((w.core.charges, w.core.flurry_charges), (3, 3));
    assert_eq!((w.core.charge_at, w.core.flurry_charge_at), (901, 0));
    w.tick(Some(Order::ExchangeDodge));
    assert_eq!((w.core.charges, w.core.flurry_charges), (3, 3));
    assert!(w.core.rejection.contains("full"));
}

#[test]
fn dodge_exchange_respects_the_learning_gate_and_requires_a_dodge() {
    let mut w = World::new(vec![]);
    w.core.flurry_charges = 2;
    w.core.flurry_charge_at = 500;
    w.f.level = 1;
    w.tick(Some(Order::ExchangeDodge));
    assert!(w.core.rejection.contains("level 2"));
    assert_eq!((w.core.charges, w.core.flurry_charges), (5, 2));
    w.f.level = 2;
    w.core.charges = 0;
    w.core.charge_at = 500;
    w.tick(Some(Order::ExchangeDodge));
    assert!(w.core.rejection.contains("no passive dodge"));
    assert_eq!((w.core.charges, w.core.flurry_charges), (0, 2));
}

#[test]
fn ai_spends_a_dodge_only_for_a_safe_flurry_when_out_of_charges() {
    let mut w = ai(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 447_000, 500_000)]);
    w.f.ult_unlocked = false;
    w.core.flurry_charges = 0;
    w.core.flurry_charge_at = 500;
    w.tick(None);
    assert_eq!(w.core.charges, 5);
    assert!(!w.core.flurrying(), "no follow-up");
    w.f.units.push(champ(ALLY, 1, 435_000, 500_000));
    w.core.flurry_cast_at = w.f.tick + 1;
    w.tick(None);
    assert_eq!(w.core.charges, 5, "the cast gap still applies");
    w.tick(None);
    assert!(w.core.flurrying(), "{}", w.core.reason);
    assert_eq!((w.core.charges, w.core.flurry_charges), (4, 0));
    assert_eq!(w.core.flurry_charge_at, 500);
}

#[test]
fn delayed_independent_echo_counts_even_during_flurry_and_no_credit_banks_at_cap() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.f.landed_hits = 2;
    w.tick(None);
    assert_eq!(w.core.flurry_credited_ticks, 0);
    w.f.basic_hits.push(FOE);
    w.tick(None);
    w.tick(Some(Order::S2));
    w.run(60);
    assert_eq!(w.core.flurry_charge_at, 933);
    w.core.flurry_charges = 1;
    w.core.flurry_charge_at = w.f.tick + 8;
    w.f.landed_hits = 2;
    let tick = w.f.tick;
    w.tick(None);
    assert_eq!(w.core.flurry_charges, 2);
    assert_eq!(w.core.flurry_charge_at, tick + 930);
}

#[test]
fn link_refills_dodges_permanently_drops_escapes_then_pulls_and_chains_control() {
    let mut w = World::new(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 460_000, 500_000),
        creep(30, 420_000, 510_000, 1000)]);
    w.core.charges = 0;
    w.core.charge_at = 500;
    w.core.last_dodge = 1;
    w.core.flurry_charges = 2;
    w.core.flurry_charge_at = 550;
    let aim = Point::new(436_000, 500_000);
    w.tick(Some(Order::Ult { aim }));
    assert_eq!(w.core.charges, 5);
    assert_eq!(w.core.last_dodge, 1);
    assert_eq!(w.core.flurry_charges, 2);
    assert_eq!(w.core.flurry_charge_at, 550);
    w.run(11);
    let Some(ShadowLink::Warning { until, linked, .. }) = &w.core.shadow_link else { panic!("warning phase") };
    assert_eq!(linked, &[10, 11, 30]);
    let until = *until;
    w.unit(11).pos.x = aim.x + 85_001;
    w.tick(None);
    w.unit(11).pos.x = aim.x + 1_000;
    while w.f.tick <= until + 42 { w.tick(None); }
    assert!(w.log.iter().any(|(tick,e)| *tick==until+12 && matches!(e, Effect::LinkAirborne { target: FOE, ticks: 30 })));
    assert!(w.log.iter().any(|(tick,e)| *tick==until+42 && matches!(e, Effect::LinkStun { target: FOE, ticks: 24 })));
    assert!(w.seen(|e| matches!(e, Effect::LinkStun { target: 30, .. })));
    assert!(!w.seen(|e| matches!(e, Effect::LinkStun { target: 11, .. } | Effect::Damage { .. })));
    assert_eq!(w.f.pos, Point::new(400_000, 500_000));
}

#[test]
fn link_drops_immune_targets_and_stops_a_pull_at_new_boundaries() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000), champ(11, 2, 453_000, 500_000)]);
    let aim = Point::new(425_000, 500_000);
    w.tick(Some(Order::Ult { aim }));
    while !matches!(w.core.shadow_link, Some(ShadowLink::Pull { .. })) { w.tick(None); }
    w.f.link_excluded.push(11);
    w.f.barriers.push(Barrier { from: Point::new(435_000, 480_000), to: Point::new(435_000, 520_000), half_width: 1000 });
    w.run(45);
    assert!(w.unit(FOE).pos.x >= 436_000);
    assert!(!w.seen(|e| matches!(e, Effect::LinkAirborne { target: 11, .. } | Effect::LinkStun { .. })));
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
        let effects = vec![Effect::Damage { target: 2, amount: 50, source: crate::contender_shadow_core::HitSource::Counter }, Effect::LinkStun { target: 2, ticks: 30 }];
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
        c.core.flurry_charges = 1;
        c.core.flurry_charge_at = 800;
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
        assert_eq!(c.core.flurry_charge_at, 695);
        assert_eq!(c.core.stacks_on(1, 101), 1);
        assert_eq!(host.borrow().damage.iter().filter(|d|d.3 == AttackTypeV1::BaseAttack.code()).count(),1);
        run(&host, &mut c, 102, 1);
        assert_eq!(c.core.flurry_charge_at, 695, "consumed receipt counts once");
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
        // then S2: a Flurry charge is spent
        let before = c.core.flurry_charges;
        host.borrow_mut().tick = 110;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 110, 1);
        assert!(c.core.flurrying() || c.core.flurry_charges < before, "Flurry started: {}", c.core.rejection);
        // a press with no charge left is refused without a panic
        c.core.flurry = None;
        c.core.flurry_charges = 0;
        c.core.charges = 5;
        host.borrow_mut().tick = 200;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 200, 1);
        assert!(!c.core.flurrying());
        assert_eq!(c.core.charges, 5, "the engine's press never trades a dodge");
    }

    #[test]
    fn a_shadow_link_marker_carries_the_engines_aim() {
        use crate::contender_shadow_core::Order;
        assert_eq!(crate::contender::cast_order("ct_cast:ult:410000:520000:77"), Some(Order::Ult { aim: Point::new(410_000, 520_000) }));
        assert_eq!(crate::contender::cast_order("ct_cast:s1:5"), Some(Order::S1));
        assert_eq!(crate::contender::cast_order("ct_cast:s2:5"), Some(Order::S2));
        assert_eq!(crate::contender::cast_order("ct_go:1:2"), None);
    }
}

/// contender-3.2: his three slots are real engine skills (the game's own AI casts them), each a Native cast marker.
#[test]
fn his_slots_are_real_engine_skills_the_game_ai_can_cast() {
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    for (slot, cooltime, effect) in [("skill", d::MERGE_COOLDOWN_TICKS, "contender_s1"), ("skill2", d::FLURRY_CAST_GAP_TICKS, "contender_s2"), ("ult", d::SHADOW_LINK_COOLDOWN_TICKS, "contender_ult")] {
        let s = &data[slot];
        assert_eq!(s["cooltime"].as_i64(), Some(cooltime), "{slot}");
        assert!(s["range"].as_i64().unwrap() > 1_000, "{slot} has a real range");
        assert_eq!(s["effect"]["type"], "Native", "{slot}");
        assert_eq!(s["effect"]["effect_ref"], format!("tfm2_custom_ai:{effect}"), "{slot}");
    }
}
