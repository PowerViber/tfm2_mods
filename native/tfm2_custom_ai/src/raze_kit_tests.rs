//! Round 144: Raze's MOBA kit (ultimate unlock, air shield, Hype, lane tools, fight rewards, scaling, mastery).
use super::*;

fn damage_of<'a>(out: &'a [Cmd], source: &str, target: usize) -> Vec<&'a Cmd> {
    out.iter()
        .filter(|v| matches!(v, Cmd::Damage { source: s, target: t, self_hit: false, .. } if s == source && *t == target))
        .collect()
}
fn raw_of(c: &Cmd) -> i64 {
    match c {
        Cmd::Damage { raw, .. } => *raw,
        _ => 0,
    }
}
fn grenade_at(c: &mut Core, f: &mut Frame, aim: Point) -> Vec<Cmd> {
    let mut out = tick(c, f, Some(Action::Grenade { aim }));
    out.extend(advance(c, f, 90));
    out
}

#[test]
fn ultimate_locked_until_the_learned_marker_then_fires() {
    let (mut c, mut f) = (ready(), frame());
    f.unlocked[2] = false;
    let out = tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::Rejected { reason } if reason.starts_with("Ultimate locked"))));
    assert!(c.cast.is_none());
    // the bridge ORs the slot flag with the ult_learned marker into this frame field
    f.unlocked[2] = true;
    equip(&mut c, &mut f);
    assert!(c.armed_until > f.tick);
    assert_eq!(c.armed_at, f.tick, "armed on release");
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(350000, 300000),
        }),
    );
    assert!(c.rocket.is_some());
}

#[test]
fn an_ai_rocket_holds_the_armed_pose_before_firing() {
    let (mut c, f) = (ready(), frame());
    let me = f.units[0].clone();
    let cand = c
        .evaluate(&f, &me)
        .into_iter()
        .find(|v| v.family == "rocket")
        .unwrap();
    let plan = c.make_plan(&f, &me, &cand, f.tick);
    let equip = plan.steps[0].at;
    let fire = plan
        .steps
        .iter()
        .find(|s| matches!(s.action, Action::FireRocket { .. }))
        .unwrap()
        .at;
    assert_eq!(fire, equip + (ROCKET_PREP + ROCKET_HOLD) as u64 + 1);
}

/// Applies the air shield's layers to the frame as the engine would (layers stack and expire).
fn shielded(c: &mut Core, f: &mut Frame, layers: &mut Vec<(i64, u64)>, a: Option<Action>) -> Vec<Cmd> {
    let out = tick(c, f, a);
    for cmd in &out {
        if let Cmd::Shield { amount, ticks } = cmd {
            layers.push((*amount, f.tick + ticks));
        }
    }
    layers.retain(|(_, end)| f.tick < *end);
    f.shield = layers.iter().map(|l| l.0).sum();
    out
}

