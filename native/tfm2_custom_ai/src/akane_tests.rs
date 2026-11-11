//! Akane's core: the passive, each skill, Crimson Night, and her own AI (with the 13 athlete values).
use crate::akane_core::*;
use crate::akane_data as d;

const ME: usize = 1;
const FOE: usize = 10;

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: 1000,
        max_hp: 1000,
        radius: 3000,
        attack: 100,
        champion: true,
        visible: true,
        targetable: true,
        ..Unit::default()
    }
}
fn minion(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, farmable: true, hp: 300, max_hp: 300, ..champ(id, 2, x, y) }
}
fn tower(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, tower: true, hp: 3000, max_hp: 3000, ..champ(id, 2, x, y) }
}
fn frame(tick: u64, units: Vec<Unit>) -> Frame {
    Frame {
        tick,
        owner: ME,
        team: 1,
        alive: true,
        pos: Point::new(400_000, 400_000),
        hp: 1600,
        max_hp: 1600,
        attack: 180,
        ult_unlocked: true,
        home: Point::new(96_000, 864_000),
        units,
        profile: [50; 13],
        manual: true,
        ..Frame::default()
    }
}
fn order(c: &mut Core, f: &Frame, o: Order) -> Vec<Effect> {
    let mut f = f.clone();
    f.order = Some(o);
    c.step(&f)
}
/// run ticks, moving her along glides and hops like the engine would
fn run_order(c: &mut Core, f: &mut Frame, first: Option<Order>, from: u64, to: u64) -> Vec<Effect> {
    let mut all = vec![];
    let mut glide: Option<(Point, Point, u64, u64)> = None;
    for tick in from..to {
        f.tick = tick;
        f.order = if tick == from { first } else { None };
        if let Some((a, b, s, n)) = glide {
            let k = (tick - s).min(n) as i64;
            f.pos = Point::new(a.x + (b.x - a.x) * k / n as i64, a.y + (b.y - a.y) * k / n as i64);
        }
        let out = c.step(f);
        for e in &out {
            match e {
                Effect::Glide { to, ticks } => glide = Some((f.pos, *to, tick, *ticks)),
                Effect::Hop { to } | Effect::Teleport { to } => {
                    f.pos = *to;
                    glide = None;
                }
                _ => {}
            }
        }
        f.basic_hits.clear();
        f.takedowns.clear();
        all.extend(out);
    }
    all
}
fn damage_to(out: &[Effect], id: usize) -> i64 {
    out.iter().filter_map(|e| match e { Effect::Damage { target, amount, .. } if *target == id => Some(*amount), _ => None }).sum()
}
fn wall_grid(cells: &[(i64, i64)]) -> Vec<bool> {
    let mut g = vec![false; (CELLS * CELLS) as usize];
    for (cx, cy) in cells {
        g[(cy * CELLS + cx) as usize] = true;
    }
    g
}
fn hit(c: &mut Core, f: &mut Frame, tick: u64, id: usize) -> Vec<Effect> {
    f.tick = tick;
    f.order = None;
    f.basic_hits = vec![id];
    let out = c.step(f);
    f.basic_hits.clear();
    out
}

// ------------------------------------------------------------------ passive: Soul Harvest

#[test]
fn a_takedown_leaves_an_orb_that_arms_after_half_a_second_and_lasts_fifteen() {
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    f.takedowns = vec![(FOE, Point::new(400_000, 400_000))];
    c.step(&f);
    assert_eq!(c.orbs.len(), 1);
    f.takedowns.clear();
    c.ready[KUNAI] = 10_000;
    // standing on it from tick 1: not before 0.5 s + 0.75 s
    for tick in 1..(d::ORB_ARM_TICKS + d::ORB_ABSORB_TICKS) as u64 {
        f.tick = tick;
        c.step(&f);
    }
    assert_eq!(c.orbs.len(), 1, "not absorbed before arming + 0.75 s");
    f.tick = (d::ORB_ARM_TICKS + d::ORB_ABSORB_TICKS) as u64;
    c.step(&f);
    assert!(c.orbs.is_empty());
    assert!(c.ready(KUNAI, f.tick), "Kunai Flip reset");
    // an untouched orb expires after 15 s
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    f.takedowns = vec![(FOE, Point::new(600_000, 400_000))];
    c.step(&f);
    f.takedowns.clear();
    f.tick = d::ORB_TICKS as u64 + 1;
    c.step(&f);
    assert!(c.orbs.is_empty());
}

