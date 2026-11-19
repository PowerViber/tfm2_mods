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

/// A tiny world: him and the units, with his effects applied the way the bridge and engine would. The ult starts ready
/// (`ult_at` 0) so the flurry fixtures can cast at once.
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
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
                shadow_link_enabled: true,
                ..Frame::default()
            },
            log: vec![],
        }
    }
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.order = order;
        let out = self.core.step(&self.f);
        self.f.basic_hits.clear();
        self.f.incoming.clear();
        let tick = self.f.tick;
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::Damage { target, amount } | Effect::ShadowPayout { target, amount } => {
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
// ------------------------------------------------------------------ dodges and free recasts

#[test]
fn a_champion_hit_spends_his_dodge_and_he_slips_aside() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    let from = Point::new(430_000, 500_000);
    assert!(w.core.try_dodge(w.f.tick, from));
    assert!(!w.core.try_dodge(w.f.tick + 1, from), "his one dodge is spent");
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1);
    let before = w.f.pos;
    w.tick(None);
    assert!(w.f.pos.dist(before) > 5_000.0 && w.f.pos.x < before.x + 1, "he slips aside, away from the hit: {:?}", w.f.pos);
    assert!(w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_swap")), "a shadow stays where he stood");
    w.run(d::CHARGES_RECHARGE_TICKS as u64);
    assert!(w.core.try_dodge(w.f.tick, from), "it recharges");
}

#[test]
fn five_dodges_come_back_one_at_a_time_every_15_seconds_and_basics_dont_speed_them() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    assert_eq!(d::CHARGES_MAX, 5);
    for _ in 0..d::CHARGES_MAX {
        w.f.incoming.push(Point::new(430_000, 500_000));
        w.run(d::CHARGES_DODGE_GAP_TICKS as u64);
    }
    assert_eq!(w.core.charges, 0);
    assert!(!w.core.try_dodge(w.f.tick, Point::new(430_000, 500_000)), "none left: the hit lands");
    let due = w.core.charge_at;
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.charge_at, due, "basics don't speed the recharge");
    while w.f.tick <= due {
        w.tick(None);
    }
    assert_eq!(w.core.charges, 1, "one back");
    w.run(d::CHARGES_RECHARGE_TICKS as u64);
    assert_eq!(w.core.charges, 2, "and another 15 s later");
}

#[test]
fn cyclone_recast_is_free_once_and_shadow_link_has_no_recast() {
    let aim = Point::new(430_000, 500_000);
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S3 { aim }));
    assert_eq!(w.core.charges, d::CHARGES_MAX, "no charge spent");
    w.tick(Some(Order::S3 { aim }));
    assert_eq!(w.core.rejection, "s3: not ready");
    w.tick(Some(Order::S2 { aim }));
    w.tick(Some(Order::S2 { aim }));
    assert!(w.core.rejection.is_empty(), "{}", w.core.rejection);
    assert_eq!(w.core.charges, d::CHARGES_MAX, "the recast is free");
    w.tick(Some(Order::S2 { aim }));
    assert_eq!(w.core.rejection, "s2");
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S2 { aim }));
    w.run(d::CHARGES_RECAST_WINDOW_TICKS as u64 + 1);
    w.tick(Some(Order::S2 { aim }));
    assert_eq!(w.core.rejection, "s2", "the window closed");
}

// ------------------------------------------------------------------ the bank

#[test]
fn the_ult_is_a_120_second_cooldown_that_each_hit_cuts_by_a_hundredth_of_a_second() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::Ult));
    assert!(w.core.flurrying());
    let cast = w.f.tick - 1;
    assert_eq!(w.core.ult_at, cast + d::ULT_COOLDOWN_TICKS as u64);
    w.run(d::ULT_TICKS as u64 + 3 * d::PACT_DELAY_TICKS as u64);
    assert_eq!(w.core.ult_cut_centis, 0, "flurry punches, their echoes and copies never cut it");
    w.tick(Some(Order::Ult));
    assert_eq!(w.core.rejection, "ult", "on cooldown");
    // 6,000 hits take 60 s off
    w.core.ult_cut_centis = 6_000;
    assert_eq!(w.core.ult_ready_at(), cast + d::ULT_COOLDOWN_TICKS as u64 - 3_600);
    w.core.ult_cut_centis = 0;
    // one basic (and its echo) cut 0.02 s
    w.f.basic_hits.push(FOE);
    w.tick(None);
    w.run(d::PACT_DELAY_TICKS as u64 + 1);
    assert_eq!(w.core.ult_cut_centis, 2 * d::ULT_CUT_CENTIS_PER_HIT as u64);
    while w.f.tick < w.core.ult_ready_at() {
        w.tick(None);
    }
    w.tick(Some(Order::Ult));
    assert!(w.core.flurrying(), "ready again after the cooldown");
}