#[test]
fn air_shield_is_one_layer_that_lasts_the_flight_and_lingers_like_landing_defence() {
    let (mut c, mut f) = (ready(), frame());
    let mut layers = vec![];
    let full = f.units[0].max_hp * AIR_SHIELD_HP_PERCENT / 100 + f.units[0].attack * AIR_SHIELD_ATTACK / 100;
    let aim = f.units[0].pos.add(Point::new(-10000, 0));
    shielded(&mut c, &mut f, &mut layers, Some(Action::Pack { aim }));
    for _ in 0..PACK_THROW + 1 {
        shielded(&mut c, &mut f, &mut layers, None);
    }
    let out = shielded(&mut c, &mut f, &mut layers, Some(Action::Detonate { id: 0 }));
    assert!(out.contains(&Cmd::Shield {
        amount: full,
        ticks: c.flight.as_ref().unwrap().until - f.tick
    }));
    // a hit takes part of it; a second launch tops up only what was lost and never stacks a second full layer
    layers[0].0 -= 50;
    f.shield -= 50;
    shielded(&mut c, &mut f, &mut layers, None);
    pack(&mut c, &mut f);
    let out = shielded(&mut c, &mut f, &mut layers, Some(Action::Detonate { id: 0 }));
    let top_up: Vec<_> = out
        .iter()
        .filter_map(|v| match v {
            Cmd::Shield { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(top_up, vec![50]);
    assert!(f.shield <= full);
    // through the flight and the landing: one enemy near the landing keeps it 180 ticks after she lands
    let mut landed = None;
    for _ in 0..400 {
        // the enemy keeps up with her, so one enemy champion is near wherever she lands
        f.units[1].pos = f.units[0].pos.add(Point::new(20000, 0));
        shielded(&mut c, &mut f, &mut layers, None);
        assert!(f.shield <= full, "never more than one full shield");
        if landed.is_none() && c.flight.is_none() {
            landed = Some(f.tick);
        }
        if c.air_shield.is_none() {
            break;
        }
    }
    let landed = landed.unwrap();
    assert_eq!(f.tick, landed + landing_protection_ticks(1));
    assert!(c.air_shield.is_none());
}

#[test]
fn landing_alone_drops_the_air_shield_at_once() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(800000, 800000);
    let mut layers = vec![];
    pack(&mut c, &mut f);
    shielded(&mut c, &mut f, &mut layers, Some(Action::Detonate { id: 0 }));
    assert!(c.air_shield.is_some());
    while c.flight.is_some() {
        shielded(&mut c, &mut f, &mut layers, None);
    }
    shielded(&mut c, &mut f, &mut layers, None);
    assert!(c.air_shield.is_none());
}

#[test]
fn hype_fills_from_explosions_and_bursts_into_carnaval() {
    let (mut c, mut f) = (ready(), frame());
    c.hype = HYPE_MAX - HYPE_CHAMPION_HIT;
    grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    assert!(c.carnaval(f.tick), "a champion hit filled the meter");
    assert_eq!(c.hype, 0);
    // free cooldowns recover twice as fast
    c.cooldowns[1] = f.tick + 100;
    advance(&mut c, &mut f, 10);
    assert_eq!(c.cooldowns[1], f.tick + 80);
    // an extra cluster lands in the middle during Carnaval
    let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    let clusters = out
        .iter()
        .filter(|v| matches!(v, Cmd::Visual { tag, .. } if tag == "cluster_blast"))
        .count();
    assert_eq!(clusters, 5);
    // out of combat it fades 5 per second
    let (mut c, mut f) = (ready(), frame());
    c.hype = 50;
    advance(&mut c, &mut f, (HYPE_COMBAT_TICKS + 2 * TPS) as usize);
    assert!((38..=42).contains(&c.hype), "{}", c.hype);
}

#[test]
fn a_skilled_athlete_saves_carnaval_for_a_fight() {
    let held = |mastery: u8, enemy: Point| {
        let (mut c, mut f) = (ready(), frame());
        f.automatic = true;
        f.mastery = mastery;
        f.units[1].pos = enemy;
        c.hype = HYPE_MAX;
        tick(&mut c, &mut f, None);
        !c.carnaval(f.tick)
    };
    let far = Point::new(800000, 800000);
    assert!(held(CARNAVAL_HOLD_RANK as u8, far));
    assert!(!held(CARNAVAL_HOLD_RANK as u8, Point::new(350000, 300000)));
    assert!(!held(0, far));
}

#[test]
fn graffiti_slows_and_marks_and_a_satchel_push_into_it_stuns() {
    let (mut c, mut f) = (ready(), frame());
    let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    assert!(out.iter().any(|v| matches!(v, Cmd::Visual { tag, .. } if tag == "graffiti")));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::Slow { target: 2, percent, .. } if *percent == GRAFFITI_SLOW)));
    // inside paint a satchel hit deals GRAFFITI_AMPLIFY more
    let satchel = |painted: bool| {
        let (mut c, mut f) = (ready(), frame());
        f.units[1].pos = Point::new(294000, 300000);
        if painted {
            c.paints.push(Zone {
                id: 99,
                pos: f.units[1].pos,
                radius: GRENADE_RADIUS,
                born: 0,
                until: 1_000_000,
                attack: 0,
            });
        }
        pack(&mut c, &mut f);
        advance(&mut c, &mut f, PACK_ARM as usize);
        let out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
        raw_of(damage_of(&out, "satchel", 2)[0])
    };
    let raw = PACK_DAMAGE + 70 * PACK_ATTACK / 100;
    assert_eq!(satchel(false), skill_raw(raw));
    assert_eq!(satchel(true), skill_raw(raw * (100 + GRAFFITI_AMPLIFY) / 100));
    // a satchel push into a fresh Paint Shells blast stuns
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(294000, 300000);
    pack(&mut c, &mut f);
    c.paint_blasts.push((Point::new(290000, 300000), f.tick));
    let out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert!(out.contains(&Cmd::Stun {
        target: 2,
        ticks: COMBO_STUN as u64
    }));
}

