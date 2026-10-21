use universal_lab::{strategic_core::*,strategic_routes};
fn main(){let actors:Vec<_>=(0..10).map(|id|Actor{id,athlete:id,team:id/5,point:(200000+id as i64*45000,400000),hp:1000,max_hp:1000,speed:1000,power:100,range:40000,role:(id%5)as u8,controlled:false,ultimate:true,visible:true,ratings:Default::default(),judgement:60,ego:50}).collect();
let targets:Vec<_>=(0..32).map(|id|Target{id:id+100,kind:(id%6)as u8,team:2,point:(100000+id as i64%8*100000,100000+id as i64/8*180000),hp:10000,max_hp:20000,loss:0,visible:true,value:20}).collect();
let wall=|p:Point|p.0>320000&&p.0<352000&&p.1<700000;let mut runs=vec![];
for run in 0..5{let mut times=vec![];let mut cold=0;for n in 0..500{let t=std::time::Instant::now();let routes=strategic_routes::build(&actors,&targets,&wall);let view=View{routes:Some(&routes),team:run%2,actors:&actors,targets:&targets,home:(96000,864000),recent_advantage:0,unseen_risk:30,blocked:&|_,_|false};std::hint::black_box(evaluate(&view));let ns=t.elapsed().as_nanos();if n==0{cold=ns;}else{times.push(ns);}}
times.sort_unstable();runs.push(format!("{{\"run\":{run},\"coldNs\":{cold},\"medianNs\":{},\"p95Ns\":{},\"p99Ns\":{}}}",times[times.len()/2],times[times.len()*95/100],times[times.len()*99/100]));}
println!("{{\"actors\":10,\"targets\":32,\"evaluationsPerRun\":500,\"coreAndRoutingOnly\":true,\"runs\":[{}]}}",runs.join(","));}