#[test]
fn flurry_punches_never_set_off_a_shadow_break_but_his_next_hit_does() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 100_000;
    for _ in 0..d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    w.core.echoes.clear();
    let burst = |w: &World| w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_break"));
    assert_eq!(burst(&w), 0);
    w.tick(Some(Order::Ult));
    while w.core.flurrying() {
        w.tick(None);
    }
    assert_eq!(burst(&w), 0, "no Break inside the flurry");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::RHYTHM_MAX_STACKS);
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(burst(&w), 1, "his next basic Breaks");
}

#[test]
fn every_hit_echoes_a_second_later_with_half_its_crowd_control() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.f.attack = 10_000; // so the small echo of a small skill hit is a whole number
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    w.run(d::CYCLONE_EVERY_TICKS as u64);
    let first = w.log.iter().find_map(|(t, e)| matches!(e, Effect::Damage { target: FOE, .. }).then_some(*t)).expect("a cleave hit");
    w.run(d::PACT_DELAY_TICKS as u64 + 2);
    assert!(w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo")));
    let slow = d::CYCLONE_SLOW_TICKS as u64 * d::PACT_CC_PCT as u64 / 100;
    assert!(w.seen(|e| matches!(e, Effect::Slow { target: FOE, ticks, .. } if *ticks == slow)), "the echo carries the slow, halved");
    assert!(w.log.iter().any(|(t, e)| *t == first + d::PACT_DELAY_TICKS as u64 && matches!(e, Effect::Damage { target: FOE, .. })));
}

#[test]
fn recast_hits_echo_once_and_echoed_crowd_control_is_capped() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    w.run(3 * d::PACT_DELAY_TICKS as u64);
    let knockup = d::CYCLONE_RECAST_KNOCK_TICKS as u64;
    let ups: Vec<_> = w.log.iter().filter_map(|(at, effect)| match effect {
        Effect::KnockUp { target: FOE, ticks } => Some((*at, *ticks)),
        _ => None,
    }).collect();
    assert_eq!(ups.len(), 2, "Cyclone finisher and one echo");
    assert_eq!(ups[0].1, knockup);
    assert_eq!(ups[1], (ups[0].0 + d::PACT_DELAY_TICKS as u64, knockup * d::PACT_CC_PCT as u64 / 100));
    assert!(ups[1].1 <= d::PACT_CC_CAP_TICKS as u64);
}

// ------------------------------------------------------------------ Shadow Rhythm

#[test]
fn stacks_make_the_target_take_2_percent_more_each_and_20_trigger_a_shadow_break_that_resets_them() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.unit(FOE).hp = 100_000;
    w.unit(FOE).max_hp = 100_000;
    let burst = 100_000 * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO;
    let breaks = |w: &World| w.count(|e| matches!(e, Effect::Damage { target: FOE, amount } if *amount == burst));
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
    assert_eq!(breaks(&w), 1, "the 21st hit is a Shadow Break: 6% max HP + 30% Attack");
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0, "its stacks reset to 0");
}

