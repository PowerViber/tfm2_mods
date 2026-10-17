fn inspect_frame<T>(f: &Frame, inspect: impl FnOnce(&Input<'_>) -> T) -> T {
    let wall=|x,y|f.blocked(x,y);
    let sight=|a,b,c,d|f.sight_clear(a,b,c,d);
    inspect(&Input{geometry:None,tick:f.tick,owner:&f.owner,fighters:&f.fighters,wall:&wall,sight:&sight,
        captures:&[],hits:&[],piece_hp:&[],controlled:f.controlled,automatic:false,
        ult_unlocked:f.ult_unlocked,action:None})
}

#[test]
fn prediction_uses_visible_motion_and_stops_at_walls() {
    let c=Core::new(1,1,7);let mut f=frame();
    f.fighters=vec![foe(2,18,16),foe(3,19,17),foe(4,20,18)];
    for u in &mut f.fighters{u.vx=100;u.move_speed=80;}
    f.fighters[1].visible=false;f.fighters[2].team=0;
    let predicted=inspect_frame(&f,|i|c.forecast(i,30));
    assert_eq!(predicted[0].x,f.fighters[0].x+2400);
    assert_eq!(predicted[1].x,f.fighters[1].x);
    assert_eq!(predicted[2].x,f.fighters[2].x);
    f.walls.push([18*TILE+1000,16*TILE-100,18*TILE+2000,16*TILE+100]);
    let predicted=inspect_frame(&f,|i|c.forecast(i,30));
    assert!(predicted[0].x<18*TILE+1000);
}

#[test]
fn counterplay_discounts_a_net_the_king_can_leave_before_the_warning() {
    let mut c=Core::new(1,1,7);let mut f=frame();
    f.fighters=vec![foe(2,20,20)];c.field_target=Some(2);c.has_move=true;
    for x in 19..=21{add(&mut c,Kind::Rook,x,18);}c.reindex();
    f.fighters[0].move_speed=2000;
    let (raw,robust)=inspect_frame(&f,|i|(c.position_value(i),c.counterplay_value(i,30)));
    assert!(raw>=1_000_000,"the stationary position is a complete net");
    assert!(robust<900_000,"reachable escapes must remove the forced-mate score");
    f.fighters[0].move_speed=1;
    assert_eq!(inspect_frame(&f,|i|c.counterplay_value(i,30)),raw);
}

#[test]
fn vulnerable_covering_piece_is_an_enemy_response_not_a_guaranteed_mate() {
    let (mut c,mut f)=trap_case(true);c.reindex();
    let rook=c.pieces.values().find(|p|p.kind==Kind::Rook).unwrap().id;
    c.pieces.get_mut(&rook).unwrap().hp=1;
    f.fighters.push(foe(3,20,21));
    assert!(inspect_frame(&f,|i|c.position_value(i))>=900_000);
    assert!(inspect_frame(&f,|i|c.counterplay_value(i,30))<900_000);
}

#[test]
fn farming_without_a_champion_targets_the_wave_and_lands_an_attack() {
    let mut c=Core::new(10,1,7);let mut f=frame();f.rank=7;
    f.automatic=true;f.ult_unlocked=false;c.pawn_at=10_000;c.develop_at=10_000;
    let rook=add(&mut c,Kind::Rook,16,14);
    let mut minion=foe(2,18,12);minion.champion=false;f.fighters=vec![minion];
    let mut damage=false;
    for t in 0..240{damage|=run(&mut c,&mut f,t,None).iter().any(|c|matches!(c,Cmd::Damage{piece,target:2,..}if *piece==rook));}
    assert!(damage,"a lone wave must receive real commands, not summons beside Magnus forever");
    assert_eq!(c.plan_target,Some(2));
}

#[test]
fn an_empty_map_does_not_consume_the_clock_rearranging_pieces() {
    let mut c=Core::new(1,1,7);let mut f=frame();f.automatic=true;f.rank=7;
    add(&mut c,Kind::Bishop,18,15);
    for t in 0..100{run(&mut c,&mut f,t,None);}
    assert!(c.pending.is_none());assert_eq!(c.stats.casts,0);assert!(c.plans.is_empty());
}

#[test]
fn farming_respects_health_control_role_and_visible_champion_danger() {
    let c=Core::new(1,1,7);let mut f=frame();
    let mut minion=foe(2,18,12);minion.champion=false;f.fighters=vec![minion];
    let target=|f:&Frame|inspect_frame(f,|i|c.planning_target(i).map(|u|u.id));
    f.owner.hp=280;assert_eq!(target(&f),Some(2));
    f.owner.hp=279;assert_eq!(target(&f),None);
    f.owner.hp=800;f.controlled=true;assert_eq!(target(&f),None);
    f.controlled=false;f.fighters[0].farmable=false;assert_eq!(target(&f),None);
    f.fighters[0].farmable=true;
    let mut threat=foe(3,19,12);threat.targetable=false;f.fighters.push(threat);
    assert_eq!(target(&f),None,"an invulnerable nearby champion still blocks farming");
    f.fighters[1].visible=false;assert_eq!(target(&f),Some(2));
    f.fighters[1].visible=true;f.fighters[1].x=f.owner.x+110_001;
    assert_eq!(target(&f),Some(2));
}

#[test]
fn version_four_roundtrips_motion_and_rejects_old_frames() {
    let mut f=frame();f.owner.vx=300;f.owner.vy=-400;f.owner.move_speed=960;
    let mut values=f.values();assert_eq!(values[0],4);
    assert_eq!(Frame::decode(&values).unwrap().values(),values);
    values[0]=3;assert!(Frame::decode(&values).is_none());
}

#[test]
fn pawn_candidates_include_every_cardinal_move_except_the_immediate_return() {
    let mut c=Core::new(1,1,7);let f=frame();
    let pawn=add(&mut c,Kind::Pawn,18,15);
    c.pieces.get_mut(&pawn).unwrap().previous=Some(Cell::new(17,15));
    let moves=inspect_frame(&f,|i|c.destinations(pawn,i));
    for cell in [Cell::new(19,15),Cell::new(18,16),Cell::new(18,14)]{assert!(moves.contains(&cell));}
    assert!(!moves.contains(&Cell::new(17,15)));assert!(!moves.contains(&Cell::new(18,17)));
}

#[test]
fn unconfirmed_receipts_cannot_qualify_pawns_but_a_later_applied_receipt_can() {
    let mut c=Core::new(1,1,7);let mut f=frame();
    let pawn=add(&mut c,Kind::Pawn,18,15);f.fighters=vec![foe(2,19,16)];
    let key=(9,pawn,2);c.awaiting.insert(key,(120,false));
    for (t,status) in [ReceiptStatus::Pending,ReceiptStatus::Unknown,ReceiptStatus::Rejected].into_iter().enumerate(){
        f.tick=t;f.hits=vec![Hit{command:9,piece:pawn,target:2,amount:100,status}];
        f.step(&mut c);assert!(!c.pieces[&pawn].qualified);assert!(c.awaiting.contains_key(&key));
        assert_eq!(c.receipts.len(),1);
    }
    f.tick=3;f.hits[0].status=ReceiptStatus::Applied;f.step(&mut c);
    assert!(c.pieces[&pawn].qualified);assert!(!c.awaiting.contains_key(&key));
    assert_eq!(Frame::decode(&f.values()).unwrap().hits[0].status,ReceiptStatus::Applied);
}
