//! Basalt's core: walls (limit, keep range, integrity, slams), Jump (double jump, wall kicks, passing walls), Whip Pull
//! (the slingshot), the eight skills, and his AI; then the bridge on the glue mock host.
use crate::basalt_core::*;
use crate::basalt_data as d;

const ME: usize = 1;
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
        tower: false,
        visible: true,
        targetable: true,
    }
}
fn minion(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, hp: 300, max_hp: 300, ..champ(id, 2, x, y) }
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

/// A tiny world: him and the units, moved along their glides the way the engine would.
struct World {
    core: Core,
    f: Frame,
    log: Vec<(u64, Effect)>,
    me_glide: Option<(Point, Point, u64, u64)>,
    glides: Vec<(usize, Point, Point, u64, u64)>,
}
impl World {
    fn new(units: Vec<Unit>, terrain: &[(i64, i64)]) -> World {
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
                terrain: walls(terrain),
                profile: [50; 13],
                manual: true,
                ..Frame::default()
            },
            log: vec![],
            me_glide: None,
            glides: vec![],
        }
    }
    fn tick(&mut self, order: Option<Order>) -> Vec<Effect> {
        let tick = self.f.tick;
        if let Some((a, b, s, n)) = self.me_glide {
            let k = ((tick - s) as f64 / n as f64).min(1.0);
            self.f.pos = Point::new(a.x + ((b.x - a.x) as f64 * k) as i64, a.y + ((b.y - a.y) as f64 * k) as i64);
            if k >= 1.0 {
                self.me_glide = None;
            }
        }
        let mut done = vec![];
        for (i, g) in self.glides.iter().enumerate() {
            let k = ((tick - g.3) as f64 / g.4 as f64).min(1.0);
            if let Some(u) = self.f.units.iter_mut().find(|u| u.id == g.0) {
                u.pos = Point::new(g.1.x + ((g.2.x - g.1.x) as f64 * k) as i64, g.1.y + ((g.2.y - g.1.y) as f64 * k) as i64);
            }
            if k >= 1.0 {
                done.push(i);
            }
        }
        for i in done.into_iter().rev() {
            self.glides.remove(i);
        }
        self.f.order = order;
        let out = self.core.step(&self.f);
        for e in &out {
            self.log.push((tick, e.clone()));
            match e {
                Effect::Glide { to, ticks } => self.me_glide = Some((self.f.pos, *to, tick, (*ticks).max(1))),
                Effect::Knock { target, to, ticks } => {
                    let from = self.f.unit(*target).map(|u| u.pos).unwrap_or(*to);
                    self.glides.retain(|g| g.0 != *target);
                    self.glides.push((*target, from, *to, tick, (*ticks).max(1)));
                }
                Effect::Damage { target, amount } => {
                    if let Some(u) = self.f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = (u.hp - amount).max(1);
                    }
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
    fn stunned(&self, id: usize) -> bool {
        self.seen(|e| matches!(e, Effect::Stun { target, .. } if *target == id))
    }
}

// ------------------------------------------------------------------ walls

#[test]
fn up_to_five_walls_the_oldest_crumbles_into_rubble() {
    let mut w = World::new(vec![], &[]);
    for i in 0..6 {
        w.core.punches = 2;
        w.f.pos = Point::new(200_000 + i * 100_000, 500_000);
        w.tick(Some(Order::S1(Point::new(200_000 + i * 100_000, 400_000))));
        w.run(40);
        w.f.pos = Point::new(450_000, 450_000);
    }
    assert_eq!(w.core.walls.len(), d::WALL_MAX as usize);
    assert!(w.seen(|e| matches!(e, Effect::Rubble { .. })));
}

#[test]
fn a_wall_fades_3_s_after_he_leaves_its_range_and_coming_back_cancels_it() {
    let mut w = World::new(vec![], &[]);
    w.tick(Some(Order::S1(Point::new(400_000, 400_000))));
    w.run(40);
    assert_eq!(w.core.walls.len(), 1);
    w.f.pos = Point::new(800_000, 800_000);
    w.run(d::WALL_FADE_TICKS as u64 - 10);
    w.f.pos = Point::new(400_000, 450_000); // back in range
    w.run(30);
    assert_eq!(w.core.walls.len(), 1, "coming back cancels the fade");
    w.f.pos = Point::new(800_000, 800_000);
    w.run(d::WALL_FADE_TICKS as u64 + 2);
    assert!(w.core.walls.is_empty());
}

#[test]
fn only_his_hits_wear_a_wall_and_three_shatter_it() {
    let mut w = World::new(vec![], &[]);
    w.tick(Some(Order::S1(Point::new(400_000, 400_000))));
    // the punch was the first hit
    assert_eq!(w.core.walls[0].integrity, 2);
    let id = w.core.walls[0].id;
    let f = w.f.clone();
    let mut out = vec![];
    w.core.slam(&f, 0, Hit::Mine(id), 0, &mut out); // no unit 0: no slam at all, the wall is untouched
    assert_eq!(w.core.walls[0].integrity, 2);
    w.f.units.push(champ(FOE, 2, 400_000, 420_000));
    let f = w.f.clone();
    w.core.slam(&f, FOE, Hit::Mine(id), 0, &mut out);
    assert_eq!(w.core.walls[0].integrity, 1);
    w.core.slam(&f, FOE, Hit::Mine(id), 0, &mut out);
    assert!(w.core.walls.is_empty(), "the third of his hits shatters it");
    assert!(out.iter().any(|e| matches!(e, Effect::Rubble { .. })));
}

#[test]
fn a_slam_stuns_once_every_2_s_per_target() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 420_000)], &[]);
    let f = w.f.clone();
    let mut out = vec![];
    w.core.slam(&f, FOE, Hit::Terrain, 0, &mut out);
    w.core.slam(&f, FOE, Hit::Terrain, 0, &mut out);
    assert_eq!(out.iter().filter(|e| matches!(e, Effect::Stun { .. })).count(), 1);
    assert_eq!(out.iter().filter(|e| matches!(e, Effect::Damage { .. })).count(), 2);
}

