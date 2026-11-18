//! Contender 2.3's core: shadow dodges, free recasts, Shadow Pact's echoes and their cap, Shadow Rhythm's stacks and
//! Breaks, Shadow Merge, Shadow Cyclone (and its aimed, pulling recast), Shadow Tether, the banked Shadow Flurry and his
//! AI; then the bridge on the glue mock host.
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

/// A tiny world: him and the units, with his effects applied the way the bridge and engine would. The bank starts full
/// so the flurry fixtures can cast at once (the bank tests empty it).
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
}
impl World {
    fn new(units: Vec<Unit>) -> World {
        let mut core = Core::default();
        core.bank = d::ULT_BANK_MAX;
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
fn pct(base: i64, ratio: i64, ad: i64) -> i64 {
    base + ratio * ad / 100
}

// ------------------------------------------------------------------ dodges and free recasts

#[test]
fn a_champion_hit_spends_a_dodge_and_he_slips_aside_once_per_gap() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    let from = Point::new(430_000, 500_000);
    assert!(w.core.try_dodge(w.f.tick, from));
    assert!(!w.core.try_dodge(w.f.tick + 1, from), "one dodge per 0.4 s");
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1);
    let before = w.f.pos;
    w.tick(None);
    assert!(w.f.pos.dist(before) > 5_000.0 && w.f.pos.x < before.x + 1, "he slips aside, away from the hit: {:?}", w.f.pos);
    assert!(w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_swap")), "a shadow stays where he stood");
    w.run(d::CHARGES_DODGE_GAP_TICKS as u64);
    assert!(w.core.try_dodge(w.f.tick, from), "after the gap another dodge works");
}

#[test]
fn dodges_run_out_and_recharge_and_basics_speed_them_up() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    for _ in 0..d::CHARGES_MAX {
        w.f.incoming.push(Point::new(430_000, 500_000));
        w.run(d::CHARGES_DODGE_GAP_TICKS as u64);
    }
    assert_eq!(w.core.charges, 0);
    assert!(!w.core.try_dodge(w.f.tick, Point::new(430_000, 500_000)), "no dodge left: the hit lands");
    let due = w.core.charge_at;
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.charge_at, due - d::CHARGES_BASIC_CUT_TICKS as u64);
}

#[test]
fn skills_cost_nothing_and_each_recast_is_free_once_in_its_window() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 440_000, 510_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    assert_eq!(w.core.charges, d::CHARGES_MAX, "no charge spent");
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    assert!(w.core.rejection.is_empty(), "{}", w.core.rejection);
    assert_eq!(w.core.charges, d::CHARGES_MAX, "the recast is free");
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    assert_eq!(w.core.rejection, "s3", "used: back on cooldown");
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    w.run(d::CHARGES_RECAST_WINDOW_TICKS as u64 + 1);
    w.tick(Some(Order::S2 { aim: Point::new(430_000, 500_000) }));
    assert_eq!(w.core.rejection, "s2", "the window closed");
}

// ------------------------------------------------------------------ the bank

