//! Kicker's core: the passives, the Wall Bounce and each skill (Bounced or not), Second Wind, and his AI's choices.
use crate::kicker_core::*;
use crate::kicker_data as d;

const ME: usize = 1;
const FOE: usize = 10;

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 3000, attack: 100, champion: true, visible: true, targetable: true, ..Unit::default() }
}
fn minion(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, farmable: true, hp: 300, max_hp: 300, ..champ(id, 2, x, y) }
}
fn tower(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, tower: true, hp: 3000, max_hp: 3000, ..champ(id, team, x, y) }
}
/// a wall grid with these (cx, cy) cells
fn walls(cells: &[(i64, i64)]) -> Vec<bool> {
    let mut g = vec![false; (CELLS * CELLS) as usize];
    for &(x, y) in cells {
        g[(y * CELLS + x) as usize] = true;
    }
    g
}
fn cell(cx: i64, cy: i64) -> Point {
    Point::new(cx * CELL + CELL / 2, cy * CELL + CELL / 2)
}
fn frame(tick: u64, pos: Point, units: Vec<Unit>) -> Frame {
    Frame {
        tick,
        owner: ME,
        team: 1,
        alive: true,
        pos,
        hp: 1500,
        max_hp: 1500,
        attack: 150,
        ult_unlocked: true,
        flash_ready: false,
        home: Point::new(96_000, 864_000),
        units,
        terrain: walls(&[]),
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
/// run ticks, moving him along his glides like the engine would, and kicked bodies along their knocks
fn run(c: &mut Core, f: &mut Frame, first: Option<Order>, from: u64, to: u64) -> Vec<Effect> {
    run_log(c, f, first, from, to).into_iter().map(|e| e.1).collect()
}
/// the same, with the tick of each effect
fn run_log(c: &mut Core, f: &mut Frame, first: Option<Order>, from: u64, to: u64) -> Vec<(u64, Effect)> {
    let mut all = vec![];
    let mut glide: Option<(Point, Point, u64, u64)> = None;
    for tick in from..to {
        f.tick = tick;
        f.order = if tick == from { first } else { None };
        if let Some((a, b, s, n)) = glide {
            let k = (tick - s).min(n) as i64;
            f.pos = Point::new(a.x + (b.x - a.x) * k / n as i64, a.y + (b.y - a.y) * k / n as i64);
            if tick >= s + n {
                glide = None;
            }
        }
        let out = c.step(f);
        for e in &out {
            match e {
                Effect::Glide { to, ticks } => glide = Some((f.pos, *to, tick, (*ticks).max(1))),
                Effect::Flash { to } => {
                    f.pos = *to;
                    f.flash_ready = false;
                    glide = None;
                }
                Effect::Knock { target, to, .. } | Effect::Carry { target, to, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::Return { to, hp } => {
                    f.pos = *to;
                    f.hp = *hp;
                }
                _ => {}
            }
        }
        all.extend(out.into_iter().map(|e| (tick, e)));
    }
    all
}
fn damage_to(out: &[Effect], id: usize) -> i64 {
    out.iter().map(|e| if let Effect::Damage { target, amount, .. } = e { if *target == id { *amount } else { 0 } } else { 0 }).sum()
}
fn knocked_up(out: &[Effect]) -> Vec<usize> {
    out.iter().filter_map(|e| if let Effect::KnockUp { target, .. } = e { Some(*target) } else { None }).collect()
}
fn knock_to(out: &[Effect], id: usize) -> Option<Point> {
    out.iter().find_map(|e| if let Effect::Knock { target, to, .. } = e { (*target == id).then_some(*to) } else { None })
}

// ------------------------------------------------------------------ the Wall Bounce

/// a wall cell (10, 10), him 50,000 west of its centre and an enemy 60,000 east of it
fn bounce_scene() -> (Core, Frame) {
    let w = cell(10, 10);
    let mut f = frame(0, Point::new(w.x - 50_000, w.y), vec![champ(FOE, 2, w.x + 60_000, w.y)]);
    f.terrain = walls(&[(10, 10)]);
    (Core::default(), f)
}

#[test]
fn a_wall_bounce_leaps_touches_hidden_then_springs_behind_the_target() {
    let (mut c, mut f) = bounce_scene();
    let foe = f.units[0].pos;
    let log = run_log(&mut c, &mut f, Some(Order::Bounce(foe)), 0, 120);
    let at = |e: &Effect| log.iter().find(|x| &x.1 == e).map(|x| x.0);
    let on = at(&Effect::Roof { on: true }).expect("the touch");
    let off = at(&Effect::Roof { on: false }).expect("the spring");
    assert_eq!(off - on, d::ROOF_TOUCH_TICKS as u64, "a 0.25 s touch");
    assert_eq!(at(&Effect::Hide), Some(on), "hidden at the touch");
    let out: Vec<Effect> = log.into_iter().map(|e| e.1).collect();
    // invulnerable and CC immune from the cast through the touch, then the spring
    let cc: Vec<u64> = out.iter().filter_map(|e| if let Effect::CcImmune { ticks } = e { Some(*ticks) } else { None }).collect();
    assert!(cc[0] >= on && out.iter().filter(|e| matches!(e, Effect::Invulnerable { .. })).count() >= 2, "{cc:?}");
    // he lands behind it, kicks it with the Drop-in bonus, knocks it up, and is shielded and Bounced
    assert!(!c.roof && f.pos.x > foe.x && !f.wall_at(f.pos), "{:?}", f.pos);
    let base = d::HOP_DROP_BASE + d::HOP_DROP_AD_RATIO * 150 / 100;
    assert_eq!(damage_to(&out, FOE), base * (100 + d::DROP_IN_BONUS_PCT) / 100);
    assert!(knocked_up(&out).contains(&FOE));
    assert!(out.contains(&Effect::Shield { amount: 1500 * d::ROOF_GROUND_SHIELD_PCT_MAX_HP / 100, ticks: d::ROOF_GROUND_SHIELD_TICKS as u64 }));
    assert!(out.contains(&Effect::Bounced { ticks: d::ROOF_BOUNCED_TICKS as u64 }));
    assert!(out.contains(&Effect::CcImmune { ticks: d::ROOF_CC_IMMUNE_TICKS as u64 }));
    assert!(c.bounced(119) && c.skyfall_ready(119));
    assert_eq!(c.hops, d::HOP_CHARGES, "a wall bounce uses no charge");
}

#[test]
fn wall_bounces_have_no_cooldown_but_the_drop_in_and_shield_keep_theirs() {
    let (mut c, mut f) = bounce_scene();
    let foe = f.units[0].pos;
    run(&mut c, &mut f, Some(Order::Bounce(foe)), 0, 100);
    // right after landing: again, off the same wall
    let again = f.units[0].pos;
    let out = run(&mut c, &mut f, Some(Order::Bounce(again)), 100, 200);
    assert!(out.contains(&Effect::Roof { on: true }), "a second bounce at once");
    assert!(knocked_up(&out).is_empty(), "the Drop-in is still cooling down");
    assert!(!out.contains(&Effect::Shield { amount: 1500 * d::ROOF_GROUND_SHIELD_PCT_MAX_HP / 100, ticks: d::ROOF_GROUND_SHIELD_TICKS as u64 }));
    assert_eq!(damage_to(&out, FOE), d::HOP_DROP_BASE + d::HOP_DROP_AD_RATIO * 150 / 100, "the landing kick, no bonus");
}

#[test]
fn bounced_turns_s2_into_rock_rain_and_s3_into_one_skyfall_for_three_seconds() {
    let mut c = Core { bounced_until: d::ROOF_BOUNCED_TICKS as u64, skyfall_until: d::ROOF_BOUNCED_TICKS as u64, ..Core::default() };
    let f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 385_000, 400_000)]);
    order(&mut c, &f, Order::Rock(Point::new(300_000, 400_000)));
    assert_eq!(c.rocks.len() as i64, d::ROCK_RAIN_COUNT);
    order(&mut c, &f, Order::Kick(FOE, Point::new(300_000, 400_000)));
    assert_eq!(c.last_skill, "skyfall");
    assert!(!c.skyfall_ready(1), "one Skyfall per bounce");
    // the next S3 is the ground kick again
    c.moving = None;
    c.last_kicked = None;
    order(&mut c, &f, Order::Kick(FOE, Point::new(300_000, 400_000)));
    assert_eq!(c.last_skill, "kick", "{}", c.rejection);
    // after the window: a single rock
    let mut c = Core { bounced_until: 180, ..Core::default() };
    let mut g = f.clone();
    g.tick = 181;
    order(&mut c, &g, Order::Rock(Point::new(300_000, 400_000)));
    assert_eq!(c.rocks.len(), 1);
}