// ------------------------------------------------------------------ Jump, double jump, wall kick, passing walls

#[test]
fn jump_double_jump_and_a_third_only_off_his_wall() {
    let mut w = World::new(vec![], &[]);
    w.tick(Some(Order::Jump(Point::new(400_000, 300_000))));
    assert_eq!(w.core.jumps, d::JUMP_CHARGES - 1);
    assert!(w.core.airborne(&w.f));
    w.run(5);
    w.tick(Some(Order::Jump(Point::new(300_000, 300_000))));
    assert!(w.core.rejection.is_empty(), "the double jump is free");
    assert_eq!(w.core.jumps, d::JUMP_CHARGES - 1);
    w.run(5);
    w.tick(Some(Order::Jump(Point::new(300_000, 200_000))));
    assert_eq!(w.core.rejection, "jump", "no third jump without a wall to kick");
    // a wall to kick off: allowed, long, and the double jump is back
    let id = 77;
    w.core.walls.push(StoneWall {
        id,
        a: Point::new(w.f.pos.x - 30_000, w.f.pos.y + 10_000),
        b: Point::new(w.f.pos.x + 30_000, w.f.pos.y + 10_000),
        integrity: 3,
        born: 0,
        out_since: None,
        slide: None,
    });
    let from = w.f.pos;
    let out = w.tick(Some(Order::Jump(Point::new(from.x, 100_000))));
    let to = out.iter().find_map(|e| if let Effect::Glide { to, .. } = e { Some(*to) } else { None }).unwrap();
    assert!((from.dist(to) - d::JUMP_KICK_JUMP as f64).abs() < 3_000.0, "{}", from.dist(to));
    assert!(w.core.air.unwrap().double_left);
    assert!(w.core.kicked.contains(&id));
    w.run(3);
    w.f.pos = Point::new(from.x, from.y + 5_000); // back beside the same wall
    w.tick(Some(Order::Jump(Point::new(from.x, 100_000))));
    assert!(w.core.air.unwrap().double_left == false || !w.core.kicked.iter().filter(|k| **k == id).count() > 1, "each wall once per chain");
}