#[test]
fn encore_knocks_up_once_after_three_explosions() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(300000, 310000);
    c.explosions = vec![f.tick; 2];
    pack(&mut c, &mut f);
    let mut out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    // the satchel itself is the third explosion; landing next to the enemy fires the shockwave
    for _ in 0..80 {
        let step = tick(&mut c, &mut f, None);
        f.units[1].pos = f.units[0].pos.add(Point::new(0, 10000));
        out.extend(step);
        if c.flight.is_none() {
            break;
        }
    }
    assert!(out.iter().any(|v| matches!(v, Cmd::KnockUp { target: 2, .. })));
    assert_eq!(damage_of(&out, "encore", 2).len(), 1);
    assert!(c.encore_ready > f.tick);
    // on cooldown: the next landing sends nothing
    c.explosions = vec![f.tick; 3];
    pack(&mut c, &mut f);
    let mut out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    out.extend(advance(&mut c, &mut f, 80));
    assert!(!out.iter().any(|v| matches!(v, Cmd::KnockUp { .. })));
}

#[test]
fn paint_shells_clears_two_minion_waves_and_clusters_hit_minions_harder() {
    let (c, mut f) = (ready(), frame());
    f.units.truncate(1);
    for (id, y) in [(10, 296000), (11, 304000)] {
        let mut m = unit(id, 1, 360000, y);
        m.kind = 1;
        m.farmable = true;
        f.units.push(m);
    }
    let me = f.units[0].clone();
    assert!(c
        .evaluate(&f, &me)
        .iter()
        .any(|v| v.family == "grenade" && v.legal));
    // a cluster on a minion deals CLUSTER_FARM_PERCENT
    let (mut c, mut f) = (ready(), frame());
    f.units[1].kind = 1;
    f.units[1].pos = Point::new(350000 + CLUSTER_OFFSET, 300000 + CLUSTER_OFFSET);
    let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    let raw = CLUSTER_DAMAGE + 70 * CLUSTER_ATTACK_NUM / CLUSTER_ATTACK_DEN;
    assert_eq!(
        raw_of(damage_of(&out, "cluster", 2)[0]),
        skill_raw(raw * CLUSTER_FARM_PERCENT / 100)
    );
}

#[test]
fn boom_bot_hunts_the_wave_when_no_champion_is_in_sight_and_hits_structures_harder() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(800000, 800000);
    let mut m = unit(10, 1, 300000, 360000);
    m.kind = 1;
    f.units.push(m);
    // thrown away from the minion: it turns to hunt it
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(250000, 300000),
        }),
    );
    advance(&mut c, &mut f, BOT_PREP as usize + 3);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(10));
    // a tower inside the blast takes BOT_STRUCTURE_PERCENT
    let (mut c, mut f) = (ready(), frame());
    let mut tower = unit(3, 1, 352000, 300000);
    tower.kind = 4;
    f.units.push(tower);
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(350000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 120);
    let raw = BOT_DAMAGE + 70 * BOT_ATTACK / 100;
    assert_eq!(raw_of(damage_of(&out, "bot", 2)[0]), skill_raw(raw));
    assert_eq!(
        raw_of(damage_of(&out, "bot", 3)[0]),
        skill_raw(raw * BOT_STRUCTURE_PERCENT / 100)
    );
}