#[test]
fn absorbing_needs_staying_in_range_and_is_cancelled_by_leaving_or_a_stun() {
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    f.takedowns = vec![(FOE, Point::new(410_000, 400_000))];
    c.step(&f);
    f.takedowns.clear();
    let arm = d::ORB_ARM_TICKS as u64;
    for tick in arm..arm + 30 {
        f.tick = tick;
        c.step(&f);
    }
    // she steps out, then back: the 0.75 s starts again
    f.pos = Point::new(500_000, 400_000);
    f.tick = arm + 30;
    c.step(&f);
    f.pos = Point::new(400_000, 400_000);
    for tick in arm + 31..arm + 31 + d::ORB_ABSORB_TICKS as u64 {
        f.tick = tick;
        c.step(&f);
    }
    assert_eq!(c.orbs.len(), 1, "restarted after leaving");
    f.stunned = true;
    f.tick += 1;
    c.step(&f);
    assert!(c.orbs[0].since.is_none(), "a stun cancels it");
    // she can move while absorbing: a different spot inside the range still counts
    f.stunned = false;
    let start = f.tick + 1;
    for (i, tick) in (start..start + d::ORB_ABSORB_TICKS as u64 + 1).enumerate() {
        f.tick = tick;
        f.pos = Point::new(400_000 + (i as i64 % 3) * 4_000, 400_000);
        c.step(&f);
    }
    assert!(c.orbs.is_empty(), "absorbed while moving inside the range");
}

#[test]
fn an_orb_refills_a_charge_resets_kunai_and_resets_edge_only_once_every_30_s() {
    let mut c = Core::default();
    c.charges = 0;
    c.ready = [5_000, 5_000, 5_000];
    let mut f = frame(0, vec![]);
    let take = |c: &mut Core, f: &mut Frame, at: u64| {
        f.tick = at;
        f.takedowns = vec![(FOE, f.pos)];
        c.step(f);
        f.takedowns.clear();
        for tick in at + 1..=at + (d::ORB_ARM_TICKS + d::ORB_ABSORB_TICKS) as u64 + 1 {
            f.tick = tick;
            c.step(f);
        }
    };
    take(&mut c, &mut f, 0);
    assert_eq!(c.charges, 1);
    assert!(c.ready(KUNAI, f.tick) && c.ready(EDGE, f.tick));
    assert!(!c.ready(NIGHT, f.tick), "never the ult");
    c.ready[EDGE] = 5_000;
    c.ready[KUNAI] = 5_000;
    take(&mut c, &mut f, 200);
    assert_eq!(c.charges, 2, "a charge every time");
    assert!(c.ready(KUNAI, f.tick), "Kunai every time");
    assert!(!c.ready(EDGE, f.tick), "Crimson Edge only once in 30 s");
    take(&mut c, &mut f, 400);
    assert_eq!(c.charges, 2, "never above two");
    take(&mut c, &mut f, d::ORB_S3_RESET_COOLDOWN_TICKS as u64 + 10);
    assert!(c.ready(EDGE, f.tick), "after 30 s it resets Crimson Edge again");
}

// ------------------------------------------------------------------ S1

#[test]
fn dismiss_and_devour_is_invisible_invulnerable_fast_move_only_with_a_heal_and_no_dash() {
    let mut c = Core::default();
    let f = frame(0, vec![]);
    let out = order(&mut c, &f, Order::Soul);
    assert!(out.contains(&Effect::Soul { ticks: d::SOUL_TICKS as u64, speed_pct: d::SOUL_SPEED_PCT }));
    assert!(out.iter().any(|e| matches!(e, Effect::Heal { amount } if *amount == 1600 * d::SOUL_HEAL_PCT_MAX_HP / 100 + 180 * d::SOUL_HEAL_AD_RATIO / 100)));
    assert!(!out.iter().any(|e| matches!(e, Effect::Glide { .. })), "not a dash");
    assert_eq!(c.charges, 1);
    // move only: no skills until it ends
    let out = order(&mut c, &frame(10, vec![]), Order::Edge);
    assert!(!out.iter().any(|e| matches!(e, Effect::Pose { .. })));
    assert!(c.rejection.contains("only move"));
    // a second charge extends it to its own end, never stacking
    order(&mut c, &frame(60, vec![]), Order::Soul);
    assert_eq!(c.soul_until, 60 + d::SOUL_TICKS as u64);
    assert_eq!(c.charges, 0);
    let out = order(&mut c, &frame(70, vec![]), Order::Soul);
    assert!(out.is_empty() || !out.iter().any(|e| matches!(e, Effect::Soul { .. })), "no charge left");
    // after it ends she acts normally
    let out = order(&mut c, &frame(60 + d::SOUL_TICKS as u64, vec![]), Order::Edge);
    assert!(out.iter().any(|e| matches!(e, Effect::Pose { .. })));
}

