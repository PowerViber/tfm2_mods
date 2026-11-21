//! Contender 4.0 core and native adapter regression tests. Every number comes from the rules (`d::`).
use crate::contender_shadow_core::*;
use crate::contender_shadow_data as d;

const ME: usize = 1;
const FOE: usize = 10;

fn champ(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, visible: true, targetable: true, farmable: false }
}
fn creep(id: usize, x: i64, y: i64, hp: i64) -> Unit {
    Unit { champion: false, farmable: true, hp, max_hp: hp, team: 9, ..champ(id, 9, x, y) }
}
fn tower(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, tower: true, hp: 5000, max_hp: 5000, ..champ(id, 2, x, y) }
}
fn share(amount: i64, pct: i64) -> i64 {
    amount * pct / 100
}

/// A tiny world: him and the units. Damage lowers HP; drags, glides and blinks move units.
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
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
        }
    }
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        self.f.order = order;
        let out = self.core.step(&self.f);
        self.f.landed_hits = 0;
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
                            self.f.landed_hits += 1;
                        }
                    }
                }
                Effect::Drag { target, to } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::Glide { to, .. } | Effect::Blink { to } => self.f.pos = *to,
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
    fn knocked(&self, id: usize, ticks: i64) -> bool {
        self.seen(|e| matches!(e, Effect::Airborne { target, ticks: t } if *target == id && *t == ticks as u64))
    }
    fn blinks(&self) -> Vec<Point> {
        self.log.iter().filter_map(|(_, e)| if let Effect::Blink { to } = e { Some(*to) } else { None }).collect()
    }
    /// keep Smother quiet in tests that aren't about it
    fn no_smother(mut self) -> Self {
        self.core.smother_at = u64::MAX / 2;
        self
    }
}

// ---------------------------------------------------------------------------------------- the shadow pool

#[test]
fn a_champion_hit_spends_a_shadow_to_dodge_and_the_shadow_counterpunches() {
    let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]).no_smother();
    w.f.incoming.push(IncomingHit { attacker: FOE, from: Point::new(430_000, 500_000), counter: false });
    let out = w.tick(None);
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1);
    let counter = d::COUNTER_BASE + d::COUNTER_AD_RATIO * w.f.attack / 100;
    assert!(out.contains(&Effect::Damage { target: FOE, amount: counter, source: HitSource::Counter }));
    assert!(out.iter().any(|e| matches!(e, Effect::Glide { .. })), "he slips aside");
    // a second hit inside the dodge gap lands
    w.f.incoming.push(IncomingHit { attacker: FOE, from: Point::new(430_000, 500_000), counter: false });
    w.tick(None);
    assert_eq!(w.core.charges, d::CHARGES_MAX - 1);
    // one shadow comes back after the (buffed) recharge
    w.run(d::CHARGES_RECHARGE_TICKS as u64);
    assert_eq!(w.core.charges, d::CHARGES_MAX);
}

#[test]
fn each_landed_hit_refunds_more_the_fewer_shadows_he_has() {
    for left in [d::CHARGES_MAX - 1, 0] {
        let mut w = World::new(vec![champ(FOE, 2, 430_000, 500_000)]).no_smother();
        w.core.charges = left;
        w.core.charge_at = 10_000;
        w.f.landed_hits = 1;
        w.tick(None);
        let refund = ((d::CHARGES_MAX - left) * d::CHARGES_HIT_REFUND_TICKS_PER_MISSING) as u64;
        assert_eq!(w.core.charge_at, 10_000 - refund, "{left} shadows left");
    }
}

#[test]
fn rhythm_caps_at_ten_stacks_and_each_adds_vulnerability() {
    let mut w = World::new(vec![champ(FOE, 2, 410_000, 500_000)]).no_smother();
    for _ in 0..d::RHYTHM_MAX_STACKS + 5 {
        w.f.basic_hits.push(FOE);
        w.tick(None);
    }
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), d::RHYTHM_MAX_STACKS);
    assert_eq!(w.core.vuln_permille(FOE, w.f.tick), d::RHYTHM_MAX_STACKS * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK);
    w.run(d::RHYTHM_STACK_TICKS as u64 + 1);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 0, "stacks fade");
}

// ---------------------------------------------------------------------------------------- S1 Shadow Grab

fn grab_until_airborne(w: &mut World, aim: Point) {
    w.tick(Some(Order::S1 { aim }));
    let mut guard = 0;
    while !w.core.casts.iter().any(|c| matches!(c, Cast::Grab { caster: Caster::Me, grab: Grab::Airborne { .. }, .. })) {
        w.tick(None);
        guard += 1;
        assert!(guard < 300, "the grab never finished: {:?}", w.core.casts);
    }
}