#[test]
fn after_a_shadow_break_his_rhythm_buffs_stay_at_full_for_8_seconds() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.unit(FOE).hp = 100_000;
    w.unit(FOE).max_hp = 100_000;
    for _ in 0..=d::RHYTHM_MAX_STACKS {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    let broke = w.f.tick - 1;
    let full = d::RHYTHM_MAX_STACKS * d::RHYTHM_ATTACK_SPEED_PERMILLE_PER_STACK / 10;
    // the echoes of those basics land a second later and stack the target again; drop them to see the buff alone
    w.core.echoes.clear();
    w.run(d::RHYTHM_BREAK_BUFF_TICKS as u64 - 60);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0);
    let last = w.log.iter().rev().find_map(|(_, e)| match e { Effect::Rhythm { attack_speed, .. } => Some(*attack_speed), _ => None });
    assert_eq!(last, Some(full), "7 s on, still the 20-stack buff");
    while w.f.tick < broke + d::RHYTHM_BREAK_BUFF_TICKS as u64 + 2 {
        w.tick(None);
    }
    let last = w.log.iter().rev().find_map(|(_, e)| match e { Effect::Rhythm { attack_speed, .. } => Some(*attack_speed), _ => None });
    assert_eq!(last, Some(0), "gone after 8 s");
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
    w.tick(Some(Order::S1 { aim: Point::new(410_000, 500_000) }));
    w.f.basic_hits.push(FOE);
    w.tick(None);
    let echo = w.f.attack * d::PACT_PCT / 100;
    assert!(w.core.echoes.iter().filter(|e| e.target == FOE).all(|e| e.amount == echo));
    assert_eq!(w.core.echoes.iter().filter(|e| e.target == FOE).count(), 2);
}

/// Reihan's emergency nerf (contender-2.5): ten seconds of basics on a 1,500 HP champion (no items, level 1) must stay
/// near a plain champion's basic damage, merged or not.
#[test]
fn ten_seconds_of_basics_stay_under_1_2_times_a_plain_champion() {
    for merged in [false, true] {
        let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
        w.unit(FOE).hp = 1_000_000;
        w.unit(FOE).max_hp = 1_500;
        let rules: serde_json::Value = serde_json::from_str(include_str!("../../../rules/contender.json")).unwrap();
        let attack = rules["stat"]["attack"].as_i64().unwrap() * (100 + d::MOD_POWER[0] as i64) / 100;
        w.f.attack = attack;
        if merged {
            w.tick(Some(Order::S1 { aim: Point::new(410_000, 500_000) }));
        }
        let (mut speed, mut next, mut basics, mut total) = (0i64, w.f.tick, 0i64, 0i64);
        let start = w.f.tick;
        while w.f.tick < start + 600 {
            if w.f.tick >= next {
                total += attack * (1_000 + w.core.vuln_permille(FOE, w.f.tick)) / 1_000;
                w.f.basic_hits.push(FOE);
                next = w.f.tick + (d::BASIC_COOLDOWN_TICKS * 100 / (100 + speed)) as u64;
                basics += 1;
            }
            for e in w.tick(None) {
                match e {
                    Effect::Damage { target: FOE, amount } => total += amount,
                    Effect::Rhythm { attack_speed, .. } => speed = attack_speed,
                    _ => {}
                }
            }
        }
        let plain = (600 / d::BASIC_COOLDOWN_TICKS + 1) * attack;
        let ratio = total as f64 / plain as f64;
        assert!(basics > 0 && ratio > 1.0 && ratio <= 1.2, "merged {merged}: {total} vs a plain {plain} ({ratio:.2}x)");
    }
}


/// Reihan, after 2.5: "its still too strong, like really really strong". His own AI in a 1v2 for 20 s, the enemies
/// basic-attacking him every 0.9 s: his total damage (basics + skills + echoes + copies + Breaks + the ult) stays near
/// a plain champion's basic damage, and his dodges undo at most about a third of the champion hits.
fn fight_1v2(level: i64) -> (f64, f64) {
    let rules: serde_json::Value = serde_json::from_str(include_str!("../../../rules/contender.json")).unwrap();
    let attack = (rules["stat"]["attack"].as_i64().unwrap() + rules["growth"]["attack"].as_i64().unwrap() * (level - 1)) * (100 + d::MOD_POWER[0] as i64) / 100;
    let mut w = World::new(vec![champ(FOE, 2, 425_000, 500_000), champ(11, 2, 440_000, 510_000)]);
    for u in w.f.units.iter_mut() {
        u.hp = 1_000_000;
        u.max_hp = 1_500;
    }
    w.f.manual = false;
    w.f.level = level;
    w.f.attack = attack;
    let (mut speed, mut next, mut total, mut hits, mut dodged) = (0i64, 1u64, 0i64, 0, 0);
    let start = w.f.tick;
    while w.f.tick < start + 1_200 {
        if w.f.tick >= next {
            total += attack * (1_000 + w.core.vuln_permille(FOE, w.f.tick)) / 1_000;
            w.f.basic_hits.push(FOE);
            next = w.f.tick + (d::BASIC_COOLDOWN_TICKS * 100 / (100 + speed)) as u64;
        }
        let incoming = (w.f.tick - start + 1) % 54 == 0;
        if incoming {
            hits += 1;
            w.f.incoming.push(Point::new(425_000, 500_000));
        }
        let tick = w.f.tick;
        for e in w.tick(None) {
            match e {
                Effect::Damage { amount, .. } => total += amount,
                Effect::Rhythm { attack_speed, .. } => speed = attack_speed,
                _ => {}
            }
        }
        if incoming && w.core.last_dodge == tick {
            dodged += 1;
        }
        w.f.pos = Point::new(400_000, 500_000);
    }
    let plain = (1_200 / d::BASIC_COOLDOWN_TICKS + 1) * attack;
    (total as f64 / plain as f64, dodged as f64 / hits as f64)
}