#[test]
fn soul_charges_recharge_ten_seconds_each() {
    let mut c = Core::default();
    order(&mut c, &frame(0, vec![]), Order::Soul);
    order(&mut c, &frame(100, vec![]), Order::Soul);
    assert_eq!(c.charges, 0);
    let mut f = frame(0, vec![]);
    for tick in 101..=d::SOUL_RECHARGE_TICKS as u64 + 2 {
        f.tick = tick;
        c.step(&f);
    }
    assert_eq!(c.charges, 1);
    for tick in d::SOUL_RECHARGE_TICKS as u64 + 3..=2 * d::SOUL_RECHARGE_TICKS as u64 + 3 {
        f.tick = tick;
        c.step(&f);
    }
    assert_eq!(c.charges, 2);
}

// ------------------------------------------------------------------ S2

#[test]
fn kunai_flips_back_marks_the_first_enemy_and_the_recast_dashes_and_slashes() {
    let mut c = Core::default();
    let mut f = frame(0, vec![minion(30, 430_000, 400_000), champ(FOE, 2, 450_000, 400_000), tower(40, 420_000, 400_000)]);
    let mut out = run_order(&mut c, &mut f, Some(Order::Kunai(Point::new(450_000, 400_000))), 0, 20);
    assert!(f.pos.x < 400_000 - d::KUNAI_FLIP + 1_000, "flipped back: {:?}", f.pos);
    assert_eq!(c.marked(20), Some(30), "the minion in front is hit first, never the tower");
    assert_eq!(damage_to(&out, 40), 0);
    out.extend(run_order(&mut c, &mut f, Some(Order::KunaiRecast), 20, 60));
    assert!(f.pos.x > 420_000, "dashed to the mark: {:?}", f.pos);
    assert!(out.iter().any(|e| matches!(e, Effect::Damage { target: 30, direct: true, .. })), "the slash is a committed strike");
    // a miss means no recast; the mark expires after 3 s
    let mut c = Core::default();
    order(&mut c, &frame(0, vec![]), Order::Kunai(Point::new(500_000, 400_000)));
    assert!(c.marked(1).is_none());
    let mut c = Core::default();
    order(&mut c, &frame(0, vec![champ(FOE, 2, 450_000, 400_000)]), Order::Kunai(Point::new(450_000, 400_000)));
    assert!(c.marked(d::KUNAI_MARK_TICKS as u64).is_none());
}

#[test]
fn the_recast_dashes_over_a_wall() {
    let mut c = Core::default();
    let mut f = frame(0, vec![champ(FOE, 2, 14 * CELL + 16_000, 400_000)]);
    f.pos = Point::new(12 * CELL + 16_000, 400_000);
    f.terrain = wall_grid(&[(13, 12)]);
    run_order(&mut c, &mut f, Some(Order::Kunai(Point::new(14 * CELL + 16_000, 400_000))), 0, 20);
    assert_eq!(c.marked(20), Some(FOE), "the kunai flies over terrain");
    let out = run_order(&mut c, &mut f, Some(Order::KunaiRecast), 20, 60);
    assert!(out.iter().any(|e| matches!(e, Effect::Hop { .. })));
    assert!(f.pos.x > 14 * CELL && !f.terrain_at(f.pos), "past the wall on open ground: {:?}", f.pos);
}

// ------------------------------------------------------------------ S3