#[test]
fn hits_outside_the_flurry_bank_up_to_40_and_the_ult_spends_them_all() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.core.bank = 0;
    w.tick(Some(Order::Ult));
    assert_eq!(w.core.rejection, "ult", "an empty bank: no ult");
    for _ in 0..d::ULT_BANK_MAX - 1 {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    w.tick(Some(Order::Ult));
    assert_eq!(w.core.rejection, "ult", "39 banked: not yet");
    for _ in 0..6 {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    assert_eq!(w.core.bank, d::ULT_BANK_MAX, "capped at 40");
    w.tick(Some(Order::Ult));
    assert!(w.core.flurrying());
    assert_eq!(w.core.bank, 0, "all 20 spent");
    w.run(d::ULT_TICKS as u64 + 3 * d::PACT_DELAY_TICKS as u64);
    assert_eq!(w.core.bank, 0, "flurry punches and their echoes never bank");
}

// ------------------------------------------------------------------ Shadow Pact

#[test]
fn every_hit_echoes_a_second_later_at_40_percent_with_half_its_crowd_control() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
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
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    w.run(3 * d::PACT_DELAY_TICKS as u64);
    let ups = w.count(|e| matches!(e, Effect::KnockUp { target: FOE, .. }));
    assert_eq!(ups, 2, "the uppercut and one echo (contender-2.6)");
    let echoed: u64 = w.log.iter().filter(|(t, _)| *t > 2).filter_map(|(_, e)| match e { Effect::KnockUp { target: FOE, ticks } => Some(*ticks), _ => None }).sum();
    assert!(echoed <= d::PACT_CC_CAP_TICKS as u64);
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
fn ten_seconds_of_basics_stay_under_1_4_times_a_plain_champion() {
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
        assert!(basics > 0 && ratio > 1.0 && ratio <= 1.4, "merged {merged}: {total} vs a plain {plain} ({ratio:.2}x)");
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
    w.core.bank = 0;
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
fn in_a_1v2_his_damage_stays_under_2x_plain_basics_and_he_dodges_at_most_35_percent() {
    for level in [3, 9] {
        let (damage, dodges) = fight_1v2(level);
        println!("level {level}: {damage:.2}x, {:.0}% dodged", dodges * 100.0);
        assert!(damage <= 2.0, "level {level}: {damage:.2}x a plain champion's basics");
        assert!(dodges <= 0.35, "level {level}: {:.0}% of champion hits dodged", dodges * 100.0);
    }
}

// ------------------------------------------------------------------ S1 Shadow Merge

#[test]
fn merge_doubles_the_dodge_recharge_adds_two_dodges_and_doubles_echoes_and_stacks() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]);
    w.tick(Some(Order::S1 { aim: Point::new(410_000, 500_000) }));
    assert!(w.core.merged(w.f.tick));
    assert_eq!(w.core.charges, d::CHARGES_MAX + d::MERGE_BONUS_DODGES);
    assert!(w.seen(|e| matches!(e, Effect::Merge { move_pct, .. } if *move_pct == d::MERGE_MOVE_SPEED_PCT)));
    w.f.basic_hits.push(FOE);
    w.tick(None);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::MERGE_STACKS_PER_HIT, "two stacks a hit");
    assert_eq!(w.core.echoes.iter().filter(|e| e.target == FOE).count(), 2, "every hit echoes twice");
    // a gap of 0.25 s between dodges while merged
    assert!(w.core.try_dodge(w.f.tick, Point::new(430_000, 500_000)));
    assert!(w.core.try_dodge(w.f.tick + d::MERGE_DODGE_GAP_TICKS as u64, Point::new(430_000, 500_000)));
    w.run(d::MERGE_TICKS as u64);
    assert!(!w.core.merged(w.f.tick));
    assert!(w.core.charges <= d::CHARGES_MAX, "back to five at most");
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
fn a_flurry_cast_while_merged_echoes_every_punch() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]);
    w.unit(FOE).hp = 1_000_000;
    w.unit(FOE).max_hp = 1_000_000;
    w.tick(Some(Order::S1 { aim: Point::new(430_000, 500_000) }));
    w.tick(Some(Order::Ult));
    assert!(w.core.flurry.is_some_and(|fl| fl.echo));
    w.run(d::ULT_TICKS as u64 + d::PACT_DELAY_TICKS as u64 + 2);
    let punches = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_flurry"));
    let echoes = w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_echo"));
    assert!(echoes >= punches, "{punches} punches, {echoes} echoes");
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

// ------------------------------------------------------------------ S3 Shadow Tether

#[test]
fn tether_latches_onto_everyone_in_reach_and_copies_his_hits_at_30_percent() {
    let mut tower = champ(50, 2, 420_000, 520_000);
    tower.champion = false;
    tower.tower = true;
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000), champ(11, 2, 430_000, 520_000), creep(30, 440_000, 480_000, 9_000), tower, champ(12, 2, 600_000, 500_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(410_000, 500_000) }));
    let ids: Vec<usize> = w.core.tethers.iter().map(|t| t.0).collect();
    assert_eq!(ids.len(), 3, "the two champions and the creep in reach (never the tower, never the far one): {ids:?}");
    w.f.basic_hits.push(FOE);
    w.tick(None);
    let copy = 100 * d::TETHER_COPY_PCT / 100;
    for id in [11, 30] {
        assert!(w.seen(|e| matches!(e, Effect::Damage { target, amount } if *target == id && *amount == copy)), "copy on {id}");
    }
    assert!(w.core.stacks_on(11, w.f.tick) >= 1, "copies stack");
    // too far: the tether breaks
    w.unit(11).pos = Point::new(400_000 + d::TETHER_BREAK_RANGE + 10_000, 500_000);
    w.tick(None);
    assert!(!w.core.tethers.iter().any(|t| t.0 == 11));
}