#[test]
fn no_wall_in_reach_means_a_short_hop_that_uses_a_charge() {
    let mut c = Core::default();
    let f = frame(0, Point::new(400_000, 400_000), vec![]);
    let out = order(&mut c, &f, Order::Bounce(Point::new(500_000, 400_000)));
    assert!(out.contains(&Effect::Glide { to: Point::new(400_000 + d::HOP_GROUND_HOP, 400_000), ticks: d::HOP_MIN_TICKS as u64 }));
    assert_eq!(c.hops, d::HOP_CHARGES - 1);
    assert!(out.iter().any(|e| matches!(e, Effect::Pose { name: "hop_low", .. })), "a short leap is a low hop");
    assert!(!out.contains(&Effect::Roof { on: true }));
    let mut c = Core { hops: 0, hop_next: 999, ..Core::default() };
    order(&mut c, &f, Order::Bounce(Point::new(500_000, 400_000)));
    assert_eq!(c.rejection, "no wall in reach and no hop charge");
}








#[test]
fn every_dash_and_leap_gives_armor_and_cc_immunity_without_stacking() {
    let shields = |out: &[Effect]| out.iter().filter(|e| matches!(e, Effect::Shield { .. })).count();
    let immune = |out: &[Effect]| out.iter().any(|e| matches!(e, Effect::CcImmune { .. }));
    let amount = 1500 * d::DASH_SHIELD_PCT_MAX_HP / 100;
    // a short hop
    let mut c = Core::default();
    let f = frame(0, Point::new(400_000, 400_000), vec![]);
    let out = order(&mut c, &f, Order::Bounce(Point::new(500_000, 400_000)));
    assert!(out.contains(&Effect::Shield { amount, ticks: d::DASH_SHIELD_TICKS as u64 }) && immune(&out));
    // a second leap inside the shield's window adds none
    let mut f2 = f.clone();
    f2.tick = 30;
    c.moving = None;
    assert_eq!(shields(&order(&mut c, &f2, Order::Bounce(Point::new(300_000, 400_000)))), 0);
    // the Homecoming Kick's dash-in
    let mut c = Core::default();
    let k = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000)]);
    let out = order(&mut c, &k, Order::Kick(FOE, Point::new(300_000, 400_000)));
    assert!(shields(&out) == 1 && immune(&out));
    // the Skyfall dive, and a Wall Bounce
    let mut c = Core { skyfall_until: 999, ..Core::default() };
    let w = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 380_000, 400_000)]);
    let out = order(&mut c, &w, Order::Skyfall(Point::new(300_000, 400_000)));
    assert!(shields(&out) == 1 && immune(&out));
    let (mut c, b) = bounce_scene();
    let out = order(&mut c, &b, Order::Bounce(b.units[0].pos));
    assert!(shields(&out) == 1 && immune(&out));
    // the Marked Dash
    let mut c = Core::default();
    c.marks.push((FOE, 999));
    let out = order(&mut c, &w, Order::Dash(FOE));
    assert!(shields(&out) == 1 && immune(&out));
}

#[test]
fn flash_is_held_unless_the_pick_is_worth_it() {
    let c = Core::default();
    // his team is east; the target west of him: only a Flash reaches its west side
    let base = |hp: i64| {
        let mut g = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 384_000, 400_000)]);
        g.units[0].hp = hp;
        g.home = Point::new(900_000, 400_000);
        g.profile[SKILL_HIT] = 100;
        g.units.push(tower(40, 2, 280_000, 400_000));
        // a wall east of him: the kick east stops on it and stuns (and his Marked Dash can follow)
        g.terrain = walls(&[(13, 12)]);
        g.flash_ready = true;
        g
    };
    // healthy and no teammate at the landing: hold Flash
    let g = base(1000);
    assert!(!matches!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None), Some(Order::FlashKick(..))));
    // low: worth it
    let g = base(300);
    assert!(matches!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None), Some(Order::FlashKick(..))));
    // but not when he's low himself, into a crowd, or by a low-judgement athlete
    let mut g = base(300);
    g.hp = 400;
    assert!(!matches!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None), Some(Order::FlashKick(..))));
    let mut g = base(300);
    g.units.push(champ(21, 2, 370_000, 410_000));
    g.units.push(champ(22, 2, 365_000, 390_000));
    assert!(!matches!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None), Some(Order::FlashKick(..))));
    let mut g = base(300);
    g.profile[JUDGEMENT] = 20;
    assert!(!matches!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None), Some(Order::FlashKick(..))));
}



// ------------------------------------------------------------------ marks, dash, Bruised

#[test]
fn rock_throw_passes_through_everything_and_marks_all_but_towers() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![minion(30, 430_000, 400_000), champ(FOE, 2, 460_000, 400_000), tower(40, 2, 480_000, 400_000), champ(11, 2, 500_000, 400_000), champ(5, 1, 440_000, 400_000)]);
    let out = run(&mut c, &mut f, Some(Order::Rock(Point::new(600_000, 400_000))), 0, 60);
    for id in [30, FOE, 11] {
        assert!(damage_to(&out, id) > 0, "{id} hit");
        assert!(c.marked(id, 60), "{id} marked");
    }
    assert_eq!(damage_to(&out, 40), 0, "never a tower");
    assert!(!c.marked(40, 60) && !c.marked(5, 60));
    assert_eq!(damage_to(&out, 5), 0, "never an ally");
}

#[test]
fn rock_rain_while_bounced_is_three_rocks_with_less_damage_each() {
    let mut c = Core { bounced_until: 999, ..Core::default() };
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 450_000, 400_000)]);
    order(&mut c, &f.clone(), Order::Rock(f.units[0].pos));
    assert_eq!(c.rocks.len() as i64, d::ROCK_RAIN_COUNT);
    let out = run(&mut c, &mut f, None, 1, 60);
    let one = (d::ROCK_BASE + d::ROCK_AD_RATIO * 150 / 100) * d::ROCK_RAIN_PCT / 100;
    assert_eq!(damage_to(&out, FOE), one, "the centre rock hits it once");
}