#[test]
fn the_grab_reaches_through_a_wall_pulls_to_the_meeting_point_then_dashes_in_for_a_haymaker() {
    assert_eq!(GRAB_DASH, d::SHADOW_GRAB_RANGE * 3 / 8);
    assert_eq!(d::SHADOW_GRAB_PULL_STOP, GRAB_MEET, "the art's shortest ribbon is the meeting distance");
    let me = Point::new(400_000, 500_000);
    let far = me.x + d::SHADOW_GRAB_RANGE - 10_000;
    let mut w = World::new(vec![champ(FOE, 2, far, 500_000), champ(11, 2, far + 5_000, 500_000)]).no_smother();
    let wall = me.x + GRAB_MEET - 5_000;
    w.f.barriers.push(Barrier { from: Point::new(wall, 450_000), to: Point::new(wall, 550_000), half_width: 1_000 });
    grab_until_airborne(&mut w, Point::new(far, 500_000));
    let hook = 100 * d::SHADOW_GRAB_AD_RATIO / 100 + d::SHADOW_GRAB_BASE;
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: FOE, amount, source: HitSource::Grab } if *amount == hook)));
    assert_eq!(w.count(|e| matches!(e, Effect::Damage { target: FOE, source: HitSource::Grab, .. })), 2, "the hook and the Haymaker");
    assert!(!w.seen(|e| matches!(e, Effect::Damage { target: 11, .. })), "only the first enemy in line");
    assert!(w.seen(|e| matches!(e, Effect::Drag { target: FOE, .. })), "dragged through the wall");
    assert!(w.unit(FOE).pos.x < wall);
    let dash = w.log.iter().find_map(|(_, e)| if let Effect::Glide { to, .. } = e { Some(*to) } else { None }).expect("he dashes");
    assert!(dash.x - me.x <= GRAB_DASH && dash.x > me.x);
    assert!(w.knocked(FOE, d::SHADOW_GRAB_HAYMAKER_KNOCK_TICKS));
    assert_eq!(w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_grab_latch")), 1);
    assert_eq!(w.core.stacks_on(FOE, w.f.tick), 2, "the hook and the Haymaker each build Rhythm");
    w.tick(Some(Order::S1 { aim: Point::new(far, 500_000) }));
    assert_eq!(w.core.rejection, "s1: locked or Shadow Grab cooling down");
}

#[test]
fn a_close_grab_skips_the_pull_and_a_wall_that_stops_the_dash_means_no_haymaker() {
    let near = 400_000 + GRAB_MEET / 2;
    let mut w = World::new(vec![champ(FOE, 2, near, 500_000)]).no_smother();
    grab_until_airborne(&mut w, Point::new(near, 500_000));
    assert!(!w.seen(|e| matches!(e, Effect::Drag { .. })), "nothing to pull");
    let dash = w.log.iter().find_map(|(_, e)| if let Effect::Glide { to, .. } = e { Some(*to) } else { None }).unwrap();
    assert_eq!(dash.x, near - d::SHADOW_GRAB_HAYMAKER_REACH);
    // a wall right in front of him stops the dash short
    let target = 400_000 + GRAB_MEET - 5_000;
    let mut w = World::new(vec![champ(FOE, 2, target, 500_000)]).no_smother();
    w.f.barriers.push(Barrier { from: Point::new(410_000, 450_000), to: Point::new(410_000, 550_000), half_width: 1_000 });
    w.tick(Some(Order::S1 { aim: Point::new(target, 500_000) }));
    w.run((d::SHADOW_GRAB_FLIGHT_TICKS + d::SHADOW_GRAB_DASH_TICKS) as u64 + 5);
    assert_eq!(w.count(|e| matches!(e, Effect::Damage { source: HitSource::Grab, .. })), 1, "the hook lands, the Haymaker doesn't");
    assert!(!w.seen(|e| matches!(e, Effect::Airborne { .. })));
}

// ---------------------------------------------------------------------------------------- Shadowborne