#[test]
fn bucky_slug_reaches_thirty_thousand_only_with_the_bucky() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(325000, 300000);
    let out = tick(&mut c, &mut f, Some(Action::Slug { target: 2 }));
    assert_eq!(raw_of(damage_of(&out, "slug", 2)[0]), 70 * SLUG_ATTACK / 100);
    assert!(out.iter().any(|v| matches!(v, Cmd::Tracer { .. })));
    // on its cooldown, out of range, or with another gun it is refused
    let out = tick(&mut c, &mut f, Some(Action::Slug { target: 2 }));
    assert!(damage_of(&out, "slug", 2).is_empty());
    advance(&mut c, &mut f, SLUG_COOLDOWN as usize);
    f.units[1].pos = Point::new(300000 + SLUG_RANGE + 1000, 300000);
    let out = tick(&mut c, &mut f, Some(Action::Slug { target: 2 }));
    assert!(damage_of(&out, "slug", 2).is_empty());
    f.units[1].pos = Point::new(325000, 300000);
    c.gun = Gun::Vandal.idx();
    let out = tick(&mut c, &mut f, Some(Action::Slug { target: 2 }));
    assert!(damage_of(&out, "slug", 2).is_empty());
    // the planner offers it for a safe mid-range trade, not up close
    let (c, mut f) = (ready(), frame());
    let me = f.units[0].clone();
    f.units[1].pos = Point::new(320000, 300000);
    assert!(c.evaluate(&f, &me).iter().any(|v| v.family == "slug" && v.legal));
    f.units[1].pos = Point::new(308000, 300000);
    assert!(!c.evaluate(&f, &me).iter().any(|v| v.family == "slug" && v.legal));
}

#[test]
fn hot_hands_combustion_and_the_blast_pack_reset() {
    // Hot Hands: the second main-blast champion hit returns the free Paint Shells charge
    let (mut c, mut f) = (ready(), frame());
    c.hot_hands = HOT_HANDS_HITS as u8 - 1;
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    assert!(c.cooldowns[0] > f.tick);
    advance(&mut c, &mut f, 60);
    assert!(c.cooldowns[0] <= f.tick);
    assert_eq!(c.stock[1], 1, "a reward never adds a bought charge");
    // Combustion: the third stack deals true damage and reveals
    let (mut c, mut f) = (ready(), frame());
    c.burns.insert(2, (BURN_STACKS as u8 - 1, 1_000_000));
    let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    let burst = damage_of(&out, "combustion", 2);
    assert_eq!(burst.len(), 1);
    assert!(matches!(burst[0], Cmd::Damage { raw, fixed: true, .. } if *raw == 2000 * BURN_MAX_HP_PERCENT / 100));
    assert!(out.contains(&Cmd::Reveal {
        target: 2,
        ticks: BURN_REVEAL_TICKS as u64
    }));
    // a kill on someone she just damaged returns the free Blast Pack and refunds part of the Showstopper
    let (mut c, mut f) = (ready(), frame());
    f.tick = 100;
    c.last_hits.insert(2, 90);
    c.pack_cooldown = 5000;
    c.cooldowns[2] = 5000;
    f.credits = vec![CreditEvent {
        id: 1,
        kind: 0,
        victim: 2,
    }];
    tick(&mut c, &mut f, None);
    assert_eq!(c.pack_cooldown, f.tick);
    assert_eq!(
        c.cooldowns[2],
        5000 - (ROCKET_COOLDOWN * ULT_TAKEDOWN_REFUND / 100) as u64
    );
    assert_eq!(c.hype, HYPE_TAKEDOWN);
    assert_eq!(c.credits, START_CREDITS + KILL_CREDITS + KILL_BOUNTY);
    // a kill on someone she never touched does not reset the pack
    let (mut c, mut f) = (ready(), frame());
    c.pack_cooldown = 5000;
    f.credits = vec![CreditEvent {
        id: 1,
        kind: 0,
        victim: 2,
    }];
    tick(&mut c, &mut f, None);
    assert_eq!(c.pack_cooldown, 5000);
}

