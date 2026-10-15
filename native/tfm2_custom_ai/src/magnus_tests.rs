use super::*;
#[path = "magnus_fixtures.rs"]
mod fixtures;
fn frame(
    c: &mut Core,
    tick: usize,
    me: &Fighter,
    fs: &[Fighter],
    action: Option<Action>,
    caps: &[Capture],
    wall: &dyn Fn(i64, i64) -> bool,
    controlled: bool,
) -> Vec<Cmd> {
    c.step(&Input {
        tick,
        owner: me,
        fighters: fs,
        action,
        captures: caps,
        wall,
        controlled,
        automatic: false,
        piece_hp: [None; 7],
    })
}
fn replay(id: usize, miss: usize) -> Core {
    let mut f = fixtures::historical(id);
    if miss == 2 {
        f.fighters[0].visible = false;
    }
    if miss == 3 {
        for m in f.moves.iter_mut().skip(1) {
            m.0 += 500;
        }
        for c in &mut f.captures {
            c.0 += 500;
        }
    }
    let mut out = vec![];
    for tick in 0..=if miss == 3 { 750 } else { f.end } {
        for &(t, id, c) in &f.relocate {
            if t == tick {
                let b = f.core.board.unwrap();
                let (x, y) = b.world(c);
                if let Some(e) = f.fighters.iter_mut().find(|e| e.id == id) {
                    e.x = x;
                    e.y = y;
                }
            }
        }
        let action = f.moves.iter().find(|m| m.0 == tick).map(|m| Action::Move {
            piece: m.1,
            to: m.2,
        });
        let caps = f
            .captures
            .iter()
            .filter(|c| c.0 == tick)
            .map(|c| Capture {
                piece: c.1,
                by: Some(5),
                combat: miss != 1,
            })
            .collect::<Vec<_>>();
        let wall = |x, y| {
            f.walls
                .iter()
                .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
        };
        out.extend(frame(
            &mut f.core,
            tick,
            &f.me,
            &f.fighters,
            action,
            &caps,
            &wall,
            false,
        ));
    }
    if miss == 0 {
        assert!(
            f.core.challenge.as_ref().unwrap().done,
            "recipe {id}: {:?}, stats {:?}, labels {:?}",
            f.core.challenge,
            f.core.stats,
            out.iter()
                .filter(|c| matches!(c, Cmd::Label { .. }))
                .collect::<Vec<_>>()
        );
        assert_eq!(f.core.stats.rewards, 1);
    }
    f.core
}
#[test]
fn every_historical_sequence_has_real_commands() {
    for id in 0..6 {
        replay(id, 0);
    }
}
#[test]
fn history_rejects_hidden_kings_and_cleanup() {
    for id in 0..6 {
        assert_eq!(replay(id, 2).stats.rewards, 0);
        if id != 1 {
            assert_eq!(replay(id, 1).stats.rewards, 0)
        }
    }
}
#[test]
fn history_no_duplicate_rewards() {
    for id in 0..6 {
        let mut c = replay(id, 0);
        let me = fixtures::owner();
        let mut out = vec![];
        let input = Input {
            tick: 230,
            owner: &me,
            fighters: &[],
            wall: &|_, _| false,
            captures: &[],
            piece_hp: [None; 7],
            controlled: false,
            automatic: false,
            action: None,
        };
        for _ in 0..4 {
            c.challenge_event(&input, 3, 0, None, &mut out)
        }
        assert_eq!(c.stats.rewards, 1);
    }
}
#[test]
fn reproducible_roster_and_colours() {
    for seed in 0..96 {
        let mut a = Core::new(seed, 1, 0);
        let mut b = a.clone();
        let me = fixtures::owner();
        for c in [&mut a, &mut b] {
            frame(
                c,
                0,
                &me,
                &[],
                Some(Action::Deploy {
                    heading: (seed % 4) as u8,
                }),
                &[],
                &|_, _| false,
                false,
            );
            frame(c, 60, &me, &[], None, &[], &|_, _| false, false);
        }
        assert_eq!(a.json(&[]), b.json(&[]));
        assert_eq!(a.pieces[0].cell, Cell::new(3, 0));
        assert_eq!(a.pieces[6].cell, Cell::new(5, 0));
        assert_ne!(a.pieces[2].cell.x % 2, a.pieces[3].cell.x % 2);
        let cells = a.pieces.iter().map(|p| p.cell).collect::<BTreeSet<_>>();
        assert_eq!(cells.len(), 7);
        assert!(!cells.contains(&Cell::new(4, 0)));
    }
}
#[test]
fn movement_blocks_sliders_and_landing_but_not_knight_jumps() {
    let mut c = fixtures::historical(1).core;
    c.pieces[0].cell = Cell::new(0, 2);
    c.pieces[4].cell = Cell::new(0, 3);
    assert!(!c.legal(0, Cell::new(0, 4), Cell::new(4, 0), &|_, _| false));
    assert!(c.legal(4, Cell::new(2, 4), Cell::new(4, 0), &|_, _| false));
    let b = c.board.unwrap();
    let (x, y) = b.world(Cell::new(2, 4));
    assert!(!c.legal(4, Cell::new(2, 4), Cell::new(4, 0), &|a, z| a == x
        && z == y));
    assert!(!c.legal(0, Cell::new(1, 3), Cell::new(4, 0), &|_, _| false));
}
#[test]
fn command_hits_each_enemy_once_and_stationary_never_hits() {
    let mut f = fixtures::historical(5);
    let mut hits = vec![];
    for t in 0..90 {
        hits.extend(frame(
            &mut f.core,
            t,
            &f.me,
            &f.fighters,
            if t == 0 {
                Some(Action::Move {
                    piece: 0,
                    to: Cell::new(4, 2),
                })
            } else {
                None
            },
            &[],
            &|_, _| false,
            false,
        ));
    }
    assert_eq!(
        hits.iter()
            .filter(|c| matches!(c, Cmd::Damage { target: 4, .. }))
            .count(),
        1
    );
}
#[test]
fn silence_interrupts_warning_but_not_committed_movement() {
    for interrupt in [10, 20] {
        let mut f = fixtures::historical(5);
        for t in 0..35 {
            frame(
                &mut f.core,
                t,
                &f.me,
                &f.fighters,
                if t == 0 {
                    Some(Action::Move {
                        piece: 0,
                        to: Cell::new(4, 2),
                    })
                } else {
                    None
                },
                &[],
                &|_, _| false,
                t >= interrupt,
            );
        }
        assert_eq!(
            f.core.pieces[0].cell,
            if interrupt == 10 {
                Cell::new(0, 2)
            } else {
                Cell::new(4, 2)
            }
        );
    }
}
#[test]
fn manual_power_is_identical_at_all_ranks() {
    let mut results = vec![];
    for rank in 0..8 {
        let mut f = fixtures::historical(5);
        f.core.rank = rank;
        let mut out = vec![];
        for t in 0..31 {
            out.extend(frame(
                &mut f.core,
                t,
                &f.me,
                &f.fighters,
                if t == 0 {
                    Some(Action::Move {
                        piece: 0,
                        to: Cell::new(4, 2),
                    })
                } else {
                    None
                },
                &[],
                &|_, _| false,
                false,
            ));
        }
        results.push(out);
    }
    for r in &results[1..] {
        assert_eq!(r, &results[0]);
    }
}
#[test]
fn reset_restores_only_adjacent_rook_and_queen_preserving_cooldowns() {
    let me = fixtures::owner();
    let mut c = Core::new(77, 1, 3);
    frame(
        &mut c,
        0,
        &me,
        &[],
        Some(Action::Deploy { heading: 0 }),
        &[],
        &|_, _| false,
        false,
    );
    frame(&mut c, 60, &me, &[], None, &[], &|_, _| false, false);
    for i in 0..7 {
        c.pieces[i].hp = 0;
        c.pieces[i].ready_at = 2200;
    }
    c.mate_at = 1800;
    c.reward_at = 1900;
    c.castle_at = 2000;
    frame(
        &mut c,
        1020,
        &me,
        &[],
        Some(Action::Deploy { heading: 1 }),
        &[],
        &|_, _| false,
        false,
    );
    frame(&mut c, 1080, &me, &[], None, &[], &|_, _| false, false);
    assert!(c.pieces[0].hp > 0 && c.pieces[6].hp > 0);
    for i in 1..6 {
        assert_eq!(c.pieces[i].hp, 0);
        assert_eq!(c.pieces[i].ready_at, 2200);
    }
    assert_eq!((c.mate_at, c.reward_at, c.castle_at), (1800, 1900, 2000));
}
#[test]
fn independent_clones_and_owners_and_death() {
    let mut f = fixtures::historical(0);
    f.core.mate_at = 1200;
    f.core.reward_at = 1300;
    let snapshot = f.core.json(&[]);
    let mut clone = f.core.clone();
    clone.pieces[0].hp = 1;
    assert_eq!(f.core.json(&[]), snapshot);
    let out = f.core.on_dead();
    assert_eq!(
        out.iter()
            .filter(|c| matches!(c, Cmd::Despawn { .. }))
            .count(),
        7
    );
    assert_eq!((f.core.mate_at, f.core.reward_at), (1200, 1300));
    assert_eq!(f.core.stats.rewards, 0);
    assert!(f.core.board.is_none());
    assert_ne!(Core::new(12, 1, 0).json(&[]), Core::new(12, 2, 0).json(&[]));
}
#[test]
fn clock_and_brilliant_boundaries() {
    let mut c = Core::new(1, 1, 0);
    assert_eq!(c.duration(Kind::Knight, 0), 30);
    assert_eq!(c.duration(Kind::Queen, 0), 45);
    c.brilliant_until = 300;
    assert_eq!(c.duration(Kind::Rook, 299), 24);
    assert_eq!(c.duration(Kind::Queen, 299), 36);
    assert_eq!(c.duration(Kind::Queen, 300), 45);
}
#[test]
fn history_window_expires() {
    for id in 0..6 {
        assert_eq!(replay(id, 3).stats.rewards, 0);
    }
}
#[test]
fn checkmate_stuns_enemies_only_and_obeys_warning() {
    let mut f = fixtures::historical(2);
    f.core.has_move = true;
    f.core.pieces[2].cell = Cell::new(4, 5);
    let b = f.core.board.unwrap();
    let mut ally = fixtures::enemy(b, 8, Cell::new(3, 3));
    ally.team = 0;
    f.fighters.push(ally);
    let mut minion = fixtures::enemy(b, 9, Cell::new(3, 3));
    minion.champion = false;
    f.fighters.push(minion);
    let wall = |x, y| {
        f.walls
            .iter()
            .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
    };
    assert!(
        !frame(&mut f.core, 0, &f.me, &f.fighters, None, &[], &wall, false)
            .iter()
            .any(|c| matches!(c, Cmd::Stun { .. }))
    );
    assert!(
        frame(&mut f.core, 23, &f.me, &f.fighters, None, &[], &wall, false)
            .iter()
            .all(|c| !matches!(c, Cmd::Stun { .. }))
    );
    let out = frame(&mut f.core, 24, &f.me, &f.fighters, None, &[], &wall, false);
    assert!(
        out.contains(&Cmd::Stun {
            target: 2,
            ticks: 75
        }) && out.contains(&Cmd::Stun {
            target: 9,
            ticks: 75
        })
    );
    assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { target: 8, .. })));
    assert_eq!(f.core.mate_at, 1224);
    assert!(frame(
        &mut f.core,
        1300,
        &f.me,
        &f.fighters,
        None,
        &[],
        &wall,
        false
    )
    .iter()
    .all(|c| !matches!(c, Cmd::Stun { .. })));
}
#[test]
fn safe_exits_and_broken_coverage_cancel_warning() {
    for broken in [false, true] {
        let mut f = fixtures::historical(2);
        f.core.has_move = true;
        f.core.pieces[2].cell = Cell::new(4, 5);
        let walls = f.walls.clone();
        let wall = |x, y| {
            walls
                .iter()
                .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
        };
        frame(&mut f.core, 0, &f.me, &f.fighters, None, &[], &wall, false);
        assert_eq!(f.core.mate_warning, Some(0));
        if broken {
            f.core.pieces[2].hp = 0;
        } else {
            let p = f.core.board.unwrap().world(Cell::new(7, 7));
            f.fighters[0].x = p.0;
            f.fighters[0].y = p.1;
        }
        let out = frame(&mut f.core, 24, &f.me, &f.fighters, None, &[], &wall, false);
        assert_eq!(f.core.mate_warning, None);
        assert!(!out.iter().any(|c| matches!(c, Cmd::Stun { .. })));
    }
}
#[test]
fn board_edge_is_an_escape_even_when_board_is_covered() {
    let mut f = fixtures::historical(2);
    f.core.has_move = true;
    f.core.pieces[0].cell = Cell::new(3, 0);
    f.core.pieces[2].cell = Cell::new(4, 1);
    f.fighters[0] = fixtures::enemy(f.core.board.unwrap(), 2, Cell::new(3, 7));
    for t in [0, 24, 48] {
        frame(
            &mut f.core,
            t,
            &f.me,
            &f.fighters,
            None,
            &[],
            &|_, _| false,
            false,
        );
    }
    assert_eq!(f.core.stats.mates, 0);
}
#[test]
fn castle_revalidates_the_owner_position_and_path() {
    for moved in [false, true] {
        let mut c = Core::new(3, 1, 0);
        let mut me = fixtures::owner();
        frame(
            &mut c,
            0,
            &me,
            &[],
            Some(Action::Deploy { heading: 0 }),
            &[],
            &|_, _| false,
            false,
        );
        frame(&mut c, 60, &me, &[], None, &[], &|_, _| false, false);
        frame(
            &mut c,
            61,
            &me,
            &[],
            Some(Action::Castle { piece: 0 }),
            &[],
            &|_, _| false,
            false,
        );
        assert!(c.pending.is_some());
        if moved {
            me.x += TILE;
        }
        let out = frame(&mut c, 79, &me, &[], None, &[], &|_, _| false, false);
        assert_eq!(
            out.iter().any(|c| matches!(
                c,
                Cmd::Shield {
                    amount: 90,
                    ticks: 60
                }
            )),
            !moved
        );
        if !moved {
            assert_eq!(c.castle_at, 799);
        }
    }
}
#[test]
fn invalid_commit_releases_command_and_reset_cancels_damage() {
    let mut f = fixtures::historical(5);
    frame(
        &mut f.core,
        0,
        &f.me,
        &f.fighters,
        Some(Action::Move {
            piece: 0,
            to: Cell::new(4, 2),
        }),
        &[],
        &|_, _| false,
        false,
    );
    let p = f.core.board.unwrap().world(Cell::new(1, 2));
    frame(
        &mut f.core,
        18,
        &f.me,
        &f.fighters,
        None,
        &[],
        &|x, y| x == p.0 && y == p.1,
        false,
    );
    assert!(f.core.pending.is_none());
    frame(
        &mut f.core,
        19,
        &f.me,
        &f.fighters,
        Some(Action::Move {
            piece: 0,
            to: Cell::new(4, 2),
        }),
        &[],
        &|_, _| false,
        false,
    );
    frame(
        &mut f.core,
        20,
        &f.me,
        &f.fighters,
        Some(Action::Dismiss),
        &[],
        &|_, _| false,
        false,
    );
    assert!(!frame(
        &mut f.core,
        49,
        &f.me,
        &f.fighters,
        None,
        &[],
        &|_, _| false,
        false
    )
    .iter()
    .any(|c| matches!(c, Cmd::Damage { .. })));
}
#[test]
fn reward_cooldown_is_not_bypassed_by_another_completion() {
    let mut f = fixtures::historical(0);
    f.core.reward_at = 1200;
    for t in 0..=f.end {
        let act = f.moves.iter().find(|m| m.0 == t).map(|m| Action::Move {
            piece: m.1,
            to: m.2,
        });
        let caps = f
            .captures
            .iter()
            .filter(|c| c.0 == t)
            .map(|c| Capture {
                piece: c.1,
                by: None,
                combat: true,
            })
            .collect::<Vec<_>>();
        frame(
            &mut f.core,
            t,
            &f.me,
            &f.fighters,
            act,
            &caps,
            &|_, _| false,
            false,
        );
    }
    assert!(f.core.challenge.unwrap().done);
    assert_eq!(f.core.stats.rewards, 0);
    assert_eq!(f.core.brilliant_until, 0);
}
#[test]
fn projections_keep_roles_and_do_not_use_hidden_information() {
    let mut c = Core::new(10, 1, 0);
    let me = fixtures::owner();
    let b = Board {
        x: me.x,
        y: me.y,
        heading: 0,
        serial: 0,
    };
    let mut fs = vec![
        fixtures::enemy(b, 9, Cell::new(4, 2)),
        fixtures::enemy(b, 2, Cell::new(4, 2)),
        fixtures::enemy(b, 3, Cell::new(6, 6)),
        fixtures::enemy(b, 4, Cell::new(2, 5)),
        fixtures::enemy(b, 5, Cell::new(1, 5)),
    ];
    fs[2].attack = 500;
    fs[3].tank = true;
    fs[3].ranged = true;
    fs[4].visible = false;
    frame(
        &mut c,
        0,
        &me,
        &fs,
        Some(Action::Deploy { heading: 0 }),
        &[],
        &|_, _| false,
        false,
    );
    frame(&mut c, 60, &me, &fs, None, &[], &|_, _| false, false);
    assert_eq!(c.projected(Role::King), Some(2));
    assert_eq!(c.projected(Role::Queen), Some(3));
    assert_eq!(
        c.projections.iter().find(|p| p.id == 4).unwrap().role,
        Role::Rook
    );
    assert!(!c.projections.iter().any(|p| p.id == 5));
    fs[4].visible = true;
    fs[4].attack = 9999;
    frame(&mut c, 61, &me, &fs, None, &[], &|_, _| false, false);
    assert_eq!(c.projected(Role::Queen), Some(3));
    assert_eq!(
        c.projections.iter().find(|p| p.id == 5).unwrap().role,
        Role::Knight
    );
    fs[1].hp = 0;
    frame(&mut c, 62, &me, &fs, None, &[], &|_, _| false, false);
    assert_eq!(c.projected(Role::King), Some(2));
    assert!(c.challenge.unwrap().failed);
}
#[test]
fn recovery_uses_a_legal_alternative_without_forgiving_its_timer() {
    let mut f = fixtures::historical(1);
    let b = f.core.board.unwrap();
    f.core.pieces[4].home = Cell::new(4, 0);
    frame(
        &mut f.core,
        1,
        &f.me,
        &f.fighters,
        None,
        &[Capture {
            piece: 4,
            by: Some(2),
            combat: true,
        }],
        &|_, _| false,
        false,
    );
    assert_eq!(f.core.pieces[4].ready_at, 481);
    assert!(!frame(
        &mut f.core,
        480,
        &f.me,
        &f.fighters,
        None,
        &[],
        &|_, _| false,
        false
    )
    .iter()
    .any(|c| matches!(c, Cmd::Spawn { piece: 4, .. })));
    let out = frame(
        &mut f.core,
        481,
        &f.me,
        &f.fighters,
        None,
        &[],
        &|_, _| false,
        false,
    );
    assert!(out.iter().any(|c| matches!(
        c,
        Cmd::Spawn {
            piece: 4,
            hp: 200,
            ..
        }
    )));
    assert_ne!(f.core.pieces[4].cell, b.cell(f.me.x, f.me.y));
}
#[test]
fn friendly_or_unknown_capture_ids_do_not_award_sacrifice() {
    for by in [1, 99999] {
        let mut f = fixtures::historical(0);
        for t in 0..46 {
            frame(
                &mut f.core,
                t,
                &f.me,
                &f.fighters,
                if t == 0 {
                    Some(Action::Move {
                        piece: 6,
                        to: Cell::new(3, 6),
                    })
                } else {
                    None
                },
                &[],
                &|_, _| false,
                false,
            );
        }
        let out = frame(
            &mut f.core,
            46,
            &f.me,
            &f.fighters,
            None,
            &[Capture {
                piece: 6,
                by: Some(by),
                combat: true,
            }],
            &|_, _| false,
            false,
        );
        assert!(!out.contains(&Cmd::Label { code: 6 }));
        assert_eq!(f.core.stats.rewards, 0);
        assert!(f.core.challenge.unwrap().failed);
    }
}
#[test]
fn fixtures_respect_bishop_colours_and_catalogue() {
    assert_eq!(catalogue_count(), 6);
    for id in 0..6 {
        let f = fixtures::historical(id);
        if f.core.pieces[2].hp > 0 && f.core.pieces[3].hp > 0 {
            let a = f.core.pieces[2].cell;
            let b = f.core.pieces[3].cell;
            assert_ne!((a.x + a.y) % 2, (b.x + b.y) % 2);
        }
        assert!(!GAMES[id].source.is_empty());
        assert!(!GAMES[id].clue.is_empty());
    }
}
#[test]
fn another_friendly_magnus_army_blocks_but_keeps_its_ownership() {
    let mut f = fixtures::historical(5);
    let b = f.core.board.unwrap();
    let mut ally = fixtures::enemy(b, 80, Cell::new(2, 2));
    ally.team = 0;
    ally.champion = false;
    ally.friendly_chess = true;
    f.fighters.push(ally);
    frame(
        &mut f.core,
        0,
        &f.me,
        &f.fighters,
        None,
        &[],
        &|_, _| false,
        false,
    );
    assert!(!f
        .core
        .legal(0, Cell::new(4, 2), Cell::new(4, 0), &|_, _| false));
    let mask = f.core.attacks(0, &|_, _| false);
    assert_ne!(mask & Cell::new(2, 2).bit(), 0);
    assert_eq!(mask & Cell::new(3, 2).bit(), 0);
    assert!(f.core.on_dead().iter().all(|c| !matches!(
        c,
        Cmd::Stun { target: 80, .. } | Cmd::Damage { target: 80, .. }
    )));
}
#[test]
fn thin_walls_and_cached_geometry_agree() {
    let f = fixtures::historical(5);
    let mut c = f.core;
    let b = c.board.unwrap();
    let a = b.world(Cell::new(0, 2));
    let wall = |x:i64, y:i64| x >= a.0 + 2500 && x <= a.0 + 4500 && (y - a.1).abs() < 4000;
    assert!(!c.legal(0, Cell::new(1, 2), Cell::new(4, 0), &wall));
    let masks = c.masks(&wall);
    c.cache_geometry(&wall);
    assert_eq!(masks, c.masks(&wall));
    assert!(!c.legal(0, Cell::new(1, 2), Cell::new(4, 0), &wall));
}
#[test]
fn board_appears_on_success_at_the_geometric_centre() {
    let me = fixtures::owner();
    let mut c = Core::new(1, 1, 0);
    frame(
        &mut c,
        0,
        &me,
        &[],
        Some(Action::Deploy { heading: 1 }),
        &[],
        &|_, _| false,
        false,
    );
    let out = frame(&mut c, 60, &me, &[], None, &[], &|_, _| false, false);
    let b = c.board.unwrap();
    let a = b.world(Cell::new(0, 0));
    let z = b.world(Cell::new(7, 7));
    assert!(out
        .iter()
        .any(|e| matches!(e,Cmd::Fx{tag:"board1",x,y,..}if *x==(a.0+z.0)/2&&*y==(a.1+z.1)/2)));
}
#[test]
fn mate_cooldown_boundary_requires_a_new_full_warning() {
    let mut f = fixtures::historical(2);
    f.core.has_move = true;
    f.core.pieces[2].cell = Cell::new(4, 5);
    f.core.mate_at = 1200;
    let wall = |x, y| {
        f.walls
            .iter()
            .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
    };
    frame(
        &mut f.core,
        1199,
        &f.me,
        &f.fighters,
        None,
        &[],
        &wall,
        false,
    );
    assert_eq!(f.core.mate_warning, None);
    frame(
        &mut f.core,
        1200,
        &f.me,
        &f.fighters,
        None,
        &[],
        &wall,
        false,
    );
    assert_eq!(f.core.mate_warning, Some(1200));
    frame(
        &mut f.core,
        1223,
        &f.me,
        &f.fighters,
        None,
        &[],
        &wall,
        false,
    );
    assert_eq!(f.core.stats.mates, 0);
    frame(
        &mut f.core,
        1224,
        &f.me,
        &f.fighters,
        None,
        &[],
        &wall,
        false,
    );
    assert_eq!(f.core.stats.mates, 1);
}