#[test]
fn marked_dash_needs_a_mark_spends_a_charge_and_shields_him() {
    let mut c = Core::default();
    let f = frame(0, Point::new(400_000, 400_000), vec![minion(30, 460_000, 400_000)]);
    order(&mut c, &f, Order::Dash(30));
    assert_eq!(c.rejection, "not marked");
    c.marks.push((30, 200));
    let out = order(&mut c, &f, Order::Dash(30));
    assert_eq!(c.dashes, d::DASH_CHARGES - 1);
    assert!(out.contains(&Effect::Shield { amount: 1500 * d::DASH_SHIELD_PCT_MAX_HP / 100, ticks: d::DASH_SHIELD_TICKS as u64 }));
    let stop = out.iter().find_map(|e| if let Effect::Glide { to, .. } = e { Some(*to) } else { None }).unwrap();
    assert!(stop.x > 460_000 && stop.within(Point::new(460_000, 400_000), 3_000 + d::DASH_GAP + 10), "ends behind it: {stop:?}");
    let far = frame(0, Point::new(400_000, 400_000), vec![minion(31, 400_000 + d::DASH_RANGE + 5_000, 400_000)]);
    c.moving = None;
    c.marks.push((31, 200));
    order(&mut c, &far, Order::Dash(31));
    assert_eq!(c.rejection, "out of dash range");
}

#[test]
fn bruised_comes_only_from_his_cc_and_his_hits_stack_it_to_five() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 410_000, 400_000)]);
    // a basic attack alone marks but does not bruise
    f.basic_hits = vec![FOE];
    let out = c.step(&f);
    assert!(c.marked(FOE, 1));
    assert!(!out.iter().any(|e| matches!(e, Effect::Bruise { .. })));
    // after a CC of his, each hit adds a stack, capped
    c.bruised.push((FOE, 1_000, 0));
    let mut stacks = vec![];
    for tick in 1..10 {
        f.tick = tick;
        for e in c.step(&f) {
            if let Effect::Bruise { stacks: s, .. } = e {
                stacks.push(s);
            }
        }
    }
    assert_eq!(stacks, vec![1, 2, 3, 4, 5]);
    assert_eq!(c.bruise_stacks(FOE, 10), Some(d::BRUISED_MAX_STACKS));
    f.tick = 1_000;
    f.basic_hits.clear();
    assert!(c.step(&f).contains(&Effect::BruiseEnd { target: FOE }));
}

// ------------------------------------------------------------------ S3

#[test]
fn homecoming_kick_dashes_behind_and_sends_it_away_from_him() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000)]);
    // kick it west (toward x smaller): he must stand east of it
    let out = run(&mut c, &mut f, Some(Order::Kick(FOE, Point::new(300_000, 400_000))), 0, 60);
    assert!(f.pos.x > 412_000, "he dashed behind it: {:?}", f.pos);
    let to = knock_to(&out, FOE).expect("kicked");
    assert_eq!(to.y, 400_000);
    assert!((412_000 - to.x - d::KICK_DISTANCE).abs() <= 2_000, "{to:?}");
    assert_eq!(damage_to(&out, FOE), d::KICK_BASE + d::KICK_AD_RATIO * 150 / 100);
    assert!(out.iter().any(|e| matches!(e, Effect::Strike { target: FOE })), "a parryable strike");
    assert_eq!(c.kicks, d::KICK_CHARGES - 1);
}

#[test]
fn a_kick_spot_beyond_the_short_dash_is_refused() {
    let mut c = Core::default();
    // it stands west of him; kicking it east means standing west of it, 30,000 away: too far for the dash
    let f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 385_000, 400_000)]);
    order(&mut c, &f, Order::Kick(FOE, Point::new(600_000, 400_000)));
    assert_eq!(c.rejection, "kick spot out of reach");
    assert_eq!(c.kicks, d::KICK_CHARGES);
}

#[test]
fn the_flight_stops_at_a_wall_and_stuns_and_never_lands_in_terrain() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(cell(10, 12).x + 8_000, cell(10, 12).y), vec![champ(FOE, 2, cell(10, 12).x, cell(10, 12).y)]);
    f.terrain = walls(&[(8, 12)]);
    let out = run(&mut c, &mut f, Some(Order::Kick(FOE, Point::new(0, cell(10, 12).y))), 0, 80);
    let to = knock_to(&out, FOE).unwrap();
    assert!(!f.wall_at(to) && to.x >= 9 * CELL - 2_000, "stopped before the wall: {to:?}");
    assert!(out.iter().any(|e| matches!(e, Effect::Stun { target: FOE, ticks } if *ticks == d::KICK_WALL_STUN_TICKS as u64)));
    // off the map edge: never
    let mut c = Core::default();
    let mut f = frame(0, Point::new(30_000, 400_000), vec![champ(FOE, 2, 20_000, 400_000)]);
    let out = run(&mut c, &mut f, Some(Order::Kick(FOE, Point::new(-200_000, 400_000))), 0, 80);
    let to = knock_to(&out, FOE).unwrap();
    assert!(to.x >= 2_000 && to.on_map(), "{to:?}");
}

#[test]
fn the_kicked_body_knocks_up_at_most_two_others() {
    let mut c = Core::default();
    let mut units = vec![champ(FOE, 2, 412_000, 400_000)];
    for (i, x) in [380_000, 360_000, 340_000].iter().enumerate() {
        units.push(champ(20 + i, 2, *x, 400_000));
    }
    let mut f = frame(0, Point::new(400_000, 400_000), units);
    let out = run(&mut c, &mut f, Some(Order::Kick(FOE, Point::new(300_000, 400_000))), 0, 120);
    let k = knocked_up(&out);
    assert_eq!(k.len() as i64, d::KICK_SECONDARY_MAX, "{k:?}");
    assert!(k.contains(&20) && k.contains(&21) && !k.contains(&22), "the first two it passes: {k:?}");
    assert!(damage_to(&out, 20) > 0 && damage_to(&out, 22) == 0);
}

#[test]
fn skyfall_carries_the_first_champion_through_walls_and_lands_him_behind_it() {
    let mut c = Core { skyfall_until: 999, ..Core::default() };
    let start = cell(10, 10);
    let mut f = frame(0, start, vec![champ(FOE, 2, start.x - 20_000, start.y), champ(11, 2, start.x - 50_000, start.y)]);
    // a wall right where the dive ends: the carry goes through it and lands both on open ground
    f.terrain = walls(&[(8, 10)]);
    let out = run(&mut c, &mut f, Some(Order::Skyfall(Point::new(start.x - 100_000, start.y))), 0, 60);
    let carried = out.iter().find_map(|e| if let Effect::Carry { target, to, .. } = e { Some((*target, *to)) } else { None }).expect("a carry");
    assert_eq!(carried.0, FOE, "the first champion in the line");
    assert!(!f.wall_at(carried.1), "it lands on open ground: {:?}", carried.1);
    assert!(f.pos.x < carried.1.x, "he ends behind it (farther along the dive): {:?} vs {:?}", f.pos, carried.1);
    assert!(!f.wall_at(f.pos));
    assert!(out.iter().any(|e| matches!(e, Effect::PassWalls { .. })));
    assert!(!c.roof && !c.skyfall_ready(59), "one dive per bounce");
    assert!(knocked_up(&out).contains(&FOE), "Drop-in on landing");
    assert_eq!(c.kicks, d::KICK_CHARGES - 1);
}