#[test]
fn omnivamp_level_scaling_takedown_armour_and_the_level_six_sheriff() {
    let grenade = |level: i64, protected: bool| {
        let (mut c, mut f) = (ready(), frame());
        f.level = level;
        c.air_protection_until = if protected { 1_000_000 } else { 0 };
        let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
        match damage_of(&out, "grenade", 2)[0] {
            Cmd::Damage { raw, vamp, .. } => (*raw, *vamp),
            _ => unreachable!(),
        }
    };
    let raw = |level: i64| GRENADE_DAMAGE + SKILL_LEVEL_BASE * (level - 1) + 70 * GRENADE_ATTACK / 100;
    assert_eq!(grenade(1, false), (skill_raw(raw(1)), OMNIVAMP));
    assert_eq!(grenade(10, false).0, skill_raw(raw(10)));
    assert_eq!(grenade(1, true).1, OMNIVAMP_PROTECTED);
    // takedowns restore armour, never past the purchased amount
    let (mut c, mut f) = (ready(), frame());
    c.armor = 2;
    f.shield = 400;
    f.credits = vec![CreditEvent {
        id: 1,
        kind: 1,
        victim: 0,
    }];
    let out = tick(&mut c, &mut f, None);
    assert!(out.contains(&Cmd::Armor { amount: 450 }));
    // still on the Bucky at level 6: a free Sheriff, once; a bought gun is kept
    let (mut c, mut f) = (ready(), frame());
    f.level = FREE_SHERIFF_LEVEL;
    let out = tick(&mut c, &mut f, None);
    assert!(out.contains(&Cmd::Gun {
        index: Gun::Sheriff.idx()
    }));
    assert!(!advance(&mut c, &mut f, 5)
        .iter()
        .any(|v| matches!(v, Cmd::Gun { .. })));
    let (mut c, mut f) = (ready(), frame());
    c.gun = Gun::Vandal.idx();
    f.level = 9;
    assert!(!tick(&mut c, &mut f, None)
        .iter()
        .any(|v| matches!(v, Cmd::Gun { .. })));
}

#[test]
fn primed_targets_take_more_from_the_next_grenade() {
    let (mut c, mut f) = (ready(), frame());
    c.primed.insert(2, 1_000_000);
    let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
    let raw = GRENADE_DAMAGE + 70 * GRENADE_ATTACK / 100;
    assert_eq!(
        raw_of(damage_of(&out, "grenade", 2)[0]),
        skill_raw(raw * (100 + PRIMED_PERCENT) / 100)
    );
    assert!(!c.primed.contains_key(&2), "consumed");
    // a Boom Bot hit primes
    let (mut c, mut f) = (ready(), frame());
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(350000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 120);
    assert!(!damage_of(&out, "bot", 2).is_empty());
    assert!(c.primed.contains_key(&2));
}

#[test]
fn showstopper_leaves_a_burning_zone() {
    let (mut c, mut f) = (ready(), frame());
    equip(&mut c, &mut f);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(350000, 300000),
        }),
    );
    let mut out = vec![];
    while c.rocket.is_some() {
        out.extend(tick(&mut c, &mut f, None));
    }
    assert!(out.iter().any(|v| matches!(v, Cmd::Visual { tag, .. } if tag == "rocket_scorch")));
    assert_eq!(c.burn_zones.len(), 1);
    let out = advance(&mut c, &mut f, BURN_ZONE_TICKS as usize + 5);
    let burns = damage_of(&out, "burn", 2);
    assert_eq!(burns.len() as i64, BURN_ZONE_TICKS / TPS);
    assert_eq!(raw_of(burns[0]), skill_raw(70 * BURN_ZONE_ATTACK / 100));
    assert!(c.burn_zones.is_empty());
}

#[test]
fn mastery_changes_planning_never_raw_power() {
    // manual play: identical commands at every rank
    let run = |mastery: u8, automatic: bool| {
        let (mut c, mut f) = (ready(), frame());
        f.mastery = mastery;
        f.automatic = automatic;
        let out = grenade_at(&mut c, &mut f, Point::new(350000, 300000));
        (out, c)
    };
    assert_eq!(run(0, false).0, run(7, false).0);
    // automatic: a Radiant athlete aims, times and plans better than the pinned profile says
    let (_, iron) = run(0, true);
    let (_, radiant) = run(7, true);
    for n in [4, 10, 12] {
        assert!(radiant.profile.get(n) > iron.profile.get(n), "{n}");
    }
    for n in [0, 1, 2, 3, 5, 6, 7, 8, 9, 11] {
        assert_eq!(radiant.profile.get(n), iron.profile.get(n), "{n}");
    }
    assert_eq!(radiant.base(GRENADE_DAMAGE), iron.base(GRENADE_DAMAGE));
}