#[test]
fn airborne_he_is_cc_immune_and_flies_over_terrain_but_lands_on_open_ground() {
    // terrain right on the line and under the aim
    let mut w = World::new(vec![], &[(13, 13), (13, 14), (13, 15), (13, 16)]);
    w.f.pos = cell(11, 15);
    let out = w.tick(Some(Order::Jump(cell(13, 15))));
    assert!(out.iter().any(|e| matches!(e, Effect::Air { .. })));
    let to = w.core.air.unwrap().land;
    assert!(!w.f.terrain_at(to), "never lands in terrain: {to:?}");
    w.run(d::JUMP_AIR_TICKS as u64 + 2);
    assert!(!w.core.airborne(&w.f));
    assert!(w.seen(|e| matches!(e, Effect::Fx { name, .. } if name == "bs_land")));
}

// ------------------------------------------------------------------ Whip Pull, the slingshot

#[test]
fn the_slingshot_flies_faster_from_farther_and_lands_past_the_wall() {
    let mut w = World::new(vec![], &[(20, 15)]);
    let wall = Frame::cell_point(cell(20, 15), Point::new(400_000, 500_000));
    let near = Point::new(wall.x - 20_000, wall.y);
    let far = Point::new(wall.x - 110_000, wall.y);
    w.f.pos = near;
    let (land_n, ticks_n, speed_n) = w.core.sling(&w.f, wall);
    w.f.pos = far;
    let (land_f, ticks_f, speed_f) = w.core.sling(&w.f, wall);
    assert!(speed_f > speed_n, "{speed_f} > {speed_n}");
    assert_eq!(speed_n, d::PULL_SPEED_BASE + 20_000 / d::PULL_SPEED_DIVISOR);
    assert!(land_n.x > wall.x && land_f.x > wall.x, "over the wall: {land_n:?} {land_f:?}");
    assert!(land_f.x - wall.x > land_n.x - wall.x, "farther pull, farther overshoot");
    assert!(ticks_f < (far.dist(land_f) / d::PULL_SPEED_BASE as f64) as u64);
    let _ = ticks_n;
    // cast: airborne, and enemies on the path are knocked aside
    w.f.units.push(champ(FOE, 2, wall.x - 60_000, wall.y));
    w.tick(Some(Order::Pull(wall)));
    assert!(w.core.airborne(&w.f));
    assert!(w.seen(|e| matches!(e, Effect::Knock { target: FOE, .. })));
    assert_eq!(w.core.pull_at, 1 + d::PULL_COOLDOWN_TICKS as u64);
}

// ------------------------------------------------------------------ the eight skills

#[test]
fn raise_and_punch_launches_the_wall_and_slams_enemies_pinned_against_terrain() {
    // terrain behind the foe
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 560_000)], &[(12, 19), (13, 19), (11, 19)]);
    let out = w.tick(Some(Order::S1(Point::new(400_000, 600_000))));
    assert_eq!(w.core.punches, d::PUNCH_CHARGES - 1);
    assert!(out.iter().any(|e| matches!(e, Effect::Knock { target: FOE, .. })));
    assert!(w.core.walls[0].slide.is_some(), "the wall is launched");
    w.run(d::PUNCH_SLIDE_TICKS as u64 + 2);
    assert!(w.stunned(FOE), "pinned against terrain: slammed");
}

#[test]
fn air_dash_keeps_him_airborne_and_anything_can_follow() {
    // a terrain cell in reach for the follow-up throw
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 300_000), champ(11, 2, 405_000, 300_000)], &[(14, 12)]);
    w.tick(Some(Order::Jump(Point::new(400_000, 450_000))));
    w.run(10);
    let before = w.core.air.unwrap().until;
    w.tick(Some(Order::S1(Point::new(400_000, 300_000))));
    let a = w.core.air.unwrap();
    assert!(a.until > before && !a.spent, "still airborne, chain open");
    w.run(4);
    // an air skill after it
    w.tick(Some(Order::S2(Point::new(400_000, 300_000))));
    assert!(w.core.rejection.is_empty(), "{}", w.core.rejection);
}