#[test]
fn flash_kick_when_the_dash_cannot_reach_a_safe_side_and_flash_re_aims_a_wind_up() {
    let mut c = Core::default();
    // his teammate is east of him; the enemy is between them? No: the enemy is west of him and his team is east,
    // so a kick toward his team needs him west of the enemy, 30,000 away: only Flash gets there
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 380_000, 400_000), champ(5, 1, 470_000, 400_000)]);
    f.manual = false;
    f.home = Point::new(900_000, 400_000);
    f.profile[SKILL_HIT] = 100;
    assert!(c.plan_kick(&f, f.unit(FOE).unwrap(), None).is_some_and(|o| matches!(o, Order::Kick(..))) || !f.flash_ready);
    f.flash_ready = true;
    let plan = c.plan_kick(&f, &f.unit(FOE).unwrap().clone(), None);
    assert!(matches!(plan, Some(Order::Kick(..)) | Some(Order::FlashKick(..))), "{plan:?}");
    // the forced case: only directions east are safe (towers west and south), and he stands east of it
    let mut g = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 384_000, 400_000), champ(5, 1, 470_000, 400_000)]);
    g.home = Point::new(900_000, 400_000);
    g.profile[SKILL_HIT] = 100;
    // the tower makes every westward landing unsafe, but its range ends short of his Flash spot
    g.units.push(tower(40, 2, 280_000, 400_000));
    g.flash_ready = true;
    let plan = c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None);
    assert!(matches!(plan, Some(Order::FlashKick(FOE, to)) if to.x > 384_000), "{plan:?}");
    let mut gg = g.clone();
    let out = run(&mut c, &mut gg, plan, 0, 60);
    let to = knock_to(&out, FOE).unwrap();
    assert!(to.x > 384_000, "kicked toward his team after the Flash: {to:?}");
    // re-aim: a kick already winding up turns when he Flashes to the other side (no second charge)
    let mut c = Core::default();
    let h = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 410_000, 400_000)]);
    order(&mut c, &h, Order::Kick(FOE, Point::new(600_000, 400_000)));
    let charges = c.kicks;
    let mut h2 = h.clone();
    h2.tick = 2;
    h2.flash_ready = true;
    order(&mut c, &h2, Order::FlashKick(FOE, Point::new(200_000, 400_000)));
    assert_eq!(c.kicks, charges);
    assert!(c.kick.as_ref().is_some_and(|k| k.dir.0 < 0.0 && k.flashed));
}

#[test]
fn kick_directions_prefer_a_teammate_then_out_of_their_tower_and_never_into_their_team() {
    let c = Core::default();
    // a teammate north of the target: kick it to them
    let f = frame(0, Point::new(400_000, 420_000), vec![champ(FOE, 2, 400_000, 400_000), champ(5, 1, 400_000, 340_000)]);
    let best = c.kick_options(&f, f.unit(FOE).unwrap(), None)[0];
    assert!(best.1 .1 < -0.9, "toward the teammate: {:?}", best.1);
    // under its tower with no teammate: out of the tower's range
    let g = frame(0, Point::new(400_000, 420_000), vec![champ(FOE, 2, 400_000, 400_000), tower(40, 2, 400_000, 330_000)]);
    let best = c.kick_options(&g, g.unit(FOE).unwrap(), None)[0];
    assert!(g.enemy_tower_at(best.2).is_none() && best.1 .1 > 0.9, "out from under it: {best:?}");
    // two of its team where a direction lands: that direction is gone
    let h = frame(0, Point::new(400_000, 420_000), vec![champ(FOE, 2, 400_000, 400_000), champ(5, 1, 400_000, 340_000), champ(21, 2, 400_000, 325_000), champ(22, 2, 405_000, 330_000)]);
    let opts = c.kick_options(&h, h.unit(FOE).unwrap(), None);
    assert!(opts.iter().all(|o| o.1 .1 > -0.9), "not into its team: {opts:?}");
}

#[test]
fn peel_kicks_the_diver_away_from_the_hurt_teammate() {
    let c = Core { kicks: 1, ..Core::default() };
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(5, 1, 420_000, 400_000), champ(FOE, 2, 410_000, 400_000)]);
    f.units[0].hp = 400;
    f.profile[SKILL_HIT] = 100;
    // peel may spend the held charge
    let o = c.plan_kick(&f, f.unit(FOE).unwrap(), Some(f.unit(5).unwrap().pos));
    assert!(matches!(o, Some(Order::Kick(FOE, to)) if to.x < 410_000), "away from the teammate: {o:?}");
}

#[test]
fn one_kick_charge_is_held_for_picks() {
    let mut c = Core { kicks: 1, ..Core::default() };
    let f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 410_000, 400_000), champ(5, 1, 330_000, 400_000)]);
    let u = f.unit(FOE).unwrap().clone();
    assert!(c.plan_kick(&f, &u, None).is_none(), "the last charge waits for a pick");
    c.last_kicked = Some((FOE, 100));
    assert!(c.plan_kick(&f, &u, None).is_none(), "the target he just kicked is hit, not kicked away again");
    c.last_kicked = None;
    let mut g = f.clone();
    g.units[1].pos = Point::new(420_000, 400_000);
    assert!(c.plan_kick(&g, &g.unit(FOE).unwrap().clone(), None).is_some(), "a target fighting his teammate may take the held charge");
    c.kicks = 2;
    c.last_kicked = None;
    assert!(c.plan_kick(&f, &u, None).is_some());
}

#[test]
fn no_second_kick_at_a_body_still_in_the_air() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), champ(5, 1, 330_000, 400_000)]);
    run(&mut c, &mut f, Some(Order::Kick(FOE, Point::new(300_000, 400_000))), 0, 25);
    assert!(!c.flights.is_empty(), "it's flying");
    let u = f.unit(FOE).unwrap().clone();
    assert!(c.plan_kick(&f, &u, None).is_none());
}

#[test]
fn a_parry_up_means_no_kick() {
    let c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 410_000, 400_000), champ(5, 1, 330_000, 400_000)]);
    f.units[0].parry = true;
    assert!(c.plan_kick(&f, f.unit(FOE).unwrap(), None).is_none());
}

// ------------------------------------------------------------------ ult

#[test]
fn second_wind_saves_a_death_at_full_hp_and_otherwise_returns_with_the_cast_hp() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![]);
    f.hp = 900;
    let out = order(&mut c, &f, Order::Ult);
    assert!(out.contains(&Effect::Undying { ticks: d::ULT_TICKS as u64 + 2 }));
    // a lethal hit: undying leaves him at 1, he's back at the tag at full HP
    f.pos = Point::new(470_000, 400_000);
    f.hp = 1;
    f.tick = 100;
    let out = c.step(&f);
    assert!(out.contains(&Effect::Return { to: Point::new(400_000, 400_000), hp: 1500 }));
    assert!(out.contains(&Effect::UndyingEnd));
    // the timer: back with the HP he had (no free heal)
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![]);
    f.hp = 900;
    order(&mut c, &f, Order::Ult);
    f.hp = 1400;
    f.tick = d::ULT_TICKS as u64;
    f.pos = Point::new(470_000, 400_000);
    let out = c.step(&f);
    assert!(out.contains(&Effect::Return { to: Point::new(400_000, 400_000), hp: 900 }));
    order(&mut c, &f, Order::Ult);
    assert_eq!(c.rejection, "ult cooling down");
}

// ------------------------------------------------------------------ AI

#[test]
fn ai_farms_with_the_rock_only_and_never_kicks_minions() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![minion(30, 440_000, 400_000), minion(31, 460_000, 400_000), minion(32, 480_000, 400_000)]);
    f.manual = false;
    let mut kinds = vec![];
    for tick in 0..300 {
        f.tick = tick;
        c.step(&f);
        if !c.last_skill.is_empty() {
            kinds.push(c.last_skill);
        }
    }
    assert!(kinds.contains(&"rock"), "{kinds:?}");
    assert!(!kinds.iter().any(|k| matches!(*k, "kick" | "skyfall" | "second_wind" | "flash_kick")), "{kinds:?}");
    assert_eq!(c.kicks, d::KICK_CHARGES);
}