#[test]
fn the_tether_recast_yanks_everyone_toward_him_and_uppercuts() {
    let mut w = World::new(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 400_000, 545_000)]);
    w.tick(Some(Order::S3 { aim: Point::new(440_000, 500_000) }));
    w.tick(Some(Order::S3 { aim: Point::new(440_000, 500_000) }));
    assert!(w.core.tethers.is_empty());
    for id in [FOE, 11] {
        assert!(w.seen(|e| matches!(e, Effect::Knock { target, .. } if *target == id)), "{id} yanked");
        assert!(w.seen(|e| matches!(e, Effect::KnockUp { target, ticks } if *target == id && *ticks == d::TETHER_KNOCK_TICKS as u64)));
    }
    assert!(w.f.units[0].pos.x < 440_000, "toward him");
}

#[test]
fn flurry_punches_copy_onto_tethered_enemies_at_5_percent_and_never_bank() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 400_000, 540_000)]);
    w.f.attack = 1_000; // so a 5% copy is a whole number
    for u in w.f.units.iter_mut() {
        u.hp = 1_000_000;
        u.max_hp = 1_000_000;
    }
    w.tick(Some(Order::S3 { aim: Point::new(430_000, 500_000) }));
    w.unit(11).pos = Point::new(430_000, 560_000); // tethered, but out of the flurry's chain reach
    w.tick(Some(Order::Ult));
    w.run(60);
    let copy = pct(d::ULT_BASE, d::ULT_AD_RATIO, 1_000) * d::TETHER_FLURRY_COPY_PCT / 100;
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: 11, amount } if *amount == copy)), "a 5% copy");
    assert_eq!(w.core.bank, 0);
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
fn the_flurry_vanishes_him_chains_36_accelerating_punches_and_brings_him_back_by_the_last() {
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
    assert!(Core::flurry_gap(0) > Core::flurry_gap(d::ULT_PUNCHES - 1), "slow at first, fast at the end");
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
    w.core.bank = 0;
    w
}

#[test]
fn the_ai_merges_going_into_a_fight_then_tethers_a_group() {
    let mut w = ai(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 445_000, 510_000), champ(ALLY, 1, 410_000, 500_000), champ(3, 1, 415_000, 505_000)]);
    w.tick(None);
    assert_eq!(w.core.reason, "S1: merge");
    w.tick(None);
    assert_eq!(w.core.reason, "S3: tether");
    assert_eq!(w.core.tethers.len(), 2);
}

#[test]
fn the_ai_never_dives_a_one_versus_four() {
    let mut w = ai(vec![champ(FOE, 2, 440_000, 500_000), champ(11, 2, 445_000, 500_000), champ(12, 2, 440_000, 505_000), champ(13, 2, 445_000, 505_000)]);
    w.run(30);
    assert!(!w.core.merged(w.f.tick) && w.core.tethers.is_empty(), "{}", w.core.reason);
}

#[test]
fn the_ai_farms_a_camp_with_its_skills_when_calm_and_banks_toward_the_ult() {
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
    for k in ["Farm: Shadow Cyclone", "Farm: tether"] {
        assert!(used.iter().any(|u| u == k), "{k}: {used:?}");
    }
    assert!(w.core.bank > 0, "farming with his skills banks toward the ult (echoes and copies never do)");
}

#[test]
fn the_ai_flurries_with_a_full_bank_on_a_group() {
    let mut w = ai(vec![champ(FOE, 2, 430_000, 500_000), champ(11, 2, 440_000, 505_000), champ(ALLY, 1, 420_000, 500_000), champ(3, 1, 425_000, 510_000)]);
    w.core.cd = [u64::MAX / 2; 3];
    w.core.bank = d::ULT_BANK_MAX;
    w.tick(None);
    assert_eq!(w.core.reason, "Shadow Flurry");
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
        assert_eq!(host.borrow().heals.len(), 1, "the second hit inside the gap lands");
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
        assert!(host.borrow().buffs[0].iter().any(|b| b.name() == "ct_merge") || c.core.merged(220) || !c.core.tethers.is_empty() || c.core.cd != [0; 3], "{}", c.core.reason);
        let _ = Point::default();
    }
}
