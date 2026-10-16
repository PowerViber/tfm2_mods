#[test]
fn ultimate_lock_applies_to_manual_automatic_and_commitment() {
    let mut c = Core::new(8, 1, 7);
    let mut f = frame();
    f.rank = 7;
    f.ult_unlocked = false;
    f.fighters = vec![foe(2, 18, 16)];
    run(&mut c, &mut f, 0, Some(Action::Deploy { heading: 0 }));
    assert!(c.pending.is_none());
    assert_eq!(c.reset_at, 0);
    f.automatic = true;
    for t in 1..90 {
        run(&mut c, &mut f, t, None);
    }
    assert!(c.board.is_none());
    assert!(
        !c.pieces.is_empty(),
        "field summons remain available before ultimate"
    );
    let mut c = Core::new(9, 1, 0);
    let mut f = frame();
    run(&mut c, &mut f, 0, Some(Action::Deploy { heading: 0 }));
    f.ult_unlocked = false;
    run(&mut c, &mut f, 60, None);
    assert!(c.board.is_none());
    assert_eq!(c.reset_at, 0);
    f.ult_unlocked = true;
    run(&mut c, &mut f, 61, Some(Action::Deploy { heading: 0 }));
    run(&mut c, &mut f, 121, None);
    assert!(c.board.is_some());
    assert_eq!(c.reset_at, 121 + 1440);
}
#[test]
fn pawn_can_step_in_all_cardinal_directions_but_not_immediately_return() {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut c = Core::new(1, 1, 0);
        let mut f = frame();
        let id = add(&mut c, Kind::Pawn, 18, 15);
        let out = moved(&mut c, &mut f, 0, id, Cell::new(18 + dx, 15 + dy));
        assert_eq!(c.pieces[&id].cell, Cell::new(18 + dx, 15 + dy));
        assert!(!out.iter().any(|c| matches!(c, Cmd::Damage { .. })));
        moved(&mut c, &mut f, 50, id, Cell::new(18, 15));
        assert_eq!(c.pieces[&id].cell, Cell::new(18 + dx, 15 + dy));
    }
}
#[test]
fn pawn_rectangle_revisits_cannot_farm_promotion() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Pawn, 18, 15);
    c.pieces.get_mut(&id).unwrap().qualified = true;
    for n in 0..20 {
        let to = [
            Cell::new(19, 15),
            Cell::new(19, 16),
            Cell::new(18, 16),
            Cell::new(18, 15),
        ][n % 4];
        moved(&mut c, &mut f, n * 50, id, to);
    }
    assert_eq!(c.pieces[&id].kind, Kind::Pawn);
    assert_eq!(c.pieces[&id].ranks, 1);
    assert_eq!(c.pieces[&id].cell, Cell::new(18, 15));
}
#[test]
fn pawn_travel_does_not_arm_checkmate_and_occupied_cardinal_squares_are_illegal() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Pawn, 18, 15);
    f.fighters = vec![foe(2, 19, 15)];
    moved(&mut c, &mut f, 0, id, Cell::new(19, 15));
    assert_eq!(c.pieces[&id].cell, Cell::new(18, 15));
    moved(&mut c, &mut f, 50, id, Cell::new(18, 16));
    assert!(!c.has_move);
}
#[test]
fn forced_pawn_yield_updates_previous_square_without_ranks() {
    let mut c = Core::new(1, 1, 0);
    let mut f = frame();
    let id = add(&mut c, Kind::Pawn, 16, 12);
    run(&mut c, &mut f, 0, None);
    run(&mut c, &mut f, 30, None);
    assert_eq!(c.pieces[&id].previous, Some(Cell::new(16, 12)));
    assert_eq!(c.pieces[&id].ranks, 0);
    let cell = c.pieces[&id].cell;
    f.owner.x = 17 * TILE;
    moved(&mut c, &mut f, 50, id, Cell::new(16, 12));
    assert_eq!(c.pieces[&id].cell, cell);
}
#[test]
fn trap_warning_holds_covering_pieces_instead_of_chasing_damage() {
    let (mut c, mut f) = trap_case(true);
    f.automatic = true;
    f.rank = 7;
    f.ult_unlocked = false;
    for t in 0..24 {
        run(&mut c, &mut f, t, None);
        assert!(
            c.pending.is_none(),
            "do not interrupt a viable trap warning"
        );
    }
    let out = run(&mut c, &mut f, 24, None);
    assert!(out.iter().any(|c| matches!(
        c,
        Cmd::Stun {
            target: 2,
            ticks: 75
        }
    )));
}
#[test]
fn planner_completes_escape_coverage_without_an_immediate_hit() {
    let (mut c, mut f) = trap_case(true);
    f.automatic = true;
    f.rank = 7;
    f.ult_unlocked = false;
    c.pawn_at = usize::MAX;
    c.develop_at = usize::MAX;
    let bishop = c.kind_ids(Kind::Bishop)[0];
    c.pieces.get_mut(&bishop).unwrap().cell = Cell::new(23, 21);
    c.reindex();
    run(&mut c, &mut f, 0, None);
    assert!(
        matches!(c.pending.as_ref().map(|p|p.action),Some(Action::Move{piece,..})if piece==bishop),
        "pending={:?}, plans={:?}",
        c.pending,
        c.plans
    );
    for t in 1..100 {
        run(&mut c, &mut f, t, None);
    }
    assert!(c.stats.mates >= 1, "planner must finish and hold the trap");
}
#[test]
fn planning_previews_are_bounded_real_continuations() {
    let mut c = Core::new(3, 1, 7);
    let mut f = frame();
    f.rank = 7;
    f.automatic = true;
    f.ult_unlocked = false;
    f.fighters = vec![foe(2, 21, 18)];
    for n in 0..16 {
        add(
            &mut c,
            [Kind::Rook, Kind::Bishop, Kind::Knight, Kind::Pawn][n % 4],
            10 + (n % 8) as i32,
            10 + (n / 8) as i32,
        );
    }
    run(&mut c, &mut f, 0, None);
    assert!(c.last_plan_candidates <= 64);
    assert!(c.plans.len() <= 3);
    assert!(!c.plans.is_empty());
    let wall = |x, y| f.blocked(x, y);
    let sight = |a, b, c, d| f.sight_clear(a, b, c, d);
    let input = Input {
            geometry: None,
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
        ult_unlocked: false,
        action: None,
    };
    for path in &c.plans {
        assert!(!path.is_empty() && path.len() <= 4);
        let mut view = c.clone();
        view.pending = None;
        for a in path {
            assert!(
                view.simulate_action(*a, &input).is_some(),
                "each step must use the preceding branch position"
            );
        }
    }
    let before = format!("{c:?}");
    let a = c.json(&input, &[]);
    let b = c.json(&input, &[]);
    assert_eq!(a, b);
    assert_eq!(
        format!("{c:?}"),
        before,
        "reading predictions cannot advance RNG or rules"
    );
}
#[test]
fn hidden_target_and_piece_capture_clear_predictions() {
    let mut c = Core::new(3, 1, 7);
    let mut f = frame();
    f.rank = 7;
    f.automatic = true;
    f.ult_unlocked = false;
    f.fighters = vec![foe(2, 21, 18)];
    add(&mut c, Kind::Rook, 18, 16);
    run(&mut c, &mut f, 0, None);
    assert!(!c.plans.is_empty());
    f.automatic = false;
    f.fighters[0].visible = false;
    run(&mut c, &mut f, 1, None);
    assert!(c.plans.is_empty());
}
#[test]
fn moving_the_bound_target_invalidates_the_displayed_continuation() {
    let mut c = Core::new(3, 1, 7);
    let mut f = frame();
    f.rank = 7;
    f.automatic = true;
    f.ult_unlocked = false;
    f.fighters = vec![foe(2, 21, 18)];
    add(&mut c, Kind::Rook, 18, 16);
    run(&mut c, &mut f, 0, None);
    assert!(!c.plans.is_empty());
    f.automatic = false;
    f.fighters[0].x += TILE;
    run(&mut c, &mut f, 1, None);
    assert!(c.plans.is_empty());
}