#[test]
fn ai_picks_an_isolated_target_and_kicks_it_toward_his_teammate() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), champ(5, 1, 330_000, 400_000)]);
    f.manual = false;
    f.units[0].hp = 500;
    f.profile[EGO] = 70;
    f.profile[SKILL_HIT] = 100;
    let out = run(&mut c, &mut f, None, 0, 60);
    let to = knock_to(&out, FOE).expect("a kick");
    assert!(to.x < 412_000, "toward the teammate: {to:?}");
}

#[test]
fn ai_bounces_off_a_wall_onto_a_target_then_dives_with_a_skyfall() {
    let (mut c, mut f) = bounce_scene();
    f.manual = false;
    f.ult_unlocked = false;
    // hurt and alone, out of reach on foot (110,000 away), a wall between
    f.units[0].hp = 350;
    let out = run(&mut c, &mut f, None, 0, 160);
    assert!(out.contains(&Effect::Roof { on: true }), "wall first");
    assert!(damage_to(&out, FOE) > 0 && knocked_up(&out).contains(&FOE));
    assert!(out.iter().any(|e| matches!(e, Effect::Carry { target: FOE, .. })), "then the Skyfall");
}

#[test]
fn ai_never_bounces_into_a_crowd_or_under_a_tower() {
    let mut o = vec![];
    // the same target with two of its team beside it
    let (mut c, mut f) = bounce_scene();
    f.manual = false;
    f.ult_unlocked = false;
    f.units[0].hp = 350;
    let foe = f.units[0].pos;
    f.units.push(champ(11, 2, foe.x + 5_000, foe.y + 10_000));
    f.units.push(champ(12, 2, foe.x + 8_000, foe.y - 10_000));
    assert!(!matches!(c.decide(&f, &mut o), Some(Order::Bounce(_))));
    // alone but under its tower, with no kick-out
    let (mut c, mut f) = bounce_scene();
    f.manual = false;
    f.ult_unlocked = false;
    f.units.push(tower(40, 2, foe.x + 20_000, foe.y));
    assert!(!matches!(c.decide(&f, &mut o), Some(Order::Bounce(_))));
}

fn ai(pos: Point, units: Vec<Unit>) -> Frame {
    let mut f = frame(0, pos, units);
    f.manual = false;
    f
}
fn steers(out: &[Effect]) -> Vec<Point> {
    out.iter().filter_map(|e| if let Effect::Steer { to } = e { Some(*to) } else { None }).collect()
}

#[test]
fn no_lock_on_under_a_tower_without_a_kick_out_and_no_aimless_steering() {
    let mut c = Core::default();
    // a healthy enemy under its tower, 70,000 away: no kick-out ready, so not a target, and no walk toward it
    let f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 470_000, 400_000), tower(40, 2, 490_000, 400_000)]);
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), None);
    assert!(steers(&o).is_empty(), "{o:?}");
    // a healthy enemy in the open with no fight on: the game walks him, not a lock-on
    let mut c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 470_000, 400_000)]);
    f.profile[EGO] = 90;
    c.rock_ready = 10_000;
    c.kicks = 0;
    let mut o = vec![];
    c.decide(&f, &mut o);
    assert!(steers(&o).is_empty(), "{o:?}");
}

#[test]
fn he_picks_the_near_isolated_enemy_over_a_far_low_one() {
    let c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 420_000, 400_000), champ(11, 2, 485_000, 400_000), champ(12, 2, 490_000, 410_000)]);
    f.units[1].hp = 300; // far, low, but next to a teammate of its
    f.profile[EGO] = 80;
    assert_eq!(c.pick(&f, d::AI_ENGAGE_RANGE).map(|u| u.id), Some(FOE));
}

#[test]
fn a_tower_kick_out_is_a_target_only_when_it_is_ready() {
    let mut c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), tower(40, 2, 470_000, 400_000)]);
    f.profile[EGO] = 80;
    assert!(c.kickout_ready(&f, f.unit(FOE).unwrap()), "a charge, a safe way out, in reach");
    assert_eq!(c.pick(&f, d::AI_ENGAGE_RANGE).map(|u| u.id), Some(FOE));
    c.kicks = 0;
    assert!(c.pick(&f, d::AI_ENGAGE_RANGE).is_none(), "no charge: no tower target");
    c.kicks = 2;
    f.hp = 500;
    assert!(c.pick(&f, d::AI_ENGAGE_RANGE).is_none(), "too hurt for a dive");
}

#[test]
fn he_joins_a_teamfight_whatever_his_ego() {
    let mut c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 440_000, 400_000), champ(5, 1, 450_000, 400_000)]);
    f.profile[EGO] = 10;
    f.ult_unlocked = false;
    let mut o = vec![];
    let order = c.decide(&f, &mut o);
    assert!(matches!(order, Some(Order::Rock(_)) | Some(Order::Kick(..))) || !steers(&o).is_empty(), "{order:?} {o:?}");
    assert!(Core::fight_on(&f));
}

#[test]
fn second_wind_death_cancel_scales_with_ego_and_is_never_cast_alone_and_safe() {
    let c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 430_000, 400_000)]);
    f.hp = 300; // 20%
    f.profile[EGO] = 50;
    assert!(c.death_cancel(&f), "20% with an enemy close");
    f.flash_ready = true;
    assert!(c.death_cancel(&f), "Flash ready or not");
    let mut alone = ai(Point::new(400_000, 400_000), vec![]);
    alone.hp = 300;
    assert!(!c.death_cancel(&alone), "alone and safe: keep it");
    alone.shots_near = 1;
    assert!(c.death_cancel(&alone), "a projectile closing in");
    // Ego: 20 casts it earlier than 80
    let mut low = f.clone();
    low.hp = 360; // 24%
    low.profile[EGO] = 20;
    let mut high = low.clone();
    high.profile[EGO] = 80;
    assert!(c.death_cancel(&low) && !c.death_cancel(&high));
}

#[test]
fn after_a_death_cancel_he_goes_all_in_instead_of_running() {
    let mut c = Core { kicks: 1, ..Core::default() };
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), champ(11, 2, 420_000, 410_000), minion(30, 330_000, 400_000), champ(5, 1, 330_000, 420_000)]);
    f.hp = 250;
    f.profile[SKILL_HIT] = 100;
    c.marks.push((30, 10_000));
    c.hp_log = vec![(0, 700)];
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Ult), "the death cancel first");
    c.execute(&f, Order::Ult, &mut o);
    f.tick = 1;
    let order = c.decide(&f, &mut o);
    assert!(matches!(order, Some(Order::Kick(..)) | Some(Order::Rock(_)) | Some(Order::Dash(FOE)) | Some(Order::Dash(11))), "fights, no escape: {order:?}");
    assert_ne!(order, Some(Order::Dash(30)));
}

#[test]
fn ego_decides_the_fearless_engage() {
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), champ(11, 2, 430_000, 420_000), champ(5, 1, 330_000, 400_000)]);
    f.profile[SKILL_HIT] = 100;
    let u = f.unit(FOE).unwrap().clone();
    f.profile[EGO] = 80;
    assert!(Core::default().engage_tag(&f, &u), "Ego 80 tags into a 1v2");
    f.profile[EGO] = 20;
    assert!(!Core::default().engage_tag(&f, &u), "Ego 20 waits");
    // never with him standing under an enemy tower (the tag would bring him back there)
    let mut g = f.clone();
    g.profile[EGO] = 80;
    g.units.push(tower(40, 2, 420_000, 400_000));
    assert!(!Core::default().engage_tag(&g, &u));
    // and the tag goes down the tick before the kick
    let mut c = Core::default();
    let mut h = f.clone();
    h.profile[EGO] = 80;
    let mut o = vec![];
    assert_eq!(c.decide(&h, &mut o), Some(Order::Ult));
    c.execute(&h, Order::Ult, &mut o);
    h.tick = 1;
    assert!(matches!(c.decide(&h, &mut o), Some(Order::Kick(FOE, _))));
}

