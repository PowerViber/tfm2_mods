use akane_lab::{akane_core::*,akane_data as d};
fn enemy(id:usize,x:i64,y:i64)->Unit{Unit{id,team:1,pos:Point::new(x,y),hp:1000,max_hp:1000,radius:3000,champion:true,visible:true,targetable:true,..Default::default()}}
fn frame(tick:u64,units:Vec<Unit>)->Frame{Frame{tick,owner:1,team:0,alive:true,pos:Point::new(400000,400000),hp:1600,max_hp:1600,attack:180,manual:true,ult_unlocked:true,units,..Default::default()}}
fn damage(out:&[Effect])->i64{out.iter().filter_map(|e|if let Effect::Damage{amount,..}=e{Some(*amount)}else{None}).sum()}
fn cast(c:&mut Core,f:&Frame,o:Order)->Vec<Effect>{let mut f=f.clone();f.order=Some(o);c.step(&f)}

#[test]fn longer_ranges_and_slower_movement_are_canonical(){
    assert_eq!((d::KUNAI_FLIP,d::KUNAI_FLIP_TICKS,d::KUNAI_RANGE,d::KUNAI_SPEED,d::KUNAI_DASH_SPEED),(30000,30,120000,3000,3000));
    assert_eq!((d::NIGHT_DASH_RANGE,d::NIGHT_DASH_SPEED),(240000,4000));
    let mut c=Core::default();let f=frame(0,vec![]);let out=cast(&mut c,&f,Order::Kunai(Point::new(500000,400000)));
    assert!(out.contains(&Effect::Glide{to:Point::new(370000,400000),ticks:30}));assert_eq!(damage(&out),0);assert_eq!(c.kunai_flights[0].end,Point::new(520000,400000));
}
#[test]fn projectile_marks_on_contact_only_and_snapshots_attack(){
    let mut c=Core::default();let mut f=frame(0,vec![enemy(2,500000,400000)]);
    assert_eq!(damage(&cast(&mut c,&f,Order::Kunai(Point::new(500000,400000)))),0);assert!(c.mark.is_none());
    f.attack=999;for tick in 1..31{f.tick=tick;assert_eq!(damage(&c.step(&f)),0);}
    f.tick=31;let out=c.step(&f);assert_eq!(damage(&out),138);assert_eq!(c.mark,Some((2,31+180)));assert!(c.kunai_flights.is_empty());
    f.tick=32;assert_eq!(damage(&c.step(&f)),0);
}
#[test]fn fixed_trajectory_can_be_dodged_and_expires_at_max_range(){
    let mut c=Core::default();let mut f=frame(0,vec![enemy(2,500000,400000)]);cast(&mut c,&f,Order::Kunai(Point::new(500000,400000)));
    let end=c.kunai_flights[0].end;f.units[0].pos.y+=30000;
    for tick in 1..=40{f.tick=tick;assert_eq!(damage(&c.step(&f)),0);if let Some(s)=c.kunai_flights.first(){assert_eq!(s.end,end);assert_eq!(s.pos.y,400000);}}
    assert!(c.kunai_flights.is_empty()&&c.mark.is_none());
}
#[test]fn kunai_pierces_minions_to_the_first_champion_towers_and_allies_are_ignored_and_walls_are_crossed(){
    let mut tower=enemy(8,415000,400000);tower.champion=false;tower.tower=true;
    let mut ally=enemy(9,412000,400000);ally.team=0;
    let mut near=enemy(2,440000,400000);near.champion=false;near.farmable=true;
    let mut f=frame(0,vec![tower,ally,near,enemy(3,470000,400000)]);f.terrain=vec![false;900];f.terrain[12*30+13]=true;
    let mut c=Core::default();cast(&mut c,&f,Order::Kunai(Point::new(510000,400000)));let mut hits=vec![];
    for tick in 1..=40{f.tick=tick;for e in c.step(&f){if let Effect::Damage{target,..}=e{hits.push(target);}}}
    assert_eq!(hits,vec![2,3]);assert_eq!(c.marked(40),Some(3));
}
#[test]fn native_dash_duration_matches_new_speeds_and_ultimate_boundary(){
    let mut c=Core::default();let mut f=frame(0,vec![enemy(2,640000,400000),enemy(3,640001,400000)]);cast(&mut c,&f,Order::Night);
    f.tick=1;cast(&mut c,&f,Order::NightDash(3));assert!(c.dash.is_none());
    f.tick=2;let out=cast(&mut c,&f,Order::NightDash(2));assert!(out.contains(&Effect::Glide{to:Point::new(633000,400000),ticks:58}));
    assert_eq!(c.dash.as_ref().unwrap().strike_at,60);
    let mut c=Core::default();c.mark=Some((2,180));let f=frame(0,vec![enemy(2,500000,400000)]);let out=cast(&mut c,&f,Order::KunaiRecast);
    assert!(out.contains(&Effect::Glide{to:Point::new(493000,400000),ticks:31}));
}
#[test]fn death_and_respawn_remove_flights_without_damage_and_owners_are_independent(){
    let mut a=Core::default();let b=Core::default();let mut f=frame(0,vec![enemy(2,510000,400000)]);cast(&mut a,&f,Order::Kunai(Point::new(510000,400000)));assert!(b.kunai_flights.is_empty());
    f.tick=20;f.alive=false;assert!(a.step(&f).is_empty()&&a.kunai_flights.is_empty());a.respawn(50);assert!(a.kunai_flights.is_empty());
}
#[test]fn terrain_clips_the_longer_retreat_and_map_edge_keeps_the_projectile_bounded(){
    let mut c=Core::default();let mut f=frame(0,vec![]);f.terrain=vec![false;900];f.terrain[12*30+11]=true;
    let out=cast(&mut c,&f,Order::Kunai(Point::new(500000,400000)));let to=out.iter().find_map(|e|if let Effect::Glide{to,..}=e{Some(*to)}else{None}).unwrap();assert!(to.x>370000&&!f.terrain_at(to));
    let mut c=Core::default();f.pos=Point::new(930000,400000);cast(&mut c,&f,Order::Kunai(Point::new(960000,400000)));assert!(c.kunai_flights[0].end.on_map());
}