#[test]
fn a_missed_grab_flurries_where_it_lands_then_knocks_up_around_the_last_enemy_and_he_teleports_behind_it() {
    // Reihan: "any flurry hit at the last enemy, he can do an area knock up and tp" (60k, 1 s)
    let landing = 400_000 + d::SHADOW_GRAB_RANGE;
    let beside = landing + d::SHADOW_GRAB_WIDTH + 10_000;
    let near = Point::new(beside, 500_000 + d::SHADOWBORNE_KNOCK_RADIUS - 5_000);
    let far = Point::new(beside, 500_000 + d::SHADOWBORNE_KNOCK_RADIUS + 20_000);
    let mut w = World::new(vec![
        champ(FOE, 2, beside, 500_000), champ(11, 2, near.x, near.y), champ(12, 2, far.x, far.y), tower(20, beside + 20_000, 500_000),
    ]).no_smother();
    let charges = w.core.charges;
    w.tick(Some(Order::S1 { aim: Point::new(beside, 500_000) }));
    w.run((d::SHADOW_GRAB_FLIGHT_TICKS + d::SHADOWBORNE_TICKS) as u64 + 5);
    assert!(w.core.casts.is_empty());
    let punches = w.count(|e| matches!(e, Effect::Damage { source: HitSource::ShadowFlurry, .. }));
    assert!(punches >= 2 && punches <= d::SHADOWBORNE_PUNCHES as usize, "{punches} punches");
    let punch = d::SHADOWBORNE_BASE + d::SHADOWBORNE_AD_RATIO * w.f.attack / 100;
    assert!(w.seen(|e| matches!(e, Effect::Damage { amount, source: HitSource::ShadowFlurry, .. } if *amount >= punch)));
    // the finisher: everyone within the radius of the last enemy punched, not beyond, never a tower
    let last = w.log.iter().rev().find_map(|(_, e)| if let Effect::Damage { target, source: HitSource::ShadowFlurry, .. } = e { Some(*target) } else { None }).unwrap();
    let centre = w.unit(last).pos;
    for id in [FOE, 11, 12] {
        let inside = w.unit(id).pos.within(centre, d::SHADOWBORNE_KNOCK_RADIUS);
        assert_eq!(w.knocked(id, d::SHADOWBORNE_KNOCK_TICKS), inside, "#{id}");
    }
    assert!(!w.seen(|e| matches!(e, Effect::Airborne { target: 20, .. })), "never a tower");
    // his own flurry: he teleports behind that enemy (the far side from where he stood)
    let blinks = w.blinks();
    assert_eq!(blinks.len(), 1);
    assert!(blinks[0].x > centre.x, "behind the target");
    assert_eq!(w.core.charges, charges, "Shadowborne costs nothing");
}

#[test]
fn a_flurry_with_nobody_near_does_nothing() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 800_000)]).no_smother();
    w.tick(Some(Order::S1 { aim: Point::new(600_000, 500_000) }));
    w.run(d::SHADOW_GRAB_FLIGHT_TICKS as u64 + 3);
    assert!(w.core.casts.is_empty() && !w.seen(|e| matches!(e, Effect::Damage { .. } | Effect::Blink { .. })));
    assert!(w.core.grab_at > w.f.tick, "the grab still cools down");
}

// ---------------------------------------------------------------------------------------- S2 Shadow Anchor

#[test]
fn the_anchor_fears_then_flurries_and_he_teleports_to_it_on_a_second_press() {
    let spot = Point::new(400_000 + d::ANCHOR_RANGE + 50_000, 500_000);
    let inside = Point::new(400_000 + d::ANCHOR_RANGE + d::ANCHOR_FEAR_RADIUS / 2, 500_000);
    let outside = Point::new(400_000 + d::ANCHOR_RANGE, 500_000 + d::ANCHOR_FEAR_RADIUS + 20_000);
    let mut w = World::new(vec![champ(FOE, 2, inside.x, inside.y), champ(11, 2, outside.x, outside.y)]).no_smother();
    w.tick(Some(Order::S2 { aim: spot }));
    let anchor = *w.core.anchor().expect("the anchor drops");
    assert_eq!(anchor.pos, Point::new(400_000 + d::ANCHOR_RANGE, 500_000), "at most its range away");
    assert!(w.seen(|e| matches!(e, Effect::Fear { target: FOE, ticks } if *ticks == d::ANCHOR_FEAR_TICKS as u64)));
    assert!(!w.seen(|e| matches!(e, Effect::Fear { target: 11, .. })), "only within the fear radius");
    // nothing hits during the fear, then the shadow flurries (a shadow's finisher never moves him)
    w.run(d::ANCHOR_FEAR_TICKS as u64 - 1);
    assert!(!w.seen(|e| matches!(e, Effect::Damage { .. })));
    w.run(d::SHADOWBORNE_TICKS as u64 + 3);
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: FOE, source: HitSource::ShadowFlurry, .. })));
    assert!(w.knocked(FOE, d::SHADOWBORNE_KNOCK_TICKS));
    assert!(w.blinks().is_empty(), "the shadow's flurry knocks up but never teleports him");
    // the second press: he teleports to the anchor, and the cooldown starts
    w.tick(Some(Order::S2 { aim: spot }));
    assert_eq!(w.blinks(), vec![anchor.pos]);
    assert!(w.core.anchor().is_none());
    assert_eq!(w.core.anchor_at, w.f.tick - 1 + d::ANCHOR_COOLDOWN_TICKS as u64);
    w.tick(Some(Order::S2 { aim: spot }));
    assert_eq!(w.core.rejection, "s2: Shadow Anchor cooling down");
}