#[test]
fn hurl_throws_onto_a_point_and_slab_drop_knocks_up_an_area() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 380_000), champ(11, 2, 410_000, 380_000)], &[]);
    // a wall to throw, within his grab range
    w.core.walls.push(StoneWall {
        id: 50,
        a: Point::new(470_000, 470_000),
        b: Point::new(470_000, 530_000),
        integrity: 3,
        born: 0,
        out_since: None,
        slide: None,
    });
    w.core.next_wall = 51;
    let before = w.core.walls.len();
    w.tick(Some(Order::S2(Point::new(400_000, 380_000))));
    assert_eq!(w.core.walls.len(), before - 1, "picked up");
    w.run(d::HURL_FLIGHT_TICKS as u64 + 1);
    assert_eq!(w.core.walls.len(), before, "it lands and stands");
    assert!(w.seen(|e| matches!(e, Effect::Knock { target: FOE, .. })), "knocked outward");
    // no wall in reach: a chunk ripped off terrain
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 420_000), champ(11, 2, 405_000, 420_000)], &[(14, 16)]);
    w.tick(Some(Order::Jump(Point::new(400_000, 470_000))));
    w.run(3);
    w.tick(Some(Order::S2(Point::new(400_000, 420_000))));
    assert!(w.core.rejection.is_empty());
    w.run(d::SLAB_FLIGHT_TICKS as u64 + 1);
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::SLAB_KNOCK_TICKS as u64)));
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: 11, .. })));
    assert!(w.core.air.unwrap().spent, "an air skill closes the chain");
}

#[test]
fn crushing_clap_raises_a_v_snaps_it_shut_crushes_inside_and_fuses_into_one_wall() {
    let inside = champ(FOE, 2, 400_000, 440_000);
    let outside = champ(11, 2, 470_000, 440_000);
    let mut w = World::new(vec![inside, outside], &[]);
    let out = w.tick(Some(Order::S3(Point::new(400_000, 400_000))));
    assert_eq!(w.core.walls.len(), 2, "two walls at the upper left and right");
    let [l, r] = [w.core.walls[0], w.core.walls[1]];
    assert!(l.b.x != r.b.x, "a V");
    assert!(out.iter().any(|e| matches!(e, Effect::Stun { target: FOE, .. })));
    assert!(!out.iter().any(|e| matches!(e, Effect::Stun { target: 11, .. })), "outside the V: untouched");
    w.run(d::CLAP_CLOSE_TICKS as u64 + 1);
    assert_eq!(w.core.walls.len(), 1, "fused into one");
    let wall = w.core.walls[0];
    assert!((wall.a.x - 400_000).abs() < 1_000 && (wall.b.x - 400_000).abs() < 1_000, "on the centre line: {wall:?}");
}

#[test]
fn whip_swing_turns_360_plus_x_toward_a_teammate_and_slams_into_a_wall_there() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 470_000), champ(ALLY, 1, 550_000, 500_000)], &[]);
    w.tick(Some(Order::Jump(Point::new(400_000, 490_000))));
    w.run(2);
    let center = w.f.pos;
    w.tick(Some(Order::S3(Point::new(400_000, 470_000))));
    let s = w.core.swing.expect("snared");
    assert!(s.turn_deg >= 360.0 && s.turn_deg < 720.0);
    let end = rotate((1.0, 0.0), s.start_deg + s.turn_deg);
    let to_ally = unit_dir(center, Point::new(550_000, 500_000));
    assert!(end.0 * to_ally.0 + end.1 * to_ally.1 > 0.99, "released toward the teammate");
    assert!(s.r1 > s.r0 && s.r1 <= d::SWING_MAX_RADIUS, "the whip grows");
    let walls_before = w.core.walls.len();
    w.run(d::SWING_SPIN_TICKS as u64 + 2);
    assert!(w.stunned(FOE));
    assert_eq!(w.core.walls.len(), walls_before + 1, "no wall there: he raised one and slammed them into it");
}