#[test]
fn second_wind_at_the_start_of_a_teamfight() {
    let c = Core::default();
    let f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 440_000, 400_000), champ(11, 2, 450_000, 420_000), champ(5, 1, 445_000, 405_000)]);
    assert!(c.teamfight_tag(&f));
    let mut g = f.clone();
    g.profile[EGO] = 20;
    assert!(!c.teamfight_tag(&g), "Ego under 35 needs three enemies");
}

#[test]
fn under_the_tag_a_target_in_a_pair_is_fair_game_but_never_a_1v3() {
    let mut c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 430_000, 400_000), champ(11, 2, 440_000, 410_000)]);
    f.profile[EGO] = 30;
    assert!(c.pick(&f, d::AI_ENGAGE_RANGE).is_none(), "two together: not alone");
    c.ult = Some(SecondWind { anchor: f.pos, hp: 1500, until: 400, roof: false });
    assert!(c.pick(&f, d::AI_ENGAGE_RANGE).is_some());
    f.units.push(champ(12, 2, 445_000, 390_000));
    assert!(c.pick(&f, d::AI_ENGAGE_RANGE).is_none(), "three against him alone: outnumbered, tag or not");
}




#[test]
fn a_wall_between_him_and_the_target_is_bounced_over() {
    let mut c = Core::default();
    let w = cell(10, 10);
    let mut f = ai(Point::new(w.x + 30_000, w.y), vec![champ(FOE, 2, w.x - 40_000, w.y)]);
    f.terrain = walls(&[(10, 10)]);
    f.profile[EGO] = 80;
    f.ult_unlocked = false;
    c.rock_ready = 10_000;
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Bounce(Point::new(w.x - 40_000, w.y))));
}






#[test]
fn on_the_ground_he_goes_to_a_teammate_in_a_fight() {
    let mut o = vec![];
    let units = || vec![minion(31, 340_000, 400_000), champ(5, 1, 250_000, 400_000), champ(FOE, 2, 240_000, 410_000)];
    let mut c = Core { rock_ready: 10_000, ..Core::default() };
    c.marks.push((31, 1_000));
    let f = ai(Point::new(400_000, 400_000), units());
    assert_eq!(c.decide(&f, &mut o), Some(Order::Dash(31)));
    // nothing marked: he walks there
    let mut c = Core { rock_ready: 10_000, ..Core::default() };
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), None);
    assert_eq!(steers(&o), vec![Point::new(250_000, 400_000)]);
}



#[test]
fn retreats_bounce_off_a_wall_toward_safety() {
    let w = cell(10, 10);
    // hurt among enemies east of him, a wall close: a bounce toward home (south-west), away from them
    let mut c = Core::default();
    let mut f = ai(Point::new(w.x + 30_000, w.y), vec![champ(FOE, 2, w.x + 70_000, w.y), champ(11, 2, w.x + 75_000, w.y + 10_000)]);
    f.terrain = walls(&[(10, 10)]);
    f.ult_unlocked = false;
    f.hp = 500;
    c.hp_log = vec![(0, 900)];
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Bounce(f.home)));
    // no wall in reach: no bounce (the game walks him out)
    let mut c = Core::default();
    let mut g = f.clone();
    g.terrain = walls(&[]);
    c.hp_log = vec![(0, 900)];
    assert!(!matches!(c.decide(&g, &mut o), Some(Order::Bounce(_))));
}

#[test]
fn a_teammate_dived_out_of_reach_is_joined_by_a_bounce_off_a_wall() {
    // his teammate is dived 130,000 west; nothing marked; a wall 62,000 west of him with the diver in spring range
    let mut c = Core { rock_ready: 10_000, ..Core::default() };
    let mut f = ai(Point::new(380_000, 400_000), vec![champ(5, 1, 250_000, 400_000), champ(FOE, 2, 240_000, 410_000)]);
    f.terrain = walls(&[(9, 12)]);
    f.ult_unlocked = false;
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Bounce(Point::new(240_000, 410_000))));
}

#[test]
fn a_kick_leads_to_hits_not_a_pushback() {
    let c = Core::default();
    // no teammate anywhere, open ground: no direction worth a kick (it would only push the enemy away)
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 410_000, 400_000)]);
    f.profile[SKILL_HIT] = 100;
    assert!(c.kick_options(&f, f.unit(FOE).unwrap(), None).is_empty());
    // a teammate where it would land: toward them
    let mut g = f.clone();
    g.units.push(champ(5, 1, 340_000, 400_000));
    let opts = c.kick_options(&g, g.unit(FOE).unwrap(), None);
    assert!(!opts.is_empty() && opts[0].1 .0 < -0.9);
    // into a wall only when he can follow it
    let mut h = f.clone();
    h.terrain = walls(&[(11, 12)]); // x 352,000..384,000: the kick west stops on it, 26,000 from him
    assert!(!c.kick_options(&h, h.unit(FOE).unwrap(), None).is_empty());
    let none = Core { dashes: 0, ..Core::default() };
    let mut far = f.clone();
    far.terrain = walls(&[(15, 12)]); // east, 75,000 out: too far to follow without a dash
    assert!(none.kick_options(&far, far.unit(FOE).unwrap(), None).is_empty());
}

#[test]
fn after_a_kick_he_goes_after_the_kicked_target() {
    let mut c = Core::default();
    let mut f = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 470_000, 400_000), champ(5, 1, 480_000, 410_000)]);
    f.ult_unlocked = false;
    c.last_kicked = Some((FOE, 200));
    c.marks.push((FOE, 200));
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Dash(FOE)));
    // no dash charge: he walks to it
    let mut c = Core { dashes: 0, ..Core::default() };
    c.last_kicked = Some((FOE, 200));
    let mut o = vec![];
    assert_eq!(c.decide(&f, &mut o), None);
    assert_eq!(steers(&o), vec![Point::new(470_000, 400_000)]);
}

#[test]
fn he_doesnt_join_a_fight_four_against_two() {
    let f = ai(
        Point::new(400_000, 400_000),
        vec![champ(5, 1, 250_000, 400_000), champ(FOE, 2, 240_000, 410_000), champ(11, 2, 245_000, 390_000), champ(12, 2, 235_000, 400_000), champ(13, 2, 250_000, 420_000)],
    );
    assert_eq!(Core::help_target(&f), None);
    assert!(Core::default().pick(&f, d::AI_HELP_RANGE).is_none());
    // a 2v2: he goes
    let g = ai(Point::new(400_000, 400_000), vec![champ(5, 1, 250_000, 400_000), champ(6, 1, 255_000, 405_000), champ(FOE, 2, 240_000, 410_000), champ(11, 2, 245_000, 390_000)]);
    assert!(Core::help_target(&g).is_some());
    // Ego 80 doesn't tag into a 1v3
    let mut h = ai(Point::new(400_000, 400_000), vec![champ(FOE, 2, 412_000, 400_000), champ(11, 2, 420_000, 410_000), champ(12, 2, 425_000, 390_000)]);
    h.profile[EGO] = 80;
    assert!(!Core::default().engage_tag(&h, &h.unit(FOE).unwrap().clone()));
}



