//! Supported ABI evidence, separate from real-game rendering/targeting acceptance.
use super::*;
use mod_api_stable::{EntityHandleV1 as E, InputTargetV1, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};

#[test]
fn wall_attachment_identities_survive_growth_translation_and_rotation() {
    let w = crate::steve::Wall {
        ax: 300000,
        ay: 300000,
        bx: 340000,
        by: 300000,
        t0: 0,
        tb: 20,
        tend: 600,
    };
    let growing = wall_surfaces(&w, 10, Some((3, 7, 0)));
    let full = wall_surfaces(&w, 20, Some((3, 7, 0)));
    for (a, b) in growing.iter().zip(&full) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.bounds.min, b.bounds.min);
    }
    let shifted = crate::steve::Wall {
        ax: 300000,
        ay: 312000,
        bx: 340000,
        by: 312000,
        ..w.clone()
    };
    let moved = wall_surfaces(&shifted, 20, Some((3, 7, 0)));
    assert_eq!(full.len(), moved.len());
    for (a, b) in full.iter().zip(&moved) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.bounds.min.y + 12000, b.bounds.min.y);
    }
    let turned = crate::steve::Wall {
        ax: 300000,
        ay: 300000,
        bx: 300000,
        by: 340000,
        ..w
    };
    let rotated = wall_surfaces(&turned, 20, Some((3, 7, 0)));
    assert_eq!(
        full.iter().map(|s| s.id).collect::<Vec<_>>(),
        rotated.iter().map(|s| s.id).collect::<Vec<_>>()
    );
}
#[derive(Default)]
struct Host {
    hp: BTreeMap<usize, usize>,
    buffs: BTreeMap<usize, Vec<BuffV1>>,
    cc: Vec<(usize, CcV1)>,
    spawns: Vec<(String, StatV1, UnitAttackV1)>,
    effects: Vec<(String, u64)>,
    invisible: usize,
    immune: bool,
    packets: Vec<(usize, usize, usize, u32)>,
    shields: BTreeMap<usize, usize>,
    magic_resistance: BTreeMap<usize, usize>,
    defence: BTreeMap<usize, usize>,
}
unsafe fn host<'a>(p: *mut c_void) -> &'a mut Host {
    &mut *(p as *mut Host)
}
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool {
    (*(p as *const Host)).hp.contains_key(&e.id().unwrap_or(0))
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    (*(p as *const Host))
        .hp
        .get(&e.id().unwrap_or(0))
        .copied()
        .unwrap_or(0)
        > 0
}
unsafe extern "C" fn targetable(p: *const c_void, e: E) -> bool {
    alive(p, e)
}
unsafe extern "C" fn shield(p: *const c_void, e: E) -> usize {
    (*(p as *const Host))
        .shields
        .get(&e.id().unwrap_or(0))
        .copied()
        .unwrap_or(0)
}
unsafe extern "C" fn hp(p: *const c_void, e: E, a: *mut usize, b: *mut usize) -> bool {
    *a = (*(p as *const Host))
        .hp
        .get(&e.id().unwrap_or(0))
        .copied()
        .unwrap_or(0);
    *b = 1000;
    true
}
unsafe extern "C" fn pos(_: *const c_void, _: E, x: *mut u64, y: *mut u64) -> bool {
    *x = 320000;
    *y = 300000;
    true
}
unsafe extern "C" fn kind(_: *const c_void, _: E) -> u32 {
    EntityKindV1::Champion.code()
}
unsafe extern "C" fn buff_count(p: *const c_void, e: E) -> usize {
    (*(p as *const Host))
        .buffs
        .get(&e.id().unwrap_or(0))
        .map_or(0, Vec::len)
}
unsafe extern "C" fn buff_at(p: *const c_void, e: E, n: usize, b: *mut BuffV1) -> bool {
    let h = &*(p as *const Host);
    if h.immune && n == 0 {
        let mut v = BuffV1::named("immune");
        v.cc_immune = true;
        *b = v;
        return true;
    }
    if let Some(v) = h.buffs.get(&e.id().unwrap_or(0)).and_then(|v| v.get(n)) {
        *b = *v;
        true
    } else {
        false
    }
}
unsafe extern "C" fn add(p: *mut c_void, id: usize, b: *const BuffV1) {
    host(p).buffs.entry(id).or_default().push(*b);
}
unsafe extern "C" fn remove(p: *mut c_void, e: E, n: *const u8, len: usize) -> usize {
    let name = std::str::from_utf8(std::slice::from_raw_parts(n, len)).unwrap();
    let v = host(p).buffs.entry(e.id().unwrap()).or_default();
    let before = v.len();
    v.retain(|b| b.name() != name);
    before - v.len()
}
unsafe extern "C" fn cc(p: *mut c_void, id: usize, c: *const CcV1) {
    host(p).cc.push((id, *c));
}
unsafe extern "C" fn damage(p: *mut c_void, _: usize, id: usize, ad: usize, ap: usize, kind: u32) {
    let h = host(p);
    h.packets.push((id, ad, ap, kind));
    let amount = ad * 100 / (100 + h.defence.get(&id).copied().unwrap_or(0))
        + ap * 100 / (100 + h.magic_resistance.get(&id).copied().unwrap_or(0));
    let shield = h.shields.entry(id).or_default();
    let absorbed = amount.min(*shield);
    *shield -= absorbed;
    if let Some(hp) = h.hp.get_mut(&id) {
        *hp = hp.saturating_sub(amount - absorbed);
    }
}
unsafe extern "C" fn spawn(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _: usize,
    _: usize,
    _: u64,
    _: u64,
    _: u64,
    s: *const StatV1,
    a: *const UnitAttackV1,
) -> usize {
    let h = host(p);
    h.spawns.push((
        String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into(),
        *s,
        *a,
    ));
    h.hp.insert(3, (*s).hp);
    3
}
unsafe extern "C" fn invisible(p: *mut c_void, _: E, _: usize) -> bool {
    host(p).invisible += 1;
    true
}
unsafe extern "C" fn effect(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _: usize,
    _: *const InputTargetV1,
    _: u64,
    _: u64,
    ticks: u64,
) -> bool {
    host(p).effects.push((
        String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into(),
        ticks,
    ));
    true
}
unsafe fn table() -> SimVtableV1 {
    let mut t: SimVtableV1 = std::mem::zeroed();
    t.size = size_of::<SimVtableV1>();
    t.entity_is_valid = Some(valid);
    t.entity_is_alive = Some(alive);
    t.entity_is_targetable = Some(targetable);
    t.entity_hp = Some(hp);
    t.entity_pos = Some(pos);
    t.entity_kind = Some(kind);
    t.entity_buff_count = Some(buff_count);
    t.entity_buff_at = Some(buff_at);
    t.add_buff = Some(add);
    t.entity_remove_buff = Some(remove);
    t.apply_cc = Some(cc);
    t.deal_damage = Some(damage);
    t.entity_shield = Some(shield);
    t.spawn_unit = Some(spawn);
    t.entity_set_invisible = Some(invisible);
    t.play_view_effect = Some(effect);
    t
}
#[test]
fn compact_carrier_is_real_damageable_and_never_uses_entity_invisibility() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![Cmd::Spawn {
                id: 7,
                bot: false,
                pos: Point::new(320000, 300000),
                hp: 40,
                ticks: 300,
            }],
            0,
            1,
            &[],
        );
        assert_eq!(h.spawns[0].0, "alchemist");
        assert_eq!(h.spawns[0].1.hp, 40);
        assert_eq!(h.spawns[0].2.attack, 0);
        assert!(sim.get_entity(3).unwrap().is_targetable());
        sim.deal_damage(2, 3, 0, 11, AttackTypeV1::Skill);
        assert_eq!(h.hp[&3], 29);
        assert_eq!(h.invisible, 0);
        assert!(h.cc.iter().any(|(id, c)| *id == 3
            && c.kind == CcKindV1::Animation.code()
            && &c.name_buf[..c.name_len] == b"rz_satchel_hidden"));
        for k in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            assert!(h.cc.iter().any(|(id, c)| *id == 3 && c.kind == k.code()));
        }
    }
}
#[test]
fn rocket_warning_replaces_and_cleans_its_exact_owner_ledger() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        for (tick, x) in [(1, 400000), (2, 410000)] {
            r.commands(
                &mut sim,
                vec![
                    Cmd::Warning {
                        id: 7,
                        tag: "rocket_impact_warning".into(),
                        pos: Point::new(x, 300000),
                        radius: 50000,
                        until: 32,
                    },
                    Cmd::EnemyGuide {
                        target: 2,
                        rocket: 7,
                        to: Point::new(450000, 350000),
                        until: 32,
                    },
                ],
                0,
                tick,
                &[],
            );
        }
        assert_eq!(r.zones.len(), 1);
        assert_eq!(
            h.buffs[&1]
                .iter()
                .filter(|b| b.name().starts_with("rzrw:7:"))
                .count(),
            1
        );
        assert!(h.effects.iter().all(|(_, ticks)| *ticks == 1));
        h.buffs
            .entry(2)
            .or_default()
            .push(BuffV1::named("rze:99:7:1:1:32"));
        h.hp.insert(1, 0);
        r.commands(&mut sim, vec![Cmd::ClearWarning { id: 7 }], 0, 3, &[]);
        assert!(r.zones.is_empty());
        assert!(r.enemy_guides.is_empty());
        assert_eq!(h.buffs[&2].len(), 1);
        assert!(h.buffs[&2][0].name().starts_with("rze:99:"));
    }
}