#[test]
fn crimson_edge_runs_its_five_hits_in_order() {
    let mut c = Core::default();
    let mut f = frame(0, vec![champ(FOE, 2, 415_000, 400_000), champ(11, 2, 425_000, 400_000)]);
    order(&mut c, &f, Order::Edge);
    let o1 = hit(&mut c, &mut f, 10, FOE);
    assert!(o1.contains(&Effect::Bleed { target: FOE, stacks: 1, ticks: d::EDGE_BLEED_TICKS as u64 }));
    let o2 = hit(&mut c, &mut f, 20, FOE);
    assert!(o2.iter().any(|e| matches!(e, Effect::Slow { target: FOE, percent, .. } if *percent == d::EDGE_SLOW_PCT)));
    let o3 = hit(&mut c, &mut f, 30, FOE);
    assert!(o3.iter().any(|e| matches!(e, Effect::Vulnerable { target: FOE, percent, .. } if *percent == d::EDGE_VULNERABLE_PCT)));
    let o4 = hit(&mut c, &mut f, 40, FOE);
    let main = 1000 * d::EDGE_BURST_PCT_MAX_HP / 100 + d::EDGE_BURST_AD_RATIO * 180 / 100;
    assert!(o4.contains(&Effect::Damage { target: FOE, amount: main, direct: false }), "{o4:?}");
    assert!(o4.contains(&Effect::Damage { target: 11, amount: main / 2, direct: false }), "half around it");
    assert!(o4.iter().any(|e| matches!(e, Effect::Crimson { target: 11, .. })));
    assert_eq!(c.stacks_on(FOE, 40), 0, "the burst consumes the stacks");
    let o5 = hit(&mut c, &mut f, 50, FOE);
    assert!(o5.iter().any(|e| matches!(e, Effect::Invulnerable { .. })));
    let to = o5.iter().find_map(|e| if let Effect::Teleport { to } = e { Some(*to) } else { None }).unwrap();
    // with an ally of the target beside it (at 425,000), she lands on the far side: the target between them
    assert!(to.x < 415_000, "target between her and its ally: {to:?}");
    assert!(o5.iter().any(|e| matches!(e, Effect::KnockUp { target: FOE, .. })) && o5.iter().any(|e| matches!(e, Effect::KnockUp { target: 11, .. })));
    assert!(c.edge.is_none());
    // alone, the target is simply passed through: she lands behind it from where she stood
    let lone = champ(FOE, 2, 415_000, 400_000);
    let f = frame(0, vec![lone.clone()]);
    assert!(Core::behind(&f, &lone).x > 415_000);
}

#[test]
fn crimson_edge_ignores_towers_and_expires_after_six_seconds() {
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(40, 410_000, 400_000), champ(FOE, 2, 415_000, 400_000)]);
    order(&mut c, &f, Order::Edge);
    let o = hit(&mut c, &mut f, 5, 40);
    assert!(o.iter().all(|e| !matches!(e, Effect::Bleed { .. })), "no debuff on a tower");
    assert_eq!(c.edge, Some((0, d::EDGE_WINDOW_TICKS as u64)), "and it didn't use a hit");
    f.tick = d::EDGE_WINDOW_TICKS as u64;
    c.step(&f);
    assert!(c.edge.is_none());
}

#[test]
fn bleed_stacks_cap_at_four_and_tick_each_second() {
    let mut c = Core::default();
    let mut f = frame(0, vec![champ(FOE, 2, 415_000, 400_000)]);
    order(&mut c, &f, Order::Edge);
    hit(&mut c, &mut f, 1, FOE);
    let mut total = 0;
    for tick in 2..=61 {
        f.tick = tick;
        total += damage_to(&c.step(&f), FOE);
    }
    assert_eq!(total, d::EDGE_BLEED_BASE + d::EDGE_BLEED_AD_RATIO * 180 / 100, "one stack, one second");
}

// ------------------------------------------------------------------ Ult

#[test]
fn crimson_night_lasts_four_seconds_and_dashes_once_to_a_visible_champion_in_reach() {
    let mut c = Core::default();
    let mut f = frame(0, vec![champ(FOE, 2, 500_000, 400_000), champ(11, 2, 600_000, 400_000)]);
    let out = order(&mut c, &f, Order::Night);
    assert!(out.contains(&Effect::Night { ticks: d::NIGHT_TICKS as u64 }));
    let far = order(&mut c, &frame(1, f.units.clone()), Order::NightDash(11));
    assert!(c.rejection.contains("no visible"), "200,000 is out of reach: {far:?}");
    let out = run_order(&mut c, &mut f, Some(Order::NightDash(FOE)), 2, 40);
    assert!(out.iter().any(|e| matches!(e, Effect::Damage { target: FOE, direct: true, amount } if *amount == d::NIGHT_BASE + d::NIGHT_AD_RATIO * 180 / 100)));
    order(&mut c, &frame(50, f.units.clone()), Order::NightDash(FOE));
    assert!(c.rejection.contains("no Crimson Night dash"), "only once");
}