#[test]
fn in_a_1v2_his_damage_stays_under_1_4x_plain_basics_and_he_dodges_at_most_30_percent() {
    for level in [3, 9] {
        let (damage, dodges) = fight_1v2(level);
        println!("level {level}: {damage:.2}x, {:.0}% dodged", dodges * 100.0);
        assert!(damage <= 1.4, "level {level}: {damage:.2}x a plain champion's basics");
        assert!(dodges <= 0.30, "level {level}: {:.0}% of champion hits dodged", dodges * 100.0);
    }
}

// ------------------------------------------------------------------ S1 Shadow Merge

#[test]
fn merge_doubles_echoes_and_adds_no_dodges() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(410_000, 500_000) }));
    assert!(w.core.merged(w.f.tick));
    assert_eq!(w.core.charges, d::CHARGES_MAX + d::MERGE_BONUS_DODGES);
    assert!(w.seen(|e| matches!(e, Effect::Merge { move_pct, .. } if *move_pct == d::MERGE_MOVE_SPEED_PCT)));
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::MERGE_STACKS_PER_HIT, "one stack a hit");
    assert_eq!(w.core.echoes.iter().filter(|e| e.target == FOE).count(), 2, "every hit echoes twice");
    // Merge adds no dodges; the 0.4 s gap between dodges still holds
    assert!(w.core.try_dodge(w.f.tick, Point::new(430_000, 500_000)));
    assert!(!w.core.try_dodge(w.f.tick + 1, Point::new(430_000, 500_000)));
    assert!(w.core.try_dodge(w.f.tick + d::CHARGES_DODGE_GAP_TICKS as u64, Point::new(430_000, 500_000)));
    w.run(d::MERGE_TICKS as u64);
    assert!(!w.core.merged(w.f.tick));
    assert!(w.core.charges <= d::CHARGES_MAX, "never more than his five");
}

#[test]
fn stepping_out_crosses_the_nearest_enemy_and_slips_away() {
    let mut w = World::new(vec![champ(FOE, 2, 412_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(412_000, 500_000) }));
    w.run(10);
    let before = w.f.pos;
    w.tick(Some(Order::S1 { aim: Point::new(412_000, 500_000) }));
    assert!(!w.core.merged(w.f.tick));
    assert!(w.seen(|e| matches!(e, Effect::Knock { target: FOE, .. })), "the shadow's cross");
    assert!(w.f.pos.x < before.x, "he slips away from it: {:?}", w.f.pos);
    assert!(w.seen(|e| matches!(e, Effect::Invulnerable { .. })));
}

#[test]
fn a_flurry_cast_while_merged_no_longer_echoes() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::S1 { aim: Point::new(430_000, 500_000) }));
    w.tick(Some(Order::Ult));
    assert!(w.core.flurry.is_some_and(|fl| !fl.echo));
    let before = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo"));
    w.run(d::ULT_TICKS as u64 + d::PACT_DELAY_TICKS as u64 + 2);
    let echoes = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo"));
    assert_eq!(echoes, before, "no flurry echoes");
}

// ------------------------------------------------------------------ S2 Shadow Cyclone