#[test]
fn a_new_wall_invalidates_a_later_preview_step_before_the_first_move_finishes(){
    let mut c=Core::new(3,1,7);let mut f=frame();f.ult_unlocked=false;
    let rook=add(&mut c,Kind::Rook,18,16);let knight=add(&mut c,Kind::Knight,20,16);
    c.plans=vec![vec![Action::Move{piece:rook,to:Cell::new(18,18)},Action::Move{piece:knight,to:Cell::new(21,18)}]];
    f.walls.push([21*TILE-100,18*TILE-100,21*TILE+100,18*TILE+100]);
    run(&mut c,&mut f,0,None);assert!(c.plans.is_empty());
    assert_eq!(c.pieces[&rook].cell,Cell::new(18,16));assert_eq!(c.pieces[&knight].cell,Cell::new(20,16));
}
#[test]
fn seventeen_badges_map_to_eleven_looks_and_never_change_manual_power() {
    assert_eq!(
        (0..7).map(|r| appearance(r, None)).collect::<Vec<_>>(),
        (0..7).collect::<Vec<_>>()
    );
    assert_eq!(appearance(7, Some(1)), 10);
    assert_eq!(appearance(7, Some(2)), 9);
    assert_eq!(appearance(7, Some(3)), 8);
    assert_eq!(appearance(7, Some(10)), 7);
    let mut commands = vec![];
    for top in 1..=10 {
        let mut c = Core::new(1, 1, 7);
        let mut f = frame();
        f.rank = 7;
        f.top_position = Some(top);
        let id = add(&mut c, Kind::Knight, 18, 15);
        f.fighters = vec![foe(2, 19, 17)];
        commands.push(moved(&mut c, &mut f, 0, id, Cell::new(19, 17)));
    }
    assert!(commands.windows(2).all(|p| p[0] == p[1]));
}
#[test]
fn transport_four_carries_unlock_and_podium_and_rejects_version_two() {
    let mut f = frame();
    f.ult_unlocked = false;
    f.rank = 7;
    f.top_position = Some(1);
    let mut values = f.values();
    let decoded = Frame::decode(&values).unwrap();
    assert!(!decoded.ult_unlocked);
    assert_eq!(decoded.top_position, Some(1));
    assert_eq!(decoded.values(), values);
    values[0] = 2;
    assert!(Frame::decode(&values).is_none());
}