#[test]
fn her_skills_never_damage_towers() {
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(40, 410_000, 400_000)]);
    let mut out = run_order(&mut c, &mut f, Some(Order::Kunai(Point::new(450_000, 400_000))), 0, 20);
    out.extend(order(&mut c, &frame(30, f.units.clone()), Order::Edge));
    out.extend(hit(&mut c, &mut f, 40, 40));
    assert_eq!(damage_to(&out, 40), 0);
}

// ------------------------------------------------------------------ the AI

fn auto(mut f: Frame) -> Frame {
    f.manual = false;
    f
}

#[test]
fn ai_holds_the_recast_when_the_target_flashed_away_and_uses_it_when_it_lands_in_reach() {
    let mut c = Core::default();
    let mut low = champ(FOE, 2, 440_000, 400_000);
    low.hp = 250;
    low.flash_ready = true;
    order(&mut c, &frame(0, vec![low.clone()]), Order::Kunai(Point::new(440_000, 400_000)));
    // it Flashes far away
    let mut gone = low.clone();
    gone.pos = Point::new(530_000, 400_000);
    gone.flash_ready = false;
    let f = auto(frame(14, vec![gone.clone()]));
    c.busy_until = 0;
    c.step(&f);
    assert_ne!(c.last_skill, "kunai_recast", "out of reach: hold the mark");
    assert!(c.marked(14).is_some());
    // it lands within reach again
    let mut back = gone;
    back.pos = Point::new(470_000, 400_000);
    c.step(&auto(frame(30, vec![back])));
    assert_eq!(c.last_skill, "kunai_recast");
}

#[test]
fn ai_refuses_a_recast_into_an_untanked_tower_unless_it_kills_with_a_charge_left() {
    let mut c = Core::default();
    let healthy = champ(FOE, 2, 440_000, 400_000);
    let tw = tower(40, 470_000, 400_000);
    order(&mut c, &frame(0, vec![healthy.clone(), tw.clone()]), Order::Kunai(Point::new(440_000, 400_000)));
    c.busy_until = 0;
    c.ready = [9_999, 9_999, 9_999];
    c.step(&auto(frame(14, vec![healthy, tw.clone()])));
    assert_ne!(c.last_skill, "kunai_recast", "a healthy foe under the tower");
    let mut c = Core::default();
    let mut low = champ(FOE, 2, 440_000, 400_000);
    low.hp = 120;
    order(&mut c, &frame(0, vec![low.clone(), tw.clone()]), Order::Kunai(Point::new(440_000, 400_000)));
    c.busy_until = 0;
    c.step(&auto(frame(14, vec![low, tw])));
    assert_eq!(c.last_skill, "kunai_recast", "a kill under the tower with a Soul charge to leave");
}

#[test]
fn ai_never_slashes_into_a_v1_parry() {
    let mut c = Core::default();
    let mut v1 = champ(FOE, 2, 440_000, 400_000);
    v1.hp = 100;
    order(&mut c, &frame(0, vec![v1.clone()]), Order::Kunai(Point::new(440_000, 400_000)));
    v1.parry = true;
    c.busy_until = 0;
    c.ready = [9_999, 9_999, 9_999];
    c.step(&auto(frame(14, vec![v1])));
    assert_ne!(c.last_skill, "kunai_recast");
}

#[test]
fn flash_dashes_and_heals_raise_the_margin_she_needs() {
    let c = Core::default();
    let f = frame(0, vec![]);
    let plain = champ(FOE, 2, 440_000, 400_000);
    let mut slippery = plain.clone();
    slippery.flash_ready = true;
    slippery.skill_ready = true;
    slippery.can = CAN_DASH | CAN_HEAL;
    assert!(c.escape_margin(&f, &slippery) >= c.escape_margin(&f, &plain) + d::AI_FLASH_MARGIN_PCT);
    let mut immune = plain.clone();
    immune.reduce = 100;
    assert!(Core::effective_hp(&immune).is_none(), "untouchable now");
}