#[test]
fn the_clone_cleaves_four_times_and_the_aimed_recast_cleaves_a_bigger_circle_pulling_them_in() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000), creep(30, 455_000, 505_000, 9_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    w.run(d::CYCLONE_HITS as u64 * d::CYCLONE_EVERY_TICKS as u64 + 1);
    assert_eq!(w.count(|e| matches!(e, Effect::Slow { target: FOE, ticks, .. } if *ticks == d::CYCLONE_SLOW_TICKS as u64)), d::CYCLONE_HITS as usize);
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: 30, .. })), "creeps too");
    // the recast lands somewhere else in reach: a foe 30,000 off that spot is inside the bigger circle and is pulled in
    let mut w = World::new(vec![champ(FOE, 2, 470_000, 530_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(440_000, 500_000) }));
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 510_000) }));
    assert!(w.f.pos.within(Point::new(450_000, 510_000), 1_000), "he went where the recast aimed, not the clone's spot");
    assert!(w.seen(|e| matches!(e, Effect::Invulnerable { .. })));
    let start = Point::new(470_000, 530_000);
    w.run(8 + d::CYCLONE_RECAST_HITS as u64 * d::CYCLONE_RECAST_EVERY_TICKS as u64 + 2);
    let now = w.f.units[0].pos;
    assert!(now.dist(Point::new(450_000, 510_000)) < start.dist(Point::new(450_000, 510_000)), "pulled toward the centre: {now:?}");
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::CYCLONE_RECAST_KNOCK_TICKS as u64)));
}

#[test]
fn the_recast_cannot_reach_past_60000() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    w.tick(Some(Order::S2 { aim: Point::new(800_000, 500_000) }));
    assert!(w.f.pos.x <= 400_000 + d::CYCLONE_RECAST_RANGE + 1);
}

// ------------------------------------------------------------------ S3 Shadow Link

fn land(w: &mut World, aim: Point, accept: &[usize]) -> (u64, Point, u64) {
    let cast = w.f.tick;
    w.tick(Some(Order::S3 { aim }));
    let Some(ShadowLink::Flight { at, landing, .. }) = w.core.shadow_link else {
        panic!("expected flight: {}", w.core.rejection)
    };
    w.f.tick = landing;
    let effects = w.tick(None);
    assert!(effects.iter().any(|effect| matches!(effect, Effect::ShadowLand { cast: id, .. } if *id == cast)));
    w.f.shadow_observation = Some(ShadowObservation::Landed { cast, targets: accept.to_vec() });
    w.tick(None);
    let Some(ShadowLink::Active { until, .. }) = w.core.shadow_link else { panic!("expected active shadow") };
    (cast, at, until)
}

#[test]
fn shadow_uses_highest_max_hp_and_pays_capped_flat_damage_once_on_expiry() {
    let mut tank = champ(FOE, 2, 450_000, 500_000);
    tank.hp = 2500;
    tank.max_hp = 3000;
    let mut assassin = champ(11, 2, 452_000, 512_000);
    assassin.max_hp = 1200;
    let mut w = World::new(vec![tank, assassin, creep(30, 443_000, 500_000, 650)]);
    let (cast, at, until) = land(&mut w, Point::new(600_000, 500_000), &[FOE, 11, 30]);
    assert_eq!(at, Point::new(450_000, 500_000));
    assert!(matches!(w.core.shadow_link, Some(ShadowLink::Active { max_hp: 3000, hp: 3000, .. })));
    assert_eq!(until, cast + d::SHADOW_LINK_FLIGHT_TICKS as u64 + d::SHADOW_LINK_TICKS as u64);
    w.f.shadow_observation = Some(ShadowObservation::Health { cast, hp: 1500 });
    assert!(!w.tick(None).iter().any(|effect| matches!(effect, Effect::ShadowPayout { .. })));
    w.f.tick = until;
    let payout: Vec<_> = w.tick(None).into_iter().filter_map(|effect| match effect {
        Effect::ShadowPayout { target, amount } => Some((target, amount)),
        _ => None,
    }).collect();
    assert_eq!(payout, vec![(FOE, 900), (11, 360), (30, 195)]);
    assert!(w.core.shadow_link.is_none() && w.core.stacks.is_empty() && w.core.echoes.is_empty());
    assert_eq!(w.core.ult_cut_centis, 0);
    assert!(!w.tick(None).iter().any(|effect| matches!(effect, Effect::ShadowPayout { .. })));
}

