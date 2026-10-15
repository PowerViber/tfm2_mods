use super::*;
#[path = "magnus_fixtures.rs"]
mod fixtures;
use crate::magnus_protocol::Frame;
fn frame() -> Frame {
    Frame::new(0, fixtures::owner())
}
fn run(c: &mut Core, f: &mut Frame, tick: usize, action: Option<Action>) -> Vec<Cmd> {
    f.tick = tick;
    f.action = action;
    let (_, out) = f.run(c);
    f.apply(&out);
    out
}
fn deployed(seed: u64) -> (Core, Frame) {
    let mut c = Core::new(seed, 1, 3);
    let mut f = frame();
    run(&mut c, &mut f, 0, Some(Action::Deploy { heading: 0 }));
    for t in 1..=60 {
        run(&mut c, &mut f, t, None);
    }
    assert_eq!(c.pieces.len(), 7);
    (c, f)
}
fn moved(c: &mut Core, f: &mut Frame, start: usize, id: usize, to: Cell) -> Vec<Cmd> {
    let mut out = run(c, f, start, Some(Action::Move { piece: id, to }));
    for t in start + 1..=start + 46 {
        out.extend(run(c, f, t, None));
    }
    out
}
fn add(c: &mut Core, k: Kind, x: i32, y: i32) -> usize {
    c.add_piece(k, Cell::new(x, y), 800 * k.hp_pct() / 100, None, 0)
}
fn foe(id: usize, x: i32, y: i32) -> Fighter {
    let mut f = fixtures::owner();
    f.id = id;
    f.team = 1;
    f.x = x as i64 * TILE;
    f.y = y as i64 * TILE;
    f.hp = 20_000;
    f.max_hp = 20_000;
    f.sight_source = false;
    f
}
fn historical(id: usize) {
    let mut f = fixtures::fixture(id).unwrap();
    for t in 0..=f.end {
        f.advance(t);
    }
    assert!(
        f.core.stats.rewards >= 1,
        "history {id}: {:?}",
        f.core.challenge
    );
    assert!(f.core.repeat_at[id] >= 3600);
}
macro_rules! histories{($($name:ident:$id:expr),*)=>{$(#[test]fn $name(){historical($id);})*}}
histories!(birthday:0,marathon:1,opera:2,century:3,immortal:4,hunt:5,double_bishop:6,overload:7,passed_pawn:8,royal_march:9,underpromotion:10,reti:11);
#[test]
fn twelve_histories_reject_hidden_kings() {
    for id in 0..12 {
        let mut f = fixtures::fixture(id).unwrap();
        for t in 0..=f.end {
            f.frame.fighters[0].visible = false;
            f.advance(t);
        }
        assert_eq!(f.core.stats.rewards, 0, "{id}");
    }
}
#[test]
fn twelve_histories_reject_interrupted_commands() {
    for id in 0..12 {
        let mut f = fixtures::fixture(id).unwrap();
        f.frame.controlled = true;
        for t in 0..=f.end {
            f.advance(t);
        }
        assert_eq!(f.core.stats.rewards, 0, "{id}");
    }
}
#[test]
fn twelve_histories_reject_missing_required_events() {
    for id in 0..12 {
        let mut f = fixtures::fixture(id).unwrap();
        if !f.captures.is_empty() {
            f.captures.clear();
        } else if id == 1 {
            f.relocate.clear();
        } else if id == 9 {
            f.walk.clear();
        }
        for t in 0..=f.end {
            if id == 10 || id == 11 {
                f.frame.hits.clear();
            }
            f.advance(t);
        }
        assert_eq!(f.core.stats.rewards, 0, "{id}");
    }
}
#[test]
fn cleanup_does_not_count_as_capture() {
    for id in [0, 2, 3, 4, 5, 6, 7, 8] {
        let mut f = fixtures::fixture(id).unwrap();
        for t in 0..=f.end {
            f.prepare(t);
            for c in &mut f.frame.captures {
                c.combat = false;
            }
            let (_, cmd) = f.frame.run(&mut f.core);
            f.frame.apply(&cmd);
        }
        assert_eq!(f.core.stats.rewards, 0, "{id}");
    }
}
#[test]
fn seed_layout_and_bishop_colours() {
    for seed in 0..96 {
        let (c, _) = deployed(seed);
        let (other, _) = deployed(seed);
        let positions = |c: &Core| {
            c.pieces
                .values()
                .map(|p| (p.slot, p.cell))
                .collect::<Vec<_>>()
        };
        assert_eq!(positions(&c), positions(&other));
        let b = c.board.unwrap();
        let me = Cell::at(b.x, b.y);
        let q = &c.pieces[&c.slots[6].unwrap()];
        assert_eq!(me.distance(q.cell), 1);
        let mut seen = BTreeSet::new();
        for p in c.pieces.values() {
            assert!(b.inside(p.cell.world().0, p.cell.world().1));
            assert!(seen.insert(p.cell));
            if p.kind != Kind::Queen {
                assert!(p.cell.distance(me) > 1);
            }
        }
        let bishops = c.kind_ids(Kind::Bishop);
        assert_ne!(
            (c.pieces[&bishops[0]].cell.x + c.pieces[&bishops[0]].cell.y) % 2,
            (c.pieces[&bishops[1]].cell.x + c.pieces[&bishops[1]].cell.y) % 2
        );
        assert!(c
            .pieces
            .values()
            .any(|p| b.cell(p.cell.world().0, p.cell.world().1).y > 0));
    }
}
#[test]
fn four_board_orientations_use_global_grid() {
    for h in 0..4 {
        let mut c = Core::new(5, 1, 0);
        let mut f = frame();
        run(&mut c, &mut f, 0, Some(Action::Deploy { heading: h }));
        run(&mut c, &mut f, 60, None);
        assert_eq!(c.board.unwrap().heading, h);
        assert_eq!(c.pieces.len(), 7);
        for p in c.pieces.values() {
            let (x, y) = p.cell.world();
            assert_eq!(x % TILE, 0);
            assert_eq!(y % TILE, 0);
        }
    }
}
#[test]
fn invalid_placement_preserves_previous_army() {
    let (mut c, mut f) = deployed(9);
    let before = format!("{:?}", c.pieces);
    let b = c.board;
    f.walls = vec![[0, 0, 960000, 960000]];
    run(&mut c, &mut f, 1600, Some(Action::Deploy { heading: 0 }));
    assert!(c.pending.is_none());
    assert_eq!(c.board, b);
    assert_eq!(format!("{:?}", c.pieces), before);
    assert_eq!(c.reset_at, 1500);
}
#[test]
fn losses_do_not_recover_with_time() {
    let (mut c, mut f) = deployed(4);
    let id = c.slots[0].unwrap();
    f.captures.push(Capture {
        piece: id,
        by: Some(2),
        combat: true,
    });
    run(&mut c, &mut f, 61, None);
    run(&mut c, &mut f, 10000, None);
    assert!(!c.pieces.contains_key(&id));
    assert_eq!(c.slots[0], None);
}
#[test]
fn rebuild_restores_queen_and_dead_slots_only() {
    let (mut c, mut f) = deployed(3);
    let rook = c.slots[0].unwrap();
    let q = c.slots[6].unwrap();
    let b = c.slots[2].unwrap();
    c.pieces.get_mut(&b).unwrap().hp = 31;
    c.pieces.get_mut(&q).unwrap().hp = 12;
    let extra = add(&mut c, Kind::Pawn, 30, 30);
    let extra_pos = c.pieces[&extra].cell;
    f.captures.push(Capture {
        piece: rook,
        by: Some(2),
        combat: true,
    });
    run(&mut c, &mut f, 61, None);
    run(&mut c, &mut f, 1500, Some(Action::Deploy { heading: 1 }));
    run(&mut c, &mut f, 1560, None);
    assert_eq!(c.pieces.len(), 8);
    assert_ne!(c.slots[0], Some(rook));
    assert_eq!(c.pieces[&c.slots[2].unwrap()].hp, 31);
    let q = &c.pieces[&c.slots[6].unwrap()];
    assert_eq!(q.hp, q.max_hp);
    assert_eq!(c.pieces[&extra].cell, extra_pos);
}
#[test]
fn departure_has_five_second_grace_and_retains_army() {
    let (mut c, mut f) = deployed(3);
    f.owner.x = 30 * TILE;
    let ids = c.pieces.keys().copied().collect::<Vec<_>>();
    run(&mut c, &mut f, 61, None);
    run(&mut c, &mut f, 360, None);
    assert!(c.board.is_some());
    run(&mut c, &mut f, 361, None);
    assert!(c.board.is_none());
    assert_eq!(c.pieces.keys().copied().collect::<Vec<_>>(), ids);
}
#[test]
fn reentry_cancels_departure() {
    let (mut c, mut f) = deployed(5);
    f.owner.x = 30 * TILE;
    run(&mut c, &mut f, 61, None);
    f.owner.x = 16 * TILE;
    run(&mut c, &mut f, 300, None);
    assert_eq!(c.outside_since, None);
    run(&mut c, &mut f, 600, None);
    assert!(c.board.is_some());
}
#[test]
fn death_cleans_without_sacrifices_and_preserves_clocks() {
    let (mut c, mut f) = deployed(5);
    c.pawn_at = 400;
    c.develop_at = 600;
    c.castle_at = 700;
    c.mate_at = 800;
    c.field_mate_at = 900;
    c.reward_at = 1000;
    c.repeat_at[0] = 4000;
    let captures = c.stats.captures;
    f.owner.hp = 0;
    let cmd = run(&mut c, &mut f, 61, None);
    assert_eq!(
        cmd.iter()
            .filter(|c| matches!(c, Cmd::Despawn { .. }))
            .count(),
        7
    );
    assert!(c.pieces.is_empty());
    assert!(c.board.is_none());
    assert_eq!(
        (
            c.pawn_at,
            c.develop_at,
            c.castle_at,
            c.mate_at,
            c.field_mate_at,
            c.reward_at
        ),
        (400, 600, 700, 800, 900, 1000)
    );
    assert_eq!(c.repeat_at[0], 4000);
    assert_eq!(c.stats.captures, captures);
}
#[test]
fn army_denominators_include_every_kind() {
    let mut c = Core::new(1, 1, 0);
    for i in 0..31 {
        add(&mut c, Kind::from_code(i % 5).unwrap(), i as i32, 30);
        if i == 6 {
            assert_eq!(c.army_denominator(), 100);
        }
        if i == 14 {
            assert_eq!(c.army_denominator(), 164);
        }
    }
    assert_eq!(c.army_denominator(), 292);
}
#[test]
fn off_board_damage_and_single_hit() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Rook, 18, 15);
    let mut f = frame();
    f.fighters.push(foe(2, 20, 15));
    let out = moved(&mut c, &mut f, 0, id, Cell::new(22, 15));
    let damage: Vec<_> = out
        .iter()
        .filter_map(|c| {
            if let Cmd::Damage { amount, .. } = c {
                Some(*amount)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(damage, vec![26]);
    assert!(c.pieces[&id].qualified);
    assert_eq!(c.pieces[&id].cell, Cell::new(22, 15));
}
#[test]
fn impact_territory_is_per_hit() {
    let mut c = Core::new(1, 1, 0);
    c.board = Some(Board {
        x: 20 * TILE,
        y: 12 * TILE,
        heading: 0,
        serial: 1,
    });
    let id = add(&mut c, Kind::Rook, 22, 15);
    let mut f = frame();
    f.owner.x = 20 * TILE;
    f.fighters = vec![foe(2, 23, 15), foe(3, 25, 15)];
    let out = moved(&mut c, &mut f, 0, id, Cell::new(27, 15));
    let amounts: Vec<_> = out
        .iter()
        .filter_map(|c| {
            if let Cmd::Damage { target, amount, .. } = c {
                Some((*target, *amount))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(amounts, vec![(2, 52), (3, 26)]);
}
#[test]
fn knight_jumps_but_cannot_land_in_wall() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Knight, 18, 15);
    let mut f = frame();
    f.walls.push([
        19 * TILE - 1000,
        15 * TILE - 1000,
        19 * TILE + 1000,
        16 * TILE + 1000,
    ]);
    let out = moved(&mut c, &mut f, 0, id, Cell::new(20, 16));
    assert!(out.iter().any(|c| matches!(c, Cmd::Move { .. })));
    assert_eq!(c.pieces[&id].cell, Cell::new(20, 16));
}
#[test]
fn sliders_respect_friends_magnus_walls_and_seven_square_limit() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Rook, 18, 15);
    let blocker = add(&mut c, Kind::Pawn, 20, 15);
    let mut f = frame();
    let input = |f: &Frame| f.values();
    assert!(!input(&f).is_empty());
    let check = |c: &Core, f: &Frame, to: Cell| {
        let wall = |x, y| f.blocked(x, y);
        let sight = |a, b, z, w| f.sight_clear(a, b, z, w);
        c.legal(
            id,
            to,
            &Input {
                tick: f.tick,
                owner: &f.owner,
                fighters: &f.fighters,
                wall: &wall,
                sight: &sight,
                captures: &[],
                hits: &[],
                piece_hp: &[],
                controlled: false,
                automatic: false,
                ult_unlocked: true,
                action: None,
            },
        )
    };
    assert!(!check(&c, &f, Cell::new(22, 15)));
    c.pieces.remove(&blocker);
    c.reindex();
    f.owner.x = 20 * TILE;
    f.owner.y = 15 * TILE;
    assert!(!check(&c, &f, Cell::new(22, 15)));
    f.owner = fixtures::owner();
    assert!(!check(&c, &f, Cell::new(26, 15)));
    f.walls.push([
        19 * TILE - 100,
        15 * TILE - 100,
        19 * TILE + 100,
        15 * TILE + 100,
    ]);
    assert!(!check(&c, &f, Cell::new(22, 15)));
}
#[test]
fn commitment_revalidates_owner_occupancy() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Rook, 18, 15);
    let mut f = frame();
    run(
        &mut c,
        &mut f,
        0,
        Some(Action::Move {
            piece: id,
            to: Cell::new(20, 15),
        }),
    );
    f.owner.x = 20 * TILE;
    f.owner.y = 15 * TILE;
    run(&mut c, &mut f, 18, None);
    assert!(c.pending.is_none());
    assert_eq!(c.pieces[&id].cell, Cell::new(18, 15));
}
#[test]
fn yielding_uses_clock_no_damage_and_no_promotion_progress() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Pawn, 16, 12);
    f.fighters = vec![foe(2, 17, 13)];
    run(&mut c, &mut f, 0, None);
    assert!(c.pending.as_ref().unwrap().yield_move);
    let mut all = vec![];
    for t in 1..=30 {
        all.extend(run(&mut c, &mut f, t, None));
    }
    assert_ne!(c.pieces[&id].cell, f.owner.cell());
    assert_eq!(c.pieces[&id].ranks, 0);
    assert!(!all.iter().any(|c| matches!(c, Cmd::Damage { .. })));
    assert!(!c.has_move);
}
#[test]
fn blocked_yield_stays_targetable_without_coverage() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Pawn, 16, 12);
    f.walls
        .push([15 * TILE, 13 * TILE - 11900, 17 * TILE, 15 * TILE]);
    run(&mut c, &mut f, 0, None);
    assert!(c.pieces[&id].yielding);
    assert!(c.pending.is_none());
    let wall = |x, y| f.blocked(x, y);
    let sight = |a, b, z, w| f.sight_clear(a, b, z, w);
    assert!(!c.threatens(
        id,
        Cell::new(17, 13),
        &Input {
            tick: 0,
            owner: &f.owner,
            fighters: &f.fighters,
            wall: &wall,
            sight: &sight,
            captures: &[],
            hits: &[],
            piece_hp: &[],
            controlled: false,
            automatic: false,
            ult_unlocked: true,
            action: None
        }
    ));
}
#[test]
fn summons_work_without_a_board_and_serialize() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    run(
        &mut c,
        &mut f,
        0,
        Some(Action::Summon {
            kind: Kind::Pawn,
            to: Cell::new(17, 12),
            direction: 1,
        }),
    );
    let out = run(
        &mut c,
        &mut f,
        10,
        Some(Action::Summon {
            kind: Kind::Knight,
            to: Cell::new(16, 13),
            direction: 0,
        }),
    );
    assert!(!out.iter().any(|c| matches!(c, Cmd::Spawn { .. })));
    run(&mut c, &mut f, 30, None);
    assert_eq!(c.pieces.len(), 1);
    assert_eq!(c.pawn_at, 390);
    let p = c.pieces.values().next().unwrap();
    assert_eq!(p.direction, 1);
    assert_eq!(p.hp, 120);
    assert!(c.board.is_none());
}
#[test]
fn remote_allied_sight_and_smoke() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let mut ally = fixtures::owner();
    ally.id = 7;
    ally.x = 26 * TILE;
    ally.y = 26 * TILE;
    ally.champion = false;
    f.fighters.push(ally);
    run(
        &mut c,
        &mut f,
        0,
        Some(Action::Summon {
            kind: Kind::Knight,
            to: Cell::new(27, 26),
            direction: 0,
        }),
    );
    run(&mut c, &mut f, 30, None);
    assert_eq!(c.pieces.len(), 1);
    let mut blocked = Core::new(1, 1, 0);
    f.smokes.push([26 * TILE, 26 * TILE, TILE]);
    run(
        &mut blocked,
        &mut f,
        0,
        Some(Action::Summon {
            kind: Kind::Knight,
            to: Cell::new(27, 26),
            direction: 0,
        }),
    );
    assert!(blocked.pending.is_none());
    f.smokes.clear();
    f.fighters[0].friendly_chess = true;
    f.fighters[0].sight_source = false;
    run(
        &mut blocked,
        &mut f,
        1,
        Some(Action::Summon {
            kind: Kind::Pawn,
            to: Cell::new(27, 26),
            direction: 0,
        }),
    );
    assert!(blocked.pending.is_none());
}
#[test]
fn pawn_cardinal_steps_do_no_damage_and_have_no_double_step() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Pawn, 18, 15);
    let mut f = frame();
    let out = moved(&mut c, &mut f, 0, id, Cell::new(18, 16));
    assert!(!out.iter().any(|c| matches!(c, Cmd::Damage { .. })));
    assert_eq!(c.pieces[&id].ranks, 1);
    moved(&mut c, &mut f, 50, id, Cell::new(18, 18));
    assert_eq!(c.pieces[&id].cell, Cell::new(18, 16));
}
#[test]
fn pawn_diagonal_damage_requires_visible_enemy() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Pawn, 18, 15);
    let mut f = frame();
    f.fighters = vec![foe(2, 19, 16)];
    f.fighters[0].visible = false;
    moved(&mut c, &mut f, 0, id, Cell::new(19, 16));
    assert_eq!(c.pieces[&id].cell, Cell::new(18, 15));
    f.fighters[0].visible = true;
    let out = moved(&mut c, &mut f, 50, id, Cell::new(19, 16));
    assert!(out
        .iter()
        .any(|c| matches!(c, Cmd::Damage { target: 2, .. })));
    assert!(c.pieces[&id].qualified);
}
#[test]
fn empty_pawn_marches_do_not_promote() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Pawn, 18, 15);
    let mut f = frame();
    for n in 0..6 {
        moved(&mut c, &mut f, n * 50, id, Cell::new(18, 16 + n as i32));
    }
    assert_eq!(c.pieces[&id].ranks, 6);
    assert_eq!(c.pieces[&id].kind, Kind::Pawn);
}
#[test]
fn promotion_preserves_identity_and_health_fraction() {
    let mut f = fixtures::fixture(10).unwrap();
    f.core.pieces.get_mut(&7).unwrap().hp = 60;
    for t in 0..=63 {
        f.advance(t);
    }
    assert_eq!(f.core.pieces[&7].kind, Kind::Rook);
    assert_eq!(f.core.pieces[&7].hp, 120);
    assert_eq!(f.core.pieces[&7].max_hp, 240);
    assert_eq!(f.core.pieces[&7].slot, None);
    assert_eq!(f.core.stats.promotions, 1);
}
#[test]
fn fake_or_zero_damage_receipts_never_qualify() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Pawn, 18, 15);
    let mut f = frame();
    f.hits = vec![Hit {
        command: 0,
        piece: id,
        target: 2,
        amount: 100,
    }];
    run(&mut c, &mut f, 0, None);
    assert!(!c.pieces[&id].qualified);
    let mut f = fixtures::fixture(10).unwrap();
    for t in 0..=63 {
        for h in &mut f.frame.hits {
            h.amount = 0;
        }
        f.advance(t);
    }
    assert_eq!(f.core.pieces[&7].kind, Kind::Pawn);
}
#[test]
fn interception_tracks_visible_lane_crossing_and_expires() {
    let mut c = Core::new(1, 1, 0);
    let id = add(&mut c, Kind::Pawn, 18, 15);
    let mut f = frame();
    f.fighters = vec![foe(2, 17, 16)];
    run(&mut c, &mut f, 0, None);
    f.fighters[0].x = 18 * TILE;
    run(&mut c, &mut f, 1, None);
    f.fighters[0].x = 19 * TILE;
    run(&mut c, &mut f, 2, None);
    assert_eq!(c.pieces[&id].intercept, Some((2, Cell::new(19, 16), 62)));
    run(&mut c, &mut f, 63, None);
    assert!(c.pieces[&id].intercept.is_none());
}
fn trap_case(board: bool) -> (Core, Frame) {
    let mut c = Core::new(1, 1, 3);
    let mut f = frame();
    f.owner.x = 20 * TILE;
    f.owner.y = 18 * TILE;
    let b = Board {
        x: 20 * TILE,
        y: 18 * TILE,
        heading: 0,
        serial: 1,
    };
    if board {
        c.board = Some(b);
    }
    c.projections = vec![Projection {
        id: 2,
        role: Role::King,
    }];
    f.fighters = vec![foe(2, 20, 20)];
    add(&mut c, Kind::Rook, 20, 22);
    add(&mut c, Kind::Bishop, 22, 22);
    f.walls = fixtures::trap(b, Cell::new(4, 2), &[Cell::new(4, 3), Cell::new(5, 3)]);
    c.has_move = true;
    c.field_target = Some(2);
    (c, f)
}
#[test]
fn board_mate_stuns_and_grants_immunity() {
    let (mut c, mut f) = trap_case(true);
    run(&mut c, &mut f, 0, None);
    let out = run(&mut c, &mut f, 24, None);
    assert!(out.contains(&Cmd::Stun {
        target: 2,
        ticks: 75
    }));
    assert!(out.contains(&Cmd::Immune {
        target: 2,
        ticks: 720
    }));
    assert_eq!(c.mate_at, 1224);
}
#[test]
fn field_mate_stuns_single_king() {
    let (mut c, mut f) = trap_case(false);
    f.fighters.push(foe(3, 21, 20));
    run(&mut c, &mut f, 0, None);
    let out = run(&mut c, &mut f, 24, None);
    assert!(out.contains(&Cmd::Stun {
        target: 2,
        ticks: 45
    }));
    assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { target: 3, .. })));
    assert_eq!(c.field_mate_at, 504);
}
#[test]
fn no_deployment_only_mate() {
    let (mut c, mut f) = trap_case(true);
    c.has_move = false;
    run(&mut c, &mut f, 0, None);
    let out = run(&mut c, &mut f, 24, None);
    assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { .. })));
}
#[test]
fn trap_warning_cancels_when_piece_dies() {
    let (mut c, mut f) = trap_case(true);
    run(&mut c, &mut f, 0, None);
    f.captures.push(Capture {
        piece: 1,
        by: Some(2),
        combat: true,
    });
    let out = run(&mut c, &mut f, 24, None);
    assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { .. })));
    assert!(c.mate_warning.is_none());
}
#[test]
fn safe_exits_cancel_warning() {
    let (mut c, mut f) = trap_case(true);
    run(&mut c, &mut f, 0, None);
    f.walls.clear();
    run(&mut c, &mut f, 24, None);
    assert!(c.mate_warning.is_none());
    assert_eq!(c.stats.mates, 0);
}
#[test]
fn mate_immunity_is_shared_by_input_between_owners() {
    let (mut a, mut f) = trap_case(true);
    run(&mut a, &mut f, 0, None);
    run(&mut a, &mut f, 24, None);
    let (mut b, mut g) = trap_case(true);
    b.owner = 8;
    g.owner.id = 8;
    g.fighters[0].mate_immune_until = f.fighters[0].mate_immune_until;
    run(&mut b, &mut g, 25, None);
    let out = run(&mut b, &mut g, 49, None);
    assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { .. })));
}
#[test]
fn repeat_mate_on_same_board_after_cooldown() {
    let (mut c, mut f) = trap_case(true);
    run(&mut c, &mut f, 0, None);
    run(&mut c, &mut f, 24, None);
    run(&mut c, &mut f, 1224, None);
    let out = run(&mut c, &mut f, 1248, None);
    assert!(out.contains(&Cmd::Stun {
        target: 2,
        ticks: 75
    }));
    assert_eq!(c.stats.mates, 2);
}
#[test]
fn repeated_ticks_emit_nothing() {
    let (mut c, mut f) = trap_case(true);
    run(&mut c, &mut f, 0, None);
    let before = format!("{c:?}");
    assert!(run(&mut c, &mut f, 0, None).is_empty());
    assert_eq!(format!("{c:?}"), before);
}
#[test]
fn manual_inputs_have_identical_results_at_every_rank() {
    let mut results = vec![];
    for rank in 0..8 {
        let mut c = Core::new(1, 1, rank);
        let mut f = frame();
        let id = add(&mut c, Kind::Queen, 18, 15);
        f.fighters = vec![foe(2, 20, 15)];
        results.push(moved(&mut c, &mut f, 0, id, Cell::new(22, 15)));
    }
    for result in &results[1..] {
        assert_eq!(result, &results[0]);
    }
}
#[test]
fn independent_clones_have_independent_queues() {
    let (mut c, mut f) = deployed(6);
    let copy = c.clone();
    let id = *c.pieces.keys().next().unwrap();
    f.captures.push(Capture {
        piece: id,
        by: Some(2),
        combat: true,
    });
    run(&mut c, &mut f, 61, None);
    assert!(copy.pieces.contains_key(&id));
    assert!(!c.pieces.contains_key(&id));
}
#[test]
fn version_three_transport_roundtrips_dynamic_inputs() {
    let mut f = frame();
    for id in 2..400 {
        f.fighters.push(foe(id, 20, 20));
        f.hp.push((id, 100));
    }
    f.hits.push(Hit {
        command: 9,
        piece: 8,
        target: 2,
        amount: 20,
    });
    let values = f.values();
    let decoded = Frame::decode(&values).unwrap();
    assert_eq!(decoded.values(), values);
    let mut bad = values;
    bad[10] = i64::MAX;
    assert!(Frame::decode(&bad).is_none());
    assert!(Frame::decode(&[1; 30]).is_none());
}
#[test]
fn changing_lab_mastery_updates_choices_without_changing_piece_stats() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Knight, 18, 15);
    f.rank = 7;
    run(&mut c, &mut f, 0, None);
    assert_eq!(c.rank, 7);
    assert_eq!(c.pieces[&id].hp, 200);
    f.rank = 0;
    run(&mut c, &mut f, 1, None);
    assert_eq!(c.rank, 0);
    assert_eq!(c.pieces[&id].hp, 200);
}
#[test]
fn challenge_compatibility_requires_its_secondary_enemy_roles() {
    let (mut c, mut f) = deployed(5);
    let available = |c: &Core, f: &Frame| {
        let wall = |_, _| false;
        let sight = |_, _, _, _| true;
        c.compatible(&Input {
            tick: 61,
            owner: &f.owner,
            fighters: &f.fighters,
            wall: &wall,
            sight: &sight,
            captures: &[],
            hits: &[],
            piece_hp: &[],
            controlled: false,
            automatic: false,
            ult_unlocked: true,
            action: None,
        })
    };
    c.projections = vec![Projection {
        id: 2,
        role: Role::King,
    }];
    f.fighters = vec![foe(2, 18, 15)];
    assert!(!available(&c, &f).contains(&7));
    assert!(!available(&c, &f).contains(&3));
    c.projections.push(Projection {
        id: 3,
        role: Role::Queen,
    });
    f.fighters.push(foe(3, 19, 16));
    assert!(!available(&c, &f).contains(&7));
    assert!(available(&c, &f).contains(&3));
    c.projections.push(Projection {
        id: 4,
        role: Role::Knight,
    });
    f.fighters.push(foe(4, 20, 17));
    assert!(available(&c, &f).contains(&7));
}
#[test]
fn global_summons_reject_occupied_enemy_and_allied_squares() {
    for team in [0, 1] {
        let mut c = Core::new(1, 1, 3);
        let mut f = frame();
        let at = f.owner.cell();
        let mut unit = foe(2, at.x + 1, at.y);
        unit.team = team;
        f.fighters.push(unit);
        run(
            &mut c,
            &mut f,
            0,
            Some(Action::Summon {
                kind: Kind::Pawn,
                to: Cell::new(at.x + 1, at.y),
                direction: 0,
            }),
        );
        assert!(c.pending.is_none());
        assert_eq!(c.pawn_at, 0);
        assert!(c.pieces.is_empty());
    }
}
#[test]
fn reward_is_distinct_and_short_lived() {
    for id in 0..12 {
        let mut f = fixtures::fixture(id).unwrap();
        for t in 0..=f.end {
            f.advance(t);
            if f.core.stats.rewards > 0 {
                let r = f.core.reward.as_ref().unwrap();
                assert_eq!(r.id, id);
                assert_eq!(r.until, t + 300);
                assert_eq!(f.core.reward_at, t + 1200);
                break;
            }
        }
    }
}
#[test]
fn no_gameplay_cap_at_128_pieces() {
    let mut c = Core::new(1, 1, 7);
    let mut f = frame();
    for i in 0..128 {
        add(
            &mut c,
            Kind::from_code(i % 5).unwrap(),
            3 + (i % 16) as i32,
            22 + (i / 16) as i32,
        );
    }
    f.fighters = vec![foe(2, 20, 20)];
    run(&mut c, &mut f, 0, None);
    assert_eq!(c.pieces.len(), 128);
    assert!(c.army_denominator() > 100);
    let snapshot = frame().state(&c);
    assert!(snapshot.contains("\"id\":127"));
    f.automatic = true;
    run(&mut c, &mut f, 1, None);
    assert!(c.last_plan_candidates <= 64);
}
#[test]
fn castling_swaps_squares_shields_and_preserves_clock() {
    let (mut c, mut f) = deployed(5);
    let id = c.slots[0].unwrap();
    let old = f.owner.cell();
    c.pieces.get_mut(&id).unwrap().cell = Cell::new(old.x + 2, old.y);
    c.reindex();
    let mut out = run(&mut c, &mut f, 61, Some(Action::Castle { piece: id }));
    for t in 62..=91 {
        out.extend(run(&mut c, &mut f, t, None));
    }
    assert_eq!(f.owner.cell(), Cell::new(old.x + 2, old.y));
    assert_eq!(c.pieces[&id].cell, old);
    assert_eq!(c.castle_at, 91 + 720);
    assert!(out.iter().any(|e| matches!(
        e,
        Cmd::Shield {
            target: None,
            amount: 90,
            ticks: 60
        }
    )));
    assert!(c.pending.is_none());
    run(&mut c, &mut f, 92, Some(Action::Castle { piece: id }));
    assert!(c.pending.is_none());
}
#[test]
fn newly_occupied_summon_destination_refunds_cooldown() {
    let mut c = Core::new(5, 1, 3);
    let mut f = frame();
    let at = f.owner.cell();
    run(
        &mut c,
        &mut f,
        0,
        Some(Action::Summon {
            kind: Kind::Pawn,
            to: Cell::new(at.x + 1, at.y),
            direction: 0,
        }),
    );
    for t in 1..30 {
        run(&mut c, &mut f, t, None);
    }
    f.owner.x += TILE;
    let out = run(&mut c, &mut f, 30, None);
    assert!(c.pieces.is_empty());
    assert_eq!(c.pawn_at, 0);
    assert!(out.iter().any(|e| matches!(e, Cmd::Failure { reason: 1 })));
}
#[test]
fn leaving_board_during_castling_cannot_teleport_or_grant_shield() {
    let (mut c, mut f) = deployed(5);
    let id = c.slots[0].unwrap();
    let at = f.owner.cell();
    c.pieces.get_mut(&id).unwrap().cell = Cell::new(at.x + 2, at.y);
    c.reindex();
    run(&mut c, &mut f, 61, Some(Action::Castle { piece: id }));
    for t in 62..91 {
        run(&mut c, &mut f, t, None);
    }
    f.owner.y -= TILE * 3;
    let out = run(&mut c, &mut f, 91, None);
    assert_eq!(c.castle_at, 0);
    assert!(!out
        .iter()
        .any(|c| matches!(c, Cmd::Shield { .. } | Cmd::OwnerPos { .. })));
}
#[test]
fn rebuilding_during_immortal_keeps_only_its_remaining_protection() {
    let (mut c, mut f) = deployed(7);
    c.reset_at = 61;
    c.reward = Some(Reward {
        id: 4,
        until: 200,
        charges: 4_294_967_295,
        piece: None,
    });
    run(&mut c, &mut f, 61, Some(Action::Deploy { heading: 1 }));
    let mut out = vec![];
    for t in 62..=121 {
        out.extend(run(&mut c, &mut f, t, None));
    }
    assert_eq!(
        out.iter()
            .filter(|c| matches!(
                c,
                Cmd::Buff {
                    kind: 4,
                    ticks: 79,
                    ..
                }
            ))
            .count(),
        7
    );
    assert_eq!(c.reward.as_ref().unwrap().until, 200);
}
#[test]
fn summoning_does_not_spend_piece_attack_or_pawn_tempo_rewards() {
    for reward in [0, 11] {
        let mut c = Core::new(1, 1, 3);
        let mut f = frame();
        let at = f.owner.cell();
        c.reward = Some(Reward {
            id: reward,
            until: 300,
            charges: 2,
            piece: None,
        });
        run(
            &mut c,
            &mut f,
            0,
            Some(Action::Summon {
                kind: if reward == 0 { Kind::Rook } else { Kind::Pawn },
                to: Cell::new(at.x + 1, at.y),
                direction: 0,
            }),
        );
        assert_eq!(c.pending.as_ref().unwrap().end, 30);
        for t in 1..=30 {
            run(&mut c, &mut f, t, None);
        }
        assert_eq!(c.pieces.len(), 1);
        assert_eq!(c.reward.as_ref().unwrap().charges, 2);
    }
}
#[test]
fn knight_coverage_requires_a_walkable_landing_and_pawns_cannot_check_through_walls() {
    let mut c = Core::new(1, 1, 3);
    let mut f = frame();
    let knight = add(&mut c, Kind::Knight, 18, 15);
    let pawn = add(&mut c, Kind::Pawn, 21, 15);
    f.walls = vec![
        [
            19 * TILE - 100,
            17 * TILE - 100,
            19 * TILE + 100,
            17 * TILE + 100,
        ],
        [
            21 * TILE + TILE / 2 - 100,
            15 * TILE + TILE / 2 - 100,
            21 * TILE + TILE / 2 + 100,
            15 * TILE + TILE / 2 + 100,
        ],
    ];
    let mut view = |id| f.inspection(&mut c, None, Some(id));
    let knight_cells = view(knight);
    let pawn_cells = view(pawn);
    assert!(!knight_cells.contains("[19,17]"));
    assert!(!pawn_cells.contains("[22,16]"));
    assert!(knight_cells.contains("[20,16]"));
}
#[test]
fn enemy_magnus_pieces_can_be_hit() {
    let mut c = Core::new(1, 1, 3);
    let mut f = frame();
    let id = add(&mut c, Kind::Rook, 18, 15);
    let mut enemy = foe(2, 20, 15);
    enemy.friendly_chess = true;
    enemy.champion = false;
    f.fighters.push(enemy);
    let out = moved(&mut c, &mut f, 0, id, Cell::new(22, 15));
    assert!(out
        .iter()
        .any(|e| matches!(e, Cmd::Damage { target: 2, .. })));
}
#[test]
fn en_passant_requires_continuous_visible_crossing() {
    let mut c = Core::new(1, 1, 3);
    let mut f = frame();
    let pawn = add(&mut c, Kind::Pawn, 18, 15);
    f.fighters.push(foe(2, 19, 16));
    run(&mut c, &mut f, 0, None);
    f.fighters[0].visible = false;
    run(&mut c, &mut f, 1, None);
    f.fighters[0].visible = true;
    f.fighters[0].x = 18 * TILE;
    run(&mut c, &mut f, 2, None);
    f.fighters[0].x = 17 * TILE;
    run(&mut c, &mut f, 3, None);
    // The observed centre-to-side crossing is valid; fog cannot bridge the first side-to-centre move.
    assert!(c.pieces[&pawn].intercept.is_some());
    c.pieces.get_mut(&pawn).unwrap().intercept = None;
    f.fighters[0].x = 19 * TILE;
    run(&mut c, &mut f, 4, None);
    assert!(
        c.pieces[&pawn].intercept.is_none(),
        "A two-cell blink is not a crossing"
    );
}
#[test]
fn teleports_do_not_advance_royal_walk() {
    let mut fx = fixtures::fixture(9).unwrap();
    for t in 0..=fx.end {
        if t == 1 {
            fx.frame.owner.x += TILE;
            fx.frame.owner.y += TILE;
        }
        fx.walk.clear();
        fx.advance(t);
    }
    assert_eq!(fx.core.stats.rewards, 0);
}

include!("magnus_v3_tests.rs");