#[test]
fn dropping_low_teleports_him_to_the_anchor_and_an_unused_anchor_expires_into_cooldown() {
    let mut w = World::new(vec![]).no_smother();
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    let at = w.core.anchor().unwrap().pos;
    w.f.hp = w.f.max_hp * d::ANCHOR_LOW_HP_PCT / 100;
    w.tick(None);
    assert_eq!(w.blinks(), vec![at], "auto-teleport at low HP");
    let mut w = World::new(vec![]).no_smother();
    w.tick(Some(Order::S2 { aim: Point::new(450_000, 500_000) }));
    w.run(d::ANCHOR_LIFE_TICKS as u64 + 1);
    assert!(w.core.anchor().is_none() && w.blinks().is_empty());
    assert!(w.core.anchor_at > w.f.tick, "the cooldown starts when it expires");
}

// ---------------------------------------------------------------------------------------- S3 Smother

#[test]
fn smother_casts_itself_silences_slows_and_shreds_then_hastes_and_shields_him_and_flurries() {
    let ids = [FOE, 11, 12, 13];
    let units = ids.iter().enumerate().map(|(i, &id)| champ(id, 2, 420_000 + i as i64 * 10_000, 500_000)).collect();
    let mut w = World::new(units);
    w.f.level = crate::unlocks::S3_LEVEL;
    w.tick(None);
    assert!(w.core.smother_at > w.f.tick, "self-cast when a champion is in reach");
    w.run(d::SMOTHER_FLIGHT_TICKS as u64 + 1);
    for id in ids {
        assert!(w.seen(|e| matches!(e, Effect::Silence { target, ticks } if *target == id && *ticks == d::SMOTHER_SILENCE_TICKS as u64)), "#{id}");
        assert!(w.seen(|e| matches!(e, Effect::Slow { target, pct, .. } if *target == id && *pct == d::SMOTHER_SLOW_PCT)));
        assert!(w.seen(|e| matches!(e, Effect::Smothered { target, armour_pct, attack_speed_pct, .. }
            if *target == id && *armour_pct == d::SMOTHER_ARMOUR_SHRED_PCT && *attack_speed_pct == d::SMOTHER_ATTACK_SPEED_SHRED_PCT)));
    }
    assert!(w.seen(|e| matches!(e, Effect::Haste { attack_speed_pct, move_pct, .. }
        if *attack_speed_pct == d::SMOTHER_HASTE_ATTACK_SPEED_PCT && *move_pct == d::SMOTHER_HASTE_MOVE_SPEED_PCT)));
    let shield: i64 = w.log.iter().filter_map(|(_, e)| if let Effect::Shield { amount, .. } = e { Some(*amount) } else { None }).sum();
    assert_eq!(shield, w.f.max_hp * d::SMOTHER_SHIELD_CAP_PERMILLE / 1_000, "8% per champion, capped at 24%");
    // then his flurry, ending with his teleport
    w.run(d::SHADOWBORNE_TICKS as u64 + 3);
    assert!(w.seen(|e| matches!(e, Effect::Damage { source: HitSource::ShadowFlurry, .. })));
    assert_eq!(w.blinks().len(), 1);
    // not before level 3, and not with nobody in reach
    let mut w = World::new(vec![champ(FOE, 2, 420_000, 500_000)]);
    w.f.level = crate::unlocks::S3_LEVEL - 1;
    w.run(5);
    assert!(!w.seen(|e| matches!(e, Effect::Haste { .. })));
    let mut w = World::new(vec![champ(FOE, 2, 400_000 + d::SMOTHER_RANGE + 20_000, 500_000)]);
    w.run(5);
    assert!(!w.seen(|e| matches!(e, Effect::Haste { .. })));
}

// ---------------------------------------------------------------------------------------- ult Clone Army and mimicry

#[test]
fn clone_army_fans_three_clones_in_front_and_three_around_the_anchor_and_refills_his_shadows() {
    let mut w = World::new(vec![champ(FOE, 2, 600_000, 500_000)]).no_smother();
    w.core.charges = 0;
    w.core.charge_at = 100_000;
    w.tick(Some(Order::S2 { aim: Point::new(400_000, 400_000) }));
    let anchor = w.core.anchor().unwrap().pos;
    w.tick(Some(Order::Ult { aim: Point::new(600_000, 500_000) }));
    assert_eq!(w.core.charges, d::CHARGES_MAX, "the ult refills his shadows");
    let clones: Vec<Point> = w.core.shadows.iter().filter(|s| !s.anchor).map(|s| s.pos).collect();
    assert_eq!(clones.len(), 6);
    let front = Point::new(400_000 + d::CLONE_ARMY_AHEAD, 500_000);
    for p in [front, Point::new(front.x, front.y + d::CLONE_ARMY_SIDE), Point::new(front.x, front.y - d::CLONE_ARMY_SIDE)] {
        assert!(clones.contains(&p), "{p:?} in the kunai fan");
    }
    assert!(clones.iter().filter(|p| p.within(anchor, d::CLONE_ARMY_AHEAD + d::CLONE_ARMY_SIDE)).count() >= 3, "three around the anchor");
    w.run(d::CLONE_ARMY_LIFE_TICKS as u64);
    assert!(w.core.shadows.iter().all(|s| s.anchor), "the clones fade; the anchor stays until it expires or he uses it");
}