#[test]
fn shadow_resolves_exact_link_boundary_and_partition_before_depletion() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000), champ(11, 2, 451_000, 500_000), champ(12, 2, 452_000, 500_000)]);
    let (cast, at, _) = land(&mut w, Point::new(450_000, 500_000), &[FOE, 11, 12]);
    w.unit(FOE).pos.x = at.x + d::SHADOW_LINK_BREAK_RANGE;
    w.unit(11).pos.x = at.x + d::SHADOW_LINK_BREAK_RANGE + 1;
    w.f.separated = vec![12];
    w.f.shadow_observation = Some(ShadowObservation::Health { cast, hp: 0 });
    let effects = w.tick(None);
    assert!(effects.iter().any(|effect| matches!(effect, Effect::ShadowPayout { target: FOE, amount: 300 })));
    assert_eq!(effects.iter().filter(|effect| matches!(effect, Effect::ShadowPayout { .. })).count(), 1);
}

#[test]
fn shadow_stops_at_wall_and_skips_immune_tower_and_unconfirmed_targets() {
    let mut tower = champ(50, 2, 420_000, 500_000);
    tower.tower = true;
    tower.champion = false;
    let mut w = World::new(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 425_000, 500_000), creep(30, 435_000, 500_000, 650), tower]);
    w.f.link_excluded = vec![11];
    w.f.barriers = vec![Barrier { from: Point::new(429_000, 480_000), to: Point::new(429_000, 520_000), half_width: 2000 }];
    let (cast, at, _) = land(&mut w, Point::new(450_000, 500_000), &[11, 30, 50]);
    assert!(at.x < 427_000);
    assert!(matches!(&w.core.shadow_link, Some(ShadowLink::Active { linked, max_hp: 650, .. }) if linked == &[30]));
    w.f.shadow_observation = Some(ShadowObservation::Health { cast, hp: 100 });
    w.f.alive = false;
    let effects = w.tick(None);
    assert!(effects.iter().any(|effect| matches!(effect, Effect::ShadowEnd { paid: false, .. })));
    assert!(!effects.iter().any(|effect| matches!(effect, Effect::ShadowPayout { .. })));
}

#[test]
fn native_feature_gate_leaves_ordinary_build_without_s3_commitment() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000)]);
    w.f.shadow_link_enabled = false;
    w.tick(Some(Order::S3 { aim: Point::new(450_000, 500_000) }));
    assert_eq!(w.core.cd[2], 0);
    assert!(w.core.shadow_link.is_none());
    assert!(w.core.rejection.contains("verification pending"));
}

// ------------------------------------------------------------------ Ult Shadow Flurry

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
fn the_flurry_vanishes_him_chains_72_steady_punches_and_brings_him_back_by_the_last() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 450_000, 500_000), creep(30, 470_000, 500_000, 90_000)]);
    for u in w.f.units.iter_mut() {
        u.hp = 90_000;
        u.max_hp = 90_000;
    }
    w.core.charges = 1;
    w.tick(Some(Order::Ult));
    assert!(w.core.flurrying());
    w.run(d::ULT_TICKS as u64);
    assert!(!w.core.flurrying());
    assert_eq!(w.core.charges, 1, "contender-2.6: the flurry no longer refills his dodges");
    let punches = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry"));
    assert_eq!(punches, d::ULT_PUNCHES as usize);
    assert_eq!(d::ULT_PUNCHES, 72);
    assert!((0..d::ULT_PUNCHES).all(|k| Core::flurry_gap(k) == d::ULT_FIRST_GAP_TICKS as u64), "a steady pace");
    let ups = w.count(|e| matches!(e, Effect::KnockUp { ticks, .. } if *ticks == d::ULT_KNOCK_TICKS as u64));
    assert!(ups >= 3, "a knock-up every second: {ups}");
    assert!(w.seen(|e| matches!(e, Effect::Teleport { .. })), "he reappears");
    assert!(w.core.haymaker_until > w.f.tick);
}