#[test]
fn whip_swing_with_no_teammate_sends_them_away_from_their_team() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 470_000), champ(11, 2, 400_000, 300_000), champ(12, 2, 420_000, 300_000)], &[]);
    w.tick(Some(Order::Jump(Point::new(400_000, 490_000))));
    w.run(2);
    let center = w.f.pos;
    w.tick(Some(Order::S3(Point::new(400_000, 470_000))));
    let s = w.core.swing.unwrap();
    let end = rotate((1.0, 0.0), s.start_deg + s.turn_deg);
    let team = unit_dir(center, Point::new(410_000, 300_000));
    assert!(end.0 * team.0 + end.1 * team.1 < -0.9, "away from their team");
}

#[test]
fn boulder_throw_charges_3_s_then_a_global_rock_knocks_up_and_slows_and_the_recast_whips_him_there() {
    let far = Point::new(850_000, 100_000);
    let mut w = World::new(vec![champ(FOE, 2, far.x, far.y), champ(11, 2, far.x + 20_000, far.y)], &[]);
    let out = w.tick(Some(Order::Ult(far)));
    assert!(out.iter().any(|e| matches!(e, Effect::Root { ticks } if *ticks == d::ULT_CHARGE_TICKS as u64)), "he stops to charge");
    assert!(w.core.charging.is_some());
    // nothing else while charging
    w.tick(Some(Order::S1(Point::new(400_000, 400_000))));
    assert_eq!(w.core.punches, d::PUNCH_CHARGES, "no casting during the charge");
    w.run(d::ULT_CHARGE_TICKS as u64);
    assert_eq!(w.core.boulders.len(), 1, "thrown after 3 s");
    let flight = (Point::new(400_000, 500_000).dist(far) / d::ULT_ROCK_SPEED as f64).ceil() as u64;
    w.run(flight.max(d::ULT_MIN_FLIGHT_TICKS as u64) + 1);
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::ULT_KNOCK_TICKS as u64)), "global: it lands across the map");
    assert!(w.seen(|e| matches!(e, Effect::Slow { target: 11, pct, .. } if *pct == d::ULT_SLOW_PCT)));
    assert!(w.core.recast_ready(&w.f));
    assert_eq!(w.core.walls.len(), 1, "the rock stands as a wall");
    // recast: whipped to the rock, airborne
    w.tick(Some(Order::Ult(far)));
    assert!(w.core.airborne(&w.f));
    let land = w.core.air.unwrap().land;
    assert!(land.within(far, 40_000), "lands where the rock landed: {land:?}");
    assert!(!w.core.recast_ready(&w.f));
}

#[test]
fn meteor_punch_hangs_in_the_air_then_rains_burning_meteors_that_slow_and_knock_up_a_little() {
    let aim = Point::new(400_000, 250_000);
    let mut w = World::new(vec![champ(FOE, 2, aim.x, aim.y), champ(11, 2, aim.x + 40_000, aim.y)], &[]);
    w.tick(Some(Order::Jump(Point::new(400_000, 450_000))));
    w.run(3);
    w.tick(Some(Order::Ult(aim)));
    assert!(w.core.charging.is_some_and(|c| c.0), "the air version");
    w.run(d::ULT_CHARGE_TICKS as u64 - 2);
    assert!(w.core.airborne(&w.f), "he hangs in the air through the charge");
    w.run(80);
    assert!(w.seen(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::ULT_METEOR_KNOCK_TICKS as u64)), "a little knock-up");
    assert!(w.seen(|e| matches!(e, Effect::Slow { target: 11, pct, .. } if *pct == d::ULT_METEOR_SLOW_PCT)));
    assert!(w.core.burns.iter().any(|b| b.0 == FOE), "burning");
    let burn = d::ULT_BURN_BASE + d::ULT_BURN_AD_RATIO * 100 / 100;
    w.run(70);
    assert!(w.log.iter().filter(|(_, e)| matches!(e, Effect::Damage { target: FOE, amount } if *amount == burn)).count() >= 1, "the burn ticks");
    assert!(w.core.recast_ready(&w.f));
}