#[test]
fn ai_engages_an_isolated_low_target_with_the_kunai_but_not_without_an_out() {
    let mut c = Core::default();
    let mut low = champ(FOE, 2, 440_000, 400_000);
    low.hp = 400;
    c.ready[NIGHT] = 9_999; // (with the ult up she'd open with Crimson Night on an isolated kill)
    c.step(&auto(frame(0, vec![low.clone()])));
    assert_eq!(c.last_skill, "kunai");
    // no Soul charge, no Flash, and the Kunai is the only out: she won't throw it in
    let mut c = Core::default();
    c.charges = 0;
    let mut healthy = champ(FOE, 2, 440_000, 400_000);
    healthy.hp = 1000;
    healthy.armor = 60;
    c.step(&auto(frame(0, vec![healthy, champ(11, 2, 445_000, 410_000), champ(12, 2, 445_000, 390_000)])));
    assert_ne!(c.last_skill, "kunai", "no reserve and not lethal into three");
}

#[test]
fn ai_dodges_with_a_soul_charge_after_her_reaction_time() {
    let mut fast = frame(0, vec![]);
    fast.profile[CONTROL_SPEED] = 100;
    fast.profile[SKILL_AVOID] = 100;
    assert_eq!(Core::reaction_ticks(&fast), 0);
    let slow = frame(0, vec![]);
    assert!(Core::reaction_ticks(&slow) > 0);
    let mut c = Core::default();
    let mut f = auto(frame(0, vec![]));
    f.profile = fast.profile;
    let mut stunner = champ(FOE, 2, 420_000, 400_000);
    stunner.skill_ready = true;
    stunner.can = CAN_CC;
    stunner.attack = 400;
    f.units = vec![stunner, champ(11, 2, 425_000, 410_000)];
    c.ready = [9_999, 9_999, 9_999];
    c.step(&f);
    assert_eq!(c.last_skill, "soul");
}

#[test]
fn ai_disengages_after_a_kill_when_the_next_one_isnt_worth_it() {
    let mut c = Core::default();
    c.charges = 1;
    let mut f = auto(frame(0, vec![champ(11, 2, 430_000, 400_000), champ(12, 2, 440_000, 410_000), champ(13, 2, 435_000, 390_000)]));
    f.takedowns = vec![(FOE, Point::new(405_000, 400_000))];
    f.profile[MACRO] = 30;
    c.ready = [9_999, 9_999, 9_999];
    c.step(&f);
    assert!(c.disengage_until > 0, "three healthy enemies converging: get out");
    f.takedowns.clear();
    f.tick = 1;
    let out = c.step(&f);
    assert!(out.iter().any(|e| matches!(e, Effect::Steer { .. })), "she runs to a safe spot");
}

#[test]
fn ai_chains_a_second_kill_when_it_is_still_safe() {
    let mut c = Core::default();
    let mut next = champ(11, 2, 440_000, 400_000);
    next.hp = 200;
    let mut f = auto(frame(0, vec![next]));
    f.takedowns = vec![(FOE, Point::new(405_000, 400_000))];
    c.step(&f);
    assert_eq!(c.disengage_until, 0, "a lone low enemy: go again");
}

#[test]
fn ai_absorbs_an_orb_only_when_it_is_safe() {
    let mut c = Core::default();
    c.ready = [9_999, 9_999, 9_999];
    c.orbs.push(Orb { pos: Point::new(450_000, 400_000), born: 0, until: 900, since: None });
    let out = c.step(&auto(frame(40, vec![])));
    assert!(out.contains(&Effect::Steer { to: Point::new(450_000, 400_000) }));
    let mut c = Core::default();
    c.ready = [9_999, 9_999, 9_999];
    c.charges = 0;
    c.orbs.push(Orb { pos: Point::new(450_000, 400_000), born: 0, until: 900, since: None });
    let mut big = champ(11, 2, 455_000, 400_000);
    big.attack = 600;
    big.skill_ready = true;
    big.can = CAN_CC;
    let out = c.step(&auto(frame(40, vec![big.clone(), Unit { id: 12, ..big.clone() }, Unit { id: 13, ..big }])));
    assert!(!out.contains(&Effect::Steer { to: Point::new(450_000, 400_000) }), "three killers on the orb");
}

#[test]
fn ai_uses_the_night_to_pick_and_saves_the_dash_for_the_right_moment() {
    let mut c = Core::default();
    let mut low = champ(FOE, 2, 500_000, 400_000);
    low.hp = 150;
    c.ready[KUNAI] = 9_999;
    c.ready[EDGE] = 9_999;
    c.step(&auto(frame(0, vec![low.clone()])));
    assert_eq!(c.last_skill, "night");
    // it still has Flash and plenty of night left: wait
    let mut slippery = low.clone();
    slippery.flash_ready = true;
    c.step(&auto(frame(5, vec![slippery.clone()])));
    assert_eq!(c.last_skill, "night");
    // it Flashes: go now
    let mut flashed = slippery;
    flashed.flash_ready = false;
    c.step(&auto(frame(6, vec![flashed])));
    assert_eq!(c.last_skill, "night_dash");
}