#[test]
fn round_bot_keeps_a_real_damageable_carrier_with_an_owner_only_pose() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![Cmd::Spawn {
                id: 8,
                bot: true,
                pos: Point::new(320000, 300000),
                hp: 250,
                ticks: 300,
            }],
            0,
            1,
            &[],
        );
        assert_eq!(h.spawns[0].0, "alchemist");
        assert_eq!(h.spawns[0].1.hp, 250);
        assert!(sim.get_entity(3).unwrap().is_targetable());
        sim.deal_damage(2, 3, 0, 11, AttackTypeV1::Skill);
        assert_eq!(h.hp[&3], 239);
        assert_eq!(h.invisible, 0);
        assert!(h.cc.iter().any(|(id, c)| *id == 3
            && c.kind == CcKindV1::Animation.code()
            && &c.name_buf[..c.name_len] == b"rz_bot_hidden"));
        // round 143: drawn once per tick as a one-tick phase, never on spawn as a 24-tick copy
        assert!(h.effects.is_empty());
        for t in 1..=7 {
            r.draw_carriers(&mut sim, t);
        }
        let names: Vec<_> = h.effects.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        assert_eq!(names.len(), 7);
        assert_eq!(names[0], ("tfm2_valorant_raze_bot_round_0_p0", 1));
        assert_eq!(names[3], ("tfm2_valorant_raze_bot_round_0_p1", 1));
        assert_eq!(names[6], ("tfm2_valorant_raze_bot_round_0_p2", 1));
    }
}
#[test]
fn rocket_knockback_skips_lethal_damage_and_control_immunity() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 10);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        let push = Cmd::Knockback {
            target: 2,
            to: Point::new(330000, 300000),
            ticks: 12,
        };
        r.commands(
            &mut sim,
            vec![
                Cmd::Damage {
                    id: 7,
                    target: 2,
                    raw: 100,
                    self_hit: false,
                    source: "fixture".into(),
                    vamp: 0,
                    fixed: false,
                },
                push.clone(),
            ],
            0,
            1,
            &[],
        );
        assert!(h.cc.is_empty());
        h.hp.insert(2, 1000);
        h.immune = true;
        h.buffs.insert(2, vec![BuffV1::named("immune")]);
        r.commands(&mut sim, vec![push.clone()], 0, 2, &[]);
        assert!(h.cc.is_empty());
        h.immune = false;
        h.buffs.clear();
        r.commands(&mut sim, vec![push], 0, 3, &[]);
        assert_eq!(h.cc.len(), 1);
        assert_eq!(h.cc[0].1.kind, CcKindV1::ForceMove.code());
        assert_eq!(h.cc[0].1.tick, 12);
    }
}