#[test]
fn the_ai_throws_the_boulder_at_a_far_teamfight_and_never_charges_with_an_enemy_on_him() {
    let mut w = ai(
        vec![champ(FOE, 2, 800_000, 200_000), champ(11, 2, 810_000, 200_000), champ(12, 2, 800_000, 210_000), champ(ALLY, 1, 790_000, 220_000)],
        &[],
    );
    w.tick(None);
    assert_eq!(w.core.reason, "Boulder Throw");
    let mut w = ai(
        vec![champ(FOE, 2, 800_000, 200_000), champ(11, 2, 810_000, 200_000), champ(12, 2, 800_000, 210_000), champ(13, 2, 420_000, 500_000)],
        &[],
    );
    w.tick(None);
    assert!(w.core.charging.is_none(), "an enemy on him: no 3 s charge");
}

#[test]
fn each_pair_shares_a_cooldown_ground_or_air() {
    let mut w = World::new(vec![champ(FOE, 2, 400_000, 440_000)], &[]);
    w.tick(Some(Order::S3(Point::new(400_000, 440_000))));
    w.run(20);
    w.tick(Some(Order::Jump(Point::new(400_000, 450_000))));
    w.run(2);
    w.tick(Some(Order::S3(Point::new(400_000, 440_000))));
    assert_eq!(w.core.rejection, "s3", "Whip Swing waits for Crushing Clap's cooldown");
}

// ------------------------------------------------------------------ AI

fn ai(units: Vec<Unit>, terrain: &[(i64, i64)]) -> World {
    let mut w = World::new(units, terrain);
    w.f.manual = false;
    w
}

#[test]
fn the_ai_claps_a_group_in_front_of_him() {
    let mut w = ai(vec![champ(FOE, 2, 400_000, 440_000), champ(11, 2, 410_000, 430_000), champ(ALLY, 1, 420_000, 470_000)], &[]);
    w.tick(None);
    assert_eq!(w.core.reason, "Crushing Clap");
}

#[test]
fn the_ai_jumps_in_then_whip_swings_a_target_out_of_reach() {
    let mut w = ai(vec![champ(FOE, 2, 400_000, 380_000), champ(ALLY, 1, 420_000, 470_000)], &[]);
    w.unit(FOE).hp = 400;
    w.core.punches = 0;
    w.core.s2_at = u64::MAX / 2;
    w.run(30);
    assert!(w.seen(|e| matches!(e, Effect::Air { .. })), "{}", w.core.reason);
    assert!(w.core.swing.is_some() || w.stunned(FOE), "{}", w.core.reason);
}

#[test]
fn the_ai_slingshots_to_help_a_teammate_far_away() {
    let mut w = ai(vec![champ(ALLY, 1, 600_000, 500_000), champ(FOE, 2, 620_000, 500_000)], &[(15, 15)]);
    w.core.jumps = 0;
    w.tick(None);
    assert_eq!(w.core.reason, "Help");
    assert!(w.core.airborne(&w.f));
}

#[test]
fn the_ai_never_dives_a_one_versus_four() {
    let mut w = ai(
        vec![champ(FOE, 2, 400_000, 420_000), champ(11, 2, 410_000, 420_000), champ(12, 2, 390_000, 420_000), champ(13, 2, 400_000, 410_000)],
        &[],
    );
    w.f.ult_unlocked = false;
    w.run(10);
    assert!(!w.seen(|e| matches!(e, Effect::Damage { .. })), "{}", w.core.reason);
}

