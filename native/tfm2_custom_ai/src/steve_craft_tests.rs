use crate::steve_craft_core::*;
use crate::steve_craft_geometry::*;
#[test]
fn rod_rechecks_charged_reach_and_cannot_grapple_empty_air(){
 let mut u=[unit(0,0,(480000,480000)),unit(1,1,(580000,480000))];let mut c=Core::default();c.step(&input(0,&u,Some(recipe(8,u[1].pos(),Some(1)))));let end=c.pending.as_ref().unwrap().end;u[1].x=700000;c.step(&input(end,&u,None));assert!(c.shots.is_empty());assert_eq!(c.reason,"Rod left charged reach");assert_eq!(c.available(end),2);assert!(c.cooldowns[8]>end);
 let mut c=Core::default();c.step(&input(0,&u,Some(recipe(8,(520000,480000),None))));assert_eq!(c.available(0),3);assert!(c.pending.is_none());assert_eq!(c.reason,"Grapple needs terrain or owned TNT");
 u[1].x=580000;let mut c=Core::default();resolved(&mut c,&u,recipe(8,u[1].pos(),Some(1)));assert_eq!(c.shots.len(),1,"{}",c.reason);
}
#[test]
fn external_wall_blocks_heavy_landing_and_short_boat_but_not_pearl_travel(){
 let u=[unit(0,0,(480000,480000))];let barriers=[((510000,450000),(510000,530000))];
 let mut c=Core::default();resolved(&mut c,&u,Action::Prepare(6));let t=c.last_tick.unwrap()+1;let mut i=input(t,&u,Some(Action::Deploy((510000,480000))));i.barriers=&barriers;c.step(&i);assert!(c.prepared.is_some());assert!(c.objects.is_empty());
 for r in [9,14]{let mut c=Core::default();let mut i=input(0,&u,Some(recipe(r,(540000,480000),None)));i.barriers=&barriers;c.step(&i);let mut moved=false;for t in 1..65{let mut i=input(t,&u,None);i.barriers=&barriers;let out=c.step(&i);moved|=out.iter().any(|v|matches!(v,Cmd::Move(0,p)if p.0>510000));}assert_eq!(moved,r==9,"recipe {r}");}
}
fn unit(id: usize, team: usize, p: Point) -> Unit {
    Unit {
        id,
        team,
        x: p.0,
        y: p.1,
        hp: 1150,
        max_hp: 1150,
        ad: 62,
        shield: 0,
        visible: true,
        targetable: true,
        champion: true,
        farmable: false,
        immune: false,
        cc_immune: false,
        immovable: false,
        epic:false,totem_locked:false,gifted_armour_until:0,
        tower: false,
        gadget: false,
        vx: 0,
        vy: 0,
        swing: false,
    }
}

