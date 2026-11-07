use std::io::{self,BufRead};
fn main(){
    if std::env::args().any(|a|a=="--bench") {bench();return;}
    let mut session=grimhald_lab::Session::default();for line in io::stdin().lock().lines(){match line{Ok(s)=>println!("{}",grimhald_lab::run(&mut session,&s)),Err(_)=>break}}
}
fn bench(){
    use grimhald_lab::{grimhald_core::*,grimhald_presentation};
    let mut runs=vec![];
    for run in 0..5 {
        let mut c=Core::default();let mut f=Frame{owner:1,alive:true,hp:1400,max_hp:1400,attack:100,move_speed:1000,pos:Point::new(300000,300000),manual:true,ult_unlocked:true,..Frame::default()};
        f.units=(0..128).map(|i|Unit{id:i+2,team:1,hp:10000,max_hp:10000,radius:8000,champion:true,visible:true,targetable:true,pos:Point::new(320000+(i%16) as i64*12000,300000+(i/16) as i64*12000),..Unit::default()}).collect();
        let mut times=vec![];
        for tick in 0..10000 {
            f.tick=tick;f.order=match tick%1000 {0=>Some(Order::Ult(Point::new(400000,300000))),150=>Some(Order::Bulwark(Point::new(340000,300000))),400=>Some(Order::Relocate(Point::new(350000,320000))),600=>Some(Order::Bash(Point::new(325000,300000))),_=>None};
            let start=std::time::Instant::now();let effects=c.step(&f);let visuals=grimhald_presentation::cues(&c,tick,true);times.push(start.elapsed().as_nanos());std::hint::black_box((&effects,&visuals));
        }
        times.sort();runs.push(serde_json::json!({"run":run+1,"medianUs":times[5000] as f64/1000.,"p95Us":times[9500] as f64/1000.,"p99Us":times[9900] as f64/1000.}));
    }
    println!("{}",serde_json::json!({"workload":"128 units; core step plus read-only presentation; no host API or rendering","runs":runs}));
}
