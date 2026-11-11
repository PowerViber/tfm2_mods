use crate::round_165_art as art;
#[test] fn wave_selection_is_bounded_and_uses_committed_geometry(){
    for dir in 0..32 {
        let a=dir as f64*std::f64::consts::TAU/32.;
        let mid=(400000+(45250.*a.cos()).round() as i64,400000+(45250.*a.sin()).round() as i64);
        let parts=art::wave_parts((400000,400000),mid);assert_eq!(parts.len(),1);
        assert_eq!((parts[0].1,parts[0].2),mid);assert!(parts[0].0.ends_with(&format!("_d{dir}")));
    }
    assert!(art::wave_parts((1,1),(1,1)).is_empty());
    for x in [5000,30000,82500,120000,500000]{let a=art::wave_parts((400000,400000),(400000+x,400000));assert!(!a.is_empty()&&a.len()<=16);assert_eq!(a,art::wave_parts((400000,400000),(400000+x,400000)));}
}
#[test] fn current_bomber_timings_select_registered_art_without_skipping_the_warning(){
    assert_eq!(art::wright_effect("wrt_warn1",60),Some("wrt_wright_warn1_t60"));
    assert_eq!(art::wright_effect("wrt_warn2",96),Some("wrt_wright_warn2_t96"));
    assert_eq!(art::wright_effect("wrt_warn3",132),Some("wrt_wright_warn3_t132"));
    assert!(art::wright_effect("wrt_carrier",1).is_none());
}
#[test] fn tsunami_art_has_no_damage_movement_or_control_side_effects(){
    use crate::glue_tests::{Host,Unit,with_sim,open_spot};use crate::thalassa::{Thalassa};use crate::thalassa_core::{Effect,Point};use mod_api_stable::{StablePassive,EntityKindV1};
    let(x,y)=open_spot(120000);let h=Host::new(vec![Unit{kind:EntityKindV1::Champion,team:0,pos:(x,y),hp:1000,name:"tfm2_custom_thalassa",targetable:true}]);h.borrow_mut().record_view_effects=true;h.borrow_mut().tick=1;
    let mut t=Thalassa::default();t.configure("{\"r165_art\":1}");
    with_sim(&h,|sim|t.apply(sim,0,vec![Effect::Fx{name:"tls_surge".into(),at:Point::new(x as i64+30000,y as i64),ticks:18,ambient:false}]));
    let g=h.borrow();assert!(g.view_effects.iter().any(|e|e.0.starts_with("tls_tsunami_")));assert!(g.damage.is_empty()&&g.moves.is_empty()&&g.ccs.iter().all(Vec::is_empty));
}