#[test]
fn creative_every_entry_resolves_and_exclusive_support_rejects_caster(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(525000,480000)),unit(2,0,(500000,505000))];
 for r in 15..45u8{
  let mut c=Core::default();c.form=Form::Herobrine;
  let mut target=None;let mut p=(520000,500000);
  if matches!(r,23..=25|28..=32|40..=41){target=Some(2);p=u[2].pos();}
  if matches!(r,31..=32){resolved(&mut c,&u,recipe(28,u[2].pos(),Some(2)));}
  if r==42{resolved(&mut c,&u,recipe(2,(520000,510000),None));ack(&mut c);target=Some(c.objects[0].id as usize);p=(540000,510000);}
  if r>=43{target=Some(1);p=u[1].pos();}if r==39{p=(540000,470000);}
  let out=resolved(&mut c,&u,recipe(r,p,target));
  assert_eq!(c.metrics.casts[r as usize],1,"{r}: {}",c.reason);
  assert!(!out.is_empty(),"{r} emitted nothing");
 }
 for r in [23,24,25,28,29,30,31,32,40,41]{
  let mut c=Core::default();c.form=Form::Herobrine;
  c.step(&input(0,&u,Some(recipe(r,u[0].pos(),Some(0)))));
  assert!(c.pending.is_none(),"self support {r} accepted");assert_eq!(c.available(0),4);
 }
}
#[test]
fn creative_form_and_charge_slots_survive_switches_without_refill(){
 let u=[unit(0,0,(480000,480000))];let mut c=Core::default();
 assert_eq!(c.available(0),3);resolved(&mut c,&u,Action::Stance);let t=c.last_tick.unwrap();assert_eq!(c.available(t),4);
 c.charges=std::array::from_fn(|_|Charge::Ready(t+300));let mut i=input(t+1081,&u,Some(Action::Stance));c.step(&i);i.tick+=24;i.action=None;c.step(&i);
 assert_eq!(c.form,Form::Steve);assert_eq!(c.available(i.tick),3);assert!(c.cooldowns.iter().all(|t|*t==0));
 let mut i=input(i.tick+1,&u,None);i.me=&u[0];let hp=u[0].hp;c.step(&i);assert_eq!(u[0].hp,hp);
}
#[test]
fn creative_shared_cooldowns_and_failed_allocation_restore_the_family(){
 let u=[unit(0,0,(480000,480000)),unit(2,0,(500000,510000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(15,(520000,480000),None));let id=c.objects[0].id;let t=c.last_tick.unwrap()+1;
 c.step(&input(t,&u,Some(recipe(19,(530000,500000),None))));assert!(c.pending.is_none());assert!(c.cooldowns[15]>t);
 c.allocated(id,false);assert_eq!(c.cooldowns[15],0);assert_eq!(c.available(t),4);
}
#[test]
fn creative_hp_is_snapshotted_and_epic_component_is_capped(){
 let mut u=[unit(0,0,(480000,480000)),unit(1,1,(525000,480000))];u[0].max_hp=10000;u[1].epic=true;
 let mut c=Core::default();c.step(&input(0,&u,Some(recipe(3,u[1].pos(),None))));u[0].max_hp=1000;
 let mut out=vec![];for t in 1..130{out.extend(c.step(&input(t,&u,None)));if !c.objects.is_empty(){ack(&mut c);}}
 assert!(out.iter().any(|c|matches!(c,Cmd::Damage{target:1,ad:260,..})),"{out:?}");
}
#[test]
fn creative_totem_is_single_use_and_shared_lock_survives_death(){
 let mut u=[unit(0,0,(480000,480000)),unit(2,0,(500000,480000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(41,u[1].pos(),Some(2)));u[1].hp=1;let t=c.last_tick.unwrap()+1;
 let out=c.step(&input(t,&u,None));assert_eq!(out.iter().filter(|c|matches!(c,Cmd::Shield(2,..))).count(),1);
 assert!(!c.step(&input(t+1,&u,None)).iter().any(|c|matches!(c,Cmd::Shield(..))));
 c.died(t+2);c.form=Form::Herobrine;c.cooldowns.fill(0);c.charges=std::array::from_fn(|_|Charge::Ready(0));
 c.step(&input(t+2,&u,Some(recipe(41,u[1].pos(),Some(2)))));assert!(c.pending.is_none());
 let mut other=Core::default();other.form=Form::Herobrine;u[1].totem_locked=true;
 other.step(&input(t+3,&u,Some(recipe(41,u[1].pos(),Some(2)))));assert!(other.pending.is_none());
}
#[test]
fn creative_redstone_rejects_cycles_and_destroyed_paths_cancel_paid_payload(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(555000,480000))];let mut c=Core::default();c.form=Form::Herobrine;
 for(r,p)in [(35,(500000,480000)),(36,(522000,480000)),(34,(544000,480000))]{resolved(&mut c,&u,recipe(r,p,None));ack(&mut c);}
 let ids:Vec<_>=c.objects.iter().map(|o|o.id).collect();
 resolved(&mut c,&u,Action::Link{from:ids[0],to:ids[1]});resolved(&mut c,&u,Action::Link{from:ids[1],to:ids[2]});
 c.step(&input(c.last_tick.unwrap()+1,&u,Some(Action::Link{from:ids[2],to:ids[0]})));assert!(c.pending.is_none());
 let t=c.last_tick.unwrap()+301;c.step(&input(t,&u,None));resolved(&mut c,&u,Action::Load{node:ids[2],recipe:43,target:Some(1),point:u[1].pos()});
 assert_eq!(c.creative.payloads.len(),1);let cooldowns=c.cooldowns;let charges=c.charges.clone();
 resolved(&mut c,&u,Action::Activate(ids[0]));assert_eq!(c.cooldowns,cooldowns);assert_eq!(c.charges,charges);
 let t=c.last_tick.unwrap()+1;let hp=[(ids[1],0)];let mut i=input(t,&u,None);i.objects=&hp;c.step(&i);
 for t in t+1..t+90{let out=c.step(&input(t,&u,None));assert!(!out.iter().any(|c|matches!(c,Cmd::Damage{..})));}
 assert!(c.shots.is_empty());assert_eq!(c.creative.payloads.len(),1);
}
#[test]
fn creative_edit_preserves_health_expiry_and_does_not_move_explosives(){
 let u=[unit(0,0,(480000,480000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(2,(520000,510000),None));ack(&mut c);let id=c.objects[0].id;let end=c.objects[0].end;let hp=c.objects[0].hp;
 resolved(&mut c,&u,recipe(42,(540000,510000),Some(id as usize)));
 assert_eq!((c.objects[0].hp,c.objects[0].end),(hp,end));assert_eq!(c.objects[0].point,(540000,510000));
 c.cooldowns.fill(0);c.charges=std::array::from_fn(|_|Charge::Ready(0));resolved(&mut c,&u,recipe(3,(520000,480000),None));
 let t=c.last_tick.unwrap()+10;for t in c.last_tick.unwrap()+1..=t{c.step(&input(t,&u,None));}ack(&mut c);
 if let Some(o)=c.objects.iter().find(|o|o.recipe==3){let id=o.id;c.cooldowns.fill(0);c.step(&input(t+1,&u,Some(recipe(42,(540000,480000),Some(id as usize)))));assert!(c.pending.is_none());}
}
#[test]
fn creative_slime_has_escape_warning_and_never_hits_twice(){
 let mut u=[unit(0,0,(480000,480000)),unit(1,1,(525000,500000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(38,(520000,500000),None));ack(&mut c);let t=c.last_tick.unwrap();
 let mut warned=false;let mut pushes=0;for n in t+1..t+80{let out=c.step(&input(n,&u,None));warned|=out.iter().any(|v|matches!(v,Cmd::Warning{..}));pushes+=out.iter().filter(|v|matches!(v,Cmd::Push(1,..))).count();}
 assert!(warned);assert_eq!(pushes,1);u[1].x=800000;
 assert!(!c.step(&input(t+81,&u,None)).iter().any(|v|matches!(v,Cmd::Push(1,..))));
}
#[test]
fn creative_followup_uses_accepted_utility_hp_snapshot(){
 let mut u=[unit(0,0,(480000,480000)),unit(1,1,(500000,480000))];let mut c=Core::default();c.form=Form::Herobrine;u[0].max_hp=2000;
 resolved(&mut c,&u,recipe(12,u[1].pos(),None));let mut found=None;
 for t in c.last_tick.unwrap()+1..80{for cmd in c.step(&input(t,&u,None)){if let Cmd::Damage{event,target:1,..}=cmd{found=Some((event,t));}}if found.is_some(){break;}}
 let(event,t)=found.expect("crossbow damage");c.receipt(event,50,t);assert_eq!(c.followup_hp,2000);
 u[0].max_hp=10000;let mut i=input(t+1,&u,None);i.basics=&[(1,1)];let out=c.step(&i);assert!(out.iter().any(|v|matches!(v,Cmd::Damage{ad:60,..})));assert!(!out.iter().any(|v|matches!(v,Cmd::Damage{ad:220,..})));
}
#[test]
fn creative_enchants_another_owners_gift_but_expires_with_that_armour(){
 let mut u=[unit(0,0,(480000,480000)),unit(2,0,(500000,480000))];u[1].gifted_armour_until=100;
 let mut c=Core::default();c.form=Form::Herobrine;let out=resolved(&mut c,&u,recipe(31,u[1].pos(),Some(2)));let now=c.last_tick.unwrap();assert!(out.iter().any(|v|matches!(v,Cmd::Status{target:2,kind:9,ticks,..}if *ticks==100-now)));
 u[1].gifted_armour_until=now;let mut c=Core::default();c.form=Form::Herobrine;c.step(&input(now,&u,Some(recipe(31,u[1].pos(),Some(2)))));assert!(c.pending.is_none());assert_eq!(c.available(now),4);
}
#[test]
fn creative_aura_healing_stops_after_leaving_and_does_not_heal_its_caster(){
 let mut u=[unit(0,0,(480000,480000)),unit(2,0,(500000,480000))];let mut c=Core::default();c.form=Form::Herobrine;resolved(&mut c,&u,recipe(19,(520000,480000),None));ack(&mut c);let begin=c.last_tick.unwrap()+1;let mut healed=0;
 for t in begin..begin+140{let out=c.step(&input(t,&u,None));healed+=out.iter().filter(|v|matches!(v,Cmd::PotionTick{target:2,kind:12,..})).count();assert!(!out.iter().any(|v|matches!(v,Cmd::PotionTick{target:0,kind:12,..})));}assert!(healed>0);
 u[1].x=800000;for t in begin+140..begin+240{assert!(!c.step(&input(t,&u,None)).iter().any(|v|matches!(v,Cmd::PotionTick{kind:12,..})));}
}
#[test]
fn creative_redstone_paid_payload_releases_exactly_once(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(555000,480000))];let mut c=Core::default();c.form=Form::Herobrine;resolved(&mut c,&u,recipe(34,(520000,480000),None));ack(&mut c);let id=c.objects[0].id;
 resolved(&mut c,&u,Action::Load{node:id,recipe:44,target:Some(1),point:u[1].pos()});let paid=c.charges.clone();let clocks=c.cooldowns;
 resolved(&mut c,&u,Action::Activate(id));let mut hits=0;let begin=c.last_tick.unwrap()+1;for t in begin..begin+80{hits+=c.step(&input(t,&u,None)).iter().filter(|v|matches!(v,Cmd::Damage{target:1,..})).count();}assert_eq!(hits,1);assert!(c.creative.payloads.is_empty());assert_eq!(c.charges,paid);assert_eq!(c.cooldowns,clocks);
 resolved(&mut c,&u,Action::Activate(id));assert!(c.shots.is_empty());
}
#[test]
fn creative_rotation_keeps_health_lifetime_and_paid_ammunition(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(555000,480000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(34,(520000,480000),None));ack(&mut c);let id=c.objects[0].id;resolved(&mut c,&u,Action::Load{node:id,recipe:44,target:Some(1),point:u[1].pos()});
 let before=c.objects[0].clone();let payload=c.creative.payloads[0].clone();resolved(&mut c,&u,Action::Rotate{object:id,heading:4});
 let after=&c.objects[0];assert_eq!((after.hp,after.end,after.point),(before.hp,before.end,before.point));assert_eq!(after.direction,(0,10000));assert_eq!(c.creative.payloads[0].recipe,payload.recipe);assert_eq!(c.creative.payloads[0].point,payload.point);
}
#[test]
fn creative_same_owner_clouds_and_potions_never_stack_periodic_strength(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(525000,500000))];let mut c=Core::default();c.form=Form::Herobrine;
 resolved(&mut c,&u,recipe(26,(520000,500000),None));let mut duplicate=c.objects[0].clone();duplicate.id=999;duplicate.point=(530000,500000);c.objects.push(duplicate);
 let begin=c.last_tick.unwrap()+1;let mut ticks=0;for t in begin..begin+100{ticks+=c.step(&input(t,&u,None)).iter().filter(|v|matches!(v,Cmd::PotionTick{target:1,kind:4,..})).count();}assert!(ticks<=3,"Overlapping fields emitted {ticks} ticks");
 let mut c=Core::default();c.form=Form::Herobrine;resolved(&mut c,&u,recipe(26,(520000,500000),None));resolved(&mut c,&u,recipe(20,u[1].pos(),None));let start=c.last_tick.unwrap()+1;
 for t in start..start+170{let out=c.step(&input(t,&u,None));assert!(out.iter().filter(|v|matches!(v,Cmd::PotionTick{target:1,kind:4,..})).count()<=1,"Same-owner poison duplicated at {t}");}
}
fn input<'a>(t: usize, u: &'a [Unit], action: Option<Action>) -> Input<'a> {
    Input {
        tick: t,
        me: &u[0],
        units: u,
        unlocked: [true; 3],
        controlled: false,
        bound: false,
        automatic: false,
        action,
        basics: &[],
        objects: &[],
        wall: &|_, _| false,
        barriers: &[],
        home: (200000, 480000),
        haste: 0,
        ult_haste: 0,
    }
}
fn recipe(r: u8, p: Point, target: Option<usize>) -> Action {
    Action::Recipe {
        recipe: r,
        target,
        point: p,
    }
}
fn resolved(c: &mut Core, u: &[Unit], a: Action) -> Vec<Cmd> {
    let t = c.last_tick.map_or(0, |t| t + 1);
    let mut out = c.step(&input(t, u, Some(a)));
    if let Some(p) = &c.pending {
        out.extend(c.step(&input(p.end.max(t + 1), u, None)));
    }
    out
}
fn ack(c: &mut Core) {
    let ids: Vec<_> = c.objects.iter().map(|o| o.id).collect();
    for id in ids {
        c.allocated(id, true)
    }
}
#[test]
fn explosive_cover_and_non_recursive_pedestals() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    resolved(&mut c, &u, Action::Deploy((500000, 480000)));
    ack(&mut c);
    let id = c.objects[0].id;
    resolved(&mut c, &u, Action::Detonate(id));
    let t = c.objects[0].blast.unwrap();
    let mut i = input(t, &u, None);
    i.wall = &|x, _| (508000..=512000).contains(&x);
    let out = c.step(&i);
    assert!(!out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { target: 1, .. })));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { target: 0, .. })));
}
#[test]
fn tnt_and_ram_knockbacks_keep_twelve_tick_duration() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(3, u[1].pos(), None));
    let t = c.last_tick.unwrap();
    c.step(&input(t + 15, &u, None));
    ack(&mut c);
    let t = c.objects[0].blast.unwrap();
    let out = c.step(&input(t, &u, None));
    assert!(out.iter().any(|v| matches!(v, Cmd::Push(1, _, 12))));
}
#[test]
fn immune_and_hidden_hits_do_not_grant_followup() {
    let mut u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    u[1].visible = false;
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(&mut c, &u, recipe(5, u[1].pos(), None));
    let ids: Vec<_> = c.receipts.iter().map(|r| r.id).collect();
    for id in ids {
        c.receipt(id, 20, 22)
    }
    assert!(c.followup.is_none());
    u[1].visible = true;
    u[1].cc_immune = true;
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(&mut c, &u, recipe(2, u[1].pos(), None));
    c.step(&input(23, &u, None));
    assert!(c.followup.is_none());
}
#[test]
fn tnt_swing_lights_mine_without_counting_as_melee_followup() {
    let mut u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(3, (500000, 480000), None));
    let t = c.last_tick.unwrap() + 15;
    c.step(&input(t, &u, None));
    ack(&mut c);
    u[0].swing = true;
    c.step(&input(t + 1, &u, None));
    assert_eq!(c.objects[0].blast, Some(t + 13));
    assert_eq!(c.metrics.combos, 0);
}
#[test]
fn malformed_protocol_extremes_reject_without_panicking() {
    let mut a = vec![0i64; 22];
    a[0] = 1;
    a[1] = 1;
    a[7] = i64::MIN;
    assert!(crate::steve_craft_protocol::Frame::decode(&a).is_none());
    a[7] = 0;
    a[15] = 1025;
    assert!(crate::steve_craft_protocol::Frame::decode(&a).is_none());
}
#[test]
fn charge_bank_independent_and_reserved() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    assert_eq!(c.available(20000), 1);
    assert!(c.prepared.is_some());
    let at = c.last_tick.unwrap() + 1;
    c.step(&input(at, &u, Some(Action::Discard)));
    assert_eq!(c.available(at + 299), 1);
    assert_eq!(c.available(at + 300), 3);
}
#[test]
fn interrupted_prepare_never_releases_gadget() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Prepare(7))));
    let mut i = input(10, &u, None);
    i.controlled = true;
    c.step(&i);
    assert!(c.prepared.is_none());
    assert!(c.pending.is_none());
    assert_eq!(c.available(309), 1);
    assert_eq!(c.available(310), 3);
}
#[test]
fn invalid_actions_spend_nothing() {
    let u = [unit(0, 0, (480000, 480000))];
    for a in [
        Action::Deploy((480000, 480000)),
        Action::Prepare(3),
        recipe(99, (480000, 480000), None),
        recipe(0, (480000, 480000), None),
        recipe(9, (-1, 480000), None),
    ] {
        let mut c = Core::default();
        c.step(&input(0, &u, Some(a)));
        assert_eq!(c.available(0), 3);
        assert!(c.pending.is_none());
    }
}
#[test]
fn all_recipes_are_reachable_without_mastery() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    for r in 0..15 {
        let mut c = Core::default();
        let target = if r == 10 {
            Some(0)
        } else if r == 8 {
            Some(1)
        } else {
            None
        };
        if r == 6 || r == 7 {
            resolved(&mut c, &u, Action::Prepare(r));
            resolved(&mut c, &u, Action::Deploy((520000, 510000)));
        } else {
            resolved(&mut c, &u, recipe(r, (520000, 510000), target));
        }
        assert_eq!(c.metrics.casts[r as usize], 1, "recipe {r}: {}", c.reason);
    }
}
#[test]
fn failure_restores_resource_without_invisible_object() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(0, (530000, 480000), None));
    let id = c.objects[0].id;
    assert!(c.segments(30).is_empty());
    c.allocated(id, false);
    assert!(c.objects.is_empty());
    assert!(c.walls.is_empty());
    assert_eq!(c.cooldowns[0], 0);
    assert_eq!(c.available(30), 3);
}
#[test]
fn steve_object_hp_and_support_bonus() {
    let u = [unit(0, 0, (480000, 480000))];
    for (form, hp, heal, shield) in [
        (Form::Steve, 287, 182, 138),
        (Form::Herobrine, 230, 0, 0),
    ] {
        let mut c = Core::default();
        c.form = form;
        resolved(&mut c, &u, recipe(0, (530000, 480000), None));
        assert_eq!(c.objects[0].hp, hp);
        let out = resolved(&mut c, &u, recipe(10, u[0].pos(), Some(0)));
        if form==Form::Herobrine {assert!(!out.iter().any(|c|matches!(c,Cmd::Heal(..)|Cmd::Shield(..))));continue;}
        assert!(out.contains(&Cmd::Heal(0, heal)));
        assert!(out.contains(&Cmd::Shield(0, shield, 240)));
    }
}
#[test]
fn form_switch_is_not_ultimate_and_interrupt_keeps_form() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Stance)));
    assert_eq!(c.ult_at, 0);
    assert_eq!(c.stance_at, 1080);
    let mut i = input(12, &u, None);
    i.controlled = true;
    c.step(&i);
    assert_eq!(c.form, Form::Steve);
    assert_eq!(c.stance_at, 1080);
}
#[test]
fn unlocks_gate_stance_wall_and_recipes() {
    let u = [unit(0, 0, (480000, 480000))];
    for a in [
        Action::Stance,
        Action::Wall {
            centre: (650000, 480000),
            heading: 0,
            bend: 0,
        },
        recipe(3, (520000, 480000), None),
        recipe(12, (520000, 480000), None),
    ] {
        let mut c = Core::default();
        let mut i = input(0, &u, Some(a));
        i.unlocked = [false; 3];
        c.step(&i);
        assert!(c.pending.is_none());
        assert_eq!(c.available(0), 3);
        assert_eq!(c.ult_at, 0);
    }
}
#[test]
fn followup_requires_confirmed_champion_hit_not_requested_damage() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    let out = resolved(&mut c, &u, recipe(12, u[1].pos(), None));
    let _ = out;
    let t = c.last_tick.unwrap() + 10;
    let out = c.step(&input(t, &u, None));
    let event = out
        .iter()
        .find_map(|v| {
            if let Cmd::Damage { event, .. } = v {
                Some(*event)
            } else {
                None
            }
        })
        .unwrap();
    assert!(c.followup.is_none());
    c.receipt(event, 0, t);
    assert!(c.followup.is_none());
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.last_owner = 0;
    c.receipts.push(Receipt {power_hp:1150,
        id: 7,
        target: 1,
        champion: true,
        utility: true,
        at: 1,
    });
    c.receipt(7, 20, 1);
    assert_eq!(c.followup, Some((1, 241)));
    c.receipt(7, 20, 2);
    assert_eq!(c.followup, Some((1, 241)));
}
#[test]
fn followup_target_binding_expiry_and_single_basic_proc() {
    let u = [
        unit(0, 0, (480000, 480000)),
        unit(1, 1, (490000, 480000)),
        unit(2, 1, (500000, 480000)),
    ];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.followup_hp=1150;c.followup = Some((1, 240));
    let mut i = input(1, &u, None);
    i.basics = &[(1, 2)];
    assert!(c.step(&i).is_empty());
    let mut i = input(2, &u, None);
    i.basics = &[(2, 1), (2, 1)];
    let out = c.step(&i);
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { .. }))
            .count(),
        1
    );
    assert!(out.iter().any(|v| matches!(v, Cmd::Damage { ad: 43, .. })));
    assert_eq!(c.metrics.combos, 1);
    assert_eq!(c.followup_at, 362);
}
#[test]
fn farming_cannot_qualify_followup() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.last_owner = 0;
    for (champion, utility) in [(false, true), (true, false)] {
        c.receipts.push(Receipt {power_hp:1150,
            id: 1,
            target: 2,
            champion,
            utility,
            at: 0,
        });
        c.receipt(1, 100, 0);
        assert!(c.followup.is_none());
    }
    let _ = u;
}
#[test]
fn axe_bonus_is_consumed_only_once() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (490000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(13, u[0].pos(), None));
    let mut i = input(30, &u, None);
    i.basics = &[(1, 1), (1, 1)];
    let out = c.step(&i);
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { ad: 23, .. }))
            .count(),
        1
    );
}
#[test]
fn crystals_warn_on_destruction_anchors_defuse() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    for r in [6, 7] {
        let mut c = Core::default();
        resolved(&mut c, &u, Action::Prepare(r));
        resolved(&mut c, &u, Action::Deploy((500000, 480000)));
        ack(&mut c);
        let t = c.last_tick.unwrap() + 1;
        let id = c.objects[0].id;
        let hp = [(id, 0)];
        let mut i = input(t, &u, None);
        i.objects = &hp;
        let out = c.step(&i);
        if r == 6 {
            assert!(out.iter().any(|v| matches!(
                v,
                Cmd::Warning {
                    owner_hazard: true,
                    ..
                }
            )));
            let out = c.step(&input(t + 36, &u, None));
            assert!(out
                .iter()
                .any(|v| matches!(v, Cmd::Damage { target: 1, .. })));
        } else {
            assert!(c.objects.is_empty());
            assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
        }
    }
}
#[test]
fn heavy_self_damage_half_raw_and_blast_occlusion() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    resolved(&mut c, &u, Action::Deploy((500000, 480000)));
    ack(&mut c);
    let id = c.objects[0].id;
    resolved(&mut c, &u, Action::Detonate(id));
    let t = c.objects[0].blast.unwrap();
    let out = c.step(&input(t, &u, None));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 0,
            ap: 83,
            ..
        }
    )));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 1,
            ap: 167,
            ..
        }
    )));
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { target: 1, .. }))
            .count(),
        1
    );
}
#[test]
fn cleanup_never_explodes_and_cooldowns_survive() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(7));
    resolved(&mut c, &u, Action::Deploy((520000, 480000)));
    ack(&mut c);
    let cd = c.cooldowns;
    c.ult_at = 1000;
    c.stance_at = 700;
    let out = c.died(100);
    assert!(out.iter().all(|v| matches!(v, Cmd::Remove(_))));
    assert_eq!(c.cooldowns, cd);
    assert_eq!(c.ult_at, 1000);
    assert_eq!(c.form, Form::Steve);
}
#[test]
fn curves_open_bounded_and_shared_path() {
    for h in 0..16 {
        for b in -4i8..=4 {
            let p = arc((480000, 480000), h, b);
            assert!(p.len() <= 17);
            assert!(
                path_length(&p) <= 300000 && (path_length(&p) - 300000).abs() < 100,
                "h{h}b{b}: {}",
                path_length(&p)
            );
            if b != 0 {
                assert!(length(p[0], *p.last().unwrap()) >= 48000)
            }
            for i in 0..p.len() - 1 {
                for j in i + 2..p.len() - 1 {
                    assert!(!crosses(p[i], p[i + 1], p[j], p[j + 1]));
                }
            }
        }
    }
}
#[test]
fn steve_rejects_curve_without_spending_ultimate() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    c.step(&input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 4,
        }),
    ));
    assert_eq!(c.ult_at, 0);
    assert!(c.pending.is_none());
}
#[test]
fn path_warning_precedes_wall_and_initial_immunity_is_short() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    let out = c.step(&input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 0,
        }),
    ));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::PathWarning { until: 30, .. })));
    assert!(c.walls.is_empty());
    let out = c.step(&input(30, &u, None));
    assert_eq!(c.walls.len(), 1);
    assert!(!out.iter().any(|v| matches!(v,Cmd::Immune(t)if *t>2)));
}
#[test]
fn empty_boat_completes_and_parry_removes_whole_curve() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(
        &mut c,
        &u,
        Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 4,
        },
    );
    assert!(c.ride.is_some(), "{}", c.reason);
    let t = c.last_tick.unwrap() + 1;
    c.step(&input(t, &u, Some(Action::Dismount((310000, 400000)))));
    assert!(!c.ride.as_ref().unwrap().rider);
    assert!(c.walls.iter().any(|w| w.ultimate));
    c.parried();
    assert!(c.ride.is_none());
    assert!(c.walls.is_empty());
}
#[test]
fn union_routes_around_u_mouth_without_crossing_any_segment() {
    let p = arc((480000, 480000), 0, 4);
    let seg: Vec<_> = p.windows(2).map(|p| (p[0], p[1])).collect();
    let a = (480000, 520000);
    let b = (480000, 640000);
    let dest = detour(a, b, &seg, &|_, _| false);
    if let Some(d) = dest {
        assert_ne!(d, a);
        assert!(!blocked(a, d, &seg));
    }
}
#[test]
fn solid_terrain_shortens_or_rejects_without_cd() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    let mut i = input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 0,
        }),
    );
    i.wall = &|x, _| x >= 350000;
    c.step(&i);
    assert_eq!(c.ult_at, 0);
}
#[test]
fn capacity_limit_does_not_replace_existing_object() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(1, (520000, 480000), None));
    ack(&mut c);
    c.cooldowns[1] = 0;
    let ids: Vec<_> = c.objects.iter().map(|v| v.id).collect();
    resolved(&mut c, &u, recipe(1, (520000, 500000), None));
    assert_eq!(ids, c.objects.iter().map(|v| v.id).collect::<Vec<_>>());
}
#[test]
fn water_removes_only_owned_lava() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(5, (520000, 480000), None));
    resolved(&mut c, &u, recipe(4, (520000, 480000), None));
    assert!(!c.objects.iter().any(|o| o.recipe == 5));
    assert!(c.objects.iter().any(|o| o.recipe == 4));
}
#[test]
fn independent_owners_clones_and_duplicate_ticks() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut a = Core::default();
    let mut b = a.clone();
    a.step(&input(0, &u, Some(Action::Prepare(6))));
    assert_eq!(b.available(1), 3);
    assert_eq!(a.available(1), 1);
    let snap = format!("{:?}", a);
    assert!(a.step(&input(0, &u, None)).is_empty());
    assert_eq!(a.available(1), 1);
    assert!(!snap.is_empty());
    b.step(&input(0, &u, Some(recipe(11, u[0].pos(), None))));
    assert_eq!(a.cooldowns[11], 0);
}
#[test]
fn bounded_planner_preserves_input_and_visiblity() {
    let mut u = vec![unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    let mut c = Core::default();
    let before = format!("{:?}", c);
    let p = c.plan_lines(&input(0, &u, None));
    assert!(p.len() <= 8);
    assert!(p.iter().all(|v| v.actions.len() <= 3));
    assert!(c.metrics.branches <= 128);
    assert!(before.contains("charges"));
    u[1].visible = false;
    let mut c = Core::default();
    assert!(c.plan_lines(&input(0, &u, None)).is_empty());
}
#[test]
fn every_recipe_recharges_on_its_own_timer_and_rolls_back_its_own(){
 for r in 0..crate::steve_craft_data::RECIPES.len() as u8 {assert_eq!(cooldown_slot(r),r as usize);}
 // Two beacons of one family can be used back to back: each starts only its own cooldown.
 let (a,b)=(15u8,16u8);
 assert_eq!(crate::steve_craft_data::RECIPES[a as usize].group,crate::steve_craft_data::RECIPES[b as usize].group,"same family");
 let mut c=Core::default();c.cooldowns[cooldown_slot(a)]=500;
 assert_eq!(c.cooldowns[cooldown_slot(b)],0,"a used beacon leaves its sibling ready");
 // A survival cobweb whose host allocation fails restores exactly the cooldown it started.
 let u=[unit(0,0,(480000,480000)),unit(1,1,(700000,480000))];let mut c=Core::default();
 c.last_tick=Some(20);c.cooldowns[2]=7;c.cooldowns[3]=11;resolved(&mut c,&u,recipe(2,(520000,510000),None));
 let id=c.objects.iter().find(|o|o.recipe==2).expect("cobweb placed").id;assert!(c.cooldowns[2]>7);
 c.allocated(id,false);assert_eq!(c.cooldowns[2],7);assert_eq!(c.cooldowns[3],11,"other recipes untouched");
}
#[test]
fn planning_branches_leave_the_live_state_untouched(){
 let u=[unit(0,0,(480000,480000)),unit(1,1,(540000,480000)),unit(2,1,(560000,500000))];
 let mut c=Core::default();c.step(&input(0,&u,None));
 let snapshot=|c:&Core|format!("{:?}",(c.charges.clone(),c.cooldowns,c.objects.len(),c.receipts.len(),c.metrics.casts,c.trails.len()));
 let before=snapshot(&c);
 let a=c.plan_lines(&input(1,&u,None));let b=c.plan_lines(&input(1,&u,None));
 assert!(!a.is_empty(),"an enemy in reach gives the planner a line");
 let lines=|v:&[Continuation]|format!("{:?}",v.iter().map(|l|(&l.actions,l.score)).collect::<Vec<_>>());
 assert_eq!(lines(&a),lines(&b));
 assert_eq!(before,snapshot(&c));
}
