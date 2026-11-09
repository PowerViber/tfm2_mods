use serde_json::{json,Value};
use wright_lab::{run,Session,REVISION};
fn input(owner:usize,tick:u64,order:Option<&str>)->String{
    json!({"version":1,"rules_revision":REVISION,"frame":{"tick":tick,"owner":owner,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":1000,"max_hp":1000,"ap":100,"stunned":false,"ult_unlocked":true,"units":[],"terrain":[],"profile":vec![50;13],"planes":[],"attack_target":null,"manual":true,"order":order.map(|kind|json!({"kind":kind,"aim":{"x":330000,"y":300000}}))}}).to_string()
}
#[test]
fn independent_owners_and_sessions_replay_without_sharing_stock_or_cooldowns(){
    let(mut a,mut b)=(Session::default(),Session::default());
    let a0=input(1,0,Some("fighters"));let b0=input(17,0,Some("repair"));
    let first:Value=serde_json::from_str(&run(&mut a,&a0)).unwrap();let other:Value=serde_json::from_str(&run(&mut b,&b0)).unwrap();
    assert_eq!(first["state"]["ready"][0],360);assert_eq!(first["state"]["ready"][2],0);
    assert_eq!(other["state"]["ready"][0],0);assert_eq!(other["state"]["ready"][2],840);
    let saved=serde_json::to_string(&b.core).unwrap();let a2=input(1,2,None);let next=run(&mut a,&a2);
    assert_eq!(saved,serde_json::to_string(&b.core).unwrap());
    let mut replay=Session::default();assert_eq!(serde_json::from_str::<Value>(&run(&mut replay,&a0)).unwrap(),first);
    assert_eq!(run(&mut replay,&a2),next);
}