#[test]
fn bounced_basics_are_45000_rock_throws_with_less_damage_taken() {
    let b = crate::kicker::bounced_buff(180);
    assert_eq!(b.range as i64 + d::BASIC_RANGE, d::ROOF_BASIC_RANGE);
    assert_eq!(d::ROOF_BASIC_RANGE, 45_000);
    assert_eq!(b.damaged_reduce as i64, d::ROOF_REDUCE_PCT);
    assert_eq!(d::ROOF_REDUCE_PCT, 25);
}

#[test]
fn ai_escapes_only_after_real_damage() {
    let mut c = Core::default();
    let mut f = frame(0, Point::new(400_000, 400_000), vec![champ(FOE, 2, 420_000, 400_000), champ(21, 2, 430_000, 410_000), minion(30, 340_000, 400_000)]);
    f.manual = false;
    f.ult_unlocked = false;
    f.profile[EGO] = 0;
    c.marks.push((30, 1_000));
    f.hp = 600;
    let mut o = vec![];
    // danger alone: no escape dash
    c.hp_log = vec![(0, 600)];
    assert_ne!(c.decide(&f, &mut o), Some(Order::Dash(30)));
    // real damage: dash to the marked minion away from them
    c.hp_log = vec![(0, 900)];
    assert_eq!(c.decide(&f, &mut o), Some(Order::Dash(30)));
}