#[test]
fn native_skill_packets_are_physical_and_record_observed_damage_once() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 2000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![Cmd::Damage {
                id: 7,
                target: 2,
                raw: 399,
                self_hit: false,
                source: "grenade".into(),
                vamp: 0,
                fixed: false,
            }],
            0,
            1,
            &[],
        );
        assert_eq!(h.hp[&2], 601);
        assert_eq!(r.receipts.len(), 1);
        assert_eq!(h.packets, vec![(2, 399, 0, AttackTypeV1::Skill.code())]);
        let note = &r.receipts[0];
        assert!(note.submitted);
        assert_eq!(
            (
                note.requested,
                note.hp_before,
                note.hp_after,
                note.shield_before,
                note.shield_after
            ),
            (399, 1000, 601, 0, 0)
        );
        r.commands(
            &mut sim,
            vec![Cmd::Damage {
                id: 8,
                target: 999,
                raw: 399,
                self_hit: false,
                source: "grenade".into(),
                vamp: 0,
                fixed: false,
            }],
            0,
            2,
            &[],
        );
        assert!(!r.receipts[1].submitted);
        assert_eq!(h.hp[&2], 601);
        h.hp.insert(2, 1000);
        h.shields.insert(2, 100);
        h.magic_resistance.insert(2, 999);
        h.defence.insert(2, 100);
        r.commands(
            &mut sim,
            vec![Cmd::Damage {
                id: 9,
                target: 2,
                raw: 399,
                self_hit: false,
                source: "grenade".into(),
                vamp: 0,
                fixed: false,
            }],
            0,
            3,
            &[],
        );
        let note = &r.receipts[2];
        assert_eq!(
            (
                note.hp_before,
                note.hp_after,
                note.shield_before,
                note.shield_after
            ),
            (1000, 901, 100, 0)
        );
        assert_eq!(h.packets.len(), 2);
    }
}