#[test]
fn every_shadow_mimics_his_grab_and_smother_at_half_damage_and_never_teleports_him() {
    // a clone stands right of him; an enemy sits only in the clone's line
    let mut w = World::new(vec![champ(FOE, 2, 400_000 + d::CLONE_ARMY_AHEAD + 60_000, 500_000)]).no_smother();
    w.tick(Some(Order::Ult { aim: Point::new(500_000, 500_000) }));
    // aim the grab north: only the fan's clones, not him, have the enemy ahead
    let target = w.unit(FOE).pos;
    w.f.pos = Point::new(target.x - d::CLONE_ARMY_AHEAD, target.y + 60_000);
    let casters = 1 + w.core.shadows.len();
    w.tick(Some(Order::S1 { aim: Point::new(w.f.pos.x, w.f.pos.y + 10_000) }));
    assert_eq!(w.core.casts.iter().filter(|c| matches!(c, Cast::Grab { .. })).count(), casters, "every shadow mimics the grab");
    // the clone straight above the enemy grabs it; the copy deals the mimic share
    let mut w = World::new(vec![champ(FOE, 2, 400_000 + d::CLONE_ARMY_AHEAD, 560_000)]).no_smother();
    w.tick(Some(Order::Ult { aim: Point::new(500_000, 500_000) }));
    w.tick(Some(Order::S1 { aim: Point::new(400_000, 600_000) }));
    w.run(d::SHADOW_GRAB_FLIGHT_TICKS as u64 + d::SHADOW_GRAB_DASH_TICKS as u64 + d::SHADOWBORNE_TICKS as u64 + 10);
    let hook = d::SHADOW_GRAB_BASE + d::SHADOW_GRAB_AD_RATIO * w.f.attack / 100;
    assert!(w.seen(|e| matches!(e, Effect::Damage { target: FOE, amount, source: HitSource::Grab } if *amount == share(hook, d::CLONE_ARMY_MIMIC_DAMAGE_PCT))),
        "{:?}", w.log.iter().filter(|(_, e)| matches!(e, Effect::Damage { .. })).collect::<Vec<_>>());
    assert_eq!(w.count(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_grab_latch")), 1, "one grab per enemy");
    assert!(w.knocked(FOE, d::SHADOW_GRAB_HAYMAKER_KNOCK_TICKS), "the clone's Haymaker knocks up in full");
    assert!(!w.seen(|e| matches!(e, Effect::Glide { .. })), "he never dashed: the clone did");
    // the misses' flurries are the clones' own: they never teleport him
    assert!(w.blinks().is_empty(), "only shadows' flurries ran: no teleport");
}

#[test]
fn the_lab_autoplay_grabs_the_nearest_champion() {
    let mut w = World::new(vec![champ(FOE, 2, 450_000, 500_000)]).no_smother();
    w.f.manual = false;
    w.tick(None);
    assert!(w.core.casts.iter().any(|c| matches!(c, Cast::Grab { caster: Caster::Me, .. })), "{}", w.core.reason);
}

#[test]
fn the_core_never_spends_on_creeps_or_towers_outside_its_rules() {
    let mut w = World::new(vec![creep(30, 410_000, 500_000, 500), tower(20, 420_000, 500_000)]);
    w.run(30);
    assert!(!w.seen(|e| matches!(e, Effect::Haste { .. })), "Smother only self-casts at champions");
    assert!(!w.seen(|e| matches!(e, Effect::Damage { target: 20, .. })));
}

// ---------------------------------------------------------------------------------------- data and art

#[test]
fn his_three_engine_slots_are_the_grab_the_anchor_and_clone_army() {
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    for (slot, cooltime, range, effect) in [
        ("skill", d::SHADOW_GRAB_COOLDOWN_TICKS, d::SHADOW_GRAB_RANGE, "contender_s1"),
        ("skill2", d::ANCHOR_SLOT_TICKS, d::ANCHOR_RANGE, "contender_s2"),
        ("ult", d::CLONE_ARMY_COOLDOWN_TICKS, d::CLONE_ARMY_RANGE, "contender_ult"),
    ] {
        let s = &data[slot];
        assert_eq!(s["cooltime"].as_i64(), Some(cooltime), "{slot}");
        assert_eq!(s["range"].as_i64(), Some(range), "{slot}");
        assert_eq!(s["effect"]["effect_ref"], format!("tfm2_custom_ai:{effect}"), "{slot}");
    }
}

#[test]
fn every_shadow_grab_art_name_is_bound_and_the_ribbon_follows_the_target() {
    use crate::contender::{grab_heading, grab_ribbon};
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    let bound: std::collections::HashSet<String> = data["view_effects"].as_array().unwrap().iter()
        .map(|e| e["name"].as_str().unwrap().to_string()).collect();
    let fanim: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/vfx/contender_grab#anim.fanim")).unwrap();
    let seconds = |tag: &str| fanim["anims"][tag]["frames"].as_array().unwrap().iter().map(|f| f["duration"].as_f64().unwrap()).sum::<f64>();
    assert!((seconds("latch") * 60.0 - GRAB_LATCH_TICKS as f64).abs() < 0.5);
    assert!((seconds("burst") * 60.0 - GRAB_BURST_TICKS as f64).abs() < 0.5);
    for name in ["ct_grab_latch", "ct_grab_burst", "ct_body_idle0", "ct_flurry"] { assert!(bound.contains(name), "{name}"); }
    let me = Point::new(400_000, 400_000);
    for h in 0..8u8 { assert!(bound.contains(&format!("ct_grab_hand_d{h}")), "hand {h}"); }
    for step in 0..32 {
        let angle = step as f64 * std::f64::consts::TAU / 32.0;
        for gap in (0..=d::SHADOW_GRAB_RANGE + 20_000).step_by(2_500) {
            let them = Point::new(me.x + (angle.cos() * gap as f64) as i64, me.y + (angle.sin() * gap as f64) as i64);
            let (name, _) = grab_ribbon(me, them);
            assert!(bound.contains(&name), "{name} for gap {gap}");
        }
    }
    assert_eq!([(1, 0), (0, 1), (-1, 0), (0, -1)].map(|(x, y)| grab_heading(me, Point::new(me.x + x * 10_000, me.y + y * 10_000))), [0, 2, 4, 6]);
}

#[test]
fn every_smother_and_clone_art_name_is_bound_and_the_fades_match_the_core() {
    // round 192: Codex's Smother trail and contact and the Clone Army clone, baked by tools/build-contender-40-art.py
    use crate::contender::smother_trail;
    let data: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_contender.data_champion")).unwrap();
    let bound: std::collections::HashSet<String> = data["view_effects"].as_array().unwrap().iter()
        .map(|e| e["name"].as_str().unwrap().to_string()).collect();
    let fanim: serde_json::Value = serde_json::from_str(include_str!("../../../mods/tfm2_custom/vfx/contender_40#anim.fanim")).unwrap();
    let ticks = |tag: &str| fanim["anims"][tag]["frames"].as_array().unwrap().iter().map(|f| f["duration"].as_f64().unwrap()).sum::<f64>() * 60.0;
    assert!((ticks("contact") - SMOTHER_CONTACT_TICKS as f64).abs() < 0.5, "the contact fade lasts the core's cue");
    assert!((ticks("clone_in") - CLONE_IN_TICKS as f64).abs() < 0.5, "the clone fade-in lasts CLONE_IN_TICKS");
    for name in ["ct_40_contact", "ct_40_clone", "ct_40_clone_l", "ct_40_clone_in", "ct_40_clone_in_l"] { assert!(bound.contains(name), "{name}"); }
    let from = Point::new(400_000, 400_000);
    assert!(smother_trail(from, from, false).is_none(), "nothing before the line moves");
    for step in 0..32 {
        let angle = step as f64 * std::f64::consts::TAU / 32.0;
        for gap in (6_000..=d::SMOTHER_RANGE + 10_000).step_by(2_500) {
            let front = Point::new(from.x + (angle.cos() * gap as f64) as i64, from.y + (angle.sin() * gap as f64) as i64);
            for shadow in [false, true] {
                let (name, mid) = smother_trail(from, front, shadow).unwrap();
                assert!(bound.contains(&name), "{name}");
                assert_eq!(name.contains("_s_"), shadow);
                assert!(mid.within(from, gap) && mid.within(front, gap), "drawn between the origin and the front");
            }
        }
    }
}

// ---------------------------------------------------------------------------------------- the game bridge

mod glue {
    use crate::contender::Contender;
    use crate::contender_shadow_core::{Cast, Caster, Effect, Order, Point};
    use crate::contender_shadow_data as d;
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use mod_api_stable::{AttackTypeV1, CcKindV1, EntityKindV1 as K, InputTargetV1, StableEffectType, StablePassive};
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
    fn cast_markers_carry_the_engines_aim_for_all_three_slots() {
        assert_eq!(crate::contender::cast_order("ct_cast:s1:410000:520000:77"), Some(Order::S1 { aim: Point::new(410_000, 520_000) }));
        assert_eq!(crate::contender::cast_order("ct_cast:s2:1:2:5"), Some(Order::S2 { aim: Point::new(1, 2) }));
        assert_eq!(crate::contender::cast_order("ct_cast:ult:3:4:5"), Some(Order::Ult { aim: Point::new(3, 4) }));
        assert_eq!(crate::contender::cast_order("ct_go:1:2"), None);
    }

    #[test]
    fn an_engine_s1_press_runs_the_grab_and_s2_drops_then_teleports() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 60_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 1);
        host.borrow_mut().tick = 101;
        with_sim(&host, |sim| crate::contender::CastMerge.apply(sim, 0, 0, InputTargetV1::target(1)));
        assert!(host.borrow().buffs[0].iter().any(|b| b.name().starts_with("ct_cast:s1:")));
        run(&host, &mut c, 101, 1);
        assert!(c.core.casts.iter().any(|k| matches!(k, Cast::Grab { caster: Caster::Me, .. })), "{}", c.core.rejection);
        assert!(!host.borrow().buffs[0].iter().any(|b| b.name().starts_with("ct_cast:")), "the marker is consumed");
        // S2: drop, then a second press teleports him there (the host places him)
        host.borrow_mut().tick = 102;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::pos(x + 50_000, y)));
        run(&host, &mut c, 102, 1);
        let anchor = c.core.anchor().expect("dropped").pos;
        host.borrow_mut().tick = 110;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::pos(x + 50_000, y)));
        run(&host, &mut c, 110, 1);
        assert!(c.core.anchor().is_none());
        assert_eq!(host.borrow().units[0].pos, (anchor.x as u64, anchor.y as u64), "teleported to the anchor");
    }

    #[test]
    fn smother_on_the_host_silences_shreds_hastes_and_shields() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        // the mock host is level 9: Smother is unlocked and casts itself at the champion in reach
        run(&host, &mut c, 100, d::SMOTHER_FLIGHT_TICKS as usize + 2);
        let g = host.borrow();
        assert!(g.ccs[1].iter().any(|cc| cc.0.kind == CcKindV1::BlockSkill.code()), "silenced");
        let smothered = g.buffs[1].iter().find(|b| b.name() == "ct_smothered").expect("smothered");
        assert_eq!((smothered.defence_mult, smothered.attack_speed_mult), (-(d::SMOTHER_ARMOUR_SHRED_PCT as i32), -(d::SMOTHER_ATTACK_SPEED_SHRED_PCT as i32)));
        assert!(g.buffs[1].iter().any(|b| b.name() == "ct_smother_slow" && b.move_speed_mult == -(d::SMOTHER_SLOW_PCT as i32)));
        let haste = g.buffs[0].iter().find(|b| b.name() == "ct_haste").expect("haste");
        assert_eq!((haste.attack_speed_mult, haste.move_speed_mult), (d::SMOTHER_HASTE_ATTACK_SPEED_PCT as i32, d::SMOTHER_HASTE_MOVE_SPEED_PCT as i32));
        assert_eq!(g.shields.iter().map(|s| s.1 as i64).sum::<i64>(), 2_000 * d::SMOTHER_SHIELD_PERMILLE_PER_HIT / 1_000, "one champion hit");
    }

    #[test]
    fn a_champion_hit_is_dodged_healed_back_but_creeps_never_trigger_it() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let creep = HostUnit { kind: K::Minion, team: 1, pos: (x + 10_000, y), hp: 500, name: "minion", targetable: true };
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 20_000, y), "fighter"), creep]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 2);
        let before = c.core.charges;
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 2, 80));
        assert!(host.borrow().heals.is_empty(), "a creep's hit is never dodged");
        with_sim(&host, |sim| c.on_damaged(sim, 0, 0, 1, 120));
        assert!(host.borrow().heals.iter().any(|h| h.1 == 0 && h.2 == 120), "the champion's hit is undone");
        assert_eq!(c.core.charges, before - 1);
    }

    #[test]
    fn his_basics_build_rhythm_and_skills_never_hit_towers() {
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
        let effects = vec![Effect::Damage { target: 2, amount: 50, source: crate::contender_shadow_core::HitSource::Counter }, Effect::Airborne { target: 2, ticks: 30 }];
        with_sim(&host, |sim| c.apply(sim, 0, effects));
        let g = host.borrow();
        assert!(!g.damage.iter().any(|d| d.1 == 2 && d.3 == AttackTypeV1::Skill.code()) && g.ccs[2].is_empty(), "never a tower");
    }

    #[test]
    fn an_s2_press_while_the_anchor_cools_down_is_not_wanted() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(120_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 30_000, y), "fighter")]);
        let mut c = Contender::default();
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        run(&host, &mut c, 100, 1);
        assert_ne!(crate::press::get(11, 0, 100).unwrap() & crate::press::S2, 0, "ready: wanted");
        c.core.anchor_at = 10_000;
        run(&host, &mut c, 101, 1);
        assert_eq!(crate::press::get(11, 0, 101).unwrap() & crate::press::S2, 0, "cooling down: not wanted");
    }

    #[test]
    fn a_real_shadow_grab_plays_codexs_hand_latch_ribbon_and_burst_in_order() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(160_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 120_000, y), "fighter")]);
        host.borrow_mut().record_view_effects = true;
        let mut c = Contender::default();
        c.configure(r#"{"shadow_art_v1":1,"shadow_directions_v1":1}"#);
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        c.core.smother_at = u64::MAX / 2;
        run(&host, &mut c, 100, 1);
        host.borrow_mut().tick = 101;
        with_sim(&host, |sim| crate::contender::CastMerge.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 101, 120);
        let names: Vec<String> = host.borrow().view_effects.iter().map(|e| e.0.clone()).filter(|n| n.starts_with("ct_grab_")).collect();
        let first = |p: &str| names.iter().position(|n| n.starts_with(p)).unwrap_or_else(|| panic!("no {p} in {names:?}"));
        let (hand, latch, ribbon, burst) = (first("ct_grab_hand_d0"), first("ct_grab_latch"), first("ct_grab_ribbon_L"), first("ct_grab_burst"));
        assert!(hand < latch && latch < ribbon && ribbon < burst, "{names:?}");
        assert!(names.iter().filter(|n| n.starts_with("ct_grab_ribbon_L")).count() >= 2, "the ribbon is renewed during the pull");
    }

    #[test]
    fn clone_army_and_smother_play_codexs_clone_trail_and_contact_art() {
        let _guard = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(160_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_contender"), champ(1, (x + 40_000, y), "fighter")]);
        host.borrow_mut().record_view_effects = true;
        let mut c = Contender::default();
        c.configure(r#"{"shadow_art_v1":1,"shadow_directions_v1":1}"#);
        with_sim(&host, |sim| c.on_spawn(sim, 0, 0));
        with_sim(&host, |sim| { sim.add_buff(0, &mod_api_stable::BuffV1::named(crate::ULT_LEARNED)); });
        c.core.smother_at = u64::MAX / 2;
        run(&host, &mut c, 100, 1);
        // the anchor, then Clone Army
        host.borrow_mut().tick = 101;
        with_sim(&host, |sim| crate::contender::CastFlurry.apply(sim, 0, 0, InputTargetV1::pos(x, y + 60_000)));
        run(&host, &mut c, 101, 1);
        host.borrow_mut().tick = 104;
        with_sim(&host, |sim| crate::contender::CastLink.apply(sim, 0, 0, InputTargetV1::target(1)));
        run(&host, &mut c, 104, 20);
        // then Smother, which every shadow mimics
        c.core.smother_at = 0;
        run(&host, &mut c, 124, d::SMOTHER_FLIGHT_TICKS as usize + 4);
        let names: Vec<String> = host.borrow().view_effects.iter().map(|e| e.0.clone()).collect();
        let first = |p: &str| names.iter().position(|n| n.starts_with(p)).unwrap_or_else(|| panic!("no {p}"));
        let standing = names.iter().position(|n| n == "ct_40_clone" || n == "ct_40_clone_l").expect("standing clones");
        assert!(first("ct_40_clone_in") < standing, "clones fade in, then stand");
        assert!(names.iter().any(|n| n.starts_with("ct_40_trail_L")), "his own trail");
        assert!(names.iter().any(|n| n.starts_with("ct_40_trail_s_L")), "the shadows' paler trails");
        assert!(names.iter().any(|n| n == "ct_40_contact"), "the contact flash");
        assert!(names.iter().any(|n| n == "ct_body_idle0"), "the anchor keeps the dark shadow body");
    }

    #[test]
    fn the_tower_guard_stops_him_outside_an_uncovered_enemy_tower() {
        use crate::steve::tower_guard;
        let r = d::AI_TOWER_RANGE;
        let (me, tower) = ((300_000, 500_000), (500_000, 500_000));
        let inside = (tower.0 - r / 2, 500_000);
        assert_eq!(tower_guard(me, inside, &[(tower, false)], r), Some((tower.0 - r - 5_000, 500_000)));
        assert_eq!(tower_guard(me, inside, &[(tower, true)], r), None, "his minions are tanking: go");
        assert_eq!(tower_guard(me, (tower.0 - r - 20_000, 500_000), &[(tower, false)], r), None);
    }
}