// ------------------------------------------------------------------ the bridge, on the stateful mock host

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::kicker::Kicker;
    use crate::kicker_core::{Effect, Point};
    use crate::kicker_data as d;
    use mod_api_stable::{AttackTypeV1, CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn a_parried_kick_sets_off_nothing_and_towers_take_nothing() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kicker"),
            champ(1, (x + 10_000, y), "tfm2_ultrakill_v1"),
            HostUnit { kind: K::Tower, team: 1, pos: (x + 20_000, y), hp: 3000, name: "tower", targetable: true },
            champ(1, (x + 30_000, y), "foe"),
        ]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| sim.add_buff(1, &crate::timed("v1_guard", 600)));
        host.borrow_mut().tick = 5;
        let to = Point::new(x as i64 - 60_000, y as i64);
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Strike { target: 1 },
                    Effect::Damage { target: 1, amount: 250, direct: true },
                    Effect::Knock { target: 1, to, ticks: 30 },
                    Effect::Bruise { target: 1, stacks: 0, ticks: 240 },
                    Effect::StrikeEnd,
                    Effect::Damage { target: 2, amount: 90, direct: false },
                    Effect::KnockUp { target: 2, ticks: 30 },
                    Effect::Damage { target: 3, amount: 90, direct: false },
                ],
            )
        });
        let g = host.borrow();
        assert!(!g.damage.iter().any(|dm| dm.1 == 1 || dm.1 == 2), "parried, and never a tower: {:?}", g.damage);
        assert!(g.ccs[1].iter().all(|c| c.0.kind != CcKindV1::ForceMove.code()), "a parried kick sends nobody flying");
        assert!(!g.buffs[1].iter().any(|b| b.name().starts_with("kk_bruised")));
        assert!(g.ccs[2].is_empty());
        assert!(g.damage.iter().any(|dm| dm.1 == 3 && dm.3 == AttackTypeV1::Skill.code()));
    }

    #[test]
    fn the_kick_flies_through_the_engine_and_bruised_cancels_regen_and_stacks_damage_taken() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 10_000, y), "foe")]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Knock { target: 1, to: Point::new(x as i64 + 70_000, y as i64), ticks: 30 },
                    Effect::Bruise { target: 1, stacks: 0, ticks: 240 },
                    Effect::Bruise { target: 1, stacks: 3, ticks: 200 },
                ],
            )
        });
        let g = host.borrow();
        assert!(g.ccs[1].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code() && c.0.name() == crate::motion::KNOCK));
        assert!(g.moves.is_empty(), "no hand-set position");
        let b: Vec<_> = g.buffs[1].iter().filter(|b| b.name().starts_with("kk_bruised:")).collect();
        assert_eq!(b.len(), 1, "one Bruised buff, the latest");
        assert_eq!(b[0].name(), "kk_bruised:3");
        assert_eq!(b[0].damaged_amplify as i64, 3 * d::BRUISED_AMP_PER_STACK);
        assert_eq!(b[0].heal_reduce as i64, d::BRUISED_HEAL_REDUCE_PCT);
        assert!(b[0].hp_regen <= 0);
    }

    #[test]
    fn the_touch_passes_walls_hides_him_and_cc_immunity_is_its_own_buff() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker")]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Roof { on: true }, Effect::Hide, Effect::Invulnerable { ticks: 60 }, Effect::PassWalls { ticks: 20 }, Effect::CcImmune { ticks: 40 }]));
        {
            let g = host.borrow();
            let roof = g.buffs[0].iter().find(|b| b.name() == crate::kicker::ROOF).expect("roof");
            assert!(roof.ignore_wall && roof.damaged_reduce as i64 == d::ROOF_REDUCE_PCT);
            assert!(g.invisible.iter().any(|i| i.0 == 0));
            let inv = g.buffs[0].iter().find(|b| b.name() == "kk_invuln").unwrap();
            assert!(inv.damaged_reduce == 100 && !inv.cc_immune);
            assert!(g.buffs[0].iter().any(|b| b.name() == "kk_pass" && b.ignore_wall));
            let cc = g.buffs[0].iter().find(|b| b.name() == crate::kicker::UNSTOPPABLE).expect("CC immune");
            assert!(cc.cc_immune && cc.duration_tick == 40);
        }
        // a shorter window never cuts a longer one short; Bounced carries the reduction and the rock range
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Roof { on: false }, Effect::CcImmune { ticks: 10 }, Effect::Bounced { ticks: 180 }]));
        let g = host.borrow();
        assert!(!g.buffs[0].iter().any(|b| b.name() == crate::kicker::ROOF));
        assert_eq!(g.buffs[0].iter().find(|b| b.name() == crate::kicker::UNSTOPPABLE).unwrap().duration_tick, 40);
        let b = g.buffs[0].iter().find(|b| b.name() == crate::kicker::BOUNCED).expect("Bounced");
        assert!(b.damaged_reduce as i64 == d::ROOF_REDUCE_PCT && b.range as i64 == d::ROOF_BASIC_RANGE - d::BASIC_RANGE);
    }

    #[test]
    fn second_wind_is_undying_then_a_teleport_home_with_the_hp_set() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker")]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Undying { ticks: 482 }]));
        assert!(host.borrow().buffs[0].iter().any(|b| b.name() == "kk_second_wind" && b.undying));
        let tag = Point::new(x as i64 - 30_000, y as i64);
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::UndyingEnd, Effect::Return { to: tag, hp: 1700 }]));
        let g = host.borrow();
        assert!(!g.buffs[0].iter().any(|b| b.name() == "kk_second_wind"));
        assert_eq!(g.moves, vec![(0, (tag.x as u64, tag.y as u64))]);
        assert_eq!(g.units[0].hp, 1700);
    }

    #[test]
    fn the_skyfall_carry_passes_walls_and_glides() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 10_000, y), "foe")]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Carry { target: 1, to: Point::new(x as i64 + 80_000, y as i64), ticks: 30 }]));
        let g = host.borrow();
        assert!(g.buffs[1].iter().any(|b| b.name() == "kk_carry" && b.ignore_wall));
        assert!(g.ccs[1].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code()));
    }

    #[test]
    fn basic_attacks_carry_the_drop_in_bonus_once_and_feed_the_core() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 10_000, y), "foe")]);
        let mut k = Kicker::default();
        host.borrow_mut().tick = 10;
        k.core.bonus_until = 100;
        let mut dmg = 100;
        with_sim(&host, |sim| k.on_attack(sim, 0, 0, 1, &mut dmg));
        assert_eq!(dmg as i64, 100 + d::DROP_IN_BONUS_PCT);
        let mut again = 100;
        with_sim(&host, |sim| k.on_attack(sim, 0, 0, 1, &mut again));
        assert_eq!(again, 100, "only the first hit after landing");
        // a tick of the passive runs the core with his hit: the target is marked
        with_sim(&host, |sim| k.on_update(sim, 0, 0, 0));
        assert!(k.core.marked(1, 11));
        assert!(host.borrow().buffs[1].iter().any(|b| b.name() == "kk_mark"));
    }

    #[test]
    fn approved_effects_preserve_all_mechanical_commands_without_animation_cc() {
        let (x,y)=open_spot(80000);
        let run=|enabled:bool|{
            let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_kicker"),champ(1,(x+10000,y),"foe")]);
            let mut k=Kicker::default();if enabled{k.configure("{\"art_fx_v1\":1}");}
            host.borrow_mut().tick=50;
            with_sim(&host,|sim|k.apply(sim,0,vec![Effect::Pose{name:"throw",ticks:12},Effect::Damage{target:1,amount:200,direct:false},Effect::Shield{amount:100,ticks:180},Effect::KnockUp{target:1,ticks:30},Effect::Fx{name:"kk_drop_in".into(),at:Point::new(x as i64,y as i64),ticks:24,ambient:false}]));
            let g=host.borrow();assert!(g.ccs[0].is_empty());
            (g.damage.clone(),g.moves.clone(),g.heals.clone(),g.shields.clone(),g.ccs.iter().map(|cs|cs.iter().map(|(c,t)|(c.kind,*t)).collect::<Vec<_>>()).collect::<Vec<_>>())
        };
        assert_eq!(run(false),run(true));
    }

    #[test]
    fn travelling_effects_and_anchor_use_short_bounded_leases_and_stop_after_cleanup() {
        use crate::kicker_core::{Rock,SecondWind};
        let (x,y)=open_spot(80000);let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_kicker")]);
        host.borrow_mut().record_view_effects=true;
        let mut k=Kicker::default();k.configure("{\"art_fx_v1\":1}");
        let p=Point::new(x as i64,y as i64);
        for i in 0..3{k.core.rocks.push(Rock{from:p,end:Point::new(p.x+60000,p.y),pos:Point::new(p.x+i*2000,p.y),attack:150,pct:100,hit:vec![]});}
        k.core.ult=Some(SecondWind{anchor:p,hp:2000,until:500,roof:false});
        for tick in 50..90{
            host.borrow_mut().tick=tick;
            let f=with_sim(&host,|sim|k.frame(sim,0));with_sim(&host,|sim|k.present(sim,0,&f));
        }
        let count=host.borrow().view_effects.len();assert_eq!(count,40,"one lease per tick");
        assert!(host.borrow().view_effects.iter().all(|e|e.3<=crate::kicker_art_data::MAX_RENEW_TICKS));
        assert!(host.borrow().view_effects.iter().any(|e|e.0=="kk_rock_e"));
        assert!(host.borrow().view_effects.iter().any(|e|e.0=="kk_tag"));
        k.core.rocks.clear();k.core.ult=None;
        host.borrow_mut().tick=90;let f=with_sim(&host,|sim|k.frame(sim,0));with_sim(&host,|sim|k.present(sim,0,&f));
        assert_eq!(host.borrow().view_effects.len(),count);
        host.borrow_mut().units[0].hp=0;
        let f=with_sim(&host,|sim|k.frame(sim,0));with_sim(&host,|sim|k.present(sim,0,&f));
        assert_eq!(host.borrow().view_effects.len(),count);
    }

    #[test]
    fn every_packaged_effect_binding_and_renewed_single_frame_resolves() {
        let data:serde_json::Value=serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_kicker.data_champion")).unwrap();
        let art:serde_json::Value=serde_json::from_str(include_str!("../../../mods/tfm2_custom/vfx/kicker#anim.fanim")).unwrap();
        for binding in data["view_effects"].as_array().unwrap().iter().chain(data["view_buffs"].as_array().unwrap()){
            let tag=binding["tag"].as_str().unwrap();assert!(art["anims"][tag].is_object(),"{tag}");
        }
        for tag in ["tag","rock_e","skyfall_se","kick_trail_w"]{assert_eq!(art["anims"][tag]["frames"].as_array().unwrap().len(),1,"{tag}");}
        assert_eq!(data["passive"]["params"],serde_json::json!({"art_fx_v1":1}));
    }

    #[test]
    fn a_basic_while_bounced_throws_a_rock_and_still_feeds_the_core() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 40_000, y), "foe")]);
        host.borrow_mut().record_view_effects = true;
        let mut k = Kicker::default();
        k.configure("{\"art_fx_v1\":1}");
        k.core.bounced_until = 100;
        host.borrow_mut().tick = 20;
        let mut dmg = 100;
        with_sim(&host, |sim| k.on_attack(sim, 0, 0, 1, &mut dmg));
        let g = host.borrow();
        assert!(g.view_effects.iter().any(|e| e.0 == "kk_rock_hit"));
        assert!(g.view_effects.iter().any(|e| e.0 == "kk_rock_e"));
        drop(g);
        // on the ground: the kick, no rock
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 10_000, y), "foe")]);
        host.borrow_mut().record_view_effects = true;
        let mut k = Kicker::default();
        with_sim(&host, |sim| k.on_attack(sim, 0, 0, 1, &mut dmg));
        assert!(host.borrow().view_effects.is_empty());
    }

    #[test]
    fn an_enemy_projectile_closing_in_is_counted_for_the_death_cancel() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker"), champ(1, (x + 60_000, y), "foe")]);
        let mut k = Kicker::default();
        // tick 1: an enemy shot 28,000 east, one of his team's, and an enemy shot moving away
        host.borrow_mut().shots = vec![(1, 1, x + 28_000, y), (5, 0, x + 20_000, y), (2, 1, x + 10_000, y)];
        host.borrow_mut().tick = 1;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        assert_eq!(f.shots_near, 0, "nothing to compare with yet");
        host.borrow_mut().shots = vec![(1, 1, x + 25_000, y), (5, 0, x + 17_000, y), (2, 1, x + 14_000, y)];
        host.borrow_mut().tick = 2;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        assert_eq!(f.shots_near, 1, "only the enemy shot closing in");
    }

    #[test]
    fn the_nexus_reads_as_a_structure() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kicker"),
            HostUnit { kind: K::Nexus, team: 1, pos: (x + 20_000, y), hp: 5000, name: "nexus", targetable: true },
        ]);
        let mut k = Kicker::default();
        host.borrow_mut().tick = 30;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        assert!(f.unit(1).is_some_and(|u| u.tower), "the nexus is a structure");
    }

    #[test]
    fn steering_is_a_short_buff_the_input_ai_reads() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kicker")]);
        let mut k = Kicker::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Steer { to: Point::new(123_000, 456_000) }, Effect::Steer { to: Point::new(124_000, 457_000) }]));
        let g = host.borrow();
        let go: Vec<_> = g.buffs[0].iter().filter_map(|b| crate::kicker::steer_of(b.name())).collect();
        assert_eq!(go, vec![(124_000, 457_000)]);
    }
}