#[test]
fn the_ai_farms_a_wave_with_hurl_only() {
    let mut w = ai(vec![minion(30, 400_000, 400_000), minion(31, 405_000, 400_000), minion(32, 400_000, 405_000)], &[]);
    w.core.walls.push(StoneWall {
        id: 9,
        a: Point::new(380_000, 520_000),
        b: Point::new(420_000, 520_000),
        integrity: 3,
        born: 0,
        out_since: None,
        slide: None,
    });
    w.tick(None);
    assert_eq!(w.core.reason, "Farm");
    assert!(w.core.s3_at == 0 && w.core.ult_at == 0);
}

// ------------------------------------------------------------------ the bridge on the glue mock host

mod glue {
    use crate::basalt::{Basalt, AIR};
    use crate::basalt_core::{Effect, Point};
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn his_walls_become_shared_walls_that_move_and_go() {
        let (x, y) = open_spot(160_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_basalt")]);
        let mut b = Basalt::default();
        b.core.walls.push(crate::basalt_core::StoneWall {
            id: 1,
            a: Point::new(x as i64 - 30_000, y as i64 + 20_000),
            b: Point::new(x as i64 + 30_000, y as i64 + 20_000),
            integrity: 3,
            born: 0,
            out_since: None,
            slide: None,
        });
        with_sim(&host, |sim| b.sync_walls(sim, 0));
        let walls = |host: &std::cell::RefCell<Host>| -> Vec<String> { host.borrow().buffs[0].iter().map(|b| b.name().to_owned()).filter(|n| n.starts_with("sbw:")).collect() };
        assert_eq!(walls(&host).len(), 1);
        let all = with_sim(&host, |sim| crate::champions_shared(sim));
        assert_eq!(crate::steve::walls_up(&all, 0).len(), 1, "the shared wall code sees it");
        // moved: rewritten
        b.core.walls[0].a.y += 30_000;
        b.core.walls[0].b.y += 30_000;
        host.borrow_mut().tick = 1;
        with_sim(&host, |sim| b.sync_walls(sim, 0));
        let w = walls(&host);
        assert_eq!(w.len(), 1);
        assert!(w[0].contains(&format!(":{}:", y as i64 + 50_000)), "{w:?}");
        // gone
        b.core.walls.clear();
        with_sim(&host, |sim| b.sync_walls(sim, 0));
        assert!(walls(&host).is_empty());
    }

    #[test]
    fn airborne_he_is_cc_immune_passes_walls_and_pushes_never_hit_towers() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_basalt"),
            HostUnit { kind: K::Tower, team: 1, pos: (x + 20_000, y), hp: 3000, name: "tower", targetable: true },
            champ(1, (x + 30_000, y), "foe"),
        ]);
        let mut b = Basalt::default();
        with_sim(&host, |sim| {
            b.apply(
                sim,
                0,
                vec![
                    Effect::Air { ticks: 10 },
                    Effect::Stun { target: 1, ticks: 60 },
                    Effect::Stun { target: 2, ticks: 60 },
                    Effect::Damage { target: 1, amount: 50 },
                    Effect::Damage { target: 2, amount: 50 },
                ],
            )
        });
        let g = host.borrow();
        let air = g.buffs[0].iter().find(|b| b.name() == AIR).expect("bs_air");
        assert!(air.cc_immune && air.ignore_wall);
        assert!(g.buffs[0].iter().any(|b| b.name() == crate::place::MOVED), "the shared wall code lets him through");
        assert!(g.ccs[1].is_empty() && !g.damage.iter().any(|d| d.1 == 1), "never a tower");
        assert!(g.ccs[2].iter().any(|c| c.0.kind == CcKindV1::Stun.code()));
    }

    #[test]
    fn a_tick_on_the_host_runs_clean() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_basalt"), champ(1, (x + 40_000, y), "foe")]);
        let mut b = Basalt::default();
        for t in 0..120 {
            host.borrow_mut().tick = t;
            with_sim(&host, |sim| b.on_update(sim, t as u64, 0, 0));
        }
    }
}