// ---- round 145: Paint Shells pay for what they are likely to hit ----

fn grenade_on(c: &Core, f: &Frame, target: usize) -> Option<Candidate> {
    let me = f.units[0].clone();
    c.evaluate(f, &me)
        .into_iter()
        .filter(|v| v.family == "grenade" && v.target == Some(target))
        .max_by_key(|v| (v.legal, v.score))
}

#[test]
fn hit_chance_falls_with_distance_and_holds_for_controlled_or_painted_targets() {
    let (c, mut f) = (ready(), frame());
    f.unlocked = [true; 3];
    let u = f.units[1].clone();
    let near = c.grenade_hit_chance(&f, &u, 20000);
    let mid = c.grenade_hit_chance(&f, &u, 60000);
    let far = c.grenade_hit_chance(&f, &u, 140000);
    assert!(near > mid && mid > far, "{near} {mid} {far}");
    assert_eq!(far, 0, "a free champion walks out of a throw from 140,000");
    let mut held = u.clone();
    held.controlled = true;
    assert!(c.grenade_hit_chance(&f, &held, 140000) >= 80, "control holds them in the blast");
    let mut painted = ready();
    painted.paints.push(Zone { id: 1, pos: u.pos, radius: 30000, born: 0, until: 1000, attack: 0 });
    assert!(painted.grenade_hit_chance(&f, &u, 60000) > mid, "Graffiti's slow helps the next throw");
    let mut minion = u.clone();
    minion.kind = 1;
    assert_eq!(c.grenade_hit_chance(&f, &minion, 140000), 100, "waves and camps are not dodging");
}

#[test]
fn a_long_throw_at_a_moving_champion_is_held_but_a_controlled_one_is_thrown() {
    let (mut c, mut f) = (ready(), frame());
    f.unlocked = [true; 3];
    c.stock = [0, 0, 0];
    f.units[1].pos = Point::new(440000, 300000);
    let free = grenade_on(&c, &f, 2).unwrap();
    assert!(!free.legal, "the last charge is not spent on a 140,000 throw at a free target: {}", free.reason);
    f.units[1].controlled = true;
    let held = grenade_on(&c, &f, 2).unwrap();
    assert!(held.legal && held.score > free.score, "{}", held.reason);
}

#[test]
fn the_last_charge_waits_for_a_trade_but_spares_and_crowds_throw() {
    let (mut c, mut f) = (ready(), frame());
    f.unlocked = [true; 3];
    f.units[1].pos = Point::new(355000, 300000);
    c.stock = [0, 0, 0];
    assert!(!grenade_on(&c, &f, 2).unwrap().legal, "one charge, a free champion at 55,000: kept");
    c.stock = [0, 1, 0];
    assert!(grenade_on(&c, &f, 2).unwrap().legal, "with a spare charge she may still throw");
    c.stock = [0, 0, 0];
    let mut second = f.units[1].clone();
    second.id = 3;
    second.pos = Point::new(357000, 305000);
    f.units.push(second);
    assert!(grenade_on(&c, &f, 2).unwrap().legal, "two champions in the blast are worth the last charge");
    // close in, the last charge is a trade that lands
    f.units.truncate(2);
    f.units[1].pos = Point::new(318000, 300000);
    assert!(grenade_on(&c, &f, 2).unwrap().legal, "18,000 away: thrown");
}

#[test]
fn farming_throws_on_a_wave_keep_the_last_charge_rule_out() {
    let (mut c, mut f) = (ready(), frame());
    f.unlocked = [true; 3];
    f.farm_window = true;
    c.stock = [0, 0, 0];
    f.units.truncate(1);
    for i in 0..3 {
        let mut m = unit(10 + i, 1, 350000, 296000 + i as i64 * 4000);
        m.kind = 1;
        m.farmable = true;
        m.hp = 400;
        m.max_hp = 400;
        f.units.push(m);
    }
    assert!(grenade_on(&c, &f, 10).unwrap().legal, "a wave clear with the last charge is unchanged");
}