#[test]
fn stats_shape_choices_never_damage() {
    // skill_hit leads the aim more
    let mut u = champ(FOE, 2, 460_000, 400_000);
    u.velocity = Point::new(0, 300);
    let mut low = frame(0, vec![]);
    low.profile[SKILL_HIT] = 0;
    let mut high = frame(0, vec![]);
    high.profile[SKILL_HIT] = 100;
    assert_eq!(Core::aim_at(&low, &u), u.pos);
    assert!(Core::aim_at(&high, &u).y > u.pos.y);
    // awareness keeps a missing enemy a worry for longer
    let mut aware = frame(0, vec![]);
    aware.profile[AWARENESS] = 100;
    let mut blind = frame(0, vec![]);
    blind.profile[AWARENESS] = 0;
    assert!(Core::memory_ticks(&aware) > Core::memory_ticks(&blind));
    // aggression shrinks the margin
    let c = Core::default();
    let mut bold = frame(0, vec![]);
    bold.profile[AGGRESSIVE] = 100;
    let mut shy = frame(0, vec![]);
    shy.profile[AGGRESSIVE] = 0;
    assert!(c.escape_margin(&bold, &u) < c.escape_margin(&shy, &u));
    // judgement: perfect judgement has no estimate error
    let mut sharp = frame(0, vec![]);
    sharp.profile[JUDGEMENT] = 100;
    assert_eq!(Core::noise(&sharp, 7), 0);
    // the kunai's damage is the same whatever the stats
    let mut a = Core::default();
    let mut b = Core::default();
    let units = vec![champ(FOE, 2, 440_000, 400_000)];
    let mut fa = frame(0, units.clone());
    fa.profile = [0; 13];
    let mut fb = frame(0, units);
    fb.profile = [100; 13];
    let da = damage_to(&order(&mut a, &fa, Order::Kunai(Point::new(440_000, 400_000))), FOE);
    let db = damage_to(&order(&mut b, &fb, Order::Kunai(Point::new(440_000, 400_000))), FOE);
    assert_eq!(da, db);
}

#[test]
fn ego_leaves_a_kill_to_a_closer_teammate_and_order_joins_the_team_fight() {
    let c = Core::default();
    let mut u = champ(FOE, 2, 440_000, 400_000);
    u.hp = 500;
    let other = Unit { id: 11, hp: 500, ..champ(11, 2, 380_000, 430_000) };
    let ally = champ(5, 1, 438_000, 400_000);
    let mut modest = frame(0, vec![u.clone(), other.clone(), ally.clone()]);
    modest.profile[EGO] = 0;
    modest.profile[ORDER] = 0;
    assert_eq!(c.hunt(&modest).map(|t| t.id), Some(11), "leaves FOE to the closer ally");
    let mut orderly = frame(0, vec![u, other, ally]);
    orderly.profile[EGO] = 100;
    orderly.profile[ORDER] = 100;
    assert_eq!(c.hunt(&orderly).map(|t| t.id), Some(FOE), "joins the teammate's fight");
}

// ------------------------------------------------------------------ glue (the mock host)

mod glue {
    use crate::akane::Akane;
    use crate::akane_core::{Effect, Order, Point};
    use crate::akane_data as d;
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use mod_api_stable::{AttackTypeV1, CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn dismiss_hides_her_makes_her_invulnerable_and_move_only_without_reading_as_stunned() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_akane")]);
        let mut k = Akane::default();
        host.borrow_mut().tick = 10;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        let mut out = vec![];
        k.core.execute(&f, Order::Soul, &mut out);
        with_sim(&host, |sim| k.apply(sim, 0, out));
        let g = host.borrow();
        assert!(g.invisible.contains(&(0, d::SOUL_TICKS as usize)));
        let soul = g.buffs[0].iter().find(|b| b.name() == "akn_soul").expect("soul buff");
        assert!(soul.damaged_reduce == 100 && soul.cc_immune && soul.move_speed_mult == d::SOUL_SPEED_PCT as i32);
        let locks: Vec<u32> = g.ccs[0].iter().filter(|c| c.0.name() == "akn_soul_lock").map(|c| c.0.kind).collect();
        assert!(locks.contains(&CcKindV1::BlockAttack.code()) && locks.contains(&CcKindV1::BlockSkill.code()));
        assert_eq!(g.heals.len(), 1);
        drop(g);
        host.borrow_mut().tick = 11;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        assert!(!f.stunned, "her own locks never read as being stunned");
    }