#[test]
fn the_flurry_ends_early_when_nothing_is_left_in_reach_and_echoes_only_when_merged() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::Ult));
    w.run(30);
    w.unit(FOE).pos = Point::new(800_000, 100_000); // it got away
    w.run(30);
    assert!(!w.core.flurrying(), "nothing within 30,000 of the last hit: over");
    assert!(w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry")) < d::ULT_PUNCHES as usize);
    assert!(!w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo")), "not merged: no echoes");
}

// ------------------------------------------------------------------ AI

fn ai(units: Vec<Unit>) -> World {
    let mut w = World::new(units);
    w.f.manual = false;
    w.core.ult_at = u64::MAX / 4; // the ult on cooldown
    w
}

#[test]
fn ai_uses_shadow_before_committing_and_does_not_dive_when_outnumbered() {
    let mut w = ai(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 445_000, 510_000), champ(12, 2, 440_000, 505_000), champ(13, 2, 445_000, 505_000)]);
    w.tick(None);
    assert_eq!(w.core.reason, "Poke: Shadow Link");
    assert!(matches!(w.core.shadow_link, Some(ShadowLink::Flight { .. })));
    w.run(30);
    assert!(!w.core.merged(w.f.tick));
    assert!(!w.seen(|effect| matches!(effect, Effect::Glide { .. } | Effect::Vanish { .. })));
}

#[test]
fn low_hp_chase_and_cyclone_recast_do_not_bypass_the_commit_checks() {
    for aggressive in [20, 80] {
        let mut w = ai(vec![champ(FOE, 2, 550_000, 500_000), champ(11, 2, 551_000, 500_000)]);
        w.f.hp = 100;
        w.f.profile[6] = aggressive;
        let effects = w.tick(None);
        assert_eq!(w.core.reason, "Retreat: low HP");
        assert!(effects.iter().any(|effect| matches!(effect, Effect::Steer { to } if to.x < 400_000)));
        w.f.manual = true;
        w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
        w.f.manual = false;
        w.unit(FOE).pos.x = 450_000;
        let mut tower = champ(50, 2, 495_000, 500_000);
        tower.tower = true;
        tower.champion = false;
        w.f.units.extend([champ(12, 2, 450_000, 520_000), tower]);
        let effects = w.tick(None);
        assert!(!effects.iter().any(|effect| matches!(effect, Effect::Glide { .. } | Effect::Vanish { .. })));
        assert!(!w.core.used[1], "kept free recast instead of entering a tower 1v3");
    }
}

#[test]
fn aggressive_values_change_supported_chase_without_relaxing_tower_safety() {
    for (aggressive, expected) in [(20, false), (80, true)] {
        let mut w = ai(vec![champ(FOE, 2, 475_000, 500_000), champ(ALLY, 1, 463_000, 510_000)]);
        w.f.profile[6] = aggressive;
        w.core.cd = [999; 3];
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
    w.core.cd = [500; 3];
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
fn the_ai_farms_a_camp_with_its_skills_when_calm_and_cuts_the_ult_cooldown() {
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
    for k in ["Farm: Shadow Cyclone", "Farm: Shadow Link"] {
        assert!(used.iter().any(|u| u == k), "{k}: {used:?}");
    }
    assert!(w.core.ult_cut_centis > 0, "farming with his skills cuts the ult cooldown");
}

#[test]
fn the_ai_flurries_with_the_ult_ready_on_a_group() {
    let mut w = ai(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 440_000, 505_000), champ(ALLY, 1, 420_000, 500_000), champ(3, 1, 425_000, 510_000)]);
    w.core.cd = [u64::MAX / 2; 3];
    w.core.ult_at = 0;
    w.tick(None);
    assert_eq!(w.core.reason, "Commit: supported Shadow Flurry");
    assert!(w.core.flurrying());
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
            c.core.merge_until=460;c.core.cd=[u64::MAX/2;3];
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
        // contender-2.9 (`basicsOnly` 0): the fighter cast its skill 0.5 s ago, and the skill hit is dodged too
        assert_eq!(crate::contender_shadow_data::CHARGES_BASICS_ONLY, 0);
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
        let effects = vec![Effect::Damage { target: 2, amount: 50 }, Effect::Stun { target: 2, ticks: 30 }];
        with_sim(&host, |sim| c.apply(sim, 0, effects));
        let g = host.borrow();
        assert!(!g.damage.iter().any(|d| d.1 == 2 && d.3 == AttackTypeV1::Skill.code()) && g.ccs[2].is_empty(), "never a tower");
    }

    #[test]
    fn a_tick_on_the_host_runs_clean_with_the_ai_fighting() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 40_000, y), "fighter"), champ(1, (x + 45_000, y + 8_000), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 120);
        assert!(host.borrow().buffs[0].iter().any(|b| b.name() == "ct_merge") || c.core.merged(220) || c.core.shadow_link.is_some() || c.core.cd != [0; 3], "{}", c.core.reason);
        let _ = Point::default();
    }
}