#[test]
fn native_bucky_modifies_one_primary_attack_and_uses_its_own_modifier() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 2000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        let mut damage = 100;
        r.on_attack(&mut sim, 0, 1, 2, &mut damage);
        assert_eq!(damage, 300);
        assert_eq!(h.hp[&2], 1000);
        let b = Gun::Bucky.buff();
        assert_eq!(b.name(), "omn_gun6");
        assert_eq!(b.attack_speed_mult, -40);
        assert_eq!(b.attack_mult, 0);
    }
}

/// Round 143: drives the shared core through the host for `ticks`, returning each tick's (effect, ticks) plays.
unsafe fn played(
    actions: &[(u64, core::Action)],
    ticks: u64,
    setup: impl Fn(&mut Core, &mut Frame),
) -> Vec<Vec<(String, u64)>> {
    let mut h = Host::default();
    h.hp.insert(1, 1000);
    h.hp.insert(2, 1000);
    let t = table();
    let mut raw = SimCtxV1 {
        size: size_of::<SimCtxV1>(),
        sim: &t,
        frame: std::ptr::null(),
        state: &mut h as *mut _ as *mut c_void,
    };
    let mut sim = StableSim::from_raw(&mut raw).unwrap();
    let mut r = Raze {
        owner: 1,
        ..Default::default()
    };
    let me = Unit {
        id: 1,
        team: 0,
        pos: Point::new(300000, 300000),
        hp: 2000,
        max_hp: 2000,
        ap: 100,
        visible: true,
        targetable: true,
        ..Default::default()
    };
    let enemy = Unit {
        id: 2,
        team: 1,
        pos: Point::new(380000, 300000),
        ..me.clone()
    };
    let mut f = Frame {
        units: vec![me, enemy],
        ..Default::default()
    };
    let mut c = Core {
        stock: [1, 1, 1],
        ..Default::default()
    };
    setup(&mut c, &mut f);
    let mut out = vec![];
    for n in 1..=ticks {
        f.tick = n;
        f.action = actions
            .iter()
            .find(|(at, _)| *at == n)
            .map(|(_, a)| a.clone());
        let cmds = c.step(&f);
        for cmd in &cmds {
            match cmd {
                Cmd::MoveOwner { to } => f.units[0].pos = *to,
                Cmd::Spawn { id, hp, .. } => f.utility.push(UtilityStatus { id: *id, hp: *hp }),
                Cmd::Remove { id } => f.utility.retain(|s| s.id != *id),
                _ => {}
            }
        }
        r.core = c.clone();
        let before = h.effects.len();
        r.commands(&mut sim, cmds, 0, n as usize, &[]);
        r.draw_carriers(&mut sim, n);
        r.phases.retain(|_, (_, last)| *last + 1 >= n);
        out.push(h.effects[before..].to_vec());
    }
    out
}
#[test]
fn each_cluster_bomblet_plays_exactly_one_single_tick_phase_per_tick() {
    let ticks = unsafe {
        played(
            &[(
                1,
                core::Action::Grenade {
                    aim: Point::new(380000, 300000),
                },
            )],
            140,
            |_, _| {},
        )
    };
    let mut seen = 0;
    for (n, plays) in ticks.iter().enumerate() {
        let flights: Vec<_> = plays
            .iter()
            .filter(|(name, _)| name.starts_with("tfm2_valorant_raze_grenade_flight"))
            .collect();
        // the main shell, or the four bomblets: one play each, one tick long, a phase of the baked loop
        assert!(flights.len() <= 4, "tick {n}: {flights:?}");
        for (name, time) in &flights {
            assert_eq!(*time, 1, "{name}");
            assert!(name.contains("_p"), "{name}");
        }
        if flights.len() == 4 {
            seen += 1;
            let phase = &flights[0].0;
            assert!(flights.iter().all(|(name, _)| name == phase), "{flights:?}");
        }
        // the per-tick warning is a single phase too
        for (name, time) in plays.iter().filter(|(n, _)| n.contains("grenade_warning")) {
            assert!(name.contains("_p") && *time == 1, "{name}");
        }
    }
    assert!(
        seen >= crate::raze_data::CLUSTER_FUSE as usize - 1,
        "{seen}"
    );
}
#[test]
fn no_visual_sent_every_tick_has_an_animation_longer_than_one_tick() {
    use serde_json::Value;
    let champ: Value = serde_json::from_str(include_str!(
        "../../../mods/tfm2_valorant/champion/tfm2_valorant_raze.data_champion"
    ))
    .unwrap();
    let vfx: Value = serde_json::from_str(include_str!(
        "../../../mods/tfm2_valorant/vfx/raze#anim.fanim"
    ))
    .unwrap();
    let body: Value = serde_json::from_str(include_str!(
        "../../../mods/tfm2_valorant/champions/tfm2_valorant_raze#anim.fanim"
    ))
    .unwrap();
    let length = |name: &str| -> f64 {
        let v = champ["view_effects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap_or_else(|| panic!("{name} is not a view effect"));
        let sheet = if v["anim"].as_str().unwrap().ends_with("vfx/raze") {
            &vfx
        } else {
            &body
        };
        let anim = &sheet["anims"][v["tag"].as_str().unwrap()];
        anim["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["duration"].as_f64().unwrap())
            .sum()
    };
    // a grenade with clusters, a Boom Bot, a satchel launch in flight and an armed rocket in flight
    let a = core::Action::Grenade {
        aim: Point::new(380000, 300000),
    };
    let b = core::Action::Bot {
        aim: Point::new(380000, 300000),
    };
    let p = core::Action::Pack {
        aim: Point::new(290000, 300000),
    };
    let ticks = unsafe {
        played(
            &[
                (1, a),
                (30, b),
                (60, p),
                (80, core::Action::Detonate { id: 0 }),
                (82, core::Action::EquipRocket),
                (
                    130,
                    core::Action::FireRocket {
                        aim: Point::new(400000, 300000),
                    },
                ),
            ],
            170,
            |_, _| {},
        )
    };
    let mut checked = 0;
    for w in ticks.windows(2) {
        for (name, _) in w[1]
            .iter()
            .filter(|(n, _)| w[0].iter().any(|(m, _)| m == n))
        {
            assert!(
                length(name) <= 1.0 / 60.0 + 1e-9,
                "{name} is renewed every tick but plays {}",
                length(name)
            );
            checked += 1;
        }
    }
    assert!(checked > 100, "{checked}");
    let all: Vec<_> = ticks.iter().flatten().map(|(n, _)| n.as_str()).collect();
    for family in [
        "grenade_flight_p",
        "bot_round_",
        "satchel_",
        "airborne",
        "rocket_flight_",
    ] {
        assert!(
            all.iter().any(|n| n.contains(family)),
            "{family} never played"
        );
    }
    // round 144: the rocket in flight is the 1.5x art, and its impact scorches and burns
    for name in ["rocket_flight_big_", "rocket_scorch", "burn_zone", "graffiti"] {
        assert!(all.iter().any(|n| n.contains(name)), "{name} never played");
    }
}
#[test]
fn ult_learned_marker_unlocks_the_ultimate_when_the_slot_flag_does_not() {
    assert!(!ult_unlocked(false, &[BuffV1::named("rz_air")]));
    assert!(ult_unlocked(false, &[BuffV1::named(crate::ULT_LEARNED)]));
    assert!(ult_unlocked(true, &[]));
}
#[test]
fn round_144_effects_and_hud_buffs_are_in_the_champion_data() {
    use serde_json::Value;
    let champ: Value = serde_json::from_str(include_str!(
        "../../../mods/tfm2_valorant/champion/tfm2_valorant_raze.data_champion"
    ))
    .unwrap();
    assert_eq!(champ["passive_ult"]["passive_ref"], "tfm2_custom_ai:ult_learned");
    let names = |key: &str| -> Vec<String> {
        champ[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap().to_string())
            .collect()
    };
    let effects = names("view_effects");
    let mut want: Vec<String> = [
        "rocket_ready_big",
        "graffiti",
        "encore",
        "carnaval",
        "combustion",
        "rocket_scorch",
        "burn_zone",
    ]
    .iter()
    .map(|t| format!("{ID}_{t}"))
    .collect();
    want.extend((0..8).flat_map(|d| (0..8).map(move |k| format!("{ID}_rocket_flight_big_{d}_p{k}"))));
    for n in &want {
        assert!(effects.contains(n), "{n}");
    }
    assert!(names("view_projectiles").contains(&"tfm2_valorant_omen_tracer".to_string()));
    let buffs = names("view_buffs");
    for n in ["rz_hype1", "rz_hype5", "rz_carnaval", "rz_air_shield"] {
        assert!(buffs.iter().any(|b| b == n), "{n}");
    }
}
#[test]
fn shields_stuns_slows_and_reveals_reach_the_host() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![
                Cmd::Stun { target: 2, ticks: 24 },
                Cmd::KnockUp { target: 2, ticks: 18 },
                Cmd::Slow {
                    target: 2,
                    percent: 20,
                    ticks: 6,
                },
                Cmd::Slow {
                    target: 2,
                    percent: 20,
                    ticks: 6,
                },
                Cmd::Reveal { target: 2, ticks: 120 },
            ],
            0,
            1,
            &[],
        );
        assert!(h.cc.iter().any(|(id, c)| *id == 2 && c.kind == CcKindV1::Stun.code()));
        assert!(h.cc.iter().any(|(id, c)| *id == 2 && c.kind == CcKindV1::Airborne.code()));
        let paint: Vec<_> = h.buffs[&2].iter().filter(|b| b.name() == "rz_paint").collect();
        assert_eq!(paint.len(), 1, "renewed, never stacked");
        assert_eq!(paint[0].move_speed_mult, -20);
        assert!(h.buffs[&2].iter().any(|b| b.name() == "rz_revealed"));
        // control immunity keeps both off
        h.cc.clear();
        h.immune = true;
        r.commands(&mut sim, vec![Cmd::Stun { target: 2, ticks: 24 }], 0, 2, &[]);
        assert!(h.cc.is_empty());
    }
}