    #[test]
    fn the_slash_is_parryable_and_towers_take_nothing() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_akane"),
            HostUnit { kind: K::Tower, team: 1, pos: (x + 20_000, y), hp: 3000, name: "tower", targetable: true },
            champ(1, (x + 30_000, y), "tfm2_ultrakill_v1"),
            champ(1, (x + 40_000, y), "foe"),
        ]);
        let mut k = Akane::default();
        with_sim(&host, |sim| sim.add_buff(2, &crate::timed("v1_guard", 600)));
        host.borrow_mut().tick = 5;
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Damage { target: 1, amount: 200, direct: false },
                    Effect::KnockUp { target: 1, ticks: 30 },
                    Effect::Damage { target: 2, amount: 150, direct: true },
                    Effect::Damage { target: 3, amount: 90, direct: false },
                ],
            )
        });
        let g = host.borrow();
        assert!(g.damage.iter().all(|dm| dm.1 != 1 && dm.1 != 2), "no tower damage, the parried slash dealt nothing: {:?}", g.damage);
        assert!(g.damage.iter().any(|dm| dm.0 == 0 && dm.1 == 3 && dm.3 == AttackTypeV1::Skill.code()));
        assert!(g.ccs[1].is_empty());
        assert!(g.buffs[2].iter().any(|b| b.name() == "v1_parry"));
    }

    #[test]
    fn glides_go_through_the_engine_and_only_hops_and_the_finale_set_positions() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_akane")]);
        let mut k = Akane::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Glide { to: Point::new(x as i64 + 10_000, y as i64), ticks: 8 }]));
        assert!(host.borrow().ccs[0].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code() && c.0.name() == crate::motion::SELF));
        assert!(host.borrow().moves.is_empty());
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Teleport { to: Point::new(x as i64 + 20_000, y as i64) }]));
        assert_eq!(host.borrow().moves.len(), 1, "the finale is a deliberate teleport");
    }

    #[test]
    fn crimson_night_hides_her_team_far_from_enemy_eyes_only() {
        let (x, y) = open_spot(200_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_akane"),
            champ(0, (x + 150_000, y), "ally"),
            champ(1, (x + 140_000, y), "enemy"),
        ]);
        let mut k = Akane::default();
        host.borrow_mut().tick = 20;
        with_sim(&host, |sim| {
            k.apply(sim, 0, vec![Effect::Night { ticks: d::NIGHT_TICKS as u64 }]);
            k.darkness(sim, 0);
        });
        let g = host.borrow();
        assert!(g.invisible.iter().any(|i| i.0 == 0), "she's far from the enemy: hidden");
        assert!(!g.invisible.iter().any(|i| i.0 == 1), "her ally stands within the enemy's nearsight: still seen");
    }

    #[test]
    fn steering_is_a_short_buff_the_input_ai_reads() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_akane")]);
        let mut k = Akane::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Steer { to: Point::new(123_000, 456_000) }, Effect::Steer { to: Point::new(124_000, 457_000) }]));
        let g = host.borrow();
        let go: Vec<_> = g.buffs[0].iter().filter_map(|b| crate::akane::steer_of(b.name())).collect();
        assert_eq!(go, vec![(124_000, 457_000)], "one, the latest");
    }

    #[test]
    fn a_credited_takedown_turns_the_just_dead_enemy_into_an_orb() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_akane"), champ(1, (x + 20_000, y), "foe")]);
        let mut k = Akane::default();
        host.borrow_mut().tick = 1;
        with_sim(&host, |sim| k.on_update(sim, 1, 0, 0));
        host.borrow_mut().units[1].hp = 0;
        host.borrow_mut().tick = 2;
        with_sim(&host, |sim| {
            k.on_kill(sim, 0, 0, 1);
            k.on_update(sim, 2, 0, 0);
        });
        assert_eq!(k.core.orbs.len(), 1);
        assert_eq!(k.core.orbs[0].pos, Point::new(x as i64 + 20_000, y as i64));
    }
}
